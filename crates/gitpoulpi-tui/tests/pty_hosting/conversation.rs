//! The conversation view of an issue, end to end: the real binary and daemon
//! through a pty, against the local mock of a GitLab (`mock.rs`) that serves
//! issues `#12` and `#7` of `g/p` with their discussions. Nothing is typed
//! into the status line, nothing is posted, and no browser is ever launched
//! (`GITPOULPI_NO_BROWSER`, its log is the stub).

use std::time::Duration;

use crate::mock::*;
use crate::support::*;
use xtask::pty::{Screen, Terminal};

const TITLE_12: &str = "#12  Crash on startup";

/// The routes of the conversation: `#12` (a description with a bold word, a
/// reference to `#7`, a web address; one comment and one label event) and `#7`.
pub fn route(method: &str, path: &str) -> Option<(u16, String)> {
    if method != "GET" {
        return None;
    }
    let body = match path {
        "/api/v4/projects/g%2Fp/issues/12" => r#"{"id":112,"iid":12,"title":"Crash on startup",
            "state":"opened","description":"Opening a repo **freezes** when files are locked.\n\nSame as #7, see https://example.org/docs and more.\n\n- [x] reproduced on Linux",
            "author":{"username":"ann","name":"Ann Archer"},"created_at":"2026-11-12T09:14:00Z",
            "updated_at":"2026-11-12T09:14:00Z","web_url":"http://127.0.0.1/g/p/-/issues/12","labels":["bug","lfs"]}"#,
        "/api/v4/projects/g%2Fp/issues/12/discussions" => r#"[
            {"id":"dn1","individual_note":true,"notes":[{"id":201,"body":"Seen on Linux too, really.",
              "author":{"username":"bo","name":"Bo Brandt"},"created_at":"2026-11-12T10:30:00Z",
              "updated_at":"2026-11-12T10:30:00Z","system":false}]},
            {"id":"dn2","individual_note":true,"notes":[{"id":202,"body":"added ~lfs label",
              "author":{"username":"bo","name":"Bo Brandt"},"created_at":"2026-11-12T10:31:00Z","system":true}]}]"#,
        "/api/v4/projects/g%2Fp/issues/7" => r#"{"id":107,"iid":7,"title":"Locked files",
            "state":"opened","description":"Locks are not released.",
            "author":{"username":"kai","name":"Kai"},"created_at":"2026-11-01T09:00:00Z",
            "web_url":"http://127.0.0.1/g/p/-/issues/7","labels":[]}"#,
        "/api/v4/projects/g%2Fp/issues/7/discussions" => "[]",
        _ => return None,
    };
    Some((200, body.to_string()))
}

/// The ISSUES section with `#12` listed.
fn launch(session: &str, cols: u16) -> (Terminal, tempfile::TempDir, Mock, gitpoulpi_testkit::FixtureRepo) {
    let mock = start_mock();
    let repo = fixture(mock.port);
    let (term, config) = launch_issues(
        &repo,
        session,
        &config_toml(mock.port, ""),
        cols,
        "Crash on startup",
    );
    (term, config, mock, repo)
}

/// Selects the row of `#12` in the sidebar and presses Enter on it.
fn open_with_enter(term: &mut Terminal) {
    let screen = term.screen();
    let (col, row) = screen
        .find_all("Crash on startup")
        .into_iter()
        .next()
        .expect("the issue row");
    term.click(col + 2, row).expect("select the row");
    term.send_keys("Enter").expect("Enter");
    wait_open(term);
}

fn wait_open(term: &mut Terminal) -> Screen {
    term.wait_for("the conversation", STEP, |s| {
        s.contains(TITLE_12) && s.contains("freezes") && s.contains("Bo Brandt")
    })
    .unwrap_or_else(|e| panic!("{e}"))
}

/// The badge character drawn left of `label` (a button's badge is on its own
/// left edge), on the row that holds it.
fn badge_before(screen: &Screen, at: (u16, u16)) -> Option<char> {
    let (col, row) = at;
    let line: Vec<char> = screen.line(row).chars().collect();
    let col = usize::from(col).min(line.len());
    (col.saturating_sub(7)..col).rev().find_map(|i| {
        (i + 2 < line.len()
            && line[i] == ' '
            && line[i + 2] == ' '
            && line[i + 1].is_ascii_alphanumeric())
        .then(|| line[i + 1])
    })
}

/// Where the tail of a label is: a badge covers the first letters of its
/// control, so the label is found by what it leaves visible. `last` takes the
/// lowest occurrence on the screen (the composer's buttons), else the first.
fn tail(label: &'static str, last: bool) -> impl Fn(&Screen) -> Option<(u16, u16)> {
    move |s| {
        let all = s.find_all(label);
        if last {
            all.into_iter().max_by_key(|(_, r)| *r)
        } else {
            all.into_iter().next()
        }
    }
}

