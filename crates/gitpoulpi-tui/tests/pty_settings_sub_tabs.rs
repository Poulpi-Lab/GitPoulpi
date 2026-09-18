//! The sub-tabs of the master-detail Settings pages, end to end through a pty:
//! every page opens on its General tab (the page's own settings, no list), the
//! list tab holds the list and the selected item's rows, a click and
//! Alt+Left / Alt+Right switch, and a search result lands on the tab its row
//! lives on. Temp config dir only.

mod common;

use std::path::Path;
use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(30);

fn fixture() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "initial commit")
        .unwrap()
        .build()
}

fn launch(repo: &Path, config: &Path, session: &str) -> Terminal {
    common::launch(repo)
        .session(session)
        .config_dir(config)
        .size(200, 60)
        .no_exit_pause()
        .ready_when("the log", |s| s.contains("Message"))
        .mouse_capture()
        .spawn()
}

fn open_settings(term: &mut Terminal) {
    let (c, r) = term
        .wait_for_text("\u{2699}", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(c, r).expect("click the gear");
    term.wait_for("Settings open", STEP, |s| {
        s.contains("Repo-specific settings")
    })
    .unwrap_or_else(|e| panic!("{e}"));
}

/// (sidebar title, list tab label, a row only the page owns, a row only the
/// selected item owns; `a|b` is either).
const PAGES: [(&str, &str, &str, &str); 5] = [
    ("Profiles", "Profiles", "Active profile", "No profiles yet"),
    ("Open in", "Entries", "Default entry", "Editing entry"),
    (
        "Images",
        "Editors",
        "Preview images",
        "Editing entry|No entries yet",
    ),
    ("Workflow", "Rules", "Workflow template", "No rules yet"),
    (
        "AI",
        "Profiles (connections)",
        "Enable AI",
        "Provider preset",
    ),
];

fn shows(s: &xtask::pty::Screen, any_of: &str) -> bool {
    any_of.split('|').any(|t| s.contains(t))
}

fn wait_for_row(term: &Terminal, what: &str, want: &str, absent: &str) {
    term.wait_for(what, STEP, |s| s.contains(want) && !s.contains(absent))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

#[test]
fn each_page_keeps_its_own_settings_and_its_list_on_separate_tabs() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch(repo.root(), config.path(), "pty-sub-tabs");
    open_settings(&mut term);
    for (page, list_tab, global, item) in PAGES {
        common::settings_page(&mut term, page);
        // Opens on General: the page's settings, no list.
        wait_for_row(&term, &format!("{page}: General"), global, "Remove");
        assert!(
            !shows(&term.screen(), item),
            "{page}: {item}\n{}",
            term.text()
        );

        // A click on the list tab: the list and the item's rows.
        common::settings_tab(&mut term, list_tab);
        term.wait_for(&format!("{page}: the list"), STEP, |s| {
            s.contains("+ Add") && shows(s, item)
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
        assert!(
            !term.screen().contains(global),
            "{page}: {global}\n{}",
            term.text()
        );

        // Alt+Left is the twin of the General tab, Alt+Right of the list.
        term.send_keys("M-Left").unwrap();
        wait_for_row(&term, &format!("{page}: Alt+Left"), global, "Remove");
        term.send_keys("M-Right").unwrap();
        term.wait_for(&format!("{page}: Alt+Right"), STEP, |s| {
            s.contains("+ Add") && shows(s, item)
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

        // Back on General for the next page's "opens on General" check.
        common::settings_tab(&mut term, "General");
        wait_for_row(&term, &format!("{page}: General again"), global, "Remove");
    }
}

#[test]
fn a_search_result_lands_on_the_tab_of_its_row() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch(repo.root(), config.path(), "pty-sub-tabs-search");
    open_settings(&mut term);
    common::settings_page(&mut term, "Open in");
    common::settings_tab(&mut term, "Entries");
    term.wait_for("the list tab", STEP, |s| s.contains("+ Add"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // A General row, searched from the list tab.
    term.send_keys("C-f").unwrap();
    term.wait_for("the search box", STEP, |s| s.contains("Search"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_text("Default entry").unwrap();
    term.wait_for("a hit", STEP, |s| s.contains("Open in"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    wait_for_row(&term, "landed on General", "Default entry", "+ Add");
}

/// Keyboard & mouse: the mouse and keyboard rows are the first tab, the
/// keybindings editor has its own. A click and Alt+Left / Alt+Right switch,
/// and the settings search finds the tab by name.
#[test]
fn the_keybindings_have_their_own_tab_on_keyboard_and_mouse() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch(repo.root(), config.path(), "pty-sub-tabs-keys");
    open_settings(&mut term);
    common::settings_page(&mut term, "Keyboard & mouse");
    // Opens on the first tab: the rows, not the editor.
    wait_for_row(&term, "rows first", "Mouse enabled", "Command Palette");

    common::settings_tab(&mut term, "Keybindings");
    wait_for_row(&term, "the editor", "Command Palette", "Mouse enabled");

    term.send_keys("M-Left").unwrap();
    wait_for_row(&term, "Alt+Left", "Mouse enabled", "Command Palette");
    term.send_keys("M-Right").unwrap();
    wait_for_row(&term, "Alt+Right", "Command Palette", "Mouse enabled");

    // The search finds the tab from another page.
    common::settings_page(&mut term, "General");
    term.send_keys("C-f").unwrap();
    term.wait_for("the search box", STEP, |s| s.contains("Search"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_text("Keybindings").unwrap();
    term.wait_for("a hit", STEP, |s| s.contains("Keyboard & mouse"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    wait_for_row(
        &term,
        "landed on the editor",
        "Command Palette",
        "Mouse enabled",
    );
}
