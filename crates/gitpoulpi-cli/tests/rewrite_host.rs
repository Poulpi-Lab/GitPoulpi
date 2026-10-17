//! `gitpoulpi rewrite push` when the host's branch protection refuses the
//! force push (S23): the answer is `rejected_protected_by_host` and carries
//! the three ways through with the exact settings link of the provider, in
//! English. Nothing is ever done on the host. Assertions are on deserialized
//! JSON; no network (the fetch URL only names the host, the push goes to a
//! local bare repository whose hook says what GitHub says).
#![cfg(feature = "history-rewrite")]

use std::path::Path;

use assert_cmd::Command;
use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use serde_json::Value;

fn config_dir() -> &'static Path {
    static DIR: std::sync::OnceLock<tempfile::TempDir> = std::sync::OnceLock::new();
    DIR.get_or_init(|| tempfile::tempdir().unwrap()).path()
}

fn run(repo: &FixtureRepo, session: &str, args: &[&str]) -> (i32, Value) {
    let output = Command::cargo_bin("gitpoulpi")
        .unwrap()
        .env("GITPOULPI_CONFIG_DIR", config_dir())
        .args(["--json", "--session", session])
        .args(["--repo", &repo.root().to_string_lossy()])
        .args(args)
        .current_dir(repo.root())
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let value = serde_json::from_str(&stdout).unwrap_or_else(|e| {
        panic!(
            "not JSON ({e}): {stdout}\n{}",
            String::from_utf8_lossy(&output.stderr)
        )
    });
    (output.status.code().unwrap_or(-1), value)
}

struct Daemon {
    session: String,
    cwd: std::path::PathBuf,
}

impl Daemon {
    fn start(repo: &FixtureRepo, session: &str) -> Daemon {
        let daemon = Daemon {
            session: session.to_string(),
            cwd: repo.root().to_path_buf(),
        };
        let status = std::process::Command::new(env!("CARGO_BIN_EXE_gitpoulpi"))
            .env("GITPOULPI_CONFIG_DIR", config_dir())
            .current_dir(&daemon.cwd)
            .args(["-s", session, "daemon", "start", "--json"])
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .unwrap();
        assert!(status.success(), "daemon start");
        daemon
    }
}

impl Drop for Daemon {
    fn drop(&mut self) {
        let _ = std::process::Command::new(env!("CARGO_BIN_EXE_gitpoulpi"))
            .env("GITPOULPI_CONFIG_DIR", config_dir())
            .current_dir(&self.cwd)
            .args(["-s", &self.session, "daemon", "stop", "--json"])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();
    }
}

/// `main` pushed to a bare `origin` that refuses every push the way a
/// protected GitHub branch does, while `origin` is *named* a GitHub URL.
fn protected_by_github() -> FixtureRepo {
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
    let bare = repo.remote_path().unwrap().to_path_buf();
    let hook = bare.join("hooks").join("pre-receive");
    std::fs::create_dir_all(hook.parent().unwrap()).unwrap();
    std::fs::write(
        &hook,
        "#!/bin/sh\necho 'remote: error: GH006: Protected branch update failed for refs/heads/main.' >&2\nexit 1\n",
    )
    .unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&hook, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    repo.git(&[
        "config",
        "remote.origin.url",
        "https://github.com/octo/thing.git",
    ]);
    repo.git(&["config", "remote.origin.pushurl", &bare.to_string_lossy()]);
    repo
}

#[test]
fn a_push_the_host_refuses_carries_the_three_ways_and_the_exact_settings_link() {
    let repo = protected_by_github();
    let session = "cli-rw-host";
    let _daemon = Daemon::start(&repo, session);
    let go = [
        "rewrite",
        "run",
        "--match-email",
        "testkit@gitpoulpi.local",
        "--set-name",
        "Casey Example",
        "--set-email",
        "casey@example.com",
        "--signatures",
        "strip",
        "--include-protected",
        "main",
        "--yes",
    ];
    let (code, applied) = run(&repo, session, &go);
    assert_eq!(code, 0, "{applied}");

    let (code, pushed) = run(
        &repo,
        session,
        &["rewrite", "push", "--include-protected", "main", "--yes"],
    );
    assert_eq!(code, 4, "{pushed}");
    assert_eq!(pushed["results"][0]["status"], "rejected_protected_by_host");

    let host = &pushed["protected_by_host"];
    assert_eq!(host["provider"], "GitHub", "{pushed}");
    assert_eq!(host["branches"][0], "main");
    let ways = host["ways"].as_array().unwrap();
    assert_eq!(
        ways.iter()
            .map(|w| w["way"].as_str().unwrap())
            .collect::<Vec<_>>(),
        ["way_lift", "way_rename", "way_recreate"]
    );
    assert_eq!(
        ways[0]["links"][0],
        "https://github.com/octo/thing/settings/branches"
    );
    assert_eq!(
        ways[2]["links"][0],
        "https://github.com/octo/thing/settings"
    );
    let loses = host["recreate_loses"].as_array().unwrap();
    assert!(loses.len() >= 6, "{loses:?}");
    let text = host["text"].as_str().unwrap();
    assert!(text.contains("never changes the host's settings"), "{text}");
}

// No test reads or writes the real config, data dir or credentials of whoever runs it.
gitpoulpi_testkit::isolate_binary!();
