//! In-app review on GitLab (gitlab.com and self-managed).
//!
//! Everything is REST. Threads are *discussions*: those whose first note
//! carries a `position` are code threads, the rest are the conversation.
//! A review is submitted as **draft notes** published together
//! (`bulk_publish`), then `approve`. Differences from GitHub that callers
//! can observe:
//!
//! * no "request changes" (REST has no equivalent): the capability is off
//!   and asking for it is `Unsupported`;
//! * no multi-line comments: a `start_line` is ignored and the comment sits
//!   on `line` (a range needs a `line_code` per end that only the diff
//!   knows);
//! * a comment on an unchanged context line needs both line numbers
//!   ([`super::PendingComment::other_line`]);
//! * an instance older than 15.9 has no draft notes: the comments are then
//!   posted at once as discussions, so they are not held back until the
//!   verdict.

use serde::Deserialize;
use serde_json::{json, Value};

use super::api_code::{base64_decode, summary_of};
use super::model::{
    DiffSide, FileStatus, MergeState, PendingComment, PullRequestDetail, ReviewCapabilities,
    ReviewComment, ReviewEvent, ReviewFile, ReviewOutcome, ReviewState, ReviewSubmission,
    ReviewThread, SubmittedReview,
};
use super::{
    count_changes, invalid, validate_submission, ApiCommit, ApiFileDiff, NOT_SHOWN_BY_THE_HOST,
};
use crate::client::encode_component;
use crate::client::fetch_pages;
use crate::gitlab::{encode_gitlab_project_path, GitLabClient, GitLabMrRow};
use crate::model::ChecksState;
use crate::{Error, Result};

pub(crate) const CAPABILITIES: ReviewCapabilities = ReviewCapabilities {
    review: true,
    approve: true,
    request_changes: false,
    line_ranges: false,
    resolve: true,
};

#[derive(Deserialize)]
struct Username {
    username: String,
}

#[derive(Deserialize)]
struct Note {
    id: u64,
    #[serde(default)]
    body: String,
    #[serde(default)]
    created_at: String,
    author: Option<Username>,
    #[serde(default)]
    system: bool,
    #[serde(default)]
    resolvable: bool,
    #[serde(default)]
    resolved: bool,
    position: Option<Position>,
    change_position: Option<Position>,
}

#[derive(Deserialize)]
struct Position {
    old_path: Option<String>,
    new_path: Option<String>,
    old_line: Option<u32>,
    new_line: Option<u32>,
    head_sha: Option<String>,
    line_range: Option<LineRange>,
}

#[derive(Deserialize)]
struct LineRange {
    start: Option<RangeEnd>,
}

#[derive(Deserialize)]
struct RangeEnd {
    old_line: Option<u32>,
    new_line: Option<u32>,
}

#[derive(Deserialize)]
struct Discussion {
    id: String,
    #[serde(default)]
    notes: Vec<Note>,
}

#[derive(Deserialize)]
struct DiffRow {
    #[serde(default)]
    old_path: String,
    #[serde(default)]
    new_path: String,
    #[serde(default)]
    new_file: bool,
    #[serde(default)]
    renamed_file: bool,
    #[serde(default)]
    deleted_file: bool,
    #[serde(default)]
    diff: String,
    #[serde(default)]
    collapsed: bool,
    #[serde(default)]
    too_large: bool,
}

#[derive(Deserialize)]
struct Changes {
    #[serde(default)]
    changes: Vec<DiffRow>,
}

#[derive(Deserialize)]
struct DraftNote {
    id: u64,
}

impl Note {
    fn into_comment(self, in_reply_to: Option<String>) -> ReviewComment {
        ReviewComment {
            id: self.id.to_string(),
            author: self.author.map(|a| a.username).unwrap_or_default(),
            body: self.body,
            created_at: self.created_at,
            in_reply_to,
            web_url: None,
        }
    }
}

