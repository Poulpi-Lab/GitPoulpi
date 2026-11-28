//! A detached HEAD on commits no branch, tag or remote reaches: the Log
//! shows them (the unfiltered walk seeds from a detached HEAD), and the WIP
//! row sits directly above the HEAD commit. Temp config dir and fixture
//! repo only.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;

const STARTUP: Duration = Duration::from_secs(60);

#[test]
fn commits_only_a_detached_head_reaches_are_in_the_log_under_the_wip_row() {
    let config = tempfile::tempdir().unwrap();
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("base one")
        .unwrap()
        .build();
    repo.git(&["checkout", "-q", "--detach", "HEAD"]);
    for name in ["lone alpha", "lone beta"] {
        repo.git(&["commit", "-q", "--allow-empty", "-m", name]);
    }
    std::fs::write(repo.root().join("dirty.txt"), "x\n").unwrap();

    let mut term = common::launch(repo.root())
        .session("detached-log")
        .config_dir(config.path())
        .size(200, 60)
        .spawn();
    term.wait_for("the detached-only commits", STARTUP, |s| {
        s.contains("lone alpha") && s.contains("lone beta") && s.contains("base one")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    let screen = term
        .wait_for_stable(Duration::from_millis(300), STARTUP)
        .unwrap_or_else(|e| panic!("{e}"));
    let row = |text: &str| -> u16 {
        screen
            .find_all(text)
            .into_iter()
            .map(|(_, r)| r)
            .next()
            .unwrap_or_else(|| panic!("no row with {text}: {}", screen.text()))
    };
    let (beta, alpha, base) = (row("lone beta"), row("lone alpha"), row("base one"));
    assert!(beta < alpha && alpha < base, "{}", screen.text());
    // The working-tree row is right above the HEAD commit, not elsewhere.
    let wip = row("// WIP");
    assert_eq!(wip + 1, beta, "{}", screen.text());

    term.send_keys("q").unwrap();
    for _ in 0..100 {
        if term.exited() {
            break;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    let _ = term.stop();
}
