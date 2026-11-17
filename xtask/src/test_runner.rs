//! The parallel half of `xtask test`.
//!
//! `cargo test` runs the test executables one after another. With ~180 fast
//! and ~230 PTY binaries that is the wall clock, and on Windows it hurts most:
//! a process spawn costs ~100 ms there (~1 ms on Linux) and the tests spawn
//! `git` a lot, so most binaries are latency bound, not CPU bound. This
//! module builds once (`cargo test --no-run --message-format=json`), then
//! runs the executables itself with bounded parallelism.
//!
//! Each executable runs as cargo would run it: working directory = its
//! package's directory, output captured. Failure output is printed in full at
//! the end; the progress lines and the summary say pass/fail and duration per
//! binary. Doc tests (rustdoc, not an executable we can list) run as one
//! `cargo test --doc` job beside the pool.
//!
//! A cache of the previous run's durations (`target/xtask-test-durations.tsv`)
//! orders the queue longest-first, so the 600 s Windows `contract` binary
//! starts at once instead of last.

use crate::test_failures::{self, Failure, TargetId};
use std::collections::{HashMap, VecDeque};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// One unit of work: a test executable, or the doc-test cargo command.
#[derive(Clone, Debug)]
pub struct Job {
    pub label: String,
    pub program: PathBuf,
    pub args: Vec<String>,
    pub cwd: Option<PathBuf>,
    pub envs: Vec<(String, String)>,
    /// Which cargo target this runs (for the failures file); None in tests.
    pub target: Option<TargetId>,
}

#[derive(Debug)]
pub struct Outcome {
    pub label: String,
    pub target: Option<TargetId>,
    /// Test names libtest reported as FAILED (empty: none parseable).
    pub failed_tests: Vec<String>,
    /// Tests that failed in the pool but passed in the alone rerun.
    pub saved: Vec<String>,
    /// Top-level entries the job left in its temp directory (then removed).
    pub leaked: usize,
    pub ok: bool,
    pub duration: Duration,
    pub output: String,
}

/// Test executables of the `compiler-artifact` lines of a
/// `--message-format=json` build, as jobs (deduplicated, sorted by label).
pub fn jobs_from_cargo_json(stdout: &str) -> Vec<Job> {
    let mut jobs: Vec<Job> = Vec::new();
    for line in stdout.lines() {
        let Ok(v) = serde_json::from_str::<serde_json::Value>(line) else {
            continue;
        };
        if v["reason"] != "compiler-artifact" || v["profile"]["test"] != true {
            continue;
        }
        let Some(exe) = v["executable"].as_str() else {
            continue;
        };
        let manifest = Path::new(v["manifest_path"].as_str().unwrap_or(""));
        let dir = manifest.parent().map(Path::to_path_buf);
        let pkg = dir
            .as_ref()
            .and_then(|d| d.file_name())
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let name = v["target"]["name"].as_str().unwrap_or("?");
        let kind = v["target"]["kind"][0].as_str().unwrap_or("");
        let package = test_failures::package_name(v["package_id"].as_str().unwrap_or(""), &pkg);
        let label = match kind {
            "test" => format!("{pkg}::{name}"),
            "bin" => format!("{pkg} (bin {name})"),
            _ => format!("{pkg} ({kind})"),
        };
        if jobs.iter().any(|j| j.program == Path::new(exe)) {
            continue;
        }
        let mut envs = Vec::new();
        if let Some(d) = &dir {
            envs.push(("CARGO_MANIFEST_DIR".to_string(), d.display().to_string()));
        }
        // A PTY test is a whole TUI plus a daemon plus child processes.
        // libtest would run one binary's tests on every CPU at once, so N
        // binaries in parallel would be N x CPUs terminals and the
        // timing-based waits fail on load. Two threads per binary keeps
        // `jobs` x 2 terminals in flight (see `default_jobs`).
        if name.starts_with("pty_") && std::env::var_os("RUST_TEST_THREADS").is_none() {
            let default = if is_heavy_pty(name) { "4" } else { "2" };
            let threads = std::env::var("GITPOULPI_PTY_TEST_THREADS").unwrap_or(default.into());
            envs.push(("RUST_TEST_THREADS".to_string(), threads));
        }
        // Every other binary gets a cap too: libtest's default is one thread
        // per CPU, so `jobs` binaries at once oversubscribed the machine
        // (15 x 20 threads on a 20-CPU box) and starved the ones measuring time.
        if !name.starts_with("pty_") && std::env::var_os("RUST_TEST_THREADS").is_none() {
            envs.push(("RUST_TEST_THREADS".to_string(), default_test_threads()));
        }
        envs.push(("CARGO_PKG_NAME".to_string(), pkg));
        jobs.push(Job {
            label,
            program: PathBuf::from(exe),
            args: Vec::new(),
            cwd: dir,
            envs,
            target: Some(TargetId {
                package,
                name: name.to_string(),
                kind: kind.to_string(),
            }),
        });
    }
    jobs.sort_by(|a, b| a.label.cmp(&b.label));
    jobs
}

