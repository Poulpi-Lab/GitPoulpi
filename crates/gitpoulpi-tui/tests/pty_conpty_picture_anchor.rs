//! Where the Log's pictures are anchored when the terminal sits behind
//! Windows' ConPTY (Rio, mintty, WezTerm 20240203).
//!
//! An `OSC 1337` crosses ConPTY at once, ahead of the text it has not painted
//! yet, and lands at the cursor ConPTY last painted: a hidden cursor is never
//! painted, so the Graph column's picture used to land on a stale row
//! (Rio 0.5.28: the graph on the wrong rows, the screen scrolled under it).
//! What the terminal receives is read back from the pty: the cursor move that
//! precedes the Graph picture must be the one to the column's first row.
//! On a Unix pty the bytes are the program's own and the assertion holds
//! trivially; behind ConPTY it is the regression test.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;

const STEP: Duration = Duration::from_secs(60);
const MESSAGE: &str = "conpty anchor probe commit";

/// `(row, col)` of the last `CUP`/`HVP` (one-based) before each `OSC 1337`
/// picture, with the picture's `height=` in cells.
fn pictures(raw: &[u8]) -> Vec<((u32, u32), u32)> {
    let marker = b"\x1b]1337;File=";
    let mut found = Vec::new();
    let mut cursor = (0, 0);
    let mut i = 0;
    while i < raw.len() {
        if raw[i..].starts_with(marker) {
            let end = raw[i..]
                .iter()
                .position(|b| *b == b':')
                .map_or(raw.len(), |p| i + p);
            let params = String::from_utf8_lossy(&raw[i..end]).into_owned();
            let height = params
                .split(';')
                .find_map(|p| p.strip_prefix("height="))
                .and_then(|h| h.parse().ok())
                .unwrap_or(0);
            found.push((cursor, height));
            i = end;
        } else if raw[i] == 0x1b && raw.get(i + 1) == Some(&b'[') {
            let body_start = i + 2;
            let mut j = body_start;
            while j < raw.len() && !(0x40..=0x7e).contains(&raw[j]) {
                j += 1;
            }
            if matches!(raw.get(j), Some(b'H') | Some(b'f')) {
                let nums: Vec<u32> = String::from_utf8_lossy(&raw[body_start..j])
                    .split(';')
                    .map(|n| n.parse().unwrap_or(1))
                    .collect();
                cursor = (
                    nums.first().copied().unwrap_or(1),
                    nums.get(1).copied().unwrap_or(1),
                );
            }
            i = j + 1;
        } else {
            i += 1;
        }
    }
    found
}

/// Falsified: with `conpty_anchors_pictures()` returning `false` the picture
/// lands behind a stale cursor on Windows and this goes red; restored. (On a
/// Unix pty it cannot fail: nothing sits between the program and the test.)
#[test]
fn the_graph_picture_is_placed_by_a_cursor_move_to_the_columns_first_row() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("an older commit under the probe")
        .unwrap()
        .commit("a newer commit under the probe")
        .unwrap()
        .commit(MESSAGE)
        .unwrap()
        .build();
    let (term, _config) = common::launch(repo.root())
        .session("pty-conpty-anchor")
        .config("[ui]\ngraph_renderer = \"image\"\nimage_protocol = \"iterm2\"\n")
        .terminal_image()
        .env("GITPOULPI_TERMINAL_CELL_PX", "10x20")
        .env("TERM_PROGRAM", "rio")
        .env("TERM", "xterm-256color")
        .env("WEZTERM_EXECUTABLE", "")
        .env("TMUX", "")
        .env("ZELLIJ", "")
        .env("STY", "")
        .env("HERDR_ENV", "")
        .env("HERDR_PANE_ID", "")
        .env("HERDR_SOCKET_PATH", "")
        .size(200, 50)
        .start();
    term.wait_for("the fixture loaded", STEP, |s| s.contains(MESSAGE))
        .unwrap_or_else(|e| panic!("{e}"));
    term.wait_for_stable(Duration::from_millis(1500), STEP)
        .unwrap();
    let (_, first_row) = term
        .screen()
        .find("conpty anchor prob")
        .expect("the newest commit");
    let raw = term.raw_output();
    let _ = term.stop();
    // The Graph column's picture is the tall one (the title logo is one row).
    let tall: Vec<_> = pictures(&raw)
        .into_iter()
        .filter(|(_, height)| *height >= 3)
        .collect();
    assert!(!tall.is_empty(), "no Graph picture was sent");
    if let Some(((row, _col), height)) = tall.first() {
        assert_eq!(
            *row,
            u32::from(first_row) + 1,
            "a {height}-row picture was placed from a cursor at row {row}, not at the Log's first row {} (one-based)",
            u32::from(first_row) + 1
        );
    }
}
