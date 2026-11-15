//! In-app review on GitHub (github.com and Enterprise Server).
//!
//! REST for the request, its files, conversation comments and the review
//! submission. Threads need GraphQL: only `pullRequest.reviewThreads`
//! carries a thread's id and resolved state, and resolving is only a
//! mutation there (`resolveReviewThread`), so threads, replies and
//! resolution all go through it.

use serde::Deserialize;
use serde_json::{json, Value};

use super::api_code::{base64_decode, encode_path, summary_of};
use super::model::{
    DiffSide, FileStatus, MergeState, PullRequestDetail, ReviewCapabilities, ReviewComment,
    ReviewEvent, ReviewFile, ReviewOutcome, ReviewState, ReviewSubmission, ReviewThread,
    SubmittedReview,
};
use super::{invalid, validate_submission, ApiCommit, ApiFileDiff, NOT_SHOWN_BY_THE_HOST};
use crate::client::encode_component;
use crate::client::{fetch_pages, MAX_PAGES};
use crate::github::{GitHubClient, GitHubPullRow};
use crate::model::ChecksState;
use crate::{Error, Result};

pub(crate) const CAPABILITIES: ReviewCapabilities = ReviewCapabilities {
    review: true,
    approve: true,
    request_changes: true,
    line_ranges: true,
    resolve: true,
};

const THREADS_QUERY: &str = "query($owner:String!,$name:String!,$number:Int!,$after:String){\
repository(owner:$owner,name:$name){pullRequest(number:$number){\
reviewThreads(first:100,after:$after){pageInfo{hasNextPage endCursor}\
nodes{id isResolved isOutdated path line originalLine startLine originalStartLine diffSide \
comments(first:100){nodes{databaseId author{login} body createdAt url replyTo{databaseId}}}}}}}}";

const REPLY_MUTATION: &str = "mutation($thread:ID!,$body:String!){\
addPullRequestReviewThreadReply(input:{pullRequestReviewThreadId:$thread,body:$body}){\
comment{databaseId author{login} body createdAt url replyTo{databaseId}}}}";

const RESOLVE_MUTATION: &str =
    "mutation($thread:ID!){resolveReviewThread(input:{threadId:$thread}){thread{isResolved}}}";
const UNRESOLVE_MUTATION: &str =
    "mutation($thread:ID!){unresolveReviewThread(input:{threadId:$thread}){thread{isResolved}}}";

#[derive(Deserialize)]
struct Login {
    login: String,
}

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
            web_url: self.html_url,
        }
    }
}

#[derive(Deserialize)]
struct ReviewRow {
    id: u64,
    #[serde(default)]
    state: String,
    body: Option<String>,
    submitted_at: Option<String>,
    user: Option<Login>,
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
    patch: Option<String>,
}

#[derive(Deserialize)]
struct CheckRuns {
    #[serde(default)]
    check_runs: Vec<CheckRun>,
}

#[derive(Deserialize)]
struct CheckRun {
    #[serde(default)]
    status: String,
    conclusion: Option<String>,
}

#[derive(Deserialize)]
struct GqlComment {
    #[serde(rename = "databaseId")]
    database_id: Option<u64>,
    author: Option<Login>,
    #[serde(default)]
    body: String,
    #[serde(default, rename = "createdAt")]
    created_at: String,
    url: Option<String>,
    #[serde(rename = "replyTo")]
    reply_to: Option<GqlReplyTo>,
}

#[derive(Deserialize)]
struct GqlReplyTo {
    #[serde(rename = "databaseId")]
    database_id: Option<u64>,
}

impl GqlComment {
    fn into_comment(self) -> ReviewComment {
        ReviewComment {
            id: self.database_id.map(|i| i.to_string()).unwrap_or_default(),
            author: self.author.map(|a| a.login).unwrap_or_default(),
            body: self.body,
            created_at: self.created_at,
            in_reply_to: self
                .reply_to
                .and_then(|r| r.database_id)
                .map(|i| i.to_string()),
            web_url: self.url,
        }
    }
}

#[derive(Deserialize)]
struct GqlThread {
    id: String,
    #[serde(default, rename = "isResolved")]
    is_resolved: bool,
    #[serde(default, rename = "isOutdated")]
    is_outdated: bool,
    #[serde(default)]
    path: String,
    line: Option<u32>,
    #[serde(rename = "originalLine")]
    original_line: Option<u32>,
    #[serde(rename = "startLine")]
    start_line: Option<u32>,
    #[serde(rename = "originalStartLine")]
    original_start_line: Option<u32>,
    #[serde(rename = "diffSide")]
    diff_side: Option<String>,
    comments: GqlNodes<GqlComment>,
}

