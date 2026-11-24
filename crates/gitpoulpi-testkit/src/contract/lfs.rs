use super::{block_on, head_oid};
use crate::TestRepoBuilder;
use gitpoulpi_core::domain::{
    BlameContent, BlobContent, BlobSource, CheckoutTarget, CloneOptions, ClonePhase, CloneProgress,
    CommitOptions, DiffTarget, FastForwardOutcome, LogOptions, ResetMode, StageTarget,
};
use gitpoulpi_core::GitBackend;

/// A committed Git LFS pointer is reported as one - by `read_blob` at a
/// commit and at the index, and by `blame` - rather than as three lines
/// of text. Needs no `git-lfs` binary: the pointer is just a blob.
pub fn check_read_blob_reports_an_lfs_pointer_at_commit_and_index(backend: &dyn GitBackend) {
    let oid = "a".repeat(64);
    let repo = TestRepoBuilder::new()
        .commit_pointer_text("big.bin", &oid, 4242, "add pointer")
        .build();
    let repo_ref = repo.repo_ref();
    let head = head_oid(backend, &repo_ref);
    let path = std::path::Path::new("big.bin");

    for source in [BlobSource::Commit(head), BlobSource::Index] {
        let blob = backend.read_blob(&repo_ref, path, &source).unwrap();
        match blob.content {
            BlobContent::LfsPointer(p) => {
                assert_eq!(p.oid, oid, "{source:?}");
                assert_eq!(p.size, 4242, "{source:?}");
            }
            other => panic!("{source:?}: expected an LFS pointer, got {other:?}"),
        }
    }

    let blame = backend.blame(&repo_ref, path, head).unwrap();
    assert!(
        matches!(blame.content, BlameContent::LfsPointer(ref p) if p.size == 4242),
        "blame of a pointer must say so, got {:?}",
        blame.content
    );
}

/// A commit that changes an LFS pointer produces a diff that
/// `lfs_change` recognises, with both the old and the new pointer.
pub fn check_diff_of_a_pointer_change_is_recognized(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .commit_pointer_text("big.bin", &"a".repeat(64), 10, "first")
        .commit_pointer_text("big.bin", &"b".repeat(64), 20, "second")
        .build();
    let repo_ref = repo.repo_ref();
    let head = head_oid(backend, &repo_ref);

    let files = backend
        .diff(&repo_ref, DiffTarget::CommitVsParent(head))
        .unwrap();
    assert_eq!(files.len(), 1);
    let change = gitpoulpi_core::domain::lfs_change(&files[0])
        .expect("a pointer-to-pointer change must be recognized as an LFS change");
    assert_eq!(change.old.expect("old pointer").size, 10);
    let new = change.new.expect("new pointer");
    assert_eq!(new.size, 20);
    assert_eq!(new.oid, "b".repeat(64));
}

// -- Git LFS: reads (no `git-lfs` program needed unless stated) ----------

/// A repository with no LFS patterns and no LFS store is reported as not
/// using LFS, with no patterns and no LFS hook - whether or not the
/// program is installed. (`filters_configured` is not asserted: the
/// backends honour the user's global config, where `git lfs install`
/// usually put the filters.)
pub fn check_lfs_status_of_a_plain_repo_says_unused(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new().with_initial().build();
    let repo_ref = repo.repo_ref();

    let status = backend.lfs_status(&repo_ref).unwrap();
    assert!(!status.uses_lfs);
    assert!(status.patterns.is_empty());
    assert!(!status.hooks_installed);
    assert!(
        backend.lfs_files(&repo_ref, None).unwrap().is_empty(),
        "a repository without LFS has no LFS files"
    );
}

/// After `git lfs install --local` and two `git lfs track`s the status
/// names both patterns (one lockable), the filters, the hook and the
/// installed program.
pub fn check_lfs_status_reports_patterns_hooks_and_binary(backend: &dyn GitBackend) {
    crate::require_git_lfs!("check_lfs_status_reports_patterns_hooks_and_binary");
    let repo = TestRepoBuilder::new()
        .with_lfs()
        .lfs_track("*.psd")
        .lfs_track_lockable("assets/*.bin")
        .build();
    let repo_ref = repo.repo_ref();

    let status = backend.lfs_status(&repo_ref).unwrap();
    assert!(status.uses_lfs);
    assert!(status.filters_configured);
    assert!(status.hooks_installed);
    assert!(
        matches!(
            status.binary,
            gitpoulpi_core::domain::LfsBinary::Installed { .. }
        ),
        "{:?}",
        status.binary
    );
    let mut found: Vec<(String, bool)> = status
        .patterns
        .iter()
        .map(|p| (p.pattern.clone(), p.lockable))
        .collect();
    found.sort();
    assert_eq!(
        found,
        [
            ("*.psd".to_string(), false),
            ("assets/*.bin".to_string(), true)
        ]
    );
    assert!(status
        .patterns
        .iter()
        .all(|p| p.source == std::path::Path::new(".gitattributes")));
}

