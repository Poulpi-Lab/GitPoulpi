//! Background jobs in the window (see the background-ops design, slices B6 and
//! B7): the status bar indicator, the jobs panel, and every way to cancel.
//!
//! A push (or a pull) is kept running by the daemon's test hook
//! `GITPOULPI_TEST_TRANSFER_DELAY_MS` (a cancellable sleep before the
//! backend call, so nothing is ever transferred): the job is `Running`
//! long enough to click, type and switch tabs. Real binary, real daemon, a
//! bare upstream in the fixture's temp dir.
//!
//! The maintainer's words: "annulable à la souris, pas que avec Escape" -- the
//! mouse tests click the very `[x]` / `Cancel` the screen shows.

mod common;

use std::time::Duration;

use gitpoulpi_session::GlobalSession;
use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::{Screen, Terminal};

const STARTUP: Duration = Duration::from_secs(60);
const STEP: Duration = Duration::from_secs(30);

/// A repository one commit ahead of its upstream.
fn fixture() -> FixtureRepo {
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

fn launch(repo: &FixtureRepo, session: &str, delay_ms: u32) -> (Terminal, tempfile::TempDir) {
    launch_with(repo, session, delay_ms, None)
}

/// `integrate_ms` holds a pull in its non-cancellable integrate phase
/// (`GITPOULPI_TEST_INTEGRATE_DELAY_MS`); without it that phase lasts a few
/// milliseconds and a test cannot reliably see it.
fn launch_with(
    repo: &FixtureRepo,
    session: &str,
    delay_ms: u32,
    integrate_ms: Option<u32>,
) -> (Terminal, tempfile::TempDir) {
    let config = tempfile::tempdir().expect("temp config dir");
    let mut builder = common::launch(repo.root())
        .session(session)
        .config_dir(config.path())
        .size(160, 40)
        .no_exit_pause()
        .env("GITPOULPI_TEST_TRANSFER_DELAY_MS", delay_ms.to_string());
    if let Some(ms) = integrate_ms {
        builder = builder.env("GITPOULPI_TEST_INTEGRATE_DELAY_MS", ms.to_string());
    }
    let term = builder.spawn();
    term.wait_for("the fixture loaded", STARTUP, |s| {
        s.contains("ahead commit")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    (term, config)
}

/// The indicator's text on the footer rows: `Push: <repo>` with its `[x]`.
fn indicator_shows(s: &Screen, what: &str) -> bool {
    s.text()
        .lines()
        .any(|l| l.contains(what) && l.contains("[x]"))
}

fn start_push(term: &mut Terminal) {
    term.send_keys("C-g").unwrap();
    term.send_keys("P").unwrap();
    term.wait_for("the indicator with the running push", STEP, |s| {
        indicator_shows(s, "Push: ")
    })
    .unwrap_or_else(|e| {
        panic!(
            "{e}
{}",
            term.text()
        )
    });
}

fn remote_main(repo: &FixtureRepo) -> String {
    repo.git_in_remote(&["rev-parse", "main"])
        .trim()
        .to_string()
}

fn canceled_flash(s: &Screen) -> bool {
    s.text()
        .lines()
        .any(|l| l.contains("Push: ") && l.contains("(Canceled)"))
}

/// The indicator is in the Home window too, and a click on its `[x]` cancels
/// the push: nothing reaches the remote.
#[test]
fn the_indicator_follows_to_home_and_clicking_its_x_cancels() {
    let repo = fixture();
    let before = remote_main(&repo);
    let (mut term, _config) = launch(&repo, "pty-jobs-click", 25_000);
    start_push(&mut term);

    // Home has no repository; the job is still there.
    term.send_keys("C-o").unwrap();
    // The `[x]` is part of the post-condition and its position comes from
    // the screen the wait returned: the indicator paints after the page, and
    // a click at a position read off a later `screen()` could land on the
    // frame before it (testing.md, waiting pattern 1).
    let screen = term
        .wait_for("Home with the indicator and its [x]", STEP, |s| {
            s.contains("Repository Management")
                && indicator_shows(s, "Push: ")
                && s.find("[x]").is_some()
        })
        .unwrap_or_else(|e| {
            panic!(
                "{e}
{}",
                term.text()
            )
        });
    let (col, row) = screen.find("[x]").expect("the [x] button");
    term.click(col + 1, row).unwrap();
    term.wait_for("the canceled push", STEP, canceled_flash)
        .unwrap_or_else(|e| {
            panic!(
                "{e}
{}",
                term.text()
            )
        });
    assert_eq!(remote_main(&repo), before, "nothing was pushed");
    term.stop().unwrap();
}

/// Launches in the fixture with a second repository already open as a tab,
/// the way `gitpoulpi repo open` leaves the session.
fn launch_with_second_tab(
    repo: &FixtureRepo,
    other: &FixtureRepo,
    session: &str,
    delay_ms: u32,
) -> (Terminal, tempfile::TempDir) {
    let config = tempfile::tempdir().expect("temp config dir");
    let dir = config.path().join("sessions");
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join(format!("{session}.toml"));
    let mut seeded = GlobalSession::load(&file).unwrap();
    seeded.open(other.root());
    seeded.save(&file).unwrap();
    let term = common::launch(repo.root())
        .session(session)
        .config_dir(config.path())
        .size(160, 40)
        .no_exit_pause()
        .env("GITPOULPI_TEST_TRANSFER_DELAY_MS", delay_ms.to_string())
        .spawn();
    term.wait_for("the fixture loaded", STARTUP, |s| {
        s.contains("ahead commit")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    (term, config)
}

/// Switches to the second repository's tab: the first name on the tab bar
/// (it was seeded into the session, so it sits before the launch folder's).
fn show_other_tab(term: &mut Terminal) {
    let deadline = std::time::Instant::now() + STEP;
    let col = loop {
        let bar = term.screen().line(1);
        if bar.matches("repo").count() >= 2 {
            break bar[..bar.find("repo").unwrap()].chars().count() as u16;
        }
        assert!(std::time::Instant::now() < deadline, "{}", term.text());
        std::thread::sleep(Duration::from_millis(300));
    };
    term.click(col + 1, 1).unwrap();
    term.wait_for("the other repository", STEP, |s| {
        s.contains("other base") && !s.contains("ahead commit")
    })
    .unwrap_or_else(|e| {
        panic!(
            "{e}
{}",
            term.text()
        )
    });
}

fn other_fixture() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit("other base")
        .unwrap()
        .build()
}

/// Another repository tab: the same indicator, the same click.
#[test]
fn the_indicator_is_there_from_another_tab() {
    let repo = fixture();
    let other = other_fixture();
    let before = remote_main(&repo);
    let (mut term, _config) = launch_with_second_tab(&repo, &other, "pty-jobs-tab", 25_000);
    start_push(&mut term);
    show_other_tab(&mut term);
    term.wait_for("the push on the other tab's status bar", STEP, |s| {
        indicator_shows(s, "Push: ")
    })
    .unwrap_or_else(|e| {
        panic!(
            "{e}
{}",
            term.text()
        )
    });

    let (col, row) = term.screen().find("[x]").expect("the [x] button");
    term.click(col + 1, row).unwrap();
    term.wait_for("the canceled push", STEP, canceled_flash)
        .unwrap_or_else(|e| {
            panic!(
                "{e}
{}",
                term.text()
            )
        });
    assert_eq!(remote_main(&repo), before);
    term.stop().unwrap();
}

/// Enter on a job of another repository switches to its tab.
#[test]
fn enter_on_a_job_of_another_repository_shows_its_tab() {
    let repo = fixture();
    let other = other_fixture();
    let (mut term, _config) = launch_with_second_tab(&repo, &other, "pty-jobs-goto", 25_000);
    start_push(&mut term);
    show_other_tab(&mut term);
    term.send_keys("C-n").unwrap();
    term.send_keys("J").unwrap();
    term.wait_for("the jobs panel", STEP, |s| s.contains("Background jobs"))
        .unwrap_or_else(|e| {
            panic!(
                "{e}
{}",
                term.text()
            )
        });
    term.send_keys("Enter").unwrap();
    term.wait_for("the pushing repository's tab", STEP, |s| {
        s.contains("ahead commit") && !s.contains("Background jobs")
    })
    .unwrap_or_else(|e| {
        panic!(
            "{e}
{}",
            term.text()
        )
    });
    term.stop().unwrap();
}

/// The palette twin: `Jobs: Cancel Push: <repo>` in the palette.
#[test]
fn the_palette_cancels_the_push() {
    let repo = fixture();
    let before = remote_main(&repo);
    let (mut term, _config) = launch(&repo, "pty-jobs-palette", 25_000);
    start_push(&mut term);

    term.send_keys("C-p").unwrap();
    term.send_text("Jobs: Cancel Push").unwrap();
    term.wait_for("the Cancel row", STEP, |s| {
        s.contains("Jobs: Cancel Push: ")
    })
    .unwrap_or_else(|e| {
        panic!(
            "{e}
{}",
            term.text()
        )
    });
    term.send_keys("Enter").unwrap();
    term.wait_for("the canceled push", STEP, canceled_flash)
        .unwrap_or_else(|e| {
            panic!(
                "{e}
{}",
                term.text()
            )
        });
    assert_eq!(remote_main(&repo), before);
    term.stop().unwrap();
}

/// The keyboard twin: `Ctrl+N Shift+J` opens the panel, `Del` cancels the
/// selected job.
#[test]
fn the_panel_opens_by_key_and_del_cancels() {
    let repo = fixture();
    let before = remote_main(&repo);
    let (mut term, _config) = launch(&repo, "pty-jobs-key", 25_000);
    start_push(&mut term);

    term.send_keys("C-n").unwrap();
    term.send_keys("J").unwrap();
    term.wait_for("the jobs panel", STEP, |s| {
        s.contains("Background jobs") && s.contains("Cancel all")
    })
    .unwrap_or_else(|e| {
        panic!(
            "{e}
{}",
            term.text()
        )
    });
    term.send_keys("Delete").unwrap();
    term.wait_for("the row says Canceled", STEP, |s| {
        s.text()
            .lines()
            .any(|l| l.contains("Push: ") && l.contains("Canceled"))
    })
    .unwrap_or_else(|e| {
        panic!(
            "{e}
{}",
            term.text()
        )
    });
    assert_eq!(remote_main(&repo), before);
    term.send_keys("Escape").unwrap();
    term.wait_for("the panel closed", STEP, |s| !s.contains("Background jobs"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.stop().unwrap();
}

/// A click on the indicator's label opens the panel; a click on the row's
/// Cancel button cancels.
#[test]
fn the_panel_opens_by_click_and_its_cancel_button_cancels() {
    let repo = fixture();
    let before = remote_main(&repo);
    let (mut term, _config) = launch(&repo, "pty-jobs-panel-click", 25_000);
    start_push(&mut term);

    let (col, row) = term.screen().find("Push: ").expect("the indicator label");
    term.click(col + 1, row).unwrap();
    term.wait_for("the jobs panel", STEP, |s| s.contains("Background jobs"))
        .unwrap_or_else(|e| {
            panic!(
                "{e}
{}",
                term.text()
            )
        });
    // The row's own button: the second "Cancel" on the screen (the first is
    // the toolbar's "Cancel all").
    let screen = term.screen();
    let row_cancel = screen
        .find_all("Cancel")
        .into_iter()
        .find(|(c, r)| {
            let line = screen.line(*r);
            line.contains("Push: ") && *c > 60
        })
        .expect("the row's Cancel button");
    term.click(row_cancel.0 + 1, row_cancel.1).unwrap();
    term.wait_for("the row says Canceled", STEP, |s| {
        s.text()
            .lines()
            .any(|l| l.contains("Push: ") && l.contains("Canceled"))
    })
    .unwrap_or_else(|e| {
        panic!(
            "{e}
{}",
            term.text()
        )
    });
    assert_eq!(remote_main(&repo), before);
    term.stop().unwrap();
}

/// A pull cannot be stopped once it integrates: its Cancel is disabled, the
/// reason is on the row, and pressing the button says why instead of
/// cancelling.
#[test]
fn a_job_that_cannot_be_stopped_shows_why() {
    let repo = fixture();
    let (mut term, _config) = launch_with(&repo, "pty-jobs-refusal", 500, Some(40_000));
    term.send_keys("C-g").unwrap();
    term.send_keys("p").unwrap();
    term.wait_for("the running pull", STEP, |s| indicator_shows(s, "Pull: "))
        .unwrap_or_else(|e| {
            panic!(
                "{e}
{}",
                term.text()
            )
        });

    term.send_keys("C-n").unwrap();
    term.send_keys("J").unwrap();
    term.wait_for("the reason on the pull's row", STEP, |s| {
        s.contains("Integrating changes: cannot be stopped safely")
    })
    .unwrap_or_else(|e| {
        panic!(
            "{e}
{}",
            term.text()
        )
    });

    let screen = term.screen();
    let (col, row) = screen
        .find_all("Cancel")
        .into_iter()
        .find(|(c, r)| screen.line(*r).contains("Pull: ") && *c > 60)
        .expect("the row's (disabled) Cancel button");
    term.click(col + 1, row).unwrap();
    term.wait_for("the refusal on the status line", STEP, |s| {
        s.contains("cannot cancel")
    })
    .unwrap_or_else(|e| {
        panic!(
            "{e}
{}",
            term.text()
        )
    });
    // Still running, not canceled.
    assert!(
        !term.screen().text().contains("Canceled"),
        "{}",
        term.text()
    );
    term.stop().unwrap();
}
