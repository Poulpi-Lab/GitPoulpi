//! The command palette: the shared catalogue with its shortcuts, fuzzy matching and availability
//! reasons, exactly as the terminal UI lists it, plus the three small pickers (theme, language, graph
//! colours) that the "Change ..." commands open.
//!
//! An entry the shared rules make unavailable is dimmed and carries its reason; an entry the desktop
//! does not implement yet is dimmed with "not available yet" and running it says so in the status line.

use std::rc::Rc;

use gitpoulpi_app::keymap;
use gitpoulpi_app::palette::{entry_shortcut, palette_matches, PaletteContext, BADGE_SUFFIX};
use gitpoulpi_core::fuzzy::{score, Anchor};
use gitpoulpi_i18n::msg;
use gpui_kit::{App, SharedString, Window};

use super::catalog::Command;
use super::context::{window_state, WindowState};
use super::ops::op_for;
use super::picker::{self, PickRow, Provider, Texts};
use super::{exec, status};
use crate::app::{set_lane_theme, set_language, set_theme};
use crate::strings;
use crate::theme;

/// The palette rows for `query` in the given window state, in the order the terminal UI lists them
/// (available before unavailable, best match first); with an empty query the commands the desktop
/// implements come first.
pub fn rows(query: &str, ctx: PaletteContext) -> Vec<PickRow> {
    let km = keymap::current();
    let mut rows: Vec<(bool, PickRow)> = palette_matches(query, ctx)
        .into_iter()
        .map(|m| {
            let command = Command::Palette(m.entry.command);
            let implemented = op_for(command).is_some();
            let reason: Option<SharedString> = if implemented {
                m.unavailable.map(Into::into)
            } else {
                Some(strings::NOT_AVAILABLE_YET.into())
            };
            let blocked = if implemented { m.unavailable } else { None };
            let shortcut = entry_shortcut(&km, m.entry).replace(BADGE_SUFFIX, "");
            let row = PickRow {
                label: m.entry.display_label().into_owned().into(),
                shortcut: shortcut.into(),
                dim: reason.is_some(),
                reason,
                run: Rc::new(move |window, cx| match blocked {
                    Some(why) => status::set(cx, why),
                    None => {
                        exec::run(command, window, cx);
                    }
                }),
            };
            (implemented, row)
        })
        .collect();
    if query.trim().is_empty() {
        // Stable: the catalogue order is kept inside each group.
        rows.sort_by_key(|(implemented, _)| !implemented);
    }
    rows.into_iter().map(|(_, row)| row).collect()
}

fn palette_texts() -> Texts {
    Texts {
        title: Rc::new(|shown, total| msg::palette::title(shown as u64, total as u64)),
        placeholder: msg::palette::HINT.get().into(),
        empty: msg::palette::NO_MATCH.get().into(),
    }
}

/// Opens the command palette.
pub fn open(window: &mut Window, cx: &mut App) {
    let state = window_state(window, cx);
    open_with_state(state, window, cx);
}

/// Opens the command palette for a given window state.
pub fn open_with_state(
    state: WindowState,
    window: &mut Window,
    cx: &mut App,
) -> gpui_kit::Entity<picker::Picker> {
    let provider: Provider = Rc::new(move |query, _| rows(query, state.palette_context()));
    picker::open(window, cx, palette_texts(), provider)
}

/// Rows filtered and ranked by `query` over their labels.
fn filtered(mut rows: Vec<PickRow>, query: &str) -> Vec<PickRow> {
    let q = query.trim().to_lowercase();
    if q.is_empty() {
        return rows;
    }
    let mut scored: Vec<(i32, PickRow)> = rows
        .drain(..)
        .filter_map(|r| score(&r.label, &q, Anchor::Whole).map(|s| (s, r)))
        .collect();
    scored.sort_by_key(|(s, _)| -*s);
    scored.into_iter().map(|(_, r)| r).collect()
}

fn simple_texts(title: &str, placeholder: &str) -> Texts {
    let title = title.to_string();
    Texts {
        title: Rc::new(move |_, _| title.clone()),
        placeholder: placeholder.to_string().into(),
        empty: msg::palette::NO_MATCH.get().into(),
    }
}

