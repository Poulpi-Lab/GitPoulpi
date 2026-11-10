//! The owner's two reports, end to end through a pty (real binary, real
//! daemon, real git), in Simplified Chinese, Japanese and Korean:
//!
//! * the Log's commit context menu: every label is contiguous (no cell
//!   between two CJK characters), the shortcut column is flush right, and the
//!   right border is intact on every row;
//! * Settings' language dropdown: the endonyms read "简体中文", "日本語",
//!   "한국어" (not spaced out) and the popup's box is whole over the page
//!   behind it.
//!
//! `Screen::line` shows a double-width glyph's blank second half as a space;
//! these tests read rows with `Screen::compact_line`, which is what a human
//! sees. The rules are `pty_ui.rs`'s: nothing is located by a hard-coded
//! coordinate, nothing sleeps, every wait describes the whole state.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::{Screen, Terminal};

const STEP: Duration = Duration::from_secs(30);

const LANGS: [&str; 3] = ["zh-CN", "ja", "ko"];

fn fixture() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit("初始提交 日本語 한국어")
        .unwrap()
        .commit("修复登录页面的错误")
        .unwrap()
        .build()
}

fn launch(repo: &FixtureRepo, session: &str, lang: &str) -> (Terminal, tempfile::TempDir) {
    common::launch(repo.root())
        .session(session)
        .size(200, 50)
        .config(format!("[ui]\nlanguage = \"{lang}\"\n"))
        .env("GITPOULPI_LANG", lang)
        .ready_when("the CJK commits loaded", |s| {
            (0..s.rows()).any(|r| s.compact_line(r).contains("修复登录页面"))
        })
        .mouse_capture()
        .start()
}

fn is_wide(c: char) -> bool {
    matches!(c,
        '\u{1100}'..='\u{115F}'
        | '\u{2E80}'..='\u{A4CF}'
        | '\u{AC00}'..='\u{D7A3}'
        | '\u{F900}'..='\u{FAFF}'
        | '\u{FE30}'..='\u{FE6F}'
        | '\u{FF00}'..='\u{FF60}')
}

/// A blank between two wide characters: the "简 体 中 文" of the report.
fn spaced_cjk(line: &str) -> Option<String> {
    let chars: Vec<char> = line.chars().collect();
    chars
        .windows(3)
        .find(|w| is_wide(w[0]) && w[1] == ' ' && is_wide(w[2]))
        .map(|w| w.iter().collect())
}

fn cell_text(screen: &Screen, col: u16, row: u16) -> String {
    screen
        .cell(col, row)
        .map(|c| c.display().to_string())
        .unwrap_or_default()
}

