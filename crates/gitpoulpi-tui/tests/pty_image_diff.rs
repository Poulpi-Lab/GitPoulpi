//! The Diff view on a picture, end to end through a pty (image view slice S5):
//! before and after side by side or one at a time, a resizable divider, one
//! zoom and scroll shared by both, and the re-adaptation after a resize.
//!
//! `GITPOULPI_TERMINAL_IMAGE=off` is what `common::launch` sets, so the
//! pictures are drawn as half-block cells (`\u{2580}`) whose colors vt100
//! keeps. Solid red is the "before" and solid blue the "after" in every
//! fixture, so which pane (or which side of the inline flip) shows what is
//! read off the cell colors.

mod common;

use std::path::Path;
use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::{Color, Screen, Terminal};

const STEP: Duration = Duration::from_secs(30);
const HALF: &str = "\u{2580}";
const LOG_HEADER: &str = "Message";

const RED: (u8, u8, u8) = (255, 0, 0);
const BLUE: (u8, u8, u8) = (0, 0, 255);

fn solid(w: u32, h: u32, c: (u8, u8, u8)) -> Vec<u8> {
    let mut rgba = Vec::new();
    for _ in 0..w * h {
        rgba.extend_from_slice(&[c.0, c.1, c.2, 255]);
    }
    gitpoulpi_testkit::png_bytes(w, h, &rgba)
}

fn launch(repo: &Path, session: &str, cols: u16) -> (Terminal, tempfile::TempDir) {
    common::launch(repo)
        .no_commit_column()
        .session(session)
        .config("[ui]\ndiff_layout = \"split\"\n")
        .size(cols, 40)
        .no_exit_pause()
        .ready_when("the commit detail of HEAD", |s| {
            s.contains("View all files") && s.contains(LOG_HEADER)
        })
        .mouse_capture()
        .start()
}

/// `size` x `size` pictures: red in the first commit, blue in the second.
fn red_to_blue(size: u32) -> gitpoulpi_testkit::FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_bytes("pic.png", &solid(size, size, RED), "red picture")
        .unwrap()
        .commit_bytes("pic.png", &solid(size, size, BLUE), "blue picture")
        .unwrap()
        .build()
}

