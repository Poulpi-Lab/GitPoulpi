//! `xtask test [--tier fast|pty|full] [--no-fail-fast] [--dry-run]`: the test
//! suite in tiers, so the slow end-to-end PTY tests can be run apart from
//! everything else.
//!
//! Cargo cannot exclude one integration-test target, only select targets. So
//! the fast tier is spelled as a selection: `--lib --bins` plus one
//! `--test <name>` per integration target of the workspace whose name does not
//! start with `pty_` (found through `cargo metadata`, so a new test file is
//! picked up with no list to edit). The pty tier is `--test <name>` for the
//! rest, which includes the `pty_hosting` directory target and the pty targets
//! of other crates. Every tier is `cargo test --workspace` with the same
//! feature flags, so the three share one build of the libraries and binaries.
//!
//! A tier is a partition: fast + pty = the workspace's `cargo test` (full is
//! exactly that command). Doc tests belong to fast.

use crate::test_failures::{self, Failure, TargetId};
use std::process::Command;

const USAGE: &str = "\
Usage: cargo run -p xtask -- test [--tier fast|pty|full] [--jobs N] [--serial] [--no-fail-fast] [--no-run] [--dry-run] [--failed [--only PATTERN]...]

  --tier fast   (default) every test except the PTY end-to-end targets (names
                starting with `pty_`): unit, integration and doc tests
  --tier pty    only the `pty_*` integration targets
  --tier full   both: plain `cargo test --workspace`
  --jobs N      run N test binaries at once (default: three quarters of the CPUs,
                or GITPOULPI_TEST_JOBS). At most half that many PTY binaries run
                at once (GITPOULPI_PTY_JOBS), 2 tests at a time each, 4 for the
                heaviest (GITPOULPI_PTY_TEST_THREADS)
  --serial      the old path: plain `cargo test`, one binary after another
  --no-fail-fast  keep running after a failing test binary
  --no-run      compile the selected tests without running them
  --dry-run     print the cargo commands without running them
  --failed      rerun only what failed in the last run (target/xtask-test-failures.tsv,
                written by every run): builds just those targets and runs each with
                its failing tests as exact filters. Rewrites the file, so repeating
                it converges to nothing. Run the full tier again before a release tag
  (xtask/run-alone.txt: tests that pass alone but fail under load run alone after
                the pool; a test failing in the pool is rerun alone and, if it passes,
                reported as "passed only when run alone" without failing the run)
  --only PATTERN  with --failed: rerun only the recorded failures whose target or
                test name contains PATTERN (repeatable), i.e. the ones a fix is
                meant to cure. The other recorded failures stay in the file";

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tier {
    Fast,
    Pty,
    Full,
}

/// Names of the integration-test targets (`kind = test`) of the workspace.
pub fn integration_targets() -> Result<Vec<String>, String> {
    let out = Command::new("cargo")
        .args(["metadata", "--no-deps", "--format-version", "1"])
        .output()
        .map_err(|e| format!("cannot run cargo metadata: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "cargo metadata failed: {}",
            String::from_utf8_lossy(&out.stderr)
        ));
    }
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).map_err(|e| e.to_string())?;
    Ok(targets_from_metadata(&v))
}

/// Sorted, de-duplicated test-target names of a `cargo metadata` document.
pub fn targets_from_metadata(v: &serde_json::Value) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    for pkg in v["packages"].as_array().into_iter().flatten() {
        for t in pkg["targets"].as_array().into_iter().flatten() {
            let is_test = t["kind"]
                .as_array()
                .is_some_and(|k| k.iter().any(|k| k == "test"));
            if let (true, Some(name)) = (is_test, t["name"].as_str()) {
                names.push(name.to_string());
            }
        }
    }
    names.sort();
    names.dedup();
    names
}

pub fn is_pty(name: &str) -> bool {
    name.starts_with("pty_")
}

/// The cargo invocations (arguments after `cargo`) of a tier.
pub fn commands(
    tier: Tier,
    targets: &[String],
    no_fail_fast: bool,
    no_run: bool,
) -> Vec<Vec<String>> {
    let base = |extra: &[&str]| {
        let mut a: Vec<String> = ["test", "--workspace"].map(String::from).to_vec();
        if no_fail_fast {
            a.push("--no-fail-fast".into());
        }
        if no_run {
            a.push("--no-run".into());
        }
        a.extend(extra.iter().map(|s| s.to_string()));
        a
    };
    let tests = |want_pty: bool| {
        let mut a = Vec::new();
        for t in targets.iter().filter(|t| is_pty(t) == want_pty) {
            a.push("--test".to_string());
            a.push(t.clone());
        }
        a
    };
    match tier {
        Tier::Full => vec![base(&[])],
        Tier::Pty => {
            let mut a = base(&[]);
            a.extend(tests(true));
            vec![a]
        }
        Tier::Fast => {
            let mut a = base(&["--lib", "--bins"]);
            a.extend(tests(false));
            // Doc tests cannot be combined with other target selections.
            if no_run {
                vec![a]
            } else {
                vec![a, base(&["--doc"])]
            }
        }
    }
}

