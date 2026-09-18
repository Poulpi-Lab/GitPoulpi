//! The Graph column as a sixel picture with the authors' faces in the
//! commit circles (Windows Terminal, foot, Konsole), read back from the
//! bytes the binary writes to its pty: one tall picture whose size is a
//! whole number of cells (the cell size is pinned with
//! `GITPOULPI_TERMINAL_CELL_PX`), lanes drawn, the avatar's many colors
//! inside it, and a text graph when pictures are off. Same rules as
//! `pty_ui.rs`.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;
use sha2::{Digest, Sha256};

const STEP: Duration = Duration::from_secs(60);
const MESSAGE: &str = "sixel graph probe";

fn hash(email: &str) -> String {
    Sha256::digest(email.trim().to_lowercase().as_bytes())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// A face with many distinct colors, so a node holding it is told from a
/// lane-only picture by its palette alone.
fn gradient_png() -> Vec<u8> {
    let (w, h) = (48u32, 48u32);
    let mut out = Vec::new();
    {
        let mut enc = png::Encoder::new(&mut out, w, h);
        enc.set_color(png::ColorType::Rgba);
        enc.set_depth(png::BitDepth::Eight);
        let mut writer = enc.write_header().unwrap();
        let mut data = Vec::new();
        for y in 0..h {
            for x in 0..w {
                data.extend_from_slice(&[(x * 5) as u8, (y * 5) as u8, 128, 255]);
            }
        }
        writer.write_image_data(&data).unwrap();
    }
    out
}

/// A decoded sixel: `pixels[y * width + x]` is `None` where undrawn.
struct Sixel {
    width: usize,
    height: usize,
    pixels: Vec<Option<[u8; 3]>>,
}

impl Sixel {
    fn drawn(&self) -> usize {
        self.pixels.iter().flatten().count()
    }

    fn colors(&self) -> usize {
        let mut seen = std::collections::HashSet::new();
        self.pixels.iter().flatten().for_each(|p| {
            seen.insert(*p);
        });
        seen.len()
    }
}

fn number(s: &[u8]) -> (usize, usize) {
    let n = s.iter().take_while(|b| b.is_ascii_digit()).count();
    (
        std::str::from_utf8(&s[..n]).unwrap().parse().unwrap_or(0),
        n,
    )
}

/// Every sixel in `raw`, decoded (registers, run-length, `$`, `-`).
fn sixels(raw: &[u8]) -> Vec<Sixel> {
    let start = b"\x1bP0;1;0q";
    let mut found = Vec::new();
    let mut at = 0;
    while let Some(p) = raw[at..].windows(start.len()).position(|w| w == start) {
        let body_start = at + p + start.len();
        let end = raw[body_start..]
            .windows(2)
            .position(|w| w == b"\x1b\\")
            .map_or(raw.len(), |e| body_start + e);
        at = end.max(body_start);
        let body = &raw[body_start..end];
        let Some(rest) = body.strip_prefix(b"\"1;1;") else {
            continue;
        };
        let (width, n) = number(rest);
        let (height, m) = number(&rest[n + 1..]);
        let mut data = &rest[n + 1 + m..];
        let mut palette = std::collections::HashMap::new();
        let mut pixels = vec![None; width * height];
        let (mut x, mut band, mut color) = (0usize, 0usize, [0u8; 3]);
        while let Some((&c, tail)) = data.split_first() {
            data = tail;
            match c {
                b'#' => {
                    let (index, n) = number(data);
                    data = &data[n..];
                    if data.first() == Some(&b';') {
                        let mut rgb = [0u8; 3];
                        data = &data[3..]; // ";2;"
                        for (i, slot) in rgb.iter_mut().enumerate() {
                            let (v, n) = number(data);
                            data = &data[n..];
                            if i < 2 {
                                data = &data[1..];
                            }
                            *slot = (v * 255 / 100) as u8;
                        }
                        palette.insert(index, rgb);
                    }
                    color = palette.get(&index).copied().unwrap_or([0; 3]);
                }
                b'$' => x = 0,
                b'-' => {
                    x = 0;
                    band += 1;
                }
                b'!' => {
                    let (count, n) = number(data);
                    data = &data[n..];
                    let ch = data[0];
                    data = &data[1..];
                    for _ in 0..count {
                        put(&mut pixels, (width, height), (x, band), ch, color);
                        x += 1;
                    }
                }
                b'?'..=b'~' => {
                    put(&mut pixels, (width, height), (x, band), c, color);
                    x += 1;
                }
                _ => {}
            }
        }
        found.push(Sixel {
            width,
            height,
            pixels,
        });
    }
    found
}

fn put(
    pixels: &mut [Option<[u8; 3]>],
    (width, height): (usize, usize),
    (x, band): (usize, usize),
    ch: u8,
    color: [u8; 3],
) {
    for bit in 0..6 {
        let y = band * 6 + bit;
        if (ch - 63) & (1 << bit) != 0 && x < width && y < height {
            pixels[y * width + x] = Some(color);
        }
    }
}

fn run(session: &str, ui: &str) -> Vec<u8> {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("first of the sixel graph")
        .unwrap()
        .commit("second of the sixel graph")
        .unwrap()
        .commit(MESSAGE)
        .unwrap()
        .build();
    let config = tempfile::tempdir().unwrap();
    std::fs::write(config.path().join("config.toml"), format!("[ui]\n{ui}")).unwrap();
    let cache = config.path().join("cache").join("avatars-v2");
    std::fs::create_dir_all(&cache).unwrap();
    std::fs::write(
        cache.join(format!("{}.img", hash("testkit@gitpoulpi.local"))),
        gradient_png(),
    )
    .unwrap();
    let term = common::launch(repo.root())
        .session(session)
        .config_dir(config.path())
        .size(200, 50)
        .terminal_image()
        .env("GITPOULPI_TERMINAL_CELL_PX", "9x21")
        .env("TERM", "xterm-256color")
        .env("HERDR_ENV", "")
        .env("HERDR_PANE_ID", "")
        .env("HERDR_SOCKET_PATH", "")
        .env("TMUX", "")
        .env("ZELLIJ", "")
        .env("STY", "")
        .env("TERM_PROGRAM", "")
        .env("WEZTERM_EXECUTABLE", "")
        .env("KITTY_WINDOW_ID", "")
        .env("GHOSTTY_RESOURCES_DIR", "")
        .spawn();
    term.wait_for("the fixture loaded", STEP, |s| s.contains(MESSAGE))
        .unwrap_or_else(|e| panic!("{e}"));
    term.wait_for_stable(Duration::from_millis(1500), STEP)
        .unwrap();
    let raw = term.raw_output();
    let _ = term.stop();
    raw
}

fn count(haystack: &[u8], needle: &[u8]) -> usize {
    haystack
        .windows(needle.len())
        .filter(|w| *w == needle)
        .count()
}

#[test]
#[cfg_attr(
    windows,
    ignore = "ConPTY drops the sixel DCS from the stream the driver reads"
)]
fn the_graph_is_a_sixel_picture_with_the_avatars_in_its_nodes() {
    let raw = run(
        "pty-graph-sixel",
        "image_protocol = \"sixel\"\ngraph_renderer = \"image\"\navatars = \"graph\"\n",
    );
    let all = sixels(&raw);
    let graph = all
        .iter()
        .filter(|s| s.height >= 3 * 21 && s.width % 9 == 0 && s.height % 21 == 0)
        .max_by_key(|s| s.colors())
        .unwrap_or_else(|| {
            panic!(
                "no graph-sized sixel among {:?}",
                all.iter().map(|s| (s.width, s.height)).collect::<Vec<_>>()
            )
        });
    // Lanes and nodes are drawn, the rest is transparent.
    assert!(graph.drawn() > 200, "{} pixels drawn", graph.drawn());
    assert!(graph.drawn() < graph.pixels.len() / 2, "mostly transparent");
    // The face's gradient is in there: far more colors than lanes have.
    assert!(graph.colors() > 20, "{} colors", graph.colors());
    // Never the other protocols.
    assert_eq!(count(&raw, b"]1337;File="), 0);
    assert_eq!(count(&raw, b"_Ga="), 0);
}

#[test]
fn pictures_off_keeps_the_text_graph() {
    let raw = run(
        "pty-graph-sixel-off",
        "image_protocol = \"none\"\navatars = \"graph\"\n",
    );
    assert_eq!(count(&raw, b"\x1bP"), 0, "a DCS string was sent");
}