#[derive(Deserialize)]
struct GqlNodes<T> {
    #[serde(default = "Vec::new")]
    nodes: Vec<T>,
}

fn side_of(text: Option<&str>) -> DiffSide {
    match text {
        Some("LEFT") => DiffSide::Old,
        _ => DiffSide::New,
    }
}

fn graphql_side(side: DiffSide) -> &'static str {
    match side {
        DiffSide::Old => "LEFT",
        DiffSide::New => "RIGHT",
    }
}

fn file_status(text: &str) -> FileStatus {
    match text {
        "added" | "copied" => FileStatus::Added,
        "removed" => FileStatus::Removed,
        "renamed" => FileStatus::Renamed,
        _ => FileStatus::Modified,
    }
}

fn merge_state(mergeable: Option<bool>, state: Option<&str>) -> MergeState {
    if mergeable == Some(false) {
        return MergeState::Conflicting;
    }
    match state {
        Some("clean" | "has_hooks" | "unstable") => MergeState::Clean,
        Some("dirty") => MergeState::Conflicting,
        Some("blocked" | "behind" | "draft") => MergeState::Blocked,
        _ => MergeState::Unknown,
    }
}

fn checks_state(runs: &[CheckRun]) -> Option<ChecksState> {
    if runs.is_empty() {
        return None;
    }
    let failing = |c: &str| {
        matches!(
            c,
            "failure" | "timed_out" | "cancelled" | "action_required" | "startup_failure"
        )
    };
    if runs
        .iter()
        .any(|r| r.conclusion.as_deref().is_some_and(failing))
    {
        Some(ChecksState::Failing)
    } else if runs.iter().any(|r| r.status != "completed") {
        Some(ChecksState::Pending)
    } else {
        Some(ChecksState::Passing)
    }
}

impl GitHubClient {
    pub(crate) fn repo_url(&self, project: &str, rest: &str) -> String {
        format!("{}/repos/{project}/{rest}", self.integration.api_base())
    }

    /// `https://api.github.com/graphql`, or `<url>/api/graphql` on an
    /// Enterprise Server (whose REST root is `<url>/api/v3`).
    fn graphql_url(&self) -> String {
        let base = self.integration.api_base();
        format!("{}/graphql", base.strip_suffix("/v3").unwrap_or(&base))
    }

    /// Runs a GraphQL document and returns its `data`. GraphQL answers
    /// `200` with an `errors` array on failure, so the status alone proves
    /// nothing.
    pub(crate) async fn graphql(&self, what: &str, query: &str, variables: Value) -> Result<Value> {
        let body = json!({ "query": query, "variables": variables });
        let response = self
            .transport
            .send(self.post(self.graphql_url(), &body))
            .await?;
        response.error_for_status(what)?;
        let mut answer: Value = response.json()?;
        if let Some(errors) = answer.get("errors").and_then(Value::as_array) {
            if !errors.is_empty() {
                let detail = errors
                    .iter()
                    .filter_map(|e| e.get("message").and_then(Value::as_str))
                    .collect::<Vec<_>>()
                    .join("; ");
                return Err(Error::Api {
                    what: what.to_string(),
                    status: 200,
                    detail,
                });
            }
        }
        Ok(answer["data"].take())
    }

    pub(crate) async fn review_detail(
        &self,
        project: &str,
        number: u64,
    ) -> Result<PullRequestDetail> {
        let response = self
            .transport
            .send(self.get(self.repo_url(project, &format!("pulls/{number}"))))
            .await?;
        response.error_for_status("reading the GitHub pull request")?;
        let raw: Value = response.json()?;
        let text = |path: &[&str]| -> String {
            path.iter()
                .try_fold(&raw, |v, k| v.get(k))
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string()
        };
        let count = |key: &str| raw.get(key).and_then(Value::as_u64).map(|n| n as u32);
        let head_sha = text(&["head", "sha"]);
        let base_sha = text(&["base", "sha"]);
        let merge = merge_state(
            raw.get("mergeable").and_then(Value::as_bool),
            raw.get("mergeable_state").and_then(Value::as_str),
        );
        let (additions, deletions, changed_files) = (
            count("additions"),
            count("deletions"),
            count("changed_files"),
        );
        let row: GitHubPullRow =
            serde_json::from_value(raw.clone()).map_err(|e| Error::Decode(e.to_string()))?;
        let created_at = text(&["created_at"]);
        let body = text(&["body"]);
        let checks = self.review_checks(project, &head_sha).await;
        Ok(PullRequestDetail {
            patch_sets: Vec::new(),
            shown_patch_set: None,
            vote_labels: Vec::new(),
            can_submit: false,
            can_abandon: false,
            summary: row.into_summary(false),
            body,
            head_sha,
            base_sha,
            merge_state: merge,
            checks,
            created_at,
            additions,
            deletions,
            changed_files,
            capabilities: CAPABILITIES,
        })
    }

