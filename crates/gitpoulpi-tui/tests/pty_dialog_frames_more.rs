//! The dialog frame beyond the Activity Log and About (`pty_dialog_frames.rs`):
//! the command palette (move + resize, kept in `session` mode, its keyboard
//! twin), a resizable Settings editor whose text area grows, and `Ctrl+N m`
//! on the surfaces that have a `Ctrl+N` of their own: the commit message form
//! (badges) and a dialog of the history rewrite page. Every wait describes
//! the whole state it expects (a terminal has no frame boundaries) and every
//! gesture that can land early is retried from the geometry it finds afresh.

#![cfg(unix)]

mod common;

use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::{Screen, Terminal};

const STEP: Duration = Duration::from_secs(30);
const ATTEMPT: Duration = Duration::from_secs(5);

fn launch_in(repo: &FixtureRepo, session: &str, ready: &str) -> (Terminal, tempfile::TempDir) {
    let config = tempfile::tempdir().unwrap();
    let term = common::launch(repo.root())
        .session(session)
        .config_dir(config.path())
        .size(200, 56)
        .no_exit_pause()
        .ready(ready)
        .mouse_capture()
        .spawn();
    (term, config)
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

/// The title: the first row it appears on.
fn title_at(s: &Screen, title: &str) -> Option<(u16, u16)> {
    s.find_all(title).into_iter().min_by_key(|(c, r)| (*r, *c))
}

fn grip(s: &Screen) -> Option<(u16, u16)> {
    s.find("\u{25e2}")
}

/// Rows of the palette's list that carry a key button.
fn palette_rows(s: &Screen) -> usize {
    s.text().lines().filter(|l| l.contains("[k]")).count()
}

#[test]
fn the_palette_moves_resizes_and_keeps_its_place_when_reopened() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "a\n", "frames-commit")
        .unwrap()
        .build();
    let (mut term, _config) = launch_in(&repo, "pty-frames-palette", "frames-commit");
    term.send_keys("C-p").unwrap();
    let s = wait(&mut term, "the palette with its corner grip", |s| {
        title_at(s, "Command Palette").is_some() && grip(s).is_some() && palette_rows(s) > 5
    });
    let (tc, tr) = title_at(&s, "Command Palette").unwrap();
    let (gx, gy) = grip(&s).unwrap();
    let rows = palette_rows(&s);
    // The top border moves it: the title, the grip and the rows follow.
    let (dx, dy) = (6u16, 3u16);
    act_until(
        &mut term,
        "the palette moved by its title",
        |t| {
            if title_at(&t.screen(), "Command Palette") == Some((tc, tr)) {
                t.drag((tc + 1, tr), (tc + 1 + dx, tr + dy))?;
            }
            Ok(())
        },
        |s| {
            title_at(s, "Command Palette") == Some((tc + dx, tr + dy))
                && grip(s) == Some((gx + dx, gy + dy))
                && palette_rows(s) == rows
        },
    );
    // The corner grows it: the list shows more rows.
    let (gx, gy) = (gx + dx, gy + dy);
    let (gw, gh) = (4u16, 3u16);
    act_until(
        &mut term,
        "the palette resized by its corner, its list longer",
        |t| {
            if grip(&t.screen()) == Some((gx, gy)) {
                t.drag((gx, gy), (gx + gw, gy + gh))?;
            }
            Ok(())
        },
        |s| {
            grip(s) == Some((gx + gw, gy + gh))
                && title_at(s, "Command Palette") == Some((tc + dx, tr + dy))
                && palette_rows(s) == rows + gh as usize
        },
    );
    // Closed and reopened (`session`): the same place and size.
    term.send_keys("Escape").unwrap();
    wait(&mut term, "the palette closed", |s| {
        title_at(s, "Command Palette").is_none()
    });
    term.send_keys("C-p").unwrap();
    wait(&mut term, "the palette back where it was left", |s| {
        title_at(s, "Command Palette") == Some((tc + dx, tr + dy))
            && grip(s) == Some((gx + gw, gy + gh))
            && palette_rows(s) == rows + gh as usize
    });
    // The keyboard twin (`palette.arrange`, Alt+M): the arrows move it while
    // it stays open, Esc leaves the mode and a second Esc closes it.
    term.send_keys("M-m").unwrap();
    wait(&mut term, "the arrange banner", |s| s.contains("Arrange:"));
    term.send_keys("Left").unwrap();
    wait(&mut term, "the palette two columns left", |s| {
        title_at(s, "Command Palette") == Some((tc + dx - 2, tr + dy)) && s.contains("Arrange:")
    });
    term.send_keys("Escape").unwrap();
    wait(&mut term, "the arrange mode left, the palette open", |s| {
        !s.contains("Arrange:") && title_at(s, "Command Palette") == Some((tc + dx - 2, tr + dy))
    });
    term.send_keys("Escape").unwrap();
    wait(&mut term, "the palette closed", |s| {
        title_at(s, "Command Palette").is_none()
    });
    term.stop().unwrap();
}

