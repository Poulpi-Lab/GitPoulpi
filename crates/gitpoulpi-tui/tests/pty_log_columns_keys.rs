//! The Log's columns by keyboard, the twins of dragging a header and a
//! boundary: `Shift+C` picks a column, `Tab` / `Shift+Tab` change it, `<` /
//! `>` move it, `[` / `]` size it, `0` resets its width, `Esc` leaves. The
//! layout is the Log's own (`ui.log_columns_*`), apart from the commit
//! column's. Temp config dirs only.

mod common;

use std::path::Path;
use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::{Screen, Terminal};

const STEP: Duration = Duration::from_secs(30);

fn fixture() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit("keyboard column commit")
        .unwrap()
        .build()
}

fn launch(repo: &FixtureRepo, config: &Path) -> Terminal {
    common::launch(repo.root())
        .session("log-cols-keys")
        .config_dir(config)
        .size(200, 60)
        .ready_when("the fixture loaded", |s| {
            s.contains("keyboard column commit")
        })
        .spawn()
}

/// The x of a title on the Log header row (the row of `Message`).
fn header_x(screen: &Screen, title: &str) -> Option<u16> {
    let (_, row) = screen.find_all("Message").into_iter().next()?;
    screen
        .find_all(title)
        .into_iter()
        .find(|(_, r)| *r == row)
        .map(|(c, _)| c)
}

