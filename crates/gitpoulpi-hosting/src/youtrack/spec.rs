//! The YouTrack family spec: the page of an issue.

use crate::spec::FamilySpec;
use gitpoulpi_config::ApiFamily;

pub static SPEC: FamilySpec = FamilySpec {
    // `{url}/issue/ABC-12`, on Cloud and Server alike.
    issue_web_url: |base, _project, reference| {
        let reference = reference.trim();
        (!reference.is_empty()).then(|| format!("{}/issue/{reference}", base.trim_end_matches('/')))
    },
    ..FamilySpec::none(ApiFamily::YouTrack, "YT", '#')
};
