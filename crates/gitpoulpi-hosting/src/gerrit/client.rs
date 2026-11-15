//! The Gerrit client (review only).

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use serde::de::DeserializeOwned;
use serde_json::{json, Value};

use crate::auth::auth_headers;
use crate::client::{
    encode_component, CreatedPullRequest, HostingClient, NewPullRequest, MAX_PAGES,
};
use crate::conversation::{
    Comment, ConversationCaps, ItemKey, ItemKind, PageCursor, PostingIdentity, TimelineOpts,
    TimelinePage,
};
use crate::http::{HttpRequest, HttpResponse, HttpTransport};
use crate::integration::Integration;
use crate::metadata::{ItemMeta, MetaKind};
use crate::model::{
    best_pull_request, HostedUser, PrFilter, PullRequestSummary, StateFilter, TokenCheck,
};
use crate::project::{HostedProject, HostedSshKey};
use crate::review::model::{
    DiffSide, MergeState, PullRequestDetail, ReviewCapabilities, ReviewComment, ReviewEvent,
    ReviewFile, ReviewOutcome, ReviewSubmission, ReviewThread, SubmittedReview,
};
use crate::review::{invalid, validate_submission};
use crate::{Error, Result};

use super::map::{self, Account, ChangeInfo, CommentInfo};

mod account;
mod code;
mod conversation;

/// Changes of one page.
const PAGE: usize = 100;
/// A list shows at most this many changes.
const MAX_CHANGES: usize = 500;

/// A change's patch-set-level comments are listed under this pseudo path.
const PATCHSET_LEVEL: &str = "/PATCHSET_LEVEL";

pub(crate) struct GerritClient {
    integration: Integration,
    token: String,
    transport: Arc<dyn HttpTransport>,
    me: Mutex<Option<Account>>,
}

impl GerritClient {
    pub(crate) fn new(
        integration: Integration,
        token: String,
        transport: Arc<dyn HttpTransport>,
    ) -> Self {
        Self {
            integration,
            token,
            transport,
            me: Mutex::new(None),
        }
    }

    fn site(&self) -> String {
        self.integration
            .url
            .trim()
            .trim_end_matches('/')
            .to_string()
    }

    fn api(&self) -> String {
        self.integration.api_base()
    }

    fn authed(&self, request: HttpRequest) -> HttpRequest {
        let mut request = request.header("Accept", "application/json");
        for (name, value) in auth_headers(&self.integration, &self.token) {
            request = request.header(name, value);
        }
        request
    }

    fn decode<T: DeserializeOwned>(response: &HttpResponse) -> Result<T> {
        serde_json::from_str(map::strip_prefix(&response.body))
            .map_err(|e| Error::Decode(e.to_string()))
    }

    async fn get_json<T: DeserializeOwned>(&self, path: &str, what: &str) -> Result<T> {
        let url = format!("{}{path}", self.api());
        let response = self
            .transport
            .send(self.authed(HttpRequest::get(url)))
            .await?;
        response.error_for_status(what)?;
        Self::decode(&response)
    }

    /// `{project}~{number}`, the id a change is addressed by.
    fn change_id(project: &str, number: u64) -> String {
        format!("{}~{number}", encode_component(project.trim()))
    }

    async fn myself(&self) -> Result<Account> {
        if let Some(me) = self.me.lock().unwrap().clone() {
            return Ok(me);
        }
        let me: Account = self
            .get_json("/accounts/self", "checking the Gerrit credentials")
            .await?;
        *self.me.lock().unwrap() = Some(me.clone());
        Ok(me)
    }

    /// One change with the options a review needs.
    async fn change(&self, project: &str, number: u64) -> Result<ChangeInfo> {
        self.get_json(
            &format!(
                "/changes/{}?o=ALL_REVISIONS&o=ALL_COMMITS&o=DETAILED_LABELS&o=DETAILED_ACCOUNTS&o=SUBMITTABLE&o=MESSAGES&o=CHANGE_ACTIONS",
                Self::change_id(project, number)
            ),
            "reading the Gerrit change",
        )
        .await
    }

