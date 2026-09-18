//! `gitpoulpi licenses`: the notices built into the binary. JSON assertions.

use assert_cmd::Command;
use serde_json::Value;

fn run(args: &[&str]) -> std::process::Output {
    let dir = tempfile::tempdir().unwrap();
    Command::cargo_bin("gitpoulpi")
        .unwrap()
        .env("GITPOULPI_CONFIG_DIR", dir.path())
        .current_dir(dir.path())
        .args(args)
        .output()
        .unwrap()
}

#[test]
fn licenses_prints_the_embedded_notices_as_json() {
    let out = run(&["--json", "licenses"]);
    assert!(out.status.success(), "{out:?}");
    let json: Value = serde_json::from_slice(&out.stdout).unwrap();
    let text = json["licenses"].as_str().unwrap();
    for needle in ["libgit2", "serde", "Mozilla Public License", "option-ext"] {
        assert!(text.contains(needle), "missing {needle}");
    }
    assert_eq!(text, gitpoulpi_core::licenses::THIRD_PARTY);
}

#[test]
fn licenses_prints_the_text_for_a_human() {
    let out = run(&["licenses"]);
    assert!(out.status.success());
    let text = String::from_utf8(out.stdout).unwrap();
    assert!(
        text.starts_with("GitPoulpi - third-party licenses"),
        "{text:.80}"
    );
}

// No test reads or writes the real config, data dir or credentials of whoever runs it.
gitpoulpi_testkit::isolate_binary!();
