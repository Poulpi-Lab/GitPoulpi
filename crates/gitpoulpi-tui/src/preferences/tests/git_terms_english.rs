//! The "English git words" checkbox: offered only for ja, ko and zh-CN, next to
//! "Translate git terms", disabled while terms are translated, and it switches
//! the rendered git words.

use super::*;
use gitpoulpi_i18n::{
    msg::footer_hint::STAGE, override_english_git_terms, override_git_terms, override_lang, Lang,
};

const KEY: &str = "ui.git_terms_english";

/// The Settings row for `KEY` (and its section), if any page offers it.
fn row(state: &mut PreferencesState) -> Option<(usize, Field)> {
    for section in 0..PrefSection::ALL.len() {
        state.section = section;
        if let Some((i, f)) = state
            .fields()
            .into_iter()
            .enumerate()
            .find(|(_, f)| matches!(f.target, Target::Global(k) if k == KEY))
        {
            return Some((i, f));
        }
    }
    None
}

#[test]
fn the_checkbox_is_offered_only_in_ja_ko_zh_cn_right_after_translate_git_terms() {
    for (lang, offered) in [
        (Lang::Ja, true),
        (Lang::Ko, true),
        (Lang::ZhCn, true),
        (Lang::En, false),
        (Lang::Fr, false),
        (Lang::De, false),
    ] {
        let _l = override_lang(lang);
        let dir = tempfile::tempdir().unwrap();
        let mut state = super::state(dir.path());
        let found = row(&mut state);
        assert_eq!(found.is_some(), offered, "{}", lang.code());
        if let Some((i, field)) = found {
            assert!(matches!(field.control, Control::Toggle(false)));
            assert!(field.coming.is_none());
            let before = &state.fields()[i - 1];
            assert!(
                matches!(before.target, Target::Global("ui.translate_git_terms")),
                "{}",
                lang.code()
            );
            // Drawn without panicking (wide glyphs are spaced in the buffer,
            // so the label is not searched in the screen text).
            let _ = draw_screen(&mut state);
        }
    }
}

#[test]
fn the_checkbox_is_disabled_while_git_terms_are_translated() {
    let _l = override_lang(Lang::Ja);
    let dir = tempfile::tempdir().unwrap();
    let mut state = super::state(dir.path());
    state.config.ui.translate_git_terms = true;
    let (_, field) = row(&mut state).expect("offered in ja");
    assert!(field.coming.is_some());
    assert!(!field.selectable());
    state.config.ui.translate_git_terms = false;
    state.config.ui.git_terms_english = true;
    let (_, field) = row(&mut state).expect("offered in ja");
    assert!(matches!(field.control, Control::Toggle(true)));
    assert!(field.selectable());
}

#[test]
fn the_option_switches_the_rendered_git_words() {
    for (lang, git_word, translated) in [
        (Lang::Ja, "ステージ", "ステージ"),
        (Lang::Ko, "스테이징", "스테이징"),
        (Lang::ZhCn, "stage", "暂存"),
    ] {
        let _l = override_lang(lang);
        let _t = override_git_terms(false);
        let off = override_english_git_terms(false);
        let normal = STAGE.get();
        assert!(normal.contains(git_word), "{}: {normal}", lang.code());
        drop(off);
        let _on = override_english_git_terms(true);
        assert_eq!(STAGE.get(), "stage", "{}", lang.code());
        // The translated vocabulary wins over the option.
        let _t = override_git_terms(true);
        assert!(STAGE.get().contains(translated), "{}", lang.code());
    }
}

#[test]
fn every_catalog_has_the_checkbox_strings() {
    use gitpoulpi_i18n::msg::settings as m;
    for &lang in gitpoulpi_i18n::SHIPPED {
        let _l = override_lang(lang);
        for s in [
            m::GIT_TERMS_ENGLISH.get(),
            m::GIT_TERMS_ENGLISH_HELP.get(),
            m::GIT_TERMS_ENGLISH_OFF.get(),
        ] {
            assert!(!s.is_empty());
            if lang != Lang::En {
                assert_ne!(s, m::GIT_TERMS_ENGLISH.english(), "{}", lang.code());
            }
        }
    }
}
