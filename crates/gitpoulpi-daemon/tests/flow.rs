//! The branching-workflow actions through the daemon (`FlowPlan`, `FlowRun`):
//! start, finish (merge, squash, rebase, merge request, double finish with a
//! tag), sync with a stack, promote, set-parent, hooks, conflicts mid-finish
//! (continue and abort) and Undo/Redo of the composite entries. Every test
//! runs on both backends against fixture repositories.

use std::sync::{Arc, OnceLock};

use gitpoulpi_core::domain::{OperationKind, RepoId};
use gitpoulpi_core::workflow::{
    FlowOutcome, FlowPlan, FlowStatus, Hooks, HooksReport, HooksTrust, MergeStrategy, StepStatus,
    Workflow, WorkflowTemplate,
};
use gitpoulpi_core::GitBackend;
use gitpoulpi_core::TypedError;
use gitpoulpi_daemon::Daemon;
use gitpoulpi_ipc::{Client, Request, ResponseError};
use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use serde_json::{json, Value};

/// The process environment is set once, before the first daemon starts
/// (`set_var` while another thread reads the environment is undefined
/// behavior); the pre-main isolation in `isolate_binary!` already gave this
/// binary its own config dir.
fn isolate() {
    static ONCE: OnceLock<()> = OnceLock::new();
    ONCE.get_or_init(|| {
        std::env::set_var("GITPOULPI_GENERAL_JOURNAL_OUTSIDE", "false");
    });
}

fn backends() -> Vec<(&'static str, Arc<dyn GitBackend>)> {
    vec![
        (
            "system",
            Arc::new(gitpoulpi_cli_backend::SystemGitBackend::new()),
        ),
        ("git2", Arc::new(gitpoulpi_git2_backend::Git2Backend::new())),
    ]
}

struct Rig {
    handle: Option<gitpoulpi_daemon::DaemonHandle>,
    client: Client,
    repo: FixtureRepo,
    id: RepoId,
    next: u64,
}

fn write_workflow(repo: &FixtureRepo, workflow: &Workflow) {
    // The state directory is ignored, so the file survives every checkout
    // and never makes the tree dirty.
    let exclude = repo.root().join(".git").join("info").join("exclude");
    std::fs::create_dir_all(exclude.parent().unwrap()).unwrap();
    std::fs::write(&exclude, "/.gitpoulpi/\n").unwrap();
    let path = gitpoulpi_session::workflow_path_for(
        repo.root(),
        gitpoulpi_config::DEFAULT_REPO_DATA_LOCATION,
    );
    gitpoulpi_session::save_shared(&path, workflow, true).unwrap();
}

impl Rig {
    async fn new(backend: Arc<dyn GitBackend>, repo: FixtureRepo, workflow: &Workflow) -> Rig {
        isolate();
        write_workflow(&repo, workflow);
        let handle = Daemon::new("flow-test").spawn(backend).await.unwrap();
        let client = Client::connect(&handle.endpoint()).await.unwrap();
        let id = repo.repo_ref().id;
        Rig {
            handle: Some(handle),
            client,
            repo,
            id,
            next: 1,
        }
    }

    async fn call(&mut self, command: OperationKind, args: Value) -> Result<Value, ResponseError> {
        self.next += 1;
        self.client
            .call(&Request {
                id: self.next,
                session: "flow-test".into(),
                repo: Some(self.id.clone()),
                command,
                args,
            })
            .await
            .unwrap()
            .result
    }

    async fn plan(&mut self, args: Value) -> FlowPlan {
        let value = self.call(OperationKind::FlowPlan, args).await.unwrap();
        serde_json::from_value(value).unwrap()
    }

    async fn run(&mut self, args: Value) -> FlowOutcome {
        match self.try_run(args).await {
            Ok(o) => o,
            Err(e) => panic!("flow run failed: {}", e.message),
        }
    }

    async fn try_run(&mut self, args: Value) -> Result<FlowOutcome, ResponseError> {
        let value = self.call(OperationKind::FlowRun, args).await?;
        Ok(serde_json::from_value(value).unwrap())
    }

    /// What a person does in the dialog: reads the hooks, then trusts that hash.
    async fn trust_hooks(&mut self) -> HooksReport {
        let seen: HooksReport = serde_json::from_value(
            self.call(OperationKind::FlowTrustHooks, json!({}))
                .await
                .unwrap(),
        )
        .unwrap();
        serde_json::from_value(
            self.call(OperationKind::FlowTrustHooks, json!({ "hash": seen.hash }))
                .await
                .unwrap(),
        )
        .unwrap()
    }

    async fn undo(&mut self) -> Value {
        self.call(OperationKind::Undo, json!({})).await.unwrap()
    }

    async fn redo(&mut self) -> Value {
        self.call(OperationKind::Redo, json!({})).await.unwrap()
    }

    fn head(&self) -> String {
        self.repo
            .git(&["rev-parse", "--abbrev-ref", "HEAD"])
            .trim()
            .to_string()
    }

    fn tip(&self, rev: &str) -> String {
        self.repo.git(&["rev-parse", rev]).trim().to_string()
    }

    fn has_branch(&self, name: &str) -> bool {
        self.repo
            .try_git(&["rev-parse", "--verify", "-q", &format!("refs/heads/{name}")])
            .is_some()
    }

    fn has_tag(&self, name: &str) -> bool {
        self.repo
            .try_git(&["rev-parse", "--verify", "-q", &format!("refs/tags/{name}")])
            .is_some()
    }

    fn parents(&self, rev: &str) -> usize {
        self.repo
            .git(&["rev-list", "--parents", "-n1", rev])
            .split_whitespace()
            .count()
            - 1
    }

    fn recorded_parent(&self, branch: &str) -> Option<String> {
        self.repo
            .try_git(&["config", &format!("branch.{branch}.gitpoulpi-parent")])
            .map(|s| s.trim().to_string())
    }

    fn file(&self, name: &str) -> String {
        std::fs::read_to_string(self.repo.root().join(name)).unwrap_or_default()
    }

    async fn stop(mut self) {
        self.handle.take().unwrap().shutdown().await.unwrap();
    }
}

/// Whether `other` is an ancestor of `branch`.
fn contains(rig: &Rig, branch: &str, other: &str) -> bool {
    rig.repo
        .try_git(&["merge-base", "--is-ancestor", other, branch])
        .is_some()
}

fn git_flow() -> Workflow {
    WorkflowTemplate::GitFlow.workflow()
}

/// main <- develop, HEAD on main.
fn base_repo() -> gitpoulpi_testkit::FixtureRepoBuilder {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "base\n", "init")
        .unwrap()
        .branch("develop")
        .unwrap()
        .checkout("main")
        .unwrap()
}

fn step_ops(plan: &FlowPlan) -> Vec<String> {
    plan.steps
        .iter()
        .map(|s| {
            serde_json::to_value(s).unwrap()["op"]
                .as_str()
                .unwrap()
                .to_string()
        })
        .collect()
}

