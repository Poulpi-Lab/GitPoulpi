//! Review without a clone: `hosting review ... --project P`, `hosting review
//! diff` and `hosting file`, against `xtask mock-host`. The commands run from
//! a directory that is not a repository (no daemon is involved), plus one
//! test inside a clone for `diff`. Assertions are on deserialized JSON and
//! on the requests the mock recorded; no network, own config dir per test.

use assert_cmd::Command;
use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use serde_json::Value;
use xtask::mock_host::{self, review, MockHost, Options};

struct Rig {
    config: tempfile::TempDir,
    /// Where the commands run: an empty directory, not a repository.
    cwd: tempfile::TempDir,
    mock: MockHost,
}

fn integration(id: &str, kind: &str, port: u16) -> String {
    format!(
        "[[integrations]]\nid = \"{id}\"\nkind = \"{kind}\"\nurl = \"http://127.0.0.1:{port}\"\ntoken_ref = \"integration:{id}\"\nname = \"{id}\"\naccount = \"mara\"\nuser = {{ login = \"mara\" }}\n"
    )
}

fn rig(kinds: &[(&str, &str)]) -> Rig {
    let mock = mock_host::start(Options::default()).expect("start the mock host");
    let config = tempfile::tempdir().unwrap();
    let text: String = kinds
        .iter()
        .map(|(id, kind)| integration(id, kind, mock.port))
        .collect();
    std::fs::write(config.path().join("config.toml"), text).unwrap();
    Rig {
        config,
        cwd: tempfile::tempdir().unwrap(),
        mock,
    }
}

