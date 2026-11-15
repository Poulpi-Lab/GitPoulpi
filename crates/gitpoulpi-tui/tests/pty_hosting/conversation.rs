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

/// The ISSUES section with `#12` listed.
fn launch(
    session: &str,
    cols: u16,
) -> (
    Terminal,
    tempfile::TempDir,
    Mock,
    gitpoulpi_testkit::FixtureRepo,
) {
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
    assert!(
        screen.contains("Bo Brandt added the label lfs"),
        "{}",
        screen.text()
    );
    assert!(
        !screen.contains("**freezes**"),
        "Markdown is rendered, not shown: {}",
        screen.text()
    );
    assert!(
        mock.saw("GET", "issues/12/discussions"),
        "{:?}",
        mock.requests()
    );
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
    assert!(
        opened_urls(&config).is_empty(),
        "nothing opened before the answer"
    );
    term.send_keys("Escape").unwrap();
    term.wait_for("the question closed", STEP, |s| {
        !s.contains("Open this link?")
    })
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

/// `o` on an issue row opens its page in the browser (the stub logs it) and
/// the app stays on the repository: Enter is the conversation, `o` the host.
#[test]
fn o_on_an_issue_row_opens_the_browser_not_the_conversation() {
    let (mut term, config, _mock, _repo) = launch("pty-conv-o-row", COLS);
    let screen = term.screen();
    let (col, row) = screen
        .find_all("Crash on startup")
        .into_iter()
        .next()
        .expect("the issue row");
    term.click(col + 2, row).expect("select the row");
    term.send_keys("o").expect("o");
    let urls = wait_opened(&config, 1);
    assert!(
        urls.iter().any(|u| u.contains("/issues/12")),
        "the issue page was opened: {urls:?}"
    );
    assert!(
        !term.screen().contains("Posting as"),
        "no conversation opened: {}",
        term.text()
    );
}

/// PTY 6: `Ctrl+N` badges reach Reply, Send (greyed while empty: it says
/// why), a link and Quote.
#[test]
fn ctrl_n_badges_reach_reply_send_a_link_and_quote() {
    let (mut term, config, _mock, _repo) = launch("pty-conv-badges", COLS);
    open_with_enter(&mut term);
    // Select Bo's comment (the description is first, then the comment).
    term.send_keys("Down").unwrap();
    term.wait_for("the Reply button on the selected entry", STEP, |s| {
        s.contains("Reply")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    // Reply: its badge opens the composer for an answer in Bo's discussion.
    // (A badge covers the first letters of its control: found by its tail.)
    press_badge(&mut term, tail("ply", false));
    term.wait_for("the reply box", STEP, |s| s.contains("Reply to Bo Brandt"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Escape").unwrap();
    term.wait_for_stable(Duration::from_millis(400), STEP)
        .unwrap();

    // Send (labelled Reply while an answer is written): greyed while there
    // is nothing to send, and it says why.
    press_badge(&mut term, |s| {
        s.find_all("ply")
            .into_iter()
            .find(|(_, r)| s.line(*r).contains("Ctrl+Enter"))
    });
    term.wait_for("the reason", STEP, |s| s.contains("write something first"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // A link: the badge before the address opens the question.
    press_badge(&mut term, tail("ps://example.org/docs", false));
    term.wait_for("the confirmation", STEP, |s| s.contains("Open this link?"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Escape").unwrap();
    term.wait_for("the question closed", STEP, |s| {
        !s.contains("Open this link?")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    // Quote: its badge puts the entry, quoted, in a new comment.
    press_badge(&mut term, tail("ote", false));
    term.wait_for("the quote in the composer", STEP, |s| {
        s.contains("> **Bo Brandt**") && s.contains("> Seen on Linux too")
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
    term.wait_for_stable(Duration::from_millis(400), STEP)
        .unwrap();
    term.send_keys("M-l").unwrap();
    term.wait_for_stable(Duration::from_millis(400), STEP)
        .unwrap();
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

// -- writing -------------------------------------------------------------------------

/// The JSON body of the first request `method` `needle` the mock saw.
fn body_of(mock: &Mock, method: &str, needle: &str) -> serde_json::Value {
    let request = mock
        .requests()
        .into_iter()
        .find(|r| r.method == method && r.target.contains(needle))
        .unwrap_or_else(|| panic!("no {method} {needle}: {:?}", mock.requests()));
    serde_json::from_str(&request.body).unwrap_or(serde_json::Value::Null)
}

fn lines_with(screen: &Screen, needle: &str) -> usize {
    screen
        .text()
        .lines()
        .filter(|line| line.contains(needle))
        .count()
}

const SENTINEL: &str = "zq9x";

/// PTY 2: `c`, two lines, `Alt+P` previews, `Ctrl+S` sends. The host gets
/// exactly the typed text, the comment appears under the author's name, and
/// what was typed never reaches the status line.
#[test]
fn a_comment_is_written_previewed_and_posted_with_exactly_its_text() {
    let (mut term, _config, mock, _repo) = launch("pty-conv-post", COLS);
    open_with_enter(&mut term);
    term.send_keys("c").unwrap();
    term.wait_for("the box has the keyboard", STEP, |s| {
        s.contains("New comment")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_text("first line").unwrap();
    term.send_keys("Enter").unwrap();
    term.send_text(&format!("second **strong** {SENTINEL}"))
        .unwrap();
    let screen = term
        .wait_for("the typed text", STEP, |s| s.contains(SENTINEL))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(
        !crate::common::status_line_text(&screen).contains(SENTINEL),
        "typed text never goes to the status line:\n{}",
        screen.text()
    );
    term.send_keys("M-p").unwrap();
    term.wait_for("the preview", STEP, |s| {
        s.contains(&format!("second strong {SENTINEL}")) && !s.contains("**strong**")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(!mock.saw("POST", "/notes"), "nothing is sent by previewing");

    term.send_keys("C-s").unwrap();
    wait_until_seen(&mock, "POST", "issues/12/notes");
    assert_eq!(
        body_of(&mock, "POST", "issues/12/notes")["body"],
        format!("first line\nsecond **strong** {SENTINEL}"),
        "the host got exactly what was typed"
    );
    // The comment is in the timeline under its author, the box is empty.
    let screen = term
        .wait_for("the posted comment", STEP, |s| {
            s.contains("Kai") && s.contains("first line") && !s.contains("Sending...")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert_eq!(
        lines_with(&screen, "first line"),
        1,
        "the text left the box:\n{}",
        screen.text()
    );
    assert!(!crate::common::status_line_text(&screen).contains(SENTINEL));
}

fn wait_until_seen(mock: &Mock, method: &str, needle: &str) {
    let deadline = std::time::Instant::now() + STEP;
    while !mock.saw(method, needle) {
        assert!(
            std::time::Instant::now() < deadline,
            "the host never saw {method} {needle}: {:?}",
            mock.requests()
        );
        std::thread::sleep(Duration::from_millis(50));
    }
}

/// PTY 3: a reply to a GitLab discussion goes to `discussions/:id/notes`,
/// not to the item's notes, and the box is titled with who it answers.
#[test]
fn a_reply_to_a_discussion_goes_to_its_notes_endpoint() {
    let (mut term, _config, mock, _repo) = launch("pty-conv-reply", COLS);
    open_with_enter(&mut term);
    term.send_keys("Down").unwrap();
    term.wait_for("Bo's comment selected", STEP, |s| s.contains("Reply"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("r").unwrap();
    term.wait_for("the reply box", STEP, |s| s.contains("Reply to Bo Brandt"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_text("Agreed, thanks").unwrap();
    term.send_keys("C-s").unwrap();
    wait_until_seen(&mock, "POST", "issues/12/discussions/dn1/notes");
    assert!(
        !mock.saw("POST", "issues/12/notes"),
        "a reply is not a new comment: {:?}",
        mock.requests()
    );
    assert_eq!(
        body_of(&mock, "POST", "discussions/dn1/notes")["body"],
        "Agreed, thanks"
    );
    term.wait_for("the answer in the view", STEP, |s| {
        s.contains("Agreed, thanks") && !s.contains("Sending...")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

/// A refused post puts the text back in the box with the host's error under
/// it; nothing the user typed is lost.
#[test]
fn a_refused_comment_keeps_its_text_and_shows_the_error() {
    let mock = start_mock();
    let repo = fixture(mock.port);
    let (mut term, _config) = launch_issues(
        &repo,
        "pty-conv-refused",
        &config_toml(mock.port, ""),
        COLS,
        "Crash on startup",
    );
    open_with_enter(&mut term);
    POST_FAILS.lock().unwrap().push((mock.port, 500));
    term.send_keys("c").unwrap();
    term.send_text("do not lose me").unwrap();
    term.send_keys("C-s").unwrap();
    let screen = term
        .wait_for("the error under the text, the entry gone", STEP, |s| {
            s.contains("not sent:")
                && !s.contains("Sending...")
                && lines_with(s, "do not lose me") == 1
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert_eq!(
        lines_with(&screen, "do not lose me"),
        1,
        "the text is back in the box, once:\n{}",
        screen.text()
    );
}

/// PTY 5: a typed draft asks before the view closes, and comes back when
/// the box is made again (here by following a link and returning).
#[test]
fn a_typed_draft_asks_before_closing_and_is_restored() {
    let (mut term, config, _mock, _repo) = launch("pty-conv-draft", COLS);
    open_with_enter(&mut term);
    term.send_keys("c").unwrap();
    term.send_text("half a thought").unwrap();
    term.wait_for("the text", STEP, |s| s.contains("half a thought"))
        .unwrap_or_else(|e| panic!("{e}"));
    // Esc leaves the box; the second Esc asks instead of closing.
    term.send_keys("Escape").unwrap();
    term.wait_for_stable(Duration::from_millis(400), STEP)
        .unwrap();
    term.send_keys("Escape").unwrap();
    term.wait_for("the question", STEP, |s| s.contains("Discard this text?"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Escape").unwrap();
    term.wait_for("back in the view", STEP, |s| {
        !s.contains("Discard this text?") && s.contains(TITLE_12)
    })
    .unwrap_or_else(|e| panic!("{e}"));
    // The draft is on disk once the typing paused.
    let drafts = config.path().join("remote-conversations.toml");
    let deadline = std::time::Instant::now() + STEP;
    while !std::fs::read_to_string(&drafts)
        .unwrap_or_default()
        .contains("half a thought")
    {
        assert!(std::time::Instant::now() < deadline, "no draft saved");
        std::thread::sleep(Duration::from_millis(100));
    }
    // Follow `#7` and come back: the box has its text again.
    let (col, row) = term
        .screen()
        .find_all("#7")
        .into_iter()
        .find(|(_, r)| *r > 3)
        .expect("the #7 reference in the text");
    term.click(col, row).expect("click #7");
    term.wait_for("issue #7", STEP, |s| s.contains("Locks are not released."))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("M-Left").unwrap();
    term.wait_for("back on #12 with the draft", STEP, |s| {
        s.contains(TITLE_12) && s.contains("half a thought")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

/// PTY 11: a comment on a host is outward. Undo has nothing to take back
/// and says why, in words that point at editing or deleting it.
#[test]
fn undo_says_a_posted_comment_cannot_be_undone_here() {
    let (mut term, _config, mock, _repo) = launch("pty-conv-undo", COLS);
    open_with_enter(&mut term);
    term.send_keys("c").unwrap();
    term.send_text("outward").unwrap();
    term.send_keys("C-s").unwrap();
    wait_until_seen(&mock, "POST", "issues/12/notes");
    term.wait_for("the comment is in", STEP, |s| {
        s.contains("Kai") && !s.contains("Sending...")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // Back to the repository, then Undo.
    term.send_keys("Escape").unwrap();
    term.wait_for("the view closed", STEP, |s| !s.contains("Posting as"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("C-z").unwrap();
    let screen = term
        .wait_for("the Undo answer", STEP, |s| {
            crate::common::status_line_text(s).contains("can't be undone here")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let status = crate::common::status_line_text(&screen);
    assert!(
        status.contains("comment posted to a host") && status.contains("edit or delete it"),
        "{status}"
    );
}

/// A comment of the user's own is edited in place (the host's PUT carries
/// the new text) and deleted after a question (the host's DELETE).
#[test]
fn an_own_comment_is_edited_and_deleted_after_a_question() {
    let (mut term, _config, mock, _repo) = launch("pty-conv-edit", COLS);
    open_with_enter(&mut term);
    term.send_keys("c").unwrap();
    term.send_text("mine").unwrap();
    term.send_keys("C-s").unwrap();
    wait_until_seen(&mock, "POST", "issues/12/notes");
    term.wait_for("the comment, selected, with Edit", STEP, |s| {
        s.contains("Edit") && s.contains("Delete") && !s.contains("Sending...")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // Bo's comment, written by someone else, has neither (the label event
    // between them is skipped).
    term.send_keys("Up").unwrap();
    term.send_keys("Up").unwrap();
    term.wait_for("Bo's comment selected", STEP, |s| {
        s.contains("Reply") && !s.contains("Edit")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Down").unwrap();
    term.send_keys("Down").unwrap();
    term.wait_for("mine selected", STEP, |s| s.contains("Edit"))
        .unwrap_or_else(|e| panic!("{e}"));
    // Edit: the text is in the box; an unchanged Save is refused.
    term.send_keys("e").unwrap();
    term.wait_for("the edit box", STEP, |s| s.contains("Edit comment"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_text(" and edited").unwrap();
    term.send_keys("C-s").unwrap();
    wait_until_seen(&mock, "PUT", "issues/12/notes/301");
    assert_eq!(
        body_of(&mock, "PUT", "notes/301")["body"],
        "mine and edited"
    );
    term.wait_for("the edited comment", STEP, |s| {
        s.contains("mine and edited") && !s.contains("Sending...")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // Delete asks first; Keep does nothing, Delete sends the DELETE.
    term.send_keys("d").unwrap();
    term.wait_for("the question", STEP, |s| s.contains("Delete this comment?"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Escape").unwrap();
    term.wait_for("the question closed", STEP, |s| {
        !s.contains("Delete this comment?")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert!(!mock.saw("DELETE", "notes/301"), "Keep deletes nothing");
    term.send_keys("d").unwrap();
    term.wait_for("the question again", STEP, |s| {
        s.contains("Delete this comment?")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Tab").unwrap();
    term.send_keys("Enter").unwrap();
    wait_until_seen(&mock, "DELETE", "issues/12/notes/301");
    term.wait_for("the comment is gone", STEP, |s| {
        !s.contains("mine and edited")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

// -- the GitHub App bot ------------------------------------------------------------

/// A repository whose ISSUES list is the GitHub mock's `o/r`, with an App
/// installation token (`ghs_...`): `GET /user` answers 403 and a post comes
/// back from `poulpi-lab[bot]`.
fn launch_bot(
    session: &str,
) -> (
    Terminal,
    tempfile::TempDir,
    xtask::mock_host::MockHost,
    gitpoulpi_testkit::FixtureRepo,
) {
    let mock = xtask::mock_host::start(xtask::mock_host::Options::default()).expect("mock host");
    let repo = fixture(mock.port);
    let dir = repo.state_dir();
    std::fs::create_dir_all(&dir).expect("state dir");
    std::fs::write(
        dir.join("repo-state.toml"),
        "[issues_source]\nintegration = \"hub\"\nproject = \"o/r\"\n",
    )
    .expect("seed repo-state.toml");
    let config = tempfile::tempdir().expect("config dir");
    std::fs::write(
        config.path().join("config.toml"),
        format!(
            "[general]\nhosting_refresh_minutes = 0\n\n[[integrations]]\nid = \"hub\"\nkind = \"github_enterprise\"\nurl = \"http://127.0.0.1:{}\"\ntoken_ref = \"integration:hub\"\nname = \"hub\"\n",
            mock.port
        ),
    )
    .expect("seed config.toml");
    let term = crate::common::launch(repo.root())
        .session(session)
        .config_dir(config.path())
        .size(COLS, ROWS)
        .env("GITPOULPI_NO_BROWSER", "1")
        .env("GITPOULPI_OPEN_LOG", config.path().join("opened.log"))
        .env("GITPOULPI_TOKEN_HUB", "ghs_installation_token")
        .spawn();
    term.wait_for("the issue listed", STARTUP, |s| {
        s.contains("Crash when the plane")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    (term, config, mock, repo)
}

/// PTY 4: with an App installation token the header names the App bot, no
/// comment can be edited by guesswork, and after the first post the new
/// comment (and only it) offers Edit; the bot's login is learned and kept.
#[test]
fn a_github_app_bot_is_named_and_edits_only_what_this_install_posted() {
    let (mut term, _config, mock, repo) = launch_bot("pty-conv-bot");
    let screen = term.screen();
    let (col, row) = screen
        .find_all("Crash when the plane")
        .into_iter()
        .next()
        .expect("the issue row");
    term.click(col + 2, row).expect("select the row");
    term.send_keys("Enter").expect("Enter");
    term.wait_for("the conversation", STEP, |s| {
        s.contains("#7") && s.contains("I can reproduce this") && s.contains("Posting as")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let screen = term.screen();
    assert!(
        screen.contains("Posting as the GitHub App bot (its name is not known yet"),
        "the App bot is named, never a guess:\n{}",
        screen.text()
    );
    // Nobody's comment offers Edit: the bot's name is not known.
    term.send_keys("Down").unwrap();
    // tomas's comment is selected once its own row carries the buttons.
    term.wait_for("tomas's comment selected", STEP, |s| {
        s.text()
            .lines()
            .any(|l| l.contains("tomas") && l.contains("Reply"))
    })
    .unwrap_or_else(|e| {
        panic!(
            "{e}
{}",
            term.text()
        )
    });
    assert!(
        !term.screen().contains("Edit"),
        "no Edit while authorship cannot be known:\n{}",
        term.text()
    );
    // Post.
    term.send_keys("c").unwrap();
    term.send_text("from the app").unwrap();
    term.send_keys("C-s").unwrap();
    wait_until_seen_host(&mock, "POST", "issues/7/comments");
    let screen = term
        .wait_for("the comment and the learned name", STEP, |s| {
            s.contains("from the app")
                && s.contains("Posting as poulpi-lab[bot] (GitHub App bot)")
                && s.contains("Edit")
                && s.contains("Delete")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let _ = screen;
    // Only the new comment: the others (tomas, ines) have none.
    term.send_keys("Up").unwrap();
    term.wait_for("an older comment selected", STEP, |s| {
        s.contains("Reply") && !s.contains("Edit")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // The learned login and the posted id are kept in the repository's state.
    let state =
        std::fs::read_to_string(repo.state_dir().join("repo-state.toml")).unwrap_or_default();
    assert!(
        state.contains("hosting_identity = \"poulpi-lab[bot]\""),
        "{state}"
    );
    assert!(state.contains("|2100"), "{state}");
}

fn wait_until_seen_host(mock: &xtask::mock_host::MockHost, method: &str, needle: &str) {
    let deadline = std::time::Instant::now() + STEP;
    while !mock.saw(method, needle) {
        assert!(
            std::time::Instant::now() < deadline,
            "the host never saw {method} {needle}: {:?}",
            mock.requests()
        );
        std::thread::sleep(Duration::from_millis(50));
    }
}