/// The palette row "Dialog: Move or Resize" arranges the palette in front and
/// leaves it open; "Dialog: Reset Position and Size" puts it back.
#[test]
fn the_palette_rows_arrange_and_reset_the_palette_itself() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "a\n", "frames-commit")
        .unwrap()
        .build();
    let (mut term, _config) = launch_in(&repo, "pty-frames-palette-rows", "frames-commit");
    term.send_keys("C-p").unwrap();
    let s = wait(&mut term, "the palette", |s| {
        title_at(s, "Command Palette").is_some() && grip(s).is_some()
    });
    let (tc, tr) = title_at(&s, "Command Palette").unwrap();
    term.send_text("dialog: move or resize").unwrap();
    wait(&mut term, "the row", |s| s.contains("Dialog: Move or Resize"));
    term.send_keys("Enter").unwrap();
    wait(&mut term, "the palette in the arrange mode, still open", |s| {
        s.contains("Arrange:") && title_at(s, "Command Palette").is_some()
    });
    term.send_keys("Down").unwrap();
    term.send_keys("Right").unwrap();
    wait(&mut term, "the palette moved by the arrows", |s| {
        title_at(s, "Command Palette").is_some_and(|(c, r)| (c, r) != (tc, tr))
    });
    term.send_keys("0").unwrap();
    wait(&mut term, "the palette back at its default place", |s| {
        title_at(s, "Command Palette") == Some((tc, tr))
    });
    term.send_keys("Escape").unwrap();
    wait(&mut term, "the arrange mode left", |s| !s.contains("Arrange:"));
    term.send_keys("Escape").unwrap();
    wait(&mut term, "the palette closed", |s| {
        title_at(s, "Command Palette").is_none()
    });
    term.stop().unwrap();
}

/// A resizable Settings editor: the text area takes the rows it gains.
#[cfg(feature = "ai")]
#[test]
fn resizing_the_settings_editor_grows_its_text_area() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "initial commit")
        .unwrap()
        .build();
    let config = tempfile::tempdir().unwrap();
    std::fs::write(config.path().join("config.toml"), "[ai]\nenabled = true\n").unwrap();
    let mut term = common::launch(repo.root())
        .session("pty-frames-editor")
        .config_dir(config.path())
        .size(200, 56)
        .no_exit_pause()
        .ready("initial commit")
        .mouse_capture()
        .spawn();
    let (c, r) = term.wait_for_text("\u{2699}", STEP).unwrap();
    term.click(c, r).unwrap();
    wait(&mut term, "Settings", |s| {
        s.contains("Repo-specific settings")
    });
    common::settings_page(&mut term, "AI");
    let (c, r) = term.wait_for_text("Custom instructions", STEP).unwrap();
    term.click(c + 1, r).unwrap();
    term.send_keys("Enter").unwrap();
    wait(&mut term, "the editor", |s| {
        s.contains("Save") && s.contains("Cancel") && grip(s).is_some()
    });
    // Many short lines, so the rows of the area can be counted.
    for i in 0..70 {
        term.send_text(&format!("row{i:02}")).unwrap();
        term.send_keys("Enter").unwrap();
    }
    let count = |s: &Screen| s.text().lines().filter(|l| l.contains("row")).count();
    // From the top: the area shows as many rows of the text as it is tall.
    term.send_keys(&"Up ".repeat(75)).unwrap();
    let s = wait(&mut term, "the first rows of the text shown", |s| {
        s.contains("row00") && s.contains("row05") && count(s) > 5
    });
    let before = count(&s);
    let (gx, gy) = grip(&s).unwrap();
    let grow = 3u16;
    act_until(
        &mut term,
        "the editor resized by its corner, more text rows shown",
        |t| {
            if grip(&t.screen()) == Some((gx, gy)) {
                t.drag((gx, gy), (gx, gy + grow))?;
            }
            Ok(())
        },
        |s| grip(s) == Some((gx, gy + grow)) && s.contains("row00") && count(s) == before + 3,
    );
    term.stop().unwrap();
}

