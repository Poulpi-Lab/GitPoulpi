//! History rewrite through the daemon (the history-rewrite design,
//! slice S5): preview as a job with progress and cancel, the plan kept in
//! the daemon and paged, apply as one journal entry that undo and redo take
//! back, a stale or unconfirmed apply refused, nothing recorded as an outside
//! change. Every test runs against both backends.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use gitpoulpi_core::domain::rewrite::{
    IdentityMatch, IdentityRule, IdentitySet, Recipe, RewriteOutcome, RewritePlan, RewriteRefusal,
    Rule, RuleKind, Scope, SignaturePolicy, Who,
};
use gitpoulpi_core::domain::{
    Direction, JournalListing, ListedEntry, Oid, OperationKind, UndoOutcome,
};
use gitpoulpi_core::{GitBackend, TypedError};
use gitpoulpi_daemon::Daemon;
use gitpoulpi_ipc::rewrite::{ApplyArgs, PreviewArgs, RowFilter, RowsArgs, RowsPage};
use gitpoulpi_ipc::transport::{read_message, write_message};
use gitpoulpi_ipc::{Client, Event, JobInfo, JobState, Request, ResponseError, ServerMessage};
use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use serde_json::{json, Value};
use tokio::net::TcpStream;

const SESSION: &str = "rewrite-undo-test";
const TICK_DELAY: &str = "GITPOULPI_TEST_REWRITE_TICK_DELAY_MS";

/// The tests share one process, and the cancel test slows every preview
/// through an environment variable: they take turns.
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

/// Everyone the fixture authored becomes Casey Example.
fn recipe() -> Recipe {
    let rule = Rule::new(RuleKind::Identity(IdentityRule {
        when: IdentityMatch::email("testkit@gitpoulpi.local"),
        set: IdentitySet {
            name: Some("Casey Example".into()),
            email: Some("casey@example.com".into()),
        },
        apply_to: Who::Author,
        include_taggers: false,
        map_trailers: false,
    }));
    let mut recipe = Recipe::new("casey", Scope::AllLocal, vec![rule]);
    recipe.options.signatures = Some(SignaturePolicy::Strip);
    // `main` is protected by default (`rewrite.protected_branches`); these
    // tests are about everything else, and rewrite it when it is pushed.
    recipe.scope_options.include_protected = vec!["main".into()];
    recipe
}

/// The config every test starts from: the defaults, not the developer's own.
fn isolated_config() {
    static DIR: std::sync::OnceLock<tempfile::TempDir> = std::sync::OnceLock::new();
    let dir = DIR.get_or_init(|| tempfile::tempdir().unwrap());
    std::env::set_var("GITPOULPI_CONFIG_DIR", dir.path());
}

struct Rig {
    handle: Option<gitpoulpi_daemon::DaemonHandle>,
    client: Client,
    repo: FixtureRepo,
    next: u64,
    _turn: tokio::sync::MutexGuard<'static, ()>,
}

impl Rig {
    /// Three commits on `main`, a topic branch on the first, a lightweight
    /// and an annotated tag, and a note.
    async fn new(backend: Arc<dyn GitBackend>) -> Rig {
        let repo = FixtureRepoBuilder::new()
            .unwrap()
            .commit("alpha")
            .unwrap()
            .branch("topic")
            .unwrap()
            .commit("beta")
            .unwrap()
            .tag("light")
            .unwrap()
            .annotated_tag("annotated")
            .unwrap()
            .commit("gamma")
            .unwrap()
            .build();
        repo.git(&["notes", "add", "-m", "reviewed", "HEAD"]);
        Rig::on(backend, repo).await
    }

