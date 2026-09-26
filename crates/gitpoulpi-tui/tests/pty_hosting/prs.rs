use crate::mock::*;
use crate::support::*;

// -- tests -----------------------------------------------------------------------

/// The layout of the pull-request panel: a header with a `+`, a
/// search box, and collapsible groups, each with its count.
#[test]
fn the_pull_request_section_has_a_plus_a_search_and_four_counted_groups() {
    let mock = start_mock();
    let repo = fixture(mock.port);
    let (mut term, _config) = launch(
        &repo,
        "pty-hosting-layout",
        &config_toml(mock.port, ""),
        COLS,
        "Add the feature",
    );
    let screen = term
        .wait_for("the PULL REQUESTS section", STARTUP, pulls_ready)
        .unwrap_or_else(|e| panic!("{e}"));

    // The `+` sits on the header row, at its right.
    let (_, header_row) = screen.find("PULL REQUESTS (2)").unwrap();
    let header = screen.line(header_row);
    assert!(header.contains(" + "), "the + button: {header:?}");

    // "My" holds the one PR kai authored, "Assigned" the one assigned to
    // him, "Awaiting" none; "All" (folded) counts both.
    let count = |label: &str| -> String {
        let (_, row) = screen.find(label).unwrap();
        let line = screen.line(row);
        let line = line.trim_end_matches([' ', '\u{2502}']);
        line.rsplit(' ').next().unwrap().to_string()
    };
    assert_eq!(count("My Pull Requests"), "1", "{}", screen.text());
    assert_eq!(count("Assigned to me"), "1");
    assert_eq!(count("Awaiting my review"), "0");
    assert_eq!(count("All Pull Requests"), "2");
    assert!(screen.contains("!53"), "kai's PR is listed under My");

    // A group folds and unfolds with a click on its header.
    let (col, row) = screen.find("My Pull Requests").unwrap();
    term.click(col + 2, row).expect("click the group");
    term.wait_for("the My group folded", STEP, |s| {
        s.contains("My Pull Requests") && !s.contains("!53  Add the parser")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    let (col, row) = term.wait_for_text("My Pull Requests", STEP).unwrap();
    term.click(col + 2, row).expect("click the group again");
    term.wait_for("the My group open again", STEP, |s| {
        s.contains("!53  Add the parser")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    // The search box narrows the groups: click it, type, Enter leaves.
    let (col, row) = term.wait_for_text("Search pull requests", STEP).unwrap();
    term.click(col + 2, row).expect("click the search box");
    term.send_text("docs").unwrap();
    term.wait_for("only the docs PR matches", STEP, |s| {
        s.contains("!54  Docs") && !s.contains("!53  Add")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    term.wait_for("the box left, the filter kept", STEP, |s| {
        s.contains("docs") && s.contains("!54  Docs") && !s.contains("!53  Add")
    })
    .unwrap_or_else(|e| panic!("{e}"));
}

/// Pressing the PR / issue filter several times must not make it disappear:
/// it is a combobox row, always there, by mouse and by keyboard, and every
/// pick asks the host again.
#[test]
fn the_pull_request_filter_is_still_there_after_many_presses() {
    let mock = start_mock();
    let repo = fixture(mock.port);
    // A narrow terminal too: the chip must not be dropped when it does not fit.
    let (mut term, _config) = launch(
        &repo,
        "pty-hosting-filter",
        &config_toml(mock.port, ""),
        110,
        "Add the feature",
    );
    term.wait_for("the PULL REQUESTS section", STARTUP, pulls_ready_narrow)
        .unwrap_or_else(|e| panic!("{e}"));

    let labels = ["Merged / closed", "All", "Open"];
    // By mouse: click the filter row (the row under the search box).
    for round in 0..6 {
        let screen = term.screen();
        let (_, search_row) = screen.find("Search pull").unwrap();
        let filter_row = search_row + 1;
        let line = screen.line(filter_row);
        assert!(
            line.contains('\u{25BE}'),
            "round {round}: the filter row has its arrow: {line:?}\n{}",
            screen.text()
        );
        let (col, _) = screen.find("PULL REQUESTS").unwrap();
        term.click(col + 4, filter_row).expect("click the filter");
        let want = labels[round % 3];
        let opened = term
            .wait_for("the filter dropdown", STEP, |s| {
                // The popup: Open / Merged / closed / All on three rows.
                s.find_all("Merged / closed").into_iter().any(|(_, r)| {
                    r > filter_row
                        && s.line(r - 1).contains("Open")
                        && s.line(r + 1).contains("All")
                })
            })
            .unwrap_or_else(|e| panic!("round {round}: {e}"));
        // Click the choice: the popup row that says exactly `want`.
        let (pc, pr) = opened
            .find_all(want)
            .into_iter()
            .find(|(_, r)| *r > filter_row)
            .unwrap_or_else(|| panic!("round {round}: {want} not in the popup\n{}", opened.text()));

        term.click(pc + 2, pr).expect("pick a choice");
        term.wait_for("the dropdown closed with the choice shown", STEP, |s| {
            pulls_ready_narrow(s)
                && !(s.contains("Merged / closed") && s.find_all("Merged / closed").len() >= 2)
                && s.line(filter_row).contains(want.split(' ').next().unwrap())
        })
        .unwrap_or_else(|e| panic!("round {round}: {e}"));
    }
    assert!(mock.saw("GET", "state=opened"), "{:?}", mock.requests());
    assert!(mock.saw("GET", "state=all"), "{:?}", mock.requests());

    // By keyboard: select a row of the section, `v` opens the dropdown,
    // Down + Enter picks; five times over.
    let (col, row) = term.wait_for_text("!53  Add", STEP).unwrap();
    term.click(col + 2, row).expect("select a pull request row");
    for round in 0..5 {
        term.send_keys("v").unwrap();
        term.wait_for("the dropdown", STEP, |s| s.contains("Merged / closed"))
            .unwrap_or_else(|e| panic!("keyboard round {round}: {e}"));
        term.send_keys("Down").unwrap();
        term.send_keys("Enter").unwrap();
        term.wait_for("the section intact", STEP, |s| {
            pulls_ready_narrow(s) && s.find_all("Merged / closed").len() <= 1
        })
        .unwrap_or_else(|e| panic!("keyboard round {round}: {e}"));
        let screen = term.screen();
        let (_, search_row) = screen.find("Search pull").unwrap();
        assert!(
            screen.line(search_row + 1).contains('\u{25BE}'),
            "keyboard round {round}: {}",
            screen.text()
        );
    }
}

/// The `+` of the PULL REQUESTS and REMOTE headers must stay reachable
/// when the sidebar is too narrow to hold it beside the title: hovering
/// the header draws it over the title's end, and clicking it works.
#[test]
fn the_header_plus_buttons_stay_reachable_in_a_narrow_sidebar() {
    let mock = start_mock();
    let repo = fixture(mock.port);
    let (mut term, _config) = launch(
        &repo,
        "pty-hosting-narrow-plus",
        &config_toml(mock.port, ""),
        110,
        "Add the feature",
    );
    term.wait_for("both headers", STARTUP, |s| {
        s.contains("PULL REQU") && s.contains("REMOTE")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    let (plus, row) = hover_plus(&mut term, "PULL REQU");
    term.click(plus, row).expect("click the +");
    term.wait_for("the Create Pull Request form", STEP, |s| {
        s.contains("From repo") && s.contains("Submit as draft")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Escape").unwrap();
    term.wait_for("the form closed", STEP, |s| !s.contains("From repo"))
        .unwrap_or_else(|e| panic!("{e}"));

    let (plus, row) = hover_plus(&mut term, "REMOTE");
    term.click(plus, row).expect("click the remote +");
    term.wait_for("the Add remote dialog", STEP, |s| {
        s.contains("Add remote") || s.contains("Add remote")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.screen().text()));
    term.stop().unwrap();
}
