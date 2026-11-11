//! The pointing hand over the surfaces that own the mouse: the Jobs panel
//! (rows, Cancel buttons) and its status bar indicator, the version chip and
//! the Release notes tab, Settings. A text field takes the I-beam (`text`): the
//! Settings search box, the Refs filter. Each shape is the `OSC 22` the
//! program wrote, read back from the raw output.
//!
//! Real binary through a pty; temp config dirs and fixture repositories.

mod common;

use std::io::{Read, Write};
use std::net::TcpListener;
use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::Terminal;

const STARTUP: Duration = Duration::from_secs(60);
const STEP: Duration = Duration::from_secs(30);

/// Every `OSC 22` pointer shape the program asked for, in order ("" is the
/// reset).
fn pointer_shapes(term: &Terminal) -> Vec<String> {
    let raw = String::from_utf8_lossy(&term.raw_output()).to_string();
    raw.split("\x1b]22;")
        .skip(1)
        .map(|s| s.split("\x1b\\").next().unwrap_or("").to_string())
        .collect()
}

/// Moves onto `(col, row)` and waits for the shape `want` to be the last one
/// requested.
fn hover(term: &mut Terminal, what: &str, (col, row): (u16, u16), want: &str) {
    term.move_to(col, row).unwrap();
    let deadline = std::time::Instant::now() + STEP;
    while pointer_shapes(term).last().map(String::as_str) != Some(want)
        && std::time::Instant::now() < deadline
    {
        std::thread::sleep(Duration::from_millis(50));
    }
    assert_eq!(
        pointer_shapes(term).last().map(String::as_str),
        Some(want),
        "over {what}: {:?}\n{}",
        pointer_shapes(term),
        term.text()
    );
}

fn ahead_fixture() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit("base commit")
        .unwrap()
        .with_origin()
        .unwrap()
        .push("main")
        .unwrap()
        .track("main", "origin")
        .unwrap()
        .commit("ahead commit")
        .unwrap()
        .build()
}