fn config_list(config: &Path, key: &str) -> Vec<String> {
    let text = std::fs::read_to_string(config.join("config.toml")).unwrap_or_default();
    let doc: toml::Table = toml::from_str(&text).unwrap_or_default();
    doc.get("ui")
        .and_then(|u| u.as_table())
        .and_then(|u| u.get(key))
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

fn config_width(config: &Path, column: &str) -> Option<i64> {
    let text = std::fs::read_to_string(config.join("config.toml")).unwrap_or_default();
    let doc: toml::Table = toml::from_str(&text).unwrap_or_default();
    doc.get("ui")?
        .as_table()?
        .get("log_column_widths")?
        .as_table()?
        .get(column)?
        .as_integer()
}

fn quit(mut term: Terminal) {
    term.send_keys("q").unwrap();
    for _ in 0..100 {
        if term.exited() {
            break;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    let _ = term.stop();
}

/// Waits until the saved Log order satisfies `ok` (the save is a background
/// write after the tick that sees the change).
fn wait_order(config: &Path, what: &str, ok: impl Fn(&[String]) -> bool) -> Vec<String> {
    for _ in 0..100 {
        let order = config_list(config, "log_columns_order");
        if ok(&order) {
            return order;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    panic!("{what}: {:?}", config_list(config, "log_columns_order"));
}

#[test]
fn keys_move_a_log_column_and_the_order_is_saved_apart_from_the_commit_column() {
    let config = tempfile::tempdir().unwrap();
    let repo = fixture();
    let mut term = launch(&repo, config.path());
    term.wait_for("the Log header", STEP, |s| header_x(s, "Ref").is_some())
        .unwrap_or_else(|e| panic!("{e}"));
    let before = header_x(&term.screen(), "Ref").unwrap();
    assert!(
        before < header_x(&term.screen(), "Graph").unwrap(),
        "Ref starts left of Graph"
    );

    // Pick the first column, move it right: Graph is now first.
    term.send_keys("C").unwrap();
    term.wait_for("Ref picked", STEP, |s| s.contains("Ref column picked"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys(">").unwrap();
    term.wait_for(
        "Ref right of Graph",
        STEP,
        |s| matches!((header_x(s, "Ref"), header_x(s, "Graph")), (Some(r), Some(g)) if g < r),
    )
    .unwrap_or_else(|e| panic!("{e}"));

    // `<` puts the pair back, `>` swaps it once more.
    term.send_keys("<").unwrap();
    term.wait_for(
        "Ref left of Graph again",
        STEP,
        |s| matches!((header_x(s, "Ref"), header_x(s, "Graph")), (Some(r), Some(g)) if r < g),
    )
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys(">").unwrap();
    term.wait_for(
        "Ref right of Graph",
        STEP,
        |s| matches!((header_x(s, "Ref"), header_x(s, "Graph")), (Some(r), Some(g)) if g < r),
    )
    .unwrap_or_else(|e| panic!("{e}"));

    // Ref is picked: Tab goes on to Message, Shift+Tab comes back.
    term.send_keys("Tab").unwrap();
    term.wait_for("Message picked", STEP, |s| {
        s.contains("Message column picked")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("S-Tab").unwrap();
    term.wait_for("Ref picked again", STEP, |s| {
        s.contains("Ref column picked")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Escape").unwrap();

    let order = wait_order(config.path(), "Graph before Ref saved", |o| {
        let at = |n: &str| o.iter().position(|c| c == n);
        matches!((at("graph"), at("ref")), (Some(g), Some(r)) if g < r)
    });
    assert!(!order.is_empty());
    // The commit column's own order is untouched.
    assert!(
        config_list(config.path(), "commit_column_columns_order").is_empty(),
        "{:?}",
        config_list(config.path(), "commit_column_columns_order")
    );
    quit(term);

    // A new process reads it back.
    let term = launch(&repo, config.path());
    term.wait_for("the Log header", STEP, |s| header_x(s, "Ref").is_some())
        .unwrap_or_else(|e| panic!("{e}"));
    let screen = term.screen();
    assert!(
        header_x(&screen, "Graph").unwrap() < header_x(&screen, "Ref").unwrap(),
        "{}",
        term.text()
    );
    quit(term);
}

#[test]
fn keys_size_a_log_column_and_zero_puts_it_back() {
    let config = tempfile::tempdir().unwrap();
    let repo = fixture();
    let mut term = launch(&repo, config.path());
    term.wait_for("the Log header", STEP, |s| header_x(s, "Author").is_some())
        .unwrap_or_else(|e| panic!("{e}"));
    let before = header_x(&term.screen(), "Author").unwrap();
    let date_before = header_x(&term.screen(), "Date").unwrap();

    // Pick the Author column: Tab four times from the first (Ref, Graph,
    // Message, Author).
    term.send_keys("C Tab Tab Tab").unwrap();
    term.wait_for("Author picked", STEP, |s| {
        s.contains("Author column picked")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    // Wider moves the boundary on its right: Date starts further right.
    term.send_keys("]").unwrap();
    term.wait_for("Date moved right", STEP, |s| {
        header_x(s, "Date").is_some_and(|x| x > date_before)
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(header_x(&term.screen(), "Author"), Some(before));
    for _ in 0..40 {
        if config_width(config.path(), "author").is_some() {
            break;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    assert!(
        config_width(config.path(), "author").is_some(),
        "the width is saved"
    );

    // `0` puts every width back to automatic.
    term.send_keys("0").unwrap();
    term.wait_for("Date back", STEP, |s| {
        header_x(s, "Date") == Some(date_before)
    })
    .unwrap_or_else(|e| panic!("{e}"));
    for _ in 0..40 {
        if config_width(config.path(), "author").is_none()
            && config_width(config.path(), "date").is_none()
        {
            break;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    assert_eq!(config_width(config.path(), "author"), None);
    assert_eq!(config_width(config.path(), "date"), None);
    quit(term);
}

#[test]
fn the_palette_moves_a_log_column_too() {
    let config = tempfile::tempdir().unwrap();
    let repo = fixture();
    let mut term = launch(&repo, config.path());
    term.wait_for("the Log header", STEP, |s| header_x(s, "Ref").is_some())
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("C-p").unwrap();
    term.send_text("Log Column: Move the Column Right").unwrap();
    term.wait_for("the command listed", STEP, |s| {
        s.contains("Move the Column Right")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    term.wait_for(
        "Ref right of Graph",
        STEP,
        |s| matches!((header_x(s, "Ref"), header_x(s, "Graph")), (Some(r), Some(g)) if g < r),
    )
    .unwrap_or_else(|e| panic!("{e}"));
    quit(term);
}
