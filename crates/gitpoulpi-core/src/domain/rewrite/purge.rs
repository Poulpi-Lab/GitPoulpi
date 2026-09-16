//! Purging the old history of a rewrite (the history-rewrite design,
//! "Purge (S15)"): the request, what still keeps the old objects alive, the
//! answer, the confirmation a frontend shows first and the checklist for the
//! host side.
//!
//! Pure data and text. The work is
//! [`GitBackend::rewrite_purge`](crate::GitBackend::rewrite_purge); the
//! daemon turns journal entries into [`PurgeEntry`]s and marks them
//! `Purged` afterwards.

use serde::{Deserialize, Serialize};

use crate::journal::RefChange;

/// One rewrite journal entry as the purge needs it: the refs it moved (the
/// old values are the history to get rid of, the new ones what stays) and
/// the names of its undo pins.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PurgeEntry {
    pub id: u64,
    pub label: String,
    pub refs: Vec<RefChange>,
    /// `refs/gitpoulpi/undo/...` names to delete.
    pub pins: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct PurgeRequest {
    pub entries: Vec<PurgeEntry>,
    /// Compute everything (the old objects, what blocks, the reflogs that
    /// would go) and change nothing.
    #[serde(default)]
    pub dry_run: bool,
}

/// What still holds the old history. `Branch`..`Other` are refs; the rest are
/// not.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PurgeBlockerKind {
    /// A local branch that was not rewritten (outside the scope).
    Branch,
    Tag,
    /// A remote-tracking ref (or the tag mirror): `fetch` after the push
    /// moves it.
    RemoteTracking,
    /// A stash based on, or made of, old commits.
    Stash,
    /// A worktree's detached HEAD (`name` is its path), or the main
    /// worktree's.
    Worktree,
    /// An undo pin of a journal entry that is not part of this purge.
    UndoPin,
    Notes,
    /// A `refs/replace/*` ref.
    Replace,
    /// Anything else under `refs/` (`refs/original`, `refs/backup`, ...).
    Other,
}

impl PurgeBlockerKind {
    /// The `snake_case` word (i18n key suffix, JSON value).
    pub fn code(self) -> &'static str {
        match self {
            Self::Branch => "branch",
            Self::Tag => "tag",
            Self::RemoteTracking => "remote_tracking",
            Self::Stash => "stash",
            Self::Worktree => "worktree",
            Self::UndoPin => "undo_pin",
            Self::Notes => "notes",
            Self::Replace => "replace",
            Self::Other => "other",
        }
    }

    /// What to do about it, in English (CLI and MCP).
    pub fn advice(self) -> &'static str {
        match self {
            Self::Branch => {
                "rewrite this branch too, or delete it if it is not needed: it still holds the old commits"
            }
            Self::Tag => "move or delete this tag: it still points into the old history",
            Self::RemoteTracking => {
                "push the rewritten branches, then fetch: this remote-tracking ref still points at the old history"
            }
            Self::Stash => {
                "drop this stash (it is work: apply it first if you need it) or keep it and accept that the old history stays in this clone"
            }
            Self::Worktree => {
                "check out a rewritten branch in that worktree: its HEAD is on the old history"
            }
            Self::UndoPin => {
                "this undo entry also keeps the old history: purge it too (--entry) or let it fall off the undo history"
            }
            Self::Notes => "rewrite or delete this notes ref: it still reaches old commits",
            Self::Replace => "delete this replace ref: it points into the old history",
            Self::Other => "delete this ref if it is not needed: it still reaches old commits",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PurgeBlocker {
    pub kind: PurgeBlockerKind,
    /// The ref name, the stash entry (`stash@{1}`) or the worktree path.
    pub name: String,
    /// How many of the old commits it reaches.
    pub commits: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PurgeStatus {
    /// A dry run: nothing blocks, a real run would go ahead.
    Ready,
    /// Something keeps the old history alive: nothing was changed.
    Blocked,
    /// Done, and every old object is gone from this clone (verified).
    Purged,
    /// Done, but some old objects are still present (see `remaining`).
    Incomplete,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PurgeOutcome {
    pub status: PurgeStatus,
    pub blockers: Vec<PurgeBlocker>,
    /// What the rewritten-away history is made of (reachable from the old
    /// tips, not from the new ones).
    pub old_commits: u32,
    pub old_blobs: u32,
    pub old_objects: u32,
    /// Reflogs whose entries go (and how many entries they hold).
    pub reflogs: Vec<String>,
    pub reflog_entries: u32,
    pub pins_dropped: u32,
    pub runs_removed: u32,
    /// Old objects still in the object store after the run (0 when
    /// `Purged`).
    pub remaining: u32,
}

impl PurgeOutcome {
    pub fn is_purged(&self) -> bool {
        self.status == PurgeStatus::Purged
    }
}

/// The steps of a run, for progress.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PurgeStep {
    Checking,
    Pins,
    Reflogs,
    Collecting,
    Verifying,
}

impl PurgeStep {
    pub fn name(self) -> &'static str {
        match self {
            Self::Checking => "Checking",
            Self::Pins => "Dropping undo pins",
            Self::Reflogs => "Expiring reflogs",
            Self::Collecting => "Collecting garbage",
            Self::Verifying => "Verifying",
        }
    }

    /// Where the step starts on a 0..100 bar.
    pub fn percent(self) -> u8 {
        match self {
            Self::Checking => 0,
            Self::Pins => 30,
            Self::Reflogs => 40,
            Self::Collecting => 55,
            Self::Verifying => 90,
        }
    }
}

/// Reports the step a purge is in. A purge cannot be stopped once it left
/// `Checking`.
pub type PurgeProgressSink<'a> = &'a mut dyn FnMut(PurgeStep);

/// One entry of the confirmation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PurgeEntryInfo {
    pub id: u64,
    pub label: String,
    /// `true`: a history rewrite. `false`: another operation whose undo pins
    /// keep the old history alive, so its undo goes with it.
    #[serde(default = "yes")]
    pub rewrite: bool,
}

