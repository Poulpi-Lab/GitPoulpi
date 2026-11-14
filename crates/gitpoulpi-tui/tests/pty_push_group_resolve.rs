//! A grouped push where one remote moved: `gitlab` has a commit on `feature`
//! that clashes with the local one. The plan defaults to Merge for it, Run
//! stops on the conflict, the dialog gives way to the ordinary merge banner
//! plus the strip "Grouped push paused (gitlab)", the conflict is resolved
//! and concluded with plain git, Continue (never automatic) turns up, and
//! pressing it finishes the push: every bare remote ends with the merge commit.
//!
//! Falsified: with the daemon-side `ready_to_continue` never true, or the strip
//! click not submitting `RemoteGroupContinue`, the last waits time out.

mod common;

use std::time::Duration;

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
        .push_to("backup", "feature")
        .unwrap()
        .track("feature", "origin")
        .unwrap()
        .push_targets("feature", &["gitlab", "backup"])
        .unwrap()
        .commit_file("clash.txt", "local side\n", "local clash")
        .unwrap()
        .build();
    // Someone else adds the same file to gitlab's feature.
    repo.push_from_elsewhere_to("gitlab", "feature", "clash");
    repo
}

fn button(screen: &Screen, label: &str) -> (u16, u16) {
    screen
        .find_all(label)
        .into_iter()
        .max_by_key(|(_, r)| *r)
        .unwrap_or_else(|| panic!("no {label} button:\n{}", screen.text()))
}

fn strip_button(screen: &Screen, label: &str) -> (u16, u16) {
    // The strip's row is the one that says "paused". `find_all` reports one
    // hit per row, and the strip's words may say the label too, so the
    // button is found cell by cell: the rightmost match on that row.
    let row = (0..screen.rows())
        .find(|y| screen.line(*y).contains("paused"))
        .unwrap_or_else(|| panic!("no strip:\n{}", screen.text()));
    let wanted: Vec<String> = label.chars().map(|c| c.to_string()).collect();
    let at = |x: u16| {
        screen
            .cell(x, row)
            .map(|c| c.display().to_string())
            .unwrap_or_default()
    };
    (0..screen.cols().saturating_sub(wanted.len() as u16))
        .rev()
        .find(|x| wanted.iter().enumerate().all(|(i, w)| at(x + i as u16) == *w))
        .map(|x| (x, row))
        .unwrap_or_else(|| panic!("no {label} on the strip:\n{}", screen.text()))
}