/// `lfs_files` lists exactly the pointer files, and `local` says whether
/// the real object is in the local store: false in a clone that left
/// pointers behind, true once `git lfs pull` has run.
pub fn check_lfs_files_lists_pointers_with_local_flag(backend: &dyn GitBackend) {
    crate::require_git_lfs!("check_lfs_files_lists_pointers_with_local_flag");
    let content: &[u8] = b"not a pointer, the real payload";
    let repo = TestRepoBuilder::new()
        .with_lfs_origin()
        .with_lfs()
        .lfs_track("*.bin")
        .commit_lfs_file("data/big.bin", content, "add big")
        .commit_file("notes.txt", "plain text\n", "add notes")
        .build();
    repo.git(&["push", "origin", "main"]);

    // The source repository has the object locally (it just cleaned it).
    let source = backend.lfs_files(&repo.repo_ref(), None).unwrap();
    assert_eq!(source.len(), 1, "{source:?}");
    assert_eq!(source[0].path, std::path::Path::new("data/big.bin"));
    assert_eq!(source[0].pointer.size, content.len() as u64);
    assert!(source[0].local);

    let dest_dir = tempfile::tempdir().unwrap();
    let dest = dest_dir.path().join("clone");
    crate::lfs::clone_leaving_pointers(repo.remote_path().unwrap(), &dest);
    let clone_ref = gitpoulpi_core::domain::RepoRef::from_root(&dest);

    let before = backend.lfs_files(&clone_ref, None).unwrap();
    assert_eq!(before.len(), 1, "{before:?}");
    assert!(!before[0].local, "nothing was downloaded yet");
    assert_eq!(before[0].pointer, source[0].pointer);

    crate::lfs::hermetic_git(&dest, &["lfs", "install", "--local"]);
    crate::lfs::hermetic_git(&dest, &["lfs", "pull"]);
    let after = backend.lfs_files(&clone_ref, None).unwrap();
    assert!(after[0].local, "git lfs pull stored the object");
}

/// `lfs_files(Some(commit))` reads the pointers *as of that commit*.
pub fn check_lfs_files_at_a_commit(backend: &dyn GitBackend) {
    crate::require_git_lfs!("check_lfs_files_at_a_commit");
    let repo = TestRepoBuilder::new()
        .with_lfs()
        .lfs_track("*.bin")
        .commit_lfs_file("a.bin", b"first version of the payload", "v1")
        .commit_lfs_file("a.bin", b"second, longer version of the payload", "v2")
        .commit_lfs_file("b.bin", b"another object", "add b")
        .build();
    let repo_ref = repo.repo_ref();
    let log = backend.log(&repo_ref, &LogOptions::default()).unwrap();
    let by_summary = |s: &str| log.nodes.iter().find(|n| n.summary == s).unwrap().id;

    let at_v1 = backend
        .lfs_files(&repo_ref, Some(by_summary("v1")))
        .unwrap();
    assert_eq!(at_v1.len(), 1);
    assert_eq!(at_v1[0].pointer.size, 28);

    let at_v2 = backend
        .lfs_files(&repo_ref, Some(by_summary("v2")))
        .unwrap();
    assert_eq!(at_v2.len(), 1);
    assert_eq!(at_v2[0].pointer.size, 37);

    let now = backend.lfs_files(&repo_ref, None).unwrap();
    let mut names: Vec<String> = now.iter().map(|f| f.path.display().to_string()).collect();
    names.sort();
    assert_eq!(names, ["a.bin", "b.bin"]);
}

/// A commit's pointers are listed whatever the checked-out attributes say
/// and without the `git-lfs` program: the View all files LFS filter asks
/// about a revision other than the one in the index (another branch, a
/// file since deleted). The index answer stays empty for a repository that
/// does not route files through LFS.
pub fn check_lfs_files_at_a_commit_needs_no_attributes_or_program(backend: &dyn GitBackend) {
    let oid = "b".repeat(64);
    let repo = TestRepoBuilder::new()
        .commit_pointer_text("assets/logo.png", &oid, 777, "add pointer")
        .build();
    let repo_ref = repo.repo_ref();
    let head = head_oid(backend, &repo_ref);
    let at = backend.lfs_files(&repo_ref, Some(head)).unwrap();
    assert_eq!(at.len(), 1, "{at:?}");
    assert_eq!(at[0].path, std::path::Path::new("assets/logo.png"));
    assert_eq!(at[0].pointer.size, 777);
    assert!(!at[0].local);
    assert!(
        backend.lfs_files(&repo_ref, None).unwrap().is_empty(),
        "no attributes and no store: the index scan is skipped"
    );
}

