//! A repository's authentication profile (ADR-0010): which identity its
//! network git processes use, chosen per repository and stored in
//! GitPoulpi's private directory, never in the repository or in git's config.
//! These are the wire and file types; the storage is `gitpoulpi-session`'s
//! `repo_auth`, the resolution into a [`crate::domain::NetworkAuth`] is the
//! daemon's.

use serde::{Deserialize, Serialize};

use super::RemoteTransport;

/// How HTTPS remotes authenticate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "source", rename_all = "snake_case")]
pub enum HttpsAuth {
    /// git's own helpers (the machine's identity), with two optional extras.
    System {
        /// `credential.username`: lets a credential manager pick one of
        /// several stored accounts for the host.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        username: Option<String>,
        /// Drop the credential helpers this repository sets itself (its
        /// `.git/config`, files that includes, a worktree config) and keep
        /// the system and global ones.
        #[serde(default, skip_serializing_if = "is_false")]
        ignore_repo_helpers: bool,
    },
    /// A GitPoulpi integration account's token. Not available yet.
    Integration {
        integration: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        username: Option<String>,
    },
    /// One credential helper command. Not available yet.
    Helper {
        helper: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        username: Option<String>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        hosts: Vec<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        confirmed_sha256: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        script_sha256: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        confirmed_at: Option<i64>,
    },
    /// A command that prints a token. Not available yet.
    TokenCommand {
        argv: Vec<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        username: Option<String>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        hosts: Vec<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        cache_minutes: Option<u32>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        confirmed_sha256: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        script_sha256: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        confirmed_at: Option<i64>,
    },
}

/// How SSH remotes authenticate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "source", rename_all = "snake_case")]
pub enum SshAuth {
    /// ssh's own configuration, the agent and the global key.
    System,
    /// A key file, offered with `IdentitiesOnly`.
    KeyFile {
        key: String,
        /// `UserKnownHostsFile`.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        known_hosts: Option<String>,
        /// `StrictHostKeyChecking=yes`.
        #[serde(default, skip_serializing_if = "is_false")]
        strict_host_key: bool,
    },
    /// The key of a GitPoulpi integration profile (`IntegrationSsh`),
    /// resolved live at each transfer.
    IntegrationKey {
        integration: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        known_hosts: Option<String>,
        #[serde(default, skip_serializing_if = "is_false")]
        strict_host_key: bool,
    },
}

fn is_false(b: &bool) -> bool {
    !*b
}

/// What a repository authenticates with. `None` for a protocol means "the
/// default" (the machine's own identity, as without a profile).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepoAuthProfile {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub https: Option<HttpsAuth>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ssh: Option<SshAuth>,
}

impl RepoAuthProfile {
    /// Whether this profile changes nothing against having none: no source,
    /// or only default ones.
    pub fn is_default(&self) -> bool {
        let https_default = match &self.https {
            None => true,
            Some(HttpsAuth::System {
                username,
                ignore_repo_helpers,
            }) => username.as_deref().is_none_or(str::is_empty) && !ignore_repo_helpers,
            Some(_) => false,
        };
        https_default && matches!(self.ssh, None | Some(SshAuth::System))
    }

    /// A short label for the push indicator (`None` when everything is
    /// default). Never contains a secret: an account id, a key's file name.
    pub fn summary(&self) -> Option<String> {
        if self.is_default() {
            return None;
        }
        let mut parts = Vec::new();
        match &self.https {
            None => {}
            Some(HttpsAuth::System {
                username,
                ignore_repo_helpers,
            }) => {
                let user = username.as_deref().filter(|u| !u.is_empty());
                match (user, ignore_repo_helpers) {
                    (Some(u), true) => parts.push(format!("HTTPS: {u}, repo helpers ignored")),
                    (Some(u), false) => parts.push(format!("HTTPS: {u}")),
                    (None, true) => parts.push("HTTPS: repo helpers ignored".to_string()),
                    (None, false) => {}
                }
            }
            Some(HttpsAuth::Integration { integration, .. }) => {
                parts.push(format!("HTTPS: {integration}"))
            }
            Some(HttpsAuth::Helper { .. }) => parts.push("HTTPS: credential helper".to_string()),
            Some(HttpsAuth::TokenCommand { .. }) => parts.push("HTTPS: token command".to_string()),
        }
        match &self.ssh {
            None | Some(SshAuth::System) => {}
            Some(SshAuth::KeyFile { key, .. }) => {
                parts.push(format!("SSH key: {}", file_name_of(key)))
            }
            Some(SshAuth::IntegrationKey { integration, .. }) => {
                parts.push(format!("SSH key of {integration}"))
            }
        }
        Some(parts.join(", "))
    }
}

