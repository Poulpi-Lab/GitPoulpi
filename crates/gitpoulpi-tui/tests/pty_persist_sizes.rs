//! Every size the user drags is remembered: across a tab switch (the window
//! is rebuilt) and across an app restart, and a size saved on a wide
//! terminal still draws on a narrow one. Driven through a pty by
//! `xtask::pty`, the real binary, the real daemon, a throwaway config dir.

mod common;

use std::path::Path;
use std::time::Duration;

use gitpoulpi_testkit::{ConflictFixture, FixtureRepoBuilder};
use xtask::pty::{Screen, Terminal};

const STEP: Duration = Duration::from_secs(30);

fn quit(mut term: Terminal) {
    // Let the tick that saves a change run before the process goes.
    let _ = term.wait_for_stable(Duration::from_millis(500), STEP);
    term.send_keys("q").unwrap();
    for _ in 0..100 {
        if term.exited() {
            break;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    let _ = term.stop();
}

/// The first column inside the Log pane (the Refs/Log divider moves it). The
/// Log has no title: its top-left corner touches the Refs pane's top-right one.
fn log_left(s: &Screen) -> u16 {
    s.find("\u{256e}\u{256d}")
        .map(|(c, _)| c + 2)
        .expect("the Refs/Log corners")
}

/// [`log_left`] equals `want`, without panicking on a frame that lacks the
/// corners (a torn frame): a wait predicate.
fn log_left_is(s: &Screen, want: u16) -> bool {
    s.find("\u{256e}\u{256d}")
        .is_some_and(|(c, _)| c + 2 == want)
}

fn launch_main(repo: &Path, config: &Path, cols: u16, rows: u16, ready: &str) -> Terminal {
    common::launch(repo)
        .session("persist-main")
        .config_dir(config)
        .size(cols, rows)
        .no_exit_pause()
        .ready(ready)
        .mouse_capture()
        .spawn()
}

/// A Refs/Log divider drag survives a tab switch (in both directions) and a
/// restart, and a narrower terminal clamps it instead of breaking.
#[test]
fn the_pane_divider_survives_a_tab_switch_a_restart_and_a_small_terminal() {
    let alpha = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "a\n", "alpha-only-commit")
        .unwrap()
        .build();
    let bravo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("b.txt", "b\n", "bravo-only-commit")
        .unwrap()
        .build();
    let config = tempfile::tempdir().unwrap();
    std::fs::write(
        config.path().join("recent.toml"),
        format!(
            "[[repos]]\npath = '{}'\nname = 'bravo-repo'\nlast_opened = 1\n",
            bravo.root().display()
        ),
    )
    .unwrap();

    let mut term = launch_main(alpha.root(), config.path(), 140, 36, "alpha-only-commit");
    let default_left = log_left(
        &term
            .wait_for_stable(Duration::from_millis(400), STEP)
            .unwrap(),
    );
    // Refs ends at 20% of the panes row; drag its divider well to the right.
    let divider = default_left - 1;
    term.drag((divider, 12), (divider + 24, 12)).unwrap();
    term.wait_for("the Log moved right", STEP, |s| {
        s.find("\u{256e}\u{256d}")
            .is_some_and(|(c, _)| c + 2 > default_left + 10)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // The drag's last frame: read once the window stopped redrawing.
    let moved = term
        .wait_for_stable(Duration::from_millis(400), STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let moved_left = log_left(&moved);

    // Open bravo as a second tab: the window is rebuilt, the size stays.
    term.send_keys("C-o").unwrap();
    term.wait_for("the management tab", STEP, |s| {
        s.contains("Repository Management") && s.contains("bravo-repo")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("/").unwrap();
    term.send_text("bravo").unwrap();
    term.wait_for("the filter applied", STEP, |s| {
        s.contains("Enter open best match")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    // One predicate for the whole state: bravo drawn with the dragged size.
    let in_bravo = term
        .wait_for("bravo open at the dragged size", STEP, |s| {
            s.contains("bravo-only-commit")
                && !s.contains("Repository Management")
                && log_left_is(s, moved_left)
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert_eq!(log_left(&in_bravo), moved_left, "{}", term.text());

    // Ctrl+PageUp back to alpha, Ctrl+PageDown to bravo again.
    term.send_text("\x1b[5;5~").unwrap();
    let back = term
        .wait_for("alpha active at the dragged size", STEP, |s| {
            s.contains("alpha-only-commit")
                && !s.contains("bravo-only-commit")
                && log_left_is(s, moved_left)
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert_eq!(log_left(&back), moved_left, "{}", term.text());
    quit(term);

    // Restart: same size.
    let term = launch_main(alpha.root(), config.path(), 140, 36, "alpha-only-commit");
    let again = term
        .wait_for_stable(Duration::from_millis(400), STEP)
        .unwrap();
    assert_eq!(log_left(&again), moved_left, "{}", term.text());
    quit(term);

    // A much narrower terminal: it still draws, with the Refs pane
    // proportionally narrower (a ratio, not absolute cells).
    let term = launch_main(alpha.root(), config.path(), 70, 24, "alpha-only-commit");
    let small = term
        .wait_for_stable(Duration::from_millis(400), STEP)
        .unwrap();
    assert!(
        log_left(&small) < 70 && log_left(&small) < moved_left,
        "{}",
        term.text()
    );
    assert!(small.contains("alpha-only-commit"), "{}", term.text());
    quit(term);
}

fn launch_merge(repo: &Path, config: &Path, cols: u16, rows: u16) -> Terminal {
    let mut term = common::launch(repo)
        .session("persist-merge")
        .config_dir(config)
        .size(cols, rows)
        .no_exit_pause()
        .ready_when("the conflicted file in the Status pane", |s| {
            s.contains("Unstaged") && s.contains("Merge in progre")
        })
        .mouse_capture()
        .spawn();
    let (c, r) = term
        .wait_for_text("a.txt", STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.click(c, r).unwrap();
    term.send_keys("Enter").unwrap();
    // The title bar, the A | B divider and the Output's title are what the
    // callers measure, and they are drawn after the toolbar's counter: all of
    // them are part of "the merge editor is up".
    term.wait_for("the merge editor", STEP, |s| {
        s.contains("Conflict 1 / 3") && s.find("\u{252c}").is_some() && output_title_at(s).is_some()
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term
}

fn output_title_at(s: &Screen) -> Option<u16> {
    (0..s.rows()).find(|r| s.line(*r).contains("Output \u{b7}"))
}

fn divider_at(s: &Screen) -> (u16, u16) {
    s.find("\u{252c}").unwrap_or((0, 0))
}

fn output_title_row(s: &Screen) -> u16 {
    (0..s.rows())
        .find(|r| s.line(*r).contains("Output \u{b7}"))
        .expect("the Output's title")
}

/// The merge editor's A | B divider and Output splitter survive closing and
/// reopening the editor, a restart and a smaller terminal; `Reset Layout`
/// clears them on disk too.
#[test]
fn the_merge_editor_sizes_survive_a_restart_and_reset_layout_clears_them() {
    let fixture = ConflictFixture::multi_conflict();
    let config = tempfile::tempdir().unwrap();

    let mut term = launch_merge(fixture.root(), config.path(), 200, 50);
    let (d0, row) = divider_at(&term.screen());
    let o0 = output_title_row(&term.screen());
    term.drag((d0, row + 3), (d0 - 30, row + 3)).unwrap();
    term.wait_for("the divider moved", STEP, |s| divider_at(s).0 == d0 - 30)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.drag((60, o0), (60, o0 - 6)).unwrap();
    let s = term
        .wait_for("the splitter moved", STEP, |s| {
            output_title_at(s).is_some_and(|t| t < o0)
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let o1 = output_title_row(&s);
    quit(term);

    // Restart: the editor opens with the dragged sizes.
    // (The sizes are applied on the frame after the editor opens, so the
    // wait is for the dragged state itself, not for a quiet screen.)
    let term = launch_merge(fixture.root(), config.path(), 200, 50);
    term.wait_for("the dragged sizes back", STEP, |s| {
        divider_at(s).0 == d0 - 30 && output_title_at(s) == Some(o1)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    quit(term);

    // A smaller terminal: the editor still lays out (the sizes are ratios).
    let term = launch_merge(fixture.root(), config.path(), 120, 36);
    term.wait_for("the editor laid out in the smaller terminal", STEP, |s| {
        let (d, _) = divider_at(s);
        d > 0 && d < 120 - 14 && output_title_at(s).is_some_and(|t| t < 36)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    quit(term);

    // Reset Layout: back to the defaults now and on the next run.
    let mut term = launch_merge(fixture.root(), config.path(), 200, 50);
    term.send_keys("C-p").unwrap();
    term.send_text("Reset Layout").unwrap();
    term.wait_for("the palette entry", STEP, |s| {
        s.contains("Navigation: Reset Layout")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    term.wait_for("the defaults back", STEP, |s| {
        divider_at(s).0 == d0 && output_title_at(s) == Some(o0)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    quit(term);
    let term = launch_merge(fixture.root(), config.path(), 200, 50);
    term.wait_for("the default sizes again", STEP, |s| {
        divider_at(s).0 == d0 && output_title_at(s) == Some(o0)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    quit(term);
}