    async fn on(backend: Arc<dyn GitBackend>, repo: FixtureRepo) -> Rig {
        let turn = TURN.lock().await;
        std::env::set_var("GITPOULPI_GENERAL_UNDO_DEPTH", "0");
        isolated_config();
        let handle = Daemon::new(SESSION).spawn(backend).await.unwrap();
        let client = Client::connect_port(handle.port).await.unwrap();
        let mut rig = Rig {
            handle: Some(handle),
            client,
            repo,
            next: 1,
            _turn: turn,
        };
        // The first look at a repository is its baseline for outside changes.
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

    async fn preview(&mut self) -> RewritePlan {
        let args = serde_json::to_value(PreviewArgs {
            recipe: recipe(),
            vouched: Vec::new(),
        })
        .unwrap();
        serde_json::from_value(self.ok(OperationKind::RewritePreview, args).await).unwrap()
    }

    fn apply_args(plan: &RewritePlan, confirmed: bool) -> Value {
        serde_json::to_value(ApplyArgs {
            run: plan.run.clone(),
            digest: plan.digest.clone(),
            confirmed,
            target: None,
        })
        .unwrap()
    }

    async fn apply(&mut self, plan: &RewritePlan) -> RewriteOutcome {
        let args = Self::apply_args(plan, false);
        serde_json::from_value(self.ok(OperationKind::RewriteApply, args).await).unwrap()
    }

    async fn refusal(&mut self, command: OperationKind, args: Value) -> TypedError {
        let e = self.call(command, args).await.unwrap_err();
        e.kind
            .unwrap_or_else(|| panic!("{command:?} refused untyped: {}", e.message))
    }

    /// The undo stack, oldest first.
    async fn undo_stack(&mut self) -> Vec<ListedEntry> {
        let listing: JournalListing =
            serde_json::from_value(self.ok(OperationKind::JournalList, json!({})).await).unwrap();
        listing.undo
    }

    fn rev(&self, spec: &str) -> Oid {
        Oid::from_hex(&self.repo.git(&["rev-parse", spec])).unwrap()
    }

    /// `name oid` of every branch, tag and notes ref.
    fn refs(&self) -> String {
        self.repo.git(&[
            "for-each-ref",
            "--format=%(refname) %(objectname)",
            "refs/heads",
            "refs/tags",
            "refs/notes",
        ])
    }

    fn run_dirs(&self) -> Vec<String> {
        let dir = self.repo.root().join(".git/gitpoulpi/rewrite");
        match std::fs::read_dir(dir) {
            Ok(entries) => entries
                .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
                .collect(),
            Err(_) => Vec::new(),
        }
    }

    fn fsck(&self) {
        // `git()` fails the test on a non-zero exit; `--connectivity-only`
        // reports a missing object or an invalid ref that way.
        self.repo
            .git(&["fsck", "--connectivity-only", "--no-dangling"]);
    }

    fn authors(&self) -> String {
        self.repo
            .git(&["log", "--branches", "--tags", "--format=%ae"])
    }

    /// A new daemon for the same session: it has no memory of any preview.
    async fn restart(&mut self, backend: Arc<dyn GitBackend>) {
        self.handle.take().unwrap().shutdown().await.unwrap();
        gitpoulpi_daemon::forget_rewrite_plans_for_tests();
        let handle = Daemon::new(SESSION).spawn(backend).await.unwrap();
        self.client = Client::connect_port(handle.port).await.unwrap();
        self.handle = Some(handle);
    }

    async fn stop(mut self) {
        self.handle.take().unwrap().shutdown().await.unwrap();
    }
}

#[tokio::test]
async fn preview_apply_undo_redo_is_one_journal_entry_and_never_an_outside_change() {
    for (name, backend) in backends() {
        let mut rig = Rig::new(backend).await;
        // Whatever the baseline recorded, only the rewrite's entry is new.
        let entries_before = rig.undo_stack().await.len();
        let refs_before = rig.refs();
        let head_before = rig.rev("HEAD");
        assert!(rig.authors().contains("testkit@gitpoulpi.local"), "{name}");

        // Preview: the plan without rows; the rows are paged.
        let plan = rig.preview().await;
        assert!(plan.rows.is_empty(), "{name}: rows stay in the daemon");
        assert_eq!(plan.counts.commits_changed, 3, "{name}");
        assert!(
            plan.refs.iter().any(|r| r.name == "refs/tags/annotated"),
            "{name}: the annotated tag follows"
        );
        assert!(
            plan.refs.iter().any(|r| r.name.starts_with("refs/notes/")),
            "{name}: notes follow"
        );
        assert_eq!(rig.refs(), refs_before, "{name}: a preview moves nothing");
        assert_eq!(rig.run_dirs(), vec![plan.run.0.clone()], "{name}");

        let page = |offset, limit| {
            serde_json::to_value(RowsArgs {
                run: plan.run.clone(),
                filter: RowFilter::Changed,
                offset,
                limit,
            })
            .unwrap()
        };
        let first: RowsPage =
            serde_json::from_value(rig.ok(OperationKind::RewriteRows, page(0, 2)).await).unwrap();
        assert_eq!((first.total, first.rows.len()), (3, 2), "{name}");
        let rest: RowsPage =
            serde_json::from_value(rig.ok(OperationKind::RewriteRows, page(2, 2)).await).unwrap();
        assert_eq!(rest.rows.len(), 1, "{name}");
        assert!(first.rows.iter().all(|r| r.new.is_some()), "{name}");

        // Apply.
        let outcome = rig.apply(&plan).await;
        assert_eq!(outcome.commits_changed, 3, "{name}");
        let rewritten = rig.refs();
        assert_ne!(rewritten, refs_before, "{name}");
        let head_after = rig.rev("HEAD");
        assert_ne!(head_after, head_before, "{name}");
        assert_eq!(
            rig.authors()
                .lines()
                .collect::<std::collections::HashSet<_>>(),
            ["casey@example.com"].into_iter().collect(),
            "{name}: every author is the new one"
        );
        assert!(rig.run_dirs().iter().all(|d| d == &plan.run.0), "{name}");
        rig.fsck();

        // One entry, words and numbers in its label, pins on the old tips.
        let stack = rig.undo_stack().await;
        assert_eq!(stack.len(), entries_before + 1, "{name}: one entry");
        let entry = stack.last().unwrap();
        assert_eq!(entry.summary.kind, OperationKind::RewriteApply, "{name}");
        assert!(
            entry
                .summary
                .label
                .starts_with("Rewrite history: 3 commits, 2 branches, 2 tags"),
            "{name}: {}",
            entry.summary.label
        );
        let pins = rig.repo.git(&[
            "for-each-ref",
            "--format=%(objectname)",
            "refs/gitpoulpi/undo",
        ]);
        assert!(
            pins.contains(&head_before.to_hex()),
            "{name}: old tip pinned"
        );

        // The outside-change sync after our own rewrite records nothing.
        rig.ok(OperationKind::JournalState, json!({})).await;
        rig.ok(OperationKind::Status, json!({})).await;
        assert_eq!(
            rig.undo_stack().await.len(),
            entries_before + 1,
            "{name}: the rewrite was not recorded again as an outside change"
        );

        // Undo puts every ref back, the old objects included.
        let out: UndoOutcome =
            serde_json::from_value(rig.ok(OperationKind::Undo, json!({})).await).unwrap();
        assert_eq!(out.direction, Direction::Undo, "{name}");
        assert_eq!(rig.refs(), refs_before, "{name}: branches, tags and notes");
        assert_eq!(rig.rev("HEAD"), head_before, "{name}");
        assert!(
            rig.repo.git(&["status", "--porcelain"]).is_empty(),
            "{name}"
        );
        rig.fsck();

        // Redo goes to the very same new objects.
        rig.ok(OperationKind::Redo, json!({})).await;
        assert_eq!(rig.refs(), rewritten, "{name}");
        assert_eq!(rig.rev("HEAD"), head_after, "{name}");
        rig.fsck();
        rig.ok(OperationKind::JournalState, json!({})).await;
        assert_eq!(
            rig.undo_stack().await.len(),
            entries_before + 1,
            "{name}: undo and redo are not outside changes either"
        );
        rig.stop().await;
    }
}

/// The preview is on disk: a daemon that never saw it (a restart, or another
/// process) applies it, and a tampered `plan.json` is refused stale.
#[tokio::test]
async fn a_preview_survives_a_daemon_restart_and_a_tampered_one_is_refused() {
    for (name, backend) in backends() {
        let mut rig = Rig::new(backend.clone()).await;
        let plan = rig.preview().await;
        let run_dir = rig
            .repo
            .root()
            .join(".git/gitpoulpi/rewrite")
            .join(&plan.run.0);
        let file = run_dir.join("plan.json");
        let good = std::fs::read(&file).unwrap();

        // Tampered: refused, nothing moves.
        let mut stored: Value = serde_json::from_slice(&good).unwrap();
        stored["refs"][0]["new"] = json!(rig.rev("HEAD").to_hex());
        std::fs::write(&file, serde_json::to_vec(&stored).unwrap()).unwrap();
        let refs = rig.refs();
        rig.restart(backend.clone()).await;
        let args = Rig::apply_args(&plan, false);
        assert!(
            matches!(
                rig.refusal(OperationKind::RewriteApply, args).await,
                TypedError::RewriteRefused(RewriteRefusal::Stale, _)
            ),
            "{name}: tampered"
        );
        assert_eq!(rig.refs(), refs, "{name}");

        // Restored, restarted again: applied.
        std::fs::write(&file, &good).unwrap();
        rig.restart(backend.clone()).await;
        let outcome = rig.apply(&plan).await;
        assert_eq!(outcome.run, plan.run, "{name}");
        assert!(!rig.authors().contains("testkit@"), "{name}");
        // A spent run is stale.
        let args = Rig::apply_args(&plan, false);
        assert!(matches!(
            rig.refusal(OperationKind::RewriteApply, args).await,
            TypedError::RewriteRefused(RewriteRefusal::Stale, _)
        ));
        rig.stop().await;
    }
}

#[tokio::test]
async fn a_stale_plan_is_refused_typed_and_changes_and_journals_nothing() {
    for (name, backend) in backends() {
        let mut rig = Rig::new(backend).await;
        let plan = rig.preview().await;
        let entries = rig.undo_stack().await.len();
        let refs = rig.refs();

        // Not the digest the preview made.
        let mut wrong = plan.clone();
        wrong.digest.0.push('x');
        let args = Rig::apply_args(&wrong, false);
        assert!(
            matches!(
                rig.refusal(OperationKind::RewriteApply, args).await,
                TypedError::RewriteRefused(RewriteRefusal::Stale, _)
            ),
            "{name}: a wrong digest"
        );
        // Not a run the daemon knows.
        let mut unknown = plan.clone();
        unknown.run.0 = "nope".into();
        let args = Rig::apply_args(&unknown, false);
        assert!(
            matches!(
                rig.refusal(OperationKind::RewriteApply, args).await,
                TypedError::RewriteRefused(RewriteRefusal::Stale, _)
            ),
            "{name}: an unknown run"
        );
        assert_eq!(rig.refs(), refs, "{name}");
        assert_eq!(rig.undo_stack().await.len(), entries, "{name}");

        // A ref moved after the preview: the compare-and-swap refuses.
        rig.repo
            .git(&["commit", "-q", "--allow-empty", "-m", "moved on"]);
        let moved_refs = rig.refs();
        let args = Rig::apply_args(&plan, false);
        assert!(
            matches!(
                rig.refusal(OperationKind::RewriteApply, args).await,
                TypedError::RewriteRefused(RewriteRefusal::Stale, _)
            ),
            "{name}: a moved ref"
        );
        assert_eq!(rig.refs(), moved_refs, "{name}: nothing was rewritten");
        let stack = rig.undo_stack().await;
        assert!(
            stack
                .iter()
                .all(|e| e.summary.kind != OperationKind::RewriteApply),
            "{name}: a refused apply leaves no entry"
        );
        rig.fsck();
        rig.stop().await;
    }
}

#[tokio::test]
async fn a_new_preview_replaces_the_live_one_and_discard_removes_it() {
    for (name, backend) in backends() {
        let mut rig = Rig::new(backend).await;
        let first = rig.preview().await;
        let second = rig.preview().await;
        assert_ne!(first.run, second.run, "{name}");
        assert_eq!(
            rig.run_dirs(),
            vec![second.run.0.clone()],
            "{name}: the first run was discarded"
        );
        let rows = |run: &gitpoulpi_core::domain::rewrite::RunId| {
            serde_json::to_value(RowsArgs {
                run: run.clone(),
                filter: RowFilter::All,
                offset: 0,
                limit: 10,
            })
            .unwrap()
        };
        assert!(
            matches!(
                rig.refusal(OperationKind::RewriteRows, rows(&first.run))
                    .await,
                TypedError::RewriteRefused(RewriteRefusal::Stale, _)
            ),
            "{name}: the replaced plan is gone"
        );
        rig.ok(OperationKind::RewriteRows, rows(&second.run)).await;

        rig.ok(OperationKind::RewriteDiscard, json!({ "run": second.run }))
            .await;
        assert!(rig.run_dirs().is_empty(), "{name}");
        // Discarding twice is fine.
        rig.ok(OperationKind::RewriteDiscard, json!({ "run": second.run }))
            .await;
        assert!(
            matches!(
                rig.refusal(OperationKind::RewriteRows, rows(&second.run))
                    .await,
                TypedError::RewriteRefused(RewriteRefusal::Stale, _)
            ),
            "{name}"
        );
        rig.stop().await;
    }
}

#[tokio::test]
async fn pushed_refs_need_a_confirmation_to_apply() {
    for (name, backend) in backends() {
        let repo = FixtureRepoBuilder::new()
            .unwrap()
            .commit("alpha")
            .unwrap()
            .with_origin()
            .unwrap()
            .push("main")
            .unwrap()
            .track("main", "origin")
            .unwrap()
            .commit("beta")
            .unwrap()
            .build();
        let mut rig = Rig::on(backend, repo).await;
        let plan = rig.preview().await;
        assert!(plan.refs.iter().any(|r| matches!(
            r.pushed,
            gitpoulpi_core::domain::rewrite::PushState::OnRemote { .. }
        )));
        let refs = rig.refs();
        let args = Rig::apply_args(&plan, false);
        assert!(
            matches!(
                rig.refusal(OperationKind::RewriteApply, args).await,
                TypedError::NeedsConfirmation(_)
            ),
            "{name}"
        );
        assert_eq!(rig.refs(), refs, "{name}: nothing changed");
        let args = Rig::apply_args(&plan, true);
        rig.ok(OperationKind::RewriteApply, args).await;
        assert_ne!(rig.refs(), refs, "{name}");
        // Undone before anyone pushed the result: the remote still has the
        // old history, so there is nothing to warn about.
        rig.ok(OperationKind::Undo, json!({})).await;
        assert_eq!(rig.refs(), refs, "{name}");
        rig.ok(OperationKind::Redo, json!({})).await;
        // Once the rewritten branch is pushed, undoing it says so first.
        rig.repo.git(&["push", "-q", "--force", "origin", "main"]);
        let e = rig.call(OperationKind::Undo, json!({})).await.unwrap_err();
        assert!(
            matches!(e.kind, Some(TypedError::NeedsConfirmation(_))),
            "{name}: {e:?}"
        );
        rig.ok(OperationKind::Undo, json!({ "confirmed": true }))
            .await;
        assert_eq!(rig.refs(), refs, "{name}");
        rig.fsck();
        rig.stop().await;
    }
}

#[tokio::test]
async fn a_refused_recipe_arrives_typed() {
    for (name, backend) in backends() {
        let mut rig = Rig::new(backend).await;
        let empty = Recipe::new("nothing", Scope::AllLocal, vec![]);
        let args = serde_json::to_value(PreviewArgs {
            recipe: empty,
            vouched: Vec::new(),
        })
        .unwrap();
        assert!(
            matches!(
                rig.refusal(OperationKind::RewritePreview, args).await,
                TypedError::RewriteRefused(RewriteRefusal::NoRule, _)
            ),
            "{name}"
        );
        assert!(rig.run_dirs().is_empty(), "{name}");
        rig.stop().await;
    }
}

#[tokio::test]
async fn the_tools_are_a_plain_read() {
    for (name, backend) in backends() {
        let mut rig = Rig::new(backend).await;
        let tools: gitpoulpi_core::domain::rewrite::RewriteTools = serde_json::from_value(
            rig.ok(OperationKind::RewriteTools, json!({ "fresh": true }))
                .await,
        )
        .unwrap();
        assert!(!tools.tools.is_empty(), "{name}");
        rig.stop().await;
    }
}

// ---------------------------------------------------- jobs, progress, cancel

/// A second way in: raw frames, so a test sees events and can send a cancel
/// while a preview is still running on the same connection.
struct Raw {
    stream: TcpStream,
    repo: PathBuf,
    next: u64,
    events: Vec<Event>,
    answers: HashMap<u64, Result<Value, ResponseError>>,
}

impl Raw {
    async fn connect(port: u16, repo: &FixtureRepo) -> Raw {
        Raw {
            stream: TcpStream::connect(("127.0.0.1", port)).await.unwrap(),
            repo: repo.root().to_path_buf(),
            next: 100,
            events: Vec::new(),
            answers: HashMap::new(),
        }
    }

