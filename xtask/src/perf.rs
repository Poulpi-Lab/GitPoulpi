//! `cargo run -p xtask -- perf`: the perf regression guard
//! (the perf design).
//!
//! Builds (or reuses) a 2k-file / 5k-commit fixture **outside the temp
//! dir**, runs an in-process daemon per backend, times every operation the
//! way the TUI issues it (over `gitpoulpi-ipc`), counts the `git` processes
//! each one spawned, prints the table, writes `target/perf/<date>.json`, and
//! fails when an operation exceeds its budget.
//!
//! Budgets are deliberately machine independent: spawns are exact counts;
//! time is `mult * floor + slack`, where the floor is the median of
//! `git --version` on this machine, so a slow or antivirus-laden box scales
//! the budget instead of failing it.

use std::collections::BTreeMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::time::Instant;

use gitpoulpi_cli_backend::{total_spawn_count, SystemGitBackend};
use gitpoulpi_core::domain::{OperationKind, RepoId};
use gitpoulpi_core::GitBackend;
use gitpoulpi_daemon::Daemon;
use gitpoulpi_git2_backend::Git2Backend;
use gitpoulpi_ipc::{Client, Request};
use serde_json::{json, Value};

pub const USAGE: &str = "\
Usage: cargo run -p xtask -- perf [options]

  --repo <path>      measure read operations on this repository (never
                     mutated); default is the generated fixture
  --fixture <dir>    where the fixture lives (default target/perf/fixture;
                     keep it out of the temp dir, antivirus scans it)
  --files <n>        fixture size, default 2000
  --commits <n>      fixture history, default 5000
  --backend <b>      git2 | system | both (default both)
  --no-budget        print and record, do not fail on a regression
  --enforce          fail on a regression (the default; accepted so the TUI
                     and daemon guards read alike)
  --rebuild          regenerate the fixture even if one exists
  --search           measure the workspace search's warm query instead
                     (50k commits x 10 repositories); see --search --help
  --tui              measure the real TUI binary instead (idle CPU, terminal
                     bytes, held key, key latency); see --tui --help
  --rewrite          time the history rewrite content engine (remove a path,
                     replace text) against the plan's targets; see
                     --rewrite --help";

struct Opts {
    repo: Option<PathBuf>,
    fixture: PathBuf,
    files: usize,
    commits: usize,
    backends: Vec<&'static str>,
    budgets: bool,
    rebuild: bool,
}

fn parse(args: &[String]) -> Result<Opts, String> {
    let mut o = Opts {
        repo: None,
        fixture: workspace_root().join("target/perf/fixture"),
        files: 2000,
        commits: 5000,
        backends: vec!["git2", "system"],
        budgets: true,
        rebuild: false,
    };
    let mut it = args.iter();
    while let Some(a) = it.next() {
        let mut val = |name: &str| it.next().cloned().ok_or(format!("{name} needs a value"));
        match a.as_str() {
            "--repo" => o.repo = Some(PathBuf::from(val("--repo")?)),
            "--fixture" => o.fixture = PathBuf::from(val("--fixture")?),
            "--files" => o.files = val("--files")?.parse().map_err(|_| "bad --files")?,
            "--commits" => o.commits = val("--commits")?.parse().map_err(|_| "bad --commits")?,
            "--backend" => {
                o.backends = match val("--backend")?.as_str() {
                    "git2" => vec!["git2"],
                    "system" => vec!["system"],
                    "both" => vec!["git2", "system"],
                    other => return Err(format!("unknown backend '{other}'")),
                }
            }
            "--no-budget" => o.budgets = false,
            "--enforce" => o.budgets = true,
            "--rebuild" => o.rebuild = true,
            other => return Err(format!("unknown option '{other}'\n\n{USAGE}")),
        }
    }
    Ok(o)
}

pub(crate) fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask lives one level below the workspace root")
        .to_path_buf()
}

// ---- fixture -------------------------------------------------------------

fn git_in(dir: &Path, args: &[&str]) -> Result<(), String> {
    let out = Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .map_err(|e| format!("cannot run git: {e}"))?;
    if out.status.success() {
        Ok(())
    } else {
        Err(format!(
            "git {args:?}: {}",
            String::from_utf8_lossy(&out.stderr)
        ))
    }
}

