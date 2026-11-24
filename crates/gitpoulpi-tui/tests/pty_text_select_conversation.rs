#![cfg(all(feature = "radar", feature = "review"))]
//! The conversation's text is selectable: a drag over a comment of the
//! review's Overview lights it and `Ctrl+C` copies it, the pointer is the
//! I-beam over the text and the hand over a button, `Esc` drops the
//! selection and keeps the view. Real binary through a pty against
//! `xtask mock-host` (127.0.0.1 only).

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

/// The lit cells of the timeline (below its identity line), row by row.
fn lit(s: &Screen) -> String {
    let mut out = String::new();
    let top = s.find("Posting as").map_or(0, |(_, r)| r + 1);
    let end = s.find("Preview").map_or(s.rows(), |(_, r)| r);
    for row in top..end {
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
fn a_drag_over_a_comment_selects_it_ctrl_c_copies_it_and_the_pointer_is_an_i_beam() {
    let mut s = setup("pty-text-select-conversation");
    let screen = open_review(&mut s);
    let (col, row) = screen.find(NEWEST).expect("the newest remark");

    // The I-beam over the comment's text.
    hover(&mut s.term, "the comment text", (col + 3, row), "text");

    // A drag over "Newest remark" lights exactly those cells.
    s.term
        .drag((col, row), (col + "Newest remark".len() as u16 - 1, row))
        .unwrap();
    s.term
        .wait_for("the words highlighted, nothing else", STEP, |s| {
            lit(s) == "Newest remark"
        })
        .unwrap_or_else(|e| panic!("{e}\nLIT={:?}\n{}", lit(&s.term.screen()), s.term.text()));

    // Ctrl+C copies it; the status line echoes what went to the clipboard.
    s.term.send_keys("C-c").unwrap();
    s.term
        .wait_for("the copy reported", STEP, |s| {
            s.contains("copied text (Newest remark)") || s.contains("no clipboard available")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));

    // The entry's buttons keep the hand.
    let screen = s.term.screen();
    let (bc, br) = screen
        .find("Reply")
        .unwrap_or_else(|| panic!("a Reply button:\n{}", screen.text()));
    hover(&mut s.term, "the Reply button", (bc + 1, br), "pointer");

    // Esc drops the selection first; the review stays open.
    s.term.send_keys("Esc").unwrap();
    s.term
        .wait_for("no highlight, the review still open", STEP, |s| {
            lit(s).is_empty() && s.contains("Review !58")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));

    // A plain click selects no text.
    s.term.click(col, row).unwrap();
    s.term
        .wait_for_stable(Duration::from_millis(300), STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(lit(&s.term.screen()).is_empty(), "a click selected text");
}
