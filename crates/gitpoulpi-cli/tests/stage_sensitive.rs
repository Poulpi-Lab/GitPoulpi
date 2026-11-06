//! `gitpoulpi stage` refuses files that look sensitive with a JSON refusal
//! and exit code 2, and `--allow-sensitive` is the retry. Deserialized JSON
//! only.

use std::path::Path;

use assert_cmd::Command;
use gitpoulpi_testkit::FixtureRepoBuilder;
use serde_json::Value;

const BODY: &str = "q7Xk2LmZp9RtV4bN8wYcJ3hGfD5sAe6U1oIx";

fn isolated_config_dir() -> &'static Path {
    static DIR: std::sync::OnceLock<tempfile::TempDir> = std::sync::OnceLock::new();
    DIR.get_or_init(|| tempfile::tempdir().unwrap()).path()
}

fn run(root: &Path, session: &str, args: &[&str]) -> (Option<i32>, Value, String) {
    let output = Command::cargo_bin("gitpoulpi")
        .unwrap()
        .env("GITPOULPI_CONFIG_DIR", isolated_config_dir())
        .arg("--json")
        .args(["--session", session])
        .args(["--repo", &root.to_string_lossy()])
        .args(args)
        .current_dir(root)
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    (
        output.status.code(),
        serde_json::from_str(&stdout).unwrap_or(Value::Null),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

fn staged(root: &Path) -> String {
    let out = std::process::Command::new("git")
        .args(["diff", "--cached", "--name-only"])
        .current_dir(root)
        .output()
        .unwrap();
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

#[test]
fn a_sensitive_file_is_refused_as_json_then_staged_with_the_flag() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build();
    std::fs::write(repo.root().join("notes.txt"), format!("key ghp_{BODY}\n")).unwrap();
    std::fs::write(repo.root().join("server.pem"), "pem text\n").unwrap();
    let session = "cli-stage-sensitive";

    let (code, json, _) = run(repo.root(), session, &["stage", "--all"]);
    assert_eq!(code, Some(2), "{json}");
    assert_eq!(json["error"], "sensitive_files");
    let files = json["files"].as_array().unwrap();
    let paths: Vec<&str> = files.iter().map(|f| f["path"].as_str().unwrap()).collect();
    assert!(
        paths.contains(&"server.pem") && paths.contains(&"notes.txt"),
        "{json}"
    );
    assert!(
        !json.to_string().contains(BODY),
        "no value in the refusal: {json}"
    );
    assert_eq!(staged(repo.root()), "", "nothing was staged");

    let (code, json, stderr) = run(
        repo.root(),
        session,
        &["stage", "--all", "--allow-sensitive"],
    );
    assert_eq!(code, Some(0), "{json} {stderr}");
    assert!(staged(repo.root()).contains("server.pem"));
}
