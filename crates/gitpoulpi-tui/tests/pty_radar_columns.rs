#![cfg(feature = "radar")]
//! The Radar's sizable columns, in the real binary against the hosting mock:
//! the repository column is cut with an ellipsis when it is narrow, a drag on
//! its separator or the keys (`C`, `[`, `]`, `0`) size it, the width is kept
//! in `radar.toml` and survives a restart, and hovering a cut name shows it
//! in full. Two repositories, so the chips carry their parent folder and are
//! long. Temp config dirs only.

#![allow(dead_code)]

#[path = "common/mod.rs"]
mod common;
#[path = "pty_hosting/mock.rs"]
mod mock;
#[path = "pty_hosting/support.rs"]
mod support;

use std::path::Path;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use mock::start_mock;
use support::{config_toml, COLS, ROWS, STARTUP, STEP};
use xtask::pty::{Screen, Terminal};

/// The width the tests start the repository column at: narrower than any
/// chip (`repo (<parent folder>)`).
const NARROW: u16 = 8;

fn link(repo: &FixtureRepo, project: &str) {
    let dir = repo.state_dir();
    std::fs::create_dir_all(&dir).expect("state dir");
    std::fs::write(
        dir.join("repo-state.toml"),
        format!(
            "[pulls_source]\nintegration = \"corp\"\nproject = \"{project}\"\n\n[issues_source]\nintegration = \"corp\"\nproject = \"{project}\"\n"
        ),
    )
    .expect("seed repo-state.toml");
}

fn name_of(repo: &FixtureRepo) -> String {
    format!(
        "repo ({})",
        repo.root()
            .parent()
            .unwrap()
            .file_name()
            .unwrap()
            .to_string_lossy()
    )
}

struct Setup {
    term: Terminal,
    config: tempfile::TempDir,
    a: FixtureRepo,
    session: String,
    b: FixtureRepo,
    _mock: mock::Mock,
}