fn merge_state(mr: &Value) -> MergeState {
    if mr["has_conflicts"].as_bool() == Some(true) {
        return MergeState::Conflicting;
    }
    match mr["detailed_merge_status"].as_str() {
        Some("mergeable") => MergeState::Clean,
        Some("conflict") => MergeState::Conflicting,
        Some("checking" | "unchecked" | "preparing" | "approvals_syncing") | None => {
            match mr["merge_status"].as_str() {
                Some("can_be_merged") => MergeState::Clean,
                Some("cannot_be_merged") => MergeState::Conflicting,
                _ => MergeState::Unknown,
            }
        }
        Some(_) => MergeState::Blocked,
    }
}

fn checks_state(mr: &Value) -> Option<ChecksState> {
    match mr["head_pipeline"]["status"].as_str()? {
        "success" => Some(ChecksState::Passing),
        "failed" | "canceled" => Some(ChecksState::Failing),
        "running"
        | "pending"
        | "created"
        | "waiting_for_resource"
        | "preparing"
        | "scheduled"
        | "waiting_for_callback" => Some(ChecksState::Pending),
        _ => None,
    }
}

fn file_of(row: DiffRow) -> ReviewFile {
    let (additions, deletions) = count_changes(&row.diff);
    let (status, path, previous_path) = if row.deleted_file {
        (FileStatus::Removed, row.old_path, None)
    } else if row.new_file {
        (FileStatus::Added, row.new_path, None)
    } else if row.renamed_file && row.old_path != row.new_path {
        (FileStatus::Renamed, row.new_path, Some(row.old_path))
    } else {
        (FileStatus::Modified, row.new_path, None)
    };
    ReviewFile {
        path,
        previous_path,
        status,
        additions,
        deletions,
    }
}

/// `(side, line, start_line)` of a position: the new side when it has a
/// new line (context lines have both), else the old side.
fn place(p: &Position) -> (DiffSide, Option<u32>, Option<u32>) {
    let start = p.line_range.as_ref().and_then(|r| r.start.as_ref());
    match (p.new_line, p.old_line) {
        (Some(line), _) => (
            DiffSide::New,
            Some(line),
            start.and_then(|s| s.new_line).filter(|s| *s < line),
        ),
        (None, Some(line)) => (
            DiffSide::Old,
            Some(line),
            start.and_then(|s| s.old_line).filter(|s| *s < line),
        ),
        (None, None) => (DiffSide::New, None, None),
    }
}

fn thread_of(d: Discussion, head_sha: &str) -> Option<ReviewThread> {
    let first = d.notes.first()?;
    if first.system {
        return None;
    }
    let original = first.position.as_ref()?;
    let moved = first.change_position.as_ref();
    // GitLab keeps the position the note was written at; a note whose
    // lines no longer exist at the current head carries no
    // `change_position`, which is what "outdated" means here.
    let outdated = moved.is_none() && original.head_sha.as_deref().is_some_and(|h| h != head_sha);
    let (side, original_line, _) = place(original);
    let (current_side, current_line, start_line) = match (outdated, moved) {
        (true, _) => (side, None, place(original).2),
        (false, Some(m)) => place(m),
        (false, None) => place(original),
    };
    let path = original
        .new_path
        .clone()
        .or_else(|| original.old_path.clone())
        .unwrap_or_default();
    let resolvable = d.notes.iter().any(|n| n.resolvable);
    let resolved = resolvable && d.notes.iter().filter(|n| n.resolvable).all(|n| n.resolved);
    let first_id = first.id.to_string();
    let comments = d
        .notes
        .into_iter()
        .filter(|n| !n.system)
        .enumerate()
        .map(|(i, n)| n.into_comment((i > 0).then(|| first_id.clone())))
        .collect();
    Some(ReviewThread {
        id: d.id,
        path,
        line: current_line,
        original_line,
        side: current_side,
        start_line,
        resolved,
        outdated,
        resolvable,
        comments,
    })
}

/// `diff_refs`, the three commits a position is anchored to.
struct DiffRefs {
    base: String,
    start: String,
    head: String,
}

