//! The fast-forward policy of a merge through the daemon: no policy in the
//! request follows the repository's own choice, else `git.merge_fast_forward`
//! (default `never`: a merge commit); a policy in the request wins over both.
//! The env var is process-global, so one test runs everything in sequence.

use std::sync::Arc;

use gitpoulpi_core::domain::{MergeOutcome, Oid, OperationKind};
use gitpoulpi_core::{GitBackend, TypedError};
use gitpoulpi_daemon::Daemon;
use gitpoulpi_ipc::{Client, Request, ResponseError};
use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use serde_json::{json, Value};

const ENV: &str = "GITPOULPI_GIT_MERGE_FAST_FORWARD";

struct Rig {
    client: Client,
    repo: FixtureRepo,
    n: u64,
}

impl Rig {
    async fn call(&mut self, args: Value) -> Result<MergeOutcome, ResponseError> {
        self.n += 1;
        let refs = self.request(OperationKind::Refs, json!({})).await.unwrap();
        let feature = refs
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["kind"]["LocalBranch"]["name"] == "feature")
            .unwrap()
            .clone();
        let mut full = json!({ "from": feature });
        if !args.is_null() {
            full["opts"] = args;
        }
        self.request(OperationKind::Merge, full)
            .await
            .map(|v| serde_json::from_value(v).unwrap())
    }

    async fn request(
        &mut self,
        command: OperationKind,
        args: Value,
    ) -> Result<Value, ResponseError> {
        self.n += 1;
        self.client
            .call(&Request {
                id: self.n,
                session: "merge-policy-test".into(),
                repo: Some(self.repo.repo_ref().id),
                command,
                args,
            })
            .await
            .unwrap()
            .result
    }

    /// Puts `main` back to where it was and drops any leftover state.
    fn reset(&self, to: &Oid) {
        self.repo.git(&["reset", "-q", "--hard", &to.to_hex()]);
    }

    fn parents(&self) -> usize {
        self.repo
            .git(&["rev-list", "--parents", "-n1", "HEAD"])
            .split_whitespace()
            .count()
            - 1
    }
}

#[tokio::test]
async fn the_policy_comes_from_the_request_then_the_repo_then_the_config() {
    std::env::remove_var(ENV);
    let backends: Vec<(&str, Arc<dyn GitBackend>)> = vec![
        (
            "system",
            Arc::new(gitpoulpi_cli_backend::SystemGitBackend::new()),
        ),
        ("git2", Arc::new(gitpoulpi_git2_backend::Git2Backend::new())),
    ];
    for (name, backend) in backends {
        let repo = FixtureRepoBuilder::new()
            .unwrap()
            .commit("initial")
            .unwrap()
            .build();
        repo.git(&["checkout", "-q", "-b", "feature"]);
        std::fs::write(repo.root().join("f.txt"), "f").unwrap();
        repo.git(&["add", "f.txt"]);
        repo.git(&["commit", "-q", "-m", "feature work"]);
        repo.git(&["checkout", "-q", "main"]);
        let old_main = Oid::from_hex(&repo.git(&["rev-parse", "HEAD"])).unwrap();
        let handle = Daemon::new("merge-policy-test")
            .spawn(backend)
            .await
            .unwrap();
        let client = Client::connect(&handle.endpoint()).await.unwrap();
        let mut rig = Rig { client, repo, n: 1 };

        // Nothing set: the default is `never`, a merge commit.
        let out = rig.call(Value::Null).await.unwrap();
        assert!(matches!(out, MergeOutcome::Merged(_)), "{name}: {out:?}");
        assert_eq!(rig.parents(), 2, "{name}");
        rig.reset(&old_main);

        // The config says `allowed`: a fast-forward.
        std::env::set_var(ENV, "allowed");
        let out = rig.call(Value::Null).await.unwrap();
        assert!(
            matches!(out, MergeOutcome::FastForwarded(_)),
            "{name}: {out:?}"
        );
        rig.reset(&old_main);

        // The repository's own choice wins over the config.
        let state = rig.repo.state_dir().join("repo-state.toml");
        std::fs::create_dir_all(state.parent().unwrap()).unwrap();
        std::fs::write(&state, "[preferences]\nmerge_fast_forward = \"never\"\n").unwrap();
        let out = rig.call(Value::Null).await.unwrap();
        assert!(matches!(out, MergeOutcome::Merged(_)), "{name}: {out:?}");
        rig.reset(&old_main);

        // A value that is not a policy is ignored: the config applies.
        std::fs::write(
            &state,
            "[preferences]\nmerge_fast_forward = \"sometimes\"\n",
        )
        .unwrap();
        let out = rig.call(Value::Null).await.unwrap();
        assert!(
            matches!(out, MergeOutcome::FastForwarded(_)),
            "{name}: {out:?}"
        );
        rig.reset(&old_main);

        // The request wins over the repository and the config.
        std::fs::write(&state, "[preferences]\nmerge_fast_forward = \"allowed\"\n").unwrap();
        let out = rig.call(json!({ "ff": "never" })).await.unwrap();
        assert!(matches!(out, MergeOutcome::Merged(_)), "{name}: {out:?}");
        rig.reset(&old_main);

        // `only` fast-forwards, and refuses (typed) once main has moved on.
        let out = rig.call(json!({ "ff": "only" })).await.unwrap();
        assert!(
            matches!(out, MergeOutcome::FastForwarded(_)),
            "{name}: {out:?}"
        );
        rig.reset(&old_main);
        std::fs::write(rig.repo.root().join("m.txt"), "m").unwrap();
        rig.repo.git(&["add", "m.txt"]);
        rig.repo.git(&["commit", "-q", "-m", "main work"]);
        let moved = Oid::from_hex(&rig.repo.git(&["rev-parse", "HEAD"])).unwrap();
        let err = rig.call(json!({ "ff": "only" })).await.unwrap_err();
        assert_eq!(
            err.kind,
            Some(TypedError::NotFastForward),
            "{name}: {err:?}"
        );
        assert_eq!(
            Oid::from_hex(&rig.repo.git(&["rev-parse", "HEAD"])).unwrap(),
            moved,
            "{name}: nothing moved"
        );

        std::env::remove_var(ENV);
        handle.shutdown().await.unwrap();
    }
}
