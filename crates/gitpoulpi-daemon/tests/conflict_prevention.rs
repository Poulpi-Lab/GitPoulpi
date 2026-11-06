//! Proactive conflict detection in the daemon: the base it picks, the cache
//! keyed by `(head, base)`, the debounce and the settings. Its own test
//! binary: it points the config at a scratch directory through the process
//! environment.

use std::sync::{Arc, Once};
use std::time::Duration;

use gitpoulpi_core::domain::conflict_prevention::{
    BaseSource, ConflictPrediction, PredictionStatus,
};
use gitpoulpi_core::domain::OperationKind;
use gitpoulpi_core::GitBackend;
use gitpoulpi_daemon::conflict_prevention::Predictor;
use gitpoulpi_daemon::Daemon;
use gitpoulpi_ipc::{Client, Request};
use gitpoulpi_session::RepoState;
use gitpoulpi_testkit::prediction_fixture::{conflicting_feature, fix_the_conflict};
use gitpoulpi_testkit::FixtureRepo;

fn scratch_config() {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        let dir = tempfile::tempdir().unwrap().keep();
        std::env::set_var("GITPOULPI_CONFIG_DIR", &dir);
        std::env::set_var("XDG_CONFIG_HOME", &dir);
        std::env::set_var("HOME", &dir);
    });
}

fn backend() -> Arc<dyn GitBackend> {
    scratch_config();
    Arc::new(gitpoulpi_git2_backend::Git2Backend::new())
}

fn state_path(repo: &FixtureRepo) -> std::path::PathBuf {
    RepoState::path_for(repo.root(), gitpoulpi_config::DEFAULT_REPO_DATA_LOCATION)
}

fn set_prefs(repo: &FixtureRepo, edit: impl FnOnce(&mut gitpoulpi_session::RepoPreferences)) {
    let path = state_path(repo);
    let mut state = RepoState::load(&path).unwrap_or_default();
    edit(&mut state.preferences);
    state.save(&path).unwrap();
}

#[test]
fn it_names_the_conflicting_file_against_the_only_remote_branch() {
    let backend = backend();
    let repo = conflicting_feature();
    let predictor = Predictor::default();
    let p = predictor
        .predict_now(backend.as_ref(), &repo.repo_ref())
        .unwrap();
    assert_eq!(p.status, PredictionStatus::Conflicts);
    assert_eq!(p.base_ref.as_deref(), Some("origin/main"));
    assert_eq!(p.base_source, Some(BaseSource::Globs));
    assert_eq!(p.files, ["a.txt"]);
    assert_eq!(
        p.head.as_deref(),
        Some(repo.git(&["rev-parse", "HEAD"]).trim())
    );
}

#[test]
fn it_is_cached_by_head_and_base_and_recomputed_when_either_moves() {
    let backend = backend();
    let repo = conflicting_feature();
    let predictor = Predictor::default();
    let r = repo.repo_ref();
    for _ in 0..3 {
        predictor.predict_now(backend.as_ref(), &r).unwrap();
    }
    assert_eq!(predictor.merges_run(), 1, "same head and base: one merge");

    // HEAD moves (a new commit that does not fix anything): a new merge.
    repo.git(&["commit", "-q", "--allow-empty", "-m", "more"]);
    let p = predictor.predict_now(backend.as_ref(), &r).unwrap();
    assert_eq!(p.status, PredictionStatus::Conflicts);
    assert_eq!(predictor.merges_run(), 2);

    // Fixing the conflict on the branch moves HEAD again: clean, no stale answer.
    fix_the_conflict(&repo);
    let p = predictor.predict_now(backend.as_ref(), &r).unwrap();
    assert_eq!(p.status, PredictionStatus::Clean);
    assert!(p.files.is_empty());
    assert_eq!(predictor.merges_run(), 3);

    // The base moves: origin/main gets a commit; head is unchanged.
    repo.push_from_elsewhere("main", "elsewhere");
    repo.git(&["fetch", "-q", "origin"]);
    predictor.predict_now(backend.as_ref(), &r).unwrap();
    assert_eq!(predictor.merges_run(), 4);
}

#[test]
fn a_branch_that_contains_or_is_contained_needs_no_merge() {
    let backend = backend();
    let repo = conflicting_feature();
    // Take origin/main in: the base is now an ancestor of HEAD.
    repo.git(&["merge", "-q", "-X", "theirs", "-m", "merge", "origin/main"]);
    let predictor = Predictor::default();
    let p = predictor
        .predict_now(backend.as_ref(), &repo.repo_ref())
        .unwrap();
    assert_eq!(p.status, PredictionStatus::Clean);
    assert_eq!(
        predictor.merges_run(),
        0,
        "nothing diverged, nothing merged"
    );
}

#[test]
fn a_debounced_burst_costs_one_merge_and_the_older_requests_are_superseded() {
    let backend = backend();
    let repo = conflicting_feature();
    let predictor = Arc::new(Predictor::default());
    let r = repo.repo_ref();
    let debounce = Duration::from_millis(300);
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    let answers: Vec<ConflictPrediction> = rt.block_on(async {
        let mut tasks = Vec::new();
        for _ in 0..3 {
            let (p, b, r) = (Arc::clone(&predictor), Arc::clone(&backend), r.clone());
            tasks.push(tokio::spawn(async move {
                p.predict_debounced(b, r, debounce).await.unwrap()
            }));
            tokio::time::sleep(Duration::from_millis(40)).await;
        }
        let mut out = Vec::new();
        for t in tasks {
            out.push(t.await.unwrap());
        }
        out
    });
    assert_eq!(answers[0].status, PredictionStatus::Superseded);
    assert_eq!(answers[1].status, PredictionStatus::Superseded);
    assert_eq!(answers[2].status, PredictionStatus::Conflicts);
    assert_eq!(predictor.merges_run(), 1);
    // Asked again with the same head and base: served from the cache, at once.
    let started = std::time::Instant::now();
    let again = rt.block_on(predictor.predict_debounced(Arc::clone(&backend), r, debounce));
    assert_eq!(again.unwrap().status, PredictionStatus::Conflicts);
    assert!(
        started.elapsed() < debounce,
        "a cached answer does not wait"
    );
    assert_eq!(predictor.merges_run(), 1);
}

