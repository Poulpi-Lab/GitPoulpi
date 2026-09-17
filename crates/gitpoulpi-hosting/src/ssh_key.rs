//! Generating an SSH key pair, in process.
//!
//! **Why not shell out to `ssh-keygen`.** It is not reliably installed
//! (Windows ships the OpenSSH client as an optional feature that managed
//! images often have off, and the binary on `PATH` may be Git for
//! Windows', WSL's or none) and it is interactive by default. `ssh-key`
//! produces the same OpenSSH-format files with no process and no prompt.
//!
//! **Ed25519 by default.** Short, fast, and accepted by both GitHub and
//! GitLab. `ssh.key_type` can ask for RSA (4096 bits) or ECDSA (NIST
//! P-256) instead, see [`KeyType`].
//!
//! **No passphrase.** A generated key is written unencrypted, as
//! `ssh-keygen -N ""` would, because the point is a key an agent-less
//! `git fetch` can use unattended. The file permissions below are what
//! protects it; the docstring on [`generate_and_write`] says so.

use std::path::{Path, PathBuf};

use crate::{Error, Result};

/// The kind of key to generate (`ssh.key_type`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum KeyType {
    #[default]
    Ed25519,
    /// RSA, 4096 bits.
    Rsa,
    /// ECDSA over NIST P-256.
    Ecdsa,
}

impl KeyType {
    /// Reads the `ssh.key_type` setting; anything unknown is ed25519.
    pub fn from_setting(value: &str) -> Self {
        match value.trim().to_ascii_lowercase().as_str() {
            "rsa" => Self::Rsa,
            "ecdsa" => Self::Ecdsa,
            _ => Self::Ed25519,
        }
    }

    /// The name used in file names and messages.
    pub fn name(self) -> &'static str {
        match self {
            Self::Ed25519 => "ed25519",
            Self::Rsa => "rsa",
            Self::Ecdsa => "ecdsa",
        }
    }

    fn algorithm(self) -> ssh_key::Algorithm {
        match self {
            Self::Ed25519 => ssh_key::Algorithm::Ed25519,
            Self::Rsa => ssh_key::Algorithm::Rsa { hash: None },
            Self::Ecdsa => ssh_key::Algorithm::Ecdsa {
                curve: ssh_key::EcdsaCurve::NistP256,
            },
        }
    }
}

/// A freshly generated pair. The private half is held only long enough
/// to be written; nothing in this crate sends it anywhere.
#[derive(Debug)]
pub struct GeneratedKey {
    /// The `ssh-ed25519 AAAA... <comment>` line, which is what gets
    /// uploaded.
    pub public_key: String,
    pub private_key_path: PathBuf,
    pub public_key_path: PathBuf,
    /// `SHA256:...`, for showing the user which key they just made and
    /// for matching against what the host already has.
    pub fingerprint: String,
}

/// The default key path, `~/.ssh/id_ed25519`, which is where ssh looks
/// without being told.
pub fn default_key_path() -> Result<PathBuf> {
    let home = home_dir().ok_or_else(|| {
        Error::Ssh("could not determine the home directory for ~/.ssh".to_string())
    })?;
    Ok(home.join(".ssh").join("id_ed25519"))
}

/// `dirs`-free home lookup. `HOME` is checked first (so a user who set it
/// deliberately on Windows, MSYS or Git Bash gets the `.ssh` directory
/// they use), then `USERPROFILE`, which ssh itself uses on Windows.
fn home_dir() -> Option<PathBuf> {
    for var in ["HOME", "USERPROFILE"] {
        if let Some(value) = std::env::var_os(var) {
            if !value.is_empty() {
                return Some(PathBuf::from(value));
            }
        }
    }
    None
}

/// Generates an Ed25519 pair (see [`generate_typed_and_write`] for the others) and writes it to `path` and `path.pub`.
///
/// **The private key is written without a passphrase**, and is readable
/// by anyone who can read the file. On Unix it is created `0600`; on
/// Windows it inherits the directory's ACL (for a path under the user
/// profile, the user and administrators).
///
/// Refuses to overwrite an existing key: replacing one would destroy
/// access to every host it was registered with. The caller is told to
/// pick another path or delete the old one deliberately.
pub fn generate_and_write(path: &Path, comment: &str) -> Result<GeneratedKey> {
    generate_typed_and_write(path, comment, KeyType::Ed25519)
}

