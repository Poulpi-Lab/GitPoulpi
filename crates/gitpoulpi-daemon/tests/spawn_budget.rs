//! Perf regression guard (the perf design) at the daemon
//! layer: how many `git` processes one IPC request spawns. The daemon runs
//! in-process here, so the process-wide spawn counter sees its workers.

use std::sync::Arc;

use gitpoulpi_cli_backend::{total_spawn_count, SystemGitBackend};
use gitpoulpi_core::domain::{OperationKind, StageTarget};
use gitpoulpi_daemon::Daemon;
use gitpoulpi_ipc::{Client, Request};
use gitpoulpi_testkit::{FixtureRepoBuilder, SpawnBudget};

async fn call(
    client: &mut Client,
    id: u64,
    repo: &gitpoulpi_core::domain::RepoId,
    command: OperationKind,
    args: serde_json::Value,
) {
    let resp = client
        .call(&Request {
            id,
            session: "spawn-budget".into(),
            repo: Some(repo.clone()),
            command,
            args,
        })
        .await
        .unwrap();
    assert!(resp.result.is_ok(), "{command:?} failed: {:?}", resp.result);
}

/// The system backend's staging is one `git add`/`git restore`; the daemon
/// adds only the undo journal's index capture on top (no status, no log, no
/// LFS probe behind a stage).
#[tokio::test]
async fn staging_through_the_daemon_adds_no_spawns() {
    let sb = SpawnBudget::serial(total_spawn_count);
    // These budgets are the journal's own overhead; the outside snapshot's
    // has its own test (`outside_detection_costs_a_known_number_of_spawns`).
    std::env::set_var("GITPOULPI_GENERAL_JOURNAL_OUTSIDE", "false");
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build();
    let repo_ref = repo.repo_ref();
    std::fs::write(repo.root().join("new.txt"), "hello").unwrap();

    let backend: Arc<dyn gitpoulpi_core::GitBackend> = Arc::new(SystemGitBackend::new());
    let handle = Daemon::new("spawn-budget").spawn(backend).await.unwrap();
    let mut client = Client::connect(&handle.endpoint()).await.unwrap();
    let id = &repo_ref.id;
    let stage = serde_json::json!({ "target": StageTarget::File("new.txt".into()) });

    // Warm the daemon (first-request setup is allowed to cost more).
    call(
        &mut client,
        1,
        id,
        OperationKind::Status,
        serde_json::json!({}),
    )
    .await;

    // Staging is undoable: the journal records the index tree before and
    // after, but the trees come from libgit2 in the daemon's own process
    // (no `write-tree`), pins are loose ref files, and the journal bytes
    // are written behind. So staging costs the one `git add`, on the first
    // stage of a burst as on every following one.
    let before = total_spawn_count();
    call(&mut client, 2, id, OperationKind::StageFile, stage.clone()).await;
    let n = total_spawn_count() - before;
    assert_eq!(n, 1, "first StageFile of a burst: {n} git spawns, budget 1");

    std::fs::write(repo.root().join("second.txt"), "again").unwrap();
    let stage2 = serde_json::json!({ "target": StageTarget::File("second.txt".into()) });
    let before = total_spawn_count();
    call(&mut client, 4, id, OperationKind::StageFile, stage2).await;
    let n = total_spawn_count() - before;
    assert_eq!(n, 1, "following StageFile: {n} git spawns, budget 1");

    let before = total_spawn_count();
    call(
        &mut client,
        5,
        id,
        OperationKind::Status,
        serde_json::json!({}),
    )
    .await;
    let n = total_spawn_count() - before;
    assert!(n <= 1, "Status request: {n} git spawns, budget 1");
    drop(sb);
    handle.shutdown().await.unwrap();
}

/// `general.undo_enabled = false` is the pre-undo cost, exactly: one `git
/// add`, nothing for the journal (no `write-tree`, no snapshot).
#[tokio::test]
async fn staging_and_discarding_with_undo_off_cost_what_they_cost_before_undo() {
    let sb = SpawnBudget::serial(total_spawn_count);
    // These budgets are the journal's own overhead; the outside snapshot's
    // has its own test (`outside_detection_costs_a_known_number_of_spawns`).
    std::env::set_var("GITPOULPI_GENERAL_JOURNAL_OUTSIDE", "false");
    std::env::set_var("GITPOULPI_GENERAL_UNDO_ENABLED", "false");
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "initial")
        .unwrap()
        .build();
    let repo_ref = repo.repo_ref();
    let backend: Arc<dyn gitpoulpi_core::GitBackend> = Arc::new(SystemGitBackend::new());
    let handle = Daemon::new("spawn-budget").spawn(backend).await.unwrap();
    let mut client = Client::connect(&handle.endpoint()).await.unwrap();
    let id = &repo_ref.id;
    call(
        &mut client,
        1,
        id,
        OperationKind::Status,
        serde_json::json!({}),
    )
    .await;

    std::fs::write(repo.root().join("new.txt"), "hello").unwrap();
    let before = total_spawn_count();
    call(
        &mut client,
        2,
        id,
        OperationKind::StageFile,
        serde_json::json!({ "target": StageTarget::File("new.txt".into()) }),
    )
    .await;
    assert_eq!(total_spawn_count() - before, 1, "StageFile with undo off");

    std::fs::write(repo.root().join("a.txt"), "two\n").unwrap();
    let before = total_spawn_count();
    call(
        &mut client,
        3,
        id,
        OperationKind::Discard,
        serde_json::json!({ "paths": ["a.txt"] }),
    )
    .await;
    // The discard itself (one `git restore`/`checkout`), no snapshots.
    let n = total_spawn_count() - before;
    assert!(
        n <= 3,
        "Discard with undo off: {n} spawns, pre-undo budget 3"
    );
    assert!(!repo.root().join(".git/gitpoulpi").exists());
    std::env::remove_var("GITPOULPI_GENERAL_UNDO_ENABLED");
    drop(sb);
    handle.shutdown().await.unwrap();
}

