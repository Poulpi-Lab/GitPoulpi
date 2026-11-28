//! The Settings as a flat list of settings, one per config key: what the
//! command palette lists and what a second interface offers. Built from the
//! row pool, never listed by hand.

use super::{Control, PoolCtx, Row, Target};
use gitpoulpi_config::split::{home_of, Home, Interface};
use gitpoulpi_config::Config;
use std::collections::HashSet;

/// How a setting is edited.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettingKind {
    /// Flipped.
    Toggle,
    /// One of these (the shown texts, in the order of the stored words).
    Choice(Vec<String>),
    /// A whole number in `min..=max`.
    Number { min: i64, max: i64 },
    /// Free text.
    Text,
    /// Needs its page (a combo, a button, a list, a number with a unit).
    Page,
}

/// One setting.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingRow {
    /// The config key (`ui.minimap`): the stable identity.
    pub key: &'static str,
    /// The page's own label, in the current language.
    pub label: String,
    pub kind: SettingKind,
}

/// Choices whose options are discovered at run time (installed terminals):
/// the page lists them, a fixed list cannot. "Become the primary" asks first
/// when another installation is primary, which only the page can show.
const PAGE_ONLY: &[&str] = &["ui.launch_terminal", "ui.language", "installation.primary"];

/// The settings of a row pool: one per key, in the pool's order, none for a
/// heading, an info line or a swatch strip. A row that is "coming" today
/// (undo depth with undo off) is still listed: running it says why it cannot
/// be edited now.
pub fn setting_rows(pool: &[Row]) -> Vec<SettingRow> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for field in pool {
        let Target::Global(key) = field.target else {
            continue;
        };
        if !seen.insert(key) {
            continue;
        }
        out.extend(setting_row(key, &field.label, &field.control, |raw| {
            field.option_label(raw)
        }));
    }
    out
}

/// The setting a row of key `key` is, `None` for a heading, an info line or
/// a swatch strip. `option_label` is the display text of a raw choice. A
/// frontend with a row type of its own calls this with the row's parts.
pub fn setting_row<A>(
    key: &'static str,
    label: &str,
    control: &Control<A>,
    option_label: impl Fn(&str) -> String,
) -> Option<SettingRow> {
    if control.is_decoration() {
        return None;
    }
    let kind = if PAGE_ONLY.contains(&key) {
        SettingKind::Page
    } else {
        match control {
            Control::Toggle(_) => SettingKind::Toggle,
            Control::Choice { options, .. } => {
                SettingKind::Choice(options.iter().map(|o| option_label(o)).collect())
            }
            Control::Number { min, max, .. } => SettingKind::Number {
                min: *min,
                max: *max,
            },
            Control::Text(_) => SettingKind::Text,
            _ => SettingKind::Page,
        }
    };
    Some(SettingRow {
        key,
        label: label.to_string(),
        kind,
    })
}

/// Every setting, labels in the current language. The list does not depend
/// on the user's file (a default configuration), so it never reads it.
pub fn rows() -> Vec<SettingRow> {
    let config = Config::default();
    let cx = PoolCtx {
        config: &config,
        own: &config,
        facts: &super::NoFacts,
    };
    setting_rows(&super::pool(&cx))
}

/// A setting as one interface offers it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InterfaceRow {
    pub row: SettingRow,
    /// Where the setting lives ([`gitpoulpi_config::split::KEY_HOME`]).
    pub home: Home,
    /// The key this interface reads and writes: the common key, or for a
    /// per-interface setting of the desktop the `desktop.ui.*` key.
    pub key: String,
}

/// The `desktop.toml` key of a per-interface `[ui]` key (`ui.minimap` ->
/// `desktop.ui.minimap`).
pub fn desktop_key(key: &str) -> String {
    format!("desktop.{key}")
}

/// Whether `interface` offers the setting `key`: everything for the terminal
/// UI; for the desktop what is common or per interface, minus what only
/// exists because of what a terminal is
/// ([`crate::surface::TERMINAL_ONLY_SETTINGS`], the keys homed in `tui.toml`).
pub fn offered_to(interface: Interface, key: &str) -> bool {
    match interface {
        Interface::Tui => true,
        Interface::Desktop => {
            home_of(key) != Home::Tui
                && !crate::surface::TERMINAL_ONLY_SETTINGS
                    .iter()
                    .any(|(k, _)| *k == key)
        }
    }
}

/// The settings `interface` shows, in the Settings order.
pub fn rows_for(interface: Interface) -> Vec<InterfaceRow> {
    rows()
        .into_iter()
        .filter(|r| offered_to(interface, r.key))
        .map(|row| {
            let home = home_of(row.key);
            let key = match (interface, home) {
                (Interface::Desktop, Home::PerInterface) => desktop_key(row.key),
                _ => row.key.to_string(),
            };
            InterfaceRow { row, home, key }
        })
        .collect()
}
