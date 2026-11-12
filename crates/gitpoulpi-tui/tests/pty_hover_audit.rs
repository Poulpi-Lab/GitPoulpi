//! Buttons highlight under the pointer, end to end through a pty: each
//! changes its cell style on hover, and the toolbar buttons show a tooltip
//! after the dwell. Each test compares the cell the pointer sits on before
//! and after, never a hard-coded color.

mod common;

use std::path::Path;
use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::{Screen, Terminal};

const STEP: Duration = Duration::from_secs(30);

fn bg(screen: &Screen, col: u16, row: u16) -> Option<xtask::pty::Color> {
    screen.cell(col, row).map(|c| c.bg)
}

fn fixture() -> gitpoulpi_testkit::FixtureRepo {
    let original: Vec<String> = (1..=30).map(|n| format!("l{n}")).collect();
    let mut edited = original.clone();
    edited[1] = "CHANGED-TOP".to_string();
    edited[14] = "CHANGED-MIDDLE".to_string();
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("many.txt", &(original.join("\n") + "\n"), "add many")
        .unwrap()
        .write_worktree("many.txt", (edited.join("\n") + "\n").as_bytes())
        .unwrap()
        .build();
    repo
}

fn launch(repo: &Path, session: &str) -> (Terminal, tempfile::TempDir) {
    common::launch(repo)
        .session(session)
        .size(200, 50)
        .no_exit_pause()
        .ready_when("the fixture's log", |s| s.contains("add many"))
        .start()
}

