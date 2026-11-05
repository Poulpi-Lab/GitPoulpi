//! After a whole remote is hidden the Log is walked again without its
//! commits, and every row's hover tooltip must still describe the commit
//! drawn on that row (the Message column's tooltip is the full message,
//! body included, which the row itself never shows).
//!
//! The real TUI in a pty, the real `gitpoulpi` CLI for the hide, real `git`.

mod common;

use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;

const STEP: Duration = Duration::from_secs(30);
const COLS: u16 = 200;
const ROWS: u16 = 50;

fn cli() -> PathBuf {
    let tui = PathBuf::from(env!("CARGO_BIN_EXE_gitpoulpi-tui"));
    let path = tui.with_file_name(format!("gitpoulpi{}", std::env::consts::EXE_SUFFIX));
    if !path.exists() {
        let status = Command::new(env!("CARGO"))
            .args(["build", "-p", "gitpoulpi-cli"])
            .status()
            .expect("run cargo build");
        assert!(status.success(), "building the gitpoulpi CLI failed");
    }
    path
}

/// Whether `text` is drawn in the Log table (the detail pane at the right
/// still shows the selected commit's body).
fn in_log(screen: &xtask::pty::Screen, text: &str) -> bool {
    screen
        .find_all(text)
        .into_iter()
        .any(|(col, _)| (40..130).contains(&col))
}

