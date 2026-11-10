//! `gitpoulpi repo-auth show|set|clear|test` end to end: the real binary, a
//! real fixture repository and its local bare remote (no network). The
//! assertions are on the deserialized JSON.

use std::path::Path;

use assert_cmd::Command;
use gitpoulpi_core::domain::{HttpsAuth, RepoAuthProfile};
use gitpoulpi_testkit::FixtureRepoBuilder;
use serde_json::Value;

gitpoulpi_testkit::isolate_binary!();

/// Runs `gitpoulpi --json ...` and returns the exit code and the payload.
fn run(root: &Path, session: &str, args: &[&str]) -> (i32, Value) {
    // Each fixture gets its own config (and so its own private repo-auth
    // store): tests never share or touch the real one.
    let config = root
        .parent()
        .expect("fixture root has a parent")
        .join("config");
    let output = Command::cargo_bin("gitpoulpi")
        .unwrap()
        .env("GITPOULPI_CONFIG_DIR", &config)
        .arg("--json")
        .args(["--session", session])
        .args(["--repo", &root.to_string_lossy()])
        .args(args)
        .current_dir(root)
        .output()
        .expect("failed to run gitpoulpi");
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let json = serde_json::from_str(&stdout).unwrap_or_else(|e| {
        panic!(
            "output was not JSON ({e}): {stdout}\nstderr: {}",
            String::from_utf8_lossy(&output.stderr)
        )
    });
    (output.status.code().unwrap_or(-1), json)
}

fn ok(root: &Path, session: &str, args: &[&str]) -> Value {
    let (code, json) = run(root, session, args);
    assert_eq!(code, 0, "`gitpoulpi {}` failed: {json}", args.join(" "));
    json
}

#[test]
fn show_without_a_profile_names_the_identity_of_each_remote() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .with_origin()
        .unwrap()
        .build();
    let shown = ok(repo.root(), "ra-show-default", &["repo-auth", "show"]);
    assert_eq!(shown["configured"], false);
    assert!(shown["summary"].is_null());
    let remotes = shown["remotes"].as_array().unwrap();
    assert_eq!(remotes.len(), 1);
    assert_eq!(remotes[0]["remote"], "origin");
    assert_eq!(remotes[0]["identity"], "no authentication");
}

#[test]
fn set_system_key_and_integration_round_trip_then_clear() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .with_origin()
        .unwrap()
        .build();
    let root = repo.root();
    let session = "ra-round-trip";

    let set = ok(
        root,
        session,
        &[
            "repo-auth",
            "set",
            "--https",
            "system",
            "--username",
            "alice",
            "--ignore-repo-helpers",
        ],
    );
    assert_eq!(set["set"], true);
    let https = &set["view"]["profile"]["https"];
    assert_eq!(https["source"], "system");
    assert_eq!(https["username"], "alice");
    assert_eq!(https["ignore_repo_helpers"], true);

    // A second protocol keeps the first.
    let set = ok(
        root,
        session,
        &[
            "repo-auth",
            "set",
            "--ssh",
            "key:/keys/work_ed25519",
            "--strict",
        ],
    );
    let profile = &set["view"]["profile"];
    assert_eq!(profile["https"]["username"], "alice");
    assert_eq!(profile["ssh"]["source"], "key_file");
    assert_eq!(profile["ssh"]["key"], "/keys/work_ed25519");
    assert_eq!(profile["ssh"]["strict_host_key"], true);

    let shown = ok(root, session, &["repo-auth", "show"]);
    assert_eq!(shown["configured"], true);
    assert_eq!(shown["profile"], set["view"]["profile"]);
    assert!(shown["summary"].as_str().unwrap().contains("work_ed25519"));

    let set = ok(
        root,
        session,
        &[
            "repo-auth",
            "set",
            "--https",
            "integration:github-corp",
            "--username",
            "bob",
            "--ssh",
            "integration-key:github-corp",
        ],
    );
    let profile = &set["view"]["profile"];
    assert_eq!(profile["https"]["source"], "integration");
    assert_eq!(profile["https"]["integration"], "github-corp");
    assert_eq!(profile["ssh"]["source"], "integration_key");

    let cleared = ok(root, session, &["repo-auth", "clear"]);
    assert_eq!(cleared["cleared"], true);
    let shown = ok(root, session, &["repo-auth", "show"]);
    assert_eq!(shown["configured"], false);
    let cleared = ok(root, session, &["repo-auth", "clear"]);
    assert_eq!(cleared["cleared"], false);
}

