use gitpoulpi_cli_backend::SystemGitBackend;
use gitpoulpi_core::domain::{DiffTarget, LineSelector, StageTarget};
use gitpoulpi_core::GitBackend;
use gitpoulpi_testkit::FixtureRepoBuilder;

fn index_diff_paths(
    backend: &SystemGitBackend,
    repo: &gitpoulpi_core::domain::RepoRef,
) -> Vec<String> {
    backend
        .diff(repo, DiffTarget::IndexVsHead)
        .unwrap()
        .into_iter()
        .map(|f| f.path.to_string_lossy().to_string())
        .collect()
}

#[test]
fn stage_file_moves_change_into_the_index() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build();
    std::fs::write(repo.root().join("initial.txt"), "changed\n").unwrap();
    let backend = SystemGitBackend::new();
    let repo_ref = repo.repo_ref();

    backend
        .stage(&repo_ref, &StageTarget::File("initial.txt".into()))
        .unwrap();

    assert_eq!(index_diff_paths(&backend, &repo_ref), vec!["initial.txt"]);
    let status = backend.status(&repo_ref).unwrap();
    assert!(status.files.iter().all(|f| f.worktree.is_none()));
}

#[test]
fn unstage_file_moves_change_back_to_worktree() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build();
    std::fs::write(repo.root().join("initial.txt"), "changed\n").unwrap();
    let backend = SystemGitBackend::new();
    let repo_ref = repo.repo_ref();
    backend
        .stage(&repo_ref, &StageTarget::File("initial.txt".into()))
        .unwrap();

    backend
        .unstage(&repo_ref, &StageTarget::File("initial.txt".into()))
        .unwrap();

    assert!(index_diff_paths(&backend, &repo_ref).is_empty());
    let status = backend.status(&repo_ref).unwrap();
    assert_eq!(
        status.files[0].worktree,
        Some(gitpoulpi_core::domain::FileStatus::Modified)
    );
}

#[test]
fn stage_folder_stages_every_file_under_it() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build();
    std::fs::create_dir_all(repo.root().join("sub")).unwrap();
    std::fs::write(repo.root().join("sub/a.txt"), "a\n").unwrap();
    std::fs::write(repo.root().join("sub/b.txt"), "b\n").unwrap();
    let backend = SystemGitBackend::new();
    let repo_ref = repo.repo_ref();

    backend
        .stage(&repo_ref, &StageTarget::Folder("sub".into()))
        .unwrap();

    let mut paths = index_diff_paths(&backend, &repo_ref);
    paths.sort();
    assert_eq!(paths, vec!["sub/a.txt", "sub/b.txt"]);
}

#[test]
fn stage_all_stages_every_change() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build();
    std::fs::write(repo.root().join("initial.txt"), "changed\n").unwrap();
    std::fs::write(repo.root().join("new.txt"), "new\n").unwrap();
    let backend = SystemGitBackend::new();
    let repo_ref = repo.repo_ref();

    backend.stage(&repo_ref, &StageTarget::All).unwrap();

    let mut paths = index_diff_paths(&backend, &repo_ref);
    paths.sort();
    assert_eq!(paths, vec!["initial.txt", "new.txt"]);
}

#[test]
fn unstage_all_unstages_every_change() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build();
    std::fs::write(repo.root().join("initial.txt"), "changed\n").unwrap();
    let backend = SystemGitBackend::new();
    let repo_ref = repo.repo_ref();
    backend.stage(&repo_ref, &StageTarget::All).unwrap();

    backend.unstage(&repo_ref, &StageTarget::All).unwrap();

    assert!(index_diff_paths(&backend, &repo_ref).is_empty());
}

#[test]
fn stage_hunk_stages_only_that_hunk() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build();
    // 20 lines so two edits far apart land in separate hunks.
    let original: String = (1..=20).map(|n| format!("line{n}\n")).collect();
    std::fs::write(repo.root().join("multi.txt"), &original).unwrap();
    std::process::Command::new("git")
        .args(["add", "multi.txt"])
        .current_dir(repo.root())
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args(["commit", "-m", "multi"])
        .current_dir(repo.root())
        .output()
        .unwrap();

    let mut lines: Vec<String> = original.lines().map(str::to_string).collect();
    lines[1] = "line2-changed".to_string();
    lines[18] = "line19-changed".to_string();
    let edited = lines.join("\n") + "\n";
    std::fs::write(repo.root().join("multi.txt"), &edited).unwrap();

    let backend = SystemGitBackend::new();
    let repo_ref = repo.repo_ref();
    let diffs = backend
        .diff(&repo_ref, DiffTarget::WorkingTreeVsIndex)
        .unwrap();
    let file = diffs
        .iter()
        .find(|f| f.path.to_str() == Some("multi.txt"))
        .unwrap();
    assert_eq!(
        file.hunks.len(),
        2,
        "expected two separate hunks for edits 17 lines apart"
    );
    let first_hunk_id = file.hunks[0].id;

    backend
        .stage(
            &repo_ref,
            &StageTarget::Hunk("multi.txt".into(), first_hunk_id),
        )
        .unwrap();

    let staged = backend.diff(&repo_ref, DiffTarget::IndexVsHead).unwrap();
    let staged_file = staged
        .iter()
        .find(|f| f.path.to_str() == Some("multi.txt"))
        .unwrap();
    assert_eq!(staged_file.hunks.len(), 1);

    let remaining_worktree = backend
        .diff(&repo_ref, DiffTarget::WorkingTreeVsIndex)
        .unwrap();
    let remaining_file = remaining_worktree
        .iter()
        .find(|f| f.path.to_str() == Some("multi.txt"))
        .unwrap();
    assert_eq!(remaining_file.hunks.len(), 1);
}

