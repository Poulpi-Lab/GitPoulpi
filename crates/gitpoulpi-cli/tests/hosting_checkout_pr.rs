//! `hosting checkout-pr`: fetches a pull/merge request head (where a
//! fork's commits live) into a tracking local branch and switches to it.
//! No host is contacted, only the repository's own origin (a local bare
//! repository), so there is no mock server. Own config dir and fixture
//! per test.

use assert_cmd::Command;
use gitpoulpi_testkit::FixtureRepoBuilder;
use serde_json::Value;

fn run(config: &std::path::Path, session: &str, cwd: &std::path::Path, args: &[&str]) -> Value {
    let output = Command::cargo_bin("gitpoulpi")
        .unwrap()
        .env("GITPOULPI_CONFIG_DIR", config)
        .arg("--json")
        .args(["--session", session])
        .args(args)
        .current_dir(cwd)
        .output()
        .expect("run gitpoulpi");
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let json: Value = serde_json::from_str(&stdout).unwrap_or_else(|e| {
        panic!(
            "not JSON ({e}): {stdout} / {}",
            String::from_utf8_lossy(&output.stderr)
        )
    });
    assert!(
        output.status.success(),
        "`{}` failed: {json}",
        args.join(" ")
    );
    json
}

fn checkout_pr(kind: &str, remote_ref: &str, branch: &str, session: &str) {
    let config = tempfile::tempdir().unwrap();
    std::fs::write(
        config.path().join("config.toml"),
        format!(
            "[[integrations]]\nid = \"h\"\nkind = \"{kind}\"\nurl = \"http://127.0.0.1:9\"\ntoken_ref = \"integration:h\"\nname = \"h\"\n"
        ),
    )
    .unwrap();
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "a\n", "start")
        .unwrap()
        .with_origin()
        .unwrap()
        .push("main")
        .unwrap()
        .branch("fork-work")
        .unwrap()
        .checkout("fork-work")
        .unwrap()
        .commit_file("f.txt", "f\n", "from a fork")
        .unwrap()
        .checkout("main")
        .unwrap()
        .build();
    let tip = repo.git(&["rev-parse", "fork-work"]);
    // The fork's commits only exist on the host's special ref.
    repo.git(&["push", "-q", "origin", &format!("fork-work:{remote_ref}")]);
    repo.git(&["branch", "-D", "fork-work"]);
    repo.git(&["gc", "--prune=now", "-q"]);

    run(
        config.path(),
        session,
        repo.root(),
        &["hosting", "link", "pulls", "--id", "h", "--project", "g/p"],
    );
    let out = run(
        config.path(),
        session,
        repo.root(),
        &["hosting", "checkout-pr", "7"],
    );
    assert_eq!(out["branch"], branch);
    assert_eq!(out["ref"], remote_ref);
    assert_eq!(out["checked_out"], Value::Bool(true));
    assert_eq!(repo.git(&["rev-parse", "HEAD"]), tip);
    assert_eq!(repo.git(&["rev-parse", "--abbrev-ref", "HEAD"]), branch);
    assert_eq!(
        repo.git(&["config", &format!("branch.{branch}.remote")]),
        "origin"
    );
}

#[test]
fn a_github_fork_pull_request_is_checked_out_as_pr_n() {
    checkout_pr("github", "refs/pull/7/head", "pr/7", "checkout-pr-github");
}

#[test]
fn a_gitlab_merge_request_is_checked_out_as_mr_n() {
    checkout_pr(
        "gitlab_self_managed",
        "refs/merge-requests/7/head",
        "mr/7",
        "checkout-pr-gitlab",
    );
}

// No test reads or writes the real config, data dir or credentials of whoever runs it.
gitpoulpi_testkit::isolate_binary!();
