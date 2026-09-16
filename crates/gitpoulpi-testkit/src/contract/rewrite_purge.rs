//! History rewrite, slice S15 (the history-rewrite design,
//! "Purge (S15)"): after a rewrite the old objects really are gone, and
//! whatever still keeps the old history alive is a typed blocker that stops
//! the run before it changes anything. Expected values come from plain `git`.

use gitpoulpi_core::domain::rewrite::{
    ContentFind, ContentReplace, PurgeBlockerKind, PurgeEntry, PurgeOutcome, PurgeRequest,
    PurgeStatus, PurgeStep, Recipe, Rule, RuleKind, Scope, SecretText, SignaturePolicy,
};
use gitpoulpi_core::journal::RefChange;
use gitpoulpi_core::GitBackend;

use super::rewrite::{git_text, run};
use crate::{FixtureRepo, FixtureRepoBuilder};

const TOKEN: &str = "ghp_Zk9xQ2mL7vB4nR8tY1wC5dF3hJ6pS0aEgUiX";

fn put(repo: &FixtureRepo, files: &[(&str, &str)], message: &str) -> String {
    for (path, text) in files {
        std::fs::write(repo.root().join(path), text).unwrap();
    }
    repo.git(&["add", "-A"]);
    repo.git(&["commit", "-q", "-m", message]);
    git_text(repo, &["rev-parse", "HEAD"])
}

/// Replaces the token everywhere in the current branch's history.
fn remove_token() -> Recipe {
    let rule = RuleKind::ContentReplace(ContentReplace {
        paths: Vec::new(),
        find: ContentFind::Secrets {
            literals: vec![SecretText(TOKEN.to_string())],
            scan: false,
        },
        with: String::new(),
        max_blob_bytes: None,
    });
    let mut recipe = Recipe::new("remove token", Scope::default(), vec![Rule::new(rule)]);
    recipe.options.signatures = Some(SignaturePolicy::Strip);
    recipe
}

/// `c1` adds the token, `c2` edits the file and keeps it, `c3` is unrelated.
struct Old {
    repo: FixtureRepo,
    /// The two blobs that held the token.
    blobs: [String; 2],
    commits: [String; 3],
}

fn old_history() -> Old {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .with_initial()
        .unwrap()
        .build();
    let c1 = put(
        &repo,
        &[("config.env", &format!("TOKEN={TOKEN}\nmode=dev\n"))],
        "add config",
    );
    let c2 = put(
        &repo,
        &[("config.env", &format!("TOKEN={TOKEN}\nmode=prod\n"))],
        "edit config",
    );
    let c3 = put(&repo, &[("notes.txt", "hello\n")], "notes");
    let blobs = [
        git_text(&repo, &["rev-parse", &format!("{c1}:config.env")]),
        git_text(&repo, &["rev-parse", &format!("{c2}:config.env")]),
    ];
    Old {
        repo,
        blobs,
        commits: [c1, c2, c3],
    }
}

fn exists(repo: &FixtureRepo, id: &str) -> bool {
    repo.try_git(&["cat-file", "-e", id]).is_some()
}

/// What the daemon builds from the journal entry of an apply: the moved refs
/// and the pins it created.
fn entry(moved: &[gitpoulpi_core::domain::rewrite::RefChange], pins: &[&str]) -> PurgeEntry {
    PurgeEntry {
        id: 1,
        label: "Rewrite history".into(),
        refs: moved
            .iter()
            .map(|r| RefChange {
                name: r.name.clone(),
                old: Some(r.old),
                new: r.new,
            })
            .collect(),
        pins: pins.iter().map(|p| p.to_string()).collect(),
    }
}

fn purge(
    backend: &dyn GitBackend,
    repo: &FixtureRepo,
    entries: Vec<PurgeEntry>,
    dry_run: bool,
) -> PurgeOutcome {
    backend
        .rewrite_purge(
            &repo.repo_ref(),
            &PurgeRequest { entries, dry_run },
            &mut |_: PurgeStep| {},
        )
        .expect("rewrite_purge")
}

