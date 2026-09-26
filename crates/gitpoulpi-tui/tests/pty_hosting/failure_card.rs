//! A failed hosting call has no framed card: the status line words it in
//! red (even over the form that is still open), the host refusing the token
//! (401) offers Open Integrations on the Activity entry, a host that breaks
//! (500) says so in plain words; the entry holds what the host said; the
//! keyboard twins work. The host is the loopback mock; nothing leaves
//! 127.0.0.1.

use xtask::pty::Terminal;

use crate::mock::*;
use crate::support::*;

/// Opens the Create Pull Request form from the section's `+` and presses
/// Create; the mock answers the POST with `status`.
fn create_and_fail(session: &str, status: u16) -> (Terminal, tempfile::TempDir, Mock) {
    let mock = start_mock();
    POST_FAILS.lock().unwrap().push((mock.port, status));
    let repo = fixture(mock.port);
    let (mut term, config) = launch(
        &repo,
        session,
        &config_toml(mock.port, ""),
        COLS,
        "Add the feature",
    );
    let screen = term
        .wait_for("the PULL REQUESTS section", STARTUP, pulls_ready)
        .unwrap_or_else(|e| panic!("{e}"));
    let (_, header_row) = screen.find("PULL REQUESTS (2)").unwrap();
    let header = screen.line(header_row);
    let plus_cell = header[..header.find(" + ").expect("the + button")]
        .chars()
        .count() as u16
        + 1;
    term.click(plus_cell, header_row).expect("click the +");
    term.wait_for("the Create Pull Request form", STEP, |s| {
        s.contains("Create Pull Request") && s.contains("From repo") && s.contains("Description")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    let (col, row) = term.wait_for_text("Create Pull Request", STEP).unwrap();
    // The title and the button share a label: the button is the last one.
    let all = term.screen().find_all("Create Pull Request");
    let (bc, br) = *all.last().unwrap_or(&(col, row));
    term.click(bc + 2, br).expect("click Create");
    (term, config, mock)
}

/// The error line is in the status line while the form is still open; then
/// the form is closed (it owns the keys while open).
fn wait_for_error_and_close_form(term: &mut Terminal, reason: &str) {
    term.wait_for("the error in the status line", STEP, |s| {
        s.contains("error: Hosting request failed") && s.contains(reason)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let screen = term.screen();
    assert!(
        !screen.contains(" Details ") && !screen.contains("Hosting request failed \u{2500}"),
        "no card:\n{}",
        screen.text()
    );
    term.send_keys("Esc").unwrap();
    term.wait_for("the form closed, the error kept", STEP, |s| {
        !s.contains("From repo") && s.contains("error: Hosting request failed")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

/// Clicks the error line, which opens the entry.
fn open_entry(term: &mut Terminal) {
    let screen = term.screen();
    let (row, line) = screen
        .text()
        .lines()
        .enumerate()
        .find(|(_, l)| l.contains("error: Hosting request failed"))
        .map(|(i, l)| (i, l.to_string()))
        .expect("the error line");
    let col = line[..line.find("error:").unwrap()].chars().count();
    term.click(col as u16 + 2, row as u16).expect("click it");
}

fn palette(term: &mut Terminal, query: &str, entry: &str) {
    term.send_keys("C-p").unwrap();
    term.wait_for_text("Command Palette", STEP).unwrap();
    term.send_text(query).unwrap();
    term.wait_for("the entry", STEP, |s| s.contains(entry))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
}

#[test]
fn a_host_that_refuses_the_token_offers_open_integrations_on_the_entry() {
    let (mut term, _config, mock) = create_and_fail("pty-hosting-card-401", 401);
    wait_for_error_and_close_form(&mut term, "The remote refused the credentials.");
    assert!(
        term.screen().contains("Activity (1)"),
        "the status bar counts the unread error:\n{}",
        term.text()
    );
    open_entry(&mut term);
    let entry = term
        .wait_for("the entry with its remedy", STEP, |s| {
            s.contains("Activity Log")
                && s.contains("401")
                && s.contains(" Open Integrations ")
                && s.contains(" Copy ")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // The remedy opens Settings on the Integrations page.
    let (c, r) = entry.find(" Open Integrations ").unwrap();
    term.click(c + 1, r).expect("click Open Integrations");
    term.wait_for("Preferences on Integrations", STEP, |s| {
        s.contains("Integrations") && !s.contains("Activity Log")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(mock.saw("POST", "merge_requests"));
    term.stop().unwrap();
}

#[test]
fn the_palette_runs_the_suggested_action_of_a_refused_token() {
    let (mut term, _config, _mock) = create_and_fail("pty-hosting-card-401-twin", 401);
    wait_for_error_and_close_form(&mut term, "The remote refused the credentials.");
    // Copy by its twin.
    palette(
        &mut term,
        "Copy the Last Error",
        "Activity Log: Copy the Last Error",
    );
    term.wait_for("copy by palette", STEP, |s| {
        s.contains("copied the error details")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    palette(
        &mut term,
        "Run the Suggested Action",
        "Activity Log: Run the Suggested Action",
    );
    term.wait_for("Preferences on Integrations", STEP, |s| {
        s.contains("Integrations") && !s.contains("Activity Log")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

#[test]
fn a_host_that_breaks_is_worded_in_plain_words_and_has_no_remedy() {
    let (mut term, _config, _mock) = create_and_fail("pty-hosting-card-500", 500);
    wait_for_error_and_close_form(&mut term, "The hosting service reported a problem.");
    open_entry(&mut term);
    let entry = term
        .wait_for("the entry with what the host said", STEP, |s| {
            s.contains("Activity Log")
                && s.contains("Open the details")
                && s.contains("500")
                && s.contains(" Copy ")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(
        !entry.contains(" Open Integrations "),
        "no remedy for a broken host:\n{}",
        entry.text()
    );
    // The palette entry says why there is nothing to run.
    term.send_keys("Esc").unwrap();
    term.stop().unwrap();
}
