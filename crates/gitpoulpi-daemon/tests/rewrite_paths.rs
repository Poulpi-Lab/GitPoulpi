//! Extracting a subdirectory through the daemon (S20): the in-place mode is
//! one journal entry that undo and redo take back (a deleted branch comes
//! back too), the clone mode records nothing, the submodule mode records
//! the one commit it adds and undo puts the plain folder back. Both
//! backends.

use std::sync::Arc;

use gitpoulpi_core::domain::rewrite::{
    ExtractMode, PathMove, Recipe, RewriteOutcome, RewritePlan, Rule, RuleKind, Scope,
    SignaturePolicy,
};
use gitpoulpi_core::domain::{Direction, JournalListing, ListedEntry, OperationKind, UndoOutcome};
use gitpoulpi_core::GitBackend;
use gitpoulpi_daemon::Daemon;
use gitpoulpi_ipc::rewrite::{ApplyArgs, PreviewArgs};
use gitpoulpi_ipc::{Client, Request, ResponseError};
use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use serde_json::{json, Value};

const SESSION: &str = "rewrite-paths-test";

static TURN: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

fn backends() -> Vec<(&'static str, Arc<dyn GitBackend>)> {
    vec![
        (
            "system",
            Arc::new(gitpoulpi_cli_backend::SystemGitBackend::new()),
        ),
        ("git2", Arc::new(gitpoulpi_git2_backend::Git2Backend::new())),
    ]
}

fn isolated_config() {
    static DIR: std::sync::OnceLock<tempfile::TempDir> = std::sync::OnceLock::new();
    let dir = DIR.get_or_init(|| tempfile::tempdir().unwrap());
    std::env::set_var("GITPOULPI_CONFIG_DIR", dir.path());
}

fn put(repo: &FixtureRepo, files: &[(&str, &str)], message: &str) {
    for (path, text) in files {
        let full = repo.root().join(path);
        std::fs::create_dir_all(full.parent().unwrap()).unwrap();
        std::fs::write(full, text).unwrap();
    }
    repo.git(&["add", "-A"]);
    repo.git(&["commit", "-q", "-m", message]);
}

fn lib_repo() -> FixtureRepo {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .with_initial()
        .unwrap()
        .build();
    put(&repo, &[("lib/a.txt", "a1\n")], "add lib");
    put(&repo, &[("other/x.txt", "x\n")], "other work");
    put(&repo, &[("lib/a.txt", "a2\n")], "edit lib");
    repo.git(&["branch", "no-lib", "main~3"]);
    repo
}

fn recipe(mode: ExtractMode) -> Recipe {
    let mut recipe = Recipe::new(
        "extract",
        Scope::AllLocal,
        vec![Rule::new(RuleKind::MovePaths(PathMove::ExtractSubdir {
            dir: "lib".into(),
            mode: Some(mode),
        }))],
    );
    recipe.options.signatures = Some(SignaturePolicy::Strip);
    recipe.scope_options.include_protected = vec!["main".into()];
    recipe
}

struct Rig {
    handle: Option<gitpoulpi_daemon::DaemonHandle>,
    client: Client,
    repo: FixtureRepo,
    next: u64,
    _turn: tokio::sync::MutexGuard<'static, ()>,
}

impl Rig {
    async fn new(backend: Arc<dyn GitBackend>) -> Rig {
        let turn = TURN.lock().await;
        std::env::set_var("GITPOULPI_GENERAL_UNDO_DEPTH", "0");
        isolated_config();
        let repo = lib_repo();
        let handle = Daemon::new(SESSION).spawn(backend).await.unwrap();
        let client = Client::connect_port(handle.port).await.unwrap();
        let mut rig = Rig {
            handle: Some(handle),
            client,
            repo,
            next: 1,
            _turn: turn,
        };
        rig.ok(OperationKind::JournalState, json!({})).await;
        rig
    }

