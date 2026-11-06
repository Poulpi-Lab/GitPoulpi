//! `<private>/trust.toml`: what each integration and AI profile was trusted
//! to talk to.
//!
//! Deliberately **not** in `config.toml`: whoever (or whatever) can edit the
//! config must not be able to widen trust by editing it. The file sits in the
//! user-only private directory and is written through
//! [`gitpoulpi_config::secure_fs::write_private`].

use std::path::{Path, PathBuf};

use gitpoulpi_config::ConfigPaths;
use serde::{Deserialize, Serialize};

/// File name inside [`ConfigPaths::private_dir`].
pub const TRUST_FILE: &str = "trust.toml";

/// How a pinned certificate is held to its record.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TlsMode {
    /// Accept the recorded key when the chain does not validate (a private
    /// CA, a self-signed server); a chain that validates is accepted too.
    Tofu,
    /// The key must match the record, whatever the chain says.
    Strict,
}

impl TlsMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Tofu => "tofu",
            Self::Strict => "strict",
        }
    }
}

/// What is known about one certificate: the key fingerprint it is pinned by
/// and the fields a person compares.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CertInfo {
    /// Lowercase hex SHA-256 of the SubjectPublicKeyInfo DER.
    pub spki_sha256: String,
    #[serde(default)]
    pub subject: String,
    #[serde(default)]
    pub issuer: String,
    #[serde(default)]
    pub not_before: String,
    #[serde(default)]
    pub not_after: String,
}

impl CertInfo {
    /// The fingerprint in groups of four hex digits (`ab12 cd34 ...`).
    pub fn grouped(&self) -> String {
        self.spki_sha256
            .as_bytes()
            .chunks(4)
            .map(|c| String::from_utf8_lossy(c).into_owned())
            .collect::<Vec<_>>()
            .join(" ")
    }
}

/// `[integration.tls]` / `[ai.tls]`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TlsRecord {
    pub mode: TlsMode,
    #[serde(flatten)]
    pub cert: CertInfo,
}

/// One `[[integration]]`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntegrationRecord {
    pub id: String,
    /// Normalized `scheme://host[:port]` plus base path.
    pub origin: String,
    /// The host's own id of the account the token belongs to; empty until
    /// the first successful check (a migrated record learns it then).
    #[serde(default)]
    pub account_id: String,
    #[serde(default)]
    pub account_login: String,
    #[serde(default)]
    pub recorded_at: u64,
    #[serde(default)]
    pub account_checked_at: u64,
    #[serde(default)]
    pub allow_plain_http: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tls: Option<TlsRecord>,
}

/// One `[[ai]]`: an API-key profile bound to the origin of its base URL.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AiRecord {
    pub profile: String,
    pub origin: String,
    #[serde(default)]
    pub recorded_at: u64,
    #[serde(default)]
    pub allow_plain_http: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tls: Option<TlsRecord>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrustStore {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub integration: Vec<IntegrationRecord>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ai: Vec<AiRecord>,
}

impl TrustStore {
    /// `<private>/trust.toml`.
    pub fn file_in(paths: &ConfigPaths) -> PathBuf {
        paths.private_dir().join(TRUST_FILE)
    }

    /// The store at `path`. A missing file is an empty store; a file that is
    /// there but cannot be read or parsed is an **error**, never an empty
    /// store: an empty one would let the migration re-trust whatever the
    /// config says now.
    pub fn load(path: &Path) -> Result<Self, String> {
        match std::fs::read_to_string(path) {
            Ok(raw) => {
                toml::from_str(&raw).map_err(|e| format!("{} is unreadable: {e}", path.display()))
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(e) => Err(format!("{} is unreadable: {e}", path.display())),
        }
    }

    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            gitpoulpi_config::secure_fs::create_private_dir_all(parent)?;
        }
        let text = toml::to_string_pretty(self).map_err(std::io::Error::other)?;
        gitpoulpi_config::secure_fs::write_private(path, text.as_bytes())
    }

    pub fn integration(&self, id: &str) -> Option<&IntegrationRecord> {
        self.integration.iter().find(|r| r.id == id)
    }

    pub fn integration_mut(&mut self, id: &str) -> Option<&mut IntegrationRecord> {
        self.integration.iter_mut().find(|r| r.id == id)
    }

    pub fn put_integration(&mut self, record: IntegrationRecord) {
        match self.integration_mut(&record.id.clone()) {
            Some(slot) => *slot = record,
            None => self.integration.push(record),
        }
    }

    pub fn ai_record(&self, profile: &str) -> Option<&AiRecord> {
        self.ai.iter().find(|r| r.profile == profile)
    }

    pub fn put_ai(&mut self, record: AiRecord) {
        match self.ai.iter_mut().find(|r| r.profile == record.profile) {
            Some(slot) => *slot = record,
            None => self.ai.push(record),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_and_a_corrupt_file_is_an_error_not_an_empty_store() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("private").join(TRUST_FILE);
        assert_eq!(TrustStore::load(&path).unwrap(), TrustStore::default());
        let mut store = TrustStore::default();
        store.put_integration(IntegrationRecord {
            id: "corp".into(),
            origin: "https://git.corp.example".into(),
            account_id: "7".into(),
            account_login: "ada".into(),
            recorded_at: 1,
            account_checked_at: 2,
            allow_plain_http: false,
            tls: Some(TlsRecord {
                mode: TlsMode::Tofu,
                cert: CertInfo {
                    spki_sha256: "ab".repeat(32),
                    subject: "CN=git".into(),
                    ..CertInfo::default()
                },
            }),
        });
        store.put_ai(AiRecord {
            profile: "p".into(),
            origin: "http://localhost:11434".into(),
            recorded_at: 3,
            allow_plain_http: false,
            tls: None,
        });
        store.save(&path).unwrap();
        assert_eq!(TrustStore::load(&path).unwrap(), store);
        std::fs::write(&path, "[[integration]\nbroken").unwrap();
        assert!(TrustStore::load(&path).is_err());
    }

    #[test]
    fn fingerprints_group_by_four() {
        let c = CertInfo {
            spki_sha256: "0123456789abcdef".into(),
            ..CertInfo::default()
        };
        assert_eq!(c.grouped(), "0123 4567 89ab cdef");
    }
}