    /// The checks state, or `None` when there are none or the token may
    /// not read them: a missing badge beats a failed review screen.
    async fn review_checks(&self, project: &str, sha: &str) -> Option<ChecksState> {
        if sha.is_empty() {
            return None;
        }
        let url = self.repo_url(project, &format!("commits/{sha}/check-runs?per_page=100"));
        let response = self.transport.send(self.get(url)).await.ok()?;
        if !response.is_success() {
            return None;
        }
        let runs: CheckRuns = response.json().ok()?;
        checks_state(&runs.check_runs)
    }

    pub(crate) async fn review_files(&self, project: &str, number: u64) -> Result<Vec<ReviewFile>> {
        let url = self.repo_url(project, &format!("pulls/{number}/files?"));
        let rows: Vec<FileRow> = fetch_pages(
            &*self.transport,
            |u| self.get(u),
            &url,
            "listing the files of a GitHub pull request",
        )
        .await?;
        Ok(rows
            .into_iter()
            .map(|f| ReviewFile {
                path: f.filename,
                previous_path: f.previous_filename,
                status: file_status(&f.status),
                additions: f.additions,
                deletions: f.deletions,
            })
            .collect())
    }

    pub(crate) async fn review_threads(
        &self,
        project: &str,
        number: u64,
    ) -> Result<Vec<ReviewThread>> {
        let (owner, name) = project
            .split_once('/')
            .ok_or_else(|| invalid("reading review threads", "expected owner/name"))?;
        let mut out = Vec::new();
        let mut after = Value::Null;
        for _ in 0..MAX_PAGES {
            let data = self
                .graphql(
                    "reading the review threads of a GitHub pull request",
                    THREADS_QUERY,
                    json!({ "owner": owner, "name": name, "number": number, "after": after }),
                )
                .await?;
            let threads = &data["repository"]["pullRequest"]["reviewThreads"];
            if threads.is_null() {
                return Err(Error::Decode("no reviewThreads in the answer".to_string()));
            }
            let nodes: Vec<GqlThread> = serde_json::from_value(threads["nodes"].clone())
                .map_err(|e| Error::Decode(e.to_string()))?;
            out.extend(nodes.into_iter().map(thread_of));
            if threads["pageInfo"]["hasNextPage"].as_bool() == Some(true) {
                after = threads["pageInfo"]["endCursor"].clone();
            } else {
                break;
            }
        }
        Ok(out)
    }

    pub(crate) async fn review_conversation(
        &self,
        project: &str,
        number: u64,
    ) -> Result<Vec<ReviewComment>> {
        let url = self.repo_url(project, &format!("issues/{number}/comments?"));
        let rows: Vec<IssueComment> = fetch_pages(
            &*self.transport,
            |u| self.get(u),
            &url,
            "listing the comments of a GitHub pull request",
        )
        .await?;
        Ok(rows.into_iter().map(IssueComment::into_comment).collect())
    }

