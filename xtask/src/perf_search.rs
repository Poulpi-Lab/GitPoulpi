//! `cargo run --release -p xtask -- perf --search`: the workspace search's
//! warm-query budget (docs/developer/perf-budgets.md, the workspaces design).
//!
//! A 50,000-commit repository (built with one `git fast-import`, outside the
//! temp dir) cloned locally into `--repos` repositories (hard-linked
//! objects, so the copies are cheap), each searched through an in-process
//! daemon over `gitpoulpi-ipc`, the way the window and the CLI do. Measures,
//! per backend:
//!
//! - the **cold** build of each repository's index (one `Search` that waits);
//! - the **warm** query: median of several queries over each repository, and
//!   the sum over all repositories (what one keystroke of a workspace search
//!   costs when it asks them one after another);
//! - one **incremental** update after a new commit.
//!
//! Budget (the maintainer's ask): the warm query over 10 repositories of 50k
//! commits under 150 ms, so under 15 ms per repository. Release builds only:
//! a dev build is several times slower and would be a false alarm.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::time::Instant;

use gitpoulpi_cli_backend::SystemGitBackend;
use gitpoulpi_core::domain::{OperationKind, RepoId, SearchHits, SearchRequest};
use gitpoulpi_core::GitBackend;
use gitpoulpi_daemon::Daemon;
use gitpoulpi_git2_backend::Git2Backend;
use gitpoulpi_ipc::{Client, Request};
use serde_json::json;

/// Warm query over one repository, milliseconds.
/// The status bar's commit total over the whole 50k history, per backend.
const BUDGET_COUNT_MS: f64 = 60.0;
/// The in-memory merge of a branch 100 commits back against `main`.
const BUDGET_CONFLICT_MS: f64 = 150.0;
const BUDGET_PER_REPO_MS: f64 = 15.0;
/// Warm query over every repository, milliseconds.
const BUDGET_TOTAL_MS: f64 = 150.0;

pub const USAGE: &str = "\
Usage: cargo run --release -p xtask -- perf --search [options]

  --commits <n>    history per repository, default 50000
  --repos <n>      repositories searched, default 10
  --backend <b>    git2 | system | both (default both)
  --no-budget      print, do not fail on a regression
  --rebuild        regenerate the fixture";

fn git(dir: &Path, args: &[&str]) -> Result<(), String> {
    let out = Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .map_err(|e| e.to_string())?;
    if out.status.success() {
        Ok(())
    } else {
        Err(format!(
            "git {args:?}: {}",
            String::from_utf8_lossy(&out.stderr)
        ))
    }
}

/// `commits` commits on `main` in one fast-import, a few with bodies, plus a
/// hundred branches and tags so the small items are not empty.
fn build(dir: &Path, commits: usize) -> Result<(), String> {
    let _ = std::fs::remove_dir_all(dir);
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    git(dir, &["init", "-q", "-b", "main"])?;
    let areas = [
        "parser", "render", "daemon", "ipc", "session", "config", "log",
    ];
    let verbs = [
        "fix", "add", "refactor", "speed up", "document", "remove", "rename",
    ];
    let mut stream = String::with_capacity(commits * 160);
    for n in 1..=commits {
        let msg = format!(
            "{}({}): {} the {} handling for case {n}",
            verbs[n % verbs.len()],
            areas[(n / 7) % areas.len()],
            verbs[(n / 3) % verbs.len()],
            areas[(n / 11) % areas.len()],
        );
        let msg = if n % 5 == 0 {
            format!("{msg}\n\nLonger explanation of why change {n} was needed.\nSecond line of the body.\n")
        } else {
            format!("{msg}\n")
        };
        stream.push_str(&format!(
            "commit refs/heads/main\nmark :{n}\ncommitter Perf Author {} <perf{}@example.com> {} +0000\ndata {}\n{msg}",
            n % 40,
            n % 40,
            1_500_000_000 + n * 60,
            msg.len()
        ));
        if n > 1 {
            stream.push_str(&format!("from :{}\n", n - 1));
        }
        // One tiny file per commit keeps the trees real without bloating them.
        let body = format!("{n}\n");
        stream.push_str(&format!(
            "M 100644 inline f.txt\ndata {}\n{body}\n",
            body.len()
        ));
    }
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
        .write_all(stream.as_bytes())
        .map_err(|e| e.to_string())?;
    if !child.wait().map_err(|e| e.to_string())?.success() {
        return Err("git fast-import failed".into());
    }
    git(dir, &["reset", "-q", "--hard", "main"])?;
    for i in 0..100 {
        git(
            dir,
            &[
                "branch",
                &format!("topic/area-{i}"),
                &format!("main~{}", i * 7),
            ],
        )?;
        if i % 4 == 0 {
            git(
                dir,
                &["tag", &format!("v{i}.0"), &format!("main~{}", i * 13)],
            )?;
        }
    }
    Ok(())
}