/// The Diff view lists the files with ONE spawn that prints no
/// patch (`DiffFiles`), then loads a file's hunks with ONE spawn restricted
/// to that file (`Diff` with `paths`); the daemon adds nothing, and the
/// restricted answer is just that file.
#[tokio::test]
async fn diff_files_then_one_files_hunks_is_two_spawns() {
    use gitpoulpi_core::domain::{DiffFileList, DiffTarget, FileDiff};
    let sb = SpawnBudget::serial(total_spawn_count);
    // These budgets are the journal's own overhead; the outside snapshot's
    // has its own test (`outside_detection_costs_a_known_number_of_spawns`).
    std::env::set_var("GITPOULPI_GENERAL_JOURNAL_OUTSIDE", "false");
    let mut builder = FixtureRepoBuilder::new().unwrap();
    for i in 0..30 {
        builder = builder
            .commit_file(&format!("f{i:02}.txt"), "1\n2\n3\n", &format!("c{i}"))
            .unwrap();
    }
    let repo = builder.build();
    let repo_ref = repo.repo_ref();
    for i in 0..30 {
        std::fs::write(repo.root().join(format!("f{i:02}.txt")), "1\nX\n3\n").unwrap();
    }

    let backend: Arc<dyn gitpoulpi_core::GitBackend> = Arc::new(SystemGitBackend::new());
    let handle = Daemon::new("spawn-budget").spawn(backend).await.unwrap();
    let mut client = Client::connect(&handle.endpoint()).await.unwrap();
    let id = &repo_ref.id;
    ask(
        &mut client,
        id,
        1,
        OperationKind::Status,
        serde_json::json!({}),
    )
    .await;

    let before = total_spawn_count();
    let list = ask(
        &mut client,
        id,
        2,
        OperationKind::DiffFiles,
        serde_json::json!({ "target": DiffTarget::WorkingTreeVsIndex }),
    )
    .await;
    assert_eq!(total_spawn_count() - before, 1, "DiffFiles: one spawn");
    let list: DiffFileList = serde_json::from_value(list).unwrap();
    assert_eq!(list.files.len(), 30);
    assert!(!list.hunks_included && list.files.iter().all(|f| f.hunks.is_empty()));

    let before = total_spawn_count();
    let one = ask(
        &mut client,
        id,
        3,
        OperationKind::Diff,
        serde_json::json!({ "target": DiffTarget::WorkingTreeVsIndex, "paths": ["f07.txt"] }),
    )
    .await;
    assert_eq!(total_spawn_count() - before, 1, "Diff paths: one spawn");
    let one: Vec<FileDiff> = serde_json::from_value(one).unwrap();
    assert_eq!(one.len(), 1);
    assert_eq!(one[0].path.to_str(), Some("f07.txt"));
    assert_eq!(one[0].hunks.len(), 1);
    drop(sb);
    handle.shutdown().await.unwrap();
}

async fn ask(
    client: &mut Client,
    repo: &gitpoulpi_core::domain::RepoId,
    id: u64,
    command: OperationKind,
    args: serde_json::Value,
) -> serde_json::Value {
    let req = Request {
        id,
        session: "spawn-budget".into(),
        repo: Some(repo.clone()),
        command,
        args,
    };
    client.call(&req).await.unwrap().result.unwrap()
}

