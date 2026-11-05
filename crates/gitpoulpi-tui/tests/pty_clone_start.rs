//! Which source the Clone dialog opens on: the one it showed when it was
//! last closed (`repos.clone_start_source = "last"`, the default), always
//! Clone with URL (`url`), or a named integration profile.
//!
//! The rules are `pty_ui.rs`'s: nothing is located by a hard-coded
//! coordinate, nothing sleeps, every wait describes the whole state it is
//! waiting for. The mock host answers the repository list.
#![cfg(all(feature = "github", feature = "gitlab"))]

mod common;

use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::mock_host::{self, Options};
use xtask::pty::{Screen, Terminal};

const STARTUP: Duration = Duration::from_secs(60);
const STEP: Duration = Duration::from_secs(30);
const COLS: u16 = 160;
const ROWS: u16 = 60;
const HUB: &str = "GitHub \u{00B7} hub";
const LAB: &str = "GitLab \u{00B7} lab";

struct Setup {
    term: Terminal,
    _mock: xtask::mock_host::MockHost,
    _bare: tempfile::TempDir,
    _root: tempfile::TempDir,
    _config: tempfile::TempDir,
    _start: gitpoulpi_testkit::FixtureRepo,
}

/// The TUI on the management tab with the Clone dialog open once already
/// (and closed again with Esc when `warm` is not wanted: the caller does it).
/// `repos_extra` is appended to the `[repos]` table.
fn open_clone(session: &str, repos_extra: &str) -> Setup {
    let bare = tempfile::tempdir().unwrap();
    mock_host::seed_bare_repos(bare.path()).unwrap();
    let mock = mock_host::start(Options {
        port: 0,
        bare_root: Some(bare.path().to_path_buf()),
    })
    .unwrap();
    let start = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "a\n", "clone-start-first-commit")
        .unwrap()
        .build();
    let root = tempfile::tempdir().unwrap();
    let config = tempfile::tempdir().unwrap();
    std::fs::write(
        config.path().join("config.toml"),
        format!(
            "[repos]\nclone_root = '{root}'\n{repos_extra}\n\n\
             [[integrations]]\nid = \"hub\"\nkind = \"github_enterprise\"\nurl = \"http://127.0.0.1:{port}\"\ntoken_ref = \"integration:hub\"\nname = \"hub\"\nuser = {{ login = \"{gh}\" }}\n\n\
             [[integrations]]\nid = \"lab\"\nkind = \"gitlab_self_managed\"\nurl = \"http://127.0.0.1:{port}\"\ntoken_ref = \"integration:lab\"\nname = \"lab\"\nuser = {{ login = \"{gl}\" }}\n",
            root = root.path().display(),
            port = mock.port,
            gh = mock_host::GITHUB_LOGIN,
            gl = mock_host::GITLAB_LOGIN,
        ),
    )
    .unwrap();
    let mut term = common::launch(start.root())
        .session(session)
        .config_dir(config.path())
        .size(COLS, ROWS)
        .env("GITPOULPI_NO_BROWSER", "1")
        .env("GITPOULPI_TOKEN_HUB", "hub-token")
        .env("GITPOULPI_TOKEN_LAB", "lab-token")
        .spawn();
    term.wait_for("the first window", STARTUP, |s| {
        s.contains("clone-start-first-commit")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("C-o").expect("the management tab");
    term.wait_for("the management tab", STEP, |s| {
        s.contains("Repository Management") && s.contains("Clone")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    let mut setup = Setup {
        term,
        _mock: mock,
        _bare: bare,
        _root: root,
        _config: config,
        _start: start,
    };
    open_dialog(&mut setup.term);
    setup
}

fn open_dialog(term: &mut Terminal) {
    term.send_keys("c").expect("Clone");
    term.wait_for("the Clone dialog with both sources", STEP, |s| {
        s.contains("Clone a repository") && s.contains(HUB) && s.contains(LAB)
    })
    .unwrap_or_else(|e| panic!("{e}"));
}

fn close_dialog(term: &mut Terminal) {
    term.send_keys("Escape").unwrap();
    term.wait_for("the dialog closed", STEP, |s| {
        !s.contains("Clone a repository")
    })
    .unwrap_or_else(|e| panic!("{e}"));
}

fn pick_source(term: &mut Terminal, label: &str) {
    let (col, row) = term.screen().find(label).unwrap();
    term.click(col + 2, row).expect("pick the source");
}

/// A field label of the dialog's right column ("Repository" exists only on
/// a hosted source, "URL" only on "Clone with URL").
fn has_label(s: &Screen, label: &str) -> bool {
    s.text().lines().any(|l| {
        l.split('\u{2502}').any(|cell| {
            let cell = cell.trim();
            cell == label || cell.starts_with(&format!("{label}  "))
        })
    })
}

fn on_hosted(s: &Screen) -> bool {
    s.contains("Where to clone to") && has_label(s, "Repository")
}

fn on_url(s: &Screen) -> bool {
    s.contains("Where to clone to") && has_label(s, "URL") && !has_label(s, "Repository")
}

#[test]
fn the_dialog_reopens_on_the_source_it_was_closed_on() {
    let mut s = open_clone("pty-clone-start-last", "");
    // First opening: nothing remembered, so Clone with URL.
    assert!(on_url(&s.term.screen()), "{}", s.term.screen().text());
    pick_source(&mut s.term, HUB);
    s.term
        .wait_for("the hub list", STEP, |s| {
            on_hosted(s) && s.contains("mara-quinn")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    close_dialog(&mut s.term);
    open_dialog(&mut s.term);
    let shown = s
        .term
        .wait_for(
            "the dialog back on the hub source with its list",
            STEP,
            |s| on_hosted(s) && s.contains("mara-quinn"),
        )
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(!on_url(&shown), "{}", shown.text());
    // Back on URL and closed: the next opening is URL again.
    pick_source(&mut s.term, "Clone with URL");
    s.term
        .wait_for("the URL form", STEP, on_url)
        .unwrap_or_else(|e| panic!("{e}"));
    close_dialog(&mut s.term);
    open_dialog(&mut s.term);
    s.term
        .wait_for("the URL form again", STEP, on_url)
        .unwrap_or_else(|e| panic!("{e}"));
}

#[test]
fn with_start_source_url_it_always_opens_on_the_url_form() {
    let mut s = open_clone("pty-clone-start-url", "clone_start_source = \"url\"");
    pick_source(&mut s.term, HUB);
    s.term
        .wait_for("the hub list", STEP, |s| {
            on_hosted(s) && s.contains("mara-quinn")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    close_dialog(&mut s.term);
    open_dialog(&mut s.term);
    s.term
        .wait_for("the URL form", STEP, on_url)
        .unwrap_or_else(|e| panic!("{e}"));
}

#[test]
fn with_a_named_profile_it_opens_on_that_profile_and_lists_it() {
    let s = open_clone("pty-clone-start-named", "clone_start_source = \"lab\"");
    let shown = s
        .term
        .wait_for("the lab list", STEP, |s| {
            on_hosted(s) && s.contains("northwind")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(!on_url(&shown), "{}", shown.text());
}
