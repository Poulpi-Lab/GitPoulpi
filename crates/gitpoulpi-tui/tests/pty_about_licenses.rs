//! About > Licenses: the third-party notices and credits built into the
//! binary, scrollable by keyboard and wheel, reachable from the palette.
//!
//! Temp config dir and fixture repo only.

#![cfg(unix)]

mod common;

use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(30);

fn start(session: &str) -> (Terminal, tempfile::TempDir, gitpoulpi_testkit::FixtureRepo) {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "a\n", "first-commit")
        .unwrap()
        .build();
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("config.toml"),
        "[ui]\ngraph_renderer = \"text\"\n",
    )
    .unwrap();
    let term = common::launch(repo.root())
        .session(session)
        .config_dir(dir.path())
        .size(160, 50)
        .no_exit_pause()
        .ready("first-commit")
        .spawn();
    (term, dir, repo)
}

/// Pages down until `needle` is on screen.
fn page_until(term: &mut Terminal, needle: &str) {
    // Each page is given time to be drawn whole (a torn frame can hide the
    // needle) before the next PageDown.
    for _ in 0..60 {
        if term
            .wait_for(needle, Duration::from_millis(1500), |s| s.contains(needle))
            .is_ok()
        {
            return;
        }
        term.send_keys("PageDown").unwrap();
    }
    panic!("never saw {needle}:\n{}", term.text());
}

#[test]
fn the_licenses_view_shows_the_credits_libgit2_and_the_crates() {
    let (mut term, _dir, _repo) = start("about-licenses-keys");
    // Retried: the octopus can be drawn before its hit target is registered.
    common::retry_until(
        &mut term,
        "the About window and its trademark notice",
        6,
        Duration::from_secs(10),
        |t| {
            if let Some((col, row)) = t.screen().find("\u{1F419}") {
                t.click(col, row).unwrap();
            }
        },
        |s| s.contains("About GitPoulpi") && s.contains("not affiliated with"),
    );
    term.send_keys("t").unwrap();
    term.wait_for("the notices", STEP, |s| {
        s.contains("third-party licenses and credits") && s.contains("Licenses and credits")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    page_until(&mut term, "libgit2 (statically linked, unmodified)");
    page_until(&mut term, "serde ");
    term.send_keys("Home").unwrap();
    term.wait_for_text("third-party licenses and credits", STEP)
        .unwrap();
    // Esc goes back to the facts, a second Esc closes.
    term.send_keys("Escape").unwrap();
    term.wait_for_text("License: MIT", STEP).unwrap();
    term.send_keys("Escape").unwrap();
    term.wait_for_text_to_go("License: MIT", STEP).unwrap();
    term.stop().unwrap();
}

#[test]
fn the_licenses_view_has_a_mouse_and_a_palette_way_in_and_scrolls_with_the_wheel() {
    let (mut term, _dir, _repo) = start("about-licenses-mouse");
    // The palette twin opens About straight on the notices.
    term.send_keys("C-p").unwrap();
    term.wait_for_text("Command Palette", STEP).unwrap();
    term.send_text("Licenses").unwrap();
    term.wait_for("the entry", STEP, |s| {
        s.contains("Help: Licenses and Credits")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    term.wait_for_text("third-party licenses and credits", STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // The title is the mouse way back.
    let (bc, br) = term.screen().find("Licenses and credits").unwrap();
    term.click(bc, br).unwrap();
    term.wait_for("the facts page and its link", STEP, |s| {
        s.contains("License: MIT") && s.contains("Licenses and credits (t: read)")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // The link opens it again (mouse), and the wheel scrolls it.
    let (lc, lr) = term
        .screen()
        .find("Licenses and credits (t: read)")
        .unwrap();
    term.click(lc + 4, lr).unwrap();
    term.wait_for_text("third-party licenses and credits", STEP)
        .unwrap();
    for _ in 0..40 {
        term.scroll(true, 40, 20).unwrap();
    }
    term.wait_for_text_to_go("third-party licenses and credits", STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}
