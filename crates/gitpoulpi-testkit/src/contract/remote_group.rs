//! Push and pull primitives a branch that lives on several remotes needs:
//! explicit leases, a differently named remote branch, a typed refusal by
//! the remote itself, and patch-aware ahead/behind counts.

use super::block_on;
use crate::TestRepoBuilder;
use gitpoulpi_core::domain::{
    Oid, PullMode, PullOptions, PullOutcome, PushLease, PushOptions, PushRejectReason,
};
use gitpoulpi_core::{Error, GitBackend};

fn oid(text: &str) -> Oid {
    Oid::from_hex(text.trim()).unwrap()
}

fn rejection_reason(err: Error) -> PushRejectReason {
    match err {
        Error::PushRejected(rejection) => rejection.reason,
        other => panic!("expected a typed push rejection, got {other:?}"),
    }
}

/// `Expect(oid)` is `--force-with-lease=<ref>:<oid>`: it overwrites the
/// remote only while the remote still holds exactly that commit, whatever
/// the remote-tracking ref says.
pub fn check_push_lease_expect_overwrites_only_the_expected_tip(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .with_initial()
        .with_origin()
        .push("main")
        .track("main", "origin")
        .commit("mine")
        .build();
    let repo_ref = repo.repo_ref();
    let first = repo.git_in_remote(&["rev-parse", "main"]);

    // The remote moved and we fetched: our tracking ref says `theirs`.
    repo.push_from_elsewhere("main", "theirs");
    repo.git(&["fetch", "origin"]);
    // It moved again behind our back: the tracking ref is now stale.
    repo.push_from_elsewhere("main", "theirs again");
    let newest = repo.git_in_remote(&["rev-parse", "main"]);
    let mine = repo.git(&["rev-parse", "HEAD"]);

    let stale = PushOptions {
        lease: Some(PushLease::Expect(oid(&first))),
        ..PushOptions::default()
    };
    let err = block_on(backend.push(&repo_ref, "origin", &stale))
        .expect_err("a lease on an old tip must refuse");
    assert_eq!(rejection_reason(err), PushRejectReason::StaleLease);
    assert_eq!(repo.git_in_remote(&["rev-parse", "main"]), newest);

    // The tracking lease follows the (stale) tracking ref, so it refuses too.
    let tracking = PushOptions {
        lease: Some(PushLease::Tracking),
        ..PushOptions::default()
    };
    let err = block_on(backend.push(&repo_ref, "origin", &tracking))
        .expect_err("the tracking lease must refuse");
    assert_eq!(rejection_reason(err), PushRejectReason::StaleLease);

    // `force_with_lease: true` with no `lease` keeps meaning the same.
    let bare = PushOptions {
        force_with_lease: true,
        ..PushOptions::default()
    };
    let err =
        block_on(backend.push(&repo_ref, "origin", &bare)).expect_err("the bare lease must refuse");
    assert_eq!(rejection_reason(err), PushRejectReason::StaleLease);

    let exact = PushOptions {
        lease: Some(PushLease::Expect(oid(&newest))),
        ..PushOptions::default()
    };
    block_on(backend.push(&repo_ref, "origin", &exact)).expect("the exact lease overwrites");
    assert_eq!(repo.git_in_remote(&["rev-parse", "main"]), mine);
}

/// `Absent` is `--force-with-lease=<ref>:`: the branch must not exist yet.
pub fn check_push_lease_absent_creates_and_refuses_an_existing_branch(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .with_initial()
        .with_origin()
        .push("main")
        .commit("mine")
        .build();
    let repo_ref = repo.repo_ref();
    let opts = PushOptions {
        lease: Some(PushLease::Absent),
        remote_branch: Some("fresh".into()),
        ..PushOptions::default()
    };

    block_on(backend.push(&repo_ref, "origin", &opts)).expect("a missing branch is created");
    let created = repo.git_in_remote(&["rev-parse", "fresh"]);
    assert_eq!(created, repo.git(&["rev-parse", "HEAD"]));

    std::fs::write(repo.root().join("more.txt"), "more\n").unwrap();
    repo.git(&["add", "more.txt"]);
    repo.git(&[
        "-c",
        "user.name=Testkit",
        "-c",
        "user.email=testkit@gitpoulpi.local",
        "commit",
        "-m",
        "more",
    ]);
    let err = block_on(backend.push(&repo_ref, "origin", &opts))
        .expect_err("an existing branch refuses the absent lease");
    assert_eq!(rejection_reason(err), PushRejectReason::StaleLease);
    assert_eq!(repo.git_in_remote(&["rev-parse", "fresh"]), created);
}

