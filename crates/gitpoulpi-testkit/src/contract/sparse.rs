use super::block_on;
use crate::TestRepoBuilder;
use gitpoulpi_core::domain::{CloneOptions, LogOptions, SparseState};
use gitpoulpi_core::GitBackend;

/// A source with files at the root and under `src`, `docs/guide` and `lib`.
fn layered_source() -> crate::FixtureRepo {
    TestRepoBuilder::new()
        .with_initial()
        .commit_file("README.md", "root", "root file")
        .commit_file("src/main.rs", "fn main() {}", "add src")
        .commit_file("docs/guide/intro.md", "intro", "add docs")
        .commit_file("lib/util.rs", "// util", "add lib")
        .build()
}

fn dirs(v: &[&str]) -> Vec<String> {
    v.iter().map(|s| (*s).to_string()).collect()
}

fn sparse_clone(
    backend: &dyn GitBackend,
    source: &crate::FixtureRepo,
    chosen: &[&str],
) -> (tempfile::TempDir, gitpoulpi_core::domain::RepoRef) {
    let dest_dir = tempfile::tempdir().unwrap();
    let dest = dest_dir.path().join("clone");
    let opts = CloneOptions {
        sparse: Some(dirs(chosen)),
        ..CloneOptions::default()
    };
    let repo = block_on(backend.clone(&source.root().to_string_lossy(), &dest, &opts)).unwrap();
    (dest_dir, repo)
}

/// A sparse clone checks out the chosen directories and the root files only,
/// yet keeps the whole history, and reports itself as sparse.
pub fn check_sparse_clone_checks_out_only_the_chosen_directories(backend: &dyn GitBackend) {
    let source = layered_source();
    let (_keep, repo) = sparse_clone(backend, &source, &["src"]);

    assert!(repo.root.join("README.md").is_file(), "root files stay");
    assert!(repo.root.join("src/main.rs").is_file());
    assert!(!repo.root.join("docs").exists(), "docs must be absent");
    assert!(!repo.root.join("lib").exists(), "lib must be absent");

    // The history is complete: the commits that touched absent files exist.
    let log = backend.log(&repo, &LogOptions::default()).unwrap();
    let summaries: Vec<&str> = log.nodes.iter().map(|n| n.summary.as_str()).collect();
    assert!(
        summaries.contains(&"add docs") && summaries.contains(&"add lib"),
        "full history expected, got {summaries:?}"
    );

    assert_eq!(
        backend.sparse_checkout_status(&repo).unwrap(),
        SparseState {
            enabled: true,
            cone: true,
            dirs: dirs(&["src"]),
        }
    );
}

/// `set` replaces the directory set: new ones appear, left-out ones go.
pub fn check_sparse_set_adds_and_removes_directories(backend: &dyn GitBackend) {
    let source = layered_source();
    let (_keep, repo) = sparse_clone(backend, &source, &["src"]);

    backend
        .sparse_checkout_set(&repo, &dirs(&["src", "docs/guide"]))
        .unwrap();
    assert!(repo.root.join("docs/guide/intro.md").is_file());
    assert!(repo.root.join("src/main.rs").is_file());
    assert!(!repo.root.join("lib").exists());
    let mut listed = backend.sparse_checkout_status(&repo).unwrap().dirs;
    listed.sort();
    assert_eq!(listed, dirs(&["docs/guide", "src"]));

    backend
        .sparse_checkout_set(&repo, &dirs(&["lib/"]))
        .unwrap();
    assert!(repo.root.join("lib/util.rs").is_file());
    assert!(!repo.root.join("src").exists(), "src left the set");
    assert!(!repo.root.join("docs").exists(), "docs left the set");
    assert_eq!(
        backend.sparse_checkout_status(&repo).unwrap().dirs,
        dirs(&["lib"])
    );

    // An empty or malformed set is refused and changes nothing.
    assert!(backend.sparse_checkout_set(&repo, &[]).is_err());
    assert!(backend.sparse_checkout_set(&repo, &dirs(&[".."])).is_err());
    assert!(repo.root.join("lib/util.rs").is_file());
}

/// `set` on a repository that was cloned in full turns sparse checkout on.
pub fn check_sparse_set_enables_it_on_a_full_checkout(backend: &dyn GitBackend) {
    let source = layered_source();
    let repo = source.repo_ref();
    assert_eq!(
        backend.sparse_checkout_status(&repo).unwrap(),
        SparseState::default()
    );
    backend
        .sparse_checkout_set(&repo, &dirs(&["docs"]))
        .unwrap();
    assert!(!source.root().join("src").exists());
    assert!(source.root().join("docs/guide/intro.md").is_file());
    assert!(backend.sparse_checkout_status(&repo).unwrap().enabled);
}

/// `disable` brings every file back, and is harmless when not sparse.
pub fn check_sparse_disable_restores_everything(backend: &dyn GitBackend) {
    let source = layered_source();
    let (_keep, repo) = sparse_clone(backend, &source, &["lib"]);
    assert!(!repo.root.join("src").exists());

    backend.sparse_checkout_disable(&repo).unwrap();
    for file in [
        "README.md",
        "src/main.rs",
        "docs/guide/intro.md",
        "lib/util.rs",
    ] {
        assert!(repo.root.join(file).is_file(), "{file} must be back");
    }
    assert_eq!(
        backend.sparse_checkout_status(&repo).unwrap(),
        SparseState::default()
    );
    // Not an error twice, and not an error on a repo that never was sparse.
    backend.sparse_checkout_disable(&repo).unwrap();
    backend.sparse_checkout_disable(&source.repo_ref()).unwrap();
}

/// A repository that never used sparse checkout is simply "disabled".
pub fn check_sparse_status_of_a_plain_repo_is_disabled(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new().with_initial().build();
    assert_eq!(
        backend.sparse_checkout_status(&repo.repo_ref()).unwrap(),
        SparseState::default()
    );
}

/// A sparse clone with no directory is refused up front, leaving no clone.
pub fn check_sparse_clone_without_directories_is_refused(backend: &dyn GitBackend) {
    let source = layered_source();
    let dest_dir = tempfile::tempdir().unwrap();
    let dest = dest_dir.path().join("clone");
    let opts = CloneOptions {
        sparse: Some(Vec::new()),
        ..CloneOptions::default()
    };
    assert!(block_on(backend.clone(&source.root().to_string_lossy(), &dest, &opts)).is_err());
    assert!(!dest.exists(), "a refused clone must leave nothing behind");
}
