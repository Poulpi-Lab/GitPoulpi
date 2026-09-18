//! A pane divider has a keyboard twin: the palette widens and narrows the
//! Refs and the Status pane, a notch at a time. Temp config dirs only.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::{Screen, Terminal};

const STEP: Duration = Duration::from_secs(30);

/// The x of a title on the Log header row (the row of `Message`).
fn header_x(screen: &Screen, title: &str) -> Option<u16> {
    let (_, row) = screen.find_all("Message").into_iter().next()?;
    screen
        .find_all(title)
        .into_iter()
        .find(|(_, r)| *r == row)
        .map(|(c, _)| c)
}

fn status_x(screen: &Screen) -> Option<u16> {
    screen.find_all("Commit details").first().map(|(x, _)| *x)
}

/// Runs a palette command by (part of) its title.
fn palette(term: &mut Terminal, title: &str) {
    term.send_keys("C-p").unwrap();
    term.send_text(title).unwrap();
    term.wait_for("the command listed", STEP, |s| s.contains(title))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
}

#[test]
fn the_palette_widens_and_narrows_the_side_panes() {
    let config = tempfile::tempdir().unwrap();
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("notch commit")
        .unwrap()
        .build();
    let mut term = common::launch(repo.root())
        .session("pane-notches")
        .config_dir(config.path())
        .size(200, 60)
        .ready_when("the fixture loaded", |s| s.contains("notch commit"))
        .spawn();
    term.wait_for("the Log header", STEP, |s| header_x(s, "Ref").is_some())
        .unwrap_or_else(|e| panic!("{e}"));
    let log_x = header_x(&term.screen(), "Ref").unwrap();
    let right_x = status_x(&term.screen()).unwrap();

    palette(&mut term, "Widen Refs Pane");
    term.wait_for("the Log starts further right", STEP, |s| {
        header_x(s, "Ref").is_some_and(|x| x > log_x)
    })
    .unwrap_or_else(|e| panic!("{e}"));
    palette(&mut term, "Narrow Refs Pane");
    term.wait_for("the Log starts where it did", STEP, |s| {
        header_x(s, "Ref") == Some(log_x)
    })
    .unwrap_or_else(|e| panic!("{e}"));

    palette(&mut term, "Widen Status Pane");
    term.wait_for("the Status pane starts further left", STEP, |s| {
        status_x(s).is_some_and(|x| x < right_x)
    })
    .unwrap_or_else(|e| panic!("{e}"));
    palette(&mut term, "Narrow Status Pane");
    term.wait_for("the Status pane starts where it did", STEP, |s| {
        status_x(s) == Some(right_x)
    })
    .unwrap_or_else(|e| panic!("{e}"));

    term.send_keys("q").unwrap();
    for _ in 0..100 {
        if term.exited() {
            break;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    let _ = term.stop();
}

fn launch_notch(repo: &std::path::Path, config: &std::path::Path) -> Terminal {
    let term = common::launch(repo)
        .session("pane-notch-keys")
        .config_dir(config)
        .size(200, 60)
        .ready_when("the fixture loaded", |s| s.contains("notch commit"))
        .spawn();
    term.wait_for("the Log header", STEP, |s| header_x(s, "Ref").is_some())
        .unwrap_or_else(|e| panic!("{e}"));
    term
}

fn quit(mut term: Terminal) {
    term.send_keys("q").unwrap();
    for _ in 0..100 {
        if term.exited() {
            break;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    let _ = term.stop();
}

fn press(term: &mut Terminal, key: &str) {
    term.send_keys("C-n").unwrap();
    term.send_keys(key).unwrap();
}

#[test]
fn ctrl_n_arrows_move_the_dividers_and_the_layout_is_kept() {
    let config = tempfile::tempdir().unwrap();
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("notch commit")
        .unwrap()
        .build();
    let mut term = launch_notch(repo.root(), config.path());
    let log_x = header_x(&term.screen(), "Ref").unwrap();
    let right_x = status_x(&term.screen()).unwrap();

    // Right moves the Refs | Log divider to the right: the Log starts later.
    press(&mut term, "Right");
    term.wait_for("the Log starts further right", STEP, |s| {
        header_x(s, "Ref").is_some_and(|x| x > log_x)
    })
    .unwrap_or_else(|e| panic!("{e}"));
    let wider_log_x = header_x(&term.screen(), "Ref").unwrap();
    assert_eq!(status_x(&term.screen()), Some(right_x));

    // Shift+Left moves the Log | Status divider to the left: Status widens.
    press(&mut term, "S-Left");
    term.wait_for("the Status pane starts further left", STEP, |s| {
        status_x(s).is_some_and(|x| x < right_x)
    })
    .unwrap_or_else(|e| panic!("{e}"));
    let wider_right_x = status_x(&term.screen()).unwrap();

    // Both are saved: a new run of the same session starts with them.
    quit(term);
    let mut term = launch_notch(repo.root(), config.path());
    assert_eq!(header_x(&term.screen(), "Ref"), Some(wider_log_x));
    assert_eq!(status_x(&term.screen()), Some(wider_right_x));

    // Left and Shift+Right undo them.
    press(&mut term, "Left");
    term.wait_for("the Log starts where it did", STEP, |s| {
        header_x(s, "Ref") == Some(log_x)
    })
    .unwrap_or_else(|e| panic!("{e}"));
    press(&mut term, "S-Right");
    term.wait_for("the Status pane starts where it did", STEP, |s| {
        status_x(s) == Some(right_x)
    })
    .unwrap_or_else(|e| panic!("{e}"));
    quit(term);
}
