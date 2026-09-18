//! Measuring stick: how long a commit's changed files
//! take to reach the render loop after the Log selection lands on it.
//!
//! `#[ignore]`d -- it reads a real repository and prints timings rather
//! than asserting on them. Run it in release, against the GitPoulpi
//! checkout itself by default:
//!
//! ```text
//! cargo test -p gitpoulpi-tui --lib --release commit_files_bench -- --ignored --nocapture
//! ```
//!
//! `GITPOULPI_BENCH_REPO=<path>` points it at another repository. The
//! numbers for this repository, before and after the fix, are recorded
//! in `docs/developer/where-we-are.md`.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use gitpoulpi_config::DefaultBackend;
use gitpoulpi_core::domain::{DiffTarget, LogOptions, Oid, RepoRef};
use gitpoulpi_core::GitBackend;

use crate::daemon_link;

fn bench_repo() -> RepoRef {
    let root = std::env::var_os("GITPOULPI_BENCH_REPO")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."));
    RepoRef::from_root(&root)
}

fn first_parent_chain(repo: &RepoRef, n: u32) -> Vec<Oid> {
    let backend = gitpoulpi_daemon::backend_for(DefaultBackend::Git2);
    let graph = backend
        .log(
            repo,
            &LogOptions {
                limit: Some(n),
                ..LogOptions::default()
            },
        )
        .expect("log of the bench repository");
    graph.nodes.iter().map(|n| n.id).collect()
}

fn summary(label: &str, mut samples: Vec<Duration>) {
    if samples.is_empty() {
        return;
    }
    samples.sort();
    let pct = |p: f64| samples[((samples.len() - 1) as f64 * p).round() as usize];
    let total: Duration = samples.iter().sum();
    println!(
        "{label:<46} n={:>3}  median {:>7.1?}  p90 {:>7.1?}  max {:>7.1?}  mean {:>7.1?}",
        samples.len(),
        pct(0.5),
        pct(0.9),
        samples[samples.len() - 1],
        total / samples.len() as u32,
    );
}

/// Waits for `commit`'s complete answer, dropping any other. Returns
/// when its file list was first shown (a `hunks_pending` answer, or the
/// complete one) and when the complete one arrived.
fn wait_for(link: &daemon_link::DaemonLink, commit: Oid) -> (Duration, Duration) {
    let started = Instant::now();
    let mut listed = None;
    loop {
        match link.commit_files_rx.try_recv() {
            Ok(outcome) if outcome.commit == commit => {
                // The list is the whole answer; a hunkless
                // outcome is final (the Diff view fetches hunks per file).
                let at = started.elapsed();
                let listed = *listed.get_or_insert(at);
                return (listed, at);
            }
            Ok(_) => {}
            Err(std::sync::mpsc::TryRecvError::Empty) => {
                assert!(started.elapsed() < Duration::from_secs(60), "no answer");
                std::thread::sleep(Duration::from_micros(200));
            }
            Err(std::sync::mpsc::TryRecvError::Disconnected) => panic!("link gone"),
        }
    }
}

