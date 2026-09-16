use super::{head_oid, rev};
use crate::TestRepoBuilder;
use gitpoulpi_core::domain::{
    BlameContent, BlobContent, BlobSource, DiffTarget, LineOrigin, LogOptions, Oid, PatchSource,
    StageTarget,
};
use gitpoulpi_core::GitBackend;

pub fn check_diff_working_tree_vs_index(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new().with_initial().build();
    let repo_ref = repo.repo_ref();
    std::fs::write(repo.root().join("initial.txt"), "changed").unwrap();

    let diffs = backend
        .diff(
            &repo_ref,
            gitpoulpi_core::domain::DiffTarget::WorkingTreeVsIndex,
        )
        .unwrap();
    assert_eq!(diffs.len(), 1);
    assert_eq!(diffs[0].path.to_str(), Some("initial.txt"));
    assert_eq!(diffs[0].hunks.len(), 1);
}

// -- what a commit changed -----------------------------

/// A four-commit history plus a merge, and the ids of each commit:
/// `root` adds `keep.txt`, `edit` changes its middle line, `add` creates
/// `dir/new.txt`, `remove` deletes it, `binary` adds a file with a NUL
/// in it, `multi` touches three files at once, and `merge` brings in
/// `side.txt` from a branch while `main.txt` landed on main.
struct CommitDiffFixture {
    _repo: crate::FixtureRepo,
    repo_ref: gitpoulpi_core::domain::RepoRef,
    root: Oid,
    edit: Oid,
    add: Oid,
    remove: Oid,
    binary: Oid,
    multi: Oid,
    merge: Oid,
}

fn commit_diff_fixture(backend: &dyn GitBackend) -> CommitDiffFixture {
    let repo = TestRepoBuilder::new()
        .commit_file("keep.txt", "1\n2\n3\n", "root")
        .commit_file("keep.txt", "1\nTWO\n3\n", "edit")
        .commit_file("dir/new.txt", "x\n", "add")
        .commit_removal("dir/new.txt", "remove")
        .commit_bytes("blob.bin", &[0, 1, 2, 0, 3], "binary")
        .write_worktree("z.txt", b"z\n")
        .write_worktree("a/b.txt", b"b\n")
        .stage("z.txt")
        .stage("a/b.txt")
        .commit("multi")
        .branch("side")
        .commit_file("side.txt", "side\n", "side work")
        .checkout("main")
        .commit_file("main.txt", "main\n", "main work")
        .build();
    repo.git(&["merge", "--no-ff", "side", "-m", "merge side"]);
    let repo_ref = repo.repo_ref();
    let at = |rev: &str| backend.resolve(&repo_ref, rev).unwrap();
    CommitDiffFixture {
        root: at("HEAD~7"),
        edit: at("HEAD~6"),
        add: at("HEAD~5"),
        remove: at("HEAD~4"),
        binary: at("HEAD~3"),
        multi: at("HEAD~2"),
        merge: at("HEAD"),
        repo_ref,
        _repo: repo,
    }
}

fn paths_and_statuses(
    files: &[gitpoulpi_core::domain::FileDiff],
) -> Vec<(String, gitpoulpi_core::domain::FileStatus)> {
    files
        .iter()
        .map(|f| (f.path.to_string_lossy().replace('\\', "/"), f.status))
        .collect()
}

/// `diff(CommitVsParent)` is what the commit-detail pane lists: the commit
/// against its *first* parent, a root commit against nothing. The root
/// commit and the merge are the cases that could go wrong.
pub fn check_diff_of_a_commit_is_against_its_first_parent(backend: &dyn GitBackend) {
    use gitpoulpi_core::domain::FileStatus::{Added, Deleted, Modified};
    let f = commit_diff_fixture(backend);
    let diff = |oid| {
        backend
            .diff(&f.repo_ref, DiffTarget::CommitVsParent(oid))
            .unwrap()
    };

    let root = diff(f.root);
    assert_eq!(paths_and_statuses(&root), vec![("keep.txt".into(), Added)]);
    let added: Vec<&str> = root[0].hunks[0]
        .lines
        .iter()
        .map(|l| {
            assert_eq!(l.origin, LineOrigin::Add);
            l.content.as_str()
        })
        .collect();
    assert_eq!(added, vec!["1", "2", "3"]);

    let edit = diff(f.edit);
    assert_eq!(
        paths_and_statuses(&edit),
        vec![("keep.txt".into(), Modified)]
    );
    let lines: Vec<(LineOrigin, &str)> = edit[0].hunks[0]
        .lines
        .iter()
        .map(|l| (l.origin.clone(), l.content.as_str()))
        .collect();
    assert_eq!(
        lines,
        vec![
            (LineOrigin::Context, "1"),
            (LineOrigin::Delete, "2"),
            (LineOrigin::Add, "TWO"),
            (LineOrigin::Context, "3"),
        ]
    );

    assert_eq!(
        paths_and_statuses(&diff(f.add)),
        vec![("dir/new.txt".into(), Added)]
    );
    assert_eq!(
        paths_and_statuses(&diff(f.remove)),
        vec![("dir/new.txt".into(), Deleted)]
    );
    let binary = diff(f.binary);
    assert_eq!(
        paths_and_statuses(&binary),
        vec![("blob.bin".into(), Added)]
    );
    assert!(
        binary[0].hunks.is_empty(),
        "a binary file has no text hunks"
    );
    assert_eq!(
        paths_and_statuses(&diff(f.multi)),
        vec![
            ("a/b.txt".into(), Added),
            ("multi.txt".into(), Added),
            ("z.txt".into(), Added),
        ],
        "every file, in path order"
    );
    assert_eq!(
        paths_and_statuses(&diff(f.merge)),
        vec![("side.txt".into(), Added)],
        "a merge is shown against its first parent, not as a combined diff"
    );
}

/// `diff_files` lists exactly what `diff` lists (same paths, statuses and
/// order) and either carries the same hunks (`hunks_included`) or none.
/// The commit-detail pane swaps the second answer in for the first, so a
/// difference would make the list visibly rearrange.
pub fn check_diff_files_lists_exactly_what_diff_lists(backend: &dyn GitBackend) {
    let f = commit_diff_fixture(backend);
    let mut targets: Vec<DiffTarget> =
        [f.root, f.edit, f.add, f.remove, f.binary, f.multi, f.merge]
            .into_iter()
            .map(DiffTarget::CommitVsParent)
            .collect();
    targets.push(DiffTarget::CommitRange(f.root, f.merge));

    for target in targets {
        let full = backend.diff(&f.repo_ref, target.clone()).unwrap();
        let listed = backend.diff_files(&f.repo_ref, target.clone()).unwrap();
        assert_eq!(
            paths_and_statuses(&listed.files),
            paths_and_statuses(&full),
            "{target:?}"
        );
        if listed.hunks_included {
            assert_eq!(
                listed.files, full,
                "{target:?}: hunks included must be the diff"
            );
        } else {
            assert!(
                listed.files.iter().all(|file| file.hunks.is_empty()),
                "{target:?}: a list without hunks must not carry some"
            );
        }
    }
}

/// `diff_paths` is `diff` restricted to the named files: the same entries
/// (status, hunks, hunk ids) in the same order, nothing else, paths taken
/// literally - on a commit, a rename-free multi-file change and the
/// working tree / index. An empty list is an empty answer.
pub fn check_diff_restricted_to_paths(backend: &dyn GitBackend) {
    use std::path::PathBuf;
    let f = commit_diff_fixture(backend);
    let target = DiffTarget::CommitVsParent(f.multi);
    let full = backend.diff(&f.repo_ref, target.clone()).unwrap();
    assert!(full.len() >= 3);
    let pick = |names: &[&str]| -> Vec<PathBuf> { names.iter().map(PathBuf::from).collect() };
    let one = backend
        .diff_paths(&f.repo_ref, target.clone(), &pick(&["multi.txt"]))
        .unwrap();
    let expected: Vec<_> = full
        .iter()
        .filter(|d| d.path.to_str() == Some("multi.txt"))
        .cloned()
        .collect();
    assert_eq!(
        one, expected,
        "one file: exactly its entry of the full diff"
    );
    assert_eq!(one.len(), 1);
    let two = backend
        .diff_paths(&f.repo_ref, target.clone(), &pick(&["z.txt", "a/b.txt"]))
        .unwrap();
    let expected: Vec<_> = full
        .iter()
        .filter(|d| d.path.to_str() == Some("z.txt") || d.path.to_str() == Some("a/b.txt"))
        .cloned()
        .collect();
    assert_eq!(two, expected, "several files keep the full diff's order");
    assert!(backend
        .diff_paths(&f.repo_ref, target.clone(), &[])
        .unwrap()
        .is_empty());
    assert!(backend
        .diff_paths(&f.repo_ref, target, &pick(&["*.txt"]))
        .unwrap()
        .is_empty());

    // The working tree and the index, with a file name git would glob.
    let repo = TestRepoBuilder::new()
        .commit_file("a.txt", "1\n2\n3\n", "a")
        .commit_file("b[1].txt", "1\n2\n3\n", "b")
        .build();
    let repo_ref = repo.repo_ref();
    std::fs::write(repo.root().join("a.txt"), "1\nA\n3\n").unwrap();
    std::fs::write(repo.root().join("b[1].txt"), "1\nB\n3\n").unwrap();
    {
        let target = DiffTarget::WorkingTreeVsIndex;
        let full = backend.diff(&repo_ref, target.clone()).unwrap();
        let only_b = backend
            .diff_paths(&repo_ref, target.clone(), &[PathBuf::from("b[1].txt")])
            .unwrap();
        let expected: Vec<_> = full
            .iter()
            .filter(|d| d.path.to_str() == Some("b[1].txt"))
            .cloned()
            .collect();
        assert_eq!(only_b.len(), 1);
        assert_eq!(only_b, expected);
    }
    backend
        .stage(&repo_ref, &StageTarget::File("a.txt".into()))
        .unwrap();
    let staged = backend
        .diff_paths(
            &repo_ref,
            DiffTarget::IndexVsHead,
            &[PathBuf::from("a.txt")],
        )
        .unwrap();
    assert_eq!(staged.len(), 1);
    assert_eq!(staged[0].path, PathBuf::from("a.txt"));
    assert!(backend
        .diff_paths(
            &repo_ref,
            DiffTarget::IndexVsHead,
            &[PathBuf::from("b[1].txt")]
        )
        .unwrap()
        .is_empty());
}