// -- Git LFS: routing (libgit2 has no smudge filter) ---------------------
//
// In a repository that uses LFS, an operation that writes the working
// tree or compares it with the index must go through the system git,
// which runs the LFS filters; natively, libgit2 would write pointer
// *text* and report every stat-dirty LFS file as modified (ADR 0009).
// These checks need the `git-lfs` program.

const LFS_V1: &[u8] = b"first version of the large file";

const LFS_V2: &[u8] = b"second, quite different large file";

fn lfs_worktree_bytes(repo: &crate::FixtureRepo, path: &str) -> Vec<u8> {
    std::fs::read(repo.root().join(path)).unwrap()
}

/// `checkout` of a branch whose LFS file differs writes the real content.
pub fn check_checkout_in_an_lfs_repo_writes_content_not_pointers(backend: &dyn GitBackend) {
    crate::require_git_lfs!("check_checkout_in_an_lfs_repo_writes_content_not_pointers");
    let repo = TestRepoBuilder::new()
        .with_lfs()
        .lfs_track("*.bin")
        .commit_lfs_file("a.bin", LFS_V1, "v1")
        .branch("other")
        .commit_lfs_file("a.bin", LFS_V2, "v2 on other")
        .checkout("main")
        .build();
    let repo_ref = repo.repo_ref();
    assert_eq!(lfs_worktree_bytes(&repo, "a.bin"), LFS_V1);

    backend
        .checkout(&repo_ref, &CheckoutTarget::Branch("other".into()))
        .unwrap();
    assert_eq!(
        lfs_worktree_bytes(&repo, "a.bin"),
        LFS_V2,
        "the working tree must hold the real content, not pointer text"
    );
    assert!(backend.status(&repo_ref).unwrap().files.is_empty());
}

/// `reset --hard` to an older commit restores the real content.
pub fn check_reset_hard_in_an_lfs_repo_writes_content(backend: &dyn GitBackend) {
    crate::require_git_lfs!("check_reset_hard_in_an_lfs_repo_writes_content");
    let repo = TestRepoBuilder::new()
        .with_lfs()
        .lfs_track("*.bin")
        .commit_lfs_file("a.bin", LFS_V1, "v1")
        .commit_lfs_file("a.bin", LFS_V2, "v2")
        .build();
    let repo_ref = repo.repo_ref();
    let log = backend.log(&repo_ref, &LogOptions::default()).unwrap();
    let v1 = log.nodes.iter().find(|n| n.summary == "v1").unwrap().id;

    backend.reset(&repo_ref, v1, ResetMode::Hard).unwrap();
    assert_eq!(lfs_worktree_bytes(&repo, "a.bin"), LFS_V1);
    assert!(backend.status(&repo_ref).unwrap().files.is_empty());
}

/// Fast-forwarding the checked-out branch updates the working tree with
/// real content.
pub fn check_fast_forward_of_the_checked_out_branch_in_an_lfs_repo(backend: &dyn GitBackend) {
    crate::require_git_lfs!("check_fast_forward_of_the_checked_out_branch_in_an_lfs_repo");
    let repo = TestRepoBuilder::new()
        .with_lfs()
        .lfs_track("*.bin")
        .commit_lfs_file("a.bin", LFS_V1, "v1")
        .branch("ahead")
        .commit_lfs_file("a.bin", LFS_V2, "v2")
        .checkout("main")
        .build();
    let repo_ref = repo.repo_ref();
    let target = backend.resolve(&repo_ref, "ahead").unwrap();

    let outcome = backend.fast_forward(&repo_ref, "main", target).unwrap();
    assert!(
        matches!(outcome, FastForwardOutcome::FastForwarded { .. }),
        "{outcome:?}"
    );
    assert_eq!(lfs_worktree_bytes(&repo, "a.bin"), LFS_V2);
    assert!(backend.status(&repo_ref).unwrap().files.is_empty());
}

/// Rewriting an LFS file with the same bytes changes its mtime and forces
/// git to compare content; the comparison must go through the clean
/// filter, so the file is still clean. (libgit2 hashes the raw file
/// against the pointer blob and calls it modified.)
pub fn check_lfs_file_is_clean_after_its_mtime_changes(backend: &dyn GitBackend) {
    crate::require_git_lfs!("check_lfs_file_is_clean_after_its_mtime_changes");
    let repo = TestRepoBuilder::new()
        .with_lfs()
        .lfs_track("*.bin")
        .commit_lfs_file("a.bin", LFS_V1, "v1")
        .build();
    let repo_ref = repo.repo_ref();

    std::thread::sleep(std::time::Duration::from_millis(1100));
    std::fs::write(repo.root().join("a.bin"), LFS_V1).unwrap();

    let status = backend.status(&repo_ref).unwrap();
    assert!(
        status.files.is_empty(),
        "an LFS file with unchanged content is clean: {:?}",
        status.files
    );
}