fn launch(session: &str, config: &Path, a: &FixtureRepo) -> Terminal {
    let term = common::launch(a.root())
        .session(session)
        .config_dir(config)
        .size(COLS, ROWS)
        .env("GITPOULPI_NO_BROWSER", "1")
        .env("GITPOULPI_TOKEN_CORP", "test-token")
        .env("GITPOULPI_DESKTOP_NO_HOOKS", "1")
        .spawn();
    term.wait_for("the first repository loaded", STARTUP, |s| {
        s.contains("Add the feature")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term
}

/// Two repositories as tabs; `radar_toml` is seeded as `radar.toml`.
fn setup(session: &str, radar_toml: &str) -> Setup {
    setup_in(session, radar_toml, false)
}

/// [`setup`], with `ui.column_resize = "header"` when `header` is set.
fn setup_in(session: &str, radar_toml: &str, header: bool) -> Setup {
    let mock = start_mock();
    let a = support::fixture(mock.port);
    link(&a, "g/p");
    let b = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("t.txt", "t\n", "second-repo-start")
        .unwrap()
        .build();
    b.git(&[
        "remote",
        "add",
        "origin",
        &format!("http://127.0.0.1:{}/team/tool.git", mock.port),
    ]);
    link(&b, "team/tool");
    let config = tempfile::tempdir().expect("temp config dir");
    let mut config_text = config_toml(mock.port, "");
    if header {
        config_text = config_text.replace("[ui]\n", "[ui]\ncolumn_resize = \"header\"\n");
    }
    std::fs::write(config.path().join("config.toml"), config_text).unwrap();
    if !radar_toml.is_empty() {
        std::fs::write(config.path().join("radar.toml"), radar_toml).unwrap();
    }
    std::fs::create_dir_all(config.path().join("sessions")).unwrap();
    std::fs::write(
        config.path().join("sessions").join(format!("{session}.toml")),
        format!(
            "[[tabs]]\nrepo_path = '{}'\nactive = true\n\n[[tabs]]\nrepo_path = '{}'\nactive = false\n",
            a.root().display(),
            b.root().display()
        ),
    )
    .unwrap();
    let term = launch(session, config.path(), &a);
    Setup {
        term,
        config,
        a,
        session: session.to_string(),
        b,
        _mock: mock,
    }
}

fn open_radar(term: &mut Terminal) -> Screen {
    term.send_keys("M-r").expect("Alt+R");
    term.wait_for("the Radar rows of both repositories", STARTUP, |s| {
        s.contains("MY PULL REQUESTS  2") && s.find_all("Add the parser").len() >= 2
    })
    .unwrap_or_else(|e| panic!("{e}"))
}

/// `(column of the chip, row)` of the first request row.
fn chip_cell(screen: &Screen) -> (u16, u16) {
    let (_, row) = screen
        .find_all("Add the parser")
        .into_iter()
        .find(|&(_, r)| screen.line(r).contains('\u{21C4}'))
        .unwrap_or_else(|| panic!("no request row:\n{}", screen.text()));
    let line = screen.line(row);
    let at = line.find("repo").expect("a chip");
    (line[..at].chars().count() as u16, row)
}

/// The screen once it stops changing (rows of other groups may still be
/// arriving above the one a test aims at).
fn settled(term: &Terminal) -> Screen {
    let mut last = term.screen();
    for _ in 0..50 {
        std::thread::sleep(std::time::Duration::from_millis(300));
        let now = term.screen();
        if now.text() == last.text() {
            return now;
        }
        last = now;
    }
    last
}

fn radar_file(config: &Path) -> String {
    std::fs::read_to_string(config.join("radar.toml")).unwrap_or_default()
}

fn wait_file(config: &Path, what: &str, ok: impl Fn(&str) -> bool) {
    for _ in 0..100 {
        if ok(&radar_file(config)) {
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    panic!("{what}: {}", radar_file(config));
}

/// The rows of the list that show a repository chip in full.
fn rows_with(screen: &Screen, name: &str) -> usize {
    screen
        .text()
        .lines()
        .filter(|l| l.contains('\u{21C4}') && l.contains(name))
        .count()
}

fn quit(mut term: Terminal) {
    let _ = term.send_keys("q");
    for _ in 0..100 {
        if term.exited() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    let _ = term.stop();
}

/// A narrow repository column cuts the name with an ellipsis; dragging its
/// separator wider shows the whole name, the width is saved, and it is still
/// that wide after a restart.
#[test]
fn dragging_the_separator_widens_the_repository_column_and_it_survives_a_restart() {
    let mut s = setup(
        "pty-radar-cols-drag",
        &format!("[columns]\nrepo = {NARROW}\n"),
    );
    let name = name_of(&s.a);
    let screen = open_radar(&mut s.term);
    assert_eq!(rows_with(&screen, &name), 0, "{}", screen.text());
    assert!(
        screen
            .text()
            .lines()
            .any(|l| l.contains('\u{21C4}') && l.contains('\u{2026}')),
        "an ellipsis marks the cut name:\n{}",
        screen.text()
    );

    // The separator is the cell right after the chip's width.
    let (x, row) = chip_cell(&screen);
    s.term
        .drag((x + NARROW, row), (x + NARROW + 14, row))
        .expect("drag the separator");
    s.term
        .wait_for("the whole name", STEP, |s| rows_with(s, &name) >= 1)
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
    wait_file(s.config.path(), "the saved width", |f| {
        f.contains("repo = 22")
    });

    // A restart keeps it.
    let Setup {
        term,
        config,
        a,
        session,
        b: _b,
        _mock,
    } = s;
    quit(term);
    let mut term = launch(&session, config.path(), &a);
    let screen = open_radar(&mut term);
    assert!(rows_with(&screen, &name) >= 1, "{}", screen.text());
    quit(term);
}

/// The keyboard twin: `C` picks the column, `]` widens, `[` narrows, `0`
/// puts the automatic width back (and forgets the saved one).
#[test]
fn the_keys_size_the_repository_column() {
    let mut s = setup(
        "pty-radar-cols-keys",
        &format!("[columns]\nrepo = {NARROW}\n"),
    );
    let name = name_of(&s.a);
    let screen = open_radar(&mut s.term);
    assert_eq!(rows_with(&screen, &name), 0, "{}", screen.text());

    s.term.send_text("C").unwrap();
    s.term
        .wait_for("the column picked", STEP, |s| {
            s.contains("Repository column (8 cells)")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
    for _ in 0..7 {
        s.term.send_text("]").unwrap();
    }
    s.term
        .wait_for("the whole name", STEP, |s| rows_with(s, &name) >= 1)
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
    wait_file(s.config.path(), "the saved width", |f| {
        f.contains("repo = 22")
    });

    s.term.send_text("[").unwrap();
    wait_file(s.config.path(), "narrowed by one step", |f| {
        f.contains("repo = 20")
    });

    s.term.send_text("0").unwrap();
    wait_file(s.config.path(), "automatic again", |f| {
        !f.contains("repo =")
    });
    s.term
        .wait_for("the whole name at the automatic width", STEP, |s| {
            rows_with(s, &name) >= 1
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
    quit(s.term);
}

/// The `#` column is sizable too: a drag on its separator widens it, the
/// title moves right with it, the width lands in `radar.toml` and a restart
/// keeps it. Red when the `#` separator grabs nothing or the width is not kept.
#[test]
fn dragging_the_number_separator_sizes_the_number_column_and_it_survives_a_restart() {
    let mut s = setup(
        "pty-radar-cols-number",
        &format!("[columns]\nrepo = {NARROW}\nnumber = 8\n"),
    );
    let screen = open_radar(&mut s.term);
    drop(screen);
    let screen = settled(&s.term);
    let header = header_row(&screen);
    let title_col = |s: &Screen| -> usize {
        let line = s.line(header);
        line[..line.find("Title").unwrap()].chars().count()
    };
    let line = screen.line(header);
    let hash = line[..line.find('#').expect("a # title")].chars().count() as u16;
    let gap = hash + 8;
    let (_, row) = chip_cell(&screen);
    assert_eq!(
        title_col(&screen),
        usize::from(gap) + 1,
        "{}",
        screen.text()
    );
    s.term
        .drag((gap, row), (gap + 4, row))
        .expect("drag the # separator");
    wait_file(s.config.path(), "the saved number width", |f| {
        f.contains("number = 12")
    });
    s.term
        .wait_for("the title moved right", STEP, |s| {
            title_col(s) == usize::from(gap) + 5
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));

    let Setup {
        term,
        config,
        a,
        session,
        b: _b,
        _mock,
    } = s;
    quit(term);
    let mut term = launch(&session, config.path(), &a);
    open_radar(&mut term);
    let screen = settled(&term);
    assert_eq!(
        title_col(&screen),
        usize::from(gap) + 5,
        "{}",
        screen.text()
    );
    quit(term);
}

/// Hovering a cut name shows it in full; selecting the row says it on the
/// status line (the keyboard's twin).
#[test]
fn a_cut_name_shows_in_full_in_a_tooltip_and_on_the_status_line() {
    let mut s = setup(
        "pty-radar-cols-tip",
        &format!("[columns]\nrepo = {NARROW}\n"),
    );
    // Either repository's row may come first.
    let names = [name_of(&s.a), name_of(&s.b)];
    let shows = |s: &Screen| names.iter().any(|n| s.contains(n));
    open_radar(&mut s.term);
    let screen = settled(&s.term);
    assert!(!shows(&screen), "{}", screen.text());
    let (x, row) = chip_cell(&screen);

    s.term.move_to(x + 2, row).unwrap();
    s.term
        .wait_for("the tooltip", STEP, |s| shows(s))
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));

    // Away from the chip the tooltip is gone; selecting the row (a click on
    // its title) puts the full name on the status line.
    let title = screen.line(row).find("Add the parser").unwrap();
    let title_x = screen.line(row)[..title].chars().count() as u16;
    s.term.click(title_x + 1, row).unwrap();
    s.term.move_to(title_x + 1, row + 3).unwrap();
    s.term
        .wait_for("the full name on the status line", STEP, |s| {
            s.text()
                .lines()
                .any(|l| l.contains("repositories") && names.iter().any(|n| l.contains(n)))
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
    quit(s.term);
}

/// The header row: the row of the "Title" column title.
fn header_row(screen: &Screen) -> u16 {
    screen
        .find_all("Title")
        .into_iter()
        .map(|(_, r)| r)
        .min()
        .unwrap_or_else(|| panic!("no header row:\n{}", screen.text()))
}

fn text_at(screen: &Screen, col: u16, row: u16) -> String {
    screen
        .cell(col, row)
        .map(|c| c.text.clone())
        .unwrap_or_default()
}

/// `line` mode (the default): the list has a header row, a tick rests in the
/// separator's header cell, the pointer on the separator anywhere down the
/// list draws a thin line from the header to the last row, and leaving it
/// takes the line away. Red when the line is a block, is only on one row, or
/// stays after the pointer left.
#[test]
fn line_mode_has_a_header_row_and_a_thin_line_down_the_hovered_separator() {
    let mut s = setup(
        "pty-radar-cols-line",
        &format!("[columns]\nrepo = {NARROW}\n"),
    );
    open_radar(&mut s.term);
    let screen = settled(&s.term);
    let header = header_row(&screen);
    for title in ["Title", "Author", "Age"] {
        assert!(
            screen.find_all(title).into_iter().any(|(_, r)| r == header),
            "{title} in the header:\n{}",
            screen.text()
        );
    }
    let (x, row) = chip_cell(&screen);
    assert!(row > header, "the rows are under the header");
    let gap = x + NARROW;
    assert_eq!(text_at(&screen, gap, header), "\u{2502}", "the tick");
    assert_ne!(text_at(&screen, gap, row), "\u{2502}", "nothing at rest");

    s.term.move_to(gap, row).unwrap();
    s.term
        .wait_for("the thin line", STEP, |s| {
            (header..=row).all(|y| text_at(s, gap, y) == "\u{2502}")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
    s.term.move_to(gap + 6, row).unwrap();
    s.term
        .wait_for("the line is gone", STEP, |s| {
            text_at(s, gap, row) != "\u{2502}" && text_at(s, gap, header) == "\u{2502}"
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
    quit(s.term);
}

/// `header` mode: no tick; the first cell after the separator, on the header
/// row, is the only grab (a press on the separator below does nothing), and a
/// drag from it sizes the column and is saved.
#[test]
fn header_mode_grabs_only_the_header_cell_after_the_separator() {
    let mut s = setup_in(
        "pty-radar-cols-header",
        &format!("[columns]\nrepo = {NARROW}\n"),
        true,
    );
    open_radar(&mut s.term);
    let screen = settled(&s.term);
    let header = header_row(&screen);
    let (x, row) = chip_cell(&screen);
    let gap = x + NARROW;
    assert!(
        (1..screen.cols() - 1).all(|c| text_at(&screen, c, header) != "\u{2502}"),
        "no tick in the header mode:\n{}",
        screen.text()
    );
    // A press in the body on the separator does not start a drag.
    s.term.drag((gap, row), (gap + 9, row)).unwrap();
    std::thread::sleep(std::time::Duration::from_millis(500));
    assert!(
        !radar_file(s.config.path()).contains("repo = 17"),
        "{}",
        radar_file(s.config.path())
    );
    // The header cell after it does.
    s.term
        .drag((gap + 1, header), (gap + 1 + 14, header))
        .expect("drag the header cell");
    wait_file(s.config.path(), "the saved width", |f| {
        f.contains("repo = 22")
    });
    quit(s.term);
}

/// The x of the first `title` on the header row.
fn header_x(screen: &Screen, title: &str) -> Option<u16> {
    let header = screen.find_all("Title").into_iter().map(|(_, r)| r).min()?;
    screen
        .find_all(title)
        .into_iter()
        .find(|(_, r)| *r == header)
        .map(|(c, _)| c)
}

/// After a drag of the author separator the thin line goes with the button
/// when it is released elsewhere (the column stays unpicked), and shows as a
/// hover when released on the separator, until the pointer moves away. Red
/// when the line stays drawn after the release.
#[test]
fn the_line_does_not_stay_after_a_separator_drag_is_released() {
    let mut s = setup(
        "pty-radar-cols-release",
        &format!("[columns]\nrepo = {NARROW}\n"),
    );
    open_radar(&mut s.term);
    let screen = settled(&s.term);
    let header = header_row(&screen);
    let (_, row) = chip_cell(&screen);
    let author = header_x(&screen, "Author").expect("an Author title");
    let gap = author - 1;
    s.term.press(gap, row).unwrap();
    s.term.drag_to(gap - 2, row).unwrap();
    s.term.drag_to(gap - 3, row).unwrap();
    s.term
        .wait_for("the author is wider", STEP, |s| {
            header_x(s, "Author") == Some(author - 3)
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
    let new_gap = gap - 3;
    // Released away: no line, the tick is back, nothing stays picked.
    s.term.release(new_gap - 10, row).unwrap();
    s.term
        .wait_for("no line, the tick is back", STEP, |s| {
            text_at(s, new_gap, row) != "\u{2502}" && text_at(s, new_gap, header) == "\u{2502}"
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
    std::thread::sleep(std::time::Duration::from_millis(400));
    assert_ne!(
        text_at(&s.term.screen(), new_gap, row),
        "\u{2502}",
        "{}",
        s.term.text()
    );
    // Released on it: a hover, gone with the next move away.
    s.term.press(new_gap, row).unwrap();
    s.term.release(new_gap, row).unwrap();
    s.term
        .wait_for("the hover line", STEP, |s| {
            (header..=row).all(|y| text_at(s, new_gap, y) == "\u{2502}")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
    s.term.move_to(new_gap - 10, row).unwrap();
    s.term
        .wait_for("the line is gone", STEP, |s| {
            text_at(s, new_gap, row) != "\u{2502}"
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
    quit(s.term);
}

/// The line lit by a pointer resting on a separator goes when the pointer
/// leaves the Radar body altogether (up onto the tab bar, down onto the status
/// bar): the Radar only hears moves inside its body, so without an explicit
/// "the pointer left" the last hover stayed. Red when the line stays drawn.
#[test]
fn the_line_goes_when_the_pointer_leaves_the_radar_body() {
    let mut s = setup(
        "pty-radar-cols-leave",
        &format!("[columns]\nrepo = {NARROW}\n"),
    );
    open_radar(&mut s.term);
    let screen = settled(&s.term);
    let header = header_row(&screen);
    let (_, row) = chip_cell(&screen);
    let author = header_x(&screen, "Author").expect("an Author title");
    let gap = author - 1;
    s.term.press(gap, row).unwrap();
    s.term.drag_to(gap - 2, row).unwrap();
    s.term.drag_to(gap - 3, row).unwrap();
    s.term
        .wait_for("the author is wider", STEP, |s| {
            header_x(s, "Author") == Some(author - 3)
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
    let new_gap = gap - 3;
    // Released on it, as a real drag ends: the pointer rests on the line.
    s.term.release(new_gap, row).unwrap();
    s.term
        .wait_for("the hover line", STEP, |s| {
            (header..=row).all(|y| text_at(s, new_gap, y) == "\u{2502}")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
    // Away, out of the body (the tab bar).
    s.term.move_to(new_gap, 0).unwrap();
    s.term
        .wait_for("the line is gone", STEP, |s| {
            text_at(s, new_gap, row) != "\u{2502}"
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
    quit(s.term);
}

/// The age column has its own separator: a drag sizes it, the width lands in
/// `radar.toml` and a restart keeps it. Red when the separator grabs nothing
/// or the width is not kept.
#[test]
fn dragging_the_age_separator_sizes_the_age_column_and_it_survives_a_restart() {
    let mut s = setup(
        "pty-radar-cols-age",
        &format!("[columns]\nrepo = {NARROW}\n"),
    );
    open_radar(&mut s.term);
    let screen = settled(&s.term);
    let (_, row) = chip_cell(&screen);
    let age = header_x(&screen, "Age").expect("an Age title");
    let gap = age - 1;
    s.term
        .drag((gap, row), (gap - 3, row))
        .expect("drag the age separator");
    wait_file(s.config.path(), "the saved age width", |f| {
        f.contains("age = ")
    });
    s.term
        .wait_for("the age title moved left", STEP, |s| {
            header_x(s, "Age") == Some(age - 3)
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
    let Setup {
        term,
        config,
        a,
        session,
        b: _b,
        _mock,
    } = s;
    quit(term);
    let mut term = launch(&session, config.path(), &a);
    open_radar(&mut term);
    let screen = settled(&term);
    assert_eq!(header_x(&screen, "Age"), Some(age - 3), "{}", screen.text());
    quit(term);
}

/// The least width is one cell: dragging the separator far left leaves the
/// repository column one cell wide (an ellipsis), the separator sits right
/// after it and the title column moves left by exactly the cells given up;
/// the width is saved as 1 and the 1-cell separator can be grabbed again.
/// Red when the minimum is above one cell.
#[test]
fn dragging_the_separator_down_leaves_a_one_cell_repository_column() {
    let mut s = setup(
        "pty-radar-cols-one",
        &format!("[columns]\nrepo = {NARROW}\n"),
    );
    open_radar(&mut s.term);
    let screen = settled(&s.term);
    let header = header_row(&screen);
    let title_col = |s: &Screen| -> usize {
        let line = s.line(header);
        line[..line.find("Title").unwrap()].chars().count()
    };
    let before = title_col(&screen);
    let (x, row) = chip_cell(&screen);
    // Far past the least width: the pointer ends on the chip's first cell.
    s.term
        .drag((x + NARROW, row), (x, row))
        .expect("drag the separator");
    wait_file(s.config.path(), "the saved one-cell width", |f| {
        f.contains("repo = 1\n") || f.ends_with("repo = 1")
    });
    let screen = s
        .term
        .wait_for("the title moved left", STEP, |s| {
            s.find_all("Title").iter().any(|(_, r)| *r == header)
                && title_col(s) == before - (usize::from(NARROW) - 1)
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
    assert_eq!(text_at(&screen, x, row), "\u{2026}", "{}", screen.text());
    assert_eq!(text_at(&screen, x + 1, header), "\u{2502}", "the tick");
    assert_ne!(text_at(&screen, x + 1, row), "\u{2502}", "nothing at rest");

    // The one-cell column's separator is still grabbable: drag it wider.
    s.term
        .drag((x + 1, row), (x + 5, row))
        .expect("drag the separator again");
    wait_file(s.config.path(), "the widened column", |f| {
        f.contains("repo = 5")
    });
    quit(s.term);
}
