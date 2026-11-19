//! `ui.avatars = "graph"`: resting the pointer on a commit's picture in the
//! Graph column shows the author tooltip, the one the Author cell shows. With
//! avatars off the plain dot has none. The picture comes from the disk cache
//! (`cache/avatars-v2` under the temp config dir): no network. Same rules as
//! `pty_ui.rs`.

mod common;

use std::time::{Duration, Instant};

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use sha2::{Digest, Sha256};

const STEP: Duration = Duration::from_secs(60);
const SHORT: Duration = Duration::from_millis(2500);
const EMAIL: &str = "testkit@gitpoulpi.local";

fn png() -> Vec<u8> {
    let (w, h) = (64u32, 64u32);
    let mut out = Vec::new();
    {
        let mut enc = png::Encoder::new(&mut out, w, h);
        enc.set_color(png::ColorType::Rgba);
        enc.set_depth(png::BitDepth::Eight);
        let mut writer = enc.write_header().unwrap();
        let data: Vec<u8> = (0..w * h).flat_map(|_| [200u8, 60, 40, 255]).collect();
        writer.write_image_data(&data).unwrap();
    }
    out
}

struct Run {
    term: xtask::pty::Terminal,
    gx: u16,
    /// The commit's row: the line under the header.
    row: u16,
    _repo: FixtureRepo,
    _config: tempfile::TempDir,
}

fn launch(avatars: &str, session: &str) -> Run {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("the avatar tip commit")
        .unwrap()
        .build();
    let config = tempfile::tempdir().unwrap();
    std::fs::write(
        config.path().join("config.toml"),
        format!("[ui]\navatars = \"{avatars}\"\nimage_protocol = \"kitty\"\n"),
    )
    .unwrap();
    let cache = config.path().join("cache").join("avatars-v2");
    std::fs::create_dir_all(&cache).unwrap();
    let hash: String = Sha256::digest(EMAIL.as_bytes())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    std::fs::write(cache.join(format!("{hash}.img")), png()).unwrap();
    let term = common::launch(repo.root())
        .session(session)
        .config_dir(config.path())
        .size(200, 50)
        .terminal_image()
        .mouse_capture()
        .env("GITPOULPI_TERMINAL_CELL_PX", "9x21")
        .env("TERM", "xterm-kitty")
        .spawn();
    let screen = term
        .wait_for("the commit row and the Graph header", STEP, |s| {
            s.contains("the avatar tip commit") && s.contains("Graph")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let (gx, header_row) = screen.find("Graph").expect("the Graph header");
    Run {
        term,
        gx,
        row: header_row + 1,
        _repo: repo,
        _config: config,
    }
}

/// Rests the pointer on one of the dot's two cells (the picture is two cells
/// wide, the Graph column's first cell and the next) until the email shows
/// or `limit` passes. The picture arrives a moment after the first frame and
/// a pointer that rested on the bare dot meanwhile has no tooltip until it
/// moves, so the pointer re-enters the cell every couple of seconds.
fn hover_dot(run: &mut Run, dx: u16, limit: Duration) -> bool {
    let row = run.row;
    let deadline = Instant::now() + limit;
    while Instant::now() < deadline {
        run.term.move_to(0, 0).unwrap();
        run.term.move_to(run.gx + dx, row).unwrap();
        run.term.move_to(run.gx + dx, row).unwrap();
        let round = Instant::now() + Duration::from_millis(2000);
        while Instant::now() < round.min(deadline) {
            if run.term.text().contains(EMAIL) {
                return true;
            }
            std::thread::sleep(Duration::from_millis(100));
        }
    }
    false
}

#[test]
fn hovering_a_graph_avatar_shows_the_author_tooltip_and_it_goes_when_leaving() {
    let mut run = launch("graph", "pty-graph-avatar-tip");
    assert!(
        !run.term.text().contains(EMAIL),
        "no email before the hover"
    );
    assert!(
        hover_dot(&mut run, 1, STEP),
        "no author tooltip on the avatar\n{}",
        run.term.text()
    );
    run.term.move_to(0, 0).unwrap();
    run.term
        .wait_for_text_to_go(EMAIL, STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", run.term.text()));
    let _ = run.term.stop();
}

#[test]
fn hovering_the_dot_with_avatars_off_shows_no_tooltip() {
    let mut run = launch("off", "pty-graph-avatar-off");
    assert!(
        !hover_dot(&mut run, 1, SHORT),
        "the plain dot grew a tooltip\n{}",
        run.term.text()
    );
    let _ = run.term.stop();
}