/// [`generate_and_write`] for a chosen [`KeyType`].
pub fn generate_typed_and_write(
    path: &Path,
    comment: &str,
    key_type: KeyType,
) -> Result<GeneratedKey> {
    use ssh_key::{LineEnding, PrivateKey};

    let public_path = public_path_for(path);
    for existing in [path, &public_path] {
        if existing.exists() {
            return Err(Error::Ssh(format!(
                "{} already exists; delete it or choose another path rather than \
                 overwriting a key other hosts may still trust",
                existing.display()
            )));
        }
    }

    let mut key = PrivateKey::random(&mut rand_core::OsRng, key_type.algorithm()).map_err(|e| {
        Error::Ssh(format!(
            "could not generate an {} key: {e}",
            key_type.name()
        ))
    })?;
    key.set_comment(comment);

    let private_pem = key
        // Always LF: an OpenSSH private key with CRLF line endings is
        // rejected by ssh on every platform, Windows included.
        .to_openssh(LineEnding::LF)
        .map_err(|e| Error::Ssh(format!("could not encode the private key: {e}")))?;
    let public_line = key
        .public_key()
        .to_openssh()
        .map_err(|e| Error::Ssh(format!("could not encode the public key: {e}")))?;
    let fingerprint = key
        .public_key()
        .fingerprint(ssh_key::HashAlg::Sha256)
        .to_string();

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| Error::Ssh(format!("could not create {}: {e}", parent.display())))?;
    }
    write_private(path, private_pem.as_bytes())?;
    std::fs::write(&public_path, format!("{public_line}\n"))
        .map_err(|e| Error::Ssh(format!("could not write {}: {e}", public_path.display())))?;

    Ok(GeneratedKey {
        public_key: public_line,
        private_key_path: path.to_path_buf(),
        public_key_path: public_path,
        fingerprint,
    })
}

fn public_path_for(path: &Path) -> PathBuf {
    let mut name = path.as_os_str().to_os_string();
    name.push(".pub");
    PathBuf::from(name)
}

/// Writes the private key with the tightest permissions the platform
/// offers: on Unix `0600`, set *at creation* (chmod afterwards leaves a
/// window where the key is world-readable, and ssh refuses a group- or
/// world-readable key).
#[cfg(unix)]
fn write_private(path: &Path, bytes: &[u8]) -> Result<()> {
    use std::io::Write;
    use std::os::unix::fs::OpenOptionsExt;

    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)
        .map_err(|e| Error::Ssh(format!("could not create {}: {e}", path.display())))?;
    file.write_all(bytes)
        .map_err(|e| Error::Ssh(format!("could not write {}: {e}", path.display())))
}

/// Windows has no mode bits. The file inherits the ACL of its directory,
/// which under the user profile is the user plus administrators --
/// which is also exactly what `ssh-keygen` produces there, so this is
/// not a weaker guarantee than the alternative.
#[cfg(not(unix))]
fn write_private(path: &Path, bytes: &[u8]) -> Result<()> {
    use std::io::Write;

    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|e| Error::Ssh(format!("could not create {}: {e}", path.display())))?;
    file.write_all(bytes)
        .map_err(|e| Error::Ssh(format!("could not write {}: {e}", path.display())))
}

/// A default title for an uploaded key: the machine's name, so a user
/// with keys from three machines can tell them apart on the host's
/// settings page.
pub fn default_key_title() -> String {
    let host = std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .ok()
        .filter(|h| !h.trim().is_empty())
        .unwrap_or_else(|| "unknown-host".to_string());
    format!("gitpoulpi@{host}")
}

/// The private-key path a profile's generated key gets inside `ssh_dir`:
/// `gitpoulpi_<profile id>_ed25519`. Per profile so two accounts on one
/// host never share (or overwrite) a key.
pub fn profile_key_path(ssh_dir: &Path, profile_id: &str) -> PathBuf {
    let safe: String = profile_id
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect();
    ssh_dir.join(format!("gitpoulpi_{safe}_ed25519"))
}

