//! The Settings language dropdown in Simplified Chinese, Japanese and
//! Korean (the owner's report: names spaced out, the popup garbled over the
//! text behind it), at three terminal sizes over a background of CJK text.

use super::*;
use crate::cjk_sweep_tests::{cjk_background, each_cjk};
use crate::render_check as rc;
use ratatui::buffer::Buffer;

/// A state on the page that holds the Language row, focused on it.
fn language_state(dir: &Path) -> PreferencesState {
    let label = gitpoulpi_i18n::msg::settings::LANGUAGE.get().to_string();
    let mut state = state(dir);
    for (i, _) in PrefSection::ALL.iter().enumerate() {
        state.section = i;
        if let Some(f) = state.fields().iter().position(|f| f.label == label) {
            state.focus = PrefFocus::Fields;
            state.field = f;
            return state;
        }
    }
    panic!("no Language row");
}

fn paint(state: &mut PreferencesState, bg: &Buffer) -> Buffer {
    crate::cjk_sweep_tests::draw_over(bg, |frame| {
        let area = frame.area();
        state.render(frame, area);
    })
}

#[test]
fn the_language_dropdown_is_whole_in_every_cjk_language_and_size() {
    each_cjk(|lang, terms, w, h| {
        let what = format!("language dropdown [{} terms={terms} {w}x{h}]", lang.code());
        let dir = tempfile::tempdir().unwrap();
        let mut state = language_state(dir.path());
        let bg = cjk_background(w, h);
        paint(&mut state, &bg);
        state.on_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        assert!(state.choice.is_some(), "{what}: Enter opens the dropdown");
        let open = paint(&mut state, &bg);
        let combo = &state.choice.as_ref().unwrap().1;
        let popup = combo.popup_rect();
        assert!(popup.width > 0, "{what}: no popup");
        let mut all = rc::wide_violations(&open);
        all.extend(rc::box_violations_of(&open, popup));
        assert!(
            all.is_empty(),
            "{what}: {}\n{}",
            all.join("\n"),
            rc::dump(&open)
        );
        // The endonyms are contiguous: no cell between two CJK characters.
        let shown: Vec<String> = combo
            .item_rects()
            .iter()
            .map(|(r, _)| rc::row_text_in(&open, r.y, *r))
            .collect();
        for name in ["简体中文", "日本語", "한국어"] {
            if let Some(line) = shown
                .iter()
                .find(|l| l.contains(name.chars().next().unwrap()))
            {
                assert!(
                    line.contains(name),
                    "{what}: {name:?} is spaced out: {line:?}"
                );
            }
        }
        assert!(
            shown.iter().any(|l| l.contains("日本語")),
            "{what}: the Japanese entry is listed: {shown:?}\n{}",
            rc::dump(&open)
        );
    });
}

/// The same dropdown drawn straight over CJK text (what lies behind a popup
/// anywhere else): at every field column, so the left edge cuts a glyph on
/// half of them. Plain `Clear` leaves the cut glyph's half and the box is
/// open at the corner.
#[test]
fn the_language_combo_over_cjk_text_cuts_no_glyph() {
    use crate::combo::{Combo, ComboOption};
    each_cjk(|lang, terms, w, h| {
        for field_x in [3u16, 4, w / 2, w / 2 + 1] {
            let what = format!(
                "language combo [{} terms={terms} {w}x{h} field at {field_x}]",
                lang.code()
            );
            let mut combo = Combo::new("");
            combo.set_options(
                super::super::language_options(gitpoulpi_config::UiLanguage::Auto)
                    .into_iter()
                    .enumerate()
                    .map(|(i, l)| ComboOption::new(l, i.to_string()))
                    .collect(),
            );
            combo.open();
            let bg = cjk_background(w, h);
            let mut buf = bg.clone();
            let field = Rect::new(field_x, 3, 24.min(w - field_x), 1);
            combo.set_anchor(field);
            combo.render_popup(&mut buf, Rect::new(0, 0, w, h));
            let popup = combo.popup_rect();
            let mut all = rc::wide_violations(&buf);
            all.extend(rc::box_violations_of(&buf, popup));
            assert!(
                all.is_empty(),
                "{what}: {}\n{}",
                all.join("\n"),
                rc::dump(&buf)
            );
        }
    });
}
