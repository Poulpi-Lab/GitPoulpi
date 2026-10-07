//! The local Markdown family spec: an issue's "web page" is its note.

use crate::spec::FamilySpec;
use gitpoulpi_config::ApiFamily;

pub static SPEC: FamilySpec = FamilySpec {
    // `project` is the folder, `reference` the issue number.
    issue_web_url: |_base, project, reference| {
        let id: u64 = reference.trim().trim_start_matches('#').parse().ok()?;
        let store = super::Store::new(project, Default::default());
        let issue = store.get(id).ok()?;
        Some(super::client::file_url(&issue.path))
    },
    ..FamilySpec::none(ApiFamily::LocalMarkdown, "MD", '#')
};
