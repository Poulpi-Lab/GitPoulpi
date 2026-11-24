#![cfg(all(feature = "radar", feature = "review"))]
//! Reviewing a request from the Radar when its repository is not cloned --
//! the real binary and daemon, driven through a pty by `xtask::pty`, against
//! `xtask mock-host` (a `TcpListener` on 127.0.0.1; no other network).
//!
//! One GitLab profile (`lab`) with its review requests on: the mock lists
//! `!58` of `northwind/billing` ("Round invoice totals") as waiting for the
//! user's review, in a repository that exists nowhere on this machine.
//!
//! * `e`, `a`: the review opens in the Home window over the Radar, the code
//!   comes from the host's API, a pending comment is kept and submitted, and
//!   Esc gives the Radar back.
//! * `e`, `c`: the Clone dialog opens prefilled, the clone (a local bare
//!   repository the mock seeds) lands, the repository's tab opens and the
//!   review of the request is open in it.
//! * An Azure DevOps request has no in-app review: Review opens its page.

#![allow(dead_code)]

#[path = "common/mod.rs"]
mod common;

use std::path::Path;
use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::mock_host::{self, MockHost, Options};
use xtask::pty::{Screen, Terminal};

const STARTUP: Duration = Duration::from_secs(60);
const STEP: Duration = Duration::from_secs(30);
const COLS: u16 = 160;
const ROWS: u16 = 50;
const ROW: &str = "Round invoice totals";

struct Setup {
    term: Terminal,
    mock: MockHost,
    config: tempfile::TempDir,
    clone_root: tempfile::TempDir,
    bare: tempfile::TempDir,
    _repo: FixtureRepo,
}

/// The mock (with bare repositories to clone from), a `lab` profile with
/// `profile` as its extra lines, and the TUI on a fixture repository (the
/// only tab).
fn setup(session: &str, kind: &str, profile: &str) -> Setup {
    setup_at(session, kind, profile, COLS)
}

