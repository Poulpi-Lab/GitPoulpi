//! Azure DevOps (Server), Jira (Data Center) and Trello in the sidebar and
//! the New issue form, against the same local mock as the rest of
//! `pty_hosting` (more-providers S10): picking the provider, the project and
//! the extra query, rows with the provider's own words, the Type / List
//! dropdown of the New issue form, the `AZ !12` chip and the checkout of an
//! Azure request (and its Undo).

use std::process::Command;
use std::time::{Duration, Instant};

use crate::mock::*;
use crate::support::*;
use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::Terminal;

const DOT: &str = "\u{00B7}";

fn base_config(profiles: &str) -> String {
    format!(
        "[general]\nhosting_refresh_minutes = 0\n\n[ui]\npr_terminology = \"pull_request\"\n\n{profiles}"
    )
}

fn jira(port: u16) -> String {
    base_config(&format!(
        "[[integrations]]\nid = \"jira\"\nkind = \"jira_data_center\"\nurl = \"http://127.0.0.1:{port}\"\ntoken_ref = \"integration:jira\"\nname = \"acme\"\nuser = {{ login = \"kai\" }}\n"
    ))
}

fn azure(port: u16) -> String {
    base_config(&format!(
        "[[integrations]]\nid = \"az\"\nkind = \"azure_devops_server\"\nurl = \"http://127.0.0.1:{port}/DefaultCollection\"\ntoken_ref = \"integration:az\"\nname = \"acme\"\nuser = {{ login = \"kai\" }}\n"
    ))
}

fn linear(port: u16) -> String {
    base_config(&format!(
        "[[integrations]]\nid = \"ln\"\nkind = \"linear\"\nurl = \"http://127.0.0.1:{port}\"\ntoken_ref = \"integration:ln\"\nname = \"acme\"\nuser = {{ login = \"kai\" }}\n"
    ))
}

fn youtrack(port: u16) -> String {
    base_config(&format!(
        "[[integrations]]\nid = \"yt\"\nkind = \"youtrack\"\nurl = \"http://127.0.0.1:{port}\"\ntoken_ref = \"integration:yt\"\nname = \"acme\"\nuser = {{ login = \"kai\" }}\n"
    ))
}

fn redmine(port: u16) -> String {
    base_config(&format!(
        "[[integrations]]\nid = \"rm\"\nkind = \"redmine\"\nurl = \"http://127.0.0.1:{port}\"\ntoken_ref = \"integration:rm\"\nname = \"acme\"\nuser = {{ login = \"kai\" }}\n"
    ))
}

fn trello(port: u16) -> String {
    base_config(&format!(
        "[[integrations]]\nid = \"tr\"\nkind = \"trello\"\nurl = \"http://127.0.0.1:{port}\"\ntoken_ref = \"integration:tr\"\naccount = \"the-key\"\nname = \"board\"\nuser = {{ login = \"kai\" }}\n"
    ))
}

