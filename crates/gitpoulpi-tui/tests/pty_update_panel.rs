//! The update panel the title bar's version chip opens (the
//! install-channels design, slice E), against a fake release server on
//! loopback (`GITPOULPI_UPDATE_API`; test builds refuse every other host).
//!
//! The binary under test is a copy of the built `gitpoulpi-tui` placed in a
//! temp install folder next to a marker that names an install channel, so
//! it counts as that kind of install and no production seam is needed. The
//! updater is never the real one: `GITPOULPI_UPDATER_EXE` (debug builds
//! only) points at a recording shell script.
//!
//! - `channel=apt`: the chip shows `available`; a click, `Ctrl+N u` and the
//!   palette entry open the panel; Copy puts the command on the clipboard
//!   and the status line; Esc and the close cross close it.
//! - `channel=archive`: `Update and restart` downloads the release, writes
//!   `job.json` and starts the updater in the same terminal.
//!
//! Unix only: the stub is a `/bin/sh` script. The Windows handoff is covered
//! by the job's unit tests.
//!
//! No input is sent after the action under test (the redraw rule).
#![cfg(unix)]

mod common;

use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::time::Duration;

use gitpoulpi_config::release::{self, InstallChannel, Marker, MARKER_FILE};
use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::dist::{checksum_line, marker_entry, sha256_hex, write_tar_gz, Entry};
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(40);
const NEW_VERSION: &str = "99.0.0";
const APT_COMMAND: &str = "sudo apt update && sudo apt install --only-upgrade gitpoulpi";

/// A fake `api.github.com` plus release-asset host. `assets`: whether the
/// release carries an archive for this platform.
fn serve(assets: bool) -> u16 {
    let target = release::host_target();
    let name = release::archive_name(NEW_VERSION, &target);
    let mut entries: Vec<Entry> = release::BINARIES
        .iter()
        .map(|b| Entry {
            name: release::exe_name(b, &target),
            data: b"#!/bin/sh\necho new\n".to_vec(),
            executable: true,
        })
        .collect();
    entries.push(marker_entry(NEW_VERSION, &target));
    let bytes = write_tar_gz(&release::package_stem(NEW_VERSION, &target), &entries).unwrap();
    let sum = checksum_line(&sha256_hex(&bytes), &name).into_bytes();
    let sig = common::release_signer()
        .sign(&name, NEW_VERSION, &bytes)
        .into_bytes();
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind loopback");
    let port = listener.local_addr().unwrap().port();
    let json = serde_json::json!([{
        "tag_name": format!("v{NEW_VERSION}"), "draft": false, "prerelease": false,
        "body": "## What is new\n- the update panel",
        "html_url": "https://example.test/releases/v99.0.0",
        "assets": if assets { serde_json::json!([
            {"name": name, "size": bytes.len(),
             "browser_download_url": format!("http://127.0.0.1:{port}/dl/{name}")},
            {"name": format!("{name}.sha256"), "size": 90,
             "browser_download_url": format!("http://127.0.0.1:{port}/dl/{name}.sha256")}
        ]) } else { serde_json::json!([]) }
    }])
    .to_string();
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { continue };
            let (json, bytes, sum, sig, name) = (
                json.clone(),
                bytes.clone(),
                sum.clone(),
                sig.clone(),
                name.clone(),
            );
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
                let body: Vec<u8> = if path == format!("/dl/{name}") {
                    bytes
                } else if path == format!("/dl/{name}.sha256") {
                    sum
                } else if path == format!("/dl/{name}.minisig") {
                    sig
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
    port
}

/// A temp install folder: a copy of the built TUI, siblings and a marker
/// naming `channel`.
fn install_dir(channel: InstallChannel) -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let exe = dir.path().join("gitpoulpi-tui");
    let built = env!("CARGO_BIN_EXE_gitpoulpi-tui");
    if std::fs::hard_link(built, &exe).is_err() {
        std::fs::copy(built, &exe).unwrap();
    }
    for sibling in ["gitpoulpi", "gitpoulpi-mcp"] {
        std::fs::write(dir.path().join(sibling), "#!/bin/sh\n").unwrap();
    }
    let mut marker = Marker::new(env!("CARGO_PKG_VERSION"), release::host_target());
    marker.channel = Some(channel);
    std::fs::write(dir.path().join(MARKER_FILE), marker.render()).unwrap();
    (dir, exe)
}

