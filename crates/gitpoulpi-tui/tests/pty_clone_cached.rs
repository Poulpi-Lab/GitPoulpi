//! The Clone dialog lists an integration's repositories at once from a saved
//! copy while the host is asked again; when the host answers, the same
//! project stays chosen and the viewport stays on the same projects, even
//! though repositories appeared above and one vanished from the view.
//!
//! The rules are `pty_ui.rs`'s: nothing is located by a hard-coded
//! coordinate and every wait describes the whole state it is waiting for.
//! The mock host's list endpoint is held shut (`MockHost::hold_lists`) so the
//! saved copy is on screen for as long as the test wants.
#![cfg(all(feature = "github", feature = "gitlab"))]

mod common;

use std::time::{Duration, Instant};

use gitpoulpi_config::{ConnectedUser, Integration, IntegrationKind};
use gitpoulpi_hosting::list_cache::{self, CachedList, ListKey};
use gitpoulpi_hosting::HostedProject;
use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::mock_host::{self, Options};

const STARTUP: Duration = Duration::from_secs(60);
const STEP: Duration = Duration::from_secs(30);
const COLS: u16 = 160;
const ROWS: u16 = 60;
const HUB: &str = "GitHub \u{00B7} hub";

fn saved(full: &str, port: u16) -> HostedProject {
    HostedProject {
        integration_id: "hub".into(),
        id: full.into(),
        name: full.rsplit('/').next().unwrap().into(),
        full_name: full.into(),
        description: None,
        default_branch: Some("main".into()),
        ssh_url: None,
        https_url: Some(format!("http://127.0.0.1:{port}/saved/{full}.git")),
        web_url: None,
        private: false,
    }
}

