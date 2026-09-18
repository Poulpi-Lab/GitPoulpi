//! `ui.graph_ghost_ref`: hovering a commit with no ref of its own shows a
//! faded pill naming the branch it belongs to in the Ref column, with a
//! tooltip; moving away removes it, clicking it does nothing special, and
//! with the setting off nothing is drawn. A commit whose branch was deleted
//! after a `--no-ff` merge still gets a ghost of that branch, read from the
//! merge commit's subject, with a "deleted" tooltip.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(30);

fn fixture() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit("older target commit")
        .unwrap()
        .commit("newer head commit")
        .unwrap()
        .build()
}

fn branch_of(repo: &FixtureRepo) -> String {
    repo.git(&["branch", "--show-current"]).trim().to_string()
}

fn launch(repo: &FixtureRepo, session: &str, config: &str) -> (Terminal, tempfile::TempDir) {
    common::launch(repo.root())
        .session(session)
        .config(config)
        .size(200, 50)
        .ready_when("the fixture loaded", |s| {
            s.contains("older target commit") && s.contains("newer head commit")
        })
        .mouse_capture()
        .start()
}

/// The column where `needle` starts on `line`, counted in characters.
fn column_of(line: &str, needle: &str) -> u16 {
    line[..line.find(needle).unwrap()].chars().count() as u16
}

#[test]
fn hovering_a_ref_less_commit_names_its_branch_faded_and_leaving_removes_it() {
    let repo = fixture();
    let branch = branch_of(&repo);
    let (mut term, _cfg) = launch(&repo, "pty-ghost-ref", "");
    let (col, row) = term.wait_for_text("older target commit", STEP).unwrap();
    assert!(
        !term.screen().line(row).contains(&branch),
        "no ghost before the hover"
    );
    term.move_to(col + 1, row).unwrap();
    term.wait_for("the ghost on the hovered row", STEP, |s| {
        s.line(row).contains(&branch)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(
        !term.screen().line(row).contains("\u{2387} +"),
        "no create-branch glyph"
    );
    // The pointer leaves the Log: the ghost goes.
    term.move_to(1, 1).unwrap();
    term.wait_for("the ghost gone", STEP, |s| !s.line(row).contains(&branch))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

#[test]
fn the_ghost_has_a_tooltip_and_a_click_on_it_opens_no_branch_box() {
    let repo = fixture();
    let branch = branch_of(&repo);
    let (mut term, _cfg) = launch(&repo, "pty-ghost-tip", "");
    let (col, row) = term.wait_for_text("older target commit", STEP).unwrap();
    term.move_to(col + 1, row).unwrap();
    let screen = term
        .wait_for("the ghost on the hovered row", STEP, |s| {
            s.line(row).contains(&branch)
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let gx = column_of(&screen.line(row), &branch);
    term.move_to(gx, row).unwrap();
    term.wait_for("the ghost's tooltip", STEP, |s| {
        s.contains(&format!("{branch} (Local Branch)"))
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.click(gx, row).unwrap();
    std::thread::sleep(Duration::from_millis(800));
    assert!(
        !term.text().contains("enter branch name"),
        "a click on the ghost creates nothing:\n{}",
        term.text()
    );
    term.stop().unwrap();
}

#[test]
fn with_the_setting_off_no_ghost_is_drawn() {
    let repo = fixture();
    let branch = branch_of(&repo);
    let (mut term, _cfg) = launch(&repo, "pty-ghost-off", "[ui]\ngraph_ghost_ref = false\n");
    let (col, row) = term.wait_for_text("older target commit", STEP).unwrap();
    term.move_to(col + 1, row).unwrap();
    std::thread::sleep(Duration::from_millis(800));
    assert!(
        !term.screen().line(row).contains(&branch),
        "no ghost when off:\n{}",
        term.text()
    );
    term.stop().unwrap();
}

/// `feature` branched off, merged with `--no-ff`, then deleted.
fn merged_and_deleted() -> FixtureRepo {
    let repo = fixture();
    let trunk = branch_of(&repo);
    repo.git(&["checkout", "-q", "-b", "feature"]);
    repo.git(&["commit", "-q", "--allow-empty", "-m", "lonely side work"]);
    repo.git(&["checkout", "-q", &trunk]);
    repo.git(&["commit", "-q", "--allow-empty", "-m", "trunk moved on"]);
    repo.git(&[
        "merge",
        "-q",
        "--no-ff",
        "feature",
        "-m",
        "Merge branch 'feature'",
    ]);
    repo.git(&["branch", "-q", "-D", "feature"]);
    repo
}

#[test]
fn a_commit_of_a_deleted_branch_shows_its_ghost_and_a_deleted_tooltip() {
    let repo = merged_and_deleted();
    let (mut term, _cfg) = common::launch(repo.root())
        .session("pty-ghost-deleted")
        .config("")
        .size(200, 50)
        .ready_when("the fixture loaded", |s| s.contains("lonely side work"))
        .mouse_capture()
        .start();
    let (col, row) = term.wait_for_text("lonely side work", STEP).unwrap();
    assert!(
        !term.screen().line(row).contains("feature "),
        "no ghost before the hover"
    );
    term.move_to(col + 1, row).unwrap();
    let screen = term
        .wait_for("the deleted branch's ghost", STEP, |s| {
            s.line(row).contains("feature")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let gx = column_of(&screen.line(row), "feature");
    term.move_to(gx, row).unwrap();
    term.wait_for("the deleted tooltip", STEP, |s| {
        s.contains("Branch feature (deleted), via merge ")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

#[test]
fn a_fast_forwarded_branch_leaves_no_ghost() {
    let repo = fixture();
    let trunk = branch_of(&repo);
    repo.git(&["checkout", "-q", "-b", "feature"]);
    repo.git(&["commit", "-q", "--allow-empty", "-m", "ff only work"]);
    repo.git(&["checkout", "-q", &trunk]);
    repo.git(&["merge", "-q", "--ff-only", "feature"]);
    repo.git(&["branch", "-q", "-D", "feature"]);
    let (mut term, _cfg) = common::launch(repo.root())
        .session("pty-ghost-ff")
        .config("")
        .size(200, 50)
        .ready_when("the fixture loaded", |s| s.contains("older target commit"))
        .mouse_capture()
        .start();
    let (col, row) = term.wait_for_text("older target commit", STEP).unwrap();
    term.move_to(col + 1, row).unwrap();
    // The trunk still explains the older commits; the fast-forwarded commit
    // is on the trunk too, so "feature" appears nowhere.
    let (col, row) = term.wait_for_text("ff only work", STEP).unwrap();
    term.move_to(col + 1, row).unwrap();
    std::thread::sleep(Duration::from_millis(800));
    assert!(
        !term.screen().line(row).contains("feature"),
        "no inferred branch for a fast-forward:\n{}",
        term.text()
    );
    term.stop().unwrap();
}