struct Run {
    term: Terminal,
    config: tempfile::TempDir,
    _repo: gitpoulpi_testkit::FixtureRepo,
}

fn launch(session: &str, port: u16, exe: &Path, envs: &[(&str, &Path)]) -> Run {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("update panel probe")
        .unwrap()
        .build();
    let mut launch = common::launch(repo.root())
        .session(session)
        .size(200, 50)
        .ready("update panel probe")
        .mouse_capture()
        .binary(exe)
        .env("GITPOULPI_UPDATES_MODE", "notify")
        .env("GITPOULPI_UPDATE_API", format!("http://127.0.0.1:{port}"))
        .env("GITPOULPI_CLIPBOARD", "memory")
        .env(
            common::RELEASE_KEY_ENV,
            common::release_signer().public_key_text(),
        );
    for (key, value) in envs {
        launch = launch.env(key, value);
    }
    let (term, config) = launch.start();
    Run {
        term,
        config,
        _repo: repo,
    }
}

/// Clicks `text` once it holds still: the title bar can still shift it.
fn click_text(term: &mut Terminal, text: &str) {
    let (col, row) = term
        .wait_for_settled_text(text, Duration::from_millis(300), STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.click(col + 1, row).unwrap();
}

fn panel_open(term: &Terminal) -> bool {
    term.screen().contains("Installed with:")
}

fn wait_panel(term: &Terminal) {
    term.wait_for("the update panel", STEP, |s| s.contains("Installed with:"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

fn wait_closed(term: &Terminal) {
    term.wait_for("the panel closed", STEP, |s| !s.contains("Installed with:"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

#[test]
fn an_apt_install_shows_its_command_and_the_panel_opens_four_ways_and_closes_two() {
    let port = serve(true);
    let (dir, exe) = install_dir(InstallChannel::Apt);
    // apt only offers its command where GitPoulpi's repository is configured.
    let sources = tempfile::tempdir().unwrap();
    std::fs::write(
        sources.path().join("gitpoulpi.list"),
        "deb http://example.test/ stable main\n",
    )
    .unwrap();
    let dump = tempfile::NamedTempFile::new().unwrap();
    let mut run = launch(
        "pty-update-panel-apt",
        port,
        &exe,
        &[
            ("GITPOULPI_APT_SOURCES_DIR", sources.path()),
            ("GITPOULPI_CLIPBOARD_DUMP", dump.path()),
        ],
    );
    let term = &mut run.term;
    term.wait_for("the chip says a newer version is available", STEP, |s| {
        s.contains("v99.0.0 available")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // 1. A click on the chip.
    click_text(term, "v99.0.0 available");
    let needles = [
        "Installed with: apt",
        "Version 99.0.0 is available",
        APT_COMMAND,
        "Copy command",
        "Open in terminal",
    ];
    // One predicate for the whole panel (a torn frame can lack lines).
    let panel = term
        .wait_for("the whole apt panel", STEP, |s| {
            needles.iter().all(|n| s.contains(n))
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    for needle in needles {
        assert!(panel.contains(needle), "{needle}\n{}", term.text());
    }
    assert!(
        !term.screen().contains("Update and restart"),
        "{}",
        term.text()
    );

    // Copy: the clipboard and the status line carry the command.
    click_text(term, "Copy command");
    term.wait_for("the status line reports the copy", STEP, |s| {
        s.contains("copied the update command")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert_eq!(std::fs::read_to_string(dump.path()).unwrap(), APT_COMMAND);

    // Esc closes it; nothing is left of the window.
    term.send_keys("Esc").unwrap();
    wait_closed(term);

    // 2. The keyboard twin of the chip: Ctrl+N, u.
    term.send_keys("C-n").unwrap();
    term.send_keys("u").unwrap();
    wait_panel(term);

    // The close cross closes it too.
    let (col, row) = term
        .wait_for_text("\u{2715}", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(col + 1, row).unwrap();
    wait_closed(term);

    // 3. The palette entry.
    term.send_keys("C-p").unwrap();
    term.send_text("Show Update").unwrap();
    term.wait_for("the palette lists Updates: Show Update", STEP, |s| {
        s.contains("Updates: Show Update")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    wait_panel(term);
    term.send_keys("Esc").unwrap();
    wait_closed(term);

    // 4. The palette's copy command works with the panel closed.
    term.send_keys("C-p").unwrap();
    term.send_text("Copy Update Command").unwrap();
    term.wait_for("the palette lists the copy command", STEP, |s| {
        s.contains("Updates: Copy Update Command")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    std::fs::write(dump.path(), "").unwrap();
    term.send_keys("Enter").unwrap();
    term.wait_for("the copy happened", STEP, |s| {
        s.contains("copied the update command")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert_eq!(std::fs::read_to_string(dump.path()).unwrap(), APT_COMMAND);
    assert!(!panel_open(term));
    // The install folder is untouched: the app never runs a Show command.
    assert!(!dir.path().join("gitpoulpi-tui.new").exists());
    run.term.stop().unwrap();
}

#[test]
fn update_and_restart_writes_the_job_and_starts_the_updater() {
    let port = serve(true);
    let (dir, exe) = install_dir(InstallChannel::Archive);
    // The updater stub: records the job it was started with, then waits.
    let record = tempfile::tempdir().unwrap();
    let stub = record.path().join("gitpoulpi-updater-stub");
    std::fs::write(
        &stub,
        format!(
            "#!/bin/sh\ncp \"$3\" '{rec}/job.json'\necho \"$@\" > '{rec}/args.txt'\n\
             echo UPDATER-STUB-RAN\nsleep 120\n",
            rec = record.path().display()
        ),
    )
    .unwrap();
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&stub, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    let mut run = launch(
        "pty-update-panel-archive",
        port,
        &exe,
        &[("GITPOULPI_UPDATER_EXE", &stub)],
    );
    let term = &mut run.term;
    term.wait_for("the chip", STEP, |s| s.contains("v99.0.0 available"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    click_text(term, "v99.0.0 available");
    term.wait_for("the archive panel", STEP, |s| {
        s.contains("Installed with: a release archive") && s.contains("Update and restart")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(!term.screen().contains("Copy command"), "{}", term.text());
    click_text(term, "Update and restart");
    term.wait_for("the updater runs in this terminal", STEP, |s| {
        s.contains("UPDATER-STUB-RAN")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // The download was staged beside the old files; the swap is the updater's.
    assert!(dir.path().join("gitpoulpi-tui.new").exists());
    assert_ne!(
        std::fs::read(dir.path().join("gitpoulpi-tui")).unwrap()[..2],
        *b"#!",
        "the TUI never replaces its own files"
    );
    let args = std::fs::read_to_string(record.path().join("args.txt")).unwrap();
    assert!(args.starts_with("run --job "), "{args}");
    let text = std::fs::read_to_string(record.path().join("job.json")).unwrap();
    let job: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(job["schema"], 1);
    assert_eq!(job["action"], "apply");
    assert_eq!(job["from_version"], env!("CARGO_PKG_VERSION"));
    assert_eq!(job["to_version"], NEW_VERSION);
    assert_eq!(job["staged"], "in_place");
    assert_eq!(job["elevation"], "none");
    assert_eq!(
        Path::new(job["install_dir"].as_str().unwrap())
            .canonicalize()
            .unwrap(),
        dir.path().canonicalize().unwrap()
    );
    assert_eq!(
        Path::new(job["relaunch"]["exe"].as_str().unwrap())
            .file_name()
            .unwrap(),
        "gitpoulpi-tui"
    );
    assert_eq!(
        Path::new(job["config_dir"].as_str().unwrap()),
        run.config.path()
    );
    assert!(job["requester_pid"].as_u64().unwrap() > 0);
    let status = job["status_path"].as_str().unwrap();
    assert!(status.ends_with("updates/status.json"), "{status}");
    // The job and the updater copy live in the data folder, not the install folder.
    let updates = run.config.path().join("data").join("updates");
    assert!(updates.join("job.json").is_file());
    assert!(
        updates.join("bin").join("gitpoulpi-updater").is_file() || {
            // The override names the file; its copy keeps the updater's name.
            updates.join("bin").read_dir().unwrap().count() > 0
        }
    );
    run.term.stop().unwrap();
}
