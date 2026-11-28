//! The Settings fields as command-palette rows.
//!
//! The palette's `Settings: ...` rows are generated from the Preferences
//! field model (the pool of plain `config.toml` rows, `PreferencesState::
//! row_pool`), never listed by hand, so a new setting appears in the palette
//! with its page row. A value is applied and persisted by the very code the
//! page runs (`commit_bool`, `commit_choice`, `commit_text` ->
//! `write_global`, with its live installs), on a *detached* state: the page
//! need not be open, and nothing here keeps the screen's state.

use super::*;
#[cfg(test)]
use gitpoulpi_app::settings as shared;
use gitpoulpi_secrets::InMemoryTokens;
#[cfg(test)]
pub(crate) use shared::SettingKind;
#[cfg(test)]
pub(crate) use shared::SettingRow;

/// A setting's value right now.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Current {
    /// What the page shows on the row (`[x]`, `3`, the choice's text).
    pub(crate) shown: String,
    /// The picked option of a choice.
    pub(crate) choice: Option<usize>,
    /// What a prompt starts with.
    pub(crate) text: String,
    /// Set when the row is not editable right now, with the page's reason.
    pub(crate) coming: Option<String>,
}

/// What to do to a setting.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Edit {
    Toggle,
    Choose(usize),
    #[cfg_attr(not(test), allow(dead_code))]
    Text(String),
}

impl PreferencesState {
    /// A state that edits `config.toml` and nothing else: no repository,
    /// no token store, no integration probes. What the palette applies a
    /// setting on.
    pub(crate) fn detached() -> Self {
        let paths = ConfigPaths::discover();
        let config_path = paths.as_ref().map(|p| p.config_file());
        let mut config = config_path
            .as_deref()
            .map(Config::load)
            .transpose()
            .ok()
            .flatten()
            .unwrap_or_default();
        config.integrations.clear();
        Self::with_parts(
            config,
            config_path,
            Path::new("."),
            RepoState::default(),
            PathBuf::new(),
            std::sync::Arc::new(InMemoryTokens::default()),
        )
    }

    #[cfg(test)]
    pub(super) fn setting_rows(&self) -> Vec<SettingRow> {
        let mut seen = std::collections::HashSet::new();
        let mut out = Vec::new();
        for field in self.row_pool() {
            let Target::Global(key) = field.target else {
                continue;
            };
            if !seen.insert(key) {
                continue;
            }
            // A row that is "coming" today (undo depth with undo off) is still
            // listed: running it says why it cannot be edited now.
            out.extend(shared::setting_row(
                key,
                &field.label,
                &field.control,
                |raw| field.option_label(raw),
            ));
        }
        out
    }

    pub(super) fn setting_current(&self, key: &str) -> Option<Current> {
        let field = self
            .row_pool()
            .into_iter()
            .find(|f| matches!(f.target, Target::Global(k) if k == key))?;
        Some(Current {
            shown: field.value_text(),
            choice: match &field.control {
                Control::Choice { index, .. } => Some(*index),
                _ => None,
            },
            text: match &field.control {
                Control::Text(t) => t.clone(),
                Control::Number { value, .. } => value.to_string(),
                _ => String::new(),
            },
            coming: field.coming.clone(),
        })
    }

    /// Edits `key` exactly as its page row would. `Err` carries the reason
    /// (the page's own message for a refused value).
    pub(super) fn apply_setting(&mut self, key: &str, edit: Edit) -> Result<String, String> {
        self.show_page(None, Some(key))?;
        let fields = self.fields();
        let field = fields.get(self.field).ok_or_else(|| key.to_string())?;
        let target = field.target.clone();
        let ok = match (edit, &field.control) {
            (Edit::Toggle, Control::Toggle(on)) => {
                let next = !*on;
                self.commit_bool(&target, next);
                true
            }
            (Edit::Choose(i), Control::Choice { .. }) => {
                self.commit_choice(&target, i);
                true
            }
            (Edit::Text(draft), _) => {
                let index = self.field;
                self.commit_text(index, &draft)
            }
            _ => false,
        };
        let message = self.take_message().unwrap_or_default();
        if ok {
            Ok(message)
        } else {
            Err(message)
        }
    }
}

/// Every setting as the terminal UI's own rows build it (the palette's list
/// comes from the shared model; a test compares the two).
#[cfg(test)]
pub(crate) fn rows() -> Vec<SettingRow> {
    PreferencesState::with_parts(
        Config::default(),
        None,
        Path::new("."),
        RepoState::default(),
        PathBuf::new(),
        std::sync::Arc::new(InMemoryTokens::default()),
    )
    .setting_rows()
}

/// `key`'s value now, read from `config.toml`.
pub(crate) fn current(key: &str) -> Option<Current> {
    PreferencesState::detached().setting_current(key)
}

/// Applies `edit` to `key` and persists it. `Ok` carries the page's
/// confirmation (`key = value`), `Err` the reason it was refused.
pub(crate) fn apply(key: &str, edit: Edit) -> Result<String, String> {
    PreferencesState::detached().apply_setting(key, edit)
}

impl PreferencesState {
    /// Opens the page on the setting `key` (selected and flashed).
    pub(crate) fn show_setting(&mut self, key: &str) -> Result<(), String> {
        self.show_page(None, Some(key))
    }
}

impl PreferencesState {
    /// Opens the page on `key` with its row already being edited (a number
    /// or text field with its value, a quantity's stepper): what a
    /// `Settings: <label>...` palette row does, so the typing happens in the
    /// page's own field and never in the status line.
    pub(crate) fn edit_setting(&mut self, key: &str) -> Result<(), String> {
        self.show_page(None, Some(key))?;
        self.focus = PrefFocus::Fields;
        self.activate();
        Ok(())
    }
}
