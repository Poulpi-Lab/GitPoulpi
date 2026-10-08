#![cfg(feature = "workspaces")]
//! Workspaces: the New workspace form, the tab bar's chip and its combo
//! (a switch swaps the tab list and back restores it), closing the last tab
//! of a workspace, the Home tab's Workspaces page, and the workspace's
//! default profile in the toolbar.
//!
//! Every test runs in its own temp config directory and session; nothing here
//! touches the maintainer's `~/.config/gitpoulpi`, and no network is involved.

mod common;

use std::path::{Path, PathBuf};
use std::time::Duration;

use gitpoulpi_session::{GlobalSession, Workspaces};
use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(30);

/// A fixture repository whose folder is called `name` (the tab's label).
fn repo(name: &str, subject: &str) -> (gitpoulpi_testkit::FixtureRepo, PathBuf) {
    let fx = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file(&format!("{name}.txt"), "x\n", subject)
        .unwrap()
        .build();
    let root = fx.root().parent().unwrap().join(name);
    std::fs::rename(fx.root(), &root).unwrap();
    (fx, root)
}

fn toml_path(p: &Path) -> String {
    format!("'{}'", p.display())
}

fn write_workspaces(config: &Path, toml: &str) {
    std::fs::write(config.join("workspaces.toml"), toml).unwrap();
}

fn session_file(config: &Path, session: &str) -> PathBuf {
    config.join("sessions").join(format!("{session}.toml"))
}

/// Starts the session in `workspace` (or none) with `tabs` open, then
/// launches in `launch`.
fn start(
    launch: &Path,
    config: &Path,
    session: &str,
    active: Option<&str>,
    ready: &str,
) -> Terminal {
    let s = GlobalSession {
        active_workspace: active.map(str::to_string),
        ..GlobalSession::default()
    };
    if let Some(parent) = session_file(config, session).parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    s.save(&session_file(config, session)).unwrap();
    common::launch(launch)
        .session(session)
        .config_dir(config)
        .size(200, 50)
        .no_exit_pause()
        .ready(ready)
        .mouse_capture()
        .spawn()
}

fn bar(term: &Terminal) -> String {
    term.text().lines().nth(1).unwrap_or("").to_string()
}

fn tab_paths(config: &Path, session: &str) -> Vec<PathBuf> {
    GlobalSession::load(&session_file(config, session))
        .unwrap()
        .tabs
        .into_iter()
        .map(|t| t.repo_path)
        .collect()
}

