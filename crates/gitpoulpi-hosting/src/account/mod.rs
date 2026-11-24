//! Account-wide lists: the requests waiting for my review, every open
//! request of my namespaces and the issues assigned to me, across every
//! repository of a host rather than the one folder a scope is about.
//!
//! One call, [`crate::HostingClient::account_items`], answers a whole
//! [`AccountQuery`] for GitHub, GitLab, Gitea/Forgejo, Azure DevOps, Bitbucket Data Center and Gerrit (the other hosts keep
//! the `Unsupported` default for now). The rows are the ordinary
//! [`PullRequestSummary`] / [`IssueSummary`] plus the project they belong to
//! and a clone URL, so a caller can tell a cloned repository from one that is
//! not. Every list is bounded ([`DEFAULT_LIMIT`] rows per kind, a few pages and
//! queries per round) because a token can see far more than anyone reads.
//!
//! Failure policy: a sub-query that fails for one namespace (a group the token
//! cannot read, a 404) becomes a note and the others still run; a rejected
//! token, a rate limit or a dead network fails the whole call, because every
//! later sub-query would fail the same way.

// The shared plumbing below serves six providers; a build with only some of
// them leaves parts of it without a caller.
#![cfg_attr(
    not(all(
        feature = "github",
        feature = "gitlab",
        feature = "gitea",
        feature = "azure-devops",
        feature = "bitbucket",
        feature = "gerrit"
    )),
    allow(dead_code)
)]

use serde::{Deserialize, Serialize};

use crate::http::{HttpRequest, HttpResponse, HttpTransport};
use crate::model::{IssueSummary, PullRequestState, PullRequestSummary};
use crate::{Error, Result};

mod branches;
mod cache;
#[cfg(feature = "gitea")]
pub(crate) mod gitea;
#[cfg(feature = "github")]
pub(crate) mod github;
#[cfg(feature = "gitlab")]
pub(crate) mod gitlab;
mod namespace;
#[cfg(all(test, feature = "github", feature = "gitlab", feature = "gitea"))]
mod tests;
#[cfg(all(test, feature = "github", feature = "gitlab", feature = "gitea"))]
mod tests_branches;
#[cfg(all(test, feature = "github", feature = "gitlab", feature = "gitea"))]
mod tests_namespaces;
#[cfg(all(
    test,
    feature = "github",
    feature = "gitlab",
    feature = "gitea",
    feature = "azure-devops",
    feature = "bitbucket",
    feature = "gerrit"
))]
mod tests_wave2;

pub use cache::{
    cache_path, AccountCache, CachedAccount, ACCOUNT_CACHE_FILE, ACCOUNT_TTL, ACCOUNT_TTL_SECS,
};
pub use namespace::{compile as compile_namespace_regex, first_regex_error, NamespaceFilter};

/// Rows kept per kind (requests, issues) when the query names no limit.
pub const DEFAULT_LIMIT: usize = 300;

/// What to fetch for one integration.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountQuery {
    /// Open requests that wait for my review.
    pub review_requests: bool,
    /// Every open request of my namespaces.
    pub open_requests: bool,
    /// Open issues assigned to me.
    pub assigned_issues: bool,
    /// Keep draft requests too.
    pub include_drafts: bool,
    /// Orgs / groups to search for `open_requests`; empty = my own namespace
    /// plus every one I am a member of.
    pub namespaces: Vec<String>,
    /// Read each namespace line as a regular expression over the full
    /// repository path instead of a path prefix.
    #[serde(default)]
    pub namespaces_regex: bool,
    /// Rows kept per kind; 0 = [`DEFAULT_LIMIT`].
    pub limit: usize,
}

impl AccountQuery {
    /// The effective per-kind row limit.
    pub fn row_limit(&self) -> usize {
        if self.limit == 0 {
            DEFAULT_LIMIT
        } else {
            self.limit
        }
    }

    /// The namespaces a host's own queries can be scoped to: the plain lines,
    /// none in regex mode (a pattern is not a group name; the filter on the
    /// results does the work).
    pub fn search_namespaces(&self) -> &[String] {
        if self.namespaces_regex {
            &[]
        } else {
            &self.namespaces
        }
    }

