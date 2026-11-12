//! The commit's changed-files list in Tree mode measures its scrollbar on the
//! rows it shows after the folders are folded: collapsed, scrolled to the end,
//! the thumb touches the bottom arrow (confirmed fine: the list's length is
//! the folded `build_tree_rows` result). Whole-state waits.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::{Color, Screen};

const STEP: Duration = Duration::from_secs(30);

/// The right panel's scrollbar: its column (right of the Log) and the rows of
/// its `↑` and `↓` arrows, the bottom-most bar in the panel.
fn bar(s: &Screen) -> Option<(u16, u16, u16)> {
    let downs: Vec<(u16, u16)> = s
        .find_all("\u{2193}")
        .into_iter()
        .filter(|(c, _)| *c > 120)
        .collect();
    let (col, down) = downs.into_iter().max_by_key(|(_, r)| *r)?;
    let up = s
        .find_all("\u{2191}")
        .into_iter()
        .filter(|(c, r)| *c == col && *r < down)
        .map(|(_, r)| r)
        .max()?;
    Some((col, up, down))
}

fn bg(s: &Screen, col: u16, row: u16) -> Option<Color> {
    s.cell(col, row).map(|c| c.bg)
}

#[test]
fn the_tree_mode_files_list_has_its_thumb_at_the_bottom_when_folded_and_scrolled() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("seed.txt", "x\n", "files tree seed")
        .unwrap()
        .build();
    for d in 0..40 {
        std::fs::create_dir_all(repo.root().join(format!("dir{d:02}"))).unwrap();
        for f in 0..2 {
            std::fs::write(repo.root().join(format!("dir{d:02}/f{f}.txt")), "x\n").unwrap();
        }
    }
    repo.git(&["add", "-A"]);
    repo.git(&["commit", "-m", "files tree many"]);
    let (mut term, _config) = common::launch(repo.root())
        .session("pty-files-tree-bar")
        .size(200, 40)
        .ready("files tree many")
        .mouse_capture()
        .start();
    // Tree mode (the header's `Tree` label).
    let (c, r) = term
        .wait_for("the files header", STEP, |s| s.contains("Tree [m]"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()))
        .find("Tree [m]")
        .unwrap();
    term.click(c + 1, r).unwrap();
    term.wait_for("the tree with its folders", STEP, |s| {
        s.contains("dir00/") && bar(s).is_some()
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // Fold every folder: 40 rows left.
    let (fc, fr) = term.screen().find("dir00/").unwrap();
    term.click(fc + 1, fr).unwrap();
    term.send_text("-").unwrap();
    term.wait_for("the folders folded", STEP, |s| {
        s.contains("dir00/") && !s.contains("f0.txt") && bar(s).is_some()
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let s = term
        .wait_for_stable(Duration::from_millis(250), STEP)
        .unwrap();
    let (col, up, _down) = bar(&s).unwrap();
    let thumb = bg(&s, col, up + 1).unwrap();
    let (wc, wr) = s.find("dir05/").unwrap();
    for _ in 0..80 {
        term.scroll(true, wc, wr).unwrap();
    }
    term.wait_for("the thumb touches the bottom arrow", STEP, |s| {
        s.contains("dir39/")
            && bar(s).is_some_and(|(col, up, down)| {
                bg(s, col, down - 1) == Some(thumb) && bg(s, col, up + 1) != Some(thumb)
            })
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let _ = term.stop();
}