/// The SHA-256 (lower-case hex) of a command-running source's command: the
/// helper string, or the arguments of a token command joined by NUL. What a
/// person confirms is shown with this hash.
pub fn command_sha256(parts: &[&str]) -> String {
    use sha2::{Digest, Sha256};
    let digest = Sha256::digest(parts.join("\0").as_bytes());
    digest.iter().map(|b| format!("{b:02x}")).collect()
}

impl HttpsAuth {
    /// The hash of the command of a helper or token-command source, `None`
    /// for the others.
    pub fn command_sha256(&self) -> Option<String> {
        match self {
            HttpsAuth::Helper { helper, .. } => Some(command_sha256(&[helper])),
            HttpsAuth::TokenCommand { argv, .. } => {
                let parts: Vec<&str> = argv.iter().map(String::as_str).collect();
                Some(command_sha256(&parts))
            }
            _ => None,
        }
    }
}

/// The last component of a path written with either separator.
fn file_name_of(path: &str) -> &str {
    path.rsplit(['/', '\\'])
        .find(|s| !s.is_empty())
        .unwrap_or(path)
}

/// `RepoAuthSet` arguments.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RepoAuthSetArgs {
    pub profile: RepoAuthProfile,
    /// The requester confirms a command-running source (helper, token
    /// command). Only a person may; those sources are not available yet, so
    /// today it changes nothing.
    #[serde(default)]
    pub confirm: bool,
}

/// `RepoAuthTest` arguments: one remote, or every remote of the repository.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RepoAuthTestArgs {
    #[serde(default)]
    pub remote: Option<String>,
    #[serde(default)]
    pub push: bool,
}

/// Which identity one remote will authenticate with.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteIdentity {
    pub remote: String,
    /// The fetch URL without userinfo.
    pub url: String,
    pub transport: RemoteTransport,
    /// Plain words, never a secret: "system git", "system git (user alice)",
    /// "ssh key work_ed25519".
    pub identity: String,
}

/// A repository setting that the profile overrides or that competes with it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthOverride {
    /// `core.sshCommand` or `credential.helper`.
    pub key: String,
    pub value: String,
    /// `local`, `worktree`, `global`, `system`...
    pub scope: String,
    /// The file it comes from.
    pub origin: String,
    /// Whether the profile ignores it (it does not apply to this
    /// repository's transfers).
    pub overridden: bool,
}

/// `RepoAuthGet`'s answer. Carries no secret: a command-running source's
/// command is blanked.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepoAuthView {
    /// Whether the repository has a stored profile at all.
    pub configured: bool,
    pub profile: RepoAuthProfile,
    pub summary: Option<String>,
    /// The common git directory recorded when the profile was saved.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub git_common_dir: Option<String>,
    pub remotes: Vec<RemoteIdentity>,
    pub overrides: Vec<AuthOverride>,
    pub warnings: Vec<String>,
    /// Why the profile cannot be used for a transfer right now, if so (the
    /// transfer would be refused with `RepoAuthUnavailable`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unavailable: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_profile_with_only_defaults_has_no_summary() {
        assert_eq!(RepoAuthProfile::default().summary(), None);
        let p = RepoAuthProfile {
            https: Some(HttpsAuth::System {
                username: Some(String::new()),
                ignore_repo_helpers: false,
            }),
            ssh: Some(SshAuth::System),
        };
        assert!(p.is_default());
        assert_eq!(p.summary(), None);
    }

    #[test]
    fn the_summary_names_the_choice_without_a_path_or_secret() {
        let p = RepoAuthProfile {
            https: Some(HttpsAuth::System {
                username: None,
                ignore_repo_helpers: true,
            }),
            ssh: Some(SshAuth::KeyFile {
                key: r"C:\Users\me\.ssh\work_ed25519".into(),
                known_hosts: None,
                strict_host_key: false,
            }),
        };
        assert_eq!(
            p.summary().as_deref(),
            Some("HTTPS: repo helpers ignored, SSH key: work_ed25519")
        );
    }
}