pub fn run(args: &[String]) -> Result<i32, String> {
    let mut tier = Tier::Fast;
    let (mut no_fail_fast, mut dry, mut no_run, mut serial) = (false, false, false, false);
    let mut failed = false;
    let mut only: Vec<String> = Vec::new();
    let mut jobs: Option<usize> = None;
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--tier" => {
                tier = match it.next().map(String::as_str) {
                    Some("fast") => Tier::Fast,
                    Some("pty") => Tier::Pty,
                    Some("full") => Tier::Full,
                    other => return Err(format!("--tier needs fast|pty|full, got {other:?}")),
                }
            }
            "--jobs" | "-j" => {
                jobs = Some(
                    it.next()
                        .and_then(|n| n.parse().ok())
                        .filter(|n| *n > 0)
                        .ok_or("--jobs needs a positive number")?,
                );
            }
            "--serial" => serial = true,
            "--failed" => failed = true,
            "--only" => only.push(it.next().ok_or("--only needs a pattern")?.clone()),
            "--no-fail-fast" => no_fail_fast = true,
            "--no-run" => no_run = true,
            "--dry-run" => dry = true,
            "-h" | "--help" => {
                println!("{USAGE}");
                return Ok(0);
            }
            other => return Err(format!("unknown option '{other}'\n\n{USAGE}")),
        }
    }
    if !only.is_empty() && !failed {
        return Err("--only goes with --failed".into());
    }
    if failed {
        if serial || no_run {
            return Err("--failed cannot be combined with --serial or --no-run".into());
        }
        return run_failed(
            jobs.unwrap_or_else(crate::test_runner::default_jobs),
            dry,
            &only,
        );
    }
    let targets = integration_targets()?;
    let cmds = commands(tier, &targets, no_fail_fast, no_run);
    if !serial && !no_run {
        let jobs = jobs.unwrap_or_else(crate::test_runner::default_jobs);
        // One pool for everything: the doc tests are listed once, and the
        // long fast binaries (the contract suite) overlap the PTY ones.
        let cmds = if tier == Tier::Full {
            let mut c = commands(Tier::Fast, &targets, no_fail_fast, no_run);
            c.extend(commands(Tier::Pty, &targets, no_fail_fast, no_run));
            c
        } else {
            cmds
        };
        return run_parallel(&cmds, jobs, no_fail_fast, dry);
    }
    let mut worst = 0;
    for c in &cmds {
        println!("cargo {}", c.join(" "));
        if dry {
            continue;
        }
        let status = Command::new("cargo")
            .args(c)
            .status()
            .map_err(|e| format!("cannot run cargo: {e}"))?;
        if !status.success() {
            worst = status.code().unwrap_or(1);
            if !no_fail_fast {
                return Ok(worst);
            }
        }
    }
    Ok(worst)
}

/// Builds every non-doc command once, runs the test executables with `jobs`
/// at a time and the doc-test command beside them (see `test_runner`).
fn run_parallel(
    cmds: &[Vec<String>],
    jobs: usize,
    no_fail_fast: bool,
    dry: bool,
) -> Result<i32, String> {
    use crate::test_runner as r;
    let started = std::time::Instant::now();
    let mut queue = Vec::new();
    for c in cmds {
        let is_doc = c.iter().any(|a| a == "--doc");
        if is_doc {
            println!("cargo {}  (as one job beside the pool)", c.join(" "));
            let mut args = c.clone();
            if !args.iter().any(|a| a == "--no-fail-fast") {
                args.push("--no-fail-fast".into());
            }
            queue.push(doc_job(args, &[]));
            continue;
        }
        println!("cargo {} --no-run --message-format=json", c.join(" "));
        if dry {
            continue;
        }
        match r::build(c) {
            Ok(j) => queue.extend(j),
            Err(e) => return Err(build_failed(e)),
        }
    }
    if dry {
        println!("(dry run: then up to {jobs} test binaries at once)");
        for l in crate::run_alone::describe(&crate::run_alone::entries()) {
            println!("{l}");
        }
        return Ok(0);
    }
    Ok(execute(queue, jobs, !no_fail_fast, started, false, &[]))
}

fn doc_job(mut args: Vec<String>, names: &[String]) -> crate::test_runner::Job {
    if !names.is_empty() {
        args.push("--".into());
        args.extend(names.iter().cloned());
        args.push("--exact".into());
    }
    crate::test_runner::Job {
        label: "doc tests".into(),
        program: "cargo".into(),
        args,
        cwd: None,
        envs: Vec::new(),
        target: Some(TargetId::doc()),
    }
}

