//! "Check signing" in the signature question (slice S19): one dummy payload
//! signed and verified through the daemon (it may open a passphrase prompt,
//! so it never runs on the render thread), then the measured time per
//! signature, the estimate for the commits that would be signed, or why
//! signing does not work with the tool's install hint.

use gitpoulpi_core::domain::rewrite::{OsFamily, RewriteRefusal, SigningCheck};
use gitpoulpi_core::domain::OperationKind;
use gitpoulpi_i18n::msg::rewrite_view as t;

use super::RewriteView;

/// The Summary line of a preview that signed commits: how many and how long.
pub(super) fn resigned_line(resigned: u64, millis: u64) -> String {
    t::sum_resigned(resigned, time_text(millis))
}

/// `850 ms`, `12.5 s`, `3 min 20 s`: the unit is the same in every language.
pub(crate) fn time_text(millis: u64) -> String {
    if millis < 1_000 {
        format!("{millis} ms")
    } else if millis < 60_000 {
        format!("{}.{} s", millis / 1_000, (millis % 1_000) / 100)
    } else {
        format!("{} min {} s", millis / 60_000, (millis % 60_000) / 1_000) // i18n-english: unit symbols
    }
}

impl RewriteView {
    /// Starts the smoke test (the "Check signing" row of the question).
    pub(super) fn request_sign_check(&mut self) {
        if self.sign_wait.is_some() {
            return;
        }
        self.sign_check = None;
        self.sign_wait = Some(
            self.requester
                .call(OperationKind::RewriteCheckSigning, serde_json::Value::Null),
        );
    }

    /// Takes the answer of the smoke test when it is there.
    pub(super) fn poll_sign_check(&mut self) {
        let Some(reply) = &self.sign_wait else { return };
        if let Some(answer) = reply.take() {
            self.sign_wait = None;
            self.sign_check = answer.ok();
        }
    }

    /// The lines the question adds under its prompt: waiting, the measured
    /// time and the estimate for `count` commits, or the failure and the
    /// install hint.
    pub(super) fn sign_check_text(&self, count: Option<u64>) -> Option<String> {
        if self.sign_wait.is_some() {
            return Some(t::Q_SIGN_CHECKING.get().to_string());
        }
        let check: &SigningCheck = self.sign_check.as_ref()?;
        Some(match check.reason {
            None => {
                let mut text = t::q_sign_check_ok(check.tool.name(), check.millis);
                if let Some(n) = count {
                    text.push(' ');
                    text.push_str(&t::q_sign_estimate(time_text(check.estimate_millis(n)), n));
                }
                text
            }
            Some(reason) => {
                let why = if check.detail.is_empty() {
                    check.tool.name().to_string()
                } else {
                    check.detail.replace('\n', " ")
                };
                let failed = t::q_sign_check_failed(why);
                if reason == RewriteRefusal::ToolMissing {
                    let hint = super::tools::tool_hint(check.tool, OsFamily::current());
                    format!("{failed} {}", hint.get())
                } else {
                    failed
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::time_text;

    #[test]
    fn times_read_in_the_unit_that_fits() {
        assert_eq!(time_text(85), "85 ms");
        assert_eq!(time_text(12_500), "12.5 s");
        assert_eq!(time_text(200_000), "3 min 20 s");
    }
}