fn yes() -> bool {
    true
}

/// What a frontend shows before a purge (`RewritePurgeCheck`): everything
/// the run would do and lose, what blocks it, and the host-side steps. The
/// run takes the same `entries`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PurgeCheck {
    /// The journal entries that become `Purged` (undo of each is refused
    /// afterwards).
    pub entries: Vec<PurgeEntryInfo>,
    /// Chosen entries that were purged already.
    pub already_purged: Vec<u64>,
    pub outcome: PurgeOutcome,
    /// One checklist per remote.
    pub hosts: Vec<HostChecklist>,
    /// The confirmation sentence (English; CLI and MCP).
    pub prompt: String,
}

impl PurgeCheck {
    pub fn can_run(&self) -> bool {
        !self.entries.is_empty() && self.outcome.blockers.is_empty()
    }
}

/// "Undo will no longer be possible for 3 operations: ..." and what else is
/// lost.
pub fn confirmation_text(entries: &[PurgeEntryInfo], outcome: &PurgeOutcome) -> String {
    let n = entries.len();
    let list = entries
        .iter()
        .map(|e| format!("#{} {}", e.id, e.label))
        .collect::<Vec<_>>()
        .join("; ");
    let mut text = format!(
        "Purging deletes the old history of this rewrite from this clone for good: {} commit(s) and {} file version(s) \
         are removed from the object store, and {} reflog entr{} of {} reflog(s) are dropped. \
         Undo will no longer be possible for {n} operation{}: {list}.",
        outcome.old_commits,
        outcome.old_blobs,
        outcome.reflog_entries,
        if outcome.reflog_entries == 1 { "y" } else { "ies" },
        outcome.reflogs.len(),
        if n == 1 { "" } else { "s" },
    );
    text.push_str(
        " Purging cannot be stopped once it starts, and it cannot be undone. \
         It does not touch any remote: a secret that was pushed is leaked, rotate it first.",
    );
    text
}

// ------------------------------------------------------------- the host side

/// Which hosting service a remote URL belongs to (by its host name; a
/// self-managed instance whose name does not say so is `Other`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HostKind {
    GitHub,
    GitLab,
    AzureDevOps,
    Bitbucket,
    Other,
}

impl HostKind {
    pub fn of_url(url: &str) -> Self {
        let lower = url.to_ascii_lowercase();
        if lower.contains("github") {
            Self::GitHub
        } else if lower.contains("gitlab") {
            Self::GitLab
        } else if lower.contains("dev.azure.com") || lower.contains("visualstudio.com") {
            Self::AzureDevOps
        } else if lower.contains("bitbucket") {
            Self::Bitbucket
        } else {
            Self::Other
        }
    }
}

