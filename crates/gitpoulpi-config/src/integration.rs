//! `[[integrations]]`: one configured hosting / issue-tracker profile.
//!
//! **This shape is a contract**, consumed by `gitpoulpi-hosting` and the
//! frontends. `token_ref` is a *key into `gitpoulpi-secrets`*, never the
//! token: a personal access token in `config.toml` would be a plaintext
//! credential in a file people paste into bug reports.
//!
//! ```toml
//! [[integrations]]
//! id = "corp"
//! kind = "gitlab_self_managed"
//! url = "https://gitlab.example.com"
//! token_ref = "integration:corp"
//! ```

use serde::{Deserialize, Serialize};

/// Which product a profile talks to. Spellings are `snake_case` and pinned
/// by tests: they are written to disk.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IntegrationKind {
    #[default]
    Github,
    GithubEnterprise,
    Gitlab,
    GitlabSelfManaged,
    Bitbucket,
    BitbucketDataCenter,
    AzureDevops,
    AzureDevopsServer,
    JiraCloud,
    JiraDataCenter,
    Trello,
    Gitea,
    Forgejo,
    Linear,
    /// YouTrack Cloud and Server (same REST API): `url` is the instance.
    Youtrack,
    /// Redmine (issues only): `url` is the instance, the token is the REST API key.
    Redmine,
    /// Gerrit Code Review (review only): HTTP basic, the user name in
    /// [`Integration::account`] and the HTTP password as the token.
    Gerrit,
    /// A folder of Markdown notes (an Obsidian vault subfolder, `docs/issues`):
    /// no token, no network. `url` holds the folder.
    LocalMarkdown,
}

/// The wire protocol a kind speaks; what a client is chosen by.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ApiFamily {
    Github,
    Gitlab,
    Bitbucket,
    AzureDevops,
    Jira,
    Trello,
    Gitea,
    Linear,
    YouTrack,
    Redmine,
    Gerrit,
    LocalMarkdown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntegrationCategory {
    Hosting,
    IssueTracker,
}

/// The non-secret identity a kind needs besides its token
/// ([`Integration::account`]); decides the form's label and validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccountField {
    /// The Atlassian account email (Jira Cloud, Bitbucket Cloud).
    Email,
    /// The Trello API key.
    ApiKey,
    /// The user name of an HTTP-basic profile (Gerrit).
    Username,
}

/// What a kind can do, and whether GitPoulpi does it yet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Capabilities {
    pub repos: bool,
    pub pull_requests: bool,
    pub issues: bool,
    pub ssh_keys: bool,
    /// Always false: personal-token integrations only, no OAuth flow. The
    /// field stays for config compatibility.
    pub oauth: bool,
    /// The user must type a host (self-managed / data-center products).
    pub needs_host: bool,
    /// A client exists: the kind is in [`IntegrationKind::IMPLEMENTED`].
    pub implemented: bool,
    /// The extra non-secret identity the profile asks for, if any.
    pub account: Option<AccountField>,
    /// The profile is scoped to an organization (Azure DevOps).
    pub org: bool,
    /// `url` is a folder on this machine: no token, no host, no network
    /// (the local Markdown issue source).
    pub local_folder: bool,
}

/// Where the user mints a token.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenLink {
    /// A path appended to the instance's web root.
    Path(&'static str),
    /// A fixed page on another host (Atlassian's API-token page).
    Absolute(&'static str),
    /// Trello's authorize page: built from the API key, so there is no link
    /// until [`Integration::account`] is set.
    Trello,
}

/// How to help the user mint a token: the scopes to tick and the page that
/// pre-fills them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TokenHelp {
    pub scopes: &'static [&'static str],
    pub link: TokenLink,
}

