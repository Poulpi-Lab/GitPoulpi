//! `hosting review show|threads|comment|reply|resolve|submit|fetch` against
//! `xtask mock-host`'s review endpoints (one pull request
//! per side). Assertions are on deserialized JSON and on the bodies the
//! mock recorded; no network, own config dir and fixture per test.

use assert_cmd::Command;
use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use serde_json::Value;
use xtask::mock_host::{self, review, MockHost, Options};

struct Rig {
    config: tempfile::TempDir,
    repo: FixtureRepo,
    mock: MockHost,
    session: String,
}

fn rig(kind: &str, project: &str, session: &str) -> Rig {
    let mock = mock_host::start(Options::default()).expect("start the mock host");
    let config = tempfile::tempdir().unwrap();
    std::fs::write(
        config.path().join("config.toml"),
        format!(
            "[[integrations]]\nid = \"h\"\nkind = \"{kind}\"\nurl = \"http://127.0.0.1:{}\"\ntoken_ref = \"integration:h\"\nname = \"h\"\nuser = {{ login = \"mara\" }}\n",
            mock.port
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
        .build();
    let rig = Rig {
        config,
        repo,
        mock,
        session: session.to_string(),
    };
    rig.json(&[
        "hosting",
        "link",
        "pulls",
        "--id",
        "h",
        "--project",
        project,
    ]);
    rig
}

impl Rig {
    fn output(&self, args: &[&str]) -> (bool, Value) {
        let output = Command::cargo_bin("gitpoulpi")
            .unwrap()
            .env("GITPOULPI_CONFIG_DIR", self.config.path())
            .env("GITPOULPI_TOKEN_H", "test-token")
            .arg("--json")
            .args(["--session", &self.session])
            .args(args)
            .current_dir(self.repo.root())
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
        let (ok, json) = self.output(args);
        assert!(ok, "`{}` failed: {json}", args.join(" "));
        json
    }

    fn error(&self, args: &[&str]) -> String {
        let (ok, json) = self.output(args);
        assert!(!ok, "`{}` unexpectedly succeeded: {json}", args.join(" "));
        json["error"].as_str().expect("an error string").to_string()
    }

    /// Bodies of the requests with this method whose target contains
    /// `needle`, as JSON.
    fn bodies(&self, method: &str, needle: &str) -> Vec<Value> {
        self.mock
            .requests()
            .iter()
            .filter(|r| r.method == method && r.target.contains(needle))
            .map(|r| serde_json::from_str(&r.body).unwrap_or(Value::Null))
            .collect()
    }

    fn posted(&self, method: &str, needle: &str) -> usize {
        self.bodies(method, needle).len()
    }
}

#[test]
fn github_show_lists_the_files_the_capabilities_and_the_local_refs() {
    let rig = rig("github_enterprise", "owner/repo", "review-gh-show");
    let out = rig.json(&["hosting", "review", "show", "9"]);
    assert_eq!(out["project"], "owner/repo");
    let detail = &out["detail"];
    assert_eq!(detail["number"], 9);
    assert_eq!(detail["head_sha"], review::HEAD_SHA);
    assert_eq!(detail["source_branch"], "feature/moons");
    assert_eq!(detail["checks"], "passing");
    assert_eq!(detail["merge_state"], "clean");
    assert_eq!(detail["capabilities"]["request_changes"], true);
    let files = out["files"].as_array().unwrap();
    assert_eq!(files.len(), 2);
    assert_eq!(files[0]["path"], "src/planets.rs");
    assert_eq!(files[0]["additions"], 14);
    assert_eq!(files[1]["status"], "added");
    assert_eq!(out["head_ref"], "refs/gitpoulpi/pr/9");
}

#[test]
fn github_threads_carry_ids_positions_and_resolution() {
    let rig = rig("github_enterprise", "owner/repo", "review-gh-threads");
    let out = rig.json(&["hosting", "review", "threads", "9"]);
    let threads = out["threads"].as_array().unwrap();
    assert_eq!(threads.len(), 2);
    assert_eq!(threads[0]["id"], "PRRT_open");
    assert_eq!(threads[0]["path"], "src/planets.rs");
    assert_eq!(threads[0]["line"], 12);
    assert_eq!(threads[0]["side"], "new");
    assert_eq!(threads[0]["resolved"], false);
    assert_eq!(threads[0]["comments"].as_array().unwrap().len(), 2);
    assert_eq!(threads[1]["resolved"], true);
    assert_eq!(out["conversation"].as_array().unwrap().len(), 2);
    // The verdicts already given ride along (GitHub's `pulls/9/reviews`).
    let reviews = out["reviews"].as_array().unwrap();
    assert_eq!(reviews.len(), 2);
    assert_eq!(reviews[0]["author"], "mara-quinn");
    assert_eq!(reviews[0]["state"], "approved");
    assert_eq!(reviews[1]["state"], "changes_requested");

    let open = rig.json(&["hosting", "review", "threads", "9", "--unresolved"]);
    let ids: Vec<_> = open["threads"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["id"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(ids, ["PRRT_open"]);
}

#[test]
fn github_comment_reply_and_resolve_post_what_they_say() {
    let rig = rig("github_enterprise", "owner/repo", "review-gh-write");
    let posted = rig.json(&["hosting", "review", "comment", "9", "--body", "Thanks all"]);
    assert_eq!(posted["comment"]["body"], "Thanks all");
    assert_eq!(
        rig.bodies("POST", "/issues/9/comments")[0]["body"],
        "Thanks all"
    );

    let reply = rig.json(&[
        "hosting",
        "review",
        "reply",
        "9",
        "PRRT_open",
        "--body",
        "Done",
    ]);
    assert_eq!(reply["thread"], "PRRT_open");
    let graphql = rig.bodies("POST", "/api/graphql");
    let last = graphql.last().unwrap();
    assert_eq!(last["variables"]["thread"], "PRRT_open");
    assert_eq!(last["variables"]["body"], "Done");

    assert_eq!(
        rig.json(&["hosting", "review", "resolve", "9", "PRRT_open"])["resolved"],
        true
    );
    assert_eq!(
        rig.json(&[
            "hosting",
            "review",
            "resolve",
            "9",
            "PRRT_open",
            "--unresolve"
        ])["resolved"],
        false
    );
    let queries: Vec<String> = rig
        .bodies("POST", "/api/graphql")
        .iter()
        .filter_map(|b| b["query"].as_str().map(str::to_string))
        .collect();
    assert!(queries.iter().any(|q| q.contains("unresolveReviewThread")));

    let err = rig.error(&["hosting", "review", "comment", "9", "--body", "  "]);
    assert!(err.contains("empty"), "{err}");
}

#[test]
fn github_review_submits_one_request_with_sides_and_the_head() {
    let rig = rig("github_enterprise", "owner/repo", "review-gh-review");
    let out = rig.json(&[
        "hosting",
        "review",
        "submit",
        "9",
        "--event",
        "request-changes",
        "--body",
        "Needs work",
        "--comment",
        "src/planets.rs:12-14:new:Extract this: it repeats",
        "--comment",
        "src/planets.rs:3:old:Why remove?",
    ]);
    assert_eq!(out["event"], "request-changes");
    assert_eq!(out["comments"], 2);
    assert_eq!(out["review_id"], "9001");

    let reviews = rig.bodies("POST", "/pulls/9/reviews");
    assert_eq!(reviews.len(), 1, "one round trip");
    let review = &reviews[0];
    assert_eq!(review["event"], "REQUEST_CHANGES");
    assert_eq!(review["body"], "Needs work");
    assert_eq!(review["commit_id"], review::HEAD_SHA);
    let comments = review["comments"].as_array().unwrap();
    assert_eq!(comments[0]["side"], "RIGHT");
    assert_eq!(comments[0]["start_line"], 12);
    assert_eq!(comments[0]["line"], 14);
    assert_eq!(comments[0]["body"], "Extract this: it repeats");
    assert_eq!(comments[1]["side"], "LEFT");

    let err = rig.error(&["hosting", "review", "submit", "9", "--event", "comment"]);
    assert!(err.contains("needs a body"), "{err}");
    let err = rig.error(&[
        "hosting",
        "review",
        "submit",
        "9",
        "--event",
        "comment",
        "--comment",
        "no-line-here",
    ]);
    assert!(err.contains("path:line"), "{err}");
    assert_eq!(
        rig.posted("POST", "/pulls/9/reviews"),
        1,
        "refusals sent nothing"
    );
}

#[test]
fn gitlab_show_threads_and_conversation_split_the_discussions() {
    let rig = rig("gitlab_self_managed", "orbit-co/orbit", "review-gl-read");
    let out = rig.json(&["hosting", "review", "show", "58"]);
    let detail = &out["detail"];
    assert_eq!(detail["iid_label"], "!58");
    assert_eq!(detail["base_sha"], review::BASE_SHA);
    assert_eq!(detail["capabilities"]["request_changes"], false);
    let files = out["files"].as_array().unwrap();
    assert_eq!(files[0]["additions"], 2);
    assert_eq!(files[0]["deletions"], 1);
    assert_eq!(files[1]["path"], "docs/moons.md");

    let out = rig.json(&["hosting", "review", "threads", "58"]);
    let threads = out["threads"].as_array().unwrap();
    assert_eq!(threads.len(), 2);
    assert_eq!(threads[0]["id"], "d_open");
    assert_eq!(threads[0]["line"], 12);
    assert_eq!(threads[1]["resolved"], true);
    assert_eq!(out["conversation"].as_array().unwrap().len(), 2);
    // GitLab's verdicts are its approvals.
    let reviews = out["reviews"].as_array().unwrap();
    assert_eq!(reviews.len(), 1);
    assert_eq!(reviews[0]["author"], "mara");
    assert_eq!(reviews[0]["state"], "approved");
}

#[test]
fn gitlab_reply_resolve_and_comment_use_discussions_and_notes() {
    let rig = rig("gitlab_self_managed", "orbit-co/orbit", "review-gl-write");
    rig.json(&["hosting", "review", "comment", "58", "--body", "hello"]);
    assert_eq!(
        rig.bodies("POST", "/merge_requests/58/notes")[0]["body"],
        "hello"
    );
    rig.json(&["hosting", "review", "reply", "58", "d_open", "--body", "ok"]);
    assert_eq!(
        rig.bodies("POST", "/discussions/d_open/notes")[0]["body"],
        "ok"
    );
    rig.json(&["hosting", "review", "resolve", "58", "d_open"]);
    rig.json(&[
        "hosting",
        "review",
        "resolve",
        "58",
        "d_open",
        "--unresolve",
    ]);
    let puts = rig.bodies("PUT", "/discussions/d_open");
    assert_eq!(puts[0]["resolved"], true);
    assert_eq!(puts[1]["resolved"], false);
}

#[test]
fn gitlab_review_publishes_drafts_then_approves_and_has_no_request_changes() {
    let rig = rig("gitlab_self_managed", "orbit-co/orbit", "review-gl-review");
    let out = rig.json(&[
        "hosting",
        "review",
        "submit",
        "58",
        "--event",
        "approve",
        "--body",
        "LGTM",
        "--comment",
        "src/planets.rs:12:Nit",
    ]);
    assert_eq!(out["comments"], 1);
    let drafts = rig.bodies("POST", "/draft_notes");
    // The comment, the summary, then the publish call (an empty object).
    let notes: Vec<_> = drafts.iter().filter(|b| b.get("note").is_some()).collect();
    assert_eq!(notes.len(), 2);
    assert_eq!(notes[0]["position"]["new_line"], 12);
    assert_eq!(notes[0]["position"]["head_sha"], review::HEAD_SHA);
    assert_eq!(notes[1]["note"], "LGTM");
    assert_eq!(rig.posted("POST", "/draft_notes/bulk_publish"), 1);
    assert_eq!(rig.bodies("POST", "/approve")[0]["sha"], review::HEAD_SHA);

    let before = rig.mock.requests().len();
    let err = rig.error(&[
        "hosting",
        "review",
        "submit",
        "58",
        "--event",
        "request-changes",
        "--body",
        "no",
    ]);
    assert!(err.contains("does not offer `request-changes`"), "{err}");
    assert_eq!(rig.mock.requests().len(), before, "nothing was sent");
}

fn fetch_rig(kind: &str, project: &str, remote_ref: &str, session: &str) -> (Rig, String, String) {
    let rig = rig(kind, project, session);
    let repo = &rig.repo;
    let main_tip = repo.git(&["rev-parse", "main"]);
    repo.git(&["checkout", "-q", "-b", "work"]);
    std::fs::write(repo.root().join("f.txt"), "f\n").unwrap();
    repo.git(&["add", "f.txt"]);
    repo.git(&["commit", "-q", "-m", "the request"]);
    let tip = repo.git(&["rev-parse", "work"]);
    repo.git(&["push", "-q", "origin", &format!("work:{remote_ref}")]);
    repo.git(&["checkout", "-q", "main"]);
    repo.git(&["branch", "-D", "work"]);
    (rig, tip, main_tip)
}

#[test]
fn fetch_brings_the_head_and_the_base_into_the_namespace_without_a_branch() {
    for (kind, project, number, remote_ref, session) in [
        (
            "github_enterprise",
            "owner/repo",
            "9",
            "refs/pull/9/head",
            "review-fetch-gh",
        ),
        (
            "gitlab_self_managed",
            "orbit-co/orbit",
            "58",
            "refs/merge-requests/58/head",
            "review-fetch-gl",
        ),
    ] {
        let (rig, tip, main_tip) = fetch_rig(kind, project, remote_ref, session);
        let out = rig.json(&["hosting", "review", "fetch", number]);
        assert_eq!(out["head_ref"], format!("refs/gitpoulpi/pr/{number}"));
        assert_eq!(out["head_sha"], review::HEAD_SHA);
        assert_eq!(
            rig.repo
                .git(&["rev-parse", &format!("refs/gitpoulpi/pr/{number}")]),
            tip,
            "{kind}"
        );
        assert_eq!(
            rig.repo
                .git(&["rev-parse", &format!("refs/gitpoulpi/pr/{number}-base")]),
            main_tip,
            "{kind}"
        );
        let branches = rig
            .repo
            .git(&["for-each-ref", "refs/heads", "--format=%(refname:short)"]);
        assert_eq!(branches, "main", "no pr/ or mr/ branch: {kind}");
        assert_eq!(rig.repo.git(&["symbolic-ref", "--short", "HEAD"]), "main");
    }
}

/// `review suggest`: the fake agent answers, the suggestions come back as
/// JSON placed on the request's diff, and nothing is posted to the host.
#[cfg(feature = "ai")]
#[test]
fn suggest_prints_placed_suggestions_and_posts_nothing() {
    use gitpoulpi_testkit::fake_agent::{agent_calls, ai_config_toml, write_fake_agent};
    let (rig, _tip, _main_tip) = fetch_rig(
        "github_enterprise",
        "owner/repo",
        "refs/pull/9/head",
        "review-suggest-gh",
    );
    let tools = tempfile::tempdir().unwrap();
    let agent = write_fake_agent(
        tools.path(),
        &[
            r#"Here is my review: {"verdict": "Changes requested", "body": "Please rename it.", "summary": "A new file, nothing risky.", "suggestions": [{"path": "f.txt", "line": 1, "severity": "warning", "body": "Name this file better."}, {"path": "f.txt", "line": 50, "body": "Far away."}, {"path": "gone.txt", "line": 2, "body": "Not in the request."}]}"#,
        ],
    );

    // AI is off by default: refused before anything is fetched or asked.
    let error = rig.error(&["hosting", "review", "suggest", "9"]);
    assert!(error.contains("turned off"), "{error}");
    assert_eq!(agent_calls(&agent), 0);

    let path = rig.config.path().join("config.toml");
    let mut config = std::fs::read_to_string(&path).unwrap();
    config.push_str(&ai_config_toml(&agent));
    std::fs::write(&path, config).unwrap();

    let out = rig.json(&["hosting", "review", "suggest", "9"]);
    assert_eq!(agent_calls(&agent), 1);
    assert_eq!(out["posted"], false);
    assert_eq!(out["summary"], "A new file, nothing risky.");
    assert_eq!(out["verdict"], "request_changes");
    assert_eq!(out["body"], "Please rename it.");
    let found = out["suggestions"].as_array().unwrap();
    assert_eq!(found.len(), 3, "none is dropped: {out}");
    let placed = &found[0];
    assert_eq!(placed["path"], "f.txt");
    assert_eq!(
        (placed["line"].as_u64(), placed["side"].as_str()),
        (Some(1), Some("new"))
    );
    assert_eq!(placed["severity"], "warning");
    assert_eq!(placed["anchored"], true);
    assert_eq!(placed["body"], "Name this file better.");
    // A line the diff does not show is listed for the file, with where it would go.
    let far = found.iter().find(|s| s["line"] == 50).unwrap();
    assert_eq!(
        (far["anchored"].clone(), far["severity"].clone()),
        (false.into(), "info".into())
    );
    assert_eq!(far["fallback"]["line"], 1);
    // A file the request does not touch has nowhere to go.
    let ghost = found.iter().find(|s| s["path"] == "gone.txt").unwrap();
    assert_eq!(ghost["anchored"], false);
    assert!(ghost.get("fallback").is_none(), "{ghost}");
    assert_eq!(out["cut"], serde_json::json!([]));

    // Reading is all it did to the host: no review, comment or reply.
    let wrote: Vec<_> = rig
        .mock
        .requests()
        .into_iter()
        .filter(|r| r.method != "GET" && !r.target.contains("graphql"))
        .collect();
    assert!(wrote.is_empty(), "{wrote:?}");
    assert_eq!(rig.posted("POST", "/reviews"), 0);
}

/// In a clone the conventions are read from the request's base ref through
/// the daemon (not the working tree), and `--run-in-repo` decides where a
/// local agent works.
#[cfg(all(feature = "ai", unix))]
#[test]
fn suggest_in_a_clone_reads_the_conventions_from_the_base_ref_and_places_the_agent() {
    use gitpoulpi_testkit::fake_agent::{agent_prompt, ai_config_toml, write_capturing_fake_agent};
    let (rig, _tip, _main_tip) = fetch_rig(
        "github_enterprise",
        "owner/repo",
        "refs/pull/9/head",
        "review-suggest-ctx",
    );
    let repo = &rig.repo;
    // The conventions live on main (what the request targets); the working
    // tree then changes them, which the review must not see.
    std::fs::write(repo.root().join("CLAUDE.md"), "BASE RULE: tests first\n").unwrap();
    repo.git(&["add", "CLAUDE.md"]);
    repo.git(&["commit", "-q", "-m", "conventions"]);
    repo.git(&["push", "-q", "origin", "main"]);
    std::fs::write(repo.root().join("CLAUDE.md"), "WORKTREE EDIT\n").unwrap();

    let tools = tempfile::tempdir().unwrap();
    let agent = write_capturing_fake_agent(
        tools.path(),
        &[r#"{"summary": "Fine.", "suggestions": []}"#],
    );
    // The agent also tells where it ran.
    let script = std::fs::read_to_string(&agent).unwrap();
    std::fs::write(
        &agent,
        script.replacen(
            "echo called",
            "pwd > \"$(dirname \"$0\")/agent.pwd\"\necho called",
            1,
        ),
    )
    .unwrap();
    let path = rig.config.path().join("config.toml");
    let mut config = std::fs::read_to_string(&path).unwrap();
    config.push_str(&ai_config_toml(&agent));
    config.push_str("workdir = \"repository\"\n");
    std::fs::write(&path, config).unwrap();
    let ran_in = || {
        std::fs::read_to_string(tools.path().join("agent.pwd"))
            .unwrap()
            .trim()
            .to_string()
    };
    let root = std::fs::canonicalize(repo.root()).unwrap();

    let out = rig.json(&["hosting", "review", "suggest", "9"]);
    let prompt = agent_prompt(&agent);
    assert!(prompt.contains("BASE RULE: tests first"), "{prompt}");
    assert!(!prompt.contains("WORKTREE EDIT"), "{prompt}");
    assert_eq!(out["context"]["files"][0]["label"], "CLAUDE.md");
    // The profile says `repository`: the agent works there, and is told the
    // checkout may not be the request's head.
    assert_eq!(std::fs::canonicalize(ran_in()).unwrap(), root);
    assert_eq!(out["context"]["agent_in_repository"], true);
    assert!(prompt.contains("authoritative"), "{prompt}");

    // `--run-in-repo no` moves it out for this run.
    let out = rig.json(&["hosting", "review", "suggest", "9", "--run-in-repo", "no"]);
    assert_ne!(std::fs::canonicalize(ran_in()).unwrap(), root);
    assert_eq!(out["context"]["agent_in_repository"], false);
    assert!(!agent_prompt(&agent).contains("authoritative"));
}

// No test reads or writes the real config, data dir or credentials of whoever runs it.
gitpoulpi_testkit::isolate_binary!();
