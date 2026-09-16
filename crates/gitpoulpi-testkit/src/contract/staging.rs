use crate::TestRepoBuilder;
use gitpoulpi_core::domain::{
    CommitOptions, DiffTarget, DiscardTarget, Hunk, LineOrigin, LineSelector, LogOptions,
    StageTarget,
};
use gitpoulpi_core::GitBackend;

pub fn check_stage_and_commit(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new().with_initial().build();
    let repo_ref = repo.repo_ref();
    std::fs::write(repo.root().join("new.txt"), "hello").unwrap();

    backend
        .stage(&repo_ref, &StageTarget::File("new.txt".into()))
        .unwrap();
    let oid = backend
        .commit(&repo_ref, "add new.txt", &CommitOptions::default())
        .unwrap();

    let log = backend.log(&repo_ref, &LogOptions::default()).unwrap();
    assert_eq!(log.nodes[0].id, oid);
    assert_eq!(log.nodes.len(), 2);
}

/// `CommitOptions::author`: the TUI's `general.active_profile` feature
/// rests on this field reaching the commit, identically on both backends.
pub fn check_commit_honours_an_author_override(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new().with_initial().build();
    let repo_ref = repo.repo_ref();
    std::fs::write(repo.root().join("new.txt"), "hello").unwrap();
    backend
        .stage(&repo_ref, &StageTarget::File("new.txt".into()))
        .unwrap();

    let oid = backend
        .commit(
            &repo_ref,
            "add new.txt",
            &CommitOptions {
                author: Some(gitpoulpi_core::domain::CommitAuthor {
                    name: "Profile Author".into(),
                    email: "profile@example.com".into(),
                }),
                ..CommitOptions::default()
            },
        )
        .unwrap();

    let log = backend.log(&repo_ref, &LogOptions::default()).unwrap();
    let node = log.nodes.iter().find(|n| n.id == oid).unwrap();
    assert_eq!(node.author_name, "Profile Author");
    assert_eq!(node.author_email, "profile@example.com");
}

/// `GitBackend::discard`: several paths, of every shape a folder discard
/// or a "discard everything" mixes together, in one call.
///
/// - A tracked file with unstaged changes only: worktree content goes back
///   to `HEAD`.
/// - A tracked file with *staged* changes: both the index and the worktree
///   go back to `HEAD`, never half-reset.
/// - A brand-new file, staged but never committed: removed from the index
///   and deleted from disk.
/// - A brand-new file never staged (plain untracked): deleted from disk,
///   whether or not it happened to be staged first.
///
/// A fifth, unmentioned path is left alone, pinning that `paths` is exactly
/// what gets discarded.
pub fn check_discard_several_paths_at_once(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .commit_file("tracked.txt", "original\n", "add tracked.txt")
        .commit_file("staged.txt", "original\n", "add staged.txt")
        .commit_file("untouched.txt", "original\n", "add untouched.txt")
        .write_worktree("tracked.txt", b"modified\n")
        .write_worktree("staged.txt", b"modified\n")
        .stage("staged.txt")
        .write_worktree("new-staged.txt", b"brand new\n")
        .stage("new-staged.txt")
        .write_worktree("new-untracked.txt", b"also new\n")
        .write_worktree("untouched.txt", b"leave me alone\n")
        .build();
    let repo_ref = repo.repo_ref();

    backend
        .discard(
            &repo_ref,
            &[
                "tracked.txt".to_string(),
                "staged.txt".to_string(),
                "new-staged.txt".to_string(),
                "new-untracked.txt".to_string(),
            ],
        )
        .unwrap();

    assert_eq!(
        std::fs::read_to_string(repo.root().join("tracked.txt")).unwrap(),
        "original\n",
        "unstaged changes to a tracked file go back to HEAD"
    );
    assert_eq!(
        std::fs::read_to_string(repo.root().join("staged.txt")).unwrap(),
        "original\n",
        "staged changes to a tracked file go back to HEAD too"
    );
    assert!(
        !repo.root().join("new-staged.txt").exists(),
        "a staged-but-uncommitted new file is deleted, not left staged"
    );
    assert!(
        !repo.root().join("new-untracked.txt").exists(),
        "a plain untracked file is deleted"
    );
    assert_eq!(
        std::fs::read_to_string(repo.root().join("untouched.txt")).unwrap(),
        "leave me alone\n",
        "a path not passed to discard is left exactly as it was"
    );

    let status = backend.status(&repo_ref).unwrap();
    let dirty_paths: Vec<_> = status
        .files
        .iter()
        .map(|f| f.path.to_string_lossy().into_owned())
        .collect();
    for clean in [
        "tracked.txt",
        "staged.txt",
        "new-staged.txt",
        "new-untracked.txt",
    ] {
        assert!(
            !dirty_paths.contains(&clean.to_string()),
            "{clean} should be clean after being discarded: {dirty_paths:?}"
        );
    }
    assert!(
        dirty_paths.contains(&"untouched.txt".to_string()),
        "the path never passed to discard should still show as dirty: {dirty_paths:?}"
    );
}

