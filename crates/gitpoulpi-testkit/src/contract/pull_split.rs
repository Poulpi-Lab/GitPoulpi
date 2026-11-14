//! A pull is two steps the daemon can tell apart (`pull_fetch`: download,
//! cancellable; `pull_integrate`: merge, the point of no return). These
//! checks pin that the two halves, run in sequence, do exactly what the
//! one-step `pull` does, on every shape of pull the suite knows, and that the
//! download half touches nothing a user can see in the checkout.

use super::block_on;
use crate::{FixtureRepo, TestRepoBuilder};
use gitpoulpi_core::domain::{PullMode, PullOptions, PullOutcome};
use gitpoulpi_core::GitBackend;

#[derive(Clone, Copy, Debug)]
enum Shape {
    /// Upstream has a commit we lack: fast-forwardable.
    Behind,
    /// Both sides have a commit.
    Diverged,
    /// Nothing to pull.
    UpToDate,
    /// Both sides added the same file with different content.
    Conflicting,
    /// No upstream configured, an explicit remote is passed.
    ExplicitRemote,
    /// No upstream and no remote: git refuses.
    NoUpstream,
}

fn build(shape: Shape, mode: PullMode) -> (FixtureRepo, PullOptions) {
    let base = TestRepoBuilder::new()
        .with_initial()
        .with_origin()
        .push("main");
    let repo = match shape {
        Shape::Behind | Shape::UpToDate => base.track("main", "origin").build(),
        Shape::Diverged => base.track("main", "origin").commit("mine").build(),
        Shape::Conflicting => base
            .track("main", "origin")
            .commit_file("clash.txt", "mine\n", "mine")
            .build(),
        Shape::ExplicitRemote | Shape::NoUpstream => base.build(),
    };
    match shape {
        Shape::Behind | Shape::Diverged | Shape::ExplicitRemote => {
            repo.push_from_elsewhere("main", "theirs")
        }
        Shape::Conflicting => repo.push_from_elsewhere("main", "clash"),
        Shape::UpToDate | Shape::NoUpstream => {}
    }
    let remote = matches!(shape, Shape::ExplicitRemote).then(|| "origin".to_string());
    (
        repo,
        PullOptions {
            mode,
            remote,
            ..PullOptions::default()
        },
    )
}

/// What a pull left behind, without anything that differs between two
/// otherwise identical repositories (paths, timestamps, object ids of merge
/// commits).
fn aftermath(repo: &FixtureRepo, result: &gitpoulpi_core::Result<PullOutcome>) -> String {
    let outcome = match result {
        Ok(PullOutcome::UpToDate) => "up-to-date",
        Ok(PullOutcome::FastForwarded(_)) => "fast-forwarded",
        Ok(PullOutcome::Merged(_)) => "merged",
        Ok(PullOutcome::Conflicted) => "conflicted",
        Err(gitpoulpi_core::Error::NotFastForward) => "err:not-fast-forward",
        Err(_) => "err:other",
    };
    let git = |args: &[&str]| repo.try_git(args).unwrap_or_default();
    let git_dir = repo.root().join(".git");
    format!(
        "{outcome}\ntree {}\nsubject {}\ncount {}\nparents {}\nstatus\n{}\nunmerged {}\nstash {}\n\
         merge-head {}\nrebase {}",
        git(&["rev-parse", "HEAD^{tree}"]).trim(),
        // "Merge branch 'main' of <the origin's temp path>": the path differs.
        git(&["log", "-1", "--format=%s"])
            .trim()
            .split(" of ")
            .next()
            .unwrap_or_default(),
        git(&["rev-list", "--count", "HEAD"]).trim(),
        git(&["rev-list", "--parents", "-n", "1", "HEAD"])
            .split_whitespace()
            .count(),
        git(&["status", "--porcelain"]),
        git(&["diff", "--name-only", "--diff-filter=U"]).trim(),
        git(&["stash", "list"]).trim(),
        git_dir.join("MERGE_HEAD").exists(),
        git_dir.join("rebase-merge").exists(),
    )
}

