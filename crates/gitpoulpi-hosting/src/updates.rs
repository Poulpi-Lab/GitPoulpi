//! Looking for a newer GitPoulpi release (the updates-and-install design,
//! slice A).
//!
//! Everything here is pure except [`check`], which asks `GET
//! /repos/<owner>/<name>/releases` through an [`HttpTransport`] (so the tests
//! never open a socket) and [`UpdateState`], the small JSON file that keeps the
//! last answer and the version the user chose to skip.
//!
//! Traps:
//! - `/releases/latest` never returns a pre-release, so the channel cannot be
//!   honoured through it: the list is fetched and filtered here.
//! - Semver, not string, order: `0.10.0` is newer than `0.9.0`, and a
//!   pre-release (`0.2.0-rc.1`) is older than its release (`0.2.0`).
//! - A private repository answers 404 without a token; that is a normal
//!   "no answer", never a user-facing error (the caller logs one line).

use std::cmp::Ordering;
use std::path::Path;

use crate::http::{HttpRequest, HttpTransport};
use crate::{Error, Result};
use gitpoulpi_config::{release, UpdateChannel};

/// Env var that points the check at another API root (tests: a loopback
/// server). Unset: `https://api.github.com`. Test builds refuse every
/// non-loopback host anyway (`http::external_host_refusal`).
pub const API_ENV: &str = "GITPOULPI_UPDATE_API";

/// The API root the check talks to.
pub fn api_base() -> String {
    std::env::var(API_ENV)
        .ok()
        .map(|v| v.trim().trim_end_matches('/').to_string())
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| "https://api.github.com".to_string())
}

/// The hosts a release download may touch, the first request and every
/// redirect: GitHub's release pages and the CDN its asset links redirect to.
/// Anything else is refused (a tampered release JSON must not send the
/// updater to a host of its choosing). Loopback joins them only while
/// [`API_ENV`] is set, which is how tests point the whole flow at a fake
/// server.
pub const DOWNLOAD_HOSTS: &[&str] = &[
    "github.com",
    "api.github.com",
    "objects.githubusercontent.com",
    "release-assets.githubusercontent.com",
    "github-releases.githubusercontent.com",
];

/// Whether `url` is `https` on one of the [`DOWNLOAD_HOSTS`] (or loopback
/// while testing).
pub fn download_host_allowed(url: &str) -> bool {
    let Some((scheme, rest)) = url.split_once("://") else {
        return false;
    };
    let authority = rest.split(['/', '?', '#']).next().unwrap_or("");
    if authority.contains('@') {
        return false;
    }
    let host = authority
        .rsplit_once(':')
        .filter(|(_, port)| port.chars().all(|c| c.is_ascii_digit()))
        .map_or(authority, |(h, _)| h)
        .to_ascii_lowercase();
    if std::env::var_os(API_ENV).is_some_and(|v| !v.is_empty())
        && (host == "127.0.0.1" || host == "localhost" || host == "[::1]")
    {
        return scheme == "http" || scheme == "https";
    }
    scheme == "https" && DOWNLOAD_HOSTS.contains(&host.as_str())
}

/// A semantic version: `major.minor.patch[-pre][+build]`, a leading `v`
/// tolerated. Build metadata is ignored, as semver says.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Version {
    pub major: u64,
    pub minor: u64,
    pub patch: u64,
    /// Dot-separated pre-release identifiers; empty for a full release.
    pub pre: Vec<String>,
}

impl Version {
    pub fn parse(text: &str) -> Option<Self> {
        let text = text.trim().trim_start_matches(['v', 'V']);
        let text = text.split('+').next()?;
        let (core, pre) = match text.split_once('-') {
            Some((core, pre)) => (core, Some(pre)),
            None => (text, None),
        };
        let mut nums = core.split('.');
        let major = nums.next()?.parse().ok()?;
        let minor = nums.next()?.parse().ok()?;
        let patch = nums.next()?.parse().ok()?;
        if nums.next().is_some() {
            return None;
        }
        let pre = match pre {
            Some("") => return None,
            Some(p) => p.split('.').map(str::to_string).collect(),
            None => Vec::new(),
        };
        Some(Self {
            major,
            minor,
            patch,
            pre,
        })
    }

