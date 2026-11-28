//! What the review view holds: the request it is about, what the host and
//! the repository answered, and the small enums the view and its palette
//! twins share.

pub(crate) use gitpoulpi_link::review_model::{DraftStore, Focus, GitData, HostData, ReviewTarget};

/// The terminal view's half of a [`ReviewTarget`].
pub(crate) trait ReviewTargetExt {
    /// What `#N`, `!N`, mentions and commit ids in the request's text mean:
    /// the host the request lives on (the Overview, the composers' previews).
    fn md_context(&self) -> crate::md_render::Context;
}

impl ReviewTargetExt for ReviewTarget {
    fn md_context(&self) -> crate::md_render::Context {
        crate::md_render::Context {
            xrefs: gitpoulpi_hosting::xrefs::XrefContext {
                family: Some(self.integration.kind.family()),
                web_base: gitpoulpi_hosting::integration_web_base(&self.integration)
                    .unwrap_or_default(),
                project: self.project.clone(),
                ..Default::default()
            },
            ..Default::default()
        }
    }
}

pub(crate) use gitpoulpi_link::review_model::{Item, Place, Sev};