    async fn send(&mut self, command: OperationKind, args: Value, with_repo: bool) -> u64 {
        self.next += 1;
        let repo = with_repo.then(|| gitpoulpi_core::domain::RepoRef::from_root(&self.repo).id);
        write_message(
            &mut self.stream,
            &Request {
                id: self.next,
                session: SESSION.into(),
                repo,
                command,
                args,
            },
        )
        .await
        .unwrap();
        self.next
    }

    async fn pump(&mut self) {
        let message: ServerMessage =
            tokio::time::timeout(Duration::from_secs(60), read_message(&mut self.stream))
                .await
                .expect("the daemon answers")
                .unwrap()
                .expect("the daemon is there");
        match message {
            ServerMessage::Response(r) => {
                self.answers.insert(r.request_id, r.result);
            }
            ServerMessage::Event(e) => self.events.push(e),
        }
    }

    async fn wait(&mut self, id: u64) -> Result<Value, ResponseError> {
        while !self.answers.contains_key(&id) {
            self.pump().await;
        }
        self.answers.remove(&id).unwrap()
    }

    async fn jobs(&mut self) -> Vec<JobInfo> {
        let id = self.send(OperationKind::JobList, json!({}), false).await;
        serde_json::from_value(self.wait(id).await.unwrap()).unwrap()
    }
}

#[tokio::test]
async fn a_preview_is_a_job_that_reports_progress() {
    let backend: Arc<dyn GitBackend> = Arc::new(gitpoulpi_cli_backend::SystemGitBackend::new());
    let rig = Rig::new(backend).await;
    let port = rig.handle.as_ref().unwrap().port;
    let mut raw = Raw::connect(port, &rig.repo).await;
    let args = serde_json::to_value(PreviewArgs {
        recipe: recipe(),
        vouched: Vec::new(),
    })
    .unwrap();
    let id = raw.send(OperationKind::RewritePreview, args, true).await;
    let plan: RewritePlan = serde_json::from_value(raw.wait(id).await.unwrap()).unwrap();
    assert!(plan.rows.is_empty());

    // Give the broadcast a moment to reach this connection after the answer.
    for _ in 0..3 {
        raw.pump_for(Duration::from_millis(100)).await;
    }
    let phases: Vec<_> = raw
        .events
        .iter()
        .filter_map(|e| match e {
            Event::RewriteProgress { progress, .. } => Some(progress.phase),
            _ => None,
        })
        .collect();
    assert!(
        phases.len() >= 2,
        "progress events arrive (walking, rewriting, ...): {phases:?}"
    );
    let job = raw
        .jobs()
        .await
        .into_iter()
        .find(|j| j.kind == OperationKind::RewritePreview)
        .expect("the preview was a job");
    assert_eq!(job.state, JobState::Done);
    assert_eq!(job.progress, Some(100));
    rig.stop().await;
}

impl Raw {
    /// Reads whatever arrives within `window`.
    async fn pump_for(&mut self, window: Duration) {
        if let Ok(Ok(Some(message))) =
            tokio::time::timeout(window, read_message::<_, ServerMessage>(&mut self.stream)).await
        {
            match message {
                ServerMessage::Response(r) => {
                    self.answers.insert(r.request_id, r.result);
                }
                ServerMessage::Event(e) => self.events.push(e),
            }
        }
    }
}

#[tokio::test]
async fn canceling_a_preview_stops_it_and_leaves_nothing() {
    let backend: Arc<dyn GitBackend> = Arc::new(gitpoulpi_cli_backend::SystemGitBackend::new());
    let rig = Rig::new(backend).await;
    std::env::set_var(TICK_DELAY, "250");
    let refs = rig.refs();
    let port = rig.handle.as_ref().unwrap().port;
    let mut raw = Raw::connect(port, &rig.repo).await;
    let args = serde_json::to_value(PreviewArgs {
        recipe: recipe(),
        vouched: Vec::new(),
    })
    .unwrap();
    let id = raw.send(OperationKind::RewritePreview, args, true).await;

    // Find the job running, then cancel it.
    let deadline = Instant::now() + Duration::from_secs(30);
    let job = loop {
        if let Some(job) = raw
            .jobs()
            .await
            .into_iter()
            .find(|j| j.kind == OperationKind::RewritePreview && j.state == JobState::Running)
        {
            break job;
        }
        assert!(Instant::now() < deadline, "the preview never ran");
        tokio::time::sleep(Duration::from_millis(30)).await;
    };
    assert!(job.is_cancellable());
    let cancel = raw
        .send(OperationKind::JobCancel, json!({ "id": job.id }), false)
        .await;
    raw.wait(cancel).await.expect("the cancel is accepted");
    let answer = raw.wait(id).await.unwrap_err();
    std::env::remove_var(TICK_DELAY);
    assert_eq!(answer.kind, Some(TypedError::Canceled { job: job.id }));

    let after = raw
        .jobs()
        .await
        .into_iter()
        .find(|j| j.id == job.id)
        .unwrap();
    assert_eq!(after.state, JobState::Canceled);
    assert!(rig.run_dirs().is_empty(), "no run directory is left");
    assert_eq!(rig.refs(), refs);
    rig.fsck();
    // The daemon is fine: a new preview works.
    let mut rig = rig;
    rig.preview().await;
    rig.stop().await;
}

/// A content rewrite changes the checked-out files: apply updates the working
/// tree, undo and redo move the files with the commits, a plan reloaded after
/// a daemon restart does the same (it has no rows), and a dirty file the
/// rewrite would change is refused by the preview.
#[tokio::test]
async fn a_content_rewrite_updates_the_worktree_and_undo_restores_it() {
    use gitpoulpi_core::domain::rewrite::{ContentFind, ContentReplace, TextPattern};

    let content_recipe = || {
        let rule = Rule::new(RuleKind::ContentReplace(ContentReplace {
            paths: Vec::new(),
            find: ContentFind::Pattern {
                pattern: TextPattern::literal("EXAMPLE_TOKEN_123"),
            },
            with: "REDACTED".into(),
            max_blob_bytes: None,
        }));
        let mut recipe = Recipe::new("redact", Scope::default(), vec![rule]);
        recipe.options.signatures = Some(SignaturePolicy::Strip);
        recipe
    };
    for (name, backend) in backends() {
        let repo = FixtureRepoBuilder::new()
            .unwrap()
            .commit("alpha")
            .unwrap()
            .commit_file("config.txt", "key = EXAMPLE_TOKEN_123\n", "add config")
            .unwrap()
            .commit_file("notes.txt", "plain\n", "add notes")
            .unwrap()
            .build();
        let mut rig = Rig::on(backend.clone(), repo).await;
        let head_before = rig.rev("HEAD");
        let config = rig.repo.root().join("config.txt");
        let args = |recipe| {
            serde_json::to_value(PreviewArgs {
                recipe,
                vouched: Vec::new(),
            })
            .unwrap()
        };

        // A local edit to a file the rewrite would change is refused.
        std::fs::write(&config, "key = EXAMPLE_TOKEN_123\nmine\n").unwrap();
        let refusal = rig
            .refusal(OperationKind::RewritePreview, args(content_recipe()))
            .await;
        assert!(
            matches!(
                refusal,
                TypedError::RewriteRefused(RewriteRefusal::DirtyPathsTouched, _)
            ),
            "{name}: {refusal:?}"
        );
        rig.repo.git(&["checkout", "--", "config.txt"]);

        // Apply: the commits and the file change, as one entry.
        let entries_before = rig.undo_stack().await.len();
        let plan: RewritePlan = serde_json::from_value(
            rig.ok(OperationKind::RewritePreview, args(content_recipe()))
                .await,
        )
        .unwrap();
        assert!(plan.counts.head_tree_changed, "{name}");
        rig.apply(&plan).await;
        assert_ne!(rig.rev("HEAD"), head_before, "{name}");
        assert_eq!(
            std::fs::read_to_string(&config).unwrap(),
            "key = REDACTED\n",
            "{name}: the working tree followed"
        );
        assert!(
            rig.repo.git(&["status", "--porcelain"]).is_empty(),
            "{name}"
        );
        assert_eq!(rig.undo_stack().await.len(), entries_before + 1, "{name}");
        let rewritten = rig.rev("HEAD");

        // Undo puts the old file back with the old commits.
        rig.ok(OperationKind::Undo, json!({})).await;
        assert_eq!(rig.rev("HEAD"), head_before, "{name}");
        assert_eq!(
            std::fs::read_to_string(&config).unwrap(),
            "key = EXAMPLE_TOKEN_123\n",
            "{name}: undo restored the file"
        );
        assert!(
            rig.repo.git(&["status", "--porcelain"]).is_empty(),
            "{name}"
        );

        // Redo goes forward, files included.
        rig.ok(OperationKind::Redo, json!({})).await;
        assert_eq!(rig.rev("HEAD"), rewritten, "{name}");
        assert_eq!(
            std::fs::read_to_string(&config).unwrap(),
            "key = REDACTED\n",
            "{name}"
        );
        rig.ok(OperationKind::Undo, json!({})).await;

        // A preview that outlived its daemon: the reloaded plan has no rows
        // and still moves the files, and undo still restores them.
        let plan: RewritePlan = serde_json::from_value(
            rig.ok(OperationKind::RewritePreview, args(content_recipe()))
                .await,
        )
        .unwrap();
        rig.restart(backend.clone()).await;
        rig.apply(&plan).await;
        assert_eq!(
            std::fs::read_to_string(&config).unwrap(),
            "key = REDACTED\n",
            "{name}: after a restart"
        );
        rig.ok(OperationKind::Undo, json!({})).await;
        assert_eq!(
            std::fs::read_to_string(&config).unwrap(),
            "key = EXAMPLE_TOKEN_123\n",
            "{name}: undo after a restart"
        );
        rig.fsck();
        rig.stop().await;
    }
}

// ---------------------------------------------------------- push (slice S13)

fn pushed_repo() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit("alpha")
        .unwrap()
        .with_origin()
        .unwrap()
        .push("main")
        .unwrap()
        .track("main", "origin")
        .unwrap()
        .commit("beta")
        .unwrap()
        .build()
}