    pub fn is_prerelease(&self) -> bool {
        !self.pre.is_empty()
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.major, self.minor, self.patch)
            .cmp(&(other.major, other.minor, other.patch))
            .then_with(|| match (self.pre.is_empty(), other.pre.is_empty()) {
                (true, true) => Ordering::Equal,
                // A release outranks its own pre-releases.
                (true, false) => Ordering::Greater,
                (false, true) => Ordering::Less,
                (false, false) => compare_pre(&self.pre, &other.pre),
            })
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn compare_pre(a: &[String], b: &[String]) -> Ordering {
    for (x, y) in a.iter().zip(b) {
        let ord = match (x.parse::<u64>(), y.parse::<u64>()) {
            (Ok(x), Ok(y)) => x.cmp(&y),
            // Numeric identifiers rank below alphanumeric ones.
            (Ok(_), Err(_)) => Ordering::Less,
            (Err(_), Ok(_)) => Ordering::Greater,
            (Err(_), Err(_)) => x.cmp(y),
        };
        if ord != Ordering::Equal {
            return ord;
        }
    }
    a.len().cmp(&b.len())
}

/// One downloadable file of a release.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Asset {
    pub name: String,
    pub url: String,
    #[serde(default)]
    pub size: u64,
}

/// A published release, as far as the notice needs it.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Release {
    pub tag: String,
    pub version: String,
    pub prerelease: bool,
    /// The release body (Markdown): what the Release notes tab shows.
    pub notes: String,
    /// The release's page on github.com: what "Download" opens.
    pub page_url: String,
    /// The archive built for this target, when the release carries one.
    pub asset: Option<Asset>,
    /// Its `.sha256` file (used by the self-updater of slice B).
    pub checksum: Option<Asset>,
}

#[derive(serde::Deserialize)]
struct RawAsset {
    name: String,
    browser_download_url: String,
    #[serde(default)]
    size: u64,
}

#[derive(serde::Deserialize)]
struct RawRelease {
    tag_name: String,
    #[serde(default)]
    draft: bool,
    #[serde(default)]
    prerelease: bool,
    #[serde(default)]
    body: Option<String>,
    #[serde(default)]
    html_url: String,
    #[serde(default)]
    assets: Vec<RawAsset>,
}

/// The archive and checksum of `version` for `target` among `assets`.
pub fn pick_assets(
    assets: &[Asset],
    version: &str,
    target: &str,
) -> (Option<Asset>, Option<Asset>) {
    let archive = release::archive_name(version, target);
    let checksum = format!("{archive}.sha256");
    let find = |name: &str| assets.iter().find(|a| a.name == name).cloned();
    (find(&archive), find(&checksum))
}

/// The version a release tag names: `releases/v0.2.0` (this repository's
/// rule), or a bare `v0.2.0` / `0.2.0`. Any other prefix is not a release.
pub fn version_of_tag(tag: &str) -> Option<&str> {
    let tag = tag.trim();
    let bare = tag.strip_prefix("releases/").unwrap_or(tag);
    let version = bare.trim_start_matches(['v', 'V']);
    (!version.contains('/') && Version::parse(version).is_some()).then_some(version)
}

/// Reads the `/releases` answer: drafts and tags that are not a version are
/// dropped. `target` picks the asset.
pub fn parse_releases(body: &str, target: &str) -> Result<Vec<Release>> {
    let raw: Vec<RawRelease> =
        serde_json::from_str(body).map_err(|e| Error::Decode(e.to_string()))?;
    Ok(raw
        .into_iter()
        .filter(|r| !r.draft)
        .filter_map(|r| {
            let version = version_of_tag(&r.tag_name)?.to_string();
            let v = Version::parse(&version)?;
            let assets: Vec<Asset> = r
                .assets
                .into_iter()
                .map(|a| Asset {
                    name: a.name,
                    url: a.browser_download_url,
                    size: a.size,
                })
                .collect();
            let (asset, checksum) = pick_assets(&assets, &version, target);
            Some(Release {
                tag: r.tag_name,
                version,
                prerelease: r.prerelease || v.is_prerelease(),
                notes: r.body.unwrap_or_default(),
                page_url: r.html_url,
                asset,
                checksum,
            })
        })
        .collect())
}

