//! Release signatures (the security-hardening design, item 4): the check every
//! file GitPoulpi fetches from a release goes through before it is trusted.
//!
//! The release workflow signs each asset with minisign and writes
//! `<asset>.minisig` next to it, with the trusted comment
//! `file:<name> version:<v>`. [`verify`] checks the signature against the
//! compiled-in keys **and** that comment against the file name and version
//! the caller expects, so a valid signature cannot be moved onto another file
//! or onto an older release.
//!
//! This lives in the hosting crate (and `gitpoulpi-update::signing` re-wraps
//! it) because the manifest and the security notices are verified here, below
//! the update engine. It never reads a key from the environment: callers pass
//! the keys in, and production passes `gitpoulpi_update::signing::release_keys()`.

pub use minisign_verify::PublicKey;
use minisign_verify::Signature;

/// Why a signature was refused.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SignError {
    /// No signature file, or one that is not a minisign signature.
    #[error("the release has no valid signature file, so it is not trusted")]
    NoSignature,
    /// The signature does not match the bytes, or no accepted key made it.
    #[error("the signature does not match (not signed by a GitPoulpi release key): not trusted")]
    BadSignature,
    /// A valid signature, but for another file or another version.
    #[error("the signature is for {found}, expected {expected}: not trusted")]
    OtherFile { expected: String, found: String },
}

/// The trusted comment a release signature carries for `name` at `version`.
pub fn trusted_comment(name: &str, version: &str) -> String {
    format!("file:{name} version:{version}")
}

