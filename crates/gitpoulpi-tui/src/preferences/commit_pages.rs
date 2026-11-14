//! The Commit, Merge options and Message template pages (Settings >
//! Repository; plan `repo-settings.md`, slice 1).
//!
//! The switches are `RepoPreferences` fields in the repo data directory
//! (`repo-state.toml`); the template is a file there that git's
//! `commit.template` points at. If `commit.template` already names another
//! file, that file is shown and edited and the setting is left alone. What
//! the switches do lives elsewhere: `commit_settings` (the composer, the
//! commit's comment stripping), `daemon_link::run_action` (push after commit,
//! squash merge).

use std::path::PathBuf;

use gitpoulpi_i18n::msg::settings_repo as t;

use super::repo_pages::{RpButton, RpTarget};
use super::*;

/// The template being edited: what the file held when the page read it (or
/// the daemon last said where it is) plus the user's edits.
#[derive(Debug, Clone, Default)]
pub(super) struct TemplateDraft {
    pub(super) path: PathBuf,
    pub(super) summary: String,
    pub(super) description: String,
    /// Edited and not saved yet: a changed `commit.template` does not reload.
    pub(super) dirty: bool,
}

/// The conventional summary limit the counter counts down from.
const SUMMARY_LIMIT: i64 = 72;

impl PreferencesState {
    // -- the template's file ----------------------------------------------------

    /// Where the template lives: what `commit.template` names, else GitPoulpi's
    /// own file beside `repo-state.toml`.
    pub(super) fn template_path(&self) -> PathBuf {
        let configured = self
            .repo_config
            .get(gitpoulpi_core::repo_config::COMMIT_TEMPLATE)
            .and_then(|v| v.as_deref());
        let own = self.own_template_path();
        match configured.map(str::trim).filter(|v| !v.is_empty()) {
            Some(_) => {
                let home = std::env::var_os("HOME")
                    .or_else(|| std::env::var_os("USERPROFILE"))
                    .map(PathBuf::from);
                gitpoulpi_session::template_path(&self.repo_root, "", configured, home.as_deref())
            }
            None => own,
        }
    }

    /// GitPoulpi's own template file, beside `repo-state.toml`.
    fn own_template_path(&self) -> PathBuf {
        self.repo_state_path
            .parent()
            .map(|dir| dir.join(gitpoulpi_session::TEMPLATE_FILE))
            .unwrap_or_else(|| PathBuf::from(gitpoulpi_session::TEMPLATE_FILE))
    }

    /// Whether `commit.template` names a file other than GitPoulpi's own.
    fn template_is_external(&self) -> bool {
        let configured = self
            .repo_config
            .get(gitpoulpi_core::repo_config::COMMIT_TEMPLATE)
            .and_then(|v| v.as_deref())
            .is_some_and(|v| !v.trim().is_empty());
        configured && self.template_path() != self.own_template_path()
    }

    /// Reads the template file into the draft, unless unsaved edits (`force`
    /// drops them) or the same file is already loaded.
    pub(super) fn template_reload(&mut self, force: bool) {
        let path = self.template_path();
        let draft = &self.repo_pages.template;
        if !force && (draft.dirty || (draft.path == path && !draft.path.as_os_str().is_empty())) {
            return;
        }
        let (summary, description) =
            gitpoulpi_session::split_template(&gitpoulpi_session::read_template(&path));
        self.repo_pages.template = TemplateDraft {
            path,
            summary,
            description,
            dirty: false,
        };
    }

    /// A changed `commit.template` (the event loop pushes git's values in)
    /// moves the page to that file.
    pub(super) fn template_follow_config(&mut self) {
        self.template_reload(false);
    }

    // -- the pages --------------------------------------------------------------

    fn toggle_row(&self, label: &str, field: RepoField, on: bool, help: &str) -> Field {
        Field::repo(label, field, Control::Toggle(on))
            .live()
            .with_help(help)
    }