fn pick(label: String, shortcut: String, run: impl Fn(&mut App) + 'static) -> PickRow {
    PickRow {
        label: label.into(),
        shortcut: shortcut.into(),
        reason: None,
        dim: false,
        run: Rc::new(move |_, cx| run(cx)),
    }
}

/// Every theme, the current one marked.
pub fn theme_rows(cx: &App) -> Vec<PickRow> {
    let current = cx
        .try_global::<theme::CurrentTheme>()
        .map(|c| c.0.clone())
        .unwrap_or_default();
    theme::all_presets()
        .into_iter()
        .map(|p| {
            let slug = p.slug.clone();
            let mark = if p.slug == current { "*" } else { "" };
            pick(p.name.clone(), format!("{mark} {}", p.slug), move |cx| {
                set_theme(&slug, cx)
            })
        })
        .collect()
}

/// Every shipped language, the current one marked.
pub fn language_rows() -> Vec<PickRow> {
    let current = gitpoulpi_i18n::current();
    gitpoulpi_i18n::SHIPPED
        .iter()
        .copied()
        .map(|l| {
            let mark = if l == current { "*" } else { "" };
            pick(
                l.native_name().to_string(),
                format!("{mark} {}", l.code()),
                move |cx| set_language(l, cx),
            )
        })
        .collect()
}

/// "Follow the theme", then every graph colour palette.
pub fn lane_rows() -> Vec<PickRow> {
    let mut rows = vec![pick(
        msg::settings_ui::THEME.get().to_string(),
        String::new(),
        |cx| set_lane_theme("theme", cx),
    )];
    for p in gitpoulpi_config::theme::lane_palettes() {
        let slug = p.slug.clone();
        rows.push(pick(p.label.clone(), p.slug.clone(), move |cx| {
            set_lane_theme(&slug, cx)
        }));
    }
    rows
}

pub fn open_theme_picker(window: &mut Window, cx: &mut App) {
    let title = msg::settings_ui::THEME.get();
    let provider: Provider = Rc::new(|query, cx| filtered(theme_rows(cx), query));
    picker::open(window, cx, simple_texts(title, title), provider);
}

pub fn open_language_picker(window: &mut Window, cx: &mut App) {
    let title = msg::settings::LANGUAGE.get();
    let provider: Provider = Rc::new(|query, _| filtered(language_rows(), query));
    picker::open(window, cx, simple_texts(title, title), provider);
}

pub fn open_lane_picker(window: &mut Window, cx: &mut App) {
    let title = msg::footer_hint::GRAPH_COLORS.get();
    let provider: Provider = Rc::new(|query, _| filtered(lane_rows(), query));
    picker::open(window, cx, simple_texts(title, title), provider);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_empty_query_lists_the_implemented_commands_first() {
        let rows = rows("", WindowState::default().palette_context());
        assert!(rows.len() > 500, "the whole catalogue is listed");
        let first_dim = rows.iter().position(|r| r.dim).expect("some are dim");
        assert!(first_dim >= 5, "implemented commands lead: {first_dim}");
        assert!(rows[..first_dim].iter().all(|r| !r.dim));
        assert!(rows[first_dim..]
            .iter()
            .any(|r| r.reason.as_deref() == Some(strings::NOT_AVAILABLE_YET)));
    }

    #[test]
    fn a_query_finds_a_command_and_shows_its_shortcut() {
        let rows = rows("quit", WindowState::default().palette_context());
        let quit = rows
            .iter()
            .find(|r| r.label.contains("Quit"))
            .expect("Quit is listed");
        assert_eq!(quit.shortcut.as_ref(), "q");
        assert!(!quit.dim);
    }

    #[test]
    fn a_command_not_implemented_yet_says_so() {
        let rows = rows("fetch all", WindowState::default().palette_context());
        let row = rows.first().expect("a match");
        assert!(row.dim);
        assert_eq!(row.reason.as_deref(), Some(strings::NOT_AVAILABLE_YET));
    }

    #[test]
    fn pickers_list_their_choices() {
        assert!(language_rows().len() >= 2);
        assert!(lane_rows().len() >= 2);
        let rows = filtered(language_rows(), "fran");
        assert!(rows.iter().any(|r| r.label.contains("Fran")));
    }
}
