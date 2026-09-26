use crate::mock::*;
use crate::support::*;
use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::Terminal;

// -- the composer's "Insert issue reference" picker (more-providers S11) --------

const POPUP_BRANCH_ROW: &str = "#7  (from branch)";
const POPUP_ISSUE_ROW: &str = "#12  Crash on startup";

/// A working-changes repository on branch `7-login` whose `origin` is the
/// mock. The mock's only issue is `#12`, so `#7` can only come from the
/// branch name.
fn branch_fixture(port: u16) -> FixtureRepo {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "a\n", "pty-ref-start")
        .unwrap()
        .with_origin()
        .unwrap()
        .push("main")
        .unwrap()
        .branch("7-login")
        .unwrap()
        .checkout("7-login")
        .unwrap()
        .write_worktree("w.txt", b"w\n")
        .unwrap()
        .build();
    repo.git(&[
        "remote",
        "set-url",
        "origin",
        &format!("http://127.0.0.1:{port}/g/p.git"),
    ]);
    repo
}

/// Linked: waits for the ISSUES rows, then puts the keyboard in the
/// Summary (the Log's `c`).
fn linked(session: &str) -> (Terminal, tempfile::TempDir, FixtureRepo, Mock) {
    let mock = start_mock();
    let repo = branch_fixture(mock.port);
    let (mut term, config) =
        launch_issues(&repo, session, &config_toml(mock.port, ""), COLS, "// WIP");
    term.wait_for("the ISSUES rows", STARTUP, |s| {
        s.contains("Crash on startup")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.screen().text()));
    focus_summary(&mut term);
    (term, config, repo, mock)
}

fn focus_summary(term: &mut Terminal) {
    term.send_keys("c").expect("press c in the Log");
    term.wait_for("the composer with its # button", STEP, |s| {
        s.contains("Staged Files") && s.contains("#\u{25BE}")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.screen().text()));
}

/// The popup's copy of `text`: the right-most match (the sidebar's ISSUES
/// rows spell `#12  Crash on startup` the same way).
fn popup_row(screen: &xtask::pty::Screen, text: &str) -> (u16, u16) {
    screen
        .find_all(text)
        .into_iter()
        .max_by_key(|(c, _)| *c)
        .unwrap_or_else(|| panic!("no {text:?}:\n{}", screen.text()))
}

fn wait_popup(term: &Terminal) -> xtask::pty::Screen {
    term.wait_for("the picker with the branch row first", STEP, |s| {
        s.contains(POPUP_BRANCH_ROW) && s.contains(POPUP_ISSUE_ROW)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.screen().text()))
}

#[test]
fn the_hash_button_lists_the_branch_issue_first_and_enter_inserts_it() {
    let (mut term, _config, _repo, _mock) = linked("pty-hosting-composer-ref-button");
    let (col, row) = term.screen().find("#\u{25BE}").unwrap();
    term.click(col, row).expect("click the # button");
    let screen = wait_popup(&term);
    let (_, branch_row) = popup_row(&screen, POPUP_BRANCH_ROW);
    let (_, issue_row) = popup_row(&screen, POPUP_ISSUE_ROW);
    assert!(
        branch_row < issue_row,
        "the branch's issue is listed first:\n{}",
        screen.text()
    );
    term.send_keys("Enter").unwrap();
    term.send_text("fix").unwrap();
    term.wait_for("the reference in the Summary", STEP, |s| {
        s.contains("#7 fix") && !s.contains(POPUP_BRANCH_ROW)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.screen().text()));
}

#[test]
fn the_keyboard_chord_opens_the_picker_and_the_search_line_filters_it() {
    let (mut term, _config, _repo, _mock) = linked("pty-hosting-composer-ref-chord");
    term.send_keys("M-r").unwrap();
    wait_popup(&term);
    term.send_text("crash").unwrap();
    term.wait_for("only the matching row is left", STEP, |s| {
        s.contains(POPUP_ISSUE_ROW) && !s.contains(POPUP_BRANCH_ROW)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.screen().text()));
    term.send_keys("Enter").unwrap();
    term.send_text("now").unwrap();
    term.wait_for("#12 in the Summary", STEP, |s| s.contains("#12 now"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.screen().text()));
}

#[test]
fn the_palette_opens_the_picker_and_escape_leaves_the_message_alone() {
    let (mut term, _config, _repo, _mock) = linked("pty-hosting-composer-ref-palette");
    term.send_keys("C-p").unwrap();
    term.send_text("Insert Issue Reference").unwrap();
    term.wait_for("the palette lists it", STEP, |s| {
        s.contains("Commit: Insert Issue Reference")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.screen().text()));
    term.send_keys("Enter").unwrap();
    wait_popup(&term);
    term.send_keys("Esc").unwrap();
    term.wait_for("the picker closed", STEP, |s| !s.contains(POPUP_BRANCH_ROW))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.screen().text()));
    assert!(
        !term.screen().contains("#7 "),
        "nothing was inserted:\n{}",
        term.screen().text()
    );
}

#[test]
fn a_click_on_a_popup_row_inserts_it_and_does_not_fall_through() {
    let (mut term, config, _repo, _mock) = linked("pty-hosting-composer-ref-click");
    let (col, row) = term.screen().find("#\u{25BE}").unwrap();
    term.click(col, row).expect("click the # button");
    let screen = wait_popup(&term);
    let (c, r) = popup_row(&screen, POPUP_ISSUE_ROW);
    // A searchable popup: a double click picks the row.
    term.double_click(c + 2, r)
        .expect("double-click the #12 row");
    term.send_text("go").unwrap();
    term.wait_for("#12 in the Summary, the picker closed", STEP, |s| {
        s.contains("#12 go") && s.find_all(POPUP_ISSUE_ROW).len() == 1
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.screen().text()));
    // Whatever lay under the popup row was not activated.
    assert!(
        opened_urls(&config).is_empty(),
        "the click reached a row behind the popup"
    );
}

#[test]
fn unlinked_the_button_and_the_palette_say_why() {
    let mock = start_mock();
    let repo = branch_fixture(mock.port);
    let (mut term, _config) = launch_unlinked(
        &repo,
        "pty-hosting-composer-ref-unlinked",
        &config_toml(mock.port, ""),
        COLS,
        "// WIP",
    );
    focus_summary(&mut term);
    let (col, row) = term.screen().find("#\u{25BE}").unwrap();
    term.click(col, row).expect("click the grayed # button");
    term.wait_for("the reason, and no picker", STEP, |s| {
        s.contains("link an issue list in the sidebar first") && !s.contains(POPUP_BRANCH_ROW)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.screen().text()));
    term.send_keys("C-p").unwrap();
    term.send_text("Insert Issue Reference").unwrap();
    term.wait_for("the palette entry names the reason", STEP, |s| {
        s.contains("Commit: Insert Issue Reference") && s.contains("link an issue list")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.screen().text()));
}
