//! The per-family descriptor table (plan `more-providers.md`, D1).
//!
//! What used to be a dozen free functions each doing
//! `match family { Github, Gitlab, _ => None }` is one [`FamilySpec`] per
//! [`ApiFamily`]. The old public functions (`pull_request_web_url`,
//! `well_known_kind`, `new_pull_request_url`, ...) are one-line wrappers
//! over [`spec_for`], so no caller changed. A provider slice fills in only
//! its own `src/<provider>/spec.rs`.
//!
//! Pure on purpose: no I/O, and compiled whatever providers are built, so
//! the URL helpers work in a no-network build.

use gitpoulpi_config::{ApiFamily, Integration, IntegrationKind};

use crate::PullRequestForm;

mod github;
mod gitlab;
#[cfg(test)]
mod tests;

/// A git remote, read by a family's own parser.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteRef {
    /// `https://host` (the instance's web root; never the API root).
    pub web_base: String,
    /// `group/sub/proj` (GitLab), `owner/name` (GitHub), ...
    pub project: String,
}

/// How to get the code of a pull request onto a local branch.
#[cfg(feature = "hosting")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CheckoutSource {
    /// The host keeps the head in a special ref of the target repository
    /// (GitHub, GitLab, Bitbucket Data Center): fetch it, track `local`.
    HeadRef { remote_ref: String, local: String },
    /// The head is an ordinary branch, possibly of a fork (Azure DevOps,
    /// Bitbucket Cloud).
    Branch {
        fork_url: Option<String>,
        branch: String,
        local: String,
    },
}

/// Everything family-specific the app needs to know without a network call.
pub struct FamilySpec {
    pub family: ApiFamily,
    /// Toolbar chip and list tag: `GH` `GL` `AZ` `BB` `JI` `TR` `GT` `LN` `YT` `RM` `GR`.
    pub tag: &'static str,
    /// The character before a pull request number: `#` or `!`.
    pub pr_sigil: char,
    /// Remote URL to instance root plus project path.
    pub parse_remote: fn(&str) -> Option<RemoteRef>,
    /// Does `integration` answer questions about `remote`? Default: same
    /// hostname. Azure DevOps compares the organization root as well.
    pub owns_remote: fn(&Integration, &RemoteRef) -> bool,
    pub pr_web_url: fn(web_root: &str, project: &str, number: u64) -> Option<String>,
    pub pr_edit_url: fn(pr_web_url: &str) -> String,
    pub new_pr_url:
        fn(web_root: &str, project: &str, source: &str, target: Option<&str>) -> Option<String>,
    pub new_pr_form_url: fn(web_root: &str, form: &PullRequestForm<'_>) -> Option<String>,
    pub issue_web_url: fn(web_root: &str, project: &str, reference: &str) -> Option<String>,
    pub new_issue_url: fn(
        web_root: &str,
        project: &str,
        title: &str,
        body: &str,
        labels: &[String],
    ) -> Option<String>,
    pub branch_url: fn(web_root: &str, project: &str, branch: &str) -> Option<String>,
    pub commit_url: fn(web_root: &str, project: &str, sha: &str) -> Option<String>,
    #[cfg(feature = "hosting")]
    pub checkout: fn(&crate::model::PullRequestSummary) -> CheckoutSource,
    /// The page that mints a token with the right scopes.
    pub token_page: fn(&Integration) -> Option<String>,
    /// The kind a lowercased web host identifies with no configuration.
    pub well_known: fn(host: &str) -> Option<IntegrationKind>,
}

impl FamilySpec {
    /// A spec with nothing filled in: every URL is `None`, remotes parse
    /// and match by hostname, a pull request is fetched by branch.
    pub const fn none(family: ApiFamily, tag: &'static str, pr_sigil: char) -> Self {
        Self {
            family,
            tag,
            pr_sigil,
            parse_remote: default_parse_remote,
            owns_remote: default_owns_remote,
            pr_web_url: |_, _, _| None,
            pr_edit_url: |url| url.to_string(),
            new_pr_url: |_, _, _, _| None,
            new_pr_form_url: |_, _| None,
            issue_web_url: |_, _, _| None,
            new_issue_url: |_, _, _, _, _| None,
            branch_url: |_, _, _| None,
            commit_url: |_, _, _| None,
            #[cfg(feature = "hosting")]
            checkout: default_checkout,
            token_page: token_page_from_help,
            well_known: |_| None,
        }
    }
}

/// The spec of `family`. Families whose provider is not built get the
/// inert one, exactly as a family with no client yet.
pub fn spec_for(family: ApiFamily) -> &'static FamilySpec {
    match family {
        ApiFamily::Github => &github::SPEC,
        ApiFamily::Gitlab => &gitlab::SPEC,
        #[cfg(feature = "azure-devops")]
        ApiFamily::AzureDevops => &crate::azure_devops::spec::SPEC,
        #[cfg(feature = "bitbucket")]
        ApiFamily::Bitbucket => &crate::bitbucket::spec::SPEC,
        #[cfg(feature = "jira")]
        ApiFamily::Jira => &crate::jira::spec::SPEC,
        #[cfg(feature = "trello")]
        ApiFamily::Trello => &crate::trello::spec::SPEC,
        #[cfg(feature = "gitea")]
        ApiFamily::Gitea => &crate::gitea::spec::SPEC,
        #[cfg(feature = "linear")]
        ApiFamily::Linear => &crate::linear::spec::SPEC,
        #[cfg(feature = "youtrack")]
        ApiFamily::YouTrack => &crate::youtrack::spec::SPEC,
        #[cfg(feature = "redmine")]
        ApiFamily::Redmine => &crate::redmine::spec::SPEC,
        #[cfg(feature = "gerrit")]
        ApiFamily::Gerrit => &crate::gerrit::spec::SPEC,
        #[cfg(feature = "local-issues")]
        ApiFamily::LocalMarkdown => &crate::local_markdown::spec::SPEC,
        #[allow(unreachable_patterns)]
        other => inert(other),
    }
}

