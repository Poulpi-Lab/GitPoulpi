#![cfg(feature = "ai")]
//! `ai review-context`: the defaults of what a code review draft is given
//! besides the diff (global `[ai.review]`, a repository's own values, a
//! project's own values for a repository that is not cloned), and the
//! context layer `ai prompt --action code_review` prints.
//!
//! Own config directory per test; assertions on deserialized JSON.

use std::path::Path;

use assert_cmd::Command;
use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use serde_json::{json, Value};

fn gp(config: &Path, cwd: &Path, args: &[&str]) -> (i32, Value, String) {
    let output = Command::cargo_bin("gitpoulpi")
        .unwrap()
        .env("GITPOULPI_CONFIG_DIR", config)
        .env("GITPOULPI_DESKTOP_NO_HOOKS", "1")
        .arg("--json")
        .args(args)
        .current_dir(cwd)
        .output()
        .expect("failed to run gitpoulpi");
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    (
        output.status.code().unwrap_or(-1),
        serde_json::from_str(&stdout).unwrap_or(Value::Null),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

fn ok(config: &Path, cwd: &Path, args: &[&str]) -> Value {
    let (code, json, stderr) = gp(config, cwd, args);
    assert_eq!(code, 0, "`{}` failed: {json} {stderr}", args.join(" "));
    json
}

fn repo() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build()
}

#[test]
fn without_flags_it_shows_the_defaults_and_works_with_ai_off() {
    let config = tempfile::tempdir().unwrap();
    let cwd = tempfile::tempdir().unwrap();
    let out = ok(config.path(), cwd.path(), &["ai", "review-context"]);
    assert_eq!(out["scope"], "global");
    assert_eq!(out["changed"], false);
    assert_eq!(
        out["effective"],
        json!({
            "run_in_repository": "profile",
            "include_claude_md": true,
            "include_agents_md": true,
            "extra_file": "",
        })
    );
    assert!(out.get("own").is_none(), "{out}");
}

#[test]
fn the_global_defaults_are_set_in_config_toml_and_nothing_else_is_touched() {
    let config = tempfile::tempdir().unwrap();
    let cwd = tempfile::tempdir().unwrap();
    let c = config.path();
    std::fs::write(
        c.join("config.toml"),
        "[general]\ndefault_backend = \"git2\"\n",
    )
    .unwrap();
    let out = ok(
        c,
        cwd.path(),
        &[
            "ai",
            "review-context",
            "--run-in-repo",
            "yes",
            "--claude-md",
            "off",
            "--context-file",
            "docs/REVIEW.md",
        ],
    );
    assert_eq!(out["changed"], true);
    assert_eq!(out["effective"]["run_in_repository"], "yes");
    assert_eq!(out["effective"]["include_claude_md"], false);
    assert_eq!(out["effective"]["include_agents_md"], true);
    assert_eq!(out["effective"]["extra_file"], "docs/REVIEW.md");
    // It is the file that changed, read back by a second command.
    let again = ok(c, cwd.path(), &["ai", "review-context"]);
    assert_eq!(again["changed"], false);
    assert_eq!(again["global"], out["global"]);
    assert_eq!(again["global"]["extra_file"], "docs/REVIEW.md");
    let text = std::fs::read_to_string(c.join("config.toml")).unwrap();
    assert!(text.contains("default_backend"), "{text}");
    let toml: toml::Value = toml::from_str(&text).unwrap();
    assert_eq!(
        toml["ai"]["review"]["include_claude_md"].as_bool(),
        Some(false)
    );

    // --no-context-file clears the extra file.
    let out = ok(
        c,
        cwd.path(),
        &["ai", "review-context", "--no-context-file"],
    );
    assert_eq!(out["effective"]["extra_file"], "");
}

#[test]
fn a_repository_keeps_its_own_values_and_inherits_the_rest() {
    let config = tempfile::tempdir().unwrap();
    let r = repo();
    let root = r.root().to_string_lossy().into_owned();
    let c = config.path();
    ok(c, r.root(), &["ai", "review-context", "--agents-md", "off"]);

    let out = ok(
        c,
        r.root(),
        &[
            "ai",
            "review-context",
            "--repo",
            &root,
            "--claude-md",
            "off",
            "--context-file",
            "https://example.org/team.md",
        ],
    );
    assert_eq!(out["scope"], "repository");
    assert_eq!(
        out["own"],
        json!({ "include_claude_md": false, "extra_file": "https://example.org/team.md" })
    );
    // The repository's values over the global ones: AGENTS.md is the global off.
    assert_eq!(out["effective"]["include_claude_md"], false);
    assert_eq!(out["effective"]["include_agents_md"], false);
    assert_eq!(
        out["effective"]["extra_file"],
        "https://example.org/team.md"
    );
    // The global table did not move.
    assert_eq!(out["global"]["include_claude_md"], true);

    // --reset forgets the repository's own values.
    let out = ok(
        c,
        r.root(),
        &["ai", "review-context", "--repo", &root, "--reset"],
    );
    assert_eq!(out["own"], json!({}));
    assert_eq!(out["effective"]["include_claude_md"], true);
    assert_eq!(
        out["effective"]["include_agents_md"], false,
        "still the global off"
    );
}

#[test]
fn no_extra_file_turns_the_global_file_off_for_one_repository_or_project() {
    let config = tempfile::tempdir().unwrap();
    let r = repo();
    let root = r.root().to_string_lossy().into_owned();
    let c = config.path();
    ok(
        c,
        r.root(),
        &["ai", "review-context", "--context-file", "docs/global.md"],
    );
    // An empty repository entry follows the global file.
    let out = ok(c, r.root(), &["ai", "review-context", "--repo", &root]);
    assert_eq!(out["effective"]["extra_file"], "docs/global.md");

    let out = ok(
        c,
        r.root(),
        &["ai", "review-context", "--repo", &root, "--no-extra-file"],
    );
    assert_eq!(out["own"], json!({ "no_extra_file": true }));
    assert_eq!(out["effective"]["extra_file"], "");
    assert_eq!(out["global"]["extra_file"], "docs/global.md", "untouched");

    // `off` follows the global file again; a file replaces the switch.
    let out = ok(
        c,
        r.root(),
        &[
            "ai",
            "review-context",
            "--repo",
            &root,
            "--no-extra-file",
            "off",
        ],
    );
    assert_eq!(out["own"], json!({}));
    assert_eq!(out["effective"]["extra_file"], "docs/global.md");
    ok(
        c,
        r.root(),
        &["ai", "review-context", "--repo", &root, "--no-extra-file"],
    );
    let out = ok(
        c,
        r.root(),
        &[
            "ai",
            "review-context",
            "--repo",
            &root,
            "--context-file",
            "docs/own.md",
        ],
    );
    assert_eq!(out["own"], json!({ "extra_file": "docs/own.md" }));

    // A project that is not cloned keeps it in remote-reviews.toml.
    let out = ok(
        c,
        r.root(),
        &[
            "ai",
            "review-context",
            "--project",
            "github.com/acme/app",
            "--no-extra-file",
        ],
    );
    assert_eq!(out["own"], json!({ "no_extra_file": true }));
    assert_eq!(out["effective"]["extra_file"], "");
    let raw = std::fs::read_to_string(c.join("remote-reviews.toml")).unwrap();
    assert!(raw.contains("no_extra_file = true"), "{raw}");

    // Not for the global table, and not together with a file.
    let (code, _, _) = gp(c, r.root(), &["ai", "review-context", "--no-extra-file"]);
    assert_ne!(code, 0);
    let (code, _, _) = gp(
        c,
        r.root(),
        &[
            "ai",
            "review-context",
            "--repo",
            &root,
            "--no-extra-file",
            "--context-file",
            "x.md",
        ],
    );
    assert_ne!(code, 0);
}

#[test]
fn a_project_that_is_not_cloned_keeps_its_values_in_remote_reviews_toml() {
    let config = tempfile::tempdir().unwrap();
    let cwd = tempfile::tempdir().unwrap();
    let c = config.path();
    let out = ok(
        c,
        cwd.path(),
        &[
            "ai",
            "review-context",
            "--project",
            "GitHub.com/acme/app.git",
            "--agents-md",
            "off",
        ],
    );
    assert_eq!(out["scope"], "project");
    assert_eq!(out["target"], "github.com/acme/app");
    assert_eq!(out["own"], json!({ "include_agents_md": false }));
    let file = std::fs::read_to_string(c.join("remote-reviews.toml")).unwrap();
    let toml: toml::Value = toml::from_str(&file).unwrap();
    assert_eq!(
        toml["ai_review"]["github.com/acme/app"]["include_agents_md"].as_bool(),
        Some(false)
    );
    // Another project is untouched.
    let other = ok(
        c,
        cwd.path(),
        &["ai", "review-context", "--project", "github.com/acme/other"],
    );
    assert_eq!(other["own"], json!({}));
    assert_eq!(other["effective"]["include_agents_md"], true);
    // Resetting the last value drops the entry.
    ok(
        c,
        cwd.path(),
        &[
            "ai",
            "review-context",
            "--project",
            "github.com/acme/app",
            "--reset",
        ],
    );
    let file = std::fs::read_to_string(c.join("remote-reviews.toml")).unwrap();
    assert!(!file.contains("acme/app"), "{file}");
}

#[test]
fn wrong_combinations_are_refused_and_change_nothing() {
    let config = tempfile::tempdir().unwrap();
    let cwd = tempfile::tempdir().unwrap();
    let c = config.path();
    let r = repo();
    let root = r.root().to_string_lossy().into_owned();
    for args in [
        // No checkout for a repository that is not cloned.
        vec![
            "ai",
            "review-context",
            "--project",
            "github.com/a/b",
            "--run-in-repo",
            "yes",
        ],
        // Both targets.
        vec![
            "ai",
            "review-context",
            "--project",
            "github.com/a/b",
            "--repo",
            root.as_str(),
        ],
        // Not HOST/PROJECT.
        vec!["ai", "review-context", "--project", "justaname"],
        // --reset has nothing to fall back to globally.
        vec!["ai", "review-context", "--reset"],
        // Mutually exclusive.
        vec![
            "ai",
            "review-context",
            "--context-file",
            "a.md",
            "--no-context-file",
        ],
    ] {
        let (code, _, stderr) = gp(c, cwd.path(), &args);
        assert_ne!(code, 0, "`{}` should be refused ({stderr})", args.join(" "));
    }
    assert!(!c.join("config.toml").exists());
    assert!(!c.join("remote-reviews.toml").exists());
}

#[test]
fn ai_prompt_prints_the_context_layer_of_a_code_review() {
    let config = tempfile::tempdir().unwrap();
    let r = repo();
    let c = config.path();
    ok(
        c,
        r.root(),
        &[
            "ai",
            "review-context",
            "--claude-md",
            "off",
            "--context-file",
            "docs/REVIEW.md",
        ],
    );
    let out = ok(c, r.root(), &["ai", "prompt", "--action", "code_review"]);
    let text = out["review_context"]["text"].as_str().unwrap();
    assert!(text.contains("AGENTS.md, docs/REVIEW.md"), "{text}");
    assert!(!text.contains("CLAUDE.md"), "{text}");
    assert_eq!(out["review_context"]["include_claude_md"], false);
    // Another action has no such layer.
    let other = ok(c, r.root(), &["ai", "prompt", "--action", "commit_message"]);
    assert!(other.get("review_context").is_none());
    assert!(other["layers"]
        .as_array()
        .unwrap()
        .iter()
        .all(|l| l["level"] != "review_context"));
}

// No test reads or writes the real config, data dir or credentials of whoever runs it.
gitpoulpi_testkit::isolate_binary!();
