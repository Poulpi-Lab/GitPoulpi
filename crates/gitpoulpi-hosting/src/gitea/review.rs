//! In-app review on Gitea and Forgejo.
//!
//! Gitea has no thread object: a review holds code comments
//! (`pulls/{n}/reviews/{id}/comments`) and the web UI groups the ones on the
//! same line into a conversation. This module does the same, so a thread is
//! the comments of one `(path, side, line)` and its id is
//! `"{root comment id}|{n|o}|{line}|{path}"`: the root id is what resolve and
//! reply take, the rest lets a reply fall back to a one-comment review on
//! servers without the reply endpoint (Forgejo 11).
//!
//! A comment's position: `position` is the line in the new file, and
//! `original_position` the line in the old one for a comment on a removed
//! line (observed on both servers: new-side `(2, 0)`, old-side `(0, 2)`).
//! The API has no "outdated" flag; a comment written on another commit than
//! the head is reported outdated, so it is listed under its file instead of
//! being drawn on a line that may have moved.

use std::collections::BTreeMap;

use serde::Deserialize;
use serde_json::{json, Value};

use super::{GiteaClient, Login, PullRow};
use crate::model::ChecksState;
use crate::review::model::{
    DiffSide, FileStatus, MergeState, PullRequestDetail, ReviewCapabilities, ReviewComment,
    ReviewEvent, ReviewFile, ReviewOutcome, ReviewState, ReviewSubmission, ReviewThread,
    SubmittedReview,
};
use crate::review::{invalid, validate_submission};
use crate::{Error, Result};
use gitpoulpi_config::IntegrationKind;

#[derive(Deserialize)]
struct IssueComment {
    id: u64,
    #[serde(default)]
    body: String,
    #[serde(default)]
    created_at: String,
    user: Option<Login>,
    html_url: Option<String>,
}

impl IssueComment {
    fn into_comment(self) -> ReviewComment {
        ReviewComment {
            id: self.id.to_string(),
            author: self.user.map(|u| u.login).unwrap_or_default(),
            body: self.body,
            created_at: self.created_at,
            in_reply_to: None,
            web_url: self.html_url.filter(|u| !u.is_empty()),
        }
    }
}

#[derive(Deserialize)]
struct FileRow {
    filename: String,
    #[serde(default)]
    status: String,
    #[serde(default)]
    additions: u32,
    #[serde(default)]
    deletions: u32,
    previous_filename: Option<String>,
}

#[derive(Deserialize)]
struct ReviewRow {
    id: u64,
    #[serde(default)]
    comments_count: u32,
    #[serde(default)]
    state: String,
    #[serde(default)]
    body: String,
    #[serde(default)]
    dismissed: bool,
    #[serde(default)]
    submitted_at: String,
    user: Option<Login>,
}

#[derive(Deserialize)]
struct CodeComment {
    id: u64,
    #[serde(default)]
    body: String,
    #[serde(default)]
    created_at: String,
    user: Option<Login>,
    resolver: Option<Login>,
    #[serde(default)]
    path: String,
    #[serde(default)]
    commit_id: String,
    #[serde(default)]
    position: u32,
    #[serde(default)]
    original_position: u32,
    html_url: Option<String>,
}

impl CodeComment {
    fn side_and_line(&self) -> Option<(DiffSide, u32)> {
        if self.position > 0 {
            Some((DiffSide::New, self.position))
        } else if self.original_position > 0 {
            Some((DiffSide::Old, self.original_position))
        } else {
            None
        }
    }

    fn into_comment(self) -> ReviewComment {
        ReviewComment {
            id: self.id.to_string(),
            author: self.user.map(|u| u.login).unwrap_or_default(),
            body: self.body,
            created_at: self.created_at,
            in_reply_to: None,
            web_url: self.html_url.filter(|u| !u.is_empty()),
        }
    }
}

fn file_status(text: &str) -> FileStatus {
    match text {
        "added" | "copied" => FileStatus::Added,
        "deleted" | "removed" => FileStatus::Removed,
        "renamed" => FileStatus::Renamed,
        _ => FileStatus::Modified,
    }
}

/// `"{root}|{n|o}|{line}|{path}"`, see the module docs.
fn thread_id(root: u64, side: DiffSide, line: u32, path: &str) -> String {
    let side = if side == DiffSide::Old { 'o' } else { 'n' };
    format!("{root}|{side}|{line}|{path}")
}

