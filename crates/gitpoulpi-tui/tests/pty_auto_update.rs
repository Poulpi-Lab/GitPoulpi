//! The self-update's download and staging, and the handoff to the updater
//! (the install-channels design), against a fake release server on loopback
//! (`GITPOULPI_UPDATE_API`: test builds refuse every other host) and a temp
//! install folder.
//!
//! The binary under test is a hard link of the built `gitpoulpi-tui` placed
//! next to the `.gitpoulpi-dist` marker, so it counts as a release build and
//! may update itself. The TUI no longer swaps files: "Restart to update" and
//! "Revert" write a job and start `gitpoulpi-updater`, which here is a
//! recording shell script (`GITPOULPI_UPDATER_EXE`, debug builds only); the
//! tests assert on the job it was given and that nothing in the install
//! folder was replaced. The maintainer's real install is never touched.
//!
//! The stand-in updater is a `/bin/sh` script on unix and a small `.exe` on
//! Windows (`common::updater_stub`). The swap itself is the updater's (its
//! own tests) and the engine's (`gitpoulpi-update`, against a simulated
//! Windows file system).

mod common;

use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use gitpoulpi_config::release::{self, Marker, MARKER_FILE};
use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::dist::{checksum_line, marker_entry, sha256_hex, Entry};
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(40);
const NEW_VERSION: &str = "99.0.0";
const SENTINEL: &str = "UPDATER-STUB-RAN";

/// A fake `api.github.com` plus release-asset host.
type Files = Vec<(String, Vec<u8>)>;

struct FakeHost {
    port: u16,
    downloads: Arc<AtomicUsize>,
}

fn script(text: &str) -> Vec<u8> {
    format!("#!/bin/sh\necho {text}\nsleep 120\n").into_bytes()
}

/// A recording stand-in for `gitpoulpi-updater` (see `common::updater_stub`).
fn updater_stub(dir: &Path) -> PathBuf {
    common::updater_stub(dir, dir, SENTINEL)
}

fn recorded_job(dir: &Path) -> serde_json::Value {
    let text = std::fs::read_to_string(dir.join("job.json")).expect("the updater got a job");
    serde_json::from_str(&text).unwrap()
}

fn archive(version: &str, target: &str) -> Vec<u8> {
    let mut entries: Vec<Entry> = release::BINARIES
        .iter()
        .map(|b| Entry {
            name: release::exe_name(b, target),
            data: script(SENTINEL),
            executable: true,
        })
        .collect();
    entries.push(marker_entry(version, target));
    common::release_archive(version, target, &entries)
}

/// What the fake release carries wrong, if anything.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Fault {
    None,
    /// A checksum that does not match the archive.
    Checksum,
    /// No `.minisig` for the archive at all.
    Unsigned,
    /// A signature made by a key the app does not trust.
    WrongKey,
}

