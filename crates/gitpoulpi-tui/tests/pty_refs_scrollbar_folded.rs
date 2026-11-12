//! A Refs section's scrollbar measures the rows the list really has after its
//! folders are folded, not the number of refs: with the folders collapsed and
//! the list scrolled to its last row the thumb touches the bottom arrow, and
//! it shrinks again when the folders are expanded. Whole-state waits.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::{Color, Screen, Terminal};

const STEP: Duration = Duration::from_secs(30);
const FOLDERS: usize = 50;

/// 50 folders of two branches each: 150 rows expanded, 51 collapsed.
fn fixture() -> FixtureRepo {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "scrollbar probe commit")
        .unwrap()
        .build();
    for f in 0..FOLDERS {
        for l in 0..2 {
            repo.git(&["branch", &format!("fold{f:02}/leaf{l}")]);
        }
    }
    repo
}

/// The sidebar's first scrollbar: its column and the rows of its `↑` and `↓`
/// end arrows.
fn bar(s: &Screen) -> Option<(u16, u16, u16)> {
    for row in 0..s.rows() {
        for (col, _) in s.find_all("\u{2191}") {
            if col > 40 || col == 0 {
                continue;
            }
            if let Some((_, down)) = s
                .find_all("\u{2193}")
                .into_iter()
                .filter(|(c, r)| *c == col && *r > row)
                .min_by_key(|(_, r)| *r)
            {
                let (_, up) = (col, row);
                // The `↑` found on this row of this column.
                if s.find_all("\u{2191}").contains(&(col, up)) {
                    return Some((col, up, down));
                }
            }
        }
    }
    None
}

fn bg(s: &Screen, col: u16, row: u16) -> Option<Color> {
    s.cell(col, row).map(|c| c.bg)
}

/// The cells of the bar between the arrows that carry the thumb's color.
fn thumb_cells(s: &Screen, col: u16, up: u16, down: u16, thumb: Color) -> Vec<u16> {
    (up + 1..down)
        .filter(|r| bg(s, col, *r) == Some(thumb))
        .collect()
}

/// Falsified: with the section's scrollbar measuring `total` (the ref count)
/// instead of the rows after folding, the thumb stops short of the bottom
/// arrow (red at "the thumb touches the bottom arrow").
#[test]
fn a_folded_refs_section_has_its_thumb_at_the_bottom_when_scrolled_to_the_end() {
    let repo = fixture();
    let (mut term, _config) = common::launch(repo.root())
        .session("pty-refs-scroll-folded")
        .size(200, 40)
        .ready("scrollbar probe commit")
        .mouse_capture()
        .start();
    term.wait_for("the LOCAL section with a bar", STEP, |s| {
        s.contains("leaf0") && bar(s).is_some()
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // Focus the LOCAL list (a click on a row), then fold every folder.
    let (c, r) = term.screen().find("leaf0").unwrap();
    term.click(c + 1, r).unwrap();
    term.send_text("-").unwrap();
    term.wait_for("the folders folded", STEP, |s| {
        !s.contains("leaf0") && s.contains("fold00/") && bar(s).is_some()
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // At the top the thumb sits right under the up arrow: that is its color.
    let s = term
        .wait_for_stable(Duration::from_millis(250), STEP)
        .unwrap();
    let (col, up, down) = bar(&s).unwrap();
    let thumb = bg(&s, col, up + 1).unwrap();
    let thumb_len_folded = thumb_cells(&s, col, up, down, thumb).len();
    assert!(thumb_len_folded >= 1, "{}", s.text());

    // To the last row: the thumb touches the bottom arrow.
    let (wc, wr) = s.find("fold05/").unwrap();
    for _ in 0..70 {
        term.scroll(true, wc, wr).unwrap();
    }
    term.wait_for("the thumb touches the bottom arrow", STEP, |s| {
        bar(s).is_some_and(|(col, up, down)| {
            let t = bg(s, col, up + 1);
            // The thumb left the top and sits on the last cell of the track.
            bg(s, col, down - 1) == Some(thumb) && t != Some(thumb) && s.contains("fold49/")
        })
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // Expanded again: three times the rows, so a shorter thumb.
    term.send_text("=").unwrap();
    term.wait_for("the thumb shrank", STEP, |s| {
        bar(s).is_some_and(|(col, up, down)| {
            let len = thumb_cells(s, col, up, down, thumb).len();
            len >= 1 && len < thumb_len_folded
        })
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let _ = term.stop();
}