/// Rust port of `scripts/perf/mkfix.py` (same shape: a main line with
/// feature branches merged back every hundred commits), minus the big file.
pub(crate) fn build_fixture(dir: &Path, nf: usize, nc: usize) -> Result<(), String> {
    let _ = std::fs::remove_dir_all(dir);
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let files: Vec<String> = (0..nf)
        .map(|i| format!("src/mod{:02}/file{:04}.txt", i % 50, i))
        .collect();
    let mut content: Vec<Vec<String>> = files
        .iter()
        .map(|f| (0..40).map(|j| format!("line {j} of {f}")).collect())
        .collect();
    let mut seed: u64 = 1;
    let mut rnd = |n: usize| {
        seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((seed >> 33) as usize) % n
    };
    let mut out: Vec<u8> = Vec::new();
    let mut mark = 0u64;
    let mut t = 1_600_000_000u64;
    let mut commit = |out: &mut Vec<u8>,
                      branch: &str,
                      msg: &str,
                      changes: &[usize],
                      content: &[Vec<String>],
                      from: Option<u64>,
                      merge: Option<u64>|
     -> u64 {
        mark += 1;
        t += 60;
        let head = format!(
            "commit refs/heads/{branch}\nmark :{mark}\nauthor Dev <dev@x> {t} +0000\ncommitter Dev <dev@x> {t} +0000\ndata {}\n{msg}\n",
            msg.len() + 1
        );
        out.extend(head.as_bytes());
        if let Some(f) = from {
            out.extend(format!("from :{f}\n").as_bytes());
        }
        if let Some(m) = merge {
            out.extend(format!("merge :{m}\n").as_bytes());
        }
        for &c in changes {
            let body = content[c].join("\n") + "\n";
            out.extend(format!("M 100644 inline {}\ndata {}\n", files[c], body.len()).as_bytes());
            out.extend(body.as_bytes());
        }
        out.push(b'\n');
        mark
    };
    let all: Vec<usize> = (0..nf).collect();
    let mut main = commit(&mut out, "main", "initial", &all, &content, None, None);
    let mut feat: Option<u64> = None;
    for i in 1..nc {
        let k = 1 + rnd(4);
        let ch: Vec<usize> = (0..k).map(|_| rnd(nf)).collect();
        for &f in &ch {
            let line = rnd(40);
            content[f][line] = format!("edit {i} {}", content[f][line]);
        }
        if i % 100 == 10 {
            feat = Some(main);
        }
        match feat {
            Some(f) if (10..20).contains(&(i % 100)) => {
                let b = format!("feature/f{}", i / 100);
                feat = Some(commit(
                    &mut out,
                    &b,
                    &format!("feature work {i}"),
                    &ch,
                    &content,
                    Some(f),
                    None,
                ));
            }
            Some(f) if i % 100 == 20 => {
                main = commit(
                    &mut out,
                    "main",
                    &format!("Merge feature {}", i / 100),
                    &ch,
                    &content,
                    Some(main),
                    Some(f),
                );
                feat = None;
            }
            _ => {
                main = commit(
                    &mut out,
                    "main",
                    &format!("commit {i}: change {k} files"),
                    &ch,
                    &content,
                    Some(main),
                    None,
                );
            }
        }
    }
    git_in(dir, &["init", "-q", "-b", "main", "."])?;
    let mut child = Command::new("git")
        .args(["fast-import", "--quiet"])
        .current_dir(dir)
        .stdin(Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;
    child
        .stdin
        .take()
        .unwrap()
        .write_all(&out)
        .map_err(|e| e.to_string())?;
    if !child.wait().map_err(|e| e.to_string())?.success() {
        return Err("git fast-import failed".into());
    }
    git_in(dir, &["checkout", "-q", "main"])?;
    git_in(dir, &["config", "user.name", "Dev"])?;
    git_in(dir, &["config", "user.email", "dev@x"])?;
    std::fs::write(dir.join(".perf-fixture"), format!("{nf}x{nc}\n")).map_err(|e| e.to_string())
}

pub(crate) fn fixture_ready(dir: &Path, nf: usize, nc: usize) -> bool {
    std::fs::read_to_string(dir.join(".perf-fixture"))
        .map(|s| s.trim() == format!("{nf}x{nc}"))
        .unwrap_or(false)
}

/// Restores the generated fixture to a clean tree (ours to reset: only ever
/// called on the fixture, never on `--repo`).
fn reset_fixture(dir: &Path) {
    let _ = git_in(dir, &["reset", "-q", "--hard", "HEAD"]);
    let _ = git_in(dir, &["clean", "-qfd", "-e", ".perf-fixture"]);
}

/// Appends a line to the first `n` tracked files, returning their paths.
fn touch(dir: &Path, n: usize, nf: usize) -> Vec<String> {
    let mut names: Vec<String> = (0..nf)
        .map(|i| format!("src/mod{:02}/file{:04}.txt", i % 50, i))
        .collect();
    names.sort();
    names.truncate(n);
    for p in &names {
        let mut f = std::fs::OpenOptions::new()
            .append(true)
            .open(dir.join(p))
            .expect("fixture file");
        f.write_all(b"perf edit\n").unwrap();
    }
    names
}

/// The first root commit of the repository, as the hex the wire carries.
fn root_commit(dir: &Path) -> Option<Value> {
    let out = Command::new("git")
        .args(["rev-list", "--max-parents=0", "HEAD"])
        .current_dir(dir)
        .output()
        .ok()?;
    let hex = String::from_utf8_lossy(&out.stdout)
        .lines()
        .next()?
        .trim()
        .to_string();
    // The wire form of an `Oid`: ask the type, never guess its encoding.
    let oid = gitpoulpi_core::domain::Oid::from_hex(&hex)?;
    serde_json::to_value(oid).ok()
}

// ---- measurement ---------------------------------------------------------

#[derive(Clone, Debug)]
struct Row {
    backend: &'static str,
    op: String,
    ms: f64,
    spawns: u64,
}

struct Bench {
    client: Client,
    repo: RepoId,
    next_id: u64,
}

impl Bench {
    async fn call(&mut self, command: OperationKind, args: Value) -> Result<(), String> {
        self.call_value(command, args).await.map(|_| ())
    }

    async fn call_value(&mut self, command: OperationKind, args: Value) -> Result<Value, String> {
        self.next_id += 1;
        let resp = self
            .client
            .call(&Request {
                id: self.next_id,
                session: "perf".into(),
                repo: Some(self.repo.clone()),
                command,
                args,
            })
            .await
            .map_err(|e| e.to_string())?;
        resp.result.map_err(|e| format!("{command:?}: {e:?}"))
    }

    /// Times one request `reps` times, reporting the median time and the
    /// largest spawn count seen.
    /// The median of one cold ask per entry of `args` (each a different
    /// read, so none is served from the daemon's cache).
    async fn cold(
        &mut self,
        rows: &mut Vec<Row>,
        backend: &'static str,
        name: &str,
        args: Vec<Value>,
    ) -> Result<(), String> {
        let (mut times, mut max_spawns) = (Vec::new(), 0);
        for a in args {
            let before = total_spawn_count();
            let t0 = Instant::now();
            self.call(OperationKind::Log, a).await?;
            times.push(t0.elapsed().as_secs_f64() * 1000.0);
            max_spawns = max_spawns.max(total_spawn_count() - before);
        }
        rows.push(Row {
            backend,
            op: name.to_string(),
            ms: median(&mut times),
            spawns: max_spawns,
        });
        Ok(())
    }

    async fn op(
        &mut self,
        rows: &mut Vec<Row>,
        backend: &'static str,
        name: &str,
        command: OperationKind,
        args: Value,
        reps: usize,
    ) -> Result<(), String> {
        let (mut times, mut max_spawns) = (Vec::new(), 0);
        for _ in 0..reps {
            let before = total_spawn_count();
            let t0 = Instant::now();
            self.call(command, args.clone()).await?;
            times.push(t0.elapsed().as_secs_f64() * 1000.0);
            max_spawns = max_spawns.max(total_spawn_count() - before);
        }
        rows.push(Row {
            backend,
            op: name.to_string(),
            ms: median(&mut times),
            spawns: max_spawns,
        });
        Ok(())
    }
}

fn median(v: &mut [f64]) -> f64 {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v[v.len() / 2]
}

/// The machine's cost of starting one `git` process, in milliseconds.
fn spawn_floor_ms() -> f64 {
    let mut t = Vec::new();
    for _ in 0..9 {
        let t0 = Instant::now();
        let _ = Command::new("git").arg("--version").output();
        t.push(t0.elapsed().as_secs_f64() * 1000.0);
    }
    median(&mut t)
}

/// `(max spawns, time multiplier of the floor, slack ms)` for an operation,
/// or `None` when it is only reported. Spawn limits are the measured value
/// today; time is `mult * floor + slack`. Since P7 the slack is about 2x
/// the median of three quiet release runs (docs/developer/perf-budgets.md),
/// never under 10 ms so a loaded dev box cannot flake a sub-millisecond op.
fn budget(backend: &str, op: &str) -> Option<(u64, f64, f64)> {
    let git2 = backend == "git2";
    Some(match (op, git2) {
        ("status (clean)", true) => (0, 0.0, 10.0),
        ("status (clean)", false) => (1, 4.0, 10.0),
        ("refs", true) => (0, 0.0, 10.0),
        // 3 spawns today (refs + the tag mirror reads).
        ("refs", false) => (3, 4.0, 10.0),
        ("log 200", _) => (1, 6.0, 40.0),
        ("log 5000", _) => (1, 10.0, 80.0),
        // The Log's first paint with `log.use_mailmap` on is the same cold
        // read as without it (S16): `%aN`/`%aE`, and a lookup per commit.
        ("log 5000 (cold, mailmap off)", _)
        | ("log 5000 (cold, mailmap on)", _)
        | ("log 5000 (cold, .mailmap of 300 entries)", _) => (1, 10.0, 80.0),
        ("commit count (cold)", _) => (1, 4.0, 35.0),
        // "commit count (warm)" is reported, not budgeted: it is the daemon
        // cache (0 spawns) only while the refs are older than the cache's 1 s
        // racy window, which this harness (it mutates refs between passes)
        // cannot promise. `cache::tests::the_commit_total_is_cached_until_a_ref_moves`
        // guards it deterministically.
        ("stage file", true) | ("unstage file", true) => (0, 1.0, 15.0),
        ("stage all", true) | ("unstage all", true) => (0, 1.0, 18.0),
        // `general.undo_enabled = false`: the pre-undo cost, exactly. git2
        // stages in process, the system backend runs the one `git add`.
        ("stage file (undo off)", true) | ("unstage file (undo off)", true) => (0, 1.0, 12.0),
        ("stage all (undo off)", true) | ("unstage all (undo off)", true) => (0, 1.0, 12.0),
        ("discard 1 file (undo off)", true) => (0, 2.0, 12.0),
        // The first system-git stage of a daemon's life also pays its
        // one-time LFS probe (3 spawns, as before undo existed).
        ("stage file (undo off)", false) => (3, 2.0, 15.0),
        ("unstage file (undo off)", false) => (1, 2.0, 12.0),
        ("stage all (undo off)", false) | ("unstage all (undo off)", false) => (1, 3.0, 12.0),
        ("discard 1 file (undo off)", false) => (3, 4.0, 15.0),
        // Staging is undoable: the journal records the index tree before and
        // after. The trees come from libgit2 in the daemon's process, pins
        // are loose ref files and the journal bytes are written behind, so
        // the system backend pays the one `git add` and nothing for the
        // journal. git2: no process at all.
        ("stage file", false) | ("unstage file", false) => (1, 2.0, 20.0),
        ("stage all", false) | ("unstage all", false) => (1, 3.0, 20.0),
        // Discard takes a safety snapshot before and after, built by
        // libgit2 on both backends: only the discard itself spawns (none on
        // git2, the bare 3 on system git).
        ("discard 50 files", true) => (0, 5.0, 35.0),
        ("discard 50 files", false) => (3, 5.0, 40.0),
        // The Diff view lists first (one tree comparison / one
        // `--name-status` spawn) and loads one file's hunks alone.
        ("diff files (big commit)", true) => (0, 0.0, 10.0),
        // The big commit is the root: system git's first `<hex>^` diff
        // fails and is retried against the empty tree (2 spawns).
        ("diff files (big commit)", false) => (2, 5.0, 10.0),
        ("diff 1 file (big commit)", true) => (0, 0.0, 10.0),
        ("diff 1 file (big commit)", false) => (2, 5.0, 10.0),
        ("diff files (200 dirty)", true) => (0, 0.0, 10.0),
        ("diff files (200 dirty)", false) => (1, 4.0, 10.0),
        ("diff 1 file (200 dirty)", true) => (0, 0.0, 10.0),
        ("diff 1 file (200 dirty)", false) => (1, 4.0, 10.0),
        // The Log "Changes" column: a viewport's commits in one batch.
        ("commit stats x60 (cold)", true) => (0, 0.0, 55.0),
        ("commit stats x60 (cold)", false) => (1, 6.0, 35.0),
        ("commit stats x60 (warm)", _) => (0, 0.0, 10.0),
        // Undo journal. Spawns on the system backend are the plain
        // operation plus the journal's own: branch create +0 (1 total),
        // branch delete +3 (4 total: snapshot, config section, pin);
        // undo/redo = ref snapshot + one transaction (+ the config
        // restore for a delete) + the snapshot behind JournalChanged.
        // git2 does all of it in process: 0.
        ("branch create (journal)", true) => (0, 0.0, 10.0),
        ("branch create (journal)", false) => (1, 3.0, 10.0),
        ("branch delete (journal)", true) => (0, 0.0, 10.0),
        ("branch delete (journal)", false) => (4, 6.0, 10.0),
        // Outside-change snapshot: a full read of refs, HEAD, stashes and
        // worktrees (one process on the system backend while a stash or a
        // linked worktree exists), diffed and recorded with its pin.
        ("outside snapshot read (nothing moved)", true) => (0, 0.0, 10.0),
        ("outside snapshot read (nothing moved)", false) => (2, 6.0, 12.0),
        ("outside change recorded (1 ref)", true) => (0, 0.0, 25.0),
        ("outside change recorded (1 ref)", false) => (3, 6.0, 25.0),
        ("branch create (journal + outside)", true) => (0, 0.0, 30.0),
        ("branch create (journal + outside)", false) => (3, 6.0, 30.0),
        ("journal state", true) => (0, 0.0, 10.0),
        ("journal state", false) => (2, 4.0, 10.0),
        ("undo (branch create)", true) | ("redo (branch create)", true) => (0, 0.0, 10.0),
        // 3 on a clean journal; one more when the entry now on top has a
        // config section to read for the JournalChanged state.
        ("undo (branch create)", false) => (4, 6.0, 12.0),
        ("redo (branch create)", false) => (3, 5.0, 12.0),
        ("undo (branch delete)", true) => (0, 0.0, 10.0),
        ("undo (branch delete)", false) => (6, 8.0, 12.0),
        // The auto-stash sequence on 200 dirty files: git's own stash push
        // and pop dominate (~35 ms and ~11 ms bare); the rest is the undo
        // journal's three entries. Journal reads run in process, so the
        // system backend spawns 6 (was 22). Slack is about 2x the median.
        ("checkout with autostash", true) => (3, 0.0, 320.0),
        ("checkout with autostash", false) => (6, 0.0, 320.0),
        _ => return None,
    })
}

async fn run_backend(
    backend: &'static str,
    repo_dir: &Path,
    mutate: bool,
    nf: usize,
    rows: &mut Vec<Row>,
) -> Result<(), String> {
    let be: Arc<dyn GitBackend> = match backend {
        "git2" => Arc::new(Git2Backend::new()),
        _ => Arc::new(SystemGitBackend::new()),
    };
    let handle = Daemon::new(format!("perf-{backend}"))
        .spawn(be.clone())
        .await
        .map_err(|e| e.to_string())?;
    let repo = be
        .discover(repo_dir)
        .map_err(|e| format!("not a repository: {e}"))?
        .repo
        .id;
    let mut b = Bench {
        client: Client::connect_port(handle.port)
            .await
            .map_err(|e| e.to_string())?,
        repo,
        next_id: 0,
    };
    // Warm: the first request pays one-off setup and the OS file cache.
    b.call(OperationKind::Status, json!({})).await?;
    b.call(OperationKind::Status, json!({})).await?;

    let log = |limit: u32| json!({ "opts": { "limit": limit, "skip": 0, "start_ref": null, "all_refs": true, "paths": [] } });
    let reads: Vec<(&str, OperationKind, Value)> = vec![
        ("status (clean)", OperationKind::Status, json!({})),
        ("log 5000", OperationKind::Log, log(5000)),
        ("log 200", OperationKind::Log, log(200)),
        ("refs", OperationKind::Refs, json!({})),
        ("stashes", OperationKind::Stashes, json!({})),
        ("worktrees", OperationKind::Worktrees, json!({})),
        ("remotes", OperationKind::Remotes, json!({})),
        ("lfs_status", OperationKind::LfsStatus, json!({})),
        ("conflicts", OperationKind::Conflicts, json!({})),
    ];
    for (name, cmd, args) in reads {
        b.op(rows, backend, name, cmd, args, 3).await?;
    }
    // `log.use_mailmap` (S16): the Log's first paint is a cold read. Each
    // row asks three times with a different limit (a different cache key, so
    // every ask walks the history) and reports the median.
    {
        let cold = |limit: u32| json!({ "opts": { "limit": limit, "skip": 0, "start_ref": null, "all_refs": true, "paths": [] } });
        let group = |name: &'static str, first: u32| {
            let args: Vec<Value> = (first..first + 3).map(cold).collect();
            (name, args)
        };
        let off = group("log 5000 (cold, mailmap off)", 4990);
        let on = group("log 5000 (cold, mailmap on)", 4993);
        let with_file = group("log 5000 (cold, .mailmap of 300 entries)", 4996);
        std::env::set_var("GITPOULPI_LOG_USE_MAILMAP", "false");
        let measured = b.cold(rows, backend, off.0, off.1).await;
        std::env::remove_var("GITPOULPI_LOG_USE_MAILMAP");
        measured?;
        b.cold(rows, backend, on.0, on.1).await?;
        if mutate {
            // The fixture is ours to touch; a repository handed in with
            // `--repo` is not.
            let path = repo_dir.join(".mailmap");
            let mut text: String = (0..299)
                .map(|i| format!("Person {i} <p{i}@example.org> Old {i} <o{i}@example.org>\n"))
                .collect();
            // The fixture's one author is among the mapped.
            text.push_str("Developer <developer@x> Dev <dev@x>\n");
            std::fs::write(&path, text).map_err(|e| e.to_string())?;
            let measured = b.cold(rows, backend, with_file.0, with_file.1).await;
            let _ = std::fs::remove_file(&path);
            measured?;
        }
    }
    // The status bar's commit total: the first answer counts the walk (one
    // `rev-list --count`), every later one is the daemon's cache while the
    // refs stand still (no spawn).
    {
        let args = json!({ "opts": { "limit": null, "skip": 0, "start_ref": null, "all_refs": true, "paths": [] } });
        let cmd = OperationKind::CountCommits;
        b.op(rows, backend, "commit count (cold)", cmd, args.clone(), 1)
            .await?;
        b.op(rows, backend, "commit count (warm)", cmd, args, 3)
            .await?;
    }
    // The 2 s poll's Log read from a client that holds the answer: the
    // daemon says "not modified" instead of re-sending the 5,000 commits
    // (reported, never budgeted; compare with "log 5000").
    {
        let held = b.call_value(OperationKind::Log, log(5000)).await?;
        let etag = gitpoulpi_ipc::etag::of_value(&held);
        let args = gitpoulpi_ipc::etag::with_if_none_match(log(5000), etag);
        let reply = b.call_value(OperationKind::Log, args.clone()).await?;
        if !gitpoulpi_ipc::etag::is_not_modified(&reply) {
            return Err("log 5000 with its own etag was not answered not-modified".into());
        }
        b.op(
            rows,
            backend,
            "log 5000 (poll, not modified)",
            OperationKind::Log,
            args,
            3,
        )
        .await?;
    }
    // Log "Changes" column: the numbers for one viewport of commits (the
    // first 60 rows). Cold is one batch -- ONE git process on the system
    // backend, not one per commit; warm is the daemon's per-commit cache.
    {
        let page = b.call_value(OperationKind::Log, log(60)).await?;
        let ids: Vec<Value> = page["nodes"]
            .as_array()
            .map(|n| n.iter().map(|c| c["id"].clone()).collect())
            .unwrap_or_default();
        if !ids.is_empty() {
            let args = json!({ "ids": ids });
            let n = ids.len();
            b.op(
                rows,
                backend,
                &format!("commit stats x{n} (cold)"),
                OperationKind::CommitStats,
                args.clone(),
                1,
            )
            .await?;
            b.op(
                rows,
                backend,
                &format!("commit stats x{n} (warm)"),
                OperationKind::CommitStats,
                args,
                3,
            )
            .await?;
        }
    }
    // The Diff view of the fixture's biggest commit (its root: every file
    // added). "diff full" is reported, never budgeted.
    if let Some(root) = root_commit(repo_dir) {
        let target = json!({ "target": { "CommitVsParent": root } });
        let first =
            json!({ "target": { "CommitVsParent": root }, "paths": ["src/mod00/file0000.txt"] });
        b.op(
            rows,
            backend,
            "diff files (big commit)",
            OperationKind::DiffFiles,
            target.clone(),
            3,
        )
        .await?;
        b.op(
            rows,
            backend,
            "diff 1 file (big commit)",
            OperationKind::Diff,
            first,
            3,
        )
        .await?;
        b.op(
            rows,
            backend,
            "diff full (big commit)",
            OperationKind::Diff,
            target,
            3,
        )
        .await?;
    }
    if mutate {
        for n in [1usize, 200] {
            let paths = touch(repo_dir, n, nf);
            b.op(
                rows,
                backend,
                &format!("status ({n} dirty)"),
                OperationKind::Status,
                json!({}),
                1,
            )
            .await?;
            if n == 200 {
                let wt = json!({ "target": "WorkingTreeVsIndex" });
                let one = json!({ "target": "WorkingTreeVsIndex", "paths": [&paths[0]] });
                b.op(
                    rows,
                    backend,
                    "diff files (200 dirty)",
                    OperationKind::DiffFiles,
                    wt.clone(),
                    1,
                )
                .await?;
                b.op(
                    rows,
                    backend,
                    "diff 1 file (200 dirty)",
                    OperationKind::Diff,
                    one,
                    1,
                )
                .await?;
                b.op(
                    rows,
                    backend,
                    "diff full (200 dirty)",
                    OperationKind::Diff,
                    wt,
                    1,
                )
                .await?;
            }
            let file = json!({ "target": { "File": paths[0] } });
            let all = json!({ "target": "All" });
            if n == 1 {
                // Undo turned off: the pre-undo cost (the daemon reads the
                // setting from the environment on every request).
                std::env::set_var("GITPOULPI_GENERAL_UNDO_ENABLED", "false");
                for (name, op, args) in [
                    ("stage file (undo off)", OperationKind::StageFile, &file),
                    ("unstage file (undo off)", OperationKind::UnstageFile, &file),
                    ("stage all (undo off)", OperationKind::StageAll, &all),
                    ("unstage all (undo off)", OperationKind::UnstageAll, &all),
                ] {
                    b.op(rows, backend, name, op, args.clone(), 1).await?;
                }
                b.op(
                    rows,
                    backend,
                    "discard 1 file (undo off)",
                    OperationKind::Discard,
                    json!({ "paths": [paths[0]] }),
                    1,
                )
                .await?;
                std::env::remove_var("GITPOULPI_GENERAL_UNDO_ENABLED");
                touch(repo_dir, n, nf);
            }
            let sfx = if n == 1 { "" } else { " (200 dirty)" };
            let nm = |s: &str| format!("{s}{sfx}");
            b.op(
                rows,
                backend,
                &nm("stage file"),
                OperationKind::StageFile,
                file.clone(),
                1,
            )
            .await?;
            b.op(
                rows,
                backend,
                &nm("unstage file"),
                OperationKind::UnstageFile,
                file,
                1,
            )
            .await?;
            b.op(
                rows,
                backend,
                &nm("stage all"),
                OperationKind::StageAll,
                all.clone(),
                1,
            )
            .await?;
            b.op(
                rows,
                backend,
                &nm("unstage all"),
                OperationKind::UnstageAll,
                all,
                1,
            )
            .await?;
            b.op(
                rows,
                backend,
                &nm("discard 1 file"),
                OperationKind::Discard,
                json!({ "paths": [paths[0]] }),
                1,
            )
            .await?;
            if n == 200 {
                b.op(
                    rows,
                    backend,
                    "discard 50 files",
                    OperationKind::Discard,
                    json!({ "paths": &paths[1..51] }),
                    1,
                )
                .await?;
            }
            reset_fixture(repo_dir);
        }
        autostash_rows(&mut b, rows, backend, repo_dir, nf).await?;
        // Undo journal (the undo-redo design): what recording
        // a mutation and undoing it cost on top of the plain operation.
        // The branch is created and removed again, so the fixture ends as
        // it started (its `.git/gitpoulpi/journal.json` aside).
        if let Some(at) = root_commit(repo_dir) {
            let create = json!({ "name": "perf-undo", "at": at });
            let delete = json!({ "name": "perf-undo" });
            b.op(
                rows,
                backend,
                "branch create (journal)",
                OperationKind::BranchCreate,
                create,
                1,
            )
            .await?;
            b.op(
                rows,
                backend,
                "journal state",
                OperationKind::JournalState,
                json!({}),
                3,
            )
            .await?;
            b.op(
                rows,
                backend,
                "undo (branch create)",
                OperationKind::Undo,
                json!({}),
                1,
            )
            .await?;
            b.op(
                rows,
                backend,
                "redo (branch create)",
                OperationKind::Redo,
                json!({}),
                1,
            )
            .await?;
            b.op(
                rows,
                backend,
                "branch delete (journal)",
                OperationKind::BranchDelete,
                delete.clone(),
                1,
            )
            .await?;
            b.op(
                rows,
                backend,
                "undo (branch delete)",
                OperationKind::Undo,
                json!({}),
                1,
            )
            .await?;
            let _ = b.call(OperationKind::BranchDelete, delete).await;
            outside_rows(&mut b, rows, backend, repo_dir).await?;
        }
    }
    handle.shutdown().await.map_err(|e| e.to_string())
}

