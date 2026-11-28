//! The parity table: every action, palette command and generated command
//! family the application offers, in one list a frontend (or a generator)
//! can walk to check it covers all of them.
//!
//! Built with every feature on: a row is listed whether or not this binary
//! was built with its feature (`feature` says which one it needs). Nothing
//! here reads the user's keymap or config.

use crate::features::Feature;
use crate::keymap::action::STATIC_ACTIONS;
use crate::palette::bindings::static_bind_action;
use crate::palette::{command_id, command_requires, PALETTE_COMMANDS};
use crate::surface::{Surface, TERMINAL_ONLY_SETTINGS};
use gitpoulpi_i18n::{with_lang, Lang};

/// What kind of entry a row is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParityKind {
    /// A bindable key action of the registry.
    Action,
    /// A command of the palette catalogue.
    Palette,
    /// A family of generated palette rows (one id stands for all of them),
    /// or one generated row that needs a row of its own.
    Family,
}

/// One row of the parity table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParityRow {
    pub kind: ParityKind,
    /// The stable id: an action's `config_id`, `palette.<command id>`, or a
    /// family id (`settings.*`, `flow.start.*`, `jobs.*`, `sequence.*`, and
    /// `settings.<key>` for a generated Settings row that is terminal-only).
    pub id: String,
    /// The context an action is bound in (`Ctx::name`); `palette` for a
    /// palette row; the family's name for a family.
    pub ctx: &'static str,
    /// The label in English, whatever the UI language.
    pub label_en: String,
    /// The shipped keys (an action's defaults; a palette row's catalogue
    /// shortcut text).
    pub default_keys: Vec<String>,
    /// The cargo feature the entry needs, if any.
    pub feature: Option<Feature>,
    pub surface: Surface,
    /// For a palette row: the `config_id` of the key action it is the twin of.
    /// The generator merges such a row into that action's row.
    pub bound_action: Option<String>,
}

fn english(label: gitpoulpi_i18n::M) -> String {
    with_lang(Lang::En, || label.get().to_string())
}