impl IntegrationKind {
    pub const ALL: &'static [IntegrationKind] = &[
        IntegrationKind::Github,
        IntegrationKind::GithubEnterprise,
        IntegrationKind::Gitlab,
        IntegrationKind::GitlabSelfManaged,
        IntegrationKind::Bitbucket,
        IntegrationKind::BitbucketDataCenter,
        IntegrationKind::AzureDevops,
        IntegrationKind::AzureDevopsServer,
        IntegrationKind::Gitea,
        IntegrationKind::Forgejo,
        IntegrationKind::Gerrit,
        IntegrationKind::JiraCloud,
        IntegrationKind::JiraDataCenter,
        IntegrationKind::Trello,
        IntegrationKind::Linear,
        IntegrationKind::Youtrack,
        IntegrationKind::Redmine,
        IntegrationKind::LocalMarkdown,
    ];

    /// The kinds a client exists for (what a picker should offer today).
    pub const IMPLEMENTED: &'static [IntegrationKind] = &[
        IntegrationKind::Github,
        IntegrationKind::GithubEnterprise,
        IntegrationKind::Gitlab,
        IntegrationKind::GitlabSelfManaged,
        IntegrationKind::Bitbucket,
        IntegrationKind::BitbucketDataCenter,
        IntegrationKind::AzureDevops,
        IntegrationKind::AzureDevopsServer,
        IntegrationKind::Gitea,
        IntegrationKind::Forgejo,
        IntegrationKind::Gerrit,
        IntegrationKind::JiraCloud,
        IntegrationKind::JiraDataCenter,
        IntegrationKind::Trello,
        IntegrationKind::Linear,
        IntegrationKind::Youtrack,
        IntegrationKind::Redmine,
        IntegrationKind::LocalMarkdown,
    ];

    /// The on-disk spelling.
    pub fn as_str(self) -> &'static str {
        match self {
            IntegrationKind::Github => "github",
            IntegrationKind::GithubEnterprise => "github_enterprise",
            IntegrationKind::Gitlab => "gitlab",
            IntegrationKind::GitlabSelfManaged => "gitlab_self_managed",
            IntegrationKind::Bitbucket => "bitbucket",
            IntegrationKind::BitbucketDataCenter => "bitbucket_data_center",
            IntegrationKind::AzureDevops => "azure_devops",
            IntegrationKind::AzureDevopsServer => "azure_devops_server",
            IntegrationKind::JiraCloud => "jira_cloud",
            IntegrationKind::JiraDataCenter => "jira_data_center",
            IntegrationKind::Trello => "trello",
            IntegrationKind::Gitea => "gitea",
            IntegrationKind::Forgejo => "forgejo",
            IntegrationKind::Linear => "linear",
            IntegrationKind::Youtrack => "youtrack",
            IntegrationKind::Redmine => "redmine",
            IntegrationKind::Gerrit => "gerrit",
            IntegrationKind::LocalMarkdown => "local_markdown",
        }
    }

    /// Human-readable name.
    pub fn label(self) -> &'static str {
        match self {
            IntegrationKind::Github => "GitHub",
            IntegrationKind::GithubEnterprise => "GitHub Enterprise",
            IntegrationKind::Gitlab => "GitLab",
            IntegrationKind::GitlabSelfManaged => "GitLab Self-Managed",
            IntegrationKind::Bitbucket => "Bitbucket",
            IntegrationKind::BitbucketDataCenter => "Bitbucket Data Center",
            IntegrationKind::AzureDevops => "Azure DevOps",
            IntegrationKind::AzureDevopsServer => "Azure DevOps Server",
            IntegrationKind::JiraCloud => "Jira Cloud",
            IntegrationKind::JiraDataCenter => "Jira Data Center",
            IntegrationKind::Trello => "Trello",
            IntegrationKind::Gitea => "Gitea",
            IntegrationKind::Forgejo => "Forgejo",
            IntegrationKind::Linear => "Linear",
            IntegrationKind::Youtrack => "YouTrack",
            IntegrationKind::Redmine => "Redmine",
            IntegrationKind::Gerrit => "Gerrit",
            IntegrationKind::LocalMarkdown => "Local Markdown",
        }
    }

    /// The fixed public host, for kinds that have exactly one.
    pub fn public_url(self) -> Option<&'static str> {
        match self {
            IntegrationKind::Github => Some("https://github.com"),
            IntegrationKind::Gitlab => Some("https://gitlab.com"),
            IntegrationKind::Bitbucket => Some("https://bitbucket.org"),
            IntegrationKind::AzureDevops => Some("https://dev.azure.com"),
            IntegrationKind::Trello => Some("https://trello.com"),
            IntegrationKind::Linear => Some("https://linear.app"),
            IntegrationKind::GithubEnterprise
            | IntegrationKind::GitlabSelfManaged
            | IntegrationKind::BitbucketDataCenter
            | IntegrationKind::AzureDevopsServer
            | IntegrationKind::JiraCloud
            | IntegrationKind::JiraDataCenter
            | IntegrationKind::Gitea
            | IntegrationKind::Forgejo
            | IntegrationKind::Youtrack
            | IntegrationKind::Redmine
            | IntegrationKind::Gerrit
            | IntegrationKind::LocalMarkdown => None,
        }
    }

    /// The host prefilled (and editable) for kinds that have a well-known
    /// public instance but are not tied to it: Codeberg for Forgejo.
    pub fn default_url(self) -> Option<&'static str> {
        match self {
            IntegrationKind::Forgejo => Some("https://codeberg.org"),
            _ => self.public_url(),
        }
    }

    pub fn family(self) -> ApiFamily {
        match self {
            IntegrationKind::Github | IntegrationKind::GithubEnterprise => ApiFamily::Github,
            IntegrationKind::Gitlab | IntegrationKind::GitlabSelfManaged => ApiFamily::Gitlab,
            IntegrationKind::Bitbucket | IntegrationKind::BitbucketDataCenter => {
                ApiFamily::Bitbucket
            }
            IntegrationKind::AzureDevops | IntegrationKind::AzureDevopsServer => {
                ApiFamily::AzureDevops
            }
            IntegrationKind::JiraCloud | IntegrationKind::JiraDataCenter => ApiFamily::Jira,
            IntegrationKind::Trello => ApiFamily::Trello,
            IntegrationKind::Gitea | IntegrationKind::Forgejo => ApiFamily::Gitea,
            IntegrationKind::Linear => ApiFamily::Linear,
            IntegrationKind::Youtrack => ApiFamily::YouTrack,
            IntegrationKind::Redmine => ApiFamily::Redmine,
            IntegrationKind::Gerrit => ApiFamily::Gerrit,
            IntegrationKind::LocalMarkdown => ApiFamily::LocalMarkdown,
        }
    }

    pub fn category(self) -> IntegrationCategory {
        match self.family() {
            ApiFamily::Jira
            | ApiFamily::Trello
            | ApiFamily::Linear
            | ApiFamily::YouTrack
            | ApiFamily::Redmine
            | ApiFamily::LocalMarkdown => IntegrationCategory::IssueTracker,
            _ => IntegrationCategory::Hosting,
        }
    }

    pub fn capabilities(self) -> Capabilities {
        let family = self.family();
        let hosting = self.category() == IntegrationCategory::Hosting;
        Capabilities {
            repos: hosting,
            pull_requests: hosting,
            // Bitbucket retired its issue tracker (2026-08-20): link Jira.
            issues: !matches!(family, ApiFamily::Bitbucket | ApiFamily::Gerrit),
            // Account SSH keys: GitHub, GitLab, Gitea/Forgejo and Bitbucket Cloud.
            ssh_keys: matches!(
                self,
                IntegrationKind::Github
                    | IntegrationKind::GithubEnterprise
                    | IntegrationKind::Gitlab
                    | IntegrationKind::GitlabSelfManaged
                    | IntegrationKind::Bitbucket
                    | IntegrationKind::Gitea
                    | IntegrationKind::Forgejo
            ),
            oauth: false,
            needs_host: self.public_url().is_none() && self != IntegrationKind::LocalMarkdown,
            implemented: Self::IMPLEMENTED.contains(&self),
            account: match self {
                IntegrationKind::JiraCloud | IntegrationKind::Bitbucket => {
                    Some(AccountField::Email)
                }
                IntegrationKind::Trello => Some(AccountField::ApiKey),
                IntegrationKind::Gerrit => Some(AccountField::Username),
                _ => None,
            },
            org: self == IntegrationKind::AzureDevops,
            local_folder: self == IntegrationKind::LocalMarkdown,
        }
    }

    /// How to mint a token for this kind.
    pub fn token_help(self) -> Option<TokenHelp> {
        if self == IntegrationKind::LocalMarkdown {
            return None;
        }
        const ATLASSIAN: TokenLink =
            TokenLink::Absolute("https://id.atlassian.com/manage-profile/security/api-tokens");
        Some(match self {
            IntegrationKind::Github | IntegrationKind::GithubEnterprise => TokenHelp {
                scopes: &["repo", "read:user", "admin:public_key"],
                link: TokenLink::Path(
                    "/settings/tokens/new?description=GitPoulpi&scopes=repo,read:user,admin:public_key",
                ),
            },
            IntegrationKind::Gitlab | IntegrationKind::GitlabSelfManaged => TokenHelp {
                scopes: &["api", "read_user"],
                link: TokenLink::Path(
                    "/-/user_settings/personal_access_tokens?name=GitPoulpi&scopes=api,read_user",
                ),
            },
            IntegrationKind::AzureDevops | IntegrationKind::AzureDevopsServer => TokenHelp {
                scopes: &["Code (Read & write)", "Work Items (Read & write)"],
                link: TokenLink::Path("/_usersSettings/tokens"),
            },
            // Scope names verified against Atlassian's API-token scope page
            // (2026-10-08).
            IntegrationKind::Bitbucket => TokenHelp {
                scopes: &[
                    "read:user:bitbucket",
                    "read:workspace:bitbucket",
                    "read:repository:bitbucket",
                    "write:repository:bitbucket",
                    "read:pullrequest:bitbucket",
                    "write:pullrequest:bitbucket",
                    "read:ssh-key:bitbucket",
                    "write:ssh-key:bitbucket",
                    "delete:ssh-key:bitbucket",
                ],
                link: ATLASSIAN,
            },
            IntegrationKind::BitbucketDataCenter => TokenHelp {
                scopes: &["Repository (Read & write)", "Projects (Read)"],
                link: TokenLink::Path("/plugins/servlet/access-tokens/manage"),
            },
            IntegrationKind::JiraCloud => TokenHelp {
                scopes: &["classic API token"],
                link: ATLASSIAN,
            },
            IntegrationKind::JiraDataCenter => TokenHelp {
                scopes: &["personal access token"],
                link: TokenLink::Path(
                    "/secure/ViewProfile.jspa?selectedTab=com.atlassian.pats.pats-plugin:jira-user-personal-access-tokens",
                ),
            },
            IntegrationKind::Trello => TokenHelp {
                scopes: &["read", "write"],
                link: TokenLink::Trello,
            },
            // The token page pre-fills nothing (Gitea has no scope query
            // string); the scopes are the ones to tick.
            IntegrationKind::Gitea | IntegrationKind::Forgejo => TokenHelp {
                scopes: &[
                    "read:user",
                    "write:repository",
                    "write:issue",
                    "write:user",
                ],
                link: TokenLink::Path("/user/settings/applications"),
            },
            // Linear: Settings > Account > Security & access > Personal API keys.
            IntegrationKind::Linear => TokenHelp {
                scopes: &["read", "write", "issues:create"],
                link: TokenLink::Absolute("https://linear.app/settings/account/security"),
            },
            // YouTrack: profile > Account Security > New token (permanent token).
            IntegrationKind::Youtrack => TokenHelp {
                scopes: &["YouTrack"],
                link: TokenLink::Path("/users/me?tab=account-security"),
            },
            // Redmine: My account > API access key (the REST API must be on).
            IntegrationKind::Redmine => TokenHelp {
                scopes: &["REST API key"],
                link: TokenLink::Path("/my/account"),
            },
            // Gerrit: Settings > HTTP Credentials > Generate new password.
            IntegrationKind::Gerrit => TokenHelp {
                scopes: &["HTTP password"],
                link: TokenLink::Path("/settings/#HTTPCredentials"),
            },
            IntegrationKind::LocalMarkdown => return None,
        })
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthMethod {
    #[default]
    Pat,
    Oauth,
}

/// Who the profile's token belongs to, cached after a successful check.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConnectedUser {
    pub login: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub name: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub id: String,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SshMode {
    #[default]
    Default,
    Generated,
    Custom,
}

/// Per-profile SSH key choice.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntegrationSsh {
    #[serde(default)]
    pub mode: SshMode,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub private_key_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub public_key_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// The provider-side key id, so it can be deleted there later.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remote_key_id: Option<String>,
}

