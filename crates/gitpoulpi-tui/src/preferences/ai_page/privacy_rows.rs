//! The Privacy rows of the AI page: what is taken out of a request before it
//! leaves (`[ai.privacy]`), and which requests are shown for review first
//! (`[ai.sent_preview]`, one switch per action, "Show all again").
//!
//! Turning a protection off is the user's choice, and the row says so in red
//! for as long as it is off.

use gitpoulpi_config::AiAction;
use gitpoulpi_i18n::msg::ai_privacy as msg;

use super::*;

/// One of the rows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::preferences) enum PrivacyRow {
    Redact,
    Exclude,
    /// "Show what is sent before ..." for one action.
    Preview(AiAction),
}

impl PrivacyRow {
    fn key(self) -> String {
        match self {
            Self::Redact => "ai.privacy.redact_secrets".to_string(),
            Self::Exclude => "ai.privacy.exclude_sensitive_files".to_string(),
            Self::Preview(action) => format!("ai.sent_preview.{}", action.key()),
        }
    }
}

/// The actions whose request the generic dialog shows. The two history-rewrite
/// dialogs show their own "What is sent" before they ask.
pub(in crate::preferences) fn previewed(action: AiAction) -> bool {
    !matches!(action, AiAction::RewriteScript | AiAction::RewriteRules)
}

impl PreferencesState {
    /// The privacy rows: the two protections, then one switch per action.
    pub(in crate::preferences) fn privacy_rows(&self) -> Vec<Field> {
        let ai = &self.config.ai;
        let mut rows = vec![Field::heading(msg::SETTINGS_HEADING.get())];
        let mut redact = Field::plain(
            msg::REDACT_LABEL.get(),
            Control::Toggle(ai.privacy.redact_secrets),
            Target::Ai(AiTarget::Privacy(PrivacyRow::Redact)),
        )
        .with_help(msg::REDACT_HELP.get());
        if !ai.privacy.redact_secrets {
            redact = redact.with_danger(msg::REDACT_WARNING.get());
        }
        rows.push(redact);
        let mut exclude = Field::plain(
            msg::EXCLUDE_LABEL.get(),
            Control::Toggle(ai.privacy.exclude_sensitive_files),
            Target::Ai(AiTarget::Privacy(PrivacyRow::Exclude)),
        )
        .with_help(msg::EXCLUDE_HELP.get());
        if !ai.privacy.exclude_sensitive_files {
            exclude = exclude.with_danger(msg::EXCLUDE_WARNING.get());
        }
        rows.push(exclude);
        rows.push(Field::heading(msg::SENT_HEADING.get()));
        let mut first = true;
        for action in AiAction::ALL {
            if !previewed(action) || !crate::preferences::ai_action_built(action) {
                continue;
            }
            let mut row = Field::plain(
                action_label(action),
                Control::Toggle(ai.sent_preview.shows(action)),
                Target::Ai(AiTarget::Privacy(PrivacyRow::Preview(action))),
            );
            if first {
                row = row.with_help(msg::SENT_HELP.get());
                first = false;
            }
            rows.push(row);
        }
        if !ai.sent_preview.all_shown() {
            rows.push(Field::button(
                msg::SHOW_ALL_AGAIN.get(),
                PrefAction::Ai(AiButton::ShowAllPreviews),
            ));
        }
        rows
    }

    /// A toggle of one of the rows.
    pub(super) fn privacy_commit_bool(&mut self, row: PrivacyRow, value: bool) {
        self.write_global(&row.key(), if value { "true" } else { "false" });
    }

    /// "Show all again": every action's switch back on.
    pub(super) fn privacy_show_all(&mut self) {
        for action in AiAction::ALL {
            if !self.config.ai.sent_preview.shows(action) {
                self.write_global(&PrivacyRow::Preview(action).key(), "true");
            }
        }
        self.message = Some(msg::SHOWN_ALL.get().to_string());
    }
}