/// What the trusted comment must say about the version.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VersionRule<'a> {
    /// Exactly this version (an archive, the manifest).
    Exactly(&'a str),
    /// A version that is not older than this one (`security.json`, which is
    /// republished for the latest release: an old copy must not replace it).
    NotOlderThan(&'a str),
}

/// Parses the text of a public key file (`untrusted comment: ...` and the
/// base64 line).
pub fn parse_key(text: &str) -> Option<PublicKey> {
    PublicKey::decode(text).ok()
}

/// The version a trusted comment names.
fn comment_version(comment: &str, name: &str) -> Result<String, SignError> {
    let other = |found: &str| SignError::OtherFile {
        expected: format!("file:{name}"),
        found: found.to_string(),
    };
    let mut file = None;
    let mut version = None;
    for part in comment.split_whitespace() {
        if let Some(f) = part.strip_prefix("file:") {
            file = Some(f);
        } else if let Some(v) = part.strip_prefix("version:") {
            version = Some(v);
        }
    }
    match (file, version) {
        (Some(f), Some(v)) if f == name => Ok(v.to_string()),
        _ => Err(other(comment)),
    }
}

/// Verifies `bytes` against the `.minisig` text `sig_text` with any of `keys`,
/// and the signature's trusted comment against `name` and `rule`.
pub fn verify_with(
    bytes: &[u8],
    sig_text: &str,
    name: &str,
    rule: VersionRule<'_>,
    keys: &[PublicKey],
) -> Result<(), SignError> {
    let signature = Signature::decode(sig_text).map_err(|_| SignError::NoSignature)?;
    if !keys
        .iter()
        .any(|key| key.verify(bytes, &signature, false).is_ok())
    {
        return Err(SignError::BadSignature);
    }
    // The comment is covered by the signature (checked above), so it can be
    // trusted now: it must name this file and the version asked for.
    let comment = signature.trusted_comment();
    let found = comment_version(comment, name)?;
    let fits = match rule {
        VersionRule::Exactly(want) => same_version(&found, want),
        VersionRule::NotOlderThan(floor) => {
            match (super::Version::parse(&found), super::Version::parse(floor)) {
                (Some(found), Some(floor)) => found >= floor,
                _ => false,
            }
        }
    };
    if fits {
        Ok(())
    } else {
        Err(SignError::OtherFile {
            expected: match rule {
                VersionRule::Exactly(v) => trusted_comment(name, v),
                VersionRule::NotOlderThan(v) => format!("file:{name} version:{v} or newer"),
            },
            found: comment.to_string(),
        })
    }
}

/// [`verify_with`] for one exact version.
pub fn verify(
    bytes: &[u8],
    sig_text: &str,
    name: &str,
    version: &str,
    keys: &[PublicKey],
) -> Result<(), SignError> {
    verify_with(bytes, sig_text, name, VersionRule::Exactly(version), keys)
}

fn same_version(a: &str, b: &str) -> bool {
    let trim = |v: &str| v.trim().trim_start_matches(['v', 'V']).to_string();
    trim(a) == trim(b)
}

#[cfg(test)]
mod tests {
    use super::*;
    use gitpoulpi_testkit::signing::TestSigner;

    fn keys(signer: &TestSigner) -> Vec<PublicKey> {
        vec![parse_key(&signer.public_key_text()).expect("the test key parses")]
    }

    #[test]
    fn a_signature_for_this_file_and_version_is_accepted() {
        let signer = TestSigner::new();
        let sig = signer.sign("a.tar.gz", "1.2.3", b"bytes");
        assert_eq!(
            verify(b"bytes", &sig, "a.tar.gz", "1.2.3", &keys(&signer)),
            Ok(())
        );
        // A leading `v` on the expected version is the same version.
        assert_eq!(
            verify(b"bytes", &sig, "a.tar.gz", "v1.2.3", &keys(&signer)),
            Ok(())
        );
    }

    #[test]
    fn no_signature_text_is_no_signature() {
        let signer = TestSigner::new();
        for text in [
            "",
            "<html>404</html>",
            "untrusted comment: x\nnot base64!!\n",
        ] {
            assert_eq!(
                verify(b"bytes", text, "a", "1.0.0", &keys(&signer)),
                Err(SignError::NoSignature),
                "{text:?}"
            );
        }
    }

    #[test]
    fn tampered_bytes_and_another_key_are_bad_signatures() {
        let signer = TestSigner::new();
        let sig = signer.sign("a", "1.0.0", b"bytes");
        assert_eq!(
            verify(b"tampered", &sig, "a", "1.0.0", &keys(&signer)),
            Err(SignError::BadSignature)
        );
        let stranger = TestSigner::new();
        assert_eq!(
            verify(b"bytes", &sig, "a", "1.0.0", &keys(&stranger)),
            Err(SignError::BadSignature)
        );
        assert_eq!(
            verify(b"bytes", &sig, "a", "1.0.0", &[]),
            Err(SignError::BadSignature),
            "no keys accept nothing"
        );
    }

    #[test]
    fn a_signature_for_another_file_or_version_is_refused() {
        let signer = TestSigner::new();
        let sig = signer.sign("other.tar.gz", "1.0.0", b"bytes");
        let err = verify(b"bytes", &sig, "a.tar.gz", "1.0.0", &keys(&signer)).unwrap_err();
        assert!(matches!(err, SignError::OtherFile { .. }), "{err:?}");
        let sig = signer.sign("a.tar.gz", "0.9.0", b"bytes");
        let err = verify(b"bytes", &sig, "a.tar.gz", "1.0.0", &keys(&signer)).unwrap_err();
        assert!(matches!(err, SignError::OtherFile { .. }), "{err:?}");
        // No comment at all is not a signature for anything in particular.
        let sig = signer.sign_with_comment("hello", b"bytes");
        let err = verify(b"bytes", &sig, "a.tar.gz", "1.0.0", &keys(&signer)).unwrap_err();
        assert!(matches!(err, SignError::OtherFile { .. }), "{err:?}");
    }

    #[test]
    fn a_rotated_key_list_accepts_either_key() {
        let (old, next) = (TestSigner::new(), TestSigner::new());
        let both = vec![
            parse_key(&old.public_key_text()).unwrap(),
            parse_key(&next.public_key_text()).unwrap(),
        ];
        for signer in [&old, &next] {
            let sig = signer.sign("a", "1.0.0", b"x");
            assert_eq!(verify(b"x", &sig, "a", "1.0.0", &both), Ok(()));
        }
    }

    #[test]
    fn a_notice_signed_for_an_older_release_than_the_floor_is_refused() {
        let signer = TestSigner::new();
        let k = keys(&signer);
        let rule = VersionRule::NotOlderThan("1.4.0");
        for (signed, ok) in [("1.4.0", true), ("1.5.0", true), ("1.3.9", false)] {
            let sig = signer.sign("security.json", signed, b"{}");
            let got = verify_with(b"{}", &sig, "security.json", rule, &k);
            assert_eq!(got.is_ok(), ok, "{signed}: {got:?}");
        }
    }
}