    async fn call(&mut self, command: OperationKind, args: Value) -> Result<Value, ResponseError> {
        self.next += 1;
        self.client
            .call(&Request {
                id: self.next,
                session: SESSION.into(),
                repo: Some(self.repo.repo_ref().id),
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

    async fn preview(&mut self, mode: ExtractMode) -> RewritePlan {
        let args = serde_json::to_value(PreviewArgs {
            recipe: recipe(mode),
            vouched: Vec::new(),
        })
        .unwrap();
        serde_json::from_value(self.ok(OperationKind::RewritePreview, args).await).unwrap()
    }

    async fn apply(
        &mut self,
        plan: &RewritePlan,
        target: Option<&std::path::Path>,
    ) -> RewriteOutcome {
        let args = serde_json::to_value(ApplyArgs {
            run: plan.run.clone(),
            digest: plan.digest.clone(),
            confirmed: false,
            target: target.map(Into::into),
        })
        .unwrap();
        serde_json::from_value(self.ok(OperationKind::RewriteApply, args).await).unwrap()
    }

    async fn undo_stack(&mut self) -> Vec<ListedEntry> {
        let listing: JournalListing =
            serde_json::from_value(self.ok(OperationKind::JournalList, json!({})).await).unwrap();
        listing.undo
    }

    async fn step(&mut self, direction: Direction) -> UndoOutcome {
        let command = match direction {
            Direction::Undo => OperationKind::Undo,
            Direction::Redo => OperationKind::Redo,
        };
        serde_json::from_value(self.ok(command, json!({ "confirmed": true })).await).unwrap()
    }

    fn refs(&self) -> String {
        self.repo.git(&[
            "for-each-ref",
            "--format=%(refname) %(objectname)",
            "refs/heads",
            "refs/tags",
        ])
    }

    async fn stop(mut self) {
        self.handle.take().unwrap().shutdown().await.unwrap();
    }
}

#[tokio::test]
async fn an_in_place_extraction_is_one_entry_and_undo_brings_back_the_deleted_branch() {
    for (name, backend) in backends() {
        let mut rig = Rig::new(backend).await;
        let before = rig.refs();
        let head = rig.repo.git(&["rev-parse", "HEAD"]);
        let plan = rig.preview(ExtractMode::InPlace).await;
        assert!(
            plan.refs
                .iter()
                .any(|r| r.name.ends_with("no-lib") && r.new.is_none()),
            "{name}: the plan deletes the branch"
        );
        rig.apply(&plan, None).await;
        assert_eq!(rig.undo_stack().await.len(), 1, "{name}");
        assert_eq!(
            rig.repo.git(&["ls-tree", "-r", "--name-only", "HEAD"]),
            "a.txt"
        );
        assert!(rig
            .repo
            .try_git(&["rev-parse", "--verify", "refs/heads/no-lib"])
            .is_none());

        rig.step(Direction::Undo).await;
        assert_eq!(rig.refs(), before, "{name}: undo recreates the branch");
        assert_eq!(rig.repo.git(&["rev-parse", "HEAD"]), head, "{name}");
        assert_eq!(rig.repo.git(&["status", "--porcelain"]), "", "{name}");
        assert!(rig.repo.root().join("lib/a.txt").exists(), "{name}");

        rig.step(Direction::Redo).await;
        assert_eq!(
            rig.repo.git(&["ls-tree", "-r", "--name-only", "HEAD"]),
            "a.txt"
        );
        assert!(rig
            .repo
            .try_git(&["rev-parse", "--verify", "refs/heads/no-lib"])
            .is_none());
        rig.stop().await;
    }
}

#[tokio::test]
async fn a_clone_extraction_records_nothing_here() {
    for (name, backend) in backends() {
        let mut rig = Rig::new(backend).await;
        let before = rig.refs();
        let out = tempfile::tempdir().unwrap();
        let target = out.path().join("lib-repo");
        let plan = rig.preview(ExtractMode::Clone).await;
        let outcome = rig.apply(&plan, Some(&target)).await;
        assert!(outcome.moved.is_empty(), "{name}");
        assert_eq!(rig.refs(), before, "{name}");
        assert!(rig.undo_stack().await.is_empty(), "{name}: nothing to undo");
        assert_eq!(
            rig.repo
                .git_at(&target, &["log", "--reverse", "--format=%s"]),
            "add lib\nedit lib",
            "{name}"
        );
        rig.stop().await;
    }
}

#[tokio::test]
async fn a_submodule_extraction_is_one_entry_and_undo_puts_the_plain_folder_back() {
    for (name, backend) in backends() {
        let mut rig = Rig::new(backend).await;
        let head = rig.repo.git(&["rev-parse", "HEAD"]);
        let out = tempfile::tempdir().unwrap();
        let target = out.path().join("lib-repo");
        let plan = rig.preview(ExtractMode::Submodule).await;
        let outcome = rig.apply(&plan, Some(&target)).await;
        assert_eq!(outcome.moved.len(), 1, "{name}");
        assert_eq!(rig.undo_stack().await.len(), 1, "{name}");
        assert_eq!(rig.repo.git(&["rev-parse", "HEAD^"]), head, "{name}");
        assert!(rig
            .repo
            .git(&["ls-tree", "HEAD", "lib"])
            .starts_with("160000 commit "));

        rig.step(Direction::Undo).await;
        assert_eq!(rig.repo.git(&["rev-parse", "HEAD"]), head, "{name}");
        assert_eq!(rig.repo.git(&["status", "--porcelain"]), "", "{name}");
        assert_eq!(rig.repo.git(&["submodule", "status"]), "", "{name}");
        assert_eq!(
            std::fs::read_to_string(rig.repo.root().join("lib/a.txt")).unwrap(),
            "a2\n"
        );

        rig.step(Direction::Redo).await;
        assert!(rig
            .repo
            .git(&["ls-tree", "HEAD", "lib"])
            .starts_with("160000 commit "));
        rig.stop().await;
    }
}

// No test reads or writes the real config, data dir or credentials of whoever runs it.
gitpoulpi_testkit::isolate_binary!();