impl Rig {
    fn run(&self, dir: &std::path::Path, args: &[&str]) -> (bool, Value) {
        let output = Command::cargo_bin("gitpoulpi")
            .unwrap()
            .env("GITPOULPI_CONFIG_DIR", self.config.path())
            .env("GITPOULPI_TOKEN_H", "test-token")
            .env("GITPOULPI_TOKEN_GH", "test-token")
            .env("GITPOULPI_TOKEN_GL", "test-token")
            .arg("--json")
            .args(args)
            .current_dir(dir)
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

    fn json(&self, args: &[&str]) -> Value {
        let (ok, json) = self.run(self.cwd.path(), args);
        assert!(ok, "`{}` failed: {json}", args.join(" "));
        json
    }

    fn error(&self, args: &[&str]) -> String {
        let (ok, json) = self.run(self.cwd.path(), args);
        assert!(!ok, "`{}` unexpectedly succeeded: {json}", args.join(" "));
        json["error"].as_str().expect("an error string").to_string()
    }

    fn targets(&self, method: &str) -> Vec<String> {
        self.mock
            .requests()
            .iter()
            .filter(|r| r.method == method)
            .map(|r| r.target.clone())
            .collect()
    }
}

fn paths(out: &Value) -> Vec<String> {
    out["files"]
        .as_array()
        .unwrap()
        .iter()
        .map(|f| f["diff"]["path"].as_str().unwrap().to_string())
        .collect()
}

#[test]
fn github_review_actions_run_against_a_project_with_no_repository() {
    let rig = rig(&[("h", "github_enterprise")]);
    let show = rig.json(&["hosting", "review", "show", "9", "--project", "owner/repo"]);
    assert_eq!(show["project"], "owner/repo");
    assert_eq!(show["source"], "project");
    assert_eq!(show["detail"]["head_sha"], review::HEAD_SHA);
    assert_eq!(show["files"].as_array().unwrap().len(), 2);

    let threads = rig.json(&[
        "hosting",
        "review",
        "threads",
        "9",
        "--project",
        "owner/repo",
    ]);
    assert_eq!(threads["threads"].as_array().unwrap().len(), 2);

    let comment = rig.json(&[
        "hosting",
        "review",
        "comment",
        "9",
        "--project",
        "owner/repo",
        "--body",
        "hello",
    ]);
    assert_eq!(comment["comment"]["body"], "hello");
    let resolve = rig.json(&[
        "hosting",
        "review",
        "resolve",
        "9",
        "PRRT_open",
        "--project",
        "owner/repo",
    ]);
    assert_eq!(resolve["resolved"], true);
    let submit = rig.json(&[
        "hosting",
        "review",
        "submit",
        "9",
        "--project",
        "owner/repo",
        "--event",
        "approve",
        "--body",
        "ship it",
    ]);
    assert_eq!(submit["event"], "approve");
    assert!(
        rig.targets("POST")
            .iter()
            .any(|t| t.contains("/repos/owner/repo/pulls/9/reviews")),
        "the review reached the host"
    );
}

#[test]
fn gitlab_review_reads_a_project_with_no_repository() {
    let rig = rig(&[("h", "gitlab_self_managed")]);
    let show = rig.json(&[
        "hosting",
        "review",
        "show",
        "58",
        "--project",
        "orbit-co/orbit",
    ]);
    assert_eq!(show["detail"]["number"], 58);
    assert_eq!(show["source"], "project");
    let threads = rig.json(&[
        "hosting",
        "review",
        "threads",
        "58",
        "--project",
        "orbit-co/orbit",
    ]);
    assert!(!threads["threads"].as_array().unwrap().is_empty());
}

#[test]
fn several_integrations_need_an_id_with_a_project_and_a_wrong_one_is_named() {
    let rig = rig(&[("gh", "github_enterprise"), ("gl", "gitlab_self_managed")]);
    let args = ["hosting", "review", "show", "9", "--project", "owner/repo"];
    let err = rig.error(&args);
    assert!(err.contains("pass --id"), "{err}");
    let mut with_id = args.to_vec();
    with_id.extend(["--id", "gh"]);
    assert_eq!(rig.json(&with_id)["integration"], "gh");
    let mut wrong = args.to_vec();
    wrong.extend(["--id", "nope"]);
    assert!(rig.error(&wrong).contains("no integration `nope`"));
}

#[test]
fn review_diff_reads_hunks_from_the_api_for_both_hosts() {
    for (kind, project, number) in [
        ("github_enterprise", "owner/repo", "9"),
        ("gitlab_self_managed", "orbit-co/orbit", "58"),
    ] {
        let rig = rig(&[("h", kind)]);
        let out = rig.json(&["hosting", "review", "diff", number, "--project", project]);
        assert_eq!(out["source"], "api", "{kind}");
        assert_eq!(paths(&out), ["src/planets.rs", "docs/moons.md"], "{kind}");
        let planets = &out["files"][0];
        assert_eq!(planets["diff"]["status"], "Modified");
        let hunks = planets["diff"]["hunks"].as_array().unwrap();
        let gitlab = kind.starts_with("gitlab");
        assert_eq!(hunks.len(), if gitlab { 1 } else { 2 }, "{kind}");
        assert_eq!(hunks[0]["new_start"], 10);
        let added: Vec<&str> = hunks[0]["lines"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|l| l["origin"] == "Add")
            .map(|l| l["content"].as_str().unwrap())
            .collect();
        if gitlab {
            assert_eq!(added, ["b", "c"]);
        } else {
            assert_eq!(
                added,
                ["    let b = 3;", "    let moons = 4;", "    let c = 5;"]
            );
            // The no-newline marker of the second hunk is not a line.
            assert_eq!(hunks[1]["lines"].as_array().unwrap().len(), 2);
        }
        assert_eq!(out["files"][1]["diff"]["status"], "Added");
        assert_eq!(out["files"][1]["omitted"], Value::Null);

        let one = rig.json(&[
            "hosting",
            "review",
            "diff",
            number,
            "--project",
            project,
            "--file",
            "docs/moons.md",
        ]);
        assert_eq!(paths(&one), ["docs/moons.md"]);
        let err = rig.error(&[
            "hosting",
            "review",
            "diff",
            number,
            "--project",
            project,
            "--file",
            "nope.rs",
        ]);
        assert!(err.contains("not among the files"), "{err}");
    }
}

#[test]
fn review_diff_in_a_clone_uses_git_and_api_flag_uses_the_host() {
    let rig = rig(&[("h", "github_enterprise")]);
    let repo: FixtureRepo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "a\n", "start")
        .unwrap()
        .with_origin()
        .unwrap()
        .push("main")
        .unwrap()
        .build();
    repo.git(&["checkout", "-q", "-b", "work"]);
    std::fs::write(repo.root().join("f.txt"), "f\n").unwrap();
    repo.git(&["add", "f.txt"]);
    repo.git(&["commit", "-q", "-m", "the request"]);
    repo.git(&["push", "-q", "origin", "work:refs/pull/9/head"]);
    repo.git(&["checkout", "-q", "main"]);
    repo.git(&["branch", "-D", "work"]);
    let run = |args: &[&str]| {
        let mut all = vec!["--session", "review-api-diff"];
        all.extend_from_slice(args);
        let (ok, json) = rig.run(repo.root(), &all);
        assert!(ok, "{json}");
        json
    };
    run(&[
        "hosting",
        "link",
        "pulls",
        "--id",
        "h",
        "--project",
        "owner/repo",
    ]);