#[test]
fn a_confirmed_command_source_is_stored_with_its_hash() {
    for args in [
        &[
            "repo-auth",
            "set",
            "--https",
            "helper",
            "--helper",
            "store",
            "--yes",
        ][..],
        &[
            "repo-auth",
            "set",
            "--https",
            "token-command",
            "--hosts",
            "github.com",
            "--yes",
            "--command",
            "pass",
            "show",
            "token",
        ][..],
    ] {
        let repo = FixtureRepoBuilder::new()
            .unwrap()
            .commit("initial")
            .unwrap()
            .build();
        let root = repo.root();
        let session = "ra-command";
        let (code, json) = run(root, session, args);
        assert_eq!(code, 0, "{args:?}: {json}");
        assert_eq!(json["set"], true, "{json}");
        let https = &json["view"]["profile"]["https"];
        let sha = https["confirmed_sha256"].as_str().expect("confirmed hash");
        assert_eq!(https["command"]["sha256"].as_str(), Some(sha), "{json}");
        let shown = ok(root, session, &["repo-auth", "show"]);
        assert_eq!(shown["configured"], true);
    }
}

#[test]
fn bad_combinations_are_failures_not_refusals() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build();
    let root = repo.root();
    for args in [
        &["repo-auth", "set"][..],
        &["repo-auth", "set", "--https", "bogus"][..],
        &["repo-auth", "set", "--https", "helper", "--yes"][..],
        &["repo-auth", "set", "--ssh", "system", "--strict"][..],
        &["repo-auth", "set", "--username", "x"][..],
    ] {
        let output = Command::cargo_bin("gitpoulpi")
            .unwrap()
            .arg("--json")
            .args(["--session", "ra-bad"])
            .args(["--repo", &root.to_string_lossy()])
            .args(args)
            .current_dir(root)
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(1), "{args:?}");
    }
}

#[test]
fn show_hides_a_stored_command_from_its_secret_but_a_person_may_read_it() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build();
    let root = repo.root();
    // Only a later version writes a command source; write one as the file
    // would hold it.
    let helper = "!f() { echo password=s3cret-token; }; f";
    let profile = RepoAuthProfile {
        https: Some(HttpsAuth::Helper {
            helper: helper.into(),
            username: None,
            hosts: vec!["github.com".into()],
            confirmed_sha256: None,
            script_sha256: None,
            confirmed_at: None,
        }),
        ssh: None,
    };
    let sha = profile.https.as_ref().unwrap().command_sha256().unwrap();
    let paths = gitpoulpi_config::ConfigPaths::for_dir(root.parent().unwrap().join("config"));
    gitpoulpi_session::save_repo_auth(&paths, root, &profile, 1).unwrap();

    let shown = ok(root, "ra-command", &["repo-auth", "show"]);
    let https = &shown["profile"]["https"];
    assert_eq!(https["source"], "helper");
    assert!(https.get("helper").is_none(), "{https}");
    // The CLI is a person: it may read the command it stored.
    assert_eq!(https["command"]["set"], true);
    assert_eq!(https["command"]["sha256"], sha.as_str());
    assert_eq!(https["command"]["helper"], helper);
    // The daemon refuses transfers with a source it cannot use yet.
    assert!(shown["unavailable"].is_string());
}