/// Log "Changes" column: a viewport's worth of commits is ONE spawn on the
/// system backend, and asking again (scrolling back) costs none - the
/// daemon remembers per commit.
#[tokio::test]
async fn commit_stats_for_a_viewport_is_one_spawn_then_zero() {
    use gitpoulpi_core::domain::{CommitStat, LogOptions, Oid};
    let sb = SpawnBudget::serial(total_spawn_count);
    // These budgets are the journal's own overhead; the outside snapshot's
    // has its own test (`outside_detection_costs_a_known_number_of_spawns`).
    std::env::set_var("GITPOULPI_GENERAL_JOURNAL_OUTSIDE", "false");
    let mut builder = FixtureRepoBuilder::new().unwrap();
    for i in 0..40 {
        builder = builder
            .commit_file(&format!("f{i:02}.txt"), "1\n2\n", &format!("c{i}"))
            .unwrap();
    }
    let repo = builder.build();
    let repo_ref = repo.repo_ref();

    let backend: Arc<dyn gitpoulpi_core::GitBackend> = Arc::new(SystemGitBackend::new());
    let handle = Daemon::new("spawn-budget").spawn(backend).await.unwrap();
    let mut client = Client::connect(&handle.endpoint()).await.unwrap();
    let id = &repo_ref.id;
    let log = ask(
        &mut client,
        id,
        1,
        OperationKind::Log,
        serde_json::json!({ "opts": LogOptions::default() }),
    )
    .await;
    let ids: Vec<Oid> = log["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| serde_json::from_value(n["id"].clone()).unwrap())
        .collect();
    assert_eq!(ids.len(), 40);

    let before = total_spawn_count();
    let stats = ask(
        &mut client,
        id,
        2,
        OperationKind::CommitStats,
        serde_json::json!({ "ids": ids }),
    )
    .await;
    assert_eq!(total_spawn_count() - before, 1, "40 commits: one spawn");
    let stats: Vec<CommitStat> = serde_json::from_value(stats).unwrap();
    assert_eq!(stats.len(), 40);
    assert!(stats.iter().all(|s| (s.files, s.insertions) == (1, 2)));

    let before = total_spawn_count();
    ask(
        &mut client,
        id,
        3,
        OperationKind::CommitStats,
        serde_json::json!({ "ids": &ids[5..30] }),
    )
    .await;
    assert_eq!(total_spawn_count() - before, 0, "cached: no spawn");
    drop(sb);
    handle.shutdown().await.unwrap();
}

/// Undo journal (the undo-redo design §2.7): what recording a
/// mutation and running an undo/redo add to the system backend's spawns.
/// `plain` is the same operation straight on the backend, so the
/// difference is the journal's own cost. The git2 backend adds none for
/// the journal's reads and ref moves.
#[tokio::test]
async fn journaling_a_mutation_costs_a_known_number_of_spawns() {
    use gitpoulpi_core::domain::{CheckoutTarget, CommitOptions};
    use gitpoulpi_core::GitBackend;
    let sb = SpawnBudget::serial(total_spawn_count);
    // These budgets are the journal's own overhead; the outside snapshot's
    // has its own test (`outside_detection_costs_a_known_number_of_spawns`).
    std::env::set_var("GITPOULPI_GENERAL_JOURNAL_OUTSIDE", "false");
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build();
    let repo_ref = repo.repo_ref();
    let system = SystemGitBackend::new();
    let backend: Arc<dyn gitpoulpi_core::GitBackend> = Arc::new(SystemGitBackend::new());
    let handle = Daemon::new("spawn-budget").spawn(backend).await.unwrap();
    let mut client = Client::connect(&handle.endpoint()).await.unwrap();
    let id = &repo_ref.id;
    let tip = gitpoulpi_core::domain::Oid::from_hex(&repo.git(&["rev-parse", "HEAD"])).unwrap();
    // First request warms the daemon.
    call(
        &mut client,
        1,
        id,
        OperationKind::Status,
        serde_json::json!({}),
    )
    .await;

    // The same operations straight on the backend, in a second repo.
    let base = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build();
    let base_ref = base.repo_ref();
    let base_tip =
        gitpoulpi_core::domain::Oid::from_hex(&base.git(&["rev-parse", "HEAD"])).unwrap();
    std::fs::write(base.root().join("b.txt"), "b").unwrap();
    system
        .stage(&base_ref, &StageTarget::File("b.txt".into()))
        .unwrap();
    let (_, plain_commit) = sb.measure(|| {
        system
            .commit(&base_ref, "b", &CommitOptions::default())
            .unwrap()
    });
    let (_, plain_create) = sb.measure(|| system.branch_create(&base_ref, "g", base_tip).unwrap());
    let (_, plain_checkout) = sb.measure(|| {
        system
            .checkout(&base_ref, &CheckoutTarget::Branch("g".into()))
            .unwrap()
    });
    system
        .checkout(&base_ref, &CheckoutTarget::Branch("main".into()))
        .unwrap();
    let (_, plain_delete) = sb.measure(|| system.branch_delete(&base_ref, "g").unwrap());

    // Commit: journal adds the one heads-only ref snapshot.
    std::fs::write(repo.root().join("a.txt"), "a").unwrap();
    call(
        &mut client,
        2,
        id,
        OperationKind::StageFile,
        serde_json::json!({ "target": StageTarget::File("a.txt".into()) }),
    )
    .await;
    let n = measure(
        &mut client,
        id,
        3,
        OperationKind::Commit,
        serde_json::json!({ "message": "a", "opts": CommitOptions::default() }),
    )
    .await;
    println!("SPAWNS commit journaled={n} plain={plain_commit}");
    assert!(
        n <= plain_commit + 1,
        "commit: {n} spawns, plain {plain_commit}, budget +1"
    );

    // Branch create: nothing added (the outcome is known from the request).
    let n = measure(
        &mut client,
        id,
        4,
        OperationKind::BranchCreate,
        serde_json::json!({ "name": "f", "at": tip }),
    )
    .await;
    println!("SPAWNS branch_create journaled={n} plain={plain_create}");
    assert!(
        n <= plain_create,
        "branch create: {n}, plain {plain_create}, budget +0"
    );

    // Checkout: a heads-only snapshot before and after (git may create refs).
    let n = measure(
        &mut client,
        id,
        5,
        OperationKind::Checkout,
        serde_json::json!({ "target": CheckoutTarget::Branch("f".into()) }),
    )
    .await;
    println!("SPAWNS checkout journaled={n} plain={plain_checkout}");
    assert!(
        n <= plain_checkout + 2,
        "checkout: {n}, plain {plain_checkout}, budget +2"
    );
    call(
        &mut client,
        50,
        id,
        OperationKind::Undo,
        serde_json::json!({}),
    )
    .await; // back on main

    // Branch delete: snapshot, config section and the pin.
    call(
        &mut client,
        51,
        id,
        OperationKind::BranchCreate,
        serde_json::json!({ "name": "g", "at": tip }),
    )
    .await;
    let n = measure(
        &mut client,
        id,
        6,
        OperationKind::BranchDelete,
        serde_json::json!({ "name": "g" }),
    )
    .await;
    println!("SPAWNS branch_delete journaled={n} plain={plain_delete}");
    assert!(
        n <= plain_delete + 3,
        "branch delete: {n}, plain {plain_delete}, budget +3"
    );

    // Undo / redo of a branch delete: facts (snapshot + config) + one
    // transaction + the config restore (read, add, add) -- bounded.
    let n = measure(
        &mut client,
        id,
        7,
        OperationKind::Undo,
        serde_json::json!({}),
    )
    .await;
    println!("SPAWNS undo(branch delete)={n}");
    assert!(n <= 8, "undo: {n} spawns, budget 8");
    let n = measure(
        &mut client,
        id,
        8,
        OperationKind::Redo,
        serde_json::json!({}),
    )
    .await;
    println!("SPAWNS redo(branch delete)={n}");
    assert!(n <= 8, "redo: {n} spawns, budget 8");
    let n = measure(
        &mut client,
        id,
        9,
        OperationKind::Undo,
        serde_json::json!({}),
    )
    .await; // checkout
    println!("SPAWNS undo(branch delete, again)={n}");
    let n = measure(
        &mut client,
        id,
        20,
        OperationKind::JournalState,
        serde_json::json!({}),
    )
    .await;
    println!("SPAWNS journal_state={n}");
    assert!(n <= 2, "JournalState: {n} spawns, budget 2");
    drop(sb);
    handle.shutdown().await.unwrap();
}

/// Sends one request and returns how many `git` processes it spawned.
async fn measure(
    client: &mut Client,
    repo: &gitpoulpi_core::domain::RepoId,
    id: u64,
    command: OperationKind,
    args: serde_json::Value,
) -> u64 {
    let req = Request {
        id,
        session: "spawn-budget".into(),
        repo: Some(repo.clone()),
        command,
        args,
    };
    let before = total_spawn_count();
    let resp = client.call(&req).await.unwrap();
    assert!(resp.result.is_ok(), "{command:?}: {:?}", resp.result);
    total_spawn_count() - before
}

/// What recording each ref-only operation adds to
/// the system backend's spawns. The git2 backend adds none (its snapshots,
/// config reads and ref moves are in-process).
#[tokio::test]
async fn journaling_the_ref_only_operations_costs_a_known_number_of_spawns() {
    use gitpoulpi_core::domain::Oid;
    use gitpoulpi_core::GitBackend;
    // The budgets are for a journal that keeps its entries (unlimited
    // depth); the default depth of 1 also pays for releasing the pins of
    // the entry each operation pushes out.
    std::env::set_var("GITPOULPI_GENERAL_UNDO_DEPTH", "0");
    let sb = SpawnBudget::serial(total_spawn_count);
    // These budgets are the journal's own overhead; the outside snapshot's
    // has its own test (`outside_detection_costs_a_known_number_of_spawns`).
    std::env::set_var("GITPOULPI_GENERAL_JOURNAL_OUTSIDE", "false");
    let build = || {
        FixtureRepoBuilder::new()
            .unwrap()
            .fixed_dates()
            .commit("initial")
            .unwrap()
            .with_origin()
            .unwrap()
            .push("main")
            .unwrap()
            .branch("work")
            .unwrap()
            .checkout("work")
            .unwrap()
            .commit_file("work.txt", "w", "work")
            .unwrap()
            .push("work")
            .unwrap()
            .checkout("main")
            .unwrap()
            .branch("other")
            .unwrap()
            .checkout("main")
            .unwrap()
            .build()
    };
    let oid = |repo: &gitpoulpi_testkit::FixtureRepo, rev: &str| {
        Oid::from_hex(&repo.git(&["rev-parse", rev])).unwrap()
    };
    let repo = build();
    let repo_ref = repo.repo_ref();
    let (tip, work) = (oid(&repo, "main"), oid(&repo, "work"));
    let base = build();
    let base_ref = base.repo_ref();
    let (btip, bwork) = (oid(&base, "main"), oid(&base, "work"));
    let system = SystemGitBackend::new();

    let backend: Arc<dyn gitpoulpi_core::GitBackend> = Arc::new(SystemGitBackend::new());
    let handle = Daemon::new("spawn-budget").spawn(backend).await.unwrap();
    let mut client = Client::connect(&handle.endpoint()).await.unwrap();
    let id = &repo_ref.id;
    call(
        &mut client,
        1,
        id,
        OperationKind::Status,
        serde_json::json!({}),
    )
    .await;

    // (label, journaled request, plain spawns, allowed extra spawns)
    let mut n = 10;
    let check = |label: &'static str, journaled: u64, plain: u64, extra: u64| {
        println!(
            "SPAWNS {label} journaled={journaled} plain={plain} extra={}",
            journaled - plain.min(journaled)
        );
        assert!(
            journaled <= plain + extra,
            "{label}: {journaled} spawns, plain {plain}, budget +{extra}"
        );
    };

    let (_, plain) = sb.measure(|| {
        system
            .branch_set_upstream(&base_ref, "main", Some("origin/work"))
            .unwrap()
    });
    n += 1;
    let j = measure(
        &mut client,
        id,
        n,
        OperationKind::BranchSetUpstream,
        serde_json::json!({ "branch": "main", "upstream": "origin/work" }),
    )
    .await;
    check("set_upstream", j, plain, 2);

    let (_, plain) = sb.measure(|| system.branch_rename(&base_ref, "other", "other2").unwrap());
    n += 1;
    let j = measure(
        &mut client,
        id,
        n,
        OperationKind::BranchRename,
        serde_json::json!({ "old_name": "other", "new_name": "other2" }),
    )
    .await;
    check("branch_rename", j, plain, 2);

    let (_, plain) = sb.measure(|| system.tag_create(&base_ref, "t", btip, None).unwrap());
    n += 1;
    let j = measure(
        &mut client,
        id,
        n,
        OperationKind::TagCreate,
        serde_json::json!({ "name": "t", "at": tip, "annotated_message": null }),
    )
    .await;
    check("tag_create", j, plain, 1);

    let (_, plain) = sb.measure(|| system.tag_annotate(&base_ref, "t", "m").unwrap());
    n += 1;
    let j = measure(
        &mut client,
        id,
        n,
        OperationKind::TagAnnotate,
        serde_json::json!({ "name": "t", "message": "m" }),
    )
    .await;
    check("tag_annotate", j, plain, 3);

    let (_, plain) = sb.measure(|| system.tag_delete(&base_ref, "t").unwrap());
    n += 1;
    let j = measure(
        &mut client,
        id,
        n,
        OperationKind::TagDelete,
        serde_json::json!({ "name": "t" }),
    )
    .await;
    check("tag_delete", j, plain, 2);

    let (_, plain) = sb.measure(|| system.fast_forward(&base_ref, "other2", bwork).unwrap());
    n += 1;
    let j = measure(
        &mut client,
        id,
        n,
        OperationKind::FastForward,
        serde_json::json!({ "branch": "other2", "target": work }),
    )
    .await;
    check("fast_forward", j, plain, 0);

    let (_, plain) = sb.measure(|| system.reword(&base_ref, btip, "renamed").unwrap());
    n += 1;
    let j = measure(
        &mut client,
        id,
        n,
        OperationKind::Reword,
        serde_json::json!({ "commit": tip, "message": "renamed" }),
    )
    .await;
    check("reword", j, plain, 2);

    let (_, plain) = sb.measure(|| {
        system
            .branch_create_tracking(&base_ref, "w2", "origin/work")
            .unwrap()
    });
    n += 1;
    let j = measure(
        &mut client,
        id,
        n,
        OperationKind::BranchCreateTracking,
        serde_json::json!({ "local_name": "w2", "upstream": "origin/work" }),
    )
    .await;
    check("branch_create_tracking", j, plain, 4);

    // Undo and redo of each: bounded, no matter the kind.
    for step in 0..8 {
        n += 1;
        let u = measure(
            &mut client,
            id,
            n,
            OperationKind::Undo,
            serde_json::json!({}),
        )
        .await;
        println!("SPAWNS undo #{step}={u}");
        assert!(u <= 10, "undo: {u} spawns, budget 10");
    }
    for step in 0..8 {
        n += 1;
        let r = measure(
            &mut client,
            id,
            n,
            OperationKind::Redo,
            serde_json::json!({}),
        )
        .await;
        println!("SPAWNS redo #{step}={r}");
        assert!(r <= 10, "redo: {r} spawns, budget 10");
    }
    drop(sb);
    handle.shutdown().await.unwrap();
}

