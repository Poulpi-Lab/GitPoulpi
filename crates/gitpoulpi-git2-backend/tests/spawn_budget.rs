//! Perf regression guard (the perf design): how many `git`
//! processes each operation spawns, on both backends. A spawn costs 10-40 ms
//! on Windows, so the count is the deterministic proxy for latency. The
//! budgets are the current measured values; raise one only with a reason.

use gitpoulpi_cli_backend::{total_spawn_count, SystemGitBackend};
use gitpoulpi_core::domain::{FetchOptions, StageTarget};
use gitpoulpi_core::GitBackend;
use gitpoulpi_git2_backend::Git2Backend;
use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder, SpawnBudget};

fn dirty_repo(files: usize) -> FixtureRepo {
    let mut b = FixtureRepoBuilder::new().unwrap();
    for i in 0..files {
        b = b
            .commit_file(&format!("src/f{i:02}.txt"), "one\n", &format!("add {i}"))
            .unwrap();
    }
    for i in 0..files {
        b = b
            .write_worktree(&format!("src/f{i:02}.txt"), b"two\n")
            .unwrap();
    }
    b.build()
}

fn budget() -> SpawnBudget {
    SpawnBudget::serial(total_spawn_count)
}

#[test]
fn git2_staging_spawns_no_git() {
    let sb = budget();
    let repo = dirty_repo(6);
    let r = repo.repo_ref();
    let b = Git2Backend::new();
    sb.assert_at_most("git2 stage file", 0, || {
        b.stage(&r, &StageTarget::File("src/f00.txt".into()))
            .unwrap()
    });
    sb.assert_at_most("git2 unstage file", 0, || {
        b.unstage(&r, &StageTarget::File("src/f00.txt".into()))
            .unwrap()
    });
    sb.assert_at_most("git2 stage all", 0, || {
        b.stage(&r, &StageTarget::All).unwrap()
    });
    sb.assert_at_most("git2 unstage all", 0, || {
        b.unstage(&r, &StageTarget::All).unwrap()
    });
}

#[test]
fn git2_status_and_refresh_reads_stay_cheap() {
    let sb = budget();
    let repo = dirty_repo(4);
    let r = repo.repo_ref();
    let b = Git2Backend::new();
    sb.assert_at_most("git2 status", 0, || b.status(&r).unwrap());
    sb.assert_at_most("git2 refs", 0, || b.refs(&r).unwrap());
    #[cfg(feature = "lfs")]
    {
        // The trait method always re-probes (one spawn, the `git-lfs` probe);
        // lower this to 0 once a warm `lfs_status` is cached.
        b.lfs_status(&r).unwrap();
        sb.assert_at_most("git2 lfs_status (warm)", 1, || b.lfs_status(&r).unwrap());
    }
}

#[test]
fn system_staging_is_one_spawn_per_call() {
    let sb = budget();
    let repo = dirty_repo(6);
    let r = repo.repo_ref();
    let b = SystemGitBackend::new();
    sb.assert_at_most("system stage file", 1, || {
        b.stage(&r, &StageTarget::File("src/f00.txt".into()))
            .unwrap()
    });
    sb.assert_at_most("system unstage file", 2, || {
        b.unstage(&r, &StageTarget::File("src/f00.txt".into()))
            .unwrap()
    });
    sb.assert_at_most("system stage all", 1, || {
        b.stage(&r, &StageTarget::All).unwrap()
    });
    sb.assert_at_most("system unstage all", 2, || {
        b.unstage(&r, &StageTarget::All).unwrap()
    });
}

#[test]
fn discarding_fifty_paths_is_a_constant_number_of_spawns() {
    let sb = budget();
    let repo = dirty_repo(50);
    let paths: Vec<String> = (0..50).map(|i| format!("src/f{i:02}.txt")).collect();
    let r = repo.repo_ref();
    sb.assert_at_most("system discard(50)", 3, || {
        SystemGitBackend::new().discard(&r, &paths).unwrap()
    });
    let repo = dirty_repo(50);
    let r = repo.repo_ref();
    sb.assert_at_most("git2 discard(50)", 3, || {
        Git2Backend::new().discard(&r, &paths).unwrap()
    });
}

#[test]
fn fetch_is_at_most_three_spawns() {
    let sb = budget();
    for (name, backend) in [
        ("git2", Box::new(Git2Backend::new()) as Box<dyn GitBackend>),
        ("system", Box::new(SystemGitBackend::new())),
    ] {
        let repo = FixtureRepoBuilder::new()
            .unwrap()
            .commit("initial")
            .unwrap()
            .with_origin()
            .unwrap()
            .push("main")
            .unwrap()
            .build();
        let r = repo.repo_ref();
        repo.push_from_elsewhere("main", "remote change");
        // Without `git fetch --porcelain` (git before 2.41) the refs a fetch
        // updated are found by listing the remote-tracking refs before and
        // after it: two more spawns, plus the one-off `git --version` probe.
        let budget = if gitpoulpi_testkit::system_git_at_least(2, 41) {
            3
        } else {
            5
        };
        sb.assert_at_most(&format!("{name} fetch"), budget, || {
            gitpoulpi_testkit::contract::block_on(backend.fetch(
                &r,
                "origin",
                &FetchOptions::default(),
            ))
            .unwrap()
        });
    }
}

/// The refresh's conflict listing reads no blob: no process under git2,
/// one `ls-files -u` under system git, however many files are unmerged.
/// A document is a handful (`ls-files`, one `cat-file --batch`, one
/// `merge-file`, plus the version probe the first time) on system git, and
/// one `merge-file` (plus the probe) under git2.
#[test]
fn conflict_listing_is_cheap_and_a_document_is_a_handful_of_spawns() {
    let sb = budget();
    let fx = gitpoulpi_testkit::ConflictFixture::multi_conflict();
    let r = fx.repo_ref();
    let git2 = Git2Backend::new();
    let system = SystemGitBackend::new();
    sb.assert_at_most("git2 conflict_entries", 0, || {
        assert_eq!(git2.conflict_entries(&r).unwrap().len(), 3)
    });
    sb.assert_at_most("system conflict_entries", 1, || {
        assert_eq!(system.conflict_entries(&r).unwrap().len(), 3)
    });
    let path = std::path::Path::new("a.txt");
    // The version probe is once per process; run one document first so the
    // budgets below are the steady state.
    system.merge_document(&r, path).unwrap();
    sb.assert_at_most("system merge_document", 3, || {
        system.merge_document(&r, path).unwrap()
    });
    sb.assert_at_most("git2 merge_document", 1, || {
        git2.merge_document(&r, path).unwrap()
    });
}

#[test]
fn libgit2_commit_count_spawns_no_git() {
    use gitpoulpi_core::domain::LogOptions;
    let sb = budget();
    let repo = dirty_repo(6);
    let r = repo.repo_ref();
    let all = LogOptions {
        all_refs: true,
        ..LogOptions::default()
    };
    // `log.commit_count = "libgit2"`: the whole point is no process.
    let total = sb.assert_at_most("libgit2 commit count", 0, || {
        gitpoulpi_git2_backend::count_commits_in_process(&r, &all).unwrap()
    });
    assert_eq!(total, 6);
    // The system-git method is the one spawn it replaces.
    sb.assert_at_most("system commit count", 1, || {
        SystemGitBackend::new().count_commits(&r, &all).unwrap()
    });
}