    /// Changes matching `query`, paged with `n` / `S`.
    async fn changes(&self, query: &str, options: &str) -> Result<Vec<ChangeInfo>> {
        let mut out: Vec<ChangeInfo> = Vec::new();
        for _ in 0..MAX_PAGES {
            let page: Vec<ChangeInfo> = self
                .get_json(
                    &format!(
                        "/changes/?q={}{options}&n={PAGE}&S={}",
                        encode_component(query),
                        out.len()
                    ),
                    "listing Gerrit changes",
                )
                .await?;
            let more = page.last().is_some_and(|c| c._more_changes);
            out.extend(page);
            if !more || out.len() >= MAX_CHANGES {
                break;
            }
        }
        out.truncate(MAX_CHANGES);
        Ok(out)
    }

    /// The comments of a change (published, then the user's drafts), each
    /// with its file.
    async fn comments(&self, project: &str, number: u64) -> Result<Vec<CommentInfo>> {
        let id = Self::change_id(project, number);
        let published: BTreeMap<String, Vec<CommentInfo>> = self
            .get_json(
                &format!("/changes/{id}/comments"),
                "reading the Gerrit comments",
            )
            .await?;
        let drafts: BTreeMap<String, Vec<CommentInfo>> = self
            .get_json(
                &format!("/changes/{id}/drafts"),
                "reading the Gerrit drafts",
            )
            .await
            // Drafts are optional: a token may not read them.
            .unwrap_or_default();
        let mut out = Vec::new();
        for (set, draft) in [(published, false), (drafts, true)] {
            for (path, rows) in set {
                for mut row in rows {
                    row.path = path.clone();
                    row.draft = draft;
                    out.push(row);
                }
            }
        }
        out.sort_by(|a, b| a.updated.cmp(&b.updated));
        Ok(out)
    }

    /// Posts a review input on a revision (`current`, a patch set number or a sha).
    async fn review(
        &self,
        project: &str,
        number: u64,
        revision: &str,
        input: &Value,
    ) -> Result<()> {
        let path = format!(
            "/changes/{}/revisions/{}/review",
            Self::change_id(project, number),
            encode_component(revision)
        );
        let url = format!("{}{path}", self.api());
        let response = self
            .transport
            .send(self.authed(HttpRequest::post_json(url, input)))
            .await?;
        response.error_for_status("posting to Gerrit")?;
        Ok(())
    }

    /// The vote an event stands for, among the values the server permits.
    async fn vote_for_event(
        &self,
        project: &str,
        number: u64,
        event: ReviewEvent,
    ) -> Result<Option<(String, i32)>> {
        let positive = match event {
            ReviewEvent::Comment => return Ok(None),
            ReviewEvent::Approve => true,
            ReviewEvent::RequestChanges => false,
        };
        let change = self.change(project, number).await?;
        let me = self.myself().await?._account_id;
        let labels = change.vote_labels(me);
        let review = labels
            .iter()
            .find(|l| l.name == "Code-Review")
            .ok_or_else(|| {
                invalid(
                    "submitting a review",
                    "you may not vote on Code-Review on this change",
                )
            })?;
        let value = if positive {
            review
                .values
                .iter()
                .map(|v| v.value)
                .filter(|v| *v > 0)
                .max()
        } else {
            // -1 ("prefer not") before the -2 veto.
            review
                .values
                .iter()
                .map(|v| v.value)
                .filter(|v| *v < 0)
                .max()
        };
        match value {
            Some(v) => Ok(Some(("Code-Review".to_string(), v))),
            None => Err(invalid(
                "submitting a review",
                format!(
                    "the server permits no {} Code-Review vote for you on this change",
                    if positive { "positive" } else { "negative" }
                ),
            )),
        }
    }

    pub(super) fn thread_id(root_last: &str, c: &CommentInfo, unresolved: bool) -> String {
        let side = if c.side.as_deref() == Some("PARENT") {
            'o'
        } else {
            'n'
        };
        format!(
            "{root_last}|{}|{}|{side}|{}|{}",
            c.patch_set,
            u8::from(unresolved),
            c.line.unwrap_or(0),
            c.path
        )
    }
}

/// What a thread id carries.
pub(super) struct ThreadRef {
    pub last: String,
    pub patch_set: u32,
    pub unresolved: bool,
    pub side: DiffSide,
    pub line: u32,
    pub path: String,
}

