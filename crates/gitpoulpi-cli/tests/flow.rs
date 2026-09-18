//! `gitpoulpi flow ...`: branching workflows, read-only plus explicit file
//! creation. Fixture repositories, deserialized JSON.

use std::path::Path;

use assert_cmd::Command;
use gitpoulpi_core::workflow::{
    BranchKind, Workflow, WorkflowOverride, WorkflowReport, WorkflowTemplate,
};
use gitpoulpi_session::RepoState;
use gitpoulpi_testkit::{FixtureRepo, TestRepoBuilder};
use serde_json::Value;

fn config_dir() -> &'static Path {
    static DIR: std::sync::OnceLock<tempfile::TempDir> = std::sync::OnceLock::new();
    DIR.get_or_init(|| tempfile::tempdir().unwrap()).path()
}

fn run(root: &Path, session: &str, args: &[&str]) -> std::process::Output {
    Command::cargo_bin("gitpoulpi")
        .unwrap()
        .env("GITPOULPI_CONFIG_DIR", config_dir())
        .env("GITPOULPI_DESKTOP_NO_HOOKS", "1")
        .arg("--json")
        .args(["--session", session])
        .args(["--repo", &root.to_string_lossy()])
        .args(args)
        .current_dir(root)
        .output()
        .unwrap()
}

fn run_json(root: &Path, session: &str, args: &[&str]) -> Value {
    let output = run(root, session, args);
    assert!(
        output.status.success(),
        "{args:?} failed: {} {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

fn status(root: &Path, session: &str) -> WorkflowReport {
    serde_json::from_value(run_json(root, session, &["flow", "status"])).unwrap()
}

/// main <- develop (d1, d2) <- feature/a (f1, branched after d1) , plus a
/// `release/1.0` and an odd branch.
fn git_flow_repo() -> FixtureRepo {
    TestRepoBuilder::new()
        .with_initial()
        .branch("develop")
        .commit("d1")
        .branch("feature/a")
        .commit("f1")
        .checkout("develop")
        .commit("d2")
        .branch("release/1.0")
        .checkout("main")
        .branch("odd-one")
        .checkout("main")
        .build()
}

fn find<'a>(r: &'a WorkflowReport, name: &str) -> &'a gitpoulpi_core::workflow::BranchReport {
    r.branches
        .iter()
        .find(|b| b.name == name && b.remote.is_none())
        .unwrap_or_else(|| panic!("{name} not reported"))
}

#[test]
fn without_a_workflow_everything_is_unclassified_and_detection_proposes_git_flow() {
    let repo = git_flow_repo();
    let r = status(repo.root(), "cli-flow-none");
    assert!(!r.configured);
    assert!(r
        .branches
        .iter()
        .all(|b| b.kind == BranchKind::Unclassified));
    assert_eq!(r.detections[0].template, WorkflowTemplate::GitFlow);

    let detect = run_json(repo.root(), "cli-flow-none", &["flow", "detect"]);
    let ranked = detect["detections"].as_array().unwrap();
    assert_eq!(ranked[0]["template"], "git-flow");
    assert!(ranked[0]["score"].as_u64().unwrap() >= 80);
    let bases: Vec<(String, Option<String>)> = ranked[0]["bases"]
        .as_array()
        .unwrap()
        .iter()
        .map(|b| {
            (
                b["base"].as_str().unwrap().to_string(),
                b["found"].as_str().map(str::to_string),
            )
        })
        .collect();
    assert_eq!(
        bases,
        [
            ("main".to_string(), Some("main".to_string())),
            ("develop".to_string(), Some("develop".to_string()))
        ]
    );
    // nothing was written
    assert!(!repo.root().join(".git/gitpoulpi/workflow.toml").exists());
}

