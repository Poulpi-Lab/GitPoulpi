//! The local Markdown issue source as a [`HostingClient`].
//!
//! `project` is the folder (absolute, or relative to the caller's working
//! directory: the frontends resolve a repository-relative link against the
//! repository root with [`resolve_folder`] before they ask). Nothing here
//! opens a socket; the transport is never used.

use std::path::{Path, PathBuf};

use crate::client::{CreatedIssue, CreatedPullRequest, HostingClient, NewIssue, NewPullRequest};
use crate::integration::Integration;
use crate::metadata::{HostedLabel, HostedMilestone, IssueEdit, ItemMeta, MetaKind};
use crate::model::{
    Choice, HostedUser, IssueCreateOptions, IssueFilter, IssueState, IssueSummary, StateFilter,
    TokenCheck,
};
use crate::project::{HostedProject, HostedSshKey};
use crate::{Error, Result};

use super::store::{Issue, NewNote, Patch, Store};

/// The `file://` URL of a note (what "open" hands to the system).
pub fn file_url(path: &Path) -> String {
    let text = path.to_string_lossy().replace('\\', "/");
    let encoded: String = text
        .split('/')
        .map(|seg| {
            let mut out = String::new();
            for b in seg.bytes() {
                if b.is_ascii_alphanumeric()
                    || matches!(b, b'-' | b'_' | b'.' | b'~' | b':' | b'@' | b'+' | b',')
                {
                    out.push(b as char);
                } else {
                    out.push_str(&format!("%{b:02X}"));
                }
            }
            out
        })
        .collect::<Vec<_>>()
        .join("/");
    if encoded.starts_with('/') {
        format!("file://{encoded}")
    } else {
        format!("file:///{encoded}")
    }
}

/// The path a `file://` URL ([`file_url`]) names.
pub fn path_from_file_url(url: &str) -> Option<PathBuf> {
    let rest = url.strip_prefix("file://")?;
    let mut bytes = Vec::with_capacity(rest.len());
    let raw = rest.as_bytes();
    let mut i = 0;
    while i < raw.len() {
        if raw[i] == b'%' && i + 2 < raw.len() {
            let hex = std::str::from_utf8(&raw[i + 1..i + 3]).ok()?;
            bytes.push(u8::from_str_radix(hex, 16).ok()?);
            i += 3;
        } else {
            bytes.push(raw[i]);
            i += 1;
        }
    }
    let text = String::from_utf8(bytes).ok()?;
    // `file:///C:/x` carries the drive after the slash.
    let text = if text.len() > 2 && text.as_bytes()[2] == b':' && text.starts_with('/') {
        text[1..].to_string()
    } else {
        text
    };
    Some(PathBuf::from(text))
}

/// The note of issue `id` in `folder`, when there is one.
pub fn note_path(integration: &Integration, folder: &str, id: u64) -> Option<PathBuf> {
    store_for(integration, folder).get(id).ok().map(|i| i.path)
}

fn store_for(integration: &Integration, folder: &str) -> Store {
    let folder = if folder.trim().is_empty() {
        integration.url.as_str()
    } else {
        folder
    };
    Store::new(folder, integration.local.clone().unwrap_or_default())
}

pub(crate) struct LocalMarkdownClient {
    integration: Integration,
}

impl LocalMarkdownClient {
    pub(crate) fn new(integration: Integration) -> Self {
        Self { integration }
    }

    fn store(&self, project: &str) -> Store {
        store_for(&self.integration, project)
    }

    fn me(&self) -> String {
        let settings = self.integration.local.clone().unwrap_or_default();
        if !settings.me.trim().is_empty() {
            return settings.me.trim().to_string();
        }
        match &self.integration.user {
            Some(u) if !u.login.is_empty() => u.login.clone(),
            _ => self.integration.display_name().to_string(),
        }
    }

    fn summary(&self, issue: &Issue) -> IssueSummary {
        let settings = self.integration.local.clone().unwrap_or_default();
        let prefix = settings.reference_prefix.trim();
        let reference = if prefix.is_empty() {
            format!("#{}", issue.id)
        } else {
            format!("{prefix}-{}", issue.id)
        };
        let closed = issue.status.eq_ignore_ascii_case(settings.done());
        IssueSummary {
            branch_name: None,
            number: issue.id,
            id: issue.id.to_string(),
            reference: reference.clone(),
            kind_label: None,
            status_label: Some(issue.status.clone()),
            iid_label: reference,
            title: issue.title.clone(),
            state: if closed {
                IssueState::Closed
            } else {
                IssueState::Open
            },
            author: String::new(),
            assignees: issue.assignees.clone(),
            labels: issue.labels.clone(),
            web_url: file_url(&issue.path),
            updated_at: issue.updated.clone(),
            milestone: issue.milestone.clone(),
        }
    }