pub(super) fn parse_thread_id(id: &str) -> Option<ThreadRef> {
    let mut parts = id.splitn(6, '|');
    let last = parts.next()?.to_string();
    let patch_set = parts.next()?.parse().ok()?;
    let unresolved = parts.next()? == "1";
    let side = if parts.next()? == "o" {
        DiffSide::Old
    } else {
        DiffSide::New
    };
    let line = parts.next()?.parse().ok()?;
    let path = parts.next()?.to_string();
    (!last.is_empty() && !path.is_empty()).then_some(ThreadRef {
        last,
        patch_set,
        unresolved,
        side,
        line,
        path,
    })
}

/// Groups the flat comment list into threads: a root and everything that
/// answers it, directly or not.
pub(super) fn build_threads(
    comments: Vec<CommentInfo>,
    shown_patch_set: u32,
    me: &str,
) -> Vec<ReviewThread> {
    let by_id: BTreeMap<String, usize> = comments
        .iter()
        .enumerate()
        .map(|(i, c)| (c.id.clone(), i))
        .collect();
    let root_of = |mut i: usize| -> usize {
        for _ in 0..comments.len() {
            match comments[i].in_reply_to.as_ref().and_then(|p| by_id.get(p)) {
                Some(&parent) => i = parent,
                None => break,
            }
        }
        i
    };
    let mut order: Vec<usize> = Vec::new();
    let mut members: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    for (i, comment) in comments.iter().enumerate() {
        if comment.path == PATCHSET_LEVEL {
            continue;
        }
        let root = root_of(i);
        if !members.contains_key(&root) {
            order.push(root);
        }
        members.entry(root).or_default().push(i);
    }
    order
        .into_iter()
        .map(|root| {
            let rows = &members[&root];
            let first = &comments[root];
            let last = &comments[*rows.last().unwrap()];
            // The newest comment decides: unresolved unless it says otherwise.
            let unresolved = last.unresolved.unwrap_or(false);
            let outdated = first.patch_set != shown_patch_set;
            let side = if first.side.as_deref() == Some("PARENT") {
                DiffSide::Old
            } else {
                DiffSide::New
            };
            // A file-level comment sits at the top of the file.
            let line = first
                .line
                .or(first.range.as_ref().map(|r| r.end_line))
                .filter(|l| *l > 0)
                .unwrap_or(1);
            let start_line = first
                .range
                .as_ref()
                .map(|r| r.start_line)
                .filter(|s| *s > 0 && *s < line);
            // Replies go to the newest comment that is not a draft.
            let reply_to = rows
                .iter()
                .rev()
                .map(|&i| &comments[i])
                .find(|c| !c.draft)
                .unwrap_or(first);
            ReviewThread {
                id: GerritClient::thread_id(&reply_to.id, first, unresolved),
                path: first.path.clone(),
                line: (!outdated).then_some(line),
                original_line: Some(line),
                side,
                start_line: if outdated { None } else { start_line },
                resolved: !unresolved,
                outdated,
                resolvable: true,
                comments: rows
                    .iter()
                    .map(|&i| {
                        let c = &comments[i];
                        let who = c
                            .author
                            .as_ref()
                            .map(Account::handle)
                            // A draft carries no author: it is the user's.
                            .unwrap_or_else(|| me.to_string());
                        ReviewComment {
                            id: c.id.clone(),
                            author: if c.draft {
                                format!("{who} (draft)")
                            } else {
                                who
                            },
                            body: c.message.clone(),
                            created_at: map::iso(&c.updated),
                            in_reply_to: c.in_reply_to.clone(),
                            web_url: None,
                        }
                    })
                    .collect(),
            }
        })
        .collect()
}

fn changes_query(project: &str, filter: &PrFilter) -> String {
    let mut q = format!("project:{}", quote(project));
    match filter.state {
        StateFilter::Open => q.push_str(" status:open"),
        StateFilter::Closed => q.push_str(" (status:merged OR status:abandoned)"),
        StateFilter::All => {}
    }
    if filter.mine {
        q.push_str(" owner:self");
    }
    if filter.review_requested {
        q.push_str(" reviewer:self -owner:self");
    }
    q
}

/// A project name as a query operand: quoted when it has odd characters.
fn quote(text: &str) -> String {
    if text
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '/'))
    {
        text.to_string()
    } else {
        format!("\"{}\"", text.replace('"', "\\\""))
    }
}

#[async_trait::async_trait]
impl HostingClient for GerritClient {
    fn integration(&self) -> &Integration {
        &self.integration
    }

