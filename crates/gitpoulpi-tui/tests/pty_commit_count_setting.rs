//! `log.commit_count` decides how the status bar learns its total:
//! `"off"` asks for nothing and shows only the loaded count, `"libgit2"`
//! counts in-process and still shows `loaded / total`.
//!
//! Fixture: 1200 commits, initial load 500 (so the total differs from the
//! loaded count).

mod common;

use common::git_ok;

use std::io::Write;
use std::process::{Command, Stdio};
use std::time::Duration;

const STARTUP: Duration = Duration::from_secs(60);
const STEP: Duration = Duration::from_secs(30);
const COLS: u16 = 200;
const ROWS: u16 = 50;
const COMMITS: usize = 1200;

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

/// The status bar's commit segment ("main - 500 / 1200 commits - clean").
fn commits_segment(screen: &xtask::pty::Screen) -> Option<String> {
    let text = screen.text();
    let end = text.find(" commits \u{2022}")?;
    let segment = text[..end].rsplit('\u{2022}').next()?;
    Some(segment.trim().to_string())
}

fn launch(repo: &std::path::Path, method: &str, config: &std::path::Path) -> xtask::pty::Terminal {
    std::fs::write(
        config.join("config.toml"),
        format!(
            "[general]\ninitial_commits_in_graph = 500\nlazy_load_commits_in_graph = true\n\
             commits_per_lazy_page = 300\n[log]\ncommit_count = \"{method}\"\n"
        ),
    )
    .unwrap();
    common::launch(repo)
        .session(&format!("pty-commit-count-{method}"))
        .config_dir(config)
        .size(COLS, ROWS)
        .spawn()
}

#[test]
fn off_shows_only_the_loaded_count_and_never_a_total() {
    let repo = big_repo();
    let config = tempfile::tempdir().expect("temp config dir");
    let term = launch(repo.path(), "off", config.path());
    term.wait_for("HEAD in the Log", STARTUP, |s| s.contains("commit-1200"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.wait_for("the plain loaded count", STEP, |s| {
        commits_segment(s).as_deref() == Some("500")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    // The default would have shown the total within a second or two.
    let until = std::time::Instant::now() + Duration::from_secs(6);
    while std::time::Instant::now() < until {
        let seg = commits_segment(&term.screen());
        assert!(
            seg.as_deref().is_none_or(|s| !s.contains('/')),
            "no total when counting is off: {seg:?}"
        );
        std::thread::sleep(Duration::from_millis(200));
    }
    term.stop().unwrap();
}

#[test]
fn libgit2_counts_in_process_and_shows_the_total() {
    let repo = big_repo();
    let config = tempfile::tempdir().expect("temp config dir");
    let term = launch(repo.path(), "libgit2", config.path());
    term.wait_for("HEAD in the Log", STARTUP, |s| s.contains("commit-1200"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.wait_for("500 / 1200 in the status bar", STEP, |s| {
        commits_segment(s).as_deref() == Some("500 / 1200")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.stop().unwrap();
}