#[tokio::test]
async fn start_creates_the_branch_records_its_parent_and_is_undoable() {
    for (name, backend) in backends() {
        let repo = base_repo().build();
        let mut rig = Rig::new(backend, repo, &git_flow()).await;
        let develop = rig.tip("develop");

        let request =
            json!({ "action": "start", "topic": "feature", "fields": { "slug": "Login form" } });
        let plan = rig.plan(request.clone()).await;
        assert!(plan.can_run(), "{name}: {:?}", plan.blockers);
        assert_eq!(plan.branch.as_deref(), Some("feature/login-form"));
        assert_eq!(
            step_ops(&plan),
            ["create_branch", "checkout", "record_parent"],
            "{name}"
        );
        assert!(plan.undoable);
        // Planning changes nothing.
        assert_eq!(rig.head(), "main");
        assert!(!rig.has_branch("feature/login-form"));

        let out = rig.run(request).await;
        assert_eq!(out.status, FlowStatus::Done, "{name}: {out:?}");
        assert_eq!(rig.head(), "feature/login-form", "{name}");
        assert_eq!(rig.tip("feature/login-form"), develop, "{name}");
        assert_eq!(
            rig.recorded_parent("feature/login-form").as_deref(),
            Some("develop"),
            "{name}"
        );

        // One entry takes it all back: the branch, its parent, the checkout.
        let undone = rig.undo().await;
        assert_eq!(undone["applied"], true, "{name}: {undone}");
        assert_eq!(rig.head(), "main", "{name}");
        assert!(!rig.has_branch("feature/login-form"), "{name}");
        assert_eq!(rig.recorded_parent("feature/login-form"), None, "{name}");

        let redone = rig.redo().await;
        assert_eq!(redone["applied"], true, "{name}: {redone}");
        assert_eq!(rig.head(), "feature/login-form", "{name}");
        assert_eq!(
            rig.recorded_parent("feature/login-form").as_deref(),
            Some("develop"),
            "{name}"
        );
        rig.stop().await;
    }
}

#[tokio::test]
async fn start_refuses_bad_requests_in_the_plan() {
    let backend = backends().remove(0).1;
    let repo = base_repo().build();
    let mut rig = Rig::new(backend, repo, &git_flow()).await;
    for (request, needle) in [
        (json!({ "action": "start" }), "branch type"),
        (
            json!({ "action": "start", "topic": "nope" }),
            "unknown branch type",
        ),
        (
            json!({ "action": "start", "topic": "release" }),
            "cannot name the branch",
        ),
        (
            json!({ "action": "start", "topic": "feature", "fields": { "slug": "x" }, "from": "ghost" }),
            "not a branch or a tag",
        ),
        (
            json!({ "action": "start", "topic": "feature", "fields": { "slug": "x" }, "publish": true }),
            "no remote",
        ),
    ] {
        let plan = rig.plan(request.clone()).await;
        assert!(!plan.can_run(), "{request}");
        assert!(
            plan.blockers.iter().any(|b| b.contains(needle)),
            "{request}: {:?}",
            plan.blockers
        );
        let error = rig.try_run(request).await.unwrap_err();
        assert!(error.message.contains(needle), "{}", error.message);
    }
    // An existing name.
    rig.run(json!({ "action": "start", "topic": "feature", "fields": { "slug": "x" } }))
        .await;
    rig.call(
        OperationKind::Checkout,
        json!({ "target": { "Branch": "main" } }),
    )
    .await
    .unwrap();
    let plan = rig
        .plan(json!({ "action": "start", "topic": "feature", "fields": { "slug": "x" } }))
        .await;
    assert!(plan.blockers.iter().any(|b| b.contains("already exists")));
    rig.stop().await;
}

#[tokio::test]
async fn start_with_publish_pushes_and_says_undo_will_not() {
    for (name, backend) in backends() {
        let repo = base_repo()
            .with_origin()
            .unwrap()
            .push("main")
            .unwrap()
            .build();
        let mut rig = Rig::new(backend, repo, &git_flow()).await;
        let request = json!({ "action": "start", "topic": "feature", "fields": { "slug": "pub" }, "publish": true });
        let plan = rig.plan(request.clone()).await;
        assert_eq!(
            step_ops(&plan),
            ["create_branch", "checkout", "record_parent", "push"],
            "{name}"
        );
        assert!(!plan.undoable);
        assert!(plan.irreversible.iter().any(|i| i.contains("Pushing")));
        let out = rig.run(request).await;
        assert_eq!(out.status, FlowStatus::Done, "{name}: {out:?}");
        assert!(rig.repo.git_in_remote(&["branch"]).contains("feature/pub"));
        assert_eq!(
            rig.repo
                .git(&["config", "branch.feature/pub.remote"])
                .trim(),
            "origin"
        );
        rig.stop().await;
    }
}

/// A feature with two commits, develop moved on meanwhile.
fn feature_repo() -> FixtureRepo {
    base_repo()
        .checkout("develop")
        .unwrap()
        .branch("feature/x")
        .unwrap()
        .commit_file("f1.txt", "1\n", "f1")
        .unwrap()
        .commit_file("f2.txt", "2\n", "f2")
        .unwrap()
        .checkout("develop")
        .unwrap()
        .commit_file("d.txt", "d\n", "d1")
        .unwrap()
        .checkout("feature/x")
        .unwrap()
        .build()
}

#[tokio::test]
async fn finishing_a_feature_merges_deletes_and_is_one_undoable_entry() {
    for (name, backend) in backends() {
        let mut rig = Rig::new(backend, feature_repo(), &git_flow()).await;
        let feature_tip = rig.tip("feature/x");
        let develop_before = rig.tip("develop");
        let request = json!({ "action": "finish" });

        let plan = rig.plan(request.clone()).await;
        assert!(plan.can_run(), "{name}: {:?}", plan.blockers);
        assert_eq!(
            step_ops(&plan),
            ["checkout", "merge", "delete_branch"],
            "{name}"
        );
        assert_eq!(plan.lines().len(), 3);

        let out = rig.run(request).await;
        assert_eq!(out.status, FlowStatus::Done, "{name}: {out:?}");
        assert_eq!(rig.head(), "develop", "{name}");
        assert!(!rig.has_branch("feature/x"), "{name}");
        assert_eq!(rig.parents("develop"), 2, "{name}: a merge commit");
        assert!(rig.file("f2.txt") == "2\n" && rig.file("d.txt") == "d\n");
        let develop_after = rig.tip("develop");

        let undone = rig.undo().await;
        assert_eq!(undone["applied"], true, "{name}: {undone}");
        assert_eq!(rig.head(), "feature/x", "{name}");
        assert_eq!(rig.tip("feature/x"), feature_tip, "{name}");
        assert_eq!(rig.tip("develop"), develop_before, "{name}");
        assert!(
            rig.file("d.txt").is_empty(),
            "{name}: files follow the checkout"
        );
        assert_eq!(rig.file("f2.txt"), "2\n");

        let redone = rig.redo().await;
        assert_eq!(redone["applied"], true, "{name}: {redone}");
        assert_eq!(rig.head(), "develop", "{name}");
        assert_eq!(rig.tip("develop"), develop_after, "{name}");
        assert!(!rig.has_branch("feature/x"), "{name}");
        rig.stop().await;
    }
}

