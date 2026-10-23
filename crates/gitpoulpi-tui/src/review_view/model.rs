//! What the review view holds: the request it is about, what the host and
//! the repository answered, and the small enums the view and its palette
//! twins share.

use std::path::PathBuf;

use gitpoulpi_core::domain::{CommitNode, Oid};
use gitpoulpi_hosting::{
    Integration, ItemMeta, PullRequestDetail, ReviewComment, ReviewFile, ReviewThread,
    SubmittedReview,
};

/// The request under review and where it lives.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ReviewTarget {
    pub(crate) integration: Integration,
    /// Every configured profile (the token source reads them all).
    pub(crate) all: Vec<Integration>,
    /// `owner/name`, `group/sub/proj`.
    pub(crate) project: String,
    pub(crate) number: u64,
    /// `#12` / `!53`.
    pub(crate) iid_label: String,
    pub(crate) title: String,
    /// The git remote the request's refs are fetched from.
    pub(crate) remote: String,
    pub(crate) web_url: String,
    pub(crate) source_branch: String,
    pub(crate) target_branch: String,
    /// The patch set the reviewer picked (Gerrit); `None`: the newest.
    pub(crate) patch_set: Option<u32>,
    /// The older patch set the diff starts from (Gerrit "compare with");
    /// `None`: the target branch.
    pub(crate) compare_patch_set: Option<u32>,
    /// The repository the work tree belongs to (the draft file lives there);
    /// `None`: reviewed through the host's API, no clone.
    pub(crate) repo_root: Option<PathBuf>,
}

impl ReviewTarget {
    /// `#12 Fix the thing`, what pickers and the status line call it.
    pub(crate) fn caption(&self) -> String {
        format!("{} {}", self.iid_label, self.title)
    }

    /// `host/project`: what `remote-reviews.toml` files a request under.
    pub(crate) fn remote_key(&self) -> String {
        gitpoulpi_session::RemoteReviews::project_key(&self.integration.url, &self.project)
    }

    /// The key the local draft is stored under.
    pub(crate) fn draft_key(&self) -> String {
        gitpoulpi_session::ReviewDraft::key(&self.integration.id, &self.project, self.number)
    }
}

/// Where the pending review of a request is kept between runs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum DraftStore {
    /// The repository's own state file.
    Repo(PathBuf),
    /// `remote-reviews.toml`, for a request of a repository that is not
    /// cloned (`None`: no config directory, nothing is kept).
    Remote(Option<PathBuf>),
}

impl DraftStore {
    pub(crate) fn load(&self, target: &ReviewTarget) -> gitpoulpi_session::ReviewDraft {
        let key = target.draft_key();
        match self {
            Self::Repo(root) => {
                let path = crate::remotes_ui::repo_state_path(root);
                gitpoulpi_session::RepoState::load(&path)
                    .ok()
                    .and_then(|mut s| s.reviews.remove(&key))
                    .unwrap_or_default()
            }
            Self::Remote(Some(path)) => gitpoulpi_session::RemoteReviews::load(path)
                .drafts
                .remove(&target.remote_key())
                .and_then(|mut of_project| of_project.remove(&key))
                .unwrap_or_default(),
            Self::Remote(None) => Default::default(),
        }
    }

    /// Writes `draft` (an empty one removes the request's entry): a small
    /// file, the same load-edit-save the other per-repository choices use.
    pub(crate) fn save(&self, target: &ReviewTarget, draft: gitpoulpi_session::ReviewDraft) {
        let key = target.draft_key();
        match self {
            Self::Repo(root) => {
                let path = crate::remotes_ui::repo_state_path(root);
                let mut state = gitpoulpi_session::RepoState::load(&path).unwrap_or_default();
                if draft.is_empty() {
                    state.reviews.remove(&key);
                } else {
                    state.reviews.insert(key, draft);
                }
                let _ = state.save(&path);
            }
            Self::Remote(Some(path)) => {
                let mut file = gitpoulpi_session::RemoteReviews::load(path);
                file.set_draft(&target.remote_key(), &key, draft);
                let _ = file.save(path);
            }
            Self::Remote(None) => {}
        }
    }
}

/// The host's half of the request.
#[derive(Debug, Clone)]
pub(crate) struct HostData {
    pub(crate) detail: PullRequestDetail,
    pub(crate) files: Vec<ReviewFile>,
    pub(crate) threads: Vec<ReviewThread>,
    pub(crate) conversation: Vec<ReviewComment>,
    pub(crate) reviews: Vec<SubmittedReview>,
    pub(crate) meta: Option<ItemMeta>,
}

/// The repository's half: the fetched refs and the request's commits.
#[derive(Debug, Clone)]
pub(crate) struct GitData {
    pub(crate) head: Oid,
    /// What the Files diff starts from: the merge base, or the older patch
    /// set picked in "compare with".
    pub(crate) diff_base: Oid,
    /// `diff_base` is a patch set: the commit list is the head patch set
    /// alone, whose "parent" is that patch set.
    pub(crate) comparing: bool,
    /// The request's commits, newest first.
    pub(crate) commits: Vec<CommitNode>,
}

/// Which side of a page has the keyboard.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Focus {
    List,
    Diff,
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
