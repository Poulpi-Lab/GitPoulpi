//! The diff's and the editor's scrollbars, end to end through a pty: the
//! real binary, the real daemon, real `git`.
//!
//! - the horizontal bar shows as soon as a line overflows the *text* column
//!   (not the pane), and dragging its thumb to the end reaches the last
//!   character of the longest line -- Inline and Split;
//! - the vertical bar is an overview ruler (added / removed rows, the cursor,
//!   every occurrence of the selected text) that leaves thumb dragging alone;
//! - the optional minimap (`ui.minimap`): drawn beside the bar, a click jumps,
//!   `M` and the palette toggle it and the choice is saved.
//!
//! Same rules as `pty_ui.rs`: nothing located by a hard-coded coordinate
//! (the block is found from its own corners), and every wait describes the
//! whole state it waits for. The pty runs with images off, so the minimap is
//! its braille fallback.

mod common;

use std::path::Path;
use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::{Color, Screen, Terminal};

const STEP: Duration = Duration::from_secs(30);
const LOG_HEADER: &str = "SHA";

/// `scrollbar.rs`'s track color: any border cell with it (or with a
/// different background, the thumb) belongs to a scrollbar.
const TRACK: Color = Color::Rgb(40, 42, 60);
/// `scroll_marks.rs`: added, removed and modified ticks, and the glyphs.
const ADD: Color = Color::Rgb(80, 200, 120);
const DEL: Color = Color::Rgb(230, 90, 90);
const MODIFIED: Color = Color::Rgb(220, 170, 90);
const HALF_BLOCK: &str = "\u{2590}";
const OCCURRENCE: &str = "\u{258c}";
const CURSOR: &str = "\u{25c6}";

fn launch_with(
    repo: &Path,
    session: &str,
    config_toml: &str,
    size: (u16, u16),
    ready: &str,
) -> (Terminal, tempfile::TempDir) {
    let ready = ready.to_string();
    common::launch(repo)
        .no_commit_column()
        .session(session)
        .config(config_toml)
        .size(size.0, size.1)
        .no_exit_pause()
        .ready_when("the repository window", move |s| {
            s.contains(&ready) && s.contains(LOG_HEADER)
        })
        .mouse_capture()
        .start()
}

/// The central block's corners, found from its own drawing: the `╭` and
/// `╮` around the title's `✕`, and the `╰` below the left one.
#[derive(Debug, Clone, Copy)]
struct Block {
    left: u16,
    right: u16,
    top: u16,
    bottom: u16,
}

impl Block {
    fn of(s: &Screen) -> Option<Block> {
        let (esc, top) = s.find("\u{2715}")?;
        let is = |c: u16, r: u16, glyph: &str| s.cell(c, r).is_some_and(|x| x.text == glyph);
        let left = (0..esc).rev().find(|&c| is(c, top, "\u{256d}"))?;
        let right = (esc..s.cols()).find(|&c| is(c, top, "\u{256e}"))?;
        let bottom = (top + 1..s.rows()).find(|&r| is(left, r, "\u{2570}"))?;
        Some(Block {
            left,
            right,
            top,
            bottom,
        })
    }

    /// Text columns' room: the inner width.
    fn inner_width(self) -> usize {
        (self.right - self.left - 1) as usize
    }
}

/// Waits for one whole state of the block (a torn frame can show part of
/// it): returns the screen and the block found on it.
fn settled(
    term: &Terminal,
    what: &str,
    mut pred: impl FnMut(&Screen, Block) -> bool,
) -> (Screen, Block) {
    let s = term
        .wait_for(what, STEP, |s| Block::of(s).is_some_and(|b| pred(s, b)))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let b = Block::of(&s).expect("the predicate just matched");
    (s, b)
}

fn on_bar(s: &Screen, col: u16, row: u16) -> bool {
    s.cell(col, row).is_some_and(|c| c.bg != Color::Default)
}

/// The horizontal bar's cells on the bottom border.
fn h_bar(s: &Screen, b: Block) -> Vec<u16> {
    (b.left + 1..b.right)
        .filter(|&c| on_bar(s, c, b.bottom))
        .collect()
}

/// The vertical bar's rows on the right border.
fn v_bar(s: &Screen, b: Block) -> Vec<u16> {
    (b.top + 1..b.bottom)
        .filter(|&r| on_bar(s, b.right, r))
        .collect()
}