/// What the snapshot-backed operations cost on the system
/// backend. Staging must stay instant: at most one process
/// more than the bare operation once the previous entry knows the index.
#[tokio::test]
async fn snapshot_operations_cost_a_known_number_of_spawns() {
    let sb = SpawnBudget::serial(total_spawn_count);
    // These budgets are the journal's own overhead; the outside snapshot's
    // has its own test (`outside_detection_costs_a_known_number_of_spawns`).
    std::env::set_var("GITPOULPI_GENERAL_JOURNAL_OUTSIDE", "false");
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "first")
        .unwrap()
        .commit_file("b.txt", "bee\n", "second")
        .unwrap()
        .build();
    let repo_ref = repo.repo_ref();
    let backend: Arc<dyn gitpoulpi_core::GitBackend> = Arc::new(SystemGitBackend::new());
    let handle = Daemon::new("spawn-budget").spawn(backend).await.unwrap();
    let mut client = Client::connect(&handle.endpoint()).await.unwrap();
    let id = &repo_ref.id;
    call(
        &mut client,
        1,
        id,
        OperationKind::Status,
        serde_json::json!({}),
    )
    .await;
    std::fs::write(repo.root().join("a.txt"), "edited a\n").unwrap();
    std::fs::write(repo.root().join("b.txt"), "edited b\n").unwrap();

    let file = |p: &str| serde_json::json!({ "target": StageTarget::File(p.into()) });

    // Stage: the bare operation is 1 spawn.
    // The index trees come from libgit2 in the daemon, so the journal adds
    // no process: first of a burst and following ones alike.
    let n = spawns(&mut client, id, OperationKind::StageFile, file("a.txt")).await;
    assert_eq!(n, 1, "first StageFile");
    let n = spawns(&mut client, id, OperationKind::StageFile, file("b.txt")).await;
    assert_eq!(n, 1, "following StageFile");
    let n = spawns(&mut client, id, OperationKind::UnstageFile, file("b.txt")).await;
    assert_eq!(n, 1, "following UnstageFile");

    // Undo / redo of a staging entry: read-tree, plus the precondition
    // (the index file's checksum names the tree: free).
    let n = spawns(&mut client, id, OperationKind::Undo, serde_json::json!({})).await;
    assert!(n <= 3, "undo of a stage: {n} spawns");
    let n = spawns(&mut client, id, OperationKind::Redo, serde_json::json!({})).await;
    assert!(n <= 3, "redo of a stage: {n} spawns");

    // Discard: the safety snapshots before and after are built by libgit2
    // in the daemon, so it costs the bare 3-spawn discard.
    let n = spawns(
        &mut client,
        id,
        OperationKind::Discard,
        serde_json::json!({ "paths": ["a.txt"] }),
    )
    .await;
    println!("SPAWNS discard={n}");
    assert_eq!(n, 3, "Discard of one file (the bare 3-spawn discard)");
    // Undo: the system backend's restore (ls-tree, ls-files, cat-file,
    // update-index: 4); the ref snapshots of the precondition and the
    // state read and the file hashes are native. An undo is not a hot path.
    let n = spawns(&mut client, id, OperationKind::Undo, serde_json::json!({})).await;
    println!("SPAWNS undo discard={n}");
    assert_eq!(n, 4, "undo of a discard");
    drop(sb);
    handle.shutdown().await.unwrap();
}