/// Targets named by `error: could not compile `pkg` (test "name")` lines of
/// cargo's stderr: the binaries whose build failed.
pub fn failed_builds(stderr: &str) -> Vec<TargetId> {
    let mut out: Vec<TargetId> = Vec::new();
    for line in stderr.lines() {
        let Some(rest) = line.trim().strip_prefix("error: could not compile `") else {
            continue;
        };
        let Some((package, rest)) = rest.split_once('`') else {
            continue;
        };
        let what = rest
            .trim_start()
            .strip_prefix('(')
            .and_then(|r| r.split_once(')'))
            .map(|(w, _)| w)
            .unwrap_or("");
        let quoted = what.split('"').nth(1);
        let (name, kind) = match (what.split_whitespace().next(), quoted) {
            (Some("test"), Some(n)) => (n, "test"),
            (Some("bin"), Some(n)) => (n, "bin"),
            // `lib`, `lib test`, or anything unknown: the package's lib.
            _ => (package, "lib"),
        };
        let t = TargetId {
            package: package.to_string(),
            name: name.to_string(),
            kind: kind.to_string(),
        };
        if !out.contains(&t) {
            out.push(t);
        }
    }
    out
}

/// A failed build, with the targets cargo said it could not compile.
pub struct BuildError {
    pub message: String,
    pub targets: Vec<TargetId>,
}

/// Runs `cargo <args> --no-run --message-format=json-render-diagnostics`
/// (compiler messages go to the terminal) and returns the test executables.
pub fn build(args: &[String]) -> Result<Vec<Job>, BuildError> {
    use std::io::{BufRead, Read};
    let fail = |message: String| BuildError {
        message,
        targets: Vec::new(),
    };
    let mut a = args.to_vec();
    for extra in ["--no-run", "--message-format=json-render-diagnostics"] {
        if !a.iter().any(|x| x == extra) {
            a.push(extra.to_string());
        }
    }
    let mut child = Command::new("cargo")
        .args(&a)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| fail(format!("cannot run cargo: {e}")))?;
    // Show stderr live, keep a copy to find which targets failed.
    let err = child.stderr.take().expect("piped");
    let tee = std::thread::spawn(move || {
        let mut all = String::new();
        for line in std::io::BufReader::new(err).lines().map_while(Result::ok) {
            eprintln!("{line}");
            all.push_str(&line);
            all.push('\n');
        }
        all
    });
    let mut stdout = String::new();
    if let Some(mut o) = child.stdout.take() {
        let mut buf = Vec::new();
        let _ = o.read_to_end(&mut buf);
        stdout = String::from_utf8_lossy(&buf).into_owned();
    }
    let status = child
        .wait()
        .map_err(|e| fail(format!("cannot run cargo: {e}")))?;
    let stderr = tee.join().unwrap_or_default();
    if !status.success() {
        return Err(BuildError {
            message: "the test build failed".to_string(),
            targets: failed_builds(&stderr),
        });
    }
    Ok(jobs_from_cargo_json(&stdout))
}

