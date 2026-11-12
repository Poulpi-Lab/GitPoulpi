//! The session name is on screen once: in the status bar, not the main
//! header (both were plain text; the status bar keeps it). The header's
//! path and the "Open in" menu are separated by " - ", and the menu's click
//! area follows the paint. Temp config dir and fixture repos only.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(30);
const STARTUP: Duration = Duration::from_secs(60);
const SESSION: &str = "hdrsess-uniq";

fn start_on_bravo() -> (
    Terminal,
    tempfile::TempDir,
    [gitpoulpi_testkit::FixtureRepo; 2],
) {
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
    std::fs::write(
        config.path().join("config.toml"),
        "[ui]\ngraph_renderer = \"text\"\n\
         [[ui.open_in]]\nlabel = \"First entry\"\ncommand = \"true\"\n\
         [[ui.open_in]]\nlabel = \"Second entry\"\ncommand = \"true\"\n",
    )
    .unwrap();
    let mut term = common::launch(alpha.root())
        .session(SESSION)
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
    term.wait_for("bravo open as a tab", STARTUP, |s| {
        s.contains("bravo-only-commit") && !s.contains("Repository Management")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.wait_for("the Open in button", STEP, |s| s.contains("Open in"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    (term, config, [alpha, bravo])
}

#[test]
fn the_session_name_is_in_the_status_bar_only_and_open_in_follows_a_spaced_hyphen() {
    let (mut term, _config, _repos) = start_on_bravo();
    let text = term.text();
    let rows: Vec<(usize, &str)> = text
        .lines()
        .enumerate()
        .filter(|(_, l)| l.contains(SESSION))
        .collect();
    assert_eq!(rows.len(), 1, "the session name appears once:\n{text}");
    assert_eq!(text.matches(SESSION).count(), 1, "{text}");
    assert!(rows[0].0 > 5, "and not on the header row:\n{text}");
    assert!(
        rows[0].1.contains(&format!("session: {SESSION}")),
        "in the status bar: {:?}",
        rows[0].1
    );
    // Header: path, then " - ", then the menu.
    let title = text.lines().next().unwrap();
    assert!(
        title.contains("GitPoulpi") && title.contains(" -- "),
        "{title:?}"
    );
    assert!(!title.contains("session"), "{title:?}");
    let (col, row) = term.screen().find("Open in").expect("the button");
    assert_eq!(row, 0, "{title:?}");
    // Cells before the button's label: ... path, ' ', '-', ' ', then the
    // combobox field's own left padding.
    let cells: Vec<char> = title.chars().collect();
    let label = col as usize;
    let sep: String = cells[label.saturating_sub(5)..label].iter().collect();
    assert!(
        sep.contains(" - "),
        "space, hyphen, space before the button: {sep:?} in {title:?}"
    );
    // The click area moved with the paint: a click on the label opens the list.
    term.click(col + 1, row).unwrap();
    term.wait_for("the Open in list", STEP, |s| {
        s.contains("First entry") && s.contains("Configure")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}