#[test]
fn the_settings_turn_it_off_and_choose_the_base() {
    let backend = backend();
    let repo = conflicting_feature();
    let r = repo.repo_ref();
    let predictor = Predictor::default();

    set_prefs(&repo, |p| p.conflict_detection = Some(false));
    let p = predictor.predict_now(backend.as_ref(), &r).unwrap();
    assert_eq!(p.status, PredictionStatus::Disabled);
    assert_eq!(predictor.merges_run(), 0);
    set_prefs(&repo, |p| p.conflict_detection = None);
    assert_eq!(
        predictor.predict_now(backend.as_ref(), &r).unwrap().status,
        PredictionStatus::Conflicts
    );

    // The globs allow no branch: nothing to compare with.
    set_prefs(&repo, |p| {
        p.conflict_base_branches = Some("upstream/**".into())
    });
    assert_eq!(
        predictor.predict_now(backend.as_ref(), &r).unwrap().status,
        PredictionStatus::NoBase
    );
    // An exclusion of the only candidate too.
    set_prefs(&repo, |p| {
        p.conflict_base_branches = Some("!origin/main".into())
    });
    assert_eq!(
        predictor.predict_now(backend.as_ref(), &r).unwrap().status,
        PredictionStatus::NoBase
    );
}

#[test]
fn the_base_is_the_branch_it_started_from_and_a_known_pull_request_wins() {
    let backend = backend();
    let repo = conflicting_feature();
    let r = repo.repo_ref();
    let predictor = Predictor::default();
    // `develop` is `origin/main` plus one commit, and feature moves on from
    // it: feature has fewer commits since the fork from develop than from main.
    repo.git(&["branch", "-q", "develop", "HEAD~1"]);
    repo.git(&["checkout", "-q", "develop"]);
    repo.git(&["commit", "-q", "--allow-empty", "-m", "on develop"]);
    repo.git(&["checkout", "-q", "-B", "feature2", "develop"]);
    std::fs::write(repo.root().join("c.txt"), "c\n").unwrap();
    repo.git(&["add", "c.txt"]);
    repo.git(&["commit", "-q", "-m", "feature2 work"]);
    let p = predictor.predict_now(backend.as_ref(), &r).unwrap();
    assert_eq!(p.base_ref.as_deref(), Some("develop"));

    // Excluding it leaves origin/main.
    set_prefs(&repo, |p| {
        p.conflict_base_branches = Some("**\n!develop".into())
    });
    let p = predictor.predict_now(backend.as_ref(), &r).unwrap();
    assert_eq!(p.base_ref.as_deref(), Some("origin/main"));

    // A known pull request (the TUI's hosting cache) wins over the globs.
    set_prefs(&repo, |p| p.conflict_base_branches = None);
    let cache = RepoState::hosting_cache_path_for(
        repo.root(),
        gitpoulpi_config::DEFAULT_REPO_DATA_LOCATION,
    );
    std::fs::create_dir_all(cache.parent().unwrap()).unwrap();
    std::fs::write(
        &cache,
        r#"{"remote":"origin","prs":[
            {"source_branch":"feature2","target_branch":"main","state":"open"},
            {"source_branch":"other","target_branch":"develop","state":"open"}]}"#,
    )
    .unwrap();
    let p = predictor.predict_now(backend.as_ref(), &r).unwrap();
    assert_eq!(p.base_ref.as_deref(), Some("origin/main"));
    assert_eq!(p.base_source, Some(BaseSource::PullRequest));
    // A merged pull request does not count.
    std::fs::write(
        &cache,
        r#"{"remote":"origin","prs":[
            {"source_branch":"feature2","target_branch":"main","state":"merged"}]}"#,
    )
    .unwrap();
    let p = predictor.predict_now(backend.as_ref(), &r).unwrap();
    assert_eq!(p.base_ref.as_deref(), Some("develop"));
}

#[tokio::test]
async fn the_request_goes_through_the_daemon_with_and_without_a_debounce() {
    let backend = backend();
    let repo = conflicting_feature();
    let handle = Daemon::new("conflict-prevention")
        .spawn(backend)
        .await
        .unwrap();
    let mut client = Client::connect(&handle.endpoint()).await.unwrap();
    let id = repo.repo_ref().id;
    let ask = |n: u64, args: serde_json::Value| Request {
        id: n,
        session: "conflict-prevention".into(),
        repo: Some(id.clone()),
        command: OperationKind::PredictConflicts,
        args,
    };
    let now: ConflictPrediction = serde_json::from_value(
        client
            .call(&ask(1, serde_json::json!({})))
            .await
            .unwrap()
            .result
            .unwrap(),
    )
    .unwrap();
    assert_eq!(now.status, PredictionStatus::Conflicts);
    assert_eq!(now.files, ["a.txt"]);
    let debounced: ConflictPrediction = serde_json::from_value(
        client
            .call(&ask(2, serde_json::json!({ "debounce_ms": 50 })))
            .await
            .unwrap()
            .result
            .unwrap(),
    )
    .unwrap();
    assert_eq!(debounced, now);
}

// No test reads or writes the real config, data dir or credentials of whoever runs it.
gitpoulpi_testkit::isolate_binary!();
