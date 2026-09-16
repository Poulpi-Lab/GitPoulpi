//! `split_source` / `split_commit` (the split-commit design):
//! one commit becomes N commits by plumbing, the last carrying the original
//! tree, so descendants keep theirs and no ref but the branch moves.

use std::path::Path;

use crate::{FixtureRepo, TestRepoBuilder};
use gitpoulpi_core::domain::{
    FileStatus, HunkId, Oid, SplitCommit, SplitPart, SplitPiece, SplitRefusal, WholeReason,
};
use gitpoulpi_core::GitBackend;

use super::head_oid;

/// `line 1\n` .. `line 40\n`, so edits at the ends land in separate hunks.
fn forty_lines() -> String {
    (1..=40).map(|i| format!("line {i}\n")).collect()
}

/// `forty_lines()` with lines 2 and 38 upper-cased: two hunks.
fn forty_lines_edited() -> String {
    forty_lines()
        .replace("line 2\n", "LINE 2\n")
        .replace("line 38\n", "LINE 38\n")
}

fn write(repo: &FixtureRepo, path: &str, contents: &[u8]) {
    std::fs::write(repo.root().join(path), contents).unwrap();
}

fn commit_all(repo: &FixtureRepo, message: &str) {
    repo.git(&["add", "-A"]);
    repo.git(&["commit", "-m", message]);
}

fn hunk(path: &str, n: u32) -> SplitPiece {
    SplitPiece::Hunk {
        path: path.into(),
        hunk: HunkId(n),
    }
}

fn file(path: &str) -> SplitPiece {
    SplitPiece::File { path: path.into() }
}

fn part(message: &str, pieces: Vec<SplitPiece>) -> SplitPart {
    SplitPart {
        message: message.into(),
        pieces,
    }
}

fn request(commit: Oid, parts: Vec<SplitPart>) -> SplitCommit {
    SplitCommit {
        commit,
        parts,
        sign: false,
    }
}

/// Author name, email and raw date of a commit: what a split must keep.
fn author(repo: &FixtureRepo, commit: &str) -> String {
    repo.git(&["log", "-1", "--date=raw", "--format=%an|%ae|%ad", commit])
}

fn assert_no_temp_index(repo: &FixtureRepo) {
    for entry in std::fs::read_dir(repo.root().join(".git")).unwrap() {
        let name = entry.unwrap().file_name().to_string_lossy().into_owned();
        assert!(
            !name.starts_with("gitpoulpi-split"),
            "a temporary index was left behind: {name}"
        );
    }
}

/// Tree, author, parent count and subject of every commit reachable from
/// HEAD, newest first: what a split keeps for every descendant.
fn shape(repo: &FixtureRepo) -> Vec<(String, String)> {
    repo.git(&[
        "log",
        "--topo-order",
        "--date=raw",
        "--format=%T|%an|%ae|%ad|%P%x01%s",
        "HEAD",
    ])
    .lines()
    .map(|line| {
        let (kept, subject) = line.split_once('\u{1}').expect("separator");
        let (fixed, parents) = kept.rsplit_once('|').expect("parents");
        (
            format!("{fixed}|parents={}", parents.split_whitespace().count()),
            subject.to_string(),
        )
    })
    .collect()
}

/// Whether `rev:path` holds exactly `bytes`, compared by blob id.
fn blob_is(repo: &FixtureRepo, rev: &str, path: &str, bytes: &[u8]) -> bool {
    let scratch = tempfile::tempdir().unwrap();
    let expected = scratch.path().join("expected");
    std::fs::write(&expected, bytes).unwrap();
    let want = repo.git(&["hash-object", "--no-filters", expected.to_str().unwrap()]);
    repo.git(&["rev-parse", &format!("{rev}:{path}")]) == want
}

