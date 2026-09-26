use crate::mock::*;
use crate::support::*;
use xtask::pty::Terminal;

/// The glyph of the row's "Edit on ..." button.
const PENCIL: &str = "\u{270E}";

/// Hovers the `!53` row and returns the cell its pencil appears at.
fn hover_pencil(term: &mut Terminal) -> (u16, u16) {
    let (col, row) = term
        .wait_for_text("!53  Add the parser", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.move_to(col + 2, row).expect("hover the row");
    let screen = term
        .wait_for("the edit pencil on the hovered row", STEP, |s| {
            s.find_all(PENCIL).iter().any(|&(_, r)| r == row)
        })
        .unwrap_or_else(|e| panic!("{e}"));
    screen
        .find_all(PENCIL)
        .into_iter()
        .find(|&(_, r)| r == row)
        .unwrap()
}

/// A pull request row grows an edit pencil while hovered; clicking it
/// opens the host's edit page (`.../merge_requests/53/edit` on GitLab);
/// with the row menu open over another row's pencil, the pointer belongs
/// to the menu.
#[test]
fn the_row_pencil_edits_on_the_host_and_yields_to_an_open_menu() {
    let mock = start_mock();
    let repo = fixture(mock.port);
    let (mut term, _config) = launch(
        &repo,
        "pty-hosting-edit-btn",
        &config_toml(mock.port, ""),
        COLS,
        "Add the feature",
    );
    term.wait_for("the section", STARTUP, pulls_ready)
        .unwrap_or_else(|e| panic!("{e}"));
    // Not hovered, not selected: no pencil anywhere.
    term.move_to(COLS - 1, ROWS - 1).unwrap();
    term.wait_for("no pencil at rest", STEP, |s| s.find_all(PENCIL).is_empty())
        .unwrap_or_else(|e| panic!("{e}"));

    let (pc, pr) = hover_pencil(&mut term);
    term.click(pc + 1, pr).expect("click the pencil");
    term.wait_for("the edit URL is opened", STEP, |s| {
        s.contains("merge_requests/53/edit")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.screen().text()));

    // Unfold All so a second `!54` row sits under the first row's menu.
    let (gc, gr) = term.wait_for_text("All Pull Requests", STEP).unwrap();
    term.click(gc + 2, gr).expect("unfold All");
    term.wait_for("All unfolded", STEP, |s| s.find_all("!54  Docs").len() >= 2)
        .unwrap_or_else(|e| panic!("{e}"));
    // Right-click the pencil cell of `!53`: the row menu opens there.
    let (pc, pr) = hover_pencil(&mut term);
    term.right_click(pc + 1, pr)
        .expect("right-click the pencil");
    let screen = term
        .wait_for("the row menu", STEP, |s| s.contains("Open in browser"))
        .unwrap_or_else(|e| panic!("{e}"));
    // A `!54` row's pencil column is covered by the menu's rows: clicking
    // there hits the menu (an item runs, the menu closes), never the
    // covered row's pencil (the 54 edit URL).
    let (_, below) = screen
        .find_all("!54  Docs")
        .into_iter()
        .find(|&(_, r)| r > pr)
        .expect("a 54 row below");
    term.click(pc + 1, below).expect("click through the menu");
    term.wait_for("the menu closed", STEP, |s| !s.contains("Open in browser"))
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(
        !term.screen().contains("merge_requests/54"),
        "the click fell through the menu:\n{}",
        term.screen().text()
    );
}

/// The keyboard twin (`e` on the selected row) and the palette twin
/// (the checked-out branch's request) open the same edit page.
#[test]
fn edit_on_the_host_has_a_keyboard_and_a_palette_twin() {
    let mock = start_mock();
    let repo = fixture(mock.port);
    let (mut term, _config) = launch(
        &repo,
        "pty-hosting-edit-keys",
        &config_toml(mock.port, ""),
        COLS,
        "Add the feature",
    );
    term.wait_for("the section", STARTUP, pulls_ready)
        .unwrap_or_else(|e| panic!("{e}"));
    // Select the 54 row by clicking it (a single click selects).
    let (col, row) = term.wait_for_text("!54  Docs", STEP).unwrap();
    term.click(col + 2, row).expect("select the row");
    term.send_keys("e").unwrap();
    term.wait_for("e opens the selected row's edit page", STEP, |s| {
        s.contains("merge_requests/54/edit")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.screen().text()));

    // The palette: the branch `feature` has !53.
    term.send_keys("C-p").unwrap();
    term.send_text("Edit Linked").unwrap();
    term.wait_for("the palette lists the command", STEP, |s| {
        s.contains("Edit Linked Pull Request")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.screen().text()));
    term.send_keys("Enter").unwrap();
    term.wait_for("the palette twin opens !53's edit page", STEP, |s| {
        s.contains("merge_requests/53/edit")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.screen().text()));
}