#[test]
fn test_reports_each_remote_with_its_identity() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .with_origin()
        .unwrap()
        .build();
    let root = repo.root();
    let session = "ra-test";

    let result = ok(root, session, &["repo-auth", "test"]);
    assert_eq!(result["ok"], true, "{result}");
    let reports = result["reports"].as_array().unwrap();
    assert_eq!(reports.len(), 1);
    assert_eq!(reports[0]["remote"], "origin");
    assert_eq!(reports[0]["ok"], true);
    assert_eq!(reports[0]["push"], false);
    assert!(reports[0]["ref_count"].is_u64());
    assert!(reports[0]["duration_ms"].is_u64());
    let identity = result["identity"].as_array().unwrap();
    assert_eq!(identity[0]["remote"], "origin");
    assert_eq!(identity[0]["identity"], "no authentication");

    let pushed = ok(
        root,
        session,
        &["repo-auth", "test", "--remote", "origin", "--push"],
    );
    assert_eq!(pushed["reports"][0]["push"], true);

    // An https remote names the profile's identity (`show` makes no
    // connection).
    ok(
        root,
        session,
        &["remote", "add", "web", "https://example.invalid/team/x.git"],
    );
    ok(
        root,
        session,
        &[
            "repo-auth",
            "set",
            "--https",
            "system",
            "--username",
            "alice",
        ],
    );
    let shown = ok(root, session, &["repo-auth", "show"]);
    let web = shown["remotes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["remote"] == "web")
        .unwrap();
    assert_eq!(web["identity"], "system git (user alice)");
    assert_eq!(web["transport"], "https");
}

/// The argv of a token command that prints `token`, in the platform's
/// scripting language (`sh` on Unix, `pwsh` on Windows).
fn token_script_argv(dir: &Path, token: &str) -> Vec<String> {
    let path = dir.join(if cfg!(windows) { "token.ps1" } else { "token.sh" });
    if cfg!(windows) {
        std::fs::write(&path, format!("Write-Output '{token}'\n")).unwrap();
        ["pwsh", "-NoProfile", "-File"]
            .into_iter()
            .map(String::from)
            .chain([path.to_string_lossy().into_owned()])
            .collect()
    } else {
        std::fs::write(&path, format!("echo '{token}'\n")).unwrap();
        vec!["sh".into(), path.to_string_lossy().into_owned()]
    }
}

/// A grant-based source works from a standalone CLI run: no TUI, no MCP and
/// no published daemon for the session, so the credential helper git runs
/// must reach the embedded daemon of this very process (ADR-0010, decision
/// 21). Loopback only.
#[test]
fn a_token_command_works_from_a_standalone_cli_run() {
    if cfg!(windows)
        && std::process::Command::new("pwsh")
            .args(["-NoProfile", "-Command", "exit 0"])
            .output()
            .is_err()
    {
        eprintln!("SKIPPED: pwsh is not installed");
        return;
    }
    const TOKEN: &str = "plainsecret-0123456789-abcdef";
    let server = gitpoulpi_testkit::AuthHttpRemote::start_accepting();
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .with_origin()
        .unwrap()
        .build();
    let root = repo.root();
    repo.git(&["remote", "set-url", "origin", &server.url()]);
    let tools = tempfile::tempdir().unwrap();
    let argv = token_script_argv(tools.path(), TOKEN);
    let session = "ra-standalone-token-command";

    let mut set = vec!["repo-auth", "set", "--https", "token-command", "--yes", "--command"];
    set.extend(argv.iter().map(String::as_str));
    ok(root, session, &set);

    let tested = ok(root, session, &["repo-auth", "test"]);
    let remotes = tested["remotes"].as_array().expect("remotes");
    assert!(
        remotes.iter().all(|r| r["ok"] == true),
        "the connection test must pass: {tested}"
    );
    let fetched = ok(root, session, &["fetch"]);
    assert!(fetched.is_object() || fetched.is_array(), "{fetched}");
    let seen = server.basic_credentials();
    assert!(
        seen.iter().any(|c| c == &format!("x-access-token:{TOKEN}")),
        "the server never got the token: {seen:?}"
    );
}
