use gitpoulpi_cli_backend::SystemGitBackend;
use gitpoulpi_core::domain::{CheckoutTarget, CommitOptions, ResetMode, StageTarget};
use gitpoulpi_core::GitBackend;
use gitpoulpi_testkit::FixtureRepoBuilder;

#[test]
fn commit_creates_a_new_head_with_the_staged_change() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build();
    std::fs::write(repo.root().join("new.txt"), "hello\n").unwrap();
    let backend = SystemGitBackend::new();
    let repo_ref = repo.repo_ref();
    backend
        .stage(&repo_ref, &StageTarget::File("new.txt".into()))
        .unwrap();

    let oid = backend
        .commit(&repo_ref, "add new.txt", &CommitOptions::default())
        .unwrap();

    let log = backend
        .log(&repo_ref, &gitpoulpi_core::domain::LogOptions::default())
        .unwrap();
    assert_eq!(log.nodes[0].id, oid);
    assert_eq!(log.nodes[0].summary, "add new.txt");
    assert_eq!(log.nodes.len(), 2);
}

#[test]
fn commit_amend_replaces_the_previous_commit_message() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build();
    let backend = SystemGitBackend::new();
    let repo_ref = repo.repo_ref();

    backend
        .commit(
            &repo_ref,
            "amended message",
            &CommitOptions {
                amend: true,
                sign: false,
                ..CommitOptions::default()
            },
        )
        .unwrap();

    let log = backend
        .log(&repo_ref, &gitpoulpi_core::domain::LogOptions::default())
        .unwrap();
    assert_eq!(log.nodes.len(), 1);
    assert_eq!(log.nodes[0].summary, "amended message");
}

#[test]
fn branch_create_checkout_and_delete_round_trip() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build();
    let backend = SystemGitBackend::new();
    let repo_ref = repo.repo_ref();
    let head = backend
        .log(&repo_ref, &gitpoulpi_core::domain::LogOptions::default())
        .unwrap()
        .nodes[0]
        .id;

    backend.branch_create(&repo_ref, "feature", head).unwrap();
    backend
        .checkout(&repo_ref, &CheckoutTarget::Branch("feature".into()))
        .unwrap();
    let refs = backend.refs(&repo_ref).unwrap();
    assert!(refs.iter().any(|r| matches!(
        &r.kind,
        gitpoulpi_core::domain::RefKind::LocalBranch { name } if name == "feature"
    )));

    backend
        .checkout(&repo_ref, &CheckoutTarget::Branch("main".into()))
        .unwrap();
    backend.branch_delete(&repo_ref, "feature").unwrap();
    let refs = backend.refs(&repo_ref).unwrap();
    assert!(!refs.iter().any(|r| matches!(
        &r.kind,
        gitpoulpi_core::domain::RefKind::LocalBranch { name } if name == "feature"
    )));
}

#[test]
fn branch_rename_changes_the_ref_name() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build();
    let backend = SystemGitBackend::new();
    let repo_ref = repo.repo_ref();

    backend.branch_rename(&repo_ref, "main", "trunk").unwrap();

    let refs = backend.refs(&repo_ref).unwrap();
    assert!(refs.iter().any(|r| matches!(
        &r.kind,
        gitpoulpi_core::domain::RefKind::LocalBranch { name } if name == "trunk"
    )));
    assert!(!refs.iter().any(|r| matches!(
        &r.kind,
        gitpoulpi_core::domain::RefKind::LocalBranch { name } if name == "main"
    )));
}

#[test]
fn tag_create_and_delete_round_trip() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build();
    let backend = SystemGitBackend::new();
    let repo_ref = repo.repo_ref();
    let head = backend
        .log(&repo_ref, &gitpoulpi_core::domain::LogOptions::default())
        .unwrap()
        .nodes[0]
        .id;

    backend
        .tag_create(&repo_ref, "v1", head, Some("release v1"))
        .unwrap();
    let refs = backend.refs(&repo_ref).unwrap();
    assert!(refs
        .iter()
        .any(|r| matches!(&r.kind, gitpoulpi_core::domain::RefKind::Tag { name } if name == "v1")));

    backend.tag_delete(&repo_ref, "v1").unwrap();
    let refs = backend.refs(&repo_ref).unwrap();
    assert!(!refs
        .iter()
        .any(|r| matches!(&r.kind, gitpoulpi_core::domain::RefKind::Tag { name } if name == "v1")));
}

#[test]
fn reset_hard_discards_the_last_commit() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build();
    let backend = SystemGitBackend::new();
    let repo_ref = repo.repo_ref();
    let first = backend
        .log(&repo_ref, &gitpoulpi_core::domain::LogOptions::default())
        .unwrap()
        .nodes[0]
        .id;
    std::fs::write(repo.root().join("new.txt"), "hello\n").unwrap();
    backend
        .stage(&repo_ref, &StageTarget::File("new.txt".into()))
        .unwrap();
    backend
        .commit(&repo_ref, "second", &CommitOptions::default())
        .unwrap();

    backend.reset(&repo_ref, first, ResetMode::Hard).unwrap();

    let log = backend
        .log(&repo_ref, &gitpoulpi_core::domain::LogOptions::default())
        .unwrap();
    assert_eq!(log.nodes.len(), 1);
    assert_eq!(log.nodes[0].id, first);
    assert!(!repo.root().join("new.txt").exists());
}

#[test]
fn stash_push_pop_round_trips_the_working_tree_change() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build();
    std::fs::write(repo.root().join("initial.txt"), "changed\n").unwrap();
    let backend = SystemGitBackend::new();
    let repo_ref = repo.repo_ref();

    let id = backend
        .stash_push(&repo_ref, Some("wip"), false)
        .unwrap()
        .expect("a stash");
    assert_eq!(
        std::fs::read_to_string(repo.root().join("initial.txt")).unwrap(),
        "initial"
    );
    let stashes = backend.stashes(&repo_ref).unwrap();
    assert_eq!(stashes.len(), 1);
    assert_eq!(stashes[0].id, id);

    backend.stash_pop(&repo_ref, id).unwrap();

    assert_eq!(
        std::fs::read_to_string(repo.root().join("initial.txt"))
            .unwrap()
            .replace("\r\n", "\n"),
        "changed\n"
    );
    assert!(backend.stashes(&repo_ref).unwrap().is_empty());
}

#[test]
fn stash_apply_keeps_the_stash_while_stash_drop_removes_it() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build();
    std::fs::write(repo.root().join("initial.txt"), "changed\n").unwrap();
    let backend = SystemGitBackend::new();
    let repo_ref = repo.repo_ref();
    let id = backend
        .stash_push(&repo_ref, None, false)
        .unwrap()
        .expect("a stash");

    backend.stash_apply(&repo_ref, id).unwrap();
    assert_eq!(
        std::fs::read_to_string(repo.root().join("initial.txt"))
            .unwrap()
            .replace("\r\n", "\n"),
        "changed\n"
    );
    assert_eq!(backend.stashes(&repo_ref).unwrap().len(), 1);

    backend.stash_drop(&repo_ref, id).unwrap();
    assert!(backend.stashes(&repo_ref).unwrap().is_empty());
}