/// `commit_stats` (the Log's "Changes" column): per commit, the files and
/// lines it changed against its first parent - a root against nothing, a
/// merge against its first parent, a binary file as a file with no lines --
/// answered in the order asked, for a whole batch at once.
pub fn check_commit_stats_count_files_and_lines_against_the_first_parent(backend: &dyn GitBackend) {
    let f = commit_diff_fixture(backend);
    let ids = [f.root, f.edit, f.add, f.remove, f.binary, f.multi, f.merge];
    let expected = [
        (1, 3, 0),
        (1, 1, 1),
        (1, 1, 0),
        (1, 0, 1),
        (1, 0, 0),
        (3, 3, 0),
        (1, 1, 0),
    ];
    let stats = backend.commit_stats(&f.repo_ref, &ids).unwrap();
    let got: Vec<_> = stats
        .iter()
        .map(|s| (s.id, s.files, s.insertions, s.deletions))
        .collect();
    let want: Vec<_> = ids
        .iter()
        .zip(expected)
        .map(|(id, (files, ins, del))| (*id, files, ins, del))
        .collect();
    assert_eq!(got, want);

    // The file count is the length of the commit's diff.
    for (id, stat) in ids.iter().zip(&stats) {
        let diff = backend
            .diff(&f.repo_ref, DiffTarget::CommitVsParent(*id))
            .unwrap();
        assert_eq!(stat.files as usize, diff.len(), "{id:?}");
    }

    // Order is the caller's, a single id works, and nothing is nothing.
    let reversed = backend
        .commit_stats(&f.repo_ref, &[f.merge, f.root])
        .unwrap();
    assert_eq!(
        reversed.iter().map(|s| s.id).collect::<Vec<_>>(),
        vec![f.merge, f.root]
    );
    assert_eq!(
        backend.commit_stats(&f.repo_ref, &[f.edit]).unwrap(),
        vec![stats[1]]
    );
    assert!(backend.commit_stats(&f.repo_ref, &[]).unwrap().is_empty());
}

/// `diff_files` on the working tree lists what `diff` lists, hunks apart.
pub fn check_diff_files_working_tree_lists_what_diff_lists(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .commit_file("keep.txt", "1\n", "k")
        .commit_file("gone.txt", "1\n", "g")
        .commit_file("sp ace.txt", "1\n", "s")
        .build();
    let repo_ref = repo.repo_ref();
    std::fs::write(repo.root().join("keep.txt"), "2\n").unwrap();
    std::fs::write(repo.root().join("sp ace.txt"), "2\n").unwrap();
    std::fs::remove_file(repo.root().join("gone.txt")).unwrap();
    for target in [DiffTarget::WorkingTreeVsIndex, DiffTarget::IndexVsHead] {
        let full = backend.diff(&repo_ref, target.clone()).unwrap();
        let listed = backend.diff_files(&repo_ref, target.clone()).unwrap();
        assert_eq!(
            paths_and_statuses(&listed.files),
            paths_and_statuses(&full),
            "{target:?}"
        );
        if !listed.hunks_included {
            assert!(listed.files.iter().all(|f| f.hunks.is_empty()));
        }
        if target == DiffTarget::WorkingTreeVsIndex {
            assert_eq!(full.len(), 3);
        }
    }
}

/// A repository whose *own* config reshapes git's output must still be
/// read correctly by both backends.
///
/// `diff.mnemonicPrefix = true` makes git print `i/f.txt w/f.txt` instead
/// of `a/f.txt b/f.txt`. The keys are set repo-locally so the test does
/// not depend on the machine's config; `color.ui` and `core.quotePath`
/// reshape parsed output the same way.
pub fn check_survives_output_reshaping_config(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .config("diff.mnemonicPrefix", "true")
        .config("diff.noprefix", "true")
        .config("color.ui", "always")
        .config("core.quotePath", "true")
        .config("log.showSignature", "true")
        .commit("initial")
        .build();
    let repo_ref = repo.repo_ref();

    // An edit to a tracked file: the diff has to name it and carry a hunk.
    std::fs::write(repo.root().join("initial.txt"), "changed").unwrap();
    let diffs = backend
        .diff(
            &repo_ref,
            gitpoulpi_core::domain::DiffTarget::WorkingTreeVsIndex,
        )
        .unwrap();
    assert_eq!(diffs.len(), 1);
    assert_eq!(diffs[0].path.to_str(), Some("initial.txt"));
    assert_eq!(diffs[0].hunks.len(), 1);

    // ...and staging it, which goes through the same parsed paths.
    backend
        .stage(&repo_ref, &StageTarget::File("initial.txt".into()))
        .unwrap();
    let staged = backend
        .diff(&repo_ref, gitpoulpi_core::domain::DiffTarget::IndexVsHead)
        .unwrap();
    assert_eq!(staged.len(), 1);
    assert_eq!(staged[0].path.to_str(), Some("initial.txt"));

    // `color.ui = always` would otherwise leave ANSI escapes in these.
    let log = backend.log(&repo_ref, &LogOptions::default()).unwrap();
    assert_eq!(log.nodes[0].summary, "initial");
    let status = backend.status(&repo_ref).unwrap();
    assert!(status
        .files
        .iter()
        .any(|f| f.path.to_str() == Some("initial.txt")));
}

/// The whole tree, not the changed subset, and sorted. The second commit
/// touches one file out of three, so a backend that answered with the diff
/// would return one path here.
pub fn check_list_tree_returns_every_path_at_the_revision(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .commit("one")
        .commit("two")
        .commit("three")
        .build();
    let repo_ref = repo.repo_ref();
    let head = head_oid(backend, &repo_ref);

    let listed = backend.list_tree(&repo_ref, head).unwrap();
    let names: Vec<String> = listed
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();
    assert_eq!(names, vec!["one.txt", "three.txt", "two.txt"], "{names:?}");
}

/// An older revision shows that revision, not HEAD. This is the whole
/// point of the parameter, and the easiest thing to get wrong is to
/// ignore it and read the working tree.
pub fn check_list_tree_is_answered_at_the_revision_asked_for(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .commit("first")
        .commit("second")
        .build();
    let repo_ref = repo.repo_ref();
    let log = backend.log(&repo_ref, &LogOptions::default()).unwrap();
    let parent = log.nodes[1].id;

    let at_parent = backend.list_tree(&repo_ref, parent).unwrap();
    let names: Vec<String> = at_parent
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();
    assert_eq!(
        names,
        vec!["first.txt"],
        "the older revision must not show what came after it: {names:?}"
    );
}

/// Nested paths come back with `/` separators, whatever the platform,
/// because every other path in the domain does and the TUI's `Tree` mode
/// splits on it. A backend that let Windows' `\` through would build a
/// folder view with one very long folder name.
pub fn check_list_tree_uses_forward_slashes_for_nested_paths(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .commit_file("src/deep/nested.txt", "hi", "nested")
        .build();
    let repo_ref = repo.repo_ref();
    let head = head_oid(backend, &repo_ref);

    let listed = backend.list_tree(&repo_ref, head).unwrap();
    let names: Vec<String> = listed
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();
    assert!(
        names.contains(&"src/deep/nested.txt".to_string()),
        "nested paths must use `/`: {names:?}"
    );
    assert!(
        !names.iter().any(|n| n.contains('\\')),
        "no backslashes anywhere: {names:?}"
    );
    // And the directories themselves are not entries.
    assert!(
        !names.iter().any(|n| n == "src" || n == "src/deep"),
        "directories are not listed: {names:?}"
    );
}

