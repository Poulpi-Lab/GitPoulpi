//! Hosting and issue-tracker API clients: GitHub, GitLab, Gitea/Forgejo, Azure
//! DevOps, Bitbucket, Gerrit, Jira, Trello, Linear, YouTrack, Redmine and local
//! Markdown issues, including self-managed instances.
//!
//! Two jobs:
//!
//! 1. **List the projects a token can see, and clone straight from that
//!    list.** [`HostingClient::list_projects`] returns a normalized
//!    [`HostedProject`] whichever host produced it, and
//!    [`HostedProject::clone_url`] hands the chosen row's URL to
//!    `GitBackend::clone`.
//! 2. **Generate an SSH key and upload it.** [`ssh_key`] makes the pair
//!    in process (no `ssh-keygen` dependency, no prompt), and
//!    [`HostingClient::upload_ssh_key`] registers the public half.
//!
//! Every network call goes through [`http::HttpTransport`], so the crate
//! is testable with no network ([`http::RecordedTransport`]); only
//! `ReqwestTransport`, behind the default `network` feature, opens a socket.
//!
//! An integration is a base URL plus a token reference, and the API root
//! is *derived* from the base URL (see [`Integration::api_base`]) rather
//! than configured separately, because two keys that must agree can
//! disagree.

#[cfg(feature = "hosting")]
pub mod account;
#[cfg(feature = "hosting")]
mod assignment_guard;
pub mod auth;
#[cfg(feature = "azure-devops")]
pub mod azure_devops;
#[cfg(feature = "bitbucket")]
pub mod bitbucket;
#[cfg(feature = "hosting")]
pub mod board;
pub mod cache_search;
#[cfg(feature = "hosting")]
pub mod client;
pub mod conditional;
#[cfg(feature = "hosting")]
pub mod conversation;
#[cfg(feature = "hosting")]
pub mod flow_engine;
#[cfg(feature = "gerrit")]
pub mod gerrit;
#[cfg(feature = "gitea")]
pub mod gitea;
#[cfg(feature = "github")]
pub mod github;
#[cfg(feature = "github")]
mod github_flow;
#[cfg(feature = "github")]
mod github_meta;
#[cfg(feature = "gitlab")]
pub mod gitlab;
#[cfg(feature = "gitlab")]
mod gitlab_flow;
#[cfg(feature = "gitlab")]
mod gitlab_meta;
pub mod http;
mod integration;
#[cfg(feature = "hosting")]
pub mod issue_flow;
#[cfg(feature = "jira")]
pub mod jira;
#[cfg(feature = "linear")]
pub mod linear;
#[cfg(feature = "hosting")]
pub mod list_cache;
#[cfg(feature = "local-issues")]
pub mod local_markdown;
#[cfg(feature = "hosting")]
pub mod metadata;
#[cfg(feature = "hosting")]
pub mod model;
#[cfg(feature = "hosting")]
mod project;
#[cfg(feature = "hosting")]
pub mod protection;
#[cfg(feature = "redmine")]
pub mod redmine;
pub mod refs;
#[cfg(feature = "hosting")]
pub mod resolve;
#[cfg(feature = "hosting")]
pub mod review;
pub mod scope;
#[cfg(any(feature = "github", feature = "gitlab"))]
mod scope_handle;
#[cfg(all(test, any(feature = "github", feature = "gitlab")))]
mod scope_fake;
#[cfg(feature = "github")]
mod scoped_transport;
pub mod spec;
pub mod ssh_key;
#[cfg(feature = "trello")]
pub mod trello;
pub mod trust;
pub mod updates;
mod web_url;
#[cfg(feature = "youtrack")]
pub mod youtrack;

