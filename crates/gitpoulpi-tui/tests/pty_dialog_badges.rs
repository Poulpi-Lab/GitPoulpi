//! `Ctrl+N` badges over the repository dialogs (Browse, Clone, Init) and the
//! Activity Log: `Ctrl+N` shows a one-character badge on each field and
//! button, and pressing the character clicks it. Real binary and daemon
//! through a pty (`xtask::pty`); temp config dirs only.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::{Screen, Terminal};

const STARTUP: Duration = Duration::from_secs(60);
const STEP: Duration = Duration::from_secs(30);

/// Which side of its label a control's badge is on: a button's is on its own
/// left edge (the label tail is just right of it); a field's is left of the
/// field, so right of the label.
#[derive(Clone, Copy)]
enum Side {
    Left,
    Right,
}

/// The badge character drawn next to `label` (a few letters the badge does not
/// cover), on the row that holds it.
fn badge_of(screen: &Screen, label: &str, side: Side) -> Option<char> {
    let (col, row) = screen.find(label)?;
    let line: Vec<char> = screen.line(row).chars().collect();
    let col = usize::from(col).min(line.len());
    let is_badge = |i: usize| {
        i + 2 < line.len()
            && line[i] == ' '
            && line[i + 2] == ' '
            && line[i + 1].is_ascii_alphanumeric()
    };
    let at = match side {
        Side::Left => (col.saturating_sub(7)..col).rev().find(|&i| is_badge(i)),
        Side::Right => {
            let end = col + label.chars().count();
            (end..end + 24).find(|&i| is_badge(i))
        }
    };
    at.map(|i| line[i + 1])
}

