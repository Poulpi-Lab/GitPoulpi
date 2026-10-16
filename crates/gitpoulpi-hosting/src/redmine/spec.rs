//! The Redmine family spec: the page of an issue.

use crate::spec::FamilySpec;
use gitpoulpi_config::ApiFamily;

pub static SPEC: FamilySpec = FamilySpec {
    // `{url}/issues/12` for the reference `#12` (or `12`).
    issue_web_url: |base, _project, reference| {
        let number = reference.trim().trim_start_matches('#');
        (!number.is_empty() && number.chars().all(|c| c.is_ascii_digit()))
            .then(|| format!("{}/issues/{number}", base.trim_end_matches('/')))
    },
    ..FamilySpec::none(ApiFamily::Redmine, "RM", '#')
};
