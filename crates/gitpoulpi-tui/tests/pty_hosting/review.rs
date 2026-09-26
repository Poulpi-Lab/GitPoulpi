//! Slice R2 end to end: the in-app code review of merge request `!58` of the
//! shared `xtask mock-host` (a GitLab). The repository is real - its `origin`
//! is a local bare repository that holds the request's head as
//! `refs/merge-requests/58/head` - so the view really fetches the refs, asks
//! the daemon for the diff and the commits, and the host only answers the
//! API calls (threads, conversation, draft notes, approvals).

use std::time::Duration;

use crate::support::*;
use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::mock_host::{self, MockHost, Options};
use xtask::pty::Terminal;

pub(crate) fn wait_until(what: &str, mut ok: impl FnMut() -> bool) {
    let deadline = std::time::Instant::now() + STEP;
    while !ok() {
        assert!(std::time::Instant::now() < deadline, "timed out: {what}");
        std::thread::sleep(Duration::from_millis(50));
    }
}

/// `main` with a 30-line `src/planets.rs`; `feature` (pushed as the
/// request's head) replaces line 12 by two lines and adds `docs/moons.md`.
pub(crate) fn review_fixture() -> FixtureRepo {
    let base: String = (1..=30).map(|n| format!("// line {n}\n")).collect();
    let moons = base.replace("// line 12\n", "let moons = Vec::new();\nmoons.push(1);\n");
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("src/planets.rs", &base, "Start the planet table")
        .unwrap()
        .with_origin()
        .unwrap()
        .push("main")
        .unwrap()
        .branch("feature")
        .unwrap()
        .commit_file("src/planets.rs", &moons, "Add moons to the table")
        .unwrap()
        .commit_file("docs/moons.md", "# Moons\ntext\n", "Document the moons")
        .unwrap()
        .push("feature:refs/merge-requests/58/head")
        .unwrap()
        .build();
    // The PULL REQUESTS section reads the mock's `o/orbit`, which lists !58.
    let dir = repo.state_dir();
    std::fs::create_dir_all(&dir).expect("state dir");
    std::fs::write(
        dir.join("repo-state.toml"),
        "[pulls_source]\nintegration = \"corp\"\nproject = \"o/orbit\"\n",
    )
    .expect("seed repo-state.toml");
    repo
}

pub(crate) fn launch_review(
    mock: &MockHost,
    repo: &FixtureRepo,
    session: &str,
) -> (Terminal, tempfile::TempDir) {
    launch_review_with(mock, repo, session, "")
}

/// [`launch_review`] with `extra` appended to `config.toml` (the AI tests).
pub(crate) fn launch_review_with(
    mock: &MockHost,
    repo: &FixtureRepo,
    session: &str,
    extra: &str,
) -> (Terminal, tempfile::TempDir) {
    launch_unlinked(
        repo,
        session,
        &config_toml(mock.port, extra),
        COLS,
        "PULL REQUESTS (4)",
    )
}

