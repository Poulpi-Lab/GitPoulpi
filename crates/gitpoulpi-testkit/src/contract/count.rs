use crate::TestRepoBuilder;
use gitpoulpi_core::domain::{LogOptions, RepoRef};
use gitpoulpi_core::{GitBackend, Result};

/// Something that counts a repository's commits.
pub type Counter<'a> = dyn Fn(&RepoRef, &LogOptions) -> Result<u64> + 'a;

/// `count_commits` is the total of exactly the walk `log` pages through:
/// the same refs (`all_refs` = branches + tags + remotes, never the
/// bookkeeping mirror), merges counted once, `skip`/`limit` ignored, path
/// filters honoured, and an unborn repository is zero rather than an error.
pub fn check_count_commits_matches_the_walk_the_graph_loads(backend: &dyn GitBackend) {
    counter_matches_the_walk(backend, &|r, o| backend.count_commits(r, o));
}

/// The same contract for any counter that must agree with `backend`'s `log`
/// (the in-process libgit2 count, `log.commit_count = "libgit2"`).
pub fn counter_matches_the_walk(backend: &dyn GitBackend, counter: &Counter<'_>) {
    let empty = TestRepoBuilder::new().build();
    assert_eq!(
        counter(&empty.repo_ref(), &LogOptions::default()).unwrap(),
        0,
        "an unborn repository has no commits"
    );

    let repo = TestRepoBuilder::new()
        .commit("c1")
        .commit("c2")
        .commit("c3")
        .build();
    let repo_ref = repo.repo_ref();
    let main = repo
        .git(&["rev-parse", "--abbrev-ref", "HEAD"])
        .trim()
        .to_string();
    // Two side branches off c1/c2, one merged back with a merge commit, one
    // left alone (reachable only through `all_refs`), plus a tag on a
    // commit reachable from nothing else.
    repo.git(&["checkout", "-q", "-b", "side-a", "HEAD~2"]);
    std::fs::write(repo.root().join("a.txt"), "a").unwrap();
    repo.git(&["add", "a.txt"]);
    repo.git(&["commit", "-q", "-m", "a1"]);
    repo.git(&["checkout", "-q", "-b", "side-b", "HEAD~1"]);
    std::fs::write(repo.root().join("b.txt"), "b").unwrap();
    repo.git(&["add", "b.txt"]);
    repo.git(&["commit", "-q", "-m", "b1"]);
    repo.git(&["checkout", "-q", &main]);
    repo.git(&["merge", "-q", "--no-ff", "-m", "merge a", "side-a"]);
    repo.git(&["checkout", "-q", "--detach", "side-b"]);
    std::fs::write(repo.root().join("t.txt"), "t").unwrap();
    repo.git(&["add", "t.txt"]);
    repo.git(&["commit", "-q", "-m", "tagged only"]);
    repo.git(&["tag", "lonely"]);
    // An annotated tag too: the walk must peel it to its commit.
    repo.git(&["tag", "-a", "-m", "annotated", "ann", "side-a"]);
    repo.git(&["checkout", "-q", &main]);
    // The bookkeeping mirror must not count.
    repo.git(&["update-ref", "refs/gitpoulpi/remote-tags/x", "side-b"]);

    let all = LogOptions {
        all_refs: true,
        ..LogOptions::default()
    };
    let walked = backend.log(&repo_ref, &all).unwrap().nodes.len();
    assert!(walked >= 6, "fixture should hold the branches: {walked}");
    assert_eq!(counter(&repo_ref, &all).unwrap(), walked as u64);

    let head_only = LogOptions::default();
    assert_eq!(
        counter(&repo_ref, &head_only).unwrap(),
        backend.log(&repo_ref, &head_only).unwrap().nodes.len() as u64
    );

    // `skip` and `limit` are page parameters, not part of the total.
    let paged = LogOptions {
        skip: 2,
        limit: Some(1),
        ..all.clone()
    };
    assert_eq!(counter(&repo_ref, &paged).unwrap(), walked as u64);

    let from_b = LogOptions {
        start_ref: Some("side-b".into()),
        ..LogOptions::default()
    };
    assert_eq!(
        counter(&repo_ref, &from_b).unwrap(),
        backend.log(&repo_ref, &from_b).unwrap().nodes.len() as u64
    );

    let by_path = LogOptions {
        all_refs: true,
        paths: vec!["a.txt".into()],
        ..LogOptions::default()
    };
    assert_eq!(
        counter(&repo_ref, &by_path).unwrap(),
        backend.log(&repo_ref, &by_path).unwrap().nodes.len() as u64
    );
}