/// Rewrites `main` (the token goes) and pins the old tip as the daemon does.
fn rewritten(backend: &dyn GitBackend, old: &Old) -> PurgeEntry {
    let old_tip = git_text(&old.repo, &["rev-parse", "main"]);
    let (_, applied) = run(backend, &old.repo, &remove_token());
    old.repo
        .git(&["update-ref", "refs/gitpoulpi/undo/1/0", &old_tip]);
    entry(&applied.moved, &["refs/gitpoulpi/undo/1/0"])
}

fn fsck_is_clean(repo: &FixtureRepo) {
    // A failing fsck panics inside `git`; "missing" or "broken" in its output
    // is how a pruned object that something still needs shows.
    let out = repo.git(&["fsck", "--full", "--strict"]);
    assert!(
        !out.contains("missing") && !out.contains("broken"),
        "fsck: {out}"
    );
}

/// After a rewrite and a purge the old blobs and commits are gone from the
/// object store, the new history is whole, and the reflogs no longer name the
/// old commits. A dry run changes nothing.
pub fn check_rewrite_purge_deletes_the_old_objects(backend: &dyn GitBackend) {
    let old = old_history();
    let entry = rewritten(backend, &old);
    let entry_again = entry.clone();
    let new_log = git_text(&old.repo, &["log", "--format=%H", "main"]);
    for blob in &old.blobs {
        assert!(exists(&old.repo, blob), "the pin keeps {blob} before");
    }

    // A dry run reports what it would do and does none of it.
    let dry = purge(backend, &old.repo, vec![entry.clone()], true);
    assert_eq!(dry.status, PurgeStatus::Ready, "{dry:?}");
    assert!(dry.blockers.is_empty(), "{dry:?}");
    assert_eq!(dry.old_commits, 3, "c1, c2 and c3 were rewritten: {dry:?}");
    assert!(dry.old_blobs >= 2, "{dry:?}");
    assert!(dry.reflog_entries > 0, "{dry:?}");
    assert!(dry.reflogs.iter().any(|r| r == "HEAD"), "{dry:?}");
    assert!(exists(&old.repo, &old.blobs[0]));
    assert!(old
        .repo
        .try_git(&["rev-parse", "--verify", "refs/gitpoulpi/undo/1/0"])
        .is_some());

    let mut steps = Vec::new();
    let done = backend
        .rewrite_purge(
            &old.repo.repo_ref(),
            &PurgeRequest {
                entries: vec![entry],
                dry_run: false,
            },
            &mut |s| steps.push(s),
        )
        .unwrap();
    assert_eq!(done.status, PurgeStatus::Purged, "{done:?}");
    assert_eq!(done.remaining, 0);
    assert_eq!(done.pins_dropped, 1);
    assert_eq!(
        steps,
        vec![
            PurgeStep::Checking,
            PurgeStep::Pins,
            PurgeStep::Reflogs,
            PurgeStep::Collecting,
            PurgeStep::Verifying
        ]
    );

    for blob in &old.blobs {
        assert!(!exists(&old.repo, blob), "old blob {blob} is still there");
    }
    for commit in &old.commits {
        assert!(!exists(&old.repo, commit), "old commit {commit}");
    }
    assert!(old
        .repo
        .try_git(&["rev-parse", "--verify", "refs/gitpoulpi/undo/1/0"])
        .is_none());
    fsck_is_clean(&old.repo);
    assert_eq!(
        git_text(&old.repo, &["log", "--format=%H", "main"]),
        new_log,
        "the new history is untouched"
    );
    // No reflog names an old commit any more.
    let reflogs = old.repo.git(&["reflog", "show", "--all", "--format=%H"]);
    for commit in &old.commits {
        assert!(!reflogs.contains(commit.as_str()), "reflog names {commit}");
    }
    // Nothing left to purge: the same request again is a clean no-op.
    let again = purge(backend, &old.repo, vec![entry_again], false);
    assert_eq!(again.status, PurgeStatus::Purged, "{again:?}");
    assert_eq!(again.old_objects, 0, "{again:?}");
}