/// The newest release allowed by `channel` that is newer than `running`;
/// `None` when the running build is up to date (or newer: a dev build).
pub fn newest_after(
    releases: &[Release],
    running: &str,
    channel: UpdateChannel,
) -> Option<Release> {
    let running = Version::parse(running)?;
    releases
        .iter()
        .filter(|r| channel == UpdateChannel::Prerelease || !r.prerelease)
        .filter_map(|r| Some((Version::parse(&r.version)?, r)))
        .filter(|(v, _)| *v > running)
        .max_by(|a, b| a.0.cmp(&b.0))
        .map(|(_, r)| r.clone())
}

/// What one check needs.
#[derive(Debug, Clone)]
pub struct CheckParams {
    /// `owner/name`.
    pub repo: String,
    pub running: String,
    pub channel: UpdateChannel,
    pub target: String,
    /// A github.com token, when the user has one stored (a private
    /// repository answers 404 without it).
    pub token: Option<String>,
}

/// Asks the host for the releases and returns the newer one, if any.
pub async fn check(
    transport: &dyn HttpTransport,
    base: &str,
    params: &CheckParams,
) -> Result<Option<Release>> {
    let url = format!(
        "{}/repos/{}/releases?per_page=30",
        base.trim_end_matches('/'),
        params.repo.trim_matches('/')
    );
    let mut request = HttpRequest::get(url).header("Accept", "application/vnd.github+json");
    if let Some(token) = params.token.as_deref().filter(|t| !t.trim().is_empty()) {
        request = request.header("Authorization", format!("Bearer {}", token.trim()));
    }
    let response = transport.send(request).await?;
    response.error_for_status("the update check")?;
    let releases = parse_releases(&response.body, &params.target)?;
    Ok(newest_after(&releases, &params.running, params.channel))
}

/// A day, in seconds: the throttle between two checks.
pub const CHECK_INTERVAL_SECS: u64 = 24 * 60 * 60;

/// What is kept between runs (`<data dir>/updates.json`): when the host was
/// last asked, what it said, and the version the user chose to skip.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct UpdateState {
    /// Unix seconds of the last attempt, successful or not.
    #[serde(default)]
    pub checked_at: u64,
    /// The newest release found, kept for later runs: the chip needs no
    /// request to say "available" again. Compared with the running version
    /// every time it is read ([`UpdateState::available`]).
    #[serde(default)]
    pub latest: Option<Release>,
    /// The version the user chose to skip: no chip for it, a newer one shows.
    #[serde(default)]
    pub skipped: Option<String>,
}

impl UpdateState {
    /// Reads the file; a missing or damaged one is an empty state.
    pub fn load(path: &Path) -> Self {
        std::fs::read_to_string(path)
            .ok()
            .and_then(|raw| serde_json::from_str(&raw).ok())
            .unwrap_or_default()
    }

