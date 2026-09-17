use super::*;
use crate::dispatch::dispatch_cached;
use gitpoulpi_core::domain::{LogOptions, OperationKind, RepoRef};
use gitpoulpi_core::GitBackend;
use gitpoulpi_git2_backend::Git2Backend;
use gitpoulpi_ipc::Request;
use gitpoulpi_testkit::FixtureRepoBuilder;

fn read(
    cache: &ReadCache,
    backend: &dyn GitBackend,
    repo: &RepoRef,
    command: OperationKind,
) -> Value {
    let args = match command {
        OperationKind::Log | OperationKind::CountCommits => serde_json::json!({
            "opts": LogOptions { all_refs: true, ..Default::default() }
        }),
        _ => serde_json::json!({}),
    };
    let request = Request {
        id: 1,
        session: "cache-test".into(),
        repo: Some(repo.id.clone()),
        command,
        args,
    };
    dispatch_cached(cache, backend, &request)
        .response
        .result
        .expect("read succeeded")
}

/// Mtimes tick coarsely on some filesystems; a pause makes a change
/// visible to the fingerprint without leaning on `invalidate`.
fn tick() {
    std::thread::sleep(Duration::from_millis(60));
}

fn summaries(log: &Value) -> Vec<String> {
    log["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| n["summary"].as_str().unwrap().to_string())
        .collect()
}

#[test]
fn log_and_refs_hit_until_a_ref_moves() {
    let fixture = FixtureRepoBuilder::new()
        .unwrap()
        .commit("first")
        .unwrap()
        .build();
    let repo = fixture.repo_ref();
    let backend = Git2Backend::new();
    let cache = ReadCache::with_racy_window(Duration::ZERO);
    tick();

    let log1 = read(&cache, &backend, &repo, OperationKind::Log);
    let refs1 = read(&cache, &backend, &repo, OperationKind::Refs);
    assert_eq!((cache.hits(), cache.misses()), (0, 2));

    // Unchanged refs: both answers come from the cache, identical.
    assert_eq!(read(&cache, &backend, &repo, OperationKind::Log), log1);
    assert_eq!(read(&cache, &backend, &repo, OperationKind::Refs), refs1);
    assert_eq!((cache.hits(), cache.misses()), (2, 2));

    // A new branch is a new ref file: Refs misses and lists it.
    let head = backend.log(&repo, &LogOptions::default()).unwrap().nodes[0].id;
    backend.branch_create(&repo, "topic", head).unwrap();
    tick();
    let refs2 = read(&cache, &backend, &repo, OperationKind::Refs);
    assert_eq!(cache.misses(), 3);
    assert!(refs2.to_string().contains("topic"));

    // A commit moves the branch ref: Log misses and has the new row.
    fixture.git(&["commit", "--allow-empty", "-m", "second"]);
    tick();
    let log2 = read(&cache, &backend, &repo, OperationKind::Log);
    assert!(summaries(&log2).contains(&"second".to_string()));
    assert_eq!(summaries(&log1).len() + 1, summaries(&log2).len());
    assert_eq!(cache.misses(), 4);
}

/// The status bar's total: computed once per ref state, served from the
/// cache on every later poll, recomputed only after a ref moves.
#[test]
fn the_commit_total_is_cached_until_a_ref_moves() {
    let fixture = FixtureRepoBuilder::new()
        .unwrap()
        .commit("first")
        .unwrap()
        .commit("second")
        .unwrap()
        .build();
    let repo = fixture.repo_ref();
    let backend = Git2Backend::new();
    let cache = ReadCache::with_racy_window(Duration::ZERO);
    tick();

    assert_eq!(
        read(&cache, &backend, &repo, OperationKind::CountCommits),
        serde_json::json!(2)
    );
    assert_eq!((cache.hits(), cache.misses()), (0, 1));
    for _ in 0..3 {
        assert_eq!(
            read(&cache, &backend, &repo, OperationKind::CountCommits),
            serde_json::json!(2)
        );
    }
    assert_eq!(
        (cache.hits(), cache.misses()),
        (3, 1),
        "same refs must not recount"
    );

    fixture.git(&["commit", "--allow-empty", "-m", "third"]);
    tick();
    assert_eq!(
        read(&cache, &backend, &repo, OperationKind::CountCommits),
        serde_json::json!(3)
    );
    assert_eq!(cache.misses(), 2, "a new commit moves a ref and recounts");
}

#[test]
fn a_fetch_that_moves_remote_refs_is_not_served_stale() {
    let fixture = FixtureRepoBuilder::new()
        .unwrap()
        .commit("first")
        .unwrap()
        .with_origin()
        .unwrap()
        .push("main")
        .unwrap()
        .track("main", "origin")
        .unwrap()
        .build();
    let repo = fixture.repo_ref();
    let backend = Git2Backend::new();
    let cache = ReadCache::with_racy_window(Duration::ZERO);
    tick();

    let before = read(&cache, &backend, &repo, OperationKind::Log);
    assert_eq!(read(&cache, &backend, &repo, OperationKind::Log), before);
    assert_eq!(cache.hits(), 1);

    fixture.push_from_elsewhere("main", "elsewhere");
    tick();
    fixture.git(&["fetch", "origin"]);
    tick();
    let after = read(&cache, &backend, &repo, OperationKind::Log);
    assert!(summaries(&after).contains(&"elsewhere".to_string()));
    assert_eq!(summaries(&before).len() + 1, summaries(&after).len());
}

#[test]
fn invalidate_drops_a_repos_entries_and_fresh_refs_are_never_cached() {
    let fixture = FixtureRepoBuilder::new()
        .unwrap()
        .commit("first")
        .unwrap()
        .build();
    let repo = fixture.repo_ref();
    let backend = Git2Backend::new();

    let cache = ReadCache::with_racy_window(Duration::ZERO);
    tick();
    read(&cache, &backend, &repo, OperationKind::Refs);
    cache.invalidate(&repo.id);
    read(&cache, &backend, &repo, OperationKind::Refs);
    assert_eq!((cache.hits(), cache.misses()), (0, 2));

    // Files written moments ago are inside the racy window.
    let racy = ReadCache::with_racy_window(Duration::from_secs(3600));
    read(&racy, &backend, &repo, OperationKind::Refs);
    read(&racy, &backend, &repo, OperationKind::Refs);
    assert_eq!((racy.hits(), racy.misses()), (0, 2));
}

/// Commit stats are asked of the backend once per commit: a second
/// request for the same ids, with or without a ref moving, is served from
/// memory, and a mixed request only asks for the unknown ones.
#[test]
fn commit_stats_are_cached_by_commit_and_survive_invalidation() {
    let fixture = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "1\n2\n", "one")
        .unwrap()
        .commit_file("a.txt", "1\n2\n3\n", "two")
        .unwrap()
        .build();
    let repo = fixture.repo_ref();
    let backend = Git2Backend::new();
    let cache = ReadCache::default();
    let nodes = backend.log(&repo, &LogOptions::default()).unwrap().nodes;
    let (newest, oldest) = (nodes[0].id, nodes[1].id);
    let ask = |ids: &[gitpoulpi_core::domain::Oid]| -> Vec<gitpoulpi_core::domain::CommitStat> {
        let request = Request {
            id: 1,
            session: "cache-test".into(),
            repo: Some(repo.id.clone()),
            command: OperationKind::CommitStats,
            args: serde_json::json!({ "ids": ids }),
        };
        let value = dispatch_cached(&cache, &backend, &request)
            .response
            .result
            .expect("stats");
        serde_json::from_value(value).unwrap()
    };

    let (known, missing) = cache.stats_lookup(&repo.id, &[newest, oldest]);
    assert!((known.len(), missing.len()) == (0, 2));
    let first = ask(&[newest]);
    assert_eq!((first[0].files, first[0].insertions), (1, 1));
    let (known, missing) = cache.stats_lookup(&repo.id, &[newest, oldest, oldest]);
    assert_eq!((known.len(), missing), (1, vec![oldest]));

    // Mixed batch keeps the caller's order, and a ref-moving mutation's
    // invalidation does not forget immutable commits.
    cache.invalidate(&repo.id);
    let both = ask(&[oldest, newest]);
    assert_eq!(
        both.iter().map(|s| s.id).collect::<Vec<_>>(),
        [oldest, newest]
    );
    assert_eq!(both[0].insertions, 2);
    let (known, missing) = cache.stats_lookup(&repo.id, &[newest, oldest]);
    assert!((known.len(), missing.len()) == (2, 0));
}

