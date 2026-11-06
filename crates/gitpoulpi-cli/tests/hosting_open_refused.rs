//! `hosting open --browser` launches only an `http`/`https` address: anything
//! else is a JSON error and nothing is launched (the test hook log stays
//! empty).

use std::path::Path;

use assert_cmd::Command;
use gitpoulpi_testkit::FixtureRepoBuilder;
use serde_json::Value;

fn run(config: &Path, log: &Path, cwd: &Path, args: &[&str]) -> (bool, Value) {
    let output = Command::cargo_bin("gitpoulpi")
        .unwrap()
        .env("GITPOULPI_CONFIG_DIR", config)
        .env("GITPOULPI_TOKEN_CORP", "test-token")
        .env("GITPOULPI_OPEN_LOG", log)
        .arg("--json")
        .args(["--session", "hosting-open-refused"])
        .args(args)
        .current_dir(cwd)
        .output()
        .expect("run gitpoulpi");
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let json = serde_json::from_str(&stdout).unwrap_or_else(|e| {
        panic!(
            "not JSON ({e}): {stdout} / {}",
            String::from_utf8_lossy(&output.stderr)
        )
    });
    (output.status.success(), json)
}

#[test]
fn a_non_web_address_is_refused_with_a_json_error_and_nothing_is_launched() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "a\n", "start")
        .unwrap()
        .with_origin()
        .unwrap()
        .build();
    repo.git(&["remote", "set-url", "origin", "http://127.0.0.1:9/g/p.git"]);
    let config = tempfile::tempdir().unwrap();
    // An integration whose base address is not a web one: the address built
    // from it must not reach the OS launcher.
    std::fs::write(
        config.path().join("config.toml"),
        "[[integrations]]\nid = \"corp\"\nkind = \"gitlab_self_managed\"\nurl = \"ftp://127.0.0.1:9\"\ntoken_ref = \"integration:corp\"\nname = \"corp\"\nuser = { login = \"kai\" }\n",
    )
    .unwrap();
    let log = config.path().join("opened.log");

    let (ok, link) = run(
        config.path(),
        &log,
        repo.root(),
        &[
            "hosting",
            "link",
            "pulls",
            "--id",
            "corp",
            "--project",
            "g/p",
        ],
    );
    assert!(ok, "{link}");

    let (ok, json) = run(
        config.path(),
        &log,
        repo.root(),
        &["hosting", "open", "pr", "5", "--browser"],
    );
    assert!(!ok, "{json}");
    let message = json["error"].as_str().expect("an error string");
    assert!(message.contains("refused"), "{message}");
    assert!(
        std::fs::read_to_string(&log).unwrap_or_default().is_empty(),
        "nothing may be launched"
    );
}

// No test reads or writes the real config, data dir or credentials of whoever runs it.
gitpoulpi_testkit::isolate_binary!();