/// A fresh temp directory for one job, under the system temp dir, with a
/// short name (`gpx-<pid>-<n>`): the job gets it as `TMPDIR`/`TMP`/`TEMP`, so
/// whatever a test leaks there (a forgotten `TempDir`, a kept fixture, an
/// image copy) is removed with the directory when the job ends.
fn job_temp_dir() -> Option<PathBuf> {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let n = NEXT.fetch_add(1, Ordering::SeqCst);
    let dir = std::env::temp_dir().join(format!("gpx-{}-{n}", std::process::id()));
    std::fs::create_dir_all(&dir).ok()?;
    Some(dir)
}

/// Entries a job left in its temp directory (top level only).
fn leaked_entries(dir: &Path) -> usize {
    std::fs::read_dir(dir).map(|r| r.count()).unwrap_or(0)
}

fn run_job(job: &Job) -> Outcome {
    let started = Instant::now();
    let mut cmd = Command::new(&job.program);
    cmd.args(&job.args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if let Some(d) = &job.cwd {
        cmd.current_dir(d);
    }
    let temp = job_temp_dir();
    if let Some(t) = &temp {
        for k in ["TMPDIR", "TMP", "TEMP"] {
            cmd.env(k, t);
        }
    }
    for (k, v) in &job.envs {
        cmd.env(k, v);
    }
    let (ok, output) = match cmd.output() {
        Ok(o) => {
            let mut s = String::from_utf8_lossy(&o.stdout).into_owned();
            let err = String::from_utf8_lossy(&o.stderr);
            if !err.is_empty() {
                s.push_str("\n--- stderr ---\n");
                s.push_str(&err);
            }
            (o.status.success(), s)
        }
        Err(e) => (
            false,
            format!("cannot start {}: {e}", job.program.display()),
        ),
    };
    let failed_tests = if ok {
        Vec::new()
    } else {
        test_failures::failed_test_names(&output)
    };
    let leaked = temp.as_deref().map(leaked_entries).unwrap_or(0);
    if let Some(t) = &temp {
        // Best effort: on Windows a daemon a test left running can still hold
        // a file; the next run's sweep (`sweep_stale_temp_dirs`) retries.
        let _ = std::fs::remove_dir_all(t);
    }
    Outcome {
        label: job.label.clone(),
        target: job.target.clone(),
        failed_tests,
        saved: Vec::new(),
        leaked,
        ok,
        duration: started.elapsed(),
        output,
    }
}

/// Seconds the heaviest binaries took in a sequential Windows run (the slow
/// platform), used to order the first run on a machine, before
/// `target/xtask-test-durations.tsv` exists. Matched on the target name.
const STATIC_WEIGHTS: &[(&str, f64)] = &[
    ("contract", 599.0),
    ("pty_ui_intent_show", 148.0),
    ("pty_hosting", 121.0),
    ("autostash", 86.0),
    ("pty_workspaces", 66.0),
    ("pty_jobs", 65.0),
    ("pty_persist_sizes", 64.0),
    ("pty_workspace_search", 63.0),
    ("pty_jobs_client", 62.0),
    ("pty_remember_tabs", 61.0),
    ("pty_ai", 61.0),
    ("flow", 48.0),
    ("pty_undo", 47.0),
    ("rewrite_undo", 44.0),
    ("pty_lfs", 44.0),
];

fn static_weight(label: &str) -> Option<f64> {
    let name = label.rsplit("::").next()?;
    STATIC_WEIGHTS
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, w)| *w)
}

