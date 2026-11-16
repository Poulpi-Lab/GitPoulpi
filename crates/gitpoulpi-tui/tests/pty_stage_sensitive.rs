//! Staging a file that looks sensitive asks first, driven through a pty: the
//! real binary, the real daemon and real `git`. Keyboard (`Ctrl+G a`), the
//! toolbar-free mouse path (the dialog's own rows and cross) and the refusal
//! that changes nothing.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::{Screen, Terminal};

const STEP: Duration = Duration::from_secs(30);

fn fixture() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "initial commit")
        .unwrap()
        .write_worktree(".env", b"MODE=dev\n")
        .unwrap()
        .write_worktree("notes.txt", b"plain notes\n")
        .unwrap()
        .build()
}

fn launch(repo: &FixtureRepo, session: &str) -> Terminal {
    common::launch(repo.root())
        .session(session)
        .size(200, 50)
        .ready_when("the fixture loaded with its two new files", |s| {
            s.contains("initial commit") && s.contains(".env") && s.contains("notes.txt")
        })
        .start()
        .0
}

fn staged(repo: &FixtureRepo) -> String {
    repo.git(&["diff", "--cached", "--name-only"])
}

fn stage_all(term: &mut Terminal) -> Screen {
    term.send_keys("C-g").expect("Ctrl+G");
    term.send_keys("a").expect("a");
    term.wait_for("the sensitive files question", STEP, |s| {
        s.contains("Sensitive files")
            && s.contains("1. Stage anyway")
            && s.contains("2. Cancel, stage nothing")
    })
    .unwrap_or_else(|e| panic!("{e}"))
}

fn wait_staged(repo: &FixtureRepo, what: &str) {
    let deadline = std::time::Instant::now() + STEP;
    while !staged(repo).contains(what) {
        assert!(
            std::time::Instant::now() < deadline,
            "{what} was never staged: {:?}",
            staged(repo)
        );
        std::thread::sleep(Duration::from_millis(200));
    }
}

#[test]
fn stage_all_asks_lists_the_file_and_stages_nothing_until_stage_anyway() {
    let repo = fixture();
    let mut term = launch(&repo, "pty-stage-sensitive-keys");
    let screen = stage_all(&mut term);
    let text = screen.text();
    assert!(text.contains(".env"), "{text}");
    assert!(text.contains("matches .env"), "{text}");
    assert!(
        text.contains("1. Stage anyway") && text.contains("2. Cancel"),
        "{text}"
    );
    assert_eq!(
        staged(&repo),
        "",
        "nothing is staged while the question is up"
    );

    // No default: Enter with nothing highlighted answers nothing.
    term.send_keys("Enter").expect("Enter");
    term.wait_for_stable(Duration::from_millis(300), STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(
        term.screen().contains("Sensitive files"),
        "Enter alone is not an answer"
    );
    assert_eq!(staged(&repo), "");

    // Esc cancels: still nothing staged.
    term.send_keys("Escape").expect("Esc");
    term.wait_for_text_to_go("Sensitive files", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(
        staged(&repo),
        "",
        "Cancel stages nothing, not even the plain file"
    );

    // Again: Down then Enter is "Stage anyway".
    stage_all(&mut term);
    term.send_keys("Down").expect("Down");
    term.wait_for_text("> 1. Stage anyway", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").expect("Enter");
    term.wait_for_text_to_go("Sensitive files", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    wait_staged(&repo, ".env");
    assert!(staged(&repo).contains("notes.txt"));
}

#[test]
fn the_rows_and_the_cross_answer_by_mouse() {
    let repo = fixture();
    let mut term = launch(&repo, "pty-stage-sensitive-mouse");

    // The close cross cancels.
    let screen = stage_all(&mut term);
    let (_, title_row) = screen.find("Sensitive files").unwrap();
    let border = screen.line(title_row);
    let cross = border
        .rfind('\u{2715}')
        .map(|byte| border[..byte].chars().count() as u16)
        .expect("a close cross on the border row");
    term.click(cross, title_row).expect("click the cross");
    term.wait_for_text_to_go("Sensitive files", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(staged(&repo), "");

    // "Cancel, stage nothing" by a click.
    let screen = stage_all(&mut term);
    let (col, row) = screen.find("2. Cancel").unwrap();
    term.click(col + 3, row).expect("click Cancel");
    term.wait_for_text_to_go("Sensitive files", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(staged(&repo), "");

    // "Stage anyway" by a click.
    let screen = stage_all(&mut term);
    let (col, row) = screen.find("1. Stage anyway").unwrap();
    term.click(col + 3, row).expect("click Stage anyway");
    term.wait_for_text_to_go("Sensitive files", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    wait_staged(&repo, ".env");
}

/// A repository whose `leak.txt` holds a token on line 55 of 70, so the line
/// is off screen unless the File View really scrolled to it.
fn leaky() -> FixtureRepo {
    let mut text = String::new();
    for i in 1..=70 {
        if i == 55 {
            text.push_str("TOKEN=ghp_q7Xk2LmZp9RtV4bN8wYcJ3hGfD5sAe6U1oIx\n");
        } else {
            text.push_str(&format!("fill {i}\n"));
        }
    }
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "initial commit")
        .unwrap()
        .write_worktree("leak.txt", text.as_bytes())
        .unwrap()
        .build()
}

fn launch_leaky(repo: &FixtureRepo, session: &str) -> Terminal {
    common::launch(repo.root())
        .session(session)
        .size(200, 50)
        .ready_when("the fixture loaded with its new file", |s| {
            s.contains("initial commit") && s.contains("leak.txt")
        })
        .start()
        .0
}

/// The question names the line of the secret and "Show in file" opens the
/// file scrolled to it; closing the File View brings the question back.
/// Keyboard (`S`), the button, and the palette entry.
#[test]
fn show_in_file_names_the_line_scrolls_to_it_and_comes_back() {
    let repo = leaky();
    let mut term = launch_leaky(&repo, "pty-stage-sensitive-show");

    let screen = stage_all(&mut term);
    assert!(screen.contains("leak.txt:55"), "{}", screen.text());
    assert!(screen.contains("Show in file"), "{}", screen.text());

    // Keyboard twin.
    term.send_keys("s").expect("s");
    term.wait_for("the file scrolled to the secret's line", STEP, |s| {
        !s.contains("Sensitive files") && s.contains("fill 54") && !s.contains("fill 10")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert!(
        term.screen().contains("ghp_"),
        "the secret's own line is on screen"
    );
    term.send_keys("Escape").expect("Esc closes the File View");
    term.wait_for("the question is back", STEP, |s| {
        s.contains("Sensitive files") && s.contains("leak.txt:55")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(staged(&repo), "", "looking stages nothing");

    // The button.
    let (col, row) = term.screen().find("Show in file").unwrap();
    term.click(col + 1, row).expect("click Show in file");
    term.wait_for("the file again", STEP, |s| {
        !s.contains("Sensitive files") && s.contains("fill 54")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Escape").expect("Esc");
    term.wait_for_text("Sensitive files", STEP)
        .unwrap_or_else(|e| panic!("{e}"));

    // The palette entry, reachable while the question is open.
    term.send_keys("C-p").expect("Ctrl+P");
    term.send_text("show in file").expect("type");
    term.wait_for("the palette entry", STEP, |s| {
        s.contains("Sensitive Files: Show in File")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").expect("Enter");
    term.wait_for("the file by the palette", STEP, |s| {
        !s.contains("Sensitive files") && s.contains("fill 54")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Escape").expect("Esc");
    term.wait_for_text("Sensitive files", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(staged(&repo), "");
}
