//! What GitPoulpi writes inside tmux, end to end through a pty: the pointer
//! shape and a forced kitty upload go out wrapped in a DCS passthrough (bare,
//! tmux drops them), sixel goes out bare when tmux's client advertises it, and
//! nothing is wrapped behind any other multiplexer.
//!
//! The environment says "inside tmux" (`TMUX`, `TERM=tmux-256color`); the
//! tmux binary is a stub on `PATH` that answers the one question detection
//! asks (`#{client_termfeatures}`), so no tmux server runs.
//!
//! The wire facts (what tmux 3.8 passes on, in kitty and foot) were measured
//! with raw escape sequences, see `terminal_image/passthrough.rs`.
#![cfg(unix)]

mod common;

use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;
use sha2::{Digest, Sha256};

const STARTUP: Duration = Duration::from_secs(60);
const STEP: Duration = Duration::from_secs(30);

const TMUX_ENV: &str = "/tmp/tmux-1000/default,1234,0";

fn count(haystack: &[u8], needle: &[u8]) -> usize {
    haystack
        .windows(needle.len())
        .filter(|w| *w == needle)
        .count()
}

/// A directory holding a `tmux` that prints `features` for any arguments.
fn stub_tmux(features: &str) -> tempfile::TempDir {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("tmux");
    std::fs::write(&path, format!("#!/bin/sh\necho '{features}'\n")).unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    dir
}

fn path_with(dir: &std::path::Path) -> String {
    format!(
        "{}:{}",
        dir.display(),
        std::env::var("PATH").unwrap_or_default()
    )
}

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

/// A config with author avatars from the disk cache (no network).
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

/// The environment of a pane of `mux` (`tmux`, `screen` or `none`), with
/// every other multiplexer and terminal variable blanked.
fn pane(launch: common::Launch, mux: &str) -> common::Launch {
    let launch = launch
        .env("HERDR_ENV", "")
        .env("HERDR_PANE_ID", "")
        .env("HERDR_SOCKET_PATH", "")
        .env("ZELLIJ", "")
        .env("TERM_PROGRAM", "")
        .env("WEZTERM_EXECUTABLE", "")
        .env("KITTY_WINDOW_ID", "")
        .env("GHOSTTY_RESOURCES_DIR", "");
    match mux {
        "tmux" => launch
            .env("TMUX", TMUX_ENV)
            .env("STY", "")
            .env("TERM", "tmux-256color")
            .env("TERM_PROGRAM", "tmux"),
        "screen" => launch
            .env("TMUX", "")
            .env("STY", "1234.pts-0.host")
            .env("TERM", "screen-256color"),
        _ => launch
            .env("TMUX", "")
            .env("STY", "")
            .env("TERM", "xterm-256color"),
    }
}

