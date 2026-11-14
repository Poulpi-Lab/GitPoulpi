//! Characterization of `run_action`: for every `Action`, the daemon requests
//! it sends (operation and JSON arguments, in order) and what it reports
//! back, captured against a fake daemon and compared with
//! `action_requests.golden`. Regenerate the golden after an intended wire
//! change with `GITPOULPI_UPDATE_GOLDEN=1 cargo test -p gitpoulpi-tui
//! action_requests`.

use super::*;
use std::fmt::Write as _;

fn oid(n: u8) -> Oid {
    Oid([n; 20])
}

fn path(p: &str) -> PathBuf {
    PathBuf::from(p)
}

fn s(v: &str) -> String {
    v.to_string()
}

fn a_ref() -> Ref {
    Ref {
        kind: gitpoulpi_core::domain::RefKind::LocalBranch { name: s("topic") },
        target: oid(9),
        visibility: Default::default(),
        is_head: false,
        upstream: None,
        tag_sync: None,
        multi_remote: None,
    }
}

fn selector() -> LineSelector {
    LineSelector {
        hunk: HunkId(2),
        line_indices: vec![1, 3],
    }
}

/// Every `Action` variant at least once, with arguments that tell the
/// fields apart in the captured JSON.
fn every_action() -> Vec<Action> {
    use gitpoulpi_core::domain::{
        InteractiveRebase, LfsTrackOptions, PullMode, RebaseAction, RebaseStep, WorktreeStart,
    };
    vec![
        Action::Undo {
            expect: Some(4),
            confirmed: false,
        },
        Action::Redo {
            expect: None,
            confirmed: true,
        },
        Action::ComposeCommits {
            commits: vec![
                ComposedCommit {
                    message: s("one"),
                    paths: vec![path("a.txt"), path("b.txt")],
                },
                ComposedCommit {
                    message: s("two"),
                    paths: vec![path("c.txt")],
                },
            ],
        },
        Action::StageFile(path("dir/a.txt")),
        Action::UnstageFile(path("dir/a.txt")),
        Action::StageAll,
        Action::UnstageAll,
        Action::StageFolder(path("dir")),
        Action::UnstageFolder(path("dir")),
        Action::StageHunk {
            path: path("a.txt"),
            hunk: HunkId(1),
        },
        Action::UnstageHunk {
            path: path("a.txt"),
            hunk: HunkId(1),
        },
        Action::StageLines {
            path: path("a.txt"),
            selector: selector(),
        },
        Action::UnstageLines {
            path: path("a.txt"),
            selector: selector(),
        },
        Action::DiscardFile {
            path: path("a.txt"),
            source: oid(1),
        },
        Action::DiscardPaths(vec![path("a.txt"), path("dir/b.txt")]),
        Action::DiscardHunk {
            path: path("a.txt"),
            hunk: HunkId(1),
        },
        Action::DiscardLines {
            path: path("a.txt"),
            selector: selector(),
        },
        Action::Commit {
            message: s("msg"),
            amend: false,
            no_verify: false,
            push: false,
        },
        Action::Commit {
            message: s("msg"),
            amend: true,
            no_verify: true,
            push: false,
        },
        Action::Fetch(s("origin")),
        Action::FetchAll,
        Action::CheckoutPullRequestHead {
            remote: s("origin"),
            remote_ref: s("pull/7/head"),
            branch: s("pr-7"),
        },
        Action::Pull(PullOptions {
            mode: PullMode::FastForwardOnly,
            remote: Some(s("origin")),
        }),
        Action::Push {
            remote: s("origin"),
            opts: PushOptions {
                force_with_lease: false,
                set_upstream: true,
                branch: Some(s("main")),
            },
        },
        Action::PullThenPush {
            remote: s("origin"),
            branch: s("main"),
        },
        Action::FetchThenForcePush {
            remote: s("origin"),
            branch: s("main"),
        },
        Action::CheckoutBranch(s("topic")),
        Action::CheckoutCommit(oid(2)),
        Action::StashPush,
        Action::DeleteFile(path("dir\\a.txt")),
        Action::AddToGitignore {
            pattern: s("*.log"),
            file: None,
        },
        Action::StashFile(path("dir\\a.txt")),
        Action::StashPushWith { message: s("wip") },
        Action::StashPop(StashId(0)),
        Action::StashApply(StashId(1)),
        Action::StashDrop(StashId(2)),
        Action::AutoStashPop(StashId(0), Some(oid(7))),
        Action::AutoStashDrop(StashId(1), None),
        Action::BranchCreate {
            name: s("topic"),
            at: oid(3),
        },
        Action::BranchCreateTracking {
            local_name: s("topic"),
            upstream: s("origin/topic"),
        },
        Action::BranchDelete(s("topic")),
        Action::BranchRename {
            old_name: s("old"),
            new_name: s("new"),
        },
        Action::TagCreate {
            name: s("v1"),
            at: oid(4),
        },
        Action::TagDelete(s("v1")),
        Action::TagCreateAnnotated {
            name: s("v1"),
            at: oid(4),
            message: s("release"),
        },
        Action::BranchDeleteRemote {
            remote: s("origin"),
            name: s("topic"),
            and_local: None,
        },
        Action::BranchDeleteRemote {
            remote: s("origin"),
            name: s("topic"),
            and_local: Some(s("topic")),
        },
        Action::TagAnnotate {
            name: s("v1"),
            message: s("notes"),
        },
        Action::TagPush {
            remote: s("origin"),
            selection: TagSelection::AllUnpushed,
        },
        Action::TagPush {
            remote: s("origin"),
            selection: TagSelection::Named(vec![s("v1")]),
        },
        Action::TagDeleteRemote {
            remote: s("origin"),
            name: s("v1"),
        },
        Action::TagDeleteBoth {
            remote: s("origin"),
            name: s("v1"),
        },
        Action::TagDeleteKeepRemote {
            name: s("v1"),
            remotes: vec![s("origin")],
        },
        Action::TagFetch {
            remote: s("origin"),
            name: s("v1"),
        },
        Action::CheckoutThen {
            branch: s("topic"),
            then: Box::new(Action::StageAll),
        },
        Action::Merge(a_ref()),
        Action::Rebase(a_ref()),
        Action::RebaseInteractive(InteractiveRebase {
            upstream: Some(oid(5)),
            onto: None,
            steps: vec![RebaseStep {
                commit: oid(6),
                action: RebaseAction::Pick,
                message: None,
            }],
            autosquash: false,
            cherry_pick: false,
        }),
        Action::RebaseContinue,
        Action::RebaseSkip,
        Action::RebaseAbort,
        Action::OperationAbort,
        Action::AmContinue,
        Action::AmSkip,
        Action::AmAbort,
        Action::IndexLockRemove,
        Action::OperationSkip,
        Action::CherryPick(oid(7)),
        Action::Revert {
            commit: oid(7),
            mainline: None,
        },
        Action::Revert {
            commit: oid(7),
            mainline: Some(2),
        },
        Action::Reset {
            target: oid(8),
            mode: ResetMode::Mixed,
        },
        Action::Reword {
            commit: oid(8),
            message: s("new message"),
        },
        Action::SplitCommit {
            req: gitpoulpi_core::domain::SplitCommit {
                commit: oid(8),
                parts: vec![
                    gitpoulpi_core::domain::SplitPart {
                        message: s("first"),
                        pieces: vec![gitpoulpi_core::domain::SplitPiece::Hunk {
                            path: path("a.txt"),
                            hunk: HunkId(0),
                        }],
                    },
                    gitpoulpi_core::domain::SplitPart {
                        message: s("second"),
                        pieces: vec![gitpoulpi_core::domain::SplitPiece::File {
                            path: path("b.bin"),
                        }],
                    },
                ],
                sign: false,
            },
        },
        Action::ResolveConflict {
            path: path("a.txt"),
            side: ConflictSide::Theirs,
        },
        Action::RemoteAdd {
            name: s("up"),
            url: s("https://example.test/r.git"),
        },
        Action::RemoteRemove(s("up")),
        Action::RemoteRename {
            old_name: s("up"),
            new_name: s("down"),
        },
        Action::RemoteSetUrl {
            name: s("up"),
            url: s("https://example.test/r.git"),
        },
        Action::SparseSet {
            dirs: vec![s("src"), s("docs/guide")],
        },
        Action::SparseDisable,
        Action::SetRefVisibility {
            hidden: vec![s("a")],
            solo: vec![s("main")],
            hidden_remotes: vec![s("archive")],
        },
        Action::RemoteSetPushUrl {
            name: s("up"),
            url: Some(s("https://example.test/p.git")),
        },
        Action::RemoteSetPushUrl {
            name: s("up"),
            url: None,
        },
        Action::BranchSetUpstream {
            branch: s("topic"),
            upstream: Some(s("origin/topic")),
        },
        Action::BranchSetUpstream {
            branch: s("topic"),
            upstream: None,
        },
        Action::FastForward {
            branch: s("topic"),
            target: oid(9),
            target_label: s("origin/topic"),
        },
        Action::WorktreeAdd {
            path: path("../wt"),
            opts: WorktreeAddOptions {
                start: WorktreeStart::Branch(s("topic")),
                new_branch: Some(s("new")),
            },
        },
        Action::PatchCreate {
            source: PatchSource::Commits(vec![oid(1), oid(2)]),
            dest: path("out.patch"),
        },
        Action::PatchApply {
            path: path("in.patch"),
        },
        Action::LfsInstall,
        Action::LfsTrack {
            patterns: vec![s("*.psd"), s("*.bin")],
            opts: LfsTrackOptions::default(),
        },
        Action::LfsUntrack(vec![s("*.psd")]),
        Action::LfsFetch(s("origin")),
        Action::LfsPull(s("origin")),
        Action::LfsPush(s("origin")),
        Action::LfsFetchAll(s("origin")),
        Action::LfsPushAll(s("origin")),
        Action::LfsCheckout,
        Action::LfsPrune,
        Action::LfsLocks(s("origin")),
        Action::LfsLock {
            remote: s("origin"),
            path: path("big.psd"),
        },
        Action::LfsUnlock {
            remote: s("origin"),
            path: path("big.psd"),
            force: true,
        },
    ]
}

