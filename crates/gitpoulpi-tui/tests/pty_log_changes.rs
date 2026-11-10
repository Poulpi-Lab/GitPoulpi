//! The Log "Changes" column: hidden by default, turned on through the
//! header menu, it fills in `N files +I -D` for the commits on screen (the
//! daemon computes them lazily, one batch per viewport). Temp config dir
//! and fixture repo only.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;

const STARTUP: Duration = Duration::from_secs(60);
const STEP: Duration = Duration::from_secs(30);

fn header_row(screen: &xtask::pty::Screen) -> Option<usize> {
    screen
        .find_all("Message")
        .into_iter()
        .next()
        .map(|(_, r)| r as usize)
}

#[test]
fn the_changes_column_is_hidden_by_default_and_fills_in_for_the_visible_rows() {
    let config = tempfile::tempdir().unwrap();
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "1\n2\n", "add a")
        .unwrap()
        .commit_file("a.txt", "1\nX\n", "edit a")
        .unwrap()
        .commit_file("b.txt", "b\nb\nb\n", "add b")
        .unwrap()
        .build();

    let mut term = common::launch(repo.root())
        .session("changes-col")
        .config_dir(config.path())
        .size(200, 60)
        .spawn();
    term.wait_for("the fixture loaded", STARTUP, |s| s.contains("edit a"))
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(
        !snap(&term).contains("Changes"),
        "hidden by default: {}",
        term.text()
    );
    assert!(!snap(&term).contains("1 file"), "{}", term.text());

    // Right-click the SHA header for the column menu and tick Changes.
    let (col, row) = {
        let screen = snap(&term);
        let header = header_row(&screen).expect("the Log header");
        let (c, r) = screen
            .find_all("SHA")
            .into_iter()
            .find(|(_, r)| *r as usize == header)
            .expect("the SHA header");
        (c, r)
    };
    term.right_click(col + 1, row).unwrap();
    term.wait_for("the column menu", STEP, |s| s.contains("[x]"))
        .unwrap_or_else(|e| panic!("{e}"));
    let (mc, mr) = {
        let screen = snap(&term);
        screen
            .find_all("Changes")
            .into_iter()
            .find(|(_, r)| *r != row)
            .expect("a Changes row in the column menu")
    };
    term.click(mc + 1, mr).unwrap();
    term.send_keys("Escape").unwrap();

    // The counts arrive for the visible rows: the file with 2 lines, the
    // one-line edit, the 3-line file.
    term.wait_for("the counts", STEP, |s| {
        s.contains("1 file +2 -0") && s.contains("1 file +1 -1") && s.contains("1 file +3 -0")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert!(snap(&term).contains("Changes"), "{}", term.text());

    term.send_keys("q").unwrap();
    for _ in 0..100 {
        if term.exited() {
            break;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    let _ = term.stop();
}

/// The screen once it has settled: a torn frame is not a place to locate a
/// cell or compare colors in.
fn snap(term: &xtask::pty::Terminal) -> xtask::pty::Screen {
    term.wait_for_stable(
        std::time::Duration::from_millis(300),
        std::time::Duration::from_secs(30),
    )
    .unwrap_or_else(|e| panic!("{e}"))
}