/// A revision that does not exist is an error, not an empty list. An
/// empty list is a legitimate answer for an empty tree, so conflating
/// the two would make "this commit is gone" indistinguishable from
/// "this commit has no files".
pub fn check_list_tree_rejects_an_unknown_revision(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new().commit("only").build();
    let repo_ref = repo.repo_ref();
    let missing = Oid::from_hex("0123456789abcdef0123456789abcdef01234567").unwrap();
    assert!(backend.list_tree(&repo_ref, missing).is_err());
}

// -- repository discovery ------------------------------------------------
//
// The two backends have different machinery underneath (`git rev-parse`
// vs `git2::Repository::discover`), so the cases where a naive
// `.join(".git").exists()` walk is wrong are the ones worth pinning.

/// `read_blob` has to give the same lines for the same file whatever the
/// line terminator on disk, and has to tell text apart from bytes.
///
/// The CRLF half matters on Windows: with `core.autocrlf` the *committed*
/// blob is LF while the checked-out file is CRLF, so a naive split on \n
/// leaves a trailing `\r` on lines read from the working tree only, and
/// every blame line number downstream is off.
pub fn check_read_blob_text_binary_and_line_endings(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .commit_file("lf.txt", "one\ntwo\nthree\n", "lf")
        .commit_bytes("crlf.txt", b"one\r\ntwo\r\nthree\r\n", "crlf")
        .commit_bytes("no-eol.txt", b"only line", "no-eol")
        .commit_bytes("blob.bin", b"\x89PNG\r\n\x1a\n\x00\x01\x02", "binary")
        .build();
    let repo_ref = repo.repo_ref();
    let head = head_oid(backend, &repo_ref);

    let lf = backend
        .read_blob(
            &repo_ref,
            std::path::Path::new("lf.txt"),
            &BlobSource::Commit(head),
        )
        .unwrap();
    assert_eq!(lf.content.lines(), ["one", "two", "three"]);
    assert_eq!(lf.size, 14);

    let crlf = backend
        .read_blob(
            &repo_ref,
            std::path::Path::new("crlf.txt"),
            &BlobSource::Commit(head),
        )
        .unwrap();
    assert_eq!(
        crlf.content.lines(),
        ["one", "two", "three"],
        "CRLF content must yield the same lines as LF content, with no trailing carriage return"
    );
    assert!(
        matches!(crlf.content, BlobContent::Text { crlf: true, .. }),
        "the CRLF-ness has to be reported, not just silently dropped"
    );

    let no_eol = backend
        .read_blob(
            &repo_ref,
            std::path::Path::new("no-eol.txt"),
            &BlobSource::Commit(head),
        )
        .unwrap();
    assert_eq!(no_eol.content.lines(), ["only line"]);
    assert!(matches!(
        no_eol.content,
        BlobContent::Text {
            final_newline: false,
            ..
        }
    ));

    let binary = backend
        .read_blob(
            &repo_ref,
            std::path::Path::new("blob.bin"),
            &BlobSource::Commit(head),
        )
        .unwrap();
    assert_eq!(
        binary.content,
        BlobContent::Binary,
        "a NUL byte makes a file binary - it must not come back as lossy text"
    );
    assert_eq!(binary.size, 11, "a binary blob still reports its size");

    assert!(
        backend
            .read_blob(
                &repo_ref,
                std::path::Path::new("nope.txt"),
                &BlobSource::Commit(head),
            )
            .is_err(),
        "a path that doesn't exist at the source is an error, not an empty file"
    );
}

/// The three sources of a path's content are three different answers, and
/// both backends have to agree on which is which: a file can be committed
/// one way, staged another and edited a third at once, and a File View
/// that showed the working-tree bytes for a commit would misstate history.
pub fn check_read_blob_distinguishes_commit_index_and_worktree(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .commit_file("f.txt", "committed\n", "initial")
        .write_worktree("f.txt", b"staged\n")
        .stage("f.txt")
        .write_worktree("f.txt", b"working\n")
        .build();
    let repo_ref = repo.repo_ref();
    let head = head_oid(backend, &repo_ref);
    let path = std::path::Path::new("f.txt");

    assert_eq!(
        backend
            .read_blob(&repo_ref, path, &BlobSource::Commit(head))
            .unwrap()
            .content
            .lines(),
        ["committed"]
    );
    assert_eq!(
        backend
            .read_blob(&repo_ref, path, &BlobSource::Index)
            .unwrap()
            .content
            .lines(),
        ["staged"]
    );
    assert_eq!(
        backend
            .read_blob(&repo_ref, path, &BlobSource::WorkingTree)
            .unwrap()
            .content
            .lines(),
        ["working"]
    );
}

/// `LogOptions::paths` has to behave like `git log -- <path>`: only the
/// commits that touched the path, in history order, including the one
/// that deleted it - and an empty `paths` must keep meaning "no filter".
pub fn check_log_filtered_by_path(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .commit_file("src/a.txt", "a1\n", "touch a")
        .commit_file("src/b.txt", "b1\n", "touch b")
        .commit_file("src/a.txt", "a2\n", "touch a again")
        .commit_removal("src/b.txt", "delete b")
        .build();
    let repo_ref = repo.repo_ref();

    let summaries = |paths: Vec<std::path::PathBuf>| -> Vec<String> {
        backend
            .log(
                &repo_ref,
                &LogOptions {
                    paths,
                    ..LogOptions::default()
                },
            )
            .unwrap()
            .nodes
            .iter()
            .map(|n| n.summary.clone())
            .collect()
    };

    assert_eq!(
        summaries(vec!["src/a.txt".into()]),
        ["touch a again", "touch a"],
        "a per-path log lists only the commits touching that path, newest first"
    );
    assert_eq!(
        summaries(vec!["src/b.txt".into()]),
        ["delete b", "touch b"],
        "the commit that deleted a path still belongs to that path's history"
    );
    // A directory pathspec covers everything under it.
    assert_eq!(summaries(vec!["src".into()]).len(), 4);
    // The default (no paths) must be untouched by all of this.
    assert_eq!(summaries(vec![]).len(), 4);
    assert!(
        summaries(vec!["no/such/path.txt".into()]).is_empty(),
        "a path with no history is an empty log, not an error and not the full log"
    );
}

/// Blame attributes each line to the commit that last changed it, numbers
/// lines 1-based, and carries the line's own text.
///
/// The text is part of the contract: the blame gutter is drawn beside
/// those lines, so blame's line 2 and `read_blob`'s line 2 must agree. The
/// last assertion pins that.
pub fn check_blame_attributes_lines_to_their_last_commit(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .commit_file("f.txt", "first\nsecond\nthird\n", "original")
        .commit_file("f.txt", "first\nCHANGED\nthird\n", "change the middle line")
        .build();
    let repo_ref = repo.repo_ref();
    let path = std::path::Path::new("f.txt");

    let log = backend.log(&repo_ref, &LogOptions::default()).unwrap();
    let tip = log.nodes[0].id;
    let original = log.nodes[1].id;

    let blame = backend.blame(&repo_ref, path, tip).unwrap();
    assert_eq!(blame.at, tip);
    let lines = match &blame.content {
        BlameContent::Lines(lines) => lines,
        other => panic!("expected blamed lines, got {other:?}"),
    };
    assert_eq!(lines.len(), 3);

    assert_eq!(
        lines[0].commit, original,
        "an untouched line keeps its original commit"
    );
    assert_eq!(
        lines[1].commit, tip,
        "the rewritten line belongs to the newer commit"
    );
    assert_eq!(lines[2].commit, original);

    assert_eq!(lines[0].final_line, 1, "line numbers are 1-based");
    assert_eq!(lines[2].final_line, 3);
    assert_eq!(lines[1].orig_line, 2);

    assert_eq!(lines[1].summary, "change the middle line");
    assert_eq!(lines[1].author_name, "GitPoulpi Testkit");
    assert_eq!(lines[1].author_email, "testkit@gitpoulpi.local");
    assert!(lines[1].timestamp > 0);

    // Blame and read_blob must be talking about the same lines.
    let blob = backend
        .read_blob(&repo_ref, path, &BlobSource::Commit(tip))
        .unwrap();
    let blamed_text: Vec<&str> = lines.iter().map(|l| l.content.as_str()).collect();
    assert_eq!(
        blamed_text,
        blob.content.lines(),
        "blame's lines must be read_blob's lines, in the same order"
    );
}

