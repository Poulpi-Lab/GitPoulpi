//! `gitpoulpi flow issue new|wait` and `flow pick-up`: host-driven flows
//! against `xtask`'s stateful flow host (a GitLab-shaped API whose bot makes
//! `{type}/{iid}-{slug}` and a merge request in the fixture's real bare
//! origin). Deserialized JSON; no network, own config dir and fixture per
//! test.

use assert_cmd::Command;
use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use serde_json::Value;
use xtask::mock_host::flow::{self, FlowHost, FlowHostOptions};

struct Rig {
    config: tempfile::TempDir,
    repo: FixtureRepo,
    host: FlowHost,
    session: String,
}

fn workflow(mode: &str) -> String {
    format!(
        "template = \"github-flow\"\n\n[[base]]\nname = \"main\"\nprotected = true\n\n[[topic]]\n\
         id = \"feature\"\nlabel = \"Feature\"\npattern = \"feature/{{slug}}\"\nparent = \"main\"\n\n\
         [[topic]]\nid = \"bugfix\"\npattern = \"bugfix/{{slug}}\"\nparent = \"main\"\n\n\
         [issue_flow]\nmode = \"{mode}\"\nissue_title = \"[{{type}}] {{title}}\"\n\
         issue_labels = [\"workflow::{{type}}\", \"bot\"]\nwait_for = \"branch-or-mr\"\ntimeout_secs = 20\n"
    )
}

fn rig(session: &str, mode: &str, bot: bool) -> Rig {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "a\n", "start")
        .unwrap()
        .with_origin()
        .unwrap()
        .push("main")
        .unwrap()
        .build();
    let mut options = FlowHostOptions::new(repo.remote_path().unwrap());
    if !bot {
        options.bot_delay = None;
    }
    let host = flow::start(options).expect("start the flow host");
    let config = tempfile::tempdir().unwrap();
    std::fs::write(
        config.path().join("config.toml"),
        format!(
            "[[integrations]]\nid = \"h\"\nkind = \"gitlab_self_managed\"\nurl = \"http://127.0.0.1:{}\"\n\
             token_ref = \"integration:h\"\nname = \"h\"\nuser = {{ login = \"mara\" }}\n",
            host.port
        ),
    )
    .unwrap();
    let exclude = repo.root().join(".git").join("info").join("exclude");
    std::fs::create_dir_all(exclude.parent().unwrap()).unwrap();
    std::fs::write(exclude, "/.gitpoulpi/\n").unwrap();
    let dir = repo.state_dir();
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("workflow.toml"), workflow(mode)).unwrap();
    let rig = Rig {
        config,
        repo,
        host,
        session: session.to_string(),
    };
    for list in ["issues", "pulls"] {
        rig.json(&["hosting", "link", list, "--id", "h", "--project", "o/p"]);
    }
    rig
}

impl Rig {
    fn output(&self, args: &[&str]) -> (std::process::Output, Value) {
        let output = Command::cargo_bin("gitpoulpi")
            .unwrap()
            .env("GITPOULPI_CONFIG_DIR", self.config.path())
            .env("GITPOULPI_DESKTOP_NO_HOOKS", "1")
            .env("GITPOULPI_TOKEN_H", "test-token")
            .env("GITPOULPI_FLOW_POLL_MS", "100")
            .arg("--json")
            .args(["--session", &self.session])
            .args(["--repo", &self.repo.root().to_string_lossy()])
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
        (output, json)
    }

    fn json(&self, args: &[&str]) -> Value {
        let (output, json) = self.output(args);
        assert!(
            output.status.success(),
            "`{}` failed ({:?}): {json}",
            args.join(" "),
            output.status.code()
        );
        json
    }

    fn head(&self) -> String {
        self.repo
            .git(&["rev-parse", "--abbrev-ref", "HEAD"])
            .trim()
            .to_string()
    }

    fn parent_of(&self, branch: &str) -> Option<String> {
        self.repo
            .try_git(&["config", &format!("branch.{branch}.gitpoulpi-parent")])
            .map(|s| s.trim().to_string())
    }

