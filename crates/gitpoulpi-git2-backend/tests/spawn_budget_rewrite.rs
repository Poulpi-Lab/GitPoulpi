//! A history rewrite spawns a fixed number of `git` processes whatever the
//! size of the history (the history-rewrite design, "Performance
//! targets": at most 8 per run), on both backends. A spawn costs 10-40 ms on
//! Windows, so the count is the deterministic proxy for latency.

use gitpoulpi_cli_backend::{total_spawn_count, SystemGitBackend};
use gitpoulpi_core::domain::rewrite::{
    ContentFind, ContentReplace, IdentityMatch, IdentityRule, IdentitySet, Recipe, Rule, RuleKind,
    Scope, SignaturePolicy, TextPattern, Who,
};
use gitpoulpi_core::GitBackend;
use gitpoulpi_git2_backend::Git2Backend;
use gitpoulpi_testkit::{FixtureRepoBuilder, SpawnBudget};

#[test]
fn a_rewrite_of_a_thousand_commits_spawns_at_most_eight_git_processes() {
    let sb = SpawnBudget::serial(total_spawn_count);
    for backend in [
        &Git2Backend::new() as &dyn GitBackend,
        &SystemGitBackend::new() as &dyn GitBackend,
    ] {
        let repo = FixtureRepoBuilder::new()
            .unwrap()
            .with_initial()
            .unwrap()
            .build();
        gitpoulpi_testkit::contract::import_linear_history(&repo, 1000);
        let r = repo.repo_ref();
        let mut recipe = Recipe::new(
            "alice to carol",
            Scope::default(),
            vec![Rule::new(RuleKind::Identity(IdentityRule {
                when: IdentityMatch::email("alice@example.com"),
                set: IdentitySet {
                    name: Some("Carol Example".into()),
                    email: Some("carol@example.com".into()),
                },
                apply_to: Who::Both,
                include_taggers: false,
                map_trailers: false,
            }))],
        );
        recipe.options.signatures = Some(SignaturePolicy::Strip);
        let ((plan, outcome), spawns) = sb.measure(|| {
            let plan = backend
                .rewrite_preview(&r, &recipe, &mut |_| std::ops::ControlFlow::Continue(()))
                .unwrap();
            let outcome = backend.rewrite_apply(&r, &plan).unwrap();
            (plan, outcome)
        });
        assert!(
            spawns <= 8,
            "preview and apply of 1000 commits: {spawns} git spawns, budget 8"
        );
        // Alice wrote the first of the 1000: all of them are re-created.
        assert_eq!(plan.counts.commits_changed, 1000);
        assert_eq!(outcome.commits_changed, 1000);
        assert_eq!(
            repo.git(&["log", "-1", "--format=%an", "main"]),
            "Bob Example"
        );
        repo.git(&["fsck", "--connectivity-only", "--no-dangling"]);
        println!("spawns for preview + apply: {spawns}");
    }
}

/// `cargo test --release -p gitpoulpi-git2-backend --test spawn_budget_rewrite
/// -- --ignored --nocapture`: the plan's targets for 10 000 commits (preview
/// under 1.5 s, apply under 1 s on the reference machine).
#[test]
#[ignore = "timing, run by hand"]
fn rewrite_timing_of_ten_thousand_commits() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .with_initial()
        .unwrap()
        .build();
    gitpoulpi_testkit::contract::import_linear_history(&repo, 10_000);
    let r = repo.repo_ref();
    let mut recipe = Recipe::new(
        "alice to carol",
        Scope::default(),
        vec![Rule::new(RuleKind::Identity(IdentityRule {
            when: IdentityMatch::email("alice@example.com"),
            set: IdentitySet {
                name: Some("Carol Example".into()),
                email: Some("carol@example.com".into()),
            },
            apply_to: Who::Both,
            include_taggers: false,
            map_trailers: false,
        }))],
    );
    recipe.options.signatures = Some(SignaturePolicy::Strip);
    let backend = SystemGitBackend::new();
    let started = std::time::Instant::now();
    let plan = backend
        .rewrite_preview(&r, &recipe, &mut |_| std::ops::ControlFlow::Continue(()))
        .unwrap();
    let preview = started.elapsed();
    let started = std::time::Instant::now();
    backend.rewrite_apply(&r, &plan).unwrap();
    println!(
        "10000 commits: preview {preview:?}, apply {:?}",
        started.elapsed()
    );
}