/// [`launch_unlinked`] with the tokens of the three extra profiles and an
/// optional seeded repo state.
fn start(
    repo: &FixtureRepo,
    session: &str,
    config: &str,
    state: Option<&str>,
) -> (Terminal, tempfile::TempDir) {
    if let Some(state) = state {
        let dir = repo.state_dir();
        std::fs::create_dir_all(&dir).expect("state dir");
        std::fs::write(dir.join("repo-state.toml"), state).expect("seed repo-state.toml");
    }
    let dir = tempfile::tempdir().expect("temp config dir");
    std::fs::write(dir.path().join("config.toml"), config).expect("seed config.toml");
    let term = crate::common::launch(repo.root())
        .session(session)
        .config_dir(dir.path())
        .size(COLS, ROWS)
        .env("GITPOULPI_NO_BROWSER", "1")
        .env("GITPOULPI_OPEN_LOG", dir.path().join("opened.log"))
        .env("GITPOULPI_TOKEN_JIRA", "test-token")
        .env("GITPOULPI_TOKEN_AZ", "test-token")
        .env("GITPOULPI_TOKEN_TR", "test-token")
        .env("GITPOULPI_TOKEN_LN", "test-token")
        .env("GITPOULPI_TOKEN_YT", "test-token")
        .env("GITPOULPI_TOKEN_RM", "test-token")
        .spawn();
    term.wait_for("the fixture loaded", STARTUP, |s| {
        s.contains("Add the feature")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    (term, dir)
}

/// `feature` pushed to a real (local) origin, so a checkout can fetch.
fn fixture_real_origin() -> FixtureRepo {
    FixtureRepoBuilder::new()
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
        .build()
}

fn wait_until(what: &str, mut ok: impl FnMut() -> bool) {
    let deadline = Instant::now() + STEP;
    while !ok() {
        assert!(Instant::now() < deadline, "timed out waiting for {what}");
        std::thread::sleep(Duration::from_millis(100));
    }
}

fn head(repo: &FixtureRepo) -> String {
    let out = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(repo.root())
        .output()
        .expect("git");
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

/// Clicks the first cell of `text` below `row` (a popup row).
fn click_below(term: &mut Terminal, text: &str, row: u16) {
    click_below_with(term, text, row, false);
}

/// [`click_below`] with a double click, for a searchable popup (one click only
/// selects).
fn double_click_below(term: &mut Terminal, text: &str, row: u16) {
    click_below_with(term, text, row, true);
}

fn click_below_with(term: &mut Terminal, text: &str, row: u16, double: bool) {
    let screen = term
        .wait_for(&format!("{text} listed"), STEP, |s| {
            s.find_all(text).iter().any(|&(_, r)| r > row)
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let (c, r) = screen
        .find_all(text)
        .into_iter()
        .find(|&(_, r)| r > row)
        .unwrap();
    if double {
        term.double_click(c + 1, r).expect("pick the row");
    } else {
        term.click(c + 1, r).expect("pick the row");
    }
}

/// Link ISSUES to Jira `ABC` through the dropdowns, type the JQL filter in
/// its row: the rows read `ABC-12 ...` with the type and status dim, and the
/// mock saw the exact JQL.
#[test]
fn linking_issues_to_jira_with_a_jql_filter_lists_keys_and_sends_the_jql() {
    let mock = start_mock();
    let repo = fixture(mock.port);
    let (mut term, _config) = start(&repo, "pty-prov-jira", &jira(mock.port), None);
    let screen = term
        .wait_for("the unlinked ISSUES section", STARTUP, |s| {
            s.contains("Select provider...")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let (c, provider_row) = screen.find("Select provider...").unwrap();
    term.click(c + 2, provider_row).expect("open the provider");
    click_below(&mut term, &format!("Jira {DOT} acme"), provider_row);
    let screen = term
        .wait_for("the repository row", STEP, |s| {
            s.contains("Select repository...")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let (c, repo_row) = screen.find("Select repository...").unwrap();
    term.click(c + 2, repo_row).expect("open the project list");
    double_click_below(&mut term, "ABC", repo_row);
    // The optional JQL row appears once a project is chosen.
    let screen = term
        .wait_for("the JQL row", STEP, |s| {
            s.contains("Filter (JQL, optional)") && s.contains("ABC-12  Crash on startup")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(screen.contains("JIRA ISSUES"), "{}", screen.text());
    let (c, r) = screen.find("Filter (JQL, optional)").unwrap();
    term.click(c + 2, r).expect("edit the JQL");
    term.send_text("component = UI").unwrap();
    term.wait_for("the JQL typed", STEP, |s| s.contains("component = UI"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    wait_until("the search with the filter", || {
        mock.requests().iter().any(|r| {
            r.method == "GET"
                && r.target.starts_with("/rest/api/2/search")
                && r.target.contains("component%20%3D%20UI")
        })
    });
    let state = repo_state(&repo);
    assert!(state.contains("project = \"ABC\""), "{state}");
    assert!(state.contains("filter = \"component = UI\""), "{state}");
    term.stop().ok();
}

/// New issue on a Jira project: the Type dropdown lists the types the host
/// offers (a mouse pick), and Create posts the picked `issuetype.id`.
#[test]
fn the_jira_new_issue_form_picks_a_type_from_the_host_and_posts_its_id() {
    let mock = start_mock();
    let repo = fixture(mock.port);
    let state = "[issues_source]\nintegration = \"jira\"\nproject = \"ABC\"\n";
    let (mut term, _config) = start(&repo, "pty-prov-jira-new", &jira(mock.port), Some(state));
    term.wait_for("the Jira rows", STARTUP, |s| {
        s.contains("ABC-12  Crash on startup")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    let (plus, row) = hover_plus(&mut term, "ISSUES");
    term.click(plus, row).expect("click the +");
    term.wait_for("the form with the default type", STEP, |s| {
        s.contains("New issue")
            && s.find_all("Task").iter().any(|&(c, _)| c >= 40)
            && s.contains("Type")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.screen().text()));
    term.send_text("Jira bug").unwrap();
    // The field shows the default; click it, pick Bug.
    let (c, field_row) = term
        .screen()
        .find_all("Task")
        .into_iter()
        .find(|&(c, _)| c >= 40)
        .unwrap();
    term.click(c + 1, field_row)
        .expect("open the Type dropdown");
    let screen = term
        .wait_for("both types listed", STEP, |s| {
            s.find_all("Bug")
                .iter()
                .any(|&(c, r)| c >= 40 && r > field_row)
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let (bc, br) = screen
        .find_all("Bug")
        .into_iter()
        .find(|&(c, r)| c >= 40 && r > field_row)
        .unwrap();
    term.click(bc + 1, br).expect("pick Bug");
    term.wait_for("Bug chosen", STEP, |s| {
        s.line(field_row).contains("Bug") && !s.line(field_row).contains("Task")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.screen().text()));
    term.send_keys("C-s").unwrap();
    wait_until("the issue posted", || mock.saw("POST", "/rest/api/2/issue"));
    let post = mock
        .requests()
        .into_iter()
        .find(|r| r.method == "POST" && r.target.starts_with("/rest/api/2/issue"))
        .unwrap();
    let body: serde_json::Value = serde_json::from_str(&post.body).expect("JSON");
    assert_eq!(body["fields"]["issuetype"]["id"], "10004", "{}", post.body);
    assert_eq!(body["fields"]["summary"], "Jira bug");
    assert_eq!(body["fields"]["project"]["key"], "ABC");
    term.stop().ok();
}

/// A Trello board: the cards are listed with their list name dim, and New
/// card picks the list by keyboard (Tab, Enter, Down, Enter).
#[test]
fn a_trello_board_lists_cards_and_new_card_picks_the_list() {
    let mock = start_mock();
    let repo = fixture(mock.port);
    let state = "[issues_source]\nintegration = \"tr\"\nproject = \"AbCd1234\"\n";
    let (mut term, _config) = start(&repo, "pty-prov-trello", &trello(mock.port), Some(state));
    let screen = term
        .wait_for("the card", STARTUP, |s| s.contains("#7  Write the docs"))
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(screen.contains("TRELLO CARDS"), "{}", screen.text());
    let (plus, row) = hover_plus(&mut term, "CARDS");
    term.click(plus, row).expect("click the +");
    term.wait_for("the form with its List", STEP, |s| {
        s.contains("New card")
            && s.find_all("List ").iter().any(|&(c, _)| c >= 40)
            && s.contains("Todo")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.screen().text()));
    term.send_text("A new card").unwrap();
    term.send_keys("Tab").unwrap();
    term.send_keys("Enter").unwrap();
    term.wait_for("the lists open", STEP, |s| s.contains("Doing"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Down").unwrap();
    term.send_keys("Enter").unwrap();
    term.wait_for("Doing chosen", STEP, |s| {
        s.find_all("Doing").len() == 1 && s.contains("A new card")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.screen().text()));
    term.send_keys("C-s").unwrap();
    wait_until("the card posted", || mock.saw("POST", "/1/cards"));
    let post = mock
        .requests()
        .into_iter()
        .find(|r| r.method == "POST" && r.target.starts_with("/1/cards"))
        .unwrap();
    let body: serde_json::Value = serde_json::from_str(&post.body).expect("JSON");
    assert_eq!(body["idList"], "l2", "{}", post.body);
    assert_eq!(body["name"], "A new card");
    term.stop().ok();
}

/// The Trello lists dropdown is a multi-pick: ticking "Doing" saves the
/// filter and hides the card of the other list; the dropdown stays open for
/// another pick.
#[test]
fn the_trello_lists_dropdown_filters_the_cards_and_is_remembered() {
    let mock = start_mock();
    let repo = fixture(mock.port);
    let state = "[issues_source]\nintegration = \"tr\"\nproject = \"AbCd1234\"\n";
    let (mut term, _config) = start(
        &repo,
        "pty-prov-trello-lists",
        &trello(mock.port),
        Some(state),
    );
    let screen = term
        .wait_for("the card and the lists row", STARTUP, |s| {
            s.contains("#7  Write the docs") && s.contains("All lists")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let (c, list_row) = screen.find("All lists").unwrap();
    term.click(c + 2, list_row).expect("open the lists");
    click_below(&mut term, "Doing", list_row);
    term.wait_for("the card hidden, the dropdown still open", STEP, |s| {
        !s.contains("#7  Write the docs") && s.contains("\u{2713} Doing")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.screen().text()));
    let state = repo_state(&repo);
    assert!(state.contains("filter = \"Doing\""), "{state}");
    term.send_keys("Escape").unwrap();
    term.stop().ok();
}

/// The palette twin of the query row: "Issues: Edit Query / Lists..." puts
/// the keyboard in the Jira JQL field (type, Enter), and opens Trello's lists
/// dropdown.
#[test]
fn the_palette_edits_the_jira_query_and_opens_the_trello_lists() {
    let mock = start_mock();
    let repo = fixture(mock.port);
    let state = "[issues_source]\nintegration = \"jira\"\nproject = \"ABC\"\n";
    let (mut term, _config) = start(&repo, "pty-prov-query-jira", &jira(mock.port), Some(state));
    term.wait_for("the Jira rows and the JQL row", STARTUP, |s| {
        s.contains("ABC-12  Crash on startup") && s.contains("Filter (JQL, optional)")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("C-p").unwrap();
    term.send_text("Edit Query").unwrap();
    term.wait_for("the palette lists it", STEP, |s| {
        s.contains("Issues: Edit Query / Lists")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.screen().text()));
    term.send_keys("Enter").unwrap();
    term.send_text("component = UI").unwrap();
    term.wait_for("the JQL typed in the row", STEP, |s| {
        s.contains("component = UI")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.screen().text()));
    term.send_keys("Enter").unwrap();
    wait_until("the search with the filter", || {
        mock.requests().iter().any(|r| {
            r.method == "GET"
                && r.target.starts_with("/rest/api/2/search")
                && r.target.contains("component%20%3D%20UI")
        })
    });
    term.stop().ok();

    let mock = start_mock();
    let repo = fixture(mock.port);
    let state = "[issues_source]\nintegration = \"tr\"\nproject = \"AbCd1234\"\n";
    let (mut term, _config) = start(
        &repo,
        "pty-prov-query-trello",
        &trello(mock.port),
        Some(state),
    );
    term.wait_for("the card and the lists row", STARTUP, |s| {
        s.contains("#7  Write the docs") && s.contains("All lists")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("C-p").unwrap();
    term.send_text("Edit Query").unwrap();
    term.wait_for("the palette lists it", STEP, |s| {
        s.contains("Issues: Edit Query / Lists")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.screen().text()));
    term.send_keys("Enter").unwrap();
    term.wait_for("the lists dropdown", STEP, |s| s.contains("Doing"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.screen().text()));
    term.send_keys("Escape").unwrap();
    term.stop().ok();
}

/// An Azure DevOps repository linked to PULL REQUESTS: rows `!12`, the
/// `AZ !12` chip on the checked-out branch, and "Check out as a local
/// branch" fetches `pr/12` and checks it out; Undo goes back.
#[test]
fn an_azure_pull_request_shows_its_chip_checks_out_as_pr_12_and_undoes() {
    let mock = start_mock();
    let repo = fixture_real_origin();
    let (mut term, _config) = start(&repo, "pty-prov-azure", &azure(mock.port), None);
    let screen = term
        .wait_for("the PULL REQUESTS section", STARTUP, |s| {
            s.contains("PULL REQUESTS") && s.contains("Select provider...")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let (col, header) = section_rows(&screen, "PULL REQUESTS").unwrap();
    let provider_row = header + 1;
    term.click(col + 4, provider_row)
        .expect("open the provider");
    click_below(&mut term, &format!("Azure DevOps {DOT} acme"), provider_row);
    let repo_row = provider_row + 1;
    term.wait_for("the repository row", STEP, |s| {
        s.line(repo_row).contains("Select repository...")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.click(col + 4, repo_row)
        .expect("open the repositories");
    double_click_below(&mut term, "P/R", repo_row);
    let screen = term
        .wait_for("the request listed, the chip on HEAD", STEP, |s| {
            s.contains("PULL REQUESTS (1)")
                && s.contains("!12  Add the feature")
                && s.contains("AZ !12")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.screen().text()));
    assert_eq!(head(&repo), "feature");
    let state = repo_state(&repo);
    assert!(state.contains("project = \"P/R\""), "{state}");

    let (c, r) = screen.find("!12  Add the feature").unwrap();
    term.right_click(c + 2, r).expect("the row menu");
    let screen = term
        .wait_for("the menu", STEP, |s| s.contains("as a local branch"))
        .unwrap_or_else(|e| panic!("{e}"));
    let (c, r) = screen.find("as a local branch").unwrap();
    term.click(c + 1, r).expect("checkout");
    wait_until("pr/12 checked out", || head(&repo) == "pr/12");

    // Undo restores the previous branch (rule 12).
    //
    // HEAD moves *before* the daemon journals the checkout and nothing on
    // screen says the entry is there, so an Undo clicked the moment HEAD
    // changes can find nothing to undo. Click again until it has taken (the
    // only journaled operation here is the checkout, so a repeat after a
    // slow success has nothing further to undo).
    let deadline = Instant::now() + xtask::pty::scaled(STEP);
    'undo: loop {
        let screen = term.screen();
        let (uc, ur) = screen
            .find_all("Undo")
            .into_iter()
            .min_by_key(|&(_, r)| r)
            .expect("the Undo button");
        term.click(uc + 1, ur).expect("undo");
        let attempt = Instant::now() + Duration::from_secs(4);
        while Instant::now() < attempt {
            if head(&repo) == "feature" {
                break 'undo;
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        assert!(
            Instant::now() < deadline,
            "timed out waiting for back on feature"
        );
    }
    term.stop().ok();
}

/// A linked Linear team: the rows read `ENG-12  Crash on startup`, the
/// section is titled after the provider, and the list is asked over the one
/// GraphQL endpoint with the bare key and the team filter.
#[test]
fn a_linear_team_lists_its_issues_over_graphql() {
    let mock = start_mock();
    let repo = fixture(mock.port);
    let state = "[issues_source]\nintegration = \"ln\"\nproject = \"ENG\"\n";
    let (term, _config) = start(&repo, "pty-prov-linear", &linear(mock.port), Some(state));
    let screen = term
        .wait_for("the Linear rows", STARTUP, |s| {
            s.contains("ENG-12  Crash on startup")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.screen().text()));
    assert!(screen.contains("LINEAR ISSUES"), "{}", screen.text());
    assert!(
        !screen.contains("optional)"),
        "Linear has no query row:\n{}",
        screen.text()
    );
    let listed = mock
        .requests()
        .into_iter()
        .find(|r| r.method == "POST" && r.body.contains("query Issues"))
        .expect("the issues query");
    assert!(
        listed
            .headers
            .to_lowercase()
            .contains("authorization: test-token"),
        "{}",
        listed.headers
    );
    let body: serde_json::Value = serde_json::from_str(&listed.body).expect("JSON");
    assert_eq!(body["variables"]["filter"]["team"]["key"]["eq"], "ENG");
    term.stop().ok();
}

/// A linked YouTrack project: the rows read `ABC-12  Crash on startup`, the
/// query row takes YouTrack's syntax, and the list is asked with the query.
#[test]
fn a_youtrack_project_lists_its_issues_and_takes_a_query() {
    let mock = start_mock();
    let repo = fixture(mock.port);
    let state = "[issues_source]\nintegration = \"yt\"\nproject = \"ABC\"\n";
    let (mut term, _config) = start(
        &repo,
        "pty-prov-youtrack",
        &youtrack(mock.port),
        Some(state),
    );
    let screen = term
        .wait_for("the YouTrack rows", STARTUP, |s| {
            s.contains("ABC-12  Crash on startup") && s.contains("Filter (YouTrack quer")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.screen().text()));
    assert!(screen.contains("YOUTRACK ISSUES"), "{}", screen.text());
    let (c, r) = screen.find("Filter (YouTrack quer").unwrap();
    term.click(c + 2, r).expect("edit the query");
    term.send_text("Type: Bug").unwrap();
    term.wait_for("the query typed", STEP, |s| s.contains("Type: Bug"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    wait_until("the list with the query", || {
        mock.requests().iter().any(|r| {
            r.method == "GET"
                && r.target.starts_with("/api/issues")
                && r.target.contains("and%20%28Type%3A%20Bug%29")
        })
    });
    assert!(
        mock.requests().iter().any(|r| r
            .headers
            .to_lowercase()
            .contains("authorization: bearer test-token")),
        "the permanent token rides as a Bearer"
    );
    let state = repo_state(&repo);
    assert!(state.contains("filter = \"Type: Bug\""), "{state}");
    term.stop().ok();
}

/// A linked Redmine project: the rows read `#12  Crash on startup`, there is
/// no extra-query row (Redmine takes none), and the API key rides in its own
/// header, never as a Bearer.
#[test]
fn a_redmine_project_lists_its_issues() {
    let mock = start_mock();
    let repo = fixture(mock.port);
    let state = "[issues_source]\nintegration = \"rm\"\nproject = \"bench\"\n";
    let (term, _config) = start(&repo, "pty-prov-redmine", &redmine(mock.port), Some(state));
    let screen = term
        .wait_for("the Redmine rows", STARTUP, |s| {
            s.contains("#12  Crash on startup")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.screen().text()));
    assert!(screen.contains("REDMINE ISSUES"), "{}", screen.text());
    assert!(
        !screen.contains("Filter ("),
        "Redmine takes no provider query:\n{}",
        screen.text()
    );
    wait_until("the list request", || {
        mock.requests().iter().any(|r| {
            r.method == "GET"
                && r.target.starts_with("/issues.json")
                && r.target.contains("project_id=bench")
                && r.headers
                    .to_lowercase()
                    .contains("x-redmine-api-key: test-token")
                && !r.headers.to_lowercase().contains("authorization:")
        })
    });
    term.stop().ok();
}
