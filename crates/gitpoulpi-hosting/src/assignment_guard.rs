//! The one place a repository's "assignment is blocked" setting is enforced
//! for the host calls: every client [`crate::client_for`] builds is wrapped, so
//! the TUI, the CLI and the MCP server are refused alike, whichever form or
//! command carried the assignee. Reads and everything else pass through.

use crate::client::{CreatedIssue, CreatedPullRequest, HostingClient, NewIssue, NewPullRequest};
use crate::conversation::{
    Comment, ConversationCaps, ItemKey, PageCursor, PostingIdentity, TimelineOpts, TimelinePage,
};
use crate::integration::Integration;
use crate::issue_flow::{CreatedBranch, IssueLinks, NewIssueBranch};
use crate::metadata::{HostedLabel, HostedMilestone, IssueEdit, ItemMeta, MetaKind, PrEdit};
use crate::model::{
    HostedUser, IssueCreateOptions, IssueFilter, IssueSummary, PrFilter, PullRequestSummary,
    TokenCheck,
};
use crate::project::{HostedProject, HostedSshKey};
use crate::review::{
    PullRequestDetail, ReviewCapabilities, ReviewComment, ReviewFile, ReviewOutcome,
    ReviewSubmission, ReviewThread,
};
use crate::{scope, Error, Result};

pub(crate) struct AssignmentGuard {
    inner: Box<dyn HostingClient>,
    id: String,
    scopes: Option<std::sync::Arc<dyn scope::RepoScopes>>,
}

impl AssignmentGuard {
    pub(crate) fn wrap(inner: Box<dyn HostingClient>) -> Box<dyn HostingClient> {
        let id = inner.integration().id.clone();
        Box::new(Self {
            inner,
            id,
            scopes: None,
        })
    }

    #[cfg(test)]
    fn with_scopes(
        inner: Box<dyn HostingClient>,
        scopes: std::sync::Arc<dyn scope::RepoScopes>,
    ) -> Self {
        let id = inner.integration().id.clone();
        Self {
            inner,
            id,
            scopes: Some(scopes),
        }
    }

    /// Refuses when `project` blocks assignment and `assigning` is true.
    fn check(&self, project: &str, assigning: bool) -> Result<()> {
        let blocked = match &self.scopes {
            Some(s) => s.assignment_blocked(&self.id, project),
            None => scope::assignment_blocked(&self.id, project),
        };
        if assigning && blocked {
            return Err(scope::refuse());
        }
        Ok(())
    }
}

fn some_and_not_empty(list: &Option<Vec<String>>) -> bool {
    list.as_ref().is_some_and(|l| !l.is_empty())
}