/// One request's `git` spawns.
async fn spawns(
    client: &mut Client,
    id: &gitpoulpi_core::domain::RepoId,
    command: OperationKind,
    args: serde_json::Value,
) -> u64 {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1000);
    let n = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let before = total_spawn_count();
    call(client, n, id, command, args).await;
    total_spawn_count() - before
}

/// What recording each history-moving operation
/// adds to the system backend's spawns: a heads-only ref snapshot before
/// and after, plus the pin (a merge, revert or cherry-pick journaled costs
/// +3 counting the server's conflict probe, a soft reset +2). The git2
/// backend adds none. Undo (7) and redo (6) of a `reset --keep` move
/// include the confirmation check (status + upstream config).
#[tokio::test]
async fn journaling_the_history_operations_costs_a_known_number_of_spawns() {
    use gitpoulpi_core::domain::{CherryPickOptions, MergeOptions, Oid, RefKind, ResetMode};
    use gitpoulpi_core::GitBackend;
    std::env::set_var("GITPOULPI_GENERAL_UNDO_DEPTH", "0");
    let sb = SpawnBudget::serial(total_spawn_count);
    // These budgets are the journal's own overhead; the outside snapshot's
    // has its own test (`outside_detection_costs_a_known_number_of_spawns`).
    std::env::set_var("GITPOULPI_GENERAL_JOURNAL_OUTSIDE", "false");
    let build = || {
        FixtureRepoBuilder::new()
            .unwrap()
            .fixed_dates()
            .commit("initial")
            .unwrap()
            .branch("feat")
            .unwrap()
            .checkout("feat")
            .unwrap()
            .commit_file("feat.txt", "f", "feat work")
            .unwrap()
            .checkout("main")
            .unwrap()
            .branch("pick")
            .unwrap()
            .checkout("pick")
            .unwrap()
            .commit_file("pick.txt", "p", "pick work")
            .unwrap()
            .checkout("main")
            .unwrap()
            .commit_file("main.txt", "m", "main work")
            .unwrap()
            .build()
    };
    let oid = |repo: &gitpoulpi_testkit::FixtureRepo, rev: &str| {
        Oid::from_hex(&repo.git(&["rev-parse", rev])).unwrap()
    };
    let repo = build();
    let repo_ref = repo.repo_ref();
    let base = build();
    let base_ref = base.repo_ref();
    let system = SystemGitBackend::new();
    let feat_ref = system
        .refs(&base_ref)
        .unwrap()
        .into_iter()
        .find(|r| matches!(&r.kind, RefKind::LocalBranch { name } if name == "feat"))
        .unwrap();
    let (pick, feat) = (oid(&repo, "pick"), oid(&repo, "feat"));

    let backend: Arc<dyn gitpoulpi_core::GitBackend> = Arc::new(SystemGitBackend::new());
    let handle = Daemon::new("spawn-budget").spawn(backend).await.unwrap();
    let mut client = Client::connect(&handle.endpoint()).await.unwrap();
    let id = &repo_ref.id;
    call(
        &mut client,
        1,
        id,
        OperationKind::Status,
        serde_json::json!({}),
    )
    .await;

    let mut n = 10;
    let check = |label: &'static str, journaled: u64, plain: u64, extra: u64| {
        println!(
            "SPAWNS {label} journaled={journaled} plain={plain} extra={}",
            journaled - plain.min(journaled)
        );
        assert!(
            journaled <= plain + extra,
            "{label}: {journaled} spawns, plain {plain}, budget +{extra}"
        );
    };

    let (_, plain) = sb.measure(|| {
        system
            .merge(&base_ref, &feat_ref, &MergeOptions::default())
            .unwrap()
    });
    n += 1;
    let j = measure(
        &mut client,
        id,
        n,
        OperationKind::Merge,
        serde_json::json!({ "from": feat_ref }),
    )
    .await;
    check("merge", j, plain, 3);

    let (_, plain) = sb.measure(|| system.revert(&base_ref, feat, None).unwrap());
    n += 1;
    let j = measure(
        &mut client,
        id,
        n,
        OperationKind::Revert,
        serde_json::json!({ "commit": feat }),
    )
    .await;
    check("revert", j, plain, 3);

    let (_, plain) = sb.measure(|| {
        system
            .cherry_pick(&base_ref, &[pick], &CherryPickOptions::default())
            .unwrap()
    });
    n += 1;
    let j = measure(
        &mut client,
        id,
        n,
        OperationKind::CherryPick,
        serde_json::json!({ "commits": [pick] }),
    )
    .await;
    check("cherry_pick", j, plain, 3);

    let target = oid(&repo, "HEAD~1");
    let (_, plain) = sb.measure(|| {
        system
            .reset(&base_ref, oid(&base, "HEAD~1"), ResetMode::Soft)
            .unwrap()
    });
    n += 1;
    let j = measure(
        &mut client,
        id,
        n,
        OperationKind::Reset,
        serde_json::json!({ "target": target, "mode": "Soft" }),
    )
    .await;
    check("reset_soft", j, plain, 2);

    for step in 0..4 {
        n += 1;
        let u = measure(
            &mut client,
            id,
            n,
            OperationKind::Undo,
            serde_json::json!({}),
        )
        .await;
        println!("SPAWNS history undo #{step}={u}");
        assert!(u <= 7, "undo: {u} spawns, budget 7");
    }
    for step in 0..4 {
        n += 1;
        let r = measure(
            &mut client,
            id,
            n,
            OperationKind::Redo,
            serde_json::json!({}),
        )
        .await;
        println!("SPAWNS history redo #{step}={r}");
        assert!(r <= 6, "redo: {r} spawns, budget 6");
    }
    drop(sb);
    handle.shutdown().await.unwrap();
}

