use super::*;

#[test]
fn the_cache_never_dirties_the_repository_and_skips_unlinked_ones() {
    let dir = tempfile::tempdir().unwrap();
    let git = |args: &[&str]| {
        let out = std::process::Command::new("git")
            .args(args)
            .current_dir(dir.path())
            .output()
            .expect("run git");
        assert!(out.status.success(), "git {args:?}");
        String::from_utf8_lossy(&out.stdout).into_owned()
    };
    git(&["init", "-q"]);
    let path = cache_path(dir.path(), ".gitpoulpi");
    let factory = FakeFactory::new();
    let unlinked = fetch_snapshot(
        &unlinked_request("https://gitlab.example.com/g/p.git"),
        &factory,
        &runtime(),
        1,
    );
    save_cache(&path, &unlinked);
    assert!(
        !path.exists(),
        "nothing to cache for an unlinked repository"
    );
    let linked = fetch_snapshot(
        &request("https://gitlab.example.com/g/p.git"),
        &factory,
        &runtime(),
        1,
    );
    save_cache(&path, &linked);
    assert!(path.exists());
    assert_eq!(git(&["status", "--porcelain", "--untracked-files=all"]), "");
}

#[test]
fn the_cache_round_trips_through_disk() {
    let dir = tempfile::tempdir().unwrap();
    let path = cache_path(dir.path(), ".gitpoulpi");
    assert!(load_cache(&path).is_none());
    let mut factory = FakeFactory::new();
    factory.prs = vec![pr(1, "b", PullRequestState::Draft)];
    let snap = fetch_snapshot(
        &request("https://gitlab.example.com/g/p.git"),
        &factory,
        &runtime(),
        42,
    );
    save_cache(&path, &snap);
    assert_eq!(load_cache(&path), Some(snap));
}