    /// The distinct milestone titles, sorted: the 1-based position is the
    /// id the pickers send back.
    fn milestone_titles(&self, project: &str) -> Result<Vec<String>> {
        Ok(self.store(project).used_values()?.milestones)
    }

    fn milestone_by_id(&self, project: &str, id: u64) -> Result<String> {
        let titles = self.milestone_titles(project)?;
        usize::try_from(id)
            .ok()
            .and_then(|i| i.checked_sub(1))
            .and_then(|i| titles.get(i).cloned())
            .ok_or_else(|| Error::Local(format!("no milestone {id} in this folder")))
    }
}

#[async_trait::async_trait]
impl HostingClient for LocalMarkdownClient {
    fn integration(&self) -> &Integration {
        &self.integration
    }

    async fn list_projects(&self) -> Result<Vec<HostedProject>> {
        let root = self.integration.url.trim().to_string();
        let mut folders = vec![root.clone()];
        let path = Path::new(&root);
        // An absolute folder (a vault) also offers its sub-folders.
        if path.is_absolute() {
            if let Ok(entries) = std::fs::read_dir(path) {
                let mut subs: Vec<String> = entries
                    .filter_map(|e| e.ok())
                    .filter(|e| e.path().is_dir())
                    .filter_map(|e| {
                        let name = e.file_name().to_string_lossy().into_owned();
                        (!name.starts_with('.')).then(|| e.path().to_string_lossy().into_owned())
                    })
                    .collect();
                subs.sort();
                folders.extend(subs.into_iter().take(200));
            }
        }
        Ok(folders
            .into_iter()
            .map(|full| HostedProject {
                integration_id: self.integration.id.clone(),
                id: full.clone(),
                name: Path::new(&full)
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_else(|| full.clone()),
                full_name: full.clone(),
                description: None,
                default_branch: None,
                ssh_url: None,
                https_url: None,
                web_url: Some(file_url(Path::new(&full))),
                private: true,
            })
            .collect())
    }

    async fn list_ssh_keys(&self) -> Result<Vec<HostedSshKey>> {
        Err(self.unsupported("list_ssh_keys"))
    }

    async fn upload_ssh_key(&self, _title: &str, _public_key: &str) -> Result<HostedSshKey> {
        Err(self.unsupported("upload_ssh_key"))
    }

    async fn create_pull_request(&self, _req: &NewPullRequest) -> Result<CreatedPullRequest> {
        Err(self.unsupported("create_pull_request"))
    }

    async fn current_user(&self) -> Result<HostedUser> {
        Ok(HostedUser {
            login: self.me(),
            name: String::new(),
            id: String::new(),
            web_url: None,
        })
    }

    async fn check_token(&self) -> Result<TokenCheck> {
        // No token: "connected" means the folder is readable.
        let dir = Path::new(self.integration.url.trim());
        if self.integration.url.trim().is_empty() {
            return Err(Error::Local("no folder is set".into()));
        }
        if dir.is_absolute() && !dir.is_dir() {
            return Err(Error::Local(format!("`{}` is not a folder", dir.display())));
        }
        Ok(TokenCheck {
            user: self.current_user().await?,
            scopes: None,
            missing_scopes: Vec::new(),
            expires_at: None,
        })
    }

    async fn list_issues(&self, project: &str, filter: &IssueFilter) -> Result<Vec<IssueSummary>> {
        let store = self.store(project);
        let settings = self.integration.local.clone().unwrap_or_default();
        let done = settings.done().to_string();
        let me = self.me();
        let wanted: Vec<String> = filter
            .query
            .as_deref()
            .unwrap_or("")
            .split(',')
            .map(|s| s.trim().to_lowercase())
            .filter(|s| !s.is_empty())
            .collect();
        let mut rows: Vec<Issue> = store
            .list()?
            .into_iter()
            .filter(|i| {
                let closed = i.status.eq_ignore_ascii_case(&done);
                match filter.state {
                    StateFilter::Open => !closed,
                    StateFilter::Closed => closed,
                    StateFilter::All => true,
                }
            })
            .filter(|i| {
                !filter.assigned_to_me || i.assignees.iter().any(|a| a.eq_ignore_ascii_case(&me))
            })
            .filter(|i| wanted.is_empty() || wanted.contains(&i.status.to_lowercase()))
            .collect();
        // Newest first, like the hosts.
        rows.sort_by_key(|a| std::cmp::Reverse(a.id));
        Ok(rows.iter().map(|i| self.summary(i)).collect())
    }

