//! External history rewrite scripts (shell, Python, external Lua) through the
//! real binary: the settings that switch a language on, the trust a person
//! gives (`rewrite script trust`, `--trust`), and the typed refusals. The
//! assertions are on deserialized JSON and exit codes.
//!
//! "Python" is the testkit's fixture program (no interpreter needed); the
//! shell test uses the machine's `sh` and is skipped without one.
#![cfg(all(feature = "history-rewrite", feature = "rewrite-scripts"))]

use std::path::{Path, PathBuf};

use assert_cmd::Command;
use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use serde_json::{json, Value};

/// A config directory per test, with `[rewrite] external_scripts` as given.
fn config_with(external_scripts: &[&str]) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let list = external_scripts
        .iter()
        .map(|s| format!("\"{s}\""))
        .collect::<Vec<_>>()
        .join(", ");
    std::fs::write(
        dir.path().join("config.toml"),
        format!("[rewrite]\nexternal_scripts = [{list}]\n"),
    )
    .unwrap();
    dir
}

fn run(repo: &FixtureRepo, config: &Path, session: &str, args: &[&str]) -> (i32, Value) {
    let mut command = Command::cargo_bin("gitpoulpi").unwrap();
    command
        .env("GITPOULPI_CONFIG_DIR", config)
        .env(
            "GITPOULPI_TOOL_PYTHON_PATH",
            gitpoulpi_testkit::script_fixture::program(),
        )
        .args(["--json", "--session", session])
        .args(["--repo", &repo.root().to_string_lossy()])
        .args(args)
        .current_dir(repo.root());
    let output = command.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let value = serde_json::from_str(&stdout).unwrap_or_else(|e| {
        panic!(
            "not JSON ({e}): {stdout}\n{}",
            String::from_utf8_lossy(&output.stderr)
        )
    });
    (output.status.code().unwrap_or(-1), value)
}

fn fixture() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit("alpha")
        .unwrap()
        .commit("beta")
        .unwrap()
        .commit("gamma")
        .unwrap()
        .build()
}

fn write(repo: &FixtureRepo, name: &str, text: &str) -> String {
    let path: PathBuf = repo.root().join(name);
    std::fs::write(&path, text).unwrap();
    path.to_string_lossy().into_owned()
}

fn hash_of(text: &str) -> String {
    gitpoulpi_core::domain::rewrite::script_hash(text)
}

fn authors(repo: &FixtureRepo) -> Vec<String> {
    repo.git(&["log", "--format=%an"])
        .lines()
        .map(str::to_string)
        .collect()
}

/// A script nobody trusted is refused with its hash; trusting it (a person,
/// on the command line) lets it run; a change asks again; `--trust` does the
/// trusting in the same command.
#[test]
fn an_external_script_needs_a_persons_trust_and_asks_again_after_a_change() {
    let repo = fixture();
    let config = config_with(&["shell", "python"]);
    let session = "cli-rw-trust";
    let text = "mode=upper\n";
    let file = write(&repo, "upper.py", text);

    // check: not trusted, not started, exit 2, with the hash.
    let (code, check) = run(
        &repo,
        config.path(),
        session,
        &["rewrite", "script", "check", &file],
    );
    assert_eq!(code, 2, "{check}");
    assert_eq!(check["trust"], "untrusted");
    assert_eq!(check["hash"], hash_of(text));

    // preview: the typed refusal names the hash.
    let preview = [
        "rewrite",
        "preview",
        "--script",
        &file,
        "--signatures",
        "strip",
    ];
    let (code, value) = run(&repo, config.path(), session, &preview);
    assert_eq!(code, 2, "{value}");
    assert_eq!(value["refused"]["reason"], "script_not_trusted");
    assert_eq!(value["refused"]["script"]["hash"], hash_of(text));

    // trust: a person's command; the same script now previews.
    let (code, value) = run(
        &repo,
        config.path(),
        session,
        &["rewrite", "script", "trust", &file],
    );
    assert_eq!(code, 0, "{value}");
    assert_eq!(value["trusted"]["hash"], hash_of(text));
    let (code, check) = run(
        &repo,
        config.path(),
        session,
        &["rewrite", "script", "check", &file],
    );
    assert_eq!(code, 0, "{check}");
    assert_eq!(check["trust"], "trusted");
    let before = authors(&repo);
    let (code, value) = run(&repo, config.path(), session, &preview);
    assert_eq!(code, 0, "{value}");
    assert_eq!(authors(&repo), before, "a preview changes nothing");

    // A changed script is a different hash: refused again.
    std::fs::write(&file, "mode=upper\n# edited\n").unwrap();
    let (code, value) = run(&repo, config.path(), session, &preview);
    assert_eq!(code, 2, "{value}");
    assert_eq!(value["refused"]["reason"], "script_not_trusted");
    assert_eq!(
        value["refused"]["script"]["hash"],
        hash_of("mode=upper\n# edited\n")
    );

    // `--trust` trusts and runs in one command: the history changes.
    let (code, value) = run(
        &repo,
        config.path(),
        session,
        &[
            "rewrite",
            "run",
            "--script",
            &file,
            "--trust",
            "--signatures",
            "strip",
        ],
    );
    assert_eq!(code, 0, "{value}");
    assert!(authors(&repo).iter().all(|a| *a == a.to_uppercase()));
}

