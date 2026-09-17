//! Which host to talk to, and how to get the token for it.
//!
//! [`Integration`] and [`IntegrationKind`] are owned by `gitpoulpi-config`
//! and re-exported here; this module only adds token resolution.

pub use gitpoulpi_config::{Integration, IntegrationKind};

use crate::{Error, Result};

/// Where a token comes from. A trait so the clients can be tested with a
/// canned token and the resolution order can change without touching a
/// client.
pub trait TokenSource: Send + Sync {
    fn token(&self, integration: &Integration) -> Result<String>;
}

/// A token held in memory. For tests, and for a CLI invocation that was
/// handed one on the command line.
pub struct StaticToken(pub String);

impl TokenSource for StaticToken {
    fn token(&self, _integration: &Integration) -> Result<String> {
        Ok(self.0.clone())
    }
}

/// The environment variable an integration's token can be supplied in
/// (CI, a one-off shell), without writing it to disk.
///
/// `my-gitlab` becomes `GITPOULPI_TOKEN_MY_GITLAB`: uppercased, with every
/// non-alphanumeric turned into `_`.
pub fn token_env_var(id: &str) -> String {
    let sanitized: String = id
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_uppercase()
            } else {
                '_'
            }
        })
        .collect();
    format!("GITPOULPI_TOKEN_{sanitized}")
}

/// The default resolution order, per profile:
///
/// 1. the `GITPOULPI_TOKEN_<ID>` environment variable (one-off override,
///    CI);
/// 2. the token stored under the profile's own `token_ref`;
/// 3. the system credential helper for the profile's host - **only** when
///    exactly one enabled profile has that host. With two profiles on one
///    host (two GitHub users, say) the helper's single per-host credential
///    would silently authenticate one as the other.
///
/// SSH is deliberately absent: per ADR-0004 this crate never sees a
/// private key; token auth is for the *API*.
pub struct DefaultTokenSource {
    store: Box<dyn TokenStore + Send + Sync>,
    /// Lowercased hosts that exactly one enabled profile uses.
    unique_hosts: std::collections::HashSet<String>,
}

impl DefaultTokenSource {
    pub fn new(store: Box<dyn TokenStore + Send + Sync>, integrations: &[Integration]) -> Self {
        Self {
            store,
            unique_hosts: unique_hosts(integrations),
        }
    }

    /// The production wiring: the git credential helper as the store.
    pub fn for_integrations(integrations: &[Integration]) -> Self {
        Self::new(Box::new(CredentialHelperTokens::new()), integrations)
    }
}

/// Whether the git credential helper may stand in for a stored token.
fn helper_fallback_allowed(integration: &Integration) -> bool {
    matches!(
        integration.kind.family(),
        gitpoulpi_config::ApiFamily::Github | gitpoulpi_config::ApiFamily::Gitlab
    )
}

/// What to ask the credential helper for: the host, spelled `http://host`
/// when the profile is a plain-http server (the helper keys entries by
/// protocol), else the bare host.
fn helper_target(integration: &Integration, host: &str) -> String {
    let plain_http = integration
        .url
        .trim()
        .get(..7)
        .is_some_and(|s| s.eq_ignore_ascii_case("http://"));
    if plain_http {
        format!("http://{host}")
    } else {
        host.to_string()
    }
}

fn unique_hosts(integrations: &[Integration]) -> std::collections::HashSet<String> {
    let mut counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for host in integrations
        .iter()
        .filter(|i| i.enabled)
        .filter_map(|i| i.host())
    {
        *counts.entry(host.to_ascii_lowercase()).or_default() += 1;
    }
    counts
        .into_iter()
        .filter(|(_, n)| *n == 1)
        .map(|(h, _)| h)
        .collect()
}

impl TokenSource for DefaultTokenSource {
    fn token(&self, integration: &Integration) -> Result<String> {
        // A folder of notes has no token to find.
        if integration.kind.capabilities().local_folder {
            return Ok(String::new());
        }
        if let Some(from_env) = std::env::var(token_env_var(&integration.id))
            .ok()
            .filter(|t| !t.trim().is_empty())
        {
            return Ok(from_env);
        }

        if let Ok(Some(stored)) = self.store.load(&integration.token_ref) {
            if !stored.trim().is_empty() {
                return Ok(stored);
            }
        }

        let host = integration
            .host()
            .ok_or_else(|| Error::BadUrl(integration.url.clone()))?;
        // Only GitHub and GitLab: Azure's helper (Git Credential Manager)
        // holds an Entra OAuth token, not a PAT, and the Atlassian helpers
        // hold a git password, not an API token.
        if helper_fallback_allowed(integration)
            && self.unique_hosts.contains(&host.to_ascii_lowercase())
        {
            if let Ok(credential) = CredentialHelper.resolve(&helper_target(integration, &host)) {
                if !credential.password.trim().is_empty() {
                    return Ok(credential.password);
                }
            }
        }
        Err(Error::NoToken {
            id: integration.id.clone(),
            token_ref: integration.token_ref.clone(),
            env_var: token_env_var(&integration.id),
            host,
        })
    }
}

/// A repository-relative folder made absolute; an absolute one is kept and
/// `~/` is expanded. How a local Markdown issue source's folder is read.
pub fn resolve_local_folder(repo_root: &std::path::Path, folder: &str) -> std::path::PathBuf {
    let folder = folder.trim();
    if let Some(rest) = folder.strip_prefix("~/") {
        if let Some(home) = gitpoulpi_config::home_dir() {
            return home.join(rest);
        }
    }
    let path = std::path::Path::new(folder);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        repo_root.join(path)
    }
}

