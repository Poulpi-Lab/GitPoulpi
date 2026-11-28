//! A real double click is two presses a human apart (50-250 ms), with a frame
//! painted in between; `Terminal::double_click` writes both in one go, so no
//! frame ever separates them. A combobox kept its "first click" in
//! `popup_list::View`, which `begin_frame` wipes when the popup did not paint
//! through it: every searchable combobox (Settings > Language, the Open in
//! list once it has more than seven entries) never saw a second click, which
//! the owner met on Windows as "double click does nothing" and "Open in does
//! nothing". Runs on every OS (no shell script involved).

mod common;

use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(30);

fn ms(n: u64) {
    std::thread::sleep(Duration::from_millis(n));
}

/// Press, hold, release: a click as a hand makes it.
fn hand_click(term: &mut Terminal, col: u16, row: u16) {
    term.press(col, row).unwrap();
    ms(60);
    term.release(col, row).unwrap();
}

fn stored(config: &std::path::Path) -> String {
    std::fs::read_to_string(config.join("tui.toml")).unwrap_or_default()
}

#[test]
fn a_hand_paced_double_click_picks_in_a_searchable_combobox() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("gap probe")
        .unwrap()
        .build();
    let (mut term, config) = common::launch(repo.root())
        .session("pty-gap-combo")
        .size(200, 50)
        .ready("gap probe")
        .mouse_capture()
        .start();
    let (col, row) = term
        .wait_for_text("\u{2699}", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(col, row).unwrap();
    term.wait_for("Settings open", STEP, |s| {
        s.contains("Repo-specific settings")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    let (col, row) = term
        .wait_for_text("General", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(col + 1, row).unwrap();
    let (col, row) = term
        .wait_for_text("\u{25BE}", STEP)
        .unwrap_or_else(|e| panic!("the language choice: {e}\n{}", term.text()));
    term.click(col, row).unwrap();
    let s = term
        .wait_for("the language popup", STEP, |s| s.contains("\u{2713}"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (c, tick_row) = s.find("\u{2713}").unwrap();
    let target = tick_row + 2;
    // Let the popup settle, then click, a frame, click.
    ms(600);
    hand_click(&mut term, c + 3, target);
    ms(150);
    assert!(
        term.screen().contains("\u{2713}"),
        "the first click only selects: the popup stays"
    );
    hand_click(&mut term, c + 3, target);
    let deadline = std::time::Instant::now() + STEP;
    while !stored(config.path()).contains("language") {
        assert!(
            std::time::Instant::now() < deadline,
            "the second click of a hand-paced pair did not pick:\n{}",
            term.text()
        );
        ms(200);
    }
    term.stop().unwrap();
}

#[test]
fn one_click_on_a_long_open_in_list_runs_the_entry() {
    let alpha = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "a\n", "alpha-only-commit")
        .unwrap()
        .build();
    let bravo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("b.txt", "b\n", "bravo-only-commit")
        .unwrap()
        .build();
    let config = tempfile::tempdir().unwrap();
    std::fs::write(
        config.path().join("recent.toml"),
        format!(
            "[[repos]]\npath = '{}'\nname = 'bravo-repo'\nlast_opened = 1\n",
            bravo.root().display()
        ),
    )
    .unwrap();
    // More than seven entries: the list gets a search line, and a click on a
    // searchable list used to need a second one.
    let command = if cfg!(windows) {
        "cmd /c exit 0"
    } else {
        "true"
    };
    let mut toml = String::from("[ui]\ngraph_renderer = \"text\"\n");
    for n in 1..=9 {
        toml.push_str(&format!(
            "[[ui.open_in]]\nlabel = \"Entry number {n}\"\ncommand = \"{command}\"\n"
        ));
    }
    std::fs::write(config.path().join("config.toml"), toml).unwrap();
    let mut term = common::launch(alpha.root())
        .session("pty-gap-open-in")
        .config_dir(config.path())
        .size(200, 50)
        .no_exit_pause()
        .ready("alpha-only-commit")
        .spawn();
    term.send_keys("C-o").unwrap();
    term.wait_for("the management tab", STEP, |s| {
        s.contains("Repository Management") && s.contains("bravo-repo")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("/").unwrap();
    term.send_text("bravo").unwrap();
    term.wait_for("the filter applied", STEP, |s| {
        s.contains("Enter open best match")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    term.wait_for("the Open in button", STEP, |s| {
        s.contains("bravo-only-commit")
            && !s.contains("Repository Management")
            && s.contains("Open in")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (col, row) = term
        .screen()
        .find_all("Open in")
        .into_iter()
        .next()
        .unwrap();
    hand_click(&mut term, col + 1, row);
    let s = term
        .wait_for("the list", STEP, |s| {
            s.contains("Entry number 3") && s.contains("Configure")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (c, r) = s.find("Entry number 3").unwrap();
    hand_click(&mut term, c + 1, r);
    term.wait_for("one click ran the entry and closed the list", STEP, |s| {
        !s.contains("Entry number 3")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}
