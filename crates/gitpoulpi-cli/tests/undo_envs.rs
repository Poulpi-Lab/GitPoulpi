//! The undo journal per environment (the installations design, 1.5), end to
//! end through the real binary. `GITPOULPI_ENV` (child environment only,
//! never a process-wide `set_var`) forces the environment layout even under
//! `GITPOULPI_CONFIG_DIR`; without it the layout stays flat.

use std::path::Path;

use assert_cmd::Command;
use gitpoulpi_core::domain::{JournalListing, OperationKind, UndoOutcome};
use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use serde_json::Value;

/// `(exit code, parsed stdout)` of `gitpoulpi --json ...` in `env` (`None`:
/// the flat layout), each env with its own sessions folder and daemon.
fn run(config: &Path, repo: &FixtureRepo, env: Option<&str>, args: &[&str]) -> (i32, Value) {
    let mut cmd = Command::cargo_bin("gitpoulpi").unwrap();
    cmd.env("GITPOULPI_CONFIG_DIR", config)
        .env("GITPOULPI_GENERAL_UNDO_DEPTH", "0")
        .env("GITPOULPI_GENERAL_JOURNAL_OUTSIDE", "true")
        .env_remove("GITPOULPI_ENV");
    if let Some(env) = env {
        cmd.env("GITPOULPI_ENV", env);
    }
    let output = cmd
        .arg("--json")
        .args(["--session", "envs"])
        .args(["--repo", &repo.root().to_string_lossy()])
        .args(args)
        .current_dir(repo.root())
        .output()
        .expect("failed to run gitpoulpi");
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let json = serde_json::from_str(&stdout).unwrap_or_else(|e| {
        panic!(
            "not JSON ({e}): {stdout}\nstderr: {}",
            String::from_utf8_lossy(&output.stderr)
        )
    });
    (output.status.code().unwrap_or(-1), json)
}

fn ok(config: &Path, repo: &FixtureRepo, env: Option<&str>, args: &[&str]) -> Value {
    let (code, json) = run(config, repo, env, args);
    assert_eq!(code, 0, "`gitpoulpi {}` failed: {json}", args.join(" "));
    json
}

fn fixture() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build()
}

/// Stages and commits `name` through GitPoulpi in `env`.
fn commit(config: &Path, repo: &FixtureRepo, env: Option<&str>, name: &str) {
    std::fs::write(repo.root().join(name), name).unwrap();
    ok(config, repo, env, &["stage", "--file", name]);
    ok(config, repo, env, &["commit", "-m", name]);
}

fn listing(config: &Path, repo: &FixtureRepo, env: Option<&str>) -> JournalListing {
    serde_json::from_value(ok(config, repo, env, &["undo", "--list"])).unwrap()
}

fn kinds(l: &JournalListing) -> Vec<OperationKind> {
    l.undo.iter().map(|e| e.summary.kind).collect()
}

fn gitpoulpi_dir(repo: &FixtureRepo) -> std::path::PathBuf {
    repo.root().join(".git").join("gitpoulpi")
}

fn pins(repo: &FixtureRepo) -> Vec<String> {
    repo.git(&[
        "for-each-ref",
        "--format=%(refname)",
        "refs/gitpoulpi/undo/",
    ])
    .lines()
    .map(str::to_string)
    .collect()
}

#[test]
fn two_envs_keep_separate_journals_and_pin_namespaces() {
    let config = tempfile::tempdir().unwrap();
    let repo = fixture();
    commit(config.path(), &repo, Some("alpha"), "a.txt");
    commit(config.path(), &repo, Some("beta"), "b.txt");

    let dir = gitpoulpi_dir(&repo);
    assert!(dir.join("env/alpha/journal.json").is_file());
    assert!(dir.join("env/beta/journal.json").is_file());
    assert!(
        !dir.join("journal.json").exists(),
        "the legacy file is not written"
    );

    let tag = |env| gitpoulpi_core::journal::env_pin_prefix(env);
    let (a, b) = (tag("alpha"), tag("beta"));
    assert_ne!(a, b);
    let pins = pins(&repo);
    assert!(
        pins.iter()
            .any(|p| p.starts_with(&format!("refs/gitpoulpi/undo/{a}"))),
        "{pins:?}"
    );
    assert!(
        pins.iter()
            .any(|p| p.starts_with(&format!("refs/gitpoulpi/undo/{b}"))),
        "{pins:?}"
    );
    assert!(
        pins.iter()
            .all(|p| p.starts_with(&format!("refs/gitpoulpi/undo/{a}"))
                || p.starts_with(&format!("refs/gitpoulpi/undo/{b}"))),
        "every pin is in an environment namespace: {pins:?}"
    );
    // Each journal lists the operations of its own environment first.
    let own = listing(config.path(), &repo, Some("alpha"));
    assert_eq!(own.undo[0].summary.kind, OperationKind::StageFile);
}

