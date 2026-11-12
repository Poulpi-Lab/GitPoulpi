//! Settings > Repository > Message template, through a pty: the real binary,
//! daemon and git. A template saved on the page is a file in the repo data
//! directory that `commit.template` points at; with "apply" on an empty
//! composer starts with it, and "remove comments" drops its comment lines from
//! the commit that is made.
//!
//! No hard-coded coordinates: every position is found on the screen.

mod common;

use std::path::Path;
use std::time::{Duration, Instant};

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(30);

fn fixture() -> FixtureRepo {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "pty-template-initial")
        .unwrap()
        .build();
    std::fs::write(repo.root().join("a.txt"), "one\ntwo\n").unwrap();
    repo.git(&["add", "a.txt"]);
    repo
}

fn launch(repo: &Path, session: &str) -> (Terminal, tempfile::TempDir) {
    common::launch(repo)
        .session(session)
        .size(204, 60)
        .no_exit_pause()
        .ready_when("the fixture and its composer", |s| {
            s.contains("pty-template-initial") && s.contains("Skip hooks")
        })
        .mouse_capture()
        .start()
}

fn open_page(term: &mut Terminal, title: &str) {
    let (c, r) = term
        .wait_for_text("\u{2699}", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(c, r).expect("click the gear");
    term.wait_for("Settings open", STEP, |s| {
        s.contains("Repo-specific settings")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    common::settings_page(term, title);
}

fn close_settings(term: &mut Terminal) {
    common::retry_until(
        term,
        "Settings closed",
        6,
        Duration::from_secs(3),
        |t| t.send_keys("Escape").unwrap(),
        |s| !s.contains("Repo-specific settings"),
    );
}

fn click_text(term: &mut Terminal, text: &str) {
    let screen = term
        .wait_for(&format!("`{text}` on screen"), STEP, |s| s.contains(text))
        .unwrap_or_else(|e| panic!("{e}"));
    let (col, row) = screen.find(text).unwrap();
    term.click(col + 1, row).unwrap();
}

fn wait_for(term: &Terminal, what: &str, mut ok: impl FnMut() -> bool) {
    let deadline = Instant::now() + STEP;
    while !ok() {
        assert!(Instant::now() < deadline, "never: {what}\n{}", term.text());
        std::thread::sleep(Duration::from_millis(100));
    }
}

fn template_file(repo: &FixtureRepo) -> std::path::PathBuf {
    repo.root().join(".git/gitpoulpi/commit-template.txt")
}

/// Fills the page (summary, description with a comment line) and saves it.
fn save_a_template(term: &mut Terminal, repo: &FixtureRepo) {
    open_page(term, "Message template");
    term.wait_for("the template page", STEP, |s| {
        s.contains("Summary (72 left)") && s.contains("Save template")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    click_text(term, "Summary (72 left)");
    term.wait_for("the summary open", STEP, |s| s.contains("\u{2588}"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_text("feat: templated").unwrap();
    term.wait_for("the counter follows the typing", STEP, |s| {
        s.contains("Summary (57 left)")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();

    // A click on the field opens the multi-line editor.
    click_text(term, "Description");
    term.wait_for("the editor", STEP, |s| {
        s.contains("Save") && s.contains("Cancel")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_text("Why this change?").unwrap();
    term.send_keys("Enter").unwrap();
    term.send_text("# a comment line").unwrap();
    term.send_keys("C-s").unwrap();
    term.wait_for("the editor closed", STEP, |s| !s.contains("Cancel"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // A button row: the first click selects it, the second runs it.
    click_text(term, "Save template");
    click_text(term, "Save template");
    wait_for(term, "the template file", || template_file(repo).is_file());
    wait_for(term, "commit.template set to the file", || {
        // `try_git`: the key is unset until the save lands, and `git config`
        // exits 1 for an unset key (a polling panic, not a failure).
        repo.try_git(&["config", "--local", "commit.template"])
            .is_some_and(|v| {
                // git keeps the path as the app wrote it: compare the slash direction-blind.
                v.trim().replace('\\', "/")
                    == template_file(repo).to_string_lossy().replace('\\', "/")
            })
    });
}

/// Saving writes the file and sets `commit.template`; the composer is not
/// pre-filled until "apply" is ticked; then an empty composer starts with the
/// template, and with "remove comments" on the made commit has none.
#[test]
fn a_saved_template_prefills_the_composer_and_comments_are_stripped_from_the_commit() {
    let repo = fixture();
    let (mut term, _config) = launch(repo.root(), "pty-repo-template");
    save_a_template(&mut term, &repo);
    assert_eq!(
        std::fs::read_to_string(template_file(&repo)).unwrap(),
        "feat: templated\n\nWhy this change?\n# a comment line\n"
    );
    close_settings(&mut term);
    std::thread::sleep(Duration::from_millis(800));
    let screen = term.screen();
    assert!(
        !screen.contains("feat: templated"),
        "apply is off: the composer stays empty\n{}",
        screen.text()
    );

    // Tick both switches.
    open_page(&mut term, "Message template");
    term.wait_for("the template page", STEP, |s| {
        s.contains("Prefill an empty commit")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    click_text(&mut term, "Prefill an empty commit");
    click_text(&mut term, "Drop comment lines");
    wait_for(&term, "both switches saved", || {
        let state = std::fs::read_to_string(repo.root().join(".git/gitpoulpi/repo-state.toml"))
            .unwrap_or_default();
        state.contains("template_apply = true") && state.contains("template_strip_comments = true")
    });
    close_settings(&mut term);
    term.wait_for("the composer prefilled with the template", STEP, |s| {
        s.contains("feat: templated") && s.contains("Why this change?")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    let (c, r) = term
        .wait_for_text("Commit changes to 1 file", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(c + 1, r).expect("click Commit");
    wait_for(&term, "the commit", || {
        repo.git(&["log", "-1", "--format=%s"]).trim() == "feat: templated"
    });
    assert_eq!(
        repo.git(&["log", "-1", "--format=%B"]).trim(),
        "feat: templated\n\nWhy this change?",
        "the comment line was removed from the commit message"
    );
    // The panel starts with the template again for the next commit.
    term.wait_for("the template offered again", STEP, |s| {
        s.contains("feat: templated")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let _ = term.stop();
}

/// Without "remove comments" the comment line stays in the commit: the
/// switch, not the template, is what strips.
#[test]
fn without_remove_comments_the_comment_line_is_committed() {
    let repo = fixture();
    let state = repo.root().join(".git/gitpoulpi");
    std::fs::create_dir_all(&state).unwrap();
    std::fs::write(
        state.join("repo-state.toml"),
        "[preferences]\ntemplate_apply = true\n",
    )
    .unwrap();
    std::fs::write(
        state.join("commit-template.txt"),
        "fix: keep\n\n# kept comment\n",
    )
    .unwrap();
    let (mut term, _config) = launch(repo.root(), "pty-repo-template-keep");
    term.wait_for("the composer prefilled from the file", STEP, |s| {
        s.contains("fix: keep") && s.contains("# kept comment")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (c, r) = term
        .wait_for_text("Commit changes to 1 file", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(c + 1, r).expect("click Commit");
    wait_for(&term, "the commit", || {
        repo.git(&["log", "-1", "--format=%s"]).trim() == "fix: keep"
    });
    assert_eq!(
        repo.git(&["log", "-1", "--format=%B"]).trim(),
        "fix: keep\n\n# kept comment"
    );
    let _ = term.stop();
}