struct ThreadRef {
    root: u64,
    side: DiffSide,
    line: u32,
    path: String,
}

fn parse_thread_id(id: &str) -> Option<ThreadRef> {
    let mut parts = id.splitn(4, '|');
    let root = parts.next()?.parse().ok()?;
    let side = match parts.next()? {
        "o" => DiffSide::Old,
        "n" => DiffSide::New,
        _ => return None,
    };
    let line = parts.next()?.parse().ok()?;
    let path = parts.next()?.to_string();
    Some(ThreadRef {
        root,
        side,
        line,
        path,
    })
}

fn comment_position(side: DiffSide, line: u32) -> (&'static str, u32) {
    match side {
        DiffSide::New => ("new_position", line),
        DiffSide::Old => ("old_position", line),
    }
}

impl GiteaClient {
    /// Gitea can resolve a conversation, Forgejo (11) cannot; neither has
    /// multi-line comments.
    pub(super) fn review_caps(&self) -> ReviewCapabilities {
        ReviewCapabilities {
            review: true,
            approve: true,
            request_changes: true,
            line_ranges: false,
            resolve: self.integration.kind == IntegrationKind::Gitea,
        }
    }

    pub(super) async fn review_detail(
        &self,
        project: &str,
        number: u64,
    ) -> Result<PullRequestDetail> {
        let raw: Value = self
            .get_json(
                self.repo_url(project, &format!("pulls/{number}")),
                &format!("reading the {} pull request", self.label()),
            )
            .await?;
        let text = |key: &str| -> String {
            raw.get(key)
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string()
        };
        let count = |key: &str| raw.get(key).and_then(Value::as_u64).map(|n| n as u32);
        let row: PullRow =
            serde_json::from_value(raw.clone()).map_err(|e| Error::Decode(e.to_string()))?;
        let head_sha = row.head.sha.clone();
        let base_sha = row.base.sha.clone();
        let mergeable = row.mergeable;
        let summary = row.into_summary(None);
        // A work-in-progress request reports `mergeable: false` whatever its
        // branches do, so a draft can only be said to be blocked.
        let merge_state = match mergeable {
            Some(_) if summary.state == crate::model::PullRequestState::Draft => {
                MergeState::Blocked
            }
            Some(false) => MergeState::Conflicting,
            Some(true) => MergeState::Clean,
            None => MergeState::Unknown,
        };
        // A missing badge beats a failed review screen.
        let checks: Option<ChecksState> = if head_sha.is_empty() {
            None
        } else {
            self.commit_checks(project, &head_sha).await.ok().flatten()
        };
        Ok(PullRequestDetail {
            patch_sets: Vec::new(),
            shown_patch_set: None,
            vote_labels: Vec::new(),
            can_submit: false,
            can_abandon: false,
            summary,
            body: text("body"),
            head_sha,
            base_sha,
            merge_state,
            checks,
            created_at: text("created_at"),
            additions: count("additions"),
            deletions: count("deletions"),
            changed_files: count("changed_files"),
            capabilities: self.review_caps(),
        })
    }

    pub(super) async fn review_files(&self, project: &str, number: u64) -> Result<Vec<ReviewFile>> {
        let rows: Vec<FileRow> = self
            .pages(
                &self.repo_url(project, &format!("pulls/{number}/files")),
                &format!("listing the files of a {} pull request", self.label()),
            )
            .await?;
        Ok(rows
            .into_iter()
            .map(|f| ReviewFile {
                path: f.filename,
                previous_path: f.previous_filename.filter(|p| !p.is_empty()),
                status: file_status(&f.status),
                additions: f.additions,
                deletions: f.deletions,
            })
            .collect())
    }