#[test]
fn an_operation_of_one_env_is_an_outside_entry_of_the_other() {
    let config = tempfile::tempdir().unwrap();
    let repo = fixture();
    // `beta` has its own baseline before `alpha` acts.
    commit(config.path(), &repo, Some("beta"), "b.txt");
    commit(config.path(), &repo, Some("alpha"), "a.txt");

    let seen = listing(config.path(), &repo, Some("beta"));
    assert!(
        kinds(&seen).contains(&OperationKind::Outside),
        "beta records alpha's commit as an outside change: {:?}",
        kinds(&seen)
    );
}

#[test]
fn undo_of_that_outside_entry_refuses_when_state_moved() {
    let config = tempfile::tempdir().unwrap();
    let repo = fixture();
    commit(config.path(), &repo, Some("beta"), "b.txt");
    commit(config.path(), &repo, Some("alpha"), "a.txt");
    let seen = listing(config.path(), &repo, Some("beta"));
    let outside = seen
        .undo
        .iter()
        .find(|e| e.summary.kind == OperationKind::Outside)
        .expect("an outside entry")
        .summary
        .id;
    // The state moves on after beta saw it.
    commit(config.path(), &repo, Some("alpha"), "c.txt");
    let tip = repo.git(&["rev-parse", "HEAD"]);

    let (code, json) = run(
        config.path(),
        &repo,
        Some("beta"),
        &["undo", "--expect", &outside.to_string()],
    );
    assert_eq!(code, 2, "refused, not applied: {json}");
    assert_eq!(repo.git(&["rev-parse", "HEAD"]), tip, "nothing changed");
}

#[test]
fn the_legacy_journal_is_adopted_once_by_copy_and_left_in_place() {
    let config = tempfile::tempdir().unwrap();
    let repo = fixture();
    // A build without environments (the flat layout) made this history.
    commit(config.path(), &repo, None, "old.txt");
    let legacy = gitpoulpi_dir(&repo).join("journal.json");
    let before = std::fs::read(&legacy).unwrap();
    let legacy_pins = pins(&repo);
    assert!(!legacy_pins.is_empty());

    // The first environment to open the repository adopts it.
    let first = listing(config.path(), &repo, Some("alpha"));
    assert!(
        kinds(&first).contains(&OperationKind::Commit),
        "{:?}",
        kinds(&first)
    );
    assert_eq!(std::fs::read(&legacy).unwrap(), before, "left in place");
    let marker = gitpoulpi_dir(&repo).join("legacy-journal-adopted");
    assert!(marker.is_file());
    assert_eq!(pins(&repo), legacy_pins, "the legacy pins are untouched");

    // A second environment starts empty: it happens once per repository.
    let second = listing(config.path(), &repo, Some("beta"));
    assert!(
        !kinds(&second).contains(&OperationKind::Commit),
        "{:?}",
        kinds(&second)
    );

    // The copied entries keep working: undo the adopted commit.
    let tip_before = repo.git(&["rev-parse", "HEAD"]);
    let undone: UndoOutcome =
        serde_json::from_value(ok(config.path(), &repo, Some("alpha"), &["undo"])).unwrap();
    assert!(undone.applied);
    assert_ne!(repo.git(&["rev-parse", "HEAD"]), tip_before);
    assert_eq!(std::fs::read(&legacy).unwrap(), before, "still in place");
    for pin in &legacy_pins {
        assert!(
            pins(&repo).contains(pin),
            "an environment never deletes a legacy pin: {pin}"
        );
    }
}

#[test]
fn the_flat_layout_keeps_todays_journal_paths() {
    let config = tempfile::tempdir().unwrap();
    let repo = fixture();
    commit(config.path(), &repo, None, "x.txt");
    let dir = gitpoulpi_dir(&repo);
    assert!(dir.join("journal.json").is_file());
    assert!(dir.join("refs-snapshot.json").is_file());
    assert!(!dir.join("env").exists());
    assert!(!dir.join("legacy-journal-adopted").exists());
    let pins = pins(&repo);
    assert!(!pins.is_empty());
    for pin in pins {
        let rest = pin.strip_prefix("refs/gitpoulpi/undo/").unwrap();
        assert!(rest.starts_with(|c: char| c.is_ascii_digit()), "{pin}");
    }
}
