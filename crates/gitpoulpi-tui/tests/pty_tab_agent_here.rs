//! "Tab: Open AI Agent Here" through a real pty: the user's terminal opens in
//! the active tab's folder running the local AI agent of the AI profile. One
//! local agent runs at once, several ask first (the "Open in" popup, one row
//! each), AI off hides the row, no local agent grays it with the reason.
//!
//! The terminal is a fake script named by `$TERMINAL`, recording its working
//! folder and arguments: no real terminal or agent is ever started and no
//! prompt is sent. Temp config dir and fixture repo only. Unix only (the
//! terminal choice on Windows and macOS is not an environment variable); the
//! launcher syntax per terminal is a unit test (`agent_launch`).
#![cfg(all(unix, not(target_os = "macos")))]

mod common;

use std::path::Path;
use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(30);

struct Setup {
    repo: gitpoulpi_testkit::FixtureRepo,
    config: tempfile::TempDir,
    work: tempfile::TempDir,
    terminal: std::path::PathBuf,
}

impl Setup {
    fn new(ai_toml: &str) -> Self {
        let repo = FixtureRepoBuilder::new()
            .unwrap()
            .commit_file("a.txt", "a\n", "agent-here-commit")
            .unwrap()
            .build();
        let config = tempfile::tempdir().unwrap();
        let work = tempfile::tempdir().unwrap();
        let log = work.path().join("launches.log");
        let terminal = common::fake_program(
            work.path(),
            "fake-terminal",
            &format!("echo \"$(pwd -P)|$*\" >> '{}'", log.display()),
            "",
        );
        std::fs::write(
            config.path().join("config.toml"),
            format!("[ui]\ngraph_renderer = \"text\"\n\n{ai_toml}"),
        )
        .unwrap();
        Self {
            repo,
            config,
            work,
            terminal,
        }
    }

    fn launches(&self) -> String {
        common::read_log(&self.work.path().join("launches.log"))
    }

    fn launched(&self) -> String {
        for _ in 0..200 {
            if !self.launches().is_empty() {
                break;
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        self.launches()
    }

    fn start(&self) -> Terminal {
        common::launch(self.repo.root())
            .session("tab-agent-here")
            .config_dir(self.config.path())
            .env("TERMINAL", &self.terminal)
            .size(200, 50)
            .no_exit_pause()
            .ready("agent-here-commit")
            .spawn()
    }
}

fn agent(id: &str, name: &str, command: &str) -> String {
    format!(
        "[[ai.profiles]]\nid = \"{id}\"\nname = \"{name}\"\nkind = \"local_agent\"\n\
         command = '{command}'\nterminal_args = [\"--hello\"]\n\n"
    )
}

fn palette_to_agent_row(term: &mut Terminal) {
    term.send_keys("C-p").unwrap();
    term.wait_for_text("Command Palette", STEP).unwrap();
    term.send_text("Agent Here").unwrap();
}

fn same_dir(a: &str, b: &Path) -> bool {
    std::fs::canonicalize(a).unwrap() == std::fs::canonicalize(b).unwrap()
}

#[test]
fn one_local_agent_opens_in_the_tabs_folder_at_once() {
    let setup = Setup::new(&format!(
        "[ai]\nenabled = true\ndefault_profile = \"solo\"\n\n{}",
        agent("solo", "Solo agent", "fake-agent-solo")
    ));
    let mut term = setup.start();
    palette_to_agent_row(&mut term);
    term.wait_for("the row", STEP, |s| s.contains("Tab: Open AI Agent Here"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    let log = setup.launched();
    let (cwd, args) = log
        .trim()
        .split_once('|')
        .unwrap_or_else(|| panic!("nothing launched:\n{}", term.text()));
    assert!(same_dir(cwd, setup.repo.root()), "{log}");
    assert_eq!(args, "-e fake-agent-solo --hello", "{log}");
    term.stop().unwrap();
}

#[test]
fn several_agents_ask_first_and_the_pick_runs() {
    let setup = Setup::new(&format!(
        "[ai]\nenabled = true\ndefault_profile = \"one\"\n\n{}{}",
        agent("one", "First agent", "fake-agent-one"),
        agent("two", "Second agent", "fake-agent-two")
    ));
    let mut term = setup.start();
    palette_to_agent_row(&mut term);
    term.wait_for("the row", STEP, |s| s.contains("Tab: Open AI Agent Here"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    term.wait_for("the chooser", STEP, |s| {
        s.contains("First agent") && s.contains("Second agent")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(setup.launches().is_empty(), "asked first, ran nothing");
    term.send_keys("Down").unwrap();
    term.send_keys("Enter").unwrap();
    let log = setup.launched();
    let (cwd, args) = log
        .trim()
        .split_once('|')
        .unwrap_or_else(|| panic!("nothing launched:\n{}", term.text()));
    assert!(same_dir(cwd, setup.repo.root()), "{log}");
    assert_eq!(args, "-e fake-agent-two --hello", "{log}");
    term.stop().unwrap();
}

#[test]
fn ai_off_hides_the_row_and_no_local_agent_grays_it() {
    let off = Setup::new(&format!(
        "[ai]\nenabled = false\n\n{}",
        agent("solo", "Solo agent", "fake-agent-solo")
    ));
    let mut term = off.start();
    palette_to_agent_row(&mut term);
    term.wait_for("the query typed", STEP, |s| s.contains("Agent Here"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    std::thread::sleep(Duration::from_millis(300));
    assert!(
        !term.screen().contains("Tab: Open AI Agent Here"),
        "{}",
        term.text()
    );
    term.stop().unwrap();

    let none = Setup::new(
        "[ai]\nenabled = true\ndefault_profile = \"api\"\n\n[[ai.profiles]]\nid = \"api\"\n\
         name = \"Api\"\nkind = \"open_ai_compatible\"\n",
    );
    let mut term = none.start();
    palette_to_agent_row(&mut term);
    term.wait_for("the row grayed with its reason", STEP, |s| {
        s.contains("Tab: Open AI Agent Here") && s.contains("no local AI agent")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    std::thread::sleep(Duration::from_millis(300));
    assert!(none.launches().is_empty());
    term.stop().unwrap();
}