/// What the fake daemon answers, typed where the arm turns the value into a
/// status line, `null` otherwise.
fn answer(command: OperationKind) -> serde_json::Value {
    use serde_json::json;
    match command {
        OperationKind::Fetch
        | OperationKind::FetchAll
        | OperationKind::FetchRefAsBranch
        | OperationKind::FetchRefspec => {
            json!({ "updated_refs": ["origin/main"] })
        }
        OperationKind::Pull | OperationKind::Merge | OperationKind::FastForward => {
            json!("UpToDate")
        }
        OperationKind::Push | OperationKind::TagPush => json!({ "pushed_refs": ["main"] }),
        OperationKind::Rebase
        | OperationKind::RebaseInteractive
        | OperationKind::RebaseContinue
        | OperationKind::RebaseSkip
        | OperationKind::AmContinue
        | OperationKind::AmSkip => json!("Completed"),
        OperationKind::CherryPick => json!({ "Completed": [Oid([7; 20])] }),
        OperationKind::Revert => json!("Conflicted"),
        OperationKind::Commit => json!(Oid([1; 20])),
        OperationKind::AddToGitignore => json!(true),
        OperationKind::ComposeCommits | OperationKind::LfsTrack | OperationKind::LfsUntrack => {
            json!([])
        }
        OperationKind::StashPush => json!(0),
        OperationKind::Reword => json!({
            "reworded": Oid([8; 20]),
            "previous_head": Oid([7; 20]),
            "new_head": Oid([8; 20]),
            "replayed": 2,
        }),
        OperationKind::SplitCommit => json!({
            "created": [Oid([9; 20]), Oid([10; 20])],
            "previous_head": Oid([7; 20]),
            "new_head": Oid([10; 20]),
            "replayed": 0,
        }),
        OperationKind::WorktreeAdd => json!({
            "path": "../wt",
            "head": Oid([1; 20]),
            "branch": null,
            "is_main": false,
            "is_locked": false,
        }),
        OperationKind::PatchCreate => json!({ "path": "out.patch", "commits": 2, "bytes": 10 }),
        OperationKind::PatchApply => json!({ "files": ["a.txt"] }),
        OperationKind::LfsLock => json!({
            "id": "1",
            "path": "big.psd",
            "owner": null,
            "locked_at": "now",
            "ours": true,
        }),
        _ => serde_json::Value::Null,
    }
}