fn wait(term: &mut Terminal, what: &str, f: impl Fn(&Screen) -> bool) {
    term.wait_for(what, STEP, f)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

/// `Ctrl+N`, then the badge of the control next to `label`.
fn press_badge(term: &mut Terminal, label: &str, side: Side) {
    term.send_keys("C-n").unwrap();
    let l = label.to_string();
    wait(term, "the badge on the control", move |s| {
        badge_of(s, &l, side).is_some()
    });
    let badge = badge_of(&term.screen(), label, side).expect("badge");
    term.send_text(&badge.to_string()).unwrap();
}

struct Setup {
    term: Terminal,
    _repo: FixtureRepo,
    _config: tempfile::TempDir,
}

/// The Home tab, ready for a dialog key.
fn home(session: &str) -> Setup {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "a\n", "badge-start-commit")
        .unwrap()
        .build();
    let config = tempfile::tempdir().unwrap();
    std::fs::write(config.path().join("config.toml"), "").unwrap();
    let mut term = common::launch(repo.root())
        .session(session)
        .config_dir(config.path())
        .size(180, 45)
        .spawn();
    term.wait_for("the first window", STARTUP, |s| {
        s.contains("badge-start-commit")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("C-o").unwrap();
    wait(&mut term, "Home", |s| s.contains("Repository Management"));
    term.wait_for_mouse_capture(STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    Setup {
        term,
        _repo: repo,
        _config: config,
    }
}

#[test]
fn init_badges_focus_a_field_and_press_buttons() {
    let mut s = home("pty-badges-init");
    s.term.send_keys("i").unwrap();
    wait(&mut s.term, "Init", |s| {
        s.contains("Initialize a repository")
    });
    // The Initial branch field: typing after its badge lands in it.
    press_badge(&mut s.term, "Initial branch", Side::Right);
    s.term.send_text("trunk").unwrap();
    wait(&mut s.term, "trunk in the branch field", |s| {
        s.contains("trunk")
    });
    // Browse opens the folder picker over the dialog.
    press_badge(&mut s.term, "owse...", Side::Left);
    wait(&mut s.term, "the folder picker", |s| {
        s.contains("Show hidden folders")
    });
    s.term.send_keys("Escape").unwrap();
    wait(&mut s.term, "the picker closed", |s| {
        !s.contains("Show hidden folders")
    });
    // Cancel closes the dialog.
    press_badge(&mut s.term, "ncel", Side::Left);
    wait(&mut s.term, "the dialog closed", |s| {
        !s.contains("Initialize a repository")
    });
}

#[test]
fn init_badge_mode_is_cancelled_by_escape() {
    let mut s = home("pty-badges-init-esc");
    s.term.send_keys("i").unwrap();
    wait(&mut s.term, "Init", |s| {
        s.contains("Initialize a repository")
    });
    s.term.send_keys("C-n").unwrap();
    wait(&mut s.term, "badges", |s| {
        badge_of(s, "ncel", Side::Left).is_some()
    });
    s.term.send_keys("Escape").unwrap();
    wait(&mut s.term, "no badges, dialog still open", |s| {
        badge_of(s, "ncel", Side::Left).is_none() && s.contains("Initialize a repository")
    });
}

#[test]
fn clone_badges_reach_the_url_field_and_cancel() {
    let mut s = home("pty-badges-clone");
    s.term.send_keys("c").unwrap();
    wait(&mut s.term, "Clone", |s| s.contains("Clone a repository"));
    press_badge(&mut s.term, "ncel", Side::Left);
    wait(&mut s.term, "the dialog closed", |s| {
        !s.contains("Clone a repository")
    });
}

#[test]
fn browse_badges_press_cancel() {
    let mut s = home("pty-badges-browse");
    s.term.send_keys("b").unwrap();
    wait(&mut s.term, "the picker", |s| {
        s.contains("Show hidden folders")
    });
    // The hidden-folders checkbox ticks through its badge.
    press_badge(&mut s.term, "Show hidden", Side::Left);
    press_badge(&mut s.term, "ncel", Side::Left);
    wait(&mut s.term, "the picker closed", |s| {
        !s.contains("Show hidden folders")
    });
}

#[test]
fn activity_log_badges_switch_tab_and_focus_the_filter() {
    let mut s = home("pty-badges-activity");
    s.term.send_keys("C-n").unwrap();
    s.term.send_text("A").unwrap();
    wait(&mut s.term, "the Activity Log", |s| {
        s.contains("Activity Log")
    });
    press_badge(&mut s.term, "lication", Side::Left);
    wait(&mut s.term, "the Application tab", |s| s.contains("Show:"));
    // The filter field: typed text lands in it.
    press_badge(&mut s.term, "Filter:", Side::Right);
    s.term.send_text("zzfilter").unwrap();
    wait(&mut s.term, "the filter text", |s| s.contains("zzfilter"));
}

/// The badge digit of a list: drawn over the first two cells of the list's
/// header, so it is found from the letters just after them (`rest`).
fn list_badge(screen: &Screen, rest: &str, row: Option<u16>) -> Option<char> {
    let (col, r) = screen
        .find_all(rest)
        .into_iter()
        .find(|(_, r)| row.map_or(true, |row| *r == row))?;
    let c = screen
        .cell(col.checked_sub(2)?, r)?
        .display()
        .chars()
        .next()?;
    c.is_ascii_alphanumeric().then_some(c)
}

/// Whether the `..` row of the folders list carries the selection highlight:
/// its cell has a different background than the empty row two lines below
/// (the selection is shown by the highlight alone, no arrow before it).
fn up_row_lit(screen: &Screen) -> bool {
    let Some((col, row)) = screen.find("up one level") else {
        return false;
    };
    let lit = screen.cell(col, row).map(|c| c.bg);
    let plain = screen.cell(col, row + 3).map(|c| c.bg);
    lit.is_some() && lit != plain
}

/// The path after `Selected: `.
fn selected_path(screen: &Screen) -> Option<String> {
    let (col, row) = screen.find("Selected: ")?;
    let line = screen.line(row);
    let from = usize::from(col) + "Selected: ".len();
    Some(
        line.chars()
            .skip(from)
            .collect::<String>()
            .trim()
            .to_string(),
    )
}

/// The Browse picker has one badge for the places list and one for the
/// folders list (not one per row), and pressing one focuses that list
/// without choosing a row: the places badge does not navigate to a place,
/// the folders badge does not select a folder.
#[test]
fn browse_has_one_badge_per_list_and_it_only_focuses_the_list() {
    let mut s = home("pty-badges-browse-lists");
    s.term.send_keys("b").unwrap();
    wait(&mut s.term, "the picker", |s| {
        s.contains("Show hidden folders") && s.contains("QUICK ACCESS")
    });
    let before = selected_path(&s.term.screen()).expect("a Selected line");
    s.term.send_keys("C-n").unwrap();
    wait(&mut s.term, "the list badges", |s| {
        !s.contains("QUICK ACCESS") && s.contains("ICK ACCESS")
    });
    let screen = s.term.screen();
    let (qcol, qrow) = screen.find("ICK ACCESS").unwrap();
    let places = list_badge(&screen, "ICK ACCESS", None).unwrap();
    // The folders list's header ("Name") is on the same row.
    let folders = list_badge(&screen, "me", Some(qrow))
        .unwrap_or_else(|| panic!("no badge on the folders list\n{}", s.term.text()));
    assert_ne!(places, folders);
    // One per list: nothing in the cell column under the first row.
    for dy in 1..4 {
        let c = screen
            .cell(qcol - 2, qrow + dy)
            .unwrap()
            .display()
            .to_string();
        assert_eq!(
            c,
            " ",
            "a badge on row {dy} of the places list\n{}",
            s.term.text()
        );
    }

    // The folders badge: focus without a selected row.
    s.term.send_text(&folders.to_string()).unwrap();
    wait(&mut s.term, "badges gone", |s| s.contains("QUICK ACCESS"));
    assert!(
        !up_row_lit(&s.term.screen()),
        "the folders badge selected a row\n{}",
        s.term.text()
    );
    s.term.send_keys("Down").unwrap();
    wait(&mut s.term, "the keyboard is in the folders list", |s| {
        up_row_lit(s)
    });

    // Up one folder, away from the Home place the list starts on.
    s.term.send_keys("Backspace").unwrap();
    let start = before.clone();
    wait(&mut s.term, "the parent folder", move |s| {
        selected_path(s).is_some_and(|p| p != start)
    });
    let before = selected_path(&s.term.screen()).unwrap();

    // The places badge: focus, no navigation.
    s.term.send_keys("C-n").unwrap();
    wait(&mut s.term, "the badges again", |s| {
        !s.contains("QUICK ACCESS") && s.contains("ICK ACCESS")
    });
    let places = list_badge(&s.term.screen(), "ICK ACCESS", None).unwrap();
    s.term.send_text(&places.to_string()).unwrap();
    wait(&mut s.term, "badges gone", |s| s.contains("QUICK ACCESS"));
    assert_eq!(
        selected_path(&s.term.screen()).as_deref(),
        Some(before.as_str()),
        "the places badge navigated\n{}",
        s.term.text()
    );
}