/// PTY binaries that are long poles even with many CPUs: they get more test
/// threads than the default two so they do not stretch the critical path.
fn is_heavy_pty(name: &str) -> bool {
    name.starts_with("pty_") && static_weight(name).is_some_and(|w| w >= 60.0)
}

fn durations_path() -> PathBuf {
    let target = std::env::var_os("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("target"));
    target.join("xtask-test-durations.tsv")
}

fn read_durations(path: &Path) -> HashMap<String, f64> {
    let mut m = HashMap::new();
    for line in std::fs::read_to_string(path).unwrap_or_default().lines() {
        if let Some((label, secs)) = line.rsplit_once('\t') {
            if let Ok(s) = secs.parse() {
                m.insert(label.to_string(), s);
            }
        }
    }
    m
}

fn write_durations(path: &Path, old: &HashMap<String, f64>, outcomes: &[Outcome]) {
    let mut m = old.clone();
    for o in outcomes {
        m.insert(o.label.clone(), o.duration.as_secs_f64());
    }
    let mut rows: Vec<_> = m.into_iter().collect();
    rows.sort_by(|a, b| a.0.cmp(&b.0));
    let text: String = rows.iter().map(|(l, s)| format!("{l}\t{s:.2}\n")).collect();
    let _ = std::fs::write(path, text);
}

/// The failures file content of a run: one row per failed binary.
pub fn failures_of(outcomes: &[Outcome]) -> Vec<Failure> {
    outcomes
        .iter()
        .filter(|o| !o.ok)
        .filter_map(|o| {
            Some(Failure {
                target: o.target.clone()?,
                tests: o.failed_tests.clone(),
            })
        })
        .collect()
}

/// Writes `target/xtask-test-failures.tsv` (empty when green).
pub fn write_failures(failures: &[Failure]) {
    let _ = test_failures::write(&test_failures::path(), failures);
}

fn is_pty_label(label: &str) -> bool {
    label.contains("::pty_")
}

/// Binaries that run alone, one at a time, after the pool has drained:
/// they measure a quiet window (or a budget) and fundamentally need an
/// unloaded machine. Matched on the target name (the label's last segment).
const SERIAL: &[&str] = &["pty_idle"];

fn is_serial_label(label: &str) -> bool {
    label
        .rsplit("::")
        .next()
        .is_some_and(|name| SERIAL.contains(&name))
}

/// `RUST_TEST_THREADS` for a non-PTY binary: `GITPOULPI_TEST_THREADS`, else
/// the CPUs divided among the parallel binaries, at least four (two
/// was measurably slower: the long binaries are the critical path).
fn default_test_threads() -> String {
    std::env::var("GITPOULPI_TEST_THREADS")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .filter(|n| *n > 0)
        .unwrap_or_else(|| {
            let cpus = std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(4);
            (cpus / default_jobs()).max(4)
        })
        .to_string()
}

/// Runs the jobs with at most `jobs` at a time (at most `pty_limit` of them
/// PTY binaries, started only once no other binary is waiting) and prints
/// progress. With `fail_fast`, no new job starts
/// after the first failure.
/// Removes the per-job temp dirs (`gpx-*`, see `job_temp_dir`) an earlier
/// run could not delete (killed run, a Windows file still held), once they
/// are older than any run.
fn sweep_stale_temp_dirs() {
    let Ok(entries) = std::fs::read_dir(std::env::temp_dir()) else {
        return;
    };
    let max_age = Duration::from_secs(6 * 3600);
    for e in entries.flatten() {
        if !e.file_name().to_string_lossy().starts_with("gpx-") {
            continue;
        }
        let old = e
            .metadata()
            .and_then(|m| m.modified())
            .ok()
            .and_then(|t| t.elapsed().ok())
            .is_some_and(|age| age > max_age);
        if old {
            let _ = std::fs::remove_dir_all(e.path());
        }
    }
}

pub fn run_all(
    mut queue: Vec<Job>,
    jobs: usize,
    pty_limit: usize,
    fail_fast: bool,
) -> Vec<Outcome> {
    sweep_stale_temp_dirs();
    let path = durations_path();
    let known = read_durations(&path);
    // Longest first; unknown binaries count as long so they are not left last.
    let weight = |j: &Job| {
        known
            .get(&j.label)
            .copied()
            .or_else(|| static_weight(&j.label))
            .unwrap_or(10.0)
    };
    queue.sort_by(|a, b| {
        weight(b)
            .partial_cmp(&weight(a))
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.label.cmp(&b.label))
    });
    let total = queue.len();
    let (serial, queue): (Vec<Job>, Vec<Job>) =
        queue.into_iter().partition(|j| is_serial_label(&j.label));
    let queue = Arc::new(Mutex::new(queue.into_iter().collect::<VecDeque<_>>()));
    let done = Arc::new(AtomicUsize::new(0));
    let stop = Arc::new(AtomicBool::new(false));
    let results: Arc<Mutex<Vec<Outcome>>> = Arc::new(Mutex::new(Vec::new()));
    let pty_running = Arc::new(AtomicUsize::new(0));
    let workers = jobs.clamp(1, total.max(1));
    let handles: Vec<_> = (0..workers)
        .map(|_| {
            let (queue, done, stop, results, pty_running) = (
                queue.clone(),
                done.clone(),
                stop.clone(),
                results.clone(),
                pty_running.clone(),
            );
            std::thread::spawn(move || loop {
                if fail_fast && stop.load(Ordering::SeqCst) {
                    return;
                }
                // The first queued job allowed to start now: PTY binaries
                // are capped at `pty_limit` at once, anything else is free.
                let job = {
                    let mut q = queue.lock().unwrap();
                    if q.is_empty() {
                        return;
                    }
                    let running = pty_running.load(Ordering::SeqCst);
                    // PTY binaries wait until every other binary has started:
                    // mixing them with a full pool of busy fast binaries made
                    // timing-based PTY waits flaky; only the last long fast
                    // binaries (the contract suite) overlap them.
                    let fast_waiting = q.iter().any(|j| !is_pty_label(&j.label));
                    let pick = q.iter().position(|j| {
                        if is_pty_label(&j.label) {
                            !fast_waiting && running < pty_limit
                        } else {
                            true
                        }
                    });
                    match pick {
                        Some(i) => {
                            let j = q.remove(i).unwrap();
                            if is_pty_label(&j.label) {
                                pty_running.fetch_add(1, Ordering::SeqCst);
                            }
                            Some(j)
                        }
                        None => None,
                    }
                };
                let Some(job) = job else {
                    std::thread::sleep(Duration::from_millis(100));
                    continue;
                };
                let o = run_job(&job);
                if is_pty_label(&job.label) {
                    pty_running.fetch_sub(1, Ordering::SeqCst);
                }
                if !o.ok {
                    stop.store(true, Ordering::SeqCst);
                }
                let n = done.fetch_add(1, Ordering::SeqCst) + 1;
                println!(
                    "[{n:>3}/{total}] {} {:>7.1}s  {}",
                    if o.ok { "ok  " } else { "FAIL" },
                    o.duration.as_secs_f64(),
                    o.label
                );
                results.lock().unwrap().push(o);
            })
        })
        .collect();
    for h in handles {
        let _ = h.join();
    }
    // The serial group: alone on the machine, after everything else.
    for job in serial {
        if fail_fast && stop.load(Ordering::SeqCst) {
            break;
        }
        let o = run_job(&job);
        if !o.ok {
            stop.store(true, Ordering::SeqCst);
        }
        let n = done.fetch_add(1, Ordering::SeqCst) + 1;
        println!(
            "[{n:>3}/{total}] {} {:>7.1}s  {}",
            if o.ok { "ok  " } else { "FAIL" },
            o.duration.as_secs_f64(),
            o.label
        );
        results.lock().unwrap().push(o);
    }
    let outcomes = Arc::try_unwrap(results)
        .map(|m| m.into_inner().unwrap())
        .unwrap_or_default();
    write_durations(&path, &known, &outcomes);
    outcomes
}