/// The sidebar row of `!58` (the "All" group is folded open first; the
/// toolbar's `GL !58` chip is not the row).
pub(crate) fn row_of_58(term: &mut Terminal) -> (u16, u16) {
    let screen = term
        .wait_for("the merge requests listed", STARTUP, |s| {
            s.contains("All Pull Requests")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    if !screen.contains("!58  Add moons") {
        let (c, r) = screen.find("All Pull Requests").unwrap();
        term.click(c + 2, r).expect("unfold the All group");
    }
    let screen = term
        .wait_for("the !58 row", STEP, |s| s.contains("!58  Add moons"))
        .unwrap_or_else(|e| panic!("{e}"));
    screen.find("!58  Add moons").unwrap()
}

/// Opens the review of `!58` from its row's menu.
pub(crate) fn open_from_menu(term: &mut Terminal) {
    let (c, r) = row_of_58(term);
    term.right_click(c + 1, r).expect("the row menu");
    let screen = term
        .wait_for("the row menu offers the review", STEP, |s| {
            s.contains("Review...")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let (c, r) = screen.find("Review...").unwrap();
    term.click(c + 1, r).expect("Review");
    term.wait_for("the review view", STEP, |s| {
        s.contains("Review !58") && s.contains("Add moons to the planet table")
    })
    .unwrap_or_else(|e| panic!("{e}"));
}

/// The Files page with the diff and the threads in.
pub(crate) fn open_files(term: &mut Terminal) {
    term.send_keys("M-2").expect("Alt+2");
    term.wait_for("the diff and its thread", STEP, |s| {
        s.contains("let moons = Vec::new();") && s.contains("Why a Vec here and not a slice?")
    })
    .unwrap_or_else(|e| panic!("{e}"));
}

#[test]
fn a_merge_request_is_reviewed_in_the_app_and_the_pending_review_reaches_the_host() {
    let mock = mock_host::start(Options::default()).expect("the mock host");
    let repo = review_fixture();
    let (mut term, _config) = launch_review(&mock, &repo, "pty-review-flow");
    open_from_menu(&mut term);

    // Overview: header facts, the description, the conversation and the
    // verdicts submitted so far.
    let screen = term
        .wait_for("the overview loaded", STEP, |s| {
            s.contains("Looks promising, one question inline.") && s.contains("approved")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    for want in [
        "main \u{2190} feature/moons",
        "Adds the moons column",
        "Conversation (",
        "Finish review",
    ] {
        assert!(screen.contains(want), "{want}:\n{}", screen.text());
    }

    // Commits: the request's own commits, not the history behind it; Enter
    // shows the selected commit's diff.
    term.send_keys("M-3").expect("Alt+3");
    let screen = term
        .wait_for("the commits listed", STEP, |s| {
            s.contains("Add moons to the table") && s.contains("Document the moons")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(
        !screen.contains("Start the planet table"),
        "the target's history is not part of the request:\n{}",
        screen.text()
    );
    term.wait_for("the first commit's diff", STEP, |s| {
        s.contains("docs/moons.md")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    // Files: the diff from git with the host's thread under its line.
    open_files(&mut term);
    let screen = term.screen();
    assert!(screen.contains("src/planets.rs") && screen.contains("docs/moons.md"));
    let (_, code) = screen.find("let moons = Vec::new();").unwrap();
    let (_, thread) = screen.find("Why a Vec here and not a slice?").unwrap();
    assert!(
        thread > code && thread <= code + 3,
        "the thread sits under its line:\n{}",
        screen.text()
    );

    // A comment on a line is pending: held back, counted, not sent.
    let (c, r) = screen.find("moons.push(1);").unwrap();
    term.click(c + 2, r).expect("the second added line");
    term.send_keys("c").expect("comment");
    term.wait_for("the comment box", STEP, |s| {
        s.contains("Comment on code") && s.contains("src/planets.rs:13")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_text("needs a doc comment").unwrap();
    term.send_keys("C-s").expect("keep it");
    term.wait_for("one pending comment", STEP, |s| {
        s.contains("1 pending comment") && s.contains("Pending - line 13")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert!(
        !mock.saw("POST", "/draft_notes"),
        "a pending comment does not reach the host yet"
    );
    assert!(
        repo_state(&repo).contains("needs a doc comment"),
        "it is kept in the repository's state:\n{}",
        repo_state(&repo)
    );

    // Finish the review with Approve: GitLab offers Comment and Approve only.
    term.send_keys("f").expect("finish");
    let screen = term
        .wait_for("the finish dialog", STEP, |s| {
            s.contains("1 pending comment will be sent.")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(!screen.contains("Request changes"), "{}", screen.text());
    term.send_keys("Tab Enter").expect("open the verdict list");
    let screen = term
        .wait_for("the verdicts", STEP, |s| s.contains("Approve"))
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(!screen.contains("Request changes"), "{}", screen.text());
    term.send_keys("Down Enter").expect("choose Approve");
    term.wait_for("Approve chosen", STEP, |s| {
        s.text()
            .lines()
            .any(|l| l.contains("Verdict") && l.contains("Approve"))
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("C-s").expect("submit");
    wait_until("the host got the draft note and the approval", || {
        mock.saw("POST", "/draft_notes/bulk_publish") && mock.saw("POST", "/approve")
    });
    let draft = mock
        .requests()
        .into_iter()
        .find(|r| r.method == "POST" && r.target.ends_with("/draft_notes"))
        .expect("a draft note");
    assert!(draft.body.contains("needs a doc comment"), "{}", draft.body);
    assert!(draft.body.contains("\"new_line\":13"), "{}", draft.body);
    term.wait_for("the review reported", STEP, |s| {
        s.contains("review submitted with 1 comment") && !s.contains("1 pending comment")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert!(
        !repo_state(&repo).contains("needs a doc comment"),
        "nothing pending is left on disk"
    );

    // Esc leaves the view.
    term.send_keys("Escape").expect("close");
    term.wait_for("the view closed", STEP, |s| !s.contains("Review !58"))
        .unwrap_or_else(|e| panic!("{e}"));
}

#[test]
fn a_thread_is_resolved_with_its_button_and_reopened_with_the_key() {
    let mock = mock_host::start(Options::default()).expect("the mock host");
    let repo = review_fixture();
    let (mut term, _config) = launch_review(&mock, &repo, "pty-review-resolve");
    open_from_menu(&mut term);
    open_files(&mut term);
    let screen = term.screen();
    let (_, head) = screen.find("tomas \u{b7}").unwrap();
    let (col, _) = screen
        .find_all("Resolve")
        .into_iter()
        .find(|(_, r)| *r == head)
        .expect("the Resolve button on the thread's first row");
    term.click(col + 1, head).expect("Resolve");
    wait_until("the host resolved the thread", || {
        mock.requests().iter().any(|r| {
            r.method == "PUT"
                && r.target.contains("/discussions/d_open")
                && r.body.contains("\"resolved\":true")
        })
    });
    term.wait_for("the thread folded with a Reopen button", STEP, |s| {
        s.contains("Reopen") && s.contains("thread resolved")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    // The keyboard twin: the cursor is on the folded thread; `x` reopens it.
    let screen = term.screen();
    let (c, r) = screen.find("tomas: Why a Vec here").unwrap();
    term.click(c + 2, r).expect("the folded thread");
    term.send_keys("x").expect("reopen");
    wait_until("the host reopened the thread", || {
        mock.requests().iter().any(|r| {
            r.method == "PUT"
                && r.target.contains("/discussions/d_open")
                && r.body.contains("\"resolved\":false")
        })
    });
    term.wait_for("the thread open again", STEP, |s| {
        s.contains("thread reopened") && s.contains("It is built once at start-up.")
    })
    .unwrap_or_else(|e| panic!("{e}"));
}

#[test]
fn enter_on_a_row_and_the_palette_twins_open_and_drive_the_review() {
    let mock = mock_host::start(Options::default()).expect("the mock host");
    let repo = review_fixture();
    let (mut term, _config) = launch_review(&mock, &repo, "pty-review-palette");
    // Select the row, then Enter: the review opens (not the browser).
    let (c, r) = row_of_58(&mut term);
    term.click(c + 1, r).expect("select the row");
    term.send_keys("Enter").expect("activate");
    term.wait_for("the review view", STEP, |s| s.contains("Review !58"))
        .unwrap_or_else(|e| panic!("{e}"));

    // The palette reaches the view's commands.
    term.send_keys("C-p").expect("the palette");
    term.wait_for("the palette", STEP, |s| s.contains("Command Palette"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_text("Review: Show Files").unwrap();
    term.wait_for("the command", STEP, |s| s.contains("Review: Show Files"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").expect("run it");
    term.wait_for("the Files page", STEP, |s| {
        s.contains("Files (2)") && s.contains("src/planets.rs")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    term.send_keys("C-p").expect("the palette");
    term.wait_for("the palette", STEP, |s| s.contains("Command Palette"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_text("Review: Finish Review").unwrap();
    term.wait_for("the command", STEP, |s| {
        s.contains("Review: Finish Review...")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").expect("run it");
    term.wait_for("the finish dialog", STEP, |s| {
        s.contains("Summary (optional)") && s.contains("Submit review")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    // Nothing to send yet: Submit explains instead of calling the host.
    term.send_keys("C-s").expect("submit");
    term.wait_for("the reason", STEP, |s| {
        s.contains("add a summary or a pending comment first")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert!(!mock.saw("POST", "/draft_notes"));
    term.send_keys("Escape").expect("leave the dialog");
    term.wait_for("the dialog closed", STEP, |s| {
        !s.contains("Summary (optional)")
    })
    .unwrap_or_else(|e| panic!("{e}"));
}