/// `remote_branch` is the destination name: `main` lands as `topic`, and
/// the remote's own `main` is left alone.
pub fn check_push_to_a_differently_named_remote_branch(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .with_initial()
        .with_origin()
        .push("main")
        .commit("mine")
        .build();
    let repo_ref = repo.repo_ref();
    let main_before = repo.git_in_remote(&["rev-parse", "main"]);

    let out = block_on(backend.push(
        &repo_ref,
        "origin",
        &PushOptions {
            remote_branch: Some("topic".into()),
            ..PushOptions::default()
        },
    ))
    .expect("push under another name");
    assert_eq!(out.pushed_refs.len(), 1);
    assert_eq!(
        repo.git_in_remote(&["rev-parse", "topic"]),
        repo.git(&["rev-parse", "HEAD"])
    );
    assert_eq!(repo.git_in_remote(&["rev-parse", "main"]), main_before);
}

/// A push the remote itself refuses (here a `pre-receive` hook) is the
/// typed `RemoteRefused`, with the remote's words in the detail.
pub fn check_push_refused_by_a_pre_receive_hook_is_remote_refused(backend: &dyn GitBackend) {
    let sh_works = std::process::Command::new("sh")
        .args(["-c", "true"])
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    if !sh_works {
        eprintln!("SKIPPED: no `sh` to run the hook");
        return;
    }
    let repo = TestRepoBuilder::new()
        .with_initial()
        .with_origin()
        .with_pre_receive_hook("origin", "echo 'no pushes today' >&2\nexit 1")
        .build();
    let repo_ref = repo.repo_ref();

    let err = block_on(backend.push(&repo_ref, "origin", &PushOptions::default()))
        .expect_err("the hook refuses the push");
    match err {
        Error::PushRejected(rejection) => {
            assert_eq!(rejection.reason, PushRejectReason::RemoteRefused);
            assert_eq!(rejection.remote, "origin");
            let detail = rejection.detail.expect("the remote's words are kept");
            assert!(
                detail.stderr.contains("no pushes today"),
                "{}",
                detail.stderr
            );
        }
        other => panic!("expected a typed rejection, got {other:?}"),
    }
}

/// A pull names its remote and, when the branch is published under
/// another name, the remote branch.
pub fn check_pull_from_a_named_remote_and_branch(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .with_initial()
        .with_remote("backup")
        .push_to("backup", "main:topic")
        .build();
    repo.push_from_elsewhere_to("backup", "topic", "theirs");
    let theirs = repo.git_in_named_remote("backup", &["rev-parse", "topic"]);

    let outcome = block_on(backend.pull(
        &repo.repo_ref(),
        &PullOptions {
            mode: PullMode::FastForwardOnly,
            remote: Some("backup".into()),
            remote_branch: Some("topic".into()),
        },
    ))
    .expect("pull from backup/topic");
    assert!(
        matches!(outcome, PullOutcome::FastForwarded(_)),
        "{outcome:?}"
    );
    assert_eq!(repo.git(&["rev-parse", "HEAD"]), theirs);
}

/// `cherry_counts` does not count a commit whose patch the other side has
/// under another id (a rebased or cherry-picked copy), where
/// `ahead_behind` does.
pub fn check_cherry_counts_ignore_rebased_copies(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .with_initial()
        .with_origin()
        .commit("a")
        .push("main")
        .build();
    let remote_tip = repo.git(&["rev-parse", "HEAD"]);
    // Local history: the same patch as `a`, replayed on top of other work.
    repo.git(&["reset", "--hard", "HEAD~1"]);
    std::fs::write(repo.root().join("other.txt"), "other\n").unwrap();
    repo.git(&["add", "other.txt"]);
    let identity = [
        "-c",
        "user.name=Testkit",
        "-c",
        "user.email=testkit@gitpoulpi.local",
    ];
    let mut commit = identity.to_vec();
    commit.extend(["commit", "-m", "other"]);
    repo.git(&commit);
    let mut pick = identity.to_vec();
    pick.extend(["cherry-pick", remote_tip.as_str()]);
    repo.git(&pick);
    let repo_ref = repo.repo_ref();
    let local = oid(&repo.git(&["rev-parse", "HEAD"]));
    let remote = oid(&remote_tip);

    assert_eq!(
        backend.ahead_behind(&repo_ref, local, remote).unwrap(),
        (2, 1)
    );
    assert_eq!(
        backend.cherry_counts(&repo_ref, local, remote).unwrap(),
        (1, 0)
    );
}