/// A failed build: the failing targets still go to the failures file.
fn build_failed(e: crate::test_runner::BuildError) -> String {
    let rows: Vec<Failure> = e
        .targets
        .into_iter()
        .map(|target| Failure {
            target,
            tests: Vec::new(),
        })
        .collect();
    crate::test_runner::write_failures(&rows);
    e.message
}

/// Runs the pool, reports, writes the failures file. Exit code.
fn execute(
    queue: Vec<crate::test_runner::Job>,
    jobs: usize,
    fail_fast: bool,
    started: std::time::Instant,
    rerun: bool,
    kept: &[Failure],
) -> i32 {
    use crate::test_runner as r;
    println!("running {} test jobs, {jobs} at a time", queue.len());
    let (pool, listed) = crate::run_alone::split(queue, &crate::run_alone::entries());
    let mut outcomes = r::run_all(pool.clone(), jobs, r::default_pty_jobs(), fail_fast);
    r::alone_phase(&mut outcomes, &pool, listed, fail_fast);
    let failures = r::report(&mut outcomes, started.elapsed());
    let mut still = r::failures_of(&outcomes);
    still.extend_from_slice(kept);
    r::write_failures(&still);
    println!("failures written to {}", test_failures::path().display());
    if !kept.is_empty() {
        println!(
            "{} other recorded failure(s) were not selected by --only and stay in the file",
            kept.len()
        );
    }
    if failures > 0 {
        println!("rerun only those: cargo run -p xtask -- test --failed");
    } else if rerun && kept.is_empty() {
        println!(
            "the targeted rerun is green; run the full tier again before a release tag \
             (cargo run -p xtask -- test --tier full)"
        );
    }
    i32::from(failures > 0)
}

/// The cargo arguments that build just the targets of the failures file.
pub fn rerun_build_args(rows: &[Failure]) -> Vec<String> {
    let mut a: Vec<String> = vec!["test".into(), "--no-run".into()];
    let mut packages: Vec<&str> = Vec::new();
    let (mut lib, mut flags) = (false, Vec::<String>::new());
    for f in rows.iter().filter(|f| !f.target.is_doc()) {
        let t = &f.target;
        if !packages.contains(&t.package.as_str()) {
            packages.push(&t.package);
        }
        match t.kind.as_str() {
            "test" => flags.extend(["--test".to_string(), t.name.clone()]),
            "bin" => flags.extend(["--bin".to_string(), t.name.clone()]),
            _ => lib = true,
        }
    }
    for p in packages {
        a.extend(["-p".to_string(), p.to_string()]);
    }
    if lib {
        a.push("--lib".into());
    }
    a.extend(flags);
    a
}

/// Whether a built test executable is the target of a failures row (a lib
/// matches on package: its target name is the crate name).
pub fn row_matches(row: &TargetId, built: &TargetId) -> bool {
    row.package == built.package
        && row.kind == built.kind
        && (row.kind == "lib" || row.name == built.name)
}

/// Splits the recorded failures into the ones `--only` selects and the ones
/// it leaves alone. A pattern found in a target name selects its whole row;
/// one found only in test names selects just those tests, and the row's other
/// tests stay recorded. No pattern selects everything.
pub fn select(rows: Vec<Failure>, only: &[String]) -> (Vec<Failure>, Vec<Failure>) {
    if only.is_empty() {
        return (rows, Vec::new());
    }
    let hit = |s: &str| only.iter().any(|p| s.contains(p.as_str()));
    let (mut picked, mut kept) = (Vec::new(), Vec::new());
    for row in rows {
        if hit(&row.target.name) {
            picked.push(row);
            continue;
        }
        let (on, off): (Vec<String>, Vec<String>) = row.tests.iter().cloned().partition(|t| hit(t));
        if on.is_empty() {
            kept.push(row);
            continue;
        }
        picked.push(Failure {
            target: row.target.clone(),
            tests: on,
        });
        if !off.is_empty() {
            kept.push(Failure {
                target: row.target,
                tests: off,
            });
        }
    }
    (picked, kept)
}