/// HEAD split in two by hunk: each part's tree is the parent plus the
/// hunks so far, the last part's tree is the original one, the author is
/// kept on every part, and a dirty worktree and index are untouched.
pub fn check_split_head_into_two_by_hunk(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new().build();
    write(&repo, "a.txt", forty_lines().as_bytes());
    write(&repo, "other.txt", b"other\n");
    commit_all(&repo, "base");
    write(&repo, "a.txt", forty_lines_edited().as_bytes());
    repo.git(&["add", "a.txt"]);
    repo.git(&[
        "commit",
        "--author",
        "Ann Example <ann@example.org>",
        "--date=2001-02-03T04:05:06+0530",
        "-m",
        "edit both ends",
    ]);
    // Dirty: one staged change, one unstaged, one untracked.
    write(&repo, "other.txt", b"staged\n");
    repo.git(&["add", "other.txt"]);
    write(&repo, "other.txt", b"staged\nunstaged\n");
    write(&repo, "untracked.txt", b"u\n");

    let repo_ref = repo.repo_ref();
    let head = head_oid(backend, &repo_ref);
    let source = backend.split_source(&repo_ref, head).unwrap();
    assert_eq!(source.commit, head);
    assert_eq!(source.message.trim_end(), "edit both ends");
    assert_eq!(
        source.parent.unwrap().to_hex(),
        repo.git(&["rev-parse", "HEAD~1"])
    );
    assert_eq!(source.files.len(), 1);
    assert_eq!(source.files[0].path, Path::new("a.txt"));
    assert_eq!(source.files[0].status, FileStatus::Modified);
    assert!(source.files[0].whole_only.is_none());
    assert_eq!(source.files[0].hunks.len(), 2);

    let original_tree = repo.git(&["rev-parse", "HEAD^{tree}"]);
    let parent = repo.git(&["rev-parse", "HEAD~1"]);
    let status_before = repo.git(&["status", "--porcelain"]);
    let cached_before = repo.git(&["diff", "--cached"]);
    let outcome = backend
        .split_commit(
            &repo_ref,
            &request(
                head,
                vec![
                    part("edit the top", vec![hunk("a.txt", 0)]),
                    part("edit the bottom\n\nwith a body", vec![hunk("a.txt", 1)]),
                ],
            ),
        )
        .unwrap();

    assert_eq!(outcome.created.len(), 2);
    assert_eq!(outcome.previous_head, head);
    assert_eq!(outcome.new_head, outcome.created[1]);
    assert_eq!(outcome.replayed, 0);
    assert_eq!(repo.git(&["rev-parse", "HEAD"]), outcome.new_head.to_hex());
    assert_eq!(repo.git(&["symbolic-ref", "HEAD"]), "refs/heads/main");
    assert_eq!(repo.git(&["rev-parse", "ORIG_HEAD"]), head.to_hex());
    let (first, second) = (outcome.created[0].to_hex(), outcome.created[1].to_hex());
    assert_eq!(repo.git(&["rev-parse", &format!("{first}~1")]), parent);
    assert_eq!(repo.git(&["rev-parse", &format!("{second}~1")]), first);
    assert_eq!(
        repo.git(&["rev-parse", &format!("{second}^{{tree}}")]),
        original_tree,
        "the last part is the original tree, verbatim"
    );
    let top_only = forty_lines().replace("line 2\n", "LINE 2\n");
    assert!(blob_is(&repo, &first, "a.txt", top_only.as_bytes()));
    assert_eq!(
        repo.git(&["log", "-1", "--format=%B", &first]),
        "edit the top"
    );
    assert_eq!(
        repo.git(&["log", "-1", "--format=%B", &second]),
        "edit the bottom\n\nwith a body"
    );
    let want_author = "Ann Example|ann@example.org|981153306 +0530";
    assert_eq!(author(&repo, &first), want_author);
    assert_eq!(author(&repo, &second), want_author);
    assert_eq!(repo.git(&["status", "--porcelain"]), status_before);
    assert_eq!(repo.git(&["diff", "--cached"]), cached_before);
    assert!(repo
        .git(&["reflog", "-1", "--format=%gs", "main"])
        .contains("split"));
    assert_no_temp_index(&repo);
}

