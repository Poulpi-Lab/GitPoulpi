//! The File View on a picture, end to end through a pty (image view slice S4).
//!
//! `GITPOULPI_TERMINAL_IMAGE=off` is what `common::launch` sets, so the
//! terminal has no image protocol and the picture is drawn as half-block
//! cells (`\u{2580}`: foreground = the top pixel, background = the bottom
//! one). vt100 keeps those colors, which is what these tests read. Pixel
//! protocols are unit-tested with a forced capability (`image_view::pane`)
//! and checked by hand.

mod common;

use std::path::Path;
use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::{Color, Screen, Terminal};

const STEP: Duration = Duration::from_secs(30);
const HALF: &str = "\u{2580}";

const GIF_2X2: [u8; 50] = [
    0x47, 0x49, 0x46, 0x38, 0x39, 0x61, 0x02, 0x00, 0x02, 0x00, 0xf1, 0x00, 0x00, 0x00, 0x00, 0x00,
    0xff, 0x00, 0x00, 0x00, 0xff, 0x00, 0x00, 0x00, 0xff, 0x21, 0xf9, 0x04, 0x01, 0x00, 0x00, 0x00,
    0x00, 0x2c, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x02, 0x00, 0x00, 0x02, 0x03, 0x8c, 0x06, 0x05,
    0x00, 0x3b,
];

const JPEG_FLAT_8X8: [u8; 285] = [
    0xff, 0xd8, 0xff, 0xe0, 0x00, 0x10, 0x4a, 0x46, 0x49, 0x46, 0x00, 0x01, 0x01, 0x00, 0x00, 0x01,
    0x00, 0x01, 0x00, 0x00, 0xff, 0xdb, 0x00, 0x43, 0x00, 0x02, 0x01, 0x01, 0x01, 0x01, 0x01, 0x02,
    0x01, 0x01, 0x01, 0x02, 0x02, 0x02, 0x02, 0x02, 0x04, 0x03, 0x02, 0x02, 0x02, 0x02, 0x05, 0x04,
    0x04, 0x03, 0x04, 0x06, 0x05, 0x06, 0x06, 0x06, 0x05, 0x06, 0x06, 0x06, 0x07, 0x09, 0x08, 0x06,
    0x07, 0x09, 0x07, 0x06, 0x06, 0x08, 0x0b, 0x08, 0x09, 0x0a, 0x0a, 0x0a, 0x0a, 0x0a, 0x06, 0x08,
    0x0b, 0x0c, 0x0b, 0x0a, 0x0c, 0x09, 0x0a, 0x0a, 0x0a, 0xff, 0xdb, 0x00, 0x43, 0x01, 0x02, 0x02,
    0x02, 0x02, 0x02, 0x02, 0x05, 0x03, 0x03, 0x05, 0x0a, 0x07, 0x06, 0x07, 0x0a, 0x0a, 0x0a, 0x0a,
    0x0a, 0x0a, 0x0a, 0x0a, 0x0a, 0x0a, 0x0a, 0x0a, 0x0a, 0x0a, 0x0a, 0x0a, 0x0a, 0x0a, 0x0a, 0x0a,
    0x0a, 0x0a, 0x0a, 0x0a, 0x0a, 0x0a, 0x0a, 0x0a, 0x0a, 0x0a, 0x0a, 0x0a, 0x0a, 0x0a, 0x0a, 0x0a,
    0x0a, 0x0a, 0x0a, 0x0a, 0x0a, 0x0a, 0x0a, 0x0a, 0x0a, 0x0a, 0x0a, 0x0a, 0x0a, 0x0a, 0xff, 0xc0,
    0x00, 0x11, 0x08, 0x00, 0x08, 0x00, 0x08, 0x03, 0x01, 0x11, 0x00, 0x02, 0x11, 0x01, 0x03, 0x11,
    0x01, 0xff, 0xc4, 0x00, 0x14, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x05, 0xff, 0xc4, 0x00, 0x14, 0x10, 0x01, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xff, 0xc4, 0x00,
    0x14, 0x01, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x08, 0xff, 0xc4, 0x00, 0x14, 0x11, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xff, 0xda, 0x00, 0x0c, 0x03, 0x01, 0x00,
    0x02, 0x11, 0x03, 0x11, 0x00, 0x3f, 0x00, 0x3c, 0x57, 0x36, 0x1f, 0xff, 0xd9,
];

fn launch(repo: &Path, session: &str) -> (Terminal, tempfile::TempDir) {
    common::launch(repo)
        .no_commit_column()
        .session(session)
        .size(140, 40)
        .no_exit_pause()
        .ready_when("the commit detail of HEAD", |s| {
            s.contains("View all files") && s.contains("Message")
        })
        .mouse_capture()
        .start()
}

