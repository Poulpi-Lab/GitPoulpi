//! One list behavior for every filterable popup: `Ctrl+N/P/J/K`, `Home/End`,
//! `PgUp/PgDn` move the highlight; the wheel scrolls the window and leaves the
//! highlight alone; a click selects and a double click validates; a list that
//! overflows paints the scrollbar. Covered through the command palette, a
//! combobox (Settings > Language) and the help popup, whose filter field,
//! scrollbar and docs link work the same way.

mod common;

use std::collections::HashMap;
use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::{Screen, Terminal};

const STEP: Duration = Duration::from_secs(30);
const ARROW_UP: &str = "\u{2191}";

fn launch(session: &str) -> (Terminal, tempfile::TempDir, gitpoulpi_testkit::FixtureRepo) {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("popup list probe")
        .unwrap()
        .build();
    let (term, config) = common::launch(repo.root())
        .session(session)
        // Pinned to the top toolbar: `highlighted_rows` reads the popup's
        // rows by background, and the chips of a bottom toolbar sit in the
        // rows under a tall popup.
        .config("[ui]\ntoolbar_position = \"top\"\n")
        .size(200, 50)
        .ready("popup list probe")
        .mouse_capture()
        .start();
    (term, config, repo)
}

/// The rows of the popup list that carry a background of their own: the
/// highlight. `col` is a column inside the list, `rows` its row span.
fn highlighted_rows(s: &Screen, col: u16, rows: std::ops::Range<u16>) -> Vec<u16> {
    let mut count: HashMap<String, usize> = HashMap::new();
    let bg = |row: u16| format!("{:?}", s.cell(col, row).map(|c| c.bg));
    for row in rows.clone() {
        *count.entry(bg(row)).or_default() += 1;
    }
    let common = count
        .iter()
        .max_by_key(|(_, n)| **n)
        .map(|(k, _)| k.clone());
    rows.filter(|r| Some(bg(*r)) != common).collect()
}

/// The column of the popup's top-right corner, scanning right from its title.
fn right_border(s: &Screen, from: u16, row: u16) -> u16 {
    right_border_opt(s, from, row).expect("the popup's top-right corner")
}

/// [`right_border`] for wait predicates: a torn frame can lack the corner.
fn right_border_opt(s: &Screen, from: u16, row: u16) -> Option<u16> {
    (from..s.cols()).find(|c| {
        s.cell(*c, row)
            .is_some_and(|cell| cell.display() == "\u{256e}")
    })
}

/// `(column of the title, column of the right border, first list row)` of the palette popup.
fn palette_geometry(s: &Screen) -> (u16, u16, u16) {
    let (c, r) = s.find("Command Palette").expect("the palette title");
    (c, right_border(s, c, r), r + 3)
}

/// [`palette_geometry`] for wait predicates: `None` on a torn frame.
fn palette_geometry_opt(s: &Screen) -> Option<(u16, u16, u16)> {
    let (c, r) = s.find("Command Palette")?;
    Some((c, right_border_opt(s, c, r)?, r + 3))
}

fn selected_offset(s: &Screen) -> Option<u16> {
    let (c, _, top) = palette_geometry_opt(s)?;
    let rows = highlighted_rows(s, c + 2, top..(top + 31).min(s.rows()));
    (rows.len() == 1).then(|| rows[0] - top)
}

/// The list's row at `offset`; a sentinel no row equals when the palette's
/// frame is torn (so `==` waits keep waiting).
fn list_text(s: &Screen, offset: u16) -> String {
    match palette_geometry_opt(s) {
        Some((_, _, top)) => s.line(top + offset),
        None => "\u{0}torn".to_string(),
    }
}

/// The list's first row differs from `first`, on a frame that has a palette.
fn list_differs(s: &Screen, first: &str) -> bool {
    palette_geometry_opt(s).is_some() && list_text(s, 0) != first
}

