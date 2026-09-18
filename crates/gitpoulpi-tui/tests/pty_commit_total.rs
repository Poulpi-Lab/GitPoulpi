//! The status bar says `loaded / total` without loading the total: the
//! daemon counts the whole walk in the background (`CountCommits`) and the
//! bar shows `500 / 2500 commits` until the last page is held, then drops
//! back to the plain count.
//!
//! Fixture: 2500 commits. Initial load 500, pages of 300.

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

/// The status bar's commit segment ("main - 500 / 2500 commits - clean").
fn commits_segment(screen: &xtask::pty::Screen) -> Option<String> {
    let text = screen.text();
    let end = text.find(" commits \u{2022}")?;
    let segment = text[..end].rsplit('\u{2022}').next()?;
    Some(segment.trim().to_string())
}

#[test]
fn the_status_bar_shows_loaded_over_total_until_everything_is_loaded() {
    let repo = big_repo();
    let config = tempfile::tempdir().expect("temp config dir");
    std::fs::write(
        config.path().join("config.toml"),
        "[general]\ninitial_commits_in_graph = 500\nlazy_load_commits_in_graph = true\n\
         commits_per_lazy_page = 300\n",
    )
    .unwrap();
    let mut term = common::launch(repo.path())
        .session("pty-commit-total")
        .config_dir(config.path())
        .size(COLS, ROWS)
        .spawn();
    term.wait_for("HEAD in the Log", STARTUP, |s| s.contains("commit-2500"))
        .unwrap_or_else(|e| panic!("{e}"));
    // The total arrives on its own, with only the first page held.
    term.wait_for("500 / 2500 in the status bar", STEP, |s| {
        commits_segment(s).as_deref() == Some("500 / 2500")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    // A commit row of the Log, located by its text (not by an offset from
    // a header) once the layout has stopped moving.
    let (col, row) = term
        .wait_for_settled_text("commit-2499", Duration::from_millis(150), STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(col, row).unwrap();
    // `End` jumps to the last loaded row, which loads the next page. Repeat
    // it (idempotent) until everything is held; a keypress swallowed while
    // a page is still arriving is simply sent again.
    let mut moved = None;
    let screen = term
        .act_until(
            "the status bar dropping to the plain 2500",
            Duration::from_millis(500),
            Duration::from_secs(90),
            |t| t.send_keys("End"),
            |s| {
                // Whatever is loaded meanwhile, the total never changes
                // under it (a half-painted bar parses as nothing).
                if let Some(seg) = commits_segment(s) {
                    if seg.contains('/') && !seg.ends_with("/ 2500") {
                        moved = Some(seg.clone());
                    }
                    return seg == "2500";
                }
                false
            },
        )
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(moved, None, "the total moved under the loading");
    assert_eq!(
        commits_segment(&screen).as_deref(),
        Some("2500"),
        "fully loaded drops the slash"
    );
    term.stop().unwrap();
}
