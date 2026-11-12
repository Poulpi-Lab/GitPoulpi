use std::time::Duration;

use crate::common;
use crate::mock::*;
use crate::support::*;

// -- New issue (the ISSUES header's `+`) -----------------------------------------

/// The ISSUES `+` is there only once a provider and repository are chosen,
/// names its action when hovered, and opens a form that creates the issue
/// through the host; the section re-fetches afterwards.
#[test]
fn the_issues_plus_creates_an_issue_through_the_host_and_refreshes_the_section() {
    let mock = start_mock();
    let repo = fixture(mock.port);
    let (mut term, _config) = launch_issues(
        &repo,
        "pty-hosting-new-issue",
        &config_toml(mock.port, ""),
        COLS,
        "Add the feature",
    );
    term.wait_for("the ISSUES section", STARTUP, |s| {
        s.contains("ISSUES (1)") && s.contains("Crash on startup")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    let (plus, row) = hover_plus(&mut term, "ISSUES");
    // Hovering the `+` itself makes it name its action.
    term.move_to(plus, row).expect("hover the +");
    term.wait_for("the tooltip names the action", STEP, |s| {
        s.contains("+ New issue")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.screen().text()));
    // The tooltip grows leftwards from the same corner: the old glyph's
    // cell is still inside the button.
    // A click on a torn frame can be lost: retry until the form shows.
    common::retry_until(
        &mut term,
        "the New issue form",
        6,
        Duration::from_secs(5),
        |t| {
            if !t.screen().contains("Labels: none") {
                t.click(plus, row).expect("click the +");
            }
        },
        |s| s.contains("New issue") && s.contains("Labels: none"),
    );
    assert!(term.screen().contains("g/p"), "{}", term.screen().text());

    term.send_text("Parser crashes").unwrap();
    term.send_keys("Tab").unwrap();
    term.send_text("line one").unwrap();
    term.send_keys("Enter").unwrap();
    term.send_text("line two").unwrap();
    term.wait_for("everything typed", STEP, |s| {
        s.contains("Parser crashes") && s.contains("line two")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    // GitLab: the labels come from the project's own, in the picker.
    common::retry_until(
        &mut term,
        "the label picker",
        6,
        Duration::from_secs(5),
        |t| {
            let screen = t.screen();
            if screen.contains("Edit labels") {
                return;
            }
            if let Some((lc, lr)) = screen.find("Labels: none") {
                t.click(lc + 2, lr).expect("open the label picker");
            }
        },
        |s| s.contains("Edit labels") && s.contains("Broken") && s.contains("ui"),
    );
    // Each check is retried until its count shows (a click on a torn frame
    // can be lost); nothing is clicked once the count holds.
    for (row_text, count) in [("bug", "1 selected"), ("ui", "2 selected")] {
        common::retry_until(
            &mut term,
            count,
            6,
            Duration::from_secs(5),
            |t| {
                let screen = t.screen();
                if screen.contains(count) {
                    return;
                }
                let Some((_, picker)) = screen.find("Edit labels") else {
                    return;
                };
                if let Some((c, r)) = screen
                    .find_all(row_text)
                    .into_iter()
                    .find(|(_, r)| *r > picker)
                {
                    t.click(c + 1, r).expect("check the label");
                }
            },
            |s| s.contains(count),
        );
    }
    term.send_keys("Enter").expect("apply");
    term.wait_for("the chip shows the choice", STEP, |s| {
        s.contains("Labels: bug +1") && !s.contains("Edit labels")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.screen().text()));

    let before = host_list_requests(&mock);
    // The Create button, by mouse (the last "Create issue" on screen).
    common::retry_until(
        &mut term,
        "the form closed with the result",
        6,
        Duration::from_secs(5),
        |t| {
            let screen = t.screen();
            if !screen.contains("Labels: bug +1") {
                return;
            }
            if let Some(&(bc, br)) = screen.find_all("Create issue").last() {
                t.click(bc + 2, br).expect("click Create issue");
            }
        },
        |s| !s.contains("Labels: none") && s.contains("created issue #13"),
    );

    let post = mock
        .requests()
        .into_iter()
        .find(|r| r.method == "POST" && r.target.contains("/issues"))
        .expect("the host was asked to create the issue");
    assert!(
        post.target.contains("/projects/g%2Fp/issues"),
        "{}",
        post.target
    );
    let body: serde_json::Value = serde_json::from_str(&post.body).expect("a JSON body");
    assert_eq!(body["title"], "Parser crashes");
    assert_eq!(body["description"], "line one\nline two");
    assert_eq!(body["labels"], "bug,ui");

    // The ISSUES section asks the host again.
    let deadline = std::time::Instant::now() + STEP;
    while host_list_requests(&mock) <= before && std::time::Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(100));
    }
    assert!(
        host_list_requests(&mock) > before,
        "no re-fetch after create"
    );
    term.stop().unwrap();
}

/// Keyboard and palette twins: `n` on the ISSUES section and the palette's
/// `Hosting: New issue` open the form; Esc closes it; Ctrl+S creates. A
/// repository whose ISSUES section is unlinked has no `+` and says why.
#[test]
fn the_new_issue_form_has_a_keyboard_and_a_palette_twin_and_needs_a_link() {
    let mock = start_mock();
    let repo = fixture(mock.port);
    let (mut term, _config) = launch_issues(
        &repo,
        "pty-hosting-new-issue-keys",
        &config_toml(mock.port, ""),
        COLS,
        "Add the feature",
    );
    term.wait_for("the ISSUES section", STARTUP, |s| {
        s.contains("ISSUES (1)") && s.contains("Crash on startup")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    let (col, row) = term.wait_for_text("Crash on startup", STEP).unwrap();
    term.click(col + 2, row).expect("focus the section");
    term.send_keys("n").unwrap();
    term.wait_for("the form from `n`", STEP, |s| {
        s.contains("New issue") && s.contains("Labels: none")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Escape").unwrap();
    term.wait_for_text_to_go("Labels: none", STEP)
        .unwrap_or_else(|e| panic!("{e}"));

    term.send_keys("C-p").unwrap();
    term.wait_for("the palette", STEP, |s| {
        s.contains("Command Palette") || s.contains("Type a command")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_text("New issue").unwrap();
    term.wait_for("the command listed", STEP, |s| {
        s.contains("Hosting: New Issue")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    term.wait_for("the form from the palette", STEP, |s| {
        s.contains("New issue") && s.contains("Labels: none")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_text("By keyboard").unwrap();
    term.send_keys("C-s").unwrap();
    term.wait_for("created", STEP, |s| {
        !s.contains("Labels: none") && s.contains("created issue #13")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    let post = mock
        .requests()
        .into_iter()
        .find(|r| r.method == "POST" && r.target.contains("/issues"))
        .expect("a POST");
    let body: serde_json::Value = serde_json::from_str(&post.body).unwrap();
    assert_eq!(body["title"], "By keyboard");
    assert!(body.get("labels").is_none(), "{body}");
    term.stop().unwrap();

    // Unlinked: no `+` on the header.
    let mock = start_mock();
    let repo = fixture(mock.port);
    let (term, _config) = launch_unlinked(
        &repo,
        "pty-hosting-new-issue-unlinked",
        &config_toml(mock.port, ""),
        COLS,
        "Add the feature",
    );
    let screen = term
        .wait_for("the ISSUES section", STARTUP, |s| s.contains("ISSUES"))
        .unwrap_or_else(|e| panic!("{e}"));
    let (_, row) = screen
        .find_all("ISSUES")
        .into_iter()
        .find(|&(c, _)| c < 40)
        .unwrap();
    assert!(
        !screen.line(row).contains(" + "),
        "no + while unlinked: {}",
        screen.line(row)
    );
    term.stop().unwrap();
}

/// "Create on GitLab": the button and its key open the provider's own new
/// issue form for the linked project, prefilled with what was typed, with
/// no API call; a hover names it; it is still there after an API refusal.
#[test]
fn create_on_the_provider_opens_the_prefilled_issue_form_by_button_and_key() {
    let mock = start_mock();
    let repo = fixture(mock.port);
    let (mut term, config) = launch_issues(
        &repo,
        "pty-hosting-issue-web",
        &config_toml(mock.port, ""),
        COLS,
        "Add the feature",
    );
    term.wait_for("the ISSUES section", STARTUP, |s| {
        s.contains("ISSUES (1)") && s.contains("Crash on startup")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    let (col, row) = term.wait_for_text("Crash on startup", STEP).unwrap();
    term.click(col + 2, row).expect("focus the section");
    term.send_keys("n").unwrap();
    term.wait_for("the form", STEP, |s| {
        s.contains("New issue") && s.contains("Create on GitLab") && s.contains("Alt+W")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.screen().text()));
    term.send_text("Parser & crashes").unwrap();
    term.send_keys("Tab").unwrap();
    term.send_text("line one").unwrap();

    // Hover: the tooltip.
    let (bc, br) = term.wait_for_text("Create on GitLab", STEP).unwrap();
    term.move_to(bc + 2, br).unwrap();
    term.wait_for("the tooltip", STEP, |s| {
        s.contains("in the browser, prefilled")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.screen().text()));

    let base = format!("http://127.0.0.1:{}/g/p/-/issues/new?", mock.port);
    let want =
        format!("{base}issue%5Btitle%5D=Parser%20%26%20crashes&issue%5Bdescription%5D=line%20one");
    // The key.
    term.send_keys("M-w").unwrap();
    assert_eq!(wait_opened(&config, 1), vec![want.clone()]);
    // The button, by mouse.
    term.click(bc + 2, br).unwrap();
    assert_eq!(wait_opened(&config, 2), vec![want.clone(), want]);
    assert!(term.screen().contains("New issue"), "the form stays open");
    assert!(
        !mock.requests().iter().any(|r| r.method == "POST"),
        "no API call"
    );
    term.stop().unwrap();
}