/// Blaming an older revision blames *that* revision's file, not HEAD's --
/// a File History view walks back through commits and blames each one.
pub fn check_blame_at_an_older_revision(backend: &dyn GitBackend) {
    // Line 2 exists in both revisions but was rewritten by the newer one,
    // so a backend that ignored `at` would attribute it to `newer` rather
    // than `original`; merely appending a line would let both pass.
    let repo = TestRepoBuilder::new()
        .commit_file("f.txt", "first\nsecond\n", "original")
        .commit_file("f.txt", "first\nREWRITTEN\nthird\n", "rewrite and append")
        .build();
    let repo_ref = repo.repo_ref();
    let log = backend.log(&repo_ref, &LogOptions::default()).unwrap();
    let original = log.nodes[1].id;

    let blame = backend
        .blame(&repo_ref, std::path::Path::new("f.txt"), original)
        .unwrap();
    let lines = blame.content.lines();
    assert_eq!(
        lines.len(),
        2,
        "blaming an older commit must see that commit's file, not HEAD's"
    );
    assert_eq!(lines[1].content, "second");
    assert!(
        lines.iter().all(|l| l.commit == original),
        "every line of the original revision belongs to it, not to the commit that later rewrote it"
    );
}

/// A binary file is blameable without exploding: the answer is
/// `BlameContent::Binary`, matching what `read_blob` says about the same
/// file, rather than an error or a list of mojibake lines.
pub fn check_blame_reports_binary_files(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .commit_bytes("blob.bin", b"\x89PNG\r\n\x1a\n\x00\x01\x02", "binary")
        .build();
    let repo_ref = repo.repo_ref();
    let head = head_oid(backend, &repo_ref);

    let blame = backend
        .blame(&repo_ref, std::path::Path::new("blob.bin"), head)
        .unwrap();
    assert_eq!(blame.content, BlameContent::Binary);

    assert!(
        backend
            .blame(&repo_ref, std::path::Path::new("nope.txt"), head)
            .is_err(),
        "blaming a path that isn't in the commit is an error"
    );
}

/// Resets the fixture repo's index and working tree back to `at`, and
/// removes anything left untracked - the cheap stand-in for "a fresh
/// checkout of an older commit" for setting up `patch_apply` cases
/// without a second clone.
fn reset_hard_and_clean(repo: &crate::FixtureRepo, at: Oid) {
    repo.git(&["reset", "--hard", &at.to_hex()]);
    repo.git(&["clean", "-fd"]);
}

pub fn check_patch_create_from_a_commit_round_trips_through_apply(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .with_initial()
        .commit_file("feature.txt", "feature work\n", "add feature")
        .build();
    let repo_ref = repo.repo_ref();
    let initial = rev(&repo, "HEAD~1");
    let feature_commit = rev(&repo, "HEAD");
    let dest = repo.root().parent().unwrap().join("0001-add-feature.patch");

    let created = backend
        .patch_create(
            &repo_ref,
            &PatchSource::Commits(vec![feature_commit]),
            &dest,
        )
        .unwrap();
    assert_eq!(created.commits, 1);
    assert!(created.bytes > 0);
    assert_eq!(created.path, dest);
    assert!(dest.exists());

    reset_hard_and_clean(&repo, initial);
    assert!(!repo.root().join("feature.txt").exists());

    let applied = backend.patch_apply(&repo_ref, &dest).unwrap();
    assert_eq!(applied.files, vec!["feature.txt".to_string()]);
    assert_eq!(
        std::fs::read_to_string(repo.root().join("feature.txt")).unwrap(),
        "feature work\n"
    );
}

pub fn check_patch_create_from_a_range_applies_both_commits_to_the_same_file(
    backend: &dyn GitBackend,
) {
    let repo = TestRepoBuilder::new()
        .commit_file("notes.txt", "one\n", "initial")
        .commit_file("notes.txt", "one\ntwo\n", "add two")
        .commit_file("notes.txt", "one\ntwo\nthree\n", "add three")
        .build();
    let repo_ref = repo.repo_ref();
    let initial = rev(&repo, "HEAD~2");
    let tip = rev(&repo, "HEAD");
    let dest = repo.root().parent().unwrap().join("range.patch");

    let created = backend
        .patch_create(&repo_ref, &PatchSource::Range { base: initial, tip }, &dest)
        .unwrap();
    assert_eq!(created.commits, 2);

    reset_hard_and_clean(&repo, initial);
    backend.patch_apply(&repo_ref, &dest).unwrap();
    assert_eq!(
        std::fs::read_to_string(repo.root().join("notes.txt")).unwrap(),
        "one\ntwo\nthree\n"
    );
}

pub fn check_patch_create_from_the_working_tree_covers_staged_and_unstaged_only(
    backend: &dyn GitBackend,
) {
    let repo = TestRepoBuilder::new()
        .commit_file("staged.txt", "before\n", "initial staged")
        .commit_file("unstaged.txt", "before\n", "initial unstaged")
        .write_worktree("staged.txt", b"after staged\n")
        .stage("staged.txt")
        .write_worktree("unstaged.txt", b"after unstaged\n")
        .write_worktree("untracked.txt", b"brand new\n")
        .build();
    let repo_ref = repo.repo_ref();

    let dest = repo.root().parent().unwrap().join("wip.patch");
    let created = backend
        .patch_create(
            &repo_ref,
            &PatchSource::WorkingTree { paths: vec![] },
            &dest,
        )
        .unwrap();
    assert_eq!(created.commits, 0);
    let text = std::fs::read_to_string(&dest).unwrap();
    assert!(text.contains("staged.txt"));
    assert!(text.contains("unstaged.txt"));
    assert!(
        !text.contains("untracked.txt"),
        "an untracked file has no HEAD side to diff against"
    );

    // Filtered to one path.
    let filtered_dest = repo.root().parent().unwrap().join("wip-one.patch");
    backend
        .patch_create(
            &repo_ref,
            &PatchSource::WorkingTree {
                paths: vec!["staged.txt".to_string()],
            },
            &filtered_dest,
        )
        .unwrap();
    let filtered_text = std::fs::read_to_string(&filtered_dest).unwrap();
    assert!(filtered_text.contains("staged.txt"));
    assert!(!filtered_text.contains("unstaged.txt"));
}

pub fn check_patch_create_from_commit_paths_carries_only_those_files(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .commit_file("base.txt", "base\n", "initial")
        .write_worktree("one.txt", b"first\n")
        .write_worktree("two.txt", b"second\n")
        .stage("one.txt")
        .stage("two.txt")
        .commit("add two files")
        .build();
    let repo_ref = repo.repo_ref();
    let initial = rev(&repo, "HEAD~1");
    let commit = rev(&repo, "HEAD");
    let dest = repo.root().parent().unwrap().join("one.patch");

    let created = backend
        .patch_create(
            &repo_ref,
            &PatchSource::CommitPaths {
                commit,
                paths: vec!["one.txt".to_string()],
            },
            &dest,
        )
        .unwrap();
    assert_eq!(created.commits, 1);
    let text = std::fs::read_to_string(&dest).unwrap();
    assert!(text.contains("one.txt"));
    assert!(!text.contains("two.txt"), "only the asked path");

    reset_hard_and_clean(&repo, initial);
    let applied = backend.patch_apply(&repo_ref, &dest).unwrap();
    assert_eq!(applied.files, vec!["one.txt".to_string()]);
    assert!(!repo.root().join("two.txt").exists());

    // A path the commit did not touch, or no path at all, writes nothing.
    let none = repo.root().parent().unwrap().join("none.patch");
    assert!(backend
        .patch_create(
            &repo_ref,
            &PatchSource::CommitPaths {
                commit,
                paths: vec!["base.txt".to_string()],
            },
            &none,
        )
        .is_err());
    assert!(backend
        .patch_create(
            &repo_ref,
            &PatchSource::CommitPaths {
                commit,
                paths: vec![],
            },
            &none,
        )
        .is_err());
    assert!(!none.exists());
}

pub fn check_patch_keeps_binary_and_crlf_bytes_exactly(backend: &dyn GitBackend) {
    let binary_before: &[u8] = &[0x89, b'P', b'N', b'G', 0x00, 0x01, 0x02, 0xFF];
    let binary_after: &[u8] = &[0x89, b'P', b'N', b'G', 0x00, 0x9, 0x9, 0xFF, 0xAB];
    let crlf_before: &[u8] = b"line one\r\nline two\r\n";
    let crlf_after: &[u8] = b"line one\r\nline two\r\nline three\r\n";

    let repo = TestRepoBuilder::new()
        .commit_bytes("image.bin", binary_before, "add binary")
        .commit_bytes("crlf.txt", crlf_before, "add crlf")
        .commit_bytes("image.bin", binary_after, "change binary")
        .commit_bytes("crlf.txt", crlf_after, "change crlf")
        .build();
    let repo_ref = repo.repo_ref();
    let before_binary_change = rev(&repo, "HEAD~2");
    let binary_commit = rev(&repo, "HEAD~1");
    let crlf_commit = rev(&repo, "HEAD");

    let dest = repo.root().parent().unwrap().join("bytes.patch");
    backend
        .patch_create(
            &repo_ref,
            &PatchSource::Commits(vec![binary_commit, crlf_commit]),
            &dest,
        )
        .unwrap();

    reset_hard_and_clean(&repo, before_binary_change);
    backend.patch_apply(&repo_ref, &dest).unwrap();
    assert_eq!(
        std::fs::read(repo.root().join("image.bin")).unwrap(),
        binary_after
    );
    assert_eq!(
        std::fs::read(repo.root().join("crlf.txt")).unwrap(),
        crlf_after
    );
}

