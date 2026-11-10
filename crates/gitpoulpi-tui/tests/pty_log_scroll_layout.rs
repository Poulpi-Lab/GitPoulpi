//! The Log's `scroll` column layout: columns keep their widths (Message too),
//! the table can be wider than the pane and scrolls sideways - by
//! `Shift+Left/Right`, the horizontal wheel or Shift+wheel, and a bar on the
//! bottom border that drags and takes track presses. The layout is chosen in
//! the column chooser, Settings or the palette (`ui.log_columns_layout`).
//! Temp config dirs only.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::{Screen, Terminal};

const STEP: Duration = Duration::from_secs(30);
const EMAIL: &str = "testkit@gitpoulpi.local";
const LONG: &str = "a very long summary that keeps going on and on and on and on and on and on and on and on and on and on and on and on and on and on and on until the end";

fn fixture() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit("older commit")
        .unwrap()
        .commit(LONG)
        .unwrap()
        .build()
}

/// The scroll layout with widths wide enough to overflow a 200 column window.
const SCROLL_TOML: &str = "[ui]\nlog_columns_layout = \"scroll\"\n\n[ui.log_column_widths_scroll]\nref = 20\ngraph = 8\nmessage = 150\nauthor = 14\ndate = 18\nsha = 8\n";

fn launch(repo: &FixtureRepo, session: &str, toml: &str) -> (Terminal, tempfile::TempDir) {
    common::launch(repo.root())
        .session(session)
        .config(toml)
        .size(200, 40)
        .ready_when("the Log header", |s| {
            s.contains("older commit") && header_row(s).is_some()
        })
        .mouse_capture()
        .start()
}

/// The Log header row: the row of the column titles, found while `Author`
/// and `Date` (or the titles still on screen) are on it.
fn header_row(s: &Screen) -> Option<u16> {
    // `Message` is the first title on screen while nothing is scrolled.
    s.find("Message").map(|(_, r)| r)
}

fn title_x(s: &Screen, row: u16, title: &str) -> Option<u16> {
    s.find_all(title)
        .into_iter()
        .find(|(_, r)| *r == row)
        .map(|(c, _)| c)
}

/// `(left arrow col, right arrow col, row)` of the Log's sideways bar: on the
/// bottom border, from the `╰` corner's neighbour.
fn bar(s: &Screen) -> Option<(u16, u16, u16)> {
    let (left, row) = s.find_all("\u{2190}").into_iter().find(|(c, r)| {
        *c > 0
            && s.cell(*c - 1, *r)
                .is_some_and(|cell| cell.text == "\u{2570}")
    })?;
    let right = (left + 1..s.cols())
        .rev()
        .find(|c| s.cell(*c, row).is_some_and(|cell| cell.text == "\u{2192}"))?;
    Some((left, right, row))
}

/// The thumb's cells as `(first, last)` at the bar's start: the leading run
/// after the left arrow sharing the first cell's background.
fn thumb(s: &Screen) -> Option<(u16, u16)> {
    let (left, right, row) = bar(s)?;
    let bg = s.cell(left + 1, row)?.bg;
    let mut last = left + 1;
    while last + 1 < right && s.cell(last + 1, row).is_some_and(|c| c.bg == bg) {
        last += 1;
    }
    (s.cell(right - 1, row)?.bg != bg).then_some((left + 1, last))
}

fn stored(config: &std::path::Path) -> String {
    std::fs::read_to_string(config.join("config.toml")).unwrap_or_default()
}

