//! `gitpoulpi conflicts ...` through the real binary and daemon, on
//! fixture repositories stopped by a real merge, cherry-pick and rebase.

use std::path::Path;

use assert_cmd::Command;
use gitpoulpi_testkit::{ConflictFixture, FixtureOperation};
use serde_json::Value;

fn config_dir() -> &'static Path {
    static DIR: std::sync::OnceLock<tempfile::TempDir> = std::sync::OnceLock::new();
    DIR.get_or_init(|| tempfile::tempdir().unwrap()).path()
}

/// Runs `gitpoulpi --json ...`; returns the exit code and the parsed stdout.
fn run(root: &Path, session: &str, args: &[&str]) -> (i32, Value) {
    let out = Command::cargo_bin("gitpoulpi")
        .unwrap()
        .env("GITPOULPI_CONFIG_DIR", config_dir())
        .arg("--json")
        .args(["--session", session])
        .args(["--repo", &root.to_string_lossy()])
        .args(args)
        .current_dir(root)
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    let json = serde_json::from_str(&stdout).unwrap_or_else(|e| {
        panic!(
            "`{}` gave no JSON ({e}): {stdout}\n{}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr)
        )
    });
    (out.status.code().unwrap_or(-1), json)
}

fn ok(root: &Path, session: &str, args: &[&str]) -> Value {
    let (code, json) = run(root, session, args);
    assert_eq!(code, 0, "`{}` failed: {json}", args.join(" "));
    json
}

fn refused(root: &Path, session: &str, args: &[&str]) -> Value {
    let (code, json) = run(root, session, args);
    assert_eq!(code, 2, "`{}` should be refused: {json}", args.join(" "));
    json["refused"].clone()
}

fn unmerged(f: &ConflictFixture) -> String {
    f.git(&["diff", "--name-only", "--diff-filter=U"])
}

#[test]
fn list_names_only_and_full_agree_on_the_paths() {
    let f = ConflictFixture::text();
    let s = "cli-conflicts-list";
    let entries = ok(f.root(), s, &["conflicts", "list", "--names-only"]);
    assert_eq!(entries[0]["path"], "notes.txt");
    assert_eq!(entries[0]["kind"], "BothModified");
    assert!(entries[0]["ours"]["oid"].is_string());
    let full = ok(f.root(), s, &["conflicts", "list"]);
    assert_eq!(full[0]["path"], "notes.txt");
    assert!(full[0]["ours"].as_str().unwrap().contains("three (ours)"));
}

#[test]
fn show_gives_numbered_regions_labels_and_sides() {
    let f = ConflictFixture::multi_conflict();
    let s = "cli-conflicts-show";
    let doc = ok(f.root(), s, &["conflicts", "show", "a.txt"]);
    assert_eq!(doc["path"], "a.txt");
    assert_eq!(doc["operation"], "Merge");
    assert!(doc["labels"]["ours"].as_str().unwrap().contains("main"));
    assert!(doc["labels"]["theirs"]
        .as_str()
        .unwrap()
        .contains("feature"));
    assert_eq!(doc["sides"]["ours"]["content_kind"], "text");
    assert!(doc["sides"]["base"]["oid"].is_string());
    assert_eq!(doc["conflict_count"], 3);
    let conflicts = doc["conflicts"].as_array().unwrap();
    assert_eq!(conflicts[0]["index"], 1);
    assert_eq!(conflicts[0]["ours"][0], "A3 ours");
    assert_eq!(conflicts[0]["theirs"][1], "A4 theirs");
    // `--conflict N` narrows it.
    let one = ok(
        f.root(),
        s,
        &["conflicts", "show", "a.txt", "--conflict", "2"],
    );
    assert_eq!(one["conflicts"].as_array().unwrap().len(), 1);
    assert_eq!(one["conflicts"][0]["index"], 2);
    assert_eq!(one["conflicts"][0]["ours"][0], "A15 ours");
}

#[test]
fn state_names_the_operation_and_abort_ends_it() {
    for (op, name) in [
        (FixtureOperation::Merge, "merge"),
        (FixtureOperation::CherryPick, "cherry-pick"),
        (FixtureOperation::Rebase, "rebase"),
    ] {
        let f = ConflictFixture::text_with(op);
        let s = format!("cli-conflicts-state-{name}");
        let state = ok(f.root(), &s, &["conflicts", "state"]);
        assert_eq!(state["operation"], name);
        let aborted = ok(f.root(), &s, &["conflicts", "abort"]);
        assert_eq!(aborted["aborted"], name);
        assert!(unmerged(&f).trim().is_empty());
        assert!(ok(f.root(), &s, &["conflicts", "state"]).is_null());
        let none = refused(f.root(), &s, &["conflicts", "abort"]);
        assert_eq!(none["reason"], "no_operation");
    }
}

