//! A repository opened for the first time fits every Log column to its
//! content; a width the user gives a column is kept per repository and comes
//! back on the next launch (not fitted again). Temp config dirs only.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::{Screen, Terminal};

const STEP: Duration = Duration::from_secs(30);
const COMMIT: &str = "fit new repo probe commit";
const AUTHOR: &str = "Bartholomew Longname-Author";

fn ms(n: u64) {
    std::thread::sleep(Duration::from_millis(n));
}

fn fixture() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .config("user.name", AUTHOR)
        .unwrap()
        .commit(COMMIT)
        .unwrap()
        .build()
}

fn launch(repo: &FixtureRepo, session: &str, config: &std::path::Path, autofit: bool) -> Terminal {
    common::launch(repo.root())
        .session(session)
        .config_dir(config)
        .size(200, 40)
        // The harness turns the first-open fit off by default.
        .env("GITPOULPI_NO_LOG_AUTOFIT", if autofit { "0" } else { "1" })
        .ready_when("the Log header", |s| {
            s.contains(COMMIT) && header(s, "Author").is_some()
        })
        .mouse_capture()
        .spawn()
}

/// Cell of `title` on the Log's header row (the row of `Message`).
fn header(s: &Screen, title: &str) -> Option<(u16, u16)> {
    let (_, row) = s.find_all("Message").into_iter().next()?;
    s.find_all(title).into_iter().find(|(_, r)| *r == row)
}

/// The Log row's Author cell, between the Author and Date titles.
fn author_cell(s: &Screen) -> String {
    let (Some((from, row)), Some((to, _))) = (header(s, "Author"), header(s, "Date")) else {
        return String::new();
    };
    (from..to)
        .map(|c| {
            s.cell(c, row + 1)
                .map(|c| c.display().to_string())
                .unwrap_or_default()
        })
        .collect()
}

fn full_author(s: &Screen) -> bool {
    author_cell(s).contains(AUTHOR)
}

fn snap(term: &Terminal) -> Screen {
    term.screen()
}

fn quit(mut term: Terminal) {
    term.send_keys("q").unwrap();
    for _ in 0..100 {
        if term.exited() {
            break;
        }
        ms(100);
    }
    let _ = term.stop();
}

fn state_file(repo: &FixtureRepo) -> std::path::PathBuf {
    gitpoulpi_config::repo_data_file(
        repo.root(),
        gitpoulpi_config::DEFAULT_REPO_DATA_LOCATION,
        "repo-state.toml",
    )
}

/// First open: no key, no click, the long author name is already whole. A
/// drag on the Author boundary then narrows it, the widths reach the
/// repository's state file, and the next launch shows the dragged widths (the
/// name stays cut: it is not fitted again).
///
/// Falsified two ways: with the tick's first-open fit removed the name stays
/// cut at "fitted on first open"; with the restore (`for_repo`) ignoring the
/// saved widths the relaunch fits again and the name is whole (red at "the
/// saved width is restored").
#[test]
fn a_new_repository_is_fitted_and_a_dragged_width_comes_back() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch(&repo, "fit-new-repo", config.path(), true);
    term.wait_for("fitted on first open", STEP, full_author)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let s = snap(&term);
    let (date_x, row) = header(&s, "Date").unwrap();
    let (author_x, _) = header(&s, "Author").unwrap();
    let gap = date_x - 1;
    term.drag((gap, row + 3), (gap - 8, row + 3)).unwrap();
    term.wait_for("the name cut by the drag", STEP, |s| !full_author(s))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let dragged = snap(&term);
    let dragged_date = header(&dragged, "Date").unwrap().0;
    assert!(dragged_date < date_x, "Date moved left\n{}", term.text());
    let path = state_file(&repo);
    let mut saved = false;
    for _ in 0..100 {
        if std::fs::read_to_string(&path).is_ok_and(|t| t.contains("log_columns")) {
            saved = true;
            break;
        }
        ms(100);
    }
    assert!(saved, "the widths reach {}", path.display());
    quit(term);

    let term = launch(&repo, "fit-new-repo", config.path(), true);
    term.wait_for("the saved width is restored", STEP, |s| {
        header(s, "Author").map(|h| h.0) == Some(author_x)
            && header(s, "Date").map(|h| h.0) == Some(dragged_date)
            && !full_author(s)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // Give a (wrong) late fit its chance to show.
    ms(800);
    assert_eq!(
        header(&snap(&term), "Date").map(|h| h.0),
        Some(dragged_date),
        "no second fit\n{}",
        term.text()
    );
    quit(term);
}