pub fn check_patch_create_refusals_write_nothing(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .commit_file("a.txt", "a\n", "initial")
        .commit_file("a.txt", "a\nb\n", "second")
        .build();
    let repo_ref = repo.repo_ref();
    let initial = rev(&repo, "HEAD~1");
    let tip = rev(&repo, "HEAD");
    let base = repo.root().parent().unwrap().to_path_buf();

    // Destination already exists.
    let taken = base.join("taken.patch");
    std::fs::write(&taken, b"keep me").unwrap();
    assert!(backend
        .patch_create(&repo_ref, &PatchSource::Commits(vec![tip]), &taken)
        .is_err());
    assert_eq!(std::fs::read(&taken).unwrap(), b"keep me");

    // Parent directory missing.
    let missing_parent = base.join("nowhere").join("out.patch");
    assert!(backend
        .patch_create(&repo_ref, &PatchSource::Commits(vec![tip]), &missing_parent)
        .is_err());
    assert!(!missing_parent.exists());

    // Empty range (base == tip).
    let empty_range = base.join("empty-range.patch");
    assert!(backend
        .patch_create(
            &repo_ref,
            &PatchSource::Range { base: tip, tip },
            &empty_range,
        )
        .is_err());
    assert!(!empty_range.exists());

    // Unknown commit.
    let unknown = base.join("unknown.patch");
    let bogus = Oid::from_hex("abc1230000000000000000000000000000000000").unwrap();
    assert!(backend
        .patch_create(&repo_ref, &PatchSource::Commits(vec![bogus]), &unknown)
        .is_err());
    assert!(!unknown.exists());

    // Unborn HEAD.
    let empty_repo = TestRepoBuilder::new().build();
    let empty_repo_ref = empty_repo.repo_ref();
    let unborn_dest = empty_repo.root().parent().unwrap().join("unborn.patch");
    assert!(backend
        .patch_create(
            &empty_repo_ref,
            &PatchSource::WorkingTree { paths: vec![] },
            &unborn_dest,
        )
        .is_err());
    assert!(!unborn_dest.exists());

    let _ = initial; // used only to document the fixture's shape
}

pub fn check_patch_apply_that_does_not_apply_changes_nothing(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .commit_file("shared.txt", "one\n", "initial")
        .commit_file("shared.txt", "one\ntwo\n", "add two")
        .build();
    let repo_ref = repo.repo_ref();
    let tip = rev(&repo, "HEAD");
    let dest = repo.root().parent().unwrap().join("conflict.patch");
    backend
        .patch_create(&repo_ref, &PatchSource::Commits(vec![tip]), &dest)
        .unwrap();

    // Diverge the file so the patch's context no longer matches.
    std::fs::write(repo.root().join("shared.txt"), "one\nsomething else\n").unwrap();
    let before = std::fs::read(repo.root().join("shared.txt")).unwrap();
    let status_before = backend.status(&repo_ref).unwrap();

    assert!(backend.patch_apply(&repo_ref, &dest).is_err());

    let after = std::fs::read(repo.root().join("shared.txt")).unwrap();
    assert_eq!(
        before, after,
        "a patch that fails to apply must change nothing"
    );
    let status_after = backend.status(&repo_ref).unwrap();
    assert_eq!(status_before.files.len(), status_after.files.len());
}

/// `gitignore_add` appends one line, creates the file when missing, keeps
/// an unterminated last line, is idempotent, and refuses anything that is
/// not a `.gitignore` inside the repository.
pub fn check_gitignore_add(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .commit_file("a.txt", "a\n", "first")
        .build();
    let repo_ref = repo.repo_ref();
    let ignore = repo.root().join(".gitignore");
    std::fs::write(repo.root().join("x.log"), "log").unwrap();

    assert!(backend.gitignore_add(&repo_ref, "*.log", None).unwrap());
    assert_eq!(std::fs::read_to_string(&ignore).unwrap(), "*.log\n");
    assert!(
        !repo.git(&["status", "--porcelain"]).contains("x.log"),
        "the new pattern hides the file"
    );
    assert!(
        !backend.gitignore_add(&repo_ref, "*.log", None).unwrap(),
        "already there: nothing written"
    );

    std::fs::write(&ignore, "target").unwrap();
    assert!(backend.gitignore_add(&repo_ref, "/build", None).unwrap());
    assert_eq!(
        std::fs::read_to_string(&ignore).unwrap(),
        "target\n/build\n"
    );

    std::fs::create_dir_all(repo.root().join("sub")).unwrap();
    assert!(backend
        .gitignore_add(&repo_ref, "*.tmp", Some("sub/.gitignore"))
        .unwrap());
    assert_eq!(
        std::fs::read_to_string(repo.root().join("sub/.gitignore")).unwrap(),
        "*.tmp\n"
    );

    for (pattern, file) in [
        ("", None),
        ("a\nb", None),
        ("x", Some("a.txt")),
        ("x", Some("../.gitignore")),
        ("x", Some(".git/.gitignore")),
    ] {
        assert!(
            backend.gitignore_add(&repo_ref, pattern, file).is_err(),
            "{pattern:?} {file:?}"
        );
    }
}

// -- git.default_encoding ----------------------------------------------

/// Non-UTF-8 text is binary to `read_blob` and `blame` - unless a fallback
/// encoding is in force (`git.default_encoding`), in which case it is
/// decoded with it, the same way by both backends, in file content, blame
/// and diff lines. Valid UTF-8 is never re-decoded, and staging the file
/// writes its bytes exactly (the decoding is display-only).
pub fn check_non_utf8_content_decodes_with_the_fallback_encoding(backend: &dyn GitBackend) {
    use gitpoulpi_core::domain::with_fallback_encoding;

    // "café\nnaïve\n" in windows-1252, next to a UTF-8 file.
    let latin1: &[u8] = b"caf\xe9\nna\xefve\n";
    let repo = TestRepoBuilder::new()
        .commit_bytes("latin.txt", latin1, "latin1")
        .commit_file("utf8.txt", "café\n", "utf8")
        .build();
    let repo_ref = repo.repo_ref();
    let head = head_oid(backend, &repo_ref);
    let path = std::path::Path::new("latin.txt");
    let at = BlobSource::Commit(head);

    // No fallback: unchanged behavior, not valid UTF-8 means binary.
    let plain = backend.read_blob(&repo_ref, path, &at).unwrap();
    assert_eq!(plain.content, BlobContent::Binary);
    let blame = backend.blame(&repo_ref, path, head).unwrap();
    assert_eq!(blame.content, BlameContent::Binary);

    for label in ["windows-1252", "latin1"] {
        with_fallback_encoding(Some(label), || {
            let blob = backend.read_blob(&repo_ref, path, &at).unwrap();
            assert_eq!(blob.content.lines(), ["café", "naïve"], "{label}");
            assert_eq!(blob.size, latin1.len() as u64, "the size is the bytes'");

            let blame = backend.blame(&repo_ref, path, head).unwrap();
            let BlameContent::Lines(lines) = blame.content else {
                panic!("{label}: blame should attribute the decoded lines");
            };
            let text: Vec<&str> = lines.iter().map(|l| l.content.as_str()).collect();
            assert_eq!(text, ["café", "naïve"], "{label}");

            // Already-UTF-8 content is left alone, not decoded twice.
            let utf8 = backend
                .read_blob(&repo_ref, std::path::Path::new("utf8.txt"), &at)
                .unwrap();
            assert_eq!(utf8.content.lines(), ["café"], "{label}");
        });
    }
    // The scope ended with the closure.
    assert_eq!(
        backend.read_blob(&repo_ref, path, &at).unwrap().content,
        BlobContent::Binary
    );

    // A diff line is decoded too; staging writes the working bytes as they are.
    let edited: &[u8] = b"caf\xe9\nna\xefve\nd\xe9j\xe0 vu\n";
    std::fs::write(repo.root().join("latin.txt"), edited).unwrap();
    with_fallback_encoding(Some("windows-1252"), || {
        let diffs = backend
            .diff(&repo_ref, DiffTarget::WorkingTreeVsIndex)
            .unwrap();
        let added: Vec<&str> = diffs
            .iter()
            .flat_map(|f| &f.hunks)
            .flat_map(|h| &h.lines)
            .filter(|l| l.origin == LineOrigin::Add)
            .map(|l| l.content.as_str())
            .collect();
        assert_eq!(added, ["déjà vu"]);
    });
    backend
        .stage(&repo_ref, &StageTarget::File("latin.txt".into()))
        .unwrap();
    let staged = std::process::Command::new("git")
        .args(["cat-file", "blob", ":latin.txt"])
        .current_dir(repo.root())
        .output()
        .unwrap()
        .stdout;
    assert_eq!(staged, edited, "staging must not re-encode the file");
}