/// Branches, tags, notes, a remote, a protected base and a stash do not cost
/// more processes: S4 reads every ref with one `for-each-ref`, tells what is
/// pushed with one `rev-list`, and reuses the one `cat-file` for tags, notes
/// and stashes. Still at most 8 for preview and apply together.
#[test]
fn a_multi_ref_rewrite_with_a_remote_spawns_at_most_eight_git_processes() {
    let sb = SpawnBudget::serial(total_spawn_count);
    for backend in [
        &Git2Backend::new() as &dyn GitBackend,
        &SystemGitBackend::new() as &dyn GitBackend,
    ] {
        let repo = FixtureRepoBuilder::new()
            .unwrap()
            .with_initial()
            .unwrap()
            .with_origin()
            .unwrap()
            .build();
        gitpoulpi_testkit::contract::import_linear_history(&repo, 1000);
        repo.git(&["push", "origin", "main"]);
        repo.git(&["branch", "feat", "main~10"]);
        repo.git(&["branch", "old", "main~900"]);
        repo.git(&["tag", "v1", "main~5"]);
        repo.git(&["tag", "-a", "v2", "-m", "second", "main~500"]);
        repo.git(&["notes", "add", "-m", "note", "main~7"]);
        let r = repo.repo_ref();
        let mut recipe = Recipe::new(
            "alice to carol",
            Scope::AllLocal,
            vec![Rule::new(RuleKind::Identity(IdentityRule {
                when: IdentityMatch::email("alice@example.com"),
                set: IdentitySet {
                    name: Some("Carol Example".into()),
                    email: Some("carol@example.com".into()),
                },
                apply_to: Who::Both,
                include_taggers: false,
                map_trailers: false,
            }))],
        );
        recipe.options.signatures = Some(SignaturePolicy::Strip);
        recipe.scope_options.include_protected = vec!["main".into()];
        let ((plan, _), spawns) = sb.measure(|| {
            let plan = backend
                .rewrite_preview(&r, &recipe, &mut |_| std::ops::ControlFlow::Continue(()))
                .unwrap();
            let outcome = backend.rewrite_apply(&r, &plan).unwrap();
            (plan, outcome)
        });
        assert!(
            spawns <= 8,
            "multi-ref preview and apply: {spawns} git spawns, budget 8"
        );
        // main, feat, old, v1, v2 and the notes move.
        assert_eq!(plan.refs.len(), 6, "{:?}", plan.refs);
        repo.git(&["fsck", "--connectivity-only", "--no-dangling"]);
        println!("spawns for a multi-ref preview + apply: {spawns}");
    }
}

fn content_recipe(rules: Vec<RuleKind>) -> Recipe {
    let mut recipe = Recipe::new(
        "content",
        Scope::AllLocal,
        rules.into_iter().map(Rule::new).collect(),
    );
    recipe.options.signatures = Some(SignaturePolicy::Strip);
    recipe
}

fn replace_token() -> RuleKind {
    RuleKind::ContentReplace(ContentReplace {
        paths: Vec::new(),
        find: ContentFind::Pattern {
            pattern: TextPattern::literal("EXAMPLE_TOKEN_123"),
        },
        with: "REDACTED".into(),
        max_blob_bytes: None,
    })
}

/// A content run spends the metadata run's eight plus two: one `git status`
/// to find the dirty files the rewrite would change, and one `git read-tree`
/// to bring the working tree along. Still independent of the history size.
#[test]
fn a_content_rewrite_of_a_thousand_commits_spawns_at_most_ten_git_processes() {
    let sb = SpawnBudget::serial(total_spawn_count);
    for backend in [
        &Git2Backend::new() as &dyn GitBackend,
        &SystemGitBackend::new() as &dyn GitBackend,
    ] {
        for (label, rules) in [
            ("replace text", vec![replace_token()]),
            (
                "remove a folder",
                vec![RuleKind::RemovePaths {
                    globs: vec!["secrets/".into()],
                }],
            ),
            (
                "replace, remove and drop big blobs",
                vec![
                    replace_token(),
                    RuleKind::RemovePaths {
                        globs: vec!["docs/".into()],
                    },
                    RuleKind::RemoveBlobsOver { bytes: 10_000 },
                ],
            ),
        ] {
            let repo = FixtureRepoBuilder::new()
                .unwrap()
                .with_initial()
                .unwrap()
                .build();
            gitpoulpi_testkit::contract::import_content_history(&repo, 1000);
            let r = repo.repo_ref();
            let recipe = content_recipe(rules);
            let ((plan, _), spawns) = sb.measure(|| {
                let plan = backend
                    .rewrite_preview(&r, &recipe, &mut |_| std::ops::ControlFlow::Continue(()))
                    .unwrap();
                let outcome = backend.rewrite_apply(&r, &plan).unwrap();
                (plan, outcome)
            });
            assert!(
                spawns <= 10,
                "{label}: preview and apply of 1000 commits: {spawns} git spawns, budget 10"
            );
            assert!(plan.counts.commits_changed >= 1000, "{label}");
            repo.git(&["fsck", "--connectivity-only", "--no-dangling"]);
            println!("spawns for a content preview + apply ({label}): {spawns}");
        }
    }
}