#[cfg(feature = "hosting")]
pub use account::{
    AccountCache, AccountIssue, AccountItems, AccountPr, AccountQuery, CachedAccount, ACCOUNT_TTL,
    ACCOUNT_TTL_SECS,
};
pub use auth::{auth_headers, git_username};
#[cfg(feature = "hosting")]
pub use client::{
    client_for, issue_web_url, pull_request_edit_url, pull_request_head_ref, pull_request_web_url,
    token_create_url, web_base as integration_web_base, CreatedIssue, CreatedPullRequest,
    HostingClient, NewIssue, NewPullRequest, UnsupportedClient,
};
#[cfg(feature = "network")]
pub use conditional::shared_network_transport;
pub use conditional::{shared_rate_note, ConditionalTransport};
#[cfg(feature = "github")]
pub use github::GitHubClient;
#[cfg(feature = "gitlab")]
pub use gitlab::GitLabClient;
#[cfg(feature = "network")]
pub use http::ReqwestTransport;
pub use http::{HttpRequest, HttpResponse, HttpTransport, Method, RecordedTransport};
pub use integration::{
    resolve_local_folder, token_env_var, DefaultTokenSource, Integration, IntegrationKind,
    StaticToken, TokenSource,
};
#[cfg(feature = "hosting")]
pub use issue_flow::{CreatedBranch, IssueLinks, NewIssueBranch};
#[cfg(feature = "hosting")]
pub use metadata::{
    list_delta, HostedLabel, HostedMilestone, IssueEdit, ItemMeta, MetaKind, PrEdit,
};
#[cfg(feature = "hosting")]
pub use model::{
    best_pull_request, excess_scopes, is_dangerous_scope, ChecksState, Choice, HostedUser,
    IssueCreateOptions, IssueFilter, IssueState, IssueSummary, PrFilter, PullRequestState,
    PullRequestSummary, StateFilter, TokenCheck, TokenFlavor,
};
#[cfg(feature = "hosting")]
pub use project::{HostedProject, HostedSshKey};
#[cfg(feature = "hosting")]
pub use resolve::{resolve_for_repo, suggest_for_repo, Resolution, ResolveRemote, Suggestion};
#[cfg(feature = "hosting")]
pub use review::{
    review_base_ref, review_base_refspec, review_compare_ref, review_compare_refspec,
    review_head_ref, review_head_refspec, review_head_refspec_at, DiffSide, FileStatus, MergeState,
    PatchSet, PendingComment, PullRequestDetail, ReviewCapabilities, ReviewComment, ReviewEvent,
    ReviewFile, ReviewOutcome, ReviewState, ReviewSubmission, ReviewThread, ReviewVote,
    SubmittedReview, VoteLabel, VoteValue,
};
pub use scope::{install as install_repo_scopes, RepoScopes};
#[cfg(feature = "hosting")]
pub use spec::CheckoutSource;
pub use spec::{spec_for, FamilySpec, RemoteRef};
pub use trust::{Account, TrustGate, TrustProblem, TrustSubject};
pub use web_url::{
    new_issue_form_url, new_pull_request_form_url, percent_encode_component, PullRequestForm,
    MAX_URL_LEN, TRUNCATION_NOTE,
};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("unrecognized hosting provider for url: {0}")]
    UnrecognizedProvider(String),

    #[error("`{0}` is not a usable base url for a hosting instance")]
    BadUrl(String),

    #[error("no integration configured with id `{0}`")]
    UnknownIntegration(String),

    /// Deliberately verbose: every way of supplying the token is named,
    /// because "no token" with no further detail is the error people
    /// file issues about.
    #[error(
        "no token for integration `{id}`: nothing in the credential helper for {host}, \
         and {env_var} is unset (config points at token_ref `{token_ref}`)"
    )]
    NoToken {
        id: String,
        token_ref: String,
        env_var: String,
        host: String,
    },

    #[error("{what} failed: the host rejected the token ({detail})")]
    Unauthorized { what: String, detail: String },

    #[error("{what} failed: the token lacks the required scope ({detail})")]
    Forbidden { what: String, detail: String },

    #[error("{what} failed with HTTP {status}: {detail}")]
    Api {
        what: String,
        status: u16,
        detail: String,
    },

    /// The host said to slow down (or we already know it is out of calls).
    #[error("{host}: rate limit reached, try again in {}", crate::conditional::wait_text(*retry_in_secs))]
    RateLimited { host: String, retry_in_secs: u64 },

    #[error("http error: {0}")]
    Http(String),

    #[error("the host's response could not be decoded: {0}")]
    Decode(String),

    /// The kind is configurable but no client exists for it yet.
    #[error("{0} is not supported yet: no client exists for it")]
    NotImplemented(String),

    /// The client exists but does not offer this operation.
    #[error("`{0}` is not supported by this client")]
    Unsupported(String),

    #[error("ssh key error: {0}")]
    Ssh(String),

    /// The local Markdown issue source: a folder or note problem.
    #[error("{0}")]
    Local(String),

    /// The integration trust check refused: nothing was sent.
    #[error("{0}")]
    Trust(trust::TrustProblem),

    /// `trust.toml` could not be read or written. Refused rather than
    /// treated as empty: an empty store would re-trust whatever the config
    /// says now.
    #[error("the integration trust store is unusable: {0}")]
    TrustStore(String),

    /// The repository blocks assignment: nothing was sent.
    #[error("{}", scope::ASSIGNMENT_BLOCKED)]
    AssignmentBlocked,

    /// The token is a GitHub App installation token: it has no user behind
    /// it, so `GET /user` (who am I) does not apply. Not a failure: the
    /// actor is the App.
    #[error("the token belongs to a GitHub App (a bot), not to a person")]
    BotToken,
}

