//! Author avatars over sixel, end to end through a pty: the Log's slots
//! and the commit detail box are sixel pictures (`ESC P ... q`), the
//! avatar comes from the disk cache (no network), and nothing sixel is
//! written when pictures are off. `GITPOULPI_TERMINAL_CELL_PX` pins the
//! geometry; the protocol is forced through `ui.image_protocol` or found
//! from `TERM=foot`. Same rules as `pty_ui.rs`.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;
use sha2::{Digest, Sha256};

const STEP: Duration = Duration::from_secs(60);

fn hash(email: &str) -> String {
    Sha256::digest(email.trim().to_lowercase().as_bytes())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

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

fn config(protocol_line: &str) -> tempfile::TempDir {
    let config = tempfile::tempdir().unwrap();
    std::fs::write(
        config.path().join("config.toml"),
        format!("[ui]\navatars = \"author\"\n{protocol_line}"),
    )
    .unwrap();
    let cache = config.path().join("cache").join("avatars-v2");
    std::fs::create_dir_all(&cache).unwrap();
    std::fs::write(
        cache.join(format!("{}.img", hash("testkit@gitpoulpi.local"))),
        gradient_png(),
    )
    .unwrap();
    config
}

fn count(haystack: &[u8], needle: &[u8]) -> usize {
    haystack
        .windows(needle.len())
        .filter(|w| *w == needle)
        .count()
}

fn run(session: &str, protocol_line: &str, term_env: &str) -> Vec<u8> {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("sixel avatar probe")
        .unwrap()
        .build();
    let config = config(protocol_line);
    let term = common::launch(repo.root())
        .session(session)
        .config_dir(config.path())
        .size(200, 50)
        .terminal_image()
        .env("GITPOULPI_TERMINAL_CELL_PX", "9x21")
        .env("TERM", term_env)
        // A pty inherits the environment this suite runs in, which may be a
        // herdr or tmux pane: detection must see a bare terminal.
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
    term.wait_for("the commit", STEP, |s| s.contains("sixel avatar probe"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.wait_for_stable(Duration::from_millis(1500), STEP)
        .unwrap();
    let raw = term.raw_output();
    let _ = term.stop();
    raw
}

#[test]
#[cfg_attr(
    windows,
    ignore = "ConPTY drops the sixel DCS from the stream the driver reads"
)]
fn forced_sixel_draws_the_log_and_detail_avatars_as_sixel() {
    let raw = run(
        "pty-avatar-sixel",
        "image_protocol = \"sixel\"\n",
        "xterm-256color",
    );
    // The Log slot and the detail box, at least.
    let sixels = count(&raw, b"\x1bP0;1;0q\"1;1;");
    assert!(sixels >= 2, "{sixels} sixel pictures sent");
    // Never the other protocols.
    assert_eq!(count(&raw, b"]1337;File="), 0);
    assert_eq!(count(&raw, b"_Ga="), 0);
    // Every picture is terminated.
    assert!(count(&raw, b"\x1b\\") >= sixels);
}

#[test]
#[cfg_attr(
    windows,
    ignore = "ConPTY drops the sixel DCS from the stream the driver reads"
)]
fn a_foot_terminal_is_detected_and_gets_sixel_without_any_setting() {
    let raw = run("pty-avatar-sixel-foot", "", "foot");
    assert!(count(&raw, b"\x1bP0;1;0q\"1;1;") >= 2);
}

#[test]
fn pictures_off_writes_no_sixel() {
    let raw = run(
        "pty-avatar-sixel-off",
        "image_protocol = \"none\"\n",
        "foot",
    );
    assert_eq!(count(&raw, b"\x1bP"), 0, "a DCS string was sent");
}