/// `Ctrl+N m` in the commit message form, which badges its controls on its
/// own `Ctrl+N`: `m` starts the arrange mode, the hint names it.
#[test]
fn ctrl_n_m_arranges_the_commit_message_form() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("first reword target")
        .unwrap()
        .commit("second on top")
        .unwrap()
        .build();
    let (mut term, _config) = launch_in(&repo, "pty-frames-reword", "second on top");
    let (col, row) = term.wait_for_text("first reword target", STEP).unwrap();
    term.right_click(col + 1, row).unwrap();
    let menu = wait(&mut term, "the commit menu", |s| {
        s.contains("Edit commit message...")
    });
    let (c, r) = menu.find("Edit commit message...").unwrap();
    term.click(c + 1, r).unwrap();
    let s = wait(&mut term, "the form", |s| {
        s.contains("Reword commit message") && s.contains("Update message")
    });
    let (tc, tr) = title_at(&s, "Reword commit message").unwrap();
    term.send_keys("C-n").unwrap();
    wait(&mut term, "the badges and the hint naming m", |s| {
        s.contains("m=arrange") && s.contains("<badge>=")
    });
    term.send_text("m").unwrap();
    wait(&mut term, "the arrange banner", |s| s.contains("Arrange:"));
    term.send_keys("Right").unwrap();
    wait(&mut term, "the form two columns right", |s| {
        title_at(s, "Reword commit message") == Some((tc + 2, tr)) && s.contains("Arrange:")
    });
    term.send_keys("Escape").unwrap();
    wait(&mut term, "the mode left, the form kept", |s| {
        !s.contains("Arrange:") && title_at(s, "Reword commit message") == Some((tc + 2, tr))
    });
    // The badges are still the form's own: Ctrl+N then a badge presses it.
    term.send_keys("Escape").unwrap();
    wait(&mut term, "the form closed", |s| {
        !s.contains("Reword commit message")
    });
    term.stop().unwrap();
}

/// `Ctrl+N m` over a dialog of the history rewrite page (its question): the
/// page's own `Ctrl+N` badges nothing there, so the frame's prefix is used.
#[test]
fn ctrl_n_m_arranges_a_history_rewrite_dialog() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("lib/a.txt", "a1\n", "add lib")
        .unwrap()
        .commit_file("other/x.txt", "x\n", "other work")
        .unwrap()
        .commit_file("lib/a.txt", "a2\n", "edit lib")
        .unwrap()
        .build();
    let (mut term, _config) = launch_in(&repo, "pty-frames-rewrite", "edit lib");
    term.send_keys("C-p").unwrap();
    term.send_text("move or rename paths").unwrap();
    wait(&mut term, "the palette row", |s| {
        s.contains("Rewrite: Move or Rename Paths...")
    });
    term.send_keys("Enter").unwrap();
    wait(&mut term, "the rule's fields", |s| {
        s.contains("Rewrite history") && s.contains("Rename a path")
    });
    term.send_keys("Enter").unwrap();
    wait(&mut term, "the kinds", |s| {
        s.contains("Extract a folder as the root")
    });
    common::retry_until(
        &mut term,
        "the Folder field",
        4,
        Duration::from_secs(10),
        |t| {
            if let Some((c, r)) = t.screen().find("Extract a folder as the root") {
                t.click(c + 1, r).expect("click the kind");
            }
        },
        |s| s.contains("Folder"),
    );
    term.send_keys("Down").unwrap();
    term.send_keys("Enter").unwrap();
    term.send_text("lib").unwrap();
    term.send_keys("Enter").unwrap();
    term.send_keys("F5").unwrap();
    let s = wait(&mut term, "the extract question", |s| {
        s.contains("How should the folder be extracted?")
            && s.contains("Into a new clone")
            && title_at(s, "Extract a folder \u{2500}").is_some()
    });
    let (tc, tr) = title_at(&s, "Extract a folder \u{2500}").unwrap();
    term.send_keys("C-n").unwrap();
    term.send_text("m").unwrap();
    wait(&mut term, "the arrange banner", |s| s.contains("Arrange:"));
    term.send_keys("Right").unwrap();
    term.send_keys("Down").unwrap();
    wait(&mut term, "the question moved by the arrows", |s| {
        title_at(s, "Extract a folder \u{2500}") == Some((tc + 2, tr + 1)) && s.contains("Arrange:")
    });
    term.send_keys("Escape").unwrap();
    wait(&mut term, "the mode left, the question kept", |s| {
        !s.contains("Arrange:")
            && title_at(s, "Extract a folder \u{2500}") == Some((tc + 2, tr + 1))
            && s.contains("How should the folder be extracted?")
    });
    term.stop().unwrap();
}