#[tokio::test]
async fn every_action_sends_the_requests_the_golden_file_records() {
    // `Fetch` reads the process-wide `AUTO_PRUNE`: pin it off, and keep the
    // Settings toggle test from flipping it meanwhile.
    let _prune = crate::live_settings::AUTO_PRUNE_TEST_LOCK.lock().await;
    crate::live_settings::install_auto_prune(false);
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0))
        .await
        .unwrap();
    let port = listener.local_addr().unwrap().port();
    let stream = tokio::net::TcpStream::connect(("127.0.0.1", port))
        .await
        .unwrap();
    let (mut server, _) = listener.accept().await.unwrap();
    let (mut read_half, write_half) = stream.into_split();
    let write_half = Arc::new(AsyncMutex::new(write_half));
    let pending: PendingMap = Arc::new(std::sync::Mutex::new(PendingRequests::default()));
    let next_id = Arc::new(AtomicU64::new(1));

    let seen: Arc<std::sync::Mutex<Vec<(OperationKind, serde_json::Value)>>> = Default::default();
    let log = Arc::clone(&seen);
    tokio::spawn(async move {
        while let Ok(Some(request)) = read_message::<_, Request>(&mut server).await {
            log.lock()
                .unwrap()
                .push((request.command, request.args.clone()));
            let reply = ServerMessage::Response(Response {
                request_id: request.id,
                result: Ok(answer(request.command)),
            });
            write_message(&mut server, &reply).await.unwrap();
        }
    });
    let reader_pending = Arc::clone(&pending);
    tokio::spawn(async move {
        while let Ok(Some(ServerMessage::Response(resp))) =
            read_message::<_, ServerMessage>(&mut read_half).await
        {
            if let Some(tx) = reader_pending
                .lock()
                .unwrap()
                .waiting
                .remove(&resp.request_id)
            {
                let _ = tx.send(resp);
            }
        }
    });

    let repo = gitpoulpi_testkit::FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build()
        .repo_ref();
    let conn = LinkConn {
        write_half,
        pending,
        next_id,
        session: s("characterize"),
        repo,
        memo: Default::default(),
    };
    let (action_error_tx, action_error_rx) = std_mpsc::channel();
    let (action_message_tx, action_message_rx) = std_mpsc::channel();
    let (push_rejected_tx, _push_rejected_rx) = std_mpsc::channel();
    let activity = Arc::new(crate::activity::Activity::default());

    let mut transcript = String::new();
    for action in every_action() {
        writeln!(transcript, "== {action:?}").unwrap();
        let ok = run_action(
            &conn,
            action,
            &action_error_tx,
            &action_message_tx,
            &push_rejected_tx,
            &std_mpsc::channel().0,
            &activity,
        )
        .await;
        for (command, args) in seen.lock().unwrap().drain(..) {
            writeln!(transcript, "  {command:?} {args}").unwrap();
        }
        while let Ok(message) = action_message_rx.try_recv() {
            writeln!(transcript, "  message: {message}").unwrap();
        }
        while let Ok(error) = action_error_rx.try_recv() {
            writeln!(transcript, "  error: {error}").unwrap();
        }
        writeln!(transcript, "  ok: {ok}").unwrap();
    }

    let golden = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src/daemon_link/action_requests.golden");
    if std::env::var_os("GITPOULPI_UPDATE_GOLDEN").is_some() {
        std::fs::write(&golden, &transcript).unwrap();
    }
    let expected = std::fs::read_to_string(&golden).expect("run with GITPOULPI_UPDATE_GOLDEN=1");
    assert_eq!(transcript, expected.replace("\r\n", "\n"));
}