/// The alone phase, after the pool: the listed jobs (`xtask/run-alone.txt`)
/// one at a time, then every test that failed in the pool rerun alone with an
/// exact filter. A pool failure whose tests all pass alone is turned green and
/// remembered in `saved` (the report says to list them); one that fails alone
/// too stays a real failure. `pool` is the jobs the pool ran, to rebuild the
/// reruns. Appends the listed jobs' outcomes to `outcomes`.
pub fn alone_phase(outcomes: &mut Vec<Outcome>, pool: &[Job], listed: Vec<Job>, fail_fast: bool) {
    // The safety net first: a load flake must not stop the listed tests.
    for o in outcomes.iter_mut().filter(|o| !o.ok) {
        let Some(t) = &o.target else { continue };
        if t.is_doc() || o.failed_tests.is_empty() {
            continue;
        }
        let Some(base) = pool.iter().find(|j| j.label == o.label) else {
            continue;
        };
        let names = o.failed_tests.clone();
        let rerun = run_job(&crate::run_alone::rerun_job(base, &names));
        let still: Vec<String> = if rerun.ok {
            Vec::new()
        } else if rerun.failed_tests.is_empty() {
            names.clone()
        } else {
            rerun.failed_tests.clone()
        };
        o.saved = names
            .iter()
            .filter(|n| !still.contains(n))
            .cloned()
            .collect();
        println!(
            "[alone] {} {:>7.1}s  {} ({} of {} test(s) rerun alone passed)",
            if still.is_empty() { "ok  " } else { "FAIL" },
            rerun.duration.as_secs_f64(),
            rerun.label,
            o.saved.len(),
            names.len()
        );
        if still.is_empty() {
            o.ok = true;
            o.failed_tests.clear();
        } else {
            o.failed_tests = still;
            o.output
                .push_str(&format!("\n--- rerun alone ---\n{}", rerun.output));
        }
    }
    let n = listed.len();
    for (i, job) in listed.iter().enumerate() {
        if fail_fast && outcomes.iter().any(|o| !o.ok) {
            break;
        }
        let o = run_job(job);
        println!(
            "[alone {}/{n}] {} {:>7.1}s  {}",
            i + 1,
            if o.ok { "ok  " } else { "FAIL" },
            o.duration.as_secs_f64(),
            o.label
        );
        outcomes.push(o);
    }
}

