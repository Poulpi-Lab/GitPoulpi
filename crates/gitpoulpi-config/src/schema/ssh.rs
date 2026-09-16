//! The `ssh` part of the config schema.

use serde::{Deserialize, Serialize};

/// How HTTPS remotes get their credentials. Read by the daemon before every
/// network operation (see `gitpoulpi_daemon::credentials`).
///
/// The encrypted-file modes of the first design left with the secrets file
/// (ADR-0004); their names stay readable as aliases of `credential_helper`
/// so an old `config.toml` still loads.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HttpsMode {
    /// git's own credential handling: the helper the user configured
    /// (Git Credential Manager, libsecret, Keychain), then git's prompt.
    #[default]
    #[serde(
        alias = "encrypted_file",
        alias = "credential_helper_then_encrypted_file"
    )]
    CredentialHelper,
    /// Ignore every configured helper (nothing is read from or saved to a
    /// store); git asks on the terminal when it has one. A daemon with no
    /// terminal fails with git's "could not read Username" instead.
    Prompt,
    /// No credentials at all: no helper, no prompt. Public remotes only.
    None,
}

impl HttpsMode {
    pub const ALL: [HttpsMode; 3] = [
        HttpsMode::CredentialHelper,
        HttpsMode::Prompt,
        HttpsMode::None,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            HttpsMode::CredentialHelper => "credential_helper",
            HttpsMode::Prompt => "prompt",
            HttpsMode::None => "none",
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct CredentialsConfig {
    pub https_mode: HttpsMode,
}

/// SSH key selection.
///
/// Paths, not key material: the private key stays on disk where the
/// system `ssh` and `ssh-agent` expect it, and nothing in GitPoulpi ever
/// reads it. SSH auth always goes through the system ssh-agent, the only
/// story that works identically on Windows, Linux and macOS.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SshConfig {
    pub private_key_path: Option<String>,
    pub public_key_path: Option<String>,
    /// What `ssh-keygen -t` is asked for when a key is generated from
    /// the Preferences screen.
    pub key_type: String,
}

impl Default for SshConfig {
    fn default() -> Self {
        Self {
            private_key_path: None,
            public_key_path: None,
            key_type: "ed25519".into(),
        }
    }
}
