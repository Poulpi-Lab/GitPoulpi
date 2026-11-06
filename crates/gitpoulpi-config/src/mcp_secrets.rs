//! The secrets of the MCP HTTP endpoint: the bearer token and the
//! self-signed TLS pair, both under `<data>/private` (user-only, see
//! [`crate::secure_fs`]).
//!
//! They live in the config crate because three programs read them: the MCP
//! server (checks the token, loads the pair), the CLI (`gitpoulpi mcp
//! token`) and the TUI (Settings > Agents & MCP shows them masked).

use std::io;
use std::path::PathBuf;

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use sha2::{Digest, Sha256};

use crate::{secure_fs, ConfigPaths};

/// Where the bearer token lives: `<private>/mcp-http.token`.
pub fn token_file(paths: &ConfigPaths) -> PathBuf {
    paths.private_dir().join("mcp-http.token")
}

/// The folder of the self-signed pair: `<private>/mcp-tls`.
pub fn tls_dir(paths: &ConfigPaths) -> PathBuf {
    paths.private_dir().join("mcp-tls")
}

/// The self-signed certificate (PEM).
pub fn self_signed_cert(paths: &ConfigPaths) -> PathBuf {
    tls_dir(paths).join("cert.pem")
}

/// The self-signed private key (PEM).
pub fn self_signed_key(paths: &ConfigPaths) -> PathBuf {
    tls_dir(paths).join("key.pem")
}

/// 32 random bytes from the operating system, as unpadded base64url.
pub fn generate_token() -> io::Result<String> {
    let mut bytes = [0u8; 32];
    getrandom::fill(&mut bytes).map_err(|e| io::Error::other(e.to_string()))?;
    Ok(URL_SAFE_NO_PAD.encode(bytes))
}

/// The stored token, trimmed; `None` when there is none (or it is empty).
pub fn read_token(paths: &ConfigPaths) -> Option<String> {
    let text = std::fs::read_to_string(token_file(paths)).ok()?;
    let text = text.trim();
    (!text.is_empty()).then(|| text.to_string())
}

/// Writes a fresh token over any old one (user-only) and returns it.
pub fn rotate_token(paths: &ConfigPaths) -> io::Result<String> {
    let token = generate_token()?;
    write_token(paths, &token)?;
    Ok(token)
}

fn write_token(paths: &ConfigPaths, token: &str) -> io::Result<()> {
    secure_fs::create_private_dir_all(&paths.private_dir())?;
    secure_fs::write_private(&token_file(paths), format!("{token}\n").as_bytes())
}

/// The stored token, created on first use. The flag is `true` when this
/// call created it.
pub fn ensure_token(paths: &ConfigPaths) -> io::Result<(String, bool)> {
    match read_token(paths) {
        Some(token) => Ok((token, false)),
        None => rotate_token(paths).map(|t| (t, true)),
    }
}

/// The token with its middle hidden: `AbCd...WxYz`. A short value is fully
/// hidden.
pub fn token_hint(token: &str) -> String {
    let chars: Vec<char> = token.chars().collect();
    if chars.len() < 12 {
        return "*".repeat(chars.len().max(4));
    }
    let head: String = chars[..4].iter().collect();
    let tail: String = chars[chars.len() - 4..].iter().collect();
    format!("{head}...{tail}")
}

/// The OSC 52 sequence that asks a terminal to set its clipboard to `text`.
pub fn osc52(text: &str) -> String {
    let encoded = base64::engine::general_purpose::STANDARD.encode(text);
    format!("\x1b]52;c;{encoded}\x07")
}

/// The SHA-256 fingerprint of the first certificate of a PEM text, as
/// colon-separated uppercase hex pairs (what a client pins). `None` when the
/// text holds no certificate.
pub fn fingerprint_of_pem(pem: &str) -> Option<String> {
    let begin = "-----BEGIN CERTIFICATE-----";
    let end = "-----END CERTIFICATE-----";
    let start = pem.find(begin)? + begin.len();
    let stop = start + pem[start..].find(end)?;
    let body: String = pem[start..stop]
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect();
    let der = base64::engine::general_purpose::STANDARD
        .decode(body)
        .ok()?;
    let digest = Sha256::digest(&der);
    Some(
        digest
            .iter()
            .map(|b| format!("{b:02X}"))
            .collect::<Vec<_>>()
            .join(":"),
    )
}

/// The fingerprint of the certificate file at `path`.
pub fn fingerprint_of_file(path: &std::path::Path) -> Option<String> {
    fingerprint_of_pem(&std::fs::read_to_string(path).ok()?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_token_is_created_once_and_rotated_on_request() {
        let dir = tempfile::tempdir().unwrap();
        let paths = ConfigPaths::for_dir(dir.path());
        assert!(read_token(&paths).is_none());
        let (first, created) = ensure_token(&paths).unwrap();
        assert!(created);
        assert_eq!(first.len(), 43, "32 bytes, unpadded base64url");
        let (again, created) = ensure_token(&paths).unwrap();
        assert!(!created);
        assert_eq!(first, again);
        let rotated = rotate_token(&paths).unwrap();
        assert_ne!(rotated, first);
        assert_eq!(read_token(&paths).unwrap(), rotated);
    }

    #[cfg(unix)]
    #[test]
    fn the_token_file_is_user_only() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let paths = ConfigPaths::for_dir(dir.path());
        ensure_token(&paths).unwrap();
        let mode = std::fs::metadata(token_file(&paths))
            .unwrap()
            .permissions()
            .mode();
        assert_eq!(mode & 0o777, 0o600);
    }

    #[test]
    fn the_hint_never_shows_the_middle() {
        let hint = token_hint("abcdefghijklmnopqrstuvwxyz0123456789ABCDEFG");
        assert_eq!(hint, "abcd...DEFG");
        assert!(!token_hint("short").contains("sh"));
    }

    #[test]
    fn osc52_wraps_the_base64_text() {
        assert_eq!(osc52("abc"), "\x1b]52;c;YWJj\x07");
    }

    #[test]
    fn a_fingerprint_is_the_sha256_of_the_der() {
        // base64("abc") inside a certificate envelope.
        let pem = "-----BEGIN CERTIFICATE-----\nYWJj\n-----END CERTIFICATE-----\n";
        assert_eq!(
            fingerprint_of_pem(pem).unwrap(),
            "BA:78:16:BF:8F:01:CF:EA:41:41:40:DE:5D:AE:22:23:B0:03:61:A3:96:17:7A:9C:B4:10:FF:61:F2:00:15:AD"
        );
        assert!(fingerprint_of_pem("no certificate here").is_none());
    }
}