    pub(super) fn commit_fields(&self) -> Vec<Field> {
        let prefs = &self.repo_state.preferences;
        vec![
            self.toggle_row(
                t::COMMIT_PUSH_AFTER.get(),
                RepoField::PushAfterCommit,
                prefs.push_after_commit.unwrap_or(false),
                t::COMMIT_PUSH_AFTER_HELP.get(),
            ),
            self.toggle_row(
                t::COMMIT_SKIP_HOOKS.get(),
                RepoField::SkipHooks,
                prefs.skip_hooks_by_default.unwrap_or(false),
                t::COMMIT_SKIP_HOOKS_HELP.get(),
            ),
        ]
    }

    pub(super) fn merge_fields(&self) -> Vec<Field> {
        let prefs = &self.repo_state.preferences;
        let own = prefs
            .merge_fast_forward
            .as_deref()
            .and_then(MergeFastForward::parse);
        let mut fields = vec![
            self.toggle_row(
                t::MERGE_SQUASH.get(),
                RepoField::SquashMerge,
                prefs.squash_merge.unwrap_or(false),
                t::MERGE_SQUASH_HELP.get(),
            ),
            Field::repo(
                t::MERGE_FAST_FORWARD.get(),
                RepoField::MergeFastForward,
                Control::Choice {
                    options: std::iter::once(
                        gitpoulpi_i18n::msg::settings_git::INHERITED
                            .get()
                            .to_string(),
                    )
                    .chain(MergeFastForward::ALL.iter().map(|m| m.as_str().to_string()))
                    .collect(),
                    index: own
                        .and_then(|o| MergeFastForward::ALL.iter().position(|m| *m == o))
                        .map_or(0, |i| i + 1),
                },
            )
            .live()
            .with_help(t::MERGE_FAST_FORWARD_HELP.get()),
        ];
        // What git itself would do under `allowed`: shown, never edited here.
        if let Some(git_ff) = self
            .repo_config
            .get(gitpoulpi_core::repo_config::MERGE_FF)
            .and_then(|v| v.as_deref())
        {
            fields.push(Field::plain(
                &t::merge_fast_forward_git(gitpoulpi_core::repo_config::MERGE_FF, git_ff),
                Control::Info(String::new()),
                Target::None,
            ));
        }
        fields
    }

    pub(super) fn template_fields(&self) -> Vec<Field> {
        let prefs = &self.repo_state.preferences;
        let draft = &self.repo_pages.template;
        let mut fields = Vec::new();
        if self.template_is_external() {
            fields.push(Field::plain(
                &t::template_external(
                    gitpoulpi_core::repo_config::COMMIT_TEMPLATE,
                    draft.path.display(),
                ),
                Control::Info(String::new()),
                Target::None,
            ));
        }
        fields.push(self.toggle_row(
            t::TEMPLATE_APPLY.get(),
            RepoField::TemplateApply,
            prefs.template_apply.unwrap_or(false),
            t::TEMPLATE_APPLY_HELP.get(),
        ));
        fields.push(self.toggle_row(
            t::TEMPLATE_STRIP.get(),
            RepoField::TemplateStripComments,
            prefs.template_strip_comments.unwrap_or(false),
            t::TEMPLATE_STRIP_HELP.get(),
        ));
        // The counter follows what is being typed, not only what was committed.
        let summary_at = fields.len();
        let shown = match &self.edit {
            Some((i, text, _)) if *i == summary_at => text.as_str(),
            _ => draft.summary.as_str(),
        };
        let left = SUMMARY_LIMIT - shown.chars().count() as i64;
        let label = if left >= 0 {
            t::template_summary_left(left)
        } else {
            t::template_summary_over(-left)
        };
        fields.push(
            Field::plain(
                &label,
                Control::Text(draft.summary.clone()),
                Target::RepoPage(RpTarget::TemplateSummary),
            )
            .with_help(t::TEMPLATE_SUMMARY_HELP.get()),
        );
        fields.push(
            Field::plain(
                t::TEMPLATE_DESCRIPTION.get(),
                Control::Area(draft.description.clone()),
                Target::RepoPage(RpTarget::TemplateDescription),
            )
            .with_placeholder(t::TEMPLATE_DESCRIPTION_PLACEHOLDER.get())
            .with_help(t::TEMPLATE_DESCRIPTION_HELP.get()),
        );
        fields.push(
            Field::button(
                t::TEMPLATE_SAVE.get(),
                PrefAction::RepoPage(RpButton::SaveTemplate),
            )
            .with_help(&t::template_save_help(
                draft.path.display(),
                gitpoulpi_core::repo_config::COMMIT_TEMPLATE,
            )),
        );
        fields
    }