/// A stash made on the old history keeps it alive: the purge names the stash,
/// changes nothing, and goes through once the stash is dropped.
pub fn check_rewrite_purge_is_blocked_by_a_stash_on_the_old_history(backend: &dyn GitBackend) {
    let old = old_history();
    std::fs::write(old.repo.root().join("notes.txt"), "work in progress\n").unwrap();
    old.repo.git(&["stash", "push", "-q", "-m", "wip"]);
    let entry = rewritten(backend, &old);

    let blocked = purge(backend, &old.repo, vec![entry.clone()], false);
    assert_eq!(blocked.status, PurgeStatus::Blocked, "{blocked:?}");
    assert_eq!(blocked.blockers.len(), 1, "{blocked:?}");
    assert_eq!(blocked.blockers[0].kind, PurgeBlockerKind::Stash);
    assert_eq!(blocked.blockers[0].name, "stash@{0}");
    assert!(blocked.blockers[0].commits >= 1);
    // Nothing was touched: the pin, the old blob and the stash are there.
    assert!(exists(&old.repo, &old.blobs[0]));
    assert!(old
        .repo
        .try_git(&["rev-parse", "--verify", "refs/gitpoulpi/undo/1/0"])
        .is_some());
    assert!(old.repo.git(&["stash", "list"]).contains("wip"));

    old.repo.git(&["stash", "drop", "-q"]);
    let done = purge(backend, &old.repo, vec![entry], false);
    assert_eq!(done.status, PurgeStatus::Purged, "{done:?}");
    assert!(!exists(&old.repo, &old.blobs[0]));
    fsck_is_clean(&old.repo);
}

/// Branches, remote-tracking refs and detached worktrees that were not
/// rewritten keep the old commits: each is named, and removing them lets the
/// purge through.
pub fn check_rewrite_purge_is_blocked_by_refs_and_worktrees_outside_the_rewrite(
    backend: &dyn GitBackend,
) {
    let old = old_history();
    let c2 = old.commits[1].clone();
    old.repo.git(&["branch", "outside", &c2]);
    old.repo
        .git(&["update-ref", "refs/remotes/origin/main", &c2]);
    let linked = old.repo.root().with_file_name("purge-linked");
    old.repo.git(&[
        "worktree",
        "add",
        "--detach",
        &linked.to_string_lossy(),
        &c2,
    ]);
    // A tag on a commit only a side branch reaches is outside the rewrite
    // (the current-branch scope retargets the tags on its own commits).
    old.repo.git(&["checkout", "-q", "outside"]);
    let side = put(&old.repo, &[("side.txt", "side\n")], "side");
    old.repo.git(&["tag", "v-old", &side]);
    old.repo.git(&["checkout", "-q", "main"]);
    let entry = rewritten(backend, &old);

    let blocked = purge(backend, &old.repo, vec![entry.clone()], false);
    assert_eq!(blocked.status, PurgeStatus::Blocked, "{blocked:?}");
    let has = |kind: PurgeBlockerKind, name: &str| {
        blocked
            .blockers
            .iter()
            .any(|b| b.kind == kind && b.name.ends_with(name) && b.commits >= 1)
    };
    assert!(
        has(PurgeBlockerKind::Branch, "refs/heads/outside"),
        "{blocked:?}"
    );
    assert!(has(PurgeBlockerKind::Tag, "refs/tags/v-old"), "{blocked:?}");
    assert!(
        has(PurgeBlockerKind::RemoteTracking, "refs/remotes/origin/main"),
        "{blocked:?}"
    );
    assert!(
        has(PurgeBlockerKind::Worktree, "purge-linked"),
        "{blocked:?}"
    );
    assert_eq!(blocked.blockers.len(), 4, "{blocked:?}");
    assert!(exists(&old.repo, &old.blobs[0]), "nothing was deleted");

    old.repo.git(&["branch", "-D", "outside"]);
    old.repo.git(&["tag", "-d", "v-old"]);
    old.repo
        .git(&["update-ref", "-d", "refs/remotes/origin/main"]);
    old.repo
        .git(&["worktree", "remove", "--force", &linked.to_string_lossy()]);
    let done = purge(backend, &old.repo, vec![entry], false);
    assert_eq!(done.status, PurgeStatus::Purged, "{done:?}");
    assert!(!exists(&old.repo, &old.blobs[1]));
    fsck_is_clean(&old.repo);
}

