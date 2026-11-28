//! What a provider can do for an item, decided by its family alone: whether
//! the metadata (assignees, reviewers, labels, milestone) can be edited and
//! whether a request can be reviewed in the app. A frontend offers the matching
//! action only when the function says yes.

use gitpoulpi_config::{ApiFamily, IntegrationKind};

/// Whether assignees, reviewers, labels and milestone can be edited on this
/// provider. GitHub, GitLab, Gitea / Forgejo, Linear, YouTrack, Redmine and
/// the local Markdown source: nothing about metadata shows for any other
/// provider.
pub fn supports_metadata(kind: IntegrationKind) -> bool {
    matches!(
        kind.family(),
        ApiFamily::Github
            | ApiFamily::Gitlab
            | ApiFamily::Gitea
            | ApiFamily::Linear
            | ApiFamily::YouTrack
            | ApiFamily::Redmine
            | ApiFamily::LocalMarkdown
    )
}

/// Whether a request of this provider can be reviewed in the app: the
/// providers that take metadata plus Gerrit, and only in a build with the
/// `review` feature.
pub fn supports_review(kind: IntegrationKind) -> bool {
    crate::features::REVIEW && (supports_metadata(kind) || kind.family() == ApiFamily::Gerrit)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metadata_follows_the_family() {
        assert!(supports_metadata(IntegrationKind::Github));
        assert!(supports_metadata(IntegrationKind::Gitlab));
        assert!(!supports_metadata(IntegrationKind::JiraCloud));
    }
}