/// [`setup`] in a window `cols` wide.
fn setup_at(session: &str, kind: &str, profile: &str, cols: u16) -> Setup {
    let bare = tempfile::tempdir().unwrap();
    mock_host::seed_bare_repos(bare.path()).unwrap();
    // The request's head, where the review of a clone fetches it from.
    let billing = bare.path().join("northwind/billing.git");
    let git = |args: &[&str]| {
        let out = std::process::Command::new("git")
            .arg("-C")
            .arg(&billing)
            .args(args)
            .output()
            .expect("git");
        assert!(out.status.success(), "{args:?}: {out:?}");
    };
    git(&["update-ref", "refs/merge-requests/58/head", "main"]);
    let mock = mock_host::start(Options {
        port: 0,
        bare_root: Some(bare.path().to_path_buf()),
    })
    .expect("the mock host");
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "a\n", "radar-review-start-commit")
        .unwrap()
        .build();
    let clone_root = tempfile::tempdir().unwrap();
    let config = tempfile::tempdir().expect("temp config dir");
    std::fs::write(
        config.path().join("config.toml"),
        format!(
            "[repos]\nclone_root = '{root}'\n\n[[integrations]]\nid = \"lab\"\nkind = \"{kind}\"\nurl = \"http://127.0.0.1:{port}\"\ntoken_ref = \"integration:lab\"\nname = \"lab\"\nuser = {{ login = \"{login}\" }}\n{profile}",
            root = clone_root.path().display(),
            port = mock.port,
            login = mock_host::GITLAB_LOGIN,
        ),
    )
    .expect("seed config.toml");
    std::fs::create_dir_all(config.path().join("sessions")).unwrap();
    std::fs::write(
        config
            .path()
            .join("sessions")
            .join(format!("{session}.toml")),
        format!(
            "[[tabs]]\nrepo_path = '{}'\nactive = true\n",
            repo.root().display()
        ),
    )
    .expect("seed the session");
    let term = common::launch(repo.root())
        .session(session)
        .config_dir(config.path())
        .size(cols, ROWS)
        .mouse_capture()
        .env("GITPOULPI_NO_BROWSER", "1")
        .env("GITPOULPI_OPEN_LOG", config.path().join("opened.log"))
        .env("GITPOULPI_TOKEN_LAB", "test-token")
        .env("GITPOULPI_DESKTOP_NO_HOOKS", "1")
        .spawn();
    term.wait_for("the repository loaded", STARTUP, |s| {
        s.contains("radar-revi")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    Setup {
        term,
        mock,
        config,
        clone_root,
        bare,
        _repo: repo,
    }
}

const GITLAB: &str = "gitlab_self_managed";
const LISTS: &str = "\n[integrations.radar]\nreview_requests = true\n";

/// The Radar, with the far request listed and selected.
fn select_request(term: &mut Terminal) -> Screen {
    term.send_keys("M-r").expect("Alt+R");
    let screen = term
        .wait_for("the far request", STARTUP, |s| {
            s.contains("Radar") && s.contains(ROW) && s.contains("Not cloned")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let (col, row) = screen.find(ROW).unwrap();
    term.click(col + 2, row).expect("select the row");
    screen
}

/// The badge character on the left edge of the Cancel button (it covers the
/// first letters of the label, so the tail is what is looked for).
fn cancel_badge(screen: &Screen) -> Option<char> {
    let (col, row) = screen.find("ncel")?;
    let line: Vec<char> = screen.line(row).chars().collect();
    let col = usize::from(col).min(line.len());
    (col.saturating_sub(5)..col).rev().find_map(|i| {
        (i + 2 < line.len()
            && line[i] == ' '
            && line[i + 2] == ' '
            && line[i + 1].is_ascii_alphanumeric())
        .then_some(line[i + 1])
    })
}

fn wait_until(what: &str, mut ok: impl FnMut() -> bool) {
    let deadline = std::time::Instant::now() + STEP;
    while !ok() {
        assert!(std::time::Instant::now() < deadline, "timed out: {what}");
        std::thread::sleep(Duration::from_millis(50));
    }
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_default()
}

/// The question, then the API: the review opens in the Home window over the
/// Radar, a pending comment is kept in `remote-reviews.toml`, submitting it
/// reaches the host, and Esc gives the Radar back.
#[test]
fn review_through_the_api_opens_over_the_radar_and_submits_to_the_host() {
    let mut s = setup("pty-radar-review-api", GITLAB, LISTS);
    select_request(&mut s.term);
    s.term.send_keys("e").expect("e");
    let screen = s
        .term
        .wait_for("the question", STEP, |s| {
            // The whole dialog: a frame still being painted holds its first
            // rows only.
            s.contains("Review request")
                && s.contains("Review through the API")
                && s.contains("Clone it, then review")
                && s.contains("Remember for")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(
        screen.contains("Clone it, then review"),
        "{}",
        screen.text()
    );
    assert!(screen.contains("Remember for"), "{}", screen.text());

    // `Ctrl+N` badges its controls like every dialog: the badge of Cancel
    // closes the question.
    s.term.send_keys("C-n").expect("Ctrl+N");
    let badge = s
        .term
        .wait_for("a badge on Cancel", STEP, |s| cancel_badge(s).is_some())
        .unwrap_or_else(|e| panic!("{e}"));
    let badge = cancel_badge(&badge).expect("badge");
    s.term.send_text(&badge.to_string()).expect("the badge");
    s.term
        .wait_for("the question closed", STEP, |s| {
            !s.contains("Review request") && s.contains(ROW)
        })
        .unwrap_or_else(|e| panic!("{e}"));
    s.term.send_keys("e").expect("e again");
    s.term
        .wait_for("the question again", STEP, |s| s.contains("Review request"))
        .unwrap_or_else(|e| panic!("{e}"));

    s.term.send_keys("a").expect("a");
    let screen = s
        .term
        .wait_for("the review over the Radar", STEP, |s| {
            s.contains("Review !58") && s.contains("Add moons to the planet table")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(
        !screen.contains("Repository northwind/billing"),
        "the question is gone"
    );

    // Files: the diff comes from the host. Put the cursor on an added line
    // and keep a comment on it.
    s.term.send_keys("M-2").expect("Alt+2");
    let screen = s
        .term
        .wait_for("the diff", STEP, |s| {
            s.contains("src/planets.rs") && s.contains("+b")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let (col, row) = screen
        .find("+b")
        .unwrap_or_else(|| panic!("an added line:\n{}", screen.text()));
    s.term.click(col + 4, row).expect("the added line");
    s.term.send_keys("c").expect("comment");
    s.term
        .wait_for("the comment box", STEP, |s| s.contains("Comment on code"))
        .unwrap_or_else(|e| panic!("{e}"));
    s.term.send_text("check the rounding").unwrap();
    s.term.send_keys("C-s").expect("keep it");
    s.term
        .wait_for("one pending comment", STEP, |s| {
            s.contains("1 pending comment")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(!s.mock.saw("POST", "/draft_notes"), "held back");
    let kept = read(&s.config.path().join("remote-reviews.toml"));
    assert!(kept.contains("check the rounding"), "{kept}");

    // The palette reaches the view's commands from the Home window.
    s.term.send_keys("C-p").expect("the palette");
    s.term
        .wait_for("the palette", STEP, |s| s.contains("Command Palette"))
        .unwrap_or_else(|e| panic!("{e}"));
    s.term.send_text("Review: Finish Review").unwrap();
    s.term
        .wait_for("the command", STEP, |s| {
            s.contains("Review: Finish Review...")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    s.term.send_keys("Enter").expect("run it");
    s.term
        .wait_for("the finish dialog", STEP, |s| {
            s.contains("1 pending comment will be sent.")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    s.term.send_keys("C-s").expect("submit");
    wait_until("the host got the review", || {
        s.mock.saw("POST", "/draft_notes/bulk_publish")
    });
    let note = s
        .mock
        .requests()
        .into_iter()
        .find(|r| r.method == "POST" && r.target.ends_with("/draft_notes"))
        .expect("a draft note");
    assert!(note.body.contains("check the rounding"), "{}", note.body);
    assert!(
        note.target.contains("northwind%2Fbilling") && note.target.contains("/58/"),
        "{}",
        note.target
    );
    s.term
        .wait_for("the review reported", STEP, |s| {
            s.contains("review submitted") && !s.contains("1 pending comment")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(
        !read(&s.config.path().join("remote-reviews.toml")).contains("check the rounding"),
        "nothing pending is left on disk"
    );

    // Esc: the Radar is back, with its row.
    s.term.send_keys("Escape").expect("close");
    s.term
        .wait_for("the Radar again", STEP, |s| {
            !s.contains("Review !58") && s.contains("Scope") && s.contains(ROW)
        })
        .unwrap_or_else(|e| panic!("{e}"));
}

/// Remember writes the profile's mode and the next Review skips the question.
#[test]
fn remember_for_the_profile_skips_the_question_next_time() {
    let mut s = setup("pty-radar-review-remember", GITLAB, LISTS);
    select_request(&mut s.term);
    s.term.send_keys("e").expect("e");
    s.term
        .wait_for("the question", STEP, |s| s.contains("Review request"))
        .unwrap_or_else(|e| panic!("{e}"));
    s.term.send_keys("r").expect("remember");
    s.term
        .wait_for("the box checked", STEP, |s| s.contains("[x] Remember for"))
        .unwrap_or_else(|e| panic!("{e}"));
    s.term.send_keys("a").expect("a");
    s.term
        .wait_for("the review", STEP, |s| s.contains("Review !58"))
        .unwrap_or_else(|e| panic!("{e}"));
    let saved = read(&s.config.path().join("config.toml"));
    assert!(saved.contains("review_mode = \"api\""), "{saved}");
    assert!(saved.contains("review_requests = true"), "{saved}");
    s.term.send_keys("Escape").expect("close");
    s.term
        .wait_for("the Radar again", STEP, |s| {
            !s.contains("Review !58") && s.contains(ROW)
        })
        .unwrap_or_else(|e| panic!("{e}"));
    s.term.send_keys("e").expect("e again");
    let screen = s
        .term
        .wait_for("the review without a question", STEP, |s| {
            s.contains("Review !58")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(!screen.contains("Review request"), "{}", screen.text());
}

/// Clone it, then review: the Clone dialog opens prefilled, the clone lands,
/// the repository is linked to the host and its tab opens on the review.
#[test]
fn clone_then_review_opens_the_new_tab_on_the_request() {
    let mut s = setup("pty-radar-review-clone", GITLAB, LISTS);
    select_request(&mut s.term);
    s.term.send_keys("e").expect("e");
    s.term
        .wait_for("the question", STEP, |s| s.contains("Review request"))
        .unwrap_or_else(|e| panic!("{e}"));
    s.term.send_keys("c").expect("c");
    // The connected-account mode of the profile the row came from, the
    // request's project selected (the check mark) and the folder filled.
    // The mock lists its bare repositories with file:// clone addresses, so
    // the clone goes through the integration's own path.
    let screen = s
        .term
        .wait_for("the Clone dialog, on the account", STEP, |s| {
            s.contains("Clone a repository")
                && s.contains("Full path:")
                && s.contains("northwind/billing")
                && s.contains("\u{2713}")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(
        screen.contains("the review of !58 opens"),
        "{}",
        screen.text()
    );
    assert!(
        !screen.contains("northwind/billing.git"),
        "no URL prefill: {}",
        screen.text()
    );
    s.term.send_keys("Enter").expect("clone");
    let screen = s
        .term
        .wait_for("the new tab on the request", STARTUP, |s| {
            s.contains("Review !58") && s.contains("Add moons to the planet table")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(!screen.contains("Clone a repository"), "{}", screen.text());
    let state = s
        .clone_root
        .path()
        .join("billing")
        .join(".git")
        .join("gitpoulpi")
        .join("repo-state.toml");
    let written = read(&state);
    assert!(
        written.contains("[pulls_source]")
            && written.contains("integration = \"lab\"")
            && written.contains("project = \"northwind/billing\""),
        "{written}"
    );
    assert!(written.contains("[issues_source]"), "{written}");
}

/// Esc in the Clone dialog forgets the request: the next ordinary clone is
/// not linked to anything.
#[test]
fn canceling_the_clone_dialog_leaves_nothing_behind() {
    let mut s = setup("pty-radar-review-cancel", GITLAB, LISTS);
    select_request(&mut s.term);
    s.term.send_keys("e").expect("e");
    s.term
        .wait_for("the question", STEP, |s| s.contains("Review request"))
        .unwrap_or_else(|e| panic!("{e}"));
    s.term.send_keys("c").expect("c");
    s.term
        .wait_for("the Clone dialog", STEP, |s| {
            s.contains("Clone a repository")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    s.term.send_keys("Escape").expect("cancel");
    let screen = s
        .term
        .wait_for("the Radar again", STEP, |s| {
            !s.contains("Clone a repository") && s.contains(ROW)
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(!screen.contains("Review !58"));
    assert!(!s.clone_root.path().join("billing").exists());
}

/// A host with no in-app review (Azure DevOps): Review opens the request's
/// page and says so.
#[test]
fn a_request_of_a_host_without_a_review_view_opens_the_browser() {
    let mut s = setup("pty-radar-review-browser", "azure_devops_server", LISTS);
    s.term.send_keys("M-r").expect("Alt+R");
    let screen = s
        .term
        .wait_for("the Azure request", STARTUP, |s| {
            s.contains("Cache the token lookups")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let (col, row) = screen.find("Cache the token lookups").unwrap();
    s.term.click(col + 2, row).expect("select the row");
    s.term.send_keys("e").expect("e");
    let screen = s
        .term
        .wait_for("the message", STEP, |s| {
            s.contains("no in-app review for this host")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(!screen.contains("Review request"), "no question either");
    let opened = read(&s.config.path().join("opened.log"));
    assert!(opened.contains("311"), "{opened}");
}

/// Opens the review question over the Radar.
fn open_question(term: &mut Terminal) {
    select_request(term);
    term.send_keys("e").expect("e");
    term.wait_for("the question", STEP, |s| {
        s.contains("Review request") && s.contains("Remember for")
    })
    .unwrap_or_else(|e| panic!("{e}"));
}

/// The cell of the toolbar's Back button.
fn back_button(screen: &Screen) -> (u16, u16) {
    let (_, row) = screen
        .find_all("Refresh")
        .into_iter()
        .find(|&(_, r)| screen.line(r).contains("Back"))
        .unwrap_or_else(|| panic!("the toolbar:\n{}", screen.text()));
    let line = screen.line(row);
    let at = line.rfind("Back").unwrap();
    (line[..at].chars().count() as u16 + 1, row)
}

/// A left press outside the question closes it and does not run the control
/// under it (Back would leave the Radar).
#[test]
fn the_review_question_closes_on_a_press_outside_without_running_it() {
    let mut s = setup("pty-radar-review-away-left", GITLAB, LISTS);
    open_question(&mut s.term);
    let (c, r) = back_button(&s.term.screen());
    s.term.click(c, r).unwrap();
    s.term
        .wait_for("the question gone, still the Radar", STEP, |s| {
            !s.contains("Review request") && s.contains(ROW) && s.contains("Not cloned")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
}

/// A right-click outside the question closes it too, and opens no row menu.
#[test]
fn the_review_question_closes_on_a_right_click_outside() {
    let mut s = setup("pty-radar-review-away-right", GITLAB, LISTS);
    open_question(&mut s.term);
    // The request's own row: a right-click there would open its menu.
    let (c, r) = s.term.screen().find(ROW).unwrap();
    s.term.right_click(c + 2, r).unwrap();
    s.term
        .wait_for("the question gone, no row menu", STEP, |s| {
            !s.contains("Review request") && s.contains(ROW) && !s.contains("Pin to top")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
}

/// Every `OSC 22` pointer shape the program asked for, in order ("" is the
/// reset).
fn pointer_shapes(term: &Terminal) -> Vec<String> {
    let raw = String::from_utf8_lossy(&term.raw_output()).to_string();
    raw.split("\x1b]22;")
        .skip(1)
        .map(|s| s.split("\x1b\\").next().unwrap_or("").to_string())
        .collect()
}

/// Moves onto `(col, row)` and waits for `want` to be the last shape asked.
fn hover_shape(term: &mut Terminal, what: &str, (col, row): (u16, u16), want: &str) {
    term.move_to(col, row).unwrap();
    wait_until(what, || {
        pointer_shapes(term).last().map(String::as_str) == Some(want)
    });
}

/// Where `needle` is on the request's own row.
fn on_row(screen: &Screen, needle: &str) -> (u16, u16) {
    let (_, row) = screen.find(ROW).expect("the request's row");
    screen
        .find_all(needle)
        .into_iter()
        .find(|&(_, r)| r == row)
        .unwrap_or_else(|| panic!("{needle:?} on the row:\n{}", screen.text()))
}

const GLOBE: &str = "\u{1F310}";
const TALK: &str = "\u{2261}";

/// The Actions column of a request not cloned: Review, the globe and the
/// conversation. Each is a hand; Review asks how, the globe opens the page
/// (the stubbed browser logs it), the conversation opens over the Radar.
#[test]
fn the_actions_buttons_review_open_the_page_and_open_the_conversation() {
    let mut s = setup("pty-radar-actions", GITLAB, LISTS);
    let screen = select_request(&mut s.term);
    assert!(screen.contains("Actions"), "{}", screen.text());
    let screen = s.term.screen();
    let review = on_row(&screen, "Review");
    let globe = on_row(&screen, GLOBE);
    let talk = on_row(&screen, TALK);
    // The hand over each button, the arrow back on the title.
    hover_shape(&mut s.term, "a hand over Review", review, "pointer");
    hover_shape(
        &mut s.term,
        "the arrow on the title",
        (review.0 - 40, review.1),
        "",
    );
    hover_shape(&mut s.term, "a hand over the globe", globe, "pointer");
    hover_shape(&mut s.term, "a hand over the conversation", talk, "pointer");

    // The globe: the page of the request is opened (never a real browser).
    s.term.click(globe.0, globe.1).unwrap();
    let log = s.config.path().join("opened.log");
    wait_until("the page was opened", || read(&log).contains("58"));

    // Review: the question.
    s.term.click(review.0, review.1).unwrap();
    s.term
        .wait_for("the review question", STEP, |s| {
            s.contains("Review request")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
    s.term.send_keys("Escape").unwrap();
    s.term
        .wait_for("the Radar again", STEP, |s| {
            !s.contains("Review request") && s.contains("Scope")
        })
        .unwrap_or_else(|e| panic!("{e}"));

    // The conversation takes the window over.
    s.term.click(talk.0, talk.1).unwrap();
    s.term
        .wait_for("the conversation over the Radar", STEP, |s| {
            !s.contains("Scope")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
}

/// Narrow, the column stays: its word becomes a glyph, the globe and the
/// conversation stay, and the buttons still work.
#[test]
fn the_actions_column_keeps_its_buttons_as_glyphs_when_narrow() {
    let mut s = setup_at("pty-radar-actions-narrow", GITLAB, LISTS, 70);
    select_request(&mut s.term);
    let screen = s.term.screen();
    let globe = on_row(&screen, GLOBE);
    on_row(&screen, TALK);
    let (rev_col, rev_row) = on_row(&screen, "\u{25CE}");
    assert!(
        !screen.line(rev_row).contains("Review"),
        "the word is a glyph:\n{}",
        screen.text()
    );
    s.term.click(globe.0, globe.1).unwrap();
    let log = s.config.path().join("opened.log");
    wait_until("the page was opened", || read(&log).contains("58"));
    s.term.click(rev_col, rev_row).unwrap();
    s.term
        .wait_for("the review question", STEP, |s| {
            s.contains("Review request")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
}

/// A double click on a request not cloned asks how to review it (the menu's
/// and the button's question) instead of saying to clone first; so does
/// Enter on the row.
#[test]
fn a_double_click_or_enter_on_a_request_not_cloned_asks_how_to_review_it() {
    let mut s = setup("pty-radar-double-click", GITLAB, LISTS);
    let screen = select_request(&mut s.term);
    let (col, row) = screen.find(ROW).unwrap();
    // Past the double-click window of the click that selected the row.
    std::thread::sleep(Duration::from_millis(900));
    s.term.double_click(col + 2, row).unwrap();
    let screen = s
        .term
        .wait_for("the review question", STEP, |s| {
            s.contains("Review request")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
    assert!(
        !screen.contains("clone the repository first"),
        "{}",
        screen.text()
    );
    s.term.send_keys("Escape").unwrap();
    s.term
        .wait_for("the question closed", STEP, |s| {
            !s.contains("Review request")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    s.term.send_keys("Enter").unwrap();
    let screen = s
        .term
        .wait_for("the review question from Enter", STEP, |s| {
            s.contains("Review request")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
    assert!(
        !screen.contains("clone the repository first"),
        "{}",
        screen.text()
    );
}

/// The buttons of a review hosted over the Radar are hands, like the buttons
/// of every other surface: its tabs and its close cross.
#[test]
fn the_buttons_of_a_review_over_the_radar_show_the_pointing_hand() {
    let mut s = setup("pty-radar-review-pointer", GITLAB, LISTS);
    select_request(&mut s.term);
    s.term.send_keys("e").unwrap();
    s.term
        .wait_for("the question", STEP, |s| {
            s.contains("Review through the API")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    s.term.send_keys("a").unwrap();
    let screen = s
        .term
        .wait_for("the review over the Radar", STEP, |s| {
            s.contains("Review !58") && s.contains("Files")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    // The tab bar sits in the top border of the review's frame.
    let tab = screen
        .find_all("Files")
        .into_iter()
        .min_by_key(|&(_, r)| r)
        .expect("the Files tab");
    hover_shape(
        &mut s.term,
        "a hand over the Files tab",
        (tab.0 + 1, tab.1),
        "pointer",
    );
    // Off any button: the arrow again.
    hover_shape(&mut s.term, "the arrow on the page", (tab.0, tab.1 + 3), "");
}