fn origin_main(repo: &FixtureRepo) -> String {
    let dir = repo.remote_path().unwrap().to_string_lossy().into_owned();
    repo.git(&["--git-dir", &dir, "rev-parse", "refs/heads/main"])
}

/// `main` is protected by default: a pushed `main` is not rewritten unless
/// the run lists it, and the refusal is typed.
#[tokio::test]
async fn a_pushed_main_is_protected_by_default() {
    for (name, backend) in backends() {
        let mut rig = Rig::on(backend, pushed_repo()).await;
        let mut plain = recipe();
        plain.scope_options.include_protected.clear();
        let args = serde_json::to_value(PreviewArgs {
            recipe: plain,
            vouched: Vec::new(),
        })
        .unwrap();
        assert!(
            matches!(
                rig.refusal(OperationKind::RewritePreview, args).await,
                TypedError::RewriteRefused(RewriteRefusal::ProtectedRef, _)
            ),
            "{name}"
        );
        rig.stop().await;
    }
}

/// The push is a network job with a lease per ref; protected refs wait for
/// an explicit include; the undo of the rewrite afterwards warns that the
/// remote has the new history, and leaves the remote alone.
#[tokio::test]
async fn a_rewrite_is_pushed_as_a_job_and_the_undo_warns() {
    use gitpoulpi_core::domain::rewrite::{PushRefStatus, PushRewritten, PushRewrittenOutcome};
    for (name, backend) in backends() {
        let mut rig = Rig::on(backend, pushed_repo()).await;
        let old_remote = origin_main(&rig.repo);
        let plan = rig.preview().await;
        let refs_before_apply = rig.refs();
        let args = Rig::apply_args(&plan, true);
        rig.ok(OperationKind::RewriteApply, args).await;
        let new_main = rig.repo.git(&["rev-parse", "main"]);
        assert_ne!(new_main, old_remote, "{name}");

        // The maps and the note come from the applied run.
        let note: gitpoulpi_core::domain::rewrite::RewriteNote = serde_json::from_value(
            rig.ok(OperationKind::RewriteNote, json!({ "run": plan.run }))
                .await,
        )
        .unwrap();
        assert_eq!(note.refs.len(), 1, "{name}");
        assert!(note.text.contains("git reset --hard origin/main"), "{name}");

        // A job; main is protected, so it is left out.
        let port = rig.handle.as_ref().unwrap().port;
        let mut raw = Raw::connect(port, &rig.repo).await;
        let args = serde_json::to_value(PushRewritten::new(plan.run.clone())).unwrap();
        let id = raw.send(OperationKind::RewritePush, args, true).await;
        let out: PushRewrittenOutcome =
            serde_json::from_value(raw.wait(id).await.unwrap()).unwrap();
        assert_eq!(out.results.len(), 1, "{name}");
        assert_eq!(
            out.results[0].status,
            PushRefStatus::SkippedProtected,
            "{name}"
        );
        assert_eq!(origin_main(&rig.repo), old_remote, "{name}: nothing sent");
        let job = raw
            .jobs()
            .await
            .into_iter()
            .find(|j| j.kind == OperationKind::RewritePush)
            .expect("the push was a job");
        assert_eq!(job.state, JobState::Done, "{name}");

        // Asked for: pushed with the lease the run recorded.
        let mut ask = PushRewritten::new(plan.run.clone());
        ask.include_protected = vec!["main".into()];
        let out: PushRewrittenOutcome = serde_json::from_value(
            rig.ok(
                OperationKind::RewritePush,
                serde_json::to_value(ask).unwrap(),
            )
            .await,
        )
        .unwrap();
        assert_eq!(
            out.results[0].status,
            PushRefStatus::Pushed,
            "{name}: {out:?}"
        );
        assert_eq!(origin_main(&rig.repo), new_main, "{name}");

        // Undo: refused until confirmed, and it says the remote has the new
        // history (even without an upstream setting to find it by).
        let e = rig.call(OperationKind::Undo, json!({})).await.unwrap_err();
        assert!(
            matches!(e.kind, Some(TypedError::NeedsConfirmation(_))),
            "{name}: {e:?}"
        );
        assert!(
            e.message.contains("was pushed to origin"),
            "{name}: {}",
            e.message
        );
        rig.ok(OperationKind::Undo, json!({ "confirmed": true }))
            .await;
        assert_eq!(rig.refs(), refs_before_apply, "{name}");
        assert_eq!(
            origin_main(&rig.repo),
            new_main,
            "{name}: the remote keeps it"
        );
        rig.fsck();
        rig.stop().await;
    }
}

