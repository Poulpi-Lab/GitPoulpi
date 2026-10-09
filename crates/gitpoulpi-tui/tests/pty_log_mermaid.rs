#![cfg(feature = "mermaid")]
//! Copy as Mermaid gitGraph through the real binary: select three commits of
//! the Log (a merge and the commits it joins), open the popup with `Shift+Y`,
//! watch the live preview, switch the detail level in its combobox and copy.
//! The copied text is read back through the test clipboard hook
//! (`GITPOULPI_CLIPBOARD_DUMP`), whole, not from the status line's echo.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::Screen;

const STEP: Duration = Duration::from_secs(30);

fn fixture() -> FixtureRepo {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "base\n", "c1 base")
        .unwrap()
        .branch("feature/x")
        .unwrap()
        .commit_file("f.txt", "f\n", "c2 feature work")
        .unwrap()
        .checkout("-")
        .unwrap()
        .commit_file("m.txt", "m\n", "c3 main work")
        .unwrap()
        .build();
    repo.git(&["merge", "--no-ff", "-qm", "c4 merge feature", "feature/x"]);
    repo
}

fn cell(s: &Screen, text: &str) -> (u16, u16) {
    s.find_all(text)
        .into_iter()
        .next()
        .unwrap_or_else(|| panic!("{text} not on screen:\n{}", s.text()))
}

#[test]
fn three_commits_with_a_merge_copy_as_a_gitgraph_at_two_detail_levels() {
    let repo = fixture();
    let dump = tempfile::tempdir().unwrap();
    let dump_file = dump.path().join("clipboard.txt");
    let (mut term, config) = common::launch(repo.root())
        .session("pty-log-mermaid")
        .size(200, 50)
        .no_exit_pause()
        .env("GITPOULPI_CLIPBOARD_DUMP", &dump_file)
        .ready_when("the log", |s| s.contains("c4 merge feature"))
        .mouse_capture()
        .start();
    std::mem::forget(config);

    let s = term
        .wait_for("all rows", STEP, |s| s.contains("c1 base"))
        .unwrap_or_else(|e| panic!("{e}"));
    let (col, merge_row) = cell(&s, "c4 merge feature");
    let (_, work_row) = cell(&s, "c2 feature work");
    let (_, main_row) = cell(&s, "c3 main work");
    term.click(col, merge_row).unwrap();
    term.ctrl_click(col, work_row).unwrap();
    term.ctrl_click(col, main_row).unwrap();

    term.send_keys("S-y").unwrap();
    term.wait_for("the popup with its preview", STEP, |s| {
        s.contains("Copy as Mermaid gitGraph")
            && s.contains("Selection (3 commits)")
            && s.contains("gitGraph LR:")
            && s.contains("type: HIGHLIGHT")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    // Refs level: the merge is a merge and the branch carries its name.
    let s = term.screen();
    assert!(s.contains("merge \"feature/x\""), "{}", s.text());

    // Tab to the Detail combobox (Scope first), open it, pick Subjects.
    term.send_keys("Tab Tab Enter").unwrap();
    term.wait_for("the detail list", STEP, |s| s.contains("Structure: "))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Down Enter").unwrap();
    term.wait_for("subjects in the preview", STEP, |s| {
        s.contains("c2 feature work")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    // Back to the preview (Orientation, Fence, Preview), Enter copies.
    term.send_keys("Tab Tab Tab Enter").unwrap();
    term.wait_for("the copy confirmation", STEP, |s| {
        s.contains("copied Mermaid gitGraph") && !s.contains("Include the")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    let copied = std::fs::read_to_string(&dump_file).unwrap();
    assert!(copied.starts_with("gitGraph LR:\n"), "{copied}");
    assert!(copied.contains("branch \"feature/x\""), "{copied}");
    assert!(copied.contains("merge \"feature/x\" id: \""), "{copied}");
    assert!(copied.contains("c2 feature work"), "{copied}");
    assert!(copied.contains("c3 main work"), "{copied}");
    assert!(copied.contains("c4 merge feature"), "{copied}");
    assert!(
        !copied.contains("c1 base"),
        "the root was not selected: {copied}"
    );
}
