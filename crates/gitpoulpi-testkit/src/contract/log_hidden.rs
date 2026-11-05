//! `LogOptions::hidden`: refs the all-refs walk leaves out, and the commits
//! only they reach with them.

use super::count::Counter;
use crate::{FixtureRepo, TestRepoBuilder};
use gitpoulpi_core::domain::{LogOptions, Oid, RefFilter};
use gitpoulpi_core::GitBackend;

/// A chain of `n` commits unrelated to everything else (HEAD's tree, no
/// parent for the first), summaries `<prefix> 1` .. `<prefix> n`. Returns
/// the tip. Nothing points at it yet.
fn unrelated_chain(repo: &FixtureRepo, prefix: &str, n: usize) -> String {
    let tree = repo.git(&["rev-parse", "HEAD^{tree}"]).trim().to_string();
    let mut parent: Option<String> = None;
    for i in 1..=n {
        let msg = format!("{prefix} {i}");
        let mut args = vec!["commit-tree", &tree, "-m", &msg];
        if let Some(p) = &parent {
            args.push("-p");
            args.push(p);
        }
        parent = Some(repo.git(&args).trim().to_string());
    }
    parent.expect("n > 0")
}

fn all_refs() -> LogOptions {
    LogOptions {
        all_refs: true,
        ..LogOptions::default()
    }
}

fn hidden(names: &[&str], remotes: &[&str], keep: &[&str]) -> LogOptions {
    LogOptions {
        all_refs: true,
        hidden: RefFilter {
            names: names.iter().map(|s| s.to_string()).collect(),
            remotes: remotes.iter().map(|s| s.to_string()).collect(),
            keep: keep.iter().map(|s| s.to_string()).collect(),
        },
        ..LogOptions::default()
    }
}

fn summaries(backend: &dyn GitBackend, repo: &FixtureRepo, opts: &LogOptions) -> Vec<String> {
    backend
        .log(&repo.repo_ref(), opts)
        .unwrap()
        .nodes
        .into_iter()
        .map(|n| n.summary)
        .collect()
}

fn sorted(mut v: Vec<String>) -> Vec<String> {
    v.sort();
    v
}

/// main (`m1`, `m2`), a branch `side` off `m1` (`s1`), and two unrelated
/// archive histories behind remote-tracking refs.
fn two_histories() -> FixtureRepo {
    let repo = TestRepoBuilder::new()
        .commit("m1")
        .branch("side")
        .commit("s1")
        .checkout("main")
        .commit("m2")
        .build();
    let a = unrelated_chain(&repo, "archive a", 3);
    let b = unrelated_chain(&repo, "archive b", 2);
    repo.git(&["update-ref", "refs/remotes/archive/old-a", &a]);
    repo.git(&["update-ref", "refs/remotes/archive/old-b", &b]);
    repo
}

pub fn check_log_hidden_drops_commits_only_hidden_refs_reach(backend: &dyn GitBackend) {
    let repo = two_histories();
    let everything = summaries(backend, &repo, &all_refs());
    assert_eq!(everything.len(), 3 + 3 + 2, "m1 m2 s1 + a1-3 + b1-2");

    let shown = summaries(
        backend,
        &repo,
        &hidden(&["archive/old-a", "archive/old-b", "side"], &[], &[]),
    );
    assert_eq!(
        sorted(shown),
        vec!["m1", "m2"],
        "s1 and the archives go; m1 stays, main still reaches it"
    );

    let only_archive_a = summaries(backend, &repo, &hidden(&["archive/old-b"], &[], &[]));
    assert!(only_archive_a.iter().all(|s| !s.starts_with("archive b")));
    assert_eq!(
        only_archive_a
            .iter()
            .filter(|s| s.starts_with("archive a"))
            .count(),
        3
    );
}

pub fn check_log_hidden_keeps_head_even_if_its_branch_is_hidden(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .commit("m1")
        .branch("topic")
        .commit("t1")
        .build();
    // Checked out: topic, whose t1 only topic reaches.
    let shown = summaries(backend, &repo, &hidden(&["topic", "main"], &[], &[]));
    assert_eq!(sorted(shown), vec!["m1", "t1"], "HEAD stays in the walk");

    // Detached HEAD on a commit no ref reaches.
    repo.git(&["checkout", "-q", "--detach", "HEAD"]);
    repo.git(&["commit", "-q", "--allow-empty", "-m", "loose"]);
    let shown = summaries(backend, &repo, &hidden(&["topic", "main"], &[], &[]));
    assert!(shown.contains(&"loose".to_string()));
}

pub fn check_log_hidden_remote_prefix_covers_nested_and_later_branches(backend: &dyn GitBackend) {
    let repo = two_histories();
    let deep = unrelated_chain(&repo, "archive deep", 1);
    repo.git(&["update-ref", "refs/remotes/archive/grp/deep", &deep]);
    let other = unrelated_chain(&repo, "archive2 x", 1);
    repo.git(&["update-ref", "refs/remotes/archive2/x", &other]);

    let filter = hidden(&[], &["archive"], &[]);
    let shown = summaries(backend, &repo, &filter);
    assert!(
        shown.iter().all(|s| !s.starts_with("archive a")
            && !s.starts_with("archive b")
            && !s.starts_with("archive deep")),
        "every ref under the remote is hidden, nested ones too: {shown:?}"
    );
    assert!(
        shown.contains(&"archive2 x 1".to_string()),
        "a remote whose name merely starts the same stays"
    );

    // A branch fetched later is covered by the same filter.
    let late = unrelated_chain(&repo, "archive late", 1);
    repo.git(&["update-ref", "refs/remotes/archive/late", &late]);
    let shown = summaries(backend, &repo, &filter);
    assert!(!shown.contains(&"archive late 1".to_string()));
    // And the plain walk sees it.
    assert!(summaries(backend, &repo, &all_refs()).contains(&"archive late 1".to_string()));
}

