//! The commit-detail avatar over the Kitty graphics protocol, end to end
//! through a pty. The avatar comes from the disk cache (`cache/avatars-v2`
//! under the temp config dir), as a hosting avatar would after its fetch:
//! no network. Same rules as `pty_ui.rs`.
//!
//! Two things are pinned: the upload is the picture at the box's own pixel
//! size (a GitHub original is several times bigger than the box: it must
//! not reach the terminal as a long chunked upload), and an author with no
//! picture gets the initials in the box, never blank reserved cells.

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

/// A GitHub-like original: bigger than the box, not square, with alpha.
fn big_png() -> Vec<u8> {
    let (w, h) = (460u32, 300u32);
    let mut out = Vec::new();
    {
        let mut enc = png::Encoder::new(&mut out, w, h);
        enc.set_color(png::ColorType::Rgba);
        enc.set_depth(png::BitDepth::Eight);
        let mut writer = enc.write_header().unwrap();
        let mut data = Vec::new();
        let mut seed = 1u64;
        for y in 0..h {
            for x in 0..w {
                seed = seed
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                data.extend_from_slice(&[
                    (seed >> 33) as u8,
                    (seed >> 41) as u8 ^ (x as u8),
                    (y % 256) as u8,
                    255,
                ]);
            }
        }
        writer.write_image_data(&data).unwrap();
    }
    out
}

fn config_with_cached_avatar(png: Option<Vec<u8>>) -> tempfile::TempDir {
    let config = tempfile::tempdir().unwrap();
    let sources = if png.is_some() {
        ""
    } else {
        "avatar_sources = \"hosting\"\navatar_fallback = \"initials\"\n"
    };
    std::fs::write(
        config.path().join("config.toml"),
        format!("[ui]\navatars = \"author\"\nimage_protocol = \"kitty\"\n{sources}"),
    )
    .unwrap();
    if let Some(png) = png {
        let cache = config.path().join("cache").join("avatars-v2");
        std::fs::create_dir_all(&cache).unwrap();
        std::fs::write(
            cache.join(format!("{}.img", hash("testkit@gitpoulpi.local"))),
            png,
        )
        .unwrap();
    }
    config
}

fn launch_kitty(
    repo: &std::path::Path,
    session: &str,
    config: &std::path::Path,
) -> xtask::pty::Terminal {
    common::launch(repo)
        .session(session)
        .config_dir(config)
        .size(200, 50)
        .terminal_image()
        .env("GITPOULPI_TERMINAL_CELL_PX", "9x21")
        .env("TERM", "xterm-kitty")
        .spawn()
}

/// The `(width, height)` of the PNG a Kitty upload (`a=T`) carried for an
/// image `cols` cells wide, reassembled across its chunks.
fn uploaded_png_size(raw: &str, cols: u16, rows: u16) -> Option<(u32, u32)> {
    let head = format!(",c={cols},r={rows},");
    let at = raw.find("\x1b_Ga=T,U=1")?;
    let mut rest = &raw[at..];
    let mut payload = String::new();
    loop {
        let end = rest.find("\x1b\\")?;
        let escape = &rest[..end];
        let (control, data) = escape.split_once(';')?;
        if payload.is_empty() && !control.contains(&head) {
            // Another picture's upload (the Graph column's, the Log's).
            rest = &rest[end + 2..];
            let next = rest.find("\x1b_Ga=T,U=1")?;
            rest = &rest[next..];
            continue;
        }
        payload.push_str(data);
        if control.contains("m=0") {
            break;
        }
        rest = &rest[end + 2..];
        if !rest.starts_with("\x1b_Gm=") {
            return None;
        }
    }
    let png = unbase64(&payload);
    Some((
        u32::from_be_bytes(png.get(16..20)?.try_into().ok()?),
        u32::from_be_bytes(png.get(20..24)?.try_into().ok()?),
    ))
}

fn unbase64(text: &str) -> Vec<u8> {
    const A: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let (mut out, mut acc, mut bits) = (Vec::new(), 0u32, 0);
    for c in text.chars().filter(|c| *c != '=') {
        acc = (acc << 6) | A.find(c).unwrap() as u32;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((acc >> bits) as u8);
            acc &= (1 << bits) - 1;
        }
    }
    out
}

/// The upload is the decoded picture as is (a downscale to the box's 1x
/// pixels lost the supersample and the alpha channel); the placeholders
/// must be on screen.
#[test]
#[cfg_attr(
    windows,
    ignore = "ConPTY drops the kitty graphics APC from the stream the driver reads"
)]
fn the_detail_avatar_is_uploaded_and_placed_over_kitty() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("detail avatar upload")
        .unwrap()
        .build();
    let config = config_with_cached_avatar(Some(big_png()));
    let term = launch_kitty(repo.root(), "pty-avatar-detail-kitty", config.path());
    term.wait_for("the commit", STEP, |s| s.contains("detail avatar upload"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.wait_for_stable(Duration::from_millis(1500), STEP)
        .unwrap();
    let raw = String::from_utf8_lossy(&term.raw_output()).into_owned();
    let _ = term.stop();
    assert!(uploaded_png_size(&raw, 5, 2).is_some(), "picture uploaded");
    assert!(
        raw.matches('\u{10EEEE}').count() >= 10,
        "placeholders drawn"
    );
}

/// Falsified before the fix: one lone initial in a box of blank cells.
#[test]
#[cfg_attr(
    windows,
    ignore = "ConPTY repaints the kitty placeholder cells (U+10EEEE + combining marks) out of place, so the screen text is not the app's grid"
)]
fn an_author_without_a_picture_gets_initials_in_the_detail_box() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("detail avatar fallback")
        .unwrap()
        .build();
    let config = config_with_cached_avatar(None);
    let term = launch_kitty(repo.root(), "pty-avatar-detail-fallback", config.path());
    term.wait_for("the commit", STEP, |s| s.contains("detail avatar fallback"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.wait_for_stable(Duration::from_millis(1500), STEP)
        .unwrap();
    let screen = term.text();
    let raw = String::from_utf8_lossy(&term.raw_output()).into_owned();
    let _ = term.stop();
    // Both letters of the initials: the Log's slot and the detail box.
    let both = screen.matches("GT ").count();
    assert!(
        both >= 2,
        "initials in the Log row and the detail box:\n{screen}"
    );
    assert_eq!(uploaded_png_size(&raw, 5, 2), None, "no picture to upload");
}
