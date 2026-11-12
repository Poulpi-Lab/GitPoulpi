//! A press outside an open menu closes it, whatever the menu: the owner's
//! rule for every context menu and dropdown. Each family is checked with a
//! press on a neutral cell right next to the menu (the surface the menu hangs
//! over), not only the far corner.
//!
//! Same rules as `pty_ui.rs`: nothing located by a hard-coded coordinate,
//! every wait describes the whole state it waits for.

mod common;

use std::path::Path;
use std::time::Duration;

use gitpoulpi_testkit::{ConflictFixture, FixtureRepoBuilder};
use xtask::pty::{Screen, Terminal};

const STEP: Duration = Duration::from_secs(30);
const LOG_HEADER: &str = "Message";

fn repo_with(file: &str, base: &[u8], bytes: &[u8]) -> gitpoulpi_testkit::FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .write_worktree(file, base)
        .unwrap()
        .stage(".")
        .unwrap()
        .commit("base")
        .unwrap()
        .write_worktree(file, bytes)
        .unwrap()
        .build()
}

fn launch(repo: &Path, session: &str) -> (Terminal, tempfile::TempDir) {
    common::launch(repo)
        .session(session)
        .size(180, 40)
        .no_exit_pause()
        .ready_when("the working-changes pane", |s| {
            s.contains("Unstaged") && s.contains(LOG_HEADER)
        })
        .mouse_capture()
        .start()
}

fn open_diff(term: &mut Terminal, file: &str) {
    common::click_text_until(term, file, "the unstaged diff", |s| {
        s.contains(&format!("{file} - unstaged")) && s.contains(" Edit ")
    });
}

fn chip(s: &Screen, label: &str) -> (u16, u16) {
    let (_, bar) = s
        .find("Ln 1, Col")
        .unwrap_or_else(|| panic!("{}", s.text()));
    let line = s.line(bar);
    let at = line
        .find(label)
        .unwrap_or_else(|| panic!("{label} on the bar: {line:?}"));
    (line[..at].chars().count() as u16, bar)
}