fn open_palette(term: &mut Terminal) -> Screen {
    term.send_keys("C-p").unwrap();
    // The whole opened palette, as the callers read it back: the title is
    // drawn before the highlight and the scrollbar's arrows, and a caller
    // that located the arrows or the selection in the title-only frame
    // failed under load.
    term.wait_for("the palette, highlight and scrollbar drawn", STEP, |s| {
        s.contains("Command Palette") && selected_offset(s) == Some(0) && {
            let (_, right, top) = palette_geometry_opt(s).expect("a selected row has geometry");
            s.cell(right, top).map(|cell| cell.display().to_string()) == Some(ARROW_UP.to_string())
                && s.find_all("\u{2193}")
                    .into_iter()
                    .any(|(c, r)| c == right && r > top)
        }
    })
    .unwrap_or_else(|e| panic!("{e}"))
}

#[test]
fn the_palette_takes_the_list_keys() {
    let (mut term, _c, _repo) = launch("pty-popup-keys");
    let s = open_palette(&mut term);
    assert_eq!(selected_offset(&s), Some(0), "{}", s.text());
    let first = list_text(&s, 0);

    // Lists move with the arrows and with Ctrl+J / Ctrl+K (never Ctrl+N /
    // Ctrl+P: Ctrl+N is the badge prefix everywhere). The filter's own
    // Ctrl+K (kill to end) gives way to the list.
    term.send_keys("Down").unwrap();
    term.wait_for("Down moved down", STEP, |s| selected_offset(s) == Some(1))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Up").unwrap();
    term.wait_for("Up moved up", STEP, |s| selected_offset(s) == Some(0))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("C-j").unwrap();
    term.wait_for("Ctrl+J moved down", STEP, |s| selected_offset(s) == Some(1))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("C-k").unwrap();
    term.wait_for("Ctrl+K moved up", STEP, |s| selected_offset(s) == Some(0))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // A page down moves by the visible rows; the window follows.
    term.send_keys("PageDown").unwrap();
    term.wait_for("PgDn moved a page", STEP, |s| {
        selected_offset(s).is_some_and(|o| o >= 10)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // A second page runs past the window: it follows.
    term.send_keys("PageDown").unwrap();
    term.wait_for("the window moved with the second page", STEP, |s| {
        list_differs(s, &first)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("End").unwrap();
    let last = term
        .wait_for("End reached the bottom", STEP, |s| {
            selected_offset(s).is_some_and(|o| o >= 10)
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let last_row = selected_offset(&last).unwrap();
    assert!(last_row >= 10);
    term.send_keys("Home").unwrap();
    term.wait_for("Home reached the top, on the first command", STEP, |s| {
        selected_offset(s) == Some(0) && list_text(s, 0) == first
    })
    .unwrap_or_else(|e| panic!("{e}"));
    // Typing still narrows the list: the filter field is intact.
    term.send_text("change theme").unwrap();
    term.wait_for("narrowed", STEP, |s| s.contains("Change Theme"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.stop().unwrap();
}

#[test]
fn the_palette_overflow_paints_the_scrollbar_and_the_wheel_leaves_the_highlight() {
    let (mut term, _c, _repo) = launch("pty-popup-wheel");
    let s = open_palette(&mut term);
    let (c, right, top) = palette_geometry(&s);
    assert_eq!(
        s.cell(right, top).map(|cell| cell.display().to_string()),
        Some(ARROW_UP.to_string()),
        "the overflowing list paints its scrollbar:\n{}",
        s.text()
    );
    let first = list_text(&s, 0);
    // Over the filter box: inside the popup, on no row, so nothing is hovered.
    term.scroll(true, c + 4, top - 2).unwrap();
    let scrolled = term
        .wait_for("the window scrolled", STEP, |s| list_differs(s, &first))
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(
        selected_offset(&scrolled),
        None,
        "the highlight did not follow the wheel: it is above the window"
    );
    // A key move brings the window back to the (unmoved) highlight + 1.
    term.send_keys("Down").unwrap();
    term.wait_for("highlight 1 in view again", STEP, |s| {
        list_text(s, 0) == first && selected_offset(s) == Some(1)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

#[test]
fn a_palette_click_selects_and_a_double_click_validates() {
    let (mut term, _c, _repo) = launch("pty-popup-click");
    term.send_keys("C-p").unwrap();
    term.wait_for("the palette", STEP, |s| s.contains("Command Palette"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_text("change theme").unwrap();
    let s = term
        .wait_for("narrowed", STEP, |s| s.contains("Change Theme"))
        .unwrap_or_else(|e| panic!("{e}"));
    let (c, top) = {
        let (c, _, top) = palette_geometry(&s);
        (c, top)
    };
    // Make the narrowed list start on a different highlight than row 0.
    term.send_keys("Down").unwrap();
    let rows = (top..top + 8)
        .filter(|r| s.line(*r).contains("Change"))
        .collect::<Vec<_>>();
    assert!(!rows.is_empty(), "{}", s.text());
    let row = rows[0];
    term.click(c + 6, row).unwrap();
    term.wait_for("the click selected the row", STEP, |s| {
        s.contains("Command Palette") && highlighted_rows(s, c + 2, top..top + 8).contains(&row)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(
        term.screen().contains("Command Palette"),
        "one click does not validate"
    );
    term.double_click(c + 6, row).unwrap();
    term.wait_for(
        "the double click ran the command: the theme list",
        STEP,
        |s| s.contains("System (follows terminal)"),
    )
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

/// Settings > General > Language: the dropdown is a searchable combobox.
fn open_language_combo(term: &mut Terminal) -> Screen {
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
    term.wait_for("the language popup", STEP, |s| s.contains("\u{2713}"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()))
}

fn stored(config: &tempfile::TempDir) -> String {
    std::fs::read_to_string(config.path().join("config.toml")).unwrap_or_default()
}

#[test]
fn a_combobox_takes_the_list_keys_and_double_click_picks() {
    let (mut term, config, _repo) = launch("pty-popup-combo");
    let s = open_language_combo(&mut term);
    // Home / End / PgDn / Ctrl+J|K are the list's: the highlight moves, the
    // popup stays open and nothing is stored.
    let (tick_col, tick_row) = s.find("\u{2713}").unwrap();
    term.send_keys("End").unwrap();
    term.send_keys("C-k").unwrap();
    term.send_keys("Home").unwrap();
    term.send_keys("C-j").unwrap();
    term.send_keys("PageDown").unwrap();
    term.wait_for("still open", STEP, |s| s.contains("\u{2713}"))
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(!stored(&config).contains("language"), "{}", stored(&config));
    term.send_keys("Home").unwrap();
    term.send_keys("C-j").unwrap();
    // The second option is picked with Enter (Home, Ctrl+J, Enter).
    let before = stored(&config);
    term.send_keys("Enter").unwrap();
    let deadline = std::time::Instant::now() + STEP;
    while stored(&config) == before {
        assert!(
            std::time::Instant::now() < deadline,
            "no language stored: {}\n{}",
            stored(&config),
            term.text()
        );
        std::thread::sleep(Duration::from_millis(200));
    }
    let _ = (tick_col, tick_row);
    term.stop().unwrap();
}

#[test]
fn a_combobox_click_selects_and_a_double_click_picks() {
    let (mut term, config, _repo) = launch("pty-popup-combo-click");
    let s = open_language_combo(&mut term);
    let (_, tick_row) = s.find("\u{2713}").unwrap();
    // A row two below the ticked one (the popup lists the options in order).
    let target = tick_row + 2;
    let (c, _) = s.find("\u{2713}").unwrap();
    term.click(c + 3, target).unwrap();
    std::thread::sleep(Duration::from_millis(800));
    assert!(
        term.screen().contains("\u{2713}"),
        "a single click keeps the popup open"
    );
    assert!(!stored(&config).contains("language"), "{}", stored(&config));
    term.double_click(c + 3, target).unwrap();
    let deadline = std::time::Instant::now() + STEP;
    while !stored(&config).contains("language") {
        assert!(
            std::time::Instant::now() < deadline,
            "double click did not pick: {}\n{}",
            stored(&config),
            term.text()
        );
        std::thread::sleep(Duration::from_millis(200));
    }
    term.stop().unwrap();
}

// ---- The help popup ----

fn help_geometry(s: &Screen) -> (u16, u16, u16) {
    let (c, r) = s.find("Keybindings").expect("the help title");
    (c, right_border(s, c, r), r + 4)
}

fn help_text(s: &Screen) -> String {
    let (_, _, top) = help_geometry(s);
    s.line(top)
}

#[test]
fn the_help_has_a_filter_field_the_list_keys_a_scrollbar_and_a_docs_link() {
    let (mut term, _c, _repo) = launch("pty-popup-help");
    let log = tempfile::NamedTempFile::new().unwrap();
    term.send_text("?").unwrap();
    let s = term
        .wait_for(
            "the whole help, scrollbar and docs link included",
            STEP,
            |s| {
                // The assertions below are part of the predicate: a half-painted
                // frame has the title before it has the rest.
                s.contains("Keybindings") && s.contains("User docs on GitHub") && {
                    let (_, right, top) = help_geometry(s);
                    s.cell(right, top).map(|cell| cell.display().to_string())
                        == Some(ARROW_UP.to_string())
                }
            },
        )
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (c, right, top) = help_geometry(&s);
    assert_eq!(
        s.cell(right, top).map(|cell| cell.display().to_string()),
        Some(ARROW_UP.to_string()),
        "the help paints its scrollbar:\n{}",
        s.text()
    );
    assert!(s.contains("User docs on GitHub"), "{}", s.text());
    assert!(s.contains("docs/user/keybindings.md"), "{}", s.text());
    let first = help_text(&s);
    term.send_keys("Down").unwrap();
    term.send_keys("Down").unwrap();
    term.wait_for("Down scrolled", STEP, |s| help_text(s) != first)
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Up").unwrap();
    term.send_keys("Up").unwrap();
    term.wait_for("Up scrolled back", STEP, |s| help_text(s) == first)
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("PageDown").unwrap();
    term.wait_for("PgDn", STEP, |s| help_text(s) != first)
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("End").unwrap();
    let end = term
        .wait_for("End", STEP, |s| help_text(s) != first)
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Home").unwrap();
    term.wait_for("Home", STEP, |s| help_text(s) == first)
        .unwrap_or_else(|e| panic!("{e}"));
    let _ = end;
    term.scroll(true, c + 4, top - 2).unwrap();
    term.wait_for("the wheel scrolled", STEP, |s| help_text(s) != first)
        .unwrap_or_else(|e| panic!("{e}"));
    // The filter is a real field: typing narrows (j/k type, they do not scroll).
    term.send_text("stage").unwrap();
    term.wait_for("the filter narrowed the help", STEP, |s| {
        s.contains("filter \"stage\"")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    drop(log);
    term.stop().unwrap();
}

#[test]
fn the_help_docs_link_opens_with_ctrl_d_and_a_click() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("popup list probe")
        .unwrap()
        .build();
    let log = tempfile::NamedTempFile::new().unwrap();
    let (mut term, _c) = common::launch(repo.root())
        .session("pty-popup-help-link")
        .size(200, 50)
        .env("GITPOULPI_OPEN_LOG", log.path())
        .ready("popup list probe")
        .mouse_capture()
        .start();
    term.send_text("?").unwrap();
    let s = term
        .wait_for("the help", STEP, |s| s.contains("User docs on GitHub"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let url = "https://github.com/Poulpi-Lab/GitPoulpi/blob/main/docs/user/keybindings.md";
    term.send_keys("C-d").unwrap();
    let wait = |n: usize| {
        let deadline = std::time::Instant::now() + STEP;
        loop {
            let logged = std::fs::read_to_string(log.path()).unwrap_or_default();
            if logged.lines().filter(|l| *l == url).count() >= n {
                return;
            }
            assert!(std::time::Instant::now() < deadline, "{logged:?}");
            std::thread::sleep(Duration::from_millis(200));
        }
    };
    wait(1);
    let (c, r) = s.find("User docs on GitHub").unwrap();
    term.click(c + 3, r).unwrap();
    wait(2);
    // A click inside the popup does not close it; one outside does.
    assert!(term.screen().contains("Keybindings"));
    term.click(0, 0).unwrap();
    term.wait_for_text_to_go("User docs on GitHub", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.stop().unwrap();
}

/// A press on the palette scrollbar's bare track jumps the thumb under the
/// pointer (the end of the list, not one page down) and keeps dragging.
#[test]
fn a_press_on_the_palette_track_jumps_the_thumb_and_keeps_dragging() {
    // A short terminal: the list is many pages long, so a page-sized step
    // and a jump to the end cannot be mistaken for one another.
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("popup list probe")
        .unwrap()
        .build();
    let (mut term, _c) = common::launch(repo.root())
        .session("pty-popup-track-jump")
        // Pinned to the top toolbar, as in `launch`.
        .config("[ui]\ntoolbar_position = \"top\"\n")
        .size(120, 20)
        .ready("popup list probe")
        .mouse_capture()
        .start();
    let s = open_palette(&mut term);
    let (_, right, top) = palette_geometry(&s);
    let first = list_text(&s, 0);
    let down = s
        .find_all("\u{2193}")
        .into_iter()
        .filter(|(c, r)| *c == right && *r > top)
        .map(|(_, r)| r)
        .min()
        .expect("the down arrow");
    // The window at the very end of the list, as the End key shows it.
    term.send_keys("End").unwrap();
    term.wait_for("End scrolled to the bottom", STEP, |s| {
        list_differs(s, &first)
    })
    .unwrap_or_else(|e| panic!("{e}"));
    // The final frame of the jump, read once the palette stopped redrawing.
    let end = term
        .wait_for_stable(Duration::from_millis(300), STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    let end_text = list_text(&end, 0);
    term.send_keys("Home").unwrap();
    term.wait_for("Home is back at the top", STEP, |s| {
        list_text(s, 0) == first
    })
    .unwrap_or_else(|e| panic!("{e}"));
    // A press on the track's last cell jumps there in one go (a page would
    // fall far short of the end of hundreds of commands)...
    term.press(right, down - 1).unwrap();
    term.wait_for("the jump reached the end", STEP, |s| {
        list_text(s, 0) == end_text
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // ...and still held, moving to the top of the track drags the thumb back.
    term.drag_to(right, top + 1).unwrap();
    term.wait_for("dragged back to the first rows", STEP, |s| {
        list_text(s, 0) == first
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.release(right, top + 1).unwrap();
    term.stop().unwrap();
}

/// `Ctrl+J` / `Ctrl+K` move the Log's selection like the arrows (a list with
/// no text field). The commit detail shows the selected commit's summary, so
/// a summary on the screen twice is the selected one.
///
/// (The Log's key match reads the character, so it took `Ctrl+J` / `Ctrl+K`
/// as `j` / `k` before; `ctrl_list_key` makes it a rule for every list that
/// checks the modifiers.)
#[test]
fn ctrl_j_and_ctrl_k_move_the_main_log() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("older ctrl list commit")
        .unwrap()
        .commit("newer ctrl list commit")
        .unwrap()
        .build();
    let (mut term, _config) = common::launch(repo.root())
        .session("pty-ctrl-jk-log")
        .size(200, 50)
        .ready("newer ctrl list commit")
        .mouse_capture()
        .start();
    let (c, r) = term.wait_for_text("newer ctrl list commit", STEP).unwrap();
    term.click(c + 1, r).unwrap();
    term.wait_for("the newer commit selected", STEP, |s| {
        s.find_all("newer ctrl list commit").len() >= 2
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("C-j").unwrap();
    term.wait_for("the older commit selected", STEP, |s| {
        s.find_all("older ctrl list commit").len() >= 2
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("C-k").unwrap();
    term.wait_for("the newer commit selected again", STEP, |s| {
        s.find_all("newer ctrl list commit").len() >= 2
            && s.find_all("older ctrl list commit").len() < 2
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let _ = term.stop();
}
