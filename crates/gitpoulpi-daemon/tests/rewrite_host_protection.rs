//! The host's own branch protection (S23) reaches the preview through the
//! recipe: a pushed branch the host protects is left out of the run exactly
//! like a workflow-protected one (per-branch tick, all/none), and a host that
//! could not be read changes nothing. Both backends.

use std::sync::Arc;

use gitpoulpi_core::domain::rewrite::{
    HostKind, HostProtection, IdentityMatch, IdentityRule, IdentitySet, Recipe, RewritePlan,
    RewriteWarning, Rule, RuleKind, Scope, SignaturePolicy, Who,
};
use gitpoulpi_core::domain::OperationKind;
use gitpoulpi_core::GitBackend;
use gitpoulpi_daemon::Daemon;
use gitpoulpi_ipc::rewrite::PreviewArgs;
use gitpoulpi_ipc::{Client, Request};
use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use serde_json::Value;

const SESSION: &str = "rewrite-host-protection-test";

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

/// `main` and `work`, both on `origin`: neither name is in the config's
/// default protected list apart from `main`.
fn repo() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit("alpha")
        .unwrap()
        .with_origin()
        .unwrap()
        .branch("work")
        .unwrap()
        .push("main")
        .unwrap()
        .push("work")
        .unwrap()
        .track("main", "origin")
        .unwrap()
        .track("work", "origin")
        .unwrap()
        .build()
}

fn recipe(host: Option<HostProtection>) -> Recipe {
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
    // `main` is protected by the config's defaults; it is not what this is about.
    recipe.scope_options.include_protected = vec!["main".into()];
    recipe.scope_options.host_protection = host;
    recipe
}

fn on_github(patterns: &[&str]) -> HostProtection {
    let mut host = HostProtection::new(HostKind::GitHub, "octo/thing", "https://github.com");
    host.asked = true;
    host.patterns = patterns.iter().map(|p| p.to_string()).collect();
    host
}

async fn preview(backend: Arc<dyn GitBackend>, repo: &FixtureRepo, recipe: Recipe) -> RewritePlan {
    let _turn = TURN.lock().await;
    let dir = tempfile::tempdir().unwrap();
    std::env::set_var("GITPOULPI_CONFIG_DIR", dir.path());
    let handle = Daemon::new(SESSION).spawn(backend).await.unwrap();
    let mut client = Client::connect(&handle.endpoint()).await.unwrap();
    let args = serde_json::to_value(PreviewArgs {
        recipe,
        vouched: Vec::new(),
    })
    .unwrap();
    let response = client
        .call(&Request {
            id: 1,
            session: SESSION.into(),
            repo: Some(repo.repo_ref().id),
            command: OperationKind::RewritePreview,
            args,
        })
        .await
        .unwrap();
    let value: Value = response.result.unwrap_or_else(|e| panic!("{}", e.message));
    let plan = serde_json::from_value(value).unwrap();
    handle.shutdown().await.unwrap();
    gitpoulpi_daemon::forget_rewrite_plans_for_tests();
    plan
}

fn work(plan: &RewritePlan) -> &gitpoulpi_core::domain::rewrite::RefChange {
    plan.refs
        .iter()
        .find(|r| r.name == "refs/heads/work")
        .expect("work is in the plan")
}

/// The plan left `work` out and said so (the protected-excluded warning).
fn left_out(plan: &RewritePlan) -> bool {
    plan.refs.iter().all(|r| r.name != "refs/heads/work")
        && plan.warnings.iter().any(|w| {
            matches!(w, RewriteWarning::ProtectedExcluded { refs } if refs.iter().any(|r| r == "refs/heads/work"))
        })
}

#[tokio::test]
async fn a_host_protected_pushed_branch_is_excluded_by_default_and_ticks_back_in() {
    for (name, backend) in backends() {
        let repo = repo();
        let plain = preview(backend.clone(), &repo, recipe(None)).await;
        assert!(
            work(&plain).included && !work(&plain).protected,
            "{name}: nothing protects work yet"
        );

        let guarded = preview(backend.clone(), &repo, recipe(Some(on_github(&["wor*"])))).await;
        assert!(left_out(&guarded), "{name}: {:?}", guarded.warnings);

        // The same checkbox as any protected branch: listed, it is included.
        let mut ticked = recipe(Some(on_github(&["wor*"])));
        ticked.scope_options.include_protected.push("work".into());
        let plan = preview(backend.clone(), &repo, ticked).await;
        assert!(work(&plan).protected && work(&plan).included, "{name}");

        // A host that could not be read (no patterns) changes nothing.
        let mut unknown = on_github(&[]);
        unknown.unknown = Some("the token lacks the scope".into());
        let plan = preview(backend, &repo, recipe(Some(unknown))).await;
        assert!(work(&plan).included && !work(&plan).protected, "{name}");
    }
}

// No test reads or writes the real config, data dir or credentials of whoever runs it.
gitpoulpi_testkit::isolate_binary!();