/// A modified LFS file's working-tree diff is a *pointer change* whose
/// new side carries the new size.
pub fn check_diff_of_a_modified_lfs_file_is_a_pointer_change(backend: &dyn GitBackend) {
    crate::require_git_lfs!("check_diff_of_a_modified_lfs_file_is_a_pointer_change");
    let repo = TestRepoBuilder::new()
        .with_lfs()
        .lfs_track("*.bin")
        .commit_lfs_file("a.bin", LFS_V1, "v1")
        .write_worktree("a.bin", LFS_V2)
        .build();
    let repo_ref = repo.repo_ref();

    let files = backend
        .diff(&repo_ref, DiffTarget::WorkingTreeVsIndex)
        .unwrap();
    assert_eq!(files.len(), 1, "{files:?}");
    let change = gitpoulpi_core::domain::lfs_change(&files[0])
        .expect("the diff of an LFS file must be a pointer diff");
    assert_eq!(change.old.expect("old").size, LFS_V1.len() as u64);
    assert_eq!(change.new.expect("new").size, LFS_V2.len() as u64);
}

// -- Git LFS: install, track, untrack ------------------------------------

/// `lfs_install` (repository scope) writes the filters into the
/// repository's own config and installs the hook.
pub fn check_lfs_install_configures_filters_and_hooks(backend: &dyn GitBackend) {
    crate::require_git_lfs!("check_lfs_install_configures_filters_and_hooks");
    let repo = TestRepoBuilder::new().with_initial().build();
    let repo_ref = repo.repo_ref();
    assert!(!backend.lfs_status(&repo_ref).unwrap().hooks_installed);

    backend
        .lfs_install(
            &repo_ref,
            gitpoulpi_core::domain::LfsInstallScope::Repository,
        )
        .unwrap();

    // Asked of git directly, `--local`: the user's global config is not
    // what answers this.
    let process = repo.git(&["config", "--local", "--get", "filter.lfs.process"]);
    assert!(process.contains("git-lfs"), "{process}");
    let status = backend.lfs_status(&repo_ref).unwrap();
    assert!(status.filters_configured);
    assert!(status.hooks_installed);
    let hook = std::fs::read_to_string(repo.root().join(".git/hooks/pre-push")).unwrap();
    assert!(hook.contains("git lfs"), "{hook}");
}

/// `lfs_track` writes `.gitattributes` (unstaged), so a file added and
/// committed afterwards is a pointer in git; with `renormalize` a file
/// that was already committed is re-staged as a pointer too.
pub fn check_lfs_track_writes_gitattributes_and_next_commit_stores_a_pointer(
    backend: &dyn GitBackend,
) {
    crate::require_git_lfs!(
        "check_lfs_track_writes_gitattributes_and_next_commit_stores_a_pointer"
    );
    let repo = TestRepoBuilder::new()
        .with_lfs()
        .commit_file(
            "old.bin",
            "committed before tracking, as plain content",
            "old",
        )
        .build();
    let repo_ref = repo.repo_ref();

    let patterns = backend
        .lfs_track(
            &repo_ref,
            &["*.bin".to_string()],
            &gitpoulpi_core::domain::LfsTrackOptions {
                lockable: false,
                renormalize: true,
            },
        )
        .unwrap();
    assert_eq!(patterns.len(), 1);
    assert_eq!(patterns[0].pattern, "*.bin");
    let attributes = std::fs::read_to_string(repo.root().join(".gitattributes")).unwrap();
    assert!(attributes.contains("*.bin filter=lfs"), "{attributes}");

    // `.gitattributes` is a visible, unstaged change.
    let status = backend.status(&repo_ref).unwrap();
    let attr = status
        .files
        .iter()
        .find(|f| f.path == std::path::Path::new(".gitattributes"))
        .expect(".gitattributes shows in status");
    assert!(attr.index.is_none(), "{attr:?}");

    // The committed file was re-staged as a pointer.
    let staged = repo.git(&["cat-file", "blob", ":old.bin"]);
    assert!(staged.starts_with("version https://git-lfs"), "{staged}");

    // A new file goes through the clean filter on the next add + commit.
    std::fs::write(repo.root().join("new.bin"), "brand new large content").unwrap();
    backend
        .stage(&repo_ref, &StageTarget::File("new.bin".into()))
        .unwrap();
    backend
        .commit(&repo_ref, "add new.bin", &CommitOptions::default())
        .unwrap();
    let stored = repo.git(&["cat-file", "blob", "HEAD:new.bin"]);
    assert!(stored.starts_with("version https://git-lfs"), "{stored}");
}

/// `lfs_untrack` removes the pattern from `.gitattributes`.
pub fn check_lfs_untrack_removes_the_pattern(backend: &dyn GitBackend) {
    crate::require_git_lfs!("check_lfs_untrack_removes_the_pattern");
    let repo = TestRepoBuilder::new()
        .with_lfs()
        .lfs_track("*.psd")
        .lfs_track("*.bin")
        .build();
    let repo_ref = repo.repo_ref();

    let left = backend
        .lfs_untrack(&repo_ref, &["*.psd".to_string()])
        .unwrap();
    let names: Vec<&str> = left.iter().map(|p| p.pattern.as_str()).collect();
    assert_eq!(names, ["*.bin"]);
    let attributes = std::fs::read_to_string(repo.root().join(".gitattributes")).unwrap();
    assert!(!attributes.contains("*.psd"), "{attributes}");
}

