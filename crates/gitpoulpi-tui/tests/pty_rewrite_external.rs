//! External scripts in the history rewrite workbench through a pty
//! (`xtask::pty`): the real binary, daemon and git. A shell script that
//! upper-cases the author name is written in the editor (so it is the user's
//! own and runs without a question), tested, previewed, applied and undone;
//! the shell script of the template, which nobody wrote, asks "Trust this
//! script?" first, with no default answer. Same rules as `pty_rewrite.rs`: no
//! coordinates for keys, no sleeps, every wait names the state it waits for.
//! Skipped on a machine without `sh`.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::Terminal;

const STARTUP: Duration = Duration::from_secs(60);
const STEP: Duration = Duration::from_secs(30);

/// A one-line shell script (the pty types it key by key; no newline needed):
/// reads each request, answers the author's name in capitals.
const UPPERCASE: &str = r#"while IFS= read -r l; do n=$(printf '%s' "$l" | sed -n 's/.*"author":{[^}]*"name":"\([^"]*\)".*/\1/p' | tr a-z A-Z); printf '{"author":{"name":"%s"}}\n' "$n"; done"#;

/// A line of the shell template the editor opens on: the platform's own
/// (PowerShell on Windows, POSIX elsewhere).
const TEMPLATE_MARK: &str = if cfg!(windows) {
    // The editor pane is narrow: the line is cut after "ConvertFrom".
    "ConvertFrom"
} else {
    "#!/bin/sh"
};

fn have_sh() -> bool {
    std::process::Command::new("sh")
        .args(["-c", ":"])
        .status()
        .is_ok_and(|s| s.success())
}

/// The interpreter of the platform's shell template: `sh`, or PowerShell
/// (`pwsh`, else Windows PowerShell) on Windows.
fn have_template_shell() -> bool {
    if !cfg!(windows) {
        return have_sh();
    }
    ["pwsh", "powershell"].iter().any(|p| {
        std::process::Command::new(p)
            .args(["-NoProfile", "-Command", "exit 0"])
            .status()
            .is_ok_and(|s| s.success())
    })
}

/// `base` by the fixture's identity, then two commits by "Old Name".
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

