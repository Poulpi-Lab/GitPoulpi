use super::head_oid;
use crate::TestRepoBuilder;
use gitpoulpi_core::GitBackend;

pub fn check_stash_push_pop(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new().with_initial().build();
    let repo_ref = repo.repo_ref();
    // Modify the fixture's own tracked file (`FixtureRepoBuilder::commit`
    // names it after the commit message) rather than creating a new
    // untracked one -- `include_untracked: false` below wouldn't stash an
    // untracked file, which isn't what this check is about.
    std::fs::write(repo.root().join("initial.txt"), "dirty").unwrap();
    let head = head_oid(backend, &repo_ref);

    let id = backend
        .stash_push(&repo_ref, Some("wip"), false)
        .unwrap()
        .expect("a stash");
    let status = backend.status(&repo_ref).unwrap();
    assert!(
        status.files.is_empty(),
        "stash_push should leave a clean working tree"
    );
    let stashes = backend.stashes(&repo_ref).unwrap();
    assert!(stashes.iter().any(|s| s.id == id));
    // The commit graph draws each stash beside the commit it was made on,
    // so a listed stash names its own commit and that base (the stash
    // commit's first parent) -- read in the same call, never one more
    // `git` spawn per stash.
    let listed = stashes.iter().find(|s| s.id == id).unwrap();
    assert_eq!(listed.base, Some(head), "a stash's base is HEAD at push");
    assert_eq!(
        listed.commit,
        Some(backend.resolve(&repo_ref, "stash@{0}").unwrap()),
        "a stash names its own commit"
    );

    backend.stash_pop(&repo_ref, id).unwrap();
    let status = backend.status(&repo_ref).unwrap();
    assert!(
        !status.files.is_empty(),
        "stash_pop should restore the dirty file"
    );
    let stashes = backend.stashes(&repo_ref).unwrap();
    assert!(!stashes.iter().any(|s| s.id == id));
}

pub fn check_stash_push_paths_stashes_only_those_paths(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new().with_initial().build();
    let repo_ref = repo.repo_ref();
    std::fs::write(repo.root().join("initial.txt"), "dirty").unwrap();
    std::fs::write(repo.root().join("other.txt"), "untracked").unwrap();

    // Only the tracked change is stashed; the untracked file stays.
    let id = backend
        .stash_push_paths(&repo_ref, Some("one file"), &["initial.txt".to_string()])
        .unwrap()
        .expect("a stash");
    let status = backend.status(&repo_ref).unwrap();
    let left: Vec<_> = status.files.iter().map(|f| f.path.clone()).collect();
    assert_eq!(left, vec![std::path::PathBuf::from("other.txt")]);
    assert!(backend
        .stashes(&repo_ref)
        .unwrap()
        .iter()
        .any(|s| s.id == id));

    // An untracked path can be stashed by name too.
    backend
        .stash_push_paths(&repo_ref, None, &["other.txt".to_string()])
        .unwrap();
    assert!(backend.status(&repo_ref).unwrap().files.is_empty());
    assert!(!repo.root().join("other.txt").exists());
}

/// Nothing to stash: no stash, no error, and `None` says so.
pub fn check_stash_push_with_nothing_to_stash_is_none(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new().with_initial().build();
    let repo_ref = repo.repo_ref();
    assert_eq!(backend.stash_push(&repo_ref, None, true).unwrap(), None);
    assert_eq!(
        backend
            .stash_push_paths(&repo_ref, None, &["initial.txt".to_string()])
            .unwrap(),
        None
    );
    // An untracked file is not stashed unless asked for.
    std::fs::write(repo.root().join("new.txt"), "x").unwrap();
    assert_eq!(backend.stash_push(&repo_ref, None, false).unwrap(), None);
    assert!(backend.stashes(&repo_ref).unwrap().is_empty());
}

pub fn check_stash_apply_keeps_the_stash(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new().with_initial().build();
    let repo_ref = repo.repo_ref();
    std::fs::write(repo.root().join("initial.txt"), "dirty").unwrap();

    let id = backend
        .stash_push(&repo_ref, Some("wip"), false)
        .unwrap()
        .expect("a stash");
    backend.stash_apply(&repo_ref, id).unwrap();
    let status = backend.status(&repo_ref).unwrap();
    assert!(
        !status.files.is_empty(),
        "stash_apply should restore the dirty file"
    );
    // Unlike pop, apply leaves the stash entry in place.
    let stashes = backend.stashes(&repo_ref).unwrap();
    assert!(stashes.iter().any(|s| s.id == id));

    backend.stash_drop(&repo_ref, id).unwrap();
    let stashes = backend.stashes(&repo_ref).unwrap();
    assert!(!stashes.iter().any(|s| s.id == id));
}
