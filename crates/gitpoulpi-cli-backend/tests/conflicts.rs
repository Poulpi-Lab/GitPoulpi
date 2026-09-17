use gitpoulpi_cli_backend::SystemGitBackend;
use gitpoulpi_core::domain::{
    CheckoutTarget, CommitOptions, ConflictResolution, ConflictSide, MergeOptions, StageTarget,
};
use gitpoulpi_core::GitBackend;
use gitpoulpi_testkit::FixtureRepoBuilder;

fn write_and_commit(
    backend: &SystemGitBackend,
    repo_ref: &gitpoulpi_core::domain::RepoRef,
    file: &str,
    content: &str,
    message: &str,
) {
    std::fs::write(repo_ref.root.join(file), content).unwrap();
    backend
        .stage(repo_ref, &StageTarget::File(file.into()))
        .unwrap();
    backend
        .commit(repo_ref, message, &CommitOptions::default())
        .unwrap();
}

fn make_conflicted_repo() -> (gitpoulpi_testkit::FixtureRepo, SystemGitBackend) {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build();
    let backend = SystemGitBackend::new();
    let repo_ref = repo.repo_ref();
    let base = backend.log(&repo_ref, &Default::default()).unwrap().nodes[0].id;
    backend.branch_create(&repo_ref, "feature", base).unwrap();
    backend
        .checkout(&repo_ref, &CheckoutTarget::Branch("feature".into()))
        .unwrap();
    write_and_commit(
        &backend,
        &repo_ref,
        "initial.txt",
        "from feature",
        "feature edit",
    );
    let feature_head = backend.log(&repo_ref, &Default::default()).unwrap().nodes[0].id;
    backend
        .checkout(&repo_ref, &CheckoutTarget::Branch("main".into()))
        .unwrap();
    write_and_commit(&backend, &repo_ref, "initial.txt", "from main", "main edit");

    let outcome = backend
        .merge(
            &repo_ref,
            &gitpoulpi_core::domain::Ref {
                kind: gitpoulpi_core::domain::RefKind::LocalBranch {
                    name: "feature".into(),
                },
                target: feature_head,
                visibility: gitpoulpi_core::domain::RefVisibility::Shown,
                is_head: false,
                upstream: None,
                tag_sync: None,
            },
            &MergeOptions::default(),
        )
        .unwrap();
    assert!(matches!(
        outcome,
        gitpoulpi_core::domain::MergeOutcome::Conflicted(_)
    ));

    (repo, backend)
}

#[test]
fn conflicts_lists_the_conflicted_paths_with_all_three_sides() {
    let (repo, backend) = make_conflicted_repo();
    let repo_ref = repo.repo_ref();

    let sets = backend.conflicts(&repo_ref).unwrap();

    assert_eq!(sets.len(), 1);
    assert_eq!(sets[0].path.to_str(), Some("initial.txt"));
    assert_eq!(sets[0].ours.as_deref(), Some("from main"));
    assert_eq!(sets[0].theirs.as_deref(), Some("from feature"));
}

#[test]
fn resolve_conflict_take_theirs_stages_the_incoming_content() {
    let (repo, backend) = make_conflicted_repo();
    let repo_ref = repo.repo_ref();

    backend
        .resolve_conflict(
            &repo_ref,
            std::path::Path::new("initial.txt"),
            ConflictResolution::TakeSide(ConflictSide::Theirs),
            &gitpoulpi_core::domain::ResolveOptions::default(),
        )
        .unwrap();

    assert!(backend.conflicts(&repo_ref).unwrap().is_empty());
    assert_eq!(
        std::fs::read_to_string(repo.root().join("initial.txt")).unwrap(),
        "from feature"
    );
}

#[test]
fn resolve_conflict_use_content_stages_the_custom_resolution() {
    let (repo, backend) = make_conflicted_repo();
    let repo_ref = repo.repo_ref();

    backend
        .resolve_conflict(
            &repo_ref,
            std::path::Path::new("initial.txt"),
            ConflictResolution::UseContent("merged by hand".into()),
            &gitpoulpi_core::domain::ResolveOptions::default(),
        )
        .unwrap();

    assert!(backend.conflicts(&repo_ref).unwrap().is_empty());
    assert_eq!(
        std::fs::read_to_string(repo.root().join("initial.txt")).unwrap(),
        "merged by hand"
    );
}