fn default_true() -> bool {
    true
}

fn is_true(v: &bool) -> bool {
    *v
}

/// One configured profile. `token_ref` is a key into `gitpoulpi-secrets`,
/// never a token.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Integration {
    pub id: String,
    pub kind: IntegrationKind,
    pub url: String,
    pub token_ref: String,
    /// Display name; empty means "use the id".
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub name: String,
    #[serde(default)]
    pub auth: AuthMethod,
    /// The non-secret identity next to the token: the Atlassian email
    /// (`jira_cloud`, `bitbucket`) or the Trello API key. Empty otherwise.
    /// The token stays the only secret.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub account: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user: Option<ConnectedUser>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ssh: Option<IntegrationSsh>,
    #[serde(default = "default_true", skip_serializing_if = "is_true")]
    pub enabled: bool,
    /// The settings of a `local_markdown` profile; absent means the defaults.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub local: Option<LocalMarkdownSettings>,
    /// `[integrations.radar]`: the account-wide lists the Radar and
    /// `hosting radar` fetch for this profile. Absent means all off.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub radar: Option<IntegrationRadar>,
}

/// What to do with a request whose repository is not cloned.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RemoteReviewMode {
    /// Ask each time.
    #[default]
    Ask,
    /// Clone the repository, then review it there.
    Clone,
    /// Review through the host's API, without a clone.
    Api,
}

impl RemoteReviewMode {
    pub const ALL: [RemoteReviewMode; 3] = [Self::Ask, Self::Clone, Self::Api];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ask => "ask",
            Self::Clone => "clone",
            Self::Api => "api",
        }
    }

    /// `ask`, `clone` or `api`, any case.
    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|m| m.as_str().eq_ignore_ascii_case(text.trim()))
    }
}

