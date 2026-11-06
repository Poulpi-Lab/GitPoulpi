//! Downloading a release archive and proving it is the one the release
//! published.
//!
//! Refusals, each a typed error and a test:
//! - the asset's name is not `archive_name(version, target)` (a release that
//!   lists another file under the expected URL is not trusted);
//! - the release carries no `.minisig` for the archive, or the signature is
//!   not by a compiled-in release key, or it names another file or version
//!   (checked before the checksum: the checksum travels with the release, the
//!   signature is what proves the release is ours);
//! - the release carries no `.sha256` asset;
//! - the checksum file does not name this archive, or its hash differs from
//!   the downloaded bytes;
//! - a URL on a host outside `gitpoulpi_hosting::updates::DOWNLOAD_HOSTS`
//!   (the transport also checks every redirect hop).

use std::sync::atomic::{AtomicBool, Ordering};

use gitpoulpi_config::release;
use gitpoulpi_hosting::http::{HttpRequest, HttpTransport};
use gitpoulpi_hosting::updates::{download_host_allowed_with, Asset, Release};
use sha2::{Digest, Sha256};

use crate::signing::{self, PublicKey};
use crate::{Result, UpdateError};

/// What the caller learns while the archive comes down.
pub type OnProgress<'a> = &'a (dyn Fn(u64, Option<u64>) + Send + Sync);

/// Lower-case hex SHA-256 of `data`.
pub fn sha256_hex(data: &[u8]) -> String {
    Sha256::digest(data)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// The hash in a `sha256sum`-style file (`<hex>  <name>`): the first line's
/// hash, which must be 64 hex digits, and its name (when given) must be
/// `archive`.
pub fn parse_checksum(text: &str, archive: &str) -> Result<String> {
    let line = text
        .lines()
        .map(str::trim)
        .find(|l| !l.is_empty())
        .ok_or(UpdateError::NoChecksum)?;
    let mut parts = line.split_whitespace();
    let hash = parts.next().unwrap_or("").to_ascii_lowercase();
    if hash.len() != 64 || !hash.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(UpdateError::NoChecksum);
    }
    if let Some(name) = parts.next() {
        let name = name.trim_start_matches('*');
        if name != archive {
            return Err(UpdateError::WrongAsset {
                expected: archive.to_string(),
                found: name.to_string(),
            });
        }
    }
    Ok(hash)
}

/// `asset` is the expected file on an allowed host: GitHub's, or one of
/// `extra_hosts` (the policy's `download_base` mirror).
fn check_asset(asset: &Asset, expected: &str, extra_hosts: &[String]) -> Result<()> {
    if asset.name != expected {
        return Err(UpdateError::WrongAsset {
            expected: expected.to_string(),
            found: asset.name.clone(),
        });
    }
    if !download_host_allowed_with(&asset.url, extra_hosts) {
        return Err(UpdateError::Download(format!(
            "{} is not a GitHub release host",
            asset.url
        )));
    }
    Ok(())
}

/// The signature file of the asset at `url`: its sibling with `.minisig`
/// added (a release page, a mirror and `latest.json` all publish it there).
pub fn minisig_url(url: &str) -> String {
    format!("{url}.minisig")
}

/// Downloads `release`'s archive for `target` and verifies it against the
/// release's `.minisig` (made by one of `keys`) and its `.sha256`. `cancel`
/// set to `true` stops it (the result is [`UpdateError::Canceled`]). Returns
/// the verified archive bytes.
pub async fn download_verified(
    transport: &dyn HttpTransport,
    release: &Release,
    target: &str,
    cancel: &AtomicBool,
    on_progress: OnProgress<'_>,
    keys: &[PublicKey],
) -> Result<Vec<u8>> {
    download_verified_with(transport, release, target, cancel, on_progress, &[], keys).await
}

