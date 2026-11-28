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

// -- AI suggestions ----------------------------------------------------------------

/// How much a suggestion matters (the view's own copy of the AI crate's
/// severity, so the view compiles without the `ai` feature).
#[cfg_attr(not(feature = "ai"), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Sev {
    Info,
    Warning,
    Issue,
}

impl Sev {
    pub(crate) fn word(self) -> &'static str {
        use gitpoulpi_i18n::msg::review_ai as ra;
        match self {
            Self::Info => ra::SEV_INFO,
            Self::Warning => ra::SEV_WARNING,
            Self::Issue => ra::SEV_ISSUE,
        }
        .get()
    }
}

/// Where an accepted suggestion becomes a comment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Place {
    pub(crate) side: gitpoulpi_hosting::DiffSide,
    pub(crate) line: u32,
    /// For a context line, its number on the other side (GitLab).
    pub(crate) other_line: Option<u32>,
    pub(crate) start_line: Option<u32>,
}

/// One suggested review comment, waiting for the reviewer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Item {
    pub(crate) id: u64,
    pub(crate) path: String,
    /// The line the model named; `0` when it named none.
    pub(crate) line: u32,
    pub(crate) severity: Sev,
    pub(crate) body: String,
    /// `line` is a line of the diff: the suggestion is drawn under it.
    /// Otherwise it is listed as about the file.
    pub(crate) anchored: bool,
    /// Where accepting puts it; `None` when the file is not in the diff.
    pub(crate) place: Option<Place>,
    /// Text put before the body when accepted ("(about line 40) ").
    pub(crate) prefix: String,
}