    /// Writes the file, creating its folder. Best effort for the caller:
    /// losing it only means one more check.
    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let raw = serde_json::to_string_pretty(self).map_err(std::io::Error::other)?;
        std::fs::write(path, raw)
    }

    /// Whether a check is due: never done, or a day old. `force` (the
    /// palette's "Check for updates now") ignores the throttle.
    pub fn due(&self, now: u64, force: bool) -> bool {
        force || self.checked_at == 0 || now.saturating_sub(self.checked_at) >= CHECK_INTERVAL_SECS
    }

    /// The kept release when it is still newer than `running`, in `channel`,
    /// and not the skipped one.
    pub fn available(&self, running: &str, channel: UpdateChannel) -> Option<Release> {
        let latest = self.latest.as_ref()?;
        let newest = newest_after(std::slice::from_ref(latest), running, channel)?;
        (self.skipped.as_deref() != Some(newest.version.as_str())).then_some(newest)
    }

    /// Records an attempt: the answer replaces the kept release only when the
    /// host answered (a failed check keeps what was known).
    pub fn record(&mut self, now: u64, answer: Option<Result<Option<Release>>>) {
        self.checked_at = now;
        if let Some(Ok(latest)) = answer {
            self.latest = latest;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::http::RecordedTransport;

    const TARGET: &str = "x86_64-unknown-linux-gnu";

    fn body() -> String {
        serde_json::json!([
            {"tag_name": "v0.3.0-rc.1", "prerelease": true, "body": "rc notes",
             "html_url": "https://example.test/r/v0.3.0-rc.1", "assets": []},
            {"tag_name": "releases/v0.2.0", "prerelease": false, "body": "## 0.2.0\n- new",
             "html_url": "https://example.test/r/v0.2.0",
             "assets": [
               {"name": "gitpoulpi-0.2.0-x86_64-unknown-linux-gnu.tar.gz",
                "browser_download_url": "https://example.test/d/linux.tar.gz", "size": 10},
               {"name": "gitpoulpi-0.2.0-x86_64-unknown-linux-gnu.tar.gz.sha256",
                "browser_download_url": "https://example.test/d/linux.sha256", "size": 1},
               {"name": "gitpoulpi-0.2.0-x86_64-pc-windows-msvc.zip",
                "browser_download_url": "https://example.test/d/win.zip", "size": 10}]},
            {"tag_name": "v0.9.0", "draft": true, "assets": []},
            {"tag_name": "nightly", "assets": []},
            {"tag_name": "v0.1.0", "assets": []}
        ])
        .to_string()
    }

    fn params(running: &str, channel: UpdateChannel) -> CheckParams {
        CheckParams {
            repo: "Poulpi-Lab/GitPoulpi".into(),
            running: running.into(),
            channel,
            target: TARGET.into(),
            token: None,
        }
    }

    #[test]
    fn versions_compare_as_semver_not_as_text() {
        let v = |s: &str| Version::parse(s).unwrap();
        assert!(v("0.10.0") > v("0.9.0"));
        assert!(v("1.0.0") > v("0.99.99"));
        assert!(v("0.2.0") > v("0.2.0-rc.1"), "a release beats its rc");
        assert!(v("0.2.0-rc.2") > v("0.2.0-rc.1"));
        assert!(v("0.2.0-rc.10") > v("0.2.0-rc.9"), "numeric identifiers");
        assert!(v("0.2.0-rc.1") > v("0.2.0-beta.7"), "alphabetic order");
        assert_eq!(v("v1.2.3+build5"), v("1.2.3"), "build metadata ignored");
        assert_eq!(Version::parse("nightly"), None);
        assert_eq!(Version::parse("1.2"), None);
        assert_eq!(Version::parse("1.2.3.4"), None);
        assert_eq!(Version::parse("1.2.3-"), None);
    }

    #[test]
    fn release_tags_are_prefixed_or_bare_and_other_prefixes_are_not_releases() {
        assert_eq!(version_of_tag("releases/v0.2.0"), Some("0.2.0"));
        assert_eq!(version_of_tag("releases/v1.0.0-rc.1"), Some("1.0.0-rc.1"));
        assert_eq!(version_of_tag("v0.2.0"), Some("0.2.0"));
        assert_eq!(version_of_tag("0.2.0"), Some("0.2.0"));
        assert_eq!(version_of_tag("checkpoints/v0.2.0"), None);
        assert_eq!(version_of_tag("releases/nightly"), None);
        assert_eq!(version_of_tag("nightly"), None);
    }

    #[test]
    fn the_stable_channel_skips_prereleases_and_drafts() {
        let releases = parse_releases(&body(), TARGET).unwrap();
        assert_eq!(releases.len(), 3, "the draft and the non-version tag drop");
        let stable = newest_after(&releases, "0.1.0", UpdateChannel::Stable).unwrap();
        assert_eq!(stable.version, "0.2.0");
        assert!(newest_after(&releases, "0.2.0", UpdateChannel::Stable).is_none());
    }

    #[test]
    fn the_prerelease_channel_sees_release_candidates() {
        let releases = parse_releases(&body(), TARGET).unwrap();
        let pre = newest_after(&releases, "0.2.0", UpdateChannel::Prerelease).unwrap();
        assert_eq!(pre.version, "0.3.0-rc.1");
        assert!(pre.prerelease);
        assert!(newest_after(&releases, "0.3.0", UpdateChannel::Prerelease).is_none());
    }

    #[test]
    fn a_dev_build_ahead_of_the_latest_release_is_up_to_date() {
        let releases = parse_releases(&body(), TARGET).unwrap();
        assert!(newest_after(&releases, "0.5.0", UpdateChannel::Stable).is_none());
        assert!(newest_after(&releases, "garbage", UpdateChannel::Stable).is_none());
    }

    #[test]
    fn the_asset_is_the_archive_named_for_the_target() {
        let linux = parse_releases(&body(), TARGET).unwrap();
        let r = linux.iter().find(|r| r.version == "0.2.0").unwrap();
        assert_eq!(
            r.asset.as_ref().unwrap().url,
            "https://example.test/d/linux.tar.gz"
        );
        assert!(r.checksum.as_ref().unwrap().name.ends_with(".sha256"));
        let win = parse_releases(&body(), "x86_64-pc-windows-msvc").unwrap();
        let r = win.iter().find(|r| r.version == "0.2.0").unwrap();
        assert!(r.asset.as_ref().unwrap().name.ends_with(".zip"));
        assert!(r.checksum.is_none(), "no checksum published for it");
        let mac = parse_releases(&body(), "aarch64-apple-darwin").unwrap();
        let r = mac.iter().find(|r| r.version == "0.2.0").unwrap();
        assert!(
            r.asset.is_none(),
            "no archive for a target the release lacks"
        );
        assert_eq!(r.page_url, "https://example.test/r/v0.2.0");
    }

    #[test]
    fn the_check_asks_the_releases_list_and_answers_the_newer_one() {
        let transport =
            RecordedTransport::new().on("/repos/Poulpi-Lab/GitPoulpi/releases", 200, &body());
        let found = futures_block(check(
            &transport,
            "https://api.example.test/",
            &CheckParams {
                token: Some("tok".into()),
                ..params("0.1.0", UpdateChannel::Stable)
            },
        ))
        .unwrap()
        .expect("a newer release");
        assert_eq!(found.version, "0.2.0");
        assert_eq!(found.notes, "## 0.2.0\n- new");
        let seen = transport.requests();
        assert_eq!(seen.len(), 1);
        assert!(seen[0]
            .url
            .starts_with("https://api.example.test/repos/Poulpi-Lab/GitPoulpi/releases"));
        assert!(
            seen[0]
                .headers
                .iter()
                .any(|(k, v)| k == "Authorization" && v == "Bearer tok"),
            "{:?}",
            seen[0].headers
        );
    }

    #[test]
    fn no_token_means_no_authorization_header_and_a_404_is_an_error_not_a_panic() {
        let transport =
            RecordedTransport::new().on("/releases", 404, "{\"message\":\"Not Found\"}");
        let err = futures_block(check(
            &transport,
            "https://api.example.test",
            &params("0.1.0", UpdateChannel::Stable),
        ))
        .unwrap_err();
        assert!(err.to_string().contains("404"), "{err}");
        assert!(transport.requests()[0]
            .headers
            .iter()
            .all(|(k, _)| k != "Authorization"));
    }

    #[test]
    fn the_check_is_due_once_a_day() {
        let mut s = UpdateState::default();
        assert!(s.due(1000, false), "never checked");
        s.record(1000, None);
        assert!(!s.due(1000 + CHECK_INTERVAL_SECS - 1, false));
        assert!(s.due(1000 + CHECK_INTERVAL_SECS, false));
        assert!(
            s.due(1001, true),
            "the palette command ignores the throttle"
        );
    }

    #[test]
    fn a_failed_check_keeps_the_known_release_and_still_counts_as_an_attempt() {
        let releases = parse_releases(&body(), TARGET).unwrap();
        let newest = newest_after(&releases, "0.1.0", UpdateChannel::Stable);
        let mut s = UpdateState::default();
        s.record(10, Some(Ok(newest.clone())));
        s.record(20, Some(Err(Error::Http("offline".into()))));
        assert_eq!(s.checked_at, 20);
        assert_eq!(s.latest, newest);
    }

    #[test]
    fn skipping_a_version_hides_it_but_not_a_newer_one() {
        let releases = parse_releases(&body(), TARGET).unwrap();
        let mut s = UpdateState {
            latest: newest_after(&releases, "0.1.0", UpdateChannel::Stable),
            skipped: Some("0.2.0".into()),
            ..Default::default()
        };
        assert!(s.available("0.1.0", UpdateChannel::Stable).is_none());
        s.latest.as_mut().unwrap().version = "0.2.1".into();
        assert!(s.available("0.1.0", UpdateChannel::Stable).is_some());
    }

    #[test]
    fn the_kept_release_stops_being_news_once_the_app_is_updated() {
        let releases = parse_releases(&body(), TARGET).unwrap();
        let s = UpdateState {
            latest: newest_after(&releases, "0.1.0", UpdateChannel::Stable),
            ..Default::default()
        };
        assert!(s.available("0.1.0", UpdateChannel::Stable).is_some());
        assert!(s.available("0.2.0", UpdateChannel::Stable).is_none());
    }

    #[test]
    fn the_state_file_round_trips_and_a_damaged_one_is_empty() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("sub").join("updates.json");
        assert_eq!(UpdateState::load(&path), UpdateState::default());
        let s = UpdateState {
            checked_at: 42,
            latest: None,
            skipped: Some("0.2.0".into()),
        };
        s.save(&path).unwrap();
        assert_eq!(UpdateState::load(&path), s);
        std::fs::write(&path, "{not json").unwrap();
        assert_eq!(UpdateState::load(&path), UpdateState::default());
    }

    /// The transport is async; the tests have no runtime of their own.
    fn futures_block<F: std::future::Future>(f: F) -> F::Output {
        tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap()
            .block_on(f)
    }
}