// -- Git LFS: fetch, pull, push over the `file://` adapter ---------------

const LFS_PAYLOAD: &[u8] = b"a payload that only exists in the LFS store";

/// A `with_lfs_origin` fixture with one LFS file committed and **pushed**
/// (by the fixture, not the backend under test), plus a clone of it that
/// holds pointer text and an empty LFS store.
fn lfs_remote_and_pointer_clone() -> (
    crate::FixtureRepo,
    tempfile::TempDir,
    std::path::PathBuf,
    gitpoulpi_core::domain::RepoRef,
) {
    let repo = TestRepoBuilder::new()
        .with_lfs_origin()
        .with_lfs()
        .lfs_track("*.bin")
        .commit_lfs_file("big.bin", LFS_PAYLOAD, "add big")
        .build();
    repo.git(&["push", "origin", "main"]);
    let dir = tempfile::tempdir().unwrap();
    let dest = dir.path().join("clone");
    crate::lfs::clone_leaving_pointers(repo.remote_path().unwrap(), &dest);
    let clone_ref = gitpoulpi_core::domain::RepoRef::from_root(&dest);
    (repo, dir, dest, clone_ref)
}

fn collecting_sink() -> (
    std::sync::Arc<std::sync::Mutex<Vec<gitpoulpi_core::domain::LfsProgress>>>,
    impl Fn(gitpoulpi_core::domain::LfsProgress) + Send + Sync,
) {
    let seen = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let writer = std::sync::Arc::clone(&seen);
    (seen, move |p| writer.lock().unwrap().push(p))
}

/// `lfs_push` uploads the current branch's objects: they appear in the
/// origin's LFS store, which the fixture reads directly.
pub fn check_lfs_push_uploads_objects_to_the_remote(backend: &dyn GitBackend) {
    crate::require_git_lfs!("check_lfs_push_uploads_objects_to_the_remote");
    let repo = TestRepoBuilder::new()
        .with_lfs_origin()
        .with_lfs()
        .lfs_track("*.bin")
        .commit_lfs_file("big.bin", LFS_PAYLOAD, "add big")
        .build();
    let repo_ref = repo.repo_ref();
    assert!(repo.remote_lfs_objects().is_empty());
    let oid = backend.lfs_files(&repo_ref, None).unwrap()[0]
        .pointer
        .oid
        .clone();

    let (_, sink) = collecting_sink();
    block_on(backend.lfs_push(
        &repo_ref,
        "origin",
        &gitpoulpi_core::domain::LfsPushOptions::default(),
        &sink,
    ))
    .unwrap();

    assert_eq!(repo.remote_lfs_objects(), [oid]);
}

/// `lfs_fetch` fills the local store without touching the working tree:
/// `lfs_files` says `local` while the file on disk is still pointer text.
pub fn check_lfs_fetch_then_files_are_local(backend: &dyn GitBackend) {
    crate::require_git_lfs!("check_lfs_fetch_then_files_are_local");
    let (_repo, _dir, dest, clone_ref) = lfs_remote_and_pointer_clone();
    assert!(!backend.lfs_files(&clone_ref, None).unwrap()[0].local);

    let (_, sink) = collecting_sink();
    block_on(backend.lfs_fetch(
        &clone_ref,
        "origin",
        &gitpoulpi_core::domain::LfsFetchOptions::default(),
        &sink,
    ))
    .unwrap();

    assert!(backend.lfs_files(&clone_ref, None).unwrap()[0].local);
    let on_disk = std::fs::read(dest.join("big.bin")).unwrap();
    assert!(
        on_disk.starts_with(b"version https://git-lfs"),
        "fetch must not touch the working tree"
    );
}

/// `lfs_pull` replaces the pointer with the real content.
pub fn check_lfs_pull_materialises_content(backend: &dyn GitBackend) {
    crate::require_git_lfs!("check_lfs_pull_materialises_content");
    let (_repo, _dir, dest, clone_ref) = lfs_remote_and_pointer_clone();

    let (_, sink) = collecting_sink();
    block_on(backend.lfs_pull(&clone_ref, "origin", &sink)).unwrap();

    assert_eq!(std::fs::read(dest.join("big.bin")).unwrap(), LFS_PAYLOAD);
    assert!(backend.lfs_files(&clone_ref, None).unwrap()[0].local);
}