/// The thumb's cells: a bar cell that is not the plain track.
fn h_thumb(s: &Screen, b: Block) -> Vec<u16> {
    h_bar(s, b)
        .into_iter()
        .filter(|&c| s.cell(c, b.bottom).is_some_and(|x| x.bg != TRACK))
        .collect()
}

fn v_thumb(s: &Screen, b: Block) -> Vec<u16> {
    v_bar(s, b)
        .into_iter()
        .filter(|&r| s.cell(b.right, r).is_some_and(|x| x.bg != TRACK))
        .collect()
}

/// Rows of the right border showing tick `glyph`, optionally of one color.
fn ticks(s: &Screen, b: Block, glyph: &str, fg: Option<Color>) -> Vec<u16> {
    (b.top + 1..b.bottom)
        .filter(|&r| {
            s.cell(b.right, r)
                .is_some_and(|c| c.text == glyph && fg.is_none_or(|f| c.fg == f))
        })
        .collect()
}

/// Braille cells (the minimap fallback) in the ten columns left of the
/// right border.
fn braille(s: &Screen, b: Block) -> usize {
    (b.top + 1..b.bottom)
        .flat_map(|r| (b.right.saturating_sub(10)..b.right).map(move |c| (c, r)))
        .filter(|&(c, r)| {
            s.cell(c, r).is_some_and(|x| {
                x.text
                    .chars()
                    .next()
                    .is_some_and(|ch| ('\u{2801}'..='\u{28ff}').contains(&ch))
            })
        })
        .count()
}

fn body_line(n: usize) -> String {
    format!("line {n:03}")
}

/// 200 lines with `needle` on 5, 100 and 180; then a commit modifying 50,
/// deleting 120 and adding a line after 150.
fn ruler_repo() -> gitpoulpi_testkit::FixtureRepo {
    let mut before = Vec::new();
    for n in 1..=200 {
        before.push(match n {
            5 | 100 | 180 => format!("{} needle here", body_line(n)),
            _ => body_line(n),
        });
    }
    let mut after = before.clone();
    after[49] = "line 050 CHANGED".into();
    after.insert(150, "an added line".into());
    after.remove(119); // line 120
    let text = |v: &[String]| v.join("\n") + "\n";
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("big.txt", &text(&before), "two hundred lines")
        .unwrap()
        .commit_file("big.txt", &text(&after), "edit")
        .unwrap()
        .build()
}

