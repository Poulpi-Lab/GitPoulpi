//! Open-to-first-paint measurement (not part of the normal suite):
//!
//! ```text
//! GITPOULPI_PERF_REPO=/path/to/repo cargo test --release -p gitpoulpi-tui \
//!     --test pty_open_perf -- --ignored --nocapture
//! ```
//!
//! Spawns the real TUI (cold: a fresh session, so the daemon start is
//! included) against the repository, and times from the spawn to the
//! frame that first shows the HEAD commit's subject in the graph. That
//! is the user-visible "the graph is here" moment. `GITPOULPI_PERF_RUNS`
//! (default 5) sets the repetitions; the median and the spread are
//! printed. Config is a temp dir each time, never the real one.

mod common;

use std::time::{Duration, Instant};

#[test]
#[ignore = "measurement tool, run by hand"]
fn open_to_first_graph_paint() {
    let Ok(repo) = std::env::var("GITPOULPI_PERF_REPO") else {
        eprintln!("set GITPOULPI_PERF_REPO to a repository path");
        return;
    };
    let runs: usize = std::env::var("GITPOULPI_PERF_RUNS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(5);
    let subject = std::process::Command::new("git")
        .args(["-C", &repo, "log", "-1", "--format=%s"])
        .output()
        .expect("git log");
    let subject = String::from_utf8_lossy(&subject.stdout).trim().to_string();
    let needle: String = subject.chars().take(14).collect();
    let mut times = Vec::new();
    for i in 0..runs {
        let config = tempfile::tempdir().unwrap();
        let started = Instant::now();
        let term = common::launch(&repo)
            .session(&format!("open-perf-{i}-{}", std::process::id()))
            .config_dir(config.path())
            .size(200, 50)
            .spawn();
        term.wait_for(
            "the HEAD commit subject in the graph",
            Duration::from_secs(120),
            |s| s.contains(&needle),
        )
        .unwrap_or_else(|e| panic!("{e}"));
        times.push(started.elapsed());
        term.stop().ok();
    }
    times.sort();
    eprintln!(
        "OPEN-TO-FIRST-GRAPH-PAINT {repo}: median {} ms, min {} ms, max {} ms over {runs} (needle {needle:?})",
        times[times.len() / 2].as_millis(),
        times[0].as_millis(),
        times[times.len() - 1].as_millis()
    );
}
