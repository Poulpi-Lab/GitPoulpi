use gitpoulpi_cli_backend::SystemGitBackend;
use gitpoulpi_core::domain::{FileStatus, LogOptions, RefKind};
use gitpoulpi_core::GitBackend;
use gitpoulpi_testkit::FixtureRepoBuilder;

#[test]
fn log_lists_commits_newest_first_with_correct_parents() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("first")
        .unwrap()
        .commit("second")
        .unwrap()
        .build();
    let backend = SystemGitBackend::new();

    let graph = backend
        .log(&repo.repo_ref(), &LogOptions::default())
        .unwrap();

    assert_eq!(graph.nodes.len(), 2);
    assert_eq!(graph.nodes[0].summary, "second");
    assert_eq!(graph.nodes[1].summary, "first");
    assert_eq!(graph.nodes[0].parents, vec![graph.nodes[1].id]);
    assert!(graph.nodes[1].parents.is_empty());
}

#[test]
fn log_on_empty_repo_returns_no_commits() {
    let repo = FixtureRepoBuilder::new().unwrap().build();
    let backend = SystemGitBackend::new();

    let graph = backend
        .log(&repo.repo_ref(), &LogOptions::default())
        .unwrap();

    assert!(graph.nodes.is_empty());
}

#[test]
fn status_reports_untracked_and_modified_files() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build();
    let backend = SystemGitBackend::new();

    std::fs::write(repo.root().join("untracked.txt"), "new").unwrap();
    std::fs::write(repo.root().join("initial.txt"), "changed").unwrap();

    let status = backend.status(&repo.repo_ref()).unwrap();

    let untracked = status
        .files
        .iter()
        .find(|f| f.path.to_str() == Some("untracked.txt"))
        .unwrap();
    assert_eq!(untracked.worktree, Some(FileStatus::Untracked));
    assert_eq!(untracked.index, None);

    let modified = status
        .files
        .iter()
        .find(|f| f.path.to_str() == Some("initial.txt"))
        .unwrap();
    assert_eq!(modified.worktree, Some(FileStatus::Modified));
    assert_eq!(modified.index, None);
}

#[test]
fn refs_lists_local_branches() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .branch("feature")
        .unwrap()
        .build();
    let backend = SystemGitBackend::new();

    let refs = backend.refs(&repo.repo_ref()).unwrap();

    let names: Vec<&str> = refs
        .iter()
        .filter_map(|r| match &r.kind {
            RefKind::LocalBranch { name } => Some(name.as_str()),
            _ => None,
        })
        .collect();
    assert!(names.contains(&"main"));
    assert!(names.contains(&"feature"));
}

#[test]
fn stashes_lists_pushed_stash() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build();
    std::fs::write(repo.root().join("initial.txt"), "dirty").unwrap();
    std::process::Command::new("git")
        .args(["stash", "push"])
        .current_dir(repo.root())
        .output()
        .unwrap();

    let backend = SystemGitBackend::new();
    let stashes = backend.stashes(&repo.repo_ref()).unwrap();

    assert_eq!(stashes.len(), 1);
    assert_eq!(stashes[0].branch_at_creation, "main");
}

#[test]
fn remotes_lists_configured_remote() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build();
    std::process::Command::new("git")
        .args([
            "remote",
            "add",
            "origin",
            "https://example.invalid/repo.git",
        ])
        .current_dir(repo.root())
        .output()
        .unwrap();

    let backend = SystemGitBackend::new();
    let remotes = backend.remotes(&repo.repo_ref()).unwrap();

    assert_eq!(remotes.len(), 1);
    assert_eq!(remotes[0].name, "origin");
    assert_eq!(remotes[0].fetch_url, "https://example.invalid/repo.git");
}