/// Line selection on a CRLF file whose last line has no newline: the
/// partial patch keeps every `\r` and the missing final newline, so the
/// middle part's blob is byte-exact.
pub fn check_split_by_lines_keeps_crlf_and_no_newline_at_eof(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new().build();
    write(&repo, "a.txt", b"one\r\ntwo\r\nthree\r\nfour");
    commit_all(&repo, "base");
    write(&repo, "a.txt", b"ONE\r\ntwo\r\nthree\r\nFOUR");
    commit_all(&repo, "shout");

    let repo_ref = repo.repo_ref();
    let head = head_oid(backend, &repo_ref);
    let source = backend.split_source(&repo_ref, head).unwrap();
    let hunks = &source.files[0].hunks;
    assert_eq!(hunks.len(), 1);
    // -one +ONE ctx ctx -four +FOUR
    assert_eq!(hunks[0].lines.len(), 6);

    let lines = |lines: Vec<u32>| SplitPiece::Lines {
        path: "a.txt".into(),
        hunk: HunkId(0),
        lines,
    };
    let outcome = backend
        .split_commit(
            &repo_ref,
            &request(
                head,
                vec![
                    part("first line", vec![lines(vec![0, 1])]),
                    part("last line", vec![lines(vec![4, 5])]),
                ],
            ),
        )
        .unwrap();
    let first = outcome.created[0].to_hex();
    assert!(blob_is(
        &repo,
        &first,
        "a.txt",
        b"ONE\r\ntwo\r\nthree\r\nfour"
    ));
    assert!(blob_is(
        &repo,
        "HEAD",
        "a.txt",
        b"ONE\r\ntwo\r\nthree\r\nFOUR"
    ));
    assert_no_temp_index(&repo);
}

/// An older commit: the descendants, a merge among them (kept a merge,
/// with its rewritten side parent), are re-created with their own trees,
/// authors and messages; a dirty worktree is left alone.
pub fn check_split_older_commit_replays_descendants_and_merges(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new().build();
    write(&repo, "t.txt", forty_lines().as_bytes());
    commit_all(&repo, "base");
    write(&repo, "t.txt", forty_lines_edited().as_bytes());
    repo.git(&["add", "-A"]);
    repo.git(&["commit", "--date=2001-02-03T04:05:06+0530", "-m", "target"]);
    repo.git(&["checkout", "-b", "side"]);
    write(&repo, "side.txt", b"s");
    commit_all(&repo, "on side");
    repo.git(&["checkout", "main"]);
    write(&repo, "main.txt", b"m");
    commit_all(&repo, "on main");
    repo.git(&["merge", "--no-ff", "--no-edit", "side"]);
    write(&repo, "last.txt", b"l");
    commit_all(&repo, "last");
    write(&repo, "base.txt", b"dirty");

    let repo_ref = repo.repo_ref();
    let old_head = head_oid(backend, &repo_ref);
    let target = backend.resolve(&repo_ref, "HEAD~3").unwrap();
    assert_eq!(
        repo.git(&["log", "-1", "--format=%s", &target.to_hex()]),
        "target"
    );
    let before = shape(&repo);
    let status_before = repo.git(&["status", "--porcelain"]);

    let outcome = backend
        .split_commit(
            &repo_ref,
            &request(
                target,
                vec![
                    part("target, top", vec![hunk("t.txt", 0)]),
                    part("target, bottom", vec![hunk("t.txt", 1)]),
                ],
            ),
        )
        .unwrap();

    assert_eq!(outcome.previous_head, old_head);
    assert_eq!(outcome.replayed, 4, "on side, on main, the merge, last");
    assert_eq!(repo.git(&["rev-parse", "HEAD"]), outcome.new_head.to_hex());
    assert_eq!(repo.git(&["symbolic-ref", "HEAD"]), "refs/heads/main");
    let after = shape(&repo);
    assert_eq!(after.len(), before.len() + 1);
    let mut after_iter = after.iter();
    for (i, b) in before.iter().enumerate() {
        let a = after_iter.next().unwrap();
        if b.1 == "target" {
            assert_eq!(a.1, "target, bottom");
            assert_eq!(a.0, b.0, "the last part has the target's tree and author");
            let top = after_iter.next().unwrap();
            assert_eq!(top.1, "target, top");
            // The target's author date survives on both parts.
            assert!(top.0.contains("981153306 +0530"), "{}", top.0);
        } else {
            assert_eq!(
                a, b,
                "commit #{i}: a descendant keeps tree, author, subject"
            );
        }
    }
    let merge_side = repo.git(&["rev-parse", "HEAD~1^2"]);
    assert_ne!(merge_side, repo.git(&["rev-parse", "side"]));
    assert_eq!(
        repo.git(&["rev-parse", &format!("{merge_side}~1")]),
        outcome.created[1].to_hex()
    );
    assert_eq!(
        repo.git(&["rev-parse", &format!("{}~1", outcome.created[1].to_hex())]),
        outcome.created[0].to_hex()
    );
    assert_eq!(repo.git(&["status", "--porcelain"]), status_before);
    assert_eq!(repo.git(&["rev-parse", "ORIG_HEAD"]), old_head.to_hex());
    assert_no_temp_index(&repo);
}