/// [`download_verified`] that also accepts assets on `extra_hosts`: the
/// update policy's `download_base` mirror. Nothing else widens the hosts.
pub async fn download_verified_with(
    transport: &dyn HttpTransport,
    release: &Release,
    target: &str,
    cancel: &AtomicBool,
    on_progress: OnProgress<'_>,
    extra_hosts: &[String],
    keys: &[PublicKey],
) -> Result<Vec<u8>> {
    let archive_name = release::archive_name(&release.version, target);
    let asset = release.asset.as_ref().ok_or_else(|| {
        UpdateError::Download(format!("this release has no {archive_name} to download"))
    })?;
    let checksum = release.checksum.as_ref().ok_or(UpdateError::NoChecksum)?;
    check_asset(asset, &archive_name, extra_hosts)?;
    check_asset(checksum, &format!("{archive_name}.sha256"), extra_hosts)?;

    let flag = |_: u64, _: Option<u64>| !cancel.load(Ordering::SeqCst);
    // The checksum first: a missing or odd one stops before the big download.
    let text = transport
        .download(HttpRequest::get(checksum.url.clone()), &flag)
        .await
        .map_err(|e| failed(e, cancel))?;
    let expected = parse_checksum(&String::from_utf8_lossy(&text), &archive_name)?;
    // The signature next, for the same reason: a release without one stops
    // before the big download.
    let signature = Asset {
        name: format!("{archive_name}.minisig"),
        url: minisig_url(&asset.url),
        size: 0,
    };
    check_asset(&signature, &format!("{archive_name}.minisig"), extra_hosts)?;
    let sig_text = transport
        .download(HttpRequest::get(signature.url.clone()), &flag)
        .await
        .map_err(|e| match failed(e, cancel) {
            UpdateError::Canceled => UpdateError::Canceled,
            // A 404 (or any answer that is not the file) is no signature.
            _ => UpdateError::NoSignature,
        })?;

    let progress = |done: u64, total: Option<u64>| {
        on_progress(done, total.or((asset.size > 0).then_some(asset.size)));
        !cancel.load(Ordering::SeqCst)
    };
    let bytes = transport
        .download(HttpRequest::get(asset.url.clone()), &progress)
        .await
        .map_err(|e| failed(e, cancel))?;
    signing::verify(
        &bytes,
        &String::from_utf8_lossy(&sig_text),
        &archive_name,
        &release.version,
        keys,
    )?;
    let actual = sha256_hex(&bytes);
    if actual != expected {
        return Err(UpdateError::ChecksumMismatch { expected, actual });
    }
    Ok(bytes)
}

