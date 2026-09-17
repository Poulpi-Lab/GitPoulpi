//! `.mailmap` through the daemon (slice S16): "Merge into..." writes the
//! `.mailmap` as one journal entry that Undo and Redo take back (the old
//! file byte for byte, or no file when there was none), the Log, blame and
//! file history show the mapped names while `log.use_mailmap` is on and the
//! stored ones when it is off. The env var is process-global, so one test
//! runs everything in sequence.

use std::sync::Arc;

use gitpoulpi_core::domain::rewrite::{Ident, IdentityList, MailmapAdded};
use gitpoulpi_core::domain::{
    Blame, BlameContent, CommitGraph, FileHistoryPage, JournalListing, Oid, OperationKind, RepoId,
    UndoOutcome, UndoRefusal,
};
use gitpoulpi_core::{GitBackend, TypedError};
use gitpoulpi_daemon::Daemon;
use gitpoulpi_ipc::{Client, Request, ResponseError};
use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use serde_json::{json, Value};

const SETTING: &str = "GITPOULPI_LOG_USE_MAILMAP";

struct Rig {
    client: Client,
    id: RepoId,
    next: u64,
}

impl Rig {
    async fn call(&mut self, command: OperationKind, args: Value) -> Result<Value, ResponseError> {
        self.next += 1;
        self.client
            .call(&Request {
                id: self.next,
                session: "mailmap-test".into(),
                repo: Some(self.id.clone()),
                command,
                args,
            })
            .await
            .unwrap()
            .result
    }

    async fn ok(&mut self, command: OperationKind, args: Value) -> Value {
        self.call(command, args)
            .await
            .unwrap_or_else(|e| panic!("{command:?} failed: {}", e.message))
    }

    async fn authors(&mut self) -> Vec<String> {
        let log: CommitGraph = serde_json::from_value(
            self.ok(
                OperationKind::Log,
                json!({ "opts": { "all_refs": true, "paths": [], "start_ref": null } }),
            )
            .await,
        )
        .unwrap();
        log.nodes
            .iter()
            .map(|n| format!("{} <{}>", n.author_name, n.author_email))
            .collect()
    }
}

fn commit_as(repo: &FixtureRepo, file: &str, who: &str, message: &str) {
    std::fs::write(repo.root().join(file), format!("{message}\n")).unwrap();
    repo.git(&["add", file]);
    repo.git(&["commit", "-q", "-m", message, &format!("--author={who}")]);
}

fn fixture() -> FixtureRepo {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .with_initial()
        .unwrap()
        .build();
    commit_as(&repo, "a.txt", "alex <alex@old.example.com>", "first");
    commit_as(&repo, "b.txt", "Alex Doe <alex@example.org>", "second");
    commit_as(
        &repo,
        "a.txt",
        "Alexander <alex@laptop.example.com>",
        "third",
    );
    repo
}