fn open_diff(term: &mut Terminal, file: &str, shown: &str) {
    let (c, r) = term
        .wait_for_text(file, STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.click(c, r).unwrap();
    term.wait_for("the diff in the center", STEP, |s| {
        s.contains("\u{2715}") && s.contains(shown) && !s.contains(LOG_HEADER)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

fn whole_file(term: &mut Terminal, shown: &str) {
    term.send_keys("H").unwrap();
    term.wait_for("the whole file", STEP, |s| s.contains(shown))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

// -- horizontal bar ---------------------------------------------------------

/// 30 short lines, then a commit turning line 10 into a 100-character line
/// ending in ENDMARK.
fn wide_repo(len: usize) -> gitpoulpi_testkit::FixtureRepo {
    let short: Vec<String> = (1..=30).map(|n| format!("short {n:02}")).collect();
    let mut long = short.clone();
    long[9] = format!("{}ENDMARK", "x".repeat(len - 7));
    let text = |v: &[String]| v.join("\n") + "\n";
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("wide.txt", &text(&short), "short lines")
        .unwrap()
        .commit_file("wide.txt", &text(&long), "one long line")
        .unwrap()
        .build()
}

/// The horizontal bar must appear as soon as the longest line overflows the
/// text column, which is 13 columns (the gutter) narrower than the pane.
/// Sweeping the window width covers the range where the line overflows the
/// text but not the pane.
#[test]
fn the_horizontal_bar_shows_as_soon_as_a_line_overflows_the_text_column() {
    const LEN: usize = 100;
    let repo = wide_repo(LEN);
    let (mut term, _config) =
        launch_with(repo.root(), "pty-hbar-appears", "", (160, 40), "wide.txt");
    open_diff(&mut term, "wide.txt", "short 09");

    let mut in_the_old_blind_spot = 0;
    for width in (120..=210u16).step_by(2) {
        term.resize(width, 40).unwrap();
        let s = term
            .wait_for_stable(Duration::from_millis(250), STEP)
            .unwrap_or_else(|e| panic!("{e}"));
        let Some(b) = Block::of(&s) else { continue };
        let text_col = b.inner_width().saturating_sub(13);
        let overflows = LEN > text_col;
        let shown = !h_bar(&s, b).is_empty();
        assert_eq!(
            shown,
            overflows,
            "width {width}: inner {}, text column {text_col}, longest line {LEN}\n{}",
            b.inner_width(),
            s.text()
        );
        if overflows && LEN <= b.inner_width() {
            in_the_old_blind_spot += 1;
        }
    }
    assert!(
        in_the_old_blind_spot > 0,
        "the sweep never reached the widths where the old rule hid the bar"
    );
}

/// Dragging the thumb all the way right reaches the end of the longest
/// line, in Inline and in Split.
#[test]
fn dragging_the_horizontal_thumb_to_the_end_reaches_the_last_character() {
    let repo = wide_repo(200);
    let (mut term, _config) = launch_with(repo.root(), "pty-hbar-end", "", (160, 40), "wide.txt");
    open_diff(&mut term, "wide.txt", "short 09");

    for layout in ["inline", "split"] {
        if layout == "split" {
            term.send_keys("m").unwrap();
            term.wait_for("split", STEP, |s| {
                s.text()
                    .lines()
                    .any(|l| l.contains('\u{2502}') && l.contains("short 09"))
            })
            .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
        }
        let (s, b) = settled(
            &term,
            &format!("{layout}: the bar and its thumb"),
            |s, b| !h_bar(s, b).is_empty() && !h_thumb(s, b).is_empty() && !s.contains("ENDMARK"),
        );
        assert!(
            !h_bar(&s, b).is_empty(),
            "{layout}: 200 chars overflow: no bar\n{}",
            s.text()
        );
        assert!(
            !s.contains("ENDMARK"),
            "{layout}: not visible before scrolling"
        );
        let thumb = h_thumb(&s, b);
        let start = *thumb.first().expect("a thumb");
        term.drag((start, b.bottom), (b.right, b.bottom)).unwrap();
        term.wait_for("the end of the long line", STEP, |s| s.contains("ENDMARK"))
            .unwrap_or_else(|e| panic!("{layout}: {e}\n{}", term.text()));
        let (s, b) = settled(&term, &format!("{layout}: the thumb at the end"), |s, b| {
            h_thumb(s, b).last().copied() == Some(b.right - 2)
        });
        let thumb = h_thumb(&s, b);
        assert_eq!(
            thumb.last().copied(),
            Some(b.right - 2),
            "{layout}: the thumb is at the very end of its track (before the → arrow)"
        );
        // Back to the left for the next layout.
        term.send_keys("Home").ok();
        term.drag((*thumb.first().unwrap(), b.bottom), (b.left, b.bottom))
            .unwrap();
        term.wait_for("scrolled back", STEP, |s| !s.contains("ENDMARK"))
            .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    }
}

// -- overview ruler ---------------------------------------------------------

#[test]
fn the_vertical_bar_marks_changes_the_cursor_and_the_selected_word_and_still_drags() {
    let repo = ruler_repo();
    let (mut term, _config) = launch_with(repo.root(), "pty-ruler", "", (170, 50), "big.txt");
    open_diff(&mut term, "big.txt", "line 050");

    // Hunks-only has no ruler.
    let (s, b) = settled(&term, "the hunk view", |s, _| s.contains("line 050"));
    assert!(
        ticks(&s, b, HALF_BLOCK, None).is_empty(),
        "no ruler in hunk mode"
    );

    whole_file(&mut term, "needle here");
    let (s, b) = settled(&term, "the whole ruler", |s, b| {
        !ticks(s, b, HALF_BLOCK, Some(ADD)).is_empty()
            && (!ticks(s, b, HALF_BLOCK, Some(DEL)).is_empty()
                || !ticks(s, b, HALF_BLOCK, Some(MODIFIED)).is_empty())
            && !ticks(s, b, CURSOR, None).is_empty()
            && s.contains("needle")
            && s.contains("line 006")
    });
    assert!(
        !ticks(&s, b, HALF_BLOCK, Some(ADD)).is_empty(),
        "an added line is green"
    );
    assert!(
        !ticks(&s, b, HALF_BLOCK, Some(DEL)).is_empty()
            || !ticks(&s, b, HALF_BLOCK, Some(MODIFIED)).is_empty(),
        "a removed line is red (or amber next to an added one)"
    );
    assert!(
        !ticks(&s, b, CURSOR, None).is_empty(),
        "the cursor line is marked"
    );

    // Double-click the visible `needle`: every other occurrence gets a tick.
    let (c, r) = s.find("needle").expect("needle on line 5");
    term.double_click(c + 2, r).unwrap();
    let (s, b) = settled(&term, "the occurrences on the bar", |s, b| {
        ticks(s, b, OCCURRENCE, None).len() >= 2
            && !v_thumb(s, b).is_empty()
            && s.contains("line 006")
    });
    assert!(
        ticks(&s, b, OCCURRENCE, None).len() >= 2,
        "lines 100 and 180"
    );

    // The ticks sit on the bar: the thumb still drags. Press on a thumb
    // cell and pull it down; the view follows.
    let thumb = v_thumb(&s, b);
    let first = *thumb.first().expect("a vertical thumb");
    assert!(s.contains("line 006"), "the top of the file shows");
    term.drag((b.right, first), (b.right, first + 15)).unwrap();
    term.wait_for("scrolled by the drag", STEP, |s| !s.contains("line 006"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

// -- minimap ----------------------------------------------------------------

#[test]
fn the_minimap_is_off_by_default_and_the_palette_and_m_toggle_and_save_it() {
    let repo = ruler_repo();
    let (mut term, config) =
        launch_with(repo.root(), "pty-minimap-toggle", "", (170, 50), "big.txt");
    open_diff(&mut term, "big.txt", "line 050");
    whole_file(&mut term, "needle here");
    let (s, b) = settled(&term, "the whole file, no minimap", |s, b| {
        s.contains("needle here") && braille(s, b) == 0
    });
    assert_eq!(braille(&s, b), 0, "off by default");

    // The palette entry.
    term.send_keys("C-p").unwrap();
    term.send_text("Toggle Minimap").unwrap();
    term.wait_for("the palette filtered to the command", STEP, |s| {
        s.contains("View: Toggle Minimap")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    term.wait_for("the minimap drawn", STEP, |s| {
        s.contains("minimap on") && Block::of(s).is_some_and(|b| braille(s, b) > 5)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let saved = std::fs::read_to_string(config.path().join("config.toml")).unwrap_or_default();
    assert!(saved.contains("minimap = true"), "{saved}");

    // The key.
    term.send_keys("M").unwrap();
    term.wait_for("the minimap gone", STEP, |s| {
        s.contains("minimap off") && Block::of(s).is_some_and(|b| braille(s, b) == 0)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let saved = std::fs::read_to_string(config.path().join("config.toml")).unwrap_or_default();
    assert!(saved.contains("minimap = false"), "{saved}");
}

#[test]
fn a_click_on_the_diff_minimap_jumps_to_that_part_of_the_file() {
    let repo = ruler_repo();
    let (mut term, _config) = launch_with(
        repo.root(),
        "pty-minimap-click",
        "[ui]\nminimap = true\n",
        (170, 50),
        "big.txt",
    );
    open_diff(&mut term, "big.txt", "line 050");
    whole_file(&mut term, "needle here");
    let (s, b) = settled(&term, "the minimap and the top of the file", |s, b| {
        braille(s, b) > 5 && s.contains("line 006")
    });
    assert!(s.contains("line 006"));
    // A click three quarters down the minimap: the view centers there.
    // Retried: a click can be lost to a torn frame.
    let row = b.top + 1 + (b.bottom - b.top - 1) * 3 / 4;
    common::retry_until(
        &mut term,
        "the view jumped",
        4,
        STEP,
        |t| {
            if t.screen().contains("line 006") {
                t.click(b.right - 5, row).unwrap();
            }
        },
        |s| !s.contains("line 006") && s.contains("line 15"),
    );
    // Every mouse action has a keyboard twin: PageUp walks back.
    for _ in 0..20 {
        term.send_keys("PageUp").unwrap();
    }
    term.wait_for("back at the top", STEP, |s| s.contains("line 006"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

// -- the editor -------------------------------------------------------------

fn editor_repo() -> gitpoulpi_testkit::FixtureRepo {
    let lines: Vec<String> = (1..=200)
        .map(|n| match n {
            5 | 100 | 180 => format!("{} needle here", body_line(n)),
            _ => body_line(n),
        })
        .collect();
    let mut edited = lines.clone();
    edited[0] = "line 001 edited".into();
    let text = |v: &[String]| v.join("\n") + "\n";
    FixtureRepoBuilder::new()
        .unwrap()
        .write_worktree("big.txt", text(&lines).as_bytes())
        .unwrap()
        .stage(".")
        .unwrap()
        .commit("base")
        .unwrap()
        .write_worktree("big.txt", text(&edited).as_bytes())
        .unwrap()
        .build()
}

fn open_editor(term: &mut Terminal) {
    let (c, r) = term
        .wait_for_text("big.txt", STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.click(c, r).unwrap();
    term.wait_for("the unstaged diff", STEP, |s| {
        s.contains("big.txt - unstaged") && s.contains(" Edit ")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (ec, er) = term.screen().find(" Edit ").unwrap();
    term.click(ec + 2, er).unwrap();
    term.wait_for("the editor", STEP, |s| {
        s.contains("Edit - big.txt @ working tree") && s.contains("line 006")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

#[test]
fn the_editor_bar_marks_the_caret_and_the_selected_word_and_has_a_minimap_that_jumps() {
    let repo = editor_repo();
    let (mut term, _config) = launch_with(
        repo.root(),
        "pty-editor-ruler",
        "[ui]\nminimap = true\n",
        (170, 50),
        "big.txt",
    );
    open_editor(&mut term);
    let (s, b) = settled(&term, "the caret and the minimap on the bar", |s, b| {
        !ticks(s, b, CURSOR, None).is_empty() && braille(s, b) > 5 && s.contains("needle")
    });
    assert!(
        !ticks(&s, b, CURSOR, None).is_empty(),
        "the caret is on the bar"
    );
    assert!(braille(&s, b) > 5, "the minimap is drawn");

    let (c, r) = s.find("needle").expect("needle on line 5");
    term.double_click(c + 2, r).unwrap();
    let (_, b) = settled(&term, "the occurrences on the bar", |s, b| {
        ticks(s, b, OCCURRENCE, None).len() >= 2 && braille(s, b) > 5 && s.contains("line 006")
    });

    // A click on the minimap centers the view on that part of the file.
    // Retried: a click can be lost to a torn frame.
    let row = b.top + 1 + (b.bottom - b.top - 1) * 3 / 4;
    common::retry_until(
        &mut term,
        "the editor scrolled",
        4,
        STEP,
        |t| {
            if t.screen().contains("line 006") {
                t.click(b.right - 5, row).unwrap();
            }
        },
        |s| !s.contains("line 006") && s.contains("line 15"),
    );
}

#[test]
fn the_editor_toggles_the_minimap_from_its_bar_and_with_alt_m() {
    let repo = editor_repo();
    let (mut term, config) =
        launch_with(repo.root(), "pty-editor-minimap", "", (170, 50), "big.txt");
    open_editor(&mut term);
    let (s, b) = settled(&term, "the editor with its Map button", |s, b| {
        braille(s, b) == 0 && s.find(" Map ").is_some()
    });
    assert_eq!(braille(&s, b), 0, "off by default");
    let (c, r) = s.find(" Map ").expect("a Map button on the editor bar");
    let idle_bg = s.cell(c + 1, r).unwrap().bg;

    // The bar's button: on, lit, saved; the editor stays open.
    term.click(c + 2, r).unwrap();
    let s = term
        .wait_for("the editor minimap on, the button lit", STEP, |s| {
            s.contains("minimap on")
                && s.contains("Edit - big.txt @ working tree")
                && Block::of(s).is_some_and(|b| braille(s, b) > 5)
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert_ne!(
        s.cell(c + 1, r).unwrap().bg,
        idle_bg,
        "the button shows its state"
    );
    let saved = std::fs::read_to_string(config.path().join("config.toml")).unwrap_or_default();
    assert!(saved.contains("minimap = true"), "{saved}");

    // The key: off again, and nothing typed into the file.
    term.send_keys("M-m").unwrap();
    term.wait_for("the editor minimap off", STEP, |s| {
        s.contains("minimap off")
            && !s.contains("modified")
            && Block::of(s).is_some_and(|b| braille(s, b) == 0)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let saved = std::fs::read_to_string(config.path().join("config.toml")).unwrap_or_default();
    assert!(saved.contains("minimap = false"), "{saved}");
}

// -- the minimap button -----------------------------------------------------

#[test]
fn the_bar_s_map_button_toggles_the_minimap_by_mouse_and_shows_its_state() {
    let repo = ruler_repo();
    let (mut term, config) =
        launch_with(repo.root(), "pty-minimap-button", "", (170, 50), "big.txt");
    open_diff(&mut term, "big.txt", "line 050");
    whole_file(&mut term, "needle here");
    let (s, b) = settled(&term, "the whole file with its Map button", |s, b| {
        braille(s, b) == 0 && s.contains("needle here") && s.find(" Map ").is_some()
    });
    assert_eq!(braille(&s, b), 0, "off by default");
    let (c, r) = s.find(" Map ").expect("a Map button on the diff bar");
    let idle_bg = s.cell(c + 1, r).unwrap().bg;

    term.click(c + 2, r).unwrap();
    let s = term
        .wait_for("the minimap on, the button lit", STEP, |s| {
            s.contains("minimap on") && Block::of(s).is_some_and(|b| braille(s, b) > 5)
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert_ne!(
        s.cell(c + 1, r).unwrap().bg,
        idle_bg,
        "the button shows its active state"
    );
    let saved = std::fs::read_to_string(config.path().join("config.toml")).unwrap_or_default();
    assert!(saved.contains("minimap = true"), "{saved}");

    term.click(c + 2, r).unwrap();
    term.wait_for("the minimap off again", STEP, |s| {
        s.contains("minimap off") && Block::of(s).is_some_and(|b| braille(s, b) == 0)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

// -- occurrences tinted in the text -----------------------------------------

/// `needle` on lines 5, 8 and 12 of 30, so several occurrences are on screen.
fn tint_lines() -> Vec<String> {
    (1..=30)
        .map(|n| match n {
            5 | 8 | 12 => format!("{} needle here", body_line(n)),
            _ => body_line(n),
        })
        .collect()
}

/// The background of the first cell of `word` on the row showing `row_text`.
fn word_bg(s: &Screen, row_text: &str, word: &str) -> Color {
    let (c, r) = s
        .find(row_text)
        .unwrap_or_else(|| panic!("no {row_text:?}\n{}", s.text()));
    let off = row_text.find(word).expect("word in row text") as u16;
    s.cell(c + off, r).unwrap().bg
}

#[test]
fn selecting_a_word_in_the_diff_tints_its_other_occurrences_and_only_those() {
    let lines = tint_lines();
    let mut edited = lines.clone();
    edited[19] = "line 020 CHANGED".into();
    let text = |v: &[String]| v.join("\n") + "\n";
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("t.txt", &text(&lines), "base")
        .unwrap()
        .commit_file("t.txt", &text(&edited), "edit")
        .unwrap()
        .build();
    let (mut term, _config) = launch_with(repo.root(), "pty-occ-diff", "", (170, 50), "t.txt");
    open_diff(&mut term, "t.txt", "line 020");
    whole_file(&mut term, "needle here");

    let s = term
        .wait_for("the rows, nothing tinted", STEP, |s| {
            ["005", "008", "012"]
                .iter()
                .all(|n| s.find(&format!("line {n} needle here")).is_some())
                && word_bg(s, "line 008 needle here", "needle")
                    == word_bg(s, "line 008 needle here", "line")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert_eq!(
        word_bg(&s, "line 008 needle here", "needle"),
        word_bg(&s, "line 008 needle here", "line"),
        "nothing is tinted before a selection"
    );

    let (c, r) = s.find("line 005 needle here").expect("line 5");
    term.double_click(c + 10, r).unwrap();
    let s = term
        .wait_for("the other occurrences tinted", STEP, |s| {
            s.find("line 008 needle here").is_some()
                && word_bg(s, "line 008 needle here", "needle")
                    != word_bg(s, "line 008 needle here", "line")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let tint = word_bg(&s, "line 008 needle here", "needle");
    assert!(matches!(tint, Color::Rgb(..)), "a soft tint, got {tint:?}");
    assert_eq!(
        tint,
        word_bg(&s, "line 012 needle here", "needle"),
        "same tint everywhere"
    );
    assert_ne!(
        tint,
        word_bg(&s, "line 005 needle here", "needle"),
        "the selection itself has its own color"
    );
    assert_eq!(
        word_bg(&s, "line 008 needle here", "here"),
        word_bg(&s, "line 008 needle here", "line"),
        "only the matching word is tinted"
    );
}

#[test]
fn selecting_a_word_in_the_editor_tints_its_other_occurrences() {
    let lines = tint_lines();
    let mut edited = lines.clone();
    edited[0] = "line 001 edited".into();
    let text = |v: &[String]| v.join("\n") + "\n";
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .write_worktree("big.txt", text(&lines).as_bytes())
        .unwrap()
        .stage(".")
        .unwrap()
        .commit("base")
        .unwrap()
        .write_worktree("big.txt", text(&edited).as_bytes())
        .unwrap()
        .build();
    let (mut term, _config) = launch_with(repo.root(), "pty-occ-editor", "", (170, 50), "big.txt");
    open_editor(&mut term);

    let s = term
        .wait_for("the rows, nothing tinted", STEP, |s| {
            ["005", "008", "012"]
                .iter()
                .all(|n| s.find(&format!("line {n} needle here")).is_some())
                && word_bg(s, "line 008 needle here", "needle")
                    == word_bg(s, "line 008 needle here", "line")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert_eq!(
        word_bg(&s, "line 008 needle here", "needle"),
        word_bg(&s, "line 008 needle here", "line"),
        "nothing is tinted before a selection"
    );
    let (c, r) = s.find("line 005 needle here").expect("line 5");
    term.double_click(c + 10, r).unwrap();
    let s = term
        .wait_for("the other occurrences tinted", STEP, |s| {
            s.find("line 008 needle here").is_some()
                && word_bg(s, "line 008 needle here", "needle")
                    != word_bg(s, "line 008 needle here", "line")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(matches!(
        word_bg(&s, "line 012 needle here", "needle"),
        Color::Rgb(..)
    ));
    assert_eq!(
        word_bg(&s, "line 008 needle here", "here"),
        word_bg(&s, "line 008 needle here", "line"),
        "only the matching word is tinted"
    );
}

// -- a click on the track brings the thumb under the pointer ----------------

/// The maintainer's rule: pressing a scrollbar's bare track puts the thumb under
/// the pointer (the content jumps in proportion, not by a page) and the press
/// keeps dragging until release.
#[test]
fn a_press_on_the_diff_track_jumps_the_thumb_under_the_pointer_and_keeps_dragging() {
    let repo = ruler_repo();
    let (mut term, _config) =
        launch_with(repo.root(), "pty-track-jump-diff", "", (170, 50), "big.txt");
    open_diff(&mut term, "big.txt", "line 050");
    whole_file(&mut term, "needle here");
    let (s, b) = settled(&term, "the top of the file and its bar", |s, b| {
        s.contains("line 006") && !v_bar(s, b).is_empty()
    });
    assert!(s.contains("line 006"), "the top of the file shows");
    let last_track = b.bottom - 2;
    term.press(b.right, last_track).unwrap();
    term.wait_for("jumped to the end, not one page down", STEP, |s| {
        s.contains("line 200") && !s.contains("line 006")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // Still held: moving without release drags the thumb back to the top.
    term.drag_to(b.right, b.top + 2).unwrap();
    term.wait_for("dragged back to the top", STEP, |s| {
        s.contains("line 006") && !s.contains("line 200")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.release(b.right, b.top + 2).unwrap();
}

#[test]
fn a_press_on_the_editor_track_jumps_the_thumb_under_the_pointer_and_keeps_dragging() {
    let repo = editor_repo();
    let (mut term, _config) = launch_with(
        repo.root(),
        "pty-track-jump-editor",
        "",
        (170, 50),
        "big.txt",
    );
    open_editor(&mut term);
    let (_, b) = settled(&term, "the editor and its bar", |s, b| {
        s.contains("line 006") && !v_bar(s, b).is_empty()
    });
    let last_track = b.bottom - 2;
    term.press(b.right, last_track).unwrap();
    term.wait_for("jumped to the end, not one page down", STEP, |s| {
        s.contains("line 200") && !s.contains("line 006")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.drag_to(b.right, b.top + 2).unwrap();
    term.wait_for("dragged back to the top", STEP, |s| {
        s.contains("line 006") && !s.contains("line 200")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.release(b.right, b.top + 2).unwrap();
}