use gitpoulpi_secrets::{CredentialHelper, CredentialHelperTokens, CredentialSource, TokenStore};

#[cfg(test)]
mod tests {
    use super::*;
    use gitpoulpi_secrets::InMemoryTokens;

    fn gl(id: &str, url: &str) -> Integration {
        Integration::new(id, IntegrationKind::GitlabSelfManaged, url)
    }

    #[test]
    fn the_credential_helper_is_asked_over_http_for_a_plain_http_profile() {
        assert_eq!(
            helper_target(&gl("a", "http://nas:8080"), "nas:8080"),
            "http://nas:8080"
        );
        assert_eq!(helper_target(&gl("a", "HTTP://nas"), "nas"), "http://nas");
        assert_eq!(
            helper_target(&gl("a", "https://gl.example.com"), "gl.example.com"),
            "gl.example.com"
        );
    }

    #[test]
    fn an_integration_id_becomes_a_legal_environment_variable_name() {
        assert_eq!(token_env_var("my-gitlab"), "GITPOULPI_TOKEN_MY_GITLAB");
        assert_eq!(token_env_var("acme.corp/2"), "GITPOULPI_TOKEN_ACME_CORP_2");
    }

    #[test]
    fn the_mandated_toml_shape_parses() {
        #[derive(serde::Deserialize)]
        struct Wrapper {
            integrations: Vec<Integration>,
        }
        let parsed: Wrapper = toml::from_str(
            r#"
            [[integrations]]
            id = "my-gitlab"
            kind = "gitlab"
            url = "https://gitlab.example.com"
            token_ref = "integration:my-gitlab"
            "#,
        )
        .unwrap();
        assert_eq!(parsed.integrations[0].kind, IntegrationKind::Gitlab);
        assert_eq!(parsed.integrations[0].token_ref, "integration:my-gitlab");
    }

    /// The stored per-profile token is found by `token_ref`, and two
    /// profiles on one host each get their own - no leak between them.
    #[test]
    fn each_profile_resolves_its_own_stored_token() {
        let a = gl("tok-a", "https://gitlab.example.com");
        let b = gl("tok-b", "https://gitlab.example.com");
        let store = InMemoryTokens::default();
        store.store(&a.token_ref, "token-a").unwrap();
        store.store(&b.token_ref, "token-b").unwrap();
        let source = DefaultTokenSource::new(Box::new(store), &[a.clone(), b.clone()]);
        assert_eq!(source.token(&a).unwrap(), "token-a");
        assert_eq!(source.token(&b).unwrap(), "token-b");
    }

    /// A shared host has no unambiguous helper credential, so a profile
    /// with no stored token errors instead of borrowing the helper's.
    #[test]
    fn same_host_profiles_never_fall_back_to_the_credential_helper() {
        let a = gl("nofallback-a", "https://gitlab.invalid.example");
        let b = gl("nofallback-b", "https://gitlab.invalid.example");
        let source =
            DefaultTokenSource::new(Box::new(InMemoryTokens::default()), &[a.clone(), b.clone()]);
        assert!(matches!(source.token(&a), Err(Error::NoToken { .. })));
        assert!(source.unique_hosts.is_empty());

        let mut disabled = b.clone();
        disabled.enabled = false;
        let source = DefaultTokenSource::new(Box::new(InMemoryTokens::default()), &[a, disabled]);
        assert!(source.unique_hosts.contains("gitlab.invalid.example"));
    }

    /// A helper credential for the host exists, yet a non-GitHub/GitLab
    /// profile must not borrow it.
    #[test]
    fn the_credential_helper_fallback_is_refused_for_other_families() {
        for kind in [
            IntegrationKind::AzureDevops,
            IntegrationKind::AzureDevopsServer,
            IntegrationKind::Bitbucket,
            IntegrationKind::BitbucketDataCenter,
            IntegrationKind::JiraCloud,
            IntegrationKind::JiraDataCenter,
            IntegrationKind::Trello,
        ] {
            let i = Integration::new("nohelper-zz", kind, "https://h.invalid.example");
            assert!(!helper_fallback_allowed(&i), "{kind:?}");
            let source = DefaultTokenSource::new(
                Box::new(InMemoryTokens::default()),
                std::slice::from_ref(&i),
            );
            assert!(source.unique_hosts.contains("h.invalid.example"));
            assert!(matches!(source.token(&i), Err(Error::NoToken { .. })));
        }
        for kind in [IntegrationKind::Github, IntegrationKind::GitlabSelfManaged] {
            let i = Integration::new("x", kind, "https://h.invalid.example");
            assert!(helper_fallback_allowed(&i), "{kind:?}");
        }
    }

    #[test]
    fn the_environment_wins_over_the_store() {
        let i = gl("envwins-zz", "https://gitlab.example.com");
        let store = InMemoryTokens::default();
        store.store(&i.token_ref, "stored").unwrap();
        let var = token_env_var(&i.id);
        std::env::set_var(&var, "from-env");
        let source = DefaultTokenSource::new(Box::new(store), std::slice::from_ref(&i));
        let got = source.token(&i);
        std::env::remove_var(&var);
        assert_eq!(got.unwrap(), "from-env");
    }
}