/// The `core.sshCommand` value that pins one private key for a repository
/// (ADR-0008): `ssh -i "<key>" -o IdentitiesOnly=yes`. Backslashes become
/// forward slashes, which git's `sh` accepts on Windows too.
pub fn ssh_command_for_key(private_key_path: &str) -> String {
    gitpoulpi_core::repo_config::ssh_command_for_key(private_key_path)
}

/// `~/.ssh`, where ssh looks for keys.
pub fn ssh_dir() -> Result<PathBuf> {
    default_key_path().map(|p| p.parent().map(Path::to_path_buf).unwrap_or_default())
}

/// The title a profile's key gets when the user does not type one:
/// `GitPoulpi <host> <YYYY-MM-DD>`.
pub fn profile_key_title(host: &str) -> String {
    format!("GitPoulpi {host} {}", today_ymd())
}

/// Today's UTC date as `YYYY-MM-DD`, without a date crate (civil-from-days).
fn today_ymd() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs() as i64);
    let z = secs.div_euclid(86_400) + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{y:04}-{m:02}-{d:02}")
}

/// `SHA256:...` of an OpenSSH public-key line, or `None` when it is not one.
pub fn fingerprint_of(public_line: &str) -> Option<String> {
    ssh_key::PublicKey::from_openssh(public_line.trim())
        .ok()
        .map(|k| k.fingerprint(ssh_key::HashAlg::Sha256).to_string())
}

