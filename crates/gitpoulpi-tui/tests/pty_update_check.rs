//! The update check, the title bar's version chip and the Release notes tab,
//! against a fake release server on loopback (`GITPOULPI_UPDATE_API`; test
//! builds refuse every other host). The real binary, in a pty.
//!
//! A newer release makes the chip read `v99.0.0 available`; a click on it opens
//! the Release notes tab with that release's body on top; Download opens the
//! release page (through the test stub of the browser), Skip this version puts
//! the chip back and stays skipped. `updates.mode = "off"` makes no request.

mod common;

use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::{Screen, Terminal};

const STEP: Duration = Duration::from_secs(40);

/// A fake `api.github.com`: answers the releases list, counts and records
/// the requests it got.
struct FakeReleases {
    port: u16,
    hits: Arc<AtomicUsize>,
    paths: Arc<Mutex<Vec<String>>>,
}

const BODY: &str = r###"[
  {"tag_name": "v99.0.0", "name": "99.0.0", "draft": false, "prerelease": false,
   "body": "## What is new\n- the **shiny** fake feature",
   "html_url": "https://example.test/releases/v99.0.0", "assets": []},
  {"tag_name": "v0.0.1", "draft": false, "prerelease": false, "body": "old",
   "html_url": "https://example.test/releases/v0.0.1", "assets": []}
]"###;

fn serve() -> FakeReleases {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind loopback");
    let port = listener.local_addr().unwrap().port();
    let hits = Arc::new(AtomicUsize::new(0));
    let paths = Arc::new(Mutex::new(Vec::new()));
    let (h, p) = (hits.clone(), paths.clone());
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { continue };
            let mut buf = [0u8; 4096];
            let n = stream.read(&mut buf).unwrap_or(0);
            let request = String::from_utf8_lossy(&buf[..n]).into_owned();
            let path = request
                .lines()
                .next()
                .and_then(|l| l.split_whitespace().nth(1))
                .unwrap_or("")
                .to_string();
            h.fetch_add(1, Ordering::SeqCst);
            p.lock().unwrap().push(path);
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{BODY}",
                BODY.len()
            );
            let _ = stream.write_all(response.as_bytes());
        }
    });
    FakeReleases { port, hits, paths }
}

fn launch(session: &str, server: &FakeReleases, mode: &str, log: &std::path::Path) -> Terminal {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("update check probe")
        .unwrap()
        .build();
    let (term, config) = common::launch(repo.root())
        .session(session)
        .size(200, 50)
        .ready("update check probe")
        .mouse_capture()
        .env("GITPOULPI_UPDATES_MODE", mode)
        .env(
            "GITPOULPI_UPDATE_API",
            format!("http://127.0.0.1:{}", server.port),
        )
        .env("GITPOULPI_OPEN_LOG", log)
        .start();
    // The fixture and the config dir must outlive the app.
    std::mem::forget(repo);
    std::mem::forget(config);
    term
}

/// The chip's cell: the text between the app name and the path in the title bar.
fn chip(s: &Screen) -> Option<(u16, u16)> {
    s.find("v99.0.0 available")
}

#[test]
fn a_newer_release_turns_the_chip_into_available_and_a_click_opens_its_notes() {
    let server = serve();
    let log = tempfile::NamedTempFile::new().unwrap();
    let mut term = launch("pty-update-available", &server, "notify", log.path());
    term.wait_for("the chip says a newer version is available", STEP, |s| {
        s.contains("v99.0.0 available")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert_eq!(server.hits.load(Ordering::SeqCst), 1, "one request");
    let path = server.paths.lock().unwrap()[0].clone();
    assert!(
        path.starts_with("/repos/Poulpi-Lab/GitPoulpi/releases"),
        "the releases list of the configured repository: {path}"
    );
    // The chip sits in the title bar: `GitPoulpi`, the chip, then `-- path`.
    let s = term.screen();
    let (col, row) = chip(&s).unwrap();
    assert_eq!(row, 0, "on the title row\n{}", s.text());
    let name = s.find("GitPoulpi").expect("the app name");
    let dashes = s.find("--").expect("the path separator");
    assert_eq!(name.1, 0);
    assert!(name.0 < col && col < dashes.0, "{name:?} {col} {dashes:?}");
    assert!(
        s.find("Activity").is_some_and(|a| a.1 != 0),
        "the status line no longer carries it"
    );
    // A click opens the tab: the release body on top, the buttons, the tab.
    term.click(col + 1, row).unwrap();
    term.wait_for("the Release notes tab with the release body", STEP, |s| {
        s.contains("the shiny fake feature")
            && s.contains("Version 99.0.0 is available")
            && s.contains("Download")
            && s.contains("Skip this version")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(
        term.screen()
            .find_all("Release notes")
            .iter()
            .any(|(_, r)| *r <= 2),
        "a Release notes tab in the tab bar\n{}",
        term.text()
    );
    // Download: the release page, through the browser stub.
    let (dc, dr) = term.screen().find("Download").unwrap();
    term.click(dc + 1, dr).unwrap();
    term.wait_for("the release page was opened", STEP, |_| {
        std::fs::read_to_string(log.path())
            .map(|t| t.contains("https://example.test/releases/v99.0.0"))
            .unwrap_or(false)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // Skip this version: the notice goes, and the chip is the plain version.
    let (sc, sr) = term.screen().find("Skip this version").unwrap();
    term.click(sc + 1, sr).unwrap();
    term.wait_for("the skipped notice", STEP, |s| {
        s.contains("(skipped)") && !s.contains("v99.0.0 available")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // Esc closes the tab.
    term.send_keys("Esc").unwrap();
    term.wait_for("back on the Home list", STEP, |s| !s.contains("(skipped)"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

#[test]
fn the_notes_tab_opens_from_the_palette_and_checks_on_demand() {
    let server = serve();
    let log = tempfile::NamedTempFile::new().unwrap();
    // Off: the chip stays the plain version and nothing is asked.
    let mut term = launch("pty-update-palette", &server, "off", log.path());
    term.send_keys("C-p").unwrap();
    term.send_text("Release notes").unwrap();
    term.wait_for("the palette lists Help: Release Notes", STEP, |s| {
        s.contains("Help: Release Notes")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    term.wait_for("the tab opened on the changelog", STEP, |s| {
        s.contains("You are running version")
            && s.contains("Update checks are turned off")
            && s.contains("Unreleased")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert_eq!(
        server.hits.load(Ordering::SeqCst),
        0,
        "mode off asks nothing"
    );
    term.stop().unwrap();
}

#[test]
fn mode_off_makes_no_request_and_the_chip_stays_the_version() {
    let server = serve();
    let log = tempfile::NamedTempFile::new().unwrap();
    let term = launch("pty-update-off", &server, "off", log.path());
    // The background check would have fired two seconds after the start.
    std::thread::sleep(Duration::from_secs(6));
    assert_eq!(
        server.hits.load(Ordering::SeqCst),
        0,
        "no request with updates.mode = off"
    );
    let s = term.screen();
    assert!(!s.contains("available"), "{}", s.text());
    assert!(
        s.contains(concat!("v", env!("CARGO_PKG_VERSION"))),
        "the running version is on the chip\n{}",
        s.text()
    );
    term.stop().unwrap();
}
