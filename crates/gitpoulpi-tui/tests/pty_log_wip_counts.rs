//! The WIP row's change counts follow the Log's Changes column. Hidden,
//! they sit in the WIP message (`ui.log_wip_counts_in_message`, on by
//! default); shown, they move into the Changes column. Temp config dir and
//! fixture repo only.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;

const STARTUP: Duration = Duration::from_secs(60);
const STEP: Duration = Duration::from_secs(30);

#[test]
fn toggling_the_changes_column_moves_the_wip_counts_between_message_and_column() {
    let config = tempfile::tempdir().unwrap();
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "1\n2\n", "add a")
        .unwrap()
        .build();
    std::fs::write(repo.root().join("a.txt"), "changed\n").unwrap();

    let mut term = common::launch(repo.root())
        .session("wip-counts")
        .config_dir(config.path())
        .size(200, 60)
        .spawn();
    term.wait_for("the WIP counts in the message", STARTUP, |s| {
        s.contains("\u{270E} 1")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    let wip_row = |s: &xtask::pty::Screen| -> Option<u16> {
        s.find_all("\u{270E} 1").into_iter().next().map(|(_, r)| r)
    };
    let (msg_col, row) = {
        let s = term.screen();
        let (c, r) = s.find_all("\u{270E} 1").into_iter().next().unwrap();
        (c, r)
    };

    // Show the Changes column through the header menu.
    let (col, hrow) = {
        let screen = term.screen();
        let header = screen
            .find_all("Message")
            .into_iter()
            .next()
            .map(|(_, r)| r)
            .expect("the Log header");
        screen
            .find_all("SHA")
            .into_iter()
            .find(|(_, r)| *r == header)
            .expect("the SHA header")
    };
    term.right_click(col + 1, hrow).unwrap();
    term.wait_for("the column menu", STEP, |s| s.contains("[x]"))
        .unwrap_or_else(|e| panic!("{e}"));
    let (mc, mr) = {
        let screen = term.screen();
        screen
            .find_all("[ ] Changes")
            .into_iter()
            .next()
            .expect("a Changes row in the column menu")
    };
    term.click(mc + 1, mr).unwrap();
    term.send_keys("Escape").unwrap();

    term.wait_for("the counts moved right", STEP, |s| {
        wip_row(s).is_some_and(|r| r == row)
            && s.find_all("\u{270E} 1").iter().all(|(c, _)| *c > msg_col)
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert!(term.screen().contains("Changes"), "{}", term.text());

    term.send_keys("q").unwrap();
    for _ in 0..100 {
        if term.exited() {
            break;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    let _ = term.stop();
}
