//! The Log table's headers and cells in French (I5a): a header must fit the
//! column it heads at the widths the window really gives it, in both git
//! vocabularies, and no header may be longer than its English word.

use super::changes_off;
use super::*;
use gitpoulpi_i18n::{override_git_terms, override_lang, Lang};
use ratatui::backend::TestBackend;
use ratatui::Terminal;

#[test]
fn no_header_is_longer_than_the_english_one() {
    let english: Vec<(LogColumn, usize)> = LogColumn::ALL
        .iter()
        .map(|&c| (c, display_width(c.title())))
        .collect();
    for translated in [false, true] {
        let _t = override_git_terms(translated);
        let _l = override_lang(Lang::Fr);
        for (c, en) in &english {
            let fr = display_width(c.title());
            assert!(
                fr <= *en,
                "{c:?}: French header {:?} ({fr}) is longer than the English one ({en})",
                c.title()
            );
        }
    }
}

/// Renders the header row at `width` columns and returns it.
fn header_row(
    width: u16,
    hidden: &std::collections::HashSet<LogColumn>,
) -> (String, Vec<LogColumn>) {
    let order: Vec<LogColumn> = LogColumn::ALL
        .into_iter()
        .filter(|c| !hidden.contains(c))
        .collect();
    let widths = LogColumnWidths::for_area_width(width, hidden, &std::collections::HashMap::new());
    let area = Rect::new(0, 0, width, 4);
    let mut terminal = Terminal::new(TestBackend::new(width, 4)).unwrap();
    terminal
        .draw(|frame| {
            let header: Vec<Cell> = order.iter().map(|&c| Cell::from(c.title())).collect();
            frame.render_stateful_clipped(
                log_table(Vec::new(), header, &order, &widths, Style::default()),
                area,
                &mut TableState::default(),
            );
        })
        .unwrap();
    let buf = terminal.backend().buffer().clone();
    let row: String = (0..width)
        .map(|x| buf[(x, 1)].symbol().to_string())
        .collect();
    (row, order)
}

#[test]
fn french_headers_are_printed_whole_at_100_and_160_columns() {
    let _l = override_lang(Lang::Fr);
    for translated in [false, true] {
        let _t = override_git_terms(translated);
        // The window's Log pane is narrower than the terminal: the sidebar
        // and the right panel take the rest (about 45 % at 100, 55 % at 160).
        for (term_width, log_width) in [(100u16, 46u16), (160, 88)] {
            let (row, order) = header_row(log_width, &changes_off());
            let widths = LogColumnWidths::for_area_width(
                log_width,
                &changes_off(),
                &std::collections::HashMap::new(),
            );
            for c in order {
                let w = widths.width_of(c) as usize;
                if w == 0 {
                    continue; // sacrificed by the narrow pane, same as in English
                }
                // Only the columns the English header fits in are held to it.
                let en = gitpoulpi_i18n::with_lang(Lang::En, || display_width(c.title()));
                if w >= en {
                    assert!(
                        row.contains(c.title()),
                        "{term_width} cols, {c:?}: {:?} missing from {row:?}",
                        c.title()
                    );
                }
            }
        }
    }
}

#[test]
fn the_changes_cell_speaks_french_and_shortens_when_narrow() {
    let _l = override_lang(Lang::Fr);
    let stat = gitpoulpi_core::domain::CommitStat {
        id: gitpoulpi_core::domain::Oid([0; 20]),
        files: 3,
        insertions: 12,
        deletions: 4,
    };
    let text = |w| -> String {
        changes_spans(Some(&stat), w, Style::default())
            .iter()
            .map(|s| s.content.to_string())
            .collect()
    };
    assert_eq!(text(40), "3 fichiers +12 -4");
    assert_eq!(text(9), "3f +12 -4");
    let one = gitpoulpi_core::domain::CommitStat {
        id: gitpoulpi_core::domain::Oid([0; 20]),
        files: 1,
        insertions: 0,
        deletions: 0,
    };
    let t: String = changes_spans(Some(&one), 40, Style::default())
        .iter()
        .map(|s| s.content.to_string())
        .collect();
    assert_eq!(t, "1 fichier");
}

/// Renders the header row for `order` at exactly `widths`, through the same
/// cell builder the Log draws with; one string per column (border excluded).
fn narrow_header(widths: &LogColumnWidths, order: &[LogColumn]) -> Vec<String> {
    let total: u16 = order.iter().map(|&c| widths.width_of(c)).sum::<u16>() + 2;
    let mut terminal = Terminal::new(TestBackend::new(total, 4)).unwrap();
    terminal
        .draw(|frame| {
            let header = log_header_cells(order, widths, None, false);
            frame.render_stateful_clipped(
                log_table(Vec::new(), header, order, widths, Style::default()),
                Rect::new(0, 0, total, 4),
                &mut TableState::default(),
            );
        })
        .unwrap();
    let buf = terminal.backend().buffer().clone();
    let mut x = 1u16;
    order
        .iter()
        .map(|&c| {
            let w = widths.width_of(c);
            // A wide glyph's second cell reads as a blank: skip it.
            let mut s = String::new();
            let mut cx = x;
            while cx < x + w {
                let sym = buf[(cx, 1)].symbol();
                s.push_str(sym);
                cx += (display_width(sym) as u16).max(1);
            }
            x += w;
            s
        })
        .collect()
}

#[test]
fn a_narrow_column_ellipsises_its_header_in_every_language_and_keeps_the_gap() {
    for lang in Lang::ALL {
        let _l = override_lang(lang);
        for narrow in 1..=8u16 {
            let widths = LogColumnWidths {
                author_w: narrow,
                date_w: 16,
                ..Default::default()
            };
            let order = [LogColumn::Author, LogColumn::Date];
            let cells = narrow_header(&widths, &order);
            let author = LogColumn::Author.title();
            let date = LogColumn::Date.title();
            // The Author header also carries the author filter's funnel when
            // the title and the funnel both fit.
            let a = cells[0]
                .trim_end()
                .trim_end_matches(crate::author_filter::FUNNEL)
                .trim_end();
            let d = cells[1].trim_end();
            assert!(
                display_width(&cells[0]) <= narrow as usize,
                "{lang:?} {narrow}: {a:?}"
            );
            // Date's header starts exactly at its own column when it fits.
            assert!(
                display_width(date) > 16 || d == date,
                "{lang:?} {narrow}: Date header {d:?} != {date:?} (Author cell {a:?})"
            );
            if display_width(author) <= narrow as usize {
                assert_eq!(a, author, "{lang:?} {narrow}");
            } else {
                assert!(
                    a.is_empty()
                        || (a.ends_with('\u{2026}') && author.starts_with(&a[..a.len() - 3])),
                    "{lang:?} {narrow}: cut header {a:?} is not an ellipsised prefix of {author:?}"
                );
            }
        }
    }
}

#[test]
fn the_chinese_author_header_never_eats_the_gap_at_three_cells() {
    let _l = override_lang(Lang::ZhCn);
    let widths = LogColumnWidths {
        author_w: 3,
        date_w: 16,
        ..Default::default()
    };
    let cells = narrow_header(&widths, &[LogColumn::Author, LogColumn::Date]);
    assert_eq!(
        cells[0], "\u{4f5c}\u{2026}",
        "wide glyph + ellipsis, 3 cells"
    );
    assert_eq!(cells[1].trim_end(), LogColumn::Date.title());
}