/// Which configured instance a remote URL belongs to, so a repository's
/// `origin` can be linked back to the integration that can answer
/// questions about it.
///
/// Compares hosts, not whole URLs: `git@gitlab.example.com:g/p.git` and
/// `https://gitlab.example.com/g/p.git` are the same instance, and an
/// integration configured as `https://gitlab.example.com` has to match
/// both.
pub fn match_integration<'a>(
    remote_url: &str,
    integrations: &'a [Integration],
) -> Option<&'a Integration> {
    let host = host_of_remote(remote_url)?;
    // A profile's host carries its port (`nas:3000`); the remote's does not
    // (ssh and http differ in them), so compare the names only.
    integrations.iter().find(|i| {
        i.host().is_some_and(|h| {
            let name = h
                .rsplit_once(':')
                .filter(|(_, p)| p.bytes().all(|b| b.is_ascii_digit()))
                .map_or(h.as_str(), |(n, _)| n);
            name.eq_ignore_ascii_case(&host)
        })
    })
}

/// The host part of any remote spelling git accepts.
///
/// Three shapes in the wild: `https://host/path`, `ssh://git@host/path`
/// and the scp-like `git@host:path`, which is not a URL at all and is
/// the one a naive `Url::parse` drops on the floor.
fn host_of_remote(remote_url: &str) -> Option<String> {
    let remote = remote_url.trim();
    if remote.is_empty() {
        return None;
    }
    let after_scheme = remote.split_once("://").map(|(_, r)| r).unwrap_or(remote);
    // Strip any `user@` / `user:password@` prefix.
    let after_user = after_scheme
        .rsplit_once('@')
        .map(|(_, r)| r)
        .unwrap_or(after_scheme);
    // Whichever of `/` (url) or `:` (scp-like, and also a port) comes
    // first ends the host.
    let end = after_user.find(['/', ':']).unwrap_or(after_user.len());
    let host = after_user[..end].trim();
    (!host.is_empty()).then(|| host.to_ascii_lowercase())
}