/// Maintainer rule: no state left only by plain git. An empty cherry-pick or
/// revert is left with `conflicts skip`; a merge whose result equals HEAD
/// is concluded by `commit` with nothing staged.
#[test]
fn skip_ends_an_empty_pick_and_commit_concludes_a_take_ours_merge() {
    for (op, name) in [
        (FixtureOperation::CherryPick, "cherry-pick"),
        (FixtureOperation::Revert, "revert"),
    ] {
        let f = ConflictFixture::text_with(op);
        let s = format!("cli-conflicts-skip-{name}");
        let head = f.git(&["rev-parse", "HEAD"]);
        f.git(&["checkout", "--ours", "notes.txt"]);
        f.git(&["add", "notes.txt"]);
        let skipped = ok(f.root(), &s, &["conflicts", "skip"]);
        assert_eq!(skipped["skipped"], name);
        assert!(ok(f.root(), &s, &["conflicts", "state"]).is_null());
        assert_eq!(f.git(&["rev-parse", "HEAD"]), head);
        let none = refused(f.root(), &s, &["conflicts", "skip"]);
        assert_eq!(none["reason"], "no_operation");
    }

    let f = ConflictFixture::text();
    let s = "cli-conflicts-skip-merge";
    let (code, _) = run(f.root(), s, &["conflicts", "skip"]);
    assert_ne!(code, 0, "a merge has nothing to skip");
    assert_eq!(
        ok(f.root(), s, &["conflicts", "state"])["operation"],
        "merge"
    );
    f.git(&["checkout", "--ours", "notes.txt"]);
    f.git(&["add", "notes.txt"]);
    let (code, json) = run(f.root(), s, &["commit", "-m", "Merge feature (ours)"]);
    assert_eq!(code, 0, "{json}");
    assert!(ok(f.root(), s, &["conflicts", "state"]).is_null());
    let parents = f.git(&["rev-list", "--parents", "-n1", "HEAD"]);
    assert_eq!(parents.split_whitespace().count(), 3, "{parents}");
}

#[test]
fn pick_composes_the_file_and_stages_it() {
    let f = ConflictFixture::multi_conflict();
    let s = "cli-conflicts-pick";
    let r = ok(
        f.root(),
        s,
        &["conflicts", "resolve", "a.txt", "--pick", "1=ab,2=b,3=none"],
    );
    assert_eq!(r["resolved"], "a.txt");
    let text = String::from_utf8(f.worktree_bytes("a.txt")).unwrap();
    assert!(!text.contains("<<<<<<<"));
    assert!(
        text.contains("A3 ours\nA4 ours\nA3 theirs\nA4 theirs\n"),
        "{text}"
    );
    assert!(text.contains("A15 theirs"));
    assert!(!text.contains("A15 ours"));
    assert!(!text.contains("A28 "));
    assert!(!unmerged(&f).contains("a.txt"));
    let after = ok(f.root(), s, &["conflicts", "list", "--names-only"]);
    assert_eq!(after.as_array().unwrap().len(), 2);
}

#[test]
fn pick_must_cover_every_conflict_unless_a_default_is_given() {
    let f = ConflictFixture::multi_conflict();
    let s = "cli-conflicts-pick-incomplete";
    let r = refused(
        f.root(),
        s,
        &["conflicts", "resolve", "a.txt", "--pick", "1=a"],
    );
    assert_eq!(
        r["reason"]["pick_incomplete"]["missing"],
        serde_json::json!([2, 3])
    );
    assert!(unmerged(&f).contains("a.txt"), "nothing was written");
    ok(
        f.root(),
        s,
        &[
            "conflicts",
            "resolve",
            "a.txt",
            "--pick",
            "1=a",
            "--default",
            "b",
        ],
    );
    let text = String::from_utf8(f.worktree_bytes("a.txt")).unwrap();
    assert!(
        text.contains("A3 ours") && text.contains("A28 theirs"),
        "{text}"
    );
    assert!(!text.contains("A15 ours"), "{text}");
}

#[test]
fn both_keeps_a_then_b_or_b_then_a() {
    let f = ConflictFixture::text();
    let s = "cli-conflicts-both";
    ok(
        f.root(),
        s,
        &[
            "conflicts",
            "resolve",
            "notes.txt",
            "--both",
            "--theirs-first",
            "--no-stage",
        ],
    );
    let text = String::from_utf8(f.worktree_bytes("notes.txt")).unwrap();
    let b = text.find("three (theirs)").unwrap();
    let a = text.find("three (ours)").unwrap();
    assert!(b < a, "{text}");
    assert!(text.contains("eight (theirs only)"), "git's own merge kept");
    // `--no-stage` saved the file and left it conflicted.
    assert!(unmerged(&f).contains("notes.txt"));
    ok(
        f.root(),
        s,
        &["conflicts", "resolve", "notes.txt", "--both"],
    );
    let text = String::from_utf8(f.worktree_bytes("notes.txt")).unwrap();
    assert!(text.find("three (ours)").unwrap() < text.find("three (theirs)").unwrap());
    assert!(unmerged(&f).trim().is_empty());
}