/// The editor's status chips' dropdown closes on a press in the text next to it.
#[test]
fn the_editor_chip_menu_closes_on_a_press_in_the_text() {
    let repo = repo_with("lines.txt", b"x\n", b"one\r\ntwo\r\nthree\r\n");
    let (mut term, _c) = launch(repo.root(), "pty-away-editor-chip");
    open_diff(&mut term, "lines.txt");
    common::click_text_until(&mut term, " Edit ", "the editor", |s| {
        s.contains("Edit - lines.txt @ working tree") && s.contains("three")
    });
    let s = term.screen();
    let (c, r) = chip(&s, "CRLF");
    common::retry_until(
        &mut term,
        "the line-break menu",
        8,
        Duration::from_secs(4),
        |t| t.click(c + 1, r).unwrap(),
        |s| s.contains("Convert to LF"),
    );
    let (tc, tr) = term.screen().find("three").unwrap();
    term.click(tc + 60, tr).unwrap();
    term.wait_for("the menu gone, the file untouched", STEP, |s| {
        !s.contains("Convert to LF") && !s.contains("\u{25CF} modified")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

fn many_lines() -> (String, String) {
    let base: String = (1..=30).map(|n| format!("l{n}\n")).collect();
    let changed = base.replace("l15", "CHANGED-MIDDLE");
    (base, changed)
}

/// The diff pane's hunk menu closes on a press in the diff text beside it.
#[test]
fn the_diff_hunk_menu_closes_on_a_press_in_the_diff() {
    let (base, changed) = many_lines();
    let repo = repo_with("c.txt", base.as_bytes(), changed.as_bytes());
    let (mut term, _c) = launch(repo.root(), "pty-away-diff-hunk");
    open_diff(&mut term, "c.txt");
    common::retry_until(
        &mut term,
        "the hunk menu",
        8,
        Duration::from_secs(4),
        |t| {
            if let Some((c, r)) = t.screen().find("CHANGED-MIDDLE") {
                t.right_click(c + 1, r).unwrap();
            }
        },
        |s| s.contains("Copy hunk"),
    );
    let s = term.screen();
    let (mc, mr) = s.find("Copy hunk").unwrap();
    println!("menu at {mc},{mr}\n{}", s.text());
    // Next to the menu: same row, a few columns to its right.
    let (lc, lr) = term.screen().find("l13").unwrap();
    term.click(lc, lr).unwrap();
    term.wait_for("the menu gone", STEP, |s| !s.contains("Copy hunk"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

/// The diff pane's own controls act with no menu open and not with one: a
/// press on the bar's `Split` closes the hunk menu and leaves the view inline.
#[test]
fn the_diff_hunk_menu_closes_on_a_press_on_a_diff_bar_button_without_running_it() {
    let (base, changed) = many_lines();
    let repo = repo_with("c.txt", base.as_bytes(), changed.as_bytes());
    let (mut term, _c) = launch(repo.root(), "pty-away-diff-bar");
    open_diff(&mut term, "c.txt");
    open_hunk_menu(&mut term);
    let (c, r) = term.screen().find("\u{25A6} Split").unwrap();
    term.click(c + 3, r).unwrap();
    term.wait_for("the menu gone, still the inline view", STEP, |s| {
        !s.contains("Copy hunk") && s.contains("+CHANGED-MIDDLE") && s.contains("-l15")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

fn open_hunk_menu(term: &mut Terminal) {
    common::retry_until(
        term,
        "the hunk menu",
        8,
        Duration::from_secs(4),
        |t| {
            if let Some((c, r)) = t.screen().find("CHANGED-MIDDLE") {
                t.right_click(c + 1, r).unwrap();
            }
        },
        |s| s.contains("Copy hunk"),
    );
}

/// A press on the diff's vertical scrollbar closes the hunk menu.
#[test]
fn the_diff_hunk_menu_closes_on_a_press_on_the_diff_scrollbar() {
    let base: String = (1..=200).map(|n| format!("l{n}\n")).collect();
    // Every line past the 40th changes: a diff far taller than the pane, so it has a bar.
    let changed: String = (1..=200)
        .map(|n| match n {
            15 => "CHANGED-MIDDLE\n".to_string(),
            n if n > 40 => format!("x{n}\n"),
            n => format!("l{n}\n"),
        })
        .collect();
    let repo = repo_with("c.txt", base.as_bytes(), changed.as_bytes());
    let (mut term, _c) = launch(repo.root(), "pty-away-diff-scroll");
    open_diff(&mut term, "c.txt");
    open_hunk_menu(&mut term);
    // The scrollbar's up arrow: the first body row's `↑` right of the title.
    let s = term.screen();
    let (_, tr) = s.find("c.txt - unstaged").unwrap();
    let line = s.line(tr + 1);
    let up = line
        .chars()
        .position(|ch| ch == '\u{2191}')
        .unwrap_or_else(|| panic!("{line:?}")) as u16;
    term.click(up, tr + 1).unwrap();
    term.wait_for("the menu gone", STEP, |s| !s.contains("Copy hunk"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

/// The blame view's right-click menu closes on a press in the code.
#[test]
fn the_blame_menu_closes_on_a_press_in_the_code() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("notes.txt", "row one\nrow two\nrow three\n", "add notes")
        .unwrap()
        .build();
    let (mut term, _c) = common::launch(repo.root())
        .no_commit_column()
        .session("pty-away-blame")
        .size(200, 40)
        .no_exit_pause()
        .ready_when("the commit detail of HEAD", |s| {
            s.contains("View all files") && s.contains(LOG_HEADER)
        })
        .mouse_capture()
        .start();
    common::click_text_until(&mut term, "notes.txt", "the diff bar", |s| {
        s.contains(" Blame ")
    });
    common::click_text_until(&mut term, " Blame ", "the blame view", |s| {
        s.contains("Blame - notes.txt @") && s.contains("row three")
    });
    common::retry_until(
        &mut term,
        "the blame menu",
        8,
        Duration::from_secs(4),
        |t| {
            if let Some((c, r)) = t.screen().find("row two") {
                t.right_click(c + 1, r).unwrap();
            }
        },
        |s| s.contains("Copy commit SHA"),
    );
    let (c, r) = term.screen().find("row one").unwrap();
    term.click(c, r).unwrap();
    term.wait_for("the menu gone, the blame still open", STEP, |s| {
        !s.contains("Copy commit SHA") && s.contains("Blame - notes.txt @")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

/// The merge editor's chunk menu closes on a press in the Output.
#[test]
fn the_merge_editor_menu_closes_on_a_press_in_the_output() {
    let fixture = ConflictFixture::multi_conflict();
    let (mut term, _c) = common::launch(fixture.root())
        .session("pty-away-merge")
        .size(200, 50)
        .no_exit_pause()
        .ready_when("the conflicted file in the Status pane", |s| {
            s.contains("Unstaged") && s.contains("Merge in progress")
        })
        .mouse_capture()
        .start();
    common::click_text_until(&mut term, "a.txt", "the merge editor", |s| {
        s.contains("Conflict 1 / 3")
    });
    common::retry_until(
        &mut term,
        "the chunk menu",
        8,
        Duration::from_secs(4),
        |t| {
            let s = t.screen();
            let Some((c, r)) = s.find("[ ] Conflict 1") else {
                return;
            };
            // B's header is the second one on the row.
            let b = (c + 1..s.cols()).find(|&k| {
                let mut text = String::new();
                for j in k..(k + 14).min(s.cols()) {
                    text.push_str(s.cell(j, r).map_or("", |cell| cell.display()));
                }
                text.starts_with("[ ] Conflict 1")
            });
            if let Some(b) = b {
                t.right_click(b + 1, r).unwrap();
            }
        },
        |s| s.contains("Take B only"),
    );
    let s = term.screen();
    let title = (0..s.rows())
        .find(|r| s.line(*r).contains("Output \u{b7}"))
        .unwrap();
    // The Output's title rule: no row, no control, nothing to run.
    term.click(60, title).unwrap();
    term.wait_for("the menu gone, the editor still open", STEP, |s| {
        !s.contains("Take B only") && s.contains("Conflict 1 / 3")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

/// A tab's right-click menu closes on a press in the Log.
#[test]
fn the_tab_menu_closes_on_a_press_in_the_log() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "first change")
        .unwrap()
        .build();
    let (mut term, _c) = common::launch(repo.root())
        .session("pty-away-tab")
        .size(180, 40)
        .no_exit_pause()
        .ready("first change")
        .mouse_capture()
        .start();
    common::retry_until(
        &mut term,
        "the tab menu",
        8,
        Duration::from_secs(4),
        |t| {
            // The tab bar is the row under the title bar.
            if let Some((c, r)) = t
                .screen()
                .find_all("repo")
                .into_iter()
                .find(|&(_, r)| r == 1)
            {
                t.right_click(c + 1, r).unwrap();
            }
        },
        |s| s.contains("Close tab"),
    );
    let (c, r) = term.screen().find("first change").unwrap();
    term.click(c + 1, r).unwrap();
    term.wait_for("the menu gone", STEP, |s| !s.contains("Close tab"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

/// The toolbar's `Pull ▾` dropdown closes on a press in the Log.
#[test]
fn the_pull_dropdown_closes_on_a_press_in_the_log() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "first change")
        .unwrap()
        .build();
    let (mut term, _c) = common::launch(repo.root())
        .session("pty-away-pull")
        .size(180, 40)
        .no_exit_pause()
        .ready("first change")
        .mouse_capture()
        .start();
    common::retry_until(
        &mut term,
        "the pull dropdown",
        8,
        Duration::from_secs(4),
        |t| {
            if let Some((c, r)) = t.screen().find("Pull \u{25BE}") {
                t.click(c + 5, r).unwrap();
            }
        },
        |s| s.contains("Fetch All"),
    );
    let (c, r) = term.screen().find("first change").unwrap();
    term.click(c + 1, r).unwrap();
    term.wait_for("the dropdown gone", STEP, |s| !s.contains("Fetch All"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}
