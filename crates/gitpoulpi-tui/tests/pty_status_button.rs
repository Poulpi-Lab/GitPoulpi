//! The Status pane's hover button (`Stage file` / `Unstage file`) is drawn
//! over the end of the row, right-aligned on a solid background, so it is
//! whole however long the file name is -- and a modified file wears a
//! pencil, not a tilde. Driven through a pty by `xtask::pty`.

mod common;

use common::git_trimmed as git;

use std::path::Path;
use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(20);
const LONG: &str = "this_is_a_really_long_file_name_that_overflows_the_pane_width_easily.txt";

fn launch(repo: &Path, session: &str, config: &Path) -> Terminal {
    launch_at(repo, session, config, 140, "this_is_a_really")
}

fn launch_at(repo: &Path, session: &str, config: &Path, cols: u16, file: &'static str) -> Terminal {
    common::launch(repo)
        .session(session)
        .config_dir(config)
        .size(cols, 40)
        .ready_when("the working changes", |s| {
            s.contains("Unstaged") && s.contains(file)
        })
        .spawn()
}

fn fixture() -> gitpoulpi_testkit::FixtureRepo {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file(LONG, "one\n", "initial")
        .unwrap()
        .build();
    std::fs::write(repo.root().join(LONG), "changed\n").unwrap();
    repo
}

#[test]
fn a_modified_file_wears_a_pencil_and_hovering_a_long_name_shows_the_whole_button() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch(repo.root(), "sb-button", config.path());

    let screen = term.screen();
    assert!(
        screen.contains("\u{270E} this_is_a_really"),
        "a modified file's marker is a pencil:\n{}",
        screen.text()
    );
    assert!(!screen.contains("~ this_is_a_really"));

    let (col, row) = screen.find("this_is_a_really").expect("the file row");
    term.move_to(col + 3, row).unwrap();
    let shown = term
        .wait_for("the whole Stage file button", STEP, |s| {
            s.contains(" Stage file ")
        })
        .unwrap_or_else(|e| panic!("{e}"));

    // The button sits on the file's own row, whole, and right of the name's
    // start; nothing of it is cut by the pane's border.
    let (bcol, brow) = shown.find(" Stage file ").unwrap();
    assert_eq!(brow, row);
    assert!(bcol > col);
    let line: Vec<char> = shown.line(brow).chars().collect();
    let after = line
        .get((bcol as usize) + " Stage file ".chars().count())
        .copied();
    assert_eq!(
        after,
        Some('\u{2502}'),
        "the button ends against the pane's border: {:?}",
        shown.line(brow)
    );

    // And it works: the file moves to the Staged half.
    // Retried: under load the click can land on a torn frame.
    common::retry_until(
        &mut term,
        "the file staged",
        4,
        Duration::from_secs(8),
        |t| {
            if t.screen().contains("Stage file") {
                t.click(bcol + 3, brow).unwrap();
            }
        },
        |s| {
            s.find("Staged Files").is_some() && s.contains(" Unstage file ")
                || !s.contains("Stage file")
        },
    );
    // The index, not the screen: the button can leave the screen (hover
    // gone, torn frame) before the stage lands.
    let deadline = std::time::Instant::now() + Duration::from_secs(30);
    while git(repo.root(), &["diff", "--cached", "--name-only"]) != LONG {
        assert!(
            std::time::Instant::now() < deadline,
            "the file was never staged:\n{}",
            term.text()
        );
        std::thread::sleep(Duration::from_millis(100));
    }
    term.stop().unwrap();
}

/// In a very narrow WIP pane the `Stage all changes` button shrinks to a
/// compact glyph drawn over the header's end - it must stay clickable.
#[test]
fn stage_all_stays_reachable_in_a_very_narrow_wip_pane() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch_at(repo.root(), "sb-narrow-bulk", config.path(), 60, "this_is");
    let screen = term.screen();
    let (_, row) = screen.find("Unstaged").expect("the Unstaged header");
    let cells: Vec<char> = screen.line(row).chars().collect();
    let plus = (0..cells.len().saturating_sub(2))
        .rev()
        .find(|&i| cells[i] == ' ' && cells[i + 1] == '+' && cells[i + 2] == ' ')
        .unwrap_or_else(|| panic!("no compact Stage All on the header:\n{}", screen.text()));
    term.click(plus as u16 + 1, row).unwrap();
    let deadline = std::time::Instant::now() + STEP;
    while git(repo.root(), &["diff", "--cached", "--name-only"]) != LONG {
        assert!(
            std::time::Instant::now() < deadline,
            "Stage All did not stage:\n{}",
            term.screen().text()
        );
        std::thread::sleep(Duration::from_millis(200));
    }
    term.stop().unwrap();
}