    /// Whether the query asks for nothing at all.
    pub fn is_empty(&self) -> bool {
        !(self.review_requests || self.open_requests || self.assigned_issues)
    }

    /// A short stable fingerprint, so a cache entry is only reused for the
    /// same question (FNV-1a over a canonical spelling; not a security hash).
    pub fn fingerprint(&self) -> String {
        let mut spaces: Vec<String> = self.namespaces.iter().map(|n| n.to_lowercase()).collect();
        spaces.sort();
        let text = format!(
            "v2r{}o{}i{}d{}l{}x{}|{}",
            self.review_requests as u8,
            self.open_requests as u8,
            self.assigned_issues as u8,
            self.include_drafts as u8,
            self.row_limit(),
            self.namespaces_regex as u8,
            spaces.join(",")
        );
        let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
        for b in text.bytes() {
            hash ^= u64::from(b);
            hash = hash.wrapping_mul(0x0100_0000_01b3);
        }
        format!("{hash:016x}")
    }
}

/// A request found account-wide.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountPr {
    /// `owner/name`, `group/sub/proj`: the repository it belongs to.
    pub project: String,
    /// Where to clone that repository from (HTTPS); empty when unknown.
    #[serde(default)]
    pub clone_url: String,
    #[serde(flatten)]
    pub pr: PullRequestSummary,
}

/// An issue found account-wide.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountIssue {
    pub project: String,
    #[serde(default)]
    pub clone_url: String,
    #[serde(flatten)]
    pub issue: IssueSummary,
}

/// The answer to an [`AccountQuery`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountItems {
    #[serde(default)]
    pub prs: Vec<AccountPr>,
    #[serde(default)]
    pub issues: Vec<AccountIssue>,
    /// More rows existed than the limit kept.
    #[serde(default)]
    pub truncated: bool,
    /// One line per sub-query that failed or was cut short.
    #[serde(default)]
    pub notes: Vec<String>,
}

/// Whether [`crate::HostingClient::account_items`] is implemented for `kind`
/// (the other code hosts answer `Unsupported` until their queries land).
pub fn supports(kind: crate::IntegrationKind) -> bool {
    use gitpoulpi_config::ApiFamily;
    match kind.family() {
        ApiFamily::Github
        | ApiFamily::Gitlab
        | ApiFamily::Gitea
        | ApiFamily::AzureDevops
        | ApiFamily::Gerrit => true,
        // Bitbucket Cloud has no account-wide endpoint.
        ApiFamily::Bitbucket => kind == crate::IntegrationKind::BitbucketDataCenter,
        _ => false,
    }
}

/// Why Bitbucket Cloud answers `Unsupported` (shown by `hosting radar`).
pub const NO_BITBUCKET_CLOUD: &str = "Bitbucket Cloud has no account-wide pull request \
     endpoint (its lists are per repository); Bitbucket Data Center does";

/// An error that makes the remaining sub-queries pointless.
pub(crate) fn is_fatal(error: &Error) -> bool {
    matches!(
        error,
        Error::Unauthorized { .. } | Error::RateLimited { .. } | Error::Http(_)
    )
}

/// `https://host/owner/name.git`, or empty when the profile has no web root.
pub(crate) fn clone_url(integration: &crate::Integration, project: &str) -> String {
    crate::spec::web_base(integration)
        .map(|base| format!("{base}/{project}.git"))
        .unwrap_or_default()
}

/// Collects the rows of every sub-query, merges duplicates, applies the draft
/// filter and the limit.
#[derive(Default)]
pub(crate) struct Collector {
    prs: Vec<AccountPr>,
    issues: Vec<AccountIssue>,
    notes: Vec<String>,
    /// A host said more rows exist than it returned.
    cut: bool,
}

impl Collector {
    pub(crate) fn note(&mut self, text: impl Into<String>) {
        self.notes.push(text.into());
    }

    /// Records that the host holds more rows than it returned.
    pub(crate) fn cut(&mut self, note: impl Into<String>) {
        self.cut = true;
        self.notes.push(note.into());
    }