fn wait_stored(config: &std::path::Path, needle: &str) {
    let deadline = std::time::Instant::now() + STEP;
    loop {
        let text = stored(config);
        if text.contains(needle) {
            return;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "{needle} not stored: {text}"
        );
        std::thread::sleep(Duration::from_millis(100));
    }
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

/// The chooser's "Scroll sideways" checkbox flips the layout (the popup stays
/// open, as every checkbox does) and `config.toml` holds it; the keyboard does
/// the same.
///
/// Falsified: with `toggle_layout` not writing the file, red at `scroll`
/// stored; with `run_chooser_row` ignoring `ChooserRow::Layout`, red at the
/// checked box.
#[test]
fn the_chooser_row_switches_the_layout_and_the_config_keeps_it() {
    let repo = fixture();
    let (mut term, config) = launch(&repo, "scroll-chooser", "");
    let row = header_row(&snap(&term)).unwrap();
    let x = title_x(&snap(&term), row, "Author").unwrap();
    term.right_click(x + 1, row).unwrap();
    let s = term
        .wait_for("the chooser", STEP, |s| {
            s.contains("[ ] Scroll sideways (columns keep their widths)")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (cx, cy) = s.find("[ ] Scroll sideways").unwrap();
    term.click(cx + 4, cy).unwrap();
    term.wait_for("the box checked", STEP, |s| {
        s.contains("[x] Scroll sideways (columns keep their widths)")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    wait_stored(config.path(), "log_columns_layout = \"scroll\"");
    // The popup stays open; the keyboard flips it back: from the first row
    // Up wraps over Reset, Fit all and the Fit row to the layout row.
    term.send_keys("Up Up Up Up").unwrap();
    std::thread::sleep(Duration::from_millis(200));
    term.send_text(" ").unwrap();
    term.wait_for("the box unchecked", STEP, |s| {
        s.contains("[ ] Scroll sideways (columns keep their widths)")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    wait_stored(config.path(), "log_columns_layout = \"fit\"");
    quit(term);
}

/// Fitting `Message` to a long summary in the scroll layout grows the table
/// past the pane and the bar appears; `Shift+Right` scrolls `Ref` off the
/// header, `Shift+Left` brings it back.
///
/// Falsified: with the bar's registration condition (`log_geo.scrolls()`)
/// replaced by `false` the bar never shows (red at "the bar"); with the
/// `LogScrollRight` arm removed from `keys_normal_lists.rs`, `Shift+Right`
/// opens nothing and `Ref` stays (red at "Ref scrolled off").
#[test]
fn fitting_message_shows_the_bar_and_shift_arrows_scroll_the_header() {
    let repo = fixture();
    let (mut term, _config) = launch(
        &repo,
        "scroll-keys",
        "[ui]\nlog_columns_layout = \"scroll\"\n",
    );
    let row = header_row(&snap(&term)).unwrap();
    assert!(bar(&snap(&term)).is_none(), "no bar while the columns fit");
    let ref_x = title_x(&snap(&term), row, "Ref").unwrap();
    // Pick Message (Shift+C, Tab, Tab) and fit it.
    term.send_keys("C Tab Tab").unwrap();
    term.wait_for("Message picked", STEP, |s| {
        s.contains("Message column picked")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_text("=").unwrap();
    term.wait_for("the bar", STEP, |s| bar(s).is_some())
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Escape").unwrap();
    assert_eq!(title_x(&snap(&term), row, "Ref"), Some(ref_x));

    // Scroll right until Ref is gone (Ref and Graph are about 30 cells).
    for _ in 0..9 {
        term.send_keys("S-Right").unwrap();
    }
    term.wait_for("Ref scrolled off", STEP, |s| {
        title_x(s, row, "Ref").is_none()
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // Picking a column (Shift+C picks the first, Ref) scrolls it into view.
    term.send_keys("C").unwrap();
    term.wait_for("the picked Ref scrolled into view", STEP, |s| {
        title_x(s, row, "Ref") == Some(ref_x)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Escape").unwrap();
    for _ in 0..9 {
        term.send_keys("S-Right").unwrap();
    }
    term.wait_for("Ref scrolled off again", STEP, |s| {
        title_x(s, row, "Ref").is_none()
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    for _ in 0..9 {
        term.send_keys("S-Left").unwrap();
    }
    term.wait_for("Ref back", STEP, |s| title_x(s, row, "Ref") == Some(ref_x))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    quit(term);
}

/// In the fit layout `Shift+Right` scrolls nothing and says why; the bar does
/// not exist.
#[test]
fn in_the_fit_layout_there_is_nothing_to_scroll() {
    let repo = fixture();
    let (mut term, _config) = launch(&repo, "scroll-fit", "");
    let row = header_row(&snap(&term)).unwrap();
    let ref_x = title_x(&snap(&term), row, "Ref").unwrap();
    term.send_keys("S-Right").unwrap();
    term.wait_for("the status line", STEP, |s| {
        s.contains("The columns fit the pane")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert_eq!(title_x(&snap(&term), row, "Ref"), Some(ref_x));
    assert!(bar(&snap(&term)).is_none());
    quit(term);
}

/// The horizontal wheel and Shift+wheel scroll the columns over the table.
///
/// Falsified: with the wheel arm of `mouse_scroll.rs` removed, `Ref` stays
/// (red at "ScrollRight").
#[test]
fn the_sideways_wheel_and_shift_wheel_scroll_the_columns() {
    let repo = fixture();
    let (mut term, _config) = launch(&repo, "scroll-wheel", SCROLL_TOML);
    let row = header_row(&snap(&term)).unwrap();
    let ref_x = title_x(&snap(&term), row, "Ref").unwrap();
    for _ in 0..8 {
        term.scroll_sideways(false, ref_x + 4, row + 3).unwrap();
    }
    term.wait_for("ScrollRight moved the header", STEP, |s| {
        title_x(s, row, "Ref").is_none_or(|x| x < ref_x)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let moved = title_x(&snap(&term), row, "Author");
    for _ in 0..8 {
        term.scroll_sideways(true, ref_x + 4, row + 3).unwrap();
    }
    term.wait_for("ScrollLeft brought it back", STEP, |s| {
        title_x(s, row, "Ref") == Some(ref_x)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    for _ in 0..8 {
        term.shift_scroll(true, ref_x + 4, row + 3).unwrap();
    }
    term.wait_for("Shift+wheel moved it", STEP, |s| {
        title_x(s, row, "Author") != Some(title_x(s, row, "Date").unwrap_or(0))
            && {
                let a = title_x(s, row, "Author");
                a.is_none() || a != moved || a.is_some_and(|x| x < ref_x + 200)
            }
            && title_x(s, row, "Ref") != Some(ref_x)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    quit(term);
}

/// The bar's thumb drags and a press on the bare track brings the thumb under
/// the pointer: the header moves with it. The press never pages.
///
/// Falsified: with the `LogHorizontal` thumb arm of `mouse_left.rs` returning
/// `None` the drag does nothing (red at "dragged"); with the track arm empty
/// the press does nothing (red at "track press").
#[test]
fn the_bar_thumb_drags_and_a_track_press_jumps() {
    let repo = fixture();
    let (mut term, _config) = launch(&repo, "scroll-bar", SCROLL_TOML);
    let row = header_row(&snap(&term)).unwrap();
    let ref_x = title_x(&snap(&term), row, "Ref").unwrap();
    let s = term
        .wait_for("the bar", STEP, |s| thumb(s).is_some())
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (first, last) = thumb(&s).unwrap();
    let (_, right, bar_row) = bar(&s).unwrap();
    // Grab the thumb and drag it a third of the way.
    let mid = (first + last) / 2;
    term.press(mid, bar_row).unwrap();
    term.drag_to(mid + 8, bar_row).unwrap();
    term.drag_to(mid + 25, bar_row).unwrap();
    term.release(mid + 25, bar_row).unwrap();
    term.wait_for("dragged", STEP, |s| {
        title_x(s, row, "Ref").is_none_or(|x| x < ref_x)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // The bare track near its end: the thumb jumps there and SHA shows.
    term.click(right - 3, bar_row).unwrap();
    term.wait_for("track press: SHA in view", STEP, |s| {
        title_x(s, row, "SHA").is_some() && title_x(s, row, "Ref").is_none()
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    quit(term);
}

/// While scrolled a click on a row selects the commit on that row and
/// hovering the Author cell shows that cell's tooltip: hit testing follows the
/// scroll, not the unscrolled layout.
///
/// Falsified: with `CommandCtx::log_geo` built with offset 0 the hover maps to
/// another column and the email never shows (red at "the email tooltip").
#[test]
fn while_scrolled_clicks_and_tooltips_hit_the_cells_they_show() {
    let repo = fixture();
    let (mut term, _config) = launch(&repo, "scroll-hit", SCROLL_TOML);
    let row = header_row(&snap(&term)).unwrap();
    // Scroll until Ref, Graph and Message are gone and Author is on screen.
    for _ in 0..30 {
        term.send_keys("S-Right").unwrap();
    }
    let s = term
        .wait_for("Ref gone, Author in view", STEP, |s| {
            title_x(s, row, "Ref").is_none() && title_x(s, row, "Author").is_some()
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let author_x = title_x(&s, row, "Author").unwrap();
    // The older commit is the second row.
    term.click(author_x + 2, row + 2).unwrap();
    term.wait_for("the older commit selected", STEP, |s| {
        s.contains("parent: (root)") || s.contains("(root)")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(!term.text().contains(EMAIL));
    term.move_to(author_x + 1, row + 1).unwrap();
    term.move_to(author_x + 2, row + 1).unwrap();
    term.wait_for("the email tooltip", STEP, |s| s.contains(EMAIL))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    quit(term);
}

/// Settings > Log & graph > Column layout applies live: the bar appears the
/// moment the choice is made.
#[test]
fn the_settings_choice_applies_live() {
    let repo = fixture();
    let toml = "[ui.log_column_widths_scroll]\nmessage = 150\n";
    let (mut term, config) = launch(&repo, "scroll-settings", toml);
    assert!(bar(&snap(&term)).is_none());
    let (gc, gr) = term
        .wait_for_text("\u{2699}", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(gc, gr).unwrap();
    term.wait_for("Settings open", STEP, |s| {
        s.contains("Repo-specific settings")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    common::settings_page(&mut term, "Log & graph");
    // Retried: a click aimed at a torn or still-scrolling frame opens nothing.
    // The popup is whole when both rows are drawn.
    let screen = common::retry_until(
        &mut term,
        "the popup",
        4,
        std::time::Duration::from_secs(10),
        |t| {
            let (col, srow) = common::scroll_settings_to_choice(t, "Fit the pane", STEP);
            t.click(col + 1, srow).unwrap();
        },
        |s| {
            s.find("\u{2713} Fit the pane")
                .is_some_and(|(_, r)| s.line(r + 1).contains("Scroll sideways"))
        },
    );
    let (tc, tr) = screen.find("\u{2713} Fit the pane").unwrap();
    assert!(
        screen.line(tr + 1).contains("Scroll sideways"),
        "{}",
        term.text()
    );
    term.click(tc + 2, tr + 1).unwrap();
    wait_stored(config.path(), "log_columns_layout = \"scroll\"");
    term.send_keys("Escape").unwrap();
    term.wait_for("the bar, live", STEP, |s| bar(s).is_some())
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    quit(term);
}

/// A narrow window in the scroll layout gives nothing up: the SHA header can
/// be reached by scrolling.
#[test]
fn a_narrow_window_can_reach_the_sha_column_by_scrolling() {
    let repo = fixture();
    let (mut term, _config) = common::launch(repo.root())
        .session("scroll-narrow")
        .config("[ui]\nlog_columns_layout = \"scroll\"\n")
        .size(90, 30)
        .ready("older commit")
        .mouse_capture()
        .start();
    let screen = snap(&term);
    let row = header_row(&screen).unwrap();
    assert!(
        title_x(&screen, row, "SHA").is_none(),
        "SHA is off to the right at this width\n{}",
        term.text()
    );
    for _ in 0..30 {
        term.send_keys("S-Right").unwrap();
    }
    term.wait_for("SHA reached", STEP, |s| title_x(s, row, "SHA").is_some())
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    quit(term);
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