    async fn list_projects(&self) -> Result<Vec<HostedProject>> {
        #[derive(serde::Deserialize)]
        struct Info {
            #[serde(default)]
            description: Option<String>,
            #[serde(default)]
            state: Option<String>,
        }
        let rows: BTreeMap<String, Info> = self
            .get_json("/projects/?type=CODE&d&n=500", "listing Gerrit projects")
            .await?;
        let site = self.site();
        Ok(rows
            .into_iter()
            .filter(|(_, i)| i.state.as_deref().unwrap_or("ACTIVE") == "ACTIVE")
            .map(|(name, i)| map::hosted_project(&site, &self.integration.id, &name, i.description))
            .collect())
    }

    async fn pull_request_diff(
        &self,
        project: &str,
        number: u64,
        patch_set: Option<u32>,
        compare: Option<u32>,
    ) -> Result<Vec<crate::review::ApiFileDiff>> {
        self.review_diff(project, number, patch_set, compare).await
    }

    async fn pull_request_commits(
        &self,
        project: &str,
        number: u64,
    ) -> Result<Vec<crate::review::ApiCommit>> {
        self.review_commits(project, number).await
    }

    async fn commit_diff(
        &self,
        project: &str,
        sha: &str,
    ) -> Result<Vec<crate::review::ApiFileDiff>> {
        self.review_commit_diff(project, sha).await
    }

    async fn file_content(
        &self,
        project: &str,
        path: &str,
        rev: Option<&str>,
    ) -> Result<Option<Vec<u8>>> {
        self.review_file(project, path, rev).await
    }

    async fn account_items(
        &self,
        query: &crate::account::AccountQuery,
    ) -> Result<crate::account::AccountItems> {
        account::fetch(self, query).await
    }

    async fn list_ssh_keys(&self) -> Result<Vec<HostedSshKey>> {
        Err(self.unsupported("list_ssh_keys"))
    }

    async fn upload_ssh_key(&self, _title: &str, _public_key: &str) -> Result<HostedSshKey> {
        Err(self.unsupported("upload_ssh_key"))
    }

    /// A change is created by pushing to `refs/for/<branch>`, never by an API call.
    async fn create_pull_request(&self, _req: &NewPullRequest) -> Result<CreatedPullRequest> {
        Err(Error::Unsupported(
            "creating a Gerrit change through the API (push to refs/for/<branch>)".into(),
        ))
    }

    async fn current_user(&self) -> Result<HostedUser> {
        let me = self.myself().await?;
        Ok(HostedUser {
            login: me.handle(),
            name: me.display(),
            id: me._account_id.to_string(),
            web_url: None,
        })
    }

    async fn check_token(&self) -> Result<TokenCheck> {
        Ok(TokenCheck {
            user: self.current_user().await?,
            // HTTP basic carries no scopes.
            scopes: None,
            missing_scopes: Vec::new(),
            expires_at: None,
            ..Default::default()
        })
    }

    async fn list_pull_requests(
        &self,
        project: &str,
        filter: &PrFilter,
    ) -> Result<Vec<PullRequestSummary>> {
        let site = self.site();
        let me = self.myself().await.ok().map(|a| a.handle());
        let rows = self
            .changes(
                &changes_query(project, filter),
                "&o=DETAILED_ACCOUNTS&o=LABELS&o=CURRENT_REVISION",
            )
            .await?;
        Ok(rows
            .iter()
            .map(|c| c.summary(&site, me.as_deref()))
            .collect())
    }

    /// Changes carry the topic the branch was pushed with
    /// (`refs/for/<target>%topic=<branch>`); the best one for `branch`.
    async fn pull_request_for_branch(
        &self,
        project: &str,
        branch: &str,
    ) -> Result<Option<PullRequestSummary>> {
        let site = self.site();
        let query = format!("project:{} topic:{}", quote(project), quote(branch));
        let rows = self
            .changes(&query, "&o=DETAILED_ACCOUNTS&o=LABELS&o=CURRENT_REVISION")
            .await?;
        Ok(best_pull_request(
            rows.iter().map(|c| c.summary(&site, None)).collect(),
        ))
    }

    // -- review ---------------------------------------------------------------

    fn review_capabilities(&self) -> ReviewCapabilities {
        ReviewCapabilities {
            review: true,
            // Approve and request changes are Code-Review votes within what
            // the server permits.
            approve: true,
            request_changes: true,
            line_ranges: true,
            resolve: true,
        }
    }