/// `xtask test --failed`: rerun only what the last run recorded as failed.
fn run_failed(jobs: usize, dry: bool, only: &[String]) -> Result<i32, String> {
    use crate::test_runner as r;
    let started = std::time::Instant::now();
    let path = test_failures::path();
    let Some(rows) = test_failures::read(&path) else {
        return Err(format!(
            "no failures file at {}: run `cargo run -p xtask -- test` first",
            path.display()
        ));
    };
    if rows.is_empty() {
        println!("nothing failed last run");
        return Ok(0);
    }
    let (rows, kept) = select(rows, only);
    if rows.is_empty() {
        return Err(format!(
            "--only {only:?} matches none of the {} recorded failure(s)",
            kept.len()
        ));
    }
    let mut queue = Vec::new();
    for f in rows.iter().filter(|f| f.target.is_doc()) {
        println!("cargo test --workspace --doc (doc tests)");
        queue.push(doc_job(
            ["test", "--workspace", "--doc", "--no-fail-fast"]
                .map(String::from)
                .to_vec(),
            &f.tests,
        ));
    }
    if rows.iter().any(|f| !f.target.is_doc()) {
        let args = rerun_build_args(&rows);
        println!("cargo {} --message-format=json", args.join(" "));
        if dry {
            println!("(dry run)");
            for l in crate::run_alone::describe(&crate::run_alone::entries()) {
                println!("{l}");
            }
            return Ok(0);
        }
        for mut job in r::build(&args).map_err(build_failed)? {
            let Some(built) = job.target.clone() else {
                continue;
            };
            let Some(row) = rows.iter().find(|f| row_matches(&f.target, &built)) else {
                continue;
            };
            if !row.tests.is_empty() {
                job.args = row.tests.clone();
                job.args.push("--exact".into());
            }
            queue.push(job);
        }
    } else if dry {
        println!("(dry run)");
        for l in crate::run_alone::describe(&crate::run_alone::entries()) {
            println!("{l}");
        }
        return Ok(0);
    }
    let missing = rows.len().saturating_sub(queue.len());
    if missing > 0 {
        println!("warning: {missing} recorded target(s) no longer exist and are dropped");
    }
    // Every recorded target runs, whatever fails first.
    Ok(execute(queue, jobs, false, started, true, &kept))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn only_selects_the_failures_a_fix_targets_and_keeps_the_rest() {
        let id = |name: &str| TargetId {
            package: "gitpoulpi-tui".into(),
            name: name.into(),
            kind: "test".into(),
        };
        let rows = vec![
            Failure {
                target: id("pty_clone_hosted"),
                tests: t(&["each_source", "other"]),
            },
            Failure {
                target: id("pty_rewrite_paths"),
                tests: t(&["extracting"]),
            },
            Failure {
                target: id("pty_crash"),
                tests: vec![],
            },
        ];
        let (all, none) = select(rows.clone(), &[]);
        assert_eq!((all.len(), none.len()), (3, 0));
        let (picked, kept) = select(rows.clone(), &t(&["rewrite_paths"]));
        assert_eq!(picked, vec![rows[1].clone()]);
        assert_eq!(kept, vec![rows[0].clone(), rows[2].clone()]);
        // A test-name match takes just that test; its sibling stays recorded.
        let (picked, kept) = select(rows.clone(), &t(&["each_source"]));
        assert_eq!(picked[0].tests, t(&["each_source"]));
        assert_eq!(kept[0].tests, t(&["other"]));
        assert_eq!(kept.len(), 3);
    }

    #[test]
    fn tiers_partition_the_targets() {
        let all = t(&["a", "pty_ui", "pty_hosting", "b"]);
        let fast = &commands(Tier::Fast, &all, false, false)[0];
        let pty = &commands(Tier::Pty, &all, false, false)[0];
        assert!(fast.join(" ").ends_with("--test a --test b"));
        assert!(pty.join(" ").ends_with("--test pty_ui --test pty_hosting"));
        assert!(!pty.contains(&"--lib".to_string()));
        assert_eq!(
            commands(Tier::Full, &all, true, false),
            vec![t(&["test", "--workspace", "--no-fail-fast"])]
        );
    }

    #[test]
    fn rerun_builds_only_the_failed_targets() {
        let f = |p: &str, n: &str, k: &str| Failure {
            target: TargetId {
                package: p.into(),
                name: n.into(),
                kind: k.into(),
            },
            tests: vec![],
        };
        let rows = [
            f("a", "pty_x", "test"),
            f("b", "b", "lib"),
            f("a", "tool", "bin"),
            Failure {
                target: TargetId::doc(),
                tests: vec![],
            },
        ];
        assert_eq!(
            rerun_build_args(&rows).join(" "),
            "test --no-run -p a -p b --lib --test pty_x --bin tool"
        );
        let lib = TargetId {
            package: "b".into(),
            name: "b_crate".into(),
            kind: "lib".into(),
        };
        assert!(row_matches(&rows[1].target, &lib));
        assert!(!row_matches(&rows[0].target, &lib));
    }

    #[test]
    fn reads_test_targets_from_metadata() {
        let v = serde_json::json!({"packages":[{"targets":[
            {"kind":["lib"],"name":"x"},{"kind":["test"],"name":"pty_a"},{"kind":["test"],"name":"b"}]}]});
        assert_eq!(targets_from_metadata(&v), t(&["b", "pty_a"]));
    }
}