pub fn check_log_hidden_keep_overrides(backend: &dyn GitBackend) {
    let repo = two_histories();
    let shown = summaries(
        backend,
        &repo,
        &hidden(&["archive/old-a", "archive/old-b"], &[], &["archive/old-b"]),
    );
    assert!(shown.contains(&"archive b 2".to_string()));
    assert!(!shown.contains(&"archive a 3".to_string()));

    let shown = summaries(
        backend,
        &repo,
        &hidden(&[], &["archive"], &["archive/old-a"]),
    );
    assert!(shown.contains(&"archive a 3".to_string()));
    assert!(!shown.contains(&"archive b 2".to_string()));
}

pub fn check_log_hidden_pages_tile(backend: &dyn GitBackend) {
    let repo = two_histories();
    let filter = hidden(&["archive/old-b"], &[], &[]);
    let whole: Vec<Oid> = backend
        .log(&repo.repo_ref(), &filter)
        .unwrap()
        .nodes
        .iter()
        .map(|n| n.id)
        .collect();
    assert_eq!(whole.len(), 3 + 3, "m1 m2 s1 + a1-3");
    let mut tiled = Vec::new();
    let mut skip = 0;
    loop {
        let page = backend
            .log(
                &repo.repo_ref(),
                &LogOptions {
                    skip,
                    limit: Some(2),
                    ..filter.clone()
                },
            )
            .unwrap();
        if page.nodes.is_empty() {
            break;
        }
        skip += page.nodes.len() as u32;
        tiled.extend(page.nodes.iter().map(|n| n.id));
    }
    assert_eq!(tiled, whole, "pages tile the unpaged filtered walk");
}

pub fn check_log_hidden_peels_annotated_tags_and_skips_non_commit_tags(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .commit("m1")
        .branch("side")
        .commit("s1")
        .checkout("main")
        .build();
    // An annotated tag is a tip through the commit it peels to.
    repo.git(&["tag", "-a", "-m", "release", "rel", "side"]);
    // Tags of a tree and of a blob point at no commit.
    repo.git(&["tag", "-a", "-m", "tree", "tree-tag", "HEAD^{tree}"]);
    std::fs::write(repo.root().join("blob.txt"), "blob").unwrap();
    let blob = repo
        .git(&["hash-object", "-w", "blob.txt"])
        .trim()
        .to_string();
    repo.git(&["tag", "-a", "-m", "blob", "blob-tag", &blob]);
    std::fs::remove_file(repo.root().join("blob.txt")).unwrap();

    // `side` is hidden but the visible annotated tag still reaches s1.
    let shown = summaries(backend, &repo, &hidden(&["side"], &[], &[]));
    assert_eq!(sorted(shown), vec!["m1", "s1"]);
    // Hiding the tag as well drops s1.
    let shown = summaries(backend, &repo, &hidden(&["side", "rel"], &[], &[]));
    assert_eq!(shown, vec!["m1"]);
}

pub fn check_count_commits_honours_hidden(backend: &dyn GitBackend) {
    counter_honours_hidden(backend, &|r, o| backend.count_commits(r, o));
}

/// The same for any counter that must agree with `backend`'s `log` (the
/// in-process libgit2 count).
pub fn counter_honours_hidden(backend: &dyn GitBackend, counter: &Counter<'_>) {
    let repo = two_histories();
    let r = repo.repo_ref();
    for opts in [
        hidden(&["archive/old-a", "archive/old-b"], &[], &[]),
        hidden(&[], &["archive"], &[]),
        hidden(&["side"], &["archive"], &["archive/old-a"]),
    ] {
        let walked = backend.log(&r, &opts).unwrap().nodes.len() as u64;
        assert_eq!(counter(&r, &opts).unwrap(), walked, "{:?}", opts.hidden);
    }
    assert_eq!(counter(&r, &hidden(&[], &["archive"], &[])).unwrap(), 3);
    assert_eq!(counter(&r, &all_refs()).unwrap(), 8);

    // Unborn repository: nothing to walk, nothing to count.
    let empty = TestRepoBuilder::new().build();
    let opts = hidden(&["x"], &[], &[]);
    assert_eq!(counter(&empty.repo_ref(), &opts).unwrap(), 0);
    assert!(backend
        .log(&empty.repo_ref(), &opts)
        .unwrap()
        .nodes
        .is_empty());
}

pub fn check_empty_filter_is_the_all_refs_walk(backend: &dyn GitBackend) {
    let repo = two_histories();
    let plain = summaries(backend, &repo, &all_refs());
    // `keep` alone hides nothing: the empty filter, the same walk.
    let kept_only = summaries(backend, &repo, &hidden(&[], &[], &["side"]));
    assert_eq!(kept_only, plain);
    // A filter naming nothing that exists drops nothing either.
    let unmatched = summaries(backend, &repo, &hidden(&["nope"], &["nowhere"], &[]));
    assert_eq!(sorted(unmatched), sorted(plain));
}