    async fn create_issue(&self, req: &NewIssue) -> Result<CreatedIssue> {
        let store = self.store(&req.project);
        let milestone = match req.milestone {
            Some(id) => Some(self.milestone_by_id(&req.project, id)?),
            None => None,
        };
        let made = store.create(&NewNote {
            title: req.title.clone(),
            body: req.body.clone(),
            assignees: req.assignees.clone(),
            labels: req.labels.clone(),
            milestone,
            status: req.list_id.clone().unwrap_or_default(),
        })?;
        let summary = self.summary(&made);
        Ok(CreatedIssue {
            number: made.id,
            web_url: summary.web_url,
            reference: summary.reference,
        })
    }

    async fn issue_create_options(&self, project: &str) -> Result<IssueCreateOptions> {
        let statuses = self.store(project).used_values()?.statuses;
        Ok(IssueCreateOptions {
            types: Vec::new(),
            default_list: statuses.first().cloned(),
            lists: statuses
                .into_iter()
                .map(|s| Choice {
                    id: s.clone(),
                    label: s,
                })
                .collect(),
            default_type: None,
        })
    }

    async fn project_members(&self, project: &str, query: &str) -> Result<Vec<HostedUser>> {
        let mut logins = self.store(project).used_values()?.assignees;
        let me = self.me();
        if !logins.iter().any(|l| l.eq_ignore_ascii_case(&me)) {
            logins.insert(0, me);
        }
        let q = query.trim().to_lowercase();
        Ok(logins
            .into_iter()
            .filter(|l| q.is_empty() || l.to_lowercase().contains(&q))
            .map(|login| HostedUser {
                login,
                ..HostedUser::default()
            })
            .collect())
    }

    async fn project_labels(&self, project: &str) -> Result<Vec<HostedLabel>> {
        Ok(self
            .store(project)
            .used_values()?
            .labels
            .into_iter()
            .map(|name| HostedLabel {
                name,
                ..HostedLabel::default()
            })
            .collect())
    }

    async fn project_milestones(&self, project: &str) -> Result<Vec<HostedMilestone>> {
        Ok(self
            .milestone_titles(project)?
            .into_iter()
            .enumerate()
            .map(|(i, title)| HostedMilestone {
                id: i as u64 + 1,
                title,
                due_on: None,
                state: "open".into(),
            })
            .collect())
    }

    async fn item_meta(&self, project: &str, number: u64, _kind: MetaKind) -> Result<ItemMeta> {
        let issue = self.store(project).get(number)?;
        let titles = self.milestone_titles(project)?;
        let milestone = issue.milestone.as_ref().and_then(|m| {
            titles.iter().position(|t| t == m).map(|i| HostedMilestone {
                id: i as u64 + 1,
                title: m.clone(),
                due_on: None,
                state: "open".into(),
            })
        });
        Ok(ItemMeta {
            assignees: issue.assignees,
            reviewers: Vec::new(),
            labels: issue.labels,
            milestone,
        })
    }

    async fn update_issue(&self, project: &str, number: u64, edit: &IssueEdit) -> Result<()> {
        let milestone = match edit.milestone {
            Some(Some(id)) => Some(Some(self.milestone_by_id(project, id)?)),
            Some(None) => Some(None),
            None => None,
        };
        self.store(project).update(
            number,
            &Patch {
                assignees: edit.assignees.clone(),
                labels: edit.labels.clone(),
                milestone,
                ..Patch::default()
            },
        )?;
        Ok(())
    }

    fn board_source(&self) -> crate::board::BoardSource {
        crate::board::BoardSource::LocalStatus
    }

    async fn issue_statuses(&self, project: &str) -> Result<Vec<String>> {
        Ok(self.store(project).used_values()?.statuses)
    }

    async fn set_issue_status(&self, project: &str, number: u64, status: &str) -> Result<()> {
        self.store(project).update(
            number,
            &Patch {
                status: Some(status.to_string()),
                ..Patch::default()
            },
        )?;
        Ok(())
    }