#[tokio::test]
async fn finishing_a_release_and_a_hotfix_merges_into_both_bases_and_tags() {
    for (name, backend) in backends() {
        for (branch, version) in [("release/1.0.0", "1.0.0"), ("hotfix/1.0.1", "1.0.1")] {
            let from = if branch.starts_with("release") {
                "develop"
            } else {
                "main"
            };
            let repo = base_repo()
                .checkout(from)
                .unwrap()
                .branch(branch)
                .unwrap()
                .commit_file("notes.txt", "notes\n", "prepare")
                .unwrap()
                .build();
            let mut rig = Rig::new(backend.clone(), repo, &git_flow()).await;
            let main_before = rig.tip("main");
            let develop_before = rig.tip("develop");
            let topic_tip = rig.tip(branch);

            let plan = rig.plan(json!({ "action": "finish" })).await;
            assert!(plan.can_run(), "{name} {branch}: {:?}", plan.blockers);
            let expected: &[&str] = if from == "develop" {
                &[
                    "checkout",
                    "merge",
                    "tag",
                    "checkout",
                    "merge",
                    "delete_branch",
                ]
            } else {
                &[
                    "checkout",
                    "merge",
                    "tag",
                    "checkout",
                    "merge",
                    "checkout",
                    "delete_branch",
                ]
            };
            assert_eq!(step_ops(&plan), expected, "{name} {branch}");
            let out = rig.run(json!({ "action": "finish" })).await;
            assert_eq!(out.status, FlowStatus::Done, "{name} {branch}: {out:?}");
            let tag = format!("v{version}");
            assert!(rig.has_tag(&tag), "{name} {branch}: tag");
            // The tag sits on main's merge commit.
            assert_eq!(rig.tip(&format!("{tag}^{{commit}}")), rig.tip("main"));
            assert!(!rig.has_branch(branch), "{name} {branch}");
            assert_eq!(rig.parents("main"), 2, "{name} {branch}");
            assert_eq!(rig.parents("develop"), 2, "{name} {branch}");
            assert_eq!(rig.file("notes.txt"), "notes\n");
            // Back on the parent: develop for a release, main for a hotfix.
            assert_eq!(
                rig.head(),
                if from == "develop" { "develop" } else { "main" }
            );

            // ONE undo restores both bases, the tag, the branch and the checkout.
            let undone = rig.undo().await;
            assert_eq!(undone["applied"], true, "{name} {branch}: {undone}");
            assert_eq!(rig.head(), branch, "{name}");
            assert_eq!(rig.tip(branch), topic_tip);
            assert_eq!(rig.tip("main"), main_before, "{name} {branch}");
            assert_eq!(rig.tip("develop"), develop_before, "{name} {branch}");
            assert!(!rig.has_tag(&tag), "{name} {branch}: tag undone");
            let redone = rig.redo().await;
            assert_eq!(redone["applied"], true, "{name} {branch}: {redone}");
            assert!(rig.has_tag(&tag));
            assert!(!rig.has_branch(branch));
            rig.stop().await;
        }
    }
}

#[tokio::test]
async fn a_squash_finish_lands_one_commit() {
    for (name, backend) in backends() {
        let mut workflow = git_flow();
        workflow.topics[0].finish[0].merge = Some(MergeStrategy::Squash);
        let mut rig = Rig::new(backend, feature_repo(), &workflow).await;
        let out = rig
            .run(json!({ "action": "finish", "title": "Add things" }))
            .await;
        assert_eq!(out.status, FlowStatus::Done, "{name}: {out:?}");
        assert_eq!(rig.parents("develop"), 1, "{name}");
        assert_eq!(
            rig.repo
                .git(&["log", "-1", "--format=%s", "develop"])
                .trim(),
            "Add things"
        );
        assert_eq!(rig.file("f2.txt"), "2\n");
        rig.stop().await;
    }
}

#[tokio::test]
async fn a_trunk_finish_rebases_and_fast_forwards() {
    for (name, backend) in backends() {
        let repo = FixtureRepoBuilder::new()
            .unwrap()
            .commit_file("a.txt", "base\n", "init")
            .unwrap()
            .branch("ann/fix")
            .unwrap()
            .commit_file("f.txt", "f\n", "fix")
            .unwrap()
            .checkout("main")
            .unwrap()
            .commit_file("m.txt", "m\n", "main moves")
            .unwrap()
            .checkout("ann/fix")
            .unwrap()
            .build();
        let mut rig = Rig::new(backend, repo, &WorkflowTemplate::Trunk.workflow()).await;
        let plan = rig.plan(json!({ "action": "finish" })).await;
        assert!(plan.can_run(), "{name}: {:?}", plan.blockers);
        assert_eq!(
            step_ops(&plan),
            ["rebase", "checkout", "merge", "delete_branch"],
            "{name}"
        );
        let out = rig.run(json!({ "action": "finish" })).await;
        assert_eq!(out.status, FlowStatus::Done, "{name}: {out:?}");
        assert_eq!(rig.head(), "main");
        assert_eq!(rig.parents("main"), 1, "{name}: linear history");
        assert_eq!(
            rig.repo.git(&["log", "--format=%s", "-3", "main"]).trim(),
            "fix\nmain moves\ninit",
            "{name}"
        );
        assert!(!rig.has_branch("ann/fix"));
        let undone = rig.undo().await;
        assert_eq!(undone["applied"], true, "{name}: {undone}");
        assert_eq!(rig.head(), "ann/fix");
        assert!(rig.has_branch("ann/fix"));
        rig.stop().await;
    }
}

#[tokio::test]
async fn a_github_flow_finish_pushes_and_hands_the_merge_request_to_the_client() {
    for (name, backend) in backends() {
        let repo = FixtureRepoBuilder::new()
            .unwrap()
            .commit_file("a.txt", "base\n", "init")
            .unwrap()
            .with_origin()
            .unwrap()
            .push("main")
            .unwrap()
            .branch("feature/mr")
            .unwrap()
            .commit_file("m.txt", "m\n", "work")
            .unwrap()
            .build();
        let mut rig = Rig::new(backend, repo, &WorkflowTemplate::GithubFlow.workflow()).await;
        let plan = rig.plan(json!({ "action": "finish" })).await;
        assert!(plan.can_run(), "{name}: {:?}", plan.blockers);
        assert_eq!(step_ops(&plan), ["push", "open_merge_request"], "{name}");
        assert!(!plan.undoable);
        assert!(plan
            .irreversible
            .iter()
            .any(|i| i.contains("merge request") && i.contains("Undo cannot")));
        let spec = plan.merge_request.clone().unwrap();
        assert_eq!(
            (
                spec.source.as_str(),
                spec.target.as_str(),
                spec.remote.as_str()
            ),
            ("feature/mr", "main", "origin")
        );
        assert_eq!(spec.title, "Mr");

        let out = rig
            .run(json!({ "action": "finish", "title": "Ship it" }))
            .await;
        assert_eq!(out.status, FlowStatus::Done, "{name}: {out:?}");
        let mr = out.merge_request.expect("the client opens it");
        assert_eq!(mr.title, "Ship it");
        assert_eq!(out.steps[1].status, StepStatus::Pending);
        assert!(rig.repo.git_in_remote(&["branch"]).contains("feature/mr"));
        // Nothing is merged or deleted locally: the request needs the branch.
        assert!(rig.has_branch("feature/mr"));
        assert_eq!(rig.head(), "feature/mr");
        rig.stop().await;
    }
}

