//! The account (integration) HTTPS source (ADR-0010): a GitPoulpi account's
//! token reaches git through a credential grant, never through an argument or
//! an environment variable.
//!
//! [`apply`] resolves the account (its trust-store origin, its git user name,
//! its token), issues the grant that serves the token (`grants`), and puts
//! into the repository's `NetworkAuth` only what git's child may see: the
//! helper's command, two flags and the grant id with the session name. Builds
//! without the `hosting` feature have no accounts: the source refuses.

use std::path::Path;
use std::sync::Mutex;

use gitpoulpi_core::domain::NetworkAuth;
use gitpoulpi_core::repo_config::sh_quote_path;
use gitpoulpi_core::Error;

use crate::grants::GrantGuard;

fn unavailable(reason: impl Into<String>) -> Error {
    Error::RepoAuthUnavailable {
        reason: reason.into(),
    }
}

/// The executable git runs as the credential helper; `None` is this process's
/// own (every GitPoulpi binary that hosts a daemon answers the helper flag).
static HELPER_EXE: Mutex<Option<std::path::PathBuf>> = Mutex::new(None);

/// Test hook: run `exe` instead of the current executable as git's credential
/// helper (a test binary has no helper entry of its own).
pub fn set_credential_helper_exe(exe: Option<std::path::PathBuf>) {
    *HELPER_EXE.lock().unwrap_or_else(|e| e.into_inner()) = exe;
}

/// `!"<exe>" --gitpoulpi-git-credential`: how git is told to run the helper.
/// The path goes through the same quoting as the SSH key command (a path with
/// spaces, `"`, `$` or backslashes survives Git for Windows' `sh`); the flag
/// is plain ASCII so MSYS path conversion leaves it alone.
pub(crate) fn helper_command() -> Result<String, Error> {
    let configured = HELPER_EXE.lock().unwrap_or_else(|e| e.into_inner()).clone();
    let exe = match configured {
        Some(exe) => exe,
        None => std::env::current_exe()
            .map_err(|e| unavailable(format!("this program's own path is unknown: {e}")))?,
    };
    Ok(format!(
        "!{} {}",
        sh_quote_path(&exe.to_string_lossy()),
        gitpoulpi_ipc::git_credential::FLAG
    ))
}

/// What git's child gets in its environment: the grant id, the session
/// whose daemon serves it and the path of the file naming that daemon's
/// endpoint. None of these is a secret.
pub(crate) fn grant_env(grant: &GrantGuard, session: &str) -> Vec<(String, String)> {
    let mut env = vec![
        (
            gitpoulpi_ipc::git_credential::ENV_GRANT.to_string(),
            grant.id().to_string(),
        ),
        (
            gitpoulpi_ipc::git_credential::ENV_SESSION.to_string(),
            session.to_string(),
        ),
    ];
    // Where the issuing daemon listens: the path of a user-only file that
    // lives as long as the grant, never the token itself.
    if let Some(file) = grant.endpoint_file(session) {
        env.push((
            gitpoulpi_ipc::git_credential::ENV_ENDPOINT.to_string(),
            file.to_string_lossy().into_owned(),
        ));
    }
    env
}

/// Resolves account `id` for a transfer of the repository at `root` of
/// session `session` and fills `auth` for it. `session: None` only checks that
/// the account can be used (no token is read, no grant issued).
#[cfg(feature = "hosting")]
pub(crate) fn apply(
    root: &Path,
    auth: &mut NetworkAuth,
    id: &str,
    username: Option<&str>,
    session: Option<&str>,
) -> Result<Option<GrantGuard>, Error> {
    hosting::apply(root, auth, id, username, session)
}

#[cfg(not(feature = "hosting"))]
pub(crate) fn apply(
    _root: &Path,
    _auth: &mut NetworkAuth,
    id: &str,
    _username: Option<&str>,
    _session: Option<&str>,
) -> Result<Option<GrantGuard>, Error> {
    Err(unavailable(format!(
        "the account \"{id}\" cannot be used: this build has no GitPoulpi accounts"
    )))
}

#[cfg(feature = "hosting")]
pub use hosting::set_token_store;

#[cfg(feature = "hosting")]
mod hosting {
    use std::sync::{Arc, Mutex};

