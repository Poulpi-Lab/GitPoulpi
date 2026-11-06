//! The update check, the title bar's version chip and the Release notes tab,
//! against a fake release server on loopback (`GITPOULPI_UPDATE_API`; test
//! builds refuse every other host). The real binary, in a pty.
//!
//! A newer release makes the chip read `v99.0.0 available`; a click on it opens
//! the update panel, whose Release notes button opens the tab with that
//! release's body on top; Download opens the release page (through the test
//! stub of the browser), Skip this version puts the chip back and stays
//! skipped. `updates.mode = "off"` makes no request.
//!
//! The security notices (`security.json` next to the release, signed): a
//! signed notice that names this version turns the chip red and the panel
//! lists it; an unsigned one changes nothing. The app is a debug build, which
//! takes the test key through `GITPOULPI_TEST_RELEASE_KEY`.
//!
//! The binary is a copy in a temp folder next to a marker (an archive
//! install): a build with no install record never asks anyone.

mod common;

use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use gitpoulpi_config::release::{self, Marker, MARKER_FILE};
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
    serve_with(None)
}

/// The security notices a fake release publishes: the file and its `.minisig`.
struct Notices {
    body: String,
    signature: String,
}

/// A notice for any version below 99.0.0 (this build), signed or not.
fn notices(signed: bool) -> Notices {
    let body = serde_json::json!({"schema": 1, "advisories": [{
        "id": "GPSA-2026-001", "affected": "<99.0.0", "fixed": "99.0.0",
        "severity": "critical", "url": "https://example.test/advisories/1",
        "summary": "a fake flaw"}]})
    .to_string();
    let signature = if signed {
        common::release_signer().sign("security.json", "99.0.0", body.as_bytes())
    } else {
        String::new()
    };
    Notices { body, signature }
}

fn serve_with(notices: Option<Notices>) -> FakeReleases {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind loopback");
    let port = listener.local_addr().unwrap().port();
    let hits = Arc::new(AtomicUsize::new(0));
    let paths = Arc::new(Mutex::new(Vec::new()));
    let (h, p) = (hits.clone(), paths.clone());
    let notices = Arc::new(notices);
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
            p.lock().unwrap().push(path.clone());
            let body = match (notices.as_ref(), path.as_str()) {
                (Some(n), p) if p.ends_with("/security.json") => n.body.as_str(),
                (Some(n), p)
                    if p.ends_with("/security.json.minisig") && !n.signature.is_empty() =>
                {
                    n.signature.as_str()
                }
                _ => BODY,
            };
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            let _ = stream.write_all(response.as_bytes());
        }
    });
    FakeReleases { port, hits, paths }
}

/// A copy of the built TUI next to the marker of an archive install.
fn installed_exe() -> std::path::PathBuf {
    let dir = tempfile::tempdir().unwrap();
    let exe = dir.path().join("gitpoulpi-tui");
    let built = env!("CARGO_BIN_EXE_gitpoulpi-tui");
    if std::fs::hard_link(built, &exe).is_err() {
        std::fs::copy(built, &exe).unwrap();
    }
    let marker = Marker::new(env!("CARGO_PKG_VERSION"), release::host_target());
    std::fs::write(dir.path().join(MARKER_FILE), marker.render()).unwrap();
    // The install folder must outlive the app.
    std::mem::forget(dir);
    exe
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
        .binary(installed_exe())
        .env("GITPOULPI_UPDATES_MODE", mode)
        .env(
            common::RELEASE_KEY_ENV,
            common::release_signer().public_key_text(),
        )
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
    // One request for the releases list; the security notices are asked for
    // next to it (this fake publishes none).
    let releases: Vec<String> = server
        .paths
        .lock()
        .unwrap()
        .iter()
        .filter(|p| p.starts_with("/repos/"))
        .cloned()
        .collect();
    assert_eq!(releases.len(), 1, "one releases request: {releases:?}");
    let path = releases[0].clone();
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
    // A click opens the update panel; its Release notes button the tab: the
    // release body on top, the buttons, the tab.
    term.click(col + 1, row).unwrap();
    term.wait_for("the update panel", STEP, |s| {
        s.contains("Installed with:") && s.contains("Release notes")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (nc, nr) = term.screen().find("Release notes").unwrap();
    term.click(nc + 1, nr).unwrap();
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

#[test]
fn a_signed_security_notice_turns_the_chip_red_and_the_panel_lists_it() {
    let server = serve_with(Some(notices(true)));
    let log = tempfile::NamedTempFile::new().unwrap();
    let mut term = launch("pty-update-notice", &server, "notify", log.path());
    term.wait_for("the chip warns about the notice", STEP, |s| {
        s.contains("Security notice: this version is affected")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // The notices are fetched from where a release publishes them.
    assert!(
        server
            .paths
            .lock()
            .unwrap()
            .iter()
            .any(|p| p == "/Poulpi-Lab/GitPoulpi/releases/latest/download/security.json"),
        "{:?}",
        server.paths.lock().unwrap()
    );
    // The chip is red: the cell under its first letter has the danger colour.
    let (col, row) = term
        .screen()
        .find("Security notice: this version")
        .expect("the chip");
    assert_eq!(row, 0);
    let fg = term.screen().cell(col + 1, row).expect("a cell").fg;
    let reddish = match fg {
        xtask::pty::Color::Rgb(r, g, b) => {
            r > 150 && r > g.saturating_add(40) && r > b.saturating_add(40)
        }
        xtask::pty::Color::Indexed(n) => matches!(n, 1 | 9 | 160..=203),
        xtask::pty::Color::Default => false,
    };
    assert!(reddish, "the chip is red, not {fg:?}");
    // A click opens the panel, which lists the notice.
    term.click(col + 1, row).unwrap();
    term.wait_for("the panel lists the notice", STEP, |s| {
        s.contains("Installed with:")
            && s.contains("Critical: a fake flaw")
            && s.contains("Fixed in version 99.0.0.")
            && s.contains("https://example.test/advisories/1")
            && s.contains("Open notice")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // The keyboard twin opens the link through the shared URL opener.
    term.send_text("a").unwrap();
    term.wait_for("the notice page was opened", STEP, |_| {
        std::fs::read_to_string(log.path())
            .map(|t| t.contains("https://example.test/advisories/1"))
            .unwrap_or(false)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

#[test]
fn an_unsigned_security_notice_is_never_shown() {
    let server = serve_with(Some(notices(false)));
    let log = tempfile::NamedTempFile::new().unwrap();
    let term = launch("pty-update-notice-unsigned", &server, "notify", log.path());
    // The release itself is announced: the check ran, and asked for the notice.
    term.wait_for("the chip announces the release", STEP, |s| {
        s.contains("v99.0.0 available")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.wait_for("the notices were asked for", STEP, |_| {
        server
            .paths
            .lock()
            .unwrap()
            .iter()
            .any(|p| p.ends_with("/security.json.minisig"))
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    std::thread::sleep(Duration::from_secs(1));
    assert!(
        !term.screen().contains("Security notice"),
        "{}",
        term.text()
    );
    term.stop().unwrap();
}
