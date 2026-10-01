//! The Trello family spec: a card's reference is already its short URL.

use crate::spec::FamilySpec;
use gitpoulpi_config::ApiFamily;

pub static SPEC: FamilySpec = FamilySpec {
    // `reference` is the card's `shortUrl` (`https://trello.com/c/AbCd1234`);
    // a bare short link is expanded against the web root.
    issue_web_url: |base, _project, reference| {
        let reference = reference.trim();
        if reference.is_empty() {
            None
        } else if reference.starts_with("http://") || reference.starts_with("https://") {
            Some(reference.to_string())
        } else {
            Some(format!("{}/c/{reference}", base.trim_end_matches('/')))
        }
    },
    ..FamilySpec::none(ApiFamily::Trello, "TR", '#')
};
