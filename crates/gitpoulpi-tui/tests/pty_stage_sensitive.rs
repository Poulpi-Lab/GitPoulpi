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
        s.contains("Sensitive files") && s.contains("Stage anyway")
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