#[tokio::test]
async fn finish_mode_ask_needs_a_choice() {
    let backend = backends().remove(0).1;
    let repo = base_repo()
        .checkout("main")
        .unwrap()
        .branch("support/1.x")
        .unwrap()
        .build();
    let mut rig = Rig::new(backend, repo, &git_flow()).await;
    let plan = rig.plan(json!({ "action": "finish" })).await;
    assert!(
        plan.blockers.iter().any(|b| b.contains("--mode")),
        "{:?}",
        plan.blockers
    );
    rig.stop().await;
}

#[tokio::test]
async fn promote_merges_the_upstream_environment_branch_one_step_at_a_time() {
    for (name, backend) in backends() {
        let repo = FixtureRepoBuilder::new()
            .unwrap()
            .commit_file("a.txt", "base\n", "init")
            .unwrap()
            .branch("pre-production")
            .unwrap()
            .branch("production")
            .unwrap()
            .checkout("main")
            .unwrap()
            .commit_file("a.txt", "v2\n", "ship v2")
            .unwrap()
            .build();
        let mut rig = Rig::new(backend, repo, &WorkflowTemplate::GitlabFlow.workflow()).await;
        let main_tip = rig.tip("main");
        let pre_before = rig.tip("pre-production");

        let plan = rig
            .plan(json!({ "action": "promote", "branch": "pre-production" }))
            .await;
        assert!(plan.can_run(), "{name}: {:?}", plan.blockers);
        assert_eq!(step_ops(&plan), ["checkout", "merge", "checkout"], "{name}");
        let out = rig
            .run(json!({ "action": "promote", "branch": "pre-production" }))
            .await;
        assert_eq!(out.status, FlowStatus::Done, "{name}: {out:?}");
        assert_eq!(rig.tip("pre-production"), main_tip, "{name}");
        assert_eq!(rig.head(), "main", "{name}: back where it started");
        assert_eq!(rig.tip("production"), pre_before, "{name}: one step only");

        let out = rig
            .run(json!({ "action": "promote", "branch": "production" }))
            .await;
        assert_eq!(out.status, FlowStatus::Done, "{name}: {out:?}");
        assert_eq!(rig.tip("production"), main_tip, "{name}");

        // Nothing left to promote.
        let plan = rig
            .plan(json!({ "action": "promote", "branch": "production" }))
            .await;
        assert!(plan.steps.is_empty(), "{name}");
        let out = rig
            .run(json!({ "action": "promote", "branch": "production" }))
            .await;
        assert_eq!(out.status, FlowStatus::NothingToDo);

        let undone = rig.undo().await;
        assert_eq!(undone["applied"], true, "{name}: {undone}");
        assert_eq!(rig.tip("production"), pre_before, "{name}");
        assert_eq!(rig.tip("pre-production"), main_tip, "{name}");
        assert_eq!(rig.head(), "main");
        rig.stop().await;
    }
}

#[tokio::test]
async fn sync_updates_a_branch_and_its_stack_in_order() {
    for (name, backend) in backends() {
        for method in ["merge", "rebase"] {
            let repo = base_repo()
                .checkout("develop")
                .unwrap()
                .branch("feature/a")
                .unwrap()
                .commit_file("a1.txt", "1\n", "a1")
                .unwrap()
                .branch("feature/b")
                .unwrap()
                .commit_file("b1.txt", "1\n", "b1")
                .unwrap()
                .checkout("develop")
                .unwrap()
                .commit_file("d.txt", "d\n", "d moves")
                .unwrap()
                .checkout("feature/a")
                .unwrap()
                .build();
            let mut rig = Rig::new(backend.clone(), repo, &git_flow()).await;
            // feature/b was started from feature/a: record it.
            let out = rig
                .run(
                    json!({ "action": "set_parent", "branch": "feature/b", "parent": "feature/a" }),
                )
                .await;
            assert_eq!(out.status, FlowStatus::Done, "{name}: {out:?}");
            assert_eq!(
                rig.recorded_parent("feature/b").as_deref(),
                Some("feature/a")
            );
            let b_before = rig.tip("feature/b");

            let request = json!({ "action": "sync", "stack": true, "method": method });
            let plan = rig.plan(request.clone()).await;
            assert!(plan.can_run(), "{name} {method}: {:?}", plan.blockers);
            let out = rig.run(request).await;
            assert_eq!(out.status, FlowStatus::Done, "{name} {method}: {out:?}");
            assert_eq!(rig.head(), "feature/a", "{name} {method}");
            assert!(contains(&rig, "feature/a", "develop"), "{name} {method}");
            assert!(
                contains(&rig, "feature/b", "feature/a"),
                "{name} {method}: stack follows"
            );
            assert!(contains(&rig, "feature/b", "develop"), "{name} {method}");
            assert_eq!(rig.file("d.txt"), "d\n");

            // Undo puts every branch back, files included.
            let undone = rig.undo().await;
            assert_eq!(undone["applied"], true, "{name} {method}: {undone}");
            assert_eq!(rig.tip("feature/b"), b_before, "{name} {method}");
            assert_eq!(rig.head(), "feature/a");
            assert!(rig.file("d.txt").is_empty(), "{name} {method}");
            assert!(!contains(&rig, "feature/a", "develop"));
            rig.stop().await;
        }
    }
}