    let local = run(&["hosting", "review", "diff", "9"]);
    assert_eq!(local["source"], "git");
    assert_eq!(paths(&local), ["f.txt"]);
    assert_eq!(local["files"][0]["additions"], 1);
    assert_eq!(local["files"][0]["diff"]["status"], "Added");

    let api = run(&["hosting", "review", "diff", "9", "--api"]);
    assert_eq!(api["source"], "api");
    assert_eq!(paths(&api), ["src/planets.rs", "docs/moons.md"]);
}

#[test]
fn file_reads_text_binary_and_missing_files_at_a_ref() {
    let rig = rig(&[("h", "github_enterprise")]);
    let claude = rig.json(&[
        "hosting",
        "file",
        "CLAUDE.md",
        "--project",
        "owner/repo",
        "--ref",
        "feature/moons",
    ]);
    assert_eq!(claude["encoding"], "utf-8");
    assert!(claude["content"]
        .as_str()
        .unwrap()
        .contains("Never use unwrap in library code."));
    assert_eq!(claude["ref"], "feature/moons");
    assert!(
        rig.targets("GET")
            .iter()
            .any(|t| t.contains("/repos/owner/repo/contents/CLAUDE.md?ref=feature%2Fmoons")),
        "{:?}",
        rig.targets("GET")
    );

    let png = rig.json(&["hosting", "file", "logo.png", "--project", "owner/repo"]);
    assert_eq!(png["encoding"], "base64");
    assert_eq!(png["content"], "iVBORw0KGgoA/w==");
    assert_eq!(png["size"], 10);

    let err = rig.error(&["hosting", "file", "docs/none.md", "--project", "owner/repo"]);
    assert!(err.contains("does not exist"), "{err}");
}

#[test]
fn gitlab_file_without_a_ref_reads_the_default_branch() {
    let rig = rig(&[("h", "gitlab_self_managed")]);
    let out = rig.json(&[
        "hosting",
        "file",
        "src/planets.rs",
        "--project",
        "orbit-co/orbit",
    ]);
    assert_eq!(out["encoding"], "utf-8");
    assert!(out["content"].as_str().unwrap().contains("Mars"));
    let gets = rig.targets("GET");
    assert!(
        gets.iter()
            .any(|t| t.ends_with("/projects/orbit-co%2Forbit")),
        "{gets:?}"
    );
    assert!(
        gets.iter()
            .any(|t| t
                .contains("/projects/orbit-co%2Forbit/repository/files/src%2Fplanets.rs?ref=main")),
        "{gets:?}"
    );
}

fn added(file: &Value) -> Vec<&str> {
    file["diff"]["hunks"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|h| h["lines"].as_array().unwrap())
        .filter(|l| l["origin"] == "Add")
        .map(|l| l["content"].as_str().unwrap())
        .collect()
}