/// `lfs_checkout` expands the pointers whose objects are already in the
/// local store (a prior `lfs_fetch`) into real content, offline.
pub fn check_lfs_checkout_replaces_pointers_with_content(backend: &dyn GitBackend) {
    crate::require_git_lfs!("check_lfs_checkout_replaces_pointers_with_content");
    let (_repo, _dir, dest, clone_ref) = lfs_remote_and_pointer_clone();

    let (_, sink) = collecting_sink();
    block_on(backend.lfs_fetch(
        &clone_ref,
        "origin",
        &gitpoulpi_core::domain::LfsFetchOptions::default(),
        &sink,
    ))
    .unwrap();
    assert!(
        std::fs::read(dest.join("big.bin"))
            .unwrap()
            .starts_with(b"version https://git-lfs"),
        "fetch leaves the pointer"
    );

    let (seen, sink) = collecting_sink();
    block_on(backend.lfs_checkout(&clone_ref, &[], &sink)).unwrap();

    assert_eq!(std::fs::read(dest.join("big.bin")).unwrap(), LFS_PAYLOAD);
    let seen = seen.lock().unwrap();
    assert!(
        seen.last().is_some_and(|p| p.done),
        "the last report says done: {seen:?}"
    );
}

/// Progress invariants, like the clone check's: the last report says
/// `done`, and within a direction no counter ever decreases. A progress
/// bar that goes backwards, or never finishes, is worse than none.
pub fn check_lfs_progress_ends_done_and_never_decreases(backend: &dyn GitBackend) {
    crate::require_git_lfs!("check_lfs_progress_ends_done_and_never_decreases");
    let (_repo, _dir, _dest, clone_ref) = lfs_remote_and_pointer_clone();

    let (seen, sink) = collecting_sink();
    block_on(backend.lfs_pull(&clone_ref, "origin", &sink)).unwrap();

    let seen = seen.lock().unwrap();
    let last = seen.last().expect("at least the final report");
    assert!(last.done, "the last report must be done: {last:?}");
    assert_eq!(last.bytes_done, last.bytes_total);
    assert!(
        seen[..seen.len() - 1].iter().all(|p| !p.done),
        "only the last report is done"
    );
    for direction in [
        gitpoulpi_core::domain::LfsDirection::Download,
        gitpoulpi_core::domain::LfsDirection::Checkout,
        gitpoulpi_core::domain::LfsDirection::Upload,
    ] {
        let mut previous = (0, 0);
        for p in seen.iter().filter(|p| p.direction == direction) {
            assert!(p.bytes_done <= p.bytes_total || p.bytes_total == 0, "{p:?}");
            assert!(
                (p.bytes_done, p.files_done) >= previous,
                "{direction:?} went backwards: {p:?} after {previous:?}"
            );
            previous = (p.bytes_done, p.files_done);
        }
    }
}

/// Cloning a repository that uses LFS leaves the real content in the
/// working tree (the objects are downloaded after the clone), reports the
/// download as its own phase, and still ends `Done` with nothing
/// decreasing within a phase.
pub fn check_clone_of_an_lfs_repo_has_real_content(backend: &dyn GitBackend) {
    crate::require_git_lfs!("check_clone_of_an_lfs_repo_has_real_content");
    let repo = TestRepoBuilder::new()
        .with_lfs_origin()
        .with_lfs()
        .lfs_track("*.bin")
        .commit_lfs_file("big.bin", LFS_PAYLOAD, "add big")
        .build();
    repo.git(&["push", "origin", "main"]);
    let dir = tempfile::tempdir().unwrap();
    let dest = dir.path().join("clone");

    let seen: std::sync::Mutex<Vec<CloneProgress>> = std::sync::Mutex::new(Vec::new());
    let sink = |p: CloneProgress| seen.lock().unwrap().push(p);
    let cloned = block_on(backend.clone_with_progress(
        &repo.remote_path().unwrap().to_string_lossy(),
        &dest,
        &CloneOptions::default(),
        &sink,
    ))
    .unwrap();

    assert_eq!(std::fs::read(dest.join("big.bin")).unwrap(), LFS_PAYLOAD);
    let files = backend.lfs_files(&cloned, None).unwrap();
    assert!(files.len() == 1 && files[0].local, "{files:?}");
    assert!(
        backend.status(&cloned).unwrap().files.is_empty(),
        "a fresh LFS clone is clean"
    );

    let seen = seen.into_inner().unwrap();
    assert_eq!(seen.last().unwrap().phase, ClonePhase::Done);
    assert!(
        seen.iter().any(|p| p.phase == ClonePhase::LfsObjects),
        "the LFS download is its own phase: {seen:?}"
    );
    let mut highest = 0;
    for p in seen.iter().filter(|p| p.phase == ClonePhase::LfsObjects) {
        assert!(p.percent >= highest, "LfsObjects went backwards: {seen:?}");
        highest = p.percent;
    }
}