/// The undo pin of an entry that is not part of the request keeps the old
/// history too, and says so.
pub fn check_rewrite_purge_is_blocked_by_the_pin_of_another_entry(backend: &dyn GitBackend) {
    let old = old_history();
    let entry = rewritten(backend, &old);
    let old_tip = old.commits[2].clone();
    old.repo
        .git(&["update-ref", "refs/gitpoulpi/undo/2/0", &old_tip]);

    let blocked = purge(backend, &old.repo, vec![entry.clone()], false);
    assert_eq!(blocked.status, PurgeStatus::Blocked, "{blocked:?}");
    assert_eq!(blocked.blockers.len(), 1);
    assert_eq!(blocked.blockers[0].kind, PurgeBlockerKind::UndoPin);
    assert_eq!(blocked.blockers[0].name, "refs/gitpoulpi/undo/2/0");

    old.repo
        .git(&["update-ref", "-d", "refs/gitpoulpi/undo/2/0"]);
    let done = purge(backend, &old.repo, vec![entry], false);
    assert_eq!(done.status, PurgeStatus::Purged, "{done:?}");
}

/// A purge is conservative: the stash list survives it, and so do the
/// reflogs of refs that never touched the old history.
pub fn check_rewrite_purge_keeps_stashes_and_unrelated_reflogs(backend: &dyn GitBackend) {
    let old = old_history();
    let root = git_text(&old.repo, &["rev-list", "--max-parents=0", "main"]);
    old.repo.git(&["branch", "feature", &root]);
    old.repo.git(&["checkout", "-q", "feature"]);
    put(&old.repo, &[("f.txt", "one\n")], "feature one");
    put(&old.repo, &[("f.txt", "two\n")], "feature two");
    old.repo.git(&["checkout", "-q", "main"]);
    let feature_log = old
        .repo
        .git(&["reflog", "show", "--format=%H %gs", "feature"]);
    assert!(feature_log.lines().count() >= 3);

    let entry = rewritten(backend, &old);
    // A stash on the new history, made after the rewrite.
    std::fs::write(old.repo.root().join("notes.txt"), "new work\n").unwrap();
    old.repo.git(&["stash", "push", "-q", "-m", "after"]);
    old.repo.git(&["stash", "push", "-q", "-m", "unused"]);
    let stash_before = old.repo.git(&["stash", "list"]);

    let done = purge(backend, &old.repo, vec![entry], false);
    assert_eq!(done.status, PurgeStatus::Purged, "{done:?}");
    assert!(!done.reflogs.iter().any(|r| r == "refs/stash"), "{done:?}");
    assert!(
        !done.reflogs.iter().any(|r| r == "refs/heads/feature"),
        "{done:?}"
    );
    assert_eq!(old.repo.git(&["stash", "list"]), stash_before);
    assert_eq!(
        old.repo
            .git(&["reflog", "show", "--format=%H %gs", "feature"]),
        feature_log
    );
    fsck_is_clean(&old.repo);
}

/// The request is the only thing that names the old history: an entry whose
/// refs did not move purges nothing and still succeeds.
pub fn check_rewrite_purge_of_nothing_is_a_clean_no_op(backend: &dyn GitBackend) {
    let old = old_history();
    let blob = old.blobs[0].clone();
    let done = purge(
        backend,
        &old.repo,
        vec![PurgeEntry {
            id: 9,
            label: "nothing".into(),
            refs: Vec::new(),
            pins: Vec::new(),
        }],
        false,
    );
    assert_eq!(done.status, PurgeStatus::Purged, "{done:?}");
    assert_eq!((done.old_commits, done.old_objects), (0, 0));
    assert!(exists(&old.repo, &blob), "unrelated history is not touched");
}