#[test]
fn a_workspace_is_created_from_the_form_and_written_to_workspaces_toml() {
    let (_a, alpha) = repo("orbit", "alpha-only-commit");
    let config = tempfile::tempdir().unwrap();
    let mut term = start(
        &alpha,
        config.path(),
        "pty-ws-create",
        None,
        "alpha-only-commit",
    );
    // No workspace yet: no chip on the bar.
    assert!(!bar(&term).contains("No workspace"), "{}", bar(&term));

    term.send_keys("C-p").unwrap();
    term.send_text("Workspace: New").unwrap();
    term.wait_for("the palette row", STEP, |s| s.contains("Workspace: New..."))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    term.wait_for("the form", STEP, |s| {
        s.contains("New workspace") && s.contains("Repositories") && s.contains("[ ] orbit")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    term.send_text("Acme").unwrap();
    // Name, Color, Description, Profile, Repositories: the list.
    for _ in 0..4 {
        term.send_keys("Tab").unwrap();
    }
    term.send_keys("Space").unwrap();
    term.wait_for("the repository checked", STEP, |s| s.contains("[x] orbit"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // Browse, Cancel, Save.
    for _ in 0..3 {
        term.send_keys("Tab").unwrap();
    }
    term.send_keys("Enter").unwrap();
    term.wait_for("the Workspaces page", STEP, |s| {
        s.contains("created workspace Acme") || (s.contains("Acme") && !s.contains("[x] orbit"))
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    let all = Workspaces::load(&Workspaces::file_in(config.path()));
    let acme = all.get("acme").unwrap_or_else(|| panic!("{all:?}"));
    assert_eq!(acme.name, "Acme");
    assert_eq!(acme.repos, vec![alpha.clone()]);
    assert_eq!(acme.profile, None);
    // The page lists it, with its repository count.
    term.wait_for("the list row", STEP, |s| s.contains("1 repository"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

#[test]
fn switching_workspace_swaps_the_tabs_and_switching_back_restores_them() {
    let (_a, alpha) = repo("orbit", "alpha-only-commit");
    let (_b, bravo) = repo("moonbase", "bravo-only-commit");
    let (_c, charlie) = repo("comet", "charlie-only-commit");
    let config = tempfile::tempdir().unwrap();
    write_workspaces(
        config.path(),
        &format!(
            "[[workspace]]\nid = \"acme\"\nname = \"Acme\"\ncolor = \"green\"\nrepos = [{}, {}]\n",
            toml_path(&bravo),
            toml_path(&charlie)
        ),
    );
    let session = "pty-ws-switch";
    let mut term = start(&alpha, config.path(), session, None, "alpha-only-commit");
    // The chip says there is no workspace, after the house and before the tabs.
    let before = bar(&term);
    assert!(
        before.contains("No workspace") && before.find("No workspace") < before.find("orbit"),
        "{before}"
    );

    // Alt+W opens the combo on "No workspace"; Down to Acme, Enter.
    term.send_keys("M-w").unwrap();
    term.wait_for("the combo", STEP, |s| s.contains("Manage workspaces..."))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Down").unwrap();
    term.send_keys("Enter").unwrap();
    term.wait_for("Acme's tabs", STEP, |s| {
        let b = s.text().lines().nth(1).unwrap_or("").to_string();
        b.contains("moonbase") && b.contains("comet") && !b.contains("orbit")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(bar(&term).contains("Acme"), "{}", bar(&term));
    assert_eq!(
        GlobalSession::load(&session_file(config.path(), session))
            .unwrap()
            .active_workspace
            .as_deref(),
        Some("acme")
    );

    // Back to no workspace: the plain tab list returns.
    term.send_keys("M-w").unwrap();
    term.wait_for("the combo", STEP, |s| s.contains("Manage workspaces..."))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Up").unwrap();
    term.send_keys("Enter").unwrap();
    term.wait_for("the plain tabs", STEP, |s| {
        let b = s.text().lines().nth(1).unwrap_or("").to_string();
        b.contains("orbit") && !b.contains("moonbase")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(bar(&term).contains("No workspace"), "{}", bar(&term));
    assert_eq!(tab_paths(config.path(), session), vec![alpha]);
    term.stop().unwrap();
}

#[test]
fn closing_the_last_tab_of_a_workspace_keeps_the_workspace_and_shows_home() {
    let (_a, alpha) = repo("orbit", "alpha-only-commit");
    let config = tempfile::tempdir().unwrap();
    write_workspaces(
        config.path(),
        &format!(
            "[[workspace]]\nid = \"acme\"\nname = \"Acme\"\nrepos = [{}]\n",
            toml_path(&alpha)
        ),
    );
    let session = "pty-ws-last-tab";
    let mut term = start(
        &alpha,
        config.path(),
        session,
        Some("acme"),
        "alpha-only-commit",
    );
    assert!(bar(&term).contains("Acme"), "{}", bar(&term));
    term.send_keys("C-w").unwrap();
    term.wait_for("the question", STEP, |s| s.contains("Close the tab"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    term.wait_for("Home", STEP, |s| s.contains("OPEN REPOSITORIES"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // Still in the workspace: the chip names it, the session remembers it.
    assert!(bar(&term).contains("Acme"), "{}", bar(&term));
    let file = GlobalSession::load(&session_file(config.path(), session)).unwrap();
    assert_eq!(file.active_workspace.as_deref(), Some("acme"));
    assert!(file.tabs.is_empty(), "{:?}", file.tabs);
    term.stop().unwrap();
}

/// Ctrl+O from a repository, then `Alt+J`: the tab opens on arrival (the
/// search field is not typed into until the owner focuses it).
#[test]
fn the_page_keys_work_right_after_arriving_on_home() {
    let (_a, alpha) = repo("orbit", "alpha-only-commit");
    let config = tempfile::tempdir().unwrap();
    let mut term = start(
        &alpha,
        config.path(),
        "pty-ws-keys",
        None,
        "alpha-only-commit",
    );
    term.send_keys("C-o").unwrap();
    term.wait_for("Home", STEP, |s| s.contains("OPEN REPOSITORIES"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("M-j").unwrap();
    term.wait_for("the Workspaces page", STEP, |s| s.contains("New workspace"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

#[test]
fn the_workspaces_page_lists_a_workspace_with_its_missing_repository() {
    let (_a, alpha) = repo("orbit", "alpha-only-commit");
    let config = tempfile::tempdir().unwrap();
    let gone = alpha.parent().unwrap().join("vanished");
    write_workspaces(
        config.path(),
        &format!(
            "[[workspace]]\nid = \"acme\"\nname = \"Acme\"\nprofile = \"work\"\nrepos = [{}, {}]\n\n\
             [workspace.remotes]\n{} = \"git@example.com:acme/vanished.git\"\n",
            toml_path(&alpha),
            toml_path(&gone),
            toml_path(&gone)
        ),
    );
    let mut term = start(
        &alpha,
        config.path(),
        "pty-ws-page",
        None,
        "alpha-only-commit",
    );
    term.send_keys("C-p").unwrap();
    term.send_text("Workspace: Manage").unwrap();
    term.wait_for("the palette row", STEP, |s| {
        s.contains("Workspace: Manage...")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    term.wait_for("the page", STEP, |s| {
        s.contains("2 repositories") && s.contains("1 missing") && s.contains("New workspace")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // Expand it: the missing repository is marked, with Locate and Clone.
    term.send_keys("Right").unwrap();
    term.wait_for("the repositories", STEP, |s| {
        s.contains("vanished") && s.contains("Locate...") && s.contains("Clone")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

#[test]
fn a_workspaces_default_profile_commits_unless_the_repository_chose_one() {
    let (_a, alpha) = repo("orbit", "alpha-only-commit");
    let config = tempfile::tempdir().unwrap();
    std::fs::write(
        config.path().join("config.toml"),
        "[general]\nactive_profile = \"home\"\n\n\
         [[profiles]]\nid = \"home\"\nname = \"Home\"\nauthor_name = \"H\"\nauthor_email = \"h@example.com\"\n\n\
         [[profiles]]\nid = \"work\"\nname = \"Work\"\nauthor_name = \"W\"\nauthor_email = \"w@example.com\"\n",
    )
    .unwrap();
    write_workspaces(
        config.path(),
        &format!(
            "[[workspace]]\nid = \"acme\"\nname = \"Acme\"\nprofile = \"work\"\nrepos = [{}]\n",
            toml_path(&alpha)
        ),
    );
    let term = start(
        &alpha,
        config.path(),
        "pty-ws-profile",
        Some("acme"),
        "alpha-only-commit",
    );
    term.wait_for("the profile field", STEP, |s| {
        s.contains("Work - from workspace Acme")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();

    // Outside the workspace the global default applies again.
    let config = tempfile::tempdir().unwrap();
    std::fs::write(
        config.path().join("config.toml"),
        "[general]\nactive_profile = \"home\"\n\n\
         [[profiles]]\nid = \"home\"\nname = \"Home\"\nauthor_name = \"H\"\nauthor_email = \"h@example.com\"\n\n\
         [[profiles]]\nid = \"work\"\nname = \"Work\"\nauthor_name = \"W\"\nauthor_email = \"w@example.com\"\n",
    )
    .unwrap();
    write_workspaces(
        config.path(),
        &format!(
            "[[workspace]]\nid = \"acme\"\nname = \"Acme\"\nprofile = \"work\"\nrepos = [{}]\n",
            toml_path(&alpha)
        ),
    );
    let term = start(
        &alpha,
        config.path(),
        "pty-ws-profile-none",
        None,
        "alpha-only-commit",
    );
    term.wait_for("the default profile", STEP, |s| {
        s.contains("Default (Home)")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(!term.text().contains("from workspace"), "{}", term.text());
    term.stop().unwrap();
}

/// Fetch all and Clone missing walk the workspace through the daemon on their
/// own connection, with progress on the Home page's footer and the outcome in
/// a line; a missing repository comes back from its recorded remote.
#[test]
fn clone_missing_brings_a_missing_repository_back_from_its_recorded_remote() {
    let (_a, alpha) = repo("orbit", "alpha-only-commit");
    let (_b, source) = repo("moonbase", "bravo-only-commit");
    let config = tempfile::tempdir().unwrap();
    let gone = alpha.parent().unwrap().join("restored");
    assert!(!gone.exists());
    write_workspaces(
        config.path(),
        &format!(
            "[[workspace]]\nid = \"acme\"\nname = \"Acme\"\nrepos = [{}, {}]\n\n\
             [workspace.remotes]\n{} = {}\n",
            toml_path(&alpha),
            toml_path(&gone),
            toml_path(&gone),
            toml_path(&source)
        ),
    );
    let mut term = start(
        &alpha,
        config.path(),
        "pty-ws-clone",
        None,
        "alpha-only-commit",
    );
    term.send_keys("C-p").unwrap();
    term.send_text("Workspace: Manage").unwrap();
    term.wait_for("the palette row", STEP, |s| {
        s.contains("Workspace: Manage...")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    term.wait_for("the page", STEP, |s| s.contains("1 missing"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // `m`: Clone missing, for the workspace under the cursor.
    term.send_keys("m").unwrap();
    term.wait_for("the outcome", STEP, |s| {
        s.contains("cloned the missing repositories of Acme: 1 of 1")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(gone.join(".git").exists(), "the clone is on disk");
    term.wait_for("no longer missing", STEP, |s| !s.contains("1 missing"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

#[test]
fn fetch_all_walks_the_workspace_and_lists_the_repositories_that_failed() {
    let (_a, alpha) = repo("orbit", "alpha-only-commit");
    let config = tempfile::tempdir().unwrap();
    let gone = alpha.parent().unwrap().join("vanished");
    write_workspaces(
        config.path(),
        &format!(
            "[[workspace]]\nid = \"acme\"\nname = \"Acme\"\nrepos = [{}, {}]\n",
            toml_path(&alpha),
            toml_path(&gone)
        ),
    );
    let mut term = start(
        &alpha,
        config.path(),
        "pty-ws-fetch",
        None,
        "alpha-only-commit",
    );
    term.send_keys("C-p").unwrap();
    term.send_text("Workspace: Manage").unwrap();
    term.wait_for("the palette row", STEP, |s| {
        s.contains("Workspace: Manage...")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    term.wait_for("the page", STEP, |s| s.contains("1 missing"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("F").unwrap();
    // One repository is gone from disk: it is listed, the walk still ends.
    term.wait_for("the outcome", STEP, |s| {
        s.contains("fetch all in Acme") && s.contains("failed: vanished: missing on disk")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

/// The real path: the CLI's `repo open` against the running session, with
/// `remember_tabs = false` (a launch keeps only the launch repository).
#[test]
fn the_search_scope_includes_a_tab_opened_by_the_cli() {
    let (_a, alpha) = repo("orbit", "alpha-only-commit");
    let (_b, bravo) = repo("orbit-conflict", "bravo-only-commit");
    let config = tempfile::tempdir().unwrap();
    std::fs::write(
        config.path().join("config.toml"),
        "[general]\nremember_tabs = false\n",
    )
    .unwrap();
    let session = "pty-ws-scope-cli";
    let mut term = start(&alpha, config.path(), session, None, "alpha-only-commit");
    let cli = PathBuf::from(env!("CARGO_BIN_EXE_gitpoulpi-tui")).with_file_name("gitpoulpi");
    if !cli.exists() {
        let status = std::process::Command::new(env!("CARGO"))
            .args(["build", "-p", "gitpoulpi-cli"])
            .status()
            .expect("cargo build");
        assert!(status.success());
    }
    let out = std::process::Command::new(&cli)
        .env("GITPOULPI_CONFIG_DIR", config.path())
        .env("GITPOULPI_DESKTOP_NO_HOOKS", "1")
        .args(["-s", session, "repo", "open"])
        .arg(&bravo)
        .output()
        .expect("run the CLI");
    assert!(out.status.success(), "{out:?}");
    // No wait for the bar: the loop idles in its poll, so the very next key
    // is what has to see the new tab.
    // Ctrl+Alt+F as a terminal sends it: ESC, then Ctrl+F.
    term.send_text("\x1b\x06").unwrap();
    term.wait_for("the popup", STEP, |s| {
        s.contains("Search open repositories")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(
        term.text().contains("Search open repositories (2)"),
        "{}",
        term.text()
    );
    term.stop().unwrap();
}

/// A tab added from outside (`gitpoulpi repo open`) arrives through the
/// external-tabs sync: the workspace search covers the tabs the bar shows.
#[test]
fn the_search_scope_includes_a_tab_added_from_outside() {
    let (_a, alpha) = repo("orbit", "alpha-only-commit");
    let (_b, bravo) = repo("orbit-conflict", "bravo-only-commit");
    let config = tempfile::tempdir().unwrap();
    let session = "pty-ws-scope";
    let mut term = start(&alpha, config.path(), session, None, "alpha-only-commit");
    let file = session_file(config.path(), session);
    let mut s = GlobalSession::load(&file).unwrap();
    s.open(&bravo);
    s.save(&file).unwrap();
    let mut tries = 0;
    while !bar(&term).contains("orbit-conflict") {
        term.send_keys("Down").unwrap();
        std::thread::sleep(Duration::from_millis(400));
        tries += 1;
        assert!(tries < 20, "{}", term.text());
    }
    term.send_keys("C-p").unwrap();
    term.send_text("Search: Workspace").unwrap();
    term.wait_for("the palette row", STEP, |s| {
        s.contains("Search: Workspace...")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    term.wait_for("the popup", STEP, |s| {
        s.contains("Search open repositories")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(
        term.text().contains("Search open repositories (2)"),
        "{}",
        term.text()
    );
    term.stop().unwrap();
}

/// `gitpoulpi workspace create` while the window shows the Workspaces tab: the new
/// workspace appears without a key press (the file is watched on the tick),
/// and the tab bar's chip, hidden until a workspace exists, shows up too.
#[test]
fn a_workspace_created_by_the_cli_appears_on_the_page_without_a_key() {
    let (_a, alpha) = repo("orbit", "alpha-only-commit");
    let config = tempfile::tempdir().unwrap();
    let session = "pty-ws-watch";
    let mut term = start(&alpha, config.path(), session, None, "alpha-only-commit");
    term.send_keys("C-o").unwrap();
    term.wait_for("Home", STEP, |s| s.contains("OPEN REPOSITORIES"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("M-j").unwrap();
    term.wait_for("the Workspaces page", STEP, |s| s.contains("New workspace"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(!term.text().contains("Freshly"), "{}", term.text());
    let cli = PathBuf::from(env!("CARGO_BIN_EXE_gitpoulpi-tui")).with_file_name("gitpoulpi");
    if !cli.exists() {
        let status = std::process::Command::new(env!("CARGO"))
            .args(["build", "-p", "gitpoulpi-cli"])
            .status()
            .expect("cargo build");
        assert!(status.success());
    }
    let out = std::process::Command::new(&cli)
        .env("GITPOULPI_CONFIG_DIR", config.path())
        .env("GITPOULPI_DESKTOP_NO_HOOKS", "1")
        .args(["-s", session, "workspace", "create", "Freshly"])
        .output()
        .expect("run the CLI");
    assert!(out.status.success(), "{out:?}");
    // No key: the loop wakes by itself and the page lists the workspace.
    term.wait_for("the new workspace", STEP, |s| s.contains("Freshly"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}