/// Commit messages in the Log follow git: a commit with
/// an `encoding` header is decoded with it; without one, valid UTF-8 is
/// kept and anything else is decoded with the fallback encoding
/// (`git.default_encoding`) - lossily as UTF-8 when none is in force.
/// Both backends agree, and the stored bytes are never touched.
pub fn check_non_utf8_commit_messages_decode_for_display(backend: &dyn GitBackend) {
    use gitpoulpi_core::domain::with_fallback_encoding;

    let repo = TestRepoBuilder::new().with_initial().build();
    let repo_ref = repo.repo_ref();
    // "café déjà\n\nnaïve body" in windows-1252, no header.
    repo.commit_raw_message(b"caf\xe9 d\xe9j\xe0\n\nna\xefve body\n", None);
    // The same text declared ISO-8859-1.
    repo.commit_raw_message(b"l\xe9gume\n\ncorps \xe0 lire\n", Some("ISO-8859-1"));
    // "日本語" in Shift-JIS, declared.
    repo.commit_raw_message(
        b"\x93\xfa\x96\x7b\x8c\xea\n\n\x93\xfa\x96\x7b\n",
        Some("Shift_JIS"),
    );
    // A UTF-8 message needs no help from the fallback.
    repo.commit_raw_message("ça va\n".as_bytes(), None);
    let opts = LogOptions::default();
    let nodes = |label: Option<&str>| {
        with_fallback_encoding(label, || backend.log(&repo_ref, &opts).unwrap().nodes)
    };

    // The declared encodings win whether or not a fallback is set.
    for label in [None, Some("windows-1252")] {
        let n = nodes(label);
        assert_eq!(n.len(), 5, "{label:?}");
        assert_eq!(n[0].summary, "ça va", "{label:?}");
        assert_eq!(n[1].summary, "日本語", "{label:?}");
        assert_eq!(n[1].message, "日本語\n\n日本", "{label:?}");
        assert_eq!(n[2].summary, "légume", "{label:?}");
        assert_eq!(n[2].message, "légume\n\ncorps à lire", "{label:?}");
    }
    // Undeclared and not UTF-8: only the fallback can read it.
    let n = nodes(Some("windows-1252"));
    assert_eq!(n[3].summary, "café déjà");
    assert_eq!(n[3].message, "café déjà\n\nnaïve body");
    let n = nodes(None);
    assert!(
        n[3].summary.contains('\u{fffd}'),
        "lossy without a fallback: {:?}",
        n[3].summary
    );
    assert!(n[3].summary.starts_with("caf"));

    // The stash message embeds the HEAD subject, so it follows the same rule.
    repo.commit_raw_message(b"caf\xe9 stash base\n", None);
    repo.git(&["reset", "--hard", "-q"]);
    std::fs::write(repo.root().join("initial.txt"), "changed for stash").unwrap();
    repo.git(&["stash", "push", "-q"]);
    let stash_message = |label: Option<&str>| {
        with_fallback_encoding(label, || {
            backend.stashes(&repo_ref).unwrap().remove(0).message
        })
    };
    assert!(
        stash_message(Some("windows-1252")).ends_with("café stash base"),
        "{}",
        stash_message(Some("windows-1252"))
    );
    assert!(stash_message(None).contains('\u{fffd}'));
}

// -- submodules ----------------------------------------------------------

/// Both backends list the same submodules with the same state, and walk
/// one through its life: declared but not cloned (`Uninitialized`), cloned
/// by `submodule_update --init` (`UpToDate`), moved on inside
/// (`Modified`) and put back by `submodule_update`. A repository without
/// submodules answers an empty list.
pub fn check_submodules_list_init_and_update(backend: &dyn GitBackend) {
    use gitpoulpi_core::domain::{CloneOptions, SubmoduleState, SubmoduleUpdateOptions};

    crate::allow_file_submodules();
    let plain = TestRepoBuilder::new().with_initial().build();
    assert_eq!(backend.submodules(&plain.repo_ref()).unwrap(), []);

    let repo = TestRepoBuilder::new()
        .with_initial()
        .with_submodule("libs/lib")
        .build();
    let listed = backend.submodules(&repo.repo_ref()).unwrap();
    assert_eq!(listed.len(), 1, "{listed:?}");
    let one = &listed[0];
    assert_eq!(
        (one.name.as_str(), one.path.as_str()),
        ("libs/lib", "libs/lib")
    );
    assert!(one.url.ends_with("submodule-source-0"), "{}", one.url);
    assert_eq!(one.state, SubmoduleState::UpToDate);
    assert!(one.expected.is_some());
    assert_eq!(one.head, one.expected);

    // A clone of it has the submodule declared and recorded, not cloned.
    let holder = tempfile::tempdir().unwrap();
    let dest = holder.path().join("clone");
    let clone = crate::contract::block_on(backend.clone(
        &repo.root().to_string_lossy(),
        &dest,
        &CloneOptions::default(),
    ))
    .unwrap();
    let listed = backend.submodules(&clone).unwrap();
    assert_eq!(listed.len(), 1, "{listed:?}");
    assert_eq!(listed[0].state, SubmoduleState::Uninitialized);
    assert_eq!(listed[0].head, None);
    assert_eq!(listed[0].expected, one.expected);

    backend
        .submodule_init(&clone, &["libs/lib".to_string()])
        .unwrap();
    assert!(
        backend
            .submodule_init(&clone, &["no/such/path".to_string()])
            .is_err(),
        "an unknown path is an error"
    );
    backend
        .submodule_update(
            &clone,
            &SubmoduleUpdateOptions {
                paths: vec![],
                init: true,
                recursive: true,
            },
        )
        .unwrap();
    let listed = backend.submodules(&clone).unwrap();
    assert_eq!(listed[0].state, SubmoduleState::UpToDate, "{listed:?}");
    assert_eq!(listed[0].head, listed[0].expected);

    // Move the checked-out submodule on: Modified, then updated back.
    let inside = dest.join("libs/lib");
    repo.git_at(
        &inside,
        &[
            "-c",
            "user.name=Testkit",
            "-c",
            "user.email=testkit@gitpoulpi.local",
            "commit",
            "--allow-empty",
            "-m",
            "ahead",
        ],
    );
    let listed = backend.submodules(&clone).unwrap();
    assert_eq!(listed[0].state, SubmoduleState::Modified, "{listed:?}");
    assert_ne!(listed[0].head, listed[0].expected);
    backend
        .submodule_update(
            &clone,
            &SubmoduleUpdateOptions {
                paths: vec!["libs/lib".to_string()],
                init: false,
                recursive: false,
            },
        )
        .unwrap();
    let listed = backend.submodules(&clone).unwrap();
    assert_eq!(listed[0].state, SubmoduleState::UpToDate, "{listed:?}");
}

// -- image blobs and exact bytes ------------------------------------------

fn rgba_4x2() -> Vec<u8> {
    (0..8u8).flat_map(|i| [i * 30, 255 - i, i, 255]).collect()
}

/// A binary blob that is an image says so (`Blob.image`), at every source;
/// text and non-image binaries do not.
pub fn check_read_blob_of_an_image_carries_its_header(backend: &dyn GitBackend) {
    use gitpoulpi_core::domain::ImageFormat;
    let repo = TestRepoBuilder::new()
        .commit_png("img/logo.png", 4, 2, &rgba_4x2(), "png")
        .commit_bytes("blob.bin", &[0, 1, 2, 0, 3], "bin")
        .commit_file("t.txt", "text\n", "text")
        .build();
    let repo_ref = repo.repo_ref();
    let head = head_oid(backend, &repo_ref);
    for source in [
        BlobSource::Commit(head),
        BlobSource::Index,
        BlobSource::WorkingTree,
    ] {
        let blob = backend
            .read_blob(&repo_ref, std::path::Path::new("img/logo.png"), &source)
            .unwrap();
        assert!(matches!(blob.content, BlobContent::Binary), "{source:?}");
        let image = blob.image.unwrap_or_else(|| panic!("{source:?}: no image"));
        assert_eq!(
            (image.format, image.width, image.height),
            (ImageFormat::Png, 4, 2),
            "{source:?}"
        );
        assert_eq!(image.has_alpha, Some(true));
    }
    for name in ["blob.bin", "t.txt"] {
        let blob = backend
            .read_blob(
                &repo_ref,
                std::path::Path::new(name),
                &BlobSource::Commit(head),
            )
            .unwrap();
        assert!(blob.image.is_none(), "{name} is not an image");
    }
}

