use crate::mock::*;
use crate::support::*;
use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::Terminal;

// -- #123 / !45 in a commit message ---------------------------------------------

/// A repository whose HEAD commit message mentions `#4242` and `!4343`
/// in its body (so the Log's summary column does not repeat them).
fn refs_fixture(port: u16) -> FixtureRepo {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_with_body("refs-subj", "Closes #4242, see also !4343 for context.")
        .unwrap()
        .with_origin()
        .unwrap()
        .push("main")
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

fn link_both(repo: &FixtureRepo) {
    let dir = repo.state_dir();
    std::fs::create_dir_all(&dir).expect("state dir");
    std::fs::write(
        dir.join("repo-state.toml"),
        "[issues_source]\nintegration = \"corp\"\nproject = \"g/p\"\n\n[pulls_source]\nintegration = \"corp\"\nproject = \"g/p\"\n",
    )
    .expect("seed repo-state.toml");
}

/// Selects the commit in the Log so its message block shows.
fn select_head_commit(term: &mut Terminal) {
    let (col, row) = term.wait_for_text("refs-subj", STEP).unwrap();
    term.click(col + 1, row).expect("select the commit");
    term.wait_for("the message body shows", STEP, |s| {
        s.contains("Closes #4242")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.screen().text()));
}

#[test]
fn commit_message_tokens_open_the_linked_issue_and_request_by_click_and_palette() {
    let mock = start_mock();
    let repo = refs_fixture(mock.port);
    link_both(&repo);
    let (mut term, _config) = launch_unlinked(
        &repo,
        "pty-hosting-commit-refs",
        &config_toml(mock.port, ""),
        COLS,
        "refs-subj",
    );
    // The tokens are links once the linked sources have loaded.
    term.wait_for("the sections", STARTUP, pulls_ready)
        .unwrap_or_else(|e| panic!("{e}"));
    select_head_commit(&mut term);

    // Click the `!4343` token in the message block (GitLab: a request).
    let (col, row) = term.wait_for_text("!4343", STEP).unwrap();
    term.click(col + 1, row).expect("click the token");
    term.wait_for("the merge request opens", STEP, |s| {
        s.contains("g/p/-/merge_requests/4343")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.screen().text()));

    // The keyboard twin lists both references.
    term.send_keys("C-p").unwrap();
    term.send_text("Open Commit Reference").unwrap();
    term.wait_for("the palette lists it", STEP, |s| {
        s.contains("Open Commit Reference")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.screen().text()));
    term.send_keys("Enter").unwrap();
    let screen = term
        .wait_for("both references are offered", STEP, |s| {
            s.contains("Open reference") && s.contains("#4242") && s.contains("!4343")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.screen().text()));
    let (c, r) = screen
        .find_all("#4242")
        .into_iter()
        .next()
        .expect("the #4242 entry");
    term.click(c + 1, r).expect("pick #4242");
    term.wait_for("the issue opens", STEP, |s| s.contains("g/p/-/issues/4242"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.screen().text()));
}

#[test]
fn commit_message_tokens_are_plain_text_when_the_repository_is_unlinked() {
    let mock = start_mock();
    let repo = refs_fixture(mock.port);
    let (mut term, _config) = launch_unlinked(
        &repo,
        "pty-hosting-commit-refs-unlinked",
        &config_toml(mock.port, ""),
        COLS,
        "refs-subj",
    );
    select_head_commit(&mut term);
    let (col, row) = term.wait_for_text("!4343", STEP).unwrap();
    term.click(col + 1, row).expect("click the plain text");
    term.send_keys("C-p").unwrap();
    term.send_text("Open Commit Reference").unwrap();
    term.send_keys("Enter").unwrap();
    // Give the (non-)action a moment through a state-describing wait.
    term.wait_for("the palette closed", STEP, |s| {
        !s.contains("Open Commit Reference")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.screen().text()));
    let text = term.screen().text();
    assert!(
        !text.contains("merge_requests/4343") && !text.contains("issues/4242"),
        "an unlinked repository must not produce links:\n{text}"
    );
}

// -- Jira: `ABC-12` in a commit message ------------------------------------------

/// A repository whose HEAD message names a Jira key and two look-alikes.
fn jira_refs_fixture(port: u16) -> FixtureRepo {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_with_body("jira-subj", "Fixes ABC-12, see UTF-8")
        .unwrap()
        .with_origin()
        .unwrap()
        .push("main")
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

#[test]
fn jira_keys_are_links_but_utf_8_is_not() {
    let mock = start_mock();
    let repo = jira_refs_fixture(mock.port);
    let dir = repo.state_dir();
    std::fs::create_dir_all(&dir).expect("state dir");
    std::fs::write(
        dir.join("repo-state.toml"),
        "[issues_source]\nintegration = \"jira\"\nproject = \"ABC\"\n\n[pulls_source]\nintegration = \"corp\"\nproject = \"g/p\"\n",
    )
    .expect("seed repo-state.toml");
    let extra = format!(
        "\n[[integrations]]\nid = \"jira\"\nkind = \"jira_data_center\"\nurl = \"http://127.0.0.1:{}\"\ntoken_ref = \"integration:jira\"\nname = \"jira\"\n",
        mock.port
    );
    let config_dir = tempfile::tempdir().expect("temp config dir");
    std::fs::write(
        config_dir.path().join("config.toml"),
        config_toml(mock.port, &extra),
    )
    .expect("seed config.toml");
    let mut term = crate::common::launch(repo.root())
        .session("pty-hosting-jira-refs")
        .config_dir(config_dir.path())
        .size(COLS, ROWS)
        .env("GITPOULPI_NO_BROWSER", "1")
        .env("GITPOULPI_OPEN_LOG", config_dir.path().join("opened.log"))
        .env("GITPOULPI_TOKEN_CORP", "test-token")
        .env("GITPOULPI_TOKEN_JIRA", "test-token")
        .spawn();
    term.wait_for("the fixture loaded", STARTUP, |s| s.contains("jira-subj"))
        .unwrap_or_else(|e| panic!("{e}"));
    // The Jira issues have loaded, so the linked source is known.
    term.wait_for("the Jira issue row", STARTUP, |s| {
        s.contains("Crash on startup")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.screen().text()));
    let (col, row) = term.wait_for_text("jira-subj", STEP).unwrap();
    term.click(col + 1, row).expect("select the commit");
    term.wait_for("the message body shows", STEP, |s| {
        s.contains("Fixes ABC-12, see UTF-8")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.screen().text()));

    // Only `ABC-12` is underlined.
    let screen = term.screen();
    let (c, r) = screen
        .find_all("Fixes ABC-12, see UTF-8")
        .into_iter()
        .next()
        .expect("the body line");
    let underlined = |col: u16| screen.cell(col, r).is_some_and(|c| c.underline);
    let key = c + "Fixes ".len() as u16;
    assert!(underlined(key), "ABC-12 should be underlined");
    let utf = c + "Fixes ABC-12, see ".len() as u16;
    assert!(!underlined(utf), "UTF-8 must not be underlined");

    term.click(key + 1, r).expect("click the key");
    let opened = wait_opened(&config_dir, 1);
    assert_eq!(
        opened,
        vec![format!("http://127.0.0.1:{}/browse/ABC-12", mock.port)]
    );
}