    /// Records a failed sub-query as a note, or returns the error when it is
    /// fatal for the whole call.
    pub(crate) fn failed(&mut self, what: &str, error: Error) -> Result<()> {
        if is_fatal(&error) {
            return Err(error);
        }
        self.notes.push(format!("{what}: {error}"));
        Ok(())
    }

    /// Adds a request; one already seen (same project and number) only gains
    /// the "review requested" flag and the reviewers the newcomer knows.
    pub(crate) fn add_pr(&mut self, row: AccountPr) {
        let seen = self
            .prs
            .iter_mut()
            .find(|p| p.project == row.project && p.pr.number == row.pr.number);
        match seen {
            Some(old) => {
                old.pr.review_requested |= row.pr.review_requested;
                for reviewer in row.pr.reviewers {
                    if !old.pr.reviewers.contains(&reviewer) {
                        old.pr.reviewers.push(reviewer);
                    }
                }
            }
            None => self.prs.push(row),
        }
    }

    pub(crate) fn add_issue(&mut self, row: AccountIssue) {
        let seen = self
            .issues
            .iter()
            .any(|i| i.project == row.project && i.issue.number == row.issue.number);
        if !seen {
            self.issues.push(row);
        }
    }

    pub(crate) fn finish(mut self, query: &AccountQuery) -> AccountItems {
        let limit = query.row_limit();
        // The namespaces apply to every list: the host's review and issue
        // queries are global, so the filter runs here on the project path.
        match NamespaceFilter::new(&query.namespaces, query.namespaces_regex) {
            Ok(filter) if filter.is_empty() => {}
            Ok(filter) => {
                self.prs.retain(|p| filter.matches(&p.project));
                self.issues.retain(|i| filter.matches(&i.project));
            }
            Err(why) => {
                self.prs.clear();
                self.issues.clear();
                self.notes
                    .push(format!("invalid namespace expression {why}"));
            }
        }
        if !query.include_drafts {
            self.prs.retain(|p| p.pr.state != PullRequestState::Draft);
        }
        // ISO-8601 strings sort chronologically; newest first, stable.
        self.prs
            .sort_by(|a, b| b.pr.updated_at.cmp(&a.pr.updated_at));
        self.issues
            .sort_by(|a, b| b.issue.updated_at.cmp(&a.issue.updated_at));
        let mut truncated = self.cut;
        if self.prs.len() > limit {
            let more = self.prs.len() - limit;
            self.prs.truncate(limit);
            self.notes.push(format!(
                "{more} more pull/merge requests are not listed (limit {limit})"
            ));
            truncated = true;
        }
        if self.issues.len() > limit {
            let more = self.issues.len() - limit;
            self.issues.truncate(limit);
            self.notes
                .push(format!("{more} more issues are not listed (limit {limit})"));
            truncated = true;
        }
        AccountItems {
            prs: self.prs,
            issues: self.issues,
            truncated,
            notes: self.notes,
        }
    }
}

/// How one paged account query is bounded.
pub(crate) struct Paging<'a> {
    /// The query parameter that carries the page size (`per_page`, `limit`).
    pub size_param: &'a str,
    pub page_size: usize,
    pub max_pages: usize,
    /// Stop once this many rows are in hand.
    pub max_rows: usize,
}

/// Fetches pages of `url` until a short page, the
/// page cap or the row cap. `rows` pulls one page's rows out of the response.
pub(crate) async fn get_rows<T>(
    transport: &dyn HttpTransport,
    request: &(dyn Fn(String) -> HttpRequest + Sync),
    url: &str,
    paging: &Paging<'_>,
    what: &str,
    rows: &(dyn Fn(&HttpResponse) -> Result<Vec<T>> + Sync),
) -> Result<Vec<T>> {
    let sep = if url.contains('?') { '&' } else { '?' };
    let mut out = Vec::new();
    for page in 1..=paging.max_pages {
        let full = format!(
            "{url}{sep}{}={}&page={page}",
            paging.size_param, paging.page_size
        );
        let response = transport.send(request(full)).await?;
        response.error_for_status(what)?;
        let page_rows = rows(&response)?;
        let count = page_rows.len();
        out.extend(page_rows);
        if count < paging.page_size || out.len() >= paging.max_rows {
            break;
        }
    }
    Ok(out)
}
