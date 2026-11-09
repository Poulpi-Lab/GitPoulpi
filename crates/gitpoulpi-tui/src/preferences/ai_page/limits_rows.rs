//! The Limits rows of the AI page: how much input a request may carry before
//! GitPoulpi warns instead of sending (`[ai.limits]`). A request over a limit
//! never fails and nothing is cut silently: a conflict opens a warning with
//! "Send anyway", "Open setting" (it lands on its row below) and "Cancel";
//! every other input opens "Send all" / "Send truncated" / "Open setting" /
//! "Cancel" (`ai_limit_dialog`).

use gitpoulpi_i18n::msg::ai_limits as msg;

use super::*;

/// Above this a very large limit is flagged: the model will most likely not
/// take that much text, and the request then fails at the provider.
const VERY_LARGE_BYTES: usize = 400_000;

impl PreferencesState {
    /// The Limits rows: one number per capped input.
    pub(in crate::preferences) fn limits_rows(&self) -> Vec<Field> {
        use gitpoulpi_config::{AiLimits, MIN_LIMIT_BYTES};
        use gitpoulpi_config::{MAX_CONFLICT_BYTES, MAX_LIMIT_BYTES, MIN_CONFLICT_BYTES};
        let l = &self.config.ai.limits;
        // (label, help, key, current, min, max): the keys are `AiLimits::KEYS`.
        let rows = [
            (
                msg::CONFLICT_LABEL.get(),
                msg::CONFLICT_HELP.get(),
                l.conflict_bytes_clamped(),
                MIN_CONFLICT_BYTES,
                MAX_CONFLICT_BYTES,
            ),
            (
                msg::DIFF_LABEL.get(),
                msg::DIFF_HELP.get(),
                l.diff_bytes_clamped(),
                MIN_LIMIT_BYTES,
                MAX_LIMIT_BYTES,
            ),
            (
                msg::REVIEW_FILE_LABEL.get(),
                msg::REVIEW_FILE_HELP.get(),
                l.review_file_bytes_clamped(),
                MIN_LIMIT_BYTES,
                MAX_LIMIT_BYTES,
            ),
            (
                msg::REVIEW_TEXT_LABEL.get(),
                msg::REVIEW_TEXT_HELP.get(),
                l.review_text_bytes_clamped(),
                MIN_LIMIT_BYTES,
                MAX_LIMIT_BYTES,
            ),
            (
                msg::REVIEW_THREADS_LABEL.get(),
                msg::REVIEW_THREADS_HELP.get(),
                l.review_threads_bytes_clamped(),
                MIN_LIMIT_BYTES,
                MAX_LIMIT_BYTES,
            ),
            (
                msg::REVIEW_CONTEXT_FILE_LABEL.get(),
                msg::REVIEW_CONTEXT_FILE_HELP.get(),
                l.review_context_file_bytes_clamped(),
                MIN_LIMIT_BYTES,
                MAX_LIMIT_BYTES,
            ),
            (
                msg::REVIEW_CONTEXT_LABEL.get(),
                msg::REVIEW_CONTEXT_HELP.get(),
                l.review_context_bytes_clamped(),
                MIN_LIMIT_BYTES,
                MAX_LIMIT_BYTES,
            ),
        ];
        let mut fields = vec![Field::heading(msg::SETTINGS_HEADING.get())];
        for ((label, help, value, min, max), key) in rows.into_iter().zip(AiLimits::KEYS) {
            let mut row = Field::global(
                label,
                key,
                Control::Number {
                    value: value as i64,
                    min: min as i64,
                    max: max as i64,
                },
            )
            .with_help(help);
            if value > VERY_LARGE_BYTES {
                row = row.with_danger(msg::CONFLICT_WARNING.get());
            }
            fields.push(row);
        }
        fields
    }
}