/// The outside-change snapshot (the outside-operations design):
/// what finding and recording one change made with plain `git` costs, and
/// what a ref-moving mutation costs once its own moves are adopted.
async fn outside_rows(
    b: &mut Bench,
    rows: &mut Vec<Row>,
    backend: &'static str,
    repo_dir: &Path,
) -> Result<(), String> {
    let git = |args: &[&str]| {
        let _ = Command::new("git")
            .args(args)
            .current_dir(repo_dir)
            .output();
    };
    std::env::set_var("GITPOULPI_GENERAL_JOURNAL_OUTSIDE", "true");
    // The first look is the baseline.
    b.call(OperationKind::JournalState, json!({})).await?;
    // The same read when nothing changed (the fingerprint says so once the
    // ref storage has been still for a few seconds; right after the baseline
    // it is a full snapshot read, which is what is measured).
    b.op(
        rows,
        backend,
        "outside snapshot read (nothing moved)",
        OperationKind::JournalState,
        json!({}),
        3,
    )
    .await?;
    git(&["branch", "perf-outside"]);
    b.op(
        rows,
        backend,
        "outside change recorded (1 ref)",
        OperationKind::JournalState,
        json!({}),
        1,
    )
    .await?;
    if let Some(at) = root_commit(repo_dir) {
        b.op(
            rows,
            backend,
            "branch create (journal + outside)",
            OperationKind::BranchCreate,
            json!({ "name": "perf-outside-2", "at": at }),
            1,
        )
        .await?;
    }
    // Undo both entries and clean up; the undo is ours, not a new entry.
    let _ = b.call(OperationKind::Undo, json!({})).await;
    let _ = b.call(OperationKind::Undo, json!({})).await;
    git(&["branch", "-D", "perf-outside"]);
    git(&["branch", "-D", "perf-outside-2"]);
    std::env::set_var("GITPOULPI_GENERAL_JOURNAL_OUTSIDE", "false");
    Ok(())
}