/// What journaling a stash or remote operation costs on top
/// of the bare backend call. The `EXTRA` numbers are the budget; a change is
/// deliberate and goes in the commit message.
#[tokio::test]
async fn stash_and_remote_operations_stay_within_their_journal_budget() {
    use gitpoulpi_core::domain::StashId;
    use gitpoulpi_core::GitBackend;

    fn fixture() -> gitpoulpi_testkit::FixtureRepo {
        FixtureRepoBuilder::new()
            .unwrap()
            .commit_file("a.txt", "one\n", "first")
            .unwrap()
            .branch("feature")
            .unwrap()
            .checkout("main")
            .unwrap()
            .with_origin()
            .unwrap()
            .push("main")
            .unwrap()
            .push("feature")
            .unwrap()
            .track("main", "origin")
            .unwrap()
            .build()
    }

    let sb = SpawnBudget::serial(total_spawn_count);
    // These budgets are the journal's own overhead; the outside snapshot's
    // has its own test (`outside_detection_costs_a_known_number_of_spawns`).
    std::env::set_var("GITPOULPI_GENERAL_JOURNAL_OUTSIDE", "false");
    let bare = SystemGitBackend::new();
    let mut raw: Vec<(&str, u64)> = Vec::new();
    {
        let repo = fixture();
        let r = repo.repo_ref();
        macro_rules! raw_spawns {
            ($label:expr, $call:expr) => {{
                let before = total_spawn_count();
                $call.unwrap();
                raw.push(($label, total_spawn_count() - before));
            }};
        }
        std::fs::write(repo.root().join("a.txt"), "one\nedited\n").unwrap();
        raw_spawns!("stash push", bare.stash_push(&r, Some("w"), false));
        raw_spawns!("stash apply", bare.stash_apply(&r, StashId(0)));
        repo.git(&["checkout", "--", "a.txt"]);
        raw_spawns!("stash pop", bare.stash_pop(&r, StashId(0)));
        repo.git(&["stash", "push", "-m", "again"]);
        raw_spawns!("stash drop", bare.stash_drop(&r, StashId(0)));
        raw_spawns!("remote add", bare.remote_add(&r, "fork", "/x.git"));
        raw_spawns!("remote set-url", bare.remote_set_url(&r, "fork", "/y.git"));
        raw_spawns!("remote rename", bare.remote_rename(&r, "origin", "up"));
        raw_spawns!("remote remove", bare.remote_remove(&r, "up"));
    }

    let repo = fixture();
    let repo_ref = repo.repo_ref();
    let backend: Arc<dyn gitpoulpi_core::GitBackend> = Arc::new(SystemGitBackend::new());
    let handle = Daemon::new("spawn-budget").spawn(backend).await.unwrap();
    let mut client = Client::connect(&handle.endpoint()).await.unwrap();
    let id = &repo_ref.id;
    ask(
        &mut client,
        id,
        1,
        OperationKind::Status,
        serde_json::json!({}),
    )
    .await;
    let mut counts: Vec<(&str, u64)> = Vec::new();
    macro_rules! spawns {
        ($label:expr, $n:expr, $op:expr, $args:expr) => {{
            let before = total_spawn_count();
            ask(&mut client, id, $n, $op, $args).await;
            counts.push(($label, total_spawn_count() - before));
        }};
    }
    std::fs::write(repo.root().join("a.txt"), "one\nedited\n").unwrap();
    spawns!(
        "stash push",
        10,
        OperationKind::StashPush,
        serde_json::json!({ "message": "w", "include_untracked": false, "paths": [] })
    );
    spawns!(
        "stash apply",
        11,
        OperationKind::StashApply,
        serde_json::json!({ "id": 0 })
    );
    repo.git(&["checkout", "--", "a.txt"]);
    spawns!(
        "stash pop",
        12,
        OperationKind::StashPop,
        serde_json::json!({ "id": 0 })
    );
    repo.git(&["stash", "push", "-m", "again"]);
    spawns!(
        "stash drop",
        13,
        OperationKind::StashDrop,
        serde_json::json!({ "id": 0 })
    );
    spawns!(
        "undo stash drop",
        14,
        OperationKind::Undo,
        serde_json::json!({})
    );
    // Drop it again so that the stash list is empty for the rest.
    repo.git(&["stash", "clear"]);
    spawns!(
        "remote add",
        15,
        OperationKind::RemoteAdd,
        serde_json::json!({ "name": "fork", "url": "/x.git" })
    );
    spawns!(
        "remote set-url",
        16,
        OperationKind::RemoteSetUrl,
        serde_json::json!({ "name": "fork", "url": "/y.git" })
    );
    spawns!(
        "remote rename",
        17,
        OperationKind::RemoteRename,
        serde_json::json!({ "old_name": "origin", "new_name": "up" })
    );
    spawns!(
        "remote remove",
        18,
        OperationKind::RemoteRemove,
        serde_json::json!({ "name": "up" })
    );
    spawns!(
        "undo remote remove",
        19,
        OperationKind::Undo,
        serde_json::json!({})
    );
    eprintln!("STASH-REMOTES RAW {raw:?}\nSTASH-REMOTES DAEMON {counts:?}");

    // Journaled spawns above the bare command: (operation, budget).
    const EXTRA: &[(&str, u64)] = &[
        ("stash push", 11),
        ("stash apply", 13),
        ("stash pop", 12),
        ("stash drop", 2),
        ("remote add", 1),
        ("remote set-url", 2),
        ("remote rename", 5),
        ("remote remove", 5),
    ];
    for (label, budget) in EXTRA {
        let bare = raw.iter().find(|(l, _)| l == label).unwrap().1;
        let with = counts.iter().find(|(l, _)| l == label).unwrap().1;
        assert!(
            with - bare <= *budget,
            "{label}: {with} spawns through the daemon, {bare} bare, journal budget +{budget}"
        );
    }
    drop(sb);
    handle.shutdown().await.unwrap();
}

