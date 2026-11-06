//! Staging a file that looks sensitive is refused with a typed error until
//! the caller allows it, on both backends, for the file, folder and all-files
//! stage. A refused stage changes nothing.

use std::sync::Arc;

use gitpoulpi_core::domain::{OperationKind, StageTarget};
use gitpoulpi_core::privacy::Reason;
use gitpoulpi_core::{GitBackend, TypedError};
use gitpoulpi_daemon::Daemon;
use gitpoulpi_ipc::{Client, Request, ResponseError};
use gitpoulpi_testkit::FixtureRepoBuilder;
use serde_json::{json, Value};

gitpoulpi_testkit::isolate_binary!();

const BODY: &str = "q7Xk2LmZp9RtV4bN8wYcJ3hGfD5sAe6U1oIx";

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
    handle: gitpoulpi_daemon::DaemonHandle,
    client: Client,
    repo: gitpoulpi_testkit::FixtureRepo,
    next: u64,
    session: String,
}

impl Rig {
    async fn new(backend: Arc<dyn GitBackend>, name: &str) -> Rig {
        let repo = FixtureRepoBuilder::new()
            .unwrap()
            .commit("initial")
            .unwrap()
            .build();
        let handle = Daemon::new(name).spawn(backend).await.unwrap();
        let client = Client::connect(&handle.endpoint()).await.unwrap();
        Rig {
            handle,
            client,
            repo,
            next: 1,
            session: name.to_string(),
        }
    }

    async fn stage(
        &mut self,
        command: OperationKind,
        target: StageTarget,
        allow: Option<bool>,
    ) -> Result<Value, ResponseError> {
        self.next += 1;
        let mut args = json!({ "target": target });
        if let Some(a) = allow {
            args["allow_sensitive"] = json!(a);
        }
        self.client
            .call(&Request {
                id: self.next,
                session: self.session.clone(),
                repo: Some(self.repo.repo_ref().id.clone()),
                command,
                args,
            })
            .await
            .unwrap()
            .result
    }

    fn staged(&self) -> String {
        self.repo.git(&["diff", "--cached", "--name-only"])
    }
}

fn write(rig: &Rig, path: &str, text: &str) {
    let full = rig.repo.root().join(path);
    std::fs::create_dir_all(full.parent().unwrap()).unwrap();
    std::fs::write(full, text).unwrap();
}

fn refused_files(err: &ResponseError) -> Vec<gitpoulpi_core::privacy::Sensitive> {
    match &err.kind {
        Some(TypedError::SensitiveStage { files }) => files.clone(),
        other => panic!("expected SensitiveStage, got {other:?} ({})", err.message),
    }
}

#[tokio::test]
async fn a_sensitive_file_is_refused_then_allowed_on_retry() {
    for (name, backend) in backends() {
        let mut rig = Rig::new(backend, &format!("guard-file-{name}")).await;
        write(&rig, ".env", "MODE=dev\n");
        write(&rig, "src/a.txt", "plain\n");

        let err = rig
            .stage(
                OperationKind::StageFile,
                StageTarget::File(".env".into()),
                None,
            )
            .await
            .expect_err(name);
        let files = refused_files(&err);
        assert_eq!(files.len(), 1, "{name}");
        assert_eq!(files[0].path, ".env");
        assert!(matches!(&files[0].reason, Reason::Pattern(p) if p == ".env"));
        assert_eq!(rig.staged(), "", "{name}: nothing was staged");

        // An ordinary file is not asked about.
        rig.stage(
            OperationKind::StageFile,
            StageTarget::File("src/a.txt".into()),
            None,
        )
        .await
        .unwrap();
        assert_eq!(rig.staged(), "src/a.txt", "{name}");

        // The retry that says so goes through.
        rig.stage(
            OperationKind::StageFile,
            StageTarget::File(".env".into()),
            Some(true),
        )
        .await
        .unwrap();
        assert!(rig.staged().contains(".env"), "{name}");
        rig.handle.shutdown().await.unwrap();
    }
}

#[tokio::test]
async fn stage_all_and_folder_name_every_flagged_file_without_its_value() {
    for (name, backend) in backends() {
        let mut rig = Rig::new(backend, &format!("guard-all-{name}")).await;
        write(&rig, "keys/server.pem", "not really a key\n");
        write(&rig, "keys/readme.txt", "hello\n");
        write(&rig, "notes.txt", &format!("token ghp_{BODY}\n"));
        write(&rig, ".env.example", "A=1\n");

        let err = rig
            .stage(OperationKind::StageAll, StageTarget::All, None)
            .await
            .expect_err(name);
        let files = refused_files(&err);
        let paths: Vec<&str> = files.iter().map(|f| f.path.as_str()).collect();
        assert!(paths.contains(&"keys/server.pem"), "{name}: {paths:?}");
        assert!(paths.contains(&"notes.txt"), "{name}: {paths:?}");
        assert!(!paths.contains(&".env.example"), "{name}: {paths:?}");
        assert!(!paths.contains(&"keys/readme.txt"), "{name}: {paths:?}");
        let secret = files
            .iter()
            .find(|f| matches!(f.reason, Reason::Secret { .. }))
            .expect("a content finding");
        assert!(matches!(&secret.reason,
            Reason::Secret { rule, .. } if rule == "github-token"));
        // No value anywhere in what came back.
        let wire = format!("{err:?}{}", serde_json::to_string(&files).unwrap());
        assert!(!wire.contains(BODY), "{name}: {wire}");
        assert_eq!(rig.staged(), "", "{name}");

        // A folder is judged by what is under it only.
        let err = rig
            .stage(
                OperationKind::StageFolder,
                StageTarget::Folder("keys".into()),
                None,
            )
            .await
            .expect_err(name);
        let files = refused_files(&err);
        assert!(files.iter().all(|f| f.path.starts_with("keys/")), "{name}");

        rig.stage(OperationKind::StageAll, StageTarget::All, Some(true))
            .await
            .unwrap();
        assert!(rig.staged().contains("notes.txt"), "{name}");
        rig.handle.shutdown().await.unwrap();
    }
}

#[tokio::test]
async fn a_binary_file_is_matched_by_name_only() {
    for (name, backend) in backends() {
        let mut rig = Rig::new(backend, &format!("guard-bin-{name}")).await;
        let mut bytes = b"\0\0ghp_".to_vec();
        bytes.extend_from_slice(BODY.as_bytes());
        std::fs::write(rig.repo.root().join("blob.dat"), bytes).unwrap();
        rig.stage(
            OperationKind::StageFile,
            StageTarget::File("blob.dat".into()),
            None,
        )
        .await
        .unwrap_or_else(|e| panic!("{name}: {e:?}"));
        assert_eq!(rig.staged(), "blob.dat", "{name}");
        rig.handle.shutdown().await.unwrap();
    }
}
