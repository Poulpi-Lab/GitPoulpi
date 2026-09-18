//! Lazy loading adds exactly `commits_per_lazy_page` commits per page, end to
//! end through the real binary. Red against the old auto-growing page, where
//! the page was as large as everything already held (500 -> 1000 -> 2000).
//!
//! Fixture: 2500 commits (`commit-2500` at HEAD down to `commit-0001`).
//! Initial load 500, page 100, ceiling 700: jumping to the last loaded row
//! with `End` shows the oldest held commit at the bottom, which steps
//! 2001 -> 1901 -> 1801 when each page is exactly 100 and skips 1901 when
//! the first page was 200.

mod common;

use common::git_ok;

use std::io::Write;
use std::process::{Command, Stdio};
use std::time::Duration;

const STARTUP: Duration = Duration::from_secs(60);
const STEP: Duration = Duration::from_secs(30);
const COLS: u16 = 200;
const ROWS: u16 = 50;
const COMMITS: usize = 2500;

fn big_repo() -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("temp repo");
    git_ok(dir.path(), &["init", "-q", "-b", "main"]);
    let mut stream = String::new();
    for n in 1..=COMMITS {
        let msg = format!("commit-{n:04}");
        stream.push_str(&format!(
            "commit refs/heads/main\nmark :{n}\ncommitter T <t@example.com> {} +0000\ndata {}\n{msg}\n",
            1_600_000_000 + n * 60,
            msg.len()
        ));
        if n > 1 {
            stream.push_str(&format!("from :{}\n", n - 1));
        }
        stream.push_str(&format!(
            "M 100644 inline f.txt\ndata {}\n{n}\n\n",
            n.to_string().len()
        ));
    }
    let mut child = Command::new("git")
        .args(["fast-import", "--quiet"])
        .current_dir(dir.path())
        .stdin(Stdio::piped())
        .spawn()
        .expect("spawn fast-import");
    child
        .stdin
        .take()
        .unwrap()
        .write_all(stream.as_bytes())
        .unwrap();
    assert!(child.wait().unwrap().success());
    git_ok(dir.path(), &["reset", "-q", "--hard", "main"]);
    dir
}

/// The loaded commit count from the status bar ("main - 700 commits - clean").
fn loaded_count(screen: &xtask::pty::Screen) -> Option<usize> {
    let text = screen.text();
    let end = text.find(" commits \u{2022}")?;
    // "main - 500 / 2500 commits - clean": the loaded half, before the slash.
    let segment = text[..end].rsplit('\u{2022}').next()?;
    segment.split(" / ").next()?.trim().parse().ok()
}

#[test]
fn each_lazy_page_adds_exactly_the_configured_commit_count() {
    let repo = big_repo();
    let config = tempfile::tempdir().expect("temp config dir");
    std::fs::write(
        config.path().join("config.toml"),
        "[general]\ninitial_commits_in_graph = 500\ncommits_per_lazy_page = 300\n",
    )
    .unwrap();
    let mut term = common::launch(repo.path())
        .session("pty-lazy-graph-pages")
        .config_dir(config.path())
        .size(COLS, ROWS)
        .spawn();
    term.wait_for("HEAD in the Log", STARTUP, |s| s.contains("commit-2500"))
        .unwrap_or_else(|e| panic!("{e}"));
    let (col, row) = term
        .wait_for_text("\u{256e}\u{256d}", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    // Focus the Log without moving the selection.
    term.click(col + 3, row + 4).unwrap();

    // End keeps the view inside the lookahead, so pages chain one after the
    // other; the status bar's count is sampled all along. 500 initial + k * 300,
    // the last page short at the root (2500). The old auto-growing page went
    // 500 -> 1000 -> 2000 -> 2500.
    let allowed = [500, 800, 1100, 1400, 1700, 2000, 2300, 2500];
    let mut seen: Vec<usize> = Vec::new();
    let deadline = std::time::Instant::now() + STEP;
    while std::time::Instant::now() < deadline {
        term.send_keys("End").unwrap();
        if let Some(n) = loaded_count(&term.screen()) {
            if seen.last() != Some(&n) {
                seen.push(n);
            }
            if n == COMMITS {
                break;
            }
        }
    }
    assert_eq!(seen.last(), Some(&COMMITS), "never fully loaded: {seen:?}");
    assert!(
        seen.iter().all(|n| allowed.contains(n)),
        "a page was not exactly 300 commits: {seen:?}"
    );
    term.stop().unwrap();
}
