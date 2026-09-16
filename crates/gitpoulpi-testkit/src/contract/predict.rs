use crate::TestRepoBuilder;
use gitpoulpi_core::domain::RepoRef;
use gitpoulpi_core::{Error, GitBackend};

/// The system backend answers `Unsupported` before git 2.38 (the product
/// says so instead of guessing); that is the contract there, so the checks
/// end early on it.
fn unsupported_here(backend: &dyn GitBackend, r: &RepoRef) -> bool {
    let head = backend.resolve(r, "HEAD").unwrap();
    let unsupported = matches!(
        backend.predict_merge_conflicts(r, head, head),
        Err(Error::Unsupported { .. })
    );
    assert!(
        !unsupported || !crate::system_git_at_least(2, 38),
        "unsupported on a git that has merge-tree --write-tree"
    );
    unsupported
}

fn write(repo: &crate::FixtureRepo, path: &str, text: &str) {
    std::fs::write(repo.root().join(path), text).unwrap();
}

/// `predict_merge_conflicts` names exactly the files a merge would conflict
/// on, in both directions, with renames followed, and leaves the repository
/// alone (HEAD, refs, index and work tree). Both backends must report the
/// same set: a content conflict, an add/add and a modify/delete conflict, with
/// a rename on one side and a modification of the same file on the other
/// merging cleanly.
pub fn check_predict_merge_conflicts_names_the_files_a_merge_would_conflict_on(
    backend: &dyn GitBackend,
) {
    let repo = TestRepoBuilder::new().build();
    let long = (1..=12)
        .map(|n| format!("line number {n} of the file\n"))
        .collect::<String>();
    write(&repo, "a.txt", "one\ntwo\nthree\n");
    write(&repo, "r.txt", &long);
    write(&repo, "d.txt", "doomed\n");
    write(&repo, "calm.txt", "calm\n");
    repo.git(&["add", "-A"]);
    repo.git(&["commit", "-q", "-m", "base"]);
    let main = repo
        .git(&["rev-parse", "--abbrev-ref", "HEAD"])
        .trim()
        .to_string();

    repo.git(&["checkout", "-q", "-b", "feature"]);
    write(&repo, "a.txt", "one\nfeature\nthree\n");
    write(&repo, "both.txt", "from feature\n");
    write(&repo, "r.txt", &long.replace("number 6", "NUMBER 6"));
    repo.git(&["rm", "-q", "d.txt"]);
    write(&repo, "calm.txt", "calm\nfeature adds\n");
    repo.git(&["add", "-A"]);
    repo.git(&["commit", "-q", "-m", "feature"]);

    repo.git(&["checkout", "-q", &main]);
    write(&repo, "a.txt", "one\nmain\nthree\n");
    write(&repo, "both.txt", "from main\n");
    repo.git(&["mv", "r.txt", "renamed.txt"]);
    write(&repo, "d.txt", "doomed but edited\n");
    repo.git(&["add", "-A"]);
    repo.git(&["commit", "-q", "-m", "main"]);

    let r = repo.repo_ref();
    let head = backend.resolve(&r, "HEAD").unwrap();
    let feature = backend.resolve(&r, "feature").unwrap();
    if unsupported_here(backend, &r) {
        return;
    }
    let before = repo.git(&["for-each-ref"]);
    let status = repo.git(&["status", "--porcelain"]);
    let expected = vec!["a.txt".to_string(), "both.txt".into(), "d.txt".into()];

    assert_eq!(
        backend.predict_merge_conflicts(&r, head, feature).unwrap(),
        expected,
        "main <- feature"
    );
    assert_eq!(
        backend.predict_merge_conflicts(&r, feature, head).unwrap(),
        expected,
        "feature <- main names the same files"
    );
    // Nothing moved or changed on disk.
    assert_eq!(backend.resolve(&r, "HEAD").unwrap(), head);
    assert_eq!(repo.git(&["for-each-ref"]), before);
    assert_eq!(repo.git(&["status", "--porcelain"]), status);
    assert!(!repo.root().join(".git").join("MERGE_HEAD").exists());
}

/// A merge that goes through, an ancestor, and a commit with itself all
/// predict no conflict.
pub fn check_predict_merge_conflicts_is_empty_when_the_merge_goes_through(
    backend: &dyn GitBackend,
) {
    let repo = TestRepoBuilder::new()
        .with_initial()
        .branch("feature")
        .commit_file("f.txt", "f", "feature")
        .checkout("main")
        .commit_file("m.txt", "m", "main")
        .build();
    let r = repo.repo_ref();
    if unsupported_here(backend, &r) {
        return;
    }
    let main = backend.resolve(&r, "main").unwrap();
    let feature = backend.resolve(&r, "feature").unwrap();
    let base = backend.resolve(&r, "main~1").unwrap();
    for (a, b) in [(main, feature), (main, base), (base, main), (main, main)] {
        assert!(
            backend
                .predict_merge_conflicts(&r, a, b)
                .unwrap()
                .is_empty(),
            "{a} <- {b}"
        );
    }
}