/// A root commit splits (its parent is the empty tree) and its
/// descendant is replayed on the new root.
pub fn check_split_root_commit(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new().build();
    write(&repo, "a.txt", b"a\n");
    write(&repo, "b.txt", b"b\n");
    commit_all(&repo, "root");
    write(&repo, "c.txt", b"c\n");
    commit_all(&repo, "second");

    let repo_ref = repo.repo_ref();
    let root = backend.resolve(&repo_ref, "HEAD~1").unwrap();
    let source = backend.split_source(&repo_ref, root).unwrap();
    assert!(source.parent.is_none());
    assert_eq!(source.files.len(), 2);
    assert!(source
        .files
        .iter()
        .all(|f| f.whole_only == Some(WholeReason::Added) && f.status == FileStatus::Added));

    let outcome = backend
        .split_commit(
            &repo_ref,
            &request(
                root,
                vec![
                    part("add a", vec![file("a.txt")]),
                    part("add b", vec![file("b.txt")]),
                ],
            ),
        )
        .unwrap();
    assert_eq!(outcome.replayed, 1);
    let (first, second) = (outcome.created[0].to_hex(), outcome.created[1].to_hex());
    assert_eq!(repo.git(&["rev-list", "--parents", "-n1", &first]), first);
    assert_eq!(repo.git(&["ls-tree", "-r", "--name-only", &first]), "a.txt");
    assert_eq!(
        repo.git(&["ls-tree", "-r", "--name-only", &second]),
        "a.txt\nb.txt"
    );
    assert_eq!(repo.git(&["rev-parse", "HEAD~2"]), first);
    assert_eq!(repo.git(&["rev-parse", "HEAD~1"]), second);
    assert_eq!(repo.git(&["rev-list", "--count", "HEAD"]), "3");
    assert_eq!(repo.git(&["log", "-1", "--format=%s"]), "second");
    assert_no_temp_index(&repo);
}