    // -- committing -------------------------------------------------------------

    /// A toggle of these pages: saved into `repo-state.toml` at once.
    pub(super) fn commit_repo_toggle(&mut self, field: RepoField, value: bool) {
        let prefs = &mut self.repo_state.preferences;
        match field {
            RepoField::PushAfterCommit => prefs.push_after_commit = Some(value),
            RepoField::SkipHooks => prefs.skip_hooks_by_default = Some(value),
            RepoField::SquashMerge => prefs.squash_merge = Some(value),
            RepoField::TemplateApply => prefs.template_apply = Some(value),
            RepoField::TemplateStripComments => prefs.template_strip_comments = Some(value),
            RepoField::ConflictDetection => prefs.conflict_detection = Some(value),
            RepoField::PushTarget(i) => {
                let Some(remote) = self.remote_names.get(i).cloned() else {
                    return;
                };
                let targets = &mut prefs.default_push_targets;
                targets.retain(|t| *t != remote);
                if value {
                    targets.push(remote);
                    // Keep the order of the remotes, which is the push order.
                    let names = &self.remote_names;
                    targets.sort_by_key(|t| names.iter().position(|n| n == t));
                }
            }
            _ => return,
        }
        self.write_repo_state();
    }

    /// The summary row's typed text.
    pub(super) fn rp_commit_text(&mut self, target: RpTarget, draft: &str) -> bool {
        if matches!(
            target,
            RpTarget::SaveAsList | RpTarget::ConflictListNew | RpTarget::ConflictListName(_)
        ) {
            return self.conflict_commit_text(target, draft);
        }
        if target == RpTarget::TemplateSummary {
            let text = draft.trim_end().replace('\n', " ");
            if text != self.repo_pages.template.summary {
                self.repo_pages.template.summary = text;
                self.repo_pages.template.dirty = true;
            }
        }
        true
    }

    /// Writes the template file and points `commit.template` at it (unless it
    /// already names a file, which is then the one edited).
    pub(super) fn template_save(&mut self) {
        let draft = self.repo_pages.template.clone();
        let text = gitpoulpi_session::join_template(&draft.summary, &draft.description);
        if let Err(err) = gitpoulpi_session::write_template(&draft.path, &text) {
            let text = gitpoulpi_i18n::msg::settings_screen::repo_settings_error(err);
            crate::activity_log::report_problem(crate::activity_log::ProblemSource::Save, &text);
            self.message = Some(text);
            return;
        }
        self.repo_pages.template.dirty = false;
        let configured = self
            .repo_config
            .get(gitpoulpi_core::repo_config::COMMIT_TEMPLATE)
            .and_then(|v| v.as_deref())
            .is_some_and(|v| !v.trim().is_empty());
        if !configured {
            let value = draft.path.to_string_lossy().into_owned();
            let key = gitpoulpi_core::repo_config::COMMIT_TEMPLATE.to_string();
            self.repo_config.insert(key.clone(), Some(value.clone()));
            self.repo_config_requests.push((key, Some(value)));
        }
        self.message = Some(t::template_saved(draft.path.display()));
    }
}
