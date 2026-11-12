//! Slice M end to end: an issue's row menu opens the metadata picker, the
//! choice goes to the host (the shared `xtask mock-host`, which keeps edits
//! in memory) and the ISSUES list shows it after the refresh.

use std::time::Duration;

use crate::support::*;
use xtask::mock_host::{self, Options};

fn wait_until(what: &str, mut ok: impl FnMut() -> bool) {
    let deadline = std::time::Instant::now() + STEP;
    while !ok() {
        assert!(std::time::Instant::now() < deadline, "timed out: {what}");
        std::thread::sleep(Duration::from_millis(50));
    }
}

#[test]
fn editing_an_issues_labels_through_the_picker_reaches_the_host_and_the_list() {
    let mock = mock_host::start(Options::default()).expect("the mock host");
    let repo = fixture(mock.port);
    let (mut term, _config) = launch_issues(
        &repo,
        "pty-hosting-meta-edit",
        &config_toml(mock.port, ""),
        COLS,
        "Add the feature",
    );
    let screen = term
        .wait_for("the issues listed", STARTUP, |s| {
            s.contains("Orbit chart flicke") && s.contains("Moons of Jupiter")
        })
        .unwrap_or_else(|e| panic!("{e}"));

    // The row menu offers the metadata entries (GitLab here).
    let (c, r) = screen.find("Orbit chart flicke").unwrap();
    term.right_click(c + 2, r).expect("the row menu");
    let screen = term
        .wait_for("the row menu", STEP, |s| {
            s.contains("Edit assignees...")
                && s.contains("Edit labels...")
                && s.contains("Set milestone...")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(
        !screen.contains("Edit reviewers..."),
        "an issue has no reviewers:\n{}",
        screen.text()
    );
    let (c, r) = screen.find("Edit labels...").unwrap();
    term.click(c + 1, r).expect("Edit labels");

    // The picker loads the project's labels and checks the issue's own.
    let screen = term
        .wait_for("the label picker", STEP, |s| {
            s.contains("Edit labels")
                && s.contains("needs-design")
                && s.contains("good first issue")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(screen.contains("[x]"), "bug is checked:\n{}", screen.text());
    let (c, r) = screen.find("needs-design").unwrap();
    term.click(c + 1, r).expect("check needs-design");
    term.wait_for("needs-design checked", STEP, |s| {
        s.line(s.find("needs-design").unwrap().1).contains("[x]")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    // Enter applies.
    term.send_keys("Enter").expect("apply");

    wait_until("the host got the edit", || {
        mock.requests().iter().any(|r| {
            r.method == "PUT"
                && r.target.contains("/issues/209")
                && serde_json::from_str::<serde_json::Value>(&r.body)
                    .map(|b| b["labels"] == "bug,needs-design")
                    .unwrap_or(false)
        })
    });
    term.wait_for("the picker closed", STEP, |s| !s.contains("selected"))
        .unwrap_or_else(|e| panic!("{e}"));

    // After the refresh the list carries the label: searching for it finds
    // this issue and no other.
    // A click that lands while the list is still refreshing is swallowed
    // (the box stayed on its placeholder for 30 s under load): repeat it, with
    // the box located afresh, until the box shows its cursor. Focusing is
    // idempotent; the typing below is not, so it waits for this.
    term.act_until(
        "the search box focused",
        Duration::from_secs(3),
        STEP,
        |t| {
            let (c, r) = t.wait_for_text("Search issues", STEP)?;
            t.click(c + 1, r)
        },
        |s| {
            s.find("\u{2588}")
                .is_some_and(|(_, r)| s.line(r).contains('\u{2315}'))
        },
    )
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_text("needs-design").expect("type the label");
    // (The status line also names the issue, with one space after `#209`:
    // the row has two.)
    term.wait_for("only the edited issue matches", STEP, |s| {
        s.contains("#209  Orbit") && !s.contains("#214  Moons")
    })
    .unwrap_or_else(|e| panic!("{e}"));
}

#[test]
fn the_palette_twin_lists_the_issues_and_escape_leaves_the_picker_without_sending() {
    let mock = mock_host::start(Options::default()).expect("the mock host");
    let repo = fixture(mock.port);
    let (mut term, _config) = launch_issues(
        &repo,
        "pty-hosting-meta-palette",
        &config_toml(mock.port, ""),
        COLS,
        "Add the feature",
    );
    term.wait_for("the issues listed", STARTUP, |s| {
        s.contains("Orbit chart flicke")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("C-p").expect("the palette");
    term.wait_for("the palette", STEP, |s| s.contains("Command Palette"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_text("Edit Assignees").expect("filter");
    term.wait_for("the command", STEP, |s| {
        s.contains("Hosting: Edit Assignees...")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").expect("run it");
    // The twin asks which item first.
    // The chooser's row: the copy right under its title (the sidebar lists
    // the same issue further down, and a click there is a click away).
    let chooser_row = |s: &xtask::pty::Screen| {
        let (_, title) = s.find("Choose a request or an issue")?;
        s.find_all("Orbit chart flicke")
            .into_iter()
            .find(|&(_, r)| r > title && r <= title + 4)
    };
    term.wait_for("the item list", STEP, |s| chooser_row(s).is_some())
        .unwrap_or_else(|e| panic!("{e}"));
    // A click on a torn frame can be lost: retry until the picker shows.
    let screen = crate::common::retry_until(
        &mut term,
        "the people picker",
        6,
        Duration::from_secs(5),
        |t| {
            let screen = t.screen();
            if screen.contains("Choose a request or an issue") {
                if let Some((c, r)) = chooser_row(&screen) {
                    t.click(c + 1, r).expect("choose the issue");
                }
            }
        },
        |s| {
            s.contains("Edit assignees")
                && s.contains("lena")
                && s.contains("Lena Hart")
                && (s.contains("AL") || s.contains("LH"))
        },
    );
    let _ = screen;
    // Filter, then leave: nothing is sent.
    term.send_text("tom").expect("filter");
    term.wait_for("filtered", STEP, |s| {
        s.contains("tomas") && !s.contains("lena")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Escape").expect("cancel");
    term.wait_for("closed", STEP, |s| {
        !s.contains("Lena Hart") && !s.contains("selected")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert!(
        !mock.requests().iter().any(|r| r.method == "PUT"),
        "{:?}",
        mock.requests()
    );
}
