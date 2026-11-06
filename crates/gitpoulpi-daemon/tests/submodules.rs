//! Submodule operations through the daemon, and
//! `general.keep_submodules_up_to_date`: after a checkout the submodules
//! follow the commits the superproject records -- only when the setting is
//! on. The env var is process-global, so everything runs in one test.

use std::sync::Arc;

use gitpoulpi_core::domain::{OperationKind, Submodule, SubmoduleState};
use gitpoulpi_core::GitBackend;
use gitpoulpi_daemon::Daemon;
use gitpoulpi_ipc::{Client, Request, ResponseError};
use gitpoulpi_testkit::FixtureRepoBuilder;
use serde_json::{json, Value};

const ENV: &str = "GITPOULPI_GENERAL_KEEP_SUBMODULES_UP_TO_DATE";

struct Rig {
    client: Client,
    id: gitpoulpi_core::domain::RepoId,
    next: u64,
}

impl Rig {
    async fn call(&mut self, command: OperationKind, args: Value) -> Result<Value, ResponseError> {
        self.next += 1;
        self.client
            .call(&Request {
                id: self.next,
                session: "submodules-test".into(),
                repo: Some(self.id.clone()),
                command,
                args,
            })
            .await
            .unwrap()
            .result
    }

    async fn list(&mut self) -> Vec<Submodule> {
        serde_json::from_value(
            self.call(OperationKind::Submodules, json!({}))
                .await
                .unwrap(),
        )
        .unwrap()
    }
}

#[tokio::test]
async fn submodule_requests_and_keeping_them_up_to_date() {
    std::env::remove_var(ENV);
    gitpoulpi_testkit::allow_file_submodules();
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
            .with_submodule("libs/lib")
            .unwrap()
            .build();
        let root = repo.root().to_path_buf();
        let inside = root.join("libs/lib");
        // `ahead` records a newer commit of the submodule than `main`.
        repo.git(&["checkout", "-q", "-b", "ahead"]);
        repo.git_at(
            &inside,
            &[
                "-c",
                "user.name=T",
                "-c",
                "user.email=t@t",
                "commit",
                "--allow-empty",
                "-m",
                "newer",
            ],
        );
        repo.git(&["add", "libs/lib"]);
        repo.git(&["commit", "-q", "-m", "bump the submodule"]);
        repo.git(&["checkout", "-q", "main"]);
        repo.git(&["submodule", "update"]);

        let handle = Daemon::new("submodules-test").spawn(backend).await.unwrap();
        let client = Client::connect(&handle.endpoint()).await.unwrap();
        let mut rig = Rig {
            client,
            id: repo.repo_ref().id,
            next: 1,
        };
        let listed = rig.list().await;
        assert_eq!(listed.len(), 1, "{name}");
        assert_eq!(listed[0].state, SubmoduleState::UpToDate, "{name}");

        // Off (the default): the checkout leaves the submodule behind.
        rig.call(
            OperationKind::Checkout,
            json!({ "target": { "Branch": "ahead" } }),
        )
        .await
        .unwrap();
        assert_eq!(
            rig.list().await[0].state,
            SubmoduleState::Modified,
            "{name}"
        );

        // An explicit update catches it up.
        rig.call(
            OperationKind::SubmoduleUpdate,
            json!({ "opts": { "paths": [], "init": false, "recursive": false } }),
        )
        .await
        .unwrap();
        assert_eq!(
            rig.list().await[0].state,
            SubmoduleState::UpToDate,
            "{name}"
        );

        // On: the checkout back brings the submodule along.
        std::env::set_var(ENV, "true");
        rig.call(
            OperationKind::Checkout,
            json!({ "target": { "Branch": "main" } }),
        )
        .await
        .unwrap_or_else(|e| panic!("{name}: {}", e.message));
        assert_eq!(
            rig.list().await[0].state,
            SubmoduleState::UpToDate,
            "{name}"
        );
        std::env::remove_var(ENV);
        handle.shutdown().await.unwrap();
    }
}

// No test reads or writes the real config, data dir or credentials of whoever runs it.
gitpoulpi_testkit::isolate_binary!();
