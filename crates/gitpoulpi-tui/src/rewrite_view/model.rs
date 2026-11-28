//! The rules list model lives in `gitpoulpi_link::rewrite_rules`; this keeps
//! its old paths and adds the one terminal-build question (is the embedded
//! script runtime compiled in).

#[allow(unused_imports)]
pub(crate) use gitpoulpi_link::rewrite_rules::*;

use gitpoulpi_i18n::M;

pub(crate) trait RuleTemplateExt {
    /// Why this kind cannot be added in this build, when it cannot.
    fn disabled_reason(self) -> Option<M>;
}

impl RuleTemplateExt for RuleTemplate {
    fn disabled_reason(self) -> Option<M> {
        self.disabled_reason_with(super::script::scripts_built())
    }
}
