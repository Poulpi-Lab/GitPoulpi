//! What the Graph column's picture really contains, read back from the
//! escape sequence the binary writes to its pty (WezTerm-style iTerm2
//! inline PNG): the `row_tint` right-edge marker, and lanes that stay thin
//! in pixels when the terminal's cells are bigger (a zoomed terminal).
//!
//! The cell size is pinned with `GITPOULPI_TERMINAL_CELL_PX`, the only way
//! a pty can stand in for a zoom.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(60);
const MESSAGE: &str = "graph picture probe commit";

fn fixture() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit("first commit of the probe")
        .unwrap()
        .commit("second commit of the probe")
        .unwrap()
        .commit(MESSAGE)
        .unwrap()
        .build()
}

/// A decoded RGBA picture.
struct Picture {
    width: usize,
    height: usize,
    rgba: Vec<u8>,
}

impl Picture {
    fn alpha(&self, x: usize, y: usize) -> u8 {
        self.rgba[(y * self.width + x) * 4 + 3]
    }
}

fn base64_decode(text: &[u8]) -> Vec<u8> {
    let value = |c: u8| match c {
        b'A'..=b'Z' => Some(c - b'A'),
        b'a'..=b'z' => Some(c - b'a' + 26),
        b'0'..=b'9' => Some(c - b'0' + 52),
        b'+' => Some(62),
        b'/' => Some(63),
        _ => None,
    };
    let mut out = Vec::new();
    let (mut acc, mut bits) = (0u32, 0);
    for v in text.iter().filter_map(|c| value(*c)) {
        acc = (acc << 6) | u32::from(v);
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((acc >> bits) as u8);
            acc &= (1 << bits) - 1;
        }
    }
    out
}

/// The last picture the binary sent.
fn last_picture(term: &Terminal) -> Picture {
    let raw = term.raw_output();
    let marker = b"]1337;File=";
    let at = raw
        .windows(marker.len())
        .rposition(|w| w == marker)
        .expect("the binary never sent a picture");
    let body = &raw[at..];
    let colon = body.iter().position(|b| *b == b':').expect("payload");
    let payload: Vec<u8> = body[colon + 1..]
        .iter()
        .copied()
        .take_while(|b| b.is_ascii_alphanumeric() || *b == b'+' || *b == b'/' || *b == b'=')
        .collect();
    let png_bytes = base64_decode(&payload);
    let mut reader = png::Decoder::new(std::io::Cursor::new(png_bytes))
        .read_info()
        .expect("a PNG");
    let mut buf = vec![0; reader.output_buffer_size().expect("size")];
    let info = reader.next_frame(&mut buf).expect("a frame");
    assert_eq!(info.color_type, png::ColorType::Rgba, "RGBA expected");
    buf.truncate(info.buffer_size());
    Picture {
        width: info.width as usize,
        height: info.height as usize,
        rgba: buf,
    }
}

fn picture_with(setting: &str, cell: &str, session: &str) -> Picture {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    std::fs::write(
        config.path().join("config.toml"),
        format!(
            "[ui]\ngraph_renderer = \"image\"\nimage_protocol = \"iterm2\"\ngraph_lane_background = \"{setting}\"\n"
        ),
    )
    .unwrap();
    let term = common::launch(repo.root())
        .session(session)
        .config_dir(config.path())
        .size(200, 60)
        .terminal_image()
        .env("GITPOULPI_TERMINAL_CELL_PX", cell)
        .env("TERM_PROGRAM", "WezTerm")
        .env("TERM", "xterm-256color")
        .env("HERDR_ENV", "")
        .env("HERDR_PANE_ID", "")
        .env("HERDR_SOCKET_PATH", "")
        .spawn();
    term.wait_for("the fixture loaded", STEP, |s| s.contains(MESSAGE))
        .unwrap_or_else(|e| panic!("{e}"));
    term.wait_for_stable(Duration::from_millis(800), STEP)
        .unwrap();
    let picture = last_picture(&term);
    let _ = term.stop();
    picture
}

/// The lane's thickness in picture pixels: the most common length of the
/// single opaque run in the rows crossed by nothing but a vertical lane.
fn lane_thickness(p: &Picture) -> usize {
    let mut lengths = std::collections::HashMap::new();
    for y in 0..p.height {
        let mut runs = Vec::new();
        let mut start = None;
        for x in 0..=p.width {
            let on = x < p.width && p.alpha(x, y) > 128;
            match (on, start) {
                (true, None) => start = Some(x),
                (false, Some(s)) => {
                    runs.push(x - s);
                    start = None;
                }
                _ => {}
            }
        }
        if let [only] = runs[..] {
            *lengths.entry(only).or_insert(0usize) += 1;
        }
    }
    lengths
        .into_iter()
        .max_by_key(|(len, rows)| (*rows, *len))
        .map(|(len, _)| len)
        .expect("a lane")
}

#[test]
fn row_tint_ends_every_commit_row_with_a_bar_at_the_columns_right_edge() {
    let tint = picture_with("row_tint", "9x21", "pic-tint");
    let off = picture_with("off", "9x21", "pic-off");
    let edge = tint.width - 1;
    let barred = (0..tint.height)
        .filter(|y| tint.alpha(edge, *y) > 200)
        .count();
    assert!(
        barred > tint.height / 4,
        "row_tint: the right-edge bar covers {barred} of {} rows",
        tint.height
    );
    assert!(
        (0..off.height).all(|y| off.alpha(off.width - 1, y) == 0),
        "off: nothing at the right edge"
    );
    // Thin: a few pixels wide, not a cell.
    let width = (0..tint.width)
        .filter(|x| tint.alpha(*x, tint.height / 2) > 200)
        .count();
    assert!(width <= 12, "the bar is {width} px wide (scaled)");
}

#[test]
fn lanes_stay_thin_in_pixels_when_the_cells_are_bigger() {
    let normal = lane_thickness(&picture_with("off", "9x21", "pic-zoom1"));
    let zoomed = lane_thickness(&picture_with("off", "27x63", "pic-zoom3"));
    assert!(
        zoomed * 4 <= normal * 5,
        "a lane is {normal}px at 9x21 and {zoomed}px at 27x63: it grew with the cell"
    );
}
