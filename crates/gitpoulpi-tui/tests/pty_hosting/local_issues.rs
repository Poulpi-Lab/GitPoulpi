//! Slice L1 end to end: a folder of Markdown notes is the ISSUES source (no
//! token, no network). Create an issue from the form, edit its labels in the
//! picker (a value nobody used yet is typed), close it from the row menu, and
//! see a note written from outside appear on its own.

use std::path::Path;
use std::time::Duration;

use crate::support::*;

const CONFIG: &str = "[general]\nhosting_refresh_minutes = 0\n\n[[integrations]]\nid = \"notes\"\nkind = \"local_markdown\"\nurl = \"docs/issues\"\ntoken_ref = \"integration:notes\"\nname = \"notes\"\nuser = { login = \"kai\" }\n";

fn wait_until(what: &str, mut ok: impl FnMut() -> bool) {
    let deadline = std::time::Instant::now() + STEP;
    while !ok() {
        assert!(std::time::Instant::now() < deadline, "timed out: {what}");
        std::thread::sleep(Duration::from_millis(50));
    }
}

fn read(notes: &Path, name: &str) -> String {
    std::fs::read_to_string(notes.join(name)).unwrap_or_default()
}

#[test]
fn a_folder_of_notes_is_an_issue_source_created_edited_closed_and_watched() {
    let repo = fixture(1);
    let notes = repo.root().join("docs").join("issues");
    std::fs::create_dir_all(&notes).unwrap();
    std::fs::write(
        notes.join("7-orbit-flicker.md"),
        "---\nid: 7\ntitle: Orbit chart flickers\nstatus: doing\nlabels: [bug]\ncssclasses: wide\n---\nKeep this body.\n",
    )
    .unwrap();
    std::fs::write(
        notes.join("8-moons.md"),
        "---\nid: 8\ntitle: Moons of Jupiter\nstatus: todo\n---\n",
    )
    .unwrap();
    let state = repo.state_dir();
    std::fs::create_dir_all(&state).unwrap();
    std::fs::write(
        state.join("repo-state.toml"),
        "[issues_source]\nintegration = \"notes\"\nproject = \"docs/issues\"\n",
    )
    .unwrap();
    let (mut term, _config) = launch_unlinked(
        &repo,
        "pty-hosting-local-issues",
        CONFIG,
        COLS,
        "Add the feature",
    );
    term.wait_for("the notes listed", STARTUP, |s| {
        s.contains("#7  Orbit chart flickers") && s.contains("#8  Moons of Jupiter")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.screen().text()));

    // -- create: the section's `n`, the form, Ctrl+S ---------------------------
    let (col, row) = term.wait_for_text("Moons of Jupiter", STEP).unwrap();
    term.click(col + 2, row).expect("focus the section");
    term.send_keys("n").unwrap();
    term.wait_for("the local New issue form", STEP, |s| {
        s.contains("New local issue") && s.contains("Status")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.screen().text()));
    term.send_text("Parser crashes").unwrap();
    term.wait_for("the title typed", STEP, |s| s.contains("Parser crashes"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("C-s").unwrap();
    wait_until("the note is on disk", || {
        notes.join("9-parser-crashes.md").exists()
    });
    let made = read(&notes, "9-parser-crashes.md");
    assert!(
        made.starts_with(
            "---\nid: 9\ntitle: Parser crashes\nstatus: todo\nassignees: []\nlabels: []\n"
        ),
        "{made}"
    );
    term.wait_for("the list shows it", STEP, |s| {
        s.contains("#9  Parser crashes")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.screen().text()));

    // -- edit labels: the row menu, the picker, a typed new label --------------
    let screen = term.screen();
    let (c, r) = screen.find("Parser crashes").unwrap();
    term.right_click(c + 2, r).expect("the row menu");
    let screen = term
        .wait_for("the local row menu", STEP, |s| {
            s.contains("Open note") && s.contains("Close issue") && s.contains("Edit labels...")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.screen().text()));
    let (c, r) = screen.find("Edit labels...").unwrap();
    term.click(c + 1, r).expect("Edit labels");
    term.wait_for("the label picker with the folder's labels", STEP, |s| {
        s.contains("Edit labels") && s.contains("bug")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.screen().text()));
    term.send_text("ui").unwrap();
    term.wait_for("the typed value is offered", STEP, |s| {
        s.contains("Add \"ui\"")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.screen().text()));
    term.send_keys("Enter").unwrap();
    wait_until("the labels are on disk", || {
        read(&notes, "9-parser-crashes.md").contains("labels: [ui]\n")
    });
    // The other notes are untouched.
    assert_eq!(
        read(&notes, "7-orbit-flicker.md"),
        "---\nid: 7\ntitle: Orbit chart flickers\nstatus: doing\nlabels: [bug]\ncssclasses: wide\n---\nKeep this body.\n"
    );

    // -- the folder is watched: a note written from outside shows up -----------
    std::fs::write(
        notes.join("10-from-obsidian.md"),
        "---\nid: 10\ntitle: Written outside\nstatus: todo\n---\n",
    )
    .unwrap();
    term.wait_for("the outside note appears", STEP, |s| {
        s.contains("#10  Written outside")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.screen().text()));

    // -- close from the row menu ------------------------------------------------
    let screen = term.screen();
    let (c, r) = screen.find("Written outside").unwrap();
    term.right_click(c + 2, r).expect("the row menu");
    let screen = term
        .wait_for("the row menu", STEP, |s| s.contains("Close issue"))
        .unwrap_or_else(|e| panic!("{e}"));
    let (c, r) = screen.find("Close issue").unwrap();
    term.click(c + 1, r).expect("Close issue");
    wait_until("the status is done on disk", || {
        read(&notes, "10-from-obsidian.md").contains("status: done\n")
    });
    term.wait_for("the closed issue left the open list", STEP, |s| {
        s.contains("#9  Parser crashes") && !s.contains("Written outside")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.screen().text()));
    term.stop().unwrap();
}
