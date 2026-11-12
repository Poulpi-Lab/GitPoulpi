//! "Configure..." in the tab bar's Open in menu lands on the Settings Open in
//! page at once, even with a huge `PATH`: preset detection never runs on the
//! input thread (it is one shared, cached, background `PATH` listing).
//!
//! Temp config dir, fixture repos and a throwaway `PATH` tail only; nothing
//! is launched.

mod common;

use std::time::{Duration, Instant};

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(30);
const STARTUP: Duration = Duration::from_secs(60);
/// Generous for a loaded CI box, far below what the old per-name `PATH` probe
/// cost on the long `PATH` below.
const PAGE_DEADLINE: Duration = Duration::from_millis(1500);

fn click_text(term: &mut Terminal, text: &str) {
    let (col, row) = term
        .screen()
        .find_all(text)
        .into_iter()
        .next()
        .unwrap_or_else(|| panic!("{text:?} not on screen:\n{}", term.text()));
    term.click(col + 1, row).unwrap();
}

/// A `PATH` that is the real one plus many empty directories: every program
/// not found costs a probe per directory when asked one name at a time.
fn long_path(junk: &std::path::Path) -> String {
    // Windows caps the whole environment block at 32 KiB: fewer directories there.
    let count = if cfg!(windows) { 500 } else { 6000 };
    let mut dirs: Vec<std::path::PathBuf> =
        std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default()).collect();
    for i in 0..count {
        let d = junk.join(i.to_string());
        std::fs::create_dir(&d).unwrap();
        dirs.push(d);
    }
    std::env::join_paths(dirs)
        .unwrap()
        .to_string_lossy()
        .into_owned()
}

fn start_on_bravo(
    alpha: &gitpoulpi_testkit::FixtureRepo,
    bravo: &gitpoulpi_testkit::FixtureRepo,
    config: &std::path::Path,
    path: &str,
) -> Terminal {
    std::fs::write(
        config.join("recent.toml"),
        format!(
            "[[repos]]\npath = '{}'\nname = 'bravo-repo'\nlast_opened = 1\n",
            bravo.root().display()
        ),
    )
    .unwrap();
    std::fs::write(
        config.join("config.toml"),
        "[ui]\ngraph_renderer = \"text\"\n\
         [[ui.open_in]]\nlabel = \"First entry\"\ncommand = \"true ${cwd}\"\n\
         [[ui.open_in]]\nlabel = \"Second entry\"\ncommand = \"true ${cwd}\"\n",
    )
    .unwrap();
    let mut term = common::launch(alpha.root())
        .session("open-in-configure")
        .config_dir(config)
        .env("PATH", path)
        .size(200, 50)
        .no_exit_pause()
        .ready("alpha-only-commit")
        .spawn();
    term.send_keys("C-o").unwrap();
    term.wait_for("the management tab", STEP, |s| {
        s.contains("Repository Management") && s.contains("bravo-repo")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("/").unwrap();
    term.send_text("bravo").unwrap();
    term.wait_for("the filter applied", STEP, |s| {
        s.contains("Enter open best match")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    term.wait_for("bravo open as a tab", STARTUP, |s| {
        s.contains("bravo-only-commit") && !s.contains("Repository Management")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.wait_for("the Open in button", STEP, |s| s.contains("Open in"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term
}

#[test]
fn configure_from_the_open_in_menu_shows_the_page_at_once_on_a_huge_path() {
    let alpha = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "a\n", "alpha-only-commit")
        .unwrap()
        .build();
    let bravo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("b.txt", "b\n", "bravo-only-commit")
        .unwrap()
        .build();
    let config = tempfile::tempdir().unwrap();
    let junk = tempfile::tempdir().unwrap();
    let path = long_path(junk.path());
    let mut term = start_on_bravo(&alpha, &bravo, config.path(), &path);

    click_text(&mut term, "Open in");
    term.wait_for("the Open in list", STEP, |s| {
        s.contains("Second entry") && s.contains("Configure")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let t0 = Instant::now();
    click_text(&mut term, "Configure");
    term.wait_for("the Open in page", PAGE_DEADLINE, |s| {
        s.contains("Default entry")
    })
    .unwrap_or_else(|e| {
        panic!(
            "page not there within {PAGE_DEADLINE:?}: {e}\n{}",
            term.text()
        )
    });
    eprintln!("CONFIGURE-TO-PAGE {} ms", t0.elapsed().as_millis());
    term.stop().unwrap();
}
