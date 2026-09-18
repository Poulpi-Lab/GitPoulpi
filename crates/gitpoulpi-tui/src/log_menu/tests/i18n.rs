//! The commit and WIP menus in French (and with "Translate git terms" on):
//! they fit the drawn menu and keep the action verb whole. Also the
//! guard of the ref-pill menu's text hints: they pick grayed rows out by a
//! piece of their label, so each must be a part of that label everywhere.

use super::*;
use gitpoulpi_i18n::{msg, override_git_terms, override_lang, Lang};
use ratatui::backend::TestBackend;
use ratatui::Terminal;

fn assert_fits<A: Clone>(menu: &ContextMenu<A>) -> String {
    let mut terminal = Terminal::new(TestBackend::new(110, 40)).unwrap();
    terminal.draw(|f| drop(menu.render(f, None))).unwrap();
    let buf = terminal.backend().buffer().clone();
    let lines: Vec<String> = (0..buf.area.height)
        .map(|y| (0..buf.area.width).map(|x| buf[(x, y)].symbol()).collect())
        .collect();
    for r in 0..menu.row_count() {
        let label = menu.row_label(r);
        if label.is_empty() {
            continue;
        }
        let line = lines
            .iter()
            .find(|l| l.contains(label.as_str()))
            .unwrap_or_else(|| panic!("row {label:?} is cut:\n{}", lines.join("\n")));
        assert!(line.trim_end().ends_with('\u{2502}'), "{line:?}");
    }
    lines.join("\n")
}

fn commit_menu(merge: bool, ai: bool) -> ContextMenu<LogContextAction> {
    let ctx = MenuContext {
        current_branch: Some("main".into()),
        ..MenuContext::default()
    }
    .with_ai(ai);
    ContextMenu::new("a1b2c3d", commit_menu_entries(&ctx, merge), 2, 1)
}

#[test]
fn the_french_commit_menu_fits_and_keeps_the_verbs() {
    let _l = override_lang(Lang::Fr);
    for merge in [false, true] {
        assert_fits(&commit_menu(merge, true));
    }
    let screen = assert_fits(&commit_menu(false, true));
    for expected in [
        "Cherry-pick sur main",
        "Revert du commit...",
        "Reset de main sur ce commit",
        "Modifier le message du commit...",
        "Rebase interactif depuis ici...",
        "Créer une branche ici...",
        "Créer un tag ici...",
        "Checkout de ce commit",
        "Créer un patch d'ici à main...",
        "Copier le lien du commit sur le distant",
        "Expliquer les modifications (IA)",
    ] {
        assert!(screen.contains(expected), "missing {expected:?}:\n{screen}");
    }
}

#[test]
fn translate_git_terms_switches_the_commit_menu() {
    let _l = override_lang(Lang::Fr);
    let _t = override_git_terms(true);
    let screen = assert_fits(&commit_menu(false, false));
    for expected in [
        "Cherry-pick sur main",
        "Inverser la validation...",
        "Réinitialiser main sur cette validation",
        "Modifier le message de validation...",
        "Rebaser interactivement depuis ici...",
        "Créer une étiquette ici...",
        "Basculer sur cette validation",
    ] {
        assert!(screen.contains(expected), "missing {expected:?}:\n{screen}");
    }
}

#[test]
fn the_french_wip_and_stash_menus_fit() {
    let _l = override_lang(Lang::Fr);
    for vocabulary in [false, true] {
        let _t = override_git_terms(vocabulary);
        assert_fits(&ContextMenu::new("WIP", wip_menu_entries(true, true), 2, 1));
        assert_fits(&ContextMenu::new(
            "WIP",
            wip_menu_entries(false, false),
            2,
            1,
        ));
    }
}

#[test]
fn the_reset_submenu_fits_in_french() {
    let _l = override_lang(Lang::Fr);
    let mut menu = commit_menu(false, false);
    let rows = menu.row_count();
    let reset = (0..rows)
        .find(|&r| menu.row_label(r).contains("Reset"))
        .expect("the Reset submenu row");
    menu.activate(reset);
    let screen = assert_fits(&menu);
    assert!(
        screen.contains("Soft") && screen.contains("Hard..."),
        "{screen}"
    );
}

/// The pill menu finds grayed rows and submenus by a piece of their label.
/// Those pieces are catalog messages; they must stay inside the labels they
/// stand for, in every shipped language and both vocabularies.
#[test]
fn the_pill_menu_hints_are_part_of_the_labels_they_find() {
    for lang in gitpoulpi_i18n::SHIPPED {
        for vocabulary in [false, true] {
            let _l = override_lang(*lang);
            let _t = override_git_terms(vocabulary);
            let pairs: [(String, &str); 7] = [
                (
                    msg::menu_ref::fast_forward("a", "b"),
                    msg::menu_pill::HINT_FAST_FORWARD.get(),
                ),
                (
                    msg::menu_log::REBASE_HERE.get().to_string(),
                    msg::menu_pill::HINT_REBASE_HERE.get(),
                ),
                (
                    msg::menu_ref::start_pr("a"),
                    msg::menu_pill::HINT_START_PR.get(),
                ),
                (
                    msg::menu_log::PATCH_FROM_COMMIT.get().to_string(),
                    msg::menu_pill::HINT_PATCH_COMMIT.get(),
                ),
                (
                    msg::menu_log::patch_to_head("HEAD"),
                    msg::menu_pill::HINT_PATCH_RANGE.get(),
                ),
                (
                    msg::menu_log::reset_to("HEAD"),
                    msg::menu_pill::HINT_RESET.get(),
                ),
                (
                    msg::menu_ref::COPY.get().to_string(),
                    msg::menu_pill::HINT_COPY.get(),
                ),
            ];
            for (label, hint) in pairs {
                assert!(
                    label.contains(hint),
                    "{lang:?} (translated git terms: {vocabulary}): hint {hint:?} not in {label:?}"
                );
            }
        }
    }
}
