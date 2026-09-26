use std::time::Duration;

use crate::mock::*;
use crate::support::*;

// -- terminology and "Edit on ..." ---------------------------------------------

/// `auto` follows the linked host: the mock is a GitLab, so every label
/// says "Merge Request" - the section, its groups, the search box, the
/// palette - and a forced `pull_request` (the other tests) keeps "Pull".
#[test]
fn auto_terminology_says_merge_request_for_a_gitlab_host_everywhere() {
    let mock = start_mock();
    let repo = fixture(mock.port);
    let (mut term, _config) = launch(
        &repo,
        "pty-hosting-term-auto",
        &config_toml_ui(mock.port, "", "auto"),
        COLS,
        "Add the feature",
    );
    let screen = term
        .wait_for("the MERGE REQUESTS section", STARTUP, |s| {
            s.contains("MERGE REQUESTS (2)")
                && s.contains("Search merge requests")
                && s.contains("My Merge Requests")
                && s.contains("All Merge Requests")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(
        !screen.contains("PULL REQUESTS") && !screen.contains("Pull Requests"),
        "{}",
        screen.text()
    );
    // The palette follows: its Pull Requests commands are Merge Requests
    // commands.
    term.send_keys("C-p").unwrap();
    term.send_text("merge request").unwrap();
    let screen = term
        .wait_for("the palette lists merge request commands", STEP, |s| {
            s.contains("Merge Requests: Create Merge Request")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(!screen.contains("Pull Requests:"), "{}", screen.text());
    term.send_keys("Escape").unwrap();
}

/// The Settings combo ("Auto (per host)" / "Pull Request" / "Merge
/// Request") is a dropdown, persists, and relabels the window at once.
#[test]
fn the_terminology_combo_in_settings_persists_and_relabels_the_window() {
    let mock = start_mock();
    let repo = fixture(mock.port);
    let (mut term, config) = launch(
        &repo,
        "pty-hosting-term-combo",
        &config_toml_ui(mock.port, "", "auto"),
        COLS,
        "Add the feature",
    );
    term.wait_for("merge requests", STARTUP, |s| {
        s.contains("MERGE REQUESTS (2)")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    let (gear_col, gear_row) = term
        .wait_for_text("\u{2699}", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(gear_col, gear_row).expect("click the gear");
    term.wait_for("Settings open", STEP, |s| s.contains("General - global"))
        .unwrap_or_else(|e| panic!("{e}"));
    // The terminology row heads the Integrations page.
    crate::common::settings_page(&mut term, "Integrations");
    let screen = term
        .wait_for("the terminology row", STEP, |s| {
            crate::common::choice_at(s, "Auto (per host)").is_some()
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(
        screen.contains("Review request terminology")
            && crate::common::choice_at(&screen, "Auto (per host)").is_some(),
        "{}",
        screen.text()
    );
    let (col, row) = crate::common::choice_at(&screen, "Auto (per host)").unwrap();
    term.click(col + 1, row).expect("open the combo");
    let screen = term
        .wait_for("the popup lists the three options", STEP, |s| {
            s.contains("\u{2713} Auto (per host)")
                && s.contains("Merge request")
                && s.contains("Pull request")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let on_disk = || std::fs::read_to_string(config.path().join("config.toml")).unwrap_or_default();
    assert!(
        on_disk().contains("pr_terminology = \"auto\""),
        "opening must not change it: {}",
        on_disk()
    );
    // The ticked "Auto" is the popup's first row, "Pull Request" the next.
    let (pc, tick_row) = screen.find("\u{2713} Auto (per host)").unwrap();
    let pr = tick_row + 1;
    assert!(
        screen.line(pr).contains("Pull request"),
        "{}",
        screen.text()
    );
    term.click(pc + 2, pr).expect("pick Pull request");
    let deadline = std::time::Instant::now() + STEP;
    while !on_disk().contains("pr_terminology = \"pull_request\"") {
        assert!(
            std::time::Instant::now() < deadline,
            "never stored: {}",
            on_disk()
        );
        std::thread::sleep(Duration::from_millis(100));
    }
    term.send_keys("Escape").unwrap();
    term.wait_for("the window says Pull Requests now", STEP, |s| {
        s.contains("PULL REQUESTS (2)") && !s.contains("MERGE REQUESTS")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.screen().text()));
}
