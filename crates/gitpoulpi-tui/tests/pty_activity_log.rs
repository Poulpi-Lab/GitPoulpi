//! A failed operation tells the user what went wrong in the status line (no
//! card; the card-less flow is in `pty_status_error`); the Activity Log lists
//! it with the reason, a next step and the raw details, and shows the
//! application log with a level box; old log files are pruned.
//! Real binary, daemon and system git through a pty (`xtask::pty`); the
//! failing remote is a closed local port, so there is no network.

mod common;

use std::path::Path;
use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(30);
const COLS: u16 = 160;
const ROWS: u16 = 44;
const SECRET: &str = "topsecret";

/// A repository whose `origin` cannot be reached (a closed port, with a
/// password in the URL that must never be shown) and one unpushed commit.
fn unreachable_origin() -> gitpoulpi_testkit::FixtureRepo {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("activity base")
        .unwrap()
        .with_origin()
        .unwrap()
        .push("main")
        .unwrap()
        .track("main", "origin")
        .unwrap()
        .commit("activity unpushed")
        .unwrap()
        .build();
    repo.git(&[
        "remote",
        "set-url",
        "origin",
        &format!("http://me:{SECRET}@127.0.0.1:1/r.git"),
    ]);
    repo
}

fn launch(repo: &Path, session: &str, config: &Path) -> Terminal {
    common::launch(repo)
        .session(session)
        .config_dir(config)
        .size(COLS, ROWS)
        .env("GITPOULPI_CLIPBOARD", "memory")
        .ready("activity unpushed")
        .spawn()
}

fn click_push(term: &mut Terminal) {
    let push = term.wait_for_text("\u{21E7} Push", STEP).unwrap();
    term.click(push.0 + 2, push.1).expect("click Push");
}

