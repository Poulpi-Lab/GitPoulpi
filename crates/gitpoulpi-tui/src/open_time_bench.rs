//! Open-time measuring stick: how long a repository (or a worktree) takes
//! to get its first `RefreshOutcome` after a window opens, and what each
//! query in that refresh costs on its own.
//!
//! `#[ignore]`d -- it reads a real repository and prints timings. Run it in
//! release, against the GitPoulpi checkout by default:
//!
//! ```text
//! cargo test -p gitpoulpi-tui --lib --release open_time_bench -- --ignored --nocapture
//! ```
//!
//! `GITPOULPI_BENCH_REPO=<path>` points it at another repository. The
//! numbers before and after the open-path work are in
//! `docs/developer/open-time.md`. A loose regression guard on a small
//! fixture (`first_outcome_on_a_small_fixture_is_prompt`) runs in the
//! normal suite.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use gitpoulpi_config::DefaultBackend;
use gitpoulpi_core::domain::{LogOptions, RepoRef};
use gitpoulpi_core::GitBackend;

use crate::daemon_link;

fn bench_repo() -> RepoRef {
    let root = std::env::var_os("GITPOULPI_BENCH_REPO")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."));
    RepoRef::from_root(&root)
}

fn timed<T>(label: &str, f: impl FnOnce() -> T) -> T {
    let t = Instant::now();
    let out = f();
    println!("    {label:<28} {:>8.1?}", t.elapsed());
    out
}

fn per_query(backend: &Arc<dyn GitBackend>, repo: &RepoRef, pass: &str) {
    println!("  per-query ({pass})");
    timed("status", || backend.status(repo).ok());
    for limit in [5000, 200] {
        timed(&format!("log (limit={limit}, all)"), || {
            backend
                .log(
                    repo,
                    &LogOptions {
                        limit: Some(limit),
                        all_refs: true,
                        ..LogOptions::default()
                    },
                )
                .ok()
        });
    }
    timed("refs", || backend.refs(repo).ok());
    timed("stashes", || backend.stashes(repo).ok());
    timed("worktrees", || backend.worktrees(repo).ok());
    timed("conflicts", || backend.conflict_entries(repo).ok());
    timed("remotes", || backend.remotes(repo).ok());
    timed("rebase_state", || backend.rebase_state(repo).ok());
}

/// The eight queries of one refresh, as a `Vec` of closures.
type Query<'a> = Box<dyn Fn() + Send + Sync + 'a>;

fn refresh_queries<'a>(
    git2: &'a Arc<dyn GitBackend>,
    system: &'a Arc<dyn GitBackend>,
    repo: &'a RepoRef,
    native_conflicts: bool,
) -> Vec<Query<'a>> {
    // `native_conflicts == false` reproduces the serial cost: conflicts and
    // rebase_state answered by spawning system `git`.
    let tail: &'a Arc<dyn GitBackend> = if native_conflicts { git2 } else { system };
    let log = LogOptions {
        limit: Some(5000),
        all_refs: true,
        ..LogOptions::default()
    };
    vec![
        Box::new(move || drop(git2.status(repo))),
        Box::new(move || drop(git2.log(repo, &log))),
        Box::new(move || drop(git2.refs(repo))),
        Box::new(move || drop(git2.stashes(repo))),
        Box::new(move || drop(git2.worktrees(repo))),
        Box::new(move || drop(tail.conflict_entries(repo))),
        Box::new(move || drop(git2.remotes(repo))),
        Box::new(move || drop(tail.rebase_state(repo))),
    ]
}

fn median(mut v: Vec<Duration>) -> Duration {
    v.sort();
    v[v.len() / 2]
}

/// The refresh's wall time, serial (two `git` spawns) against parallel
/// (native), interleaved in one process so machine load hits both alike.
fn refresh_shape_ab(repo: &RepoRef) {
    let git2 = gitpoulpi_daemon::backend_for(DefaultBackend::Git2);
    let system = gitpoulpi_daemon::backend_for(DefaultBackend::System);
    let (mut old, mut new) = (Vec::new(), Vec::new());
    for _ in 0..5 {
        let qs = refresh_queries(&git2, &system, repo, false);
        let t = Instant::now();
        qs.iter().for_each(|q| q());
        old.push(t.elapsed());

        let qs = refresh_queries(&git2, &system, repo, true);
        let t = Instant::now();
        std::thread::scope(|s| {
            for q in &qs {
                s.spawn(q);
            }
        });
        new.push(t.elapsed());
    }
    println!(
        "  refresh wall time, median of 5: old serial+spawns {:>8.1?}   new parallel+native {:>8.1?}",
        median(old),
        median(new)
    );
}

fn first_outcome(session: &str, repo: &RepoRef, port_file: PathBuf) -> Duration {
    let t = Instant::now();
    let link = daemon_link::start_at(session.to_string(), repo.clone(), Some(port_file));
    let outcome = link
        .outcome_rx
        .recv_timeout(Duration::from_secs(120))
        .expect("a first outcome");
    let took = t.elapsed();
    assert!(outcome.log_result.is_ok() || outcome.status_result.is_ok());
    took
}

#[test]
#[ignore]
fn open_time_bench() {
    let repo = bench_repo();
    let backend = gitpoulpi_daemon::backend_for(DefaultBackend::Git2);
    println!("repo: {}", repo.root.display());
    per_query(&backend, &repo, "cold");
    per_query(&backend, &repo, "warm");
    refresh_shape_ab(&repo);

    let dir = tempfile::tempdir().unwrap();
    let port_file = dir.path().join("port");
    let session = "open-bench";
    println!("  link -> first RefreshOutcome");
    println!(
        "    first open (spawns daemon)   {:>8.1?}",
        first_outcome(session, &repo, port_file.clone())
    );
    println!(
        "    second open (daemon warm)    {:>8.1?}",
        first_outcome(session, &repo, port_file.clone())
    );

    // A worktree of the same repository, as a second tab.
    let wt = dir.path().join("bench-wt");
    let made = std::process::Command::new("git")
        .current_dir(&repo.root)
        .args(["worktree", "add", "--detach"])
        .arg(&wt)
        .arg("HEAD")
        .output();
    if made.map(|o| o.status.success()).unwrap_or(false) {
        let wt_ref = RepoRef::from_root(&wt);
        println!(
            "    worktree tab (same daemon)   {:>8.1?}",
            first_outcome(session, &wt_ref, port_file.clone())
        );
        println!(
            "    worktree tab again           {:>8.1?}",
            first_outcome(session, &wt_ref, port_file)
        );
        let _ = std::process::Command::new("git")
            .current_dir(&repo.root)
            .args(["worktree", "remove", "--force"])
            .arg(&wt)
            .output();
    }
}

/// Loose guard, small fixture: the first refresh of a tiny repository must
/// not take anywhere near the seconds a serial walk plus eight round trips
/// take on a big one. 10 s is far above any healthy machine, so this only
/// fires on a real regression (a blocking call on the path).
#[test]
fn first_outcome_on_a_small_fixture_is_prompt() {
    let fixture = gitpoulpi_testkit::FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build();
    let dir = tempfile::tempdir().unwrap();
    let took = first_outcome("open-guard", &fixture.repo_ref(), dir.path().join("port"));
    assert!(
        took < Duration::from_secs(10),
        "first outcome took {took:?}"
    );
}
