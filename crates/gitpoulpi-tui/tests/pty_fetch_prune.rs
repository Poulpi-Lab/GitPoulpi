//! A branch deleted on the server (a merged merge request on a code host)
//! must leave the REMOTE section after the next Fetch, with the default
//! settings (`general.auto_prune` is on): the tracking ref is pruned and the
//! sidebar refreshes without a restart. Real binary, real daemon, real `git`,
//! a bare upstream inside the fixture's temp dir (no network).

mod common;

use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;

const STARTUP: Duration = Duration::from_secs(60);
const STEP: Duration = Duration::from_secs(30);

#[test]
fn fetch_drops_a_remote_branch_deleted_on_the_server_from_the_remote_section() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("base commit")
        .unwrap()
        .with_origin()
        .unwrap()
        .push("main")
        .unwrap()
        .push("main:zz-gone-upstream")
        .unwrap()
        .push("main:zz-kept-upstream")
        .unwrap()
        .track("main", "origin")
        .unwrap()
        .build();
    let config = tempfile::tempdir().expect("temp config dir");
    let mut term = common::launch(repo.root())
        .session("pty-fetch-prune")
        .config_dir(config.path())
        .size(200, 50)
        .spawn();
    term.wait_for("both remote branches listed", STARTUP, |s| {
        s.contains("zz-gone-upstream") && s.contains("zz-kept-upstream")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    // Someone deletes the branch on the server.
    repo.git_in_remote(&["branch", "-D", "zz-gone-upstream"]);

    term.send_keys("C-g").unwrap();
    term.send_keys("f").unwrap();
    term.wait_for("the pruned branch gone, the other kept", STEP, |s| {
        !s.contains("zz-gone-upstream") && s.contains("zz-kept-upstream")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(
        repo.git(&["for-each-ref", "refs/remotes/origin/zz-gone-upstream"])
            .trim(),
        "",
        "the tracking ref is gone from the repository"
    );
}
