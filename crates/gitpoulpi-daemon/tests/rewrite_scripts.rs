//! External history rewrite scripts through the daemon (slice S11): the policy
//! is the daemon's, never the client's; trust is remembered per repository by
//! content hash; a request can vouch for a script it typed; the workbench is
//! told which languages the settings allow.
//!
//! "Python" is the testkit's fixture program, so no interpreter is needed.
#![cfg(feature = "rewrite-scripts")]

use std::sync::Arc;

use gitpoulpi_core::domain::rewrite::script::ScriptCheck;
use gitpoulpi_core::domain::rewrite::{
    script_hash, Recipe, RewritePlan, RewriteRefusal, RewriteTools, Rule, RuleKind, Scope,
    ScriptLanguage, ScriptPolicy, ScriptRef, ScriptSource, ScriptTrust, SignaturePolicy,
};
use gitpoulpi_core::domain::OperationKind;
use gitpoulpi_core::{GitBackend, TypedError};
use gitpoulpi_daemon::Daemon;
use gitpoulpi_ipc::rewrite::{PreviewArgs, ScriptCheckArgs, ToolsArgs, TrustArgs};
use gitpoulpi_ipc::{Client, Request, ResponseError};
use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use serde_json::{json, Value};

const SESSION: &str = "rewrite-scripts-test";

/// Python is switched on for this test binary; everything else is default.
fn config() {
    static DIR: std::sync::OnceLock<tempfile::TempDir> = std::sync::OnceLock::new();
    let dir = DIR.get_or_init(|| {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("config.toml"),
            "[rewrite]\nexternal_scripts = [\"shell\", \"python\"]\n",
        )
        .unwrap();
        dir
    });
    std::env::set_var("GITPOULPI_CONFIG_DIR", dir.path());
    gitpoulpi_testkit::script_fixture::install();
}

struct Rig {
    handle: Option<gitpoulpi_daemon::DaemonHandle>,
    client: Client,
    repo: FixtureRepo,
    next: u64,
}