#[test]
fn the_saved_list_shows_at_once_and_the_refresh_keeps_the_view_and_the_choice() {
    let bare = tempfile::tempdir().unwrap();
    mock_host::seed_bare_repos(bare.path()).unwrap();
    let mock = mock_host::start(Options {
        port: 0,
        bare_root: Some(bare.path().to_path_buf()),
    })
    .unwrap();
    let gate = mock.hold_lists();
    let start = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "a\n", "clone-cached-start-commit")
        .unwrap()
        .build();
    let clone_root = tempfile::tempdir().unwrap();
    let config = tempfile::tempdir().unwrap();
    let url = format!("http://127.0.0.1:{}", mock.port);
    std::fs::write(
        config.path().join("config.toml"),
        format!(
            "[repos]\nclone_root = '{root}'\n\n\
             [[integrations]]\nid = \"hub\"\nkind = \"github_enterprise\"\nurl = \"{url}\"\ntoken_ref = \"integration:hub\"\nname = \"hub\"\nuser = {{ login = \"{gh}\" }}\n",
            root = clone_root.path().display(),
            gh = mock_host::GITHUB_LOGIN,
        ),
    )
    .unwrap();

    // The saved copy: the host's list without `mara-quinn/dotfiles`, plus a
    // repository that is gone from the host by now.
    let mut profile = Integration::new("hub", IntegrationKind::GithubEnterprise, &url);
    profile.user = Some(ConnectedUser {
        login: mock_host::GITHUB_LOGIN.into(),
        ..Default::default()
    });
    let key = ListKey::of(&profile, Some(mock_host::GITHUB_LOGIN)).unwrap();
    let mut projects: Vec<HostedProject> = mock_host::GITHUB_REPOS
        .iter()
        .map(|r| r.0)
        .filter(|name| *name != "mara-quinn/dotfiles")
        .map(|name| saved(name, mock.port))
        .collect();
    projects.push(saved("harbor-guild/harbor-attic", mock.port));
    list_cache::store(
        &config.path().join("cache").join("clone-lists"),
        &CachedList::new(key, 1, projects),
    )
    .unwrap();

    let mut term = common::launch(start.root())
        .session("pty-clone-cached")
        .config_dir(config.path())
        .size(COLS, ROWS)
        .env("GITPOULPI_NO_BROWSER", "1")
        .env("GITPOULPI_TOKEN_HUB", "hub-token")
        .spawn();
    term.wait_for("the first window", STARTUP, |s| {
        s.contains("clone-cached-start-commit")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("C-o").expect("the management tab");
    term.wait_for("the management tab", STEP, |s| {
        s.contains("Repository Management") && s.contains("Clone")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("c").expect("Clone");
    term.wait_for("the Clone dialog", STEP, |s| {
        s.contains("Clone a repository") && s.contains(HUB)
    })
    .unwrap_or_else(|e| panic!("{e}"));
    let (col, row) = snap(&term).find(HUB).unwrap();
    term.click(col + 2, row).expect("pick the source");

    // The saved list is there while the host has not answered.
    let cached = term
        .wait_for("the saved list, refreshing", STEP, |s| {
            s.contains("refreshing...")
                && s.contains("harbor-attic")
                && s.contains("mara-quinn (1)")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let deadline = Instant::now() + STEP;
    while gate.waiting() == 0 {
        assert!(
            Instant::now() < deadline,
            "the host was never asked:\n{}",
            cached.text()
        );
        std::thread::sleep(Duration::from_millis(20));
    }

    // Walk to lumen-labs/lumen-cli (search, then Tab into the list) and choose it.
    let (col, row) = term.wait_for_text("Where to clone to", STEP).unwrap();
    term.click(col + 2, row + 4).expect("focus the search");
    term.send_keys("Tab").unwrap();
    term.send_keys("Down Down Down Down").unwrap();
    term.send_keys("Enter").unwrap();
    let first_row = |s: &xtask::pty::Screen| {
        s.text()
            .lines()
            .position(|l| l.contains("harbor-guild (") || l.contains("mara-quinn ("))
            .map(|i| s.line(i as u16).to_string())
            .unwrap_or_default()
    };
    let chosen = term
        .wait_for(
            "lumen-cli chosen, the harbor-guild section on top",
            STEP,
            |s| {
                s.contains("\u{2713}")
                    && s.text()
                        .lines()
                        .any(|l| l.contains("Full path:") && l.contains("lumen-cli"))
                    && first_row(s).contains("harbor-guild (3)")
            },
        )
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(
        first_row(&chosen).contains("harbor-guild (3)"),
        "the list is scrolled to lumen-cli:\n{}",
        chosen.text()
    );

    // The host answers: dotfiles appears above, harbor-attic is gone.
    gate.release();
    let fresh = term
        .wait_for("the fresh list in place", STEP, |s| {
            !s.contains("refreshing...")
                && first_row(s).contains("harbor-guild (2)")
                && !s.contains("mara-quinn (")
                && !s.contains("harbor-attic")
                && s.text()
                    .lines()
                    .any(|l| l.contains("Full path:") && l.contains("lumen-cli"))
                && s.contains("\u{2713}")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(
        first_row(&fresh).contains("harbor-guild (2)"),
        "the viewport stayed on the same projects:\n{}",
        fresh.text()
    );
    assert!(
        !fresh.contains("mara-quinn (") && !fresh.contains("harbor-attic"),
        "{}",
        fresh.text()
    );
    assert!(
        fresh
            .text()
            .lines()
            .any(|l| l.contains("Full path:") && l.contains("lumen-cli")),
        "the chosen project is still chosen:\n{}",
        fresh.text()
    );
    assert!(fresh.contains("\u{2713}"), "{}", fresh.text());
    assert!(
        mock.saw("GET", "/api/v3/user/repos"),
        "the list was fetched from the host"
    );
}

/// The screen once it has settled: a torn frame is not a place to locate a
/// cell or compare colors in.
fn snap(term: &xtask::pty::Terminal) -> xtask::pty::Screen {
    term.wait_for_stable(
        std::time::Duration::from_millis(300),
        std::time::Duration::from_secs(30),
    )
    .unwrap_or_else(|e| panic!("{e}"))
}
