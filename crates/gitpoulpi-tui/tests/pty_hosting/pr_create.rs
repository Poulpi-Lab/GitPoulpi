use std::time::Duration;

use crate::common;
use crate::mock::*;
use crate::support::*;

/// The header's `+` opens the Create Pull Request form: only supported
/// providers as tabs, From/To repository and branch comboboxes, Title,
/// a multi-line Description and a draft checkbox; Create posts to the
/// host.
#[test]
fn the_plus_opens_the_create_pull_request_form_and_creates_through_the_host() {
    let mock = start_mock();
    let repo = fixture(mock.port);
    let extra = "\n[[integrations]]\nid = \"bb\"\nkind = \"bitbucket\"\nurl = \"https://bitbucket.org\"\ntoken_ref = \"integration:bb\"\nname = \"bb\"\n";
    let (mut term, _config) = launch(
        &repo,
        "pty-hosting-create",
        &config_toml(mock.port, extra),
        COLS,
        "Add the feature",
    );
    let screen = term
        .wait_for("the PULL REQUESTS section", STARTUP, pulls_ready)
        .unwrap_or_else(|e| panic!("{e}"));
    let (_, header_row) = screen.find("PULL REQUESTS (2)").unwrap();
    let header = screen.line(header_row);
    let plus_col = header.find(" + ").expect("the + button") as u16 + 1;
    // The line is ASCII up to the button except for the glyphs of the
    // title: count cells, not bytes.
    let plus_cell = header[..header.find(" + ").unwrap()].chars().count() as u16 + 1;
    let _ = plus_col;
    term.click(plus_cell, header_row).expect("click the +");

    let form = term
        .wait_for("the Create Pull Request form", STEP, |s| {
            s.contains("Create Pull Request")
                && s.contains("From repo")
                && s.contains("To repo")
                && s.contains("Submit as draft")
                && s.contains("Description")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(form.contains("GitLab"), "{}", form.text());
    // GitLab takes assignees, reviewers, labels and a milestone.
    for chip in [
        "Assignees: none",
        "Reviewers: none",
        "Labels: none",
        "Milestone: none",
    ] {
        assert!(form.contains(chip), "{chip}: {}", form.text());
    }
    // Every provider with a pull-request client gets a tab (Bitbucket since
    // more-providers S10); an issue tracker never does.
    assert!(
        form.contains("Bitbucket"),
        "the Bitbucket profile has a tab: {}",
        form.text()
    );
    assert!(
        !form.contains("Generate title"),
        "the AI hook is hidden: {}",
        form.text()
    );
    // Defaults: this repository, feature -> main; the title is the tip's.
    for want in ["g/p", "feature", "main", "Add the feature"] {
        assert!(form.contains(want), "{want}: {}", form.text());
    }

    // Title: replace it; Description: two lines; the draft box by mouse.
    term.send_keys("C-u").unwrap();
    term.send_text("Ship the feature").unwrap();
    term.send_keys("Tab").unwrap();
    term.send_text("first line").unwrap();
    term.send_keys("Enter").unwrap();
    term.send_text("second line").unwrap();
    term.wait_for("both description lines", STEP, |s| {
        s.contains("first line") && s.contains("second line") && s.contains("Ship the feature")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    let (col, row) = term.wait_for_text("Submit as draft", STEP).unwrap();
    term.click(col + 1, row).expect("tick the draft box");
    term.wait_for("the box ticked", STEP, |s| {
        s.contains("[x] Submit as draft")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    // Create (the button, by mouse).
    let (col, row) = term.wait_for_text("Create Pull Request", STEP).unwrap();
    // "Create Pull Request" is both the title and the button: the button
    // is the last occurrence on screen.
    let screen = term.screen();
    let all = screen.find_all("Create Pull Request");
    let (bc, br) = *all.last().unwrap_or(&(col, row));
    term.click(bc + 2, br).expect("click Create");
    term.wait_for("the form closed with the result", STEP, |s| {
        !s.contains("From repo") && s.contains("opened #7")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    let post = mock
        .requests()
        .into_iter()
        .find(|r| r.method == "POST")
        .expect("the host was asked to create the merge request");
    assert!(
        post.target.contains("/projects/g%2Fp/merge_requests"),
        "{}",
        post.target
    );
    let body: serde_json::Value = serde_json::from_str(&post.body).expect("a JSON body");
    assert_eq!(body["source_branch"], "feature");
    assert_eq!(body["target_branch"], "main");
    assert_eq!(body["title"], "Draft: Ship the feature", "{body}");
    assert_eq!(body["description"], "first line\nsecond line");
}

/// Every mouse action has a keyboard twin: `n` in the section (and the
/// palette) opens the same form, Tab/Space/Ctrl+S drive it, Esc closes.
#[test]
fn the_create_form_is_reachable_and_drivable_from_the_keyboard() {
    let mock = start_mock();
    let repo = fixture(mock.port);
    let (mut term, _config) = launch(
        &repo,
        "pty-hosting-create-keys",
        &config_toml(mock.port, ""),
        COLS,
        "Add the feature",
    );
    term.wait_for("the PULL REQUESTS section", STARTUP, pulls_ready)
        .unwrap_or_else(|e| panic!("{e}"));
    // Select a row of the section so `n` means "new pull request here".
    let (col, row) = term.wait_for_text("Add the parser", STEP).unwrap();
    term.click(col + 2, row).expect("focus the section");
    term.send_keys("n").unwrap();
    term.wait_for("the form", STEP, |s| {
        s.contains("From repo") && s.contains("Submit as draft")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Escape").unwrap();
    term.wait_for_text_to_go("From repo", STEP)
        .unwrap_or_else(|e| panic!("{e}"));

    // Through the palette.
    term.send_keys("C-p").unwrap();
    term.wait_for("the palette", STEP, |s| {
        s.contains("Command Palette") || s.contains("Type a command")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_text("Create Pull Request").unwrap();
    term.wait_for("the command listed", STEP, |s| {
        s.contains("Pull Requests: Create Pull Request")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    term.wait_for("the form from the palette", STEP, |s| {
        s.contains("From repo") && s.contains("To repo")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    // Keyboard only: Title (focused) -> type, Ctrl+S creates.
    term.send_keys("C-u").unwrap();
    term.send_text("By keyboard").unwrap();
    term.send_keys("C-s").unwrap();
    term.wait_for("created", STEP, |s| {
        !s.contains("From repo") && s.contains("opened #7")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    let post = mock
        .requests()
        .into_iter()
        .find(|r| r.method == "POST")
        .expect("a POST");
    let body: serde_json::Value = serde_json::from_str(&post.body).unwrap();
    assert_eq!(body["title"], "By keyboard");
}

/// The first opening of Create Pull Request says it is loading while the
/// host (slow here) lists repositories, then fills in.
#[test]
fn the_create_form_shows_a_loading_state_while_the_host_is_slow() {
    let mock = start_mock();
    SLOW_PROJECTS_MS.lock().unwrap().push((mock.port, 4000));
    let repo = fixture(mock.port);
    let (mut term, _config) = launch(
        &repo,
        "pty-hosting-slow",
        &config_toml(mock.port, ""),
        COLS,
        "PULL REQUESTS (2)",
    );
    term.wait_for("the section", STARTUP, |s| s.contains("PULL REQUESTS (2)"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Escape").unwrap();
    // The click on the + can be dropped while the Escape is still being
    // processed; repeat it until the form is up. The form opens loading
    // (the mock holds the repositories for 4 s), so the loading state is
    // still on once the form shows.
    common::retry_until(
        &mut term,
        "the Create Pull Request form",
        4,
        Duration::from_secs(3),
        |t| {
            let screen = t.screen();
            if let Some((col, row)) = screen.find("PULL REQUESTS") {
                if let Some(plus) = plus_cell(&screen, row, col + 60) {
                    t.click(plus, row).expect("click the +");
                }
            }
        },
        |s| s.contains("From repo"),
    );
    term.wait_for(
        "the form with its loading state",
        Duration::from_secs(3),
        |s| s.contains("From repo") && s.contains("Loading repositories and branches"),
    )
    .unwrap_or_else(|e| panic!("{e}"));
    term.wait_for("the loading state gone", STEP, |s| {
        s.contains("From repo") && !s.contains("Loading repositories and branches")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.stop().unwrap();
}

/// The PULL REQUESTS `+` names its action on hover like the REMOTE one,
/// in the configured terminology.
#[test]
fn the_pull_request_plus_names_its_action_in_the_configured_terminology() {
    for (terminology, tooltip) in [
        ("pull_request", "+ New pull request"),
        ("merge_request", "+ New merge request"),
    ] {
        let mock = start_mock();
        let repo = fixture(mock.port);
        let (mut term, _config) = launch(
            &repo,
            &format!("pty-hosting-plus-tooltip-{terminology}"),
            &config_toml_ui(mock.port, "", terminology),
            COLS,
            "Add the feature",
        );
        term.wait_for("the section", STARTUP, |s| s.contains("REQUESTS (2)"))
            .unwrap_or_else(|e| panic!("{e}"));
        let (plus, row) = hover_plus(&mut term, "REQUESTS");
        term.move_to(plus, row).expect("hover the +");
        term.wait_for("the tooltip", STEP, |s| s.contains(tooltip))
            .unwrap_or_else(|e| panic!("{terminology}: {e}\n{}", term.screen().text()));
        term.stop().unwrap();
    }
}

/// "Create on GitLab" in the merge request form: opens the provider's own
/// new MR page with the branches, the title and the description as typed.
#[test]
fn create_on_the_provider_opens_the_prefilled_merge_request_form() {
    let mock = start_mock();
    let repo = fixture(mock.port);
    let (mut term, config) = launch(
        &repo,
        "pty-hosting-pr-web",
        &config_toml(mock.port, ""),
        COLS,
        "Add the feature",
    );
    term.wait_for("the section", STARTUP, pulls_ready)
        .unwrap_or_else(|e| panic!("{e}"));
    let (col, row) = term.wait_for_text("Add the parser", STEP).unwrap();
    term.click(col + 2, row).expect("focus the section");
    term.send_keys("n").unwrap();
    term.wait_for("the form", STEP, |s| {
        s.contains("From repo") && s.contains("Create on GitLab")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.screen().text()));
    term.send_keys("C-u").unwrap();
    term.send_text("Ship it").unwrap();
    term.send_keys("Tab").unwrap();
    term.send_text("why").unwrap();
    let (bc, br) = term.wait_for_text("Create on GitLab", STEP).unwrap();
    term.move_to(bc + 2, br).unwrap();
    term.wait_for("the tooltip", STEP, |s| {
        s.contains("in the browser, prefilled")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.screen().text()));
    term.click(bc + 2, br).unwrap();
    let want = format!(
        "http://127.0.0.1:{}/g/p/-/merge_requests/new?merge_request%5Bsource_branch%5D=feature&merge_request%5Btarget_branch%5D=main&merge_request%5Btitle%5D=Ship%20it&merge_request%5Bdescription%5D=why",
        mock.port
    );
    assert_eq!(wait_opened(&config, 1), vec![want.clone()]);
    // The key twin.
    term.send_keys("M-w").unwrap();
    assert_eq!(wait_opened(&config, 2), vec![want.clone(), want]);
    assert!(term.screen().contains("From repo"), "the form stays open");
    assert!(!mock.requests().iter().any(|r| r.method == "POST"));
    term.stop().unwrap();
}