#[tokio::test]
async fn set_parent_records_clears_and_refuses_a_cycle() {
    let backend = backends().remove(1).1;
    let repo = base_repo()
        .checkout("develop")
        .unwrap()
        .branch("feature/a")
        .unwrap()
        .branch("feature/b")
        .unwrap()
        .build();
    let mut rig = Rig::new(backend, repo, &git_flow()).await;
    let out = rig
        .run(json!({ "action": "set_parent", "branch": "feature/b", "parent": "feature/a" }))
        .await;
    assert_eq!(out.status, FlowStatus::Done);
    assert_eq!(
        rig.recorded_parent("feature/b").as_deref(),
        Some("feature/a")
    );
    // The other way round would be a cycle; a branch cannot be its own parent.
    for (branch, parent, needle) in [
        ("feature/a", "feature/b", "cycle"),
        ("feature/a", "feature/a", "own parent"),
        ("feature/a", "ghost", "no local branch"),
    ] {
        let plan = rig
            .plan(json!({ "action": "set_parent", "branch": branch, "parent": parent }))
            .await;
        assert!(
            plan.blockers.iter().any(|b| b.contains(needle)),
            "{branch}->{parent}: {:?}",
            plan.blockers
        );
    }
    let undone = rig.undo().await;
    assert_eq!(undone["applied"], true);
    assert_eq!(rig.recorded_parent("feature/b"), None);
    rig.redo().await;
    let out = rig
        .run(json!({ "action": "set_parent", "branch": "feature/b", "clear": true }))
        .await;
    assert_eq!(out.status, FlowStatus::Done);
    assert_eq!(rig.recorded_parent("feature/b"), None);
    rig.stop().await;
}

fn conflict_repo() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\ntwo\nthree\n", "init")
        .unwrap()
        .branch("develop")
        .unwrap()
        .branch("feature/c")
        .unwrap()
        .commit_file("a.txt", "one\nFEATURE\nthree\n", "feature edit")
        .unwrap()
        .checkout("develop")
        .unwrap()
        .commit_file("a.txt", "one\nDEVELOP\nthree\n", "develop edit")
        .unwrap()
        .checkout("feature/c")
        .unwrap()
        .build()
}

#[tokio::test]
async fn a_conflict_mid_finish_stops_cleanly_and_continues_after_the_resolution() {
    for (name, backend) in backends() {
        let mut rig = Rig::new(backend, conflict_repo(), &git_flow()).await;
        let out = rig.run(json!({ "action": "finish" })).await;
        assert_eq!(out.status, FlowStatus::Conflicted, "{name}: {out:?}");
        assert_eq!(out.conflicts, ["a.txt"], "{name}");
        assert!(out.resumable);
        assert_eq!(out.steps[1].status, StepStatus::Conflicted);
        assert_eq!(out.steps[2].status, StepStatus::NotRun);
        assert_eq!(rig.head(), "develop", "{name}");
        assert!(rig.has_branch("feature/c"), "{name}: nothing deleted yet");

        // The flow is waiting: a new one is refused, the plan says what is left.
        let refused = rig
            .try_run(json!({ "action": "finish" }))
            .await
            .unwrap_err();
        assert!(
            refused.message.contains("--continue"),
            "{name}: {}",
            refused.message
        );
        let rest = rig
            .plan(json!({ "action": "finish", "resume": "continue" }))
            .await;
        assert!(rest.resumed);
        assert_eq!(step_ops(&rest), ["merge", "delete_branch"], "{name}");
        // Continuing before resolving is an error that says what is left.
        let early = rig
            .try_run(json!({ "action": "finish", "resume": "continue" }))
            .await
            .unwrap_err();
        assert!(early.message.contains("a.txt"), "{name}: {}", early.message);

        std::fs::write(rig.repo.root().join("a.txt"), "one\nBOTH\nthree\n").unwrap();
        rig.repo.git(&["add", "a.txt"]);
        let out = rig
            .run(json!({ "action": "finish", "resume": "continue" }))
            .await;
        assert_eq!(out.status, FlowStatus::Done, "{name}: {out:?}");
        assert_eq!(rig.head(), "develop", "{name}");
        assert!(!rig.has_branch("feature/c"), "{name}");
        assert_eq!(rig.parents("develop"), 2, "{name}");
        assert_eq!(rig.file("a.txt"), "one\nBOTH\nthree\n");
        // The state is gone: nothing left to continue.
        let error = rig
            .try_run(json!({ "action": "finish", "resume": "continue" }))
            .await
            .unwrap_err();
        assert!(error.message.contains("no flow"), "{}", error.message);
        rig.stop().await;
    }
}

#[tokio::test]
async fn a_conflict_mid_finish_can_be_aborted() {
    for (name, backend) in backends() {
        let mut rig = Rig::new(backend, conflict_repo(), &git_flow()).await;
        let develop = rig.tip("develop");
        let out = rig.run(json!({ "action": "finish" })).await;
        assert_eq!(out.status, FlowStatus::Conflicted, "{name}: {out:?}");
        let out = rig
            .run(json!({ "action": "finish", "resume": "abort" }))
            .await;
        assert_eq!(out.status, FlowStatus::Aborted, "{name}: {out:?}");
        assert_eq!(
            rig.head(),
            "feature/c",
            "{name}: back on the starting branch"
        );
        assert_eq!(rig.tip("develop"), develop, "{name}");
        assert!(rig.has_branch("feature/c"));
        assert!(
            rig.repo
                .try_git(&["rev-parse", "--verify", "-q", "MERGE_HEAD"])
                .is_none(),
            "{name}: no merge left"
        );
        assert_eq!(rig.file("a.txt"), "one\nFEATURE\nthree\n");
        // A new finish can start (the saved flow is gone).
        let plan = rig.plan(json!({ "action": "finish" })).await;
        assert!(plan.can_run(), "{name}: {:?}", plan.blockers);
        rig.stop().await;
    }
}

#[tokio::test]
async fn a_squash_conflict_continues_with_the_squash_message() {
    for (name, backend) in backends() {
        let mut workflow = git_flow();
        workflow.topics[0].finish[0].merge = Some(MergeStrategy::Squash);
        let mut rig = Rig::new(backend, conflict_repo(), &workflow).await;
        let out = rig
            .run(json!({ "action": "finish", "title": "Squashed feature" }))
            .await;
        assert_eq!(out.status, FlowStatus::Conflicted, "{name}: {out:?}");
        std::fs::write(rig.repo.root().join("a.txt"), "one\nBOTH\nthree\n").unwrap();
        rig.repo.git(&["add", "a.txt"]);
        let out = rig
            .run(json!({ "action": "finish", "resume": "continue" }))
            .await;
        assert_eq!(out.status, FlowStatus::Done, "{name}: {out:?}");
        assert_eq!(rig.parents("develop"), 1, "{name}");
        assert_eq!(
            rig.repo
                .git(&["log", "-1", "--format=%s", "develop"])
                .trim(),
            "Squashed feature"
        );
        assert!(!rig.has_branch("feature/c"));
        rig.stop().await;
    }
}

fn hooked(after_start: &[&str], before_finish: &[&str]) -> Workflow {
    let mut w = git_flow();
    w.hooks = Hooks {
        after_start: after_start.iter().map(|s| s.to_string()).collect(),
        before_finish: before_finish.iter().map(|s| s.to_string()).collect(),
    };
    w
}