    pub(crate) async fn review_list_reviews(
        &self,
        project: &str,
        number: u64,
    ) -> Result<Vec<SubmittedReview>> {
        let url = self.repo_url(project, &format!("pulls/{number}/reviews?"));
        let rows: Vec<ReviewRow> = fetch_pages(
            &*self.transport,
            |u| self.get(u),
            &url,
            "listing the reviews of a GitHub pull request",
        )
        .await?;
        Ok(rows
            .into_iter()
            .filter_map(|r| {
                let state = match r.state.to_ascii_uppercase().as_str() {
                    "APPROVED" => ReviewState::Approved,
                    "CHANGES_REQUESTED" => ReviewState::ChangesRequested,
                    // Without a summary it only carries code comments, which are
                    // already threads (a reply creates one such review).
                    "COMMENTED" if !r.body.as_deref().unwrap_or_default().trim().is_empty() => {
                        ReviewState::Commented
                    }
                    "DISMISSED" => ReviewState::Dismissed,
                    // PENDING is somebody's unsent draft.
                    _ => return None,
                };
                Some(SubmittedReview {
                    id: r.id.to_string(),
                    author: r.user.map(|u| u.login).unwrap_or_default(),
                    state,
                    body: r.body.unwrap_or_default(),
                    submitted_at: r.submitted_at.unwrap_or_default(),
                })
            })
            .collect())
    }

    pub(crate) async fn review_reply(&self, thread_id: &str, body: &str) -> Result<ReviewComment> {
        if body.trim().is_empty() {
            return Err(invalid("replying to a thread", "the reply is empty"));
        }
        let data = self
            .graphql(
                "replying to a GitHub review thread",
                REPLY_MUTATION,
                json!({ "thread": thread_id, "body": body }),
            )
            .await?;
        let comment: GqlComment =
            serde_json::from_value(data["addPullRequestReviewThreadReply"]["comment"].clone())
                .map_err(|e| Error::Decode(e.to_string()))?;
        Ok(comment.into_comment())
    }

    pub(crate) async fn review_resolve(&self, thread_id: &str, resolved: bool) -> Result<()> {
        let (what, query) = if resolved {
            ("resolving a GitHub review thread", RESOLVE_MUTATION)
        } else {
            ("reopening a GitHub review thread", UNRESOLVE_MUTATION)
        };
        self.graphql(what, query, json!({ "thread": thread_id }))
            .await
            .map(|_| ())
    }

