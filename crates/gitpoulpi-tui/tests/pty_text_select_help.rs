//! The help popup's text is selectable: a drag over the body lights exactly
//! those words and `Ctrl+C` copies them, the pointer is the I-beam over the
//! text, the hand on a tab header and the docs link, `Esc` drops the selection
//! first (the popup stays), a plain click on a tab still switches it and
//! selects nothing, and a plain click on the text selects nothing.
//! Real binary through a pty.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::{Screen, Terminal};

const STEP: Duration = Duration::from_secs(30);
/// `ui.text_selection_bg` of the default theme.
const SELECTED: xtask::pty::Color = xtask::pty::Color::Rgb(64, 156, 255);

/// The lit cells below the popup's tab row (the active tab has the
/// selection colour too), row by row.
fn lit(s: &Screen) -> String {
    let mut out = String::new();
    let top = s.find_all("AI & CLI").first().map_or(0, |(_, r)| r + 1);
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

fn launch() -> Terminal {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("help select probe")
        .unwrap()
        .build();
    let (term, _config) = common::launch(repo.root())
        .session("pty-text-select-help")
        .size(200, 50)
        .ready("help select probe")
        .mouse_capture()
        .start();
    // The fixture repo and the config dir have to outlive the app.
    std::mem::forget(repo);
    std::mem::forget(_config);
    term
}

#[test]
fn a_drag_over_the_help_selects_it_ctrl_c_copies_it_and_the_tabs_keep_the_hand() {
    let mut term = launch();
    term.send_text("?").unwrap();
    let screen = term
        .wait_for("the help popup", STEP, |s| {
            s.contains("AI & CLI") && s.contains("Ctrl+P")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.wait_for_stable(Duration::from_millis(300), STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    let tab_row = screen.find_all("AI & CLI").first().expect("the tab row").1;
    let (col, row) = screen
        .find_all("Ctrl+P")
        .into_iter()
        .find(|(_, r)| *r > tab_row)
        .unwrap_or_else(|| panic!("Ctrl+P in the body:\n{}", screen.text()));

    // The I-beam over the body's text.
    hover(&mut term, "the body text", (col + 2, row), "text");

    // A drag over the key lights exactly its cells; Ctrl+C copies them.
    term.drag((col, row), (col + "Ctrl+P".len() as u16 - 1, row))
        .unwrap();
    term.wait_for("exactly the words lit", STEP, |s| lit(s) == "Ctrl+P")
        .unwrap_or_else(|e| panic!("{e}\nLIT={:?}\n{}", lit(&term.screen()), term.text()));
    term.send_keys("C-c").unwrap();
    term.wait_for("the copy reported", STEP, |s| {
        s.contains("copied the selection (Ctrl+P)") || s.contains("no clipboard available")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // A selection survives a scroll: it moves up with its text (and one
    // that scrolled out of view comes back with it).
    let (ac, ar) = screen
        .find_all("Alt+C")
        .into_iter()
        .find(|(_, r)| *r > tab_row)
        .unwrap_or_else(|| panic!("Alt+C in the body:\n{}", screen.text()));
    term.drag((ac, ar), (ac + "Alt+C".len() as u16 - 1, ar))
        .unwrap();
    term.wait_for("Alt+C lit", STEP, |s| lit(s) == "Alt+C")
        .unwrap_or_else(|e| panic!("{e}\nLIT={:?}\n{}", lit(&term.screen()), term.text()));
    term.scroll(true, ac, ar).unwrap();
    term.wait_for("scrolled, the selection moved with its text", STEP, |s| {
        lit(s) == "Alt+C"
            && s.find_all("Alt+C")
                .into_iter()
                .any(|(_, r)| r + 3 == ar && r > tab_row)
    })
    .unwrap_or_else(|e| panic!("{e}\nLIT={:?}\n{}", lit(&term.screen()), term.text()));
    term.scroll(false, ac, ar).unwrap();
    term.wait_for("back where it was", STEP, |s| {
        lit(s) == "Alt+C" && s.find_all("Alt+C").into_iter().any(|(_, r)| r == ar)
    })
    .unwrap_or_else(|e| panic!("{e}\nLIT={:?}\n{}", lit(&term.screen()), term.text()));

    // The tabs and the docs link keep the hand.
    let (tc, tr) = screen
        .find_all("AI & CLI")
        .first()
        .copied()
        .expect("a tab header");
    hover(&mut term, "a tab header", (tc + 1, tr), "pointer");
    let (lc, lr) = screen.find("User docs on GitHub").expect("the docs link");
    hover(&mut term, "the docs link", (lc + 2, lr), "pointer");

    // Esc drops the selection first; the popup stays.
    term.send_keys("Esc").unwrap();
    term.wait_for("no highlight, the popup still open", STEP, |s| {
        lit(s).is_empty() && s.contains("AI & CLI")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // A plain click on the text selects nothing.
    term.click(col, row).unwrap();
    term.wait_for_stable(Duration::from_millis(300), STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(lit(&term.screen()).is_empty(), "a click selected text");

    // A plain click on a tab still switches to it, and selects nothing.
    term.click(tc + 1, tr).unwrap();
    term.wait_for("the tools tab", STEP, |s| s.contains("AI (optional)"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(lit(&term.screen()).is_empty(), "a tab click selected text");
}