    use gitpoulpi_config::Integration;
    use gitpoulpi_hosting::{git_username, DefaultTokenSource, TokenSource, TrustGate};
    use gitpoulpi_secrets::TokenStore;

    use super::*;
    use crate::grants::{self, GrantCredential};

    /// Where account tokens are read from; `None` is the credential helper
    /// store the rest of GitPoulpi uses.
    type SharedStore = Arc<dyn TokenStore + Send + Sync>;
    static TOKEN_STORE: Mutex<Option<SharedStore>> = Mutex::new(None);

    /// Test hook: read account tokens from `store` instead of the system
    /// credential helper (so a test never touches the real one).
    pub fn set_token_store(store: Option<SharedStore>) {
        *TOKEN_STORE.lock().unwrap_or_else(|e| e.into_inner()) = store;
    }

    /// A shared store as the boxed one `DefaultTokenSource` takes.
    struct Shared(SharedStore);

    impl TokenStore for Shared {
        fn store(&self, token_ref: &str, token: &str) -> gitpoulpi_secrets::Result<()> {
            self.0.store(token_ref, token)
        }
        fn load(&self, token_ref: &str) -> gitpoulpi_secrets::Result<Option<String>> {
            self.0.load(token_ref)
        }
        fn delete(&self, token_ref: &str) -> gitpoulpi_secrets::Result<()> {
            self.0.delete(token_ref)
        }
    }

    /// An account with everything a grant is made from but its token.
    pub(super) struct Account {
        pub integration: Integration,
        pub integrations: Vec<Integration>,
        pub username: String,
        /// `http` or `https`, from the trust record's origin.
        pub scheme: String,
        /// `host` or `host:port`, lowercase, from the trust record's origin.
        pub host: String,
    }

    /// `(scheme, host[:port])` of a normalized origin (`scheme://host[:port]`
    /// with an optional base path), lowercase.
    pub(super) fn split_origin(origin: &str) -> Option<(String, String)> {
        let (scheme, rest) = origin.split_once("://")?;
        let authority = rest.split(['/', '?', '#']).next()?;
        let host = authority.rsplit('@').next()?;
        if scheme.is_empty() || host.is_empty() {
            return None;
        }
        Some((scheme.to_ascii_lowercase(), host.to_ascii_lowercase()))
    }

    pub(super) fn account(id: &str, username: Option<&str>) -> Result<Account, Error> {
        let settings = gitpoulpi_config::Settings::resolve_ambient()
            .map_err(|e| unavailable(format!("the settings could not be read: {e}")))?;
        let integrations = settings.config().integrations.clone();
        let integration = integrations
            .iter()
            .find(|i| i.id == id)
            .cloned()
            .ok_or_else(|| unavailable(format!("the account \"{id}\" no longer exists")))?;
        if !integration.enabled {
            return Err(unavailable(format!("the account \"{id}\" is disabled")));
        }
        // The address the token is bound to is the **trust store's** record,
        // never the config's (whoever can edit the config must not be able to
        // send the token elsewhere): `admit` refuses a config that no longer
        // matches its record.
        let admitted = TrustGate::ambient()
            .admit(&integration)
            .map_err(|e| unavailable(format!("the account \"{id}\" is not trusted: {e}")))?;
        let (scheme, host) = split_origin(&admitted.record.origin)
            .ok_or_else(|| unavailable(format!("the account \"{id}\" has no usable address")))?;
        let login = integration
            .user
            .as_ref()
            .map(|u| u.login.as_str())
            .filter(|l| !l.is_empty())
            .unwrap_or(&admitted.record.account_login);
        let default = git_username(integration.kind, &integration.account, login)
            .ok_or_else(|| unavailable(format!("the account \"{id}\" is not a code host")))?;
        let username = username
            .map(str::trim)
            .filter(|u| !u.is_empty())
            .map(str::to_string)
            .unwrap_or(default);
        Ok(Account {
            integration,
            integrations,
            username,
            scheme,
            host,
        })
    }

    pub(super) fn token_of(account: &Account) -> Result<String, Error> {
        let store: Box<dyn TokenStore + Send + Sync> = match TOKEN_STORE
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
        {
            Some(store) => Box::new(Shared(store)),
            None => Box::new(gitpoulpi_secrets::CredentialHelperTokens::new()),
        };
        DefaultTokenSource::new(store, &account.integrations)
            .token(&account.integration)
            .ok()
            .filter(|t| !t.trim().is_empty())
            .ok_or_else(|| {
                unavailable(format!(
                    "the account \"{}\" has no token on this machine",
                    account.integration.id
                ))
            })
    }

