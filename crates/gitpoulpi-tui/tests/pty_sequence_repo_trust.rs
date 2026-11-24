//! A repository that carries its own sequences (`.gitpoulpi/sequences.toml`,
//! committed by other people) asks before anything of it is offered, end to
//! end through a pty (real binary, daemon and git): the notice over the Log,
//! Review listing every step, nothing in the palette until Trust, then the
//! sequence runs from the palette. A changed file asks again.
//!
//! Temp config dir and fixture repo only.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::{Screen, Terminal};

const STEP: Duration = Duration::from_secs(40);

const SEQUENCES: &str = r#"
[[sequences]]
id = "ship"
name = "Ship it"

[[sequences.steps]]
run = "stage_all"
"#;

fn fixture() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "initial commit")
        .unwrap()
        .commit_file(".gitpoulpi/sequences.toml", SEQUENCES, "add the sequences")
        .unwrap()
        .write_worktree("a.txt", b"one\ntwo\n")
        .unwrap()
        .build()
}

/// The column and row of `label` on the first line of `screen` holding both
/// `anchor` and `label`.
fn find(screen: &Screen, anchor: &str, label: &str) -> (u16, u16) {
    let row = (0..screen.rows())
        .find(|&r| {
            let l = screen.line(r);
            l.contains(anchor) && l.contains(label)
        })
        .unwrap_or_else(|| panic!("no row with {anchor:?} and {label:?}"));
    let line = screen.line(row);
    let at = line.find(label).unwrap();
    (line[..at].chars().count() as u16 + 1, row)
}

#[test]
fn an_untrusted_repository_file_asks_first_and_trust_makes_its_sequence_run() {
    let repo = fixture();
    let (mut term, _config): (Terminal, _) = common::launch(repo.root())
        .session("pty-seq-repo-trust")
        .size(200, 50)
        .no_exit_pause()
        .mouse_capture()
        .ready("initial commit")
        .start();

    // The notice, over the Log, with its three answers.
    let screen = term
        .wait_for("the notice about the repository's sequences", STEP, |s| {
            s.contains("This repository defines 1 sequence")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let line = (0..screen.rows())
        .map(|r| screen.line(r))
        .find(|l| l.contains("This repository defines"))
        .unwrap();
    assert!(
        line.contains("Review") && line.contains("Trust") && line.contains("Ignore"),
        "{line}"
    );

    // Nothing is offered while untrusted.
    term.send_keys("C-p").unwrap();
    term.wait_for_text("Command Palette", STEP).unwrap();
    term.send_text("Sequence: Ship").unwrap();
    std::thread::sleep(Duration::from_millis(600));
    assert!(
        !term.text().contains("Sequence: Ship it"),
        "an untrusted sequence is in the palette:\n{}",
        term.text()
    );
    term.send_keys("Escape").unwrap();
    term.wait_for("the palette closed", STEP, |s| {
        !s.contains("Command Palette")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    // Review lists the steps, trusts nothing by itself.
    let screen = term.screen();
    let (col, row) = find(&screen, "This repository defines", "Review");
    term.click(col, row).unwrap();
    let screen = term
        .wait_for("the review of the steps", STEP, |s| {
            s.contains("Sequences of this repository") && s.contains("1. stage_all")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let text = screen.text();
    assert!(
        text.contains("Ship it (ship)") && text.contains("1. stage_all"),
        "{text}"
    );
    term.send_text("c").unwrap();
    term.wait_for("the review closed", STEP, |s| {
        !s.contains("Sequences of this repository")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert!(
        term.text().contains("This repository defines 1 sequence"),
        "closing the review trusts nothing"
    );

    // Trust.
    let screen = term.screen();
    let (col, row) = find(&screen, "This repository defines", "Trust");
    term.click(col, row).unwrap();
    term.wait_for("the notice gone and the status line", STEP, |s| {
        !s.contains("This repository defines") && s.contains("Repository sequences trusted: 1")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    // Now the palette has it, and it runs.
    term.send_keys("C-p").unwrap();
    term.wait_for_text("Command Palette", STEP).unwrap();
    term.send_text("Sequence: Ship").unwrap();
    term.wait_for("the row of the trusted sequence", STEP, |s| {
        s.contains("Sequence: Ship it")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    let deadline = std::time::Instant::now() + STEP;
    loop {
        if repo
            .git(&["diff", "--cached", "--name-only"])
            .contains("a.txt")
        {
            break;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "the repository sequence never staged the file:\n{}",
            term.text()
        );
        std::thread::sleep(Duration::from_millis(200));
    }

    // A change of the file asks again (and the sequence leaves the palette).
    std::fs::write(
        repo.root().join(".gitpoulpi/sequences.toml"),
        SEQUENCES.replace("stage_all", "fetch"),
    )
    .unwrap();
    term.wait_for("the notice about the changed sequences", STEP, |s| {
        s.contains("This repository's sequence changed")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.stop().unwrap();
}
