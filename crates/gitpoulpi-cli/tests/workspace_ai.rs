#![cfg(all(feature = "ai", feature = "workspaces"))]
//! Workspace AI instructions (slice R3): `workspace set-ai-instructions`,
//! `workspace show`, and `ai prompt`, which prints the layered system prompt
//! of an action (base, global, workspace, repository, task).
//!
//! Own config directory per test; assertions on deserialized JSON.

use std::path::Path;

use assert_cmd::Command;
use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use serde_json::Value;

fn gp(config: &Path, cwd: &Path, workspace: Option<&str>, args: &[&str]) -> (i32, Value, String) {
    let mut cmd = Command::cargo_bin("gitpoulpi").unwrap();
    cmd.env("GITPOULPI_CONFIG_DIR", config)
        .env("GITPOULPI_DESKTOP_NO_HOOKS", "1")
        .env_remove("GITPOULPI_WORKSPACE");
    if let Some(id) = workspace {
        cmd.env("GITPOULPI_WORKSPACE", id);
    }
    let output = cmd
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

fn ok(config: &Path, cwd: &Path, workspace: Option<&str>, args: &[&str]) -> Value {
    let (code, json, stderr) = gp(config, cwd, workspace, args);
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

fn path(repo: &FixtureRepo) -> String {
    repo.root().to_string_lossy().into_owned()
}

#[test]
fn workspace_ai_instructions_are_set_shown_and_cleared() {
    let config = tempfile::tempdir().unwrap();
    let cwd = tempfile::tempdir().unwrap();
    let c = config.path();
    ok(c, cwd.path(), None, &["workspace", "create", "Acme"]);

    let out = ok(
        c,
        cwd.path(),
        None,
        &[
            "workspace",
            "set-ai-instructions",
            "acme",
            "--text",
            "Use the Acme glossary.",
            "--mode",
            "replace",
        ],
    );
    let ai = &out["workspace"]["ai_instructions"];
    assert_eq!(ai["text"], "Use the Acme glossary.");
    assert_eq!(ai["mode"], "replace");

    let out = ok(
        c,
        cwd.path(),
        None,
        &[
            "workspace",
            "set-ai-instructions",
            "acme",
            "--action",
            "code_review",
            "--text",
            "Flag missing tests.",
        ],
    );
    let ai = &out["workspace"]["ai_instructions"];
    assert_eq!(
        ai["text"], "Use the Acme glossary.",
        "the general text stays"
    );
    assert_eq!(ai["actions"]["code_review"]["text"], "Flag missing tests.");
    assert_eq!(
        ai["actions"]["code_review"]["mode"], "append",
        "the default"
    );

    // It is in the file the TUI reads, under the workspace.
    let raw = std::fs::read_to_string(c.join("workspaces.toml")).unwrap();
    assert!(raw.contains("Flag missing tests."), "{raw}");

    // `show` and `list` print it.
    let shown = ok(c, cwd.path(), None, &["workspace", "show", "acme"]);
    assert_eq!(
        shown["workspace"]["ai_instructions"]["actions"]["code_review"]["text"],
        "Flag missing tests."
    );

    // Clearing one action leaves the general text; clearing that empties it.
    let out = ok(
        c,
        cwd.path(),
        None,
        &[
            "workspace",
            "set-ai-instructions",
            "acme",
            "--action",
            "code_review",
            "--clear",
        ],
    );
    let ai = &out["workspace"]["ai_instructions"];
    assert!(ai["actions"].as_object().unwrap().is_empty());
    assert_eq!(ai["text"], "Use the Acme glossary.");
    let out = ok(
        c,
        cwd.path(),
        None,
        &["workspace", "set-ai-instructions", "acme", "--clear"],
    );
    assert_eq!(out["workspace"]["ai_instructions"]["text"], "");
    let raw = std::fs::read_to_string(c.join("workspaces.toml")).unwrap();
    assert!(!raw.contains("ai_instructions"), "{raw}");
}

#[test]
fn a_bad_action_or_workspace_is_refused_before_anything_is_saved() {
    let config = tempfile::tempdir().unwrap();
    let cwd = tempfile::tempdir().unwrap();
    let c = config.path();
    ok(c, cwd.path(), None, &["workspace", "create", "Acme"]);
    let (code, _, stderr) = gp(
        c,
        cwd.path(),
        None,
        &[
            "workspace",
            "set-ai-instructions",
            "acme",
            "--action",
            "nope",
            "--text",
            "x",
        ],
    );
    assert_ne!(code, 0);
    assert!(
        stderr.contains("unknown AI action") && stderr.contains("code_review"),
        "{stderr}"
    );
    let (code, json, _) = gp(
        c,
        cwd.path(),
        None,
        &["workspace", "set-ai-instructions", "ghost", "--text", "x"],
    );
    assert_ne!(code, 0);
    assert!(
        json["error"].as_str().unwrap().contains("no workspace"),
        "{json}"
    );
    // A text or --clear is required.
    let (code, _, _) = gp(
        c,
        cwd.path(),
        None,
        &["workspace", "set-ai-instructions", "acme"],
    );
    assert_ne!(code, 0);
}

fn layers(out: &Value) -> Vec<(String, String)> {
    out["layers"]
        .as_array()
        .unwrap()
        .iter()
        .map(|l| {
            (
                l["level"].as_str().unwrap().to_string(),
                l["text"].as_str().unwrap().to_string(),
            )
        })
        .collect()
}

#[test]
fn ai_prompt_prints_the_layers_in_order_and_follows_the_workspace_rules() {
    let config = tempfile::tempdir().unwrap();
    let r = repo();
    let root = path(&r);
    let c = config.path();
    // AI stays OFF: the prompt is still printable, and says so.
    std::fs::write(
        c.join("config.toml"),
        "[ai]\nglobal_instructions = \"GLOBAL RULE\"\n\n[ai.actions.code_review]\ninstructions = \"GLOBAL REVIEW RULE\"\n",
    )
    .unwrap();
    // Two workspaces hold the repository; the first one wins by default.
    for name in ["Alpha", "Beta"] {
        ok(
            c,
            r.root(),
            None,
            &["workspace", "create", name, "--add", &root],
        );
    }
    ok(
        c,
        r.root(),
        None,
        &[
            "workspace",
            "set-ai-instructions",
            "alpha",
            "--text",
            "ALPHA RULE",
        ],
    );
    ok(
        c,
        r.root(),
        None,
        &[
            "workspace",
            "set-ai-instructions",
            "beta",
            "--text",
            "BETA RULE",
            "--mode",
            "replace",
        ],
    );
    ok(
        c,
        r.root(),
        None,
        &[
            "workspace",
            "set-ai-instructions",
            "beta",
            "--action",
            "code_review",
            "--text",
            "BETA REVIEW RULE",
        ],
    );
    let args = [
        "--session",
        "ws-ai-prompt",
        "--repo",
        root.as_str(),
        "ai",
        "prompt",
        "--action",
        "code_review",
    ];

    // No active workspace: the first containing it (Alpha), appended.
    let out = ok(c, r.root(), None, &args);
    assert_eq!(out["action"], "code_review");
    assert_eq!(out["ai_enabled"], false);
    let l = layers(&out);
    let levels: Vec<&str> = l.iter().map(|(k, _)| k.as_str()).collect();
    assert_eq!(levels, ["base", "global", "workspace", "task"], "{l:?}");
    assert!(l[0].1.contains("JSON"), "the base prompt fixes the format");
    assert_eq!(l[1].1, "GLOBAL RULE");
    assert_eq!(l[2].1, "ALPHA RULE");
    assert_eq!(l[3].1, "GLOBAL REVIEW RULE");
    let system = out["system_prompt"].as_str().unwrap();
    let at = |t: &str| system.find(t).unwrap();
    assert!(at("GLOBAL RULE") < at("ALPHA RULE") && at("ALPHA RULE") < at("GLOBAL REVIEW RULE"));

    // The active workspace (the env var) wins when it holds the repository;
    // Beta replaces the global text, and its review rule is appended.
    let out = ok(c, r.root(), Some("beta"), &args);
    let l = layers(&out);
    let levels: Vec<&str> = l.iter().map(|(k, _)| k.as_str()).collect();
    assert_eq!(levels, ["base", "workspace", "task"], "{l:?}");
    assert_eq!(l[1].1, "BETA RULE");
    assert_eq!(l[2].1, "GLOBAL REVIEW RULE\n\nBETA REVIEW RULE");
    assert!(!out["system_prompt"]
        .as_str()
        .unwrap()
        .contains("GLOBAL RULE"));

    // The repository's own instructions come last and may replace the rest.
    let state = r.state_dir();
    std::fs::create_dir_all(&state).unwrap();
    std::fs::write(
        state.join("repo-state.toml"),
        "[preferences.ai_instructions]\ntext = \"REPO RULE\"\nmode = \"replace\"\n",
    )
    .unwrap();
    let out = ok(c, r.root(), Some("beta"), &args);
    let l = layers(&out);
    let levels: Vec<&str> = l.iter().map(|(k, _)| k.as_str()).collect();
    assert_eq!(levels, ["base", "repository", "task"], "{l:?}");
    assert_eq!(l[1].1, "REPO RULE");

    // An unknown action is refused and names the known ones.
    let mut bad = args;
    bad[7] = "nope";
    let (code, json, _) = gp(c, r.root(), None, &bad);
    assert_ne!(code, 0);
    assert!(
        json["error"].as_str().unwrap().contains("code_review"),
        "{json}"
    );
}

// No test reads or writes the real config, data dir or credentials of whoever runs it.
gitpoulpi_testkit::isolate_binary!();
