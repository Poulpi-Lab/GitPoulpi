//! Security notices (the security-hardening design, item 7): the signed
//! `security.json` a release publishes, and which of its notices name the
//! running version.
//!
//! Traps:
//! - A notice that is unsigned, badly signed or signed for another file is
//!   never shown ([`fetch`] returns an error the caller logs). The notices come
//!   from the same place as the releases, so without the signature a hijacked
//!   mirror could put any text, and any link, in front of the user.
//! - `security.json` is republished for the latest release without a build,
//!   so its trusted comment names the latest version, not the running one: it
//!   must be at least the running version (an old copy is a replay).
//! - The `affected` range is a semver requirement. A pre-release build is
//!   matched on its core version too (`1.4.3-rc.1` is checked as `1.4.3`),
//!   because the semver crate otherwise never matches a pre-release against a
//!   range without one.
//! - With update checks off or the `None` policy nothing is asked at all
//!   ([`super::sources::SourceSpec::notices_url`] is `None`).

use serde::{Deserialize, Serialize};

use super::signing::{self, PublicKey, VersionRule};
use crate::http::{HttpRequest, HttpTransport};
use crate::{Error, Result};

/// The file name of the notices (and of the signature base: `security.json.minisig`).
pub const FILE: &str = "security.json";

/// The notices are a small JSON file; anything larger is not one.
const MAX_BYTES: usize = 256 * 1024;

/// One published notice.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Advisory {
    pub id: String,
    /// A semver range, `>=1.2.0, <1.4.3`.
    pub affected: String,
    /// The release that fixes it.
    #[serde(default)]
    pub fixed: Option<String>,
    /// `low`, `medium`, `high` or `critical`.
    pub severity: String,
    pub url: String,
    pub summary: String,
}

impl Advisory {
    /// 0 (low) to 3 (critical); an unknown word ranks with `low`.
    pub fn severity_rank(&self) -> u8 {
        match self.severity.trim().to_ascii_lowercase().as_str() {
            "critical" => 3,
            "high" => 2,
            "medium" => 1,
            _ => 0,
        }
    }

    /// Whether `running` is inside the `affected` range.
    pub fn affects(&self, running: &str) -> bool {
        let Ok(req) = semver::VersionReq::parse(&self.affected) else {
            return false;
        };
        let Ok(version) = semver::Version::parse(running.trim().trim_start_matches(['v', 'V']))
        else {
            return false;
        };
        let mut plain = version.clone();
        plain.build = semver::BuildMetadata::EMPTY;
        if req.matches(&plain) {
            return true;
        }
        if !plain.pre.is_empty() {
            plain.pre = semver::Prerelease::EMPTY;
            return req.matches(&plain);
        }
        false
    }

    /// The notice's link, only when it is an `https` URL (the file is signed,
    /// but a link that opens something else is never worth the click).
    pub fn safe_url(&self) -> Option<&str> {
        let url = self.url.trim();
        (url.starts_with("https://")
            && url.len() > "https://".len()
            && !url.contains(char::is_whitespace))
        .then_some(url)
    }
}

/// The notices that name `running`, most severe first.
pub fn affecting(advisories: &[Advisory], running: &str) -> Vec<Advisory> {
    let mut hit: Vec<Advisory> = advisories
        .iter()
        .filter(|a| a.affects(running))
        .cloned()
        .collect();
    hit.sort_by(|a, b| {
        b.severity_rank()
            .cmp(&a.severity_rank())
            .then_with(|| a.id.cmp(&b.id))
    });
    hit
}

#[derive(Deserialize)]
struct File {
    #[serde(default)]
    advisories: Vec<Advisory>,
}

/// Reads the body of a verified `security.json`.
pub fn parse(body: &[u8]) -> Result<Vec<Advisory>> {
    let file: File = serde_json::from_slice(body).map_err(|e| Error::Decode(e.to_string()))?;
    Ok(file.advisories)
}

