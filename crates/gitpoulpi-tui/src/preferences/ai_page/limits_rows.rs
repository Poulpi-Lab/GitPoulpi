//! The Limits rows of the AI page: how much input a request may carry before
//! GitPoulpi warns instead of sending (`[ai.limits]`). A request over a limit
//! never fails: the warning dialog offers "Send anyway", "Open setting" (it
//! lands on the row below) and "Cancel".

use gitpoulpi_i18n::msg::ai_limits as msg;

use super::*;

/// Above this a very large limit is flagged: the model will most likely not
/// take that much text, and the request then fails at the provider.
const VERY_LARGE_BYTES: usize = 400_000;

impl PreferencesState {
    /// The Limits rows: one number per capped input.
    pub(in crate::preferences) fn limits_rows(&self) -> Vec<Field> {
        let limit = self.config.ai.limits.conflict_bytes_clamped();
        let mut row = Field::global(
            msg::CONFLICT_LABEL.get(),
            crate::guide_dialog::CONFLICT_LIMIT_KEY,
            Control::Number {
                value: limit as i64,
                min: gitpoulpi_config::MIN_CONFLICT_BYTES as i64,
                max: gitpoulpi_config::MAX_CONFLICT_BYTES as i64,
            },
        )
        .with_help(msg::CONFLICT_HELP.get());
        if limit > VERY_LARGE_BYTES {
            row = row.with_danger(msg::CONFLICT_WARNING.get());
        }
        vec![Field::heading(msg::SETTINGS_HEADING.get()), row]
    }
}