/// A fetch of every remote by the palette.
fn fetch_all(term: &mut Terminal) {
    term.send_keys("C-p").unwrap();
    term.wait_for_text("Command Palette", STEP).unwrap();
    term.send_text("Fetch All Remotes").unwrap();
    term.wait_for("the entry", STEP, |s| s.contains("Git: Fetch All Remotes"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
}

/// The error in the status line: title and reason.
fn wait_for_error(term: &Terminal) -> xtask::pty::Screen {
    term.wait_for("the failure in the status line", STEP, |s| {
        s.contains("error: Fetch failed: The remote could not be reached.")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()))
}

#[test]
fn a_failed_fetch_is_in_the_status_line_and_the_log_keeps_it_with_details_and_copy() {
    let repo = unreachable_origin();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch(repo.root(), "pty-activity-card", config.path());
    fetch_all(&mut term);
    wait_for_error(&term);
    // The unread count is the status bar's, drawn from its own event: it can
    // land a frame after the line, so it is waited for rather than asserted.
    let screen = term
        .wait_for("the status bar counting the unread error", STEP, |s| {
            s.contains("Fetch failed") && s.contains("Activity (1)")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(!screen.contains(SECRET), "{}", screen.text());
    assert!(
        !screen.contains("exit code:") && !screen.contains(" Details "),
        "no card:\n{}",
        screen.text()
    );

    // The Activity Log lists the failure, first on the tab it opens on; the
    // entry opens: the reason, the step, the command git ran, its exit code
    // and what it said, with Copy and the remedy.
    let (c, r) = term.screen().find("Activity (1)").unwrap();
    term.click(c + 2, r).expect("click the Activity button");
    term.wait_for("the Activity Log", STEP, |s| {
        s.contains("Activity Log")
            && s.contains(" Application ")
            && s.contains(" Git operations ")
            && s.contains("Fetch failed")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    let shown = term
        .wait_for("the details", STEP, |s| {
            s.contains("Check your connection")
                && s.contains("command: git fetch")
                && s.contains("exit code:")
                && s.contains("stderr:")
                && s.contains(" Retry ")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(!shown.contains(SECRET), "{}", shown.text());

    // Copy by its key: the window says what went to the clipboard.
    term.send_keys("c").unwrap();
    term.wait_for("the copy confirmation", STEP, |s| {
        s.contains("copied the error details")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // The application log tab shows the daemon's and the window's lines.
    term.send_keys("Tab").unwrap();
    let log = term
        .wait_for("the application log", STEP, |s| {
            s.contains("Level:")
                && s.contains("Fetch all remotes failed [op #")
                && s.contains("gitpoulpi-tui starting")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(!log.contains(SECRET), "{}", log.text());

    // The file on disk has the same lines, with no password.
    let logs = config.path().join("cache").join("logs");
    let today = std::fs::read_dir(&logs)
        .expect("the log folder exists")
        .flatten()
        .map(|e| std::fs::read_to_string(e.path()).unwrap_or_default())
        .collect::<String>();
    assert!(today.contains("Fetch all remotes failed"), "{today}");
    assert!(!today.contains(SECRET), "{today}");
}

#[test]
fn the_level_box_filters_the_application_log() {
    let repo = unreachable_origin();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch(repo.root(), "pty-activity-level", config.path());
    click_push(&mut term);
    term.wait_for("the push error in the status line", STEP, |s| {
        s.contains("error: Push failed") && s.contains("Activity (1)")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("C-n").unwrap();
    term.send_text("A").unwrap();
    term.wait_for("the Activity Log by its key", STEP, |s| {
        s.contains("Activity Log")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Tab").unwrap();
    term.wait_for("both kinds of line at the default level", STEP, |s| {
        s.contains("gitpoulpi-tui starting") && s.contains("Push failed (")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // Open the level combobox with the mouse and pick Warning.
    let (c, r) = term.screen().find("Level:").unwrap();
    term.click(c + 8, r).expect("click the level box");
    let list = term
        .wait_for("the level list", STEP, |s| {
            // The display checkboxes name the levels too: the list is the second.
            s.find_all("Warning").len() >= 2 && s.find_all("Trace").len() >= 2
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // The display checkbox row also says "Warning": the list is the last one.
    let (c, r) = *list.find_all("Warning").last().unwrap();
    term.click(c + 1, r).expect("pick Warning");
    term.wait_for("only warnings and errors", STEP, |s| {
        s.contains("Push failed (") && !s.contains("gitpoulpi-tui starting")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // Asking for Debug raises what is recorded and says so.
    term.send_keys("l").unwrap();
    let list = term
        .wait_for("the level list again", STEP, |s| {
            s.find_all("Trace").len() >= 2
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let (c, r) = *list.find_all("Debug").last().unwrap();
    term.click(c + 1, r).expect("pick Debug");
    term.wait_for("the recording notice", STEP, |s| {
        s.contains("Recording Debug lines from now on.")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let toml = std::fs::read_to_string(config.path().join("config.toml")).unwrap_or_default();
    assert!(toml.contains("level = \"debug\""), "{toml}");

    // Esc closes the window.
    term.send_keys("Esc").unwrap();
    term.wait_for_text_to_go("Level:", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
}

#[test]
fn every_entry_control_has_a_keyboard_twin() {
    let repo = unreachable_origin();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch(repo.root(), "pty-activity-keys", config.path());
    fetch_all(&mut term);
    wait_for_error(&term);

    let palette = |term: &mut Terminal, query: &str, entry: &str| {
        term.send_keys("C-p").unwrap();
        term.wait_for_text("Command Palette", STEP).unwrap();
        term.send_text(query).unwrap();
        term.wait_for("the entry", STEP, |s| s.contains(entry))
            .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
        term.send_keys("Enter").unwrap();
    };
    palette(
        &mut term,
        "Open the Last Error",
        "Activity Log: Open the Last Error",
    );
    term.wait_for("the entry by palette", STEP, |s| {
        s.contains("exit code:") && s.contains(" Retry ")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // The window takes the keys: Esc closes it, and the error was read.
    term.send_keys("Esc").unwrap();
    term.wait_for("the window closed and the error read", STEP, |s| {
        !s.contains("Application") && s.contains("Activity") && !s.contains("Activity (1)")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    palette(
        &mut term,
        "Copy the Last Error",
        "Activity Log: Copy the Last Error",
    );
    term.wait_for("copy by palette", STEP, |s| {
        s.contains("copied the error details")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    palette(&mut term, "Copy diagnostics", "View: Copy Diagnostics");
    term.wait_for("diagnostics copied", STEP, |s| {
        s.contains("copied the diagnostics")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

#[test]
fn old_log_files_are_pruned_at_startup_to_the_retention_window() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("retention base")
        .unwrap()
        .build();
    let config = tempfile::tempdir().unwrap();
    let logs = config.path().join("cache").join("logs");
    std::fs::create_dir_all(&logs).unwrap();
    for ago in [2, 3, 9] {
        std::fs::write(logs.join(gitpoulpi_log::file_name_days_ago(ago)), "old\n").unwrap();
    }
    std::fs::write(logs.join("notes.txt"), "not a log file\n").unwrap();
    std::fs::write(
        config.path().join("config.toml"),
        "[log]\nretention_days = 3\n",
    )
    .unwrap();
    let mut term = common::launch(repo.root())
        .session("pty-activity-retention")
        .config_dir(config.path())
        .size(COLS, ROWS)
        .ready("retention base")
        .spawn();
    // The prune runs on its own thread, a moment after start-up.
    let gone = logs.join(gitpoulpi_log::file_name_days_ago(9));
    let deadline = std::time::Instant::now() + STEP;
    while gone.exists() && std::time::Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(50));
    }
    assert!(!gone.exists(), "the 9 day old file is past the window");
    assert!(
        logs.join(gitpoulpi_log::file_name_days_ago(2)).exists(),
        "two days ago is inside a 3 day window"
    );
    assert!(
        !logs.join(gitpoulpi_log::file_name_days_ago(3)).exists(),
        "today counts as the first day: three days ago is outside"
    );
    assert!(
        logs.join("notes.txt").exists(),
        "other files are left alone"
    );
    assert!(
        logs.join(gitpoulpi_log::file_name_days_ago(0)).exists(),
        "today's file is being written"
    );
    term.send_keys("q").ok();
}
