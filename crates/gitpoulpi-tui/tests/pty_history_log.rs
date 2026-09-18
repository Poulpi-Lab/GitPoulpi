//! The File History view over a long history: older pages load as the
//! cursor nears the end of the list, and "Show in Log" finds a commit that
//! is beyond what the Log has loaded.

mod common;

use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Duration;

use common::git_ok;
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(60);
const LOG_HEADER: &str = "Message";

/// `count` commits built with `git fast-import` (one process). `f.txt` is
/// touched by every commit when `every` is true, otherwise only by the
/// first and the last; `g.txt` by all of them.
fn long_repo(count: usize, every: bool) -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("temp repo");
    git_ok(dir.path(), &["init", "-q", "-b", "main"]);
    let mut stream = String::new();
    for n in 1..=count {
        let msg = format!("commit-{n:04}");
        stream.push_str(&format!(
            "commit refs/heads/main\nmark :{n}\ncommitter T <t@example.com> {} +0000\ndata {}\n{msg}\n",
            1_600_000_000 + n * 60,
            msg.len()
        ));
        if n > 1 {
            stream.push_str(&format!("from :{}\n", n - 1));
        }
        if every || n == 1 || n == count {
            stream.push_str(&format!(
                "M 100644 inline f.txt\ndata {}\nf {n}\n\n",
                format!("f {n}\n").len()
            ));
        }
        stream.push_str(&format!(
            "M 100644 inline g.txt\ndata {}\ng {n}\n\n",
            format!("g {n}\n").len()
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

fn launch(repo: &Path, session: &str, config: Option<&str>) -> (Terminal, tempfile::TempDir) {
    let mut launch = common::launch(repo)
        .no_commit_column()
        .session(session)
        .size(180, 40)
        .no_exit_pause()
        .ready_when("the commit detail of HEAD", |s| {
            s.contains("View all files") && s.contains(LOG_HEADER)
        })
        .mouse_capture();
    if let Some(config) = config {
        launch = launch.config(config);
    }
    launch.start()
}

fn open_history_of_f(term: &mut Terminal) {
    let (c, r) = term
        .wait_for_text("f.txt", STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.click(c, r).unwrap();
    let (bc, br) = term
        .wait_for_text(" File \u{2502} All ", STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.click(bc + 2, br).unwrap();
    term.wait_for("the History split", STEP, |s| s.contains("History - f.txt"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

/// The first page holds 100 commits; moving to its end asks for the next
/// one, and the last commit of a 130-commit history is reachable. Red when
/// the list stops at the first page.
#[test]
fn older_pages_load_as_the_cursor_nears_the_end() {
    let repo = long_repo(130, true);
    let (mut term, _config) = launch(repo.path(), "pty-history-paging", None);
    open_history_of_f(&mut term);
    term.wait_for("the newest rows", STEP, |s| s.contains("M commit-0129"))
        .unwrap();
    assert!(
        !term.text().contains("commit-0001"),
        "the oldest commit is not in the first page"
    );
    // End goes to the last loaded row, which asks for the older page; End
    // again goes to the very last commit.
    term.send_keys("End").unwrap();
    term.wait_for("the cursor at the end of the first page", STEP, |s| {
        s.contains("M commit-0031") && !s.contains("M commit-0129")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // The older page arrives behind it: End again reaches the very last
    // commit once it has.
    let mut reached = false;
    for _ in 0..40 {
        term.send_keys("End").unwrap();
        std::thread::sleep(Duration::from_millis(250));
        if term.text().contains("A commit-0001") {
            reached = true;
            break;
        }
    }
    assert!(reached, "the oldest commit never came:\n{}", term.text());
    term.stop().unwrap();
}

/// Enter on a row selects its commit in the Log even when the Log has not
/// loaded that far (500 commits initially, the commit is the 600th back).
/// Red when it answers "not in the loaded Log" and stays in the History.
#[test]
fn show_in_log_loads_the_log_as_far_as_the_commit() {
    let repo = long_repo(600, false);
    let config = "[general]\ninitial_commits_in_graph = 500\nlazy_load_commits_in_graph = true\n\
                  commits_per_lazy_page = 50\n";
    let (mut term, _config) = launch(repo.path(), "pty-history-show-in-log", Some(config));
    open_history_of_f(&mut term);
    term.wait_for("both rows", STEP, |s| {
        s.contains("commit-0600") && s.contains("commit-0001")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Down").unwrap();
    term.send_keys("Enter").unwrap();
    let oldest = String::from_utf8(
        Command::new("git")
            .args(["rev-parse", "--short=7", "main~599"])
            .current_dir(repo.path())
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap();
    let expected = format!("commit: {}", oldest.trim());
    term.wait_for("the Log with the old commit selected", STEP, |s| {
        s.contains(LOG_HEADER) && !s.contains("History - f.txt") && s.contains(&expected)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}