/// Downloads `url` (the notices) and `url.minisig`, verifies the signature
/// with `keys` and returns the notices. `running` is the floor of the version
/// the signature must name.
pub async fn fetch(
    transport: &dyn HttpTransport,
    url: &str,
    running: &str,
    keys: &[PublicKey],
) -> Result<Vec<Advisory>> {
    let go = |_: u64, _: Option<u64>| true;
    let body = transport
        .download(HttpRequest::get(url.to_string()), &go)
        .await?;
    if body.len() > MAX_BYTES {
        return Err(Error::Decode("the security notices are too large".into()));
    }
    let sig = transport
        .download(HttpRequest::get(format!("{url}.minisig")), &go)
        .await
        .map_err(|e| Error::Decode(format!("the security notices have no signature: {e}")))?;
    signing::verify_with(
        &body,
        &String::from_utf8_lossy(&sig),
        FILE,
        VersionRule::NotOlderThan(running),
        keys,
    )
    .map_err(|e| Error::Decode(format!("the security notices are not trusted: {e}")))?;
    parse(&body)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::http::RecordedTransport;
    use gitpoulpi_testkit::signing::TestSigner;

    fn advisory(id: &str, affected: &str, severity: &str) -> Advisory {
        Advisory {
            id: id.into(),
            affected: affected.into(),
            fixed: Some("1.4.3".into()),
            severity: severity.into(),
            url: "https://example.test/a".into(),
            summary: "s".into(),
        }
    }

    fn block<F: std::future::Future>(f: F) -> F::Output {
        tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap()
            .block_on(f)
    }

    #[test]
    fn a_range_matches_the_versions_inside_it_only() {
        let a = advisory("GPSA-2026-001", ">=1.2.0, <1.4.3", "high");
        for (v, hit) in [
            ("1.1.9", false),
            ("1.2.0", true),
            ("1.4.2", true),
            ("v1.4.2", true),
            ("1.4.3", false),
            ("2.0.0", false),
            ("1.4.2+build5", true),
            ("1.3.0-rc.1", true),
            ("not a version", false),
        ] {
            assert_eq!(a.affects(v), hit, "{v}");
        }
        assert!(!advisory("x", "garbage", "low").affects("1.0.0"));
    }

    #[test]
    fn matching_notices_are_listed_most_severe_first() {
        let list = vec![
            advisory("GPSA-2026-001", "<2.0.0", "low"),
            advisory("GPSA-2026-002", "<2.0.0", "critical"),
            advisory("GPSA-2026-003", ">=3.0.0", "critical"),
            advisory("GPSA-2026-004", "<2.0.0", "high"),
        ];
        let ids: Vec<String> = affecting(&list, "1.0.0")
            .into_iter()
            .map(|a| a.id)
            .collect();
        assert_eq!(ids, ["GPSA-2026-002", "GPSA-2026-004", "GPSA-2026-001"]);
        assert!(affecting(&list, "2.5.0").is_empty());
    }

    #[test]
    fn only_https_links_are_offered() {
        let mut a = advisory("x", "*", "low");
        assert!(a.safe_url().is_some());
        a.url = "javascript:alert(1)".into();
        assert!(a.safe_url().is_none());
        a.url = "http://example.test".into();
        assert!(a.safe_url().is_none());
    }

    const BODY: &str = r#"{"schema":1,"advisories":[{"id":"GPSA-2026-001","affected":"<2.0.0",
        "fixed":"2.0.0","severity":"high","url":"https://example.test/a","summary":"bad"}]}"#;

    fn served(signer: &TestSigner, signed_for: &str) -> RecordedTransport {
        RecordedTransport::new()
            .on_file(
                "security.json.minisig",
                signer.sign(FILE, signed_for, BODY.as_bytes()).into_bytes(),
            )
            .on_file("security.json", BODY.as_bytes().to_vec())
    }

    #[test]
    fn a_signed_file_is_read() {
        let signer = TestSigner::new();
        let keys = vec![signing::parse_key(&signer.public_key_text()).unwrap()];
        let got = block(fetch(
            &served(&signer, "1.5.0"),
            "https://x.example/security.json",
            "1.4.0",
            &keys,
        ))
        .unwrap();
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].id, "GPSA-2026-001");
    }

    #[test]
    fn an_unsigned_badly_signed_or_replayed_file_is_never_read() {
        let signer = TestSigner::new();
        let keys = vec![signing::parse_key(&signer.public_key_text()).unwrap()];
        let url = "https://x.example/security.json";
        // No signature file at all.
        let t = RecordedTransport::new().on_file("security.json", BODY.as_bytes().to_vec());
        assert!(block(fetch(&t, url, "1.4.0", &keys)).is_err());
        // Signed by another key.
        let stranger = TestSigner::new();
        assert!(block(fetch(&served(&stranger, "1.5.0"), url, "1.4.0", &keys)).is_err());
        // A copy signed for an older release than the running one.
        assert!(block(fetch(&served(&signer, "1.3.0"), url, "1.4.0", &keys)).is_err());
        // Tampered after signing.
        let t = RecordedTransport::new()
            .on_file(
                "security.json.minisig",
                signer.sign(FILE, "1.5.0", BODY.as_bytes()).into_bytes(),
            )
            .on_file("security.json", br#"{"advisories":[]}"#.to_vec());
        assert!(block(fetch(&t, url, "1.4.0", &keys)).is_err());
    }
}
