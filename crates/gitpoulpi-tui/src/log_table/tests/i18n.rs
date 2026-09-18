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