/// `[integrations.radar]` of a code-host profile: which account-wide lists
/// (not tied to a cloned repository) the Radar shows, all off by default.
///
/// ```toml
/// [integrations.radar]
/// review_requests = true      # requests waiting for my review, every repository
/// open_requests = false       # every open request of my namespaces
/// include_drafts = false      # account rows: drafts too
/// assigned_issues = true      # open issues assigned to me, every repository
/// namespaces = ["acme"]       # empty = my own namespace plus every membership
/// review_mode = "ask"         # ask | clone | api
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntegrationRadar {
    #[serde(default)]
    pub review_requests: bool,
    #[serde(default)]
    pub open_requests: bool,
    #[serde(default)]
    pub include_drafts: bool,
    #[serde(default)]
    pub assigned_issues: bool,
    /// Orgs / groups / workspaces / Azure projects / Gerrit project
    /// prefixes; empty means the account's own namespace plus every one it
    /// is a member of.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub namespaces: Vec<String>,
    #[serde(default)]
    pub review_mode: RemoteReviewMode,
}

impl IntegrationRadar {
    /// Whether any account-wide list is on (the drafts switch and the
    /// namespaces only refine them).
    pub fn any_list(&self) -> bool {
        self.review_requests || self.open_requests || self.assigned_issues
    }
}

/// `[integrations.local]` of a `local_markdown` profile: how its notes'
/// `status` values are ordered and which one means closed.
///
/// ```toml
/// [integrations.local]
/// statuses = ["todo", "doing", "review", "done"]
/// done_status = "done"
/// reference_prefix = "LOC"   # shows LOC-12 instead of #12
/// me = "kai"                 # who "assigned to me" means
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalMarkdownSettings {
    /// The board's column order; statuses used by notes but not listed
    /// follow, in order of appearance. Empty: [`DEFAULT_STATUSES`].
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub statuses: Vec<String>,
    /// The status that closes an issue. Empty: `done`.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub done_status: String,
    /// Shown before the number (`LOC-12`); empty: `#12`.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub reference_prefix: String,
    /// The login "assigned to me" matches; empty: the connected user, else
    /// the profile name.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub me: String,
}

/// The column order of a local issue folder with no `statuses` configured.
pub const DEFAULT_STATUSES: [&str; 3] = ["todo", "doing", "done"];

impl LocalMarkdownSettings {
    /// The configured order, or the default one.
    pub fn status_order(&self) -> Vec<String> {
        if self.statuses.is_empty() {
            DEFAULT_STATUSES.iter().map(|s| s.to_string()).collect()
        } else {
            self.statuses.clone()
        }
    }

    /// The status that closes an issue.
    pub fn done(&self) -> &str {
        let d = self.done_status.trim();
        if d.is_empty() {
            "done"
        } else {
            d
        }
    }

    /// The status a reopened issue returns to: the first one in the order
    /// that is not the done status.
    pub fn open_status(&self) -> String {
        self.status_order()
            .into_iter()
            .find(|s| !s.eq_ignore_ascii_case(self.done()))
            .unwrap_or_else(|| "todo".to_string())
    }
}

impl Integration {
    /// The profile's `[integrations.radar]`, or all-off defaults.
    pub fn radar_options(&self) -> IntegrationRadar {
        self.radar.clone().unwrap_or_default()
    }

    /// The canonical `token_ref` for an id.
    pub fn token_ref_for(id: &str) -> String {
        format!("integration:{id}")
    }

    pub fn new(id: impl Into<String>, kind: IntegrationKind, url: impl Into<String>) -> Self {
        let id = id.into();
        let token_ref = Self::token_ref_for(&id);
        Self {
            id,
            kind,
            url: url.into(),
            token_ref,
            name: String::new(),
            auth: AuthMethod::Pat,
            account: String::new(),
            user: None,
            ssh: None,
            enabled: true,
            local: None,
            radar: None,
        }
    }

    /// A fresh id (`gitlab`, `gitlab-2`, ...) not used by `existing`.
    pub fn new_id<'a>(
        kind: IntegrationKind,
        existing: impl IntoIterator<Item = &'a Integration>,
    ) -> String {
        let taken: Vec<&str> = existing.into_iter().map(|i| i.id.as_str()).collect();
        let base = kind.as_str().replace('_', "-");
        if !taken.contains(&base.as_str()) {
            return base;
        }
        (2..)
            .map(|n| format!("{base}-{n}"))
            .find(|c| !taken.contains(&c.as_str()))
            .expect("infinite range")
    }

    pub fn display_name(&self) -> &str {
        if self.name.is_empty() {
            &self.id
        } else {
            &self.name
        }
    }

    /// `host[:port]` of `url`, the key credential helpers use.
    pub fn host(&self) -> Option<String> {
        host_of(&self.url)
    }

    /// The API root, derived from `url` (never configured separately).
    ///
    /// github.com uses `https://api.github.com`; GitHub Enterprise Server
    /// uses `<url>/api/v3`; GitLab always `<url>/api/v4`. Other families
    /// have no client yet and return the bare base URL.
    pub fn api_base(&self) -> String {
        let base = self.url.trim_end_matches('/');
        match self.kind.family() {
            ApiFamily::Github => {
                if is_dot_com(base, "github.com") {
                    "https://api.github.com".to_string()
                } else {
                    format!("{base}/api/v3")
                }
            }
            ApiFamily::Gitlab => format!("{base}/api/v4"),
            ApiFamily::AzureDevops => match self.kind {
                IntegrationKind::AzureDevops => normalize_azure_url(base),
                _ => base.to_string(),
            },
            ApiFamily::Bitbucket => match self.kind {
                IntegrationKind::BitbucketDataCenter => format!("{base}/rest/api/1.0"),
                _ if is_dot_com(base, "bitbucket.org") => {
                    "https://api.bitbucket.org/2.0".to_string()
                }
                _ => format!("{base}/2.0"),
            },
            ApiFamily::Jira => match self.kind {
                IntegrationKind::JiraCloud => format!("{base}/rest/api/3"),
                _ => format!("{base}/rest/api/2"),
            },
            ApiFamily::Trello => {
                if is_dot_com(base, "trello.com") {
                    "https://api.trello.com/1".to_string()
                } else {
                    format!("{base}/1")
                }
            }
            ApiFamily::Gitea => format!("{base}/api/v1"),
            // The GraphQL endpoint is `<root>/graphql`.
            ApiFamily::Linear => {
                if is_dot_com(base, "linear.app") {
                    "https://api.linear.app".to_string()
                } else {
                    base.to_string()
                }
            }
            ApiFamily::YouTrack => format!("{base}/api"),
            // The REST resources sit at the root (`/issues.json`).
            ApiFamily::Redmine => base.to_string(),
            // Authenticated calls live under `/a/`.
            ApiFamily::Gerrit => format!("{base}/a"),
            // A folder: there is no API.
            ApiFamily::LocalMarkdown => base.to_string(),
        }
    }

    /// Jira Cloud creates issues through `/rest/api/2` (plain-text
    /// description) while reading through `/3`; `None` for other kinds.
    pub fn api_base_v2(&self) -> Option<String> {
        (self.kind == IntegrationKind::JiraCloud)
            .then(|| format!("{}/rest/api/2", self.url.trim_end_matches('/')))
    }

    /// The Azure DevOps Services organization of a `dev.azure.com/{org}` or
    /// `{org}.visualstudio.com` URL. `None` for other kinds, Server, or a
    /// URL without an organization.
    pub fn azure_org(&self) -> Option<String> {
        if self.kind != IntegrationKind::AzureDevops {
            return None;
        }
        let normalized = normalize_azure_url(&self.url);
        let rest = normalized.strip_prefix("https://dev.azure.com/")?;
        let org = rest.split('/').next().unwrap_or("");
        (!org.is_empty()).then(|| org.to_string())
    }

    /// The page that mints this profile's token, given the instance's web
    /// root (`https://github.com`, ...). `None` when the kind has no flow or
    /// Trello's key is not typed yet.
    pub fn token_link_url(&self, web_base: &str) -> Option<String> {
        let help = self.kind.token_help()?;
        match help.link {
            TokenLink::Path(path) => Some(format!("{}{path}", web_base.trim_end_matches('/'))),
            TokenLink::Absolute(url) => Some(url.to_string()),
            TokenLink::Trello => {
                let key = self.account.trim();
                (!key.is_empty()).then(|| {
                    format!(
                        "https://trello.com/1/authorize?expiration=never&name=GitPoulpi&scope=read,write&response_type=token&key={key}"
                    )
                })
            }
        }
    }

    /// Applies the legacy-kind migration; see
    /// [`crate::Config::normalize_integrations`].
    pub(crate) fn normalize_kind(&mut self) {
        if self.url.trim().is_empty() {
            return;
        }
        match self.kind {
            IntegrationKind::Github if !is_dot_com(&self.url, "github.com") => {
                self.kind = IntegrationKind::GithubEnterprise;
            }
            IntegrationKind::Gitlab if !is_dot_com(&self.url, "gitlab.com") => {
                self.kind = IntegrationKind::GitlabSelfManaged;
            }
            _ => {}
        }
    }
}

