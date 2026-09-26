use std::time::Duration;

use crate::common;
use crate::mock::*;
use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::{Screen, Terminal};

pub const STARTUP: Duration = Duration::from_secs(60);
pub const STEP: Duration = Duration::from_secs(30);
pub const COLS: u16 = 160;
pub const ROWS: u16 = 60;

// -- the fixture ---------------------------------------------------------------

/// A repository on `feature` (pushed, tracking `origin/feature`) whose
/// `origin` URL points at the mock.
pub fn fixture(port: u16) -> FixtureRepo {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "a\n", "pty-hosting-start")
        .unwrap()
        .with_origin()
        .unwrap()
        .push("main")
        .unwrap()
        .branch("feature")
        .unwrap()
        .commit_file("f.txt", "f\n", "Add the feature")
        .unwrap()
        .push("feature")
        .unwrap()
        .track("feature", "origin")
        .unwrap()
        .build();
    repo.git(&[
        "remote",
        "set-url",
        "origin",
        &format!("http://127.0.0.1:{port}/g/p.git"),
    ]);
    repo
}

/// Opens the ISSUES repository dropdown (the row under the provider row).
pub fn open_repo_dropdown(term: &mut Terminal) -> u16 {
    let screen = term
        .wait_for("the ISSUES section", STARTUP, |s| {
            s.contains("GITLAB ISSUES") && s.contains("GitLab \u{00B7} corp")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let (_, provider_row) = screen.find("GitLab \u{00B7} corp").unwrap();
    let repo_row = provider_row + 1;
    let (col, _) = screen.find("GITLAB ISSUES").unwrap();
    term.click(col + 4, repo_row)
        .expect("open the repository dropdown");
    repo_row
}

// -- narrow sidebar ---------------------------------------------------------

/// The cell column of the right-most ` + ` on `row`, left of `limit`.
pub fn plus_cell(screen: &Screen, row: u16, limit: u16) -> Option<u16> {
    let cells: Vec<char> = screen.line(row).chars().collect();
    (0..cells.len().saturating_sub(2))
        .rev()
        .find(|&i| {
            (i as u16) < limit && cells[i] == ' ' && cells[i + 1] == '+' && cells[i + 2] == ' '
        })
        .map(|i| i as u16 + 1)
}

/// Hovers a sidebar header and returns the cell its `+` is drawn at,
/// waiting for it to appear (drawn over the title when it does not fit).
pub fn hover_plus(term: &mut Terminal, title: &str) -> (u16, u16) {
    let screen = term.screen();
    let (col, row) = screen
        .find_all(title)
        .into_iter()
        .find(|(c, _)| *c < 40)
        .unwrap_or_else(|| panic!("no {title} header:\n{}", screen.text()));
    term.move_to(col + 2, row).expect("hover the header");
    let screen = term
        .wait_for("the + over the header", STEP, |s| {
            plus_cell(s, row, col + 24).is_some()
        })
        .unwrap_or_else(|e| panic!("{e}"));
    (plus_cell(&screen, row, col + 24).unwrap(), row)
}

pub fn config_toml(port: u16, extra: &str) -> String {
    config_toml_ui(port, extra, "pull_request")
}

/// [`config_toml`] with `ui.pr_terminology` set (`auto` leaves the host
/// to decide: the mock is a GitLab, so `auto` says "Merge Request").
pub fn config_toml_ui(port: u16, extra: &str, terminology: &str) -> String {
    format!(
        "[general]\nhosting_refresh_minutes = 0\n\n[ui]\npr_terminology = \"{terminology}\"\n\n[[integrations]]\nid = \"corp\"\nkind = \"gitlab_self_managed\"\nurl = \"http://127.0.0.1:{port}\"\ntoken_ref = \"integration:corp\"\nname = \"corp\"\nuser = {{ login = \"kai\" }}\n{extra}"
    )
}

/// Seeds the repository's state with a linked PULL REQUESTS section
/// (`corp`, `g/p`): a fresh repository is unlinked, so the tests that are
/// about the section itself start from this.
pub fn link_pulls(repo: &FixtureRepo) {
    let dir = repo.state_dir();
    std::fs::create_dir_all(&dir).expect("state dir");
    std::fs::write(
        dir.join("repo-state.toml"),
        "[pulls_source]\nintegration = \"corp\"\nproject = \"g/p\"\n",
    )
    .expect("seed repo-state.toml");
}

/// The ISSUES counterpart of [`link_pulls`].
pub fn link_issues(repo: &FixtureRepo) {
    let dir = repo.state_dir();
    std::fs::create_dir_all(&dir).expect("state dir");
    std::fs::write(
        dir.join("repo-state.toml"),
        "[issues_source]\nintegration = \"corp\"\nproject = \"g/p\"\n",
    )
    .expect("seed repo-state.toml");
}

/// [`launch_unlinked`] after [`link_issues`] (the PULL REQUESTS section
/// stays unlinked, so its provider row does not echo the ISSUES one).
pub fn launch_issues(
    repo: &FixtureRepo,
    session: &str,
    config: &str,
    cols: u16,
    wait_for: &str,
) -> (Terminal, tempfile::TempDir) {
    link_issues(repo);
    launch_unlinked(repo, session, config, cols, wait_for)
}

/// [`launch_unlinked`] after [`link_pulls`].
pub fn launch(
    repo: &FixtureRepo,
    session: &str,
    config: &str,
    cols: u16,
    wait_for: &str,
) -> (Terminal, tempfile::TempDir) {
    link_pulls(repo);
    launch_unlinked(repo, session, config, cols, wait_for)
}

pub fn launch_unlinked(
    repo: &FixtureRepo,
    session: &str,
    config: &str,
    cols: u16,
    wait_for: &str,
) -> (Terminal, tempfile::TempDir) {
    let dir = tempfile::tempdir().expect("temp config dir");
    std::fs::write(dir.path().join("config.toml"), config).expect("seed config.toml");
    let term = common::launch(repo.root())
        .session(session)
        .config_dir(dir.path())
        .size(cols, ROWS)
        .env("GITPOULPI_NO_BROWSER", "1")
        // The browser stub: every opened URL is appended to this file.
        .env("GITPOULPI_OPEN_LOG", dir.path().join("opened.log"))
        .env("GITPOULPI_TOKEN_CORP", "test-token")
        .spawn();
    let needle = wait_for.to_string();
    term.wait_for("the fixture loaded", STARTUP, move |s| s.contains(&needle))
        .unwrap_or_else(|e| panic!("{e}"));
    (term, dir)
}

/// The section is up: its header with a count, the search box, the
/// filter and the four groups.
pub fn pulls_ready(s: &Screen) -> bool {
    s.contains("PULL REQUESTS (2)")
        && s.contains("Search pull requests")
        && s.contains("My Pull Requests")
        && s.contains("Assigned to me")
        && s.contains("Awaiting my review")
        && s.contains("All Pull Requests")
}

/// [`pulls_ready`] for a narrow sidebar, where labels are ellipsised.
pub fn pulls_ready_narrow(s: &Screen) -> bool {
    s.contains("PULL REQUESTS") && s.contains("Search pull") && s.contains("Pull Req")
}

/// A repository's section header row and the combo rows under it: the
/// title (`PULL REQUESTS` / `ISSUES`, which may carry a provider or a
/// count) is on the header row, the provider combo right below it and the
/// repository combo below that.
pub fn section_rows(s: &Screen, title: &str) -> Option<(u16, u16)> {
    let (col, row) = s.find_all(title).into_iter().find(|&(c, _)| c < 40)?;
    Some((col, row))
}

/// The state file's text, empty when there is none yet.
pub fn repo_state(repo: &FixtureRepo) -> String {
    std::fs::read_to_string(repo.state_dir().join("repo-state.toml")).unwrap_or_default()
}

pub fn host_list_requests(mock: &Mock) -> usize {
    mock.requests()
        .iter()
        .filter(|r| {
            r.method == "GET"
                && (r.target.contains("/merge_requests") || r.target.contains("/issues"))
        })
        .count()
}

/// The URLs the app "opened in the browser" so far (the stub's log).
pub fn opened_urls(config: &tempfile::TempDir) -> Vec<String> {
    std::fs::read_to_string(config.path().join("opened.log"))
        .unwrap_or_default()
        .lines()
        .map(str::to_string)
        .collect()
}

/// Waits until the stub's log holds at least `n` URLs.
pub fn wait_opened(config: &tempfile::TempDir, n: usize) -> Vec<String> {
    let deadline = std::time::Instant::now() + STEP;
    loop {
        let urls = opened_urls(config);
        if urls.len() >= n || std::time::Instant::now() > deadline {
            return urls;
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}
