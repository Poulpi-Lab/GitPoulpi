//! The Pull button and dropdown on a branch with an extra pull source:
//! `feature` tracks `origin` and also pulls from `gitlab` (`gitpoulpi-pull-from`).
//! Each remote has a commit of its own (different files), so the Pull button
//! integrates the primary first (a fast-forward) and then gitlab (a merge),
//! with no dialog to answer. The dropdown's "Pull from gitlab only" takes
//! gitlab's commit alone.
//!
//! Falsified: with the Pull button routed back to the plain upstream pull
//! gitlab's commit never arrives; with `pull_one` ignoring the remote the
//! dropdown test pulls origin's commit instead.

mod common;

use std::time::{Duration, Instant};

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::{Screen, Terminal};

const STEP: Duration = Duration::from_secs(40);

fn fixture() -> FixtureRepo {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("base commit")
        .unwrap()
        .with_origin()
        .unwrap()
        .with_remote("gitlab")
        .unwrap()
        .push("main")
        .unwrap()
        .branch("feature")
        .unwrap()
        .commit("feature commit")
        .unwrap()
        .push("feature")
        .unwrap()
        .push_to("gitlab", "feature")
        .unwrap()
        .track("feature", "origin")
        .unwrap()
        .build();
    repo.git(&[
        "config",
        "--add",
        "branch.feature.gitpoulpi-pull-from",
        "gitlab",
    ]);
    repo.push_from_elsewhere("feature", "origin news");
    repo.push_from_elsewhere_to("gitlab", "feature", "gitlab news");
    repo
}

fn launch(repo: &FixtureRepo, session: &str) -> Terminal {
    let (term, config) = common::launch(repo.root())
        .session(session)
        .size(200, 50)
        .no_exit_pause()
        .ready("feature commit")
        .start();
    std::mem::forget(config);
    term
}

fn button(screen: &Screen, label: &str) -> (u16, u16) {
    screen
        .find_all(label)
        .into_iter()
        .next()
        .unwrap_or_else(|| panic!("no {label}:\n{}", screen.text()))
}

fn subjects(repo: &FixtureRepo) -> String {
    repo.git(&["log", "--format=%s", "feature"])
}

fn wait_subject(repo: &FixtureRepo, term: &Terminal, subject: &str) {
    let deadline = Instant::now() + STEP;
    while !subjects(repo).lines().any(|l| l == subject) {
        assert!(
            Instant::now() < deadline,
            "{subject:?} never reached feature:\n{}",
            term.text()
        );
        std::thread::sleep(Duration::from_millis(200));
    }
}

#[test]
fn the_pull_button_integrates_the_primary_then_the_pull_source() {
    let repo = fixture();
    let mut term = launch(&repo, "pty-pull-sources-button");
    let screen = term.screen();
    let (c, r) = button(&screen, "\u{21E9} Pull");
    term.click(c + 1, r).unwrap();
    wait_subject(&repo, &term, "gitlab news");
    wait_subject(&repo, &term, "origin news");
    // Origin fast-forwarded, gitlab merged on top: a merge commit with two
    // parents is HEAD.
    assert_eq!(
        repo.git(&["rev-list", "--parents", "-n1", "feature"])
            .split_whitespace()
            .count(),
        3,
        "HEAD is the merge of gitlab: {}",
        subjects(&repo)
    );
}

#[test]
fn the_pull_dropdown_pulls_from_one_remote_only() {
    let repo = fixture();
    let mut term = launch(&repo, "pty-pull-sources-only");
    let screen = term.screen();
    let (c, r) = button(&screen, "\u{21E9} Pull");
    let line = screen.line(r);
    let chars: Vec<char> = line.chars().collect();
    let arrow = (usize::from(c)..chars.len())
        .find(|i| chars[*i] == '\u{25BE}')
        .expect("an arrow after Pull");
    term.click(arrow as u16, r).unwrap();
    let menu = term
        .wait_for("the Pull dropdown with the group rows", STEP, |s| {
            s.contains("Pull from all sources (origin + gitlab)")
                && s.contains("Pull from gitlab only")
                && s.contains("Pull from origin only")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (c, r) = button(&menu, "Pull from gitlab only");
    term.click(c + 3, r).unwrap();
    wait_subject(&repo, &term, "gitlab news");
    std::thread::sleep(Duration::from_millis(800));
    assert!(
        !subjects(&repo).lines().any(|l| l == "origin news"),
        "origin was not pulled: {}",
        subjects(&repo)
    );
}