    pub(super) async fn review_threads(
        &self,
        project: &str,
        number: u64,
    ) -> Result<Vec<ReviewThread>> {
        let reviews: Vec<ReviewRow> = self
            .pages(
                &self.repo_url(project, &format!("pulls/{number}/reviews")),
                &format!("listing the reviews of a {} pull request", self.label()),
            )
            .await?;
        let mut comments: Vec<CodeComment> = Vec::new();
        for review in reviews.into_iter().filter(|r| r.comments_count > 0) {
            let rows: Vec<CodeComment> = self
                .get_json(
                    self.repo_url(
                        project,
                        &format!("pulls/{number}/reviews/{}/comments", review.id),
                    ),
                    "reading the comments of a review",
                )
                .await?;
            comments.extend(rows);
        }
        if comments.is_empty() {
            return Ok(Vec::new());
        }
        let head: PullRow = self
            .get_json(
                self.repo_url(project, &format!("pulls/{number}")),
                "reading the head of a pull request",
            )
            .await?;
        let head_sha = head.head.sha;
        comments.sort_by_key(|c| c.id);
        // Forgejo (11) has no resolve endpoint: its threads cannot be resolved.
        let resolvable = self.review_caps().resolve;

        // One thread per (path, side, line), ordered by its root comment.
        let mut groups: BTreeMap<(String, bool, u32), Vec<CodeComment>> = BTreeMap::new();
        let mut unanchored: Vec<CodeComment> = Vec::new();
        for c in comments {
            match c.side_and_line() {
                Some((side, line)) => groups
                    .entry((c.path.clone(), side == DiffSide::Old, line))
                    .or_default()
                    .push(c),
                None => unanchored.push(c),
            }
        }
        let mut threads: Vec<ReviewThread> = groups
            .into_iter()
            .map(|((path, old, line), group)| {
                let side = if old { DiffSide::Old } else { DiffSide::New };
                let root = &group[0];
                let outdated = !root.commit_id.is_empty() && root.commit_id != head_sha;
                ReviewThread {
                    id: thread_id(root.id, side, line, &path),
                    resolved: group.iter().any(|c| c.resolver.is_some()),
                    outdated,
                    line: (!outdated).then_some(line),
                    original_line: Some(line),
                    side,
                    start_line: None,
                    resolvable,
                    comments: group.into_iter().map(CodeComment::into_comment).collect(),
                    path,
                }
            })
            .collect();
        // A comment with no line at all (its code is gone): one outdated
        // thread each, nothing to draw inline.
        for c in unanchored {
            threads.push(ReviewThread {
                id: thread_id(c.id, DiffSide::New, 0, &c.path),
                path: c.path.clone(),
                line: None,
                original_line: None,
                side: DiffSide::New,
                start_line: None,
                resolved: c.resolver.is_some(),
                outdated: true,
                resolvable,
                comments: vec![c.into_comment()],
            });
        }
        threads.sort_by_key(|t| {
            t.comments
                .first()
                .and_then(|c| c.id.parse::<u64>().ok())
                .unwrap_or(0)
        });
        Ok(threads)
    }

    /// The submitted reviews, oldest first. A `COMMENT` review without a
    /// summary only carries code comments (they are the threads), and
    /// `REQUEST_REVIEW` / `PENDING` rows are requests and drafts, not reviews.
    pub(super) async fn review_list_reviews(
        &self,
        project: &str,
        number: u64,
    ) -> Result<Vec<SubmittedReview>> {
        let rows: Vec<ReviewRow> = self
            .pages(
                &self.repo_url(project, &format!("pulls/{number}/reviews")),
                &format!("listing the reviews of a {} pull request", self.label()),
            )
            .await?;
        Ok(rows
            .into_iter()
            .filter_map(|r| {
                let state = if r.dismissed {
                    ReviewState::Dismissed
                } else {
                    match r.state.to_ascii_uppercase().as_str() {
                        "APPROVED" => ReviewState::Approved,
                        "REQUEST_CHANGES" => ReviewState::ChangesRequested,
                        "COMMENT" if !r.body.trim().is_empty() => ReviewState::Commented,
                        _ => return None,
                    }
                };
                Some(SubmittedReview {
                    id: r.id.to_string(),
                    author: r.user.map(|u| u.login).unwrap_or_default(),
                    state,
                    body: r.body,
                    submitted_at: r.submitted_at,
                })
            })
            .collect())
    }

    pub(super) async fn review_conversation(
        &self,
        project: &str,
        number: u64,
    ) -> Result<Vec<ReviewComment>> {
        let rows: Vec<IssueComment> = self
            .pages(
                &self.repo_url(project, &format!("issues/{number}/comments")),
                &format!("listing the comments of a {} pull request", self.label()),
            )
            .await?;
        Ok(rows.into_iter().map(IssueComment::into_comment).collect())
    }