/// Binary, added, deleted and mode-only files are assigned whole, next to
/// a text file split by hunk, and each part's tree shows exactly its own.
pub fn check_split_whole_only_files_binary_added_deleted_mode(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new().build();
    write(&repo, "bin.dat", &[0, 1, 2, 3, 0, 5]);
    write(&repo, "del.txt", b"gone\n");
    write(&repo, "exec.sh", b"#!/bin/sh\n");
    write(&repo, "t.txt", forty_lines().as_bytes());
    commit_all(&repo, "base");
    write(&repo, "bin.dat", &[0, 9, 9, 9, 0, 5]);
    std::fs::remove_file(repo.root().join("del.txt")).unwrap();
    write(&repo, "new.txt", b"new\n");
    write(&repo, "t.txt", forty_lines_edited().as_bytes());
    repo.git(&["add", "-A"]);
    repo.git(&["update-index", "--chmod=+x", "exec.sh"]);
    repo.git(&["commit", "-m", "everything"]);

    let repo_ref = repo.repo_ref();
    let head = head_oid(backend, &repo_ref);
    let source = backend.split_source(&repo_ref, head).unwrap();
    let reason = |p: &str| {
        source
            .files
            .iter()
            .find(|f| f.path == Path::new(p))
            .unwrap_or_else(|| panic!("{p} missing from {:?}", source.files))
            .whole_only
    };
    assert_eq!(reason("bin.dat"), Some(WholeReason::Binary));
    assert_eq!(reason("del.txt"), Some(WholeReason::Deleted));
    assert_eq!(reason("new.txt"), Some(WholeReason::Added));
    assert_eq!(reason("exec.sh"), Some(WholeReason::ModeOnly));
    assert_eq!(reason("t.txt"), None);
    let text = source
        .files
        .iter()
        .find(|f| f.path == Path::new("t.txt"))
        .unwrap();
    assert_eq!(text.hunks.len(), 2);
    assert!(source
        .files
        .iter()
        .filter(|f| f.whole_only.is_some())
        .all(|f| f.hunks.is_empty()));

    let original_tree = repo.git(&["rev-parse", "HEAD^{tree}"]);
    let outcome = backend
        .split_commit(
            &repo_ref,
            &request(
                head,
                vec![
                    part("binary and delete", vec![file("bin.dat"), file("del.txt")]),
                    part(
                        "add, mode and top",
                        vec![file("new.txt"), file("exec.sh"), hunk("t.txt", 0)],
                    ),
                    part("bottom", vec![hunk("t.txt", 1)]),
                ],
            ),
        )
        .unwrap();
    assert_eq!(outcome.created.len(), 3);
    let (c1, c2, c3) = (
        outcome.created[0].to_hex(),
        outcome.created[1].to_hex(),
        outcome.created[2].to_hex(),
    );
    assert_eq!(
        repo.git(&["rev-parse", &format!("{c3}^{{tree}}")]),
        original_tree
    );

    let mode_of = |rev: &str, path: &str| {
        repo.git(&["ls-tree", rev, "--", path])
            .split_whitespace()
            .next()
            .map(str::to_string)
    };
    // After part 1: binary changed, file deleted, nothing else.
    assert!(blob_is(&repo, &c1, "bin.dat", &[0, 9, 9, 9, 0, 5]));
    assert_eq!(mode_of(&c1, "del.txt"), None);
    assert_eq!(mode_of(&c1, "new.txt"), None);
    assert_eq!(mode_of(&c1, "exec.sh").as_deref(), Some("100644"));
    assert!(blob_is(&repo, &c1, "t.txt", forty_lines().as_bytes()));
    // After part 2: the add, the mode and the top hunk join them.
    assert_eq!(mode_of(&c2, "new.txt").as_deref(), Some("100644"));
    assert_eq!(mode_of(&c2, "exec.sh").as_deref(), Some("100755"));
    let top_only = forty_lines().replace("line 2\n", "LINE 2\n");
    assert!(blob_is(&repo, &c2, "t.txt", top_only.as_bytes()));
    assert_no_temp_index(&repo);
}

