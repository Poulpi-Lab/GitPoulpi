//! The "run alone" list of `xtask test` (`xtask/run-alone.txt`).
//!
//! A test that passes alone but fails under load runs alone, not beside the
//! others. Entries are `<crate>::<binary>` (a whole binary) or
//! `<crate>::<binary>::<test name>` (one test). The parallel phase leaves them
//! out; after it drains, each entry runs by itself. A whole binary keeps its
//! own `RUST_TEST_THREADS` (it is alone on the machine, its tests still run in
//! parallel with each other); one test runs with a single thread.
//!
//! The safety net lives here too: a test that failed in the parallel phase is
//! rerun alone with an exact filter (`rerun_job`), and `saved` tells which
//! of them passed.

use crate::test_runner::Job;

const LIST: &str = include_str!("../run-alone.txt");

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    pub package: String,
    pub binary: String,
    /// None: the whole binary.
    pub test: Option<String>,
}

impl Entry {
    pub fn line(&self) -> String {
        match &self.test {
            Some(t) => format!("{}::{}::{t}", self.package, self.binary),
            None => format!("{}::{}", self.package, self.binary),
        }
    }
}

/// The checked-in list.
pub fn entries() -> Vec<Entry> {
    parse(LIST).unwrap_or_default()
}

/// Parses a list; an error names the offending line.
pub fn parse(text: &str) -> Result<Vec<Entry>, String> {
    let mut out = Vec::new();
    for (i, raw) in text.lines().enumerate() {
        let line = raw.split(" #").next().unwrap_or("").trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut parts = line.splitn(3, "::");
        let (Some(package), Some(binary)) = (parts.next(), parts.next()) else {
            return Err(format!(
                "run-alone.txt line {}: `{line}` needs <crate>::<binary>",
                i + 1
            ));
        };
        if package.is_empty() || binary.is_empty() || package.contains(char::is_whitespace) {
            return Err(format!("run-alone.txt line {}: bad entry `{line}`", i + 1));
        }
        out.push(Entry {
            package: package.to_string(),
            binary: binary.to_string(),
            test: parts
                .next()
                .map(|t| t.trim().to_string())
                .filter(|t| !t.is_empty()),
        });
    }
    Ok(out)
}

fn matches(job: &Job, e: &Entry) -> bool {
    job.target
        .as_ref()
        .is_some_and(|t| t.package == e.package && t.name == e.binary && t.kind == "test")
}

/// Splits the queue: the jobs for the parallel pool (single-test entries
/// excluded with exact `--skip` filters, whole-binary entries removed) and the
/// alone jobs, in list order. A job that already carries `<name> --exact`
/// filters (a `--failed` rerun) gives the listed names to the alone phase.
pub fn split(queue: Vec<Job>, entries: &[Entry]) -> (Vec<Job>, Vec<Job>) {
    let mut pool = Vec::new();
    let mut alone: Vec<(usize, Job)> = Vec::new();
    for mut job in queue {
        let mine: Vec<(usize, &Entry)> = entries
            .iter()
            .enumerate()
            .filter(|(_, e)| matches(&job, e))
            .collect();
        if mine.is_empty() {
            pool.push(job);
            continue;
        }
        if let Some((i, _)) = mine.iter().find(|(_, e)| e.test.is_none()) {
            alone.push((*i, job));
            continue;
        }
        let filtered = job.args.iter().any(|a| a == "--exact");
        for (i, e) in &mine {
            let name = e.test.as_ref().expect("single-test entry");
            if filtered && !job.args.contains(name) {
                continue;
            }
            alone.push((*i, single_test_job(&job, name)));
        }
        if filtered {
            job.args.retain(|a| {
                a == "--exact" || !mine.iter().any(|(_, e)| e.test.as_deref() == Some(a))
            });
            if job.args.iter().all(|a| a == "--exact") {
                continue;
            }
        } else {
            for (_, e) in &mine {
                job.args
                    .extend(["--skip".to_string(), e.test.clone().unwrap()]);
            }
            // libtest rejects a repeated flag: one `--exact` for all the skips.
            job.args.push("--exact".to_string());
        }
        pool.push(job);
    }
    alone.sort_by_key(|(i, _)| *i);
    (pool, alone.into_iter().map(|(_, j)| j).collect())
}

fn single_test_job(base: &Job, name: &str) -> Job {
    exact_job(
        base,
        &[name.to_string()],
        format!("{} :: {name} [alone]", base.label),
    )
}

/// The job running `names` exactly, one test thread, nothing else.
fn exact_job(base: &Job, names: &[String], label: String) -> Job {
    let mut job = base.clone();
    job.label = label;
    job.args = names.to_vec();
    job.args.push("--exact".into());
    job.envs.push(("RUST_TEST_THREADS".into(), "1".into()));
    job
}

/// The safety-net job: `names` of a failed pool job, rerun alone.
pub fn rerun_job(base: &Job, names: &[String]) -> Job {
    exact_job(base, names, format!("{} [alone rerun]", base.label))
}