/// One thing to do on the host after a purge. `code()` is the i18n key
/// suffix.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HostStep {
    RotateSecret,
    PushRewritten,
    TellCollaborators,
    GithubSupport,
    GithubForks,
    GithubRecreate,
    GitlabCleanup,
    GitlabExpire,
    GitlabRecreate,
    AzureSupport,
    AzureRecreate,
    BitbucketSupport,
    BitbucketForks,
    BitbucketRecreate,
    ServerGc,
}

impl HostStep {
    pub fn code(self) -> &'static str {
        match self {
            Self::RotateSecret => "rotate_secret",
            Self::PushRewritten => "push_rewritten",
            Self::TellCollaborators => "tell_collaborators",
            Self::GithubSupport => "github_support",
            Self::GithubForks => "github_forks",
            Self::GithubRecreate => "github_recreate",
            Self::GitlabCleanup => "gitlab_cleanup",
            Self::GitlabExpire => "gitlab_expire",
            Self::GitlabRecreate => "gitlab_recreate",
            Self::AzureSupport => "azure_support",
            Self::AzureRecreate => "azure_recreate",
            Self::BitbucketSupport => "bitbucket_support",
            Self::BitbucketForks => "bitbucket_forks",
            Self::BitbucketRecreate => "bitbucket_recreate",
            Self::ServerGc => "server_gc",
        }
    }

    /// The step in English (CLI, MCP, the confirmation).
    pub fn text(self) -> &'static str {
        match self {
            Self::RotateSecret => {
                "Revoke the secret and issue a new one first: a secret that was pushed must be treated as leaked, whatever you remove afterwards."
            }
            Self::PushRewritten => {
                "Push the rewritten branches and tags (`gitpoulpi rewrite push`), so the remote refs no longer name the old commits."
            }
            Self::TellCollaborators => {
                "Send the note to everyone who has a clone (`gitpoulpi rewrite note`): their copies still hold the old history and must be reset or re-cloned."
            }
            Self::GithubSupport => {
                "GitHub keeps old commits reachable by their id, in cached views and in pull request refs (refs/pull/*) after a force push. Follow \"Removing sensitive data from a repository\" in GitHub's documentation and contact GitHub Support (https://support.github.com/contact) to remove the cached views and the references in pull requests and to run garbage collection."
            }
            Self::GithubForks => {
                "Forks keep the old history: ask the owners of the forks to delete them or rewrite theirs, or ask GitHub Support about the fork network."
            }
            Self::GithubRecreate => {
                "If you can lose the pull requests, issues and stars: create a new repository from the cleaned clone and delete the old one. It is the only way to be sure the old objects are gone without support."
            }
            Self::GitlabCleanup => {
                "GitLab: open Settings > Repository > Maintenance (\"Clean up repository\" in older versions) and upload the commit map (`gitpoulpi rewrite maps --out DIR`, the file called commit-map): GitLab then removes the internal refs (merge request, pipeline and environment refs) that keep the old commits."
            }
            Self::GitlabExpire => {
                "GitLab: the cleanup runs in the background and objects are only removed after housekeeping. On a self-managed instance an administrator can run housekeeping on the project (Admin > Projects > the project > Run housekeeping), or wait for the next scheduled one."
            }
            Self::GitlabRecreate => {
                "GitLab: if the cleanup is not enough, export nothing you need from the old project, create a new project from the cleaned clone and delete the old one."
            }
            Self::AzureSupport => {
                "Azure DevOps: a force push does not delete the old commits from the service, and there is no setting that purges them. Open a support request with Microsoft (Azure DevOps support) naming the project, the repository and the commit ids, and ask for them to be removed."
            }
            Self::AzureRecreate => {
                "Azure DevOps: or create a new repository from the cleaned clone, delete the old one, then delete it from the recycle bin as well (Project settings > Repositories > Deleted repositories) so it is gone for good."
            }
            Self::BitbucketSupport => {
                "Bitbucket Cloud: old commits stay reachable through pull requests and by their id after a force push. Contact Atlassian Support and ask for garbage collection of the repository. On Bitbucket Data Center an administrator can run garbage collection from the repository's maintenance settings."
            }
            Self::BitbucketForks => {
                "Bitbucket: forks keep the old history: delete them or have their owners rewrite them."
            }
            Self::BitbucketRecreate => {
                "Bitbucket: or create a new repository from the cleaned clone and delete the old one (this loses pull requests and issues)."
            }
            Self::ServerGc => {
                "Self-hosted or unknown host: in the bare repository on the server run `git reflog expire --expire=now --all` and `git gc --prune=now`, after deleting any hidden refs (for example refs/pull/*) that still point at the old commits. Then have everyone re-clone."
            }
        }
    }
}