/// `LfsCloneMode::Skip` leaves the pointers where they are.
pub fn check_clone_with_lfs_skip_keeps_pointers(backend: &dyn GitBackend) {
    crate::require_git_lfs!("check_clone_with_lfs_skip_keeps_pointers");
    let repo = TestRepoBuilder::new()
        .with_lfs_origin()
        .with_lfs()
        .lfs_track("*.bin")
        .commit_lfs_file("big.bin", LFS_PAYLOAD, "add big")
        .build();
    repo.git(&["push", "origin", "main"]);
    let dir = tempfile::tempdir().unwrap();
    let dest = dir.path().join("clone");

    let cloned = block_on(backend.clone(
        &repo.remote_path().unwrap().to_string_lossy(),
        &dest,
        &CloneOptions {
            lfs: gitpoulpi_core::domain::LfsCloneMode::Skip,
            ..CloneOptions::default()
        },
    ))
    .unwrap();

    let on_disk = std::fs::read(dest.join("big.bin")).unwrap();
    assert!(
        on_disk.starts_with(b"version https://git-lfs"),
        "Skip must leave the pointer text"
    );
    let files = backend.lfs_files(&cloned, None).unwrap();
    assert!(files.len() == 1 && !files[0].local, "{files:?}");
}

/// A clone of an LFS repository made where `git-lfs` is missing still
/// succeeds - never a failed clone because of LFS - and leaves pointers.
/// Not a `check_*` (it needs a differently-built backend); each backend
/// crate calls it. The fixture itself needs the program to be built, so it
/// is skipped without it.
pub fn assert_clone_without_lfs_program_succeeds_and_keeps_pointers(backend: &dyn GitBackend) {
    crate::require_git_lfs!("assert_clone_without_lfs_program_succeeds_and_keeps_pointers");
    let repo = TestRepoBuilder::new()
        .with_lfs_origin()
        .with_lfs()
        .lfs_track("*.bin")
        .commit_lfs_file("big.bin", LFS_PAYLOAD, "add big")
        .build();
    repo.git(&["push", "origin", "main"]);
    let dir = tempfile::tempdir().unwrap();
    let dest = dir.path().join("clone");

    let cloned = block_on(backend.clone(
        &repo.remote_path().unwrap().to_string_lossy(),
        &dest,
        &CloneOptions::default(),
    ))
    .expect("a clone never fails because of LFS");

    let on_disk = std::fs::read(dest.join("big.bin")).unwrap();
    assert!(on_disk.starts_with(b"version https://git-lfs"));
    // The reads still work, and say the program is missing.
    let status = backend.lfs_status(&cloned).unwrap();
    assert_eq!(status.binary, gitpoulpi_core::domain::LfsBinary::Missing);
    assert!(status.uses_lfs);
}

// -- Git LFS: locks, against the testkit's loopback server ---------------

/// A repository with one lockable LFS file whose LFS endpoint is
/// `server` (no credential helper, so nothing can prompt).
fn lfs_lock_fixture(server: &crate::lfs::LfsLockServer) -> crate::FixtureRepo {
    TestRepoBuilder::new()
        .with_origin()
        .with_lfs()
        .lfs_track_lockable("*.bin")
        .commit_lfs_file("x.bin", b"a lockable large file", "add x")
        .config("lfs.url", &server.url())
        .config("credential.helper", "")
        .build()
}

/// lock, list (marked ours), unlock.
pub fn check_lfs_lock_list_unlock_round_trip(backend: &dyn GitBackend) {
    crate::require_git_lfs!("check_lfs_lock_list_unlock_round_trip");
    let server = crate::lfs::LfsLockServer::start();
    let repo = lfs_lock_fixture(&server);
    let repo_ref = repo.repo_ref();
    let path = std::path::Path::new("x.bin");

    assert!(block_on(backend.lfs_locks(&repo_ref, "origin"))
        .unwrap()
        .is_empty());

    let lock = block_on(backend.lfs_lock(&repo_ref, "origin", path)).unwrap();
    assert_eq!(lock.path, path);
    assert_eq!(lock.owner.as_deref(), Some(crate::lfs::LOCK_SERVER_USER));
    assert!(lock.ours);
    assert_eq!(server.locked_paths(), ["x.bin"]);

    let listed = block_on(backend.lfs_locks(&repo_ref, "origin")).unwrap();
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].id, lock.id);
    assert!(listed[0].ours);

    block_on(backend.lfs_unlock(&repo_ref, "origin", path, false)).unwrap();
    assert!(server.locked_paths().is_empty());
    assert!(block_on(backend.lfs_locks(&repo_ref, "origin"))
        .unwrap()
        .is_empty());
}