fn serve(fault: Fault) -> FakeHost {
    let tamper = fault == Fault::Checksum;
    let target = release::host_target();
    let name = release::archive_name(NEW_VERSION, &target);
    let bytes = archive(NEW_VERSION, &target);
    let signature = match fault {
        Fault::Unsigned => Vec::new(),
        Fault::WrongKey => gitpoulpi_testkit::signing::TestSigner::new()
            .sign(&name, NEW_VERSION, &bytes)
            .into_bytes(),
        _ => common::release_signer()
            .sign(&name, NEW_VERSION, &bytes)
            .into_bytes(),
    };
    let sum_hash = if tamper {
        sha256_hex(b"not the archive")
    } else {
        sha256_hex(&bytes)
    };
    let sum = checksum_line(&sum_hash, &name).into_bytes();
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind loopback");
    let port = listener.local_addr().unwrap().port();
    let downloads = Arc::new(AtomicUsize::new(0));
    let counter = downloads.clone();
    let files: Arc<Mutex<Files>> = Arc::new(Mutex::new(vec![
        (format!("/dl/{name}.sha256"), sum),
        (format!("/dl/{name}.minisig"), signature),
        (format!("/dl/{name}"), bytes),
    ]));
    let json = serde_json::json!([{
        "tag_name": format!("v{NEW_VERSION}"), "draft": false, "prerelease": false,
        "body": "## What is new\n- the self-update",
        "html_url": "https://example.test/releases/v99.0.0",
        "assets": [
            {"name": name, "size": files.lock().unwrap()[2].1.len(),
             "browser_download_url": format!("http://127.0.0.1:{port}/dl/{name}")},
            {"name": format!("{name}.sha256"), "size": 90,
             "browser_download_url": format!("http://127.0.0.1:{port}/dl/{name}.sha256")}
        ]
    }])
    .to_string();
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { continue };
            let (files, json, counter) = (files.clone(), json.clone(), counter.clone());
            std::thread::spawn(move || {
                let mut buf = [0u8; 4096];
                let n = stream.read(&mut buf).unwrap_or(0);
                let request = String::from_utf8_lossy(&buf[..n]).into_owned();
                let path = request
                    .lines()
                    .next()
                    .and_then(|l| l.split_whitespace().nth(1))
                    .unwrap_or("")
                    .to_string();
                let body: Vec<u8> = if path.starts_with("/dl/") {
                    counter.fetch_add(1, Ordering::SeqCst);
                    files
                        .lock()
                        .unwrap()
                        .iter()
                        .find(|(p, _)| *p == path)
                        .map(|(_, b)| b.clone())
                        .unwrap_or_default()
                } else {
                    json.into_bytes()
                };
                let head = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\n\
                     Content-Length: {}\r\nConnection: close\r\n\r\n",
                    body.len()
                );
                let _ = stream.write_all(head.as_bytes());
                let _ = stream.write_all(&body);
            });
        }
    });
    FakeHost { port, downloads }
}

/// A temp install folder: a hard link of the built TUI, placeholder siblings
/// and the marker of version 0.1.0 (this build's).
fn install_dir(marker: bool) -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let exe = dir.path().join(common::exe("gitpoulpi-tui"));
    let built = env!("CARGO_BIN_EXE_gitpoulpi-tui");
    if std::fs::hard_link(built, &exe).is_err() {
        std::fs::copy(built, &exe).unwrap();
    }
    for sibling in ["gitpoulpi", "gitpoulpi-mcp"] {
        std::fs::write(dir.path().join(common::exe(sibling)), script("old sibling")).unwrap();
    }
    if marker {
        let marker = Marker::new(env!("CARGO_PKG_VERSION"), release::host_target());
        std::fs::write(dir.path().join(MARKER_FILE), marker.render()).unwrap();
    }
    (dir, exe)
}

fn launch(session: &str, host: &FakeHost, exe: &Path, mode: &str) -> Terminal {
    launch_with(session, host, exe, mode, None)
}

fn launch_with(
    session: &str,
    host: &FakeHost,
    exe: &Path,
    mode: &str,
    updater: Option<&Path>,
) -> Terminal {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("auto update probe")
        .unwrap()
        .build();
    let mut launch = common::launch(repo.root())
        .session(session)
        .size(200, 50)
        .ready("auto update probe")
        .mouse_capture()
        .binary(exe)
        .env("GITPOULPI_UPDATES_MODE", mode)
        .env(
            common::RELEASE_KEY_ENV,
            common::release_signer().public_key_text(),
        )
        .env(
            "GITPOULPI_UPDATE_API",
            format!("http://127.0.0.1:{}", host.port),
        );
    if let Some(updater) = updater {
        launch = launch.env("GITPOULPI_UPDATER_EXE", updater);
    }
    let (term, config) = launch.start();
    std::mem::forget(repo);
    std::mem::forget(config);
    term
}

