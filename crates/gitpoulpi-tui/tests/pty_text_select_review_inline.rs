#![cfg(all(feature = "radar", feature = "review"))]
//! The inline comment rows of the review's diff are selectable: a drag over
//! the words of a pending comment lights them and `Ctrl+C` copies them, the
//! pointer is the I-beam over the words and the hand over the row's buttons,
//! `Esc` drops the selection first and keeps the view, and a plain click on
//! the row selects nothing (a button on it still runs). Real binary through
//! a pty against `xtask mock-host` (127.0.0.1 only).

#![allow(dead_code)]

#[path = "common/mod.rs"]
mod common;

use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::mock_host::{self, MockHost, Options};
use xtask::pty::{Screen, Terminal};

const STARTUP: Duration = Duration::from_secs(60);
const STEP: Duration = Duration::from_secs(30);
const COLS: u16 = 160;
const ROWS: u16 = 50;
const ROW: &str = "Round invoice totals";
/// What the mock's newest discussion says.
const NEWEST: &str = "Newest remark from the mock.";
/// What the mock's first page of discussions says (a plain comment).
const OLDER: &str = "Looks promising, one question inline.";

struct Setup {
    term: Terminal,
    mock: MockHost,
    port_file: std::path::PathBuf,
    _config: tempfile::TempDir,
    _repo: FixtureRepo,
}