/// The deterministic report: every binary sorted by label, the ten slowest,
/// then the full output of each failure. Returns the number of failures.
pub fn report(outcomes: &mut [Outcome], wall: Duration) -> usize {
    outcomes.sort_by(|a, b| a.label.cmp(&b.label));
    let failed: Vec<&Outcome> = outcomes.iter().filter(|o| !o.ok).collect();
    println!("\n===== summary =====");
    for o in outcomes.iter() {
        println!(
            "{} {:>7.1}s  {}",
            if o.ok { "ok  " } else { "FAIL" },
            o.duration.as_secs_f64(),
            o.label
        );
    }
    let mut slow: Vec<&Outcome> = outcomes.iter().collect();
    slow.sort_by(|a, b| {
        b.duration
            .cmp(&a.duration)
            .then_with(|| a.label.cmp(&b.label))
    });
    println!("\nslowest ten:");
    for o in slow.iter().take(10) {
        println!("  {:>7.1}s  {}", o.duration.as_secs_f64(), o.label);
    }
    let busy: f64 = outcomes.iter().map(|o| o.duration.as_secs_f64()).sum();
    let mut leaky: Vec<&Outcome> = outcomes.iter().filter(|o| o.leaked > 0).collect();
    if !leaky.is_empty() {
        leaky.sort_by(|a, b| b.leaked.cmp(&a.leaked).then_with(|| a.label.cmp(&b.label)));
        let total: usize = leaky.iter().map(|o| o.leaked).sum();
        println!(
            "\ntemp leaks (removed with each binary's TMPDIR): {total} entries from {} binaries",
            leaky.len()
        );
        for o in leaky.iter().take(15) {
            println!("  {:>6}  {}", o.leaked, o.label);
        }
    }
    let saved: Vec<(&Outcome, &String)> = outcomes
        .iter()
        .flat_map(|o| o.saved.iter().map(move |t| (o, t)))
        .collect();
    if !saved.is_empty() {
        println!("\npassed only when run alone (add to xtask/run-alone.txt):");
        for (o, t) in &saved {
            if let Some(id) = &o.target {
                println!("  {}::{}::{t}", id.package, id.name);
            }
        }
    }
    for o in &failed {
        println!("\n===== FAILED: {} =====\n{}", o.label, o.output.trim_end());
    }
    println!(
        "\n{} binaries, {} failed; {} saved by the alone rerun; wall {:.0}s, sum of binary times {:.0}s",
        outcomes.len(),
        failed.len(),
        saved.len(),
        wall.as_secs_f64(),
        busy
    );
    if !failed.is_empty() {
        println!("failed:");
        for o in &failed {
            println!("  {}", o.label);
        }
    }
    failed.len()
}

