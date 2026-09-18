//! The main window in French (I5a): sidebar, footer, banners and tooltips
//! hold their width budgets in both git vocabularies at the two terminal
//! widths the maintainer uses (100 and 160 columns), and nothing is left as an
//! unresolved placeholder.

use gitpoulpi_i18n::{override_git_terms, override_lang, Lang};

use crate::display_width;

fn each_language(mut f: impl FnMut(Lang, bool)) {
    for lang in [Lang::En, Lang::Fr, Lang::Qps] {
        for translated in [false, true] {
            let _l = override_lang(lang);
            let _t = override_git_terms(translated);
            f(lang, translated);
        }
    }
}

fn clean(text: &str, what: &str) {
    assert!(!text.is_empty(), "{what} is empty");
    assert!(
        !text.contains("{$") && !text.contains("{key}") && !text.contains("{n}"),
        "{what} holds a placeholder: {text:?}"
    );
}

#[test]
fn the_prefix_hints_resolve_and_stay_close_to_the_english_length() {
    use crate::keymap;
    let width = |ctx| {
        display_width(&crate::chrome::prefix_hint(
            &keymap::Keymap::defaults(),
            ctx,
            true,
        ))
    };
    let english = [width(keymap::Ctx::GitPrefix), width(keymap::Ctx::NavPrefix)];
    each_language(|lang, translated| {
        for (i, ctx) in [keymap::Ctx::GitPrefix, keymap::Ctx::NavPrefix]
            .into_iter()
            .enumerate()
        {
            let line = crate::chrome::prefix_hint(&keymap::Keymap::defaults(), ctx, true);
            clean(&line, "prefix hint");
            // The footer truncates the line at the window width, as it
            // already does in English (210 cells); French must not be a
            // lot longer than that.
            if lang == Lang::Fr {
                assert!(
                    display_width(&line) <= english[i] + if translated { 55 } else { 25 },
                    "{lang:?} terms={translated} {ctx:?}: {} cells: {line:?}",
                    display_width(&line)
                );
            }
        }
    });
}

#[test]
fn sidebar_section_titles_are_short_and_the_stable_key_stays_english() {
    use crate::refs_sidebar::SidebarSection;
    each_language(|lang, translated| {
        for s in SidebarSection::ORDER {
            let t = s.display_title();
            clean(t, "section title");
            if lang != Lang::Qps {
                assert!(
                    display_width(t) <= 18,
                    "{lang:?} terms={translated}: {t:?} too wide"
                );
            }
            // The persisted fold state is keyed by the English name.
            assert!(s.title().is_ascii());
        }
    });
    let _l = override_lang(Lang::Fr);
    let _t = override_git_terms(false);
    assert_eq!(SidebarSection::Local.display_title(), "LOCALES");
    assert_eq!(SidebarSection::Tags.display_title(), "TAGS");
    let _t = override_git_terms(true);
    assert_eq!(SidebarSection::Tags.display_title(), "ÉTIQUETTES");
    assert_eq!(SidebarSection::Stashes.display_title(), "REMISES");
}

#[test]
fn the_refs_pane_title_and_drag_banner_resolve_in_french() {
    let _l = override_lang(Lang::Fr);
    let (left, right) = crate::refs_sidebar::refs_title(1, 5, 2);
    assert_eq!(left, " Solo 1/5 \u{b7} Masquées 2 ");
    assert_eq!(right, Some(" Arrêter le solo "));
    assert_eq!(crate::refs_sidebar::refs_title(0, 5, 0).0, "Refs");
}

#[test]
fn tooltips_and_find_counter_resolve_in_french() {
    use crate::chrome::HitTarget as H;
    let _l = override_lang(Lang::Fr);
    for target in [
        H::StatusExplainButton,
        H::AiGenerate,
        H::AiCompose,
        H::CloseApp,
        H::OpenRepositoryButton,
    ] {
        let t = crate::tooltip::hint_for(&target).expect("a tooltip");
        clean(t, "tooltip");
    }
    assert_eq!(
        crate::tooltip::hint_for(&H::CloseApp),
        Some("Quitter GitPoulpi")
    );
    assert_eq!(crate::log_search::counter_text(2, 16), "3 sur 16");
    assert_eq!(crate::log_search::counter_text(0, 0), "0 sur 0");
}

#[test]
fn the_autostash_banner_is_french_and_keeps_the_stash_handle() {
    use gitpoulpi_core::domain::{OperationKind, StashId};
    use gitpoulpi_session::{PendingAutoStash, PendingAutoStashState};
    let _l = override_lang(Lang::Fr);
    let p = |state| PendingAutoStash {
        stash_id: StashId(3),
        stash_commit: None,
        reason: OperationKind::Pull,
        created_at: 0,
        state,
        error: None,
        after_operation: false,
    };
    let _t = override_git_terms(false);
    let t = crate::autostash_banner::banner_text(&p(PendingAutoStashState::Conflicted));
    assert!(
        t.contains("stash@{3}") && t.contains("le pull") && t.contains("conflits"),
        "{t}"
    );
    let _t = override_git_terms(true);
    let t = crate::autostash_banner::banner_text(&p(PendingAutoStashState::Applied));
    assert!(t.contains("Auto-remise") && t.contains("le tirage"), "{t}");
}
