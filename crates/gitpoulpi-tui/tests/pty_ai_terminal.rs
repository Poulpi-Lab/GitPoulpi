#![cfg(feature = "ai")]
//! The terminal run mode of a local agent, end to end through a pty: the
//! real TUI, the real daemon, real `git`, and the testkit's **fake
//! interactive agent** (a small program that paints a ready marker, a busy
//! marker and its answer between the sentinel markers, like a real agent
//! would) standing in for Claude Code, Codex and Gemini. No real agent, no
//! account and no socket is involved.
//!
//! The rules are `pty_ui.rs`'s: nothing is located by a hard-coded
//! coordinate, nothing sleeps, and every wait describes the whole state it
//! is waiting for.

mod common;

use std::path::Path;
use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(40);

fn fixture() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "initial commit")
        .unwrap()
        .write_worktree("a.txt", b"one\ntwo\n")
        .unwrap()
        .stage("a.txt")
        .unwrap()
        .build()
}

/// `config.toml`: AI on, one Gemini-type profile in terminal mode whose
/// command is the fake agent, running in the repository.
fn config(env: &[(&str, &str)]) -> String {
    let agent = gitpoulpi_testkit::terminal_agent::program();
    let mut toml = format!(
        "[ai]\nenabled = true\n\n[[ai.profiles]]\nid = \"fake\"\nname = \"Fake terminal agent\"\n\
         kind = \"local_agent\"\ncommand = '{}'\nharness = \"gemini\"\nargs = []\n\
         run_mode = \"terminal\"\nworkdir = \"repository\"\ntimeout_secs = 60\n\n\
         [ai.profiles.env]\n",
        agent.display()
    );
    for (k, v) in env {
        toml.push_str(&format!("{k} = '{v}'\n"));
    }
    toml
}

fn launch(repo: &FixtureRepo, session: &str, config_toml: &str) -> (Terminal, tempfile::TempDir) {
    common::launch(repo.root())
        .session(session)
        .config(config_toml)
        .size(200, 50)
        .ready_when("the fixture loaded, WIP row and all", |s| {
            s.contains("initial commit")
                && s.contains("// WIP")
                && s.contains("Amend previous commit")
        })
        .start()
}

fn generate_pos(s: &xtask::pty::Screen) -> Option<(u16, u16)> {
    s.find_all("\u{2726}")
        .into_iter()
        .find(|&(_, r)| s.line(r).contains("Summary"))
}

fn click_generate(term: &mut Terminal) {
    let screen = term
        .wait_for("the Generate sparkle on the Summary row", STEP, |s| {
            generate_pos(s).is_some()
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let (col, row) = generate_pos(&screen).unwrap();
    term.click(col + 1, row).expect("click Generate");
}

/// The Generate button fills the composer from an agent driven in a virtual
/// terminal, and the agent ran in the repository (the profile's working
/// folder), with the profile's terminal arguments.
#[test]
fn generate_fills_the_composer_through_the_virtual_terminal() {
    let repo = fixture();
    let scratch = tempfile::tempdir().unwrap();
    let args_file = scratch.path().join("args");
    let cfg = config(&[
        (
            "FAKE_ANSWER",
            "feat: via the virtual terminal\\n\\nBody from the terminal agent.",
        ),
        ("FAKE_ARGS_FILE", args_file.to_str().unwrap()),
    ]);
    let (mut term, _config) = launch(&repo, "pty-ai-terminal", &cfg);
    click_generate(&mut term);

    term.wait_for("the composer filled from the terminal agent", STEP, |s| {
        s.contains("feat: via the virtual terminal") && s.contains("Body from the terminal agent.")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(
        repo.git(&["log", "-1", "--format=%s"]),
        "initial commit",
        "generating a message commits nothing"
    );
    let written = std::fs::read_to_string(&args_file).unwrap();
    let cwd = written.lines().nth(1).unwrap_or_default();
    assert_eq!(
        Path::new(cwd).canonicalize().unwrap(),
        repo.root().canonicalize().unwrap(),
        "the agent ran in the repository"
    );
}

/// A first-run dialog is never answered: the run stops with a message that
/// names the way out ("Open the agent"), and nothing is typed into the agent.
#[test]
fn a_first_run_dialog_stops_the_run_and_points_to_open_the_agent() {
    let repo = fixture();
    let (mut term, _config) = launch(
        &repo,
        "pty-ai-terminal-trust",
        &config(&[("FAKE_TRUST", "1")]),
    );
    click_generate(&mut term);
    common::open_status_error(&mut term, "AI request failed");
    term.wait_for("the one-time setup cause", STEP, |s| {
        s.contains("Open the agent")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(repo.git(&["log", "-1", "--format=%s"]), "initial commit");
}