/// The dry-run description of the alone phase.
pub fn describe(entries: &[Entry]) -> Vec<String> {
    let mut out = vec![format!(
        "alone phase (xtask/run-alone.txt, {} entries): after the pool drains, one at a time; then every test that failed in the pool is rerun alone",
        entries.len()
    )];
    for e in entries {
        out.push(match &e.test {
            Some(t) => format!("  {}::{}  --exact {t}  (RUST_TEST_THREADS=1; skipped in the pool with --skip {t} --exact)", e.package, e.binary),
            None => format!("  {}::{}  (whole binary, not in the pool)", e.package, e.binary),
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_failures::TargetId;
    use std::path::PathBuf;

    fn job(pkg: &str, name: &str, args: &[&str]) -> Job {
        Job {
            label: format!("{pkg}::{name}"),
            program: PathBuf::from(name),
            args: args.iter().map(|s| s.to_string()).collect(),
            cwd: None,
            envs: vec![("RUST_TEST_THREADS".into(), "2".into())],
            target: Some(TargetId {
                package: pkg.into(),
                name: name.into(),
                kind: "test".into(),
            }),
        }
    }

    #[test]
    fn the_checked_in_list_parses() {
        let e = parse(LIST).unwrap();
        assert!(!e.is_empty());
        assert!(e
            .iter()
            .any(|e| e.binary == "pty_hosting" && e.test.is_some()));
    }

    #[test]
    fn parsing_skips_comments_and_reads_both_forms() {
        let e = parse("# c\n\n a::b # why\n a::c::t::x\n  # more\n").unwrap();
        assert_eq!(
            e,
            [
                Entry {
                    package: "a".into(),
                    binary: "b".into(),
                    test: None
                },
                Entry {
                    package: "a".into(),
                    binary: "c".into(),
                    test: Some("t::x".into())
                },
            ]
        );
        assert_eq!(e[1].line(), "a::c::t::x");
        assert!(parse("lonely\n").is_err());
    }

    #[test]
    fn a_single_test_is_skipped_exactly_in_the_pool_and_runs_alone() {
        let entries = parse("p::pty_a::m::t1\np::pty_a::m::t2\n").unwrap();
        let (pool, alone) = split(
            vec![job("p", "pty_a", &[]), job("p", "other", &[])],
            &entries,
        );
        assert_eq!(pool.len(), 2);
        assert_eq!(
            pool[0].args,
            ["--skip", "m::t1", "--skip", "m::t2", "--exact"]
        );
        assert!(pool[1].args.is_empty());
        assert_eq!(alone.len(), 2);
        assert_eq!(alone[0].args, ["m::t1", "--exact"]);
        assert!(alone[0]
            .envs
            .contains(&("RUST_TEST_THREADS".into(), "1".into())));
        assert_eq!(alone[1].args, ["m::t2", "--exact"]);
    }

    #[test]
    fn a_whole_binary_leaves_the_pool_and_keeps_its_threads() {
        let entries = parse("p::pty_a\n").unwrap();
        let (pool, alone) = split(vec![job("p", "pty_a", &[]), job("p", "b", &[])], &entries);
        assert_eq!(pool.len(), 1);
        assert_eq!(alone.len(), 1);
        assert_eq!(
            alone[0].envs,
            [("RUST_TEST_THREADS".to_string(), "2".to_string())]
        );
    }

    #[test]
    fn other_packages_and_unlisted_binaries_are_untouched() {
        let entries = parse("p::pty_a::t\n").unwrap();
        let (pool, alone) = split(vec![job("q", "pty_a", &[])], &entries);
        assert_eq!((pool.len(), alone.len()), (1, 0));
        assert!(pool[0].args.is_empty());
    }

    #[test]
    fn a_failed_rerun_filter_hands_the_listed_name_to_the_alone_phase() {
        let entries = parse("p::pty_a::t1\n").unwrap();
        let (pool, alone) = split(vec![job("p", "pty_a", &["t1", "t2", "--exact"])], &entries);
        assert_eq!(pool[0].args, ["t2", "--exact"]);
        assert_eq!(alone[0].args, ["t1", "--exact"]);
        let (pool, alone) = split(vec![job("p", "pty_a", &["t1", "--exact"])], &entries);
        assert!(pool.is_empty());
        assert_eq!(alone.len(), 1);
    }

    #[test]
    fn the_rerun_job_is_exact_and_single_threaded() {
        let r = rerun_job(&job("p", "pty_a", &[]), &["a".into(), "b".into()]);
        assert_eq!(r.args, ["a", "b", "--exact"]);
        assert_eq!(
            r.envs.last().unwrap(),
            &("RUST_TEST_THREADS".to_string(), "1".to_string())
        );
        assert!(describe(&parse("p::q::t\n").unwrap())[1].contains("--exact t"));
    }
}