    async fn pull_request_detail(&self, project: &str, number: u64) -> Result<PullRequestDetail> {
        self.pull_request_detail_at(project, number, None).await
    }

    async fn pull_request_detail_at(
        &self,
        project: &str,
        number: u64,
        patch_set: Option<u32>,
    ) -> Result<PullRequestDetail> {
        let site = self.site();
        let change = self.change(project, number).await?;
        let me = self.myself().await?;
        let (sha, revision) = change.revision(patch_set).ok_or_else(|| Error::Api {
            what: "reading the Gerrit change".into(),
            status: 404,
            detail: match patch_set {
                Some(n) => format!("the change has no patch set {n}"),
                None => "the change has no patch set".into(),
            },
        })?;
        let shown = revision._number;
        let commit = revision.commit.as_ref();
        let base_sha = commit
            .and_then(|c| c.parents.first())
            .map(|p| p.commit.clone())
            .unwrap_or_default();
        let message = commit.map(|c| c.message.as_str()).unwrap_or("");
        let mut summary = change.summary(&site, Some(&me.handle()));
        summary.source_branch = map::change_ref(number, shown);
        let merge_state = if change.mergeable == Some(false) {
            MergeState::Conflicting
        } else if change.submittable == Some(true) {
            MergeState::Clean
        } else if change.state() == crate::model::PullRequestState::Open {
            MergeState::Blocked
        } else {
            MergeState::Unknown
        };
        Ok(PullRequestDetail {
            body: description_of(message),
            head_sha: sha.clone(),
            base_sha,
            merge_state,
            checks: change.checks(),
            created_at: map::iso(&change.created),
            additions: change.insertions,
            deletions: change.deletions,
            changed_files: None,
            capabilities: self.review_capabilities(),
            patch_sets: change.patch_sets(),
            shown_patch_set: Some(shown),
            vote_labels: change.vote_labels(me._account_id),
            can_submit: change.can_submit(),
            can_abandon: change.can_abandon(),
            summary,
        })
    }

    async fn pull_request_files(&self, project: &str, number: u64) -> Result<Vec<ReviewFile>> {
        self.pull_request_files_at(project, number, None).await
    }

    async fn pull_request_files_at(
        &self,
        project: &str,
        number: u64,
        patch_set: Option<u32>,
    ) -> Result<Vec<ReviewFile>> {
        self.pull_request_files_against(project, number, patch_set, None)
            .await
    }

    async fn pull_request_files_against(
        &self,
        project: &str,
        number: u64,
        patch_set: Option<u32>,
        base_patch_set: Option<u32>,
    ) -> Result<Vec<ReviewFile>> {
        let revision = patch_set.map_or_else(|| "current".to_string(), |n| n.to_string());
        let base = base_patch_set.map_or_else(String::new, |n| format!("?base={n}"));
        let files: BTreeMap<String, map::FileInfo> = self
            .get_json(
                &format!(
                    "/changes/{}/revisions/{revision}/files/{base}",
                    Self::change_id(project, number)
                ),
                "listing the files of a Gerrit change",
            )
            .await?;
        Ok(map::map_files(files))
    }

    async fn submit_change(&self, project: &str, number: u64) -> Result<()> {
        self.post_action(
            project,
            number,
            "submit",
            &json!({}),
            "submitting the change",
        )
        .await
    }

    async fn abandon_change(
        &self,
        project: &str,
        number: u64,
        message: Option<&str>,
    ) -> Result<()> {
        let input = match message.map(str::trim).filter(|m| !m.is_empty()) {
            Some(m) => json!({ "message": m }),
            None => json!({}),
        };
        self.post_action(project, number, "abandon", &input, "abandoning the change")
            .await
    }

    async fn pull_request_threads(&self, project: &str, number: u64) -> Result<Vec<ReviewThread>> {
        self.pull_request_threads_at(project, number, None).await
    }

    async fn pull_request_threads_at(
        &self,
        project: &str,
        number: u64,
        patch_set: Option<u32>,
    ) -> Result<Vec<ReviewThread>> {
        let shown = match patch_set {
            Some(n) => n,
            None => self.change_latest(project, number).await?,
        };
        let me = self.myself().await.map(|a| a.handle()).unwrap_or_default();
        Ok(build_threads(
            self.comments(project, number).await?,
            shown,
            &me,
        ))
    }

