//! The thematic Settings pages, through a pty: every page of the regrouping
//! opens from its sidebar entry and shows the rows that moved onto it, and
//! the page that draws its own body (Integrations) keeps
//! its generic rows reachable from the keyboard.

#![cfg(unix)]

mod common;

use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(30);

fn open_settings(session: &str) -> (Terminal, tempfile::TempDir, gitpoulpi_testkit::FixtureRepo) {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("settings pages probe")
        .unwrap()
        .build();
    let (mut term, config) = common::launch(repo.root())
        .session(session)
        .size(200, 60)
        .ready("settings pages probe")
        .start();
    let (c, r) = term
        .wait_for_text("\u{2699}", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(c, r).expect("click the gear");
    term.wait_for("Settings open", STEP, |s| {
        s.contains("Repo-specific settings")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    (term, config, repo)
}

#[test]
fn every_regrouped_page_opens_and_shows_its_rows() {
    let (mut term, _config, _repo) = open_settings("pty-settings-pages");
    for (title, row) in [
        ("General", "Default branch name"),
        ("Synchronization", "Auto-prune"),
        ("SSH & credentials", "Forget all usernames and"),
        ("Integrations", "Review request terminology"),
        ("Agents & MCP", "Allow AI agents to drive the"),
        ("Appearance", "Date/time locale"),
        ("Log & graph", "Graph renderer"),
        ("Diff & editor", "Tab size"),
        ("Open in", "Follow the active tab"),
        ("Keyboard & mouse", "Mouse enabled"),
    ] {
        common::settings_page(&mut term, title);
        common::scroll_settings_to(&mut term, row, STEP);
    }
    // The removed rows are nowhere.
    for title in ["General", "Appearance", "Keyboard & mouse"] {
        common::settings_page(&mut term, title);
        let text = term.text();
        assert!(!text.contains("Keymap preset"), "{title}: {text}");
        assert!(!text.contains("Graph colorisation"), "{title}: {text}");
    }
}

#[test]
fn the_mouse_rows_are_the_first_tab_of_keyboard_and_mouse_and_space_ticks_one() {
    let (mut term, config, _repo) = open_settings("pty-settings-strip");
    common::settings_page(&mut term, "Keyboard & mouse");
    // Into the page (its first tab): Kitty keyboard, Mouse enabled, Mouse
    // wheel lines.
    term.send_keys("Enter").unwrap();
    term.send_keys("Down").unwrap();
    term.send_keys("Space").unwrap();
    let deadline = std::time::Instant::now() + STEP;
    loop {
        let stored = std::fs::read_to_string(config.path().join("config.toml")).unwrap_or_default();
        if stored.contains("mouse_enabled = false") {
            break;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "Space on Mouse enabled never reached config.toml: {stored}\n{}",
            term.text()
        );
        std::thread::sleep(Duration::from_millis(150));
    }
}

/// The scrollbar's down arrow on the right of the settings body: the last
/// `↓` on screen (the bar's own glyph, below everything else on the page).
fn down_arrow(term: &Terminal) -> (u16, u16) {
    let arrow = |s: &xtask::pty::Screen| {
        (0..s.rows())
            .rev()
            .flat_map(|r| (0..s.cols()).rev().map(move |c| (c, r)))
            .find(|&(c, r)| s.cell(c, r).is_some_and(|x| x.text == "\u{2193}"))
    };
    // Waited for: a torn frame can lack the bar.
    let s = term
        .wait_for("the scrollbar's arrow", Duration::from_secs(30), |s| {
            arrow(s).is_some()
        })
        .unwrap_or_else(|e| panic!("no scrollbar arrow: {e}\n{}", term.text()));
    arrow(&s).expect("the predicate just matched")
}

#[test]
fn a_long_settings_page_has_a_scrollbar_whose_thumb_drags_the_page() {
    let (mut term, _config, _repo) = open_settings("pty-settings-scrollbar");
    common::settings_page(&mut term, "Appearance");
    term.wait_for("the Appearance page from its top", STEP, |s| {
        s.contains("Colors of the whole interface")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (c, r) = down_arrow(&term);
    // The thumb sits just under the up arrow at the top: drag it down to
    // the bottom of the track.
    let up = (0..r)
        .rev()
        .find(|&y| {
            term.screen()
                .cell(c, y)
                .is_some_and(|x| x.text == "\u{2191}")
        })
        .expect("the bar's up arrow");
    term.drag((c, up + 1), (c, r - 1)).unwrap();
    term.wait_for("the page scrolled to its end", STEP, |s| {
        !s.contains("Colors of the whole interface") && s.contains("Appearance")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

#[test]
fn the_keyboard_page_list_has_a_scrollbar_and_a_track_click_jumps() {
    let (mut term, _config, _repo) = open_settings("pty-settings-keys-scrollbar");
    common::settings_page(&mut term, "Keyboard & mouse");
    common::settings_tab(&mut term, "Keybindings");
    term.wait_for("the shortcut list from its top", STEP, |s| {
        s.contains("Git Actions Prefix") && s.contains("Command Palette")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (c, r) = down_arrow(&term);
    term.click(c, r - 1).unwrap();
    term.wait_for("the list jumped towards its end", STEP, |s| {
        !s.contains("Git Actions Prefix") && s.contains("Keyboard & mouse")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}
