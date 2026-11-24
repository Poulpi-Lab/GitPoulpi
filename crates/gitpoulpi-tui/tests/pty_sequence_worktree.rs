//! A worktree sequence, end to end through a pty (real binary, daemon and
//! git): a step makes a worktree from the name template and opens it as a
//! tab, and the next step (open the tab's terminal) acts on that new tab, not
//! on the tab the run started in.
//!
//! The terminal is a fake program recording where it was opened; no real
//! terminal is started. Temp config dir and fixture repo only.

mod common;

use std::time::{Duration, Instant};

use gitpoulpi_testkit::FixtureRepoBuilder;

const STEP: Duration = Duration::from_secs(40);

#[test]
fn the_name_template_makes_the_worktree_and_the_terminal_opens_in_its_tab() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "initial commit")
        .unwrap()
        .build();
    let work = tempfile::tempdir().unwrap();
    let log = work.path().join("launches.log");
    let fake = common::fake_recorder(work.path(), "fake-term", &log);
    let config = format!(
        "[ui]\nterminal_launch = \"custom\"\nterminal_launch_command = \"{fake} --here {{repo}}\"\n\n\
         [[sequences]]\nid = \"wt\"\nname = \"Worktree run\"\nscope = \"machine\"\n\n\
         [[sequences.steps]]\nrun = \"create_worktree_from_current_branch\"\npause = \"never\"\n\
         [sequences.steps.options]\nname = \"seq-wip-${{n}}\"\nopen_tab = true\n\n\
         [[sequences.steps]]\nrun = \"tab_dir_terminal\"\n",
        fake = common::toml_path(&fake)
    );
    let (mut term, _config) = common::launch(repo.root())
        .session("pty-seq-worktree")
        .config(&config)
        .size(200, 50)
        .no_exit_pause()
        .ready("initial commit")
        .start();
    term.send_keys("C-p").unwrap();
    term.wait_for_text("Command Palette", STEP).unwrap();
    term.send_text("Sequence: Worktree run").unwrap();
    term.wait_for("the row", STEP, |s| s.contains("Sequence: Worktree run"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();

    // The name template's first free number: `<repo>-seq-wip-1`.
    let name = repo.root().file_name().unwrap().to_string_lossy().into_owned();
    let worktree = repo
        .root()
        .parent()
        .unwrap()
        .join(format!("{name}-seq-wip-1"));
    let deadline = Instant::now() + STEP;
    let launched = loop {
        let text = common::read_log(&log);
        if !text.is_empty() {
            break text;
        }
        assert!(
            Instant::now() < deadline,
            "the terminal never opened\n{}",
            term.text()
        );
        std::thread::sleep(Duration::from_millis(200));
    };
    assert!(worktree.join(".git").exists(), "the worktree was created");
    let opened_in = launched.trim().rsplit(' ').next().unwrap_or_default().to_string();
    assert_eq!(
        std::fs::canonicalize(&opened_in).unwrap(),
        std::fs::canonicalize(&worktree).unwrap(),
        "the terminal opened in the new worktree's tab: {launched}"
    );
    // And the window moved to that tab.
    term.wait_for("the new tab on screen", STEP, |s| {
        s.contains(&format!("{name}-seq-wip-1"))
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.stop().unwrap();
}