#[test]
fn stage_and_unstage_single_line_out_of_a_hunk() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build();
    let original = "a\nb\nc\n";
    std::fs::write(repo.root().join("f.txt"), original).unwrap();
    std::process::Command::new("git")
        .args(["add", "f.txt"])
        .current_dir(repo.root())
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args(["commit", "-m", "f"])
        .current_dir(repo.root())
        .output()
        .unwrap();

    // Two separate added lines within the same hunk.
    std::fs::write(repo.root().join("f.txt"), "a\nnew1\nb\nnew2\nc\n").unwrap();

    let backend = SystemGitBackend::new();
    let repo_ref = repo.repo_ref();
    let diffs = backend
        .diff(&repo_ref, DiffTarget::WorkingTreeVsIndex)
        .unwrap();
    let file = diffs
        .iter()
        .find(|f| f.path.to_str() == Some("f.txt"))
        .unwrap();
    assert_eq!(file.hunks.len(), 1);
    let hunk = &file.hunks[0];
    let first_add_index = hunk.lines.iter().position(|l| l.content == "new1").unwrap() as u32;

    backend
        .stage(
            &repo_ref,
            &StageTarget::Line(
                "f.txt".into(),
                LineSelector {
                    hunk: hunk.id,
                    line_indices: vec![first_add_index],
                },
            ),
        )
        .unwrap();

    let staged_content = std::process::Command::new("git")
        .args(["show", ":f.txt"])
        .current_dir(repo.root())
        .output()
        .unwrap()
        .stdout;
    let staged_content = String::from_utf8(staged_content).unwrap();
    assert_eq!(staged_content, "a\nnew1\nb\nc\n");

    // The other addition should still be unstaged.
    let worktree_diff = backend
        .diff(&repo_ref, DiffTarget::WorkingTreeVsIndex)
        .unwrap();
    let remaining = worktree_diff
        .iter()
        .find(|f| f.path.to_str() == Some("f.txt"))
        .unwrap();
    use gitpoulpi_core::domain::LineOrigin;
    assert!(remaining.hunks[0]
        .lines
        .iter()
        .any(|l| l.origin == LineOrigin::Add && l.content == "new2"));
    assert!(!remaining.hunks[0]
        .lines
        .iter()
        .any(|l| l.origin == LineOrigin::Add && l.content == "new1"));
}

#[test]
fn unstage_hunk_moves_only_that_hunk_back_to_worktree() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build();
    let original: String = (1..=20).map(|n| format!("line{n}\n")).collect();
    std::fs::write(repo.root().join("multi.txt"), &original).unwrap();
    std::process::Command::new("git")
        .args(["add", "multi.txt"])
        .current_dir(repo.root())
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args(["commit", "-m", "multi"])
        .current_dir(repo.root())
        .output()
        .unwrap();

    let mut lines: Vec<String> = original.lines().map(str::to_string).collect();
    lines[1] = "line2-changed".to_string();
    lines[18] = "line19-changed".to_string();
    let edited = lines.join("\n") + "\n";
    std::fs::write(repo.root().join("multi.txt"), &edited).unwrap();

    let backend = SystemGitBackend::new();
    let repo_ref = repo.repo_ref();
    backend.stage(&repo_ref, &StageTarget::All).unwrap();
    let staged = backend.diff(&repo_ref, DiffTarget::IndexVsHead).unwrap();
    let staged_file = staged
        .iter()
        .find(|f| f.path.to_str() == Some("multi.txt"))
        .unwrap();
    assert_eq!(staged_file.hunks.len(), 2);
    let first_hunk_id = staged_file.hunks[0].id;

    backend
        .unstage(
            &repo_ref,
            &StageTarget::Hunk("multi.txt".into(), first_hunk_id),
        )
        .unwrap();

    let still_staged = backend.diff(&repo_ref, DiffTarget::IndexVsHead).unwrap();
    let still_staged_file = still_staged
        .iter()
        .find(|f| f.path.to_str() == Some("multi.txt"))
        .unwrap();
    assert_eq!(still_staged_file.hunks.len(), 1);

    let back_in_worktree = backend
        .diff(&repo_ref, DiffTarget::WorkingTreeVsIndex)
        .unwrap();
    let worktree_file = back_in_worktree
        .iter()
        .find(|f| f.path.to_str() == Some("multi.txt"))
        .unwrap();
    assert_eq!(worktree_file.hunks.len(), 1);
}