/// Clicks `text` once it holds still: the status bar can still shift it (an
/// "Activity (n)" count appearing to its left), and a position read from an
/// earlier frame then lands beside the chip.
fn click_text(term: &mut Terminal, text: &str) {
    let (col, row) = term
        .wait_for_settled_text(text, Duration::from_millis(300), STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.click(col + 1, row).unwrap();
}

/// Clicks the `text` that is on the tab's body (below the title and the tab
/// bar): once the update is staged the title bar's chip reads the same
/// "Restart to update", and a click on it only opens the tab it is already on.
fn click_body_text(term: &mut Terminal, text: &str) {
    term.wait_for_settled_text(text, Duration::from_millis(300), STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (col, row) = term
        .screen()
        .find_all(text)
        .into_iter()
        .find(|(_, r)| *r >= 2)
        .unwrap_or_else(|| panic!("no {text} on the body\n{}", term.text()));
    term.click(col + 1, row).expect("write the click");
}

#[test]
fn update_now_downloads_stages_offers_the_restart_and_the_restart_hands_the_job_to_the_updater() {
    let host = serve(Fault::None);
    let (dir, exe) = install_dir(true);
    let record = tempfile::tempdir().unwrap();
    let stub = updater_stub(record.path());
    let mut term = launch_with("pty-auto-update", &host, &exe, "notify", Some(&stub));
    term.wait_for("the chip announces the new version", STEP, |s| {
        s.contains("v99.0.0 available")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // The chip opens the update panel; its Release notes button the tab.
    click_text(&mut term, "v99.0.0 available");
    term.wait_for("the panel offers the update", STEP, |s| {
        s.contains("Update and restart") && s.contains("Release notes")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    click_text(&mut term, "Release notes");
    term.wait_for("the tab offers Update now", STEP, |s| {
        s.contains("Update now") && s.contains("Skip this version")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(
        !term.screen().contains("cannot update itself"),
        "a release build says nothing about being unable"
    );
    click_text(&mut term, "Update now");
    term.wait_for(
        "the download is staged and the restart offered",
        STEP,
        |s| s.contains("Version 99.0.0 is ready: restart to update"),
    )
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(
        term.screen().contains("Restart to update"),
        "{}",
        term.text()
    );
    // The checksum, the signature and the archive were fetched; the files are
    // staged beside the old ones, and the running binary is untouched.
    assert_eq!(host.downloads.load(Ordering::SeqCst), 3);
    assert!(dir
        .path()
        .join(format!("{}.new", common::exe("gitpoulpi-tui")))
        .exists());
    assert!(dir.path().join(format!("{MARKER_FILE}.new")).exists());
    assert_ne!(
        std::fs::read(&exe).unwrap()[..2],
        *b"#!",
        "nothing is replaced before the user restarts"
    );
    // The user's action: a job is written and the updater is started.
    click_body_text(&mut term, "Restart to update");
    common::wait_updater_ran(&term, record.path(), SENTINEL);
    let job = recorded_job(record.path());
    assert_eq!(job["action"], "apply");
    assert_eq!(job["to_version"], NEW_VERSION);
    assert_eq!(job["from_version"], env!("CARGO_PKG_VERSION"));
    assert_eq!(job["staged"], "in_place");
    // The TUI did not swap anything itself.
    assert_ne!(std::fs::read(&exe).unwrap()[..2], *b"#!");
    assert!(!dir
        .path()
        .join(format!("{}.old", common::exe("gitpoulpi-tui")))
        .exists());
    term.stop().unwrap();
}

#[test]
fn a_checksum_mismatch_is_refused_and_nothing_is_staged() {
    let host = serve(Fault::Checksum);
    let (dir, exe) = install_dir(true);
    let mut term = launch("pty-auto-update-bad", &host, &exe, "notify");
    term.wait_for("the chip", STEP, |s| s.contains("v99.0.0 available"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // The palette twin of the button.
    term.send_keys("C-p").unwrap();
    term.send_text("Update now").unwrap();
    term.wait_for("the palette lists the command", STEP, |s| {
        s.contains("Updates: Update Now")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    term.wait_for("the checksum refusal is shown", STEP, |s| {
        s.contains("checksum does not match")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(!dir
        .path()
        .join(format!("{}.new", common::exe("gitpoulpi-tui")))
        .exists());
    assert!(!term.screen().contains("Restart to update"));
    term.stop().unwrap();
}

#[test]
fn a_release_without_a_valid_signature_is_refused_and_nothing_is_staged() {
    for (fault, session) in [
        (Fault::Unsigned, "pty-auto-update-unsigned"),
        (Fault::WrongKey, "pty-auto-update-wrong-key"),
    ] {
        let host = serve(fault);
        let (dir, exe) = install_dir(true);
        let mut term = launch(session, &host, &exe, "notify");
        term.wait_for("the chip", STEP, |s| s.contains("v99.0.0 available"))
            .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
        term.send_keys("C-p").unwrap();
        term.send_text("Update now").unwrap();
        term.wait_for("the palette lists the command", STEP, |s| {
            s.contains("Updates: Update Now")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
        term.send_keys("Enter").unwrap();
        term.wait_for("the signature refusal is shown", STEP, |s| {
            s.contains("signature")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
        assert!(!dir
            .path()
            .join(format!("{}.new", common::exe("gitpoulpi-tui")))
            .exists());
        assert!(!term.screen().contains("Restart to update"));
        term.stop().unwrap();
    }
}

#[test]
fn a_build_without_the_marker_never_asks_never_downloads_and_shows_only_its_version() {
    let host = serve(Fault::None);
    // The plain cargo binary: no `.gitpoulpi-dist` next to it.
    let term = launch(
        "pty-auto-update-dev",
        &host,
        Path::new(env!("CARGO_BIN_EXE_gitpoulpi-tui")),
        "auto",
    );
    // The background check would have fired two seconds after the start.
    std::thread::sleep(Duration::from_secs(6));
    let screen = term.screen();
    assert!(!screen.contains("available"), "{}", term.text());
    assert!(
        screen.contains(concat!("v", env!("CARGO_PKG_VERSION"))),
        "the running version is on the chip\n{}",
        term.text()
    );
    // Auto mode downloaded nothing for a build that cannot take it.
    assert_eq!(host.downloads.load(Ordering::SeqCst), 0);
    term.stop().unwrap();
}

#[test]
fn auto_mode_downloads_by_itself_and_waits_for_the_restart() {
    let host = serve(Fault::None);
    let (dir, exe) = install_dir(true);
    let term = launch("pty-auto-update-auto", &host, &exe, "auto");
    term.wait_for("the chip asks for the restart", STEP, |s| {
        s.contains("Restart to update")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(dir
        .path()
        .join(format!("{}.new", common::exe("gitpoulpi-tui")))
        .exists());
    // Never restarts without the user: still the TUI, not the sentinel.
    std::thread::sleep(Duration::from_secs(2));
    assert!(!term.screen().contains(SENTINEL), "{}", term.text());
    term.stop().unwrap();
}

#[test]
fn revert_hands_a_revert_job_to_the_updater_and_swaps_nothing_itself() {
    let host = serve(Fault::None);
    let (dir, exe) = install_dir(true);
    // As after an update: the previous version's files wait as `.old`.
    let old = dir
        .path()
        .join(format!("{}.old", common::exe("gitpoulpi-tui")));
    common::write_sh_script(&old, &String::from_utf8(script("previous")).unwrap());
    let previous = Marker::new("0.0.9", release::host_target());
    std::fs::write(
        dir.path().join(format!("{MARKER_FILE}.old")),
        previous.render(),
    )
    .unwrap();
    let record = tempfile::tempdir().unwrap();
    let stub = updater_stub(record.path());
    let mut term = launch_with("pty-auto-update-revert", &host, &exe, "notify", Some(&stub));
    term.send_keys("C-p").unwrap();
    term.send_text("Revert to the previous").unwrap();
    term.wait_for("the palette lists Revert", STEP, |s| {
        s.contains("Updates: Revert to the Previous Version")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    common::wait_updater_ran(&term, record.path(), SENTINEL);
    let job = recorded_job(record.path());
    assert_eq!(job["action"], "revert");
    assert_eq!(job["to_version"], "0.0.9");
    // Nothing was swapped by the TUI: the previous files still wait.
    assert!(old.exists());
    let marker = std::fs::read_to_string(dir.path().join(MARKER_FILE)).unwrap();
    assert_eq!(
        Marker::parse(&marker).unwrap().version,
        env!("CARGO_PKG_VERSION")
    );
    term.stop().unwrap();
}
