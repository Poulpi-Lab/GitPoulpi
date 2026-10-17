//! `gitpoulpi rewrite` path rules (S20), end to end through the real binary,
//! the daemon and the backend: rename, into a subdirectory, extract in the
//! three modes. Assertions are on deserialized JSON, exit codes and plain
//! `git` state.
#![cfg(feature = "history-rewrite")]

use std::path::Path;

use assert_cmd::Command;
use gitpoulpi_core::domain::rewrite::RewriteOutcome;
use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use serde_json::Value;

fn config_dir() -> &'static Path {
    static DIR: std::sync::OnceLock<tempfile::TempDir> = std::sync::OnceLock::new();
    DIR.get_or_init(|| tempfile::tempdir().unwrap()).path()
}

/// Runs the CLI; returns (exit code, parsed stdout).
fn run(repo: &FixtureRepo, session: &str, args: &[&str]) -> (i32, Value) {
    let mut command = Command::cargo_bin("gitpoulpi").unwrap();
    command
        .env("GITPOULPI_CONFIG_DIR", config_dir())
        .args(["--json", "--session", session])
        .args(["--repo", &repo.root().to_string_lossy()])
        .args(args)
        .current_dir(repo.root());
    let output = command.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let value = serde_json::from_str(&stdout).unwrap_or_else(|e| {
        panic!(
            "not JSON ({e}): {stdout}\n{}",
            String::from_utf8_lossy(&output.stderr)
        )
    });
    (output.status.code().unwrap_or(-1), value)
}

fn ok(repo: &FixtureRepo, session: &str, args: &[&str]) -> Value {
    let (code, value) = run(repo, session, args);
    assert_eq!(code, 0, "{args:?}: {value}");
    value
}

fn put(repo: &FixtureRepo, files: &[(&str, &str)], message: &str) {
    for (path, text) in files {
        let full = repo.root().join(path);
        std::fs::create_dir_all(full.parent().unwrap()).unwrap();
        std::fs::write(full, text).unwrap();
    }
    repo.git(&["add", "-A"]);
    repo.git(&["commit", "-q", "-m", message]);
}

/// `initial`, `add lib`, `other work`, `edit lib`.
fn lib_repo() -> FixtureRepo {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .with_initial()
        .unwrap()
        .build();
    put(&repo, &[("lib/a.txt", "a1\n")], "add lib");
    put(&repo, &[("other/x.txt", "x\n")], "other work");
    put(&repo, &[("lib/a.txt", "a2\n")], "edit lib");
    repo
}

fn refs(repo: &FixtureRepo) -> String {
    repo.git(&[
        "for-each-ref",
        "--format=%(refname) %(objectname)",
        "refs/heads",
        "refs/tags",
    ])
}

fn subjects(dir: &Path, repo: &FixtureRepo) -> Vec<String> {
    repo.git_at(dir, &["log", "--reverse", "--format=%s"])
        .lines()
        .map(str::to_string)
        .collect()
}

#[test]
fn a_rename_runs_and_a_collision_is_refused_with_exit_2_and_the_paths() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("docs/a.md", "a\n", "add docs")
        .unwrap()
        .commit_file("guide/g.md", "g\n", "add guide")
        .unwrap()
        .build();
    let session = "cli-rw-paths-rename";
    let head = repo.git(&["rev-parse", "HEAD"]);
    let blob = repo.git(&["rev-parse", "HEAD:docs/a.md"]);

    let (code, value) = run(
        &repo,
        session,
        &[
            "rewrite",
            "run",
            "--move-path",
            "docs=guide",
            "--signatures",
            "strip",
        ],
    );
    assert_eq!(code, 2, "{value}");
    assert_eq!(value["refused"]["reason"], "path_collision");
    assert!(
        value["refused"]["detail"]
            .as_str()
            .unwrap()
            .contains("guide"),
        "{value}"
    );
    assert_eq!(repo.git(&["rev-parse", "HEAD"]), head, "nothing changed");

    let outcome: RewriteOutcome = serde_json::from_value(ok(
        &repo,
        session,
        &[
            "rewrite",
            "run",
            "--move-path",
            "docs=manual",
            "--signatures",
            "strip",
        ],
    ))
    .map(|v: Value| v["outcome"].clone())
    .and_then(serde_json::from_value)
    .unwrap();
    assert!(outcome.commits_changed >= 1);
    assert_eq!(repo.git(&["rev-parse", "HEAD:manual/a.md"]), blob);
    assert!(repo.try_git(&["rev-parse", "HEAD:docs"]).is_none());

    ok(&repo, session, &["undo"]);
    assert_eq!(repo.git(&["rev-parse", "HEAD"]), head);
    assert_eq!(repo.git(&["status", "--porcelain"]), "");
}

#[test]
fn the_whole_repository_moves_into_a_subdirectory() {
    let repo = lib_repo();
    let session = "cli-rw-paths-into";
    let root_tree = repo.git(&["rev-parse", "HEAD^{tree}"]);
    ok(
        &repo,
        session,
        &[
            "rewrite",
            "run",
            "--into-subdir",
            "vendor/app",
            "--signatures",
            "strip",
        ],
    );
    assert_eq!(repo.git(&["rev-parse", "HEAD:vendor/app"]), root_tree);
    assert_eq!(repo.git(&["ls-tree", "--name-only", "HEAD"]), "vendor");
}