/// Every refusal is typed and changes nothing: HEAD, refs, index,
/// worktree - and no temporary index is left in the git dir.
pub fn check_split_refusals_are_typed_and_change_nothing(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new().build();
    write(&repo, "a.txt", forty_lines().as_bytes());
    commit_all(&repo, "base");
    repo.git(&["branch", "other"]);
    write(&repo, "a.txt", forty_lines_edited().as_bytes());
    write(&repo, "added.txt", b"added\n");
    commit_all(&repo, "target");
    repo.git(&["checkout", "-b", "feature"]);
    write(&repo, "e.txt", b"e");
    commit_all(&repo, "elsewhere");
    repo.git(&["checkout", "main"]);
    write(&repo, "m.txt", b"m");
    commit_all(&repo, "on main");
    repo.git(&["merge", "--no-ff", "--no-edit", "feature"]);
    let merge = repo.git(&["rev-parse", "HEAD"]);
    repo.git(&["commit", "--allow-empty", "-m", "empty"]);
    let empty = repo.git(&["rev-parse", "HEAD"]);
    repo.git(&["checkout", "other"]);
    write(&repo, "o.txt", b"o");
    commit_all(&repo, "unmerged");
    let unmerged = repo.git(&["rev-parse", "HEAD"]);
    repo.git(&["checkout", "main"]);
    write(&repo, "dirty.txt", b"dirty");

    let repo_ref = repo.repo_ref();
    let oid = |rev: &str| backend.resolve(&repo_ref, rev).unwrap();
    let target = oid("HEAD~3");
    assert_eq!(
        repo.git(&["log", "-1", "--format=%s", &target.to_hex()]),
        "target"
    );
    let good = || {
        vec![
            part("top", vec![hunk("a.txt", 0)]),
            part("bottom", vec![hunk("a.txt", 1), file("added.txt")]),
        ]
    };
    let refs_before = repo.git(&["for-each-ref"]);
    let status_before = repo.git(&["status", "--porcelain"]);
    let refused = |req: SplitCommit| match backend.split_commit(&repo_ref, &req) {
        Err(gitpoulpi_core::Error::SplitRefused(r)) => r,
        other => panic!("expected a typed refusal, got {other:?}"),
    };
    let check_unchanged = || {
        assert_eq!(repo.git(&["rev-parse", "HEAD"]), empty);
        assert_eq!(repo.git(&["for-each-ref"]), refs_before);
        assert_eq!(repo.git(&["status", "--porcelain"]), status_before);
        assert_no_temp_index(&repo);
    };

    assert_eq!(
        refused(request(oid(&unmerged), good())),
        SplitRefusal::NotOnCurrentBranch
    );
    assert_eq!(
        refused(request(oid(&merge), good())),
        SplitRefusal::MergeCommit
    );
    assert_eq!(
        refused(request(oid(&empty), good())),
        SplitRefusal::EmptyCommit
    );
    assert_eq!(
        refused(request(target, vec![part("only", vec![hunk("a.txt", 0)])])),
        SplitRefusal::TooFewParts
    );
    assert_eq!(
        refused(request(
            target,
            vec![
                part(
                    "top",
                    vec![hunk("a.txt", 0), hunk("a.txt", 1), file("added.txt")]
                ),
                part("none", vec![])
            ]
        )),
        SplitRefusal::EmptyPart
    );
    let mut blank = good();
    blank[1].message = " \r\n ".into();
    assert_eq!(refused(request(target, blank)), SplitRefusal::EmptyMessage);
    assert_eq!(
        refused(request(
            target,
            vec![
                part("top", vec![hunk("a.txt", 0)]),
                part("bottom", vec![hunk("a.txt", 1)])
            ]
        )),
        SplitRefusal::Unassigned
    );
    let mut twice = good();
    twice[0].pieces.push(hunk("a.txt", 1));
    assert_eq!(refused(request(target, twice)), SplitRefusal::AssignedTwice);
    let mut unknown = good();
    unknown[0].pieces = vec![hunk("a.txt", 9)];
    assert_eq!(
        refused(request(target, unknown)),
        SplitRefusal::UnknownPiece
    );
    let mut whole_text = good();
    whole_text[0].pieces = vec![file("a.txt")];
    assert_eq!(
        refused(request(target, whole_text)),
        SplitRefusal::AssignedTwice,
        "a whole text file overlaps its hunks"
    );
    let mut partial_add = good();
    partial_add[1].pieces = vec![hunk("a.txt", 1), hunk("added.txt", 0)];
    assert_eq!(
        refused(request(target, partial_add)),
        SplitRefusal::WholeFileOnly
    );
    check_unchanged();

    // A key that does not exist: git cannot sign, and nothing moved.
    repo.git(&["config", "gpg.format", "ssh"]);
    repo.git(&["config", "user.signingkey", "/nonexistent/gitpoulpi-key"]);
    let mut signed = request(target, good());
    signed.sign = true;
    assert_eq!(refused(signed), SplitRefusal::SigningFailed);
    check_unchanged();

    // A merge in progress, as git records it.
    std::fs::write(
        repo.root().join(".git").join("MERGE_HEAD"),
        format!("{unmerged}\n"),
    )
    .unwrap();
    assert_eq!(
        refused(request(target, good())),
        SplitRefusal::OperationInProgress
    );
    check_unchanged();
}

