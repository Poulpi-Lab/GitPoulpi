//! The ref menu in French: every row fits the drawn menu (the width comes
//! from the label's display width, accents and « » included), the action
//! verb is never cut, and "Translate git terms" switches the vocabulary.

use super::*;
use gitpoulpi_i18n::{override_git_terms, override_lang, Lang};
use ratatui::backend::TestBackend;
use ratatui::Terminal;

/// Draws `menu` on a roomy screen and returns the screen lines with the
/// menu's own row labels.
fn drawn<A: Clone>(menu: &ContextMenu<A>) -> (Vec<String>, Vec<String>) {
    let mut terminal = Terminal::new(TestBackend::new(110, 40)).unwrap();
    terminal.draw(|f| drop(menu.render(f, None))).unwrap();
    let buf = terminal.backend().buffer().clone();
    let lines = (0..buf.area.height)
        .map(|y| (0..buf.area.width).map(|x| buf[(x, y)].symbol()).collect())
        .collect();
    let labels = (0..menu.row_count()).map(|r| menu.row_label(r)).collect();
    (lines, labels)
}

fn local_menu() -> ContextMenu<RefContextAction> {
    let ctx = MenuContext {
        current_branch: Some("main".into()),
        remote_twin: Some("origin/feature".into()),
        ..MenuContext::default()
    };
    let entries = ref_menu_entries(SidebarSection::Local, "feature", &ctx, false, false, false);
    ContextMenu::new("feature", entries, 2, 1)
}

/// Every row's label is on screen in full, and every menu line closes with
/// the border: a wider French label widens the menu instead of being cut.
fn assert_fits(menu: &ContextMenu<RefContextAction>) {
    let (lines, labels) = drawn(menu);
    for label in labels.iter().filter(|l| !l.is_empty()) {
        let line = lines
            .iter()
            .find(|l| l.contains(label.as_str()))
            .unwrap_or_else(|| panic!("row {label:?} is cut or missing:\n{}", lines.join("\n")));
        assert!(line.trim_end().ends_with('\u{2502}'), "{line:?}");
    }
}

#[test]
#[cfg(feature = "hosting")]
fn the_french_ref_menu_fits_and_keeps_git_words_in_english_by_default() {
    let _l = override_lang(Lang::Fr);
    let menu = local_menu();
    assert_fits(&menu);
    let (lines, _) = drawn(&menu);
    let screen = lines.join("\n");
    for expected in [
        "Checkout feature",
        "Merge feature dans main",
        "Rebase main sur feature",
        "Fast-forward de feature vers main",
        "Créer un worktree depuis feature...",
        "Push feature",
        "Démarrer une pull request depuis feature...",
        "Définir l'upstream de feature...",
        "Renommer feature...",
        "Supprimer feature et origin/feature",
        "Masquer dans le graphe",
        "Copier",
    ] {
        assert!(screen.contains(expected), "missing {expected:?}:\n{screen}");
    }
}

#[test]
fn translate_git_terms_switches_the_ref_menu_to_the_local_vocabulary() {
    let _l = override_lang(Lang::Fr);
    let _t = override_git_terms(true);
    let menu = local_menu();
    assert_fits(&menu);
    let (lines, _) = drawn(&menu);
    let screen = lines.join("\n");
    for expected in [
        "Basculer sur feature",
        "Fusionner feature dans main",
        "Rebaser main sur feature",
        "Avance rapide de feature vers main",
        "Pousser feature",
        "Définir l'amont de feature...",
    ] {
        assert!(screen.contains(expected), "missing {expected:?}:\n{screen}");
    }
    if crate::build_features::WORKTREE {
        assert!(
            screen.contains("Créer un arbre de travail depuis feature..."),
            "{screen}"
        );
    }
    assert!(!screen.contains("Checkout"), "{screen}");
}

#[test]
fn the_french_tag_stash_and_folder_menus_fit() {
    let _l = override_lang(Lang::Fr);
    for vocabulary in [false, true] {
        let _t = override_git_terms(vocabulary);
        let ctx = MenuContext::default();
        for section in [
            SidebarSection::Tags,
            SidebarSection::Stashes,
            SidebarSection::Submodules,
            SidebarSection::Remote,
        ] {
            let entries = ref_menu_entries(section, "v1.0", &ctx, false, false, true);
            assert_fits(&ContextMenu::new("v1.0", entries, 2, 1));
        }
        assert_fits(&ContextMenu::new(
            "origin",
            folder_menu_entries(false, true),
            2,
            1,
        ));
    }
}

#[test]
fn the_menu_width_follows_the_display_width_of_the_translated_label() {
    let _l = override_lang(Lang::Fr);
    let menu = local_menu();
    let mut terminal = Terminal::new(TestBackend::new(110, 40)).unwrap();
    let mut rects = Vec::new();
    terminal.draw(|f| rects = menu.render(f, None)).unwrap();
    for (r, rect) in rects.iter().enumerate() {
        let label = menu.row_label(r);
        assert!(
            usize::from(rect.width) >= crate::display_width(&label),
            "{label:?} is wider than its row ({})",
            rect.width
        );
    }
}
