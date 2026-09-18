//! Expand all / collapse all on every tree of the app, end to end through a
//! pty: the TAGS section of the Refs pane (a section menu, a folder menu, the
//! keys, the palette rows, the name filter) and the right pane's file lists
//! (Unstaged, the commit's changed files).
//!
//! The filter rule under test: while a ref filter is active every folder that
//! holds a match is shown open, whatever the user had folded, and the user's
//! own folds are neither changed nor saved; clearing the filter brings them
//! back. Folds made during a filter act on the visible tree and are forgotten
//! with it.

mod common;

use common::git_trimmed as git;

use std::path::Path;
use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::{Screen, Terminal};

const STEP: Duration = Duration::from_secs(20);
const SIDEBAR_RIGHT: u16 = 40;

/// Tags: two nested folders and two loose ones.
///
/// ```text
/// ▾ TAGS (5)
///   ▾ backups/
///         old
///       plain
///   ▾ releases/
///     ▾ rc/
///           rc1
///       v1.0.0
/// ```
fn tag_fixture() -> gitpoulpi_testkit::FixtureRepo {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "initial")
        .unwrap()
        .commit_file("b.txt", "two\n", "second")
        .unwrap()
        .build();
    for tag in ["releases/v1.0.0", "releases/rc/rc1", "backups/old", "plain"] {
        git(repo.root(), &["tag", tag]);
    }
    repo
}

fn tag_label(screen: &Screen, label: &str) -> Option<(u16, u16)> {
    let header = screen
        .find_all("TAGS (")
        .into_iter()
        .find(|(c, _)| *c < SIDEBAR_RIGHT)?
        .1;
    screen
        .find_all(label)
        .into_iter()
        .filter(|(c, r)| *c < SIDEBAR_RIGHT && *r > header)
        .min_by_key(|(_, r)| *r)
}

fn expect_tag(term: &Terminal, label: &str) -> (u16, u16) {
    let screen = term
        .wait_for(&format!("{label:?} under TAGS"), STEP, |s| {
            tag_label(s, label).is_some()
        })
        .unwrap_or_else(|e| panic!("{e}"));
    tag_label(&screen, label).expect("just waited for it")
}

fn tags_header(term: &Terminal) -> (u16, u16) {
    term.screen()
        .find_all("TAGS (")
        .into_iter()
        .find(|(c, _)| *c < SIDEBAR_RIGHT)
        .expect("the TAGS header")
}

fn launch_tags(repo: &Path, session: &str, config: &Path) -> Terminal {
    common::launch(repo)
        .session(session)
        .config_dir(config)
        .size(200, 50)
        .ready_when("the log and the tag folders", |s| {
            s.contains("second")
                && tag_label(s, "releases/").is_some()
                && tag_label(s, "rc1").is_some()
        })
        .spawn()
}

fn all_open(s: &Screen) -> bool {
    ["old", "plain", "rc/", "rc1", "v1.0.0"]
        .iter()
        .all(|l| tag_label(s, l).is_some())
}

fn all_folded(s: &Screen) -> bool {
    tag_label(s, "releases/").is_some()
        && tag_label(s, "backups/").is_some()
        && tag_label(s, "plain").is_some()
        && ["old", "rc/", "rc1", "v1.0.0"]
            .iter()
            .all(|l| tag_label(s, l).is_none())
}

/// The menu row `label` (not the list header's own "Expand all" button).
fn menu_row(s: &Screen, label: &str) -> Option<(u16, u16)> {
    s.find_all(label)
        .into_iter()
        .find(|(_, row)| !s.line(*row).contains("Tree [m]"))
}