/// The box around the popup that holds `(col, row)`: `(x0, y0, x1, y1)`,
/// found by walking out to its corners, and asserted whole: a `│` (or a
/// scrollbar's cell, under an arrow) on both sides of every row, `╰ ─ ╯`
/// along the bottom.
fn assert_box_around(term: &Terminal, what: &str, col: u16, row: u16) -> (u16, u16, u16, u16) {
    let screen = term.screen();
    let dump = || {
        (0..screen.rows())
            .map(|r| screen.compact_line(r))
            .collect::<Vec<_>>()
            .join("\n")
    };
    // Up along the left side to the top-left corner.
    let mut x0 = col;
    while x0 > 0 && cell_text(&screen, x0, row) != "\u{2502}" {
        x0 -= 1;
    }
    let mut y0 = row;
    while y0 > 0 && cell_text(&screen, x0, y0) != "\u{256D}" {
        assert_eq!(
            cell_text(&screen, x0, y0),
            "\u{2502}",
            "{what}: the left side breaks at ({x0},{y0})\n{}",
            dump()
        );
        y0 -= 1;
    }
    assert_eq!(cell_text(&screen, x0, y0), "\u{256D}", "{what}\n{}", dump());
    // Right along the top to the top-right corner.
    let mut x1 = x0 + 1;
    while x1 < screen.cols() && cell_text(&screen, x1, y0) != "\u{256E}" {
        x1 += 1;
    }
    assert!(
        x1 < screen.cols(),
        "{what}: no top-right corner\n{}",
        dump()
    );
    // Down to the bottom-left corner.
    let mut y1 = y0 + 1;
    while y1 < screen.rows() && cell_text(&screen, x0, y1) != "\u{2570}" {
        y1 += 1;
    }
    assert!(
        y1 < screen.rows(),
        "{what}: no bottom-left corner\n{}",
        dump()
    );
    let has_bar = (y0 + 1..y1).any(|y| {
        matches!(
            cell_text(&screen, x1, y).as_str(),
            "\u{2191}" | "\u{25B2}" | "\u{2193}" | "\u{25BC}"
        )
    });
    for y in y0 + 1..y1 {
        let l = cell_text(&screen, x0, y);
        assert_eq!(l, "\u{2502}", "{what}: left side at ({x0},{y})\n{}", dump());
        let r = cell_text(&screen, x1, y);
        let ok = r == "\u{2502}" || (has_bar && matches!(r.as_str(), " " | "\u{2588}" | "\u{2503}" | "\u{2191}" | "\u{2193}" | "\u{25A0}" | "\u{2591}" | "\u{2592}"));
        assert!(ok, "{what}: right side at ({x1},{y}) is {r:?}\n{}", dump());
    }
    assert_eq!(
        cell_text(&screen, x1, y1),
        "\u{256F}",
        "{what}: bottom-right corner\n{}",
        dump()
    );
    for x in x0 + 1..x1 {
        assert_eq!(
            cell_text(&screen, x, y1),
            "\u{2500}",
            "{what}: bottom at ({x},{y1})\n{}",
            dump()
        );
    }
    (x0, y0, x1, y1)
}

#[test]
fn the_commit_context_menu_is_whole_and_unspaced_in_cjk() {
    let repo = fixture();
    let short = repo
        .git(&["rev-parse", "--short=7", "HEAD~1"])
        .trim()
        .to_string();
    for lang in LANGS {
        let (mut term, _cfg) = launch(&repo, &format!("pty-cjk-menu-{lang}"), lang);
        let what = format!("commit menu in {lang}");
        // The older commit's row, located by its first glyph.
        let (col, row) = term
            .screen()
            .find("\u{521D}")
            .unwrap_or_else(|| panic!("{what}: the base commit is not on screen"));
        term.right_click(col + 1, row).unwrap();
        let screen = term
            .wait_for("the commit menu", STEP, |s| {
                (0..s.rows()).any(|r| s.compact_line(r).contains(&format!("\u{256D}{short}")))
            })
            .unwrap_or_else(|e| panic!("{what}: {e}"));
        // The menu's title row, then its box.
        let title_row = (0..screen.rows())
            .find(|r| screen.compact_line(*r).contains(&format!("\u{256D}{short}")))
            .unwrap();
        let title_col = screen.find(&format!("\u{256D}{short}")).unwrap().0;
        let (x0, y0, x1, y1) = assert_box_around(&term, &what, title_col + 1, title_row + 1);
        assert_eq!(y0, title_row, "{what}");
        // Every row of the menu: no blank between two CJK characters, and
        // the text inside the borders fits in them.
        let screen = term.screen();
        let mut shortcut_ends = Vec::new();
        let mut cjk_rows = 0;
        for y in y0 + 1..y1 {
            let inner: String = (x0 + 1..x1)
                .filter(|&x| !screen.cell(x, y).is_some_and(|c| c.wide_continuation))
                .map(|x| cell_text(&screen, x, y))
                .collect();
            if let Some(bad) = spaced_cjk(&inner) {
                panic!("{what}: spaced out {bad:?} in {inner:?}");
            }
            if inner.chars().any(is_wide) {
                cjk_rows += 1;
            }
            // The shortcut column: `Ctrl+G x` ends flush at the inner edge
            // (one cell left of the right border, or of the scrollbar).
            if let Some(at) = inner.find("Ctrl+") {
                let hint = inner[at..].trim_end();
                shortcut_ends.push((y, hint.chars().count() as u16, inner.trim_end().len()));
            }
        }
        assert!(cjk_rows >= 5, "{what}: the menu is not in {lang}?\n{}", term.text());
        // Every shortcut ends at the same cell, flush with the right border.
        let mut ends = Vec::new();
        for (y, _, _) in &shortcut_ends {
            let mut x = x1 - 1;
            while x > x0 && cell_text(&screen, x, *y) == " " {
                x -= 1;
            }
            ends.push(x);
        }
        ends.dedup();
        assert!(
            ends.len() <= 1,
            "{what}: the shortcut column is ragged: ends at {ends:?}\n{}",
            term.text()
        );
        if let Some(&end) = ends.first() {
            assert_eq!(end, x1 - 1, "{what}: the shortcuts are not flush right");
        }
    }
}