/// A repository whose HEAD changed only `note.txt`, so every other file is
/// "unchanged" and View all files opens it in the File View.
fn repo_with(files: &[(&str, Vec<u8>)]) -> gitpoulpi_testkit::FixtureRepo {
    let mut b = FixtureRepoBuilder::new().unwrap();
    for (name, bytes) in files {
        b = b.write_worktree(name, bytes).unwrap();
    }
    b.write_worktree("note.txt", b"first\n")
        .unwrap()
        .stage(".")
        .unwrap()
        .commit("add files")
        .unwrap()
        .commit_file("note.txt", "second\n", "touch the note")
        .unwrap()
        .build()
}

/// Opens `name` (an unchanged file) in the central File View.
fn open_file_view(term: &mut Terminal, name: &str) {
    // View all files stays on after the first time (a second open in one
    // session finds the list already there).
    if snap(term).find(name).is_none() {
        let (c, r) = term.wait_for_text("View all files", STEP).unwrap();
        term.click(c + 6, r).unwrap();
    }
    let (fc, fr) = term
        .wait_for_text(name, STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.click(fc, fr).unwrap();
    term.wait_for("the File View", STEP, |s| {
        s.contains(&format!("File View - {name} @"))
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

/// Every half-block cell on screen as `(col, row, fg, bg)`.
fn half_blocks(s: &Screen) -> Vec<(u16, u16, Color, Color)> {
    let mut out = Vec::new();
    for row in 0..s.rows() {
        for col in 0..s.cols() {
            if let Some(c) = s.cell(col, row) {
                if c.text == HALF {
                    out.push((col, row, c.fg, c.bg));
                }
            }
        }
    }
    out
}

fn near(c: Color, rgb: (u8, u8, u8)) -> bool {
    match c {
        Color::Rgb(r, g, b) => {
            r.abs_diff(rgb.0) < 8 && g.abs_diff(rgb.1) < 8 && b.abs_diff(rgb.2) < 8
        }
        _ => false,
    }
}

const RED: (u8, u8, u8) = (255, 0, 0);
const GREEN: (u8, u8, u8) = (0, 255, 0);
const BLUE: (u8, u8, u8) = (0, 0, 255);
const YELLOW: (u8, u8, u8) = (255, 255, 0);
const MAGENTA: (u8, u8, u8) = (255, 0, 255);

/// 4x4: pixel rows red / green / blue / yellow, with the top-left pixel
/// magenta. As cells: row 0 is fg red-ish over bg green, row 1 fg blue over
/// bg yellow.
fn four_by_four() -> Vec<u8> {
    let rows = [RED, GREEN, BLUE, YELLOW];
    let mut rgba = Vec::new();
    for (y, c) in rows.iter().enumerate() {
        for x in 0..4 {
            let c = if (x, y) == (0, 0) { MAGENTA } else { *c };
            rgba.extend_from_slice(&[c.0, c.1, c.2, 255]);
        }
    }
    gitpoulpi_testkit::png_bytes(4, 4, &rgba)
}

/// A PNG in the File View is drawn as half-block cells carrying its colors,
/// with the format and size in the header; `+` enlarges the picture, `0`
/// restores the fit. Red without the image pane: the old line "binary file
/// (N bytes) - nothing to show".
#[test]
fn a_png_is_drawn_as_half_block_cells_and_zoom_changes_then_restores_them() {
    let repo = repo_with(&[("pic.png", four_by_four())]);
    let (mut term, _config) = launch(repo.root(), "pty-image-png");
    open_file_view(&mut term, "pic.png");
    term.wait_for("the picture", STEP, |s| !half_blocks(s).is_empty())
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let text = term.text();
    assert!(text.contains("PNG 4\u{d7}4"), "the info line: {text}");
    assert!(text.contains("100%"), "the zoom label: {text}");
    assert!(!text.contains("nothing to show"), "{text}");

    let cells = half_blocks(&snap(&term));
    assert_eq!(cells.len(), 8, "4 columns x 2 cell rows: {cells:?}");
    let top = cells.iter().map(|c| c.1).min().unwrap();
    let left = cells.iter().map(|c| c.0).min().unwrap();
    let at = |dx: u16, dy: u16| {
        cells
            .iter()
            .find(|c| c.0 == left + dx && c.1 == top + dy)
            .unwrap_or_else(|| panic!("no cell at +{dx},+{dy}: {cells:?}"))
    };
    assert!(near(at(0, 0).2, MAGENTA), "the odd pixel: {:?}", at(0, 0));
    assert!(near(at(1, 0).2, RED) && near(at(1, 0).3, GREEN));
    assert!(near(at(3, 0).2, RED) && near(at(3, 0).3, GREEN));
    assert!(near(at(2, 1).2, BLUE) && near(at(2, 1).3, YELLOW));

    // + enlarges it (a different cell pattern), 0 restores the fit.
    term.send_keys("+").unwrap();
    term.wait_for("the picture grown", STEP, |s| half_blocks(s).len() > 8)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(term.text().contains("141%"), "{}", term.text());
    term.send_keys("0").unwrap();
    term.wait_for("the fit back", STEP, |s| half_blocks(s).len() == 8)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(term.text().contains("100%"));
    assert_eq!(half_blocks(&snap(&term)), cells, "the same cells again");
    term.stop().unwrap();
}

/// A header button works like its key: clicking `+` zooms in, `Fit` restores.
#[test]
fn the_header_buttons_zoom_and_fit() {
    let repo = repo_with(&[("pic.png", four_by_four())]);
    let (mut term, _config) = launch(repo.root(), "pty-image-buttons");
    open_file_view(&mut term, "pic.png");
    term.wait_for("the picture", STEP, |s| half_blocks(s).len() == 8)
        .unwrap_or_else(|e| panic!("{e}"));
    term.wait_for_text(" Fit ", STEP).unwrap();
    // The tab bar's own `+` (row 1) is not the header's.
    let (c, r) = snap(&term)
        .find_all(" + ")
        .into_iter()
        .find(|(_, row)| *row > 1)
        .expect("the header's + button");
    term.click(c + 1, r).unwrap();
    term.wait_for("zoomed in by the button", STEP, |s| {
        half_blocks(s).len() > 8
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (c, r) = term.wait_for_text(" Fit ", STEP).unwrap();
    term.click(c + 1, r).unwrap();
    term.wait_for("fit by the button", STEP, |s| half_blocks(s).len() == 8)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

/// A binary that is not an image keeps today's one-line notice.
#[test]
fn a_binary_that_is_not_an_image_keeps_the_old_notice() {
    let repo = repo_with(&[("data.bin", vec![0, 1, 2, 3, 0, 255, 0, 7, 0, 0])]);
    let (mut term, _config) = launch(repo.root(), "pty-image-not");
    open_file_view(&mut term, "data.bin");
    term.wait_for("the old notice", STEP, |s| {
        s.contains("binary file (10 bytes)")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(half_blocks(&snap(&term)).is_empty());
    term.stop().unwrap();
}

/// JPEG and GIF render too (a flat orange 8x8 JPEG; a 2x2 GIF whose last
/// pixel is transparent).
#[test]
fn jpeg_and_gif_render() {
    let repo = repo_with(&[
        ("flat.jpg", JPEG_FLAT_8X8.to_vec()),
        ("tiny.gif", GIF_2X2.to_vec()),
    ]);
    let (mut term, _config) = launch(repo.root(), "pty-image-jpeg-gif");
    open_file_view(&mut term, "flat.jpg");
    term.wait_for("the jpeg", STEP, |s| half_blocks(s).len() == 8 * 4)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(term.text().contains("JPEG 8\u{d7}8"));
    // The flat color (200, 100, 50) within the JPEG's rounding.
    assert!(
        half_blocks(&snap(&term))
            .iter()
            .all(|c| near(c.2, (200, 100, 50)) && near(c.3, (200, 100, 50))),
        "{:?}",
        half_blocks(&snap(&term))
    );
    term.send_keys("Escape").unwrap();
    term.wait_for("back to the graph", STEP, |s| s.contains("Message"))
        .unwrap_or_else(|e| panic!("{e}"));
    open_file_view(&mut term, "tiny.gif");
    term.wait_for("the gif", STEP, |s| !half_blocks(s).is_empty())
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(term.text().contains("GIF 2\u{d7}2"));
    let cells = half_blocks(&snap(&term));
    assert!(
        cells.iter().any(|c| near(c.2, RED)),
        "the red pixel: {cells:?}"
    );
    term.stop().unwrap();
}

/// A PNG whose data is damaged shows a card, not a crash.
#[test]
fn a_corrupt_png_shows_a_card() {
    let mut bytes = four_by_four();
    bytes.truncate(40);
    let repo = repo_with(&[("bad.png", bytes)]);
    let (mut term, _config) = launch(repo.root(), "pty-image-corrupt");
    open_file_view(&mut term, "bad.png");
    term.wait_for("the card", STEP, |s| s.contains("data is damaged"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(half_blocks(&snap(&term)).is_empty());
    assert!(!term.exited());
    term.stop().unwrap();
}

/// The screen once it has settled: a torn frame is not a place to locate a
/// cell or compare colors in.
fn snap(term: &xtask::pty::Terminal) -> xtask::pty::Screen {
    term.wait_for_stable(
        std::time::Duration::from_millis(300),
        std::time::Duration::from_secs(30),
    )
    .unwrap_or_else(|e| panic!("{e}"))
}