#[test]
fn side_flags_and_content_file_resolve_a_cherry_pick() {
    let f = ConflictFixture::text_with(FixtureOperation::CherryPick);
    let s = "cli-conflicts-content";
    let file = f.root().join("..").join("resolved-notes.txt");
    let file = file.canonicalize().unwrap_or(file);
    std::fs::write(&file, "one\nfinal text\n").unwrap();
    let r = ok(
        f.root(),
        s,
        &[
            "conflicts",
            "resolve",
            "notes.txt",
            "--content-file",
            &file.to_string_lossy(),
        ],
    );
    assert_eq!(r["staged"], true);
    assert_eq!(f.worktree_bytes("notes.txt"), b"one\nfinal text\n");
    assert!(unmerged(&f).trim().is_empty());
    let _ = std::fs::remove_file(file);
}

#[test]
fn ours_resolves_a_rebase_conflict_and_unresolve_brings_it_back() {
    let f = ConflictFixture::text_with(FixtureOperation::Rebase);
    let s = "cli-conflicts-unresolve";
    // Nothing was resolved during this operation yet: an error, not a change.
    let (code, _) = run(f.root(), s, &["conflicts", "unresolve", "notes.txt"]);
    assert_eq!(code, 1);
    assert!(unmerged(&f).contains("notes.txt"));
    ok(
        f.root(),
        s,
        &["conflicts", "resolve", "notes.txt", "--ours"],
    );
    assert!(ok(f.root(), s, &["conflicts", "list", "--names-only"])
        .as_array()
        .unwrap()
        .is_empty());
    let r = ok(f.root(), s, &["conflicts", "unresolve", "notes.txt"]);
    assert_eq!(r["unresolved"], "notes.txt");
    let listed = ok(f.root(), s, &["conflicts", "list", "--names-only"]);
    assert_eq!(listed[0]["path"], "notes.txt");
}

#[test]
fn a_stale_expectation_and_a_clean_path_are_refused() {
    let f = ConflictFixture::text();
    let s = "cli-conflicts-refusals";
    let bogus = "1".repeat(40);
    let r = refused(
        f.root(),
        s,
        &[
            "conflicts",
            "resolve",
            "notes.txt",
            "--ours",
            "--expect",
            &format!("{bogus},{bogus},{bogus}"),
        ],
    );
    assert_eq!(r["reason"], "stale");
    assert!(
        unmerged(&f).contains("notes.txt"),
        "a refusal writes nothing"
    );

    ok(
        f.root(),
        s,
        &["conflicts", "resolve", "notes.txt", "--theirs"],
    );
    let r = refused(
        f.root(),
        s,
        &["conflicts", "resolve", "notes.txt", "--ours"],
    );
    assert_eq!(r["reason"], "not_conflicted");
}

#[test]
fn delete_accepts_a_deletion_and_exactly_one_way_is_required() {
    let f = ConflictFixture::delete_modify(true);
    let s = "cli-conflicts-delete";
    let path = f.path().to_string();
    let entries = ok(f.root(), s, &["conflicts", "list", "--names-only"]);
    assert_eq!(entries[0]["kind"], "DeletedByUs");
    // No regions to pick from.
    let r = refused(
        f.root(),
        s,
        &["conflicts", "resolve", &path, "--pick", "1=a"],
    );
    assert_eq!(r["reason"], "not_a_text_conflict");
    ok(f.root(), s, &["conflicts", "resolve", &path, "--delete"]);
    assert!(!f.root().join(&path).exists());

    // Neither no way nor two ways parse.
    let out = Command::cargo_bin("gitpoulpi")
        .unwrap()
        .env("GITPOULPI_CONFIG_DIR", config_dir())
        .args(["conflicts", "resolve", "x"])
        .output()
        .unwrap();
    assert!(!out.status.success());
    let out = Command::cargo_bin("gitpoulpi")
        .unwrap()
        .env("GITPOULPI_CONFIG_DIR", config_dir())
        .args(["conflicts", "resolve", "x", "--ours", "--delete"])
        .output()
        .unwrap();
    assert!(!out.status.success());
}

// No test reads or writes the real config, data dir or credentials of whoever runs it.
gitpoulpi_testkit::isolate_binary!();