fn conditional_log(
    cache: &ReadCache,
    backend: &dyn GitBackend,
    repo: &RepoRef,
    etag: Option<u64>,
) -> Vec<u8> {
    let mut args = serde_json::json!({
        "opts": LogOptions { all_refs: true, ..Default::default() }
    });
    if let Some(etag) = etag {
        args = gitpoulpi_ipc::etag::with_if_none_match(args, etag);
    }
    let request = Request {
        id: 1,
        session: "cache-test".into(),
        repo: Some(repo.id.clone()),
        command: OperationKind::Log,
        args,
    };
    match crate::dispatch::dispatch_cached_reply(cache, backend, &request) {
        crate::dispatch::CachedReply::Json { json, .. } => json.to_vec(),
        crate::dispatch::CachedReply::Dispatched(_) => panic!("the read failed"),
    }
}

#[test]
fn a_matching_etag_is_answered_not_modified_until_the_log_changes() {
    let fixture = FixtureRepoBuilder::new()
        .unwrap()
        .commit("first")
        .unwrap()
        .build();
    let repo = fixture.repo_ref();
    let backend = Git2Backend::new();
    let cache = ReadCache::with_racy_window(Duration::ZERO);
    tick();

    // No etag: the full answer, whose etag the client can compute itself.
    let full = conditional_log(&cache, &backend, &repo, None);
    let value: Value = serde_json::from_slice(&full).unwrap();
    assert!(!gitpoulpi_ipc::etag::is_not_modified(&value));
    let etag = gitpoulpi_ipc::etag::of_value(&value);
    assert_eq!(etag, gitpoulpi_ipc::etag::of(&full));

    // The held etag, served from the cache: nothing but the marker.
    assert_eq!(
        conditional_log(&cache, &backend, &repo, Some(etag)),
        gitpoulpi_ipc::etag::NOT_MODIFIED_JSON
    );
    assert_eq!(
        cache.misses(),
        1,
        "the conditional read still hit the cache"
    );

    // A stale etag gets the full answer.
    assert_eq!(
        conditional_log(&cache, &backend, &repo, Some(etag ^ 1)),
        full
    );

    // A moved ref changes the answer, so the old etag no longer matches.
    fixture.git(&["commit", "--allow-empty", "-m", "second"]);
    cache.invalidate(&repo.id);
    let fresh = conditional_log(&cache, &backend, &repo, Some(etag));
    assert_ne!(fresh, gitpoulpi_ipc::etag::NOT_MODIFIED_JSON);
    assert_eq!(
        summaries(&serde_json::from_slice(&fresh).unwrap())[0],
        "second"
    );
}