/// Secret scanning through the daemon: a job that reports progress, whose
/// answer, events and log lines hold no secret value; a recipe that names a
/// finding by id previews, applies and undoes as one journal entry; the scan
/// can be canceled.
#[tokio::test]
async fn a_secret_scan_is_a_job_and_no_value_reaches_the_wire_or_the_log() {
    use gitpoulpi_core::domain::rewrite::{ContentFind, ContentReplace, SecretScan, SecretText};
    use gitpoulpi_ipc::rewrite::ScanSecretsArgs;

    // The application log, in memory, at its most talkative level.
    let logger = gitpoulpi_log::init(gitpoulpi_log::Options {
        dir: None,
        level: gitpoulpi_log::Level::Trace,
        retention_days: 1,
        max_file_bytes: 1 << 20,
        process: "daemon",
    });
    let token = format!("ghp_{}", "Zk9xQ2mL7vB4nR8tY1wC5dF3hJ6pS0aEgUiX");
    // A secret `redact` does not recognise by itself (no known prefix, and
    // `DB_PASSWORD` is not a key it masks): the log's silence about it is the
    // scanner's own.
    let password = "Zq8!vK3mX9wL";

    for (name, backend) in backends() {
        let repo = FixtureRepoBuilder::new()
            .unwrap()
            .commit("alpha")
            .unwrap()
            .commit_file("deploy.env", &format!("GITHUB_TOKEN={token}\n"), "add env")
            .unwrap()
            .commit_file("app.env", &format!("DB_PASSWORD={password}\n"), "add db")
            .unwrap()
            .commit_file("README.md", "docs\n", "add readme")
            .unwrap()
            .build();
        let mut rig = Rig::on(backend, repo).await;
        let head_before = rig.rev("HEAD");
        let mut wire = String::new();

        // The scan, as a job on a raw connection so its events are seen too.
        let port = rig.handle.as_ref().unwrap().port;
        let mut raw = Raw::connect(port, &rig.repo).await;
        let args = serde_json::to_value(ScanSecretsArgs::default()).unwrap();
        let id = raw
            .send(OperationKind::RewriteScanSecrets, args, true)
            .await;
        let answer = raw.wait(id).await.unwrap();
        for _ in 0..3 {
            raw.pump_for(Duration::from_millis(100)).await;
        }
        wire.push_str(&answer.to_string());
        for event in &raw.events {
            wire.push_str(&serde_json::to_string(event).unwrap());
        }
        let scan: SecretScan = serde_json::from_value(answer).unwrap();
        assert_eq!(scan.findings.len(), 2, "{name}: {scan:?}");
        let finding = scan
            .findings
            .iter()
            .find(|f| f.path == "deploy.env")
            .unwrap()
            .clone();
        assert_eq!(finding.path, "deploy.env", "{name}");
        assert_eq!(
            finding.preview,
            format!("ghp_****{}", &token[token.len() - 4..]),
            "{name}"
        );
        assert!(
            raw.events
                .iter()
                .any(|e| matches!(e, Event::RewriteProgress { .. })),
            "{name}: progress events"
        );
        let job = raw
            .jobs()
            .await
            .into_iter()
            .find(|j| j.kind == OperationKind::RewriteScanSecrets)
            .expect("the scan was a job");
        assert_eq!(job.state, JobState::Done, "{name}");
        wire.push_str(&serde_json::to_string(&job).unwrap());

        // A finding id in a rule: preview, apply, one journal entry, undo.
        let rule = Rule::new(RuleKind::ContentReplace(ContentReplace {
            paths: Vec::new(),
            find: ContentFind::Secrets {
                literals: vec![SecretText(finding.id.clone())],
                scan: true,
            },
            with: String::new(),
            max_blob_bytes: None,
        }));
        let mut recipe = Recipe::new("remove", Scope::default(), vec![rule]);
        recipe.options.signatures = Some(SignaturePolicy::Strip);
        let sent = serde_json::to_value(PreviewArgs {
            recipe,
            vouched: Vec::new(),
        })
        .unwrap();
        assert!(
            !sent.to_string().contains(&token),
            "{name}: the request holds only the id"
        );
        let entries_before = rig.undo_stack().await.len();
        let plan_value = rig.ok(OperationKind::RewritePreview, sent).await;
        wire.push_str(&plan_value.to_string());
        let plan: RewritePlan = serde_json::from_value(plan_value).unwrap();
        assert_eq!(plan.counts.commits_changed, 3, "{name}");
        let outcome = rig.apply(&plan).await;
        wire.push_str(&serde_json::to_string(&outcome).unwrap());
        let log = rig.repo.git(&["log", "-p", "--branches", "--tags"]);
        assert!(!log.contains(&token), "{name}: gone from every commit");
        assert_eq!(rig.undo_stack().await.len(), entries_before + 1, "{name}");
        rig.ok(OperationKind::Undo, json!({})).await;
        assert_eq!(rig.rev("HEAD"), head_before, "{name}: undo");
        assert!(
            rig.repo.git(&["log", "-p", "--branches"]).contains(&token),
            "{name}: the old history is back"
        );
        rig.fsck();
        for page in rig.undo_stack().await {
            wire.push_str(&serde_json::to_string(&page.summary).unwrap());
        }

        // Nothing that crossed the wire held a value, whole or in part.
        for value in [token.as_str(), &token[4..], password] {
            assert!(!wire.contains(value), "{name}: a value crossed the wire");
        }
        rig.stop().await;
    }

    // The log took part (so the search below means something) and holds no value.
    let lines: Vec<String> = logger.recent().iter().map(|r| r.to_line()).collect();
    assert!(
        lines.iter().any(|l| l.contains("RewriteScanSecrets")),
        "the log recorded the scans: {} line(s)",
        lines.len()
    );
    let text = lines.join("\n");
    for value in [token.as_str(), &token[4..], password] {
        assert!(!text.contains(value), "a value reached the log");
    }
}