/// The numbering comes from `--no-renames`, so `diff.renames=copies`
/// changes nothing: a copy is an addition, a rename a delete plus an add.
pub fn check_split_source_numbering_ignores_diff_renames_config(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new().build();
    write(&repo, "big.txt", forty_lines().as_bytes());
    write(&repo, "r1.txt", b"to be renamed\nwith two lines\n");
    commit_all(&repo, "base");
    repo.git(&["config", "diff.renames", "copies"]);
    write(
        &repo,
        "big.txt",
        forty_lines().replace("line 3\n", "LINE 3\n").as_bytes(),
    );
    write(
        &repo,
        "copy.txt",
        forty_lines().replace("line 10\n", "LINE 10\n").as_bytes(),
    );
    std::fs::remove_file(repo.root().join("r1.txt")).unwrap();
    write(&repo, "r2.txt", b"to be renamed\nwith two lines\n");
    commit_all(&repo, "copy and rename");

    let repo_ref = repo.repo_ref();
    let head = head_oid(backend, &repo_ref);
    let source = backend.split_source(&repo_ref, head).unwrap();
    let names: Vec<_> = source
        .files
        .iter()
        .map(|f| f.path.to_str().unwrap())
        .collect();
    assert_eq!(names, ["big.txt", "copy.txt", "r1.txt", "r2.txt"]);
    assert_eq!(source.files[0].hunks.len(), 1);
    assert_eq!(source.files[0].hunks[0].id, HunkId(0));
    assert_eq!(source.files[1].whole_only, Some(WholeReason::Added));
    assert_eq!(source.files[2].whole_only, Some(WholeReason::Deleted));
    assert_eq!(source.files[3].whole_only, Some(WholeReason::Added));

    let original_tree = repo.git(&["rev-parse", "HEAD^{tree}"]);
    backend
        .split_commit(
            &repo_ref,
            &request(
                head,
                vec![
                    part("edit big", vec![hunk("big.txt", 0)]),
                    part(
                        "copy and rename",
                        vec![file("copy.txt"), file("r1.txt"), file("r2.txt")],
                    ),
                ],
            ),
        )
        .unwrap();
    assert_eq!(repo.git(&["rev-parse", "HEAD^{tree}"]), original_tree);
    assert_no_temp_index(&repo);
}

