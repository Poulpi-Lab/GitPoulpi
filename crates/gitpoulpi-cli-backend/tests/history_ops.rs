use gitpoulpi_cli_backend::SystemGitBackend;
use gitpoulpi_core::domain::{
    CheckoutTarget, CherryPickOptions, CherryPickOutcome, CommitOptions, MergeOptions,
    MergeOutcome, RebaseOptions, RebaseOutcome, Ref, RefKind, RefVisibility, RevertOutcome,
    StageTarget,
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

fn branch_ref(name: &str, target: gitpoulpi_core::domain::Oid) -> Ref {
    Ref {
        kind: RefKind::LocalBranch {
            name: name.to_string(),
        },
        target,
        visibility: RefVisibility::Shown,
        is_head: false,
        upstream: None,
        // A branch, so it has no tag-sync state by definition.
        tag_sync: None,
        multi_remote: None,
    }
}

#[test]
fn merge_fast_forwards_when_history_is_linear() {
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
    write_and_commit(&backend, &repo_ref, "feature.txt", "f", "feature work");
    let feature_head = backend.log(&repo_ref, &Default::default()).unwrap().nodes[0].id;
    backend
        .checkout(&repo_ref, &CheckoutTarget::Branch("main".into()))
        .unwrap();

    let outcome = backend
        .merge(
            &repo_ref,
            &branch_ref("feature", feature_head),
            &MergeOptions::default(),
        )
        .unwrap();

    assert_eq!(outcome, MergeOutcome::FastForwarded(feature_head));
}

#[test]
fn merge_reports_conflicted_paths_without_erroring() {
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
            &branch_ref("feature", feature_head),
            &MergeOptions::default(),
        )
        .unwrap();

    assert_eq!(
        outcome,
        MergeOutcome::Conflicted(vec!["initial.txt".to_string()])
    );
}

#[test]
fn cherry_pick_single_commit_applies_it_onto_head() {
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
    write_and_commit(&backend, &repo_ref, "feature.txt", "f", "feature commit");
    let feature_head = backend.log(&repo_ref, &Default::default()).unwrap().nodes[0].id;
    backend
        .checkout(&repo_ref, &CheckoutTarget::Branch("main".into()))
        .unwrap();

    let outcome = backend
        .cherry_pick(&repo_ref, &[feature_head], &CherryPickOptions::default())
        .unwrap();

    match outcome {
        CherryPickOutcome::Completed(oids) => assert_eq!(oids.len(), 1),
        other => panic!("expected Completed, got {other:?}"),
    }
    assert!(repo.root().join("feature.txt").exists());
}

#[test]
fn rebase_replays_the_branch_onto_the_new_base_cleanly() {
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
    write_and_commit(&backend, &repo_ref, "feature.txt", "f", "feature commit");
    backend
        .checkout(&repo_ref, &CheckoutTarget::Branch("main".into()))
        .unwrap();
    write_and_commit(&backend, &repo_ref, "main.txt", "m", "main commit");
    let main_head = backend.log(&repo_ref, &Default::default()).unwrap().nodes[0].id;
    backend
        .checkout(&repo_ref, &CheckoutTarget::Branch("feature".into()))
        .unwrap();

    let outcome = backend
        .rebase(
            &repo_ref,
            &branch_ref("main", main_head),
            &RebaseOptions::default(),
        )
        .unwrap();

    assert_eq!(outcome, RebaseOutcome::Completed);
    let log = backend.log(&repo_ref, &Default::default()).unwrap();
    assert_eq!(log.nodes.len(), 3);
}

#[test]
fn revert_undoes_a_commit_with_a_new_commit() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build();
    let backend = SystemGitBackend::new();
    let repo_ref = repo.repo_ref();
    write_and_commit(&backend, &repo_ref, "new.txt", "hello", "add new.txt");
    let to_revert = backend.log(&repo_ref, &Default::default()).unwrap().nodes[0].id;

    let outcome = backend.revert(&repo_ref, to_revert, None).unwrap();

    match outcome {
        RevertOutcome::Completed(_) => {}
        other => panic!("expected Completed, got {other:?}"),
    }
    assert!(!repo.root().join("new.txt").exists());
}