#[tokio::test]
async fn canceling_a_secret_scan_stops_it() {
    use gitpoulpi_ipc::rewrite::ScanSecretsArgs;

    let backend: Arc<dyn GitBackend> = Arc::new(gitpoulpi_cli_backend::SystemGitBackend::new());
    let rig = Rig::new(backend).await;
    std::env::set_var(TICK_DELAY, "250");
    let port = rig.handle.as_ref().unwrap().port;
    let mut raw = Raw::connect(port, &rig.repo).await;
    let args = serde_json::to_value(ScanSecretsArgs::default()).unwrap();
    let id = raw
        .send(OperationKind::RewriteScanSecrets, args, true)
        .await;
    let deadline = Instant::now() + Duration::from_secs(30);
    let job =
        loop {
            if let Some(job) = raw.jobs().await.into_iter().find(|j| {
                j.kind == OperationKind::RewriteScanSecrets && j.state == JobState::Running
            }) {
                break job;
            }
            assert!(Instant::now() < deadline, "the scan never ran");
            tokio::time::sleep(Duration::from_millis(30)).await;
        };
    assert!(job.is_cancellable());
    let cancel = raw
        .send(OperationKind::JobCancel, json!({ "id": job.id }), false)
        .await;
    raw.wait(cancel).await.expect("the cancel is accepted");
    let answer = raw.wait(id).await.unwrap_err();
    std::env::remove_var(TICK_DELAY);
    assert_eq!(answer.kind, Some(TypedError::Canceled { job: job.id }));
    rig.stop().await;
}