/// Splits a git remote URL into the host's *web* base (`https://host`,
/// never the API root) and the project path (`group/sub/proj`), for
/// building a pull/merge-request URL by hand.
///
/// Handles every spelling `host_of_remote` does - scp-like
/// (`git@host:group/proj.git`), `ssh://` (a port, if any, is dropped and
/// the scheme is `https`: the ssh daemon says nothing about the web UI)
/// and `http(s)://` (scheme and port are the web UI's own, so a plain-http
/// server in a container stays `http://host:3000`) - and always strips a
/// trailing `.git`.
pub fn remote_web_base(remote_url: &str) -> Option<(String, String)> {
    let remote = remote_url.trim();
    if remote.is_empty() {
        return None;
    }
    let host = host_of_remote(remote)?;
    // Repeats `host_of_remote`'s scheme/user/host split so that everything
    // after the host can be taken as the path; keep the two in step.
    let has_scheme = remote.contains("://");
    let after_scheme = remote.split_once("://").map(|(_, r)| r).unwrap_or(remote);
    let after_user = after_scheme
        .rsplit_once('@')
        .map(|(_, r)| r)
        .unwrap_or(after_scheme);
    // The host ends at the first `/` or `:` (a port, or the scp-like
    // separator); whichever it is, the path starts just after it.
    let end = after_user.find(['/', ':']).unwrap_or(after_user.len());
    let mut rest = &after_user[end..];
    // A `:port` before the path (`ssh://` with an explicit port -- the
    // colon there is a port, not the scp-like separator, which is only
    // scheme-less `user@host:path`): drop it, keep the `/path`.
    let mut port = None;
    if has_scheme {
        if let Some(stripped) = rest.strip_prefix(':') {
            let path_start = stripped.find('/').unwrap_or(stripped.len());
            port = Some(&stripped[..path_start]);
            rest = &stripped[path_start..];
        }
    }
    let path = rest.trim_start_matches(['/', ':']).trim_end_matches('/');
    let path = path.strip_suffix(".git").unwrap_or(path);
    if path.is_empty() {
        return None;
    }
    // An http(s) remote is the web UI's own address, scheme and port
    // included (a self-hosted server is often plain http on a port);
    // ssh and scp-like remotes say nothing about it, so https it is.
    let scheme = remote
        .split_once("://")
        .map(|(s, _)| s.to_ascii_lowercase())
        .unwrap_or_default();
    let base = match (scheme.as_str(), port) {
        ("http" | "https", Some(port)) if port.bytes().all(|b| b.is_ascii_digit()) => {
            format!("{scheme}://{host}:{port}")
        }
        ("http" | "https", _) => format!("{scheme}://{host}"),
        _ => format!("https://{host}"),
    };
    Some((base, path.to_string()))
}

/// `web_base` (from [`remote_web_base`]), moved to the address the
/// matched profile says the server lives at when it is the same host:
/// an ssh remote of a server configured as `http://host:3000` opens its
/// web pages there, not on `https://host`.
pub fn web_base_for(integration: Option<&Integration>, web_base: String) -> String {
    let Some(integration) = integration else {
        return web_base;
    };
    let Some((scheme, rest)) = integration.url.trim().split_once("://") else {
        return web_base;
    };
    let authority = rest.split('/').next().unwrap_or("");
    let bare = |x: &str| {
        let x = x.split_once("://").map_or(x, |(_, r)| r);
        let x = x.split('/').next().unwrap_or("");
        x.rsplit_once(':')
            .filter(|(_, p)| p.bytes().all(|b| b.is_ascii_digit()))
            .map_or(x, |(h, _)| h)
            .to_ascii_lowercase()
    };
    if authority.is_empty() || bare(authority) != bare(&web_base) {
        return web_base;
    }
    format!("{}://{authority}", scheme.to_ascii_lowercase())
}

/// The two hosts recognized with no configuration at all: `github.com` /
/// `gitlab.com` (or a `*.github.com` / `*.gitlab.com` subdomain), so a
/// remote with no matching `[[integrations]]` entry still gets a usable
/// compare/new-request URL.
pub fn well_known_kind(web_base: &str) -> Option<IntegrationKind> {
    let host = web_base
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .to_ascii_lowercase();
    [
        gitpoulpi_config::ApiFamily::Github,
        gitpoulpi_config::ApiFamily::Gitlab,
        gitpoulpi_config::ApiFamily::AzureDevops,
        gitpoulpi_config::ApiFamily::Bitbucket,
        gitpoulpi_config::ApiFamily::Gitea,
    ]
    .into_iter()
    .find_map(|family| (spec::spec_for(family).well_known)(&host))
}

/// Percent-encodes the characters that would break a branch name out of a
/// query-string value or URL path segment: `#`, `%`, `&` and space.
/// Deliberately narrow: a legal git branch name cannot contain most other
/// URL-unsafe characters.
fn percent_encode_branch(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    for b in name.bytes() {
        match b {
            b'%' => out.push_str("%25"),
            b'#' => out.push_str("%23"),
            b'&' => out.push_str("%26"),
            b' ' => out.push_str("%20"),
            _ => out.push(b as char),
        }
    }
    out
}

