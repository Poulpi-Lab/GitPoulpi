//! Settings > Git & encoding: `core.autocrlf` / `core.eol` are applied to the
//! repository's real git config (not just remembered in `repo-state.toml`),
//! shown from what git reports, and undone by Undo. Also: the rows that used
//! to say "coming" carry no such note any more. Fixture repo and temp config
//! dir only.

mod common;

use std::time::{Duration, Instant};

use gitpoulpi_testkit::FixtureRepoBuilder;

const STEP: Duration = Duration::from_secs(30);

fn wait_git(repo: &std::path::Path, key: &str, want: Option<&str>) {
    let deadline = Instant::now() + STEP;
    loop {
        let now = common::git_try(repo, &["config", "--local", "--get", key])
            .map(|s| s.trim().to_string());
        if now.as_deref() == want {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "{key}: wanted {want:?}, git says {now:?}"
        );
        std::thread::sleep(Duration::from_millis(150));
    }
}

#[test]
fn picking_core_autocrlf_in_settings_writes_git_config_and_undo_restores_it() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("git config probe")
        .unwrap()
        .build();
    // Known starting point: set outside the app, so the row must read git.
    common::git_ok(repo.root(), &["config", "--local", "core.autocrlf", "true"]);
    let (mut term, _config) = common::launch(repo.root())
        .session("settings-git-config")
        .size(200, 60)
        .ready("git config probe")
        .start();

    let (gc, gr) = term
        .wait_for_text("\u{2699}", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(gc, gr).unwrap();
    term.wait_for("Settings open", STEP, |s| {
        s.contains("Repo-specific settings")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    // Walk the sidebar to the Git & encoding page.
    // One Down per poll that has not yet shown the page: a fixed sleep after
    // each key raced the redraw on a loaded machine and walked past the page.
    let screen = common::retry_until(
        &mut term,
        "the Git page",
        40,
        Duration::from_secs(2),
        |t| {
            let before = t.screen().text();
            if !before.contains("core.autocrlf") {
                t.send_keys("Down").unwrap();
                // The key is answered by a redraw: do not send the next one
                // before it, or a burst walks past the page.
                let _ = t.wait_for("a redraw", Duration::from_secs(5), |s| s.text() != before);
            }
        },
        |s| {
            s.find("core.autocrlf")
                .is_some_and(|(_, r)| s.line(r).contains("true"))
        },
    );
    assert!(
        !screen.contains("nothing calls"),
        "the 'coming' note is gone:\n{}",
        screen.text()
    );
    // What git says is what the row shows (set outside the app above).
    let (_, row) = screen.find("core.autocrlf").unwrap();
    let line = screen.line(row);
    assert!(line.contains("true"), "row shows git's value: {line}");

    // Open the row's dropdown (click its value) and pick `input`.
    let (vc, vr) = screen
        .find_all("true")
        .into_iter()
        .find(|(_, r)| *r == row)
        .expect("the row's value");
    term.click(vc + 1, vr).unwrap();
    let popup = term
        .wait_for("the options", STEP, |s| {
            s.find("\u{2713} true").is_some_and(|(tc, tr)| {
                s.find_all("input")
                    .into_iter()
                    .any(|(c, r)| r > tr && c.abs_diff(tc) < 6)
            })
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let (tc, tr) = popup.find("\u{2713} true").unwrap();
    let (ic, ir) = popup
        .find_all("input")
        .into_iter()
        .find(|(c, r)| *r > tr && c.abs_diff(tc) < 6)
        .expect("the input option under the ticked one");
    term.click(ic, ir).unwrap();
    wait_git(repo.root(), "core.autocrlf", Some("input"));

    // Close Settings and Undo: the previous value comes back.
    term.send_keys("Escape").unwrap();
    term.send_keys("Escape").unwrap();
    term.send_keys("C-z").unwrap();
    wait_git(repo.root(), "core.autocrlf", Some("true"));
    let _ = term.stop();
}

/// The SSH page: the key path rows are real rows with their own help, not
/// "coming" notes (the date locale row sits below the fold; its unit test
/// covers it).
#[test]
fn the_ssh_key_rows_have_no_coming_note() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("coming rows probe")
        .unwrap()
        .build();
    let (mut term, _config) = common::launch(repo.root())
        .session("settings-no-coming")
        .size(200, 60)
        .ready("coming rows probe")
        .start();
    let (gc, gr) = term
        .wait_for_text("\u{2699}", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(gc, gr).unwrap();
    term.wait_for("Settings open", STEP, |s| {
        s.contains("Repo-specific settings")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    for (row, help) in [("Private key path", "GIT_SSH_COMMAND")] {
        // One Down per poll that has not yet shown the row, each answered by
        // a redraw before the next (a fixed sleep raced it under load).
        let screen = common::retry_until(
            &mut term,
            row,
            40,
            Duration::from_secs(2),
            |t| {
                let before = t.screen().text();
                if !before.contains(row) {
                    t.send_keys("Down").unwrap();
                    let _ = t.wait_for("a redraw", Duration::from_secs(5), |s| s.text() != before);
                }
            },
            |s| s.contains(row) && s.contains(help),
        );
        let text = screen.text();
        assert!(text.contains(help), "{row} explains itself:\n{text}");
        for stale in [
            "roadmap",
            "nothing outside this screen",
            "always spells them in English",
            "Date word Format",
        ] {
            assert!(
                !text.contains(stale),
                "{row}: '{stale}' still shown:\n{text}"
            );
        }
    }
    let _ = term.stop();
}
