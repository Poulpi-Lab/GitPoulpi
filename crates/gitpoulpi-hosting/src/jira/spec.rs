//! The Jira family spec: the browse page of an issue.

use crate::spec::FamilySpec;
use gitpoulpi_config::ApiFamily;

pub static SPEC: FamilySpec = FamilySpec {
    // `{site}/browse/ABC-123`, the same on Cloud and Data Center.
    issue_web_url: |base, _project, reference| {
        let reference = reference.trim();
        (!reference.is_empty())
            .then(|| format!("{}/browse/{reference}", base.trim_end_matches('/')))
    },
    ..FamilySpec::none(ApiFamily::Jira, "JI", '#')
};