#[test]
#[ignore = "benchmark: reads a real repository and prints timings"]
fn commit_files_bench() {
    let repo = bench_repo();
    println!("repository: {}", repo.root.display());
    let all = first_parent_chain(&repo, 200);
    let (commits, fresh) = all.split_at(all.len().min(60));
    println!("commits sampled: {}", commits.len());

    // 1. The backend call alone, no daemon, no IPC.
    let backends: Vec<(&str, Arc<dyn GitBackend>)> = vec![
        ("git2", gitpoulpi_daemon::backend_for(DefaultBackend::Git2)),
        (
            "system git",
            gitpoulpi_daemon::backend_for(DefaultBackend::System),
        ),
    ];
    for (name, backend) in &backends {
        let mut samples = Vec::new();
        let mut files = 0usize;
        for &c in commits {
            let t = Instant::now();
            let diff = backend.diff(&repo, DiffTarget::CommitVsParent(c)).unwrap();
            let took = t.elapsed();
            if std::env::var_os("GITPOULPI_BENCH_VERBOSE").is_some() {
                let lines: usize = diff
                    .iter()
                    .flat_map(|f| &f.hunks)
                    .map(|h| h.lines.len())
                    .sum();
                println!(
                    "    {name} {} {:>3} files {:>6} lines {took:>8.1?}",
                    &c.to_hex()[..7],
                    diff.len(),
                    lines
                );
            }
            files += diff.len();
            samples.push(took);
        }
        summary(&format!("backend diff(CommitVsParent) [{name}]"), samples);
        println!("    ({files} files over the sample)");
    }
    // The refresh a commit-files request can queue behind.
    for (name, backend) in &backends {
        let mut per_op: Vec<(&str, Vec<Duration>)> = Vec::new();
        for _ in 0..5 {
            let mut ops: Vec<(&str, Duration)> = Vec::new();
            let t = Instant::now();
            backend.status(&repo).unwrap();
            ops.push(("status", t.elapsed()));
            let t = Instant::now();
            backend
                .log(
                    &repo,
                    &LogOptions {
                        limit: Some(daemon_link::log_page_limit()),
                        all_refs: true,
                        ..LogOptions::default()
                    },
                )
                .unwrap();
            ops.push(("log", t.elapsed()));
            let t = Instant::now();
            backend.refs(&repo).unwrap();
            ops.push(("refs", t.elapsed()));
            let t = Instant::now();
            backend.stashes(&repo).unwrap();
            ops.push(("stashes", t.elapsed()));
            let t = Instant::now();
            backend.worktrees(&repo).unwrap();
            ops.push(("worktrees", t.elapsed()));
            let t = Instant::now();
            backend.conflicts(&repo).unwrap();
            ops.push(("conflicts", t.elapsed()));
            let t = Instant::now();
            backend.remotes(&repo).unwrap();
            ops.push(("remotes", t.elapsed()));
            for (op, d) in ops {
                match per_op.iter_mut().find(|(o, _)| *o == op) {
                    Some((_, v)) => v.push(d),
                    None => per_op.push((op, vec![d])),
                }
            }
        }
        for (op, samples) in per_op {
            summary(&format!("backend {op} [{name}]"), samples);
        }
    }

    // 2. End to end over the real link and an embedded daemon (the
    // backend `config.toml` selects, git2 by default), with the 2-second
    // refresh ticking underneath, exactly as the render loop sees it.
    let dir = tempfile::tempdir().unwrap();
    let link = daemon_link::start_at(
        "commit-files-bench".to_string(),
        repo.clone(),
        Some(dir.path().join("port")),
    );
    let started = Instant::now();
    while link.outcome_rx.try_recv().is_err() {
        assert!(started.elapsed() < Duration::from_secs(60), "no refresh");
        std::thread::sleep(Duration::from_millis(5));
    }

    // j, one row at a time at a human's pace, every commit once -- the
    // render loop asks for the selection and prefetches the rows either
    // side of it, and so does this.
    let (mut listed, mut full) = (Vec::new(), Vec::new());
    for (i, &c) in commits.iter().enumerate() {
        link.request_commit_files(c, false);
        let neighbors = [i + 1, i.wrapping_sub(1)]
            .into_iter()
            .filter_map(|n| commits.get(n).copied())
            .collect();
        link.prefetch_commit_files(neighbors);
        let (l, f) = wait_for(&link, c);
        listed.push(l);
        full.push(f);
        std::thread::sleep(Duration::from_millis(90 + (i as u64 * 37) % 200));
    }
    summary("e2e: j walk, first visit: list shown", listed);
    summary("e2e: j walk, first visit: hunks in", full);

    // Coming back to a commit already seen (the second j/k pass).
    let (mut listed, mut full) = (Vec::new(), Vec::new());
    for &c in commits.iter().take(20) {
        link.request_commit_files(c, false);
        let (l, f) = wait_for(&link, c);
        listed.push(l);
        full.push(f);
        std::thread::sleep(Duration::from_millis(60));
    }
    summary("e2e: revisit: list shown", listed);
    summary("e2e: revisit: hunks in", full);

    // Clicks all over the Log: commits never asked for, in an order no
    // prefetch can guess.
    let (clicks, burst_pool) = fresh.split_at(fresh.len().min(40));
    let (mut listed, mut full) = (Vec::new(), Vec::new());
    for i in 0..clicks.len() {
        let c = clicks[(i * 17) % clicks.len()];
        link.request_commit_files(c, false);
        let (l, f) = wait_for(&link, c);
        listed.push(l);
        full.push(f);
        std::thread::sleep(Duration::from_millis(150 + (i as u64 * 53) % 300));
    }
    summary("e2e: random click, first visit: list shown", listed);
    summary("e2e: random click, first visit: hunks in", full);

    // Holding j: one selection every 33ms (a key-repeat frame), over
    // commits not asked for before; the number that matters is how long
    // the *last* one takes once the key is let go.
    let (mut listed, mut full) = (Vec::new(), Vec::new());
    for burst in burst_pool.chunks(15).take(3) {
        for &c in burst {
            link.request_commit_files(c, false);
            std::thread::sleep(Duration::from_millis(33));
        }
        let last = *burst.last().unwrap();
        let (l, f) = wait_for(&link, last);
        listed.push(l + Duration::from_millis(33));
        full.push(f + Duration::from_millis(33));
        std::thread::sleep(Duration::from_millis(300));
    }
    summary("e2e: held j, 15 rows, after release: list", listed);
    summary("e2e: held j, 15 rows, after release: hunks", full);
}
