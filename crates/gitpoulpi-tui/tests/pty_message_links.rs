//! Web addresses in the Commit Details pane's message (a bot's
//! `[flate2](https://...)` link): GitPoulpi owns the click. It asks first,
//! showing the whole address (the text is untrusted), then opens it; Esc and
//! Cancel open nothing. The palette's `Commit Details: Open Link in
//! Message...` is the keyboard twin. The browser is never launched:
//! `GITPOULPI_OPEN_LOG` appends the target to a file instead.

mod common;

use std::time::{Duration, Instant};

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(30);
const FLATE: &str = "https://example.test/rust-lang/flate2-rs";
const OTHER: &str = "https://example.test/other/path";

fn start(session: &str, log: &std::path::Path) -> (Terminal, tempfile::TempDir) {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_with_body(
            "chore: bump flate2",
            &format!(
                "Bumps [flate2]({FLATE}) from 1 to 2.\n\nSee also {OTHER}, or javascript:alert(1)."
            ),
        )
        .unwrap()
        .build();
    let started = common::launch(repo.root())
        .session(session)
        .size(200, 50)
        .ready("chore: bump flate2")
        .mouse_capture()
        .env("GITPOULPI_OPEN_LOG", log)
        .start();
    // The fixture repo has to outlive the app: leak it with the process.
    std::mem::forget(repo);
    started
}

fn opened(log: &std::path::Path) -> Vec<String> {
    std::fs::read_to_string(log)
        .unwrap_or_default()
        .lines()
        .map(str::to_string)
        .collect()
}

fn wait_opened(log: &std::path::Path, url: &str) {
    let deadline = Instant::now() + STEP;
    while !opened(log).iter().any(|l| l == url) {
        assert!(Instant::now() < deadline, "{:?}", opened(log));
        std::thread::sleep(Duration::from_millis(100));
    }
}

/// Clicks the address `url` where the message paints it.
fn click_address(term: &mut Terminal, url: &str) {
    term.wait_for("the message with its address", STEP, |s| s.contains(url))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (col, row) = term.screen().find(url).expect("the address on screen");
    term.click(col + 4, row).unwrap();
}

#[test]
fn a_click_on_an_address_asks_then_opens_it_and_cancel_opens_nothing() {
    let log = tempfile::NamedTempFile::new().unwrap();
    let (mut term, _dir) = start("pty-message-links-click", log.path());
    click_address(&mut term, FLATE);
    term.wait_for("the question with the whole address", STEP, |s| {
        s.contains("Open this link?") && s.contains(FLATE)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(
        opened(log.path()).is_empty(),
        "nothing opens before the answer"
    );
    // Esc is Cancel.
    term.send_keys("Escape").unwrap();
    term.wait_for_text_to_go("Open this link?", STEP).unwrap();
    assert!(opened(log.path()).is_empty(), "{:?}", opened(log.path()));
    // Again, and a click on the Open button.
    click_address(&mut term, FLATE);
    term.wait_for_text("Open this link?", STEP).unwrap();
    let (col, row) = term.screen().find("Open (O)").expect("the Open button");
    term.click(col + 1, row).unwrap();
    wait_opened(log.path(), FLATE);
    term.wait_for_text_to_go("Open this link?", STEP).unwrap();
    term.stop().unwrap();
}

#[test]
fn the_palette_twin_lists_the_messages_addresses_and_only_web_ones() {
    let log = tempfile::NamedTempFile::new().unwrap();
    let (mut term, _dir) = start("pty-message-links-palette", log.path());
    term.wait_for("the message", STEP, |s| s.contains(FLATE))
        .unwrap();
    term.send_keys("C-p").unwrap();
    term.wait_for_text("Command Palette", STEP).unwrap();
    term.send_text("Open Link in Message").unwrap();
    term.wait_for_text("Commit Details: Open Link in Message", STEP)
        .unwrap();
    term.send_keys("Enter").unwrap();
    term.wait_for("link 1 of 2", STEP, |s| {
        s.contains("Open this link?") && s.contains(FLATE) && s.contains("Link 1 of 2")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // The message holds three addresses; `javascript:` is not offered ("of 2").
    // Next shows the second, `o` opens it.
    term.send_text("n").unwrap();
    term.wait_for("link 2 of 2", STEP, |s| {
        s.contains("Link 2 of 2") && s.contains(OTHER)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_text("o").unwrap();
    wait_opened(log.path(), OTHER);
    assert_eq!(opened(log.path()), [OTHER]);
    term.stop().unwrap();
}