/// The hunk header's `Stage hunk` / `Discard hunk` fill under the pointer.
#[test]
fn the_diff_hunk_buttons_light_up_under_the_pointer() {
    let repo = fixture();
    let (mut term, _config) = launch(repo.root(), "pty-hover-hunk");
    let (col, row) = term.wait_for_text("\u{25CC}", STEP).unwrap();
    term.click(col, row).unwrap();
    let (col, row) = term.wait_for_text("\u{270E} many.txt", STEP).unwrap();
    term.click(col + 1, row).unwrap();
    term.wait_for("the unstaged diff", STEP, |s| {
        s.contains("many.txt - unstaged") && s.contains("Stage hunk") && s.contains("Discard hunk")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.wait_for_mouse_capture(STEP).unwrap();
    for label in ["Stage hunk", "Discard hunk"] {
        // Park the pointer off the buttons first so "before" is the rest look.
        term.move_to(0, 0).unwrap();
        let rest = term
            .wait_for("the button at rest", STEP, |s| s.contains(label))
            .unwrap();
        let (bcol, brow) = rest.find(label).unwrap();
        let before = bg(&rest, bcol, brow);
        term.move_to(bcol + 1, brow).unwrap();
        term.wait_for(&format!("{label} filled under the pointer"), STEP, |s| {
            s.find(label).is_some_and(|(c, r)| bg(s, c, r) != before)
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    }
    term.stop().unwrap();
}

/// The Log scrollbar's end arrow lights while the pointer is on it.
#[test]
fn a_scrollbar_end_arrow_lights_up_under_the_pointer() {
    let mut builder = FixtureRepoBuilder::new().unwrap();
    for n in 10..=99 {
        builder = builder
            .commit_file("f.txt", &format!("{n}\n"), &format!("commit-{n}"))
            .unwrap();
    }
    let repo = builder.build();
    let config = tempfile::tempdir().unwrap();
    let mut term = common::launch(repo.root())
        .session("pty-hover-scrollbar")
        .config_dir(config.path())
        .size(160, 30)
        .spawn();
    // The Log's down arrow is part of the wait: the scrollbar is drawn once
    // the pane has its final size, which can be a frame after the rows.
    let down_arrow = |s: &Screen| {
        let (_, log_top) = s.find("\u{256e}\u{256d}")?;
        s.find_all("\u{2193}")
            .into_iter()
            .filter(|(_, r)| *r > log_top)
            .max_by_key(|(_, r)| *r)
    };
    // The selected commit's details pane fills in after the first frame and
    // the layout settles with it, so wait for it and for the arrow to sit in
    // the same cell on two reads (the layout was not final on the first
    // frame under load).
    term.wait_for("the Log and the commit details", STEP, |s| {
        s.contains("commit-99") && s.contains("Changed files (1)") && down_arrow(s).is_some()
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.wait_for_mouse_capture(STEP).unwrap();
    term.move_to(0, 0).unwrap();
    let mut seen = None;
    let rest = term
        .wait_for("the arrow at rest and still", STEP, |s| {
            let now = down_arrow(s);
            let still = now.is_some() && now == seen;
            seen = now;
            still
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let (col, down_row) = down_arrow(&rest).expect("the Log's down arrow");
    let before = bg(&rest, col, down_row);
    term.move_to(col, down_row).unwrap();
    term.wait_for("the arrow lit", STEP, |s| bg(s, col, down_row) != before)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.move_to(0, 0).unwrap();
    term.wait_for("the arrow back at rest", STEP, |s| {
        bg(s, col, down_row) == before
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.stop().unwrap();
}

/// The Discard All button is a heavy cross button (the wastebasket
/// did not render in every font), sits two empty cells clear of the pane's
/// collapse chevron, and lights up under the pointer.
#[test]
fn discard_all_is_a_cross_clear_of_the_chevron_and_lights_up() {
    let repo = fixture();
    let (mut term, _config) = launch(repo.root(), "pty-hover-trash");
    let screen = term
        .wait_for("the header trash", STEP, |s| s.contains(" \u{2716}"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.wait_for_mouse_capture(STEP).unwrap();
    let (col, row) = screen.find(" \u{2716}").unwrap();
    let (chev, chev_row) = screen.find("\u{203A}").expect("the Status pane's chevron");
    assert_eq!(chev_row, row, "chevron and button share the header row");
    assert!(
        col >= chev + 3,
        "two empty cells between the chevron ({chev}) and the button ({col})"
    );
    let before = bg(&screen, col + 1, row);
    term.move_to(col + 1, row).unwrap();
    term.wait_for("the trash filled", STEP, |s| bg(s, col + 1, row) != before)
        .unwrap_or_else(|e| panic!("{e}"));
    term.stop().unwrap();
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

/// The clickable-control audit: on Settings > Integrations (the Radar block of
/// a connected profile) every control answers with the pointer of its kind,
/// the checkboxes (including Regular expressions) and the Ask list the hand,
/// the Namespaces area the text cursor, and no button is bracketed text.
#[test]
fn the_integrations_controls_show_their_pointer_and_no_button_is_bracketed() {
    const PROFILE: &str = "[[integrations]]\nid = \"work\"\nkind = \"github\"\n\
        url = \"https://github.com\"\ntoken_ref = \"integration:work\"\nname = \"WorkAccount\"\n\n\
        [integrations.user]\nlogin = \"kai\"\nname = \"Kai\"\nid = \"7\"\n";
    let repo = fixture();
    let (mut term, _config) = common::launch(repo.root())
        .session("pty-hover-audit-integrations")
        .config(PROFILE)
        .size(180, 60)
        .no_exit_pause()
        .ready_when("the fixture's log", |s| s.contains("add many"))
        .start();
    term.send_keys("C-p").unwrap();
    term.send_text("Radar Options of a Profile").unwrap();
    term.wait_for("the palette narrowed", STEP, |s| {
        s.contains("Command Palette (1/")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    let screen = term
        .wait_for("the Radar block of the profile", STEP, |s| {
            s.contains("Requests waiting for my review")
                && s.contains("Regular expressions")
                && s.contains("e.g. acme, platform")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // The marks of checkboxes aside, nothing is bracketed.
    let bare = screen.text().replace("[x]", "   ").replace("[ ]", "   ");
    assert!(
        !bare.contains("[ ") && !bare.contains(" ]") && !bare.contains("[+"),
        "a bracketed button:\n{}",
        screen.text()
    );
    term.wait_for_mouse_capture(STEP).unwrap();
    let hover = |term: &mut Terminal, what: &str, (col, row): (u16, u16), want: &str| {
        term.move_to(col, row).unwrap();
        let deadline = std::time::Instant::now() + STEP;
        while pointer_shapes(term).last().map(String::as_str) != Some(want)
            && std::time::Instant::now() < deadline
        {
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        assert_eq!(
            pointer_shapes(term).last().map(String::as_str),
            Some(want),
            "over {what}: {:?}\n{}",
            pointer_shapes(term),
            term.text()
        );
    };
    for label in [
        "Requests waiting for my review",
        "Include drafts",
        "Regular expressions",
    ] {
        let (c, r) = screen.find(label).unwrap_or_else(|| panic!("no {label}"));
        // On the label, not only on the mark.
        hover(&mut term, label, (c + 1, r), "pointer");
    }
    let (c, r) = screen.find("e.g. acme, platform").unwrap();
    hover(&mut term, "the Namespaces area", (c + 1, r), "text");
    term.stop().unwrap();
}

/// A toolbar button shows its tooltip once the pointer rests on it.
#[test]
fn a_toolbar_button_shows_a_tooltip_after_the_dwell() {
    let repo = fixture();
    let (mut term, _config) = launch(repo.root(), "pty-hover-tooltip");
    let (col, row) = term.wait_for_text("\u{2387} Branch", STEP).unwrap();
    term.wait_for_mouse_capture(STEP).unwrap();
    term.move_to(col + 2, row).unwrap();
    term.wait_for("the Branch tooltip", STEP, |s| {
        s.contains("Create a branch at")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}