/// The spec of a family whose provider module is compiled out.
#[allow(dead_code)]
fn inert(family: ApiFamily) -> &'static FamilySpec {
    static AZ: FamilySpec = FamilySpec::none(ApiFamily::AzureDevops, "AZ", '!');
    static BB: FamilySpec = FamilySpec::none(ApiFamily::Bitbucket, "BB", '#');
    static JI: FamilySpec = FamilySpec::none(ApiFamily::Jira, "JI", '#');
    static TR: FamilySpec = FamilySpec::none(ApiFamily::Trello, "TR", '#');
    static GT: FamilySpec = FamilySpec::none(ApiFamily::Gitea, "GT", '#');
    static LN: FamilySpec = FamilySpec::none(ApiFamily::Linear, "LN", '#');
    static YT: FamilySpec = FamilySpec::none(ApiFamily::YouTrack, "YT", '#');
    static RM: FamilySpec = FamilySpec::none(ApiFamily::Redmine, "RM", '#');
    static GR: FamilySpec = FamilySpec::none(ApiFamily::Gerrit, "GR", '#');
    static MD: FamilySpec = FamilySpec::none(ApiFamily::LocalMarkdown, "MD", '#');
    match family {
        ApiFamily::LocalMarkdown => &MD,
        ApiFamily::AzureDevops => &AZ,
        ApiFamily::Jira => &JI,
        ApiFamily::Trello => &TR,
        ApiFamily::Gitea => &GT,
        ApiFamily::Linear => &LN,
        ApiFamily::YouTrack => &YT,
        ApiFamily::Redmine => &RM,
        ApiFamily::Gerrit => &GR,
        _ => &BB,
    }
}

/// `remote_web_base`, as a [`RemoteRef`].
pub fn default_parse_remote(remote_url: &str) -> Option<RemoteRef> {
    let (web_base, project) = crate::remote_web_base(remote_url)?;
    Some(RemoteRef { web_base, project })
}

/// Same hostname (port, `www.` and an `api.` prefix on a public host
/// ignored).
pub fn default_owns_remote(integration: &Integration, remote: &RemoteRef) -> bool {
    integration_hostname(integration) == hostname(&remote.web_base).trim_start_matches("www.")
}

#[cfg(feature = "hosting")]
fn default_checkout(pr: &crate::model::PullRequestSummary) -> CheckoutSource {
    CheckoutSource::Branch {
        fork_url: None,
        branch: pr.source_branch.clone(),
        local: format!("pr/{}", pr.number),
    }
}

/// `host` of a web base (`https://host`), lowercased, port removed.
pub(crate) fn hostname(web_base: &str) -> String {
    let rest = web_base
        .split_once("://")
        .map(|(_, r)| r)
        .unwrap_or(web_base);
    let authority = rest.split('/').next().unwrap_or("");
    let authority = authority
        .rsplit_once('@')
        .map(|(_, h)| h)
        .unwrap_or(authority);
    let host = match authority.rsplit_once(':') {
        // Not an IPv6 literal's inner colon.
        Some((h, port)) if port.chars().all(|c| c.is_ascii_digit()) => h,
        _ => authority,
    };
    host.trim().to_ascii_lowercase()
}

pub(crate) fn integration_hostname(integration: &Integration) -> String {
    let url = if integration.url.trim().is_empty() {
        integration.kind.public_url().unwrap_or("")
    } else {
        integration.url.as_str()
    };
    // `api.github.com` style URLs identify the same instance as the web host.
    let host = hostname(url);
    host.strip_prefix("api.")
        .filter(|_| integration.kind.public_url().is_some())
        .map(str::to_string)
        .unwrap_or(host)
        .trim_start_matches("www.")
        .to_string()
}

/// The instance's web root: the configured URL, or the kind's fixed public
/// host when none is set.
pub fn web_base(integration: &Integration) -> Option<String> {
    let url = integration.url.trim().trim_end_matches('/');
    if !url.is_empty() {
        return Some(url.to_string());
    }
    integration.kind.public_url().map(str::to_string)
}

/// The token page, as the profile's kind defines it (S1's `token_link_url`).
pub(crate) fn token_page_from_help(integration: &Integration) -> Option<String> {
    let base = web_base(integration)?;
    integration.token_link_url(&base)
}
