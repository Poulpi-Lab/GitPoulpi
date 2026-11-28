//! The terminal UI's name for the daemon client, which lives in
//! `gitpoulpi-link`: every `crate::daemon_link::...` path of the TUI resolves
//! to it. The host the client calls back is `daemon_link_host.rs`.

pub(crate) use gitpoulpi_link::*;

/// The shared test harness of a link against an embedded daemon, started
/// with this frontend's own host.
#[cfg(test)]
pub(crate) mod ticket_tests {
    pub(crate) use gitpoulpi_link::ticket_tests::*;

    pub(crate) fn harness(name: &str, fixture: &gitpoulpi_testkit::FixtureRepo) -> Harness {
        harness_with_host(name, fixture.repo_ref(), crate::daemon_link_host::host())
    }

    /// A repository whose `main` was pushed, then moved on at the remote,
    /// then moved on here: a plain push is refused as non-fast-forward,
    /// and pulling first (the files differ) merges cleanly.
    pub(crate) fn diverged_fixture() -> gitpoulpi_testkit::FixtureRepo {
        let fixture = gitpoulpi_testkit::FixtureRepoBuilder::new()
            .unwrap()
            .commit_file("base.txt", "base\n", "base")
            .unwrap()
            .with_origin()
            .unwrap()
            .build();
        fixture.git(&["push", "-u", "origin", "main"]);
        fixture.push_from_elsewhere("main", "theirs");
        std::fs::write(fixture.root().join("mine.txt"), "mine\n").unwrap();
        fixture.git(&["add", "mine.txt"]);
        fixture.git(&["commit", "-m", "mine"]);
        fixture
    }
}