fn median(v: &mut [f64]) -> f64 {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v[v.len() / 2]
}

struct Bench {
    client: Client,
    next_id: u64,
}

impl Bench {
    async fn search(
        &mut self,
        repo: &RepoId,
        query: &str,
        wait: bool,
    ) -> Result<SearchHits, String> {
        self.next_id += 1;
        let resp = self
            .client
            .call(&Request {
                id: self.next_id,
                session: "perf-search".into(),
                repo: Some(repo.clone()),
                command: OperationKind::Search,
                args: json!(SearchRequest {
                    query: query.into(),
                    kinds: Vec::new(),
                    limit: 50,
                    wait,
                }),
            })
            .await
            .map_err(|e| e.to_string())?;
        let value = resp.result.map_err(|e| format!("Search: {e:?}"))?;
        serde_json::from_value(value).map_err(|e| e.to_string())
    }
}

pub fn run(args: &[String]) -> Result<i32, String> {
    if args.iter().any(|a| a == "-h" || a == "--help") {
        println!("{USAGE}");
        return Ok(0);
    }
    let value = |flag: &str| {
        args.iter()
            .position(|a| a == flag)
            .and_then(|i| args.get(i + 1))
    };
    let commits: usize = value("--commits")
        .map(|v| v.parse().map_err(|_| format!("bad --commits {v}")))
        .transpose()?
        .unwrap_or(50_000);
    let repos: usize = value("--repos")
        .map(|v| v.parse().map_err(|_| format!("bad --repos {v}")))
        .transpose()?
        .unwrap_or(10);
    let backends: Vec<&'static str> = match value("--backend").map(String::as_str) {
        Some("git2") => vec!["git2"],
        Some("system") => vec!["system"],
        _ => vec!["git2", "system"],
    };
    let enforce = !args.iter().any(|a| a == "--no-budget");
    if cfg!(debug_assertions) {
        eprintln!("note: a debug build is several times slower; measure with --release");
    }

    // `GITPOULPI_PERF_DIR` moves the fixtures (a few hundred MB) off a full disk.
    let root = std::env::var_os("GITPOULPI_PERF_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| crate::perf::workspace_root().join("target/perf"));
    std::env::set_var("GITPOULPI_CONFIG_DIR", root.join("cfg"));
    let base = root.join(format!("search-{commits}"));
    if args.iter().any(|a| a == "--rebuild") || !base.join(".git").exists() {
        eprintln!("building {commits} commits in {} ...", base.display());
        let t0 = Instant::now();
        build(&base, commits)?;
        eprintln!("built in {:.1} s", t0.elapsed().as_secs_f64());
    }
    // The copies: local clones share the objects by hard links.
    let mut dirs: Vec<PathBuf> = vec![base.clone()];
    for i in 1..repos {
        let dir = root.join(format!("search-{commits}-copy{i}"));
        if !dir.join(".git").exists() {
            let status = Command::new("git")
                .args(["clone", "-q", "--local", "--no-checkout"])
                .arg(&base)
                .arg(&dir)
                .status()
                .map_err(|e| e.to_string())?;
            if !status.success() {
                return Err(format!("could not clone into {}", dir.display()));
            }
        }
        dirs.push(dir);
    }
    dirs.truncate(repos);

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|e| e.to_string())?;
    let mut failures = Vec::new();
    println!(
        "{:<7} {:<34} {:>10}  budget",
        "backend", "measurement", "ms"
    );
    for backend in backends {
        let be: Arc<dyn GitBackend> = match backend {
            "git2" => Arc::new(Git2Backend::new()),
            _ => Arc::new(SystemGitBackend::new()),
        };
        rt.block_on(async {
            let handle = Daemon::new(format!("perf-search-{backend}"))
                .spawn(be.clone())
                .await
                .map_err(|e| e.to_string())?;
            let mut b = Bench {
                client: Client::connect(&handle.endpoint())
                    .await
                    .map_err(|e| e.to_string())?,
                next_id: 0,
            };
            let ids: Vec<RepoId> = dirs
                .iter()
                .map(|d| be.discover(d).map(|f| f.repo.id).map_err(|e| e.to_string()))
                .collect::<Result<_, _>>()?;

            // Cold: the first search of each repository builds its index.
            let mut cold = Vec::new();
            for id in &ids {
                let t0 = Instant::now();
                let hits = b.search(id, "parser", true).await?;
                cold.push(t0.elapsed().as_secs_f64() * 1000.0);
                if hits.partial || hits.hits.is_empty() {
                    return Err(format!("cold search of {} answered {hits:?}", id.0));
                }
            }
            println!("{backend:<7} {:<34} {:>10.0}", "cold index build (median)", median(&mut cold));

            // The status bar's commit total on the same history: the walk
            // counted without loading a commit (`GitBackend::count_commits`),
            // straight on the backend (no daemon cache), median of three.
            {
                let repo_ref = be.discover(&dirs[0]).map_err(|e| e.to_string())?.repo;
                let opts = gitpoulpi_core::domain::LogOptions {
                    all_refs: true,
                    ..Default::default()
                };
                let mut counts = Vec::new();
                let mut total = 0;
                for _ in 0..3 {
                    let t0 = Instant::now();
                    total = be.count_commits(&repo_ref, &opts).map_err(|e| e.to_string())?;
                    counts.push(t0.elapsed().as_secs_f64() * 1000.0);
                }
                let m = median(&mut counts);
                let note = format!("<= {BUDGET_COUNT_MS:.0} ms");
                println!("{backend:<7} {:<34} {m:>10.1}  {note}", format!("commit total ({total} commits)"));
                if m > BUDGET_COUNT_MS {
                    failures.push(format!("{backend}: commit total {m:.1} ms > {BUDGET_COUNT_MS} ms"));
                }
                // `log.commit_count = "libgit2"`: the in-process revwalk, no
                // process. Reported, not budgeted: it exists for machines
                // where a spawn costs far more than the extra milliseconds.
                let mut in_process = Vec::new();
                for _ in 0..3 {
                    let t0 = Instant::now();
                    gitpoulpi_git2_backend::count_commits_in_process(&repo_ref, &opts)
                        .map_err(|e| e.to_string())?;
                    in_process.push(t0.elapsed().as_secs_f64() * 1000.0);
                }
                println!(
                    "{backend:<7} {:<34} {:>10.1}  reported",
                    "commit total (libgit2, no spawn)",
                    median(&mut in_process)
                );
            }

            // Proactive conflict detection (`predict_merge_conflicts`, the
            // daemon's `PredictConflicts`) on the same history: a branch
            // forked 100 commits back that edits `f.txt`, which every later
            // commit edits too, against `main`. Its own copy of the
            // repository, so the commits above stay untouched.
            {
                let copy = root.join(format!("search-{commits}-conflict-{backend}"));
                if copy.exists() {
                    std::fs::remove_dir_all(&copy).map_err(|e| e.to_string())?;
                }
                let status = Command::new("git")
                    .args(["clone", "-q", "--local"])
                    .arg(&base)
                    .arg(&copy)
                    .status()
                    .map_err(|e| e.to_string())?;
                if !status.success() {
                    return Err(format!("could not clone into {}", copy.display()));
                }
                git(&copy, &["checkout", "-q", "-b", "perf-feature", "main~100"])?;
                std::fs::write(copy.join("f.txt"), "feature\n").map_err(|e| e.to_string())?;
                git(&copy, &["-c", "user.name=P", "-c", "user.email=p@example.com", "commit", "-q", "-am", "feature"])?;
                let data = copy.join(".git").join("gitpoulpi");
                std::fs::create_dir_all(&data).map_err(|e| e.to_string())?;
                std::fs::write(
                    data.join("repo-state.toml"),
                    "[preferences]\nconflict_base_branches = \"main\"\n",
                )
                .map_err(|e| e.to_string())?;
                let repo_ref = be.discover(&copy).map_err(|e| e.to_string())?.repo;
                let head = be.resolve(&repo_ref, "HEAD").map_err(|e| e.to_string())?;
                let main = be.resolve(&repo_ref, "main").map_err(|e| e.to_string())?;
                let mut merges = Vec::new();
                for _ in 0..3 {
                    let t0 = Instant::now();
                    let files = be
                        .predict_merge_conflicts(&repo_ref, head, main)
                        .map_err(|e| e.to_string())?;
                    merges.push(t0.elapsed().as_secs_f64() * 1000.0);
                    if files != ["f.txt"] {
                        return Err(format!("the perf fixture predicted {files:?}"));
                    }
                }
                let m = median(&mut merges);
                let note = format!("<= {BUDGET_CONFLICT_MS:.0} ms");
                println!("{backend:<7} {:<34} {m:>10.1}  {note}", "conflict check: the merge");
                if m > BUDGET_CONFLICT_MS {
                    failures.push(format!("{backend}: conflict check merge {m:.1} ms > {BUDGET_CONFLICT_MS} ms"));
                }
                // The daemon's request: base resolution + the merge (cold),
                // then the answer cached by (head, base) (warm).
                let id = repo_ref.id.clone();
                let ask = |n: u64| Request {
                    id: n,
                    session: "perf-search".into(),
                    repo: Some(id.clone()),
                    command: OperationKind::PredictConflicts,
                    args: json!({}),
                };
                let t0 = Instant::now();
                let first = b.client.call(&ask(9_001)).await.map_err(|e| e.to_string())?;
                let cold = t0.elapsed().as_secs_f64() * 1000.0;
                first.result.map_err(|e| format!("PredictConflicts: {e:?}"))?;
                let mut warm = Vec::new();
                for n in 0..3 {
                    let t0 = Instant::now();
                    b.client.call(&ask(9_002 + n)).await.map_err(|e| e.to_string())?;
                    warm.push(t0.elapsed().as_secs_f64() * 1000.0);
                }
                println!("{backend:<7} {:<34} {cold:>10.1}  reported", "conflict check: request (cold)");
                println!("{backend:<7} {:<34} {:>10.1}  reported", "conflict check: request (cached)", median(&mut warm));
                // The default Base branches (`**`): all 100 topic branches and
                // `main` are measured for the nearest fork on every request.
                std::fs::remove_file(data.join("repo-state.toml")).map_err(|e| e.to_string())?;
                let mut every = Vec::new();
                for n in 0..4 {
                    let t0 = Instant::now();
                    b.client.call(&ask(9_100 + n)).await.map_err(|e| e.to_string())?;
                    every.push(t0.elapsed().as_secs_f64() * 1000.0);
                }
                let first_every = every.remove(0);
                println!("{backend:<7} {:<34} {first_every:>10.1}  reported", "conflict check: ** x101 (new HEAD)");
                println!("{backend:<7} {:<34} {:>10.1}  reported", "conflict check: ** x101 (memoized)", median(&mut every));
            }

            // Warm: several shapes of query, a few times each.
            let queries = ["parser", "fix(", "c: case 4999", "b: area", "rfctr dmn", "zzzz"];
            let mut per_repo = Vec::new();
            let mut worst = 0.0f64;
            for id in &ids {
                let mut samples = Vec::new();
                for _ in 0..3 {
                    for q in queries {
                        let t0 = Instant::now();
                        let hits = b.search(id, q, false).await?;
                        samples.push(t0.elapsed().as_secs_f64() * 1000.0);
                        if hits.partial {
                            return Err("a warm search said partial".into());
                        }
                    }
                }
                let m = median(&mut samples);
                worst = worst.max(samples.iter().cloned().fold(0.0, f64::max));
                per_repo.push(m);
            }
            let typical = median(&mut per_repo.clone());
            let total: f64 = per_repo.iter().sum();
            let per = format!("<= {BUDGET_PER_REPO_MS:.0} ms");
            println!("{backend:<7} {:<34} {typical:>10.1}  {per}", "warm query, one repository (median)");
            println!("{backend:<7} {:<34} {worst:>10.1}", "warm query, slowest sample");
            let all = format!("<= {BUDGET_TOTAL_MS:.0} ms");
            println!("{backend:<7} {:<34} {total:>10.1}  {all}", format!("warm query, {} repositories", ids.len()));
            if typical > BUDGET_PER_REPO_MS {
                failures.push(format!("{backend}: warm query {typical:.1} ms > {BUDGET_PER_REPO_MS} ms per repository"));
            }
            if total > BUDGET_TOTAL_MS * ids.len() as f64 / 10.0 {
                failures.push(format!("{backend}: warm query over {} repositories {total:.1} ms > budget", ids.len()));
            }

            // Incremental: one new commit, then the next query.
            git(&dirs[0], &["commit", "-q", "--allow-empty", "-m", "perf: one more commit"])?;
            tokio::time::sleep(std::time::Duration::from_millis(1100)).await;
            let t0 = Instant::now();
            let hits = b.search(&ids[0], "c: one more commit", true).await?;
            let ms = t0.elapsed().as_secs_f64() * 1000.0;
            if hits.hits.is_empty() {
                return Err("the new commit was not found".into());
            }
            println!("{backend:<7} {:<34} {ms:>10.1}", "incremental update (1 new commit)");
            git(&dirs[0], &["reset", "-q", "--hard", "HEAD~1"])?;
            handle.shutdown().await.map_err(|e| e.to_string())?;
            Ok::<(), String>(())
        })?;
    }
    if failures.is_empty() {
        println!("\nsearch budgets: ok");
        Ok(0)
    } else if !enforce {
        println!(
            "\nsearch budgets: {} over (ignored, --no-budget)",
            failures.len()
        );
        Ok(0)
    } else {
        eprintln!("\nsearch budgets exceeded:");
        for f in &failures {
            eprintln!("  {f}");
        }
        Ok(1)
    }
}