#[test]
fn the_settings_language_dropdown_reads_unspaced_and_closes_its_box() {
    let repo = fixture();
    for lang in LANGS {
        let (mut term, _cfg) = launch(&repo, &format!("pty-cjk-lang-{lang}"), lang);
        let what = format!("language dropdown in {lang}");
        let (col, row) = term
            .wait_for_text("\u{2699}", STEP)
            .unwrap_or_else(|e| panic!("{what}: {e}"));
        term.click(col, row).expect("click the gear");
        // Settings opens on General, whose first row is the Language choice:
        // its value is the current language's endonym followed by the arrow.
        let endonym = match lang {
            "zh-CN" => "\u{7B80}\u{4F53}\u{4E2D}\u{6587}",
            "ja" => "\u{65E5}\u{672C}\u{8A9E}",
            _ => "\u{D55C}\u{AD6D}\u{C5B4}",
        };
        let screen = term
            .wait_for("the Language choice", STEP, |s| {
                (0..s.rows()).any(|r| {
                    let l = s.compact_line(r);
                    l.find(endonym)
                        .is_some_and(|i| l[i + endonym.len()..].trim_start().starts_with('\u{25BE}'))
                })
            })
            .unwrap_or_else(|e| panic!("{what}: {e}\n{}", term.text()));
        let first = endonym.chars().next().unwrap().to_string();
        let (c, r) = (0..screen.rows())
            .find_map(|r| {
                let l = screen.compact_line(r);
                l.find(endonym)
                    .filter(|i| l[i + endonym.len()..].trim_start().starts_with('\u{25BE}'))
                    .and(screen.find_all(&first).into_iter().find(|(_, rr)| *rr == r))
            })
            .unwrap();
        term.click(c + 1, r).unwrap();
        let screen = term
            .wait_for("the language popup", STEP, |s| {
                (0..s.rows()).any(|r| s.compact_line(r).contains("Fran\u{e7}ais"))
            })
            .unwrap_or_else(|e| panic!("{what}: {e}\n{}", term.text()));
        // The popup row of Français anchors the box walk.
        let (fc, fr) = screen.find("Fran\u{e7}ais").unwrap();
        let (x0, y0, x1, y1) = assert_box_around(&term, &what, fc, fr);
        // Each endonym that is listed reads whole inside the box.
        let screen = term.screen();
        for name in [
            "\u{7B80}\u{4F53}\u{4E2D}\u{6587}",
            "\u{65E5}\u{672C}\u{8A9E}",
            "\u{D55C}\u{AD6D}\u{C5B4}",
        ] {
            let found = (y0 + 1..y1).any(|y| {
                let inner: String = (x0 + 1..x1)
                    .filter(|&x| !screen.cell(x, y).is_some_and(|c| c.wide_continuation))
                    .map(|x| cell_text(&screen, x, y))
                    .collect();
                inner.contains(name)
            });
            assert!(found, "{what}: {name:?} is missing or spaced out\n{}", term.text());
        }
    }
}