/// `sign: true` signs the parts and the replayed descendants (ssh
/// signing with a generated key); an unsigned request never signs, even
/// where `commit.gpgSign` is on. Skipped without `ssh-keygen`.
pub fn check_split_signs_when_asked(backend: &dyn GitBackend) {
    let keys = tempfile::tempdir().unwrap();
    let key = keys.path().join("key");
    let generated = std::process::Command::new("ssh-keygen")
        .args(["-q", "-t", "ed25519", "-N", "", "-C", "split-test", "-f"])
        .arg(&key)
        .output();
    if !matches!(&generated, Ok(out) if out.status.success()) {
        eprintln!("SKIPPED: ssh-keygen is not available");
        return;
    }
    let repo = TestRepoBuilder::new().build();
    write(&repo, "a.txt", forty_lines().as_bytes());
    commit_all(&repo, "base");
    write(&repo, "a.txt", forty_lines_edited().as_bytes());
    commit_all(&repo, "target");
    write(&repo, "later.txt", b"later\n");
    commit_all(&repo, "later");
    repo.git(&["config", "gpg.format", "ssh"]);
    repo.git(&["config", "user.signingkey", key.to_str().unwrap()]);

    // Plain git must be able to sign here (it runs ssh-keygen itself, and
    // that depends on the machine: key file permissions, the tool git finds
    // on its PATH). When it cannot, the engine cannot either, and that is
    // not what this contract checks.
    let probe = std::process::Command::new("git")
        .current_dir(repo.root())
        .args(["commit-tree", "-S", "-m", "probe", "HEAD^{tree}"])
        .output();
    if !matches!(&probe, Ok(out) if out.status.success()) {
        eprintln!("SKIPPED: git cannot sign with the generated ssh key on this machine");
        return;
    }

    let repo_ref = repo.repo_ref();
    let target = backend.resolve(&repo_ref, "HEAD~1").unwrap();
    let parts = || {
        vec![
            part("top", vec![hunk("a.txt", 0)]),
            part("bottom", vec![hunk("a.txt", 1)]),
        ]
    };
    let mut req = request(target, parts());
    req.sign = true;
    let outcome = backend.split_commit(&repo_ref, &req).unwrap();
    assert_eq!(outcome.replayed, 1);
    for commit in outcome.created.iter().chain([&outcome.new_head]) {
        let raw = repo.git(&["cat-file", "commit", &commit.to_hex()]);
        assert!(raw.contains("gpgsig"), "{commit:?} is not signed:\n{raw}");
    }

    // Not asked: unsigned, although the repository signs by default.
    repo.git(&["config", "commit.gpgsign", "false"]);
    write(&repo, "p.txt", b"p\n");
    write(&repo, "q.txt", b"q\n");
    commit_all(&repo, "two files");
    repo.git(&["config", "commit.gpgsign", "true"]);
    let two = head_oid(backend, &repo_ref);
    let unsigned = backend
        .split_commit(
            &repo_ref,
            &request(
                two,
                vec![
                    part("p", vec![file("p.txt")]),
                    part("q", vec![file("q.txt")]),
                ],
            ),
        )
        .unwrap();
    for commit in &unsigned.created {
        let raw = repo.git(&["cat-file", "commit", &commit.to_hex()]);
        assert!(!raw.contains("gpgsig"), "unsigned request signed:\n{raw}");
    }
}

/// A whole text file is one piece: every hunk of it goes to that part.
pub fn check_split_by_whole_text_file(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new().build();
    write(&repo, "a.txt", forty_lines().as_bytes());
    write(&repo, "b.txt", forty_lines().as_bytes());
    commit_all(&repo, "base");
    write(&repo, "a.txt", forty_lines_edited().as_bytes());
    write(&repo, "b.txt", forty_lines_edited().as_bytes());
    commit_all(&repo, "both");

    let repo_ref = repo.repo_ref();
    let head = head_oid(backend, &repo_ref);
    let original_tree = repo.git(&["rev-parse", "HEAD^{tree}"]);
    let outcome = backend
        .split_commit(
            &repo_ref,
            &request(
                head,
                vec![
                    part("a", vec![file("a.txt")]),
                    part("b", vec![file("b.txt")]),
                ],
            ),
        )
        .unwrap();
    let first = outcome.created[0].to_hex();
    assert!(blob_is(
        &repo,
        &first,
        "a.txt",
        forty_lines_edited().as_bytes()
    ));
    assert!(blob_is(&repo, &first, "b.txt", forty_lines().as_bytes()));
    assert_eq!(repo.git(&["rev-parse", "HEAD^{tree}"]), original_tree);
    assert_no_temp_index(&repo);
}