/// Builds the "open a compare/new pull-request page" URL by hand: the
/// no-token fallback, and what a token-holding flow opens on success too
/// (see [`client::HostingClient::create_pull_request`]).
///
/// `web_base` and `project` are `remote_web_base`'s output; `source` is the
/// branch to merge from, `target` the branch to merge into (the host's
/// default when `None`).
pub fn new_pull_request_url(
    kind: IntegrationKind,
    web_base: &str,
    project: &str,
    source: &str,
    target: Option<&str>,
) -> String {
    let spec = spec::spec_for(kind.family());
    // A family with no URL of its own keeps the GitLab shape it always had.
    (spec.new_pr_url)(web_base, project, source, target)
        .or_else(|| {
            (spec::spec_for(gitpoulpi_config::ApiFamily::Gitlab).new_pr_url)(
                web_base, project, source, target,
            )
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn integrations() -> Vec<Integration> {
        vec![
            Integration::new("gh", IntegrationKind::Github, "https://github.com"),
            Integration::new(
                "corp",
                IntegrationKind::Gitlab,
                "https://gitlab.example.com",
            ),
        ]
    }

    /// Every remote spelling of one instance must find the same
    /// integration - especially the scp-like form, which is what an
    /// SSH clone leaves in `.git/config`.
    #[test]
    fn every_remote_spelling_of_one_host_matches_its_integration() {
        let integrations = integrations();
        for remote in [
            "https://gitlab.example.com/group/thing.git",
            "ssh://git@gitlab.example.com/group/thing.git",
            "git@gitlab.example.com:group/thing.git",
            "ssh://git@gitlab.example.com:2222/group/thing.git",
            "HTTPS://GitLab.Example.COM/group/thing.git",
        ] {
            let found = match_integration(remote, &integrations);
            assert_eq!(found.map(|i| i.id.as_str()), Some("corp"), "{remote}");
        }
    }

    #[test]
    fn an_unconfigured_host_matches_nothing() {
        assert!(match_integration("https://bitbucket.org/a/b.git", &integrations()).is_none());
        assert!(match_integration("", &integrations()).is_none());
        assert!(match_integration("   ", &integrations()).is_none());
    }

    #[test]
    fn github_and_gitlab_do_not_match_each_other() {
        let integrations = integrations();
        let found = match_integration("git@github.com:me/thing.git", &integrations);
        assert_eq!(found.map(|i| i.id.as_str()), Some("gh"));
    }

    /// Every spelling a git remote can be typed in must yield the same
    /// `https://host` web base and project path - this is what the
    /// no-token compare/merge-request URL is built from, so getting it
    /// wrong silently sends the user to the wrong page.
    #[test]
    fn web_base_from_every_remote_spelling() {
        for remote in [
            "https://gitlab.example.com/group/sub/proj.git",
            "https://gitlab.example.com/group/sub/proj",
            "ssh://git@gitlab.example.com/group/sub/proj.git",
            "ssh://git@gitlab.example.com:2222/group/sub/proj.git",
            "git@gitlab.example.com:group/sub/proj.git",
        ] {
            assert_eq!(
                remote_web_base(remote),
                Some((
                    "https://gitlab.example.com".to_string(),
                    "group/sub/proj".to_string()
                )),
                "{remote}"
            );
        }
        assert_eq!(
            remote_web_base("https://github.com/me/thing.git"),
            Some(("https://github.com".to_string(), "me/thing".to_string()))
        );
        assert_eq!(remote_web_base(""), None);
        assert_eq!(remote_web_base("not a url"), None);
    }

    /// A self-hosted server is often plain http on a port (a Docker
    /// container): an http(s) remote's own scheme and port are the web
    /// UI's, so the compare / merge-request URL must not become https.
    #[test]
    fn web_base_keeps_the_scheme_and_port_of_an_http_remote() {
        for (remote, base) in [
            ("http://localhost:3000/o/r.git", "http://localhost:3000"),
            ("http://gitea:3000/o/r", "http://gitea:3000"),
            (
                "http://me:tok@192.168.1.10:8080/o/r.git",
                "http://192.168.1.10:8080",
            ),
            ("https://host.corp:8443/o/r.git", "https://host.corp:8443"),
            ("HTTP://Nas/o/r.git", "http://nas"),
            // ssh stays https on the default port: the daemon's port is not the web UI's.
            ("ssh://git@nas:2222/o/r.git", "https://nas"),
            ("git@nas:o/r.git", "https://nas"),
        ] {
            assert_eq!(
                remote_web_base(remote),
                Some((base.to_string(), "o/r".to_string())),
                "{remote}"
            );
        }
    }

    /// An ssh remote of a profile configured as `http://host:port` still
    /// opens the web UI where the profile says it is.
    #[test]
    fn a_profile_url_sets_the_web_base_of_its_own_host() {
        let gitea = Integration::new("g", IntegrationKind::Gitea, "http://nas:3000/");
        assert_eq!(
            web_base_for(Some(&gitea), "https://nas".to_string()),
            "http://nas:3000"
        );
        assert_eq!(
            web_base_for(Some(&gitea), "https://elsewhere".to_string()),
            "https://elsewhere"
        );
        assert_eq!(web_base_for(None, "https://nas".to_string()), "https://nas");
    }

    /// The profile matches an http remote with a port, and an ssh remote
    /// of the same machine.
    #[test]
    fn an_http_profile_matches_http_and_ssh_remotes() {
        let ints = vec![Integration::new(
            "g",
            IntegrationKind::Gitea,
            "http://nas:3000",
        )];
        for remote in [
            "http://nas:3000/o/r.git",
            "ssh://git@nas:2222/o/r.git",
            "git@nas:o/r.git",
        ] {
            assert!(match_integration(remote, &ints).is_some(), "{remote}");
        }
    }

    #[test]
    fn well_known_kind_recognises_github_and_gitlab_but_nothing_else() {
        assert_eq!(
            well_known_kind("https://github.com"),
            Some(IntegrationKind::Github)
        );
        assert_eq!(
            well_known_kind("https://gitlab.com"),
            Some(IntegrationKind::Gitlab)
        );
        assert_eq!(well_known_kind("https://gitlab.example.com"), None);
        #[cfg(feature = "bitbucket")]
        assert_eq!(
            well_known_kind("https://bitbucket.org"),
            Some(IntegrationKind::Bitbucket)
        );
        assert_eq!(well_known_kind("https://bitbucket.example.com"), None);
    }

    /// `#`, `%`, `&` and space in a branch name must not break out of
    /// the query string or the URL path segment, and `/` in the project
    /// path must survive untouched - it is a path separator, not part
    /// of a branch name.
    #[test]
    fn new_pull_request_url_encodes_slashes_and_ampersands() {
        let gh = new_pull_request_url(
            IntegrationKind::Github,
            "https://github.com",
            "me/thing",
            "feature & fix #1",
            Some("main"),
        );
        assert_eq!(
            gh,
            "https://github.com/me/thing/compare/main...feature%20%26%20fix%20%231?expand=1"
        );
        assert_eq!(
            new_pull_request_url(
                IntegrationKind::Github,
                "https://github.com",
                "me/thing",
                "feature",
                None
            ),
            "https://github.com/me/thing/compare/feature?expand=1"
        );

        let gl = new_pull_request_url(
            IntegrationKind::Gitlab,
            "https://gitlab.example.com",
            "group/sub/proj",
            "feature/login",
            Some("main"),
        );
        assert_eq!(
            gl,
            "https://gitlab.example.com/group/sub/proj/-/merge_requests/new?merge_request%5Bsource_branch%5D=feature/login&merge_request%5Btarget_branch%5D=main"
        );
        assert_eq!(
            new_pull_request_url(
                IntegrationKind::Gitlab,
                "https://gitlab.example.com",
                "group/proj",
                "feature",
                None
            ),
            "https://gitlab.example.com/group/proj/-/merge_requests/new?merge_request%5Bsource_branch%5D=feature"
        );
    }
}
