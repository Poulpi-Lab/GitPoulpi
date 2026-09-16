use gitpoulpi_testkit::FixtureRepoBuilder;

#[test]
fn builds_a_repo_with_a_commit_and_a_branch() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .branch("feature")
        .unwrap()
        .commit("on-feature")
        .unwrap()
        .build();

    assert!(repo.root().join(".git").is_dir());
    assert!(repo.root().join("initial.txt").exists());
    assert!(repo.root().join("on-feature.txt").exists());
}

#[test]
fn the_panicking_builder_chains_without_unwraps() {
    let repo = gitpoulpi_testkit::TestRepoBuilder::new()
        .with_initial()
        .branch("feature")
        .commit("on-feature")
        .build();

    assert!(repo.root().join("initial.txt").exists());
    assert!(repo.root().join("on-feature.txt").exists());
}

#[test]
fn with_initial_is_the_initial_commit() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .with_initial()
        .unwrap()
        .build();

    assert_eq!(repo.git(&["log", "--format=%s"]).trim(), "initial");
}
