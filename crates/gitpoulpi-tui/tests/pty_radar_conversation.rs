#![cfg(all(feature = "radar", feature = "conversation"))]
//! The conversation view opened from the Radar for an issue whose repository
//! is not cloned -- the real binary and daemon, driven through a pty by
//! `xtask::pty`, against `xtask mock-host` (a `TcpListener` on 127.0.0.1; no
//! other network).
//!
//! One GitLab profile (`lab`) with its assigned issues on: the mock lists
//! issue `#31` of `northwind/ledger-ui` ("Ledger export drops the last row")
//! in a repository that exists nowhere on this machine. Its description holds
//! a ```mermaid block.
//!
//! * `i` (and the row menu, and the palette): the view opens over the Radar
//!   with the host's timeline, no checkout; Esc and the close cross give the
//!   Radar back.
//! * The mermaid block shows an "Open diagram" button under it (only in a
//!   build with the `mermaid` feature): a click, the links menu and the
//!   palette all open the diagram viewer, which has its own Close button.

#![allow(dead_code)]

#[path = "common/mod.rs"]
mod common;

use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::mock_host::{self, MockHost, Options};
use xtask::pty::{Screen, Terminal};

const STARTUP: Duration = Duration::from_secs(60);
const STEP: Duration = Duration::from_secs(30);
const COLS: u16 = 160;
const ROWS: u16 = 50;
const ROW: &str = "Ledger export drops the last row";
/// What the mock's discussion of the issue says.
const COMMENT: &str = "I can reproduce this.";

struct Setup {
    term: Terminal,
    mock: MockHost,
    config: tempfile::TempDir,
    _repo: FixtureRepo,
}

