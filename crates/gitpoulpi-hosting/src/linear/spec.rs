//! The Linear family spec: the page of an issue.

use crate::spec::FamilySpec;
use gitpoulpi_config::ApiFamily;

pub static SPEC: FamilySpec = FamilySpec {
    // `https://linear.app/<workspace>/issue/ENG-123`. The workspace slug is
    // part of the profile's URL when the user typed it
    // (`https://linear.app/acme`); the listed rows carry the exact URL the
    // API gave, this one serves commit-message references.
    issue_web_url: |base, _project, reference| {
        let reference = reference.trim();
        (!reference.is_empty()).then(|| format!("{}/issue/{reference}", base.trim_end_matches('/')))
    },
    ..FamilySpec::none(ApiFamily::Linear, "LN", '#')
};
