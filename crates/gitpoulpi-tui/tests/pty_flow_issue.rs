//! Host-driven flows (plan `branching-workflows.md`, slice W4) through a pty:
//! the real binary, daemon and git, against `xtask`'s flow host, whose bot
//! makes `{type}/{iid}-{slug}` in the fixture's real bare origin and a merge
//! request for each new issue. No network.
//!
//! - "Flow: New Feature Issue...": the form, the issue made from the template,
//!   the waiting card, the bot's branch picked up (checked out, upstream set,
//!   parent recorded);
//! - stopping the wait keeps the issue watched: the next fetch raises "a
//!   branch appeared" with a Pick up action.
//!
//! The rules are `pty_ui.rs`'s: nothing is located by a hard-coded
//! coordinate, nothing sleeps, and every wait describes the whole state it
//! is waiting for.

mod common;

use std::time::{Duration, Instant};

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::mock_host::flow::{self, FlowHost, FlowHostOptions};
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(30);

const WORKFLOW: &str = r#"template = "github-flow"

[[base]]
name = "main"
protected = true

[[topic]]
id = "feature"
label = "Feature"
pattern = "feature/{slug}"
parent = "main"

[issue_flow]
mode = "bot"
issue_title = "[{type}] {title}"
issue_labels = ["workflow::{type}", "bot"]
wait_for = "branch-or-mr"
timeout_secs = 60
"#;

struct Rig {
    repo: FixtureRepo,
    host: FlowHost,
    term: Terminal,
}

fn rig(session: &str, bot_delay_ms: u64) -> Rig {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "initial commit")
        .unwrap()
        .with_origin()
        .unwrap()
        .push("main")
        .unwrap()
        .build();
    let mut options = FlowHostOptions::new(repo.remote_path().unwrap());
    options.bot_delay = Some(Duration::from_millis(bot_delay_ms));
    let host = flow::start(options).expect("start the flow host");

    let exclude = repo.root().join(".git").join("info").join("exclude");
    std::fs::create_dir_all(exclude.parent().unwrap()).unwrap();
    std::fs::write(exclude, "/.gitpoulpi/\n").unwrap();
    let state = repo.state_dir();
    std::fs::create_dir_all(&state).unwrap();
    std::fs::write(state.join("workflow.toml"), WORKFLOW).unwrap();
    std::fs::write(
        state.join("repo-state.toml"),
        "[issues_source]\nintegration = \"h\"\nproject = \"o/p\"\n\n\
         [pulls_source]\nintegration = \"h\"\nproject = \"o/p\"\n",
    )
    .unwrap();

    let config = tempfile::tempdir().expect("temp config dir");
    std::fs::write(
        config.path().join("config.toml"),
        format!(
            "[ui]\ngraph_renderer = \"text\"\n\n[[integrations]]\nid = \"h\"\n\
             kind = \"gitlab_self_managed\"\nurl = \"http://127.0.0.1:{}\"\n\
             token_ref = \"integration:h\"\nname = \"h\"\nuser = {{ login = \"mara\" }}\n",
            host.port
        ),
    )
    .unwrap();
    let term = common::launch(repo.root())
        .session(session)
        .config_dir(config.path())
        .size(200, 60)
        .no_exit_pause()
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GITPOULPI_TOKEN_H", "test-token")
        .env("GITPOULPI_FLOW_POLL_MS", "200")
        .mouse_capture()
        .spawn();
    // The config dir lives as long as the test: leak it with the terminal.
    std::mem::forget(config);
    Rig { repo, host, term }
}

fn palette(term: &mut Terminal, query: &str, entry: &str) {
    term.send_keys("C-p").unwrap();
    term.wait_for_text("Command Palette", STEP).unwrap();
    term.send_text(query).unwrap();
    term.wait_for("the palette entry", STEP, |s| s.contains(entry))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
}

fn head(repo: &FixtureRepo) -> String {
    repo.git(&["rev-parse", "--abbrev-ref", "HEAD"])
        .trim()
        .to_string()
}

fn wait_head(rig: &Rig, want: &str) {
    let until = Instant::now() + xtask::pty::scaled(STEP);
    while head(&rig.repo) != want {
        assert!(
            Instant::now() < until,
            "HEAD never became {want} (is {}):\n{}",
            head(&rig.repo),
            rig.term.text()
        );
        std::thread::sleep(Duration::from_millis(100));
    }
}

