//! `Settings: ...` rows: the settings as command-palette entries.
//!
//! Generated from the row pool ([`super::rows`]), never listed by hand. One
//! row per setting, by what it is:
//! - a toggle: `Settings: Toggle <label>`, runs at once;
//! - a number or text: `Settings: <label>...`, the Settings page opens on the
//!   row with it being edited;
//! - a choice: `Settings: <label>...` narrows the palette to its choices
//!   (`Settings: <label>: <choice>` rows, the current one marked);
//! - anything that needs its page: `Settings: <label>...` opens Settings on it.
//!
//! The rows are built once per language (their text is leaked: they are
//! `&'static PaletteEntry` like the catalogue's). The number of a row (the
//! `u16` in [`SettingOp`]) is its place in [`rows`]. The values shown beside
//! them are the frontend's to read (`Families::settings_value_text`).

use super::rows::{rows, SettingKind, SettingRow};
use crate::palette::{PaletteCommand, PaletteEntry, SettingOp};
use gitpoulpi_i18n::msg;
use std::sync::Mutex;

struct Model {
    rows: Vec<SettingRow>,
    entries: Vec<PaletteEntry>,
    /// Per row: the lower-case text a query must start with for its
    /// `Choose` rows to be listed (`settings: log level:`).
    prefixes: Vec<String>,
}

fn leak(text: String) -> &'static str {
    Box::leak(text.into_boxed_str())
}

fn build() -> Model {
    let rows = rows();
    let mut entries = Vec::new();
    let mut prefixes = Vec::new();
    for (r, row) in rows.iter().enumerate() {
        let r = r as u16;
        // The English side of a row is its config key in words (`ui minimap`):
        // a query typed in English finds a row whatever the UI language, and
        // the key is the one stable name a setting has. (The page labels
        // exist only in the current language.)
        let words = row.key.replace(['.', '_'], " ");
        let mut push = |op: SettingOp, now: String, english: String| {
            entries.push(PaletteEntry {
                label: msg::palette::SETTINGS_CURRENT,
                shortcut: "",
                hint: None,
                command: PaletteCommand::Setting(op),
                requires: None,
                text: Some(leak(now)),
                text_en: Some(leak(english)),
                surface: crate::surface::settings_surface(row.key),
            });
        };
        let (op, now) = match &row.kind {
            SettingKind::Toggle => (
                SettingOp::Toggle(r),
                msg::palette::settings_toggle(&row.label),
            ),
            SettingKind::Choice(_) => (SettingOp::Sub(r), msg::palette::settings_open(&row.label)),
            SettingKind::Number { .. } | SettingKind::Text => {
                (SettingOp::Edit(r), msg::palette::settings_open(&row.label))
            }
            SettingKind::Page => (SettingOp::Page(r), msg::palette::settings_open(&row.label)),
        };
        push(op, now, format!("Settings: {words}"));
        let mut prefix = String::new();
        if let SettingKind::Choice(options) = &row.kind {
            prefix = msg::palette::settings_choice(&row.label, "")
                .trim_end()
                .to_lowercase();
            for (o, option) in options.iter().enumerate() {
                push(
                    SettingOp::Choose(r, o as u16),
                    msg::palette::settings_choice(&row.label, option),
                    format!("Settings: {words}: {option}"),
                );
            }
        }
        prefixes.push(prefix);
    }
    Model {
        rows,
        entries,
        prefixes,
    }
}

type ModelKey = (u64, u8, bool);
/// One model per key, kept: a single slot thrashed when threads with
/// different languages (the tests' overrides) took turns, rebuilding the
/// whole model and leaking the old one on every call. Built once per
/// language epoch, so the leak is bounded by the language switches.
static MODEL: Mutex<Vec<(ModelKey, &'static Model)>> = Mutex::new(Vec::new());

fn model() -> &'static Model {
    let key = (
        gitpoulpi_i18n::epoch(),
        gitpoulpi_i18n::current() as u8,
        gitpoulpi_i18n::git_terms_translated(),
    );
    let mut models = MODEL.lock().unwrap_or_else(|e| e.into_inner());
    if let Some((_, model)) = models.iter().find(|(k, _)| *k == key) {
        return model;
    }
    let model: &'static Model = Box::leak(Box::new(build()));
    models.push((key, model));
    model
}

/// How many rows the palette offers when nothing is typed (the choices of a
/// setting only show once it is opened).
pub fn listed_count() -> usize {
    entries()
        .iter()
        .filter(|e| !matches!(e.command, PaletteCommand::Setting(SettingOp::Choose(..))))
        .count()
}

/// Every `Settings: ...` entry, in the current language.
pub fn entries() -> &'static [PaletteEntry] {
    &model().entries
}

/// Each setting's `Settings: <label>:` (lower case), for
/// [`crate::palette::settings_listed`].
pub fn prefixes() -> &'static [String] {
    &model().prefixes
}

/// The setting a row number of an entry names.
pub fn row_of(op: SettingOp) -> Option<&'static SettingRow> {
    model().rows.get(op.row() as usize)
}

/// The query that narrows the palette to `op`'s choices.
pub fn sub_query(op: SettingOp) -> Option<String> {
    if !matches!(op, SettingOp::Sub(_)) {
        return None;
    }
    let model = model();
    let row = model.rows.get(op.row() as usize)?;
    matches!(row.kind, SettingKind::Choice(_))
        .then(|| format!("{} ", model.prefixes[op.row() as usize]))
}