/// The size analysis through the daemon: a job that reports progress, answers
/// with the typed rows, changes no ref and records no journal entry; it can
/// be canceled.
#[tokio::test]
async fn a_size_analysis_is_a_job_with_progress_and_changes_nothing() {
    use gitpoulpi_core::domain::rewrite::SizeAnalysis;
    use gitpoulpi_ipc::rewrite::AnalyzeArgs;

    for (name, backend) in backends() {
        let repo = FixtureRepoBuilder::new()
            .unwrap()
            .commit("alpha")
            .unwrap()
            .commit_file("big.bin", &"x".repeat(5000), "add big")
            .unwrap()
            .commit_file("README.md", "docs\n", "add readme")
            .unwrap()
            .build();
        let rig = Rig::on(backend, repo).await;
        let head_before = rig.rev("HEAD");
        let port = rig.handle.as_ref().unwrap().port;
        let mut raw = Raw::connect(port, &rig.repo).await;
        let args = serde_json::to_value(AnalyzeArgs::default()).unwrap();
        let id = raw.send(OperationKind::RewriteAnalyze, args, true).await;
        let answer = raw.wait(id).await.unwrap();
        for _ in 0..3 {
            raw.pump_for(Duration::from_millis(100)).await;
        }
        let found: SizeAnalysis = serde_json::from_value(answer).unwrap();
        assert_eq!(found.blobs[0].path, "big.bin", "{name}: {found:?}");
        assert_eq!(found.blobs[0].size, 5000, "{name}");
        assert_eq!(found.top, 50, "{name}: 0 means the default");
        assert!(
            raw.events
                .iter()
                .any(|e| matches!(e, Event::RewriteProgress { .. })),
            "{name}: progress events"
        );
        let job = raw
            .jobs()
            .await
            .into_iter()
            .find(|j| j.kind == OperationKind::RewriteAnalyze)
            .expect("the analysis was a job");
        assert_eq!(job.state, JobState::Done, "{name}");
        assert_eq!(rig.rev("HEAD"), head_before, "{name}: a read");
        rig.stop().await;
    }
}

#[tokio::test]
async fn canceling_a_size_analysis_stops_it() {
    use gitpoulpi_ipc::rewrite::AnalyzeArgs;

    let backend: Arc<dyn GitBackend> = Arc::new(gitpoulpi_cli_backend::SystemGitBackend::new());
    let rig = Rig::new(backend).await;
    std::env::set_var(TICK_DELAY, "250");
    let port = rig.handle.as_ref().unwrap().port;
    let mut raw = Raw::connect(port, &rig.repo).await;
    let args = serde_json::to_value(AnalyzeArgs::default()).unwrap();
    let id = raw.send(OperationKind::RewriteAnalyze, args, true).await;
    let deadline = Instant::now() + Duration::from_secs(30);
    let job = loop {
        if let Some(job) = raw
            .jobs()
            .await
            .into_iter()
            .find(|j| j.kind == OperationKind::RewriteAnalyze && j.state == JobState::Running)
        {
            break job;
        }
        assert!(Instant::now() < deadline, "the analysis never ran");
        tokio::time::sleep(Duration::from_millis(30)).await;
    };
    assert!(job.is_cancellable());
    let cancel = raw
        .send(OperationKind::JobCancel, json!({ "id": job.id }), false)
        .await;
    raw.wait(cancel).await.expect("the cancel is accepted");
    let answer = raw.wait(id).await.unwrap_err();
    std::env::remove_var(TICK_DELAY);
    assert_eq!(answer.kind, Some(TypedError::Canceled { job: job.id }));
    rig.stop().await;
}

// --------------------------------------------------------- purge (slice S15)

/// `from` becomes `to` (a second rewrite on top of the first).
fn recipe_for(from: &str, to: (&str, &str)) -> Recipe {
    let rule = Rule::new(RuleKind::Identity(IdentityRule {
        when: IdentityMatch::email(from),
        set: IdentitySet {
            name: Some(to.0.into()),
            email: Some(to.1.into()),
        },
        apply_to: Who::Author,
        include_taggers: false,
        map_trailers: false,
    }));
    let mut recipe = Recipe::new("again", Scope::AllLocal, vec![rule]);
    recipe.options.signatures = Some(SignaturePolicy::Strip);
    recipe
}

async fn apply_recipe(rig: &mut Rig, recipe: Recipe) -> RewriteOutcome {
    let args = serde_json::to_value(PreviewArgs {
        recipe,
        vouched: Vec::new(),
    })
    .unwrap();
    let plan: RewritePlan =
        serde_json::from_value(rig.ok(OperationKind::RewritePreview, args).await).unwrap();
    rig.apply(&plan).await
}

#[tokio::test]
async fn a_purged_rewrite_is_listed_purged_and_undo_is_refused_with_the_typed_reason() {
    use gitpoulpi_core::domain::rewrite::{PurgeCheck, PurgeOutcome, PurgeStatus};
    use gitpoulpi_core::domain::{EntryState, JournalState, UndoRefusal};
    for (name, backend) in backends() {
        let mut rig = Rig::new(backend).await;
        let original_tip = rig.rev("HEAD");
        apply_recipe(&mut rig, recipe()).await;
        apply_recipe(
            &mut rig,
            recipe_for("casey@example.com", ("Dana Example", "dana@example.com")),
        )
        .await;
        let stack = rig.undo_stack().await;
        let ids: Vec<u64> = stack
            .iter()
            .filter(|e| e.summary.kind == OperationKind::RewriteApply)
            .map(|e| e.summary.id)
            .collect();
        assert_eq!(ids.len(), 2, "{name}");

        // The confirmation lists both entries, what is lost, and the hosts.
        let check: PurgeCheck = serde_json::from_value(
            rig.ok(OperationKind::RewritePurgeCheck, json!({ "all": true }))
                .await,
        )
        .unwrap();
        assert_eq!(
            check.entries.iter().map(|e| e.id).collect::<Vec<_>>(),
            ids,
            "{name}"
        );
        assert!(
            check.entries[0].label.starts_with("Rewrite history"),
            "{name}"
        );
        assert_eq!(
            check.outcome.status,
            PurgeStatus::Ready,
            "{name}: {check:?}"
        );
        assert!(check.outcome.old_commits >= 6, "{name}: {check:?}");
        assert!(check.outcome.reflog_entries > 0, "{name}");
        assert!(check.can_run(), "{name}");
        assert!(
            check
                .prompt
                .contains(&format!("#{} Rewrite history", ids[0]))
                && check
                    .prompt
                    .contains(&format!("#{} Rewrite history", ids[1]))
                && check
                    .prompt
                    .contains("Undo will no longer be possible for 2 operations"),
            "{name}: {}",
            check.prompt
        );
        assert!(check.hosts.is_empty(), "{name}: no remote, no host steps");
        // A check changes nothing.
        assert!(rig
            .repo
            .try_git(&["cat-file", "-e", &original_tip.to_hex()])
            .is_some());

        // The default selection is the newest rewrite only.
        let latest: PurgeCheck =
            serde_json::from_value(rig.ok(OperationKind::RewritePurgeCheck, json!({})).await)
                .unwrap();
        assert_eq!(latest.entries.len(), 1, "{name}");
        assert_eq!(latest.entries[0].id, ids[1], "{name}");

        // The purge is a job that runs to the end.
        let port = rig.handle.as_ref().unwrap().port;
        let mut raw = Raw::connect(port, &rig.repo).await;
        let id = raw
            .send(OperationKind::RewritePurge, json!({ "all": true }), true)
            .await;
        let out: PurgeOutcome = serde_json::from_value(raw.wait(id).await.unwrap()).unwrap();
        assert_eq!(out.status, PurgeStatus::Purged, "{name}: {out:?}");
        let job = raw
            .jobs()
            .await
            .into_iter()
            .find(|j| j.kind == OperationKind::RewritePurge)
            .expect("the purge was a job");
        assert_eq!(job.state, JobState::Done, "{name}");

        // The old objects and every pin are gone; the new history is whole.
        assert!(
            rig.repo
                .try_git(&["cat-file", "-e", &original_tip.to_hex()])
                .is_none(),
            "{name}: the original tip is still there"
        );
        assert!(
            rig.repo
                .git(&["for-each-ref", "refs/gitpoulpi/undo"])
                .is_empty(),
            "{name}: pins"
        );
        rig.repo.git(&["fsck", "--full", "--strict"]);

        // The entries stay in the list, marked.
        let stack = rig.undo_stack().await;
        for id in &ids {
            let entry = stack.iter().find(|e| e.summary.id == *id).unwrap();
            assert_eq!(entry.summary.state, EntryState::Purged, "{name}");
        }

        // Undo is refused with the typed reason, and changes nothing.
        let head = rig.rev("HEAD");
        let refusal = rig.refusal(OperationKind::Undo, json!({})).await;
        assert_eq!(
            refusal,
            TypedError::UndoRefused(UndoRefusal::Purged { entry: ids[1] }),
            "{name}"
        );
        assert_eq!(rig.rev("HEAD"), head, "{name}");
        let state: JournalState =
            serde_json::from_value(rig.ok(OperationKind::JournalState, json!({})).await).unwrap();
        assert_eq!(
            state.blocked,
            Some(UndoRefusal::Purged { entry: ids[1] }),
            "{name}"
        );
        rig.stop().await;
    }
}

