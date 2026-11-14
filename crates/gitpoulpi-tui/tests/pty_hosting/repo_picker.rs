use std::time::Duration;

use crate::common;
use crate::mock::*;
use crate::support::*;

/// The Clone dialog lists a connected account's repositories, searchable,
/// and choosing one fills the clone.
#[test]
fn the_clone_dialog_lists_a_connected_accounts_repositories() {
    let mock = start_mock();
    let repo = fixture(mock.port);
    let (mut term, _config) = launch(
        &repo,
        "pty-hosting-clone",
        &config_toml(mock.port, ""),
        COLS,
        "Add the feature",
    );
    term.send_keys("C-o").expect("the management tab");
    term.wait_for("the management tab", STEP, |s| {
        s.contains("Repository Management") && s.contains("Clone")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("c").expect("Clone");
    let dialog = term
        .wait_for("the Clone dialog with the connected profile", STEP, |s| {
            s.contains("Clone a repository")
                && s.contains("Clone with URL")
                && s.contains("GitLab \u{00B7} corp")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(!dialog.contains("GitHub.com"), "{}", dialog.text());

    // Pick the account by mouse: its repositories load.
    let (col, row) = dialog.find("GitLab \u{00B7} corp").unwrap();
    term.click(col + 2, row).expect("pick the account");
    term.wait_for("the repositories listed", STEP, |s| {
        s.contains("g/p") && s.contains("team/tool") && s.contains("Repository")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    // Search narrows the list (the search box has the focus after a click).
    let (col, row) = term.wait_for_text("search repositories", STEP).unwrap();
    term.click(col + 2, row).expect("focus the search");
    term.send_text("tool").unwrap();
    term.wait_for("only team/tool", STEP, |s| {
        s.contains("team/tool") && !s.contains("the main one")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    // Keyboard: Down + Enter chooses the highlighted row (the first);
    // the full path then names the repository.
    term.send_keys("Enter").unwrap();
    term.wait_for("the choice filled the clone", STEP, |s| {
        s.contains("Full path:") && s.contains("tool") && s.contains("\u{2713}")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert!(
        mock.saw("GET", "/api/v4/projects"),
        "the account's projects were requested: {:?}",
        mock.requests()
    );
}

/// The hosted list is one section per owner with a header row, a lock on
/// private repositories, the search narrows across sections, and Up/Down
/// never stop on a header.
#[test]
fn the_clone_list_is_grouped_by_owner_with_a_lock_on_private_repositories() {
    let mock = start_mock();
    let repo = fixture(mock.port);
    let (mut term, _config) = launch(
        &repo,
        "pty-hosting-clone-groups",
        &config_toml(mock.port, ""),
        COLS,
        "Add the feature",
    );
    term.send_keys("C-o").expect("the management tab");
    term.wait_for("the management tab", STEP, |s| {
        s.contains("Repository Management") && s.contains("Clone")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("c").expect("Clone");
    let dialog = term
        .wait_for("the Clone dialog", STEP, |s| {
            s.contains("Clone a repository") && s.contains("GitLab \u{00B7} corp")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let (col, row) = dialog.find("GitLab \u{00B7} corp").unwrap();
    term.click(col + 2, row).expect("pick the account");
    let listed = term
        .wait_for("two owner sections", STEP, |s| {
            s.contains("g (1)") && s.contains("team (1)") && s.contains("team/tool")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let (_, header_g) = listed.find("g (1)").unwrap();
    let (_, header_team) = listed.find("team (1)").unwrap();
    let (_, row_gp) = listed.find("g/p").unwrap();
    let (_, row_tool) = listed.find("team/tool").unwrap();
    assert_eq!(
        (header_g + 1, header_team),
        (row_gp, row_tool - 1),
        "{}",
        listed.text()
    );
    assert!(
        listed.line(row_gp).contains('\u{1F512}'),
        "lock on the private repository: {}",
        listed.text()
    );
    assert!(
        !listed.line(row_tool).contains('\u{1F512}'),
        "no lock on the public one: {}",
        listed.text()
    );

    // The search narrows across sections: the empty section and its header go.
    let (col, row) = term.wait_for_text("search repositories", STEP).unwrap();
    term.click(col + 2, row).expect("focus the search");
    term.send_text("tool").unwrap();
    term.wait_for("only the team section", STEP, |s| {
        s.contains("team (1)") && !s.contains("g (1)") && !s.contains("g/p")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("C-u").unwrap();
    term.wait_for("both sections again", STEP, |s| {
        s.contains("g (1)") && s.contains("team (1)")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    // Down from the first repository lands on the next *repository*, not on
    // the `team` header between them.
    term.send_keys("Down").unwrap();
    term.send_keys("Enter").unwrap();
    term.wait_for("team/tool chosen", STEP, |s| {
        s.contains("\u{2713} team/tool") && s.contains("Full path:")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.stop().ok();
}

/// The repository combobox scrolls over every project the host lists
/// (two API pages here): by keyboard (End) and by mouse wheel.
#[test]
fn the_repository_dropdown_scrolls_past_its_first_page_by_keyboard_and_wheel() {
    let mock = start_mock();
    EXTRA_PROJECTS.lock().unwrap().push((mock.port, 130));
    let repo = fixture(mock.port);
    let (mut term, _config) = launch_issues(
        &repo,
        "pty-hosting-scroll",
        &config_toml(mock.port, ""),
        COLS,
        "Add the feature",
    );
    let repo_row = open_repo_dropdown(&mut term);
    term.wait_for("the projects listed", STEP, |s| s.contains("many/repo00"))
        .unwrap_or_else(|e| panic!("{e}"));
    let screen = term.screen();
    assert!(
        !screen.contains("many/repo129"),
        "the last project is beyond the first rows: {}",
        screen.text()
    );

    // Keyboard: End jumps to the last of the 132, Home comes back.
    term.send_keys("End").unwrap();
    term.wait_for("the last project visible", STEP, |s| {
        s.contains("many/repo129")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("PageUp").unwrap();
    term.send_keys("Home").unwrap();
    term.wait_for("the top again", STEP, |s| {
        s.contains("team/tool") && !s.contains("many/repo129")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("End").unwrap();
    term.send_keys("Enter").unwrap();
    term.wait_for("the last project chosen", STEP, |s| {
        s.line(repo_row).contains("many/repo129")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    let deadline = std::time::Instant::now() + STEP;
    while !mock.saw("GET", "many%2Frepo129/issues") {
        assert!(
            std::time::Instant::now() < deadline,
            "{:?}",
            mock.requests()
        );
        std::thread::sleep(Duration::from_millis(50));
    }

    // Mouse wheel: reopen and scroll down over the popup until the end
    // of the second page shows, then click a row there.
    let (col, _) = term.screen().find("GITLAB ISSUES").unwrap();
    term.click(col + 4, repo_row).expect("reopen");
    // Reopened, the highlight is on the current choice: the end of the list.
    // (The field itself shows the choice: look for a row below it.)
    term.wait_for("the popup", STEP, |s| {
        s.find_all("many/repo129")
            .iter()
            .any(|(_, r)| *r > repo_row)
    })
    .unwrap_or_else(|e| panic!("{e}"));
    let (pc, pr) = term
        .screen()
        .find_all("many/repo129")
        .into_iter()
        .find(|(_, r)| *r > repo_row)
        .expect("repo129 in the popup");
    // Wheel bursts can be coalesced or dropped while the TUI is busy, so
    // each burst is repeated until the list has actually got there.
    common::retry_until(
        &mut term,
        "the wheel reached the top",
        6,
        Duration::from_secs(5),
        |t| {
            for _ in 0..70 {
                t.scroll(false, pc + 2, pr).unwrap();
            }
        },
        |s| s.contains("team/tool"),
    );
    common::retry_until(
        &mut term,
        "the wheel reached the end",
        6,
        Duration::from_secs(5),
        |t| {
            for _ in 0..70 {
                t.scroll(true, pc + 2, pr).unwrap();
            }
        },
        |s| s.contains("many/repo128") && !s.contains("team/tool"),
    );
    // The wheel events still queued keep moving the list for a moment: take
    // the row once it has held still, and click it afresh on every attempt
    // (a double click is two events the TUI times itself).
    term.act_until(
        "repo128 chosen",
        Duration::from_secs(4),
        STEP,
        |t| {
            // Gone already (a slow success): nothing to click again.
            match t.wait_for_settled_text(
                "many/repo128",
                Duration::from_millis(150),
                Duration::from_secs(5),
            ) {
                Ok((pc, pr)) => t.double_click(pc + 2, pr),
                Err(_) => Ok(()),
            }
        },
        |s| s.line(repo_row).contains("many/repo128"),
    )
    .unwrap_or_else(|e| panic!("{e}"));
    term.stop().ok();
}

/// The right ends of the search field and the combo under it line up:
/// the magnifier and the arrow sit in the same column, and the field
/// backgrounds both stop at the same column.
#[test]
fn the_search_field_and_the_combo_end_in_the_same_column() {
    let mock = start_mock();
    let repo = fixture(mock.port);
    let (term, _config) = launch(
        &repo,
        "pty-hosting-align",
        &config_toml(mock.port, ""),
        COLS,
        "Add the feature",
    );
    let screen = term
        .wait_for("the PULL REQUESTS section", STARTUP, pulls_ready)
        .unwrap_or_else(|e| panic!("{e}"));
    let (_, search_row) = screen.find("Search pull requests").unwrap();
    let filter_row = search_row + 1;
    let col_of = |row: u16, glyph: char| -> u16 {
        let line = screen.line(row);
        let idx = line
            .chars()
            .position(|c| c == glyph)
            .unwrap_or_else(|| panic!("{glyph} missing in {line:?}"));
        idx as u16
    };
    let mag = col_of(search_row, '\u{2315}');
    let arrow = col_of(filter_row, '\u{25BE}');
    assert_eq!(mag, arrow, "magnifier and arrow in one column");
    // Same first and last field-background column on both rows.
    let bg_span = |row: u16| -> (u16, u16) {
        let cols: Vec<u16> = (0..screen.cols())
            .filter(|c| {
                screen
                    .cell(*c, row)
                    .is_some_and(|x| x.bg == xtask::pty::Color::Rgb(42, 42, 46))
            })
            .collect();
        (*cols.first().unwrap(), *cols.last().unwrap())
    };
    assert_eq!(
        bg_span(search_row),
        bg_span(filter_row),
        "{}",
        screen.text()
    );
    term.stop().unwrap();
}

/// AI only ever starts on an explicit action. The To repo popup lies over
/// the "Generate title and description" button; picking a row from it --
/// even with a double click, whose second press lands on the button once the
/// popup has closed - must never reach the agent.
#[test]
fn picking_from_a_combo_popup_over_the_generate_button_never_starts_ai() {
    use gitpoulpi_testkit::fake_agent::{agent_calls, ai_config_toml, write_fake_agent};
    let mock = start_mock();
    let repo = fixture(mock.port);
    let tools = tempfile::tempdir().unwrap();
    let agent = write_fake_agent(tools.path(), &["Title", "", "Body"]);
    let (mut term, _config) = launch(
        &repo,
        "pty-hosting-ai-popup",
        &config_toml(mock.port, &format!("\n{}", ai_config_toml(&agent))),
        COLS,
        "Add the feature",
    );
    term.wait_for("the PULL REQUESTS section", STARTUP, pulls_ready)
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("C-p").unwrap();
    term.send_text("create pull request").unwrap();
    term.wait_for("the entry offered", STEP, |s| {
        s.contains("Create Pull Request")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    let screen = term
        .wait_for("the form with its Generate button", STEP, |s| {
            s.contains("Generate title and description") && s.contains("To repo")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let (gx, gy) = screen.find("Generate title and description").unwrap();
    // The To repo combo is four rows above the Title label.
    term.click(gx + 4, gy - 4).expect("open the To repo popup");
    // The host's repositories are listed, and a row lies over the button.
    term.wait_for("the popup drawn over the button", STEP, |s| {
        !s.line(gy).contains("Generate title and description")
            && s.contains("type your own value")
            && s.line(gy).contains("team/tool")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.double_click(gx + 4, gy).expect("pick a row twice");
    term.wait_for("the popup closed, the button back", STEP, |s| {
        s.contains("Generate title and description")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.wait_for_stable(Duration::from_millis(1000), STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(
        agent_calls(&agent),
        0,
        "no AI call without an explicit click"
    );
}
