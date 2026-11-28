//! `Settings: ...` rows: the Preferences fields as palette commands.
//!
//! Generated from the field model (`preferences::palette_settings`), never
//! listed by hand. One row per setting, by what it is:
//! - a toggle: `Settings: Toggle <label>`, runs at once;
//! - a number or text: `Settings: <label>...`, the Settings page opens on the row with
//!   it being edited, refused like the page refuses it;
//! - a choice: `Settings: <label>...` narrows the palette to its choices
//!   (`Settings: <label>: <choice>` rows, the current one marked);
//! - anything that needs its page: `Settings: <label>...` opens Settings on it.
//!
//! The rows are built once per language (their text is leaked: they are
//! `&'static PaletteEntry` like the catalogue's), the values shown beside
//! them are read again whenever `config.toml` changes.

use crate::preferences::palette_settings::{self as ps, Current, Edit, SettingKind, SettingRow};
#[allow(unused_imports)]
pub(crate) use gitpoulpi_app::palette::SettingOp;
use gitpoulpi_app::palette::{PaletteCommand, PaletteEntry};
use gitpoulpi_i18n::msg;
use std::collections::HashMap;
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
    let rows = ps::rows();
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
                surface: gitpoulpi_app::surface::settings_surface(row.key),
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
pub(crate) fn listed_count() -> usize {
    entries()
        .iter()
        .filter(|e| !matches!(e.command, PaletteCommand::Setting(SettingOp::Choose(..))))
        .count()
}

/// Every `Settings: ...` entry, in the current language.
pub(crate) fn entries() -> &'static [PaletteEntry] {
    &model().entries
}

/// Each setting's `Settings: <label>:` (lower case), for [`listed`].
pub(crate) fn prefixes() -> &'static [String] {
    &model().prefixes
}

/// The query that narrows the palette to `op`'s choices.
pub(crate) fn sub_query(op: SettingOp) -> Option<String> {
    if !matches!(op, SettingOp::Sub(_)) {
        return None;
    }
    let model = model();
    let row = model.rows.get(op.row() as usize)?;
    matches!(row.kind, SettingKind::Choice(_))
        .then(|| format!("{} ", model.prefixes[op.row() as usize]))
}

/// `config.toml` and `tui.toml` (mtime, length each), then the language epoch.
type Stamp = (
    (Option<std::time::SystemTime>, u64),
    (Option<std::time::SystemTime>, u64),
    u64,
);
type Values = (Stamp, HashMap<&'static str, Current>);
static VALUES: Mutex<Option<Values>> = Mutex::new(None);

fn config_stamp() -> Stamp {
    let paths = gitpoulpi_config::ConfigPaths::discover();
    let stat = |file: Option<std::path::PathBuf>| {
        let meta = file.and_then(|f| std::fs::metadata(f).ok());
        (
            meta.as_ref().and_then(|m| m.modified().ok()),
            meta.as_ref().map_or(0, |m| m.len()),
        )
    };
    (
        stat(paths.as_ref().map(|p| p.config_file())),
        stat(paths.as_ref().map(|p| p.tui_file())),
        gitpoulpi_i18n::epoch(),
    )
}

fn current_of(op: SettingOp) -> Option<Current> {
    let key = model().rows.get(op.row() as usize)?.key;
    let stamp = config_stamp();
    let mut slot = VALUES.lock().unwrap_or_else(|e| e.into_inner());
    if slot.as_ref().is_none_or(|(s, _)| *s != stamp) {
        *slot = Some((stamp, HashMap::new()));
    }
    let map = &mut slot.as_mut()?.1;
    if let Some(found) = map.get(key) {
        return Some(found.clone());
    }
    let found = ps::current(key)?;
    map.insert(key, found.clone());
    Some(found)
}

/// The text a `Settings: ...` row shows in the shortcut column: the value
/// now, or the "current" mark on the choice that is set.
pub(crate) fn value_text(op: SettingOp) -> String {
    let Some(current) = current_of(op) else {
        return String::new();
    };
    match op {
        SettingOp::Choose(_, o) => {
            if current.choice == Some(o as usize) {
                msg::palette::SETTINGS_CURRENT.get().to_string()
            } else {
                String::new()
            }
        }
        SettingOp::Page(_) => String::new(),
        _ => current.shown,
    }
}

/// What running a `Settings: ...` row asks the window to do.
pub(crate) enum Outcome {
    /// Done (or refused): say so in the status line.
    Message(String),
    /// Open Settings on this key (a page row only).
    OpenPage(&'static str),
    /// Open Settings on this key with the row already being edited: the
    /// page's own field, validation and messages, never a footer prompt.
    EditOnPage(&'static str),
}

fn not_editable(current: &Current) -> Option<String> {
    current.coming.clone()
}

pub(crate) fn run(op: SettingOp) -> Outcome {
    let model = model();
    let Some(row) = model.rows.get(op.row() as usize) else {
        return Outcome::Message(String::new());
    };
    let current = current_of(op);
    if let Some(why) = current.as_ref().and_then(not_editable) {
        return Outcome::Message(format!("{}: {why}", row.label));
    }
    let done = |result: Result<String, String>| {
        Outcome::Message(result.unwrap_or_else(|why| format!("{}: {why}", row.label)))
    };
    match op {
        SettingOp::Toggle(_) => done(ps::apply(row.key, Edit::Toggle)),
        SettingOp::Choose(_, o) => done(ps::apply(row.key, Edit::Choose(o as usize))),
        SettingOp::Edit(_) => Outcome::EditOnPage(row.key),
        SettingOp::Page(_) => Outcome::OpenPage(row.key),
        // The palette itself narrows to the choices (its Enter arm).
        SettingOp::Sub(_) => Outcome::Message(String::new()),
    }
}