/// Every row, actions first, then the palette catalogue, then the families.
pub fn parity_rows() -> Vec<ParityRow> {
    let mut rows = Vec::new();

    for def in STATIC_ACTIONS {
        rows.push(ParityRow {
            kind: ParityKind::Action,
            id: def.static_id.to_string(),
            ctx: def.ctx.name(),
            label_en: english(def.static_label),
            default_keys: def.defaults.iter().map(|k| (*k).to_string()).collect(),
            feature: def.requires,
            surface: def.surface,
            bound_action: None,
        });
    }

    for entry in PALETTE_COMMANDS {
        let bound = static_bind_action(entry).and_then(|action| {
            STATIC_ACTIONS
                .iter()
                .find(|d| d.action == action)
                .map(|d| d.static_id.to_string())
        });
        rows.push(ParityRow {
            kind: ParityKind::Palette,
            id: format!("palette.{}", command_id(&entry.command)),
            ctx: "palette",
            label_en: entry.english().to_string(),
            default_keys: entry
                .shortcut
                .split(" / ")
                .map(str::trim)
                .filter(|k| !k.is_empty())
                .map(str::to_string)
                .collect(),
            feature: entry.requires.or(command_requires(&entry.command)),
            surface: entry.surface,
            bound_action: bound,
        });
    }

    let family =
        |id: &str, ctx: &'static str, label: &str, feature: Option<Feature>, surface| ParityRow {
            kind: ParityKind::Family,
            id: id.to_string(),
            ctx,
            label_en: label.to_string(),
            default_keys: Vec::new(),
            feature,
            surface,
            bound_action: None,
        };
    rows.push(family(
        "settings.*",
        "settings",
        "Settings: <setting>",
        None,
        Surface::Any,
    ));
    for (key, reason) in TERMINAL_ONLY_SETTINGS {
        rows.push(family(
            &format!("settings.{key}"),
            "settings",
            &format!("Settings: {}", key.replace(['.', '_'], " ")),
            None,
            Surface::TerminalOnly(reason),
        ));
    }
    rows.push(family(
        "flow.start.*",
        "flow",
        "Flow: Start <type>",
        None,
        Surface::Any,
    ));
    rows.push(family(
        "flow.new_issue.*",
        "flow",
        "Flow: New issue <type>",
        Some(Feature::Hosting),
        Surface::Any,
    ));
    rows.push(family(
        "jobs.*",
        "jobs",
        "Jobs: Cancel <job>",
        None,
        Surface::Any,
    ));
    rows.push(family(
        "sequence.*",
        "sequence",
        "Sequence: <name>",
        None,
        Surface::Any,
    ));
    rows
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    /// The MCP palette entry exists only in builds with the `mcp` feature.
    const PINNED_ROWS: usize = 1194 + cfg!(feature = "mcp") as usize;

    #[test]
    fn ids_are_unique() {
        let rows = parity_rows();
        let mut seen = HashSet::new();
        for row in &rows {
            assert!(seen.insert(row.id.clone()), "duplicate id {}", row.id);
        }
    }

    /// The table lists the same rows as before the Settings rows came from the
    /// shared model: its size and the Settings family rows (the generic one,
    /// then the terminal-only settings in `TERMINAL_ONLY_SETTINGS` order).
    #[test]
    fn the_table_and_its_settings_rows_are_pinned() {
        let rows = parity_rows();
        assert_eq!(rows.len(), PINNED_ROWS);
        let settings: Vec<(&str, bool)> = rows
            .iter()
            .filter(|r| r.id.starts_with("settings."))
            .map(|r| (r.id.as_str(), r.surface.terminal_only_reason().is_some()))
            .collect();
        assert_eq!(
            settings,
            [
                ("settings.*", false),
                ("settings.ui.color_depth", true),
                ("settings.ui.kitty_keyboard", true),
                ("settings.ui.image_protocol", true),
                ("settings.ui.image_view_text_fallback", true),
                ("settings.ui.text_cursor", true),
                ("settings.ui.mouse_enabled", true),
                ("settings.ui.launch_terminal", true),
                ("settings.ui.relaunch_in_terminal", true),
            ]
        );
    }

    #[test]
    fn every_registry_item_has_a_row() {
        let rows = parity_rows();
        let ids: HashSet<&str> = rows.iter().map(|r| r.id.as_str()).collect();
        for def in STATIC_ACTIONS {
            assert!(ids.contains(def.static_id), "no row for {}", def.static_id);
        }
        for entry in PALETTE_COMMANDS {
            let id = format!("palette.{}", command_id(&entry.command));
            assert!(ids.contains(id.as_str()), "no row for {id}");
        }
        for family in ["settings.*", "flow.start.*", "jobs.*", "sequence.*"] {
            assert!(ids.contains(family), "no family row {family}");
        }
    }

    #[test]
    fn a_bound_palette_row_names_an_action_row() {
        let rows = parity_rows();
        let actions: HashSet<&str> = rows
            .iter()
            .filter(|r| r.kind == ParityKind::Action)
            .map(|r| r.id.as_str())
            .collect();
        let bound: Vec<_> = rows
            .iter()
            .filter(|r| r.kind == ParityKind::Palette)
            .filter_map(|r| r.bound_action.as_deref())
            .collect();
        assert!(bound.len() > 20, "the catalogue binds many actions");
        for id in bound {
            assert!(actions.contains(id), "{id} is not an action row");
        }
        let quit = rows
            .iter()
            .find(|r| r.id == "palette.quit")
            .expect("palette.quit");
        assert_eq!(quit.bound_action.as_deref(), Some("global.quit"));
    }

    #[test]
    fn action_rows_carry_their_defaults_context_and_feature() {
        let rows = parity_rows();
        let palette = rows
            .iter()
            .find(|r| r.id == "global.palette")
            .expect("global.palette");
        assert_eq!(palette.kind, ParityKind::Action);
        assert_eq!(palette.ctx, "global");
        assert_eq!(palette.default_keys, ["Ctrl+P"]);
        assert!(!palette.label_en.is_empty());
        assert!(
            rows.iter()
                .any(|r| r.kind == ParityKind::Action && r.feature.is_some()),
            "some action needs a feature"
        );
    }

    #[test]
    fn labels_are_english_whatever_the_ui_language() {
        let before = parity_rows();
        let lang = gitpoulpi_i18n::Lang::ALL
            .iter()
            .copied()
            .find(|l| *l != Lang::En && l.is_built());
        let Some(lang) = lang else { return };
        let after = with_lang(lang, parity_rows);
        assert_eq!(before, after);
    }

    #[test]
    fn terminal_only_rows_have_a_reason() {
        for row in parity_rows() {
            if let Surface::TerminalOnly(reason) = row.surface {
                assert!(!reason.trim().is_empty(), "{} has no reason", row.id);
            }
        }
        for (key, reason) in TERMINAL_ONLY_SETTINGS {
            assert!(key.starts_with("ui."), "{key}");
            assert!(!reason.trim().is_empty(), "{key} has no reason");
        }
    }

    #[test]
    fn the_terminal_only_settings_are_listed_as_family_rows() {
        let rows = parity_rows();
        for key in [
            "ui.color_depth",
            "ui.kitty_keyboard",
            "ui.image_protocol",
            "ui.image_view_text_fallback",
            "ui.text_cursor",
            "ui.mouse_enabled",
            "ui.launch_terminal",
            "ui.relaunch_in_terminal",
        ] {
            let id = format!("settings.{key}");
            let row = rows
                .iter()
                .find(|r| r.id == id)
                .unwrap_or_else(|| panic!("no row {id}"));
            assert!(matches!(row.surface, Surface::TerminalOnly(_)), "{id}");
            assert_eq!(row.kind, ParityKind::Family);
        }
    }

    #[test]
    fn nothing_but_the_settings_is_terminal_only_today() {
        for row in parity_rows() {
            if row.surface != Surface::Any {
                assert!(
                    row.id.starts_with("settings."),
                    "{} is terminal-only: declare why in the parity plan first",
                    row.id
                );
            }
        }
    }
}
