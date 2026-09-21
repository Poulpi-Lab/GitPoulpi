//! `general.group_worktree_tabs`: the tabs of a repository's linked worktrees
//! sit right after that repository's tab - at launch, and a worktree tab
//! dragged away snaps back. Off, tabs stay where they are put.

mod common;

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use gitpoulpi_session::GlobalSession;
use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(30);
const SESSION: &str = "pty-worktree-group";

fn repo(name: &str, subject: &str) -> (gitpoulpi_testkit::FixtureRepo, PathBuf) {
    let fx = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "x\n", subject)
        .unwrap()
        .build();
    let root = fx.root().parent().unwrap().join(name);
    std::fs::rename(fx.root(), &root).unwrap();
    (fx, root)
}

struct Fixture {
    _keep: Vec<gitpoulpi_testkit::FixtureRepo>,
    _wt_dir: tempfile::TempDir,
    orbit: PathBuf,
    feature: PathBuf,
    comet: PathBuf,
    config: tempfile::TempDir,
}

/// `orbit` (with a linked worktree `feature-x`) and an unrelated `comet`,
/// saved as tabs in `order` (indexes into [orbit, feature-x, comet]).
fn fixture(order: [usize; 3], config_toml: &str) -> Fixture {
    let (a, orbit) = repo("orbit", "alpha-only-commit");
    let (c, comet) = repo("comet", "charlie-only-commit");
    let wt_dir = tempfile::tempdir().unwrap();
    let feature = wt_dir.path().join("feature-x");
    let ok = Command::new("git")
        .args(["worktree", "add", "-q", "-b", "feature-x"])
        .arg(&feature)
        .current_dir(&orbit)
        .status()
        .unwrap()
        .success();
    assert!(ok, "git worktree add");
    let config = tempfile::tempdir().unwrap();
    std::fs::write(config.path().join("config.toml"), config_toml).unwrap();
    let file = session_file(config.path());
    std::fs::create_dir_all(file.parent().unwrap()).unwrap();
    let all = [&orbit, &feature, &comet];
    let mut s = GlobalSession::default();
    for i in order {
        s.open(all[i]);
    }
    let at = s.index_of(&orbit).unwrap();
    s.activate(at);
    s.save(&file).unwrap();
    Fixture {
        _keep: vec![a, c],
        _wt_dir: wt_dir,
        orbit,
        feature,
        comet,
        config,
    }
}

fn session_file(config: &Path) -> PathBuf {
    config.join("sessions").join(format!("{SESSION}.toml"))
}

fn order(f: &Fixture) -> Vec<&'static str> {
    GlobalSession::load(&session_file(f.config.path()))
        .unwrap()
        .tabs
        .iter()
        .map(|t| {
            if t.repo_path == f.orbit {
                "orbit"
            } else if t.repo_path == f.feature {
                "feature-x"
            } else {
                assert_eq!(t.repo_path, f.comet);
                "comet"
            }
        })
        .collect()
}

fn start(f: &Fixture) -> Terminal {
    common::launch(&f.orbit)
        .session(SESSION)
        .config_dir(f.config.path())
        .size(200, 50)
        .no_exit_pause()
        .ready("alpha-only-commit")
        .mouse_capture()
        .spawn()
}

fn bar_order(line: &str) -> Vec<&'static str> {
    let mut found: Vec<(usize, &'static str)> = ["orbit", "feature-x", "comet"]
        .into_iter()
        .filter_map(|n| line.find(n).map(|i| (i, n)))
        .collect();
    found.sort();
    found.into_iter().map(|(_, n)| n).collect()
}

fn wait_bar(term: &mut Terminal, want: &[&str]) {
    let want = want.to_vec();
    term.wait_for("the tab order", STEP, |s| {
        bar_order(s.text().lines().nth(1).unwrap_or("")) == want
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

fn tab_pos(term: &Terminal, name: &str) -> (u16, u16) {
    term.screen()
        .find_all(name)
        .into_iter()
        .find(|(_, r)| *r == 1)
        .unwrap_or_else(|| panic!("no {name} tab\n{}", term.text()))
}

#[test]
fn a_launch_groups_worktree_tabs_behind_their_repository() {
    // Saved ungrouped: the worktree first, its repository last.
    let f = fixture([1, 2, 0], "");
    let mut term = start(&f);
    wait_bar(&mut term, &["comet", "orbit", "feature-x"]);
    assert_eq!(order(&f), ["comet", "orbit", "feature-x"]);
    term.stop().unwrap();
}

#[test]
fn a_worktree_tab_dragged_away_snaps_back_next_to_its_repository() {
    let f = fixture([0, 1, 2], "");
    let mut term = start(&f);
    wait_bar(&mut term, &["orbit", "feature-x", "comet"]);
    let (fx, row) = tab_pos(&term, "feature-x");
    let (cx, _) = tab_pos(&term, "comet");
    term.press(fx + 1, row).unwrap();
    term.drag_to(fx + 5, row).unwrap();
    term.drag_to(cx + 8, row).unwrap();
    term.release(cx + 8, row).unwrap();
    // The drop reordered, then the grouping put it back.
    std::thread::sleep(Duration::from_millis(800));
    wait_bar(&mut term, &["orbit", "feature-x", "comet"]);
    assert_eq!(order(&f), ["orbit", "feature-x", "comet"]);
    term.stop().unwrap();
}

#[test]
fn with_grouping_off_tabs_stay_where_they_are_put() {
    let f = fixture([1, 2, 0], "[general]\ngroup_worktree_tabs = false\n");
    let mut term = start(&f);
    wait_bar(&mut term, &["feature-x", "comet", "orbit"]);
    let (fx, row) = tab_pos(&term, "feature-x");
    let (cx, _) = tab_pos(&term, "comet");
    term.press(fx + 1, row).unwrap();
    term.drag_to(fx + 5, row).unwrap();
    term.drag_to(cx + 8, row).unwrap();
    term.release(cx + 8, row).unwrap();
    wait_bar(&mut term, &["comet", "feature-x", "orbit"]);
    assert_eq!(order(&f), ["comet", "feature-x", "orbit"]);
    term.stop().unwrap();
}