/// `git add -A` semantics for `stage(All)`: deletions, renames, nested
/// new files and modifications are all staged; an ignored file is not.
/// (Native index staging has to reproduce this by hand.)
pub fn check_stage_all_matches_git_add_dash_a(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .commit_file(".gitignore", "*.log\n", "ignore logs")
        .commit_file("del.txt", "gone soon\n", "add del")
        .commit_file(
            "old.txt",
            "line one\nline two\nline three\nline four\n",
            "add old",
        )
        .commit_file("mod.txt", "before\n", "add mod")
        .build();
    let repo_ref = repo.repo_ref();
    std::fs::remove_file(repo.root().join("del.txt")).unwrap();
    std::fs::rename(repo.root().join("old.txt"), repo.root().join("renamed.txt")).unwrap();
    std::fs::write(repo.root().join("mod.txt"), "after\n").unwrap();
    std::fs::write(repo.root().join("fresh.txt"), "fresh\n").unwrap();
    std::fs::create_dir_all(repo.root().join("sub/deeper")).unwrap();
    std::fs::write(repo.root().join("sub/deeper/deep.txt"), "deep\n").unwrap();
    std::fs::write(repo.root().join("ignored.log"), "noise\n").unwrap();
    std::fs::write(repo.root().join("sub/also.log"), "noise\n").unwrap();

    backend.stage(&repo_ref, &StageTarget::All).unwrap();

    let staged = repo
        .git(&["diff", "--cached", "--name-status"])
        .replace("\r\n", "\n");
    for expected in [
        "D\tdel.txt",
        "M\tmod.txt",
        "A\tfresh.txt",
        "A\tsub/deeper/deep.txt",
    ] {
        assert!(
            staged.lines().any(|l| l == expected),
            "{expected} missing from staged changes:\n{staged}"
        );
    }
    assert!(
        staged
            .lines()
            .any(|l| l.starts_with('R') && l.ends_with("old.txt\trenamed.txt")),
        "the rename is staged as old.txt -> renamed.txt:\n{staged}"
    );
    assert!(
        !staged.contains(".log"),
        "ignored files are not added by stage(All):\n{staged}"
    );
    let status = backend.status(&repo_ref).unwrap();
    for f in &status.files {
        assert!(
            f.worktree.is_none(),
            "nothing left unstaged after stage(All): {:?}",
            f.path
        );
    }
}

/// `stage`/`unstage` of a File and of a Folder: a deleted tracked file is
/// staged as a deletion, a folder takes everything under it and nothing
/// outside it, and unstaging puts the index back to `HEAD` for exactly
/// those paths.
pub fn check_stage_and_unstage_file_and_folder(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .commit_file("dir/a.txt", "a\n", "add a")
        .commit_file("dir/b.txt", "b\n", "add b")
        .commit_file("other.txt", "o\n", "add other")
        .build();
    let repo_ref = repo.repo_ref();
    std::fs::remove_file(repo.root().join("dir/a.txt")).unwrap();
    std::fs::write(repo.root().join("dir/b.txt"), "b2\n").unwrap();
    std::fs::write(repo.root().join("dir/new.txt"), "n\n").unwrap();
    std::fs::write(repo.root().join("other.txt"), "o2\n").unwrap();

    let cached = || {
        repo.git(&["diff", "--cached", "--name-status"])
            .replace("\r\n", "\n")
    };

    backend
        .stage(&repo_ref, &StageTarget::File("dir/a.txt".into()))
        .unwrap();
    assert_eq!(cached().trim(), "D\tdir/a.txt");

    backend
        .stage(&repo_ref, &StageTarget::Folder("dir".into()))
        .unwrap();
    let c = cached();
    assert!(c.lines().any(|l| l == "D\tdir/a.txt"), "{c}");
    assert!(c.lines().any(|l| l == "M\tdir/b.txt"), "{c}");
    assert!(c.lines().any(|l| l == "A\tdir/new.txt"), "{c}");
    assert!(!c.contains("other.txt"), "{c}");

    backend
        .unstage(&repo_ref, &StageTarget::File("dir/new.txt".into()))
        .unwrap();
    let c = cached();
    assert!(!c.contains("new.txt"), "{c}");
    assert!(c.contains("dir/b.txt"), "{c}");
    assert!(repo.root().join("dir/new.txt").exists());

    backend
        .unstage(&repo_ref, &StageTarget::Folder("dir".into()))
        .unwrap();
    assert_eq!(cached().trim(), "");
    assert_eq!(
        std::fs::read_to_string(repo.root().join("dir/b.txt")).unwrap(),
        "b2\n",
        "unstaging never touches the working tree"
    );
}