/// The bytes come back exactly as committed / staged / on disk, including
/// a CRLF text file (no line splitting) and a binary one.
pub fn check_read_blob_bytes_are_exact_for_commit_index_and_worktree(backend: &dyn GitBackend) {
    use gitpoulpi_core::domain::BlobBody;
    let png = crate::png_bytes(4, 2, &rgba_4x2());
    let repo = TestRepoBuilder::new()
        .commit_png("a.png", 4, 2, &rgba_4x2(), "png")
        .commit_bytes("crlf.txt", b"one\r\ntwo", "crlf")
        .build();
    let repo_ref = repo.repo_ref();
    let head = head_oid(backend, &repo_ref);
    let bytes = |name: &str, source: &BlobSource| {
        let b = backend
            .read_blob_bytes(&repo_ref, std::path::Path::new(name), source, 1 << 20)
            .unwrap();
        assert_eq!(b.path, std::path::Path::new(name));
        assert_eq!(&b.source, source);
        match b.body {
            BlobBody::Bytes(v) => {
                assert_eq!(b.size, v.len() as u64);
                v
            }
            other => panic!("{name} {source:?}: expected bytes, got {other:?}"),
        }
    };
    assert_eq!(bytes("a.png", &BlobSource::Commit(head)), png);
    assert_eq!(bytes("a.png", &BlobSource::Index), png);
    assert_eq!(bytes("a.png", &BlobSource::WorkingTree), png);
    assert_eq!(bytes("crlf.txt", &BlobSource::Commit(head)), b"one\r\ntwo");

    // Staged and on-disk versions differ from the committed one.
    let staged = vec![1u8, 0, 2, 0, 3];
    std::fs::write(repo.root().join("a.png"), &staged).unwrap();
    repo.git(&["add", "a.png"]);
    let on_disk = vec![9u8, 0, 9];
    std::fs::write(repo.root().join("a.png"), &on_disk).unwrap();
    assert_eq!(bytes("a.png", &BlobSource::Commit(head)), png);
    assert_eq!(bytes("a.png", &BlobSource::Index), staged);
    assert_eq!(bytes("a.png", &BlobSource::WorkingTree), on_disk);
}

/// Over the cap nothing is read (`TooLarge`, with the real size); a path
/// that is not there is an `Err`.
pub fn check_read_blob_bytes_respects_the_cap_and_missing_paths(backend: &dyn GitBackend) {
    use gitpoulpi_core::domain::BlobBody;
    let repo = TestRepoBuilder::new()
        .commit_bytes("big.bin", &[7u8; 100], "big")
        .build();
    let repo_ref = repo.repo_ref();
    let head = head_oid(backend, &repo_ref);
    let path = std::path::Path::new("big.bin");
    for source in [
        BlobSource::Commit(head),
        BlobSource::Index,
        BlobSource::WorkingTree,
    ] {
        let at_cap = backend
            .read_blob_bytes(&repo_ref, path, &source, 100)
            .unwrap();
        assert!(matches!(at_cap.body, BlobBody::Bytes(ref v) if v.len() == 100));
        let over = backend
            .read_blob_bytes(&repo_ref, path, &source, 99)
            .unwrap();
        assert_eq!(over.size, 100, "{source:?}");
        assert_eq!(over.body, BlobBody::TooLarge, "{source:?}");
        assert!(
            backend
                .read_blob_bytes(&repo_ref, std::path::Path::new("nope.png"), &source, 100)
                .is_err(),
            "{source:?}: a missing path is an error"
        );
    }
}

/// A pointer blob: with the object in the local LFS store the real bytes
/// come back, without it `object` is `None`, and a cap (still above the
/// pointer text itself) smaller than the
/// object also leaves it `None`. No `git-lfs` program involved.
pub fn check_read_blob_bytes_of_an_lfs_pointer_carries_the_local_object(backend: &dyn GitBackend) {
    use gitpoulpi_core::domain::BlobBody;
    let oid = "ab".repeat(32);
    let real: Vec<u8> = (0..500u32).map(|i| (i % 251) as u8).collect();
    let repo = TestRepoBuilder::new()
        .commit_pointer_text("pic.png", &oid, real.len() as u64, "pointer")
        .build();
    let repo_ref = repo.repo_ref();
    let head = head_oid(backend, &repo_ref);
    let path = std::path::Path::new("pic.png");
    let read = |max| {
        backend
            .read_blob_bytes(&repo_ref, path, &BlobSource::Commit(head), max)
            .unwrap()
    };

    match read(1 << 20).body {
        BlobBody::Lfs { pointer, object } => {
            assert_eq!(pointer.oid, oid);
            assert_eq!(object, None, "object not in the store yet");
        }
        other => panic!("expected an LFS body, got {other:?}"),
    }

    let dir = repo
        .root()
        .join(".git/lfs/objects")
        .join(&oid[0..2])
        .join(&oid[2..4]);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join(&oid), &real).unwrap();
    match read(1 << 20).body {
        BlobBody::Lfs { object, .. } => assert_eq!(object, Some(real.clone())),
        other => panic!("expected an LFS body, got {other:?}"),
    }
    match read(real.len() as u64 - 1).body {
        BlobBody::Lfs { object, .. } => assert_eq!(object, None, "object over the cap"),
        other => panic!("expected an LFS body, got {other:?}"),
    }
}

// -- which rule file a pattern goes to ----------------------------------

/// `gitignore_add` also writes `.git/info/exclude` (the per-clone file), and
/// a pattern made for a folder's own `.gitignore` really ignores the file
/// there (`git check-ignore`), and only there.
pub fn check_gitignore_add_into_a_chosen_file(backend: &dyn GitBackend) {
    use gitpoulpi_core::domain::{rule_pattern, RuleKind, RuleScope};
    let repo = TestRepoBuilder::new()
        .commit_file("a.txt", "a\n", "first")
        .build();
    let repo_ref = repo.repo_ref();
    for dir in ["src/a", "other/a"] {
        std::fs::create_dir_all(repo.root().join(dir)).unwrap();
        std::fs::write(repo.root().join(dir).join("b.txt"), "x").unwrap();
    }
    let ignored = |path: &str| {
        std::process::Command::new("git")
            .args(["check-ignore", "-q", path])
            .current_dir(repo.root())
            .status()
            .unwrap()
            .success()
    };

    // Inside src/.gitignore: relative to src/, so it never reaches other/.
    let pattern = rule_pattern(
        RuleKind::Ignore,
        "src/.gitignore",
        "src/a/b.txt",
        RuleScope::File,
    )
    .unwrap();
    assert_eq!(pattern, "/a/b.txt");
    assert!(backend
        .gitignore_add(&repo_ref, &pattern, Some("src/.gitignore"))
        .unwrap());
    assert!(ignored("src/a/b.txt"));
    assert!(!ignored("other/a/b.txt"), "anchored to src/");
    assert!(!repo.root().join(".gitignore").exists());

    // The per-clone file: ignored, nothing in the working tree changes.
    let ext = rule_pattern(
        RuleKind::Ignore,
        ".git/info/exclude",
        "other/a/b.txt",
        RuleScope::Extension,
    )
    .unwrap();
    assert!(backend
        .gitignore_add(&repo_ref, &ext, Some(".git/info/exclude"))
        .unwrap());
    assert!(ignored("other/a/b.txt"));
    let exclude = std::fs::read_to_string(repo.root().join(".git/info/exclude")).unwrap();
    assert!(exclude.lines().any(|l| l == "*.txt"), "{exclude}");
    assert!(
        !backend
            .gitignore_add(&repo_ref, &ext, Some(".git/info/exclude"))
            .unwrap(),
        "already there"
    );
    assert!(backend
        .gitignore_add(&repo_ref, "x", Some(".git/info/attributes"))
        .is_err());
}

fn working_diff_has_hunks(
    backend: &dyn GitBackend,
    repo: &gitpoulpi_core::domain::RepoRef,
    path: &str,
) -> bool {
    backend
        .diff(repo, DiffTarget::WorkingTreeVsIndex)
        .unwrap()
        .iter()
        .find(|f| f.path.to_str() == Some(path))
        .is_some_and(|f| !f.hunks.is_empty())
}