/// The auto-stash sequence (`general.auto_stash = always`: refused
/// checkout, stash push, checkout, stash pop) in ONE request. The journal
/// reads (refs, stash list, the files' safety snapshots) run in process, so
/// what is left is git's own work: the two checkout attempts, the status
/// scan, the push, the pop and the pop's path list. Was 22 before.
#[tokio::test]
async fn an_auto_stashed_checkout_stays_within_its_spawn_budget() {
    let sb = SpawnBudget::serial(total_spawn_count);
    // These budgets are the journal's own overhead; the outside snapshot's
    // has its own test (`outside_detection_costs_a_known_number_of_spawns`).
    std::env::set_var("GITPOULPI_GENERAL_JOURNAL_OUTSIDE", "false");
    std::env::set_var("GITPOULPI_GENERAL_AUTO_STASH", "always");
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "1\n2\n3\n4\n5\n", "base")
        .unwrap()
        .branch("other")
        .unwrap()
        .checkout("other")
        .unwrap()
        .commit_file("a.txt", "1 other\n2\n3\n4\n5\n", "other edit")
        .unwrap()
        .checkout("main")
        .unwrap()
        .build();
    std::fs::write(repo.root().join("a.txt"), "1\n2\n3\n4\n5 local\n").unwrap();
    let repo_ref = repo.repo_ref();
    let backend: Arc<dyn gitpoulpi_core::GitBackend> = Arc::new(SystemGitBackend::new());
    let handle = Daemon::new("spawn-budget").spawn(backend).await.unwrap();
    let mut client = Client::connect(&handle.endpoint()).await.unwrap();
    let id = &repo_ref.id;
    call(
        &mut client,
        1,
        id,
        OperationKind::Status,
        serde_json::json!({}),
    )
    .await;

    let before = total_spawn_count();
    call(
        &mut client,
        2,
        id,
        OperationKind::Checkout,
        serde_json::json!({ "target": { "Branch": "other" } }),
    )
    .await;
    let n = total_spawn_count() - before;
    std::env::remove_var("GITPOULPI_GENERAL_AUTO_STASH");
    assert_eq!(
        repo.git(&["rev-parse", "--abbrev-ref", "HEAD"]).trim(),
        "other"
    );
    assert!(n <= 7, "auto-stashed Checkout: {n} git spawns, budget 7");
    drop(sb);
    handle.shutdown().await.unwrap();
}

// No test reads or writes the real config, data dir or credentials of whoever runs it.
gitpoulpi_testkit::isolate_binary!();