/// `unstage(All)` in a repository with no commit yet: there is no `HEAD`
/// to reset to, so it empties the index. The files stay on disk.
pub fn check_unstage_all_on_an_unborn_branch(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new().build();
    let repo_ref = repo.repo_ref();
    std::fs::write(repo.root().join("a.txt"), "a\n").unwrap();
    std::fs::create_dir_all(repo.root().join("d")).unwrap();
    std::fs::write(repo.root().join("d/b.txt"), "b\n").unwrap();

    backend.stage(&repo_ref, &StageTarget::All).unwrap();
    assert_eq!(repo.git(&["ls-files"]).lines().count(), 2);

    backend.unstage(&repo_ref, &StageTarget::All).unwrap();
    assert_eq!(repo.git(&["ls-files"]).trim(), "");
    assert!(repo.root().join("a.txt").exists());
    assert!(repo.root().join("d/b.txt").exists());

    backend
        .stage(&repo_ref, &StageTarget::File("a.txt".into()))
        .unwrap();
    backend
        .unstage(&repo_ref, &StageTarget::File("a.txt".into()))
        .unwrap();
    assert_eq!(repo.git(&["ls-files"]).trim(), "");
}

/// `discard` over a mix of every shape at once, including nested paths,
/// a tracked file deleted from the worktree (restored), a staged
/// deletion (restored), a path with a space, and a path that does not
/// exist (ignored, not an error). A dirty path not named stays dirty.
pub fn check_discard_a_mixed_batch(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .commit_file("d/one.txt", "1\n", "add one")
        .commit_file("d/two.txt", "2\n", "add two")
        .commit_file("d/gone.txt", "g\n", "add gone")
        .commit_file("d/staged del.txt", "s\n", "add staged del")
        .commit_file("keep.txt", "k\n", "add keep")
        .write_worktree("d/one.txt", b"1 changed\n")
        .write_worktree("d/two.txt", b"2 staged\n")
        .stage("d/two.txt")
        .write_worktree("d/two.txt", b"2 staged then more\n")
        .write_worktree("d/untracked.txt", b"u\n")
        .write_worktree("keep.txt", b"k changed\n")
        .build();
    let repo_ref = repo.repo_ref();
    std::fs::remove_file(repo.root().join("d/gone.txt")).unwrap();
    repo.git(&["rm", "-q", "--", "d/staged del.txt"]);

    let paths: Vec<String> = [
        "d/one.txt",
        "d/two.txt",
        "d/gone.txt",
        "d/staged del.txt",
        "d/untracked.txt",
        "d/never-existed.txt",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    backend.discard(&repo_ref, &paths).unwrap();

    let read = |p: &str| std::fs::read_to_string(repo.root().join(p)).unwrap();
    assert_eq!(read("d/one.txt"), "1\n");
    assert_eq!(read("d/two.txt"), "2\n");
    assert_eq!(read("d/gone.txt"), "g\n");
    assert_eq!(read("d/staged del.txt"), "s\n");
    assert!(!repo.root().join("d/untracked.txt").exists());
    assert_eq!(read("keep.txt"), "k changed\n");
    let status = repo.git(&["status", "--porcelain"]).replace("\r\n", "\n");
    assert_eq!(status.trim(), "M keep.txt", "only keep.txt stays dirty");
}

/// `paths` empty is a no-op success, not an error - a caller that
/// computed an empty folder selection should not have to special-case it.
pub fn check_discard_with_no_paths_is_a_harmless_no_op(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .with_initial()
        .write_worktree("dirty.txt", b"still here\n")
        .build();
    let repo_ref = repo.repo_ref();

    backend.discard(&repo_ref, &[]).unwrap();

    assert!(repo.root().join("dirty.txt").exists());
}

/// The hunks of `path` in `target`, as the backend under test numbers
/// them - the numbering every `StageTarget::{Hunk,Line}` and
/// `DiscardTarget` is resolved in, and the one a frontend shows.
fn hunks_of(
    backend: &dyn GitBackend,
    repo: &gitpoulpi_core::domain::RepoRef,
    target: DiffTarget,
    path: &str,
) -> Vec<Hunk> {
    backend
        .diff(repo, target)
        .unwrap()
        .into_iter()
        .find(|f| f.path.to_string_lossy().replace('\\', "/") == path)
        .map(|f| f.hunks)
        .unwrap_or_default()
}

/// Index into `hunk.lines` of the first line with this origin and content.
fn line_index(hunk: &Hunk, origin: LineOrigin, content: &str) -> u32 {
    hunk.lines
        .iter()
        .position(|l| l.origin == origin && l.content.trim_end_matches('\r') == content)
        .unwrap_or_else(|| panic!("no {origin:?} {content:?} in {:?}", hunk.lines)) as u32
}

/// The index's copy of `path` (`git show :path`). `FixtureRepo::git`
/// trims its output, so the final newline every file here ends with is
/// put back.
fn index_copy(repo: &crate::FixtureRepo, path: &str) -> String {
    format!("{}\n", repo.git(&["show", &format!(":{path}")]))
}

fn twenty_lines() -> String {
    (1..=20).map(|n| format!("line{n}\n")).collect()
}

/// Discard Hunk reverts that hunk and only that hunk, in the working tree
/// only: the other hunk's change survives, and the index is not touched.
pub fn check_discard_hunk_reverts_only_that_hunk(backend: &dyn GitBackend) {
    let original = twenty_lines();
    let edited = original
        .replace("line2\n", "line2-changed\n")
        .replace("line19\n", "line19-changed\n");
    let repo = TestRepoBuilder::new()
        .commit_file("multi.txt", &original, "multi")
        .write_worktree("multi.txt", edited.as_bytes())
        .build();
    let repo_ref = repo.repo_ref();
    let hunks = hunks_of(
        backend,
        &repo_ref,
        DiffTarget::WorkingTreeVsIndex,
        "multi.txt",
    );
    assert_eq!(hunks.len(), 2, "two edits 17 lines apart are two hunks");

    backend
        .discard_part(
            &repo_ref,
            &DiscardTarget::Hunk("multi.txt".into(), hunks[0].id),
        )
        .unwrap();

    assert_eq!(
        std::fs::read_to_string(repo.root().join("multi.txt")).unwrap(),
        original.replace("line19\n", "line19-changed\n"),
        "the first hunk is reverted, the second survives"
    );
    assert_eq!(
        index_copy(&repo, "multi.txt"),
        original,
        "the index is not touched"
    );
}

fn discard_lines(
    backend: &dyn GitBackend,
    repo: &gitpoulpi_core::domain::RepoRef,
    path: &str,
    hunk: &Hunk,
    line_indices: Vec<u32>,
) {
    backend
        .discard_part(
            repo,
            &DiscardTarget::Line(
                path.into(),
                LineSelector {
                    hunk: hunk.id,
                    line_indices,
                },
            ),
        )
        .unwrap();
}

/// A selection spanning a removed line and the added line that replaced
/// it reverts that change; a lone added line is removed; a lone removed
/// line is put back - all inside one hunk, every other change in it
/// surviving.
pub fn check_discard_lines_spanning_adds_and_deletes(backend: &dyn GitBackend) {
    let original = "a\nb\nc\nd\ne\n";
    // b -> B, c -> C, and a new line after d: one hunk, three changes.
    let edited = "a\nB\nC\nd\nnew\ne\n";
    let repo = TestRepoBuilder::new()
        .commit_file("f.txt", original, "f")
        .write_worktree("f.txt", edited.as_bytes())
        .build();
    let repo_ref = repo.repo_ref();
    let read = || std::fs::read_to_string(repo.root().join("f.txt")).unwrap();
    let hunk = || hunks_of(backend, &repo_ref, DiffTarget::WorkingTreeVsIndex, "f.txt")[0].clone();

    let h = hunk();
    let selection = vec![
        line_index(&h, LineOrigin::Delete, "b"),
        line_index(&h, LineOrigin::Add, "B"),
    ];
    discard_lines(backend, &repo_ref, "f.txt", &h, selection);
    assert_eq!(
        read(),
        "a\nb\nC\nd\nnew\ne\n",
        "b -> B reverted, nothing else"
    );

    // A lone added line: removed.
    let h = hunk();
    let selection = vec![line_index(&h, LineOrigin::Add, "new")];
    discard_lines(backend, &repo_ref, "f.txt", &h, selection);
    assert_eq!(read(), "a\nb\nC\nd\ne\n");

    // A lone removed line: put back, beside the line that replaced it.
    let h = hunk();
    let selection = vec![line_index(&h, LineOrigin::Delete, "c")];
    discard_lines(backend, &repo_ref, "f.txt", &h, selection);
    assert_eq!(read(), "a\nb\nc\nC\nd\ne\n");
    assert_eq!(
        index_copy(&repo, "f.txt"),
        original,
        "the index is not touched"
    );
}

/// The first line of a file, and a line appended after its last one: the
/// hunks there have context on one side only, and the partial patch's
/// header has to stay valid without it.
pub fn check_discard_lines_at_the_file_edges(backend: &dyn GitBackend) {
    let original = twenty_lines();
    // Line 1 removed, one line appended at the end, one more changed
    // beside it so the end hunk also has a change that must survive.
    let edited = original
        .replacen("line1\n", "", 1)
        .replace("line19\n", "line19-changed\n")
        + "tail\n";
    let repo = TestRepoBuilder::new()
        .commit_file("edge.txt", &original, "edge")
        .write_worktree("edge.txt", edited.as_bytes())
        .build();
    let repo_ref = repo.repo_ref();
    let read = || std::fs::read_to_string(repo.root().join("edge.txt")).unwrap();
    let hunks = || {
        hunks_of(
            backend,
            &repo_ref,
            DiffTarget::WorkingTreeVsIndex,
            "edge.txt",
        )
    };

    let first = hunks()[0].clone();
    assert_eq!(hunks().len(), 2, "{:?}", hunks());
    let selection = vec![line_index(&first, LineOrigin::Delete, "line1")];
    discard_lines(backend, &repo_ref, "edge.txt", &first, selection);
    assert_eq!(
        read(),
        original.replace("line19\n", "line19-changed\n") + "tail\n",
        "the first line is back"
    );

    let last = hunks().last().unwrap().clone();
    let selection = vec![line_index(&last, LineOrigin::Add, "tail")];
    discard_lines(backend, &repo_ref, "edge.txt", &last, selection);
    assert_eq!(
        read(),
        original.replace("line19\n", "line19-changed\n"),
        "the appended line is gone, the change beside it survives"
    );
}

/// A file whose last line has no newline, and one with CRLF endings:
/// discarding must give back exactly the bytes that were there - no
/// newline invented, no `\r` lost.
pub fn check_discard_part_keeps_line_endings_exactly(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .commit_file("noeol.txt", "one\ntwo\nthree", "no final newline")
        .commit_file("crlf.txt", "one\r\ntwo\r\nthree\r\n", "crlf")
        .write_worktree("noeol.txt", b"one\ntwo\nTHREE")
        .write_worktree("crlf.txt", b"one\r\nTWO\r\nthree\r\n")
        .build();
    let repo_ref = repo.repo_ref();

    for (path, original) in [
        ("noeol.txt", &b"one\ntwo\nthree"[..]),
        ("crlf.txt", &b"one\r\ntwo\r\nthree\r\n"[..]),
    ] {
        let hunk = &hunks_of(backend, &repo_ref, DiffTarget::WorkingTreeVsIndex, path)[0];
        backend
            .discard_part(&repo_ref, &DiscardTarget::Hunk(path.into(), hunk.id))
            .unwrap_or_else(|e| panic!("{path}: {e}"));
        assert_eq!(
            std::fs::read(repo.root().join(path)).unwrap(),
            original,
            "{path}: the exact original bytes"
        );
    }
}

/// The working tree goes back to the *index*, not to `HEAD`: a staged
/// change survives discarding the unstaged edit made on top of it. And a
/// hunk id that names nothing is an error that changes nothing.
pub fn check_discard_part_reverts_to_the_index_and_refuses_an_unknown_hunk(
    backend: &dyn GitBackend,
) {
    let repo = TestRepoBuilder::new()
        .commit_file("s.txt", "a\nb\nc\n", "s")
        .write_worktree("s.txt", b"a\nstaged\nc\n")
        .stage("s.txt")
        .write_worktree("s.txt", b"a\nstaged\nc\nmore\n")
        .build();
    let repo_ref = repo.repo_ref();

    let unknown = backend.discard_part(
        &repo_ref,
        &DiscardTarget::Hunk("s.txt".into(), gitpoulpi_core::domain::HunkId(7)),
    );
    assert!(unknown.is_err(), "an unknown hunk is an error");
    assert_eq!(
        std::fs::read_to_string(repo.root().join("s.txt")).unwrap(),
        "a\nstaged\nc\nmore\n",
        "...that changes nothing"
    );

    let hunk = &hunks_of(backend, &repo_ref, DiffTarget::WorkingTreeVsIndex, "s.txt")[0];
    backend
        .discard_part(&repo_ref, &DiscardTarget::Hunk("s.txt".into(), hunk.id))
        .unwrap();
    assert_eq!(
        std::fs::read_to_string(repo.root().join("s.txt")).unwrap(),
        "a\nstaged\nc\n",
        "back to the staged content, not to HEAD"
    );
    assert_eq!(index_copy(&repo, "s.txt"), "a\nstaged\nc\n");
}

/// Unstaging one line of a staged hunk that holds several changes takes
/// only that line out of the index - the Staged diff's line-level
/// Unstage. The reverse patch has to be built the other way round from
/// the forward one: in the index an unselected addition is *context*,
/// not absent.
pub fn check_unstage_one_line_of_a_hunk_with_several_changes(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .commit_file("u.txt", "a\nb\nc\n", "u")
        .write_worktree("u.txt", b"a\nnew1\nb\nnew2\nc\n")
        .stage("u.txt")
        .build();
    let repo_ref = repo.repo_ref();
    let hunks = hunks_of(backend, &repo_ref, DiffTarget::IndexVsHead, "u.txt");
    assert_eq!(hunks.len(), 1);

    backend
        .unstage(
            &repo_ref,
            &StageTarget::Line(
                "u.txt".into(),
                LineSelector {
                    hunk: hunks[0].id,
                    line_indices: vec![line_index(&hunks[0], LineOrigin::Add, "new1")],
                },
            ),
        )
        .unwrap();

    assert_eq!(index_copy(&repo, "u.txt"), "a\nb\nnew2\nc\n");
    assert_eq!(
        std::fs::read_to_string(repo.root().join("u.txt")).unwrap(),
        "a\nnew1\nb\nnew2\nc\n",
        "unstaging never touches the working tree"
    );
}

/// Staging one change out of a hunk that holds several - the forward
/// twin of the check above, pinned beside it so the two patch directions
/// cannot be "fixed" into each other.
pub fn check_stage_one_change_of_a_hunk_with_several_changes(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .commit_file("t.txt", "a\nb\nc\n", "t")
        .write_worktree("t.txt", b"a\nnew1\nB\nc\n")
        .build();
    let repo_ref = repo.repo_ref();
    let hunk = &hunks_of(backend, &repo_ref, DiffTarget::WorkingTreeVsIndex, "t.txt")[0];

    backend
        .stage(
            &repo_ref,
            &StageTarget::Line(
                "t.txt".into(),
                LineSelector {
                    hunk: hunk.id,
                    line_indices: vec![
                        line_index(hunk, LineOrigin::Delete, "b"),
                        line_index(hunk, LineOrigin::Add, "B"),
                    ],
                },
            ),
        )
        .unwrap();

    assert_eq!(index_copy(&repo, "t.txt"), "a\nB\nc\n");
}

/// A file git has never seen has no hunks in `diff` (git diffs tracked
/// files), so the frontends show it as one all-added hunk numbered
/// `HunkId(0)`, its lines indexed like the file's. That numbering is what
/// `StageTarget::{Hunk,Line}` and `DiscardTarget` must accept for it:
/// staging the hunk stages the whole new file, staging lines puts just those
/// lines in the index, and discarding lines removes them from the working
/// copy (discarding the hunk deletes the file).
pub fn check_stage_and_discard_parts_of_an_untracked_file(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .with_initial()
        .write_worktree("fresh.txt", b"one\ntwo\nthree\n")
        .write_worktree("second.txt", b"x\ny\n")
        .build();
    let repo_ref = repo.repo_ref();
    let hunk = gitpoulpi_core::domain::HunkId(0);
    let read = |name: &str| std::fs::read_to_string(repo.root().join(name)).unwrap();
    let lines = |indices: &[u32]| LineSelector {
        hunk,
        line_indices: indices.to_vec(),
    };

    // Discard one line: only that line leaves the working copy.
    backend
        .discard_part(
            &repo_ref,
            &DiscardTarget::Line("fresh.txt".into(), lines(&[1])),
        )
        .unwrap();
    assert_eq!(read("fresh.txt"), "one\nthree\n");

    // Stage one line: the file enters the index with just that line.
    backend
        .stage(
            &repo_ref,
            &StageTarget::Line("fresh.txt".into(), lines(&[0])),
        )
        .unwrap();
    assert_eq!(index_copy(&repo, "fresh.txt"), "one\n");
    assert_eq!(
        read("fresh.txt"),
        "one\nthree\n",
        "staging never touches the working copy"
    );

    // Stage the hunk of the other file: the whole new file is staged.
    backend
        .stage(&repo_ref, &StageTarget::Hunk("second.txt".into(), hunk))
        .unwrap();
    assert_eq!(index_copy(&repo, "second.txt"), "x\ny\n");

    // Discard the hunk of a third untracked file: it is deleted.
    std::fs::write(repo.root().join("third.txt"), "p\nq\n").unwrap();
    backend
        .discard_part(&repo_ref, &DiscardTarget::Hunk("third.txt".into(), hunk))
        .unwrap();
    assert!(!repo.root().join("third.txt").exists());
}

/// Installs a `pre-commit` hook that refuses every commit.
#[cfg(unix)]
fn install_failing_pre_commit(root: &std::path::Path) {
    use std::os::unix::fs::PermissionsExt;
    let hooks = root.join(".git").join("hooks");
    std::fs::create_dir_all(&hooks).unwrap();
    let hook = hooks.join("pre-commit");
    std::fs::write(&hook, "#!/bin/sh\necho hook-refused >&2\nexit 1\n").unwrap();
    std::fs::set_permissions(&hook, std::fs::Permissions::from_mode(0o755)).unwrap();
}

/// `CommitOptions::no_verify`: a repository's `pre-commit` hook blocks a
/// commit (on both backends the hooks of the system commit run), and
/// `no_verify` commits past it, leaving the work in the commit.
#[cfg(unix)]
pub fn check_commit_runs_hooks_unless_no_verify(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new().with_initial().build();
    let repo_ref = repo.repo_ref();
    install_failing_pre_commit(repo.root());
    std::fs::write(repo.root().join("new.txt"), "hello").unwrap();
    backend
        .stage(&repo_ref, &StageTarget::File("new.txt".into()))
        .unwrap();
    let before = backend.log(&repo_ref, &LogOptions::default()).unwrap();

    let refused = backend.commit(&repo_ref, "blocked", &CommitOptions::default());
    assert!(refused.is_err(), "the failing hook must stop the commit");
    assert_eq!(
        backend
            .log(&repo_ref, &LogOptions::default())
            .unwrap()
            .nodes
            .len(),
        before.nodes.len(),
        "nothing was committed"
    );

    let oid = backend
        .commit(
            &repo_ref,
            "past the hook",
            &CommitOptions {
                no_verify: true,
                ..CommitOptions::default()
            },
        )
        .expect("no_verify skips the pre-commit hook");
    let log = backend.log(&repo_ref, &LogOptions::default()).unwrap();
    assert_eq!(log.nodes[0].id, oid);
    assert_eq!(log.nodes[0].summary, "past the hook");
    assert_eq!(
        repo.git(&["show", "--name-only", "--format=", "HEAD"])
            .trim(),
        "new.txt"
    );
}

/// Needs a `sh` hook: run on Unix only.
#[cfg(not(unix))]
pub fn check_commit_runs_hooks_unless_no_verify(_backend: &dyn GitBackend) {}
