//! `git.default_encoding` in the TUI: a Latin-1 commit message shows
//! decoded in the Log and the commit detail instead of as U+FFFD.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;

const STEP: Duration = Duration::from_secs(30);

#[test]
fn a_latin1_commit_message_is_decoded_with_the_configured_encoding() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "a\n", "base")
        .unwrap()
        .build();
    repo.commit_raw_message(b"caf\xe9 cr\xe8me\n\nun corps \xe0 lire\n", None);

    let (term, _config) = common::launch(repo.root())
        .session("pty-commit-encoding")
        .config("[git]\ndefault_encoding = \"windows-1252\"\n")
        .size(200, 40)
        .no_exit_pause()
        .ready_when("the commit detail of HEAD", |s| {
            s.contains("View all files") && s.contains("Filter files:")
        })
        .start();
    term.wait_for("the decoded summary and body", STEP, |s| {
        s.contains("café crème") && s.contains("un corps à lire") && !s.contains("\u{fffd}")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.stop().unwrap();
}
