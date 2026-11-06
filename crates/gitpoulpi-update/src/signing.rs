//! Release signatures on the client (the security-hardening design, item 4):
//! the compiled-in release keys and the check an asset goes through before its
//! hash is even looked at.
//!
//! `xtask sign` (release workflow) signs every asset with the project key and
//! writes `<asset>.minisig` with the trusted comment `file:<name>
//! version:<v>`. [`verify`] accepts an asset only when a compiled-in key made
//! the signature **and** that comment names this file and this version.
//!
//! Traps:
//! - [`release_keys`] is the only production source of keys. It is a list so a
//!   rotation can ship the next key before the old one retires. Nothing reads
//!   a key from the environment or a config file; tests pass their own keys to
//!   [`verify`] and to the code above it. The one exception is a debug-build
//!   hook for the PTY tests, compiled out of every release build.
//! - An older, unsigned release can no longer be installed through the
//!   updater: pinning or going back to one is refused.

pub use gitpoulpi_hosting::updates::signing::{PublicKey, SignError, VersionRule};

use crate::UpdateError;

/// The public key files compiled in, current first. `packaging/minisign.pub`
/// is the project's key (id `B461FFCBF6F6A4F2`).
const KEY_FILES: &[&str] = &[include_str!("../../../packaging/minisign.pub")];

/// The debug-build override for the PTY tests (the test builds a key pair and
/// signs a fake release with it). Release and dist builds do not contain it.
#[cfg(debug_assertions)]
pub const TEST_KEY_ENV: &str = "GITPOULPI_TEST_RELEASE_KEY";

/// The release keys the client trusts.
pub fn release_keys() -> Vec<PublicKey> {
    #[cfg(debug_assertions)]
    if let Some(text) = std::env::var(TEST_KEY_ENV).ok().filter(|t| !t.is_empty()) {
        if let Some(key) = gitpoulpi_hosting::updates::signing::parse_key(&text) {
            return vec![key];
        }
    }
    KEY_FILES
        .iter()
        .filter_map(|text| gitpoulpi_hosting::updates::signing::parse_key(text))
        .collect()
}

impl From<SignError> for UpdateError {
    fn from(e: SignError) -> Self {
        match e {
            SignError::NoSignature => UpdateError::NoSignature,
            SignError::BadSignature => UpdateError::BadSignature,
            SignError::OtherFile { expected, found } => {
                UpdateError::SignatureForOtherFile { expected, found }
            }
        }
    }
}

/// Checks `bytes` (the asset called `name` of release `version`) against the
/// `.minisig` text `sig_text` with any of `keys`.
pub fn verify(
    bytes: &[u8],
    sig_text: &str,
    name: &str,
    version: &str,
    keys: &[PublicKey],
) -> Result<(), UpdateError> {
    gitpoulpi_hosting::updates::signing::verify(bytes, sig_text, name, version, keys)
        .map_err(UpdateError::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use gitpoulpi_testkit::signing::TestSigner;

    fn keys_of(signer: &TestSigner) -> Vec<PublicKey> {
        vec![gitpoulpi_hosting::updates::signing::parse_key(&signer.public_key_text()).unwrap()]
    }

    #[test]
    fn the_compiled_release_key_parses() {
        let keys = release_keys();
        assert_eq!(keys.len(), KEY_FILES.len(), "every compiled key parses");
        assert!(!keys.is_empty());
        // The project's key: a signature by a stranger is not accepted by it.
        let stranger = TestSigner::new();
        let sig = stranger.sign("a", "1.0.0", b"x");
        assert_eq!(
            verify(b"x", &sig, "a", "1.0.0", &keys),
            Err(UpdateError::BadSignature)
        );
    }

    #[test]
    fn the_compiled_key_is_the_projects() {
        assert!(KEY_FILES[0].contains("B461FFCBF6F6A4F2"));
    }

    #[test]
    fn the_happy_path_and_each_refusal_map_to_their_error() {
        let signer = TestSigner::new();
        let keys = keys_of(&signer);
        let sig = signer.sign("a.tar.gz", "1.2.3", b"bytes");
        assert_eq!(verify(b"bytes", &sig, "a.tar.gz", "1.2.3", &keys), Ok(()));
        assert_eq!(
            verify(b"bytes", "", "a.tar.gz", "1.2.3", &keys),
            Err(UpdateError::NoSignature)
        );
        assert_eq!(
            verify(b"other", &sig, "a.tar.gz", "1.2.3", &keys),
            Err(UpdateError::BadSignature)
        );
        for (name, version) in [("b.tar.gz", "1.2.3"), ("a.tar.gz", "1.2.2")] {
            let err = verify(b"bytes", &sig, name, version, &keys).unwrap_err();
            assert!(
                matches!(err, UpdateError::SignatureForOtherFile { .. }),
                "{name} {version}: {err:?}"
            );
        }
    }
}