fn position_json(refs: &DiffRefs, c: &PendingComment) -> Value {
    let mut position = json!({
        "position_type": "text",
        "base_sha": refs.base,
        "start_sha": refs.start,
        "head_sha": refs.head,
        "new_path": c.path,
        "old_path": c.previous_path.as_deref().unwrap_or(&c.path),
    });
    match (c.side, c.other_line) {
        (DiffSide::New, other) => {
            position["new_line"] = json!(c.line);
            if let Some(old) = other {
                position["old_line"] = json!(old);
            }
        }
        (DiffSide::Old, other) => {
            position["old_line"] = json!(c.line);
            if let Some(new) = other {
                position["new_line"] = json!(new);
            }
        }
    }
    position
}

impl GitLabClient {
    fn mr_url(&self, project: &str, number: u64, rest: &str) -> String {
        format!(
            "{}/projects/{}/merge_requests/{number}{rest}",
            self.integration.api_base(),
            encode_gitlab_project_path(project)
        )
    }

    async fn review_mr(&self, project: &str, number: u64) -> Result<Value> {
        let response = self
            .transport
            .send(self.get(self.mr_url(project, number, "")))
            .await?;
        response.error_for_status("reading the GitLab merge request")?;
        response.json()
    }

    async fn discussions(&self, project: &str, number: u64) -> Result<Vec<Discussion>> {
        let url = self.mr_url(project, number, "/discussions?");
        fetch_pages(
            &*self.transport,
            |u| self.get(u),
            &url,
            "listing the discussions of a GitLab merge request",
        )
        .await
    }

    pub(crate) async fn review_detail(
        &self,
        project: &str,
        number: u64,
    ) -> Result<PullRequestDetail> {
        let mr = self.review_mr(project, number).await?;
        let row: GitLabMrRow =
            serde_json::from_value(mr.clone()).map_err(|e| Error::Decode(e.to_string()))?;
        let text = |v: &Value| v.as_str().unwrap_or_default().to_string();
        let head_sha = match mr["diff_refs"]["head_sha"].as_str() {
            Some(h) => h.to_string(),
            None => text(&mr["sha"]),
        };
        Ok(PullRequestDetail {
            patch_sets: Vec::new(),
            shown_patch_set: None,
            vote_labels: Vec::new(),
            can_submit: false,
            can_abandon: false,
            summary: row.into_summary(false),
            body: text(&mr["description"]),
            head_sha,
            // GitLab's `base_sha` is the merge base, which is exactly
            // the commit the diff is taken from.
            base_sha: text(&mr["diff_refs"]["base_sha"]),
            merge_state: merge_state(&mr),
            checks: checks_state(&mr),
            created_at: text(&mr["created_at"]),
            additions: None,
            deletions: None,
            changed_files: mr["changes_count"]
                .as_str()
                .and_then(|s| s.trim_end_matches('+').parse().ok()),
            capabilities: CAPABILITIES,
        })
    }

    /// The per-file diff rows of a request: `/diffs` (15.7+) pages; the
    /// older `/changes` returns the lot.
    async fn diff_rows(&self, project: &str, number: u64) -> Result<Vec<DiffRow>> {
        const WHAT: &str = "listing the files of a GitLab merge request";
        let url = self.mr_url(project, number, "/diffs?");
        match fetch_pages(&*self.transport, |u| self.get(u), &url, WHAT).await {
            Err(Error::Api { status: 404, .. }) => {
                let response = self
                    .transport
                    .send(self.get(self.mr_url(project, number, "/changes")))
                    .await?;
                response.error_for_status(WHAT)?;
                Ok(response.json::<Changes>()?.changes)
            }
            other => other,
        }
    }

    pub(crate) async fn review_files(&self, project: &str, number: u64) -> Result<Vec<ReviewFile>> {
        Ok(self
            .diff_rows(project, number)
            .await?
            .into_iter()
            .map(file_of)
            .collect())
    }

    pub(crate) async fn review_threads(
        &self,
        project: &str,
        number: u64,
    ) -> Result<Vec<ReviewThread>> {
        let mr = self.review_mr(project, number).await?;
        let head = mr["diff_refs"]["head_sha"]
            .as_str()
            .or(mr["sha"].as_str())
            .unwrap_or_default()
            .to_string();
        Ok(self
            .discussions(project, number)
            .await?
            .into_iter()
            .filter_map(|d| thread_of(d, &head))
            .collect())
    }