fn failed(e: gitpoulpi_hosting::Error, cancel: &AtomicBool) -> UpdateError {
    if cancel.load(Ordering::SeqCst) {
        UpdateError::Canceled
    } else {
        UpdateError::Download(e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::{archive_for, release_for};
    use gitpoulpi_hosting::http::RecordedTransport;
    use gitpoulpi_testkit::signing::TestSigner;
    use std::sync::atomic::AtomicU64;

    const LINUX: &str = "x86_64-unknown-linux-gnu";

    fn signer() -> &'static TestSigner {
        static SIGNER: std::sync::OnceLock<TestSigner> = std::sync::OnceLock::new();
        SIGNER.get_or_init(TestSigner::new)
    }

    fn keys() -> Vec<PublicKey> {
        vec![gitpoulpi_hosting::updates::signing::parse_key(&signer().public_key_text()).unwrap()]
    }

    /// The `.minisig` of `bytes` as the release workflow writes it.
    fn sig_of(bytes: &[u8]) -> String {
        signer().sign(&release::archive_name("0.2.0", LINUX), "0.2.0", bytes)
    }

    fn run(
        transport: &RecordedTransport,
        release: &Release,
        cancel: &AtomicBool,
    ) -> Result<Vec<u8>> {
        tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap()
            .block_on(download_verified(
                transport,
                release,
                LINUX,
                cancel,
                &|_, _| {},
                &keys(),
            ))
    }

    fn served(bytes: &[u8], sum_text: &str) -> RecordedTransport {
        served_signed(bytes, sum_text, &sig_of(bytes))
    }

    fn served_signed(bytes: &[u8], sum_text: &str, sig: &str) -> RecordedTransport {
        RecordedTransport::new()
            .on_file("/a.tar.gz.sha256", sum_text.as_bytes().to_vec())
            .on_file("/a.tar.gz.minisig", sig.as_bytes().to_vec())
            .on_file("/a.tar.gz", bytes.to_vec())
    }

    #[test]
    fn a_matching_checksum_returns_the_archive_and_reports_progress() {
        let bytes = archive_for("0.2.0", LINUX);
        let name = release::archive_name("0.2.0", LINUX);
        let sum = format!("{}  {name}\n", sha256_hex(&bytes));
        let transport = served(&bytes, &sum);
        let release = release_for(
            "0.2.0",
            LINUX,
            "https://github.com/o/r/releases/download/v/a.tar.gz",
        );
        let seen = AtomicU64::new(0);
        let got = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap()
            .block_on(download_verified(
                &transport,
                &release,
                LINUX,
                &AtomicBool::new(false),
                &|done, _| seen.store(done, Ordering::SeqCst),
                &keys(),
            ))
            .unwrap();
        assert_eq!(got, bytes);
        assert_eq!(seen.load(Ordering::SeqCst), bytes.len() as u64);
    }

    #[test]
    fn a_mismatching_checksum_is_refused() {
        let bytes = archive_for("0.2.0", LINUX);
        let name = release::archive_name("0.2.0", LINUX);
        let sum = format!("{}  {name}\n", sha256_hex(b"something else"));
        let release = release_for(
            "0.2.0",
            LINUX,
            "https://github.com/o/r/releases/download/v/a.tar.gz",
        );
        let err = run(&served(&bytes, &sum), &release, &AtomicBool::new(false)).unwrap_err();
        assert!(
            matches!(err, UpdateError::ChecksumMismatch { .. }),
            "{err:?}"
        );
    }

    #[test]
    fn an_unsigned_release_is_refused() {
        let bytes = archive_for("0.2.0", LINUX);
        let name = release::archive_name("0.2.0", LINUX);
        let sum = format!("{}  {name}\n", sha256_hex(&bytes));
        let release = || {
            release_for(
                "0.2.0",
                LINUX,
                "https://github.com/o/r/releases/download/v/a.tar.gz",
            )
        };
        // No .minisig on the host at all (the fake serves by URL fragment, so
        // the archive's entry also answers the `.minisig` URL with the
        // archive bytes: not a signature either).
        let t = RecordedTransport::new()
            .on_file("/a.tar.gz.sha256", sum.clone().into_bytes())
            .on_file("/a.tar.gz", bytes.clone());
        let err = run(&t, &release(), &AtomicBool::new(false)).unwrap_err();
        assert_eq!(err, UpdateError::NoSignature);
        // A .minisig that is not a signature (an error page).
        let t = served_signed(&bytes, &sum, "<html>404</html>");
        let err = run(&t, &release(), &AtomicBool::new(false)).unwrap_err();
        assert_eq!(err, UpdateError::NoSignature);
    }

    #[test]
    fn a_bad_signature_is_refused_even_when_the_checksum_matches() {
        let bytes = archive_for("0.2.0", LINUX);
        let name = release::archive_name("0.2.0", LINUX);
        let sum = format!("{}  {name}\n", sha256_hex(&bytes));
        let release = || {
            release_for(
                "0.2.0",
                LINUX,
                "https://github.com/o/r/releases/download/v/a.tar.gz",
            )
        };
        // Signed for other bytes (a swapped archive with its own checksum).
        let t = served_signed(&bytes, &sum, &sig_of(b"the real archive"));
        let err = run(&t, &release(), &AtomicBool::new(false)).unwrap_err();
        assert_eq!(err, UpdateError::BadSignature);
        // Signed by a key that is not ours.
        let stranger = TestSigner::new().sign(&name, "0.2.0", &bytes);
        let t = served_signed(&bytes, &sum, &stranger);
        let err = run(&t, &release(), &AtomicBool::new(false)).unwrap_err();
        assert_eq!(err, UpdateError::BadSignature);
    }

    #[test]
    fn a_signature_for_another_file_or_version_is_refused() {
        let bytes = archive_for("0.2.0", LINUX);
        let name = release::archive_name("0.2.0", LINUX);
        let sum = format!("{}  {name}\n", sha256_hex(&bytes));
        let release = || {
            release_for(
                "0.2.0",
                LINUX,
                "https://github.com/o/r/releases/download/v/a.tar.gz",
            )
        };
        for (file, version) in [
            ("gitpoulpi-0.2.0-aarch64-apple-darwin.tar.gz", "0.2.0"),
            (name.as_str(), "0.1.0"),
        ] {
            let sig = signer().sign(file, version, &bytes);
            let t = served_signed(&bytes, &sum, &sig);
            let err = run(&t, &release(), &AtomicBool::new(false)).unwrap_err();
            assert!(
                matches!(err, UpdateError::SignatureForOtherFile { .. }),
                "{file} {version}: {err:?}"
            );
        }
    }

    #[test]
    fn the_signature_is_checked_before_the_checksum() {
        // Both are wrong: the signature refusal is the one reported.
        let bytes = archive_for("0.2.0", LINUX);
        let name = release::archive_name("0.2.0", LINUX);
        let sum = format!("{}  {name}\n", sha256_hex(b"something else"));
        let release = release_for(
            "0.2.0",
            LINUX,
            "https://github.com/o/r/releases/download/v/a.tar.gz",
        );
        let t = served_signed(&bytes, &sum, &sig_of(b"not these bytes"));
        let err = run(&t, &release, &AtomicBool::new(false)).unwrap_err();
        assert_eq!(err, UpdateError::BadSignature);
    }

    #[test]
    fn a_missing_checksum_asset_is_refused_before_any_download() {
        let bytes = archive_for("0.2.0", LINUX);
        let mut release = release_for(
            "0.2.0",
            LINUX,
            "https://github.com/o/r/releases/download/v/a.tar.gz",
        );
        release.checksum = None;
        let transport = served(&bytes, "");
        let err = run(&transport, &release, &AtomicBool::new(false)).unwrap_err();
        assert_eq!(err, UpdateError::NoChecksum);
        assert!(transport.requests().is_empty(), "nothing was downloaded");
        // A checksum file that is not a hash is as good as missing.
        let release = release_for(
            "0.2.0",
            LINUX,
            "https://github.com/o/r/releases/download/v/a.tar.gz",
        );
        let err = run(
            &served(&bytes, "<html>404</html>"),
            &release,
            &AtomicBool::new(false),
        )
        .unwrap_err();
        assert_eq!(err, UpdateError::NoChecksum);
    }

    #[test]
    fn an_asset_with_the_wrong_name_is_refused() {
        let bytes = archive_for("0.2.0", LINUX);
        let mut release = release_for(
            "0.2.0",
            LINUX,
            "https://github.com/o/r/releases/download/v/a.tar.gz",
        );
        release.asset.as_mut().unwrap().name = "gitpoulpi-0.2.0-aarch64-apple-darwin.tar.gz".into();
        let transport = served(&bytes, "");
        let err = run(&transport, &release, &AtomicBool::new(false)).unwrap_err();
        assert!(matches!(err, UpdateError::WrongAsset { .. }), "{err:?}");
        assert!(transport.requests().is_empty());
        // The checksum file naming another archive is refused too.
        let release = release_for(
            "0.2.0",
            LINUX,
            "https://github.com/o/r/releases/download/v/a.tar.gz",
        );
        let sum = format!("{}  other.tar.gz\n", sha256_hex(&bytes));
        let err = run(&served(&bytes, &sum), &release, &AtomicBool::new(false)).unwrap_err();
        assert!(matches!(err, UpdateError::WrongAsset { .. }), "{err:?}");
    }

    #[test]
    fn a_host_outside_github_is_refused() {
        let bytes = archive_for("0.2.0", LINUX);
        let release = release_for("0.2.0", LINUX, "https://evil.example/a.tar.gz");
        let transport = served(&bytes, "");
        let err = run(&transport, &release, &AtomicBool::new(false)).unwrap_err();
        assert!(matches!(err, UpdateError::Download(_)), "{err:?}");
        assert!(transport.requests().is_empty());
    }

    #[test]
    fn a_policy_mirror_is_allowed_only_when_it_is_given() {
        let bytes = archive_for("0.2.0", LINUX);
        let name = release::archive_name("0.2.0", LINUX);
        let sum = format!("{}  {name}\n", sha256_hex(&bytes));
        let release = release_for("0.2.0", LINUX, "https://mirror.example/gp/a.tar.gz");
        let mirror = vec!["https://mirror.example/gp/".to_string()];
        let go = |hosts: &[String]| {
            tokio::runtime::Builder::new_current_thread()
                .build()
                .unwrap()
                .block_on(download_verified_with(
                    &served(&bytes, &sum),
                    &release,
                    LINUX,
                    &AtomicBool::new(false),
                    &|_, _| {},
                    hosts,
                    &keys(),
                ))
        };
        assert!(matches!(go(&[]).unwrap_err(), UpdateError::Download(_)));
        assert_eq!(go(&mirror).unwrap(), bytes);
        let other = vec!["https://other.example/".to_string()];
        assert!(matches!(go(&other).unwrap_err(), UpdateError::Download(_)));
    }

    #[test]
    fn cancel_stops_the_download() {
        let bytes = vec![7u8; 300 * 1024];
        let name = release::archive_name("0.2.0", LINUX);
        let sum = format!("{}  {name}\n", sha256_hex(&bytes));
        let release = release_for(
            "0.2.0",
            LINUX,
            "https://github.com/o/r/releases/download/v/a.tar.gz",
        );
        let cancel = AtomicBool::new(false);
        let transport = served(&bytes, &sum);
        let got = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap()
            .block_on(download_verified(
                &transport,
                &release,
                LINUX,
                &cancel,
                &|_, _| cancel.store(true, Ordering::SeqCst),
                &keys(),
            ));
        assert_eq!(got.unwrap_err(), UpdateError::Canceled);
    }
}