#[test]
fn a_diverged_remote_merges_stops_on_the_conflict_and_continue_finishes_the_push() {
    let repo = fixture();
    let (mut term, config) = common::launch(repo.root())
        .session("pty-push-group-resolve")
        .size(200, 50)
        .no_exit_pause()
        .ready("local clash")
        .start();
    std::mem::forget(config);
    term.send_keys("C-p").unwrap();
    term.send_text("Push to Selected Remotes").unwrap();
    term.wait_for("the palette row", STEP, |s| {
        s.contains("Git: Push to Selected Remotes...")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    term.wait_for("the selection step", STEP, |s| {
        s.contains("Push feature to selected remotes") && s.contains("[x] backup")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    let plan = term
        .wait_for("the plan with gitlab's merge default", STEP, |s| {
            s.contains("Push feature to 3 remotes")
                && s.contains("Merge from gitlab")
                && s.contains("diverged")
                && !s.contains("Asking the remotes")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(
        plan.contains("merge would conflict in 1 file: clash.txt"),
        "the conflict is predicted:\n{}",
        plan.text()
    );
    let (c, r) = button(&plan, "Run");
    term.click(c + 1, r).unwrap();
    // The dialog gives way to the strip; the merge banner is the ordinary one.
    let paused = term
        .wait_for("the paused strip", STEP, |s| {
            s.contains("Grouped push paused (gitlab)") && !s.contains("Push feature to")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(
        paused.contains("Abort group"),
        "Abort group is on the strip:\n{}",
        paused.text()
    );
    assert!(
        !paused.contains("Continue to push"),
        "Continue is not offered while the conflict is open:\n{}",
        paused.text()
    );
    assert!(
        repo.try_git(&["rev-parse", "-q", "--verify", "MERGE_HEAD"])
            .is_some_and(|s| !s.trim().is_empty()),
        "the merge is in progress in the repository"
    );
    // Nothing continued on its own: the others have not got the merge.
    assert_ne!(
        repo.git_in_named_remote("backup", &["rev-parse", "refs/heads/feature"])
            .trim(),
        repo.git(&["rev-parse", "HEAD"]).trim()
    );
    // Resolve and conclude with plain git.
    std::fs::write(repo.root().join("clash.txt"), "resolved\n").unwrap();
    repo.git(&["add", "clash.txt"]);
    repo.git(&["-c", "core.editor=true", "commit", "--no-edit"]);
    let ready = term
        .wait_for("Continue turned on", STEP, |s| {
            s.contains("Grouped push paused (gitlab)") && s.contains("Continue to push")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // Still nothing pushed: Continue is a button.
    assert_ne!(
        repo.git_in_named_remote("backup", &["rev-parse", "refs/heads/feature"])
            .trim(),
        repo.git(&["rev-parse", "HEAD"]).trim()
    );
    let (c, r) = strip_button(&ready, "Continue");
    term.click(c + 1, r).unwrap();
    let seen = std::cell::RefCell::new(Vec::<String>::new());
    let done = term
        .wait_for("the run finished", STEP, |s| {
            let status = common::status_line_text(s);
            let mut seen = seen.borrow_mut();
            if seen.last() != Some(&status) {
                seen.push(status);
            }
            s.contains("Done.")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}\nstatus lines seen: {:#?}", term.text(), seen.borrow()));
    assert!(!done.contains("Grouped push paused"), "{}", done.text());
    let merged = repo.git(&["rev-parse", "HEAD"]).trim().to_string();
    assert_eq!(
        repo.git(&["rev-list", "--parents", "-n1", "HEAD"])
            .split_whitespace()
            .count(),
        3,
        "HEAD is the merge commit"
    );
    assert_eq!(
        repo.git_in_remote(&["rev-parse", "refs/heads/feature"]).trim(),
        merged
    );
    for name in ["gitlab", "backup"] {
        assert_eq!(
            repo.git_in_named_remote(name, &["rev-parse", "refs/heads/feature"])
                .trim(),
            merged,
            "{name} has the merge commit"
        );
    }
}

/// Abort group throws the merge away after a confirmation and the strip goes.
#[test]
fn abort_group_asks_first_then_drops_the_merge_and_the_strip() {
    let repo = fixture();
    let (mut term, config) = common::launch(repo.root())
        .session("pty-push-group-abort")
        .size(200, 50)
        .no_exit_pause()
        .ready("local clash")
        .start();
    std::mem::forget(config);
    term.send_keys("C-p").unwrap();
    term.send_text("Push to Selected Remotes").unwrap();
    term.wait_for("the palette row", STEP, |s| {
        s.contains("Git: Push to Selected Remotes...")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    term.wait_for("the selection step", STEP, |s| {
        s.contains("[x] backup") && s.contains("Push feature to selected remotes")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    let plan = term
        .wait_for("the plan", STEP, |s| {
            s.contains("Merge from gitlab") && !s.contains("Asking the remotes")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (c, r) = button(&plan, "Run");
    term.click(c + 1, r).unwrap();
    let paused = term
        .wait_for("the paused strip", STEP, |s| {
            s.contains("Grouped push paused (gitlab)")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (c, r) = strip_button(&paused, "Abort group");
    term.click(c + 1, r).unwrap();
    let ask = term
        .wait_for("the confirmation", STEP, |s| {
            s.contains("Abort the grouped push of feature")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // The confirm's own Abort group button is the lowest one.
    let (c, r) = button(&ask, "Abort group");
    term.click(c + 1, r).unwrap();
    term.wait_for("the strip gone", STEP, |s| {
        !s.contains("Grouped push paused") && !s.contains("Abort the grouped push")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(
        repo.try_git(&["rev-parse", "-q", "--verify", "MERGE_HEAD"])
            .is_none_or(|s| s.trim().is_empty()),
        "the merge was aborted"
    );
}
