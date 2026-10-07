//! Slice L2 end to end: the Kanban board of the ISSUES source. A folder of
//! Markdown notes (no token, no network) and the shared `xtask mock-host`
//! (GitLab, labels as columns). Cards move with `Shift+Right`, with the
//! mouse (a real press / motion / release drag) and from the palette, and
//! what the board shows is what is on disk / what the host was told.

use std::path::Path;
use std::time::Duration;

use crate::support::*;
use xtask::mock_host::{self, Options};
use xtask::pty::Terminal;

fn wait_until(what: &str, mut ok: impl FnMut() -> bool) {
    let deadline = std::time::Instant::now() + STEP;
    while !ok() {
        assert!(std::time::Instant::now() < deadline, "timed out: {what}");
        std::thread::sleep(Duration::from_millis(50));
    }
}

fn palette(term: &mut Terminal, query: &str, entry: &str) {
    term.send_keys("C-p").expect("the palette");
    term.wait_for_text("Command Palette", STEP).unwrap();
    term.send_text(query).unwrap();
    term.wait_for("the entry", STEP, |s| s.contains(entry))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
}

fn read(notes: &Path, name: &str) -> String {
    std::fs::read_to_string(notes.join(name)).unwrap_or_default()
}

const LOCAL: &str = "[general]\nhosting_refresh_minutes = 0\n\n[[integrations]]\nid = \"notes\"\nkind = \"local_markdown\"\nurl = \"docs/issues\"\ntoken_ref = \"integration:notes\"\nname = \"notes\"\nuser = { login = \"kai\" }\n";

fn note(id: u64, slug: &str, status: &str) -> String {
    format!("---\nid: {id}\ntitle: Card {slug}\nstatus: {status}\nlabels: [bug]\ncssclasses: wide\n---\nKeep this body {id}.\n")
}

/// The row and the first column of `text` that is on the board's header row
/// (the first match below the panel's top border).
fn header_cell(term: &Terminal, text: &str) -> (u16, u16) {
    term.screen()
        .find_all(text)
        .into_iter()
        .min_by_key(|&(_, r)| r)
        .unwrap_or_else(|| panic!("no {text:?} on\n{}", term.text()))
}