/// `Ctrl+N`, then the badge drawn before the control `locate` finds.
fn press_badge(term: &mut Terminal, locate: impl Fn(&Screen) -> Option<(u16, u16)>) {
    term.send_keys("C-n").unwrap();
    term.wait_for("the badge on the control", STEP, |s| {
        locate(s).and_then(|at| badge_before(s, at)).is_some()
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let screen = term.screen();
    let at = locate(&screen).expect("the control");
    let badge = badge_before(&screen, at).expect("badge");
    term.send_text(&badge.to_string()).unwrap();
}

/// PTY 1: Enter on an issue row opens the conversation in the app, the
/// formatting is drawn, `#7` opens in the view and Back returns, and a typed
/// web address asks before anything opens.
#[test]
fn enter_on_an_issue_opens_its_conversation_and_links_work() {
    let (mut term, config, mock, _repo) = launch("pty-conv-open", COLS);
    open_with_enter(&mut term);
    let screen = term.screen();
    assert!(
        screen.contains("Posting as kai (personal token)"),
        "who posts is shown: {}",
        screen.text()
    );
    assert!(screen.contains("Ann Archer"), "{}", screen.text());
    assert!(screen.contains("Bo Brandt added the label lfs"), "{}", screen.text());
    assert!(
        !screen.contains("**freezes**"),
        "Markdown is rendered, not shown: {}",
        screen.text()
    );
    assert!(mock.saw("GET", "issues/12/discussions"), "{:?}", mock.requests());
    assert!(
        opened_urls(&config).is_empty(),
        "Enter reads the issue in the app, never in the browser"
    );

    // `#7` is a link that opens in the view.
    let (col, row) = term
        .screen()
        .find_all("#7")
        .into_iter()
        .find(|(_, r)| *r > 3)
        .expect("the #7 reference in the text");
    term.click(col, row).expect("click #7");
    term.wait_for("issue #7", STEP, |s| {
        s.contains("#7  Locked files") && s.contains("Locks are not released.")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(mock.saw("GET", "issues/7"), "{:?}", mock.requests());

    // Back (Alt+Left) returns to `#12`.
    term.send_keys("M-Left").unwrap();
    term.wait_for("back on #12", STEP, |s| {
        s.contains(TITLE_12) && !s.contains("Locks are not released.")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // A web address in the text asks first, showing the whole address.
    let (col, row) = term
        .screen()
        .find("https://example.org/docs")
        .expect("the address in the text");
    term.click(col + 2, row).expect("click the address");
    term.wait_for("the confirmation", STEP, |s| {
        s.contains("Open this link?") && s.contains("https://example.org/docs")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(opened_urls(&config).is_empty(), "nothing opened before the answer");
    term.send_keys("Escape").unwrap();
    term.wait_for("the question closed", STEP, |s| !s.contains("Open this link?"))
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(opened_urls(&config).is_empty());

    // The close cross returns to the repository.
    let screen = term.screen();
    let (col, row) = screen
        .find_all("\u{2715}")
        .into_iter()
        .find(|(_, r)| *r <= 2)
        .expect("the close cross on the frame");
    term.click(col + 1, row).expect("click the cross");
    term.wait_for("the view closed", STEP, |s| !s.contains("Posting as"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

/// PTY 6: `Ctrl+N` badges reach Reply, a link and Send (greyed: it says why).
#[test]
fn ctrl_n_badges_reach_reply_a_link_and_send() {
    let (mut term, config, _mock, _repo) = launch("pty-conv-badges", COLS);
    open_with_enter(&mut term);
    // Select Bo's comment (the description is first, then the comment).
    term.send_keys("Down").unwrap();
    term.wait_for("the Reply button on the selected entry", STEP, |s| {
        s.contains("Reply")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    // Reply: its badge opens the composer with the entry quoted.
    // (A badge covers the first letters of its control: found by its tail.)
    press_badge(&mut term, tail("ply", false));
    term.wait_for("the quote in the composer", STEP, |s| {
        s.contains("> **Bo Brandt**") && s.contains("> Seen on Linux too")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Escape").unwrap();
    term.wait_for_stable(Duration::from_millis(400), STEP).unwrap();

    // A link: the badge before the address opens the question.
    press_badge(&mut term, tail("ps://example.org/docs", false));
    term.wait_for("the confirmation", STEP, |s| s.contains("Open this link?"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Escape").unwrap();
    term.wait_for("the question closed", STEP, |s| !s.contains("Open this link?"))
        .unwrap_or_else(|e| panic!("{e}"));

    // Send: greyed, and it says why.
    press_badge(&mut term, tail("nd", true));
    term.wait_for("the reason", STEP, |s| {
        s.contains("writing from this view is not available yet")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(opened_urls(&config).is_empty());
}

/// PTY 7: at 70 columns Split is refused with its reason and Preview works.
#[test]
fn at_70_columns_split_is_refused_and_preview_works() {
    let (mut term, _config, _mock, _repo) = launch("pty-conv-narrow", COLS);
    open_with_enter(&mut term);
    term.resize(70, ROWS).expect("shrink the window");
    term.wait_for("the narrow view", STEP, |s| {
        s.contains("Posting as") && s.contains("New comment")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // New comment, then Alt+L: refused with the reason.
    term.send_keys("c").unwrap();
    term.wait_for_stable(Duration::from_millis(400), STEP).unwrap();
    term.send_keys("M-l").unwrap();
    term.wait_for_stable(Duration::from_millis(400), STEP).unwrap();
    term.wait_for("the Split reason", STEP, |s| {
        s.contains("Split needs a window")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // Preview shows the text as it will look.
    term.send_text("**strong**").unwrap();
    term.send_keys("M-p").unwrap();
    term.wait_for("the preview", STEP, |s| {
        s.contains("strong") && !s.contains("**strong**")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}