/// A push held running by the daemon's test hook: the indicator, the panel
/// and the row's Cancel are all on screen to hover.
#[test]
fn the_jobs_indicator_the_panel_rows_and_cancel_buttons_show_the_pointing_hand() {
    let repo = ahead_fixture();
    let config = tempfile::tempdir().unwrap();
    let mut term = common::launch(repo.root())
        .session("pty-pointer-jobs")
        .config_dir(config.path())
        .size(160, 40)
        .no_exit_pause()
        .mouse_capture()
        .env("GITPOULPI_TEST_TRANSFER_DELAY_MS", "25000")
        .spawn();
    term.wait_for("the fixture loaded", STARTUP, |s| {
        s.contains("ahead commit")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("C-g").unwrap();
    term.send_keys("P").unwrap();
    term.wait_for("the indicator with the running push", STEP, |s| {
        s.text()
            .lines()
            .any(|l| l.contains("Push: ") && l.contains("[x]"))
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // The indicator's label opens the panel: the hand.
    let (col, row) = term
        .wait_for_text("Push: ", STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    hover(&mut term, "the indicator", (col + 1, row), "pointer");
    term.click(col + 1, row).unwrap();
    // One predicate for the whole panel: a torn frame can show the title
    // before its buttons.
    let screen = term
        .wait_for("the jobs panel", STEP, |s| {
            s.contains("Background jobs")
                && s.contains("Cancel all")
                && s.find_all("Cancel")
                    .into_iter()
                    .any(|(c, r)| s.line(r).contains("Push: ") && c > 60)
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // "Cancel all" is the toolbar's button; the row's own Cancel sits on the
    // row of the job.
    let all = screen.find("Cancel all").expect("Cancel all");
    let row_cancel = screen
        .find_all("Cancel")
        .into_iter()
        .find(|(c, r)| screen.line(*r).contains("Push: ") && *c > 60)
        .expect("the row's Cancel button");
    let job_row = screen
        .find_all("Push: ")
        .into_iter()
        .find(|(_, r)| *r != row)
        .map(|(c, r)| (c + 1, r))
        .expect("the job row");
    // The hand first (the indicator), so the row's arrow is a change that is
    // written: a job row is a list row, only its buttons are hands.
    hover(&mut term, "the row", job_row, "");
    hover(
        &mut term,
        "the row's Cancel",
        (row_cancel.0 + 1, row_cancel.1),
        "pointer",
    );
    hover(&mut term, "Cancel all", (all.0 + 1, all.1), "pointer");
    // Blank space inside the window: the arrow.
    let (bc, br) = (job_row.0, job_row.1 + 8);
    hover(&mut term, "blank space in the panel", (bc, br), "");
    term.stop().unwrap();
}

/// A fake `api.github.com` answering the releases list.
const BODY: &str = r###"[
  {"tag_name": "v99.0.0", "name": "99.0.0", "draft": false, "prerelease": false,
   "body": "## What is new\n- the **shiny** fake feature",
   "html_url": "https://example.test/releases/v99.0.0", "assets": []}
]"###;

fn serve() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind loopback");
    let port = listener.local_addr().unwrap().port();
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { continue };
            let mut buf = [0u8; 4096];
            let _ = stream.read(&mut buf);
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{BODY}",
                BODY.len()
            );
            let _ = stream.write_all(response.as_bytes());
        }
    });
    port
}

/// The version chip and the update panel's Release notes button: the hand;
/// the release text: the arrow. The build under test is a marker'd copy (a
/// development build does not check for updates).
#[test]
fn the_version_chip_and_the_release_notes_buttons_show_the_pointing_hand() {
    use gitpoulpi_config::release::{self, InstallChannel, Marker, MARKER_FILE};
    let port = serve();
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("pointer notes probe")
        .unwrap()
        .build();
    let config = tempfile::tempdir().unwrap();
    let install = tempfile::tempdir().unwrap();
    let exe = install.path().join("gitpoulpi-tui");
    let built = env!("CARGO_BIN_EXE_gitpoulpi-tui");
    if std::fs::hard_link(built, &exe).is_err() {
        std::fs::copy(built, &exe).unwrap();
    }
    for sibling in ["gitpoulpi", "gitpoulpi-mcp"] {
        std::fs::write(install.path().join(sibling), "#!/bin/sh\n").unwrap();
    }
    let mut marker = Marker::new(env!("CARGO_PKG_VERSION"), release::host_target());
    marker.channel = Some(InstallChannel::Archive);
    std::fs::write(install.path().join(MARKER_FILE), marker.render()).unwrap();
    let mut term = common::launch(repo.root())
        .session("pty-pointer-notes")
        .config_dir(config.path())
        .size(200, 50)
        .no_exit_pause()
        .mouse_capture()
        .binary(&exe)
        .env("GITPOULPI_UPDATES_MODE", "notify")
        .env("GITPOULPI_UPDATE_API", format!("http://127.0.0.1:{port}"))
        .spawn();
    term.wait_for("the chip says a newer version is available", STARTUP, |s| {
        s.contains("v99.0.0 available")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (col, row) = term
        .wait_for_text("v99.0.0 available", STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    hover(&mut term, "the version chip", (col + 1, row), "pointer");
    term.click(col + 1, row).unwrap();
    // The chip opens the update panel; its Release notes button leads on.
    let panel = term
        .wait_for("the update panel", STEP, |s| {
            s.contains("Installed with:") && s.contains("Release notes")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let notes_button = panel.find("Release notes").unwrap();
    hover(
        &mut term,
        "the panel's Release notes",
        (notes_button.0 + 1, notes_button.1),
        "pointer",
    );
    term.click(notes_button.0 + 1, notes_button.1).unwrap();
    term.wait_for("the Release notes tab", STEP, |s| {
        s.contains("the shiny fake feature") && s.contains("Skip this version")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // The release text first (arrow), then each button (hand).
    let screen = term.screen();
    let text = screen.find("the shiny fake feature").unwrap();
    let skip = screen.find("Skip this version").unwrap();
    let download = screen.find("Download").unwrap();
    hover(
        &mut term,
        "Download",
        (download.0 + 1, download.1),
        "pointer",
    );
    hover(&mut term, "the release text", (text.0 + 2, text.1), "");
    hover(
        &mut term,
        "Skip this version",
        (skip.0 + 1, skip.1),
        "pointer",
    );
    term.stop().unwrap();
}

/// Settings: a section of the list is a row (the arrow), the search box (a
/// text field) the I-beam.
#[test]
fn settings_show_the_arrow_on_sections_and_the_i_beam_in_the_search_box() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("pointer settings probe")
        .unwrap()
        .build();
    let config = tempfile::tempdir().unwrap();
    let mut term = common::launch(repo.root())
        .session("pty-pointer-settings")
        .config_dir(config.path())
        .size(160, 45)
        .no_exit_pause()
        .mouse_capture()
        .spawn();
    term.wait_for("the window", STARTUP, |s| {
        s.contains("pointer settings probe")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (gear_col, gear_row) = term
        .wait_for_text("\u{2699}", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(gear_col, gear_row).expect("click the gear");
    term.wait_for("Settings open", STEP, |s| {
        s.contains("Repo-specific settings") && s.contains("Search settings")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let screen = term.screen();
    let section = screen.find("Synchronization").expect("a section");
    let search = screen.find("Search settings").expect("the search box");
    hover(
        &mut term,
        "the search box",
        (search.0 + 1, search.1),
        "text",
    );
    // A section of the list is a row: it lights and keeps the arrow.
    hover(&mut term, "a section", (section.0 + 1, section.1), "");
    term.stop().unwrap();
}

/// The Refs filter is a text field (the I-beam), a Log message cell is a
/// plain table cell (the arrow), and only the ref pill of the Ref column is
/// a hand.
#[test]
fn the_refs_filter_is_an_i_beam_log_rows_keep_the_arrow_and_ref_pills_are_hands() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("pointer log probe")
        .unwrap()
        .build();
    let config = tempfile::tempdir().unwrap();
    let mut term = common::launch(repo.root())
        .session("pty-pointer-log-rows")
        .config_dir(config.path())
        .size(180, 45)
        .no_exit_pause()
        .mouse_capture()
        .spawn();
    let screen = term
        .wait_for("the window", STARTUP, |s| {
            s.contains("pointer log probe") && s.contains("(click or press / to filter)")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let filter = screen.find("(click or press / to filter)").unwrap();
    // The Log's row: the one line holding both the ref and the message (the
    // details pane repeats the message lower down).
    let message = screen
        .find_all("pointer log")
        .into_iter()
        .find(|(_, r)| screen.line(*r).contains("main"))
        .expect("the Log row's message cell");
    let pill = screen
        .find_all("main")
        .into_iter()
        .find(|(c, r)| *r == message.1 && *c < message.0)
        .expect("the ref pill on the commit's row");
    hover(
        &mut term,
        "the Refs filter",
        (filter.0 + 1, filter.1),
        "text",
    );
    hover(
        &mut term,
        "a Log message cell",
        (message.0 + 2, message.1),
        "",
    );
    hover(&mut term, "a ref pill", (pill.0 + 1, pill.1), "pointer");
    term.stop().unwrap();
}

/// The file editor: its text is an I-beam, its buttons the hand.
#[test]
fn the_file_editor_text_is_an_i_beam_and_its_buttons_the_hand() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .write_worktree("notes.txt", b"alpha\nbeta\n")
        .unwrap()
        .stage(".")
        .unwrap()
        .commit("base")
        .unwrap()
        .write_worktree("notes.txt", b"alpha\nbeta\ngamma\n")
        .unwrap()
        .build();
    let config = tempfile::tempdir().unwrap();
    let mut term = common::launch(repo.root())
        .no_commit_column()
        .session("pty-pointer-editor")
        .config_dir(config.path())
        .size(200, 40)
        .no_exit_pause()
        .mouse_capture()
        .spawn();
    let (c, r) = term
        .wait_for_text("notes.txt", STARTUP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.click(c, r).unwrap();
    term.wait_for("the unstaged diff", STEP, |s| {
        s.contains("notes.txt - unstaged") && s.contains(" Edit ")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    let (ec, er) = term
        .wait_for_text(" Edit ", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(ec + 2, er).unwrap();
    term.wait_for("the editor", STEP, |s| {
        s.contains("Edit - notes.txt @ working tree") && s.contains("alpha")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    let screen = term.screen();
    let text = screen
        .find_all("beta")
        .into_iter()
        .next()
        .expect("the editor's text");
    let save = screen.find(" Save ").expect("the Save button");
    // The hand first, so the I-beam is a change that is written.
    hover(&mut term, "Save", (save.0 + 2, save.1), "pointer");
    hover(&mut term, "the editor text", (text.0 + 1, text.1), "text");
    hover(&mut term, "Save again", (save.0 + 2, save.1), "pointer");
    term.stop().unwrap();
}

/// The global search: the I-beam over its text field, the hand on its close
/// cross, the arrow on everything else (the toolbar and tabs behind it do
/// not leak through).
#[test]
fn the_global_search_is_an_i_beam_on_its_field_and_an_arrow_elsewhere() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("pointer search probe")
        .unwrap()
        .build();
    let config = tempfile::tempdir().unwrap();
    let mut term = common::launch(repo.root())
        .session("pty-pointer-search")
        .config_dir(config.path())
        .size(180, 45)
        .no_exit_pause()
        .mouse_capture()
        .spawn();
    term.wait_for("the window", STARTUP, |s| {
        s.contains("pointer search probe")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // Ctrl+Alt+F as a terminal sends it: ESC, then Ctrl+F.
    term.send_text("\x1b\x06").unwrap();
    let screen = term
        .wait_for("the popup with its close cross", STEP, |s| {
            s.find("Search open repositories")
                .is_some_and(|(_, tr)| s.find_all("\u{2715}").into_iter().any(|(_, r)| r == tr))
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (title_col, title_row) = screen.find("Search open repositories").unwrap();
    let cross = screen
        .find_all("\u{2715}")
        .into_iter()
        .find(|(_, r)| *r == title_row)
        .expect("the close cross");
    hover(
        &mut term,
        "the text field",
        (title_col + 3, title_row + 1),
        "text",
    );
    // The tab bar behind the popup is not a hand any more.
    hover(&mut term, "the tab bar behind it", (4, 1), "");
    hover(&mut term, "the cross", (cross.0 + 1, cross.1), "pointer");
    hover(&mut term, "an empty area", (1, 30), "");
    term.stop().unwrap();
}

/// A file row of the commit details' Changed files list lights across its
/// whole width under the pointer and keeps the arrow.
#[test]
fn a_changed_files_row_lights_on_hover_and_keeps_the_arrow() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .write_worktree("a.txt", b"a\n")
        .unwrap()
        .write_worktree("b.txt", b"b\n")
        .unwrap()
        .stage(".")
        .unwrap()
        .commit("pointer files probe")
        .unwrap()
        .build();
    let config = tempfile::tempdir().unwrap();
    let mut term = common::launch(repo.root())
        .no_commit_column()
        .session("pty-pointer-files")
        .config_dir(config.path())
        .size(200, 45)
        .no_exit_pause()
        .mouse_capture()
        .spawn();
    let screen = term
        .wait_for("the file list", STARTUP, |s| {
            s.contains("Changed files") && s.contains("b.txt")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (bc, br) = screen
        .find_all("b.txt")
        .into_iter()
        .find(|(c, _)| *c > 100)
        .expect("b.txt in the details pane");
    let far = (bc + 25, br);
    // At rest: the background of the row's far end.
    term.move_to(1, 30).unwrap();
    std::thread::sleep(Duration::from_millis(400));
    let at_rest = term.screen().cell(far.0, far.1).unwrap().bg;
    term.move_to(bc + 1, br).unwrap();
    let deadline = std::time::Instant::now() + STEP;
    while term.screen().cell(far.0, far.1).unwrap().bg == at_rest
        && std::time::Instant::now() < deadline
    {
        std::thread::sleep(Duration::from_millis(50));
    }
    let lit = term.screen().cell(far.0, far.1).unwrap().bg;
    assert_ne!(lit, at_rest, "the hovered row lights\n{}", term.text());
    // The row above does not light, and the pointer stays the arrow.
    let above = term.screen().cell(far.0, br - 1).unwrap().bg;
    assert_ne!(above, lit, "only the hovered row");
    assert_ne!(
        pointer_shapes(&term).last().map(String::as_str),
        Some("pointer"),
        "a row keeps the arrow"
    );
    term.stop().unwrap();
}