fn setup(session: &str) -> Setup {
    let mock = mock_host::start(Options {
        port: 0,
        bare_root: None,
    })
    .expect("the mock host");
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "a\n", "radar-conversation-start-commit")
        .unwrap()
        .build();
    let config = tempfile::tempdir().expect("temp config dir");
    std::fs::write(
        config.path().join("config.toml"),
        format!(
            "[[integrations]]\nid = \"lab\"\nkind = \"gitlab_self_managed\"\nurl = \"http://127.0.0.1:{port}\"\ntoken_ref = \"integration:lab\"\nname = \"lab\"\nuser = {{ login = \"{login}\" }}\n\n[integrations.radar]\nassigned_issues = true\n",
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
        .size(COLS, ROWS)
        .env("GITPOULPI_NO_BROWSER", "1")
        .env("GITPOULPI_OPEN_LOG", config.path().join("opened.log"))
        .env("GITPOULPI_TOKEN_LAB", "test-token")
        .env("GITPOULPI_DESKTOP_NO_HOOKS", "1")
        .spawn();
    term.wait_for("the repository loaded", STARTUP, |s| {
        s.contains("radar-conversation-start-commit")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    Setup {
        term,
        mock,
        config,
        _repo: repo,
    }
}

/// The Radar, with the far issue listed and selected.
fn select_issue(term: &mut Terminal) -> Screen {
    term.send_keys("M-r").expect("Alt+R");
    let screen = term
        .wait_for("the far issue", STARTUP, |s| {
            s.contains("Radar") && s.contains(ROW) && s.contains("Not cloned")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let (col, row) = screen.find(ROW).unwrap();
    term.click(col + 2, row).expect("select the row");
    screen
}

fn wait_open(term: &mut Terminal) -> Screen {
    term.wait_for("the conversation over the Radar", STEP, |s| {
        s.contains("Crash when the planet list is empty") && s.contains(COMMENT)
    })
    .unwrap_or_else(|e| panic!("{e}"))
}

fn wait_radar(term: &mut Terminal) {
    term.wait_for("the Radar again", STEP, |s| {
        !s.contains(COMMENT) && s.contains("Scope") && s.contains(ROW)
    })
    .unwrap_or_else(|e| panic!("{e}"));
}

fn palette(term: &mut Terminal, command: &str) {
    term.send_keys("C-p").expect("the palette");
    term.wait_for("the palette", STEP, |s| s.contains("Command Palette"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_text(command).unwrap();
    term.wait_for("the command", STEP, |s| s.contains(command))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").expect("run it");
}

/// `i` on a not-cloned issue: the view opens in the Home window over the
/// Radar, the timeline comes from the host (nothing was cloned), Esc gives
/// the Radar back.
#[test]
fn i_opens_the_conversation_of_a_not_cloned_issue_over_the_radar() {
    let mut s = setup("pty-radar-conv-key");
    select_issue(&mut s.term);
    s.term.send_keys("i").expect("i");
    let screen = wait_open(&mut s.term);
    assert!(
        !screen.contains("clone it first"),
        "no clone asked: {}",
        screen.text()
    );
    assert!(
        s.mock
            .saw("GET", "northwind%2Fledger-ui/issues/31/discussions"),
        "read from the host: {:?}",
        s.mock.requests()
    );
    s.term.send_keys("Escape").expect("close");
    wait_radar(&mut s.term);
    // The Radar answers again.
    assert!(s.term.screen().contains("Not cloned"));
}

/// The same from the row's menu (right-click), and the view's close cross
/// gives the Radar back.
#[test]
fn the_row_menu_and_the_close_cross_do_the_same() {
    let mut s = setup("pty-radar-conv-menu");
    let screen = select_issue(&mut s.term);
    let (col, row) = screen.find(ROW).unwrap();
    s.term.right_click(col + 2, row).expect("the row menu");
    let screen = s
        .term
        .wait_for("the row menu", STEP, |s| s.contains("Open conversation"))
        .unwrap_or_else(|e| panic!("{e}"));
    let (c, r) = screen.find("Open conversation").unwrap();
    s.term.click(c + 1, r).expect("pick it");
    let screen = wait_open(&mut s.term);
    // The close cross of the frame: the top-right cell of the surface.
    let cross = screen
        .find_all("\u{2715}")
        .into_iter()
        .find(|&(_, r)| r <= 4)
        .unwrap_or_else(|| panic!("a close cross:\n{}", screen.text()));
    s.term.click(cross.0, cross.1).expect("the cross");
    wait_radar(&mut s.term);
}

/// The palette reaches the command from the Radar and the view's own commands
/// once it is open.
#[test]
fn the_palette_opens_it_and_reaches_the_view() {
    let mut s = setup("pty-radar-conv-palette");
    select_issue(&mut s.term);
    palette(&mut s.term, "Open conversation");
    wait_open(&mut s.term);
    // A view command, answered by the hosted view.
    palette(&mut s.term, "Conversation: Refresh");
    s.term
        .wait_for("a second read", STEP, |_| {
            s.mock
                .requests()
                .iter()
                .filter(|r| r.target.contains("issues/31/discussions"))
                .count()
                >= 2
        })
        .unwrap_or_else(|e| panic!("{e}"));
    s.term.send_keys("Escape").expect("close");
    wait_radar(&mut s.term);
}

/// The first entry of the view is the issue's description, with the diagram
/// button under its block (only with the `mermaid` feature).
#[cfg(feature = "mermaid")]
mod mermaid {
    use super::*;

    const BUTTON: &str = "Open diagram";
    const SCRIPT: &str = "Export[Export job] --> Ledger[Ledger table]";

    fn open_with_button(s: &mut Setup) -> Screen {
        select_issue(&mut s.term);
        s.term.send_keys("i").expect("i");
        s.term
            .wait_for("the description with its button", STEP, |s| {
                s.contains("Crash when the planet list is empty") && s.contains(BUTTON)
            })
            .unwrap_or_else(|e| panic!("{e}"))
    }

    fn wait_viewer(term: &mut Terminal) -> Screen {
        term.wait_for("the diagram viewer", STEP, |s| {
            s.contains("Mermaid diagram") && s.contains("Copy")
        })
        .unwrap_or_else(|e| panic!("{e}"))
    }

    fn wait_closed(term: &mut Terminal) {
        term.wait_for("the viewer closed", STEP, |s| {
            !s.contains("Mermaid diagram") && s.contains("Crash when the planet list is empty")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    }

    /// A click on the button opens the viewer; its Close button closes it
    /// and the conversation is still there.
    #[test]
    fn the_button_opens_the_viewer_and_close_leaves_the_conversation_up() {
        let mut s = setup("pty-radar-conv-diagram-click");
        let screen = open_with_button(&mut s);
        let (col, row) = screen.find(BUTTON).unwrap();
        s.term.click(col + 1, row).expect("the button");
        let screen = wait_viewer(&mut s.term);
        let (c, r) = screen
            .find_all("Close")
            .into_iter()
            .max_by_key(|&(_, r)| r)
            .expect("the Close button");
        s.term.click(c + 1, r).expect("Close");
        wait_closed(&mut s.term);
    }

    /// The keyboard twin: the links menu (`l`) lists the diagram; Enter opens
    /// the viewer; Esc closes it.
    #[test]
    fn the_links_menu_opens_the_viewer_from_the_keyboard() {
        let mut s = setup("pty-radar-conv-diagram-keys");
        open_with_button(&mut s);
        // The description is the first entry; select it, then list its links.
        s.term.send_keys("Home").expect("first entry");
        s.term.send_keys("l").expect("links");
        s.term
            .wait_for("the diagram in the links menu", STEP, |s| {
                s.contains("Diagram: graph TD")
            })
            .unwrap_or_else(|e| panic!("{e}"));
        s.term.send_keys("Enter").expect("pick it");
        wait_viewer(&mut s.term);
        s.term.send_keys("Escape").expect("close the viewer");
        wait_closed(&mut s.term);
    }

    /// The palette reaches it too; Enter in the viewer copies the script and
    /// closes it.
    #[test]
    fn the_palette_opens_the_viewer_and_enter_copies_the_script() {
        let mut s = setup("pty-radar-conv-diagram-palette");
        open_with_button(&mut s);
        s.term.send_keys("Home").expect("first entry");
        palette(&mut s.term, "Open Diagram of Entry");
        wait_viewer(&mut s.term);
        s.term.send_keys("Enter").expect("copy");
        s.term
            .wait_for("the viewer closed by the copy", STEP, |s| {
                !s.contains("Mermaid diagram") && s.contains("Open diagram")
            })
            .unwrap_or_else(|e| panic!("{e}"));
    }
}