#[tokio::test]
async fn a_blocked_purge_changes_nothing_and_leaves_the_entries_undoable() {
    use gitpoulpi_core::domain::rewrite::{
        PurgeBlockerKind, PurgeCheck, PurgeOutcome, PurgeStatus,
    };
    use gitpoulpi_core::domain::EntryState;
    for (name, backend) in backends() {
        let mut rig = Rig::new(backend).await;
        // A stash on the old history keeps it alive.
        std::fs::write(rig.repo.root().join("wip.txt"), "wip\n").unwrap();
        rig.repo.git(&["add", "wip.txt"]);
        rig.repo.git(&["stash", "push", "-q", "-m", "wip"]);
        let head_before = rig.rev("HEAD");
        apply_recipe(&mut rig, recipe()).await;

        let check: PurgeCheck =
            serde_json::from_value(rig.ok(OperationKind::RewritePurgeCheck, json!({})).await)
                .unwrap();
        assert!(!check.can_run(), "{name}");
        assert_eq!(check.outcome.blockers.len(), 1, "{name}: {check:?}");
        assert_eq!(check.outcome.blockers[0].kind, PurgeBlockerKind::Stash);

        let out: PurgeOutcome =
            serde_json::from_value(rig.ok(OperationKind::RewritePurge, json!({})).await).unwrap();
        assert_eq!(out.status, PurgeStatus::Blocked, "{name}");
        assert!(rig
            .repo
            .try_git(&["cat-file", "-e", &head_before.to_hex()])
            .is_some());
        let stack = rig.undo_stack().await;
        assert!(
            stack.iter().all(|e| e.summary.state == EntryState::Done),
            "{name}: nothing was marked"
        );

        // No such entry: a typed refusal, nothing done.
        let kind = rig
            .refusal(OperationKind::RewritePurge, json!({ "entries": [9999] }))
            .await;
        assert!(
            matches!(kind, TypedError::RewriteRefused(RewriteRefusal::Stale, _)),
            "{name}: {kind:?}"
        );

        // The undo still works, and so does the purge once the stash is gone.
        rig.ok(OperationKind::Undo, json!({})).await;
        assert_eq!(rig.rev("HEAD"), head_before, "{name}");
        rig.ok(OperationKind::Redo, json!({})).await;
        rig.repo.git(&["stash", "drop", "-q"]);
        // The earlier operation whose pin holds the old history joins the
        // purge: its undo goes with the objects it needs.
        let check: PurgeCheck =
            serde_json::from_value(rig.ok(OperationKind::RewritePurgeCheck, json!({})).await)
                .unwrap();
        assert!(check.can_run(), "{name}: {check:?}");
        assert!(
            check.entries.iter().any(|e| !e.rewrite) && check.entries.iter().any(|e| e.rewrite),
            "{name}: {check:?}"
        );
        let out: PurgeOutcome =
            serde_json::from_value(rig.ok(OperationKind::RewritePurge, json!({})).await).unwrap();
        assert_eq!(out.status, PurgeStatus::Purged, "{name}: {out:?}");
        rig.stop().await;
    }
}

/// S21: an LFS import through the daemon. The working tree shows the real
/// file after the apply (smudged, nothing modified), undo puts the real blob
/// back, redo does the import again; the file on disk never changes.
#[tokio::test]
async fn an_lfs_import_keeps_real_files_on_disk_through_apply_undo_and_redo() {
    gitpoulpi_testkit::require_git_lfs!("an_lfs_import_keeps_real_files_on_disk");
    use gitpoulpi_core::domain::rewrite::LfsImportRule;
    for (name, backend) in backends() {
        let content: Vec<u8> = (0..4000u32).map(|i| (i * 7) as u8).collect();
        let repo = FixtureRepoBuilder::new()
            .unwrap()
            .commit("alpha")
            .unwrap()
            .commit_bytes("art.bin", &content, "add art")
            .unwrap()
            .build();
        let mut rig = Rig::on(backend, repo).await;
        let head_before = rig.rev("HEAD");
        let blob_before = rig.repo.git(&["rev-parse", "HEAD:art.bin"]);

        let mut recipe = Recipe::new(
            "lfs",
            Scope::CurrentBranch { since: None },
            vec![Rule::new(RuleKind::LfsImport(LfsImportRule {
                patterns: vec!["*.bin".into()],
                min_size: None,
            }))],
        );
        recipe.options.signatures = Some(SignaturePolicy::Strip);
        recipe.scope_options.include_protected = vec!["main".into()];
        let args = serde_json::to_value(PreviewArgs {
            recipe,
            vouched: Vec::new(),
        })
        .unwrap();
        let plan: RewritePlan =
            serde_json::from_value(rig.ok(OperationKind::RewritePreview, args).await).unwrap();
        assert_eq!(plan.counts.lfs_blobs, 1, "{name}");
        rig.apply(&plan).await;

        let on_disk = |rig: &Rig| std::fs::read(rig.repo.root().join("art.bin")).unwrap();
        assert_ne!(rig.repo.git(&["rev-parse", "HEAD:art.bin"]), blob_before);
        assert_eq!(on_disk(&rig), content, "{name}: smudged after the apply");
        assert_eq!(rig.repo.git(&["status", "--porcelain"]), "", "{name}");

        rig.ok(OperationKind::Undo, json!({})).await;
        assert_eq!(rig.rev("HEAD"), head_before, "{name}");
        assert_eq!(rig.repo.git(&["rev-parse", "HEAD:art.bin"]), blob_before);
        assert_eq!(on_disk(&rig), content, "{name}: the real file after undo");
        assert_eq!(rig.repo.git(&["status", "--porcelain"]), "", "{name}");

        rig.ok(OperationKind::Redo, json!({})).await;
        assert_ne!(rig.repo.git(&["rev-parse", "HEAD:art.bin"]), blob_before);
        assert_eq!(on_disk(&rig), content, "{name}: the real file after redo");
        assert_eq!(rig.repo.git(&["status", "--porcelain"]), "", "{name}");
        rig.stop().await;
    }
}

// No test reads or writes the real config, data dir or credentials of whoever runs it.
gitpoulpi_testkit::isolate_binary!();