    /// The change messages, oldest first (comments on code are threads).
    async fn pull_request_conversation(
        &self,
        project: &str,
        number: u64,
    ) -> Result<Vec<ReviewComment>> {
        let change = self.change(project, number).await?;
        let mut out: Vec<ReviewComment> = change
            .messages
            .iter()
            .map(|m| ReviewComment {
                id: m.id.clone(),
                author: m.author.as_ref().map(Account::handle).unwrap_or_default(),
                body: m.message.clone(),
                created_at: map::iso(&m.date),
                in_reply_to: None,
                web_url: None,
            })
            .collect();
        out.sort_by(|a, b| a.created_at.cmp(&b.created_at));
        Ok(out)
    }

    /// The votes on Code-Review, as verdicts.
    async fn pull_request_reviews(
        &self,
        project: &str,
        number: u64,
    ) -> Result<Vec<SubmittedReview>> {
        let change = self.change(project, number).await?;
        Ok(change.code_review_votes())
    }

    /// A patch-set comment, posted at once as a change message.
    async fn add_comment(&self, project: &str, number: u64, body: &str) -> Result<ReviewComment> {
        if body.trim().is_empty() {
            return Err(invalid("commenting", "the comment is empty"));
        }
        self.review(project, number, "current", &json!({ "message": body }))
            .await?;
        Ok(ReviewComment {
            id: String::new(),
            author: self.myself().await?.handle(),
            body: body.to_string(),
            created_at: String::new(),
            in_reply_to: None,
            web_url: None,
        })
    }

    /// An answer to a thread: a comment that names the newest one it
    /// answers, posted at once on that comment's patch set.
    async fn reply_to_thread(
        &self,
        project: &str,
        number: u64,
        thread_id: &str,
        body: &str,
    ) -> Result<ReviewComment> {
        let thread = parse_thread_id(thread_id)
            .ok_or_else(|| invalid("replying", "that is not a Gerrit thread"))?;
        if body.trim().is_empty() {
            return Err(invalid("replying", "the reply is empty"));
        }
        self.post_thread_comment(project, number, &thread, body, thread.unresolved)
            .await
    }

    /// Resolving is a reply with `unresolved: false` (Gerrit keeps the state
    /// on the newest comment of the thread).
    async fn resolve_thread(
        &self,
        project: &str,
        number: u64,
        thread_id: &str,
        resolved: bool,
    ) -> Result<()> {
        let thread = parse_thread_id(thread_id)
            .ok_or_else(|| invalid("resolving", "that is not a Gerrit thread"))?;
        // Gerrit's own "Done" quick reply; reopening says so.
        let text = if resolved { "Done" } else { "Reopened" };
        self.post_thread_comment(project, number, &thread, text, !resolved)
            .await?;
        Ok(())
    }

    async fn submit_review(
        &self,
        project: &str,
        number: u64,
        review: &ReviewSubmission,
    ) -> Result<ReviewOutcome> {
        validate_submission(self.review_capabilities(), review)?;
        let mut labels = serde_json::Map::new();
        for vote in &review.votes {
            labels.insert(vote.label.clone(), json!(vote.value));
        }
        if labels.is_empty() {
            if let Some((name, value)) = self.vote_for_event(project, number, review.event).await? {
                labels.insert(name, json!(value));
            }
        }
        let mut comments: BTreeMap<String, Vec<Value>> = BTreeMap::new();
        for c in &review.comments {
            let mut row = json!({
                "line": c.line,
                "message": c.body,
                // Like Gerrit's own web UI: a new thread asks for an answer.
                // Without the flag the server stores it as already resolved.
                "unresolved": true,
            });
            if c.side == DiffSide::Old {
                row["side"] = json!("PARENT");
            }
            if let Some(start) = c.start_line {
                row["range"] = json!({
                    "start_line": start, "start_character": 0,
                    "end_line": c.line, "end_character": 0
                });
            }
            comments.entry(c.path.clone()).or_default().push(row);
        }
        let mut input = json!({ "drafts": "PUBLISH_ALL_REVISIONS" });
        if !review.body.trim().is_empty() {
            input["message"] = json!(review.body);
        }
        if !labels.is_empty() {
            input["labels"] = Value::Object(labels.clone());
        }
        if !comments.is_empty() {
            input["comments"] = json!(comments);
        }
        // The sha of the patch set that was read: a review of an old patch
        // set stays on it.
        let revision = review.head_sha.clone().unwrap_or_else(|| "current".into());
        self.review(project, number, &revision, &input).await?;
        Ok(ReviewOutcome {
            event: review.event,
            comments: review.comments.len(),
            review_id: None,
        })
    }