#[test]
fn gitea_and_forgejo_review_a_project_through_the_api() {
    for kind in ["gitea", "forgejo"] {
        let rig = rig(&[("h", kind)]);
        let show = rig.json(&["hosting", "review", "show", "12", "--project", "owner/repo"]);
        assert_eq!(show["source"], "project", "{kind}");
        assert_eq!(show["files"].as_array().unwrap().len(), 2, "{kind}");

        let out = rig.json(&["hosting", "review", "diff", "12", "--project", "owner/repo"]);
        assert_eq!(out["source"], "api", "{kind}");
        assert_eq!(paths(&out), ["src/planets.rs", "docs/moons.md"], "{kind}");
        assert_eq!(
            out["files"][0]["diff"]["hunks"].as_array().unwrap().len(),
            2
        );
        assert_eq!(out["files"][0]["additions"], 4);
        assert_eq!(out["files"][1]["diff"]["status"], "Added");
        assert_eq!(added(&out["files"][1]), ["# Moons", "text"]);
        assert!(
            rig.targets("GET")
                .iter()
                .any(|t| t.ends_with("/repos/owner/repo/pulls/12.diff")),
            "{:?}",
            rig.targets("GET")
        );

        let one = rig.json(&[
            "hosting",
            "review",
            "diff",
            "12",
            "--project",
            "owner/repo",
            "--file",
            "docs/moons.md",
        ]);
        assert_eq!(paths(&one), ["docs/moons.md"]);

        let claude = rig.json(&[
            "hosting",
            "file",
            "CLAUDE.md",
            "--project",
            "owner/repo",
            "--ref",
            "feature/moons",
        ]);
        assert_eq!(claude["encoding"], "utf-8");
        assert!(claude["content"]
            .as_str()
            .unwrap()
            .contains("Never use unwrap in library code."));
        let png = rig.json(&["hosting", "file", "logo.png", "--project", "owner/repo"]);
        assert_eq!(png["encoding"], "base64");
        assert_eq!(png["content"], "iVBORw0KGgoA/w==");
        let err = rig.error(&["hosting", "file", "docs/none.md", "--project", "owner/repo"]);
        assert!(err.contains("does not exist"), "{err}");
    }
}

#[test]
fn gerrit_review_reads_a_project_with_no_repository() {
    let rig = rig(&[("h", "gerrit")]);
    let show = rig.json(&["hosting", "review", "show", "7", "--project", "sandbox"]);
    assert_eq!(show["source"], "project");
    assert_eq!(show["detail"]["number"], 7);

    let out = rig.json(&["hosting", "review", "diff", "7", "--project", "sandbox"]);
    assert_eq!(out["source"], "api");
    assert_eq!(paths(&out), ["docs/moons.md", "src/planets.rs"]);
    let planets = &out["files"][1];
    assert_eq!(planets["diff"]["status"], "Modified");
    assert_eq!(planets["additions"], 2);
    assert_eq!(planets["deletions"], 1);
    let hunk = &planets["diff"]["hunks"][0];
    assert_eq!(hunk["old_start"], 10);
    assert_eq!(hunk["new_start"], 10);
    assert_eq!(added(planets), ["    let b = 3;", "    let moons = 4;"]);
    assert_eq!(out["files"][0]["diff"]["status"], "Added");
    assert!(
        rig.targets("GET")
            .iter()
            .any(|t| t.contains("/revisions/current/files/src%2Fplanets.rs/diff?context=3")),
        "{:?}",
        rig.targets("GET")
    );

    let one = rig.json(&[
        "hosting",
        "review",
        "diff",
        "7",
        "--project",
        "sandbox",
        "--file",
        "docs/moons.md",
    ]);
    assert_eq!(paths(&one), ["docs/moons.md"]);

    let file = rig.json(&[
        "hosting",
        "file",
        "src/planets.rs",
        "--project",
        "sandbox",
        "--ref",
        "main",
    ]);
    assert_eq!(file["encoding"], "utf-8");
    assert!(file["content"].as_str().unwrap().contains("Mars"));
    assert!(
        rig.targets("GET")
            .iter()
            .any(|t| t.contains("/projects/sandbox/branches/main/files/src%2Fplanets.rs/content")),
        "{:?}",
        rig.targets("GET")
    );
    let png = rig.json(&["hosting", "file", "logo.png", "--project", "sandbox"]);
    assert_eq!(png["encoding"], "base64");
    let err = rig.error(&["hosting", "file", "docs/none.md", "--project", "sandbox"]);
    assert!(err.contains("does not exist"), "{err}");
}

