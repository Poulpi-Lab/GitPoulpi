//! What the review of a pull / merge request holds: the request it is about,
//! where its pending review is kept, what the host and the repository
//! answered, and the focus a page can have. A frontend's review view owns the
//! values; the types are the same for every frontend.

use std::path::PathBuf;

use gitpoulpi_core::domain::{CommitNode, Oid};
use gitpoulpi_hosting::{
    Integration, ItemMeta, PullRequestDetail, ReviewComment, ReviewFile, ReviewThread,
};

/// The request under review and where it lives.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReviewTarget {
    pub integration: Integration,
    /// Every configured profile (the token source reads them all).
    pub all: Vec<Integration>,
    /// `owner/name`, `group/sub/proj`.
    pub project: String,
    pub number: u64,
    /// `#12` / `!53`.
    pub iid_label: String,
    pub title: String,
    /// The git remote the request's refs are fetched from.
    pub remote: String,
    pub web_url: String,
    pub source_branch: String,
    pub target_branch: String,
    /// The patch set the reviewer picked (Gerrit); `None`: the newest.
    pub patch_set: Option<u32>,
    /// The older patch set the diff starts from (Gerrit "compare with");
    /// `None`: the target branch.
    pub compare_patch_set: Option<u32>,
    /// The repository the work tree belongs to (the draft file lives there);
    /// `None`: reviewed through the host's API, no clone.
    pub repo_root: Option<PathBuf>,
}

impl ReviewTarget {
    /// The request as the conversation widget names it: the Overview shows
    /// the same timeline an issue has.
    pub fn item_target(&self) -> crate::item_target::ItemTarget {
        crate::item_target::ItemTarget {
            integration: self.integration.clone(),
            all: self.all.clone(),
            project: self.project.clone(),
            key: gitpoulpi_hosting::conversation::ItemKey::pull_request(self.number),
            reference: self.iid_label.clone(),
            title: self.title.clone(),
            web_url: self.web_url.clone(),
            repo_root: self.repo_root.clone(),
            bases: Default::default(),
        }
    }

    /// `#12 Fix the thing`, what pickers and the status line call it.
    pub fn caption(&self) -> String {
        format!("{} {}", self.iid_label, self.title)
    }

    /// `host/project`: what `remote-reviews.toml` files a request under.
    pub fn remote_key(&self) -> String {
        gitpoulpi_session::RemoteReviews::project_key(&self.integration.url, &self.project)
    }

    /// The key the local draft is stored under.
    pub fn draft_key(&self) -> String {
        gitpoulpi_session::ReviewDraft::key(&self.integration.id, &self.project, self.number)
    }
}

/// Where the pending review of a request is kept between runs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DraftStore {
    /// The repository's own state file.
    Repo(PathBuf),
    /// `remote-reviews.toml`, for a request of a repository that is not
    /// cloned (`None`: no config directory, nothing is kept).
    Remote(Option<PathBuf>),
}

impl DraftStore {
    pub fn load(&self, target: &ReviewTarget) -> gitpoulpi_session::ReviewDraft {
        let key = target.draft_key();
        match self {
            Self::Repo(root) => {
                let path = crate::repo_state_path::repo_state_path(root);
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
    pub fn save(&self, target: &ReviewTarget, draft: gitpoulpi_session::ReviewDraft) {
        let key = target.draft_key();
        match self {
            Self::Repo(root) => {
                let path = crate::repo_state_path::repo_state_path(root);
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
pub struct HostData {
    pub detail: PullRequestDetail,
    pub files: Vec<ReviewFile>,
    pub threads: Vec<ReviewThread>,
    pub conversation: Vec<ReviewComment>,
    pub meta: Option<ItemMeta>,
}

/// The repository's half: the fetched refs and the request's commits.
#[derive(Debug, Clone)]
pub struct GitData {
    pub head: Oid,
    /// What the Files diff starts from: the merge base, or the older patch
    /// set picked in "compare with".
    pub diff_base: Oid,
    /// `diff_base` is a patch set: the commit list is the head patch set
    /// alone, whose "parent" is that patch set.
    pub comparing: bool,
    /// The request's commits, newest first.
    pub commits: Vec<CommitNode>,
}

/// Which side of a page has the keyboard.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    List,
    Diff,
}

// -- AI suggestions ----------------------------------------------------------------

/// How much a suggestion matters (a frontend's own copy of the AI crate's
/// severity, so a build without the `ai` feature has it too).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sev {
    Info,
    Warning,
    Issue,
}

impl Sev {
    /// The translated word a card shows for it.
    pub fn word(self) -> &'static str {
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
pub struct Place {
    pub side: gitpoulpi_hosting::DiffSide,
    pub line: u32,
    /// For a context line, its number on the other side (GitLab).
    pub other_line: Option<u32>,
    pub start_line: Option<u32>,
}

/// One suggested review comment, waiting for the reviewer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item {
    pub id: u64,
    pub path: String,
    /// The line the model named; `0` when it named none.
    pub line: u32,
    pub severity: Sev,
    pub body: String,
    /// `line` is a line of the diff: the suggestion is drawn under it.
    /// Otherwise it is listed as about the file.
    pub anchored: bool,
    /// Where accepting puts it; `None` when the file is not in the diff.
    pub place: Option<Place>,
    /// Text put before the body when accepted ("(about line 40) ").
    pub prefix: String,
}