    fn wait_for_bot(&self) {
        for _ in 0..100 {
            if !self.host.merge_requests().is_empty() {
                return;
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        panic!("the bot never made its merge request");
    }
}

#[test]
fn a_dry_run_prints_the_issue_it_would_create_and_creates_nothing() {
    let rig = rig("flow-issue-dry", "bot", true);
    let out = rig.json(&[
        "flow",
        "issue",
        "new",
        "--type",
        "feature",
        "--title",
        "Login page",
        "--wait",
        "--pick-up",
        "--dry-run",
    ]);
    let plan = &out["plan"];
    assert_eq!(plan["mode"], "bot");
    assert_eq!(plan["issue"]["title"], "[feature] Login page");
    assert_eq!(
        plan["issue"]["labels"],
        serde_json::json!(["workflow::feature", "bot"])
    );
    assert_eq!(plan["steps"].as_array().unwrap().len(), 3);
    assert!(rig.host.issues().is_empty(), "nothing was created");
    assert!(!rig.host.requests().iter().any(|r| r.method == "POST"));
}

#[test]
fn bot_mode_creates_the_issue_waits_for_the_bot_and_picks_its_branch_up() {
    let rig = rig("flow-issue-bot", "bot", true);
    let out = rig.json(&[
        "flow",
        "issue",
        "new",
        "--type",
        "feature",
        "--title",
        "Login page",
        "--wait",
        "--pick-up",
    ]);
    assert_eq!(out["issue"]["number"], 1);
    assert_eq!(out["timed_out"], false);
    assert_eq!(
        out["found"]["branches"],
        serde_json::json!(["feature/1-login-page"])
    );
    assert_eq!(out["found"]["requests"][0]["target_branch"], "main");
    assert_eq!(out["picked_up"]["status"], "done");

    // The issue went through the template: title, labels.
    let issue = &rig.host.bodies("POST", "/issues")[0];
    assert_eq!(issue["title"], "[feature] Login page");
    assert_eq!(issue["labels"], "workflow::feature,bot");

    // The bot's branch is checked out, tracking the remote, parent recorded.
    assert_eq!(rig.head(), "feature/1-login-page");
    assert_eq!(
        rig.parent_of("feature/1-login-page").as_deref(),
        Some("main")
    );
    assert_eq!(
        rig.repo
            .git(&[
                "rev-parse",
                "--abbrev-ref",
                "feature/1-login-page@{upstream}"
            ])
            .trim(),
        "origin/feature/1-login-page"
    );
    assert!(rig.repo.root().join("a.txt").exists());

    // One undo takes the pick-up back.
    let undone = rig.json(&["undo"]);
    assert!(undone.to_string().contains("applied") || undone.is_object());
    assert_eq!(rig.head(), "main");
}

#[test]
fn waiting_that_runs_out_exits_with_code_4_and_says_so() {
    let rig = rig("flow-issue-timeout", "bot", false);
    let (output, json) = rig.output(&["flow", "issue", "wait", "#7", "--timeout", "1"]);
    assert_eq!(output.status.code(), Some(4), "{json}");
    assert_eq!(json["timed_out"], true);
    assert_eq!(json["found"]["branches"], serde_json::json!([]));
    // `--once` looks a single time.
    let (output, json) = rig.output(&["flow", "issue", "wait", "#7", "--once"]);
    assert_eq!(output.status.code(), Some(4), "{json}");
}

#[test]
fn a_later_wait_finds_what_the_bot_made_and_pick_up_takes_it_by_issue_number() {
    let rig = rig("flow-issue-later", "bot", true);
    let created = rig.json(&[
        "flow",
        "issue",
        "new",
        "--type",
        "bugfix",
        "--title",
        "Crash on start",
    ]);
    assert_eq!(created["issue"]["number"], 1);
    assert!(created.get("found").is_none(), "no --wait: nothing awaited");
    rig.wait_for_bot();

    let waited = rig.json(&["flow", "issue", "wait", "#1", "--for", "mr"]);
    assert_eq!(waited["timed_out"], false);
    assert_eq!(
        waited["found"]["requests"][0]["source_branch"],
        "bugfix/1-crash-on-start"
    );

    let dry = rig.json(&["flow", "pick-up", "#1", "--dry-run"]);
    let ops: Vec<&str> = dry["plan"]["steps"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s["op"].as_str().unwrap())
        .collect();
    assert_eq!(
        ops,
        ["create_branch", "set_upstream", "checkout", "record_parent"]
    );
    assert_eq!(rig.head(), "main", "a dry run changes nothing");

    let picked = rig.json(&["flow", "pick-up", "#1"]);
    assert_eq!(picked["status"], "done");
    assert_eq!(picked["merge_request"]["target_branch"], "main");
    assert_eq!(rig.head(), "bugfix/1-crash-on-start");
    assert_eq!(
        rig.parent_of("bugfix/1-crash-on-start").as_deref(),
        Some("main")
    );
}

#[test]
fn host_creates_branch_makes_the_branch_and_a_draft_request_that_closes_the_issue() {
    let rig = rig("flow-issue-host", "host-creates-branch", false);
    let out = rig.json(&[
        "flow",
        "issue",
        "new",
        "--type",
        "feature",
        "--title",
        "Dark mode",
        "--pick-up",
    ]);
    assert_eq!(out["branch"], "feature/1-dark-mode");
    assert_eq!(out["merge_request"]["target_branch"], "main");
    assert_eq!(out["picked_up"]["status"], "done");

    let branch = &rig.host.bodies("POST", "/repository/branches")[0];
    assert_eq!(branch["branch"], "feature/1-dark-mode");
    assert_eq!(branch["ref"], "main");
    let mr = &rig.host.bodies("POST", "/merge_requests")[0];
    assert_eq!(mr["description"], "Closes #1");
    assert_eq!(mr["source_branch"], "feature/1-dark-mode");
    assert!(mr["title"].as_str().unwrap().starts_with("Draft: "));

    assert_eq!(rig.head(), "feature/1-dark-mode");
    assert_eq!(
        rig.parent_of("feature/1-dark-mode").as_deref(),
        Some("main")
    );
    assert!(rig
        .repo
        .git_in_remote(&["branch"])
        .contains("feature/1-dark-mode"));
}

#[test]
fn pick_up_names_what_is_wrong() {
    let rig = rig("flow-issue-errors", "bot", false);
    let (output, json) = rig.output(&["flow", "pick-up", "feature/99-ghost"]);
    assert!(!output.status.success());
    assert!(json["error"].as_str().unwrap().contains("fetch"), "{json}");
    let (output, json) = rig.output(&["flow", "pick-up", "#5"]);
    assert!(!output.status.success());
    assert!(
        json["error"].as_str().unwrap().contains("nothing is known"),
        "{json}"
    );
    let (output, json) = rig.output(&["flow", "issue", "new", "--type", "nope", "--title", "x"]);
    assert!(!output.status.success());
    assert!(
        json["error"]
            .as_str()
            .unwrap()
            .contains("known: feature, bugfix"),
        "{json}"
    );
}

// No test reads or writes the real config, data dir or credentials of whoever runs it.
gitpoulpi_testkit::isolate_binary!();
