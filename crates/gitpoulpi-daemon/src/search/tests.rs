use super::*;
use gitpoulpi_core::domain::SearchKind as K;
use gitpoulpi_testkit::TestRepoBuilder;

const FULL: usize = 200_000;

fn backends() -> Vec<(&'static str, Arc<dyn GitBackend>)> {
    vec![
        ("git2", Arc::new(gitpoulpi_git2_backend::Git2Backend::new())),
        (
            "system",
            Arc::new(gitpoulpi_cli_backend::SystemGitBackend::new()),
        ),
    ]
}

fn ask(
    idx: &SearchIndexes,
    backend: &Arc<dyn GitBackend>,
    repo: &RepoRef,
    query: &str,
    wait: bool,
) -> SearchHits {
    answer(
        idx,
        backend,
        repo,
        &SearchRequest {
            query: query.into(),
            kinds: Vec::new(),
            limit: 0,
            wait,
        },
        FULL,
    )
}

fn labels(hits: &SearchHits, kind: K) -> Vec<String> {
    hits.hits
        .iter()
        .filter(|h| h.kind == kind)
        .map(|h| h.label.clone())
        .collect()
}

/// Every kind is found, with the target a client navigates by, on both
/// backends.
#[test]
fn every_kind_is_found_with_its_navigation_target() {
    let repo = TestRepoBuilder::new()
        .commit("first login page")
        .branch("feature/login")
        .commit("login validation")
        .checkout("main")
        .tag("v1-login")
        .build();
    std::fs::write(repo.root().join("dirty.txt"), "x").unwrap();
    repo.git(&["add", "dirty.txt"]);
    repo.git(&["stash", "push", "-q", "-m", "park the login work"]);
    let linked = repo.root().parent().unwrap().join("login-tree");
    repo.git(&[
        "worktree",
        "add",
        "-q",
        "-b",
        "wt-login",
        linked.to_str().unwrap(),
    ]);
    let repo_ref = repo.repo_ref();

    for (name, backend) in backends() {
        let idx = SearchIndexes::default();
        let hits = ask(&idx, &backend, &repo_ref, "login", true);
        assert!(!hits.partial, "{name}");
        assert_eq!(
            labels(&hits, K::Branch),
            ["wt-login", "feature/login"],
            "{name}"
        );
        assert_eq!(labels(&hits, K::Tag), ["v1-login"], "{name}");
        assert_eq!(
            labels(&hits, K::Stash),
            ["On main: park the login work"],
            "{name}"
        );
        assert_eq!(labels(&hits, K::Worktree), ["login-tree"], "{name}");
        let commits = labels(&hits, K::Commit);
        assert!(commits.contains(&"first login page".to_string()), "{name}");
        assert!(commits.contains(&"login validation".to_string()), "{name}");

        let branch = hits
            .hits
            .iter()
            .find(|h| h.label == "feature/login")
            .unwrap();
        assert_eq!(
            branch.target,
            SearchTarget::Ref {
                name: "refs/heads/feature/login".into()
            }
        );
        let stash = hits.hits.iter().find(|h| h.kind == K::Stash).unwrap();
        assert!(
            matches!(
                &stash.target,
                SearchTarget::Stash {
                    index: 0,
                    id: Some(_)
                }
            ),
            "{name}"
        );
        let tree = hits.hits.iter().find(|h| h.kind == K::Worktree).unwrap();
        // git (and libgit2) list a worktree with forward slashes on
        // Windows, and resolve the macOS `/var` link; compare the same
        // spelling.
        assert_eq!(
            tree.target,
            SearchTarget::Worktree {
                path: gitpoulpi_core::paths::canonicalize_display(&linked)
                    .to_string_lossy()
                    .replace('\\', "/")
            },
            "{name}"
        );
        let commit = hits
            .hits
            .iter()
            .find(|h| h.label == "login validation")
            .unwrap();
        assert_eq!(
            commit.target,
            SearchTarget::Commit {
                id: Oid::from_hex(&repo.git(&["rev-parse", "feature/login"])).unwrap()
            }
        );
        // The matched characters are named.
        assert_eq!(commit.positions, [0, 1, 2, 3, 4], "{name}");
    }
}

#[test]
fn a_prefix_narrows_to_one_kind_and_is_not_part_of_the_text() {
    let repo = TestRepoBuilder::new()
        .commit("login commit")
        .branch("login-branch")
        .tag("login-tag")
        .build();
    let repo_ref = repo.repo_ref();
    let backend = backends().remove(0).1;
    let idx = SearchIndexes::default();
    for (query, kind, label) in [
        ("b: login", K::Branch, "login-branch"),
        ("t: login", K::Tag, "login-tag"),
        ("c: login", K::Commit, "login commit"),
    ] {
        let hits = ask(&idx, &backend, &repo_ref, query, true);
        assert!(hits.hits.iter().all(|h| h.kind == kind), "{query}");
        assert_eq!(hits.hits[0].label, label, "{query}");
    }
    // Only the filter given, without a prefix.
    let only_tags = answer(
        &idx,
        &backend,
        &repo_ref,
        &SearchRequest {
            query: "login".into(),
            kinds: vec![K::Tag],
            limit: 0,
            wait: true,
        },
        FULL,
    );
    assert!(only_tags.hits.iter().all(|h| h.kind == K::Tag));
}