/// `gitattributes_treat`: "Treat as Binary" makes a text file's diff
/// hunk-less, "Treat as Text" gives a NUL-holding file hunks, a
/// contradicting rule for the same pattern is replaced rather than
/// appended, every target file works, and `binary_paths` agrees with the
/// diff in all of it. Both backends must see the same.
pub fn check_gitattributes_treat_binary_and_text(backend: &dyn GitBackend) {
    use gitpoulpi_core::domain::{RuleScope, Treat};
    let repo = TestRepoBuilder::new()
        .commit_file("notes.dat", "one\ntwo\n", "text")
        .commit_bytes("raw.bin", b"a\0b\nc\n", "binary-looking")
        .build();
    let repo_ref = repo.repo_ref();
    std::fs::write(repo.root().join("notes.dat"), "one\nchanged\n").unwrap();
    std::fs::write(repo.root().join("raw.bin"), b"a\0b\nchanged\n").unwrap();
    let paths = |p: &[&str]| p.iter().map(|s| s.to_string()).collect::<Vec<_>>();
    let treat = |p: &[&str], scope, file: Option<&str>, treat| {
        backend
            .gitattributes_treat(&repo_ref, &paths(p), scope, file, treat)
            .unwrap()
    };
    let attrs = |name: &str| std::fs::read_to_string(repo.root().join(name)).unwrap();

    assert_eq!(
        backend
            .binary_paths(&repo_ref, &paths(&["notes.dat", "raw.bin", "missing"]))
            .unwrap(),
        [false, true, false]
    );
    assert!(working_diff_has_hunks(backend, &repo_ref, "notes.dat"));
    assert!(!working_diff_has_hunks(backend, &repo_ref, "raw.bin"));

    // Text -> binary, this file only.
    let done = treat(&["notes.dat"], RuleScope::File, None, Treat::Binary);
    assert_eq!(done.file, ".gitattributes");
    assert_eq!(done.patterns, ["/notes.dat"]);
    assert!(done.changed && done.effective);
    assert_eq!(attrs(".gitattributes"), "/notes.dat binary\n");
    assert_eq!(
        backend
            .binary_paths(&repo_ref, &paths(&["notes.dat"]))
            .unwrap(),
        [true]
    );
    assert!(!working_diff_has_hunks(backend, &repo_ref, "notes.dat"));

    // Again: nothing to write. Then flipped back: the line is replaced.
    let again = treat(&["notes.dat"], RuleScope::File, None, Treat::Binary);
    assert!(!again.changed && again.effective);
    treat(&["notes.dat"], RuleScope::File, None, Treat::Text);
    assert_eq!(attrs(".gitattributes"), "/notes.dat text diff\n");
    assert!(working_diff_has_hunks(backend, &repo_ref, "notes.dat"));

    // Binary-looking -> text, by extension, in a folder's own file.
    std::fs::create_dir_all(repo.root().join("sub")).unwrap();
    std::fs::write(repo.root().join("sub/x.bin"), b"q\0r\n").unwrap();
    let sub = treat(
        &["sub/x.bin"],
        RuleScope::Extension,
        Some("sub/.gitattributes"),
        Treat::Text,
    );
    assert_eq!(sub.file, "sub/.gitattributes");
    assert_eq!(sub.patterns, ["*.bin"]);
    assert_eq!(
        backend
            .binary_paths(&repo_ref, &paths(&["sub/x.bin", "raw.bin"]))
            .unwrap(),
        [false, true],
        "the rule governs sub/ only"
    );

    // The root-level extension rule, and the diff follows.
    treat(&["raw.bin"], RuleScope::Extension, None, Treat::Text);
    assert!(working_diff_has_hunks(backend, &repo_ref, "raw.bin"));

    // The per-clone file: nothing lands in the working tree, and it wins.
    let local = treat(
        &["notes.dat"],
        RuleScope::Extension,
        Some(".git/info/attributes"),
        Treat::Binary,
    );
    assert_eq!(local.file, ".git/info/attributes");
    assert!(local.effective);
    assert!(!working_diff_has_hunks(backend, &repo_ref, "notes.dat"));

    // A rule closer to the file wins: say so instead of claiming success.
    let shadow = treat(&["sub/x.bin"], RuleScope::File, None, Treat::Binary);
    assert!(
        shadow.changed && !shadow.effective,
        "sub/.gitattributes still says text"
    );

    // Folders: everything below.
    let folder = treat(&["sub"], RuleScope::Folder, None, Treat::Binary);
    assert_eq!(folder.patterns, ["/sub/**"]);

    for (file, path) in [
        (Some("notes.dat"), "notes.dat"),
        (Some("../.gitattributes"), "notes.dat"),
        (Some(".gitignore"), "notes.dat"),
        (None, "../x"),
        (None, ".git/config"),
    ] {
        assert!(
            backend
                .gitattributes_treat(
                    &repo_ref,
                    &paths(&[path]),
                    RuleScope::File,
                    file,
                    Treat::Binary
                )
                .is_err(),
            "{file:?} {path:?}"
        );
    }
    assert!(backend
        .gitattributes_treat(&repo_ref, &[], RuleScope::File, None, Treat::Binary)
        .is_err());
    assert!(backend
        .gitattributes_treat(
            &repo_ref,
            &paths(&["sub/no_ext"]),
            RuleScope::Extension,
            None,
            Treat::Binary
        )
        .is_err());
}

/// A repository addressed through a symlink to its directory (macOS puts
/// temp dirs behind `/var` -> `/private/var`; Windows has junctions and
/// `subst` drives) answers exactly as when addressed by its real path:
/// attributes, status, diff, staging, blame, file history and rule files.
/// The application canonicalises the root once (`RepoRef::from_root`), but
/// the backends must not depend on that for path math.
#[cfg(unix)]
pub fn check_a_symlinked_root_behaves_like_the_real_one(backend: &dyn GitBackend) {
    use gitpoulpi_core::domain::{RepoRef, RuleScope, Treat};
    let repo = TestRepoBuilder::new()
        .commit_file(".gitattributes", "*.dat binary\n", "rules")
        .commit_file("notes.dat", "one\ntwo\n", "text")
        .commit_file("sub/plain.txt", "a\nb\n", "plain")
        .build();
    let links = tempfile::tempdir().unwrap();
    let link = links.path().join("via-link");
    std::os::unix::fs::symlink(repo.root(), &link).unwrap();
    let real = repo.repo_ref();
    let linked = RepoRef {
        id: real.id.clone(),
        root: link.clone(),
    };
    std::fs::write(repo.root().join("notes.dat"), "one\nchanged\n").unwrap();
    std::fs::write(repo.root().join("sub/plain.txt"), "a\nchanged\n").unwrap();
    let paths = |p: &[&str]| p.iter().map(|s| s.to_string()).collect::<Vec<_>>();

    // Attributes: the committed rule is seen, the content sniff is not used.
    assert_eq!(
        backend
            .binary_paths(&linked, &paths(&["notes.dat", "sub/plain.txt"]))
            .unwrap(),
        [true, false],
        "a `*.dat binary` rule must be seen through a symlinked root"
    );
    assert_eq!(
        backend.status(&linked).unwrap().files,
        backend.status(&real).unwrap().files
    );
    assert_eq!(
        backend
            .diff(&linked, DiffTarget::WorkingTreeVsIndex)
            .unwrap(),
        backend.diff(&real, DiffTarget::WorkingTreeVsIndex).unwrap()
    );

    // Blame and file history by repo-relative path.
    let tip = backend.log(&real, &LogOptions::default()).unwrap().nodes[0].id;
    let path = std::path::Path::new("sub/plain.txt");
    assert_eq!(
        backend.blame(&linked, path, tip).unwrap(),
        backend.blame(&real, path, tip).unwrap()
    );
    assert_eq!(
        backend.file_history(&linked, path, tip, 0, 10).unwrap(),
        backend.file_history(&real, path, tip, 0, 10).unwrap()
    );

    // Rule files are written inside the real directory.
    let done = backend
        .gitattributes_treat(
            &linked,
            &paths(&["sub/plain.txt"]),
            RuleScope::File,
            None,
            Treat::Binary,
        )
        .unwrap();
    assert!(done.changed && done.effective, "{done:?}");
    assert_eq!(
        std::fs::read_to_string(repo.root().join(".gitattributes")).unwrap(),
        "*.dat binary\n/sub/plain.txt binary\n"
    );
    assert!(backend
        .gitignore_add(&linked, "/scratch.tmp", None)
        .unwrap());

    // Staging goes through the linked root too.
    backend
        .stage(&linked, &StageTarget::File("notes.dat".into()))
        .unwrap();
    let staged = backend.status(&real).unwrap();
    assert!(
        staged
            .files
            .iter()
            .any(|f| f.path.to_str() == Some("notes.dat") && f.index.is_some()),
        "{staged:?}"
    );
}

/// Windows junctions need no privilege but `std` has no call for them; the
/// symlink case above is what the suite can build portably.
#[cfg(not(unix))]
pub fn check_a_symlinked_root_behaves_like_the_real_one(_backend: &dyn GitBackend) {}