/// A reftable repository keeps no ref files under `refs/`: a new commit must
/// still change the fingerprint, or the cache would serve a stale Log.
#[test]
fn the_fingerprint_sees_a_change_in_a_reftable_repository() {
    let Ok(builder) = FixtureRepoBuilder::new_reftable() else {
        eprintln!("SKIPPED: this git cannot create a reftable repository");
        return;
    };
    let fixture = builder.commit("first").unwrap().build();
    tick();
    let before = fingerprint(fixture.root(), Duration::ZERO).expect("a fingerprint");
    assert_eq!(
        fingerprint(fixture.root(), Duration::ZERO),
        Some(before),
        "stable while nothing moves"
    );
    fixture.git(&["commit", "--allow-empty", "-m", "second"]);
    tick();
    assert_ne!(fingerprint(fixture.root(), Duration::ZERO), Some(before));
}

/// The workflow answer depends on two files outside git's own state: editing
/// `workflow.toml` must not be served from the cache, an untouched file is.
#[test]
fn the_workflow_read_is_cached_but_follows_its_file() {
    let fixture = FixtureRepoBuilder::new()
        .unwrap()
        .commit("first")
        .unwrap()
        .build();
    let repo = fixture.repo_ref();
    let backend = Git2Backend::new();
    let cache = ReadCache::with_racy_window(Duration::ZERO);
    tick();

    let before = read(&cache, &backend, &repo, OperationKind::Workflow);
    assert_eq!(before["configured"], false);
    let hits = cache.hits();
    let again = read(&cache, &backend, &repo, OperationKind::Workflow);
    assert_eq!(again, before);
    assert_eq!(cache.hits(), hits + 1, "an untouched repository is a hit");

    let dir = repo.root.join(".git").join("gitpoulpi");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("workflow.toml"),
        "template = \"trunk\"\n[[base]]\nname = \"main\"\n",
    )
    .unwrap();
    let after = read(&cache, &backend, &repo, OperationKind::Workflow);
    assert_eq!(after["configured"], true);
    assert_eq!(after["branches"][0]["kind"], "base");
    tick();
    std::fs::write(dir.join("workflow.toml"), "template = \"custom\"\n").unwrap();
    let emptied = read(&cache, &backend, &repo, OperationKind::Workflow);
    assert_eq!(
        emptied["configured"], false,
        "a rewritten file is read again"
    );
}

/// `log.commit_count = "libgit2"` counts in-process even when the session
/// runs the system-git backend, and the method is part of the cache key, so
/// switching it recounts instead of serving the other method's answer.
#[test]
fn the_libgit2_count_method_is_its_own_cache_entry_and_works_on_either_backend() {
    let fixture = FixtureRepoBuilder::new()
        .unwrap()
        .commit("first")
        .unwrap()
        .commit("second")
        .unwrap()
        .build();
    let repo = fixture.repo_ref();
    let backend = gitpoulpi_cli_backend::SystemGitBackend::new();
    let cache = ReadCache::with_racy_window(Duration::ZERO);
    tick();
    let count = |method: Option<&str>| {
        let mut args = serde_json::json!({
            "opts": LogOptions { all_refs: true, ..Default::default() }
        });
        if let Some(m) = method {
            args["method"] = m.into();
        }
        let request = Request {
            id: 1,
            session: "cache-test".into(),
            repo: Some(repo.id.clone()),
            command: OperationKind::CountCommits,
            args,
        };
        dispatch_cached(&cache, &backend, &request)
            .response
            .result
            .expect("count succeeded")
    };
    assert_eq!(count(None), serde_json::json!(2));
    let misses = cache.misses();
    assert_eq!(count(Some("libgit2")), serde_json::json!(2));
    assert_eq!(cache.misses(), misses + 1, "another method recounts");
    assert_eq!(count(Some("libgit2")), serde_json::json!(2));
    assert_eq!(cache.misses(), misses + 1, "and is cached on its own");
}