    pub(crate) async fn review_submit(
        &self,
        project: &str,
        number: u64,
        review: &ReviewSubmission,
    ) -> Result<ReviewOutcome> {
        validate_submission(CAPABILITIES, review)?;
        let event = match review.event {
            ReviewEvent::Approve => "APPROVE",
            ReviewEvent::RequestChanges => "REQUEST_CHANGES",
            ReviewEvent::Comment => "COMMENT",
        };
        let comments: Vec<Value> = review
            .comments
            .iter()
            .map(|c| {
                let side = graphql_side(c.side);
                let mut row = json!({
                    "path": c.path, "body": c.body, "line": c.line, "side": side,
                });
                if let Some(start) = c.start_line.filter(|s| *s < c.line) {
                    row["start_line"] = json!(start);
                    row["start_side"] = json!(side);
                }
                row
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
        response.error_for_status("submitting a GitHub review")?;
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

fn thread_of(t: GqlThread) -> ReviewThread {
    let side = side_of(t.diff_side.as_deref());
    let outdated = t.is_outdated;
    // GitHub keeps `line` for a thread that still sits on code and nulls it
    // once outdated; `originalLine` is where it was written.
    let line = if outdated { None } else { t.line };
    let original_line = t.original_line.or(t.line);
    let start_line = if outdated {
        t.original_start_line
    } else {
        t.start_line.or(t.original_start_line)
    };
    ReviewThread {
        id: t.id,
        path: t.path,
        line,
        original_line,
        side,
        start_line,
        resolved: t.is_resolved,
        outdated,
        resolvable: true,
        comments: t
            .comments
            .nodes
            .into_iter()
            .map(GqlComment::into_comment)
            .collect(),
    }
}

// ---- the code through the API (slice S3a) ----

#[derive(Deserialize)]
struct CommitRow {
    #[serde(default)]
    sha: String,
    #[serde(default)]
    parents: Vec<ShaRow>,
    commit: Option<CommitBody>,
    author: Option<Login>,
}

#[derive(Deserialize)]
struct ShaRow {
    #[serde(default)]
    sha: String,
}

#[derive(Deserialize)]
struct CommitBody {
    #[serde(default)]
    message: String,
    author: Option<CommitPerson>,
}

#[derive(Deserialize)]
struct CommitPerson {
    #[serde(default)]
    name: String,
    #[serde(default)]
    date: String,
}

#[derive(Deserialize)]
struct CommitWithFiles {
    #[serde(default)]
    files: Vec<FileRow>,
}

#[derive(Deserialize)]
struct ContentRow {
    #[serde(default)]
    content: String,
    #[serde(default)]
    encoding: String,
    #[serde(default)]
    size: u64,
}

fn api_file_of(row: FileRow) -> ApiFileDiff {
    let status = match file_status(&row.status) {
        FileStatus::Added => gitpoulpi_core::domain::FileStatus::Added,
        FileStatus::Removed => gitpoulpi_core::domain::FileStatus::Deleted,
        FileStatus::Renamed => gitpoulpi_core::domain::FileStatus::Renamed,
        FileStatus::Modified => gitpoulpi_core::domain::FileStatus::Modified,
    };
    // A file that changed lines but came without a patch is one the host
    // chose not to show (too big, too many files); a binary file, a pure
    // rename or a mode change has no changed lines to show.
    let omitted = (row.patch.is_none() && row.additions + row.deletions > 0)
        .then(|| NOT_SHOWN_BY_THE_HOST.to_string());
    ApiFileDiff {
        diff: gitpoulpi_core::diff_text::parse_hunks(
            &row.filename,
            status,
            row.patch.as_deref().unwrap_or_default(),
        ),
        previous_path: row.previous_filename,
        additions: row.additions,
        deletions: row.deletions,
        omitted,
    }
}

impl GitHubClient {
    pub(crate) async fn review_diff(&self, project: &str, number: u64) -> Result<Vec<ApiFileDiff>> {
        let url = self.repo_url(project, &format!("pulls/{number}/files?"));
        let rows: Vec<FileRow> = fetch_pages(
            &*self.transport,
            |u| self.get(u),
            &url,
            "reading the diff of a GitHub pull request",
        )
        .await?;
        Ok(rows.into_iter().map(api_file_of).collect())
    }

    pub(crate) async fn review_commits(
        &self,
        project: &str,
        number: u64,
    ) -> Result<Vec<ApiCommit>> {
        let url = self.repo_url(project, &format!("pulls/{number}/commits?"));
        let rows: Vec<CommitRow> = fetch_pages(
            &*self.transport,
            |u| self.get(u),
            &url,
            "listing the commits of a GitHub pull request",
        )
        .await?;
        Ok(rows
            .into_iter()
            .map(|r| {
                let (message, person) = r.commit.map(|c| (c.message, c.author)).unwrap_or_default();
                let date = person.as_ref().map(|p| p.date.clone()).unwrap_or_default();
                let author = r
                    .author
                    .map(|a| a.login)
                    .or_else(|| person.map(|p| p.name))
                    .unwrap_or_default();
                ApiCommit {
                    sha: r.sha,
                    parents: r.parents.into_iter().map(|p| p.sha).collect(),
                    author,
                    date,
                    summary: summary_of(&message),
                }
            })
            .collect())
    }

    pub(crate) async fn review_commit_diff(
        &self,
        project: &str,
        sha: &str,
    ) -> Result<Vec<ApiFileDiff>> {
        let url = self.repo_url(project, &format!("commits/{}", encode_component(sha)));
        let commit: CommitWithFiles = self
            .get_json(url, "reading a commit of a GitHub repository")
            .await?;
        Ok(commit.files.into_iter().map(api_file_of).collect())
    }

    /// `contents/<path>?ref=`: JSON with the bytes in base64, so a binary
    /// file survives (the transport hands bodies over as text). The route
    /// answers up to 1 MB; a bigger file is an error, not a cut.
    pub(crate) async fn review_file(
        &self,
        project: &str,
        path: &str,
        rev: Option<&str>,
    ) -> Result<Option<Vec<u8>>> {
        let mut url = self.repo_url(project, &format!("contents/{}", encode_path(path)));
        if let Some(rev) = rev {
            url.push_str(&format!("?ref={}", encode_component(rev)));
        }
        let response = self.transport.send(self.get(url)).await?;
        if response.status == 404 {
            return Ok(None);
        }
        response.error_for_status("reading a file of a GitHub repository")?;
        let row: ContentRow = response
            .json()
            .map_err(|_| invalid("reading a file", format!("{path} is not a file")))?;
        if row.encoding != "base64" {
            return Err(invalid(
                "reading a file",
                format!(
                    "{path} is too large for the contents API ({} bytes, 1 MB at most)",
                    row.size
                ),
            ));
        }
        base64_decode(&row.content)
            .map(Some)
            .ok_or_else(|| Error::Decode(format!("the content of {path} is not base64")))
    }
}