    fn conversation_caps(&self, kind: ItemKind) -> ConversationCaps {
        ConversationCaps::gerrit(kind)
    }

    async fn posting_identity(&self, project: &str) -> Result<PostingIdentity> {
        self.conv_identity(project).await
    }

    async fn item_timeline(
        &self,
        project: &str,
        key: &ItemKey,
        older: Option<&PageCursor>,
        opts: &TimelineOpts,
    ) -> Result<TimelinePage> {
        self.conv_timeline(project, key, older, opts).await
    }

    async fn post_item_comment(
        &self,
        project: &str,
        key: &ItemKey,
        body: &str,
        internal: bool,
    ) -> Result<Comment> {
        self.conv_post(project, key, body, internal).await
    }

    async fn reply_in_discussion(
        &self,
        project: &str,
        key: &ItemKey,
        discussion_id: &str,
        body: &str,
    ) -> Result<Comment> {
        self.conv_reply(project, key, discussion_id, body).await
    }

    async fn edit_item_comment(
        &self,
        _project: &str,
        _key: &ItemKey,
        _comment_id: &str,
        _body: &str,
    ) -> Result<Comment> {
        Err(Self::conv_edit_refused())
    }

    async fn delete_item_comment(
        &self,
        _project: &str,
        _key: &ItemKey,
        _comment_id: &str,
    ) -> Result<()> {
        Err(Self::conv_delete_refused())
    }

    async fn item_meta(&self, project: &str, number: u64, _kind: MetaKind) -> Result<ItemMeta> {
        let change = self.change(project, number).await?;
        Ok(ItemMeta {
            assignees: Vec::new(),
            reviewers: change
                .reviewers
                .reviewer
                .iter()
                .map(Account::handle)
                .collect(),
            labels: change.hashtags,
            milestone: None,
        })
    }
}

impl GerritClient {
    /// `POST /changes/{id}/<action>` (submit, abandon).
    async fn post_action(
        &self,
        project: &str,
        number: u64,
        action: &str,
        input: &Value,
        what: &str,
    ) -> Result<()> {
        let url = format!(
            "{}/changes/{}/{action}",
            self.api(),
            Self::change_id(project, number)
        );
        let response = self
            .transport
            .send(self.authed(HttpRequest::post_json(url, input)))
            .await?;
        response.error_for_status(what)?;
        Ok(())
    }

    async fn change_latest(&self, project: &str, number: u64) -> Result<u32> {
        Ok(self.change(project, number).await?.latest_patch_set())
    }

    async fn post_thread_comment(
        &self,
        project: &str,
        number: u64,
        thread: &ThreadRef,
        body: &str,
        unresolved: bool,
    ) -> Result<ReviewComment> {
        let mut row = json!({
            "in_reply_to": thread.last,
            "message": body,
            "unresolved": unresolved,
        });
        if thread.line > 0 {
            row["line"] = json!(thread.line);
        }
        if thread.side == DiffSide::Old {
            row["side"] = json!("PARENT");
        }
        let input = json!({ "comments": { thread.path.clone(): [row] } });
        self.review(project, number, &thread.patch_set.to_string(), &input)
            .await?;
        Ok(ReviewComment {
            id: String::new(),
            author: self.myself().await?.handle(),
            body: body.to_string(),
            created_at: String::new(),
            in_reply_to: Some(thread.last.clone()),
            web_url: None,
        })
    }
}

/// The commit message without its subject line and its trailers
/// (`Change-Id:`, `Signed-off-by:`...).
fn description_of(message: &str) -> String {
    let mut lines: Vec<&str> = message.lines().skip(1).collect();
    let is_trailer = |l: &str| {
        l.split_once(": ").is_some_and(|(k, _)| {
            !k.is_empty() && k.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
        })
    };
    while lines
        .last()
        .is_some_and(|l| l.trim().is_empty() || is_trailer(l))
    {
        lines.pop();
    }
    lines.join("\n").trim().to_string()
}