/// `pull_fetch` then `pull_integrate` leaves the repository exactly as `pull`
/// does, for each `(shape, mode)`.
fn assert_split_matches_whole(backend: &dyn GitBackend, cases: &[(Shape, PullMode)]) {
    for &(shape, mode) in cases {
        let (one_step, opts) = build(shape, mode);
        let whole = block_on(backend.pull(&one_step.repo_ref(), &opts));
        let whole = aftermath(&one_step, &whole);

        let (two_steps, opts) = build(shape, mode);
        let split = block_on(async {
            backend.pull_fetch(&two_steps.repo_ref(), &opts).await?;
            backend.pull_integrate(&two_steps.repo_ref(), &opts).await
        });
        let split = aftermath(&two_steps, &split);

        assert_eq!(
            split, whole,
            "{shape:?} / {mode:?}: the split pull differs from the one-step pull"
        );
    }
}

/// Split and one-step pulls agree when the branch is behind (every mode) and
/// when there is nothing to pull.
pub fn check_a_split_pull_matches_the_one_step_pull_when_behind(backend: &dyn GitBackend) {
    assert_split_matches_whole(
        backend,
        &[
            (Shape::Behind, PullMode::Merge),
            (Shape::Behind, PullMode::FastForwardOnly),
            (Shape::Behind, PullMode::Rebase),
            (Shape::UpToDate, PullMode::Merge),
        ],
    );
}

/// ... when the branches diverged (merge, rebase, and the typed ff-only
/// refusal).
pub fn check_a_split_pull_matches_the_one_step_pull_when_diverged(backend: &dyn GitBackend) {
    assert_split_matches_whole(
        backend,
        &[
            (Shape::Diverged, PullMode::Merge),
            (Shape::Diverged, PullMode::FastForwardOnly),
            (Shape::Diverged, PullMode::Rebase),
        ],
    );
}

/// ... on conflicts, with an explicit remote, and when git refuses (no
/// upstream).
pub fn check_a_split_pull_matches_the_one_step_pull_on_conflicts_and_refusals(
    backend: &dyn GitBackend,
) {
    assert_split_matches_whole(
        backend,
        &[
            (Shape::Conflicting, PullMode::Merge),
            (Shape::Conflicting, PullMode::Rebase),
            (Shape::ExplicitRemote, PullMode::FastForwardOnly),
            (Shape::NoUpstream, PullMode::Merge),
        ],
    );
}

/// The download half brings the remote's commits in and changes nothing a
/// user would notice: `HEAD`, the index, the working tree (dirty files
/// included), the stash and any merge state stay as they were.
pub fn check_the_download_half_of_a_pull_leaves_the_checkout_alone(backend: &dyn GitBackend) {
    let (repo, opts) = build(Shape::Diverged, PullMode::Merge);
    std::fs::write(repo.root().join("initial.txt"), "edited\n").unwrap();
    std::fs::write(repo.root().join("staged.txt"), "staged\n").unwrap();
    repo.git(&["add", "staged.txt"]);
    std::fs::write(repo.root().join("loose.txt"), "untracked\n").unwrap();

    let state = |repo: &FixtureRepo| {
        format!(
            "{}|{}|{}|{}|{}",
            repo.git(&["rev-parse", "HEAD"]).trim(),
            repo.git(&["ls-files", "-s"]),
            repo.git(&["status", "--porcelain"]),
            repo.git(&["stash", "list"]),
            repo.root().join(".git").join("MERGE_HEAD").exists(),
        )
    };
    let before = state(&repo);
    let remote_tip = repo.git_in_remote(&["rev-parse", "main"]);

    block_on(backend.pull_fetch(&repo.repo_ref(), &opts)).expect("the download half");

    assert_eq!(state(&repo), before, "the download changed the checkout");
    assert_eq!(
        repo.git(&["rev-parse", "refs/remotes/origin/main"]),
        remote_tip,
        "the download must bring the remote's commits in"
    );
    assert_eq!(
        std::fs::read_to_string(repo.root().join("initial.txt")).unwrap(),
        "edited\n"
    );
}