/// The steps for one host, in order. The first three are the same
/// everywhere.
pub fn host_steps(host: HostKind) -> Vec<HostStep> {
    let mut steps = vec![HostStep::RotateSecret, HostStep::PushRewritten];
    match host {
        HostKind::GitHub => steps.extend([
            HostStep::GithubSupport,
            HostStep::GithubForks,
            HostStep::GithubRecreate,
        ]),
        HostKind::GitLab => steps.extend([
            HostStep::GitlabCleanup,
            HostStep::GitlabExpire,
            HostStep::GitlabRecreate,
        ]),
        HostKind::AzureDevOps => steps.extend([HostStep::AzureSupport, HostStep::AzureRecreate]),
        HostKind::Bitbucket => steps.extend([
            HostStep::BitbucketSupport,
            HostStep::BitbucketForks,
            HostStep::BitbucketRecreate,
        ]),
        HostKind::Other => steps.push(HostStep::ServerGc),
    }
    steps.push(HostStep::TellCollaborators);
    steps
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostChecklist {
    pub remote: String,
    pub host: HostKind,
    pub steps: Vec<HostStep>,
}

impl HostChecklist {
    pub fn new(remote: impl Into<String>, url: &str) -> Self {
        let host = HostKind::of_url(url);
        Self {
            remote: remote.into(),
            host,
            steps: host_steps(host),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hosts_are_told_apart_by_their_name() {
        assert_eq!(HostKind::of_url("git@github.com:a/b.git"), HostKind::GitHub);
        assert_eq!(
            HostKind::of_url("https://gitlab.example.org/a/b"),
            HostKind::GitLab
        );
        assert_eq!(
            HostKind::of_url("https://dev.azure.com/o/p/_git/r"),
            HostKind::AzureDevOps
        );
        assert_eq!(
            HostKind::of_url("https://o.visualstudio.com/p/_git/r"),
            HostKind::AzureDevOps
        );
        assert_eq!(
            HostKind::of_url("https://bitbucket.org/a/b.git"),
            HostKind::Bitbucket
        );
        assert_eq!(HostKind::of_url("/srv/git/x.git"), HostKind::Other);
    }

    #[test]
    fn every_checklist_starts_with_rotating_and_ends_with_telling_people() {
        for host in [
            HostKind::GitHub,
            HostKind::GitLab,
            HostKind::AzureDevOps,
            HostKind::Bitbucket,
            HostKind::Other,
        ] {
            let steps = host_steps(host);
            assert_eq!(steps.first(), Some(&HostStep::RotateSecret));
            assert_eq!(steps.last(), Some(&HostStep::TellCollaborators));
            assert!(steps.iter().all(|s| !s.text().contains('\u{2014}')));
        }
    }

    #[test]
    fn the_confirmation_names_every_entry() {
        let outcome = PurgeOutcome {
            status: PurgeStatus::Ready,
            blockers: Vec::new(),
            old_commits: 12,
            old_blobs: 3,
            old_objects: 40,
            reflogs: vec!["HEAD".into(), "refs/heads/main".into()],
            reflog_entries: 9,
            pins_dropped: 0,
            runs_removed: 0,
            remaining: 0,
        };
        let entries = vec![
            PurgeEntryInfo {
                id: 4,
                label: "Rewrite history: 12 commits".into(),
                rewrite: true,
            },
            PurgeEntryInfo {
                id: 7,
                label: "Rewrite history: 2 commits".into(),
                rewrite: true,
            },
        ];
        let text = confirmation_text(&entries, &outcome);
        assert!(text.contains("Undo will no longer be possible for 2 operations"));
        assert!(text.contains("#4 Rewrite history: 12 commits"));
        assert!(text.contains("#7 Rewrite history: 2 commits"));
        assert!(text.contains("9 reflog entries"));
    }
}
