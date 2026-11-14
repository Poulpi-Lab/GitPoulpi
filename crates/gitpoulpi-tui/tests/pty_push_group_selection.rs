//! The selection step of "Push to selected remotes...": every remote of the
//! repository with a checkbox, pre-ticked with the branch's upstream and push
//! targets. Unticking one pushes to the others only and leaves the config
//! alone; with "Remember for this branch" the ticked remotes become the
//! branch's push targets.
//!
//! The fixture: `feature` tracks `origin` and pushes to `gitlab`; `backup` is
//! a remote the branch does not use. Falsified: with the `Space` toggle, the
//! target list of Next or the Remember write broken, the matching test goes
//! red.

mod common;

use std::time::{Duration, Instant};

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::{Screen, Terminal};

const STEP: Duration = Duration::from_secs(40);
const PUSH_TO: &str = "branch.feature.gitpoulpi-push-to";

fn fixture() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit("base commit")
        .unwrap()
        .with_origin()
        .unwrap()
        .with_remote("gitlab")
        .unwrap()
        .with_remote("backup")
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
        .push_targets("feature", &["gitlab"])
        .unwrap()
        .commit("second commit")
        .unwrap()
        .build()
}

fn open(repo: &FixtureRepo, session: &str) -> Terminal {
    let (mut term, config) = common::launch(repo.root())
        .session(session)
        .size(200, 50)
        .no_exit_pause()
        .ready("second commit")
        .start();
    std::mem::forget(config);
    term.send_keys("C-p").unwrap();
    term.send_text("Push to Selected Remotes").unwrap();
    term.wait_for("the palette row", STEP, |s| {
        s.contains("Git: Push to Selected Remotes...")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    term.wait_for("the selection with its pre-ticked rows", STEP, |s| {
        s.contains("Push feature to selected remotes")
            && s.contains("[x] origin")
            && s.contains("[x] gitlab")
            && s.contains("[ ] backup")
            && s.contains("Remember for this branch")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term
}

fn button(screen: &Screen, label: &str) -> (u16, u16) {
    screen
        .find_all(label)
        .into_iter()
        .max_by_key(|(_, r)| *r)
        .unwrap_or_else(|| panic!("no {label} button:\n{}", screen.text()))
}

fn push_to(repo: &FixtureRepo) -> Vec<String> {
    repo.try_git(&["config", "--local", "--get-all", PUSH_TO])
        .unwrap_or_default()
        .lines()
        .map(str::to_string)
        .collect()
}

fn remote_head(repo: &FixtureRepo, name: &str) -> String {
    if name == "origin" {
        repo.git_in_remote(&["rev-parse", "feature"])
    } else {
        repo.git_in_named_remote(name, &["rev-parse", "refs/heads/feature"])
    }
    .trim()
    .to_string()
}

/// The rows are the repository's remotes in name order: backup, gitlab,
/// origin. Tick backup, untick gitlab, Next, Run: origin and backup get the
/// commit, gitlab keeps its old one, and the config is as it was.
#[test]
fn unticking_a_remote_pushes_to_the_others_only_and_changes_no_config() {
    let repo = fixture();
    let gitlab_before = remote_head(&repo, "gitlab");
    let mut term = open(&repo, "pty-push-group-select");
    term.send_keys("Space").unwrap();
    term.send_keys("Down").unwrap();
    term.send_keys("Space").unwrap();
    term.wait_for("gitlab unticked, backup ticked", STEP, |s| {
        s.contains("[x] origin") && s.contains("[ ] gitlab") && s.contains("[x] backup")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    let plan = term
        .wait_for("the plan of the two ticked remotes", STEP, |s| {
            s.contains("Push feature to 2 remotes")
                && s.contains("backup")
                && !s.contains("Asking the remotes")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(
        !plan.contains("Then: push to 3 remotes"),
        "{}",
        plan.text()
    );
    let (c, r) = button(&plan, "Run");
    term.click(c + 1, r).unwrap();
    term.wait_for("Done", STEP, |s| s.contains("Done."))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let local = repo.git(&["rev-parse", "feature"]).trim().to_string();
    assert_eq!(remote_head(&repo, "origin"), local);
    assert_eq!(remote_head(&repo, "backup"), local);
    assert_eq!(
        remote_head(&repo, "gitlab"),
        gitlab_before,
        "the unticked remote was not pushed to"
    );
    assert_eq!(push_to(&repo), ["gitlab"], "the config is unchanged");
}

/// With Remember ticked, Next writes the ticked remotes as the push targets.
#[test]
fn remember_writes_the_ticked_remotes_as_the_branchs_push_targets() {
    let repo = fixture();
    let mut term = open(&repo, "pty-push-group-remember");
    // Tick backup (the first row) and Remember.
    term.send_keys("Space").unwrap();
    term.send_text("m").unwrap();
    term.wait_for("backup and Remember ticked", STEP, |s| {
        s.contains("[x] backup") && s.contains("[x] Remember for this branch")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    term.wait_for("the plan of three remotes", STEP, |s| {
        s.contains("Push feature to 3 remotes") && !s.contains("Asking the remotes")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let deadline = Instant::now() + STEP;
    loop {
        let mut now = push_to(&repo);
        now.sort();
        if now == ["backup", "gitlab"] {
            break;
        }
        assert!(Instant::now() < deadline, "push-to is {now:?}");
        std::thread::sleep(Duration::from_millis(200));
    }
}