#[test]
fn init_prints_then_writes_once_and_status_classifies_with_counts() {
    let repo = git_flow_repo();
    let root = repo.root();
    let session = "cli-flow-init";

    let printed = run_json(
        root,
        session,
        &["flow", "config", "init", "--template", "git-flow"],
    );
    assert_eq!(printed["written"], false);
    let wf: Workflow = toml::from_str(printed["toml"].as_str().unwrap()).unwrap();
    assert_eq!(wf, WorkflowTemplate::GitFlow.workflow());
    assert!(!root.join(".git/gitpoulpi/workflow.toml").exists());

    let wrote = run_json(
        root,
        session,
        &[
            "flow",
            "config",
            "init",
            "--template",
            "git-flow",
            "--write",
        ],
    );
    assert_eq!(wrote["written"], true);
    let file = root.join(".git/gitpoulpi/workflow.toml");
    let first = std::fs::read_to_string(&file).unwrap();
    assert_eq!(first, printed["toml"].as_str().unwrap());

    // a second --write refuses and leaves the file alone
    std::fs::write(&file, format!("{first}\n# mine\n")).unwrap();
    let again = run(
        root,
        session,
        &["flow", "config", "init", "--template", "trunk", "--write"],
    );
    assert!(!again.status.success());
    assert!(std::fs::read_to_string(&file)
        .unwrap()
        .ends_with("# mine\n"));

    let r = status(root, session);
    assert!(r.configured && r.sources.shared_file);
    assert_eq!(r.template, WorkflowTemplate::GitFlow);
    assert_eq!(find(&r, "main").kind, BranchKind::Base);
    let develop = find(&r, "develop");
    assert_eq!(develop.parent.as_ref().unwrap().name, "main");
    assert_eq!((develop.ahead, develop.behind), (Some(2), Some(0)));
    let feature = find(&r, "feature/a");
    assert_eq!(feature.type_id.as_deref(), Some("feature"));
    assert_eq!(feature.fields["slug"], "a");
    assert_eq!(feature.parent.as_ref().unwrap().name, "develop");
    assert_eq!((feature.ahead, feature.behind), (Some(1), Some(1)));
    assert_eq!(find(&r, "release/1.0").type_id.as_deref(), Some("release"));
    assert_eq!(find(&r, "odd-one").kind, BranchKind::Unclassified);
    // the repository stays clean: the workflow file lives in the git dir
    let porcelain = repo.git(&["status", "--porcelain", "--untracked-files=all"]);
    assert_eq!(porcelain.trim(), "");
}

#[test]
fn a_recorded_parent_wins_over_the_type() {
    let repo = git_flow_repo();
    let root = repo.root();
    repo.git(&["config", "branch.feature/a.gitpoulpi-parent", "release/1.0"]);
    run_json(
        root,
        "cli-flow-parent",
        &[
            "flow",
            "config",
            "init",
            "--template",
            "git-flow",
            "--write",
        ],
    );
    let r = status(root, "cli-flow-parent");
    let p = find(&r, "feature/a").parent.clone().unwrap();
    assert_eq!(p.name, "release/1.0");
    assert_eq!(serde_json::to_value(p.source).unwrap(), "recorded");
}

#[test]
fn the_user_override_layers_over_the_shared_file() {
    let repo = git_flow_repo();
    let root = repo.root();
    let session = "cli-flow-override";
    run_json(
        root,
        session,
        &[
            "flow",
            "config",
            "init",
            "--template",
            "git-flow",
            "--write",
        ],
    );
    let state = RepoState {
        workflow: Some(WorkflowOverride {
            disable_topics: vec!["feature".into()],
            ..Default::default()
        }),
        ..Default::default()
    };
    state
        .save(&RepoState::path_for(
            root,
            gitpoulpi_config::DEFAULT_REPO_DATA_LOCATION,
        ))
        .unwrap();

    let r = status(root, session);
    assert!(r.sources.user_override);
    assert_eq!(find(&r, "feature/a").kind, BranchKind::Unclassified);
    assert_eq!(find(&r, "release/1.0").kind, BranchKind::Topic);

    let shared = run_json(root, session, &["flow", "config"]);
    assert_eq!(shared["exists"], true);
    let shared_wf: Workflow = serde_json::from_value(shared["workflow"].clone()).unwrap();
    assert!(shared_wf.topics.iter().any(|t| t.id == "feature"));

    let effective = run_json(root, session, &["flow", "config", "--show-effective"]);
    let eff: Workflow = serde_json::from_value(effective["workflow"].clone()).unwrap();
    assert!(eff.topics.iter().all(|t| t.id != "feature"));
    let same = run_json(
        root,
        session,
        &["flow", "config", "show", "--show-effective"],
    );
    assert_eq!(same["workflow"], effective["workflow"]);
}