/// Somebody else's lock is listed as not ours, is refused without
/// `force`, and released with it.
pub fn check_lfs_unlock_of_someone_elses_lock_needs_force(backend: &dyn GitBackend) {
    crate::require_git_lfs!("check_lfs_unlock_of_someone_elses_lock_needs_force");
    let server = crate::lfs::LfsLockServer::start();
    server.seed_lock("x.bin", "other");
    let repo = lfs_lock_fixture(&server);
    let repo_ref = repo.repo_ref();
    let path = std::path::Path::new("x.bin");

    let listed = block_on(backend.lfs_locks(&repo_ref, "origin")).unwrap();
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].owner.as_deref(), Some("other"));
    assert!(!listed[0].ours);

    assert!(
        block_on(backend.lfs_unlock(&repo_ref, "origin", path, false)).is_err(),
        "releasing someone else's lock needs force"
    );
    assert_eq!(server.locked_paths(), ["x.bin"], "the lock survived");

    block_on(backend.lfs_unlock(&repo_ref, "origin", path, true)).unwrap();
    assert!(server.locked_paths().is_empty());
}

/// A server that does not implement locking makes `lfs_lock` a typed
/// `LfsLockingUnsupported` naming the remote; listing gives what
/// git-lfs gives, an empty list.
pub fn check_lfs_locking_on_a_server_without_locking_is_typed(backend: &dyn GitBackend) {
    crate::require_git_lfs!("check_lfs_locking_on_a_server_without_locking_is_typed");
    let server = crate::lfs::LfsLockServer::start_without_locking();
    let repo = lfs_lock_fixture(&server);
    let repo_ref = repo.repo_ref();

    let error = block_on(backend.lfs_lock(&repo_ref, "origin", std::path::Path::new("x.bin")))
        .expect_err("locking must be refused");
    assert!(
        matches!(&error, gitpoulpi_core::Error::LfsLockingUnsupported { remote } if remote == "origin"),
        "{error:?}"
    );
    assert!(block_on(backend.lfs_locks(&repo_ref, "origin"))
        .unwrap()
        .is_empty());
}

/// Every LFS operation that needs the `git-lfs` program, called on a
/// backend that reports it missing (`without_lfs()` on either backend):
/// each fails with the typed `LfsUnavailable` and the repository is left
/// exactly as it was. Not a `check_*` (it needs a differently-built
/// backend, and an environment variable would leak into parallel tests);
/// each backend crate calls it from its own test.
pub fn assert_lfs_mutations_without_binary_are_typed_unavailable(backend: &dyn GitBackend) {
    use gitpoulpi_core::domain::{LfsInstallScope, LfsTrackOptions, LfsUnavailableReason};
    use gitpoulpi_core::Error;

    let repo = TestRepoBuilder::new().with_initial().build();
    let repo_ref = repo.repo_ref();
    let patterns = ["*.bin".to_string()];

    let results = [
        backend
            .lfs_install(&repo_ref, LfsInstallScope::Repository)
            .err(),
        backend
            .lfs_track(&repo_ref, &patterns, &LfsTrackOptions::default())
            .err(),
        backend.lfs_untrack(&repo_ref, &patterns).err(),
        backend.lfs_prune(&repo_ref).err(),
        block_on(backend.lfs_fetch(
            &repo_ref,
            "origin",
            &gitpoulpi_core::domain::LfsFetchOptions::default(),
            &|_| {},
        ))
        .err(),
        block_on(backend.lfs_pull(&repo_ref, "origin", &|_| {})).err(),
        block_on(backend.lfs_checkout(&repo_ref, &[], &|_| {})).err(),
        block_on(backend.lfs_locks(&repo_ref, "origin")).err(),
        block_on(backend.lfs_lock(&repo_ref, "origin", std::path::Path::new("x.bin"))).err(),
        block_on(backend.lfs_unlock(&repo_ref, "origin", std::path::Path::new("x.bin"), true))
            .err(),
        block_on(backend.lfs_push(
            &repo_ref,
            "origin",
            &gitpoulpi_core::domain::LfsPushOptions::default(),
            &|_| {},
        ))
        .err(),
    ];
    for (i, error) in results.into_iter().enumerate() {
        assert!(
            matches!(
                error,
                Some(Error::LfsUnavailable {
                    reason: LfsUnavailableReason::Missing
                })
            ),
            "operation #{i} must fail with LfsUnavailable, got {error:?}"
        );
    }
    assert!(
        !repo.root().join(".gitattributes").exists(),
        "a refused operation must leave the repository untouched"
    );
    assert!(!backend.lfs_status(&repo_ref).unwrap().hooks_installed);
    assert_eq!(
        backend.lfs_status(&repo_ref).unwrap().binary,
        gitpoulpi_core::domain::LfsBinary::Missing
    );
}

// -- tags: local, remote, and the state between the two -----------------
//
// git cannot tell a local tag from a remote one (a fetch drops a remote's
// tags into `refs/tags/*`), so GitPoulpi keeps a mirror namespace (see
// `gitpoulpi_core::domain::REMOTE_TAG_MIRROR_PREFIX`); these checks pin
// every way the two backends could disagree about maintaining it.
//
// Every push goes to a **bare repository inside the fixture's temp dir**;
// nothing here touches the network.
