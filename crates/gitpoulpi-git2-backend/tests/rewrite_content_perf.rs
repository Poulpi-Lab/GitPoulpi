//! The content engine's timing against the plan's targets
//! (the history-rewrite design, "Performance targets";
//! `docs/developer/perf-budgets.md`). Ignored: run it with
//! `cargo run --release -p xtask -- perf --rewrite [--commits N]`, which
//! builds in release mode and sets the size.
//!
//! The fixture is `import_content_history`: a linear history whose every
//! commit adds two text files holding a fake token (so twice as many unique
//! text blobs as commits) and rewrites a key file, plus 20 branches and 20
//! tags. The plan's table is for 10 000 and 100 000 commits; other sizes are
//! interpolated linearly from the 10 000 column.

use std::ops::ControlFlow;
use std::time::{Duration, Instant};

use gitpoulpi_cli_backend::SystemGitBackend;
use gitpoulpi_core::domain::rewrite::{
    ContentFind, ContentReplace, Recipe, RewritePlan, Rule, RuleKind, Scope, SignaturePolicy,
    TextPattern,
};
use gitpoulpi_core::GitBackend;
use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};

fn commits() -> usize {
    std::env::var("GITPOULPI_REWRITE_PERF_COMMITS")
        .ok()
        .and_then(|v| v.trim().parse().ok())
        .unwrap_or(10_000)
}

/// `at_10k` seconds for 10 000 commits, `at_100k` for 100 000, linear
/// between and beyond.
fn budget(commits: usize, at_10k: f64, at_100k: f64) -> Duration {
    let c = commits as f64;
    let secs = if c <= 10_000.0 {
        at_10k * c / 10_000.0
    } else {
        at_10k + (at_100k - at_10k) * (c - 10_000.0) / 90_000.0
    };
    Duration::from_secs_f64(secs)
}

fn recipe(rule: RuleKind) -> Recipe {
    let mut recipe = Recipe::new("perf", Scope::AllLocal, vec![Rule::new(rule)]);
    recipe.options.signatures = Some(SignaturePolicy::Strip);
    recipe
}

struct Timed {
    plan: RewritePlan,
    preview: Duration,
    apply: Duration,
}

fn run(backend: &dyn GitBackend, repo: &FixtureRepo, recipe: &Recipe) -> Timed {
    let r = repo.repo_ref();
    let started = Instant::now();
    let plan = backend
        .rewrite_preview(&r, recipe, &mut |_| ControlFlow::Continue(()))
        .expect("preview");
    let preview = started.elapsed();
    let started = Instant::now();
    backend.rewrite_apply(&r, &plan).expect("apply");
    Timed {
        plan,
        preview,
        apply: started.elapsed(),
    }
}

#[test]
#[ignore = "timing, run by `xtask perf --rewrite`"]
fn content_rewrite_timing() {
    let n = commits();
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .with_initial()
        .unwrap()
        .build();
    let started = Instant::now();
    gitpoulpi_testkit::contract::import_content_history(&repo, n);
    println!("fixture of {n} commits built in {:?}", started.elapsed());
    let backend = SystemGitBackend::new();

    // Scan for secrets: every unique blob read once, nothing written. Run
    // first, while all the history is there. The budget is the replace-text
    // one (the same blobs are read; nothing is written to a pack).
    let started = Instant::now();
    let scan = backend
        .rewrite_scan_secrets(&repo.repo_ref(), &Scope::AllLocal, &mut |_| {
            ControlFlow::Continue(())
        })
        .expect("scan");
    let scanned = started.elapsed();
    println!(
        "{n} commits, scan for secrets: {scanned:?} ({} blobs read, {} finding(s))",
        scan.blobs_scanned,
        scan.findings.len()
    );
    assert!(scan.blobs_scanned as usize >= n);
    let allowed = budget(n, 20.0, 240.0);
    assert!(
        scanned <= allowed,
        "scan for secrets took {scanned:?}, budget {allowed:?}"
    );

    // Size analysis: one `log --raw` pass and one `cat-file --batch-check`,
    // no blob is read.
    let started = Instant::now();
    let analysis = backend
        .rewrite_analyze(&repo.repo_ref(), &Scope::AllLocal, 50, &mut |_| {
            ControlFlow::Continue(())
        })
        .expect("analyze");
    let analysed = started.elapsed();
    println!(
        "{n} commits, size analysis: {analysed:?} ({} blobs, {} folder(s) listed)",
        analysis.blobs_total,
        analysis.folders.len()
    );
    assert!(analysis.blobs_total as usize >= n);
    let allowed = budget(n, 20.0, 240.0);
    assert!(
        analysed <= allowed,
        "size analysis took {analysed:?}, budget {allowed:?}"
    );

    // Remove a path: no blob is read.
    let remove = run(
        &backend,
        &repo,
        &recipe(RuleKind::RemovePaths {
            globs: vec!["secrets/".into()],
        }),
    );
    println!(
        "{n} commits, remove secrets/: preview {:?}, apply {:?} ({} changed)",
        remove.preview, remove.apply, remove.plan.counts.commits_changed
    );
    assert_eq!(remove.plan.counts.commits_changed as usize, n);
    let allowed = budget(n, 3.0, 30.0);
    assert!(
        remove.preview <= allowed,
        "remove a path: preview took {:?}, budget {allowed:?}",
        remove.preview
    );

    // Replace text over twice as many unique text blobs as commits.
    let replace = run(
        &backend,
        &repo,
        &recipe(RuleKind::ContentReplace(ContentReplace {
            paths: Vec::new(),
            find: ContentFind::Pattern {
                pattern: TextPattern::literal("EXAMPLE_TOKEN_123"),
            },
            with: "REDACTED".into(),
            max_blob_bytes: None,
        })),
    );
    println!(
        "{n} commits, replace text: preview {:?}, apply {:?} ({} blobs changed)",
        replace.preview, replace.apply, replace.plan.counts.blobs_changed
    );
    assert!(replace.plan.counts.blobs_changed as usize >= n);
    let allowed = budget(n, 20.0, 240.0);
    assert!(
        replace.preview <= allowed,
        "replace text: preview took {:?}, budget {allowed:?}",
        replace.preview
    );
}
