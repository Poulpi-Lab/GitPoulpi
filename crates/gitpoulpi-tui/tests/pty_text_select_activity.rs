//! The Activity Log's text is selectable (both tabs): a drag over the words of
//! an entry or a log line lights them and `Ctrl+C` copies them, the pointer is
//! the I-beam over the text and the hand over a button, `Esc` drops the
//! selection first (the window stays), a plain click keeps its action (a
//! header opens its details, a button runs) and selects nothing, and a drag
//! that starts on a header selects instead of opening it.
//! Real binary, daemon and system git through a pty; the failing remote is a
//! closed local port.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::{Screen, Terminal};

const STEP: Duration = Duration::from_secs(30);
const COLS: u16 = 160;
const ROWS: u16 = 44;
/// `ui.text_selection_bg` of the default theme.
const SELECTED: xtask::pty::Color = xtask::pty::Color::Rgb(64, 156, 255);

fn launch(config: &std::path::Path, repo: &std::path::Path, session: &str) -> Terminal {
    common::launch(repo)
        .session(session)
        .config_dir(config)
        .size(COLS, ROWS)
        .env("GITPOULPI_CLIPBOARD", "memory")
        .ready("activity unpushed")
        .spawn()
}

/// The lit cells of the window below its tabs (the active tab has the
/// selection colour too), row by row.
fn lit(s: &Screen) -> String {
    let mut out = String::new();
    let top = s.find(" Git operations ").map_or(0, |(_, r)| r + 1);
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

/// The first place `needle` is in the window (not the status line below it).
fn find_in_window(s: &Screen, needle: &str) -> (u16, u16) {
    s.find_all(needle)
        .into_iter()
        .find(|(_, r)| *r < ROWS - 3)
        .unwrap_or_else(|| panic!("{needle:?} in the window:\n{}", s.text()))
}

fn repo_with_unreachable_origin() -> gitpoulpi_testkit::FixtureRepo {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("activity base")
        .unwrap()
        .with_origin()
        .unwrap()
        .push("main")
        .unwrap()
        .track("main", "origin")
        .unwrap()
        .commit("activity unpushed")
        .unwrap()
        .build();
    repo.git(&["remote", "set-url", "origin", "http://127.0.0.1:1/r.git"]);
    repo
}

/// A failed fetch, then the Activity Log open on the Git operations tab.
fn open_log_after_a_failed_fetch(term: &mut Terminal) -> Screen {
    term.send_keys("C-p").unwrap();
    term.wait_for_text("Command Palette", STEP).unwrap();
    term.send_text("Fetch All Remotes").unwrap();
    term.wait_for("the entry", STEP, |s| s.contains("Git: Fetch All Remotes"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    term.wait_for("the error and its count", STEP, |s| {
        s.contains("error: Fetch failed") && s.contains("Activity (1)")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (c, r) = term.screen().find("Activity (1)").unwrap();
    term.click(c + 2, r).expect("click the Activity button");
    term.wait_for("the Activity Log", STEP, |s| {
        s.contains(" Git operations ") && s.contains("Fetch failed")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()))
}

#[test]
fn a_drag_over_an_entry_selects_it_a_click_still_opens_it_and_a_button_still_runs() {
    let repo = repo_with_unreachable_origin();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch(config.path(), repo.root(), "pty-text-select-activity");
    let screen = open_log_after_a_failed_fetch(&mut term);

    // The I-beam over the header's words; a plain click opens the details
    // and selects nothing.
    let (hc, hr) = find_in_window(&screen, "Fetch failed");
    hover(&mut term, "the header text", (hc + 2, hr), "text");
    term.click(hc + 2, hr).unwrap();
    let opened = term
        .wait_for("the details opened by the click", STEP, |s| {
            s.contains("command: git fetch") && s.contains(" Copy ")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(lit(&opened).is_empty(), "a click selected text");

    // A drag over a detail line lights exactly those words; Ctrl+C copies.
    let (cc, cr) = find_in_window(&opened, "command: git fetch");
    term.drag((cc, cr), (cc + "command:".len() as u16 - 1, cr))
        .unwrap();
    term.wait_for("exactly the words lit", STEP, |s| lit(s) == "command:")
        .unwrap_or_else(|e| panic!("{e}\nLIT={:?}\n{}", lit(&term.screen()), term.text()));
    term.send_keys("C-c").unwrap();
    term.wait_for("the copy reported", STEP, |s| {
        s.contains("copied the selection (command:)")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // Esc drops the selection first, the window stays.
    term.send_keys("Esc").unwrap();
    term.wait_for("no highlight, the window open", STEP, |s| {
        lit(s).is_empty() && s.contains(" Git operations ")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // A drag that starts on the header selects its words and does not close
    // the details (a drag is not a click).
    let screen = term.screen();
    let (hc, hr) = find_in_window(&screen, "Fetch failed");
    term.drag((hc, hr), (hc + "Fetch failed".len() as u16 - 1, hr))
        .unwrap();
    term.wait_for("the header words lit", STEP, |s| lit(s) == "Fetch failed")
        .unwrap_or_else(|e| panic!("{e}\nLIT={:?}\n{}", lit(&term.screen()), term.text()));
    assert!(
        term.screen().contains("command: git fetch"),
        "the drag folded the entry:\n{}",
        term.text()
    );
    term.send_keys("Esc").unwrap();
    term.wait_for("no highlight", STEP, |s| lit(s).is_empty())
        .unwrap_or_else(|e| panic!("{e}"));

    // The buttons keep the hand and their click.
    let screen = term.screen();
    // (Not the toolbar's "Copy diagnostics": the one under the entry's details.)
    let below = find_in_window(&screen, "exit code:").1;
    let (bc, br) = screen
        .find_all(" Copy ")
        .into_iter()
        .find(|(_, r)| *r > below)
        .unwrap_or_else(|| panic!("the entry's Copy button:\n{}", screen.text()));
    hover(&mut term, "the Copy button", (bc + 2, br), "pointer");
    term.click(bc + 2, br).unwrap();
    term.wait_for("the button's copy", STEP, |s| {
        s.contains("copied the error details")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

#[test]
fn a_drag_over_a_line_of_the_application_log_selects_it_and_a_click_selects_nothing() {
    let repo = repo_with_unreachable_origin();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch(config.path(), repo.root(), "pty-text-select-applog");
    open_log_after_a_failed_fetch(&mut term);
    term.send_keys("Tab").unwrap();
    let log = term
        .wait_for("the application log", STEP, |s| {
            s.contains("Level:") && s.contains("gitpoulpi-tui starting")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    let (c, r) = find_in_window(&log, "gitpoulpi-tui starting");
    hover(&mut term, "a log line", (c + 3, r), "text");
    term.drag((c, r), (c + "gitpoulpi-tui".len() as u16 - 1, r))
        .unwrap();
    term.wait_for("exactly the words lit", STEP, |s| lit(s) == "gitpoulpi-tui")
        .unwrap_or_else(|e| panic!("{e}\nLIT={:?}\n{}", lit(&term.screen()), term.text()));
    term.send_keys("C-c").unwrap();
    term.wait_for("the copy reported", STEP, |s| {
        s.contains("copied the selection (gitpoulpi-tui)")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Esc").unwrap();
    term.wait_for("no highlight, the window open", STEP, |s| {
        lit(s).is_empty() && s.contains("Level:")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // A plain click on a line selects no text.
    term.click(c, r).unwrap();
    term.wait_for_stable(Duration::from_millis(300), STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(lit(&term.screen()).is_empty(), "a click selected text");
}
