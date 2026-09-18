//! The Conflict prevention page (Settings > Repository; plan
//! `repo-settings.md`, slice 3): this repository's proactive conflict
//! detection switch and its "Base branches" patterns.
//!
//! Both live in `repo-state.toml` (`RepoPreferences`); the global switch is
//! `general.conflict_detection` on the General page. What they do lives in the
//! daemon (`gitpoulpi_daemon::conflict_prevention`) and the window's badge
//! (`conflict_badge`): a value changed here is read again when Settings closes.

use gitpoulpi_core::domain::conflict_prevention::DEFAULT_BASE_GLOBS;
use gitpoulpi_i18n::msg::conflict_prevention as t;

use super::repo_pages::RpTarget;
use super::*;

/// The pattern table's rows: a pattern and what it matches.
type Example = (&'static str, fn() -> &'static str);
const EXAMPLES: [Example; 4] = [
    ("origin/main", || t::EX_ONE.get()),
    ("origin/**", || t::EX_REMOTE.get()),
    ("origin/feature/**", || t::EX_PREFIX.get()),
    ("!origin/abc", || t::EX_EXCLUDE.get()),
];

impl PreferencesState {
    /// The Base branches field: what was saved, else the default `**`.
    pub(super) fn base_branches_text(&self) -> String {
        self.repo_state
            .preferences
            .conflict_base_branches
            .clone()
            .unwrap_or_else(|| DEFAULT_BASE_GLOBS.to_string())
    }

    /// Saves the edited list. An empty one (or the default) is stored as
    /// "not set", so it follows any later change of the default.
    pub(super) fn set_base_branches(&mut self, text: &str) {
        let lines = gitpoulpi_core::domain::conflict_prevention::parse_base_globs(text);
        let value = (!lines.is_empty() && lines != [DEFAULT_BASE_GLOBS]).then(|| lines.join("\n"));
        if value != self.repo_state.preferences.conflict_base_branches {
            self.repo_state.preferences.conflict_base_branches = value;
            self.write_repo_state();
        }
    }

    pub(super) fn conflict_fields(&self) -> Vec<Field> {
        let prefs = &self.repo_state.preferences;
        let mut fields = vec![
            Field::repo(
                t::REPO_ENABLE.get(),
                RepoField::ConflictDetection,
                Control::Toggle(prefs.conflict_detection.unwrap_or(true)),
            )
            .live()
            .with_help(&t::repo_enable_help(&self.repo_label)),
            Field::plain(
                t::BASES.get(),
                Control::Area(self.base_branches_text()),
                Target::RepoPage(RpTarget::BaseBranches),
            )
            .with_placeholder(t::BASES_PLACEHOLDER.get())
            .with_help(t::BASES_HELP.get()),
            Field::plain(t::EXPLAIN.get(), Control::Info(String::new()), Target::None),
        ];
        for (pattern, what) in EXAMPLES {
            fields.push(Field::plain(
                pattern,
                Control::Info(what().to_string()),
                Target::None,
            ));
        }
        fields
    }
}