fn open_the_diff(term: &mut Terminal) {
    let (c, r) = term
        .wait_for_text("pic.png", STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.click(c, r).unwrap();
    term.wait_for("the diff in the center", STEP, |s| {
        s.contains("\u{2715}") && !s.contains(LOG_HEADER)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

fn near(c: Color, rgb: (u8, u8, u8)) -> bool {
    match c {
        Color::Rgb(r, g, b) => {
            r.abs_diff(rgb.0) < 8 && g.abs_diff(rgb.1) < 8 && b.abs_diff(rgb.2) < 8
        }
        _ => false,
    }
}

/// The columns and rows of the half-block cells painted in `rgb`.
fn cells_of(s: &Screen, rgb: (u8, u8, u8)) -> Vec<(u16, u16)> {
    let mut out = Vec::new();
    for row in 0..s.rows() {
        for col in 0..s.cols() {
            if let Some(c) = s.cell(col, row) {
                if c.text == HALF && near(c.fg, rgb) && near(c.bg, rgb) {
                    out.push((col, row));
                }
            }
        }
    }
    out
}

fn left_of(cells: &[(u16, u16)]) -> u16 {
    cells.iter().map(|c| c.0).min().unwrap()
}

fn right_of(cells: &[(u16, u16)]) -> u16 {
    cells.iter().map(|c| c.0).max().unwrap()
}

fn wait_cells(term: &Terminal, what: &str, rgb: (u8, u8, u8)) -> Vec<(u16, u16)> {
    term.wait_for(what, STEP, |s| !cells_of(s, rgb).is_empty())
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    cells_of(&term.screen(), rgb)
}

/// The red and blue cell counts once they stop changing: the first frames of
/// a picture can be a coarser raster that a later one replaces, so a count
/// read at the first sight of the color is not the settled Fit. Waits for
/// the same non-zero pair on 6 reads 100 ms apart.
fn settled_counts(term: &Terminal) -> (usize, usize) {
    let deadline = std::time::Instant::now() + STEP;
    let (mut last, mut same) = ((0, 0), 0);
    loop {
        let s = term.screen();
        let now = (cells_of(&s, RED).len(), cells_of(&s, BLUE).len());
        if now.0 > 0 && now.1 > 0 && now == last {
            same += 1;
            if same >= 6 {
                return now;
            }
        } else {
            same = 0;
            last = now;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "the pictures never settled ({now:?})\n{}",
            term.text()
        );
        std::thread::sleep(Duration::from_millis(100));
    }
}

/// With `ui.diff_layout = split`: the red before on the left, the blue after
/// on the right, each under its caption. `m` makes it inline (the after),
/// `Tab` flips the one pane to the before and back.
#[test]
fn split_shows_before_left_and_after_right_and_inline_flips_with_tab() {
    let repo = red_to_blue(4);
    let (mut term, _config) = launch(repo.root(), "pty-image-diff-split", 140);
    open_the_diff(&mut term);
    let red = wait_cells(&term, "the before", RED);
    let blue = wait_cells(&term, "the after", BLUE);
    assert!(
        right_of(&red) < left_of(&blue),
        "red left of blue: {red:?} / {blue:?}"
    );
    let text = term.text();
    assert!(text.contains("Before") && text.contains("After"), "{text}");
    assert!(text.contains("PNG 4\u{d7}4"), "the info line: {text}");

    term.send_keys("m").unwrap();
    term.wait_for("inline: the red pane is gone", STEP, |s| {
        cells_of(s, RED).is_empty() && !cells_of(s, BLUE).is_empty()
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Tab").unwrap();
    term.wait_for("flipped to the before", STEP, |s| {
        cells_of(s, BLUE).is_empty() && !cells_of(s, RED).is_empty()
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Tab").unwrap();
    term.wait_for("flipped back to the after", STEP, |s| {
        cells_of(s, RED).is_empty() && !cells_of(s, BLUE).is_empty()
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

/// Alt+Right widens the before pane (the picture centered in it moves right),
/// Alt+0 restores the equal share.
#[test]
fn the_keys_move_the_divider_and_reset_it() {
    let repo = red_to_blue(4);
    let (mut term, _config) = launch(repo.root(), "pty-image-diff-keys", 140);
    open_the_diff(&mut term);
    let start = left_of(&wait_cells(&term, "the before", RED));
    // Alt+Right as the CSI-with-modifier sequence terminals send.
    term.send_text("\x1b[1;3C").unwrap();
    term.wait_for("the before pane wider", STEP, |s| {
        let red = cells_of(s, RED);
        !red.is_empty() && left_of(&red) > start
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("M-0").unwrap();
    term.wait_for("the equal share back", STEP, |s| {
        let red = cells_of(s, RED);
        !red.is_empty() && left_of(&red) == start
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_text("\x1b[1;3D").unwrap();
    term.wait_for("the before pane narrower", STEP, |s| {
        let red = cells_of(s, RED);
        !red.is_empty() && left_of(&red) < start
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

/// Dragging the divider with the mouse moves it; a double click resets it.
#[test]
fn dragging_the_divider_resizes_the_panes_and_a_double_click_resets() {
    let repo = red_to_blue(4);
    let (mut term, _config) = launch(repo.root(), "pty-image-diff-drag", 140);
    open_the_diff(&mut term);
    let red = wait_cells(&term, "the before", RED);
    let start = left_of(&red);
    let screen = term.screen();
    let (bc, caption_row) = screen
        .find("Before")
        .unwrap_or_else(|| panic!("no caption\n{}", term.text()));
    let (ac, _) = screen.find("After").unwrap();
    // The divider is the column of rules between the two captions.
    let row = red[0].1;
    let divider = (bc..ac)
        .find(|c| {
            screen.cell(*c, row).is_some_and(|x| x.text == "\u{2502}")
                && screen
                    .cell(*c, caption_row + 1)
                    .is_some_and(|x| x.text == "\u{2502}")
        })
        .unwrap_or_else(|| panic!("no divider between {bc} and {ac}\n{}", term.text()));
    term.drag((divider, row), (divider + 12, row)).unwrap();
    term.wait_for("the before pane dragged wider", STEP, |s| {
        let red = cells_of(s, RED);
        !red.is_empty() && left_of(&red) > start + 3
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let moved = term.screen();
    let divider = (bc..moved.cols())
        .find(|c| {
            *c > divider
                && moved.cell(*c, row).is_some_and(|x| x.text == "\u{2502}")
                && moved
                    .cell(*c, caption_row + 1)
                    .is_some_and(|x| x.text == "\u{2502}")
        })
        .unwrap_or(divider + 12);
    term.double_click(divider, row).unwrap();
    term.wait_for("the equal share back", STEP, |s| {
        let red = cells_of(s, RED);
        !red.is_empty() && left_of(&red) == start
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

/// The zoom is shared: `+` enlarges both pictures, `0` fits both again; the
/// fit follows a resize of the terminal (the pictures shrink with the panes).
#[test]
fn zoom_is_shared_and_fit_follows_a_resize() {
    let repo = red_to_blue(200);
    let (mut term, _config) = launch(repo.root(), "pty-image-diff-zoom", 140);
    open_the_diff(&mut term);
    let red = wait_cells(&term, "the before", RED);
    let blue = wait_cells(&term, "the after", BLUE);
    assert!(right_of(&red) < left_of(&blue));
    let fit = settled_counts(&term);

    term.send_keys("+").unwrap();
    term.wait_for("both enlarged", STEP, |s| {
        let (r, b) = (cells_of(s, RED), cells_of(s, BLUE));
        r.len() > fit.0 && b.len() > fit.1
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("0").unwrap();
    term.wait_for("both fitted again", STEP, |s| {
        (cells_of(s, RED).len(), cells_of(s, BLUE).len()) == fit
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert_eq!(settled_counts(&term), fit, "Fit is stable");

    // A narrower terminal: still Fit, so the pictures are refitted smaller.
    term.resize(100, 40).unwrap();
    term.wait_for("refitted to the narrower panes", STEP, |s| {
        let (r, b) = (cells_of(s, RED), cells_of(s, BLUE));
        !r.is_empty() && !b.is_empty() && r.len() < fit.0 && b.len() < fit.1
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let narrow = settled_counts(&term);
    assert!(
        narrow.0 < fit.0 && narrow.1 < fit.1,
        "{narrow:?} vs {fit:?}"
    );
    term.stop().unwrap();
}

/// A picture the commit added has no before: one pane, "New file".
#[test]
fn an_added_picture_shows_one_pane_and_no_toggle() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("note.txt", "hello\n", "a note")
        .unwrap()
        .commit_bytes("pic.png", &solid(4, 4, BLUE), "add the picture")
        .unwrap()
        .build();
    let (mut term, _config) = launch(repo.root(), "pty-image-diff-added", 140);
    open_the_diff(&mut term);
    wait_cells(&term, "the after", BLUE);
    let text = term.text();
    assert!(text.contains("New file"), "{text}");
    assert!(!text.contains("Before"), "{text}");
    assert!(!text.contains("Split  Inline"), "no layout toggle: {text}");
    term.stop().unwrap();
}

/// A working-tree edit: the unstaged diff is the index (red) against the
/// disk (blue).
#[test]
fn an_unstaged_edit_shows_the_index_against_the_working_tree() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_bytes("pic.png", &solid(4, 4, RED), "red picture")
        .unwrap()
        .write_worktree("pic.png", &solid(4, 4, BLUE))
        .unwrap()
        .build();
    let (mut term, _config) = common::launch(repo.root())
        .no_commit_column()
        .session("pty-image-diff-wip")
        .config("[ui]\ndiff_layout = \"split\"\n")
        .size(140, 40)
        .no_exit_pause()
        .ready_when("the WIP row", |s| {
            s.contains("Unstaged Files") && s.contains(LOG_HEADER)
        })
        .mouse_capture()
        .start();
    let (c, r) = term
        .wait_for_text("pic.png", STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.click(c, r).unwrap();
    let red = wait_cells(&term, "the before", RED);
    let blue = wait_cells(&term, "the after", BLUE);
    assert!(right_of(&red) < left_of(&blue), "{red:?} / {blue:?}");
    assert!(term.text().contains("working tree"), "{}", term.text());
    term.stop().unwrap();
}

/// A modified binary that is not an image keeps today's notice.
#[test]
fn a_binary_that_is_not_an_image_keeps_the_notice() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_bytes("pic.png", &[0, 1, 2, 3, 0, 255, 0, 7], "junk")
        .unwrap()
        .commit_bytes("pic.png", &[0, 9, 2, 3, 0, 255, 0, 8, 0], "other junk")
        .unwrap()
        .build();
    let (mut term, _config) = launch(repo.root(), "pty-image-diff-notimage", 140);
    open_the_diff(&mut term);
    term.wait_for("the old notice", STEP, |s| s.contains("no textual changes"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(cells_of(&term.screen(), RED).is_empty());
    term.stop().unwrap();
}

// --- "Edit in" on the Diff view's pictures ---------------------------------------

/// A fake editor script that appends its argument to a log, and the config
/// that lists it (no real editor is ever started).
fn fake_editor() -> (tempfile::TempDir, String) {
    let dir = tempfile::tempdir().unwrap();
    let script = common::fake_recorder(dir.path(), "ok", &dir.path().join("launches.log"));
    let config = format!(
        "[ui]\ndiff_layout = \"split\"\nimage_editor_default = \"Fake editor\"\n\
         [[ui.image_editors]]\nlabel = \"Fake editor\"\ncommand = \"{} --edit ${{file}}\"\n",
        common::toml_path(&script)
    );
    (dir, config)
}

fn launched(dir: &tempfile::TempDir, nth: usize) -> std::path::PathBuf {
    let log = dir.path().join("launches.log");
    let deadline = std::time::Instant::now() + STEP;
    loop {
        let text = common::read_log(&log);
        if let Some(line) = text.lines().nth(nth) {
            return std::path::PathBuf::from(
                line.strip_prefix("--edit ")
                    .unwrap_or(line)
                    .trim()
                    .trim_matches('"'),
            );
        }
        assert!(std::time::Instant::now() < deadline, "the editor never ran");
        std::thread::sleep(Duration::from_millis(50));
    }
}

/// A commit's picture diff: "Edit in" hands over a read-only copy of the side
/// shown - the after (blue) in a split, the before (red) once the inline view
/// is flipped to it.
#[test]
fn edit_in_on_a_commit_diff_hands_over_the_side_shown_as_a_copy() {
    let (dir, config) = fake_editor();
    let repo = red_to_blue(4);
    let (mut term, _config) = common::launch(repo.root())
        .no_commit_column()
        .session("pty-image-diff-edit-commit")
        .config(&config)
        .size(140, 40)
        .no_exit_pause()
        .ready_when("the commit detail of HEAD", |s| {
            s.contains("View all files") && s.contains(LOG_HEADER)
        })
        .mouse_capture()
        .start();
    open_the_diff(&mut term);
    wait_cells(&term, "the after", BLUE);
    let (c, r) = term.wait_for_text("Edit in", STEP).unwrap();
    term.click(c + 1, r).unwrap();
    let (c, r) = term
        .wait_for_text("Fake editor", STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.click(c, r).unwrap();
    let got = launched(&dir, 0);
    assert!(!got.starts_with(repo.root()), "a copy: {got:?}");
    assert_eq!(std::fs::read(&got).unwrap(), solid(4, 4, BLUE));
    assert!(std::fs::metadata(&got).unwrap().permissions().readonly());

    // Inline, flipped to the before: the keyboard twin runs the default.
    term.send_keys("m").unwrap();
    term.wait_for("inline", STEP, |s| cells_of(s, RED).is_empty())
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Tab").unwrap();
    term.wait_for("inline, flipped to the before", STEP, |s| {
        cells_of(s, BLUE).is_empty() && !cells_of(s, RED).is_empty()
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("o").unwrap();
    let got = launched(&dir, 1);
    assert_eq!(
        std::fs::read(&got).unwrap_or_else(|e| panic!("{e}: {got:?}")),
        solid(4, 4, RED)
    );
    term.stop().unwrap();
}

/// An unstaged picture diff: the after side is the working-tree file itself.
#[test]
fn edit_in_on_an_unstaged_diff_edits_the_working_tree_file() {
    let (dir, config) = fake_editor();
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_bytes("pic.png", &solid(4, 4, RED), "red picture")
        .unwrap()
        .write_worktree("pic.png", &solid(4, 4, BLUE))
        .unwrap()
        .build();
    let (mut term, _config) = common::launch(repo.root())
        .no_commit_column()
        .session("pty-image-diff-edit-wip")
        .config(&config)
        .size(140, 40)
        .no_exit_pause()
        .ready_when("the WIP row", |s| {
            s.contains("Unstaged Files") && s.contains(LOG_HEADER)
        })
        .mouse_capture()
        .start();
    let (c, r) = term.wait_for_text("pic.png", STEP).unwrap();
    term.click(c, r).unwrap();
    wait_cells(&term, "the after", BLUE);
    term.send_keys("o").unwrap();
    let got = launched(&dir, 0);
    assert_eq!(
        std::fs::canonicalize(&got).unwrap(),
        std::fs::canonicalize(repo.root().join("pic.png")).unwrap()
    );
    term.stop().unwrap();
}

/// The after side of an unstaged diff is read from the disk; when the file
/// changes under the view (another program, "Edit in"), the picture follows
/// without reopening the diff.
#[test]
fn an_external_edit_refreshes_the_working_tree_picture() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_bytes("pic.png", &solid(4, 4, RED), "red picture")
        .unwrap()
        .write_worktree("pic.png", &solid(4, 4, BLUE))
        .unwrap()
        .build();
    let (mut term, _config) = common::launch(repo.root())
        .no_commit_column()
        .session("pty-image-diff-refresh")
        .config("[ui]\ndiff_layout = \"split\"\n")
        .size(140, 40)
        .no_exit_pause()
        .ready_when("the WIP row", |s| {
            s.contains("Unstaged Files") && s.contains(LOG_HEADER)
        })
        .mouse_capture()
        .start();
    let (c, r) = term
        .wait_for_text("pic.png", STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.click(c, r).unwrap();
    wait_cells(&term, "the before", RED);
    wait_cells(&term, "the after", BLUE);

    // Another program saves a green picture (a different size, so the size
    // differs even on a filesystem with coarse timestamps).
    let green = (0, 255, 0);
    std::fs::write(repo.root().join("pic.png"), solid(6, 6, green)).unwrap();
    term.wait_for("the after pane turns green", STEP, |s| {
        !cells_of(s, green).is_empty() && cells_of(s, BLUE).is_empty()
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(
        !cells_of(&term.screen(), RED).is_empty(),
        "the before stays"
    );
    term.stop().unwrap();
}

/// A new picture nobody has staged yet (untracked) is previewed when selected
/// in the WIP: one pane, "New file", read from the disk.
#[test]
fn an_untracked_picture_is_previewed_as_a_new_file() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("note.txt", "hello\n", "a note")
        .unwrap()
        .write_worktree("fresh.png", &solid(6, 6, BLUE))
        .unwrap()
        .build();
    let (mut term, _config) = common::launch(repo.root())
        .no_commit_column()
        .session("pty-image-diff-untracked")
        .config("[ui]\ndiff_layout = \"split\"\n")
        .size(140, 40)
        .no_exit_pause()
        .ready_when("the WIP row", |s| {
            s.contains("Unstaged Files") && s.contains(LOG_HEADER)
        })
        .mouse_capture()
        .start();
    let (c, r) = term
        .wait_for_text("fresh.png", STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.click(c, r).unwrap();
    wait_cells(&term, "the new picture", BLUE);
    let text = term.text();
    assert!(text.contains("New file"), "{text}");
    assert!(!text.contains("Before"), "{text}");
    assert!(!text.contains("no textual changes"), "{text}");
    term.stop().unwrap();
}
