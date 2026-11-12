//! A press outside an open dropdown of the history rewrite workbench closes it and
//! runs nothing underneath; a right-click outside closes it too.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::Terminal;

const STARTUP: Duration = Duration::from_secs(60);
const STEP: Duration = Duration::from_secs(30);

fn fixture() -> FixtureRepo {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "base")
        .unwrap()
        .build();
    for message in ["first by old", "second by old"] {
        repo.git(&[
            "-c",
            "user.name=Old Name",
            "-c",
            "user.email=old@example.com",
            "commit",
            "--allow-empty",
            "-q",
            "-m",
            message,
        ]);
    }
    repo
}

fn launch(repo: &FixtureRepo, session: &str) -> (Terminal, tempfile::TempDir) {
    let config = tempfile::tempdir().expect("temp config dir");
    let term = common::launch(repo.root())
        .session(session)
        .config_dir(config.path())
        .size(200, 50)
        .mouse_capture()
        .spawn();
    term.wait_for("the fixture loaded", STARTUP, |s| {
        s.contains("second by old") && s.contains("first by old")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    (term, config)
}

fn open_workbench(term: &mut Terminal) {
    term.send_keys("C-g").unwrap();
    term.send_text("H").unwrap();
    term.wait_for("the workbench", STEP, |s| {
        s.contains("Rewrite history") && s.contains("Add a rule to start")
    })
    .unwrap_or_else(|e| panic!("{e}"));
}

/// The control the presses land on: the Tags checkbox, ticked. If a press
/// ran it, it would be unticked.
fn tags_ticked(s: &xtask::pty::Screen) -> bool {
    s.contains("[x] Tags")
}

/// Opens `open` (a control's text, clicked until `marker` shows), then: a left
/// press on the Tags checkbox closes the popup and leaves the box ticked;
/// reopened, a right-click there closes it too.
fn dropdown_closes_outside(session: &str, open: &str, marker: &str, prepare: fn(&mut Terminal)) {
    let repo = fixture();
    let (mut term, _c) = launch(&repo, session);
    open_workbench(&mut term);
    prepare(&mut term);
    let popup_open = |s: &xtask::pty::Screen| s.contains(marker) && tags_ticked(s);
    common::click_text_until(&mut term, open, "the dropdown", popup_open);
    let (c, r) = term.screen().find("[x] Tags").unwrap();
    term.click(c + 1, r).unwrap();
    term.wait_for("the dropdown gone, Tags still ticked", STEP, |s| {
        !s.contains(marker) && tags_ticked(s)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    common::click_text_until(&mut term, open, "the dropdown again", popup_open);
    let (c, r) = term.screen().find("[x] Tags").unwrap();
    term.right_click(c + 1, r).unwrap();
    term.wait_for("the dropdown gone after a right-click", STEP, |s| {
        !s.contains(marker) && tags_ticked(s)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

#[test]
fn the_add_rule_menu_closes_on_a_press_outside_without_running_it() {
    dropdown_closes_outside("pty-rw-away-add", "+ Add rule", "Replace an author", |_| {});
}

#[test]
fn the_scope_dropdown_closes_on_a_press_outside_without_running_it() {
    dropdown_closes_outside(
        "pty-rw-away-scope",
        "Current branch",
        "All local branches",
        |_| {},
    );
}

/// The Commits tab's filter dropdown.
#[test]
fn the_commits_filter_dropdown_closes_on_a_press_outside_without_running_it() {
    // "Changed  " (two spaces) is the combo's padded label, not the "Changed
    // files" pane title.
    dropdown_closes_outside("pty-rw-away-filter", "Changed  ", "Removed", |term| {
        // The filter belongs to a previewed rule: an author rule, F5.
        term.send_text("a").unwrap();
        term.wait_for("the Add menu", STEP, |s| s.contains("Replace an author"))
            .unwrap_or_else(|e| panic!("{e}"));
        term.send_keys("Enter").unwrap();
        term.wait_for("the rule's fields", STEP, |s| s.contains("Email is"))
            .unwrap_or_else(|e| panic!("{e}"));
        term.send_keys("Down").unwrap();
        term.send_keys("Enter").unwrap();
        term.send_text("old@example.com").unwrap();
        term.send_keys("Enter").unwrap();
        for _ in 0..3 {
            term.send_keys("Down").unwrap();
        }
        term.send_keys("Enter").unwrap();
        term.send_text("new@example.org").unwrap();
        term.send_keys("Enter").unwrap();
        term.wait_for("both emails in the fields", STEP, |s| {
            s.contains("old@example.com") && s.contains("new@example.org")
        })
        .unwrap_or_else(|e| panic!("{e}"));
        term.send_keys("F5").unwrap();
        term.wait_for("the summary", STEP, |s| s.contains("keep their id"))
            .unwrap_or_else(|e| panic!("{e}"));
        // The tab row is the line holding "Summary" ("Commits" is also a
        // word elsewhere on the screen).
        common::retry_until(
            term,
            "the Commits tab",
            8,
            Duration::from_secs(4),
            |t| {
                let s = t.screen();
                if let Some((_, row)) = s.find("Summary") {
                    let line = s.line(row);
                    if let Some(at) = line.find("Commits") {
                        t.click(line[..at].chars().count() as u16 + 1, row).unwrap();
                    }
                }
            },
            |s| s.contains("Changed"),
        );
    });
}

/// A fake `gpg.program`: git believes the commit was signed.
#[cfg(unix)]
fn fake_gpg(dir: &std::path::Path) -> std::path::PathBuf {
    use std::os::unix::fs::PermissionsExt;
    let path = dir.join("fake-gpg.sh");
    std::fs::write(
        &path,
        "#!/bin/sh\ncat >/dev/null\necho '[GNUPG:] SIG_CREATED ' >&2\nprintf -- '-----BEGIN PGP SIGNATURE-----\\n\\nZmFrZQ==\\n-----END PGP SIGNATURE-----\\n'\n",
    )
    .unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    path
}

/// The signature question (a modal question of the workbench): a left press
/// outside it, then a right-click outside it, each closes it without running
/// anything (no preview appears, Tags stays ticked).
#[cfg(unix)]
#[test]
fn the_signature_question_closes_on_a_press_outside_without_running_anything() {
    let scratch = tempfile::tempdir().unwrap();
    let repo = fixture();
    repo.git(&[
        "-c",
        "user.name=Old Name",
        "-c",
        "user.email=old@example.com",
        "-c",
        &format!("gpg.program={}", fake_gpg(scratch.path()).display()),
        "-c",
        "user.signingkey=ABCDEF0123456789",
        "commit",
        "--allow-empty",
        "-q",
        "-S",
        "-m",
        "signed tip",
    ]);
    let (mut term, _c) = launch(&repo, "pty-rw-away-question");
    open_workbench(&mut term);
    term.send_text("a").unwrap();
    term.wait_for("the Add menu", STEP, |s| s.contains("Replace an author"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    term.wait_for("the rule's fields", STEP, |s| s.contains("Email is"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Down").unwrap();
    term.send_keys("Enter").unwrap();
    term.send_text("old@example.com").unwrap();
    term.send_keys("Enter").unwrap();
    for _ in 0..3 {
        term.send_keys("Down").unwrap();
    }
    term.send_keys("Enter").unwrap();
    term.send_text("new@example.org").unwrap();
    term.send_keys("Enter").unwrap();
    term.wait_for("both emails in the fields", STEP, |s| {
        s.contains("old@example.com") && s.contains("new@example.org")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    for right in [false, true] {
        term.send_keys("F5").unwrap();
        term.wait_for("the question", STEP, |s| {
            s.contains("Signed commits change") && s.contains("Remove the signatures")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
        let (c, r) = term.screen().find("[x] Tags").unwrap();
        if right {
            term.right_click(c + 1, r).unwrap();
        } else {
            term.click(c + 1, r).unwrap();
        }
        term.wait_for("the question gone, nothing ran", STEP, |s| {
            !s.contains("Signed commits change") && tags_ticked(s) && !s.contains("keep their id")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    }
    term.stop().unwrap();
}
