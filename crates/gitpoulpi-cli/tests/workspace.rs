#![cfg(all(feature = "workspaces", feature = "global-search"))]
//! `gitpoulpi workspace ...`, `commit --workspace` and `gitpoulpi search`,
//! end to end through the real binary (the workspaces design).
//!
//! Every test has its own config directory, so none of them can see the
//! developer's workspaces, profiles or sessions. Asserts are on
//! deserialized JSON.

use std::path::Path;

use assert_cmd::Command;
use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use serde_json::Value;

fn gp(config: &Path, cwd: &Path, args: &[&str]) -> (i32, Value, String) {
    let output = Command::cargo_bin("gitpoulpi")
        .unwrap()
        .env("GITPOULPI_CONFIG_DIR", config)
        .env("GITPOULPI_DESKTOP_NO_HOOKS", "1")
        .env_remove("GITPOULPI_WORKSPACE")
        .arg("--json")
        .args(args)
        .current_dir(cwd)
        .output()
        .expect("failed to run gitpoulpi");
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let json = serde_json::from_str(&stdout).unwrap_or(Value::Null);
    (
        output.status.code().unwrap_or(-1),
        json,
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

fn ok(config: &Path, cwd: &Path, args: &[&str]) -> Value {
    let (code, json, stderr) = gp(config, cwd, args);
    assert_eq!(
        code,
        0,
        "`gitpoulpi {}` failed: {json} {stderr}",
        args.join(" ")
    );
    json
}

fn repo_with_origin(commit: &str) -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit(commit)
        .unwrap()
        .with_origin()
        .unwrap()
        .build()
}

/// The repository's path as the product reports it: canonical (macOS temp
/// dirs sit behind the `/var` -> `/private/var` link), no verbatim prefix.
fn path(repo: &FixtureRepo) -> String {
    gitpoulpi_core::paths::canonicalize_display(repo.root())
        .to_string_lossy()
        .into_owned()
}

#[test]
fn a_workspace_goes_through_its_whole_life() {
    let config = tempfile::tempdir().unwrap();
    let a = repo_with_origin("alpha");
    let b = repo_with_origin("beta");
    let cwd = tempfile::tempdir().unwrap();

    assert_eq!(
        ok(config.path(), cwd.path(), &["workspace", "list"])["workspaces"],
        serde_json::json!([])
    );

    // Create with one repository: its origin is recorded.
    let made = ok(
        config.path(),
        cwd.path(),
        &[
            "workspace",
            "create",
            "Acme Corp",
            "--add",
            &path(&a),
            "--profile",
            "work",
            "--color",
            "teal",
            "--description",
            "the customer",
        ],
    );
    let w = &made["workspace"];
    assert_eq!(w["id"], "acme-corp");
    assert_eq!(w["name"], "Acme Corp");
    assert_eq!(w["color"], "teal");
    assert_eq!(w["description"], "the customer");
    assert_eq!(w["profile"], "work");
    assert_eq!(w["repos"][0]["path"], path(&a).as_str());
    assert_eq!(w["repos"][0]["exists"], true);
    assert_eq!(
        w["repos"][0]["remote"],
        // git stores the url with forward slashes, whatever the OS.
        a.remote_path()
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/")
            .as_str()
    );
    // The profile id names nothing in this empty config: saved, flagged.
    assert_eq!(w["profile_exists"], false);

    // Two workspaces cannot share a name.
    let (code, json, _) = gp(
        config.path(),
        cwd.path(),
        &["workspace", "create", "acme corp"],
    );
    assert_ne!(code, 0);
    assert!(
        json["error"].as_str().unwrap().contains("acme-corp"),
        "{json}"
    );

    // Add: from anywhere inside the repository, recorded once.
    let added = ok(
        config.path(),
        cwd.path(),
        &[
            "workspace",
            "add",
            "acme-corp",
            &path(&b),
            &path(&b),
            &path(&a),
        ],
    );
    assert_eq!(added["added"].as_array().unwrap().len(), 1, "{added}");
    let repos = added["workspace"]["repos"].as_array().unwrap();
    assert_eq!(repos.len(), 2);

    // Something that is not a repository is refused and nothing is saved.
    let (code, json, _) = gp(
        config.path(),
        cwd.path(),
        &[
            "workspace",
            "add",
            "acme-corp",
            &cwd.path().to_string_lossy(),
        ],
    );
    assert_ne!(code, 0);
    assert!(json["error"].is_string(), "{json}");

    // Rename keeps the id; set-profile and --none.
    let renamed = ok(
        config.path(),
        cwd.path(),
        &["workspace", "rename", "acme-corp", "Acme"],
    );
    assert_eq!(renamed["workspace"]["id"], "acme-corp");
    assert_eq!(renamed["workspace"]["name"], "Acme");
    let cleared = ok(
        config.path(),
        cwd.path(),
        &["workspace", "set-profile", "acme-corp", "--none"],
    );
    assert!(cleared["workspace"]["profile"].is_null());
    let set = ok(
        config.path(),
        cwd.path(),
        &["workspace", "set-profile", "acme-corp", "home"],
    );
    assert_eq!(set["workspace"]["profile"], "home");

    // Show and list agree.
    let shown = ok(
        config.path(),
        cwd.path(),
        &["workspace", "show", "acme-corp"],
    );
    let listed = ok(config.path(), cwd.path(), &["workspace", "list"]);
    assert_eq!(listed["workspaces"][0], shown["workspace"]);

    // Remove one repository; the other stays.
    let removed = ok(
        config.path(),
        cwd.path(),
        &["workspace", "remove", "acme-corp", &path(&a)],
    );
    assert_eq!(removed["removed"].as_array().unwrap().len(), 1);
    assert_eq!(removed["workspace"]["repos"].as_array().unwrap().len(), 1);
    assert_eq!(removed["workspace"]["repos"][0]["path"], path(&b).as_str());

    // A repository gone from disk stays listed, marked missing, and can
    // still be removed by its path.
    let gone = tempfile::tempdir().unwrap();
    let doomed = gone.path().join("doomed");
    std::fs::create_dir_all(&doomed).unwrap();
    // Write the entry as the TUI would have: through the file.
    let file = config.path().join("workspaces.toml");
    let mut all = gitpoulpi_session::Workspaces::load(&file);
    all.get_mut("acme-corp")
        .unwrap()
        .add_repo(&doomed, Some("git@h:o/doomed.git"));
    all.save(&file).unwrap();
    let shown = ok(
        config.path(),
        cwd.path(),
        &["workspace", "show", "acme-corp"],
    );
    let entry = shown["workspace"]["repos"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["path"] == doomed.to_string_lossy().as_ref())
        .unwrap()
        .clone();
    assert_eq!(entry["exists"], true);
    drop(gone);
    let shown = ok(
        config.path(),
        cwd.path(),
        &["workspace", "show", "acme-corp"],
    );
    let entry = shown["workspace"]["repos"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["path"] == doomed.to_string_lossy().as_ref())
        .unwrap()
        .clone();
    assert_eq!(entry["exists"], false);
    assert_eq!(entry["remote"], "git@h:o/doomed.git");
    let removed = ok(
        config.path(),
        cwd.path(),
        &[
            "workspace",
            "remove",
            "acme-corp",
            &doomed.to_string_lossy(),
        ],
    );
    assert_eq!(removed["removed"].as_array().unwrap().len(), 1);

    // Delete leaves the repositories alone.
    let deleted = ok(
        config.path(),
        cwd.path(),
        &["workspace", "delete", "acme-corp"],
    );
    assert_eq!(deleted["deleted"], "acme-corp");
    assert!(a.root().join(".git").exists() && b.root().join(".git").exists());
    let (code, json, _) = gp(
        config.path(),
        cwd.path(),
        &["workspace", "show", "acme-corp"],
    );
    assert_ne!(code, 0);
    assert!(
        json["error"].as_str().unwrap().contains("no workspace"),
        "{json}"
    );
}

const PROFILES: &str = r#"
[general]
active_profile = "home"

[[profiles]]
id = "work"
name = "Work"
author_name = "Wendy Work"
author_email = "wendy@work.example"

[[profiles]]
id = "home"
name = "Home"
author_name = "Hank Home"
author_email = "hank@home.example"
"#;

/// A path that does not exist is refused with a message saying so, and the
/// workspace is not created; one that vanishes later stays listed as missing.
#[test]
fn a_missing_path_is_refused_and_a_vanished_repository_is_listed_missing() {
    let config = tempfile::tempdir().unwrap();
    let cwd = tempfile::tempdir().unwrap();
    let gone = cwd.path().join("gone");
    let (code, json, _) = gp(
        config.path(),
        cwd.path(),
        &[
            "workspace",
            "create",
            "Archive",
            "--add",
            &gone.to_string_lossy(),
        ],
    );
    assert_ne!(code, 0);
    assert!(
        json["error"].as_str().unwrap().contains("does not exist"),
        "{json}"
    );
    let list = ok(config.path(), cwd.path(), &["workspace", "list"]);
    assert_eq!(list["workspaces"], serde_json::json!([]));

    let a = repo_with_origin("alpha");
    ok(
        config.path(),
        cwd.path(),
        &["workspace", "create", "Archive", "--add", &path(&a)],
    );
    let kept = a.root().to_path_buf();
    drop(a);
    assert!(!kept.exists());
    let shown = ok(config.path(), cwd.path(), &["workspace", "show", "archive"]);
    assert_eq!(shown["workspace"]["repos"][0]["exists"], false, "{shown}");
}

fn commit_as(config: &Path, repo: &FixtureRepo, extra: &[&str], env: Option<&str>) -> String {
    std::fs::write(repo.root().join("change.txt"), format!("{extra:?}{env:?}")).unwrap();
    repo.git(&["add", "change.txt"]);
    let mut cmd = Command::cargo_bin("gitpoulpi").unwrap();
    cmd.env("GITPOULPI_CONFIG_DIR", config)
        .env("GITPOULPI_DESKTOP_NO_HOOKS", "1")
        .env_remove("GITPOULPI_WORKSPACE")
        .current_dir(repo.root())
        .args(["--json", "commit", "-m", "by workspace"])
        .args(extra);
    if let Some(id) = env {
        cmd.env("GITPOULPI_WORKSPACE", id);
    }
    let output = cmd.output().unwrap();
    assert!(
        output.status.success(),
        "{} {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    repo.git(&["log", "-1", "--format=%an <%ae>"])
}

/// `--workspace` / `GITPOULPI_WORKSPACE` make the commit use the workspace's
/// profile when the repository is in it; every other row of the table
/// (`Config::profile_for`) is covered by its unit tests, here only that the
/// CLI applies it.
#[test]
fn commit_uses_the_workspaces_profile_only_when_asked_and_only_inside_it() {
    let config = tempfile::tempdir().unwrap();
    std::fs::write(config.path().join("config.toml"), PROFILES).unwrap();
    let inside = repo_with_origin("inside");
    let outside = repo_with_origin("outside");
    let cwd = tempfile::tempdir().unwrap();
    ok(
        config.path(),
        cwd.path(),
        &[
            "workspace",
            "create",
            "Acme",
            "--add",
            &path(&inside),
            "--profile",
            "work",
        ],
    );

    // No flag, no variable: today's behavior, the repository's own identity.
    let plain = commit_as(config.path(), &inside, &[], None);
    assert!(
        !plain.contains("Wendy") && !plain.contains("Hank"),
        "{plain}"
    );
    // The flag, then the variable.
    assert_eq!(
        commit_as(config.path(), &inside, &["--workspace", "acme"], None),
        "Wendy Work <wendy@work.example>"
    );
    assert_eq!(
        commit_as(config.path(), &inside, &[], Some("acme")),
        "Wendy Work <wendy@work.example>"
    );
    // A repository outside the workspace gets `general.active_profile`.
    assert_eq!(
        commit_as(config.path(), &outside, &["--workspace", "acme"], None),
        "Hank Home <hank@home.example>"
    );
    // A workspace whose profile was deleted falls back the same way.
    ok(
        config.path(),
        cwd.path(),
        &["workspace", "set-profile", "acme", "deleted"],
    );
    assert_eq!(
        commit_as(config.path(), &inside, &["--workspace", "acme"], None),
        "Hank Home <hank@home.example>"
    );
    // An unknown workspace is an error, not a silent fallback.
    std::fs::write(inside.root().join("c2.txt"), "x").unwrap();
    inside.git(&["add", "c2.txt"]);
    let (code, json, _) = gp(
        config.path(),
        inside.root(),
        &["commit", "-m", "x", "--workspace", "nope"],
    );
    assert_ne!(code, 0);
    assert!(
        json["error"].as_str().unwrap().contains("no workspace"),
        "{json}"
    );
}

fn labels(json: &Value, kind: &str) -> Vec<String> {
    json["hits"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|h| h["kind"] == kind)
        .map(|h| h["label"].as_str().unwrap().to_string())
        .collect()
}

#[test]
fn search_finds_every_kind_in_the_current_repository() {
    let config = tempfile::tempdir().unwrap();
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("fix the parser")
        .unwrap()
        .build();
    repo.git(&["branch", "parser-rewrite"]);
    repo.git(&["tag", "parser-v1"]);
    let json = ok(
        config.path(),
        repo.root(),
        &["search", "parser", "--repo", &path(&repo)],
    );
    assert_eq!(json["partial"], false);
    assert!(json["workspace"].is_null());
    assert_eq!(labels(&json, "branch"), ["parser-rewrite"]);
    assert_eq!(labels(&json, "tag"), ["parser-v1"]);
    assert_eq!(labels(&json, "commit"), ["fix the parser"]);
    let commit = json["hits"]
        .as_array()
        .unwrap()
        .iter()
        .find(|h| h["kind"] == "commit")
        .unwrap();
    assert_eq!(commit["target"]["type"], "commit");
    assert_eq!(
        commit["target"]["id"],
        repo.git(&["rev-parse", "main"]).trim()
    );
    assert_eq!(
        commit["positions"],
        serde_json::json!([8, 9, 10, 11, 12, 13])
    );
    assert_eq!(commit["repo"], path(&repo).as_str());

    // A prefix narrows, `--kind` filters, `--limit` cuts.
    let only = ok(config.path(), repo.root(), &["search", "t: parser"]);
    assert!(only["hits"]
        .as_array()
        .unwrap()
        .iter()
        .all(|h| h["kind"] == "tag"));
    let kinds = ok(
        config.path(),
        repo.root(),
        &["search", "parser", "--kind", "branch", "--kind", "tag"],
    );
    assert!(kinds["hits"]
        .as_array()
        .unwrap()
        .iter()
        .all(|h| h["kind"] != "commit"));
    let one = ok(
        config.path(),
        repo.root(),
        &["search", "parser", "--limit", "1"],
    );
    assert_eq!(one["hits"].as_array().unwrap().len(), 1);
}

#[test]
fn a_workspace_search_fans_out_over_its_repositories_and_merges_by_score() {
    let config = tempfile::tempdir().unwrap();
    let api = repo_with_origin("api login handler");
    let web = repo_with_origin("web login page");
    let other = repo_with_origin("login in an unlisted repository");
    web.git(&["branch", "login"]);
    let cwd = tempfile::tempdir().unwrap();
    ok(
        config.path(),
        cwd.path(),
        &[
            "workspace",
            "create",
            "Acme",
            "--add",
            &path(&api),
            "--add",
            &path(&web),
        ],
    );
    // A repository of the workspace that is gone is reported, not fatal.
    let gone = tempfile::tempdir().unwrap();
    let file = config.path().join("workspaces.toml");
    let mut all = gitpoulpi_session::Workspaces::load(&file);
    all.get_mut("acme")
        .unwrap()
        .add_repo(&gone.path().join("vanished"), None);
    all.save(&file).unwrap();

    let json = ok(
        config.path(),
        cwd.path(),
        &["search", "login", "--workspace", "acme"],
    );
    assert_eq!(json["workspace"], "acme");
    let hits = json["hits"].as_array().unwrap();
    let repos: std::collections::BTreeSet<&str> =
        hits.iter().map(|h| h["repo"].as_str().unwrap()).collect();
    assert!(repos.contains(path(&api).as_str()) && repos.contains(path(&web).as_str()));
    assert!(!repos.contains(path(&other).as_str()));
    // Best first across repositories: the branch named exactly `login`
    // outranks the longer commit subjects.
    assert_eq!(hits[0]["kind"], "branch");
    assert_eq!(hits[0]["label"], "login");
    assert!(hits
        .windows(2)
        .all(|w| w[0]["score"].as_i64() >= w[1]["score"].as_i64()));
    let searched = json["repos"].as_array().unwrap();
    assert_eq!(searched.len(), 3);
    let missing = searched.iter().find(|r| r["searched"] == false).unwrap();
    assert!(missing["path"].as_str().unwrap().ends_with("vanished"));

    // The environment variable stands in for the flag; `r:` finds the
    // repositories themselves.
    let (code, by_env, _) = {
        let output = Command::cargo_bin("gitpoulpi")
            .unwrap()
            .env("GITPOULPI_CONFIG_DIR", config.path())
            .env("GITPOULPI_DESKTOP_NO_HOOKS", "1")
            .env("GITPOULPI_WORKSPACE", "acme")
            .current_dir(cwd.path())
            .args(["--json", "search", "r: vanish"])
            .output()
            .unwrap();
        (
            output.status.code().unwrap(),
            serde_json::from_slice::<Value>(&output.stdout).unwrap(),
            String::new(),
        )
    };
    assert_eq!(code, 0);
    let repo_hits = by_env["hits"].as_array().unwrap();
    assert_eq!(repo_hits.len(), 1, "{by_env}");
    assert_eq!(repo_hits[0]["kind"], "repo");
    assert!(repo_hits[0]["target"]["path"]
        .as_str()
        .unwrap()
        .ends_with("vanished"));

    // An unknown workspace is an error that names the known ones.
    let (code, json, _) = gp(
        config.path(),
        cwd.path(),
        &["search", "x", "--workspace", "nope"],
    );
    assert_ne!(code, 0);
    assert!(json["error"].as_str().unwrap().contains("acme"), "{json}");
}

/// Pull/merge requests and issues come from the repository's hosting cache
/// (a fixture file here, in the TUI's `.gitpoulpi/hosting-cache.json`), never
/// from a host; a corrupt cache is no hits.
#[test]
fn search_finds_pull_requests_and_issues_in_the_hosting_cache() {
    let config = tempfile::tempdir().unwrap();
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build();
    let cache_dir = repo.state_dir();
    std::fs::create_dir_all(&cache_dir).unwrap();
    let cache = cache_dir.join("hosting-cache.json");
    std::fs::write(
        &cache,
        serde_json::json!({
            "prs": [{"number": 12, "iid_label": "#12", "title": "Fix login redirect",
                     "author": "ana", "source_branch": "fix/login", "state": "open"}],
            "issues": [{"number": 7, "iid_label": "#7", "title": "Login page is slow",
                        "author": "cy", "state": "open"}],
        })
        .to_string(),
    )
    .unwrap();

    let json = ok(
        config.path(),
        repo.root(),
        &["search", "login", "--repo", &path(&repo)],
    );
    assert_eq!(labels(&json, "pull_request"), ["#12 Fix login redirect"]);
    assert_eq!(labels(&json, "issue"), ["#7 Login page is slow"]);
    let pr = json["hits"]
        .as_array()
        .unwrap()
        .iter()
        .find(|h| h["kind"] == "pull_request")
        .unwrap();
    assert_eq!(
        pr["target"],
        serde_json::json!({"type": "pull_request", "number": 12})
    );
    assert_eq!(pr["repo"], path(&repo).as_str());

    // `#` is both hosting kinds, `pr:` one; `--kind` filters too.
    let hash = ok(config.path(), repo.root(), &["search", "#7"]);
    assert_eq!(labels(&hash, "issue"), ["#7 Login page is slow"]);
    assert!(labels(&hash, "pull_request").is_empty());
    let prefixed = ok(config.path(), repo.root(), &["search", "pr: login"]);
    assert!(prefixed["hits"]
        .as_array()
        .unwrap()
        .iter()
        .all(|h| h["kind"] == "pull_request"));
    let by_flag = ok(
        config.path(),
        repo.root(),
        &["search", "login", "--kind", "issue"],
    );
    assert!(by_flag["hits"]
        .as_array()
        .unwrap()
        .iter()
        .all(|h| h["kind"] == "issue"));

    // Half-written cache: no hosting hits, no error.
    std::fs::write(&cache, "{\"prs\": [{\"number\":").unwrap();
    let corrupt = ok(config.path(), repo.root(), &["search", "#login"]);
    assert!(corrupt["hits"].as_array().unwrap().is_empty());
}

// No test reads or writes the real config, data dir or credentials of whoever runs it.
gitpoulpi_testkit::isolate_binary!();