#[test]
fn config_without_a_file_says_so() {
    let repo = git_flow_repo();
    let v = run_json(repo.root(), "cli-flow-nofile", &["flow", "config"]);
    assert_eq!(v["exists"], false);
    assert!(v["path"].as_str().unwrap().ends_with("workflow.toml"));
}

#[test]
fn an_invalid_workflow_file_is_reported_not_ignored() {
    let repo = git_flow_repo();
    let root = repo.root();
    std::fs::create_dir_all(root.join(".git").join("gitpoulpi")).unwrap();
    std::fs::write(root.join(".git/gitpoulpi/workflow.toml"), "template = [").unwrap();
    let out = run(root, "cli-flow-bad", &["flow", "status"]);
    assert!(!out.status.success());
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(text.contains("workflow.toml"), "{text}");
}

#[test]
fn templates_lists_the_five_presets_without_a_repository() {
    let dir = tempfile::tempdir().unwrap();
    let out = Command::cargo_bin("gitpoulpi")
        .unwrap()
        .env("GITPOULPI_CONFIG_DIR", config_dir())
        .args([
            "--json",
            "--session",
            "cli-flow-templates",
            "flow",
            "templates",
        ])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let v: Value = serde_json::from_slice(&out.stdout).unwrap();
    let ids: Vec<&str> = v["templates"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["id"].as_str().unwrap())
        .collect();
    assert_eq!(
        ids,
        ["git-flow", "github-flow", "gitlab-flow", "trunk", "custom"]
    );
    let gitlab: Workflow = serde_json::from_value(v["templates"][2]["workflow"].clone()).unwrap();
    assert_eq!(gitlab.bases.len(), 3);
}

#[test]
fn import_reads_classic_git_flow_settings_and_writes_only_when_asked() {
    let repo = TestRepoBuilder::new()
        .with_initial()
        .config("gitflow.branch.master", "main")
        .config("gitflow.branch.develop", "next")
        .config("gitflow.prefix.feature", "feat/")
        .config("gitflow.prefix.versiontag", "v")
        .branch("next")
        .branch("feat/login")
        .checkout("main")
        .build();
    let root = repo.root();
    let session = "cli-flow-import";

    let none = run(root, session, &["flow", "import", "--from", "git-town"]);
    assert!(!none.status.success(), "no Git Town settings here");

    let status0 = status(root, session);
    assert!(!status0.configured);
    assert_eq!(status0.imports.len(), 1);
    assert_eq!(status0.imports[0].source, "git-flow");

    let printed = run_json(root, session, &["flow", "import", "--from", "git-flow"]);
    assert_eq!(printed["written"], false);
    assert!(!root.join(".git/gitpoulpi/workflow.toml").exists());
    let wf: Workflow = toml::from_str(printed["toml"].as_str().unwrap()).unwrap();
    assert_eq!(wf.bases[1].name, "next");

    run_json(
        root,
        session,
        &["flow", "import", "--from", "git-flow", "--write"],
    );
    let r = status(root, session);
    assert!(r.configured);
    assert_eq!(find(&r, "feat/login").type_id.as_deref(), Some("feature"));
    assert_eq!(find(&r, "feat/login").parent.as_ref().unwrap().name, "next");
    assert_eq!(find(&r, "next").parent.as_ref().unwrap().name, "main");
}

#[test]
fn import_reads_git_town_settings_and_parents() {
    let repo = TestRepoBuilder::new()
        .with_initial()
        .config("git-town.main-branch", "main")
        .config("git-town.perennial-branches", "qa")
        .branch("qa")
        .branch("login")
        .config("git-town-branch.login.parent", "qa")
        .checkout("main")
        .build();
    let root = repo.root();
    let session = "cli-flow-town";
    run_json(
        root,
        session,
        &["flow", "import", "--from", "git-town", "--write"],
    );
    let r = status(root, session);
    assert_eq!(find(&r, "qa").kind, BranchKind::Base);
    let login = find(&r, "login");
    assert_eq!(login.kind, BranchKind::Topic);
    let parent = login.parent.clone().unwrap();
    assert_eq!(parent.name, "qa");
    assert_eq!(serde_json::to_value(parent.source).unwrap(), "git_town");
}

// No test reads or writes the real config, data dir or credentials of whoever runs it.
gitpoulpi_testkit::isolate_binary!();
