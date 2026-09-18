//! Lazy loading of the Log, end to end through the real binary: scrolling
//! to the trigger must (1) keep the viewport where it is when the next
//! page is appended and (2) load one configured page (500 here).
//!
//! Fixture: 2500 commits built with `git fast-import` (one process, not
//! 2500 commits). Config: initial load 500 (the floor), a 1000-commit
//! ceiling, so the ceiling note ("1000 commits -- graph.max reached")
//! reports exactly how many commits the client ended up holding.

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

/// Number of the first `commit-NNNN` visible in the Log pane.
fn top_visible(screen: &xtask::pty::Screen) -> Option<usize> {
    let text = screen.text();
    let idx = text.find("commit-")?;
    text.get(idx + 7..idx + 11)?.parse().ok()
}

#[test]
fn scrolling_to_the_lazy_load_trigger_keeps_the_view_and_loads_a_big_page() {
    let repo = big_repo();
    let config = tempfile::tempdir().expect("temp config dir");
    std::fs::write(
        config.path().join("config.toml"),
        "[general]\ninitial_commits_in_graph = 500\nlazy_load_commits_in_graph = true\n\
         commits_per_lazy_page = 500\nmax_commits_in_graph = 1000\n",
    )
    .unwrap();
    let mut term = common::launch(repo.path())
        .session("pty-log-paging")
        .config_dir(config.path())
        .size(COLS, ROWS)
        .spawn();
    term.wait_for("HEAD in the Log", STARTUP, |s| s.contains("commit-2500"))
        .unwrap_or_else(|e| panic!("{e}"));
    let (col, row) = term
        .wait_for_text("\u{256e}\u{256d}", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    let (wheel_col, wheel_row) = (col + 3, row + 4);

    // Wheel down toward the end of the 500 initially held commits
    // (commit-2500 .. commit-2001). The trigger fires within ~200 rows of
    // the bottom; keep scrolling until the ceiling note proves the page
    // arrived, recording the top row just before it does.
    let mut before = None;
    let mut after = None;
    for _ in 0..400 {
        term.scroll(true, wheel_col, wheel_row).unwrap();
        let screen = term.screen();
        if screen.contains("graph.max reached") {
            after = top_visible(&screen);
            break;
        }
        if let Some(t) = top_visible(&screen) {
            before = Some(t);
        }
    }
    term.wait_for("1000 commits held (500 + a 500-commit page)", STEP, |s| {
        s.contains("1000 commits - graph.max reached")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    let after = after
        .or_else(|| top_visible(&term.screen()))
        .expect("a commit is visible");
    let before = before.expect("scrolled at least once");
    assert!(
        before < 2500 - 100,
        "the wheel really scrolled before the page arrived (top was commit-{before})"
    );
    // The view stayed: the top commit is still around where it was (a few
    // wheel notches of drift at most), not back at HEAD (commit-2500).
    assert!(
        after.abs_diff(before) < 100,
        "the viewport jumped when the page was appended: top commit-{before} -> commit-{after}"
    );
    term.stop().unwrap();
}
