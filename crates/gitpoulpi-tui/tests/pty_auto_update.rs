//! The self-update (slice B of the updates-and-install design),
//! against a fake release server on loopback (`GITPOULPI_UPDATE_API`: test
//! builds refuse every other host) and a temp install folder.
//!
//! The binary under test is a hard link of the built `gitpoulpi-tui` placed
//! next to the `.gitpoulpi-dist` marker, so it counts as a release build and
//! may replace itself; the "new version" in the fake archive is a shell
//! script that prints a sentinel, which proves the restart really executed
//! the swapped-in file. The maintainer's real install is never touched.
//!
//! Unix only: the scripts need `/bin/sh`. The Windows swap is covered by the
//! engine's unit tests (`gitpoulpi-update`) against a simulated Windows file
//! system.
#![cfg(unix)]

mod common;

use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use gitpoulpi_config::release::{self, Marker, MARKER_FILE};
use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::dist::{checksum_line, marker_entry, sha256_hex, write_tar_gz, Entry};
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(40);
const NEW_VERSION: &str = "99.0.0";
const SENTINEL: &str = "STARTED-THE-SWAPPED-IN-FILE";

/// A fake `api.github.com` plus release-asset host.
type Files = Vec<(String, Vec<u8>)>;

struct FakeHost {
    port: u16,
    downloads: Arc<AtomicUsize>,
}

fn script(text: &str) -> Vec<u8> {
    format!("#!/bin/sh\necho {text}\nsleep 120\n").into_bytes()
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
    write_tar_gz(&release::package_stem(version, target), &entries).unwrap()
}