    async fn set_issue_state(&self, project: &str, number: u64, closed: bool) -> Result<()> {
        let settings = self.integration.local.clone().unwrap_or_default();
        let status = if closed {
            settings.done().to_string()
        } else {
            settings.open_status()
        };
        self.set_issue_status(project, number, &status).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gitpoulpi_config::{IntegrationKind, LocalMarkdownSettings};

    fn block_on<T>(f: impl std::future::Future<Output = T>) -> T {
        tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap()
            .block_on(f)
    }

    fn client(dir: &Path) -> LocalMarkdownClient {
        let mut i = Integration::new(
            "notes",
            IntegrationKind::LocalMarkdown,
            dir.to_string_lossy(),
        );
        i.local = Some(LocalMarkdownSettings {
            me: "kai".into(),
            ..LocalMarkdownSettings::default()
        });
        LocalMarkdownClient::new(i)
    }

    #[test]
    fn create_list_edit_close_reopen() {
        let dir = tempfile::tempdir().unwrap();
        let c = client(dir.path());
        let p = dir.path().to_string_lossy().into_owned();
        let made = block_on(c.create_issue(&NewIssue {
            project: p.clone(),
            title: "First".into(),
            body: "Body".into(),
            assignees: vec!["kai".into()],
            labels: vec!["bug".into()],
            ..NewIssue::default()
        }))
        .unwrap();
        assert_eq!((made.number, made.reference.as_str()), (1, "#1"));
        block_on(c.create_issue(&NewIssue {
            project: p.clone(),
            title: "Second".into(),
            ..NewIssue::default()
        }))
        .unwrap();
        let open = IssueFilter::default();
        let list = block_on(c.list_issues(&p, &open)).unwrap();
        assert_eq!(list.iter().map(|i| i.number).collect::<Vec<_>>(), [2, 1]);
        let mine = IssueFilter {
            assigned_to_me: true,
            ..IssueFilter::default()
        };
        assert_eq!(block_on(c.list_issues(&p, &mine)).unwrap().len(), 1);

        block_on(c.update_issue(
            &p,
            2,
            &IssueEdit {
                labels: Some(vec!["ui".into()]),
                assignees: Some(vec!["ana".into()]),
                ..IssueEdit::default()
            },
        ))
        .unwrap();
        let meta = block_on(c.item_meta(&p, 2, MetaKind::Issue)).unwrap();
        assert_eq!(
            (meta.labels, meta.assignees),
            (vec!["ui".into()], vec!["ana".into()])
        );
        let labels = block_on(c.project_labels(&p)).unwrap();
        assert_eq!(labels.len(), 2);

        block_on(c.set_issue_state(&p, 1, true)).unwrap();
        assert_eq!(block_on(c.list_issues(&p, &open)).unwrap().len(), 1);
        let closed = IssueFilter {
            state: StateFilter::Closed,
            ..IssueFilter::default()
        };
        let done = block_on(c.list_issues(&p, &closed)).unwrap();
        assert_eq!(done[0].state, IssueState::Closed);
        block_on(c.set_issue_state(&p, 1, false)).unwrap();
        assert_eq!(block_on(c.list_issues(&p, &open)).unwrap().len(), 2);
        let all = IssueFilter {
            state: StateFilter::All,
            query: Some("doing".into()),
            ..IssueFilter::default()
        };
        assert!(block_on(c.list_issues(&p, &all)).unwrap().is_empty());
    }

    #[test]
    fn milestones_round_trip_through_their_position() {
        let dir = tempfile::tempdir().unwrap();
        let c = client(dir.path());
        let p = dir.path().to_string_lossy().into_owned();
        std::fs::write(
            dir.path().join("1-a.md"),
            "---\nid: 1\ntitle: A\nmilestone: v0.2\n---\n",
        )
        .unwrap();
        std::fs::write(
            dir.path().join("2-b.md"),
            "---\nid: 2\ntitle: B\nmilestone: v0.1\n---\n",
        )
        .unwrap();
        let ms = block_on(c.project_milestones(&p)).unwrap();
        assert_eq!(
            ms.iter().map(|m| m.title.as_str()).collect::<Vec<_>>(),
            ["v0.1", "v0.2"]
        );
        block_on(c.update_issue(
            &p,
            2,
            &IssueEdit {
                milestone: Some(Some(2)),
                ..IssueEdit::default()
            },
        ))
        .unwrap();
        let meta = block_on(c.item_meta(&p, 2, MetaKind::Issue)).unwrap();
        assert_eq!(meta.milestone.unwrap().title, "v0.2");
        block_on(c.update_issue(
            &p,
            2,
            &IssueEdit {
                milestone: Some(None),
                ..IssueEdit::default()
            },
        ))
        .unwrap();
        assert!(block_on(c.item_meta(&p, 2, MetaKind::Issue))
            .unwrap()
            .milestone
            .is_none());
    }

    #[test]
    fn urls_and_folders() {
        assert_eq!(file_url(Path::new("/a b/c#1.md")), "file:///a%20b/c%231.md");
        assert_eq!(
            path_from_file_url("file:///a%20b/c%231.md"),
            Some(PathBuf::from("/a b/c#1.md"))
        );
        assert_eq!(
            path_from_file_url(&file_url(Path::new("/x/\u{e9}t\u{e9}.md"))),
            Some(PathBuf::from("/x/\u{e9}t\u{e9}.md"))
        );
        assert_eq!(
            crate::resolve_local_folder(Path::new("/repo"), "docs/issues"),
            PathBuf::from("/repo/docs/issues")
        );
        assert_eq!(
            crate::resolve_local_folder(Path::new("/repo"), "/vault/Issues"),
            PathBuf::from("/vault/Issues")
        );
    }
}
