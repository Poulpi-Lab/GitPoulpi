//! The movable (and resizable) dialog frame on dialogs other than the Undo
//! history: the Activity Log (move + resize) and the About window (move only),
//! by mouse and by `Ctrl+N m`, end to end through a pty. Every wait describes
//! the whole state it expects (a terminal has no frame boundaries) and every
//! gesture that can land early is retried from the geometry it finds afresh.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::{Screen, Terminal};

const STEP: Duration = Duration::from_secs(30);
const ATTEMPT: Duration = Duration::from_secs(5);

fn launch(session: &str) -> (Terminal, FixtureRepo, tempfile::TempDir) {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "a\n", "frames-commit")
        .unwrap()
        .build();
    let config = tempfile::tempdir().unwrap();
    let term = common::launch(repo.root())
        .session(session)
        .config_dir(config.path())
        .size(200, 56)
        .no_exit_pause()
        .ready("frames-commit")
        .mouse_capture()
        .spawn();
    (term, repo, config)
}

fn wait(term: &mut Terminal, what: &str, f: impl FnMut(&Screen) -> bool) -> Screen {
    term.wait_for(what, STEP, f)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()))
}

fn act_until(
    term: &mut Terminal,
    what: &str,
    act: impl FnMut(&mut Terminal) -> xtask::pty::Result<()>,
    ok: impl FnMut(&Screen) -> bool,
) -> Screen {
    term.act_until(what, ATTEMPT, STEP, act, ok)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()))
}

/// The window title: the first row it appears on (a button of the status bar
/// carries the same words lower down).
fn title_at(s: &Screen, title: &str) -> Option<(u16, u16)> {
    s.find_all(title).into_iter().min_by_key(|(c, r)| (*r, *c))
}

fn grip(s: &Screen) -> Option<(u16, u16)> {
    s.find("\u{25e2}")
}

/// `Ctrl+N` then the letter, reopening the prefix if the key landed early.
fn arrange(term: &mut Terminal) {
    term.send_keys("C-n").unwrap();
    term.send_text("m").unwrap();
}

#[test]
fn the_activity_log_moves_resizes_and_arranges() {
    let (mut term, _repo, _config) = launch("pty-frames-activity");
    term.send_keys("C-n").unwrap();
    term.send_text("A").unwrap();
    let s = wait(&mut term, "the Activity Log with its corner grips", |s| {
        title_at(s, "Activity Log").is_some() && grip(s).is_some()
    });
    let (tc, tr) = title_at(&s, "Activity Log").unwrap();
    let (gx, gy) = grip(&s).unwrap();
    // A title drag: the window, its title and its grip all move together.
    let (dx, dy) = (6u16, 3u16);
    act_until(
        &mut term,
        "the window moved by the title",
        |t| {
            if let Some((c, r)) = title_at(&t.screen(), "Activity Log") {
                if (c, r) == (tc, tr) {
                    t.drag((c + 1, r), (c + 1 + dx, r + dy))?;
                }
            }
            Ok(())
        },
        |s| {
            title_at(s, "Activity Log") == Some((tc + dx, tr + dy))
                && grip(s) == Some((gx + dx, gy + dy))
        },
    );
    // The corner drag resizes (the top left stays).
    let (gx, gy) = (gx + dx, gy + dy);
    act_until(
        &mut term,
        "the window resized by its corner",
        |t| {
            if let Some(g) = grip(&t.screen()) {
                if g == (gx, gy) {
                    t.drag(g, (gx + 4, gy + 2))?;
                }
            }
            Ok(())
        },
        |s| {
            grip(s) == Some((gx + 4, gy + 2))
                && title_at(s, "Activity Log") == Some((tc + dx, tr + dy))
        },
    );
    // The keyboard twin: Ctrl+N m, then the arrows move it.
    arrange(&mut term);
    wait(&mut term, "the arrange banner", |s| s.contains("Arrange:"));
    term.send_keys("Left").unwrap();
    wait(&mut term, "the window two columns left", |s| {
        title_at(s, "Activity Log") == Some((tc + dx - 2, tr + dy)) && s.contains("Arrange:")
    });
    term.send_keys("Enter").unwrap();
    wait(&mut term, "the arrange mode left", |s| {
        !s.contains("Arrange:") && title_at(s, "Activity Log") == Some((tc + dx - 2, tr + dy))
    });
    term.send_keys("Escape").unwrap();
    wait(&mut term, "the window closed", |s| grip(s).is_none());
    term.stop().unwrap();
}

#[test]
fn the_about_window_moves_by_title_and_by_keyboard_but_has_no_resize_grip() {
    let (mut term, _repo, _config) = launch("pty-frames-about");
    term.send_keys("C-n").unwrap();
    term.send_text("a").unwrap();
    let s = wait(&mut term, "the About window", |s| {
        title_at(s, "About GitPoulpi").is_some()
    });
    assert!(
        grip(&s).is_none(),
        "a movable-only window has no corner grip"
    );
    let (tc, tr) = title_at(&s, "About GitPoulpi").unwrap();
    let (dx, dy) = (8u16, 2u16);
    act_until(
        &mut term,
        "the window moved by the title",
        |t| {
            if let Some((c, r)) = title_at(&t.screen(), "About GitPoulpi") {
                if (c, r) == (tc, tr) {
                    t.drag((c + 1, r), (c + 1 + dx, r + dy))?;
                }
            }
            Ok(())
        },
        |s| title_at(s, "About GitPoulpi") == Some((tc + dx, tr + dy)) && grip(s).is_none(),
    );
    // Ctrl+N m over a dialog with no badges of its own.
    arrange(&mut term);
    wait(&mut term, "the arrange banner", |s| s.contains("Arrange:"));
    term.send_keys("Right").unwrap();
    term.send_keys("Down").unwrap();
    wait(&mut term, "the window moved by the arrows", |s| {
        title_at(s, "About GitPoulpi") == Some((tc + dx + 2, tr + dy + 1)) && s.contains("Arrange:")
    });
    term.send_keys("Escape").unwrap();
    wait(
        &mut term,
        "the arrange mode left and the window kept",
        |s| {
            !s.contains("Arrange:")
                && title_at(s, "About GitPoulpi") == Some((tc + dx + 2, tr + dy + 1))
        },
    );
    // Esc again closes it: the arrange mode did not swallow the dialog's key.
    term.send_keys("Escape").unwrap();
    wait(&mut term, "About closed", |s| {
        title_at(s, "About GitPoulpi").is_none()
    });
    term.stop().unwrap();
}