#[test]
fn a_row_tooltip_after_hiding_a_remote_is_that_rows_message() {
    let mut b = FixtureRepoBuilder::new().unwrap();
    for n in 1..=4 {
        b = b
            .commit(&format!("origin work {n}\n\nBODY-ORIGIN-{n}"))
            .unwrap();
    }
    let repo = b.build();
    let mut a = FixtureRepoBuilder::new().unwrap();
    for n in 1..=3 {
        a = a
            .commit(&format!("archive work {n}\n\nBODY-ARCHIVE-{n}"))
            .unwrap();
    }
    let archive = a.build();
    let root = repo.root();
    repo.git(&[
        "remote",
        "add",
        "archive",
        &archive.root().to_string_lossy(),
    ]);
    repo.git(&["fetch", "-q", "archive"]);
    let config = tempfile::tempdir().unwrap();
    let session = "hidden-tooltip";
    let mut term = common::launch(root)
        .session(session)
        .config_dir(config.path())
        .size(COLS, ROWS)
        .mouse_capture()
        .ready_when("both histories in the Log", |s| {
            s.contains("origin work 4") && s.contains("archive work 3")
        })
        .spawn();
    // The pointer rests on `origin work 4` with its tooltip open while the
    // remote is hidden from elsewhere: the three archive rows above it leave
    // and the row under the pointer now holds `origin work 1`, so the
    // tooltip must not go on describing `origin work 4` there.
    let (col, row) = term
        .screen()
        .find_all("origin work 4")
        .into_iter()
        .find(|(c, _)| (40..130).contains(c))
        .expect("the origin work 4 row");
    term.move_to(col + 1, row).unwrap();
    term.move_to(col + 2, row).unwrap();
    term.wait_for("the tooltip of origin work 4", STEP, |s| {
        in_log(s, "BODY-ORIGIN-4")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let out = Command::new(cli())
        .env("GITPOULPI_CONFIG_DIR", config.path())
        .args(["--json", "--session", session])
        .args(["--repo", &root.to_string_lossy()])
        .args(["refs", "hide", "--remote", "archive"])
        .current_dir(root)
        .output()
        .expect("run gitpoulpi");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    term.wait_for("the archive commits gone", STEP, |s| {
        s.contains("origin work 4") && !s.contains("archive work")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    std::thread::sleep(Duration::from_millis(500));
    assert!(
        !in_log(&term.screen(), "BODY-ORIGIN-4"),
        "a tooltip of a commit that left the pointer's row\n{}",
        term.text()
    );
    term.move_to(0, 0).unwrap();

    // Hover each unselected commit's subject in turn: the tooltip is its body
    // (the selected one, `origin work 4`, already shows its body in the
    // detail pane).
    for n in (1..=3).rev() {
        let subject = format!("origin work {n}");
        let screen = term.screen();
        let (col, row) = screen
            .find_all(&subject)
            .into_iter()
            .find(|(c, _)| (40..130).contains(c))
            .unwrap_or_else(|| panic!("no row for {subject}\n{}", term.text()));
        term.move_to(col + 1, row).unwrap();
        term.move_to(col + 2, row).unwrap();
        let want = format!("BODY-ORIGIN-{n}");
        term.wait_for(&format!("the tooltip of {subject}"), STEP, |s| {
            s.contains(&want)
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
        let text = term.text();
        for other in (1..=3).filter(|k| *k != n) {
            assert!(
                !text.contains(&format!("BODY-ORIGIN-{other}")),
                "{subject}: tooltip of another commit\n{text}"
            );
        }
        term.move_to(0, 0).unwrap();
        term.wait_for_text_to_go(&want, STEP)
            .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    }
    let _ = term.stop();
}

/// `main` and three side branches off `base`, one commit each, every
/// message with a body the row itself never shows.
fn forked() -> gitpoulpi_testkit::FixtureRepo {
    let msg = |name: &str| format!("{name} commit\n\nBODY-{}", name.to_uppercase());
    FixtureRepoBuilder::new()
        .unwrap()
        .commit(&msg("base"))
        .unwrap()
        .branch("side1")
        .unwrap()
        .commit(&msg("s1"))
        .unwrap()
        .checkout("main")
        .unwrap()
        .branch("other")
        .unwrap()
        .commit(&msg("o1"))
        .unwrap()
        .checkout("main")
        .unwrap()
        .branch("side2")
        .unwrap()
        .commit(&msg("s2"))
        .unwrap()
        .checkout("main")
        .unwrap()
        .commit(&msg("m1"))
        .unwrap()
        .build()
}

fn row_of(term: &xtask::pty::Terminal, subject: &str) -> (u16, u16) {
    term.screen()
        .find_all(subject)
        .into_iter()
        .find(|(c, _)| (40..130).contains(c))
        .unwrap_or_else(|| panic!("no row for {subject}\n{}", term.text()))
}

/// Launches the `forked` repository and solos `main` and `side1` through
/// the CLI (the owner's state: several refs soloed): the Log then draws
/// `m1`, `s1`, `base`, while the held walk still has `o1` and `s2` between.
fn solo_main_and_side1(
    session: &str,
) -> (
    xtask::pty::Terminal,
    gitpoulpi_testkit::FixtureRepo,
    tempfile::TempDir,
) {
    let repo = forked();
    let root = repo.root().to_path_buf();
    let config = tempfile::tempdir().unwrap();
    let term = common::launch(&root)
        .session(session)
        .config_dir(config.path())
        .size(COLS, ROWS)
        .mouse_capture()
        .ready_when("every commit in the Log", |s| {
            s.contains("o1 commit") && s.contains("s2 commit") && s.contains("m1 commit")
        })
        .spawn();
    let out = Command::new(cli())
        .env("GITPOULPI_CONFIG_DIR", config.path())
        .args(["--json", "--session", session])
        .args(["--repo", &root.to_string_lossy()])
        .args(["refs", "solo", "main", "side1"])
        .current_dir(&root)
        .output()
        .expect("run gitpoulpi");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    // The detail pane shows the selected commit's body: the top row, `m1`,
    // not the commit at the same position of the unfiltered walk.
    term.wait_for("only main and side1 left", STEP, |s| {
        in_log(s, "s1 commit")
            && !in_log(s, "o1 commit")
            && !in_log(s, "s2 commit")
            && s.contains("BODY-M1")
    })
    .unwrap_or_else(|e| {
        panic!(
            "{e}
{}",
            term.text()
        )
    });
    (term, repo, config)
}

/// A row's tooltip must belong to the commit drawn there, not to the one at
/// the same position of the unfiltered walk.
#[test]
fn with_several_refs_soloed_a_row_tooltip_is_that_rows_message() {
    let (mut term, _repo, _config) = solo_main_and_side1("solo-tooltip");
    // `m1` is selected: hover the others.
    for name in ["s1", "base"] {
        let (col, row) = row_of(&term, &format!("{name} commit"));
        term.move_to(col + 1, row).unwrap();
        term.move_to(col + 2, row).unwrap();
        let want = format!("BODY-{}", name.to_uppercase());
        term.wait_for(&format!("the tooltip of {name}"), STEP, |s| {
            in_log(s, &want)
        })
        .unwrap_or_else(|e| {
            panic!(
                "{e}
{}",
                term.text()
            )
        });
        term.move_to(0, 0).unwrap();
        term.wait_for("the tooltip gone", STEP, |s| !in_log(s, &want))
            .unwrap_or_else(|e| {
                panic!(
                    "{e}
{}",
                    term.text()
                )
            });
    }
    let _ = term.stop();
}

/// The ref-less `base` shows its ghost pill (the branch it belongs to) on
/// its own row, deduced from the soloed refs and the filtered graph.
#[test]
fn with_several_refs_soloed_the_ghost_ref_is_on_the_hovered_row() {
    let (mut term, _repo, _config) = solo_main_and_side1("solo-ghost");
    let (col, row) = row_of(&term, "base commit");
    assert!(!term.screen().line(row).contains("main"), "no ghost yet");
    term.move_to(col + 1, row).unwrap();
    term.wait_for("the ghost pill on the base row", STEP, |s| {
        s.line(row).contains("main")
    })
    .unwrap_or_else(|e| {
        panic!(
            "{e}
{}",
            term.text()
        )
    });
    // The others keep their own pills and gain no ghost.
    let (_, s1_row) = row_of(&term, "s1 commit");
    assert!(!term.screen().line(s1_row).contains("main"));
    let _ = term.stop();
}