/// `tamper`: serve a checksum that does not match the archive.
fn serve(tamper: bool) -> FakeHost {
    let target = release::host_target();
    let name = release::archive_name(NEW_VERSION, &target);
    let bytes = archive(NEW_VERSION, &target);
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
        (format!("/dl/{name}"), bytes),
    ]));
    let json = serde_json::json!([{
        "tag_name": format!("v{NEW_VERSION}"), "draft": false, "prerelease": false,
        "body": "## What is new\n- the self-update",
        "html_url": "https://example.test/releases/v99.0.0",
        "assets": [
            {"name": name, "size": files.lock().unwrap()[1].1.len(),
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
    let exe = dir.path().join("gitpoulpi-tui");
    let built = env!("CARGO_BIN_EXE_gitpoulpi-tui");
    if std::fs::hard_link(built, &exe).is_err() {
        std::fs::copy(built, &exe).unwrap();
    }
    for sibling in ["gitpoulpi", "gitpoulpi-mcp"] {
        std::fs::write(dir.path().join(sibling), script("old sibling")).unwrap();
    }
    if marker {
        let marker = Marker::new(env!("CARGO_PKG_VERSION"), release::host_target());
        std::fs::write(dir.path().join(MARKER_FILE), marker.render()).unwrap();
    }
    (dir, exe)
}

fn launch(session: &str, host: &FakeHost, exe: &Path, mode: &str) -> Terminal {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("auto update probe")
        .unwrap()
        .build();
    let (term, config) = common::launch(repo.root())
        .session(session)
        .size(200, 50)
        .ready("auto update probe")
        .mouse_capture()
        .binary(exe)
        .env("GITPOULPI_UPDATES_MODE", mode)
        .env(
            "GITPOULPI_UPDATE_API",
            format!("http://127.0.0.1:{}", host.port),
        )
        .start();
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
fn update_now_downloads_stages_offers_the_restart_and_the_restart_runs_the_new_file() {
    let host = serve(false);
    let (dir, exe) = install_dir(true);
    let mut term = launch("pty-auto-update", &host, &exe, "notify");
    term.wait_for("the chip announces the new version", STEP, |s| {
        s.contains("v99.0.0 available")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    click_text(&mut term, "v99.0.0 available");
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
    // Both the checksum and the archive were fetched; the files are staged
    // beside the old ones, and the running binary is untouched.
    assert_eq!(host.downloads.load(Ordering::SeqCst), 2);
    assert!(dir.path().join("gitpoulpi-tui.new").exists());
    assert!(dir.path().join(format!("{MARKER_FILE}.new")).exists());
    assert_ne!(
        std::fs::read(dir.path().join("gitpoulpi-tui")).unwrap()[..2],
        *b"#!",
        "nothing is replaced before the user restarts"
    );
    // The user's action: the files are swapped and the new file is started.
    click_body_text(&mut term, "Restart to update");
    term.wait_for("the swapped-in file runs", STEP, |s| s.contains(SENTINEL))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let marker = std::fs::read_to_string(dir.path().join(MARKER_FILE)).unwrap();
    assert_eq!(Marker::parse(&marker).unwrap().version, NEW_VERSION);
    let old = std::fs::read_to_string(dir.path().join(format!("{MARKER_FILE}.old"))).unwrap();
    assert_eq!(
        Marker::parse(&old).unwrap().version,
        env!("CARGO_PKG_VERSION"),
        "the previous version is kept for a revert"
    );
    assert!(dir.path().join("gitpoulpi-tui.old").exists());
    term.stop().unwrap();
}

#[test]
fn a_checksum_mismatch_is_refused_and_nothing_is_staged() {
    let host = serve(true);
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
    assert!(!dir.path().join("gitpoulpi-tui.new").exists());
    assert!(!term.screen().contains("Restart to update"));
    term.stop().unwrap();
}

#[test]
fn a_build_without_the_marker_only_downloads_and_says_why() {
    let host = serve(false);
    // The plain cargo binary: no `.gitpoulpi-dist` next to it.
    let mut term = launch(
        "pty-auto-update-dev",
        &host,
        Path::new(env!("CARGO_BIN_EXE_gitpoulpi-tui")),
        "auto",
    );
    term.wait_for("the chip", STEP, |s| s.contains("v99.0.0 available"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    click_text(&mut term, "v99.0.0 available");
    term.wait_for("the tab explains and offers Download only", STEP, |s| {
        s.contains("cannot update itself") && s.contains("Download")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let screen = term.screen();
    let (_, row) = screen.find("Skip this version").expect("the buttons row");
    let buttons = screen.text().lines().nth(row as usize).unwrap().to_string();
    assert!(!buttons.contains("Update now"), "{buttons}");
    // Auto mode did not download for a build that cannot take it.
    assert_eq!(host.downloads.load(Ordering::SeqCst), 0);
    term.stop().unwrap();
}

#[test]
fn auto_mode_downloads_by_itself_and_waits_for_the_restart() {
    let host = serve(false);
    let (dir, exe) = install_dir(true);
    let term = launch("pty-auto-update-auto", &host, &exe, "auto");
    term.wait_for("the chip asks for the restart", STEP, |s| {
        s.contains("Restart to update")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(dir.path().join("gitpoulpi-tui.new").exists());
    // Never restarts without the user: still the TUI, not the sentinel.
    std::thread::sleep(Duration::from_secs(2));
    assert!(!term.screen().contains(SENTINEL), "{}", term.text());
    term.stop().unwrap();
}

#[test]
fn revert_swaps_the_previous_files_back_and_restarts_into_them() {
    let host = serve(false);
    let (dir, exe) = install_dir(true);
    // As after an update: the previous version's files wait as `.old`.
    let old = dir.path().join("gitpoulpi-tui.old");
    std::fs::write(&old, script(SENTINEL)).unwrap();
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&old, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    let previous = Marker::new("0.0.9", release::host_target());
    std::fs::write(
        dir.path().join(format!("{MARKER_FILE}.old")),
        previous.render(),
    )
    .unwrap();
    let mut term = launch("pty-auto-update-revert", &host, &exe, "notify");
    term.send_keys("C-p").unwrap();
    term.send_text("Revert to the previous").unwrap();
    term.wait_for("the palette lists Revert", STEP, |s| {
        s.contains("Updates: Revert to the Previous Version")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    term.wait_for("the previous file runs", STEP, |s| s.contains(SENTINEL))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let marker = std::fs::read_to_string(dir.path().join(MARKER_FILE)).unwrap();
    assert_eq!(Marker::parse(&marker).unwrap().version, "0.0.9");
    term.stop().unwrap();
}