/// The auto-stash path (`general.auto_stash = always`): a checkout blocked
/// by 200 dirty files is stashed, run again and popped inside one request.
/// The stash push and pop are also timed alone, to show where the time goes.
/// Needs a branch that edits a file the working tree edits too.
async fn autostash_rows(
    b: &mut Bench,
    rows: &mut Vec<Row>,
    backend: &'static str,
    repo_dir: &Path,
    nf: usize,
) -> Result<(), String> {
    let target = "src/mod00/file0000.txt";
    let have = Command::new("git")
        .args(["branch", "--list", "perf-as"])
        .current_dir(repo_dir)
        .output()
        .map_err(|e| e.to_string())?;
    if have.stdout.is_empty() {
        git_in(repo_dir, &["checkout", "-q", "-b", "perf-as"])?;
        let path = repo_dir.join(target);
        let text = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
        std::fs::write(&path, text.replacen("line 0", "line 0 edited", 1))
            .map_err(|e| e.to_string())?;
        git_in(repo_dir, &["commit", "-qam", "perf-as edit"])?;
        git_in(repo_dir, &["checkout", "-q", "main"])?;
    }
    std::env::set_var("GITPOULPI_GENERAL_AUTO_STASH", "always");
    let (mut push, mut pop, mut co) = (Vec::new(), Vec::new(), Vec::new());
    let mut spawns = (0u64, 0u64, 0u64);
    for _ in 0..3 {
        reset_fixture(repo_dir);
        touch(repo_dir, 200, nf);
        let (s0, t0) = (total_spawn_count(), Instant::now());
        b.call(OperationKind::StashPush, json!({ "message": "perf" }))
            .await?;
        push.push(t0.elapsed().as_secs_f64() * 1000.0);
        spawns.0 = spawns.0.max(total_spawn_count() - s0);
        let (s0, t0) = (total_spawn_count(), Instant::now());
        b.call(OperationKind::StashPop, json!({ "id": 0 })).await?;
        pop.push(t0.elapsed().as_secs_f64() * 1000.0);
        spawns.1 = spawns.1.max(total_spawn_count() - s0);

        let (s0, t0) = (total_spawn_count(), Instant::now());
        b.call(
            OperationKind::Checkout,
            json!({ "target": { "Branch": "perf-as" } }),
        )
        .await?;
        co.push(t0.elapsed().as_secs_f64() * 1000.0);
        spawns.2 = spawns.2.max(total_spawn_count() - s0);
        reset_fixture(repo_dir);
        git_in(repo_dir, &["checkout", "-q", "main"])?;
    }
    std::env::remove_var("GITPOULPI_GENERAL_AUTO_STASH");
    for (name, mut t, sp) in [
        ("stash push (200 dirty)", push, spawns.0),
        ("stash pop (200 dirty)", pop, spawns.1),
        ("checkout with autostash", co, spawns.2),
    ] {
        rows.push(Row {
            backend,
            op: name.to_string(),
            ms: median(&mut t),
            spawns: sp,
        });
    }
    Ok(())
}