/// Whether two public-key lines are the same key (the comment, which
/// hosts rewrite, is ignored).
pub fn same_public_key(a: &str, b: &str) -> bool {
    let body = |s: &str| {
        let mut it = s.split_whitespace();
        Some((it.next()?.to_string(), it.next()?.to_string()))
    };
    matches!((body(a), body(b)), (Some(x), Some(y)) if x == y)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_key_type_setting_is_read_with_ed25519_as_the_fallback() {
        assert_eq!(KeyType::from_setting("rsa"), KeyType::Rsa);
        assert_eq!(KeyType::from_setting(" ECDSA "), KeyType::Ecdsa);
        assert_eq!(KeyType::from_setting("ed25519"), KeyType::Ed25519);
        assert_eq!(KeyType::from_setting("nonsense"), KeyType::Ed25519);
    }

    /// Every type produces files ssh can read back, of the type asked for.
    #[test]
    fn each_key_type_is_generated_as_asked() {
        let dir = tempfile::tempdir().unwrap();
        for (key_type, prefix, bits) in [
            (KeyType::Ed25519, "ssh-ed25519 ", None),
            (KeyType::Ecdsa, "ecdsa-sha2-nistp256 ", None),
            (KeyType::Rsa, "ssh-rsa ", Some(4096)),
        ] {
            let path = dir.path().join(format!("k_{}", key_type.name()));
            let key = generate_typed_and_write(&path, "c", key_type).unwrap();
            assert!(key.public_key.starts_with(prefix), "{}", key.public_key);
            assert!(path.is_file() && public_path_for(&path).is_file());
            let private = std::fs::read_to_string(&path).unwrap();
            let parsed = ssh_key::PrivateKey::from_openssh(&private).unwrap();
            assert_eq!(
                parsed.public_key().to_openssh().unwrap(),
                key.public_key,
                "the private file matches the public line"
            );
            if let (Some(bits), Some(rsa)) = (bits, parsed.public_key().key_data().rsa()) {
                assert_eq!(rsa.n.as_positive_bytes().unwrap().len() * 8, bits);
            }
        }
    }

    #[test]
    fn the_ssh_command_pins_one_key() {
        assert_eq!(
            ssh_command_for_key(r"C:\Users\me\.ssh\k"),
            r#"ssh -i "C:/Users/me/.ssh/k" -o IdentitiesOnly=yes"#
        );
    }

    #[test]
    fn profile_paths_titles_and_fingerprints() {
        let p = profile_key_path(Path::new("/h/.ssh"), "gitlab-self-managed-2");
        assert!(p.ends_with("gitpoulpi_gitlab-self-managed-2_ed25519"));
        assert!(profile_key_path(Path::new("/h"), "a/b c").ends_with("gitpoulpi_a_b_c_ed25519"));
        let t = profile_key_title("gitlab.example.com");
        assert!(t.starts_with("GitPoulpi gitlab.example.com 20"), "{t}");
        let dir = tempfile::tempdir().unwrap();
        let k = generate_and_write(&dir.path().join("k"), "c").unwrap();
        assert_eq!(
            fingerprint_of(&k.public_key).as_deref(),
            Some(k.fingerprint.as_str())
        );
        assert!(same_public_key(
            &k.public_key,
            &format!(
                "{} other",
                k.public_key
                    .split(' ')
                    .take(2)
                    .collect::<Vec<_>>()
                    .join(" ")
            )
        ));
        assert!(!same_public_key("ssh-ed25519 AAA", "ssh-ed25519 BBB"));
        assert_eq!(fingerprint_of("garbage"), None);
    }

    #[test]
    fn a_generated_key_is_a_usable_openssh_pair() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("keys").join("id_ed25519");

        let key = generate_and_write(&path, "gitpoulpi@test").unwrap();

        assert!(key.public_key.starts_with("ssh-ed25519 "));
        assert!(key.public_key.ends_with("gitpoulpi@test"));
        assert!(key.fingerprint.starts_with("SHA256:"));
        assert_eq!(key.public_key_path, path.with_extension("pub"));

        let private = std::fs::read_to_string(&path).unwrap();
        assert!(private.starts_with("-----BEGIN OPENSSH PRIVATE KEY-----"));
        assert!(
            !private.contains('\r'),
            "an OpenSSH private key with CRLF is rejected by ssh everywhere"
        );

        // The written public file must be the same line that gets
        // uploaded, or the host trusts a key the machine does not have.
        let public_file = std::fs::read_to_string(&key.public_key_path).unwrap();
        assert_eq!(public_file.trim_end(), key.public_key);

        // And it must round-trip through a real parser, not just look
        // right.
        let parsed = ssh_key::PublicKey::from_openssh(&key.public_key).unwrap();
        assert_eq!(parsed.algorithm(), ssh_key::Algorithm::Ed25519);
    }

    #[test]
    fn two_generated_keys_are_different() {
        let dir = tempfile::tempdir().unwrap();
        let a = generate_and_write(&dir.path().join("a"), "a").unwrap();
        let b = generate_and_write(&dir.path().join("b"), "b").unwrap();
        assert_ne!(a.public_key, b.public_key);
        assert_ne!(a.fingerprint, b.fingerprint);
    }

    /// Overwriting a key revokes access to every host that trusts it,
    /// with no undo.
    #[test]
    fn an_existing_key_is_never_overwritten() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("id_ed25519");
        let first = generate_and_write(&path, "first").unwrap();

        let err = generate_and_write(&path, "second").unwrap_err();
        assert!(err.to_string().contains("already exists"), "{err}");
        assert_eq!(
            std::fs::read_to_string(&first.public_key_path)
                .unwrap()
                .trim_end(),
            first.public_key,
            "the original key must be untouched"
        );
    }

    /// A stray `id_ed25519.pub` with no private half still means "there
    /// is something here"; clobbering it would leave a public key that
    /// no private key matches.
    #[test]
    fn a_lone_public_key_also_blocks_generation() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("id_ed25519");
        std::fs::write(path.with_extension("pub"), "ssh-ed25519 AAAA old\n").unwrap();
        assert!(generate_and_write(&path, "new").is_err());
    }

    #[cfg(unix)]
    #[test]
    fn the_private_key_is_not_readable_by_anyone_else() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("id_ed25519");
        generate_and_write(&path, "test").unwrap();
        let mode = std::fs::metadata(&path).unwrap().permissions().mode();
        assert_eq!(mode & 0o077, 0, "ssh refuses a group/world-readable key");
    }
}