    pub(crate) async fn review_conversation(
        &self,
        project: &str,
        number: u64,
    ) -> Result<Vec<ReviewComment>> {
        let mut out = Vec::new();
        for d in self.discussions(project, number).await? {
            if d.notes.first().map_or(true, |n| n.position.is_some()) {
                continue;
            }
            let first_id = d.notes[0].id.to_string();
            out.extend(
                d.notes
                    .into_iter()
                    .filter(|n| !n.system)
                    .enumerate()
                    .map(|(i, n)| n.into_comment((i > 0).then(|| first_id.clone()))),
            );
        }
        out.sort_by(|a, b| a.created_at.cmp(&b.created_at));
        Ok(out)
    }

    /// GitLab has no review object: the verdicts are the approvals (who,
    /// not when), read from `approvals`.
    pub(crate) async fn review_list_reviews(
        &self,
        project: &str,
        number: u64,
    ) -> Result<Vec<SubmittedReview>> {
        let response = self
            .transport
            .send(self.get(self.mr_url(project, number, "/approvals")))
            .await?;
        response.error_for_status("reading the approvals of a GitLab merge request")?;
        let value: Value = response.json()?;
        Ok(value["approved_by"]
            .as_array()
            .map(|list| {
                list.iter()
                    .filter_map(|a| a["user"]["username"].as_str())
                    .map(|login| SubmittedReview {
                        id: format!("approval-{login}"),
                        author: login.to_string(),
                        state: ReviewState::Approved,
                        body: String::new(),
                        submitted_at: String::new(),
                    })
                    .collect()
            })
            .unwrap_or_default())
    }

    pub(crate) async fn review_add_comment(
        &self,
        project: &str,
        number: u64,
        body: &str,
    ) -> Result<ReviewComment> {
        if body.trim().is_empty() {
            return Err(invalid(
                "commenting on a merge request",
                "the comment is empty",
            ));
        }
        let response = self
            .transport
            .send(self.post(
                self.mr_url(project, number, "/notes"),
                &json!({ "body": body }),
            ))
            .await?;
        response.error_for_status("commenting on a GitLab merge request")?;
        Ok(response.json::<Note>()?.into_comment(None))
    }

    pub(crate) async fn review_reply(
        &self,
        project: &str,
        number: u64,
        thread_id: &str,
        body: &str,
    ) -> Result<ReviewComment> {
        if body.trim().is_empty() {
            return Err(invalid("replying to a thread", "the reply is empty"));
        }
        let url = self.mr_url(
            project,
            number,
            &format!(
                "/discussions/{}/notes",
                crate::client::encode_component(thread_id)
            ),
        );
        let response = self
            .transport
            .send(self.post(url, &json!({ "body": body })))
            .await?;
        response.error_for_status("replying to a GitLab discussion")?;
        Ok(response
            .json::<Note>()?
            .into_comment(Some(thread_id.to_string())))
    }

    pub(crate) async fn review_resolve(
        &self,
        project: &str,
        number: u64,
        thread_id: &str,
        resolved: bool,
    ) -> Result<()> {
        let url = self.mr_url(
            project,
            number,
            &format!(
                "/discussions/{}?resolved={resolved}",
                crate::client::encode_component(thread_id)
            ),
        );
        let response = self
            .transport
            .send(self.put(url, &json!({ "resolved": resolved })))
            .await?;
        response.error_for_status(if resolved {
            "resolving a GitLab discussion"
        } else {
            "reopening a GitLab discussion"
        })
    }