#[cfg(test)]
mod download_host_tests {
    use super::*;
    use crate::http::RecordedTransport;

    #[test]
    fn only_https_github_release_hosts_may_be_downloaded_from() {
        for ok in [
            "https://github.com/o/r/releases/download/v1/a.tar.gz",
            "https://objects.githubusercontent.com/github-production-release-asset/1",
            "https://release-assets.githubusercontent.com/x?sig=1",
            "https://api.github.com/repos/o/r/releases/assets/1",
        ] {
            assert!(download_host_allowed(ok), "{ok}");
        }
        for refused in [
            "http://github.com/o/r/a.tar.gz",
            "https://evil.example/a.tar.gz",
            "https://github.com.evil.example/a.tar.gz",
            "https://github.com@evil.example/a.tar.gz",
            "https://notgithub.com/a",
            "https://127.0.0.1:9/a",
            "ftp://github.com/a",
            "github.com/a",
        ] {
            assert!(!download_host_allowed(refused), "{refused}");
        }
    }

    #[tokio::test]
    async fn the_recorded_transport_serves_files_in_chunks_and_can_be_canceled() {
        let transport = RecordedTransport::new().on_file("/a.bin", vec![1u8; 200 * 1024]);
        let calls = std::sync::atomic::AtomicUsize::new(0);
        let bytes = transport
            .download(HttpRequest::get("https://github.com/a.bin"), &|_, total| {
                assert_eq!(total, Some(200 * 1024));
                calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                true
            })
            .await
            .unwrap();
        assert_eq!(bytes.len(), 200 * 1024);
        assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 4);
        let canceled = transport
            .download(HttpRequest::get("https://github.com/a.bin"), &|_, _| false)
            .await;
        assert!(canceled.is_err());
    }
}
