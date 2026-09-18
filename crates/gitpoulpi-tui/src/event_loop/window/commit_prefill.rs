//! What the commit composer starts with (plan `repo-settings.md`, slice 1):
//! this repository's commit settings, the template of an empty panel and the
//! message a squash merge leaves behind.

use crate::commit_settings::{squash_message, CommitSettings};
use crate::event_loop::*;
use init::WindowState;

impl WindowState {
    /// Re-reads the commit settings when git's config was just reported (the
    /// template file is `commit.template`) or Settings was closed (the
    /// switches may have changed), then offers the composer its starting text.
    /// `refreshed`: a refresh landed this tick.
    pub(in crate::event_loop) fn apply_commit_settings(&mut self, refreshed: bool) {
        let prefs_open = self.preferences.is_some();
        let reload = self.commit_settings_version != self.repo_config_version
            || (self.prefs_open_seen && !prefs_open);
        self.prefs_open_seen = prefs_open;
        if reload && self.repo_config_version > 0 {
            self.commit_settings_version = self.repo_config_version;
            let next = CommitSettings::load(&self.repo.root, &self.repo_config);
            let first = !self.commit_settings_ready;
            if first || next.skip_hooks_default != self.commit_settings.skip_hooks_default {
                self.commit_skip_hooks = next.skip_hooks_default;
            }
            // A changed template (or the apply switch turned on) is offered
            // to an empty composer again.
            if next.template_apply != self.commit_settings.template_apply
                || next.template_path != self.commit_settings.template_path
            {
                self.commit_template_applied = false;
            }
            self.commit_settings = next;
            self.commit_settings_ready = true;
        }
        if !self.commit_settings_ready
            || self.commit_amend
            || self.composer_focus.is_some()
            || !self.commit_summary.is_empty()
            || !self.commit_description.is_empty()
        {
            return;
        }
        // A squash merge left git's message: the panel opens with it, once
        // per squash (the user may then empty it on purpose).
        if refreshed {
            if let Some((stamp, message)) = squash_message(&self.repo.root) {
                if self.commit_squash_seen != Some(stamp) {
                    self.commit_squash_seen = Some(stamp);
                    let (summary, description) = gitpoulpi_session::split_template(&message);
                    self.fill_composer(summary, description);
                    return;
                }
            }
        }
        if self.commit_settings.template_apply && !self.commit_template_applied {
            self.commit_template_applied = true;
            let (summary, description) = self.commit_settings.template_fields();
            self.fill_composer(summary, description);
        }
    }

    fn fill_composer(&mut self, summary: String, description: String) {
        self.commit_summary = summary;
        self.commit_description = description;
        self.field_editor.caret_mut(MainField::Summary).set_cursor(
            &self.commit_summary,
            self.commit_summary.len(),
            false,
        );
        self.field_editor
            .caret_mut(MainField::Description)
            .set_cursor(
                &self.commit_description,
                self.commit_description.len(),
                false,
            );
    }
}