// ---- reporting -----------------------------------------------------------

/// Days since 1970-01-01 to a civil date (Howard Hinnant's algorithm), so
/// the report needs no date crate.
pub(crate) fn today() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let z = (secs / 86400) as i64 + 719468;
    let era = z.div_euclid(146097);
    let doe = z.rem_euclid(146097);
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    format!("{y:04}-{m:02}-{d:02}")
}

pub fn run(args: &[String]) -> Result<i32, String> {
    if args.iter().any(|a| a == "-h" || a == "--help") {
        println!("{USAGE}");
        return Ok(0);
    }
    if args.iter().any(|a| a == "--search") {
        let rest: Vec<String> = args.iter().filter(|a| *a != "--search").cloned().collect();
        return crate::perf_search::run(&rest);
    }
    if args.iter().any(|a| a == "--rewrite") {
        let rest: Vec<String> = args.iter().filter(|a| *a != "--rewrite").cloned().collect();
        return crate::perf_rewrite::run(&rest);
    }
    if args.iter().any(|a| a == "--tui") {
        let rest: Vec<String> = args.iter().filter(|a| *a != "--tui").cloned().collect();
        return crate::perf_tui::run(&rest);
    }
    let o = parse(args)?;
    let out_dir = workspace_root().join("target/perf");
    std::fs::create_dir_all(&out_dir).map_err(|e| e.to_string())?;
    // Belt and braces: nothing here may touch the maintainer's real config.
    std::env::set_var("GITPOULPI_CONFIG_DIR", out_dir.join("cfg"));
    // The rows below are the plain operations' baselines: the outside-change
    // snapshot has its own rows (it is switched on for them only).
    std::env::set_var("GITPOULPI_GENERAL_JOURNAL_OUTSIDE", "false");

    let (repo_dir, mutate) = match &o.repo {
        Some(p) => (p.clone(), false),
        None => {
            if o.rebuild || !fixture_ready(&o.fixture, o.files, o.commits) {
                eprintln!(
                    "building {}x{} fixture in {} ...",
                    o.files,
                    o.commits,
                    o.fixture.display()
                );
                build_fixture(&o.fixture, o.files, o.commits)?;
            }
            reset_fixture(&o.fixture);
            (o.fixture.clone(), true)
        }
    };
    let floor = spawn_floor_ms();
    println!("spawn floor (git --version median): {floor:.1} ms\n");

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|e| e.to_string())?;
    let mut rows = Vec::new();
    for backend in &o.backends {
        rt.block_on(run_backend(backend, &repo_dir, mutate, o.files, &mut rows))?;
    }

    let previous = latest_previous(&out_dir);
    println!(
        "{:<7} {:<24} {:>9} {:>7}  budget",
        "backend", "operation", "ms", "spawns"
    );
    let mut failures = Vec::new();
    for r in &rows {
        let mut note = String::new();
        if let Some((max_sp, mult, slack)) = budget(r.backend, &r.op) {
            let max_ms = mult * floor + slack;
            note = format!("<= {max_sp} spawns, <= {max_ms:.0} ms");
            if r.spawns > max_sp {
                failures.push(format!(
                    "{} {}: {} spawns > {max_sp}",
                    r.backend, r.op, r.spawns
                ));
            }
            if r.ms > max_ms {
                failures.push(format!(
                    "{} {}: {:.0} ms > {max_ms:.0} ms",
                    r.backend, r.op, r.ms
                ));
            }
        }
        if let Some(prev) = previous.get(&format!("{}/{}", r.backend, r.op)) {
            note.push_str(&format!("  (prev {prev:.1} ms)"));
        }
        println!(
            "{:<7} {:<24} {:>9.1} {:>7}  {note}",
            r.backend, r.op, r.ms, r.spawns
        );
    }

    let report = json!({
        "date": today(),
        "spawn_floor_ms": floor,
        "ops": rows.iter().map(|r| json!({
            "backend": r.backend, "op": r.op, "ms": r.ms, "spawns": r.spawns
        })).collect::<Vec<_>>(),
    });
    let file = out_dir.join(format!("{}.json", today()));
    std::fs::write(&file, serde_json::to_string_pretty(&report).unwrap())
        .map_err(|e| e.to_string())?;
    println!("\nwrote {}", file.display());

    if failures.is_empty() {
        println!("perf budgets: ok");
        Ok(0)
    } else if !o.budgets {
        println!(
            "perf budgets: {} over (ignored, --no-budget)",
            failures.len()
        );
        Ok(0)
    } else {
        eprintln!("\nperf budgets exceeded:");
        for f in &failures {
            eprintln!("  {f}");
        }
        Ok(1)
    }
}

/// `backend/op -> ms` from the newest earlier report, to print a diff
/// column (a report from today is replaced, so it counts as the previous).
fn latest_previous(dir: &Path) -> BTreeMap<String, f64> {
    let mut files: Vec<PathBuf> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == "json"))
        .collect();
    files.sort();
    let mut map = BTreeMap::new();
    if let Some(v) = files
        .last()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .and_then(|s| serde_json::from_str::<Value>(&s).ok())
    {
        for r in v["ops"].as_array().into_iter().flatten() {
            if let (Some(b), Some(o), Some(ms)) =
                (r["backend"].as_str(), r["op"].as_str(), r["ms"].as_f64())
            {
                map.insert(format!("{b}/{o}"), ms);
            }
        }
    }
    map
}
