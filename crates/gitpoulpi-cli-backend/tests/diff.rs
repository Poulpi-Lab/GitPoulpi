use gitpoulpi_cli_backend::SystemGitBackend;
use gitpoulpi_core::domain::{DiffTarget, FileStatus, LineOrigin};
use gitpoulpi_core::GitBackend;
use gitpoulpi_testkit::FixtureRepoBuilder;

#[test]
fn working_tree_vs_index_shows_unstaged_edit() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build();
    std::fs::write(repo.root().join("initial.txt"), "changed content\n").unwrap();
    let backend = SystemGitBackend::new();

    let diffs = backend
        .diff(&repo.repo_ref(), DiffTarget::WorkingTreeVsIndex)
        .unwrap();

    assert_eq!(diffs.len(), 1);
    let file = &diffs[0];
    assert_eq!(file.path.to_str(), Some("initial.txt"));
    assert_eq!(file.status, FileStatus::Modified);
    assert_eq!(file.hunks.len(), 1);
    let hunk = &file.hunks[0];
    assert!(hunk
        .lines
        .iter()
        .any(|l| l.origin == LineOrigin::Delete && l.content == "initial"));
    assert!(hunk
        .lines
        .iter()
        .any(|l| l.origin == LineOrigin::Add && l.content == "changed content"));
}

#[test]
fn index_vs_head_shows_staged_new_file() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build();
    std::fs::write(repo.root().join("new.txt"), "hello\n").unwrap();
    std::process::Command::new("git")
        .args(["add", "new.txt"])
        .current_dir(repo.root())
        .output()
        .unwrap();
    let backend = SystemGitBackend::new();

    let diffs = backend
        .diff(&repo.repo_ref(), DiffTarget::IndexVsHead)
        .unwrap();

    assert_eq!(diffs.len(), 1);
    assert_eq!(diffs[0].path.to_str(), Some("new.txt"));
    assert_eq!(diffs[0].status, FileStatus::Added);
}

#[test]
fn commit_vs_parent_diffs_a_single_commit() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .commit("second")
        .unwrap()
        .build();
    let backend = SystemGitBackend::new();
    let repo_ref = repo.repo_ref();
    let graph = backend
        .log(&repo_ref, &gitpoulpi_core::domain::LogOptions::default())
        .unwrap();
    let second_commit = graph.nodes[0].id;

    let diffs = backend
        .diff(&repo_ref, DiffTarget::CommitVsParent(second_commit))
        .unwrap();

    assert_eq!(diffs.len(), 1);
    assert_eq!(diffs[0].path.to_str(), Some("second.txt"));
    assert_eq!(diffs[0].status, FileStatus::Added);
}

#[test]
fn commit_vs_parent_on_initial_commit_diffs_against_empty_tree() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build();
    let backend = SystemGitBackend::new();
    let repo_ref = repo.repo_ref();
    let graph = backend
        .log(&repo_ref, &gitpoulpi_core::domain::LogOptions::default())
        .unwrap();
    let initial_commit = graph.nodes[0].id;

    let diffs = backend
        .diff(&repo_ref, DiffTarget::CommitVsParent(initial_commit))
        .unwrap();

    assert_eq!(diffs.len(), 1);
    assert_eq!(diffs[0].path.to_str(), Some("initial.txt"));
    assert_eq!(diffs[0].status, FileStatus::Added);
}