/// How many PTY binaries may run at once: `GITPOULPI_PTY_JOBS`, else half the
/// CPU count (each runs two tests at a time, so about one terminal per CPU).
pub fn default_pty_jobs() -> usize {
    std::env::var("GITPOULPI_PTY_JOBS")
        .ok()
        .and_then(|v| v.parse().ok())
        .filter(|n| *n > 0)
        .unwrap_or_else(|| {
            let cpus = std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(4);
            (cpus / 2).max(2)
        })
}

/// The default parallelism: `GITPOULPI_TEST_JOBS`, else the CPU count.
pub fn default_jobs() -> usize {
    std::env::var("GITPOULPI_TEST_JOBS")
        .ok()
        .and_then(|v| v.parse().ok())
        .filter(|n| *n > 0)
        .unwrap_or_else(|| {
            // Three quarters of the CPUs: on the 8-CPU Windows VM, 6 jobs
            // finished the full tier in 875 s against 1013 s with 8 (less
            // contention, and the timing-sensitive PTY tests stop flaking).
            let cpus = std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(4);
            (cpus * 3 / 4).max(1)
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn picks_test_executables_only() {
        let lines = [
            r#"{"reason":"compiler-artifact","profile":{"test":false},"executable":"/t/dep","manifest_path":"/w/crates/dep/Cargo.toml","target":{"name":"dep","kind":["lib"]}}"#,
            r#"{"reason":"compiler-artifact","profile":{"test":true},"executable":"/t/a-1","manifest_path":"/w/crates/foo/Cargo.toml","target":{"name":"pty_a","kind":["test"]}}"#,
            r#"{"reason":"compiler-artifact","profile":{"test":true},"executable":"/t/foo-2","manifest_path":"/w/crates/foo/Cargo.toml","target":{"name":"foo","kind":["lib"]}}"#,
            r#"{"reason":"compiler-artifact","profile":{"test":true},"executable":null,"manifest_path":"/w/crates/foo/Cargo.toml","target":{"name":"x","kind":["lib"]}}"#,
            "not json",
        ]
        .join("\n");
        let jobs = jobs_from_cargo_json(&lines);
        let labels: Vec<_> = jobs.iter().map(|j| j.label.as_str()).collect();
        assert_eq!(labels, ["foo (lib)", "foo::pty_a"]);
        assert_eq!(jobs[0].cwd.as_deref(), Some(Path::new("/w/crates/foo")));
    }

    #[test]
    fn job_targets_carry_the_package_name() {
        let line = r#"{"reason":"compiler-artifact","package_id":"path+file:///w/crates/foo#foo-pkg@0.1.0","profile":{"test":true},"executable":"/t/a-1","manifest_path":"/w/crates/foo/Cargo.toml","target":{"name":"pty_a","kind":["test"]}}"#;
        let jobs = jobs_from_cargo_json(line);
        let t = jobs[0].target.clone().unwrap();
        assert_eq!(
            (t.package.as_str(), t.name.as_str(), t.kind.as_str()),
            ("foo-pkg", "pty_a", "test")
        );
    }

    #[test]
    fn failed_builds_come_from_cargo_stderr() {
        let err = "error: could not compile `gitpoulpi-core` (lib test) due to 1 previous error\n\
error: could not compile `gitpoulpi-tui` (test \"pty_x\") due to 2 previous errors\n\
error: could not compile `xtask` (bin \"xtask\" test) due to 1 previous error\n\
warning: something else\n";
        let got = failed_builds(err);
        let s: Vec<_> = got
            .iter()
            .map(|t| (t.package.as_str(), t.name.as_str(), t.kind.as_str()))
            .collect();
        assert_eq!(
            s,
            [
                ("gitpoulpi-core", "gitpoulpi-core", "lib"),
                ("gitpoulpi-tui", "pty_x", "test"),
                ("xtask", "xtask", "bin"),
            ]
        );
    }

    #[test]
    fn outcomes_become_failure_rows() {
        let o = |n: &str, ok: bool, names: &[&str]| Outcome {
            label: n.into(),
            target: Some(TargetId {
                package: "p".into(),
                name: n.into(),
                kind: "test".into(),
            }),
            failed_tests: names.iter().map(|s| s.to_string()).collect(),
            saved: Vec::new(),
            leaked: 0,
            ok,
            duration: Duration::ZERO,
            output: String::new(),
        };
        let rows = failures_of(&[
            o("a", true, &[]),
            o("b", false, &["x::y"]),
            o("c", false, &[]),
        ]);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].tests, ["x::y"]);
        assert!(rows[1].tests.is_empty());
        assert!(failures_of(&[o("a", true, &[])]).is_empty());
    }

    #[test]
    fn a_failure_stops_new_work_only_when_fail_fast() {
        let bad = |l: &str| Job {
            label: l.into(),
            program: PathBuf::from("definitely-not-a-program-xyz"),
            args: vec![],
            cwd: None,
            envs: vec![],
            target: None,
        };
        let all = run_all(vec![bad("a"), bad("b"), bad("c")], 1, 1, false);
        assert_eq!(all.len(), 3);
        let first = run_all(vec![bad("a"), bad("b"), bad("c")], 1, 1, true);
        assert_eq!(first.len(), 1);
    }

    #[test]
    fn the_serial_group_is_matched_on_the_target_name() {
        assert!(is_serial_label("gitpoulpi-tui::pty_idle"));
        assert!(!is_serial_label("gitpoulpi-tui::pty_idle_extra"));
        assert!(!is_serial_label("gitpoulpi-tui (lib)"));
    }

    #[test]
    fn serial_jobs_start_only_after_the_pool_drained() {
        let job = |label: &str| Job {
            label: label.to_string(),
            program: PathBuf::from(if cfg!(windows) { "cmd" } else { "true" }),
            args: if cfg!(windows) {
                vec!["/c".into(), "exit".into()]
            } else {
                Vec::new()
            },
            cwd: None,
            envs: Vec::new(),
            target: None,
        };
        let queue = vec![job("p::pty_idle"), job("p::a"), job("p::b")];
        let outcomes = run_all(queue, 3, 1, false);
        assert_eq!(outcomes.len(), 3);
        // Outcomes are pushed as jobs finish: the serial one is last.
        assert_eq!(outcomes.last().unwrap().label, "p::pty_idle");
    }
}