/// A tighter match ranks first; a fuzzy (scattered) one still matches, and a
/// body only matches when the subject does not.
#[test]
fn hits_rank_by_tier_and_bodies_match_last() {
    let repo = TestRepoBuilder::new()
        .commit("unrelated")
        .commit_with_body("tidy things", "mentions the frobnicator in passing")
        .commit("fix: frobnicator crash")
        .commit("frobnicate everything")
        .commit("find roots of bananas near ice")
        .build();
    let repo_ref = repo.repo_ref();
    let backend = backends().remove(0).1;
    let hits = ask(
        &SearchIndexes::default(),
        &backend,
        &repo_ref,
        "c: frobnic",
        true,
    );
    let order: Vec<_> = hits.hits.iter().map(|h| h.label.as_str()).collect();
    // Verbatim subjects, then the scattered subject, then the body.
    assert_eq!(
        order,
        [
            "frobnicate everything",
            "fix: frobnicator crash",
            "find roots of bananas near ice",
            "tidy things"
        ]
    );
    assert!(
        hits.hits[3].positions.is_empty(),
        "a body match highlights nothing"
    );
    // A scattered match is found but ranks under the verbatim ones.
    let hits = ask(
        &SearchIndexes::default(),
        &backend,
        &repo_ref,
        "c: frbnc",
        true,
    );
    assert!(hits
        .hits
        .iter()
        .any(|h| h.label == "fix: frobnicator crash"));
}

/// The first answer of a cold repository does not wait for the commits (the
/// refs come at once, `partial` says the rest is coming); asked again it
/// completes. A caller that waits gets everything at once.
#[test]
fn a_cold_search_answers_at_once_then_completes() {
    let repo = TestRepoBuilder::new()
        .commit("login one")
        .branch("login-branch")
        .build();
    let repo_ref = repo.repo_ref();
    let backend = backends().remove(0).1;
    let idx = SearchIndexes::default();
    let first = ask(&idx, &backend, &repo_ref, "login", false);
    // The branch (a small item) is in the first answer, whatever the thread
    // has done by now.
    assert_eq!(labels(&first, K::Branch), ["login-branch"]);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    loop {
        let again = ask(&idx, &backend, &repo_ref, "login", false);
        if !again.partial {
            assert_eq!(labels(&again, K::Commit), ["login one"]);
            break;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "the build never finished"
        );
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
}

/// New commits are found after the mutation's invalidation; a rewrite
/// (amend) replaces the old text; a deleted branch stops being a hit.
#[test]
fn the_index_follows_commits_amends_and_deleted_refs() {
    for (name, backend) in backends() {
        let repo = TestRepoBuilder::new()
            .commit("alpha base")
            .branch("temp-branch")
            .checkout("main")
            .build();
        let repo_ref = repo.repo_ref();
        let idx = SearchIndexes::default();
        let found = |q: &str, kind| labels(&ask(&idx, &backend, &repo_ref, q, true), kind);
        assert_eq!(found("alpha", K::Commit), ["alpha base"], "{name}");
        assert_eq!(found("temp", K::Branch), ["temp-branch"], "{name}");

        repo.git(&["commit", "-q", "--allow-empty", "-m", "beta new"]);
        idx.invalidate(&repo_ref.id);
        assert_eq!(found("beta", K::Commit), ["beta new"], "{name}");
        assert_eq!(found("alpha", K::Commit), ["alpha base"], "{name}");

        repo.git(&[
            "commit",
            "-q",
            "--amend",
            "--allow-empty",
            "-m",
            "beta amended",
        ]);
        idx.invalidate(&repo_ref.id);
        assert_eq!(found("beta", K::Commit), ["beta amended"], "{name}");

        repo.git(&["branch", "-q", "-D", "temp-branch"]);
        idx.invalidate(&repo_ref.id);
        assert!(found("temp", K::Branch).is_empty(), "{name}");
    }
}

/// `search.commit_limit` keeps the newest commits and says the answer is
/// partial; 0 turns commit search off.
#[test]
fn the_commit_limit_cuts_and_says_so() {
    let repo = TestRepoBuilder::new()
        .commit("c-one")
        .commit("c-two")
        .commit("c-three")
        .build();
    let repo_ref = repo.repo_ref();
    let backend = backends().remove(0).1;
    let request = |limit: usize| {
        answer(
            &SearchIndexes::default(),
            &backend,
            &repo_ref,
            &SearchRequest {
                query: "c-".into(),
                kinds: Vec::new(),
                limit: 0,
                wait: true,
            },
            limit,
        )
    };
    let cut = request(2);
    assert!(cut.partial);
    assert_eq!(labels(&cut, K::Commit).len(), 2);
    let none = request(0);
    assert!(labels(&none, K::Commit).is_empty() && !none.partial);
    assert!(!request(FULL).partial);
}

#[test]
fn an_empty_query_lists_without_scoring() {
    let repo = TestRepoBuilder::new().commit("one").branch("b1").build();
    let repo_ref = repo.repo_ref();
    let backend = backends().remove(0).1;
    let hits = ask(&SearchIndexes::default(), &backend, &repo_ref, "", true);
    assert!(hits.hits.iter().any(|h| h.kind == K::Branch));
    assert!(hits.hits.iter().any(|h| h.kind == K::Commit));
    assert!(hits
        .hits
        .iter()
        .all(|h| h.score == 0 && h.positions.is_empty()));
}

/// The limit applies to the merged list, best first.
#[test]
fn the_hit_limit_applies_after_merging_kinds() {
    let mut b = TestRepoBuilder::new().commit("seed");
    for i in 0..6 {
        b = b.branch(&format!("topic-{i}"));
    }
    let repo = b.build();
    let repo_ref = repo.repo_ref();
    let backend = backends().remove(0).1;
    let hits = answer(
        &SearchIndexes::default(),
        &backend,
        &repo_ref,
        &SearchRequest {
            query: "topic".into(),
            kinds: Vec::new(),
            limit: 3,
            wait: true,
        },
        FULL,
    );
    assert_eq!(hits.hits.len(), 3);
    assert!(hits.hits.windows(2).all(|w| w[0].score >= w[1].score));
}