    pub(super) async fn review_add_comment(
        &self,
        project: &str,
        number: u64,
        body: &str,
    ) -> Result<ReviewComment> {
        if body.trim().is_empty() {
            return Err(invalid(
                "commenting on a pull request",
                "the comment is empty",
            ));
        }
        let url = self.repo_url(project, &format!("issues/{number}/comments"));
        let response = self
            .transport
            .send(self.post(url, &json!({ "body": body })))
            .await?;
        response.error_for_status("commenting on a pull request")?;
        Ok(response.json::<IssueComment>()?.into_comment())
    }

    pub(super) async fn review_reply(
        &self,
        project: &str,
        number: u64,
        thread_id: &str,
        body: &str,
    ) -> Result<ReviewComment> {
        if body.trim().is_empty() {
            return Err(invalid("replying to a thread", "the reply is empty"));
        }
        let thread = parse_thread_id(thread_id)
            .ok_or_else(|| invalid("replying to a thread", "not a thread of this host"))?;
        let url = self.repo_url(
            project,
            &format!("pulls/{number}/comments/{}/replies", thread.root),
        );
        let response = self
            .transport
            .send(self.post(url, &json!({ "body": body })))
            .await?;
        // A server without the reply endpoint (Forgejo 11): say it on the
        // same line in a review of its own, which the UI groups the same.
        if !matches!(response.status, 404 | 405) {
            response.error_for_status("replying to a review thread")?;
            return Ok(response.json::<CodeComment>()?.into_comment());
        }
        let (key, line) = comment_position(thread.side, thread.line);
        let review = json!({
            "event": "COMMENT",
            "comments": [{ "path": thread.path, "body": body, key: line }],
        });
        let url = self.repo_url(project, &format!("pulls/{number}/reviews"));
        let response = self.transport.send(self.post(url, &review)).await?;
        response.error_for_status("replying to a review thread")?;
        let posted: ReviewRow = response.json()?;
        let rows: Vec<CodeComment> = self
            .get_json(
                self.repo_url(
                    project,
                    &format!("pulls/{number}/reviews/{}/comments", posted.id),
                ),
                "reading the posted reply",
            )
            .await?;
        rows.into_iter()
            .next()
            .map(CodeComment::into_comment)
            .ok_or_else(|| Error::Decode("the reply was posted but not returned".to_string()))
    }

    pub(super) async fn review_resolve(
        &self,
        project: &str,
        thread_id: &str,
        resolved: bool,
    ) -> Result<()> {
        let thread = parse_thread_id(thread_id)
            .ok_or_else(|| invalid("resolving a thread", "not a thread of this host"))?;
        let verb = if resolved { "resolve" } else { "unresolve" };
        let url = self.repo_url(project, &format!("pulls/comments/{}/{verb}", thread.root));
        let response = self.transport.send(self.post(url, &json!({}))).await?;
        // The endpoint is newer than the rest of the API.
        if matches!(response.status, 404 | 405) {
            return Err(Error::Unsupported(format!(
                "resolving threads on {}",
                self.label()
            )));
        }
        response.error_for_status("resolving a review thread")
    }

    pub(super) async fn review_submit(
        &self,
        project: &str,
        number: u64,
        review: &ReviewSubmission,
    ) -> Result<ReviewOutcome> {
        validate_submission(self.review_caps(), review)?;
        let event = match review.event {
            ReviewEvent::Approve => "APPROVED",
            ReviewEvent::RequestChanges => "REQUEST_CHANGES",
            ReviewEvent::Comment => "COMMENT",
        };
        let comments: Vec<Value> = review
            .comments
            .iter()
            .map(|c| {
                let (key, line) = comment_position(c.side, c.line);
                json!({ "path": c.path, "body": c.body, key: line })
            })
            .collect();
        let mut body = json!({ "event": event, "comments": comments });
        if !review.body.trim().is_empty() {
            body["body"] = json!(review.body);
        }
        if let Some(sha) = review.head_sha.as_deref().filter(|s| !s.is_empty()) {
            body["commit_id"] = json!(sha);
        }
        let url = self.repo_url(project, &format!("pulls/{number}/reviews"));
        let response = self.transport.send(self.post(url, &body)).await?;
        response.error_for_status("submitting a review")?;
        let posted: Value = response.json()?;
        Ok(ReviewOutcome {
            event: review.event,
            comments: review.comments.len(),
            review_id: posted
                .get("id")
                .and_then(Value::as_u64)
                .map(|i| i.to_string()),
        })
    }
}