#[test]
fn extract_asks_for_its_mode_with_exit_3_and_previews_nothing() {
    let repo = lib_repo();
    let session = "cli-rw-paths-ask";
    let head = repo.git(&["rev-parse", "HEAD"]);
    for verb in ["preview", "run"] {
        let (code, value) = run(
            &repo,
            session,
            &["rewrite", verb, "--extract", "lib", "--signatures", "strip"],
        );
        assert_eq!(code, 3, "{verb}: {value}");
        assert_eq!(value["needs_choice"]["choice"], "extract_mode");
        assert_eq!(
            value["needs_choice"]["options"],
            serde_json::json!(["clone", "in-place", "submodule"])
        );
    }
    assert_eq!(repo.git(&["rev-parse", "HEAD"]), head);
    assert!(!repo.root().join(".git/gitpoulpi/rewrite").exists());
}

#[test]
fn a_clone_or_submodule_extraction_needs_a_target() {
    let repo = lib_repo();
    let (code, value) = run(
        &repo,
        "cli-rw-paths-notarget",
        &[
            "rewrite",
            "run",
            "--extract",
            "lib",
            "--extract-mode",
            "clone",
            "--signatures",
            "strip",
        ],
    );
    assert_ne!(code, 0, "{value}");
    assert!(value.to_string().contains("--target"), "{value}");
}

#[test]
fn extract_in_place_rewrites_this_repository_and_undo_puts_it_back() {
    let repo = lib_repo();
    repo.git(&["branch", "no-lib", "main~3"]);
    let session = "cli-rw-paths-inplace";
    let before = refs(&repo);
    let head = repo.git(&["rev-parse", "HEAD"]);

    ok(
        &repo,
        session,
        &[
            "rewrite",
            "run",
            "--all-local",
            "--extract",
            "lib",
            "--extract-mode",
            "in-place",
            "--signatures",
            "strip",
        ],
    );
    assert_eq!(subjects(repo.root(), &repo), ["add lib", "edit lib"]);
    assert_eq!(repo.git(&["ls-tree", "-r", "--name-only", "HEAD"]), "a.txt");
    assert!(
        repo.try_git(&["rev-parse", "--verify", "refs/heads/no-lib"])
            .is_none(),
        "the branch that has nothing of the folder is deleted"
    );

    ok(&repo, session, &["undo"]);
    assert_eq!(refs(&repo), before, "undo brings back the branch too");
    assert_eq!(repo.git(&["rev-parse", "HEAD"]), head);
    assert_eq!(repo.git(&["status", "--porcelain"]), "");
    assert!(repo.root().join("lib/a.txt").exists());
}

#[test]
fn extract_into_a_clone_makes_a_new_repository_and_touches_nothing_here() {
    let repo = lib_repo();
    let session = "cli-rw-paths-clone";
    let before = refs(&repo);
    let out = tempfile::tempdir().unwrap();
    let target = out.path().join("lib-repo");

    let value = ok(
        &repo,
        session,
        &[
            "rewrite",
            "run",
            "--extract",
            "lib",
            "--extract-mode",
            "clone",
            "--target",
            &target.to_string_lossy(),
            "--signatures",
            "strip",
        ],
    );
    assert_eq!(value["outcome"]["moved"], serde_json::json!([]), "{value}");
    assert_eq!(refs(&repo), before);
    assert_eq!(repo.git(&["status", "--porcelain"]), "");
    assert_eq!(subjects(&target, &repo), ["add lib", "edit lib"]);
    assert_eq!(
        repo.git_at(&target, &["ls-tree", "-r", "--name-only", "HEAD"]),
        "a.txt"
    );
    // Not an undo entry: there is nothing here to put back.
    let (code, _) = run(&repo, session, &["undo", "--dry-run"]);
    assert_ne!(code, 0, "nothing to undo");
}

#[test]
fn extract_as_a_submodule_adds_one_commit_and_undo_restores_the_folder() {
    let repo = lib_repo();
    let session = "cli-rw-paths-submodule";
    let head = repo.git(&["rev-parse", "HEAD"]);
    let out = tempfile::tempdir().unwrap();
    let target = out.path().join("lib-repo");

    let value = ok(
        &repo,
        session,
        &[
            "rewrite",
            "run",
            "--extract",
            "lib",
            "--extract-mode",
            "submodule",
            "--target",
            &target.to_string_lossy(),
            "--signatures",
            "strip",
        ],
    );
    assert_eq!(
        value["outcome"]["moved"].as_array().map(Vec::len),
        Some(1),
        "{value}"
    );
    assert_eq!(repo.git(&["rev-parse", "HEAD^"]), head, "history is kept");
    let mode = repo.git(&["ls-tree", "HEAD", "lib"]);
    assert!(mode.starts_with("160000 commit "), "{mode}");
    assert_eq!(
        repo.git(&["config", "-f", ".gitmodules", "submodule.lib.path"]),
        "lib"
    );
    assert!(repo.git(&["submodule", "status"]).contains(" lib"));

    ok(&repo, session, &["undo"]);
    assert_eq!(repo.git(&["rev-parse", "HEAD"]), head);
    assert_eq!(repo.git(&["status", "--porcelain"]), "");
    assert_eq!(repo.git(&["submodule", "status"]), "");
    assert_eq!(
        std::fs::read_to_string(repo.root().join("lib/a.txt")).unwrap(),
        "a2\n"
    );
}

// No test reads or writes the real config, data dir or credentials of whoever runs it.
gitpoulpi_testkit::isolate_binary!();
