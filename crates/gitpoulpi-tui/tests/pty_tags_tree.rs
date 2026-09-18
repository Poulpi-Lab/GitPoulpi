//! The TAGS section groups `/`-separated tag names into folders like the
//! branch sections do (`releases/v1.0.0` is the leaf `v1.0.0` under the
//! folder `releases`), driven through a pty by `xtask::pty`.
//!
//! ```text
//! ▾ 🏷 TAGS (4)
//!   ▾ 📁 backups/
//!         a
//!       plain
//!   ▾ 📁 releases/
//!         v1.0.0
//!         v1.1.0
//! ```

mod common;

use common::git_trimmed as git;

use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::{Color, Screen, Terminal};

const STEP: Duration = Duration::from_secs(20);
const SIDEBAR_RIGHT: u16 = 40;
/// `SELECTION_BG` in `gitpoulpi_tui`.
const SELECTION_BG: Color = Color::Rgb(52, 58, 90);

fn fixture() -> gitpoulpi_testkit::FixtureRepo {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "initial")
        .unwrap()
        .commit_file("b.txt", "two\n", "second")
        .unwrap()
        .build();
    for tag in ["releases/v1.0.0", "releases/v1.1.0", "backups/a", "plain"] {
        git(repo.root(), &["tag", tag]);
    }
    repo
}

/// Where `label` is drawn in the sidebar below the `TAGS (` header.
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

fn expect_tag_label(term: &Terminal, label: &str) -> (u16, u16) {
    let screen = term
        .wait_for(&format!("{label:?} under TAGS"), STEP, |s| {
            tag_label(s, label).is_some()
        })
        .unwrap_or_else(|e| panic!("{e}"));
    tag_label(&screen, label).expect("just waited for it")
}

fn launch(repo: &std::path::Path, session: &str, config: &std::path::Path) -> Terminal {
    common::launch(repo)
        .session(session)
        .config_dir(config)
        .size(200, 50)
        .ready_when("the fixture's log and the tag folders", |s| {
            s.contains("second") && tag_label(s, "releases/").is_some()
        })
        .spawn()
}

#[test]
fn tag_folders_collapse_with_a_click_expand_with_the_keyboard_and_the_filter_finds_a_leaf() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch(repo.root(), "tags-tree", config.path());

    // Folders, then their leaves by last segment only.
    expect_tag_label(&term, "backups/");
    expect_tag_label(&term, "v1.0.0");
    expect_tag_label(&term, "plain");
    assert!(tag_label(&term.screen(), "releases/v1.0.0").is_none());

    // A click on the folder row folds it.
    let (col, row) = expect_tag_label(&term, "releases/");
    term.click(col + 1, row).unwrap();
    term.wait_for("releases folded", STEP, |s| {
        tag_label(s, "v1.0.0").is_none() && tag_label(s, "releases/").is_some()
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.stop().unwrap();

    // Persisted per repository, like branch folders.
    let state = std::fs::read_to_string(repo.state_dir().join("repo-state.toml"))
        .expect("repo-state.toml written");
    assert!(state.contains("collapsed_tag_ref_folders"), "{state}");

    let mut term = launch(repo.root(), "tags-tree-2", config.path());
    assert!(
        tag_label(&term.screen(), "v1.0.0").is_none(),
        "folded after a restart"
    );

    // Select the folder row with a click (which toggles it open), then the
    // keyboard twin: Enter folds, Enter unfolds.
    let (col, row) = expect_tag_label(&term, "releases/");
    term.click(col + 1, row).unwrap();
    term.wait_for("releases unfolded by the click", STEP, |s| {
        tag_label(s, "v1.0.0").is_some()
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    term.wait_for("releases folded by Enter", STEP, |s| {
        tag_label(s, "v1.0.0").is_none()
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    term.wait_for("releases expanded by Enter", STEP, |s| {
        tag_label(s, "v1.0.0").is_some() && tag_label(s, "v1.1.0").is_some()
    })
    .unwrap_or_else(|e| panic!("{e}"));

    // The filter matches the full name and narrows to that leaf's folder.
    term.send_keys("/").unwrap();
    term.send_text("v1.1").unwrap();
    term.send_keys("Enter").unwrap();
    term.wait_for("only v1.1.0 left", STEP, |s| {
        tag_label(s, "v1.1.0").is_some()
            && tag_label(s, "v1.0.0").is_none()
            && tag_label(s, "plain").is_none()
    })
    .unwrap_or_else(|e| panic!("{e}"));
}

#[test]
fn the_filter_opens_a_folded_tag_folder_and_a_click_selects_a_tag_inside_it() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch(repo.root(), "tags-tree-filter", config.path());

    let (col, row) = expect_tag_label(&term, "releases/");
    term.click(col + 1, row).unwrap();
    term.wait_for("releases folded", STEP, |s| {
        tag_label(s, "v1.1.0").is_none() && tag_label(s, "releases/").is_some()
    })
    .unwrap_or_else(|e| panic!("{e}"));

    // The committed filter opens the folder holding its match.
    term.send_keys("/").unwrap();
    term.send_text("v1.1").unwrap();
    term.send_keys("Enter").unwrap();
    term.wait_for("the folded folder opened by the filter", STEP, |s| {
        tag_label(s, "v1.1.0").is_some() && tag_label(s, "v1.0.0").is_none()
    })
    .unwrap_or_else(|e| panic!("{e}"));

    // A click on a tag inside the folder selects that row.
    let (col, row) = expect_tag_label(&term, "v1.1.0");
    term.click(col + 1, row).unwrap();
    term.wait_for("the tag row selected", STEP, |s| {
        s.cell(col, row).is_some_and(|c| c.bg == SELECTION_BG)
    })
    .unwrap_or_else(|e| panic!("{e}"));
}