#[tokio::test]
async fn hooks_never_run_unless_asked_and_show_their_output() {
    let hook = if cfg!(windows) {
        "echo hello> hook-ran.txt && echo from-the-hook"
    } else {
        "echo hello > hook-ran.txt; echo from-the-hook"
    };
    for (name, backend) in backends() {
        let repo = base_repo().build();
        let mut rig = Rig::new(backend, repo, &hooked(&[hook], &[])).await;
        rig.trust_hooks().await;
        let request = json!({ "action": "start", "topic": "feature", "fields": { "slug": "h" } });

        // Not asked: listed, not run.
        let plan = rig.plan(request.clone()).await;
        assert_eq!(plan.hooks.len(), 1);
        assert!(!plan.hooks_run);
        assert!(!step_ops(&plan).contains(&"hook".to_string()));
        assert!(plan.warnings.iter().any(|w| w.contains("hooks")));

        // Asked: a step, run in the repository, output reported, irreversible.
        let mut asked = request.clone();
        asked["run_hooks"] = json!(true);
        let plan = rig.plan(asked.clone()).await;
        assert!(plan.hooks_run);
        assert_eq!(step_ops(&plan).last().unwrap(), "hook");
        assert!(plan.irreversible.iter().any(|i| i.contains("hook")));
        let out = rig.run(asked).await;
        assert_eq!(out.status, FlowStatus::Done, "{name}: {out:?}");
        let hook_step = out.steps.last().unwrap();
        assert_eq!(hook_step.exit_code, Some(0));
        assert!(hook_step
            .output
            .as_deref()
            .unwrap()
            .contains("from-the-hook"));
        assert!(
            rig.repo.root().join("hook-ran.txt").exists(),
            "{name}: cwd is the repo"
        );
        rig.stop().await;
    }
}

#[tokio::test]
async fn a_failing_hook_stops_the_flow_and_shows_its_output() {
    let failing = if cfg!(windows) {
        "echo boom 1>&2 && exit 3"
    } else {
        "echo boom >&2; exit 3"
    };
    for (name, backend) in backends() {
        // After a start: the branch exists, the flow stops at the hook and
        // `continue` retries it; Undo takes the branch back.
        let repo = base_repo().build();
        let mut rig = Rig::new(backend.clone(), repo, &hooked(&[failing], &[])).await;
        rig.trust_hooks().await;
        let out = rig
            .run(json!({ "action": "start", "topic": "feature", "fields": { "slug": "h" }, "run_hooks": true }))
            .await;
        assert_eq!(out.status, FlowStatus::Failed, "{name}: {out:?}");
        assert!(out.resumable);
        let last = out.steps.last().unwrap();
        assert_eq!(last.status, StepStatus::Failed);
        assert_eq!(last.exit_code, Some(3));
        assert!(
            last.output.as_deref().unwrap().contains("boom"),
            "{name}: {last:?}"
        );
        assert_eq!(rig.head(), "feature/h", "{name}: what ran stays");
        // Retrying the step fails the same way: a plain error, the saved flow stays.
        let retry = rig
            .try_run(json!({ "action": "start", "resume": "continue" }))
            .await
            .unwrap_err();
        assert!(
            retry.message.contains("exited with 3"),
            "{name}: {}",
            retry.message
        );
        let aborted = rig
            .run(json!({ "action": "start", "resume": "abort" }))
            .await;
        assert_eq!(aborted.status, FlowStatus::Aborted);
        rig.stop().await;

        // Before a finish, nothing has changed yet: a plain error, no state.
        let mut rig = Rig::new(backend, feature_repo(), &hooked(&[], &[failing])).await;
        rig.trust_hooks().await;
        let error = rig
            .try_run(json!({ "action": "finish", "run_hooks": true }))
            .await
            .unwrap_err();
        assert!(
            error.message.contains("exited with 3"),
            "{name}: {}",
            error.message
        );
        assert!(error.message.contains("boom"));
        assert_eq!(rig.head(), "feature/x");
        assert!(rig.has_branch("feature/x"));
        rig.stop().await;
    }
}

fn touch(file: &str) -> String {
    if cfg!(windows) {
        format!("echo x> {file}")
    } else {
        format!("echo x > {file}")
    }
}

fn run_start(slug: &str) -> Value {
    json!({ "action": "start", "topic": "feature", "fields": { "slug": slug }, "run_hooks": true })
}

#[tokio::test]
async fn untrusted_committed_hooks_refuse_a_run_before_any_step() {
    for (name, backend) in backends() {
        let repo = base_repo().build();
        let mut rig = Rig::new(backend, repo, &hooked(&[&touch("hook-ran.txt")], &[])).await;

        // The plan says what to ask, and lists the exact commands.
        let plan = rig.plan(run_start("t")).await;
        let HooksTrust::Untrusted { hash, previous } = &plan.hooks_trust else {
            panic!("{name}: {:?}", plan.hooks_trust);
        };
        assert_eq!(hash.len(), 64);
        assert!(previous.is_none());
        assert_eq!(plan.hooks[0].command, touch("hook-ran.txt"));

        let err = rig.try_run(run_start("t")).await.unwrap_err();
        match &err.kind {
            Some(TypedError::HooksNotTrusted { hash: h, hooks }) => {
                assert_eq!(h, hash);
                assert_eq!(hooks[0].command, touch("hook-ran.txt"));
            }
            other => panic!("{name}: {other:?} {}", err.message),
        }
        // Nothing ran: no branch, no checkout, no hook, no saved flow.
        assert!(!rig.has_branch("feature/t"), "{name}");
        assert_eq!(rig.head(), "main");
        assert!(!rig.repo.root().join("hook-ran.txt").exists());

        // Not asking for hooks never needs trust.
        let out = rig
            .run(json!({ "action": "start", "topic": "feature", "fields": { "slug": "t" } }))
            .await;
        assert_eq!(out.status, FlowStatus::Done, "{name}");
        rig.stop().await;
    }
}

#[tokio::test]
async fn trusting_the_shown_hooks_lets_them_run_and_a_change_asks_again() {
    for (name, backend) in backends() {
        let repo = base_repo().build();
        let mut rig = Rig::new(backend, repo, &hooked(&[&touch("one.txt")], &[])).await;
        let report = rig.trust_hooks().await;
        assert!(report.trusted, "{name}");
        let plan = rig.plan(run_start("a")).await;
        assert_eq!(plan.hooks_trust, HooksTrust::Trusted, "{name}");
        let out = rig.run(run_start("a")).await;
        assert_eq!(out.status, FlowStatus::Done, "{name}: {out:?}");
        assert!(rig.repo.root().join("one.txt").exists());

        // The file changes after the trust: the old hash no longer matches.
        write_workflow(&rig.repo, &hooked(&[&touch("two.txt")], &[]));
        let plan = rig.plan(run_start("b")).await;
        match &plan.hooks_trust {
            HooksTrust::Untrusted { hash, previous } => {
                assert_ne!(*hash, report.hash);
                let old = previous.as_ref().expect("the old list is shown");
                assert_eq!(old[0].command, touch("one.txt"));
            }
            other => panic!("{name}: {other:?}"),
        }
        let err = rig.try_run(run_start("b")).await.unwrap_err();
        assert!(
            matches!(err.kind, Some(TypedError::HooksNotTrusted { .. })),
            "{name}: {}",
            err.message
        );
        assert!(!rig.repo.root().join("two.txt").exists());

        // Trusting the hash that was shown for the old file is refused: only
        // what is on disk now can be trusted.
        let stale = rig
            .call(
                OperationKind::FlowTrustHooks,
                json!({ "hash": report.hash }),
            )
            .await
            .unwrap_err();
        assert!(
            stale.message.contains("differs"),
            "{name}: {}",
            stale.message
        );
        let wrong = rig
            .call(
                OperationKind::FlowTrustHooks,
                json!({ "hash": "0".repeat(64) }),
            )
            .await;
        assert!(wrong.is_err());
        rig.stop().await;
    }
}