#[test]
fn a_notes_folder_board_moves_cards_with_the_keyboard_the_mouse_and_the_palette() {
    let repo = fixture(1);
    let notes = repo.root().join("docs").join("issues");
    std::fs::create_dir_all(&notes).unwrap();
    std::fs::write(notes.join("1-alpha.md"), note(1, "alpha", "todo")).unwrap();
    std::fs::write(notes.join("2-beta.md"), note(2, "beta", "todo")).unwrap();
    std::fs::write(notes.join("3-gamma.md"), note(3, "gamma", "doing")).unwrap();
    let state = repo.state_dir();
    std::fs::create_dir_all(&state).unwrap();
    std::fs::write(
        state.join("repo-state.toml"),
        "[issues_source]\nintegration = \"notes\"\nproject = \"docs/issues\"\n",
    )
    .unwrap();
    let (mut term, _config) = launch_unlinked(
        &repo,
        "pty-hosting-board-local",
        LOCAL,
        COLS,
        "Add the feature",
    );
    term.wait_for("the notes listed", STARTUP, |s| {
        s.contains("#1  Card alpha") && s.contains("#3  Card gamma")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // -- open: the palette, then the columns with their cards ------------------
    palette(&mut term, "Open Board", "Issues: Open Board");
    term.wait_for("the board", STEP, |s| {
        s.contains("Issue board") && s.contains("todo") && s.contains("doing") && s.contains("done")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.wait_for("the cards", STEP, |s| {
        s.contains("Card alpha") && s.contains("Card beta") && s.contains("Card gamma")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // -- Shift+Right: the selected card (#2, the first of todo) moves ----------
    term.send_keys("S-Right").unwrap();
    wait_until("the note says doing", || {
        read(&notes, "2-beta.md").contains("status: doing\n")
    });
    // Only the frontmatter it owns changed: the unknown property and the body stay.
    let moved = read(&notes, "2-beta.md");
    for kept in [
        "title: Card beta\n",
        "labels: [bug]\ncssclasses: wide\n",
        "---\nKeep this body 2.\n",
    ] {
        assert!(moved.contains(kept), "{kept:?} is gone from\n{moved}");
    }
    assert_eq!(read(&notes, "1-alpha.md"), note(1, "alpha", "todo"));

    // -- the mouse: drag #1 onto the done column --------------------------------
    // The note on disk is not the board: the board re-reads after the write,
    // and until it has, alpha is still on the second row of todo (beta
    // above it), where a press finds no card. Wait for beta to have left the
    // todo column, which also lifts alpha to the first row.
    term.wait_for("beta in the doing column, right of alpha", STEP, |s| {
        matches!((s.find("Card beta"), s.find("Card alpha")),
            (Some((beta, _)), Some((alpha, _))) if beta > alpha + 10)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (cx, cy) = term
        .wait_for_text("Card alpha", STEP)
        .expect("the card to drag");
    let (hx, hy) = header_cell(&term, "done");
    // A press, two motions, a release: each a short write (a 90-report
    // burst can leave the tail of the last escape sequence waiting in
    // crossterm's reader).
    let resting = term.screen().cell(hx + 1, hy).map(|c| c.bg).unwrap();
    term.press(cx + 1, cy).unwrap();
    term.drag_to(cx + 12, cy + 1).unwrap();
    term.drag_to(hx + 2, hy + 6).unwrap();
    term.wait_for("the drop target is lit", STEP, |s| {
        s.cell(hx + 1, hy).map(|c| c.bg) != Some(resting)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.release(hx + 2, hy + 6).unwrap();
    wait_until("the dragged note says done", || {
        read(&notes, "1-alpha.md").contains("status: done\n")
    });
    assert!(read(&notes, "3-gamma.md").contains("status: doing\n"));

    // -- the palette twin: "Move Card Left" on the selected card (#1, now in done)
    palette(&mut term, "Board: Move Card Left", "Board: Move Card Left");
    wait_until("the palette move reached the disk", || {
        read(&notes, "1-alpha.md").contains("status: doing\n")
    });

    // -- live: a note written from outside shows on the board -------------------
    std::fs::write(
        notes.join("4-from-obsidian.md"),
        "---\nid: 4\ntitle: Written outside\nstatus: todo\n---\n",
    )
    .unwrap();
    term.wait_for("the outside note appears on the board", STEP, |s| {
        s.contains("Written outside")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // -- Enter opens the note (the system opener, here a stub) ------------------
    term.send_keys("Escape").unwrap();
    term.wait_for("the board closed", STEP, |s| !s.contains("Issue board"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

#[test]
fn a_host_board_swaps_status_labels_and_closes_through_the_host() {
    let mock = mock_host::start(Options::default()).expect("the mock host");
    let repo = fixture(mock.port);
    // The columns of this repository's board: two labels the mock knows.
    let dir = repo.state_dir();
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("repo-state.toml"),
        "board_labels = [\"data\", \"bug\"]\n\n[issues_source]\nintegration = \"corp\"\nproject = \"g/p\"\n",
    )
    .unwrap();
    let (mut term, _config) = launch_unlinked(
        &repo,
        "pty-hosting-board-gitlab",
        &config_toml(mock.port, ""),
        COLS,
        "Add the feature",
    );
    term.wait_for("the issues listed", STARTUP, |s| {
        s.contains("Moons of Jupiter") && s.contains("Orbit chart flicke")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    palette(&mut term, "Open Board", "Issues: Open Board");
    term.wait_for("Open, the two labels, Closed", STEP, |s| {
        s.contains("Issue board")
            && s.contains("Open")
            && s.contains("data")
            && s.contains("bug")
            && s.contains("Closed")
            && s.contains("Clarify the moon")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // #190 (docs) is the only card of Open and is selected: into `data`.
    term.send_keys("S-Right").unwrap();
    wait_until("the host got the label swap", || {
        mock.requests().iter().any(|r| {
            r.method == "PUT"
                && r.target.contains("/issues/190")
                && serde_json::from_str::<serde_json::Value>(&r.body)
                    .map(|b| b["labels"] == "docs,data")
                    .unwrap_or(false)
        })
    });
    // Then `bug`: the previous status label goes, the other labels stay.
    term.send_keys("S-Right").unwrap();
    wait_until("the host got the second swap", || {
        mock.requests().iter().any(|r| {
            r.method == "PUT"
                && r.target.contains("/issues/190")
                && serde_json::from_str::<serde_json::Value>(&r.body)
                    .map(|b| b["labels"] == "docs,bug")
                    .unwrap_or(false)
        })
    });
    // Then Closed: the state call, labels untouched.
    term.send_keys("S-Right").unwrap();
    wait_until("the host closed it", || {
        mock.requests().iter().any(|r| {
            r.method == "PUT"
                && r.target.contains("/issues/190")
                && serde_json::from_str::<serde_json::Value>(&r.body)
                    .map(|b| b["state_event"] == "close")
                    .unwrap_or(false)
        })
    });
    // After the board read the host again, the card is still in Closed.
    term.send_keys("r").unwrap();
    term.wait_for("the card sits in Closed", STEP, |s| {
        let (_, closed_row) = match s.find("Closed") {
            Some(p) => p,
            None => return false,
        };
        s.find("Clarify the moon")
            .is_some_and(|(c, r)| r > closed_row && c > 80)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}