/// The authors of the history; a read that fails while a ref swap is in
/// flight (the rewrite's apply or undo) is retried.
fn names(repo: &FixtureRepo) -> Vec<String> {
    let mut out = None;
    for _ in 0..100 {
        out = repo.try_git(&["log", "--format=%an"]);
        if out.is_some() {
            break;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    out.unwrap_or_else(|| repo.git(&["log", "--format=%an"]))
        .lines()
        .map(str::to_string)
        .collect()
}

fn launch(repo: &FixtureRepo, session: &str) -> (Terminal, tempfile::TempDir) {
    let config = tempfile::tempdir().expect("temp config dir");
    let term = common::launch(repo.root())
        .session(session)
        .config_dir(config.path())
        .size(200, 50)
        .spawn();
    term.wait_for("the fixture loaded", STARTUP, |s| {
        s.contains("second by old") && s.contains("first by old")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    (term, config)
}

fn wait_until(what: &str, mut ok: impl FnMut() -> bool) {
    let deadline = std::time::Instant::now() + STEP;
    while !ok() {
        assert!(std::time::Instant::now() < deadline, "timed out: {what}");
        std::thread::sleep(Duration::from_millis(50));
    }
}

/// Palette: a Script rule, the editor on the Lua template; Esc leaves the
/// editor on the rule's pane, where Language is the first row; the dropdown's
/// second row is the shell; then Enter on the Script row opens the editor on
/// the shell template.
fn open_shell_editor(term: &mut Terminal) {
    term.send_keys("C-p").unwrap();
    term.send_text("run a script").unwrap();
    term.wait_for("the palette row", STEP, |s| {
        s.contains("Rewrite: Run a Script...")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    term.wait_for("the editor on the Lua template", STEP, |s| {
        s.contains("Lua script") && s.contains("OK: rewrite_commit found")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Esc").unwrap();
    term.wait_for("the rule's pane with its language", STEP, |s| {
        s.contains("Language") && s.contains("Lua (built in)") && !s.contains("Test on selection")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    // The dropdown lists the four languages.
    term.send_keys("Enter").unwrap();
    term.wait_for("the language dropdown", STEP, |s| {
        s.contains("Shell script") && s.contains("Python") && s.contains("Lua (installed)")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Down").unwrap();
    term.send_keys("Enter").unwrap();
    // The pane says the script is not sandboxed, as soon as it is a shell.
    term.wait_for("the shell language with its warning", STEP, |s| {
        s.contains("Not sandboxed: it runs with your rights.")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    // Language, Source, Script: Enter on the third opens the editor.
    term.send_keys("Down").unwrap();
    term.send_keys("Down").unwrap();
    term.send_keys("Enter").unwrap();
    term.wait_for("the editor on the shell template", STEP, |s| {
        s.contains("Script (not sandboxed)")
            && s.contains(TEMPLATE_MARK)
            && s.contains("Test on selection")
    })
    .unwrap_or_else(|e| panic!("{e}"));
}

/// The shell script is typed, checked (it answers the trial request), tested
/// on commits, previewed, applied and undone, without any question.
#[test]
fn a_shell_script_written_here_is_tested_previewed_applied_and_undone() {
    if !have_sh() {
        eprintln!("skipped: no sh on this machine");
        return;
    }
    let repo = fixture();
    let original = names(&repo);
    let tip = repo.git(&["rev-parse", "HEAD"]);
    let (mut term, _config) = launch(&repo, "pty-rewrite-external");
    open_shell_editor(&mut term);
    term.send_keys("C-a").unwrap();
    // The shebang pins the POSIX dialect: without one, Windows runs a shell
    // script as PowerShell.
    term.send_text("#!/bin/sh").unwrap();
    term.send_keys("Enter").unwrap();
    term.send_text(UPPERCASE).unwrap();
    // Written here, so it runs without being trusted first: the live check
    // starts it once with a made-up commit and says it answered.
    term.wait_for("the typed script checked", STEP, |s| {
        s.contains("OK: it answered a trial run.")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("F5").unwrap();
    // No "Trust this script?" in the whole state: a script typed here is not
    // asked about.
    term.wait_for("the test result, no trust question", STEP, |s| {
        s.contains("3 commits tested, 3 would change.")
            && s.contains("author name: Old Name -> OLD NAME")
            && !s.contains("Trust this script?")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(names(&repo), original, "a test writes nothing");

    term.send_keys("Esc").unwrap();
    term.wait_for("the workbench again", STEP, |s| {
        s.contains("Rewrite history") && !s.contains("Test on selection")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("F5").unwrap();
    term.wait_for("the summary", STEP, |s| {
        s.contains("keep their id") && s.contains("3 of 3")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(names(&repo), original, "the preview changes nothing");
    term.send_keys("C-s").unwrap();
    term.wait_for("the result panel", STEP, |s| s.contains("Rewrote 3"))
        .unwrap_or_else(|e| panic!("{e}"));
    let shouted: Vec<String> = original.iter().map(|n| n.to_uppercase()).collect();
    wait_until("authors upper-cased", || names(&repo) == shouted);
    assert_ne!(repo.git(&["rev-parse", "HEAD"]), tip);
    // Disk before journal: a Ctrl+Z the moment the authors changed can find
    // nothing to undo yet. Press it again until they are back (the rewrite is
    // the only journaled operation here).
    let mut undone = false;
    for _ in 0..30 {
        term.send_keys("C-z").unwrap();
        let until = std::time::Instant::now() + xtask::pty::scaled(Duration::from_secs(2));
        while std::time::Instant::now() < until {
            undone = names(&repo) == original;
            if undone {
                break;
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        if undone {
            break;
        }
    }
    assert!(undone, "authors restored");
    assert_eq!(repo.git(&["rev-parse", "HEAD"]), tip);
}

/// The template nobody wrote is not trusted: the check line says so, the test
/// asks "Trust this script?" with two numbered answers and none preselected;
/// `2` leaves it, `1` trusts it and the test goes on.
#[test]
fn a_script_nobody_wrote_asks_for_trust_with_no_default() {
    if !have_template_shell() {
        eprintln!("skipped: no shell for the template on this machine");
        return;
    }
    let repo = fixture();
    let original = names(&repo);
    let (mut term, _config) = launch(&repo, "pty-rewrite-external-trust");
    open_shell_editor(&mut term);
    term.wait_for("the template, not trusted yet", STEP, |s| {
        s.contains("Not trusted yet")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("F5").unwrap();
    term.wait_for("the question with both answers", STEP, |s| {
        s.contains("Trust this script?")
            && s.contains("Trust it and go on")
            && s.contains("Do not trust it")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    // Enter alone does not answer: no default.
    term.send_keys("Enter").unwrap();
    term.wait_for("still asking", STEP, |s| s.contains("Trust this script?"))
        .unwrap_or_else(|e| panic!("{e}"));
    // 2: not trusted, nothing runs.
    term.send_keys("2").unwrap();
    term.wait_for("the question closed", STEP, |s| {
        !s.contains("Trust this script?") && s.contains("Test on selection")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(names(&repo), original);
    // Again, and 1: trusted, and the test that was refused runs.
    term.send_keys("F5").unwrap();
    term.wait_for("the question again", STEP, |s| {
        s.contains("Trust this script?")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("1").unwrap();
    term.wait_for("the test result, the question gone", STEP, |s| {
        s.contains("would change")
            && s.contains("author name: Old Name -> OLD NAME")
            && !s.contains("Trust this script?")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(names(&repo), original, "a test writes nothing");
}