#[tokio::test]
async fn merging_identities_writes_the_mailmap_undo_restores_it_and_the_views_follow() {
    std::env::remove_var(SETTING);
    let backends: Vec<(&str, Arc<dyn GitBackend>)> = vec![
        (
            "system",
            Arc::new(gitpoulpi_cli_backend::SystemGitBackend::new()),
        ),
        ("git2", Arc::new(gitpoulpi_git2_backend::Git2Backend::new())),
    ];
    for (name, backend) in backends {
        let repo = fixture();
        let handle = Daemon::new("mailmap-test").spawn(backend).await.unwrap();
        let mut rig = Rig {
            client: Client::connect_port(handle.port).await.unwrap(),
            id: repo.repo_ref().id,
            next: 1,
        };
        let mailmap = repo.root().join(".mailmap");
        let raw = rig.authors().await;
        assert!(
            raw.contains(&"alex <alex@old.example.com>".to_string()),
            "{name}"
        );

        // The list: three Alexes, none mapped yet.
        let list: IdentityList =
            serde_json::from_value(rig.ok(OperationKind::MailmapIdentities, json!({})).await)
                .unwrap();
        assert!(!list.has_mailmap, "{name}");
        let old = list
            .identities
            .iter()
            .find(|i| i.email == "alex@old.example.com")
            .unwrap();
        assert_eq!(
            (old.as_author, old.mapped_to.is_none()),
            (1, true),
            "{name}"
        );

        // Merge into: the file is written, nothing is committed, the Log
        // already shows the canonical name.
        let head = repo.git(&["rev-parse", "HEAD"]);
        let canonical = Ident::new("Alex Doe", "alex@example.org");
        let from = [
            Ident::new("alex", "alex@old.example.com"),
            Ident::new("Alexander", "alex@laptop.example.com"),
        ];
        let added: MailmapAdded = serde_json::from_value(
            rig.ok(
                OperationKind::MailmapAdd,
                json!({ "canonical": canonical, "from": from }),
            )
            .await,
        )
        .unwrap();
        assert_eq!(added.added, 2, "{name}");
        let written = std::fs::read_to_string(&mailmap).unwrap();
        assert_eq!(written.lines().count(), 2, "{name}: {written}");
        assert_eq!(
            repo.git(&["rev-parse", "HEAD"]),
            head,
            "{name}: nothing committed"
        );
        assert!(
            repo.git(&["status", "--porcelain"]).contains(".mailmap"),
            "{name}"
        );
        let shown = rig.authors().await;
        assert_eq!(
            shown
                .iter()
                .filter(|a| *a == "Alex Doe <alex@example.org>")
                .count(),
            3,
            "{name}: {shown:?}"
        );

        // Blame and file history show the mapped names too.
        let history: FileHistoryPage = serde_json::from_value(
            rig.ok(
                OperationKind::FileHistory,
                json!({ "path": "a.txt", "start": Oid::from_hex(&head).unwrap(), "limit": 10 }),
            )
            .await,
        )
        .unwrap();
        assert!(
            history
                .entries
                .iter()
                .all(|e| e.author_email == "alex@example.org"),
            "{name}: {:?}",
            history.entries
        );
        let blame: Blame = serde_json::from_value(
            rig.ok(
                OperationKind::Blame,
                json!({ "path": "a.txt", "at": Oid::from_hex(&head).unwrap() }),
            )
            .await,
        )
        .unwrap();
        let BlameContent::Lines(lines) = blame.content else {
            panic!("{name}: no blame lines");
        };
        assert!(
            lines.iter().all(|l| l.author_email == "alex@example.org"),
            "{name}"
        );

        // The journal has the entry; Undo takes the file away again.
        let listing: JournalListing =
            serde_json::from_value(rig.ok(OperationKind::JournalList, json!({})).await).unwrap();
        assert_eq!(
            listing.undo[0].summary.label, "Merge identities into Alex Doe",
            "{name}"
        );
        let out: UndoOutcome =
            serde_json::from_value(rig.ok(OperationKind::Undo, json!({})).await).unwrap();
        assert_eq!(out.entry.kind, OperationKind::MailmapAdd, "{name}");
        assert!(!mailmap.exists(), "{name}: the file did not exist before");
        assert_eq!(rig.authors().await, raw, "{name}: the Log is as before");
        let out: UndoOutcome =
            serde_json::from_value(rig.ok(OperationKind::Redo, json!({})).await).unwrap();
        assert!(out.applied, "{name}");
        assert_eq!(
            std::fs::read_to_string(&mailmap).unwrap(),
            written,
            "{name}"
        );

        // An existing file comes back byte for byte.
        rig.ok(OperationKind::Undo, json!({})).await;
        std::fs::write(&mailmap, "# mine\r\nCarol <c@x.org> <c@y.org>").unwrap();
        rig.ok(
            OperationKind::MailmapAdd,
            json!({ "canonical": canonical, "from": from[..1] }),
        )
        .await;
        assert_eq!(
            std::fs::read_to_string(&mailmap).unwrap().lines().count(),
            3,
            "{name}"
        );
        rig.ok(OperationKind::Undo, json!({})).await;
        assert_eq!(
            std::fs::read(&mailmap).unwrap(),
            b"# mine\r\nCarol <c@x.org> <c@y.org>",
            "{name}"
        );
        rig.ok(OperationKind::Redo, json!({})).await;

        // Edited since: undo refuses and leaves the edit alone.
        std::fs::write(&mailmap, "edited\n").unwrap();
        let refused = rig.call(OperationKind::Undo, json!({})).await.unwrap_err();
        assert!(
            matches!(
                refused.kind,
                Some(TypedError::UndoRefused(UndoRefusal::PathChanged { .. }))
            ),
            "{name}: {refused:?}"
        );
        assert_eq!(
            std::fs::read_to_string(&mailmap).unwrap(),
            "edited\n",
            "{name}"
        );

        // The setting off shows the stored names, even with a good file.
        std::fs::write(&mailmap, written).unwrap();
        assert!(!rig
            .authors()
            .await
            .contains(&"alex <alex@old.example.com>".into()));
        std::env::set_var(SETTING, "false");
        assert_eq!(rig.authors().await, raw, "{name}: log.use_mailmap = false");
        std::env::remove_var(SETTING);
        assert!(!rig
            .authors()
            .await
            .contains(&"alex <alex@old.example.com>".into()));

        handle.shutdown().await.unwrap();
    }
}

// No test reads or writes the real config, data dir or credentials of whoever runs it.
gitpoulpi_testkit::isolate_binary!();