    pub(super) fn apply(
        root: &Path,
        auth: &mut NetworkAuth,
        id: &str,
        username: Option<&str>,
        session: Option<&str>,
    ) -> Result<Option<GrantGuard>, Error> {
        let account = account(id, username)?;
        let Some(session) = session else {
            return Ok(None);
        };
        let token = token_of(&account)?;
        // From here on this value is masked wherever it appears in a log line
        // or an error report, whatever it looks like.
        gitpoulpi_core::redact_register_secret(&token);
        authorize(root, auth, &account, token, session).map(Some)
    }

    /// Issues the grant that serves `token` and puts everything git's child
    /// needs into `auth`. The token goes into the grant only: `auth` holds the
    /// helper's command, two flags and the grant id.
    pub(super) fn authorize(
        root: &Path,
        auth: &mut NetworkAuth,
        account: &Account,
        token: String,
        session: &str,
    ) -> Result<GrantGuard, Error> {
        let grant = grants::issue(
            root,
            GrantCredential {
                username: account.username.clone(),
                password: token,
                scheme: account.scheme.clone(),
                hosts: vec![account.host.clone()],
                allow_plain_http: false,
                command: None,
                identity: format!(
                    "account {} (token, user {})",
                    account.integration.id, account.username
                ),
            },
        )
        .map_err(|e| unavailable(format!("no random grant could be made: {e}")))?;
        auth.credential_helpers = Some(vec![helper_command()?]);
        auth.no_prompt = true;
        auth.reset_extra_header = true;
        auth.env = grant_env(&grant, session);
        Ok(grant)
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use gitpoulpi_config::IntegrationKind;

        /// The token is in the grant and nowhere in what git's child gets: not
        /// in the `-c` helper list, not in the environment, not in anything
        /// `NetworkAuth` holds.
        #[test]
        fn no_secret_is_in_the_auth_the_child_is_built_from() {
            let token = "zz-plain-0123456789-secret";
            let account = Account {
                integration: Integration::new(
                    "work",
                    IntegrationKind::Github,
                    "https://github.com",
                ),
                integrations: Vec::new(),
                username: "x-access-token".into(),
                scheme: "https".into(),
                host: "github.com".into(),
            };
            let mut auth = NetworkAuth::default();
            let grant = authorize(
                Path::new("/repo-no-secret"),
                &mut auth,
                &account,
                token.to_string(),
                "my-session",
            )
            .unwrap();
            let shown = format!("{auth:?}");
            assert!(!shown.contains(token), "{shown}");
            let helpers = auth.credential_helpers.clone().unwrap();
            assert_eq!(helpers.len(), 1);
            assert!(helpers[0].starts_with("!\""), "{}", helpers[0]);
            assert!(helpers[0].ends_with(" --gitpoulpi-git-credential"));
            assert!(!helpers[0].contains(token));
            let names: Vec<&str> = auth.env.iter().map(|(k, _)| k.as_str()).collect();
            assert_eq!(
                names,
                ["GITPOULPI_CREDENTIAL_GRANT", "GITPOULPI_CREDENTIAL_SESSION"]
            );
            assert_eq!(auth.env[0].1, grant.id());
            assert_eq!(auth.env[1].1, "my-session");
            assert!(auth.no_prompt && auth.reset_extra_header);
            assert!(auth.ssh_command.is_none() && auth.credential_username.is_none());
            for (_, value) in &auth.env {
                assert!(!value.contains(token));
            }
        }

        #[test]
        fn an_origin_is_split_into_scheme_and_host_with_its_port() {
            assert_eq!(
                split_origin("https://GitHub.com"),
                Some(("https".into(), "github.com".into()))
            );
            assert_eq!(
                split_origin("http://127.0.0.1:8080/base/path"),
                Some(("http".into(), "127.0.0.1:8080".into()))
            );
            assert_eq!(
                split_origin("https://user@git.corp.example:8443/x"),
                Some(("https".into(), "git.corp.example:8443".into()))
            );
            assert_eq!(split_origin("no-scheme"), None);
            assert_eq!(split_origin("https://"), None);
        }
    }
}