fn setup(session: &str) -> Setup {
    let mock = mock_host::start(Options {
        port: 0,
        bare_root: None,
    })
    .expect("the mock host");
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "a\n", "review-overview-start-commit")
        .unwrap()
        .build();
    let config = tempfile::tempdir().expect("temp config dir");
    std::fs::write(
        config.path().join("config.toml"),
        format!(
            "[[integrations]]\nid = \"lab\"\nkind = \"gitlab_self_managed\"\nurl = \"http://127.0.0.1:{port}\"\ntoken_ref = \"integration:lab\"\nname = \"lab\"\nuser = {{ login = \"{login}\" }}\n\n[integrations.radar]\nreview_requests = true\n",
            port = mock.port,
            login = mock_host::GITLAB_LOGIN,
        ),
    )
    .expect("seed config.toml");
    std::fs::create_dir_all(config.path().join("sessions")).unwrap();
    std::fs::write(
        config
            .path()
            .join("sessions")
            .join(format!("{session}.toml")),
        format!(
            "[[tabs]]\nrepo_path = '{}'\nactive = true\n",
            repo.root().display()
        ),
    )
    .expect("seed the session");
    let term = common::launch(repo.root())
        .session(session)
        .config_dir(config.path())
        .size(COLS, ROWS)
        .mouse_capture()
        .env("GITPOULPI_NO_BROWSER", "1")
        .env("GITPOULPI_TOKEN_LAB", "test-token")
        .env("GITPOULPI_DESKTOP_NO_HOOKS", "1")
        .spawn();
    term.wait_for("the repository loaded", STARTUP, |s| {
        s.contains("review-overview-start-commit")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    Setup {
        term,
        mock,
        port_file: config
            .path()
            .join("sessions")
            .join(format!("{session}.port")),
        _config: config,
        _repo: repo,
    }
}

fn wait_until(what: &str, mut ok: impl FnMut() -> bool) {
    let deadline = std::time::Instant::now() + STEP;
    while !ok() {
        assert!(std::time::Instant::now() < deadline, "timed out: {what}");
        std::thread::sleep(Duration::from_millis(50));
    }
}

/// The review of `!58` over the Radar, on its Overview with the newest
/// window of the conversation in.
fn open_review(s: &mut Setup) -> Screen {
    s.term.send_keys("M-r").expect("Alt+R");
    let screen = s
        .term
        .wait_for("the far request", STARTUP, |s| {
            s.contains("Radar") && s.contains(ROW) && s.contains("Not cloned")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let (col, row) = screen.find(ROW).unwrap();
    s.term.click(col + 2, row).expect("select the row");
    s.term.send_keys("e").expect("e");
    s.term
        .wait_for("the question", STEP, |s| {
            s.contains("Review through the API")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    s.term.send_keys("a").expect("a");
    s.term
        .wait_for("the Overview's timeline", STEP, |s| {
            s.contains("Review !58") && s.contains(NEWEST) && s.contains("New comment")
        })
        .unwrap_or_else(|e| panic!("{e}"))
}

/// `ui.text_selection_bg` of the default theme.
const SELECTED: xtask::pty::Color = xtask::pty::Color::Rgb(64, 156, 255);

/// The lit cells below the diff's first hunk header (the buttons above it
/// have the selection colour too), row by row.
fn lit(s: &Screen) -> String {
    let mut out = String::new();
    let top = s.find("@@ -10").map_or(0, |(_, r)| r + 1);
    for row in top..s.rows() {
        for col in 0..s.cols() {
            if let Some(cell) = s.cell(col, row) {
                if cell.bg == SELECTED {
                    out.push_str(cell.display());
                }
            }
        }
    }
    out
}

fn pointer_shapes(term: &Terminal) -> Vec<String> {
    let raw = String::from_utf8_lossy(&term.raw_output()).to_string();
    raw.split("\x1b]22;")
        .skip(1)
        .map(|s| s.split("\x1b\\").next().unwrap_or("").to_string())
        .collect()
}

fn hover(term: &mut Terminal, what: &str, (col, row): (u16, u16), want: &str) {
    term.move_to(col, row).unwrap();
    let deadline = std::time::Instant::now() + STEP;
    while pointer_shapes(term).last().map(String::as_str) != Some(want)
        && std::time::Instant::now() < deadline
    {
        std::thread::sleep(Duration::from_millis(50));
    }
    assert_eq!(
        pointer_shapes(term).last().map(String::as_str),
        Some(want),
        "over {what}: {:?}\n{}",
        pointer_shapes(term),
        term.text()
    );
}

#[test]
fn a_drag_over_an_inline_comment_selects_it_and_a_click_or_a_button_still_acts() {
    let mut s = setup("pty-text-select-inline");
    open_review(&mut s);
    s.term.send_keys("M-2").expect("Alt+2");
    let screen = s
        .term
        .wait_for("the diff", STEP, |s| {
            s.contains("src/planets.rs") && s.contains("+b")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let (col, row) = screen.find("+b").expect("an added line");
    s.term.click(col + 4, row).expect("the added line");
    s.term.send_keys("c").expect("comment");
    s.term
        .wait_for("the comment box", STEP, |s| s.contains("Comment on code"))
        .unwrap_or_else(|e| panic!("{e}"));
    s.term.send_text("check the rounding").unwrap();
    s.term.send_keys("C-s").expect("keep it");
    let screen = s
        .term
        .wait_for("the pending comment under its line", STEP, |s| {
            s.contains("1 pending comment") && s.contains("check the rounding")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
    let (c, r) = screen.find("check the rounding").expect("the comment row");

    // The I-beam over the words.
    hover(&mut s.term, "the comment words", (c + 3, r), "text");

    // A drag over "check" lights exactly those cells; Ctrl+C copies them.
    s.term.drag((c, r), (c + 4, r)).unwrap();
    s.term
        .wait_for("exactly the word lit", STEP, |s| lit(s) == "check")
        .unwrap_or_else(|e| panic!("{e}\nLIT={:?}\n{}", lit(&s.term.screen()), s.term.text()));
    s.term.send_keys("C-c").unwrap();
    s.term
        .wait_for("the copy reported", STEP, |s| {
            s.contains("copied the selection (check)") || s.contains("no clipboard available")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));

    // The comment's buttons keep the hand.
    let screen = s.term.screen();
    let (bc, br) = screen
        .find_all("Edit")
        .into_iter()
        .find(|(_, row)| *row + 1 == r)
        .unwrap_or_else(|| panic!("an Edit button above the words:\n{}", screen.text()));
    hover(&mut s.term, "the Edit button", (bc + 1, br), "pointer");

    // Esc drops the selection first; the review stays open.
    s.term.send_keys("Esc").unwrap();
    s.term
        .wait_for("no highlight, the review still open", STEP, |s| {
            lit(s).is_empty() && s.contains("Review !58")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));

    // A plain click on the words selects nothing (a pause first: two presses
    // on the row close together are a double-click, which adds a comment).
    std::thread::sleep(Duration::from_millis(900));
    s.term.click(c, r).unwrap();
    s.term
        .wait_for_stable(Duration::from_millis(300), STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(
        lit(&s.term.screen()).is_empty(),
        "a click selected text: {:?}",
        lit(&s.term.screen())
    );

    // A press on the button is still a click: it opens the comment to edit.
    std::thread::sleep(Duration::from_millis(900));
    s.term.click(bc + 1, br).unwrap();
    s.term
        .wait_for("the comment box to edit", STEP, |s| {
            s.contains("Edit pending comment")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
}

#[test]
fn a_drag_over_a_thread_head_selects_its_words_and_a_click_still_folds_it() {
    let mut s = setup("pty-text-select-thread-head");
    open_review(&mut s);
    s.term.send_keys("M-2").expect("Alt+2");
    let screen = s
        .term
        .wait_for("the diff with its thread", STEP, |s| {
            s.contains("Why a Vec here and not a slice?")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
    let (c, r) = screen
        .find("tomas \u{b7} 2026-10-11")
        .expect("the thread head");

    // The I-beam over the head's words; a drag lights them and does not fold.
    hover(&mut s.term, "the thread head words", (c + 2, r), "text");
    s.term.drag((c, r), (c + 4, r)).unwrap();
    s.term
        .wait_for("exactly the name lit", STEP, |s| lit(s) == "tomas")
        .unwrap_or_else(|e| panic!("{e}\nLIT={:?}\n{}", lit(&s.term.screen()), s.term.text()));
    assert!(
        s.term.screen().contains("It is built once at start-up."),
        "the drag folded the thread:\n{}",
        s.term.text()
    );
    s.term.send_keys("Esc").unwrap();
    s.term
        .wait_for("no highlight, the review still open", STEP, |s| {
            lit(s).is_empty() && s.contains("Review !58")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));

    // A plain click on the head still folds the thread, and selects nothing.
    s.term.click(c, r).unwrap();
    s.term
        .wait_for("the thread folded", STEP, |s| {
            !s.contains("It is built once at start-up.")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
    assert!(lit(&s.term.screen()).is_empty(), "a click selected text");
}