/// `review suggest --project`: the diff comes from the API, no repository.
#[cfg(feature = "ai")]
#[test]
fn suggest_with_a_project_reads_the_diff_from_the_api_and_posts_nothing() {
    use gitpoulpi_testkit::fake_agent::{agent_calls, ai_config_toml, write_fake_agent};
    let rig = rig(&[("h", "github_enterprise")]);
    let tools = tempfile::tempdir().unwrap();
    let agent = write_fake_agent(
        tools.path(),
        &[
            r#"{"summary": "Moons added.", "suggestions": [{"path": "src/planets.rs", "line": 11, "severity": "warning", "body": "Why 3?"}]}"#,
        ],
    );
    let args = [
        "hosting",
        "review",
        "suggest",
        "9",
        "--project",
        "owner/repo",
    ];

    // AI is off by default: refused before anything is asked.
    let err = rig.error(&args);
    assert!(err.contains("turned off"), "{err}");
    assert_eq!(agent_calls(&agent), 0);

    let path = rig.config.path().join("config.toml");
    let mut config = std::fs::read_to_string(&path).unwrap();
    config.push_str(&ai_config_toml(&agent));
    std::fs::write(&path, config).unwrap();

    let out = rig.json(&args);
    assert_eq!(agent_calls(&agent), 1);
    assert_eq!(out["posted"], false);
    assert_eq!(out["diff_source"], "api");
    let found = out["suggestions"].as_array().unwrap();
    assert_eq!(found.len(), 1);
    assert_eq!(found[0]["anchored"], true, "{out}");
    assert_eq!(found[0]["line"], 11);
    assert!(
        rig.mock
            .requests()
            .iter()
            .all(|r| r.method == "GET" || r.target.contains("graphql")),
        "reading is all it did"
    );
}

