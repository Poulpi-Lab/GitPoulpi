//! Trust for a repository's committed workflow hooks through a pty: the real
//! binary, daemon and git, a temp config dir, no network.
//!
//! The Start Feature dialog's preview lists the hook and says it is not
//! trusted yet; the action button then asks "Trust them?" with the exact
//! command, no answer chosen, and
//!
//! - `1` trusts and runs: the hook ran in the repository;
//! - `2` runs without the hook: it did not;
//! - Esc goes back to the preview with nothing run, and Enter alone answers
//!   nothing.
//!
//! The rules are `pty_ui.rs`'s: nothing is located by a hard-coded
//! coordinate, nothing sleeps, and every wait describes the whole state it
//! is waiting for.

mod common;

use std::path::Path;
use std::time::{Duration, Instant};

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(30);

const HOOK: &str = if cfg!(windows) {
    "echo trusted> hook-ran.txt"
} else {
    "echo trusted > hook-ran.txt"
};

fn fixture() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "initial commit")
        .unwrap()
        .branch("develop")
        .unwrap()
        .build()
}

fn workflow() -> String {
    format!(
        r#"template = "git-flow"

[[base]]
name = "main"
protected = true

[[base]]
name = "develop"
parent = "main"
merge = "merge"

[[topic]]
id = "feature"
label = "Feature"
pattern = "feature/{{slug}}"
parent = "develop"
finish = [{{ into = "develop", merge = "merge" }}]

[hooks]
after_start = ["{HOOK}"]
"#
    )
}

fn launch(repo: &FixtureRepo, session: &str) -> Terminal {
    let dir = repo.root().join(".git").join("gitpoulpi");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("workflow.toml"), workflow()).unwrap();
    let config = tempfile::tempdir().expect("temp config dir");
    std::fs::write(
        config.path().join("config.toml"),
        "[ui]\ngraph_renderer = \"text\"\n",
    )
    .unwrap();
    let l = common::launch(repo.root())
        .session(session)
        .config_dir(config.path())
        .size(200, 60)
        .no_exit_pause()
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .mouse_capture();
    // The config dir lives as long as the test: leak it with the terminal.
    std::mem::forget(config);
    l.spawn()
}

/// Opens Start Feature through the palette, types a slug, previews, and
/// presses the action button: the trust question is on screen.
fn reach_the_question(term: &mut Terminal, repo: &Path) {
    term.wait_for("the Flow button", STEP, |s| s.contains("Flow \u{25BE}"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("C-p").unwrap();
    term.wait_for_text("Command Palette", STEP).unwrap();
    term.send_text("Start Feature").unwrap();
    term.wait_for("the palette entry", STEP, |s| {
        s.contains("Flow: Start Feature...")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    term.wait_for("the Start Feature form", STEP, |s| {
        s.contains("Slug") && s.contains("Start from")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_text("trusted work").unwrap();
    term.wait_for("the name preview", STEP, |s| {
        s.contains("feature/trusted-work")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    // The preview names the hook and says it is not trusted yet: no checkbox.
    term.wait_for("the preview with the untrusted note", STEP, |s| {
        s.contains("This will:") && s.contains("not trusted yet") && !s.contains("Run the hooks")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    term.wait_for("the trust question", STEP, |s| {
        s.contains("Trust them?")
            && s.contains(HOOK)
            && s.contains("1. Trust and run")
            && s.contains("2. Run without hooks")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(
        !repo.join("hook-ran.txt").exists(),
        "nothing runs while the question is open"
    );
}

fn wait_for_file(path: &Path) {
    let until = Instant::now() + xtask::pty::scaled(STEP);
    while !path.exists() {
        assert!(Instant::now() < until, "{} never appeared", path.display());
        std::thread::sleep(Duration::from_millis(100));
    }
}

#[test]
fn trust_and_run_runs_the_hook_and_enter_alone_answers_nothing() {
    let repo = fixture();
    let mut term = launch(&repo, "pty-flow-hook-trust-yes");
    reach_the_question(&mut term, repo.root());
    // No default: Enter before any arrow or digit leaves the question open.
    term.send_keys("Enter").unwrap();
    // The arrow starts the highlight on answer 1, the digit answers at once.
    term.send_keys("Down").unwrap();
    term.wait_for("the first answer highlighted", STEP, |s| {
        s.contains("> 1. Trust and run") && s.contains("Trust them?")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(!repo.root().join("hook-ran.txt").exists());
    term.send_text("1").unwrap();
    term.wait_for("the run done", STEP, |s| s.contains("Done."))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    wait_for_file(&repo.root().join("hook-ran.txt"));
    assert!(repo
        .git(&["branch", "--list", "feature/trusted-work"])
        .contains("feature/trusted-work"));
}

#[test]
fn escape_goes_back_and_run_without_hooks_skips_the_hook() {
    let repo = fixture();
    let mut term = launch(&repo, "pty-flow-hook-trust-no");
    reach_the_question(&mut term, repo.root());
    term.send_keys("Esc").unwrap();
    term.wait_for("the preview again, the question gone", STEP, |s| {
        s.contains("This will:") && s.contains("not trusted yet") && !s.contains("Trust them?")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(!repo.root().join("hook-ran.txt").exists());
    term.send_keys("Enter").unwrap();
    term.wait_for("the trust question again", STEP, |s| {
        s.contains("Trust them?")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_text("2").unwrap();
    term.wait_for("the run done", STEP, |s| s.contains("Done."))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(repo
        .git(&["branch", "--list", "feature/trusted-work"])
        .contains("feature/trusted-work"));
    assert!(
        !repo.root().join("hook-ran.txt").exists(),
        "run without hooks never ran the hook"
    );
}
