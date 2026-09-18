//! The Diff view's bar and buttons in French (I5b): at 100 columns the
//! bar still offers its controls, the wide bar shows every button, git
//! terms on or off, and no label is left as a placeholder or in English.

use super::*;
use crate::th;
use gitpoulpi_i18n::{override_git_terms, override_lang, Lang};

fn each_french(mut f: impl FnMut(bool)) {
    for terms in [false, true] {
        let _l = override_lang(Lang::Fr);
        let _t = override_git_terms(terms);
        f(terms);
    }
}

#[test]
fn the_french_bar_fits_at_100_and_200_columns() {
    let state = DiffViewState::for_working(DiffSource::Unstaged, PathBuf::from("a"));
    let english_total: usize = {
        DIFF_BAR_BUTTONS
            .iter()
            .map(|b| display_width(&format!(" {} ", b.bar_label())) + 1)
            .sum()
    };
    each_french(|terms| {
        let wide = diff_bar(&state, None, false, 200);
        // The column selector is one item with two hits.
        assert_eq!(
            wide.buttons.len(),
            DIFF_BAR_BUTTONS.len() + 1,
            "terms={terms}"
        );
        let total: usize = DIFF_BAR_BUTTONS
            .iter()
            .map(|b| display_width(&format!(" {} ", b.bar_label())) + 1)
            .sum();
        // The maintainer chose "Côte à côte" over a shorter "Séparé", and "Vue
        // fichier" / "Vue diff" keep the File View button apart from the
        // column selector's "Fichier".
        assert!(
            total <= english_total + 24,
            "terms={terms}: French bar is {total} cells, English {english_total}"
        );
        // 100 columns: the pane is narrower than the window, but the bar
        // keeps its first controls and stays inside the width.
        let narrow = diff_bar(&state, None, false, 100);
        assert!(
            narrow.buttons.len() >= 9,
            "terms={terms}: {}",
            narrow.buttons.len()
        );
        for (x, w, _) in &narrow.buttons {
            assert!(x + w <= 100);
        }
        for b in DIFF_BAR_BUTTONS {
            let l = b.bar_label();
            assert!(!l.is_empty() && !l.contains("{$"), "{l:?}");
            let tip = b.tooltip();
            assert!(!tip.contains("{$"), "{tip:?}");
        }
    });
    let _l = override_lang(Lang::Fr);
    let _t = override_git_terms(false);
    assert_eq!(DiffBarButton::History.bar_label(), "Fichier \u{2502} Tout");
    assert_eq!(DiffBarButton::Blame.bar_label(), "Blame");
    assert!(DiffBarButton::Mode(DiffViewMode::Split)
        .bar_label()
        .ends_with("Côte à côte"));
}

#[test]
fn hunk_and_line_buttons_are_french_and_cut_the_code_not_the_label() {
    each_french(|terms| {
        // Git mode keeps the git words ("Stage du hunk"), the translated vocabulary indexes.
        let (stage, unstage) = if terms {
            ("Indexer le hunk", "Désindexer le hunk")
        } else {
            ("Stage du hunk", "Unstage du hunk")
        };
        assert_eq!(DiffSource::Unstaged.hunk_label(), Some(stage));
        assert_eq!(DiffSource::Staged.hunk_label(), Some(unstage));
        assert_eq!(interaction::discard_hunk_label(), "Abandonner le hunk");
        assert_eq!(interaction::line_discard_label(), "\u{2716} Abandonner");
        // The width of a button is its display width, accents included.
        let mut lines = vec![Line::from("x".repeat(10))];
        let placed =
            interaction::row_buttons(&mut lines, 0, 60, &[(unstage, th().ui.accent)], false, None)
                .expect("fits");
        assert_eq!(placed[0].1 as usize, display_width(unstage) + 2);
    });
}