fn click_menu_row(term: &mut Terminal, label: &str) {
    let screen = term
        .wait_for(&format!("a menu row {label:?}"), STEP, |s| {
            menu_row(s, label).is_some()
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (col, row) = menu_row(&screen, label).unwrap();
    term.click(col + 1, row).unwrap();
}

fn saved_tag_folds(repo: &gitpoulpi_testkit::FixtureRepo) -> String {
    std::fs::read_to_string(repo.state_dir().join("repo-state.toml")).unwrap_or_default()
}

fn wait_saved(repo: &gitpoulpi_testkit::FixtureRepo, what: &str, ok: impl Fn(&str) -> bool) {
    let deadline = std::time::Instant::now() + STEP;
    loop {
        let state = saved_tag_folds(repo);
        if ok(&state) {
            return;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "timed out waiting for {what}\n{state}"
        );
        std::thread::sleep(Duration::from_millis(100));
    }
}

fn palette(term: &mut Terminal, query: &str, entry: &str) {
    term.send_keys("C-p").unwrap();
    term.send_text(query).unwrap();
    term.wait_for("the palette entry", STEP, |s| s.contains(entry))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
}

#[test]
fn the_section_menu_collapses_and_expands_every_folder_of_the_section() {
    let repo = tag_fixture();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch_tags(repo.root(), "tree-fold-section", config.path());
    assert!(all_open(&term.screen()));

    // The header's menu: the two fold rows, then the way on to the checklist.
    let (col, row) = tags_header(&term);
    term.right_click(col + 2, row).unwrap();
    term.wait_for("the section menu", STEP, |s| {
        s.contains("Expand all")
            && s.contains("Collapse all")
            && s.contains("Show/hide sections...")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    click_menu_row(&mut term, "Collapse all");
    term.wait_for("every tag folder folded", STEP, all_folded)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // Saved per repository, nested folder included.
    wait_saved(&repo, "the folds on disk", |s| {
        s.contains("collapsed_tag_ref_folders") && s.contains("releases/rc")
    });

    let (col, row) = tags_header(&term);
    term.right_click(col + 2, row).unwrap();
    click_menu_row(&mut term, "Expand all");
    term.wait_for("every tag folder open", STEP, all_open)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    wait_saved(&repo, "the folds cleared on disk", |s| {
        !s.contains("releases/rc")
    });
}

#[test]
fn the_section_menu_leads_on_to_the_sections_checklist() {
    let repo = tag_fixture();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch_tags(repo.root(), "tree-fold-checklist", config.path());
    let (col, row) = tags_header(&term);
    term.right_click(col + 2, row).unwrap();
    click_menu_row(&mut term, "Show/hide sections...");
    term.wait_for("the checklist", STEP, |s| {
        s.contains("Use for all repositories") || s.contains("Reset to default")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

#[test]
fn a_folder_menu_folds_that_folder_and_its_subfolders_only() {
    let repo = tag_fixture();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch_tags(repo.root(), "tree-fold-below", config.path());

    let (col, row) = expect_tag(&term, "releases/");
    term.right_click(col + 2, row).unwrap();
    term.wait_for("the folder menu", STEP, |s| {
        s.contains("Expand all subfolders") && s.contains("Collapse all subfolders")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    click_menu_row(&mut term, "Collapse all subfolders");
    term.wait_for("releases folded, backups untouched", STEP, |s| {
        tag_label(s, "releases/").is_some()
            && tag_label(s, "rc1").is_none()
            && tag_label(s, "v1.0.0").is_none()
            && tag_label(s, "old").is_some()
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // The subfolder was folded too: opening `releases/` alone shows `rc/`
    // still closed.
    let (col, row) = expect_tag(&term, "releases/");
    term.click(col + 1, row).unwrap();
    term.wait_for("releases open, rc closed", STEP, |s| {
        tag_label(s, "v1.0.0").is_some()
            && tag_label(s, "rc/").is_some()
            && tag_label(s, "rc1").is_none()
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    let (col, row) = expect_tag(&term, "releases/");
    term.right_click(col + 2, row).unwrap();
    click_menu_row(&mut term, "Expand all subfolders");
    term.wait_for("rc opened by the subtree expand", STEP, |s| {
        tag_label(s, "rc1").is_some()
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

#[test]
fn the_keys_fold_the_cursor_folder_and_the_whole_section() {
    let repo = tag_fixture();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch_tags(repo.root(), "tree-fold-keys", config.path());

    // Put the cursor on `releases/` (a click folds it, a second one opens it).
    let (col, row) = expect_tag(&term, "releases/");
    term.click(col + 1, row).unwrap();
    term.wait_for("releases folded", STEP, |s| {
        tag_label(s, "v1.0.0").is_none()
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    term.wait_for("releases open", STEP, |s| tag_label(s, "v1.0.0").is_some())
        .unwrap_or_else(|e| panic!("{e}"));

    // `_` collapses it and everything below, `*` opens it all again.
    term.send_keys("_").unwrap();
    term.wait_for("releases and rc folded", STEP, |s| {
        tag_label(s, "v1.0.0").is_none() && tag_label(s, "old").is_some()
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("*").unwrap();
    term.wait_for("releases and rc open", STEP, |s| {
        tag_label(s, "rc1").is_some() && tag_label(s, "v1.0.0").is_some()
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // `-` folds every folder of the section, `=` opens them all.
    term.send_keys("-").unwrap();
    term.wait_for("everything folded", STEP, all_folded)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("=").unwrap();
    term.wait_for("everything open", STEP, all_open)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

#[test]
fn the_palette_rows_fold_the_active_section() {
    let repo = tag_fixture();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch_tags(repo.root(), "tree-fold-palette", config.path());
    // The TAGS section is the active one once a row of it was clicked.
    let (col, row) = expect_tag(&term, "plain");
    term.click(col + 1, row).unwrap();

    palette(
        &mut term,
        "Refs: Collapse All Folders",
        "Refs: Collapse All Folders",
    );
    term.wait_for("everything folded", STEP, all_folded)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    palette(
        &mut term,
        "Refs: Expand All Folders",
        "Refs: Expand All Folders",
    );
    term.wait_for("everything open", STEP, all_open)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

/// The maintainer's question: how do folds behave under a name filter?
#[test]
fn a_filter_opens_the_folders_with_a_match_without_touching_the_saved_folds() {
    let repo = tag_fixture();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch_tags(repo.root(), "tree-fold-filter", config.path());

    // The user folds `releases/` (saved).
    let (col, row) = expect_tag(&term, "releases/");
    term.click(col + 1, row).unwrap();
    term.wait_for("releases folded", STEP, |s| tag_label(s, "rc1").is_none())
        .unwrap_or_else(|e| panic!("{e}"));
    wait_saved(&repo, "releases saved as folded", |s| {
        s.contains("collapsed_tag_ref_folders") && s.contains("\"releases\"")
    });

    // A filter whose match lives in the folded folder shows it open.
    term.send_keys("/").unwrap();
    term.send_text("rc1").unwrap();
    term.send_keys("Enter").unwrap();
    term.wait_for("the match shown inside the folded folder", STEP, |s| {
        tag_label(s, "rc1").is_some() && tag_label(s, "v1.0.0").is_none()
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // The saved state still says folded: nothing the filter opened is saved.
    std::thread::sleep(Duration::from_millis(500));
    assert!(
        saved_tag_folds(&repo).contains("\"releases\""),
        "{}",
        saved_tag_folds(&repo)
    );

    // A fold made during the filter acts on the visible tree...
    term.send_keys("-").unwrap();
    term.wait_for("the visible tree folded", STEP, |s| {
        tag_label(s, "rc1").is_none() && tag_label(s, "releases/").is_some()
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(
        saved_tag_folds(&repo).contains("\"releases\""),
        "the saved folds are untouched by a fold during a filter"
    );

    // ...and clearing the filter brings the user's own folds back: `releases`
    // folded, nothing else (the fold above is forgotten with the filter).
    term.send_keys("Escape").unwrap();
    term.wait_for("the user's folds back after the filter", STEP, |s| {
        tag_label(s, "releases/").is_some()
            && tag_label(s, "backups/").is_some()
            && tag_label(s, "old").is_some()
            && tag_label(s, "rc1").is_none()
            && tag_label(s, "v1.0.0").is_none()
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (col, row) = expect_tag(&term, "releases/");
    term.click(col + 1, row).unwrap();
    term.wait_for("releases open: rc was never folded", STEP, |s| {
        tag_label(s, "rc1").is_some() && tag_label(s, "v1.0.0").is_some()
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

// -- the right pane's file lists ------------------------------------------

/// Unstaged changes in two levels of folders, over a commit that adds them.
///
/// ```text
/// ▾ docs/
///     z.txt
/// ▾ src/
///   ▾ a/
///       x.txt
///   ▾ b/
///       y.txt
/// ```
fn files_fixture() -> gitpoulpi_testkit::FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("src/a/x.txt", "1\n", "add nested")
        .unwrap()
        .commit_file("src/b/y.txt", "2\n", "add y")
        .unwrap()
        .commit_file("docs/z.txt", "3\n", "add z")
        .unwrap()
        .write_worktree("src/a/x.txt", b"1 changed\n")
        .unwrap()
        .write_worktree("src/b/y.txt", b"2 changed\n")
        .unwrap()
        .write_worktree("docs/z.txt", b"3 changed\n")
        .unwrap()
        .build()
}

fn launch_files(repo: &Path, session: &str, config: &Path) -> Terminal {
    common::launch(repo)
        .session(session)
        .config_dir(config)
        .size(200, 50)
        .no_exit_pause()
        .ready_when("the fixture's log", |s| s.contains("add z"))
        .spawn()
}

/// Selects the WIP row so the Status pane lists the changes as a tree.
fn show_changes(term: &mut Terminal) {
    let (col, row) = term
        .wait_for_text("\u{25CC}", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(col, row).unwrap();
    term.wait_for("the folder rows", STEP, |s| {
        s.contains("x.txt") && s.contains("y.txt") && s.contains("z.txt")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

/// Every file row hidden, the two top folders still there.
fn files_folded(s: &Screen) -> bool {
    s.contains("src/")
        && s.contains("docs/")
        && !s.contains("x.txt")
        && !s.contains("y.txt")
        && !s.contains("z.txt")
}

#[test]
fn the_unstaged_list_menus_and_keys_fold_its_tree() {
    let repo = files_fixture();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch_files(repo.root(), "tree-fold-files", config.path());
    show_changes(&mut term);

    // The list's empty area (under the last row; the pane hugs its rows, so
    // the spot below is its border).
    let (col, row) = term
        .wait_for_text("z.txt", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    let last_row = [("x.txt", 0u16), ("y.txt", 0), ("z.txt", 0)]
        .iter()
        .filter_map(|(l, _)| term.screen().find(l).map(|(_, r)| r))
        .max()
        .unwrap_or(row);
    term.right_click(col + 1, last_row + 1).unwrap();
    term.wait_for("the empty-area menu with its fold rows", STEP, |s| {
        menu_row(s, "Expand all").is_some() && menu_row(s, "Collapse all").is_some()
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    click_menu_row(&mut term, "Collapse all");
    term.wait_for("every folder of the list folded", STEP, files_folded)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // The same keys as the refs trees, in the focused list: Tab goes from the
    // Log to the Status pane.
    term.send_keys("Tab").unwrap();
    term.send_keys("=").unwrap();
    term.wait_for("every folder open (=)", STEP, |s| {
        s.contains("x.txt") && s.contains("y.txt") && s.contains("z.txt")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("-").unwrap();
    term.wait_for("every folder folded (-)", STEP, files_folded)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // And the palette rows.
    palette(
        &mut term,
        "Files: Expand All Folders",
        "Files: Expand All Folders",
    );
    term.wait_for("every folder open (palette)", STEP, |s| {
        s.contains("x.txt") && s.contains("z.txt")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

#[test]
fn an_unstaged_folder_menu_folds_below_that_folder_only() {
    let repo = files_fixture();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch_files(repo.root(), "tree-fold-files-below", config.path());
    show_changes(&mut term);

    let (col, row) = term
        .wait_for_text("\u{25BE} src/", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.right_click(col + 3, row).unwrap();
    term.wait_for("the folder menu with its fold rows", STEP, |s| {
        s.contains("Expand all subfolders") && s.contains("Collapse all subfolders")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    click_menu_row(&mut term, "Collapse all subfolders");
    term.wait_for("src folded, docs untouched", STEP, |s| {
        s.contains("src/") && !s.contains("x.txt") && !s.contains("y.txt") && s.contains("z.txt")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // Opening `src/` alone shows `a/` and `b/` still closed.
    let (col, row) = term
        .wait_for_text("\u{25B8} src/", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(col + 1, row).unwrap();
    term.wait_for("src open, a/ and b/ still closed", STEP, |s| {
        s.contains("a/") && s.contains("b/") && !s.contains("x.txt") && !s.contains("y.txt")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    let (col, row) = term
        .wait_for_text("\u{25BE} src/", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.right_click(col + 3, row).unwrap();
    click_menu_row(&mut term, "Expand all subfolders");
    term.wait_for("everything under src open", STEP, |s| {
        s.contains("x.txt") && s.contains("y.txt")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

#[test]
fn the_commit_files_folder_menu_folds_below_that_folder() {
    let repo = files_fixture();
    // One commit with the nested files, selected in the Log.
    git(repo.root(), &["checkout", "-q", "--orphan", "nested"]);
    git(repo.root(), &["add", "-A"]);
    git(
        repo.root(),
        &["commit", "-q", "-m", "nested-all", "--allow-empty"],
    );
    let config = tempfile::tempdir().unwrap();
    let mut term = common::launch(repo.root())
        .session("tree-fold-commit")
        .config_dir(config.path())
        .size(200, 50)
        .no_exit_pause()
        .ready_when("the nested commit", |s| s.contains("nested-all"))
        .spawn();
    let (col, row) = term
        .wait_for_text("nested-all", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(col + 1, row).unwrap();
    term.wait_for("the commit's file tree", STEP, |s| {
        s.contains("x.txt") && s.contains("y.txt") && s.contains("z.txt")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // The commit's files open as a flat list: switch to the tree.
    let (col, row) = term
        .wait_for_text("Tree [m]", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(col + 1, row).unwrap();

    let (col, row) = term
        .wait_for_text("\u{25BE} src/", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.right_click(col + 3, row).unwrap();
    click_menu_row(&mut term, "Collapse all subfolders");
    term.wait_for("src folded in the commit's files, docs open", STEP, |s| {
        !s.contains("x.txt") && !s.contains("y.txt") && s.contains("z.txt")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

/// The commit's files filter follows the same rule as the refs filter: a
/// match inside a folded folder is shown, and the folder is folded again once
/// the filter is cleared.
#[test]
fn the_commit_files_filter_shows_a_match_inside_a_folded_folder_and_gives_the_fold_back() {
    let repo = files_fixture();
    git(repo.root(), &["checkout", "-q", "--orphan", "nested"]);
    git(repo.root(), &["add", "-A"]);
    git(
        repo.root(),
        &["commit", "-q", "-m", "nested-all", "--allow-empty"],
    );
    let config = tempfile::tempdir().unwrap();
    let mut term = common::launch(repo.root())
        .session("tree-fold-commit-filter")
        .config_dir(config.path())
        .size(200, 50)
        .no_exit_pause()
        .ready_when("the nested commit", |s| s.contains("nested-all"))
        .spawn();
    let (col, row) = term
        .wait_for_text("nested-all", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(col + 1, row).unwrap();
    let (col, row) = term
        .wait_for_text("Tree [m]", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(col + 1, row).unwrap();
    let (col, row) = term
        .wait_for_text("\u{25BE} src/", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.right_click(col + 3, row).unwrap();
    click_menu_row(&mut term, "Collapse all subfolders");
    term.wait_for("src folded", STEP, |s| {
        !s.contains("x.txt") && s.contains("z.txt")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // Filter on `x.txt`: shown although `src/` is folded.
    let (col, row) = term
        .wait_for_text("Filter files:", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(col + 14, row).unwrap();
    term.send_text("x.txt").unwrap();
    // (`x.txt` is also on the filter field's own line: look for a file row.)
    term.wait_for("the match inside the folded folder", STEP, |s| {
        let row = |name: &str| {
            s.find_all(name)
                .into_iter()
                .any(|(_, r)| !s.line(r).contains("Filter files:"))
        };
        row("x.txt") && !row("z.txt")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // Cleared: `src/` is folded again, `docs/` open.
    for _ in 0..5 {
        term.send_keys("Backspace").unwrap();
    }
    term.wait_for("the fold back after the filter", STEP, |s| {
        !s.contains("x.txt") && s.contains("z.txt") && s.contains("src/")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}