/// Rewrites `https://{org}.visualstudio.com[/...]` to
/// `https://dev.azure.com/{org}`; anything else loses only a trailing slash.
pub fn normalize_azure_url(url: &str) -> String {
    let trimmed = url.trim().trim_end_matches('/');
    if let Some(host) = host_of(trimmed) {
        let lower = host.to_ascii_lowercase();
        if let Some(org) = lower.strip_suffix(".visualstudio.com") {
            if !org.is_empty() && !org.contains('.') {
                return format!("https://dev.azure.com/{org}");
            }
        }
    }
    trimmed.to_string()
}

fn host_of(url: &str) -> Option<String> {
    let rest = url.split_once("://").map(|(_, rest)| rest).unwrap_or(url);
    let host = rest.split('/').next().unwrap_or("").trim();
    (!host.is_empty()).then(|| host.to_string())
}

/// Whether `url` is the public `domain` (also `www.` and, for GitHub,
/// `api.`), ignoring scheme, case, a trailing slash and any path.
fn is_dot_com(url: &str, domain: &str) -> bool {
    let Some(host) = host_of(url) else {
        return false;
    };
    let host = host.to_ascii_lowercase();
    host == domain || host == format!("www.{domain}") || host == format!("api.{domain}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Config;

    fn migrated(kind: &str, url: &str) -> IntegrationKind {
        let raw = format!(
            "[[integrations]]\nid = \"x\"\nkind = \"{kind}\"\nurl = \"{url}\"\ntoken_ref = \"integration:x\"\n"
        );
        Config::from_toml_str(&raw).unwrap().0.integrations[0].kind
    }

    #[test]
    fn an_integration_carries_a_reference_not_a_token() {
        let integration =
            Integration::new("my-gitlab", IntegrationKind::Gitlab, "https://g.example");
        assert_eq!(integration.token_ref, "integration:my-gitlab");
        let raw = toml::to_string(&integration).unwrap();
        assert!(raw.contains("token_ref"), "{raw}");
        assert!(!raw.contains("token ="), "{raw}");
    }

    #[test]
    fn serde_spellings_are_pinned() {
        let expected = [
            (IntegrationKind::Github, "github"),
            (IntegrationKind::GithubEnterprise, "github_enterprise"),
            (IntegrationKind::Gitlab, "gitlab"),
            (IntegrationKind::GitlabSelfManaged, "gitlab_self_managed"),
            (IntegrationKind::Bitbucket, "bitbucket"),
            (
                IntegrationKind::BitbucketDataCenter,
                "bitbucket_data_center",
            ),
            (IntegrationKind::AzureDevops, "azure_devops"),
            (IntegrationKind::AzureDevopsServer, "azure_devops_server"),
            (IntegrationKind::JiraCloud, "jira_cloud"),
            (IntegrationKind::JiraDataCenter, "jira_data_center"),
            (IntegrationKind::Trello, "trello"),
            (IntegrationKind::Gitea, "gitea"),
            (IntegrationKind::Forgejo, "forgejo"),
            (IntegrationKind::Linear, "linear"),
            (IntegrationKind::Youtrack, "youtrack"),
            (IntegrationKind::Redmine, "redmine"),
            (IntegrationKind::Gerrit, "gerrit"),
            (IntegrationKind::LocalMarkdown, "local_markdown"),
        ];
        assert_eq!(expected.len(), IntegrationKind::ALL.len());
        for (kind, spelling) in expected {
            assert_eq!(kind.as_str(), spelling);
            let raw = toml::to_string(&Integration::new("x", kind, "u")).unwrap();
            assert!(raw.contains(&format!("kind = \"{spelling}\"")), "{raw}");
        }
    }

    #[test]
    fn old_toml_migrates_and_round_trips_stably() {
        let raw = r#"
[[integrations]]
id = "gh"
kind = "github"
url = "https://github.com"
token_ref = "integration:gh"

[[integrations]]
id = "ghe"
kind = "github"
url = "https://ghe.example.com"
token_ref = "integration:ghe"

[[integrations]]
id = "gl"
kind = "gitlab"
url = "https://gitlab.com/"
token_ref = "integration:gl"

[[integrations]]
id = "corp"
kind = "gitlab"
url = "https://gitlab.example.com"
token_ref = "integration:corp"
"#;
        let (config, _) = Config::from_toml_str(raw).unwrap();
        let kinds: Vec<_> = config.integrations.iter().map(|i| i.kind).collect();
        assert_eq!(
            kinds,
            [
                IntegrationKind::Github,
                IntegrationKind::GithubEnterprise,
                IntegrationKind::Gitlab,
                IntegrationKind::GitlabSelfManaged
            ]
        );
        assert!(config.integrations.iter().all(|i| i.enabled));
        // Stable: write it out and read it back, nothing moves.
        let written = toml::to_string_pretty(&config).unwrap();
        let (again, _) = Config::from_toml_str(&written).unwrap();
        assert_eq!(again.integrations, config.integrations);
        let mut twice = again.clone();
        twice.normalize_integrations();
        assert_eq!(twice.integrations, config.integrations);
    }

    #[test]
    fn migration_ignores_www_scheme_case_and_trailing_slash() {
        assert_eq!(
            migrated("github", "https://www.github.com"),
            IntegrationKind::Github
        );
        assert_eq!(
            migrated("github", "HTTPS://GitHub.com/"),
            IntegrationKind::Github
        );
        assert_eq!(
            migrated("gitlab", "https://www.gitlab.com/"),
            IntegrationKind::Gitlab
        );
        assert_eq!(
            migrated("github", "https://github.example.com/"),
            IntegrationKind::GithubEnterprise
        );
        // Never demoted, never touched when already specific.
        assert_eq!(
            migrated("github_enterprise", "https://github.com"),
            IntegrationKind::GithubEnterprise
        );
        assert_eq!(migrated("trello", "https://x"), IntegrationKind::Trello);
    }

    #[test]
    fn all_and_implemented_list_hosting_before_issue_trackers() {
        for list in [IntegrationKind::ALL, IntegrationKind::IMPLEMENTED] {
            let first_tracker = list
                .iter()
                .position(|k| k.category() == IntegrationCategory::IssueTracker)
                .unwrap();
            assert!(list[first_tracker..]
                .iter()
                .all(|k| k.category() == IntegrationCategory::IssueTracker));
        }
    }

    #[test]
    fn capabilities_and_implemented_agree() {
        for kind in IntegrationKind::ALL {
            let implemented = matches!(
                kind,
                IntegrationKind::Github
                    | IntegrationKind::GithubEnterprise
                    | IntegrationKind::Gitlab
                    | IntegrationKind::GitlabSelfManaged
                    | IntegrationKind::JiraCloud
                    | IntegrationKind::JiraDataCenter
                    | IntegrationKind::Trello
                    | IntegrationKind::Bitbucket
                    | IntegrationKind::BitbucketDataCenter
                    | IntegrationKind::AzureDevops
                    | IntegrationKind::AzureDevopsServer
                    | IntegrationKind::Gitea
                    | IntegrationKind::Forgejo
                    | IntegrationKind::Linear
                    | IntegrationKind::Youtrack
                    | IntegrationKind::Redmine
                    | IntegrationKind::Gerrit
                    | IntegrationKind::LocalMarkdown
            );
            assert_eq!(kind.capabilities().implemented, implemented, "{kind:?}");
            assert_eq!(IntegrationKind::IMPLEMENTED.contains(kind), implemented);
            assert_eq!(
                kind.token_help().is_some(),
                *kind != IntegrationKind::LocalMarkdown,
                "{kind:?}"
            );
            assert_eq!(
                kind.capabilities().needs_host,
                kind.public_url().is_none() && *kind != IntegrationKind::LocalMarkdown
            );
        }
        assert_eq!(
            IntegrationKind::JiraCloud.category(),
            IntegrationCategory::IssueTracker
        );
        assert_eq!(
            IntegrationKind::Github.category(),
            IntegrationCategory::Hosting
        );
    }

    #[test]
    fn the_api_base_is_derived_per_kind() {
        for (kind, url, want) in [
            (
                IntegrationKind::Github,
                "https://github.com",
                "https://api.github.com",
            ),
            (
                IntegrationKind::Github,
                "https://github.com/",
                "https://api.github.com",
            ),
            (
                IntegrationKind::GithubEnterprise,
                "https://ghe.example.com",
                "https://ghe.example.com/api/v3",
            ),
            (
                IntegrationKind::Gitlab,
                "https://gitlab.com",
                "https://gitlab.com/api/v4",
            ),
            (
                IntegrationKind::GitlabSelfManaged,
                "https://gitlab.example.com/",
                "https://gitlab.example.com/api/v4",
            ),
        ] {
            assert_eq!(Integration::new("x", kind, url).api_base(), want, "{url}");
        }
    }

    #[test]
    fn host_and_new_id() {
        let i = Integration::new(
            "a",
            IntegrationKind::Gitlab,
            "https://gitlab.example.com/sub",
        );
        assert_eq!(i.host().as_deref(), Some("gitlab.example.com"));
        assert_eq!(
            Integration::new("a", IntegrationKind::Gitlab, "").host(),
            None
        );
        let existing = vec![
            Integration::new("gitlab", IntegrationKind::Gitlab, "u"),
            Integration::new("gitlab-2", IntegrationKind::Gitlab, "u"),
        ];
        assert_eq!(
            Integration::new_id(IntegrationKind::Gitlab, &existing),
            "gitlab-3"
        );
        assert_eq!(
            Integration::new_id(IntegrationKind::GithubEnterprise, &existing),
            "github-enterprise"
        );
    }
    #[test]
    fn gitea_and_forgejo_share_a_family_and_forgejo_defaults_to_codeberg() {
        for kind in [IntegrationKind::Gitea, IntegrationKind::Forgejo] {
            assert_eq!(kind.family(), ApiFamily::Gitea);
            assert!(
                kind.capabilities().needs_host,
                "{kind:?}: the host stays editable"
            );
        }
        assert_eq!(
            IntegrationKind::Forgejo.default_url(),
            Some("https://codeberg.org")
        );
        assert_eq!(IntegrationKind::Gitea.default_url(), None);
        let i = Integration::new("c", IntegrationKind::Forgejo, "https://codeberg.org/");
        assert_eq!(i.api_base(), "https://codeberg.org/api/v1");
        let g = Integration::new("g", IntegrationKind::Gitea, "http://localhost:3000");
        assert_eq!(g.api_base(), "http://localhost:3000/api/v1");
    }

    #[test]
    fn azure_devops_server_spelling_is_pinned() {
        assert_eq!(
            IntegrationKind::AzureDevopsServer.as_str(),
            "azure_devops_server"
        );
        assert!(IntegrationKind::ALL.contains(&IntegrationKind::AzureDevopsServer));
        assert_eq!(IntegrationKind::ALL.len(), 18);
        assert_eq!(
            IntegrationKind::AzureDevopsServer.family(),
            ApiFamily::AzureDevops
        );
        assert!(IntegrationKind::AzureDevopsServer.capabilities().needs_host);
    }

    #[test]
    fn the_capability_table_covers_every_kind() {
        use AccountField::{ApiKey, Email, Username};
        // (kind, issues, pull_requests, ssh_keys, org, account)
        let table = [
            (IntegrationKind::Github, true, true, true, false, None),
            (
                IntegrationKind::GithubEnterprise,
                true,
                true,
                true,
                false,
                None,
            ),
            (IntegrationKind::Gitlab, true, true, true, false, None),
            (
                IntegrationKind::GitlabSelfManaged,
                true,
                true,
                true,
                false,
                None,
            ),
            (
                IntegrationKind::Bitbucket,
                false,
                true,
                true,
                false,
                Some(Email),
            ),
            (
                IntegrationKind::BitbucketDataCenter,
                false,
                true,
                false,
                false,
                None,
            ),
            (IntegrationKind::AzureDevops, true, true, false, true, None),
            (
                IntegrationKind::AzureDevopsServer,
                true,
                true,
                false,
                false,
                None,
            ),
            (
                IntegrationKind::JiraCloud,
                true,
                false,
                false,
                false,
                Some(Email),
            ),
            (
                IntegrationKind::JiraDataCenter,
                true,
                false,
                false,
                false,
                None,
            ),
            (
                IntegrationKind::Trello,
                true,
                false,
                false,
                false,
                Some(ApiKey),
            ),
            (IntegrationKind::Gitea, true, true, true, false, None),
            (IntegrationKind::Forgejo, true, true, true, false, None),
            (IntegrationKind::Linear, true, false, false, false, None),
            (IntegrationKind::Youtrack, true, false, false, false, None),
            (IntegrationKind::Redmine, true, false, false, false, None),
            (
                IntegrationKind::Gerrit,
                false,
                true,
                false,
                false,
                Some(Username),
            ),
            (
                IntegrationKind::LocalMarkdown,
                true,
                false,
                false,
                false,
                None,
            ),
        ];
        assert_eq!(table.len(), IntegrationKind::ALL.len());
        for (kind, issues, prs, ssh, org, account) in table {
            let c = kind.capabilities();
            assert_eq!(c.issues, issues, "{kind:?} issues");
            assert_eq!(c.pull_requests, prs, "{kind:?} pull_requests");
            assert_eq!(c.ssh_keys, ssh, "{kind:?} ssh_keys");
            assert_eq!(c.org, org, "{kind:?} org");
            assert_eq!(c.account, account, "{kind:?} account");
            assert!(!c.oauth, "{kind:?} oauth is off everywhere");
        }
    }

    #[test]
    fn account_round_trips_and_is_skipped_when_empty() {
        let mut i = Integration::new(
            "jira",
            IntegrationKind::JiraCloud,
            "https://acme.atlassian.net",
        );
        assert!(!toml::to_string(&i).unwrap().contains("account"));
        i.account = "me@acme.com".into();
        let raw = toml::to_string(&i).unwrap();
        assert!(raw.contains("account = \"me@acme.com\""), "{raw}");
        assert_eq!(toml::from_str::<Integration>(&raw).unwrap(), i);
        // An old file without the field loads.
        let old = "id = \"x\"\nkind = \"jira_cloud\"\nurl = \"u\"\ntoken_ref = \"t\"\n";
        assert_eq!(toml::from_str::<Integration>(old).unwrap().account, "");
    }

    #[test]
    fn radar_options_round_trip_and_an_old_file_has_none() {
        let mut i = Integration::new("gh", IntegrationKind::Github, "https://github.com");
        assert!(!toml::to_string(&i).unwrap().contains("radar"));
        assert_eq!(i.radar_options(), IntegrationRadar::default());
        assert!(!i.radar_options().any_list());
        i.radar = Some(IntegrationRadar {
            review_requests: true,
            namespaces: vec!["acme".into(), "tools".into()],
            review_mode: RemoteReviewMode::Api,
            ..IntegrationRadar::default()
        });
        let raw = toml::to_string(&i).unwrap();
        assert!(raw.contains("[radar]"), "{raw}");
        assert!(raw.contains("review_mode = \"api\""), "{raw}");
        assert_eq!(toml::from_str::<Integration>(&raw).unwrap(), i);
        assert!(i.radar_options().any_list());
        // A table naming one key leaves the others off and the mode on ask.
        let old = "id = \"x\"\nkind = \"github\"\nurl = \"u\"\ntoken_ref = \"t\"\n[radar]\nassigned_issues = true\n";
        let radar = toml::from_str::<Integration>(old).unwrap().radar_options();
        assert!(radar.assigned_issues && !radar.review_requests && !radar.open_requests);
        assert_eq!(radar.review_mode, RemoteReviewMode::Ask);
        assert_eq!(
            RemoteReviewMode::parse(" API "),
            Some(RemoteReviewMode::Api)
        );
        assert_eq!(RemoteReviewMode::parse("nope"), None);
    }

    #[test]
    fn azure_org_parses_both_url_spellings() {
        let az = |url: &str| Integration::new("a", IntegrationKind::AzureDevops, url);
        assert_eq!(
            az("https://dev.azure.com/acme").azure_org().as_deref(),
            Some("acme")
        );
        assert_eq!(
            az("https://dev.azure.com/acme/").azure_org().as_deref(),
            Some("acme")
        );
        assert_eq!(
            az("https://acme.visualstudio.com").azure_org().as_deref(),
            Some("acme")
        );
        assert_eq!(az("https://dev.azure.com").azure_org(), None);
        let server = Integration::new(
            "s",
            IntegrationKind::AzureDevopsServer,
            "https://tfs.corp/tfs/Coll",
        );
        assert_eq!(server.azure_org(), None);
        let gh = Integration::new("g", IntegrationKind::Github, "https://dev.azure.com/acme");
        assert_eq!(gh.azure_org(), None);
        assert_eq!(
            normalize_azure_url("https://acme.visualstudio.com/"),
            "https://dev.azure.com/acme"
        );
        assert_eq!(
            normalize_azure_url("https://dev.azure.com/acme/"),
            "https://dev.azure.com/acme"
        );
    }

    #[test]
    fn normalize_kind_leaves_the_new_kinds_alone() {
        for kind in [
            IntegrationKind::AzureDevopsServer,
            IntegrationKind::AzureDevops,
            IntegrationKind::Bitbucket,
            IntegrationKind::BitbucketDataCenter,
            IntegrationKind::JiraCloud,
            IntegrationKind::JiraDataCenter,
            IntegrationKind::Trello,
        ] {
            assert_eq!(migrated(kind.as_str(), "https://github.com"), kind);
            assert_eq!(migrated(kind.as_str(), "https://x.example.com"), kind);
        }
    }

    #[test]
    fn the_api_base_of_the_new_kinds() {
        for (kind, url, want) in [
            (
                IntegrationKind::AzureDevops,
                "https://dev.azure.com/acme",
                "https://dev.azure.com/acme",
            ),
            (
                IntegrationKind::AzureDevops,
                "https://acme.visualstudio.com/",
                "https://dev.azure.com/acme",
            ),
            (
                IntegrationKind::AzureDevopsServer,
                "https://tfs.corp/tfs/DefaultCollection/",
                "https://tfs.corp/tfs/DefaultCollection",
            ),
            (
                IntegrationKind::Bitbucket,
                "https://bitbucket.org",
                "https://api.bitbucket.org/2.0",
            ),
            (
                IntegrationKind::Bitbucket,
                "http://127.0.0.1:9",
                "http://127.0.0.1:9/2.0",
            ),
            (
                IntegrationKind::BitbucketDataCenter,
                "https://bb.corp/",
                "https://bb.corp/rest/api/1.0",
            ),
            (
                IntegrationKind::JiraCloud,
                "https://acme.atlassian.net/",
                "https://acme.atlassian.net/rest/api/3",
            ),
            (
                IntegrationKind::JiraDataCenter,
                "https://jira.corp",
                "https://jira.corp/rest/api/2",
            ),
            (
                IntegrationKind::Trello,
                "https://trello.com",
                "https://api.trello.com/1",
            ),
            (
                IntegrationKind::Trello,
                "http://127.0.0.1:9",
                "http://127.0.0.1:9/1",
            ),
            (
                IntegrationKind::Linear,
                "https://linear.app",
                "https://api.linear.app",
            ),
            (
                IntegrationKind::Linear,
                "http://127.0.0.1:9",
                "http://127.0.0.1:9",
            ),
            (
                IntegrationKind::Youtrack,
                "https://acme.youtrack.cloud/",
                "https://acme.youtrack.cloud/api",
            ),
            (
                IntegrationKind::Redmine,
                "http://127.0.0.1:3002/",
                "http://127.0.0.1:3002",
            ),
            (
                IntegrationKind::Gerrit,
                "http://127.0.0.1:8081/",
                "http://127.0.0.1:8081/a",
            ),
        ] {
            assert_eq!(
                Integration::new("x", kind, url).api_base(),
                want,
                "{kind:?} {url}"
            );
        }
        let cloud = Integration::new(
            "j",
            IntegrationKind::JiraCloud,
            "https://acme.atlassian.net",
        );
        assert_eq!(
            cloud.api_base_v2().as_deref(),
            Some("https://acme.atlassian.net/rest/api/2")
        );
        let dc = Integration::new("j", IntegrationKind::JiraDataCenter, "https://jira.corp");
        assert_eq!(dc.api_base_v2(), None);
    }

    #[test]
    fn the_token_link_per_kind() {
        let link = |kind: IntegrationKind| kind.token_help().expect("token help").link;
        assert!(
            matches!(link(IntegrationKind::Github), TokenLink::Path(p) if p.starts_with("/settings/tokens"))
        );
        assert!(matches!(
            link(IntegrationKind::AzureDevops),
            TokenLink::Path("/_usersSettings/tokens")
        ));
        assert!(
            matches!(link(IntegrationKind::JiraCloud), TokenLink::Absolute(u) if u.contains("id.atlassian.com"))
        );
        assert!(matches!(link(IntegrationKind::Trello), TokenLink::Trello));
        assert_eq!(
            IntegrationKind::Trello.token_help().unwrap().scopes,
            &["read", "write"]
        );
        // Trello has no link until the key is typed.
        let mut trello = Integration::new("t", IntegrationKind::Trello, "https://trello.com");
        assert_eq!(trello.token_link_url("https://trello.com"), None);
        trello.account = "KEY123".into();
        let url = trello.token_link_url("https://trello.com").unwrap();
        assert!(url.starts_with("https://trello.com/1/authorize?"), "{url}");
        assert!(
            url.contains("key=KEY123")
                && url.contains("scope=read,write")
                && url.contains("name=GitPoulpi"),
            "{url}"
        );
        let gl = Integration::new("g", IntegrationKind::Gitlab, "https://gitlab.com");
        assert!(gl
            .token_link_url("https://gitlab.com")
            .unwrap()
            .starts_with("https://gitlab.com/-/user_settings/personal_access_tokens"));
        let jira = Integration::new(
            "j",
            IntegrationKind::JiraCloud,
            "https://acme.atlassian.net",
        );
        assert_eq!(
            jira.token_link_url("https://acme.atlassian.net").as_deref(),
            Some("https://id.atlassian.com/manage-profile/security/api-tokens")
        );
    }
}