    pub(crate) async fn review_submit(
        &self,
        project: &str,
        number: u64,
        review: &ReviewSubmission,
    ) -> Result<ReviewOutcome> {
        validate_submission(CAPABILITIES, review)?;
        const WHAT: &str = "submitting a GitLab review";
        let mr = self.review_mr(project, number).await?;
        let refs = DiffRefs {
            base: mr["diff_refs"]["base_sha"]
                .as_str()
                .unwrap_or_default()
                .into(),
            start: mr["diff_refs"]["start_sha"]
                .as_str()
                .unwrap_or_default()
                .into(),
            head: mr["diff_refs"]["head_sha"]
                .as_str()
                .unwrap_or_default()
                .into(),
        };
        if let Some(expected) = review.head_sha.as_deref().filter(|s| !s.is_empty()) {
            if !refs.head.is_empty() && expected != refs.head {
                return Err(invalid(
                    WHAT,
                    "the merge request moved since it was read; refresh and review again",
                ));
            }
        }

        // Pending notes first, so a failure part-way leaves nothing
        // published: they are deleted again before the error is returned.
        let drafts_url = self.mr_url(project, number, "/draft_notes");
        let mut drafts: Vec<u64> = Vec::new();
        let mut notes: Vec<Value> = review
            .comments
            .iter()
            .map(|c| json!({ "note": c.body, "position": position_json(&refs, c) }))
            .collect();
        if !review.body.trim().is_empty() {
            notes.push(json!({ "note": review.body }));
        }
        let mut published_directly = false;
        for (i, note) in notes.iter().enumerate() {
            let response = self
                .transport
                .send(self.post(drafts_url.clone(), note))
                .await;
            let response = match response {
                Ok(r) if r.status == 404 && i == 0 => {
                    published_directly = true;
                    break;
                }
                Ok(r) => r.error_for_status(WHAT).map(|_| r),
                Err(e) => Err(e),
            };
            match response.and_then(|r| r.json::<DraftNote>()) {
                Ok(d) => drafts.push(d.id),
                Err(e) => {
                    self.discard_drafts(&drafts_url, &drafts).await;
                    return Err(e);
                }
            }
        }
        if published_directly {
            // No draft notes on this instance: post each comment as a
            // discussion (a plain note for the summary).
            for c in &review.comments {
                let url = self.mr_url(project, number, "/discussions");
                let body = json!({ "body": c.body, "position": position_json(&refs, c) });
                let response = self.transport.send(self.post(url, &body)).await?;
                response.error_for_status(WHAT)?;
            }
            if !review.body.trim().is_empty() {
                self.review_add_comment(project, number, &review.body)
                    .await?;
            }
        } else if !drafts.is_empty() {
            let url = format!("{drafts_url}/bulk_publish");
            let response = self.transport.send(self.post(url, &json!({}))).await;
            if let Err(e) = response.and_then(|r| r.error_for_status(WHAT)) {
                self.discard_drafts(&drafts_url, &drafts).await;
                return Err(e);
            }
        }

        if review.event == ReviewEvent::Approve {
            let mut body = json!({});
            if !refs.head.is_empty() {
                body["sha"] = json!(refs.head);
            }
            let response = self
                .transport
                .send(self.post(self.mr_url(project, number, "/approve"), &body))
                .await?;
            response.error_for_status("approving a GitLab merge request")?;
        }
        Ok(ReviewOutcome {
            event: review.event,
            comments: review.comments.len(),
            review_id: None,
        })
    }

    /// Best effort: a draft that cannot be deleted stays in the author's
    /// pending list, where it is visible and deletable by hand.
    async fn discard_drafts(&self, drafts_url: &str, ids: &[u64]) {
        for id in ids {
            let _ = self
                .transport
                .send(self.delete(format!("{drafts_url}/{id}")))
                .await;
        }
    }
}

// ---- the code through the API (slice S3a) ----

#[derive(Deserialize)]
struct CommitRow {
    #[serde(default)]
    id: String,
    #[serde(default)]
    parent_ids: Vec<String>,
    #[serde(default)]
    author_name: String,
    #[serde(default)]
    created_at: String,
    #[serde(default)]
    authored_date: String,
    #[serde(default)]
    title: String,
    #[serde(default)]
    message: String,
}

#[derive(Deserialize)]
struct FileContent {
    #[serde(default)]
    content: String,
    #[serde(default)]
    encoding: String,
}