/// Python is off by default (only the shell is on): refused with the setting
/// named; `rewrite.external_scripts` switches it on.
#[test]
fn a_language_that_is_off_in_the_settings_is_refused_with_its_reason() {
    let repo = fixture();
    let text = "mode=upper\n";
    let file = write(&repo, "upper.py", text);
    let args = [
        "rewrite",
        "preview",
        "--script",
        &file,
        "--trust",
        "--signatures",
        "strip",
    ];

    let off = config_with(&["shell"]);
    let (code, value) = run(&repo, off.path(), "cli-rw-off", &args);
    assert_eq!(code, 2, "{value}");
    assert_eq!(value["refused"]["reason"], "scripts_disabled");
    assert!(value["refused"]["message"]
        .as_str()
        .unwrap()
        .contains("rewrite.external_scripts"));

    let on = config_with(&["shell", "python"]);
    let (code, value) = run(&repo, on.path(), "cli-rw-on", &args);
    assert_eq!(code, 0, "{value}");

    // `tools` tells the workbench which languages the setting allows.
    let (code, tools) = run(&repo, off.path(), "cli-rw-off", &["rewrite", "tools"]);
    assert_eq!(code, 0);
    assert_eq!(
        tools["external_scripts"],
        json!({"shell": true, "python": false, "lua": false})
    );
}

/// A shell script, with the machine's `sh` (skipped without one).
#[test]
fn a_shell_script_runs_after_it_is_trusted() {
    if !std::process::Command::new("sh")
        .args(["-c", ":"])
        .status()
        .is_ok_and(|s| s.success())
    {
        eprintln!("skipped: no sh");
        return;
    }
    let repo = fixture();
    let config = config_with(&["shell"]);
    let session = "cli-rw-shell";
    let text = "#!/bin/sh\nwhile IFS= read -r line; do\n  printf '{\"message\":\"shell\\\\n\"}\\n'\ndone\n";
    let file = write(&repo, "msg.sh", text);
    let run_args = ["rewrite", "run", "--script", &file, "--signatures", "strip"];

    let (code, value) = run(&repo, config.path(), session, &run_args);
    assert_eq!(code, 2, "{value}");
    assert_eq!(value["refused"]["reason"], "script_not_trusted");
    assert_eq!(repo.git(&["log", "--format=%s"]).lines().count(), 3);

    let (code, _) = run(
        &repo,
        config.path(),
        session,
        &["rewrite", "script", "trust", &file],
    );
    assert_eq!(code, 0);
    let (code, value) = run(&repo, config.path(), session, &run_args);
    assert_eq!(code, 0, "{value}");
    assert!(repo
        .git(&["log", "--format=%s"])
        .lines()
        .all(|s| s == "shell"));
}

// No test reads or writes the real config, data dir or credentials of whoever runs it.
gitpoulpi_testkit::isolate_binary!();