#[tokio::test]
async fn a_resume_checks_every_saved_command_and_refuses_an_injected_one() {
    let failing = if cfg!(windows) {
        "echo boom 1>&2 && exit 3"
    } else {
        "echo boom >&2; exit 3"
    };
    for (name, backend) in backends() {
        let repo = base_repo().build();
        let mut rig = Rig::new(backend, repo, &hooked(&[failing], &[])).await;
        rig.trust_hooks().await;
        let out = rig.run(run_start("r")).await;
        assert_eq!(out.status, FlowStatus::Failed, "{name}");
        assert!(out.resumable);

        // Someone edits the saved state in the repository's own directory.
        let path = rig.repo.root().join(".git/gitpoulpi/flow-state.json");
        let text = std::fs::read_to_string(&path).unwrap();
        let injected = touch("injected.txt");
        let edited = text.replace(
            &serde_json::to_string(failing).unwrap(),
            &serde_json::to_string(&injected).unwrap(),
        );
        assert_ne!(text, edited, "{name}: the saved command was found");
        std::fs::write(&path, edited).unwrap();

        let err = rig
            .try_run(json!({ "action": "start", "resume": "continue" }))
            .await
            .unwrap_err();
        match &err.kind {
            Some(TypedError::HooksNotTrusted { hooks, .. }) => {
                assert_eq!(hooks[0].command, injected, "{name}");
            }
            other => panic!("{name}: {other:?} {}", err.message),
        }
        assert!(!rig.repo.root().join("injected.txt").exists(), "{name}");
        // Aborting runs no hook and is still possible.
        let aborted = rig
            .run(json!({ "action": "start", "resume": "abort" }))
            .await;
        assert_eq!(aborted.status, FlowStatus::Aborted);
        rig.stop().await;
    }
}