/// Runs the TUI over a repository with one commit and returns everything it
/// wrote once the Log is up and has settled.
fn run_picture(session: &str, protocol_line: &str, features: &str, mux: &str) -> Vec<u8> {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("tmux passthrough probe")
        .unwrap()
        .build();
    let config = config(protocol_line);
    let bin = stub_tmux(features);
    let term = pane(
        common::launch(repo.root())
            .session(session)
            .config_dir(config.path())
            .size(200, 50)
            .terminal_image()
            .env("PATH", path_with(bin.path()))
            .env("GITPOULPI_TERMINAL_CELL_PX", "9x21"),
        mux,
    )
    .spawn();
    term.wait_for("the commit", STARTUP, |s| {
        s.contains("tmux passthrough probe")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.wait_for_stable(Duration::from_millis(1500), STEP)
        .unwrap();
    let raw = term.raw_output();
    let _ = term.stop();
    raw
}

const SIXEL: &[u8] = b"\x1bP0;1;0q\"1;1;";

#[test]
fn tmux_in_a_sixel_terminal_gets_sixel_pictures_without_any_setting() {
    let raw = run_picture(
        "pty-tmux-sixel",
        "",
        "256,bpaste,clipboard,RGB,sixel,sync,title",
        "tmux",
    );
    let sixels = count(&raw, SIXEL);
    assert!(sixels >= 2, "{sixels} sixel pictures sent");
    // tmux draws sixel itself: never wrapped, never another protocol.
    assert_eq!(count(&raw, b"\x1bPtmux;"), 0, "sixel must not be wrapped");
    assert_eq!(count(&raw, b"_Ga="), 0);
    assert_eq!(count(&raw, b"]1337;File="), 0);
}

#[test]
fn tmux_in_a_terminal_without_sixel_stays_text() {
    let raw = run_picture(
        "pty-tmux-nosixel",
        "",
        "bpaste,clipboard,RGB,sync,title",
        "tmux",
    );
    assert_eq!(count(&raw, SIXEL), 0);
    assert_eq!(count(&raw, b"_Ga="), 0);
}

#[test]
fn forced_kitty_in_tmux_is_written_inside_a_passthrough() {
    let raw = run_picture(
        "pty-tmux-kitty",
        "image_protocol = \"kitty\"\n",
        "bpaste,clipboard,RGB,sync,title",
        "tmux",
    );
    let apcs = count(&raw, b"_Ga=");
    assert!(apcs >= 1, "no kitty upload was sent");
    // Every APC sits inside `ESC P tmux ; ESC ESC _ G`: a bare one is dropped
    // by tmux and leaves the picture blank.
    assert_eq!(
        count(&raw, b"\x1bPtmux;\x1b\x1b_Ga="),
        apcs,
        "a kitty upload went out bare"
    );
}

#[test]
fn forced_kitty_outside_tmux_is_not_wrapped() {
    let raw = run_picture(
        "pty-notmux-kitty",
        "image_protocol = \"kitty\"\n",
        "",
        "none",
    );
    assert!(count(&raw, b"_Ga=") >= 1, "no kitty upload was sent");
    assert_eq!(count(&raw, b"\x1bPtmux;"), 0);
}

/// Every `OSC 22` payload the program wrote, wrapped or not, as
/// `(wrapped, shape)`.
fn pointer_writes(raw: &[u8]) -> Vec<(bool, String)> {
    let text = String::from_utf8_lossy(raw).to_string();
    let mut found = Vec::new();
    let mut rest = text.as_str();
    while let Some(at) = rest.find("]22;") {
        let wrapped = rest[..at].ends_with("\x1bPtmux;\x1b\x1b");
        let tail = &rest[at + 4..];
        let end = tail.find('\x1b').unwrap_or(tail.len());
        found.push((wrapped, tail[..end].to_string()));
        rest = &tail[end..];
    }
    found
}

/// Hovers the Radar tab (a `pointer` shape) in a pane of `mux` and returns
/// what was written about the pointer.
fn hover_radar(session: &str, mux: &str) -> Vec<(bool, String)> {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "a\n", "pointer-tmux-commit")
        .unwrap()
        .build();
    let config = tempfile::tempdir().unwrap();
    let mut term = pane(
        common::launch(repo.root())
            .session(session)
            .config_dir(config.path())
            .size(180, 45)
            .no_exit_pause()
            .mouse_capture(),
        mux,
    )
    .spawn();
    term.wait_for("the window", STARTUP, |s| {
        s.contains("pointer-tmux-commit")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.wait_for("the Radar tab", STEP, |s| s.line(1).contains("Radar"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let bar = term.screen().line(1);
    let at = bar.find("Radar").unwrap();
    let col = bar[..at].chars().count() as u16 + 1;
    term.move_to(col, 1).unwrap();
    let deadline = std::time::Instant::now() + STEP;
    // Give the write time to show up; behind screen there is none to wait
    // for, so a short fixed look is all that can be said.
    if mux == "screen" {
        std::thread::sleep(Duration::from_millis(1500));
    }
    while mux != "screen"
        && pointer_writes(&term.raw_output()).is_empty()
        && std::time::Instant::now() < deadline
    {
        std::thread::sleep(Duration::from_millis(50));
    }
    std::thread::sleep(Duration::from_millis(300));
    let writes = pointer_writes(&term.raw_output());
    term.stop().unwrap();
    writes
}

#[test]
fn the_pointer_shape_in_tmux_goes_through_a_passthrough() {
    let writes = hover_radar("pty-tmux-pointer", "tmux");
    assert!(
        writes.contains(&(true, "pointer".to_string())),
        "no wrapped pointer shape: {writes:?}"
    );
    assert!(
        writes.iter().all(|(wrapped, _)| *wrapped),
        "a bare OSC 22 reached tmux: {writes:?}"
    );
}

#[test]
fn the_pointer_shape_outside_a_multiplexer_stays_bare() {
    let writes = hover_radar("pty-nomux-pointer", "none");
    assert!(
        writes.contains(&(false, "pointer".to_string())),
        "no pointer shape: {writes:?}"
    );
    assert!(writes.iter().all(|(wrapped, _)| !*wrapped), "{writes:?}");
}

#[test]
fn behind_screen_no_pointer_shape_is_written() {
    // Screen swallows OSC 22 and has no passthrough that carries it.
    let writes = hover_radar("pty-screen-pointer", "screen");
    assert!(writes.is_empty(), "{writes:?}");
}