fn api_file_of(row: DiffRow) -> ApiFileDiff {
    use gitpoulpi_core::domain::FileStatus as Core;
    let (additions, deletions) = count_changes(&row.diff);
    let omitted = (row.collapsed || row.too_large).then(|| NOT_SHOWN_BY_THE_HOST.to_string());
    let status = if row.deleted_file {
        Core::Deleted
    } else if row.new_file {
        Core::Added
    } else if row.renamed_file && row.old_path != row.new_path {
        Core::Renamed
    } else {
        Core::Modified
    };
    let path = if row.deleted_file {
        row.old_path.clone()
    } else {
        row.new_path.clone()
    };
    let previous_path = (status == Core::Renamed).then(|| row.old_path.clone());
    ApiFileDiff {
        // GitLab's `diff` is the hunks alone, with no file header.
        diff: gitpoulpi_core::diff_text::parse_hunks(path, status, &row.diff),
        previous_path,
        additions,
        deletions,
        omitted,
    }
}

impl GitLabClient {
    fn project_url(&self, project: &str, rest: &str) -> String {
        format!(
            "{}/projects/{}{rest}",
            self.integration.api_base(),
            encode_gitlab_project_path(project)
        )
    }

    pub(crate) async fn review_diff(&self, project: &str, number: u64) -> Result<Vec<ApiFileDiff>> {
        Ok(self
            .diff_rows(project, number)
            .await?
            .into_iter()
            .map(api_file_of)
            .collect())
    }

    pub(crate) async fn review_commits(
        &self,
        project: &str,
        number: u64,
    ) -> Result<Vec<ApiCommit>> {
        let url = self.mr_url(project, number, "/commits?");
        let rows: Vec<CommitRow> = fetch_pages(
            &*self.transport,
            |u| self.get(u),
            &url,
            "listing the commits of a GitLab merge request",
        )
        .await?;
        // GitLab lists the newest first.
        Ok(rows
            .into_iter()
            .rev()
            .map(|r| ApiCommit {
                sha: r.id,
                parents: r.parent_ids,
                author: r.author_name,
                date: if r.authored_date.is_empty() {
                    r.created_at
                } else {
                    r.authored_date
                },
                summary: if r.title.is_empty() {
                    summary_of(&r.message)
                } else {
                    r.title
                },
            })
            .collect())
    }

    pub(crate) async fn review_commit_diff(
        &self,
        project: &str,
        sha: &str,
    ) -> Result<Vec<ApiFileDiff>> {
        let url = self.project_url(
            project,
            &format!("/repository/commits/{}/diff?", encode_component(sha)),
        );
        let rows: Vec<DiffRow> = fetch_pages(
            &*self.transport,
            |u| self.get(u),
            &url,
            "reading a commit of a GitLab project",
        )
        .await?;
        Ok(rows.into_iter().map(api_file_of).collect())
    }

    /// `repository/files/<path>?ref=`: JSON with the bytes in base64, so a
    /// binary file survives (the transport hands bodies over as text). The
    /// route needs a `ref`; without one the project's default branch is used.
    pub(crate) async fn review_file(
        &self,
        project: &str,
        path: &str,
        rev: Option<&str>,
    ) -> Result<Option<Vec<u8>>> {
        let rev = match rev {
            Some(r) => r.to_string(),
            None => {
                let response = self
                    .transport
                    .send(self.get(self.project_url(project, "")))
                    .await?;
                response.error_for_status("reading a GitLab project")?;
                let found: Value = response.json()?;
                found["default_branch"]
                    .as_str()
                    .unwrap_or("main")
                    .to_string()
            }
        };
        let url = self.project_url(
            project,
            &format!(
                "/repository/files/{}?ref={}",
                encode_component(path.trim_start_matches('/')),
                encode_component(&rev)
            ),
        );
        let response = self.transport.send(self.get(url)).await?;
        if response.status == 404 {
            return Ok(None);
        }
        response.error_for_status("reading a file of a GitLab project")?;
        let file: FileContent = response.json()?;
        if file.encoding != "base64" {
            return Err(Error::Decode(format!(
                "{path}: unexpected encoding `{}`",
                file.encoding
            )));
        }
        base64_decode(&file.content)
            .map(Some)
            .ok_or_else(|| Error::Decode(format!("the content of {path} is not base64")))
    }
}