#[tokio::test]
async fn personal_hooks_run_without_asking_and_replace_the_committed_ones() {
    for (name, backend) in backends() {
        let repo = base_repo().build();
        let mut rig = Rig::new(backend, repo, &hooked(&[&touch("shared.txt")], &[])).await;
        // The user's own override (repo state) carries its own hooks.
        let path = gitpoulpi_session::RepoState::path_for(
            rig.repo.root(),
            gitpoulpi_config::DEFAULT_REPO_DATA_LOCATION,
        );
        let state = gitpoulpi_session::RepoState {
            workflow: Some(gitpoulpi_core::workflow::WorkflowOverride {
                hooks: Some(Hooks {
                    after_start: vec![touch("personal.txt")],
                    before_finish: Vec::new(),
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        state.save(&path).unwrap();

        let plan = rig.plan(run_start("p")).await;
        assert_eq!(plan.hooks_trust, HooksTrust::Personal, "{name}");
        let out = rig.run(run_start("p")).await;
        assert_eq!(out.status, FlowStatus::Done, "{name}: {out:?}");
        assert!(rig.repo.root().join("personal.txt").exists(), "{name}");
        assert!(!rig.repo.root().join("shared.txt").exists(), "{name}");
        rig.stop().await;
    }
}

#[tokio::test]
async fn a_dirty_tree_follows_the_auto_stash_setting() {
    for (name, backend) in backends() {
        // feature/x edits a.txt differently from develop: the checkout of
        // develop is blocked by a local change to it.
        let repo = FixtureRepoBuilder::new()
            .unwrap()
            .commit_file("a.txt", "1\n2\n3\n4\n5\n", "init")
            .unwrap()
            .branch("develop")
            .unwrap()
            .branch("feature/x")
            .unwrap()
            .commit_file("a.txt", "1\n2\n3\n4\nfeature\n", "feature edit")
            .unwrap()
            .checkout("develop")
            .unwrap()
            .commit_file("b.txt", "b\n", "develop work")
            .unwrap()
            .checkout("feature/x")
            .unwrap()
            .build();
        let mut rig = Rig::new(backend, repo, &git_flow()).await;
        std::fs::write(rig.repo.root().join("a.txt"), "1\n2\n3\n4\nlocal\n").unwrap();
        let plan = rig.plan(json!({ "action": "finish" })).await;
        assert!(plan.dirty);
        assert!(plan.warnings.iter().any(|w| w.contains("stashed")));

        // The mode travels with the request: no process-global switch.
        let refused = rig
            .try_run(json!({ "action": "finish", "autostash": "never" }))
            .await
            .unwrap_err();
        assert!(
            refused.message.to_lowercase().contains("overwritten")
                || refused.message.to_lowercase().contains("prevents checkout")
                || refused.message.to_lowercase().contains("local changes"),
            "{name}: {}",
            refused.message
        );
        assert_eq!(rig.head(), "feature/x", "{name}: nothing happened");

        // The mode travels with the request: no process-global switch.
        let asked = rig
            .try_run(json!({ "action": "finish", "autostash": "ask" }))
            .await
            .unwrap_err();
        assert!(
            matches!(
                asked.kind,
                Some(gitpoulpi_core::TypedError::NeedsAutoStash { .. })
            ),
            "{name}: {asked:?}"
        );

        let out = rig
            .try_run(json!({ "action": "finish", "autostash": "always" }))
            .await;
        let out = out.unwrap_or_else(|e| panic!("{name}: {}", e.message));
        assert_eq!(out.status, FlowStatus::Done, "{name}: {out:?}");
        assert_eq!(rig.head(), "develop");
        assert!(!rig.has_branch("feature/x"));
        // The local edit is back on top of the merge (the file merged into develop's).
        assert_eq!(rig.file("a.txt"), "1\n2\n3\n4\nlocal\n", "{name}");
        assert!(rig.repo.git(&["stash", "list"]).trim().is_empty(), "{name}");
        rig.stop().await;
    }
}

#[tokio::test]
async fn finish_refuses_what_is_not_a_topic_branch_and_a_target_behind_its_remote() {
    let backend = backends().remove(0).1;
    let repo = base_repo()
        .checkout("develop")
        .unwrap()
        .branch("feature/r")
        .unwrap()
        .commit_file("r.txt", "r\n", "r")
        .unwrap()
        .checkout("main")
        .unwrap()
        .build();
    let mut rig = Rig::new(backend, repo, &git_flow()).await;
    let plan = rig
        .plan(json!({ "action": "finish", "branch": "main" }))
        .await;
    assert!(plan.blockers.iter().any(|b| b.contains("base branch")));
    let plan = rig
        .plan(json!({ "action": "finish", "branch": "ghost" }))
        .await;
    assert!(plan.blockers.iter().any(|b| b.contains("no local branch")));
    rig.repo.git(&["branch", "odd-name"]);
    let plan = rig
        .plan(json!({ "action": "finish", "branch": "odd-name" }))
        .await;
    assert!(plan
        .blockers
        .iter()
        .any(|b| b.contains("not a topic branch")));
    rig.stop().await;

    // develop behind its remote: refused unless allowed.
    for (name, backend) in backends() {
        let repo = base_repo()
            .with_origin()
            .unwrap()
            .push("main")
            .unwrap()
            .push("develop")
            .unwrap()
            .track("develop", "origin")
            .unwrap()
            .branch("feature/r")
            .unwrap()
            .commit_file("r.txt", "r\n", "r")
            .unwrap()
            .build();
        repo.push_from_elsewhere("develop", "someone else pushed");
        repo.git(&["fetch", "origin"]);
        let mut rig = Rig::new(backend, repo, &git_flow()).await;
        let plan = rig.plan(json!({ "action": "finish" })).await;
        assert!(
            plan.blockers
                .iter()
                .any(|b| b.contains("behind origin/develop")),
            "{name}: {:?}",
            plan.blockers
        );
        let plan = rig
            .plan(json!({ "action": "finish", "allow_behind": true }))
            .await;
        assert!(plan.can_run(), "{name}: {:?}", plan.blockers);
        rig.stop().await;
    }
}

#[tokio::test]
async fn delete_remote_removes_the_branch_on_the_remote_too() {
    for (name, backend) in backends() {
        let repo = base_repo()
            .with_origin()
            .unwrap()
            .push("main")
            .unwrap()
            .push("develop")
            .unwrap()
            .branch("feature/gone")
            .unwrap()
            .commit_file("g.txt", "g\n", "g")
            .unwrap()
            .push("feature/gone")
            .unwrap()
            .track("feature/gone", "origin")
            .unwrap()
            .build();
        let mut rig = Rig::new(backend, repo, &git_flow()).await;
        assert!(rig.repo.git_in_remote(&["branch"]).contains("feature/gone"));
        let plan = rig
            .plan(json!({ "action": "finish", "delete_remote": true }))
            .await;
        assert!(
            plan.undoable && plan.irreversible.is_empty(),
            "{name}: deleting a remote branch is taken back by Undo: {:?}",
            plan.irreversible
        );
        let tip = rig.tip("feature/gone");
        let out = rig
            .run(json!({ "action": "finish", "delete_remote": true }))
            .await;
        assert_eq!(out.status, FlowStatus::Done, "{name}: {out:?}");
        assert!(
            !rig.repo.git_in_remote(&["branch"]).contains("feature/gone"),
            "{name}"
        );
        assert!(!rig.has_branch("feature/gone"));

        // One entry: undo (confirmed, it pushes) brings the branch back
        // here and on the remote at the same commit, redo deletes both.
        let undone = rig
            .call(OperationKind::Undo, json!({ "confirmed": true }))
            .await
            .unwrap();
        assert_eq!(undone["applied"], true, "{name}: {undone}");
        assert!(rig.has_branch("feature/gone"), "{name}");
        assert_eq!(
            rig.repo
                .git_in_remote(&["rev-parse", "refs/heads/feature/gone"])
                .trim(),
            tip.trim(),
            "{name}"
        );
        rig.call(OperationKind::Redo, json!({ "confirmed": true }))
            .await
            .unwrap();
        assert!(
            !rig.repo.git_in_remote(&["branch"]).contains("feature/gone"),
            "{name}"
        );
        rig.stop().await;
    }
}

#[tokio::test]
async fn pick_up_tracks_a_branch_a_bot_made_records_its_parent_and_is_one_undoable_entry() {
    for (name, backend) in backends() {
        let repo = base_repo()
            .with_origin()
            .unwrap()
            .push("main")
            .unwrap()
            .push("develop")
            .unwrap()
            .branch("feature/12-login")
            .unwrap()
            .commit_file("l.txt", "l\n", "bot commit")
            .unwrap()
            .push("feature/12-login")
            .unwrap()
            .checkout("main")
            .unwrap()
            .build();
        // The bot's branch exists on the remote only.
        repo.git(&["branch", "-D", "feature/12-login"]);
        let mut rig = Rig::new(backend, repo, &git_flow()).await;
        let remote_tip = rig.tip("origin/feature/12-login");
        assert!(!rig.has_branch("feature/12-login"));

        let request =
            json!({ "action": "pick_up", "branch": "feature/12-login", "parent": "develop" });
        let plan = rig.plan(request.clone()).await;
        assert!(plan.can_run(), "{name}: {:?}", plan.blockers);
        assert_eq!(
            step_ops(&plan),
            ["create_branch", "set_upstream", "checkout", "record_parent"],
            "{name}"
        );
        assert!(plan.undoable, "{name}");
        assert!(
            !rig.has_branch("feature/12-login"),
            "planning changes nothing"
        );

        let out = rig.run(request).await;
        assert_eq!(out.status, FlowStatus::Done, "{name}: {out:?}");
        assert_eq!(rig.head(), "feature/12-login", "{name}");
        assert_eq!(rig.tip("feature/12-login"), remote_tip, "{name}");
        assert_eq!(
            rig.recorded_parent("feature/12-login").as_deref(),
            Some("develop"),
            "{name}"
        );
        assert_eq!(
            rig.repo
                .git(&["rev-parse", "--abbrev-ref", "feature/12-login@{upstream}"])
                .trim(),
            "origin/feature/12-login",
            "{name}"
        );

        let undone = rig.undo().await;
        assert_eq!(undone["applied"], true, "{name}: {undone}");
        assert_eq!(rig.head(), "main", "{name}");
        assert!(!rig.has_branch("feature/12-login"), "{name}");
        assert_eq!(rig.recorded_parent("feature/12-login"), None, "{name}");
        rig.stop().await;
    }
}

#[tokio::test]
async fn pick_up_of_a_branch_the_remote_does_not_have_is_refused_in_the_plan() {
    let backend = backends().remove(0).1;
    let repo = base_repo()
        .with_origin()
        .unwrap()
        .push("main")
        .unwrap()
        .build();
    let mut rig = Rig::new(backend, repo, &git_flow()).await;
    let request = json!({ "action": "pick_up", "branch": "feature/99-ghost" });
    let plan = rig.plan(request.clone()).await;
    assert!(plan.blockers.iter().any(|b| b.contains("fetch")));
    let error = rig.try_run(request).await.unwrap_err();
    assert!(error.message.contains("fetch"), "{}", error.message);
    rig.stop().await;
}

// No test reads or writes the real config, data dir or credentials of whoever runs it.
gitpoulpi_testkit::isolate_binary!();