/// The model is given the project's CLAUDE.md, read through the API at the
/// branch the request targets; the run's flags and links change that.
#[cfg(all(feature = "ai", unix))]
#[test]
fn suggest_gives_the_model_the_conventions_read_at_the_target_branch() {
    use gitpoulpi_testkit::fake_agent::{agent_prompt, ai_config_toml, write_capturing_fake_agent};
    let rig = rig(&[("h", "github_enterprise")]);
    let tools = tempfile::tempdir().unwrap();
    let agent = write_capturing_fake_agent(
        tools.path(),
        &[r#"{"summary": "Fine.", "suggestions": []}"#],
    );
    let path = rig.config.path().join("config.toml");
    let mut config = std::fs::read_to_string(&path).unwrap();
    config.push_str(&ai_config_toml(&agent));
    std::fs::write(&path, config).unwrap();
    let base = [
        "hosting",
        "review",
        "suggest",
        "9",
        "--project",
        "owner/repo",
    ];
    let with = |extra: &[&str]| -> Vec<String> {
        base.iter()
            .chain(extra.iter())
            .map(|s| s.to_string())
            .collect()
    };
    let run = |extra: &[&str]| {
        let args = with(extra);
        let refs: Vec<&str> = args.iter().map(String::as_str).collect();
        rig.json(&refs)
    };

    // The defaults: CLAUDE.md and AGENTS.md, at the request's target branch.
    let out = run(&[]);
    let prompt = agent_prompt(&agent);
    let block = prompt.find("Project conventions").expect(&prompt);
    assert!(block < prompt.find("The diff:").unwrap(), "{prompt}");
    assert!(prompt.contains("### CLAUDE.md"), "{prompt}");
    assert!(
        prompt.contains("Never use unwrap in library code."),
        "{prompt}"
    );
    assert_eq!(
        out["context"]["files"],
        serde_json::json!([{ "label": "CLAUDE.md", "cut": false }])
    );
    assert_eq!(out["context"]["agent_in_repository"], false);
    let gets = rig.targets("GET");
    assert!(
        gets.iter()
            .any(|t| t.contains("/contents/CLAUDE.md?ref=main")),
        "{gets:?}"
    );
    assert!(
        gets.iter()
            .any(|t| t.contains("/contents/AGENTS.md?ref=main")),
        "{gets:?}"
    );

    // Off for this run, plus an extra file that does not exist.
    let out = run(&[
        "--claude-md",
        "off",
        "--agents-md",
        "off",
        "--context-file",
        "docs/none.md",
    ]);
    assert!(!agent_prompt(&agent).contains("Project conventions"));
    assert_eq!(out["context"]["files"], serde_json::json!([]));
    assert_eq!(
        out["context"]["problems"],
        serde_json::json!(["docs/none.md: not found"])
    );
    assert_eq!(out["context"]["effective"]["include_claude_md"], false);

    // A link to a file of the host goes through the profile of that host.
    let url = format!(
        "http://127.0.0.1:{}/owner/repo/blob/main/CLAUDE.md",
        rig.mock.port
    );
    let before = rig.targets("GET").len();
    let out = run(&[
        "--claude-md",
        "off",
        "--agents-md",
        "off",
        "--context-file",
        &url,
    ]);
    assert_eq!(out["context"]["files"][0]["label"], url.as_str());
    assert!(agent_prompt(&agent).contains("Never use unwrap in library code."));
    let linked = rig.targets("GET").split_off(before);
    assert!(
        linked
            .iter()
            .any(|t| t.contains("/repos/owner/repo/contents/CLAUDE.md?ref=main")),
        "{linked:?}"
    );

    // The request's own flags beat the saved defaults.
    let saved = rig.json(&["ai", "review-context", "--claude-md", "off"]);
    assert_eq!(saved["effective"]["include_claude_md"], false);
    run(&[]);
    assert!(
        !agent_prompt(&agent).contains("Project conventions"),
        "global off"
    );
    run(&["--claude-md", "on"]);
    assert!(
        agent_prompt(&agent).contains("### CLAUDE.md"),
        "the run wins"
    );
}

/// A project that is not cloned has no checkout to run an agent in.
#[cfg(feature = "ai")]
#[test]
fn suggest_with_a_project_refuses_run_in_repo_yes_before_asking_anything() {
    use gitpoulpi_testkit::fake_agent::{agent_calls, ai_config_toml, write_fake_agent};
    let rig = rig(&[("h", "github_enterprise")]);
    let tools = tempfile::tempdir().unwrap();
    let agent = write_fake_agent(tools.path(), &[r#"{"summary": "x", "suggestions": []}"#]);
    let path = rig.config.path().join("config.toml");
    let mut config = std::fs::read_to_string(&path).unwrap();
    config.push_str(&ai_config_toml(&agent));
    std::fs::write(&path, config).unwrap();
    let err = rig.error(&[
        "hosting",
        "review",
        "suggest",
        "9",
        "--project",
        "owner/repo",
        "--run-in-repo",
        "yes",
    ]);
    assert!(err.contains("needs a clone"), "{err}");
    assert_eq!(agent_calls(&agent), 0);
    assert!(
        rig.mock.requests().is_empty(),
        "nothing was asked of the host"
    );
}

// No test reads or writes the real config, data dir or credentials of whoever runs it.
gitpoulpi_testkit::isolate_binary!();