#[async_trait::async_trait]
impl HostingClient for AssignmentGuard {
    fn integration(&self) -> &Integration {
        self.inner.integration()
    }
    fn unsupported(&self, what: &str) -> Error {
        self.inner.unsupported(what)
    }
    async fn list_projects(&self) -> Result<Vec<HostedProject>> {
        self.inner.list_projects().await
    }
    async fn list_ssh_keys(&self) -> Result<Vec<HostedSshKey>> {
        self.inner.list_ssh_keys().await
    }
    async fn upload_ssh_key(&self, title: &str, public_key: &str) -> Result<HostedSshKey> {
        self.inner.upload_ssh_key(title, public_key).await
    }
    async fn create_pull_request(&self, req: &NewPullRequest) -> Result<CreatedPullRequest> {
        self.check(&req.project, !req.assignees.is_empty())?;
        self.inner.create_pull_request(req).await
    }
    async fn current_user(&self) -> Result<HostedUser> {
        self.inner.current_user().await
    }
    async fn protected_branches(
        &self,
        project: &str,
    ) -> Result<crate::protection::ProtectedBranches> {
        self.inner.protected_branches(project).await
    }
    async fn check_token(&self) -> Result<TokenCheck> {
        self.inner.check_token().await
    }
    async fn list_pull_requests(
        &self,
        project: &str,
        filter: &PrFilter,
    ) -> Result<Vec<PullRequestSummary>> {
        self.inner.list_pull_requests(project, filter).await
    }
    async fn pull_request_for_branch(
        &self,
        project: &str,
        branch: &str,
    ) -> Result<Option<PullRequestSummary>> {
        self.inner.pull_request_for_branch(project, branch).await
    }
    async fn list_issues(&self, project: &str, filter: &IssueFilter) -> Result<Vec<IssueSummary>> {
        self.inner.list_issues(project, filter).await
    }
    async fn create_issue(&self, req: &NewIssue) -> Result<CreatedIssue> {
        self.check(&req.project, !req.assignees.is_empty())?;
        self.inner.create_issue(req).await
    }
    async fn issue_create_options(&self, project: &str) -> Result<IssueCreateOptions> {
        self.inner.issue_create_options(project).await
    }
    async fn delete_ssh_key(&self, id: &str) -> Result<()> {
        self.inner.delete_ssh_key(id).await
    }
    fn review_capabilities(&self) -> ReviewCapabilities {
        self.inner.review_capabilities()
    }
    async fn pull_request_detail(&self, project: &str, number: u64) -> Result<PullRequestDetail> {
        self.inner.pull_request_detail(project, number).await
    }
    async fn pull_request_detail_at(
        &self,
        project: &str,
        number: u64,
        patch_set: Option<u32>,
    ) -> Result<PullRequestDetail> {
        self.inner
            .pull_request_detail_at(project, number, patch_set)
            .await
    }
    async fn pull_request_files_at(
        &self,
        project: &str,
        number: u64,
        patch_set: Option<u32>,
    ) -> Result<Vec<ReviewFile>> {
        self.inner
            .pull_request_files_at(project, number, patch_set)
            .await
    }
    async fn pull_request_files_against(
        &self,
        project: &str,
        number: u64,
        patch_set: Option<u32>,
        base_patch_set: Option<u32>,
    ) -> Result<Vec<ReviewFile>> {
        self.inner
            .pull_request_files_against(project, number, patch_set, base_patch_set)
            .await
    }
    async fn submit_change(&self, project: &str, number: u64) -> Result<()> {
        self.inner.submit_change(project, number).await
    }
    async fn abandon_change(
        &self,
        project: &str,
        number: u64,
        message: Option<&str>,
    ) -> Result<()> {
        self.inner.abandon_change(project, number, message).await
    }
    async fn pull_request_threads_at(
        &self,
        project: &str,
        number: u64,
        patch_set: Option<u32>,
    ) -> Result<Vec<ReviewThread>> {
        self.inner
            .pull_request_threads_at(project, number, patch_set)
            .await
    }
    async fn pull_request_files(&self, project: &str, number: u64) -> Result<Vec<ReviewFile>> {
        self.inner.pull_request_files(project, number).await
    }
    async fn pull_request_threads(&self, project: &str, number: u64) -> Result<Vec<ReviewThread>> {
        self.inner.pull_request_threads(project, number).await
    }
    async fn pull_request_conversation(
        &self,
        project: &str,
        number: u64,
    ) -> Result<Vec<ReviewComment>> {
        self.inner.pull_request_conversation(project, number).await
    }
    async fn pull_request_reviews(
        &self,
        project: &str,
        number: u64,
    ) -> Result<Vec<crate::review::SubmittedReview>> {
        self.inner.pull_request_reviews(project, number).await
    }
    async fn add_comment(&self, project: &str, number: u64, body: &str) -> Result<ReviewComment> {
        self.inner.add_comment(project, number, body).await
    }
    async fn reply_to_thread(
        &self,
        project: &str,
        number: u64,
        thread_id: &str,
        body: &str,
    ) -> Result<ReviewComment> {
        self.inner
            .reply_to_thread(project, number, thread_id, body)
            .await
    }
    async fn resolve_thread(
        &self,
        project: &str,
        number: u64,
        thread_id: &str,
        resolved: bool,
    ) -> Result<()> {
        self.inner
            .resolve_thread(project, number, thread_id, resolved)
            .await
    }
    async fn submit_review(
        &self,
        project: &str,
        number: u64,
        review: &ReviewSubmission,
    ) -> Result<ReviewOutcome> {
        self.inner.submit_review(project, number, review).await
    }
    fn conversation_caps(&self, kind: crate::conversation::ItemKind) -> ConversationCaps {
        self.inner.conversation_caps(kind)
    }
    async fn posting_identity(&self, project: &str) -> Result<PostingIdentity> {
        self.inner.posting_identity(project).await
    }
    async fn item_timeline(
        &self,
        project: &str,
        key: &ItemKey,
        older: Option<&PageCursor>,
        opts: &TimelineOpts,
    ) -> Result<TimelinePage> {
        self.inner.item_timeline(project, key, older, opts).await
    }
    async fn post_item_comment(
        &self,
        project: &str,
        key: &ItemKey,
        body: &str,
        internal: bool,
    ) -> Result<Comment> {
        self.inner
            .post_item_comment(project, key, body, internal)
            .await
    }
    async fn reply_in_discussion(
        &self,
        project: &str,
        key: &ItemKey,
        discussion_id: &str,
        body: &str,
    ) -> Result<Comment> {
        self.inner
            .reply_in_discussion(project, key, discussion_id, body)
            .await
    }
    async fn edit_item_comment(
        &self,
        project: &str,
        key: &ItemKey,
        comment_id: &str,
        body: &str,
    ) -> Result<Comment> {
        self.inner
            .edit_item_comment(project, key, comment_id, body)
            .await
    }
    async fn delete_item_comment(
        &self,
        project: &str,
        key: &ItemKey,
        comment_id: &str,
    ) -> Result<()> {
        self.inner
            .delete_item_comment(project, key, comment_id)
            .await
    }
    async fn create_issue_branch(&self, req: &NewIssueBranch) -> Result<CreatedBranch> {
        self.inner.create_issue_branch(req).await
    }
    async fn issue_links(&self, project: &str, number: u64) -> Result<IssueLinks> {
        self.inner.issue_links(project, number).await
    }
    async fn branch_names(&self, project: &str, search: &str) -> Result<Vec<String>> {
        self.inner.branch_names(project, search).await
    }
    async fn project_members(&self, project: &str, query: &str) -> Result<Vec<HostedUser>> {
        self.inner.project_members(project, query).await
    }
    async fn project_labels(&self, project: &str) -> Result<Vec<HostedLabel>> {
        self.inner.project_labels(project).await
    }
    async fn project_milestones(&self, project: &str) -> Result<Vec<HostedMilestone>> {
        self.inner.project_milestones(project).await
    }
    async fn item_meta(&self, project: &str, number: u64, kind: MetaKind) -> Result<ItemMeta> {
        self.inner.item_meta(project, number, kind).await
    }
    async fn update_issue(&self, project: &str, number: u64, edit: &IssueEdit) -> Result<()> {
        self.check(project, some_and_not_empty(&edit.assignees))?;
        self.inner.update_issue(project, number, edit).await
    }
    async fn issue_statuses(&self, project: &str) -> Result<Vec<String>> {
        self.inner.issue_statuses(project).await
    }
    async fn set_issue_status(&self, project: &str, number: u64, status: &str) -> Result<()> {
        self.inner.set_issue_status(project, number, status).await
    }
    async fn set_issue_state(&self, project: &str, number: u64, closed: bool) -> Result<()> {
        self.inner.set_issue_state(project, number, closed).await
    }
    fn board_source(&self) -> crate::board::BoardSource {
        self.inner.board_source()
    }
    async fn move_issue(
        &self,
        project: &str,
        issue: &IssueSummary,
        spec: &crate::board::BoardSpec,
        to: &crate::board::BoardColumn,
    ) -> Result<()> {
        self.inner.move_issue(project, issue, spec, to).await
    }
    async fn update_pull_request(&self, project: &str, number: u64, edit: &PrEdit) -> Result<()> {
        self.check(project, some_and_not_empty(&edit.assignees))?;
        self.inner.update_pull_request(project, number, edit).await
    }
    async fn pull_request_diff(
        &self,
        project: &str,
        number: u64,
        patch_set: Option<u32>,
        compare: Option<u32>,
    ) -> Result<Vec<crate::review::ApiFileDiff>> {
        self.inner
            .pull_request_diff(project, number, patch_set, compare)
            .await
    }
    async fn pull_request_commits(
        &self,
        project: &str,
        number: u64,
    ) -> Result<Vec<crate::review::ApiCommit>> {
        self.inner.pull_request_commits(project, number).await
    }
    async fn commit_diff(
        &self,
        project: &str,
        sha: &str,
    ) -> Result<Vec<crate::review::ApiFileDiff>> {
        self.inner.commit_diff(project, sha).await
    }
    async fn file_content(
        &self,
        project: &str,
        path: &str,
        rev: Option<&str>,
    ) -> Result<Option<Vec<u8>>> {
        self.inner.file_content(project, path, rev).await
    }
    async fn account_items(
        &self,
        query: &crate::account::AccountQuery,
    ) -> Result<crate::account::AccountItems> {
        self.inner.account_items(query).await
    }
}

#[cfg(test)]
mod tests;