impl Rig {
    async fn new() -> Rig {
        config();
        let repo = FixtureRepoBuilder::new()
            .unwrap()
            .commit("alpha")
            .unwrap()
            .commit("beta")
            .unwrap()
            .build();
        let backend: Arc<dyn GitBackend> = Arc::new(gitpoulpi_cli_backend::SystemGitBackend::new());
        let handle = Daemon::new(SESSION).spawn(backend).await.unwrap();
        let client = Client::connect(&handle.endpoint()).await.unwrap();
        Rig {
            handle: Some(handle),
            client,
            repo,
            next: 1,
        }
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

    async fn ok<T: serde::de::DeserializeOwned>(
        &mut self,
        command: OperationKind,
        args: Value,
    ) -> T {
        let value = self
            .call(command, args)
            .await
            .unwrap_or_else(|e| panic!("{command:?} failed: {}", e.message));
        serde_json::from_value(value).unwrap()
    }

    async fn refusal(&mut self, command: OperationKind, args: Value) -> (RewriteRefusal, String) {
        match self.call(command, args).await {
            Err(ResponseError {
                kind: Some(TypedError::RewriteRefused(reason, detail)),
                ..
            }) => (reason, detail),
            other => panic!("expected a refusal, got {other:?}"),
        }
    }

    async fn stop(mut self) {
        if let Some(handle) = self.handle.take() {
            let _ = handle.shutdown().await;
        }
    }

    fn state_trusts(&self, hash: &str) -> bool {
        let path = gitpoulpi_session::RepoState::path_for(
            &self.repo.repo_ref().root,
            gitpoulpi_config::DEFAULT_REPO_DATA_LOCATION,
        );
        gitpoulpi_session::RepoState::load(&path).is_ok_and(|s| s.trusts_script(hash))
    }
}

fn script(text: &str) -> ScriptRef {
    ScriptRef {
        language: ScriptLanguage::Python,
        source: ScriptSource::Inline {
            text: text.to_string(),
        },
        params: Default::default(),
    }
}

fn recipe(text: &str, forged: ScriptPolicy) -> Recipe {
    let mut recipe = Recipe::new(
        "external",
        Scope::default(),
        vec![Rule::new(RuleKind::Script(script(text)))],
    );
    recipe.options.signatures = Some(SignaturePolicy::Strip);
    recipe.script_policy = forged;
    recipe
}

fn preview_args(text: &str, forged: ScriptPolicy, vouched: Vec<String>) -> Value {
    serde_json::to_value(PreviewArgs {
        recipe: recipe(text, forged),
        vouched,
    })
    .unwrap()
}

#[tokio::test(flavor = "multi_thread")]
async fn trust_is_the_daemons_to_give_a_client_cannot_forge_it() {
    let mut rig = Rig::new().await;
    let text = "mode=upper\n";
    let hash = script_hash(text);
    // A client that claims everything is trusted, with a huge timeout.
    let forged = ScriptPolicy {
        shell: true,
        python: true,
        lua: true,
        timeout_ms: 1,
        trusted: vec![hash.clone()],
        ..ScriptPolicy::default()
    };
    let (reason, detail) = rig
        .refusal(
            OperationKind::RewritePreview,
            preview_args(text, forged.clone(), vec![]),
        )
        .await;
    assert_eq!(reason, RewriteRefusal::ScriptNotTrusted);
    assert_eq!(detail, hash);
    assert!(!rig.state_trusts(&hash));

    // A person trusts it: remembered per repository, and the preview works
    // (the daemon's own 5 s timeout, not the forged 1 ms).
    let _: Value = rig
        .ok(
            OperationKind::RewriteTrustScript,
            serde_json::to_value(TrustArgs {
                hash: hash.clone(),
                label: "upper.py".into(),
            })
            .unwrap(),
        )
        .await;
    assert!(rig.state_trusts(&hash));
    let plan: RewritePlan = rig
        .ok(
            OperationKind::RewritePreview,
            preview_args(text, forged, vec![]),
        )
        .await;
    assert_eq!(plan.counts.commits_changed, 2);

    // A changed script is another hash: refused again.
    let (reason, _) = rig
        .refusal(
            OperationKind::RewritePreview,
            preview_args("mode=upper\n# edited\n", ScriptPolicy::default(), vec![]),
        )
        .await;
    assert_eq!(reason, RewriteRefusal::ScriptNotTrusted);

    // Not a hash: refused, nothing stored.
    let (reason, _) = rig
        .refusal(
            OperationKind::RewriteTrustScript,
            serde_json::to_value(TrustArgs {
                hash: "abc".into(),
                label: String::new(),
            })
            .unwrap(),
        )
        .await;
    assert_eq!(reason, RewriteRefusal::InvalidRule);
    rig.stop().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn a_vouched_script_runs_for_that_request_only() {
    let mut rig = Rig::new().await;
    let text = "mode=count\n";
    let hash = script_hash(text);
    let plan: RewritePlan = rig
        .ok(
            OperationKind::RewritePreview,
            preview_args(text, ScriptPolicy::default(), vec![hash.clone()]),
        )
        .await;
    assert_eq!(plan.counts.commits_changed, 2);
    assert!(!rig.state_trusts(&hash), "vouching stores nothing");
    // The next request without it is refused again.
    let (reason, _) = rig
        .refusal(
            OperationKind::RewritePreview,
            preview_args(text, ScriptPolicy::default(), vec![]),
        )
        .await;
    assert_eq!(reason, RewriteRefusal::ScriptNotTrusted);

    // The check: vouched is trusted and handshakes, unvouched is an answer.
    let check_args = |vouched: Vec<String>| {
        serde_json::to_value(ScriptCheckArgs {
            script: script(text),
            vouched,
        })
        .unwrap()
    };
    let check: ScriptCheck = rig
        .ok(
            OperationKind::RewriteScriptCheck,
            check_args(vec![hash.clone()]),
        )
        .await;
    assert!(check.ok(), "{:?}", check.problems);
    assert_eq!(check.trust, ScriptTrust::Trusted);
    let check: ScriptCheck = rig
        .ok(OperationKind::RewriteScriptCheck, check_args(vec![]))
        .await;
    assert_eq!(check.trust, ScriptTrust::Untrusted);
    assert_eq!(check.hash.as_deref(), Some(hash.as_str()));
    rig.stop().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn the_settings_decide_the_languages_and_the_workbench_is_told() {
    let mut rig = Rig::new().await;
    // Python and the shell are on in this config, external Lua is not.
    let tools: RewriteTools = rig
        .ok(
            OperationKind::RewriteTools,
            serde_json::to_value(ToolsArgs { fresh: false }).unwrap(),
        )
        .await;
    let switches = tools
        .external_scripts
        .expect("the daemon fills the switches");
    assert!(switches.shell && switches.python && !switches.lua);

    let text = "mode=upper\n";
    let mut lua_external = script(text);
    lua_external.language = ScriptLanguage::LuaExternal;
    let args = serde_json::to_value(ScriptCheckArgs {
        script: lua_external,
        vouched: vec![script_hash(text)],
    })
    .unwrap();
    let (reason, detail) = rig.refusal(OperationKind::RewriteScriptCheck, args).await;
    assert_eq!(reason, RewriteRefusal::ScriptsDisabled);
    assert!(detail.contains("rewrite.external_scripts"), "{detail}");
    let _ = json!(null);
    rig.stop().await;
}

// No test reads or writes the real config, data dir or credentials of whoever runs it.
gitpoulpi_testkit::isolate_binary!();