fn start_new_issue(rig: &mut Rig, title: &str) {
    rig.term
        .wait_for("the Flow button", STEP, |s| s.contains("Flow \u{25BE}"))
        .unwrap_or_else(|e| panic!("{e}\n{}", rig.term.text()));
    palette(
        &mut rig.term,
        "New Feature Issue",
        "Flow: New Feature Issue...",
    );
    rig.term
        .wait_for("the New issue form", STEP, |s| {
            s.contains("New Feature issue")
                && s.contains("a bot (picked up when the branch appears)")
                && s.contains("Pick the branch up when it is there")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", rig.term.text()));
    // The title field has the focus.
    rig.term.send_text(title).unwrap();
    rig.term.send_keys("Enter").unwrap();
}

#[test]
fn a_new_issue_waits_for_the_bot_and_picks_its_branch_up() {
    let mut rig = rig("pty-flow-issue-bot", 1500);
    start_new_issue(&mut rig, "Login page");

    // The waiting card: what it waits for, the elapsed time, the way out.
    rig.term
        .wait_for("the waiting card", STEP, |s| {
            s.contains("issue #1")
                && s.contains("so far, giving up after 60 s")
                && s.contains("Stop waiting")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", rig.term.text()));

    // The bot makes feature/1-login-page; the dialog picks it up by itself.
    wait_head(&rig, "feature/1-login-page");
    rig.term
        .wait_for("the pick-up result", STEP, |s| {
            s.contains("picked up feature/1-login-page")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", rig.term.text()));
    assert_eq!(
        rig.repo
            .try_git(&["config", "branch.feature/1-login-page.gitpoulpi-parent"])
            .map(|s| s.trim().to_string())
            .as_deref(),
        Some("main"),
        "the parent is the request's target"
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
    // The issue went through the template.
    let issue = &rig.host.bodies("POST", "/issues")[0];
    assert_eq!(issue["title"], "[feature] Login page");
    assert_eq!(issue["labels"], "workflow::feature,bot");
}

#[test]
fn stopping_the_wait_keeps_the_issue_and_the_next_fetch_offers_the_branch() {
    // The bot is slow: the user stops waiting first.
    let mut rig = rig("pty-flow-issue-later", 2500);
    start_new_issue(&mut rig, "Dark mode");
    rig.term
        .wait_for("the waiting card", STEP, |s| s.contains("Stop waiting"))
        .unwrap_or_else(|e| panic!("{e}\n{}", rig.term.text()));
    rig.term.send_keys("Escape").unwrap();
    rig.term
        .wait_for("the card closed", STEP, |s| !s.contains("Stop waiting"))
        .unwrap_or_else(|e| panic!("{e}\n{}", rig.term.text()));
    assert_eq!(head(&rig.repo), "main", "nothing was picked up");

    // The bot finishes later; a fetch brings its branch and the app says so.
    let until = Instant::now() + xtask::pty::scaled(STEP);
    while rig.host.merge_requests().is_empty() {
        assert!(Instant::now() < until, "the bot never made its request");
        std::thread::sleep(Duration::from_millis(100));
    }
    palette(&mut rig.term, "Fetch All Remotes", "Git: Fetch All Remotes");
    rig.term
        .wait_for("the appeared card", STEP, |s| {
            s.contains("A branch for issue #1 appeared")
                && s.contains("feature/1-dark-mode")
                && s.contains("Pick up")
                && s.contains("Later")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", rig.term.text()));
    assert_eq!(head(&rig.repo), "main");

    // Enter on the focused Pick up button takes it over.
    rig.term.send_keys("Enter").unwrap();
    wait_head(&rig, "feature/1-dark-mode");
    assert_eq!(
        rig.repo
            .try_git(&["config", "branch.feature/1-dark-mode.gitpoulpi-parent"])
            .map(|s| s.trim().to_string())
            .as_deref(),
        Some("main")
    );
}

/// The form's close cross (top right of its frame) closes it like Esc does:
/// nothing is created.
#[test]
fn the_new_issue_form_closes_with_its_cross() {
    let mut rig = rig("pty-flow-issue-cross", 1500);
    rig.term
        .wait_for("the Flow button", STEP, |s| s.contains("Flow \u{25BE}"))
        .unwrap_or_else(|e| panic!("{e}\n{}", rig.term.text()));
    palette(
        &mut rig.term,
        "New Feature Issue",
        "Flow: New Feature Issue...",
    );
    rig.term
        .wait_for("the New issue form", STEP, |s| {
            s.contains("New Feature issue") && s.contains("Pick the branch up when it is there")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", rig.term.text()));
    // The cross sits on the title row of the frame; the click is retried from
    // where the row is found afresh (a torn frame can lack it).
    common::retry_until(
        &mut rig.term,
        "the form closed by its cross",
        6,
        Duration::from_secs(10),
        |t| {
            let s = t.screen();
            if let Some((_, row)) = s.find("New Feature issue") {
                if let Some(&(c, r)) = s.find_all("\u{2715}").iter().find(|(_, r)| *r == row) {
                    t.click(c, r).expect("click the cross");
                }
            }
        },
        |s| !s.contains("Pick the branch up when it is there") && !s.contains("New Feature issue"),
    );
    assert_eq!(head(&rig.repo), "main");
    assert!(
        rig.host.bodies("POST", "/issues").is_empty(),
        "closing creates no issue"
    );
}
