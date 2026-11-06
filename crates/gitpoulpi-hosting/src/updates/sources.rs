//! Where an update check asks (the install-channels design, section 1.5).
//!
//! A [`SourceSpec`] is a plain value the caller builds from the install
//! channel and the policy; [`SourceSpec::check`] runs it through an
//! [`HttpTransport`] (tests use a fake, never a socket). Every source ends in
//! [`newest_after_pinned`], so semver, channel and pin filtering live once.
//!
//! Traps:
//! - NuGet ids are lowercase in the v3 `index.json` path; the feed's own
//!   casing (`GitPoulpi`) 404s on several servers.
//! - A v3 feed URL ends in `.json`; anything else is a v2 (OData/Atom) root.
//!   The v2 answer is pages of `<entry>` with a `rel="next"` link, followed
//!   up to [`MAX_V2_PAGES`] times.
//! - NuGet versions may carry a fourth part (`1.4.2.0`); a zero one is
//!   dropped, a non-zero one is not a version this app can order, so it is
//!   skipped.
//! - Release notes are never taken from NuGet or the manifest's host
//!   beyond what the manifest says; the panel falls back to the compiled-in
//!   CHANGELOG.
//! - The manifest names assets without URLs: they are siblings of the
//!   manifest file, which is what "a mirror only copies files" means.

use std::collections::BTreeMap;

use async_trait::async_trait;

use super::{
    newest_after_pinned, parse_releases, pick_assets, Asset, CheckParams, Release, Version,
};
use crate::http::{HttpRequest, HttpTransport};
use crate::{Error, Result};
use gitpoulpi_config::release;

/// The package id on NuGet and Chocolatey feeds.
pub const NUGET_ID: &str = "gitpoulpi";

/// Chocolatey's community feed (a v2 root).
pub const CHOCOLATEY_COMMUNITY_FEED: &str = "https://community.chocolatey.org/api/v2/";

/// Pages of a v2 answer followed before giving up (the feed is small).
pub const MAX_V2_PAGES: usize = 5;

/// Which source answers the check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceSpec {
    /// The GitHub (or GitHub Enterprise Server) releases API.
    GitHubReleases {
        /// `https://api.github.com`, or a GHES `https://host/api/v3`.
        api_base: String,
        /// `owner/name`.
        repo: String,
        token: Option<String>,
    },
    /// A NuGet v3 service index (`.../index.json`) or a v2 root.
    NuGet { feed: String },
    /// A static `latest.json`.
    Manifest { url: String },
    /// Never asks anyone (a managed deployment).
    None,
}

/// One way of finding the newest release.
#[async_trait]
pub trait CheckSource: Send + Sync {
    async fn latest(
        &self,
        transport: &dyn HttpTransport,
        params: &CheckParams,
    ) -> Result<Option<Release>>;
}

impl SourceSpec {
    /// Asks the source for the newest release newer than `params.running`.
    pub async fn check(
        &self,
        transport: &dyn HttpTransport,
        params: &CheckParams,
    ) -> Result<Option<Release>> {
        self.latest(transport, params).await
    }
}

#[async_trait]
impl CheckSource for SourceSpec {
    async fn latest(
        &self,
        transport: &dyn HttpTransport,
        params: &CheckParams,
    ) -> Result<Option<Release>> {
        let releases = match self {
            SourceSpec::None => return Ok(None),
            SourceSpec::GitHubReleases {
                api_base,
                repo,
                token,
            } => {
                github_releases(transport, api_base, repo, token.as_deref(), &params.target).await?
            }
            SourceSpec::NuGet { feed } => nuget_releases(transport, feed).await?,
            SourceSpec::Manifest { url } => {
                let body = get(transport, url, "the update manifest", &[]).await?;
                vec![manifest_release(
                    url,
                    &parse_manifest(&body)?,
                    &params.target,
                )?]
            }
        };
        Ok(newest_after_pinned(
            &releases,
            &params.running,
            params.channel,
            params.pin.as_deref(),
        ))
    }
}

async fn get(
    transport: &dyn HttpTransport,
    url: &str,
    what: &str,
    headers: &[(&str, String)],
) -> Result<String> {
    let mut request = HttpRequest::get(url);
    for (k, v) in headers {
        request = request.header(*k, v.clone());
    }
    let response = transport.send(request).await?;
    response.error_for_status(what)?;
    Ok(response.body)
}

// ---- GitHub ----------------------------------------------------------

async fn github_releases(
    transport: &dyn HttpTransport,
    api_base: &str,
    repo: &str,
    token: Option<&str>,
    target: &str,
) -> Result<Vec<Release>> {
    let url = format!(
        "{}/repos/{}/releases?per_page=30",
        api_base.trim_end_matches('/'),
        repo.trim_matches('/')
    );
    let mut headers = vec![("Accept", "application/vnd.github+json".to_string())];
    if let Some(token) = token.map(str::trim).filter(|t| !t.is_empty()) {
        headers.push(("Authorization", format!("Bearer {token}")));
    }
    let body = get(transport, &url, "the update check", &headers).await?;
    parse_releases(&body, target)
}

// ---- NuGet -----------------------------------------------------------

async fn nuget_releases(transport: &dyn HttpTransport, feed: &str) -> Result<Vec<Release>> {
    let feed = feed.trim();
    let versions = if feed
        .split(['?', '#'])
        .next()
        .unwrap_or("")
        .ends_with(".json")
    {
        nuget_v3_versions(transport, feed).await?
    } else {
        nuget_v2_versions(transport, feed).await?
    };
    let page_base = feed
        .starts_with(CHOCOLATEY_COMMUNITY_FEED)
        .then_some("https://community.chocolatey.org/packages/gitpoulpi");
    Ok(versions
        .into_iter()
        .filter_map(|raw| {
            let version = normalize_nuget_version(&raw)?;
            let v = Version::parse(&version)?;
            Some(Release {
                tag: version.clone(),
                prerelease: v.is_prerelease(),
                notes: String::new(),
                page_url: page_base.map_or_else(String::new, |b| format!("{b}/{version}")),
                asset: None,
                checksum: None,
                version,
            })
        })
        .collect())
}

/// `1.4.2.0` -> `1.4.2`; a non-zero fourth part cannot be ordered here.
fn normalize_nuget_version(raw: &str) -> Option<String> {
    let raw = raw.trim();
    let (core, rest) = match raw.find(['-', '+']) {
        Some(i) => raw.split_at(i),
        None => (raw, ""),
    };
    let mut parts: Vec<&str> = core.split('.').collect();
    if parts.len() == 4 {
        if parts[3] != "0" {
            return None;
        }
        parts.pop();
    }
    while parts.len() < 3 && !parts.is_empty() {
        parts.push("0");
    }
    Some(format!("{}{rest}", parts.join(".")))
}

async fn nuget_v3_versions(transport: &dyn HttpTransport, index_url: &str) -> Result<Vec<String>> {
    let body = get(transport, index_url, "the NuGet service index", &[]).await?;
    let index: serde_json::Value =
        serde_json::from_str(&body).map_err(|e| Error::Decode(e.to_string()))?;
    let base = index["resources"]
        .as_array()
        .into_iter()
        .flatten()
        .find(|r| {
            r["@type"]
                .as_str()
                .is_some_and(|t| t.starts_with("PackageBaseAddress"))
        })
        .and_then(|r| r["@id"].as_str())
        .ok_or_else(|| Error::Decode("the NuGet index has no PackageBaseAddress".into()))?;
    let url = format!("{}/{NUGET_ID}/index.json", base.trim_end_matches('/'));
    let body = get(transport, &url, "the NuGet version list", &[]).await?;
    let list: serde_json::Value =
        serde_json::from_str(&body).map_err(|e| Error::Decode(e.to_string()))?;
    Ok(list["versions"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|v| v.as_str().map(str::to_string))
        .collect())
}

async fn nuget_v2_versions(transport: &dyn HttpTransport, root: &str) -> Result<Vec<String>> {
    let mut url = format!(
        "{}/FindPackagesById()?id='{NUGET_ID}'",
        root.trim_end_matches('/')
    );
    let mut versions = Vec::new();
    for _ in 0..MAX_V2_PAGES {
        let body = get(transport, &url, "the NuGet feed", &[]).await?;
        versions.extend(atom_versions(&body));
        match atom_next_link(&body) {
            Some(next) if next != url => url = next,
            _ => break,
        }
    }
    Ok(versions)
}

/// The text of every `<d:Version>` element of an Atom/OData answer. A plain
/// scan, not an XML parser: the shape is fixed by the OData v2 contract, and
/// `<d:NormalizedVersion>` / `<d:VersionDownloadCount>` must not match.
fn atom_versions(body: &str) -> Vec<String> {
    const OPEN: &str = "<d:Version";
    let mut out = Vec::new();
    let mut rest = body;
    while let Some(i) = rest.find(OPEN) {
        rest = &rest[i + OPEN.len()..];
        if !rest.starts_with(['>', ' ', '/']) {
            continue;
        }
        let Some(gt) = rest.find('>') else { break };
        if rest[..gt].ends_with('/') {
            continue;
        }
        let after = &rest[gt + 1..];
        let Some(end) = after.find("</d:Version>") else {
            break;
        };
        out.push(unescape(after[..end].trim()));
        rest = &after[end..];
    }
    out
}

/// The `href` of `<link rel="next" href="..."/>`, if the answer is paged.
fn atom_next_link(body: &str) -> Option<String> {
    let mut rest = body;
    while let Some(i) = rest.find("<link") {
        let tail = &rest[i..];
        let end = tail.find('>')?;
        let tag = &tail[..end];
        if tag.contains("rel=\"next\"") {
            let href = tag.split("href=\"").nth(1)?.split('"').next()?;
            return Some(unescape(href));
        }
        rest = &tail[end..];
    }
    None
}

fn unescape(s: &str) -> String {
    s.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&amp;", "&")
}

// ---- Manifest --------------------------------------------------------

/// The static `latest.json` a release attaches.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct Manifest {
    pub version: String,
    #[serde(default)]
    pub notes: String,
    /// File name -> SHA-256 (lowercase hex).
    #[serde(default)]
    pub assets: BTreeMap<String, String>,
}

impl Manifest {
    /// The SHA-256 the manifest states for `name`.
    pub fn sha256_of(&self, name: &str) -> Option<&str> {
        self.assets.get(name).map(String::as_str)
    }
}

pub fn parse_manifest(body: &str) -> Result<Manifest> {
    serde_json::from_str(body).map_err(|e| Error::Decode(e.to_string()))
}

fn manifest_release(url: &str, manifest: &Manifest, target: &str) -> Result<Release> {
    let version = manifest.version.trim().trim_start_matches(['v', 'V']);
    let v = Version::parse(version)
        .ok_or_else(|| Error::Decode(format!("the manifest version {version:?} is not semver")))?;
    let dir = url
        .split(['?', '#'])
        .next()
        .unwrap_or(url)
        .rsplit_once('/')
        .map_or("", |(d, _)| d);
    let archive = release::archive_name(version, target);
    let sibling = |name: &str| Asset {
        name: name.to_string(),
        url: format!("{dir}/{name}"),
        size: 0,
    };
    // The archive and its `.sha256` sit next to the manifest, like on the
    // release page; only archives the manifest lists count.
    let listed: Vec<Asset> = manifest.assets.keys().map(|n| sibling(n)).collect();
    let (asset, mut checksum) = pick_assets(&listed, version, target);
    if asset.is_some() && checksum.is_none() {
        checksum = Some(sibling(&format!("{archive}.sha256")));
    }
    Ok(Release {
        tag: version.to_string(),
        version: version.to_string(),
        prerelease: v.is_prerelease(),
        notes: manifest.notes.clone(),
        page_url: String::new(),
        asset,
        checksum,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::http::RecordedTransport;
    use crate::updates::{download_host_allowed, download_host_allowed_with};
    use gitpoulpi_config::UpdateChannel;

    const TARGET: &str = "x86_64-unknown-linux-gnu";

    fn block<F: std::future::Future>(f: F) -> F::Output {
        tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap()
            .block_on(f)
    }

    fn params(running: &str) -> CheckParams {
        CheckParams {
            repo: "Poulpi-Lab/GitPoulpi".into(),
            running: running.into(),
            channel: UpdateChannel::Stable,
            target: TARGET.into(),
            ..Default::default()
        }
    }

    fn run(t: &RecordedTransport, spec: &SourceSpec, p: &CheckParams) -> Option<Release> {
        block(spec.check(t, p)).unwrap()
    }

    fn github_body() -> String {
        serde_json::json!([
            {"tag_name": "v1.5.0", "prerelease": false, "body": "five", "html_url": "u5", "assets": []},
            {"tag_name": "v1.4.9", "prerelease": false, "body": "four nine", "html_url": "u49", "assets": []},
            {"tag_name": "v1.4.2", "prerelease": false, "body": "four two", "html_url": "u42", "assets": []},
            {"tag_name": "v1.6.0-rc.1", "prerelease": true, "body": "rc", "html_url": "u6", "assets": []}
        ])
        .to_string()
    }

    fn github(api_base: &str) -> SourceSpec {
        SourceSpec::GitHubReleases {
            api_base: api_base.into(),
            repo: "Poulpi-Lab/GitPoulpi".into(),
            token: Some("tok".into()),
        }
    }

    #[test]
    fn github_asks_the_releases_list_with_the_token() {
        let t = RecordedTransport::new().on(
            "/repos/Poulpi-Lab/GitPoulpi/releases",
            200,
            &github_body(),
        );
        let found = run(&t, &github("https://api.github.com/"), &params("1.0.0")).unwrap();
        assert_eq!(found.version, "1.5.0");
        let seen = t.requests();
        assert!(seen[0]
            .url
            .starts_with("https://api.github.com/repos/Poulpi-Lab/GitPoulpi/releases"));
        assert!(seen[0]
            .headers
            .iter()
            .any(|(k, v)| k == "Authorization" && v == "Bearer tok"));
    }

    #[test]
    fn github_enterprise_uses_its_own_api_base() {
        let t = RecordedTransport::new().on("/api/v3/repos/", 200, &github_body());
        let found = run(
            &t,
            &github("https://ghe.corp.example/api/v3"),
            &params("1.0.0"),
        )
        .unwrap();
        assert_eq!(found.version, "1.5.0");
        assert!(t.requests()[0]
            .url
            .starts_with("https://ghe.corp.example/api/v3/repos/Poulpi-Lab/GitPoulpi/releases"));
    }

    const V3_INDEX: &str = r#"{"version":"3.0.0","resources":[
        {"@id":"https://feed.example/query","@type":"SearchQueryService"},
        {"@id":"https://feed.example/flat/","@type":"PackageBaseAddress/3.0.0"}]}"#;
    const V3_VERSIONS: &str =
        r#"{"versions":["1.3.0","1.4.2","1.4.9","1.5.0","1.6.0-rc.1","1.7.0.0","1.8.0.3"]}"#;

    fn nuget_v3() -> (RecordedTransport, SourceSpec) {
        let t = RecordedTransport::new()
            .on("/flat/gitpoulpi/index.json", 200, V3_VERSIONS)
            .on("/v3/index.json", 200, V3_INDEX);
        (
            t,
            SourceSpec::NuGet {
                feed: "https://feed.example/v3/index.json".into(),
            },
        )
    }

    #[test]
    fn nuget_v3_follows_the_service_index_to_the_lowercase_id_list() {
        let (t, spec) = nuget_v3();
        let found = run(&t, &spec, &params("1.0.0")).unwrap();
        // 1.8.0.3 has a non-zero fourth part and is skipped; 1.7.0.0 is 1.7.0.
        assert_eq!(found.version, "1.7.0");
        assert!(!found.prerelease);
        let urls: Vec<String> = t.requests().into_iter().map(|r| r.url).collect();
        assert_eq!(
            urls,
            [
                "https://feed.example/v3/index.json",
                "https://feed.example/flat/gitpoulpi/index.json"
            ]
        );
        let pre = CheckParams {
            channel: UpdateChannel::Prerelease,
            running: "1.7.0".into(),
            ..params("1.7.0")
        };
        assert!(
            run(&t, &spec, &pre).is_none(),
            "1.6.0-rc.1 is older than 1.7.0"
        );
    }

    fn atom(versions: &[&str], next: Option<&str>) -> String {
        let entries: String = versions
            .iter()
            .map(|v| {
                format!(
                    "<entry><title type=\"text\">gitpoulpi</title><m:properties>\
                     <d:Version>{v}</d:Version><d:NormalizedVersion>{v}</d:NormalizedVersion>\
                     <d:VersionDownloadCount m:type=\"Edm.Int32\">3</d:VersionDownloadCount>\
                     </m:properties></entry>"
                )
            })
            .collect();
        let link = next
            .map(|n| format!("<link rel=\"next\" href=\"{n}\" />"))
            .unwrap_or_default();
        format!("<?xml version=\"1.0\"?><feed xmlns:d=\"x\">{entries}{link}</feed>")
    }

    #[test]
    fn nuget_v2_reads_atom_entries_and_follows_the_next_page() {
        let page2 = atom(&["1.5.0"], None);
        let page1 = atom(
            &["1.3.0", "1.4.2"],
            Some("https://feed.example/api/v2/FindPackagesById()?id='gitpoulpi'&amp;$skip=2"),
        );
        let t = RecordedTransport::new().on("$skip=2", 200, &page2).on(
            "FindPackagesById()?id='gitpoulpi'",
            200,
            &page1,
        );
        let spec = SourceSpec::NuGet {
            feed: "https://feed.example/api/v2/".into(),
        };
        let found = run(&t, &spec, &params("1.0.0")).unwrap();
        assert_eq!(found.version, "1.5.0");
        assert_eq!(t.requests().len(), 2);
        assert!(
            t.requests()[1].url.contains("&$skip=2"),
            "&amp; is unescaped"
        );
    }

    #[test]
    fn the_chocolatey_community_feed_links_its_package_page() {
        let t = RecordedTransport::new().on("FindPackagesById", 200, &atom(&["2.0.0"], None));
        let spec = SourceSpec::NuGet {
            feed: CHOCOLATEY_COMMUNITY_FEED.into(),
        };
        let found = run(&t, &spec, &params("1.0.0")).unwrap();
        assert_eq!(
            found.page_url,
            "https://community.chocolatey.org/packages/gitpoulpi/2.0.0"
        );
        assert!(t.requests()[0]
            .url
            .starts_with("https://community.chocolatey.org/api/v2/FindPackagesById()"));
    }

    #[test]
    fn a_v3_index_without_package_base_address_is_a_decode_error() {
        let t = RecordedTransport::new().on("/index.json", 200, r#"{"resources":[]}"#);
        let spec = SourceSpec::NuGet {
            feed: "https://feed.example/index.json".into(),
        };
        assert!(block(spec.check(&t, &params("1.0.0"))).is_err());
    }

    fn manifest_body() -> String {
        serde_json::json!({
            "version": "1.4.2",
            "notes": "## 1.4.2\n- fix",
            "assets": {
                "gitpoulpi-1.4.2-x86_64-unknown-linux-gnu.tar.gz": "ab12",
                "gitpoulpi-1.4.2-x86_64-pc-windows-msvc.zip": "cd34"
            }
        })
        .to_string()
    }

    #[test]
    fn the_manifest_names_the_archive_beside_itself() {
        let url = "https://mirror.example/gitpoulpi/latest.json";
        let t = RecordedTransport::new().on("latest.json", 200, &manifest_body());
        let found = run(
            &t,
            &SourceSpec::Manifest { url: url.into() },
            &params("1.0.0"),
        )
        .unwrap();
        assert_eq!(found.version, "1.4.2");
        assert_eq!(found.notes, "## 1.4.2\n- fix");
        assert_eq!(
            found.asset.unwrap().url,
            "https://mirror.example/gitpoulpi/gitpoulpi-1.4.2-x86_64-unknown-linux-gnu.tar.gz"
        );
        assert_eq!(
            found.checksum.unwrap().url,
            "https://mirror.example/gitpoulpi/gitpoulpi-1.4.2-x86_64-unknown-linux-gnu.tar.gz.sha256"
        );
        assert_eq!(
            parse_manifest(&manifest_body())
                .unwrap()
                .sha256_of("gitpoulpi-1.4.2-x86_64-pc-windows-msvc.zip"),
            Some("cd34")
        );
        let none = run(
            &t,
            &SourceSpec::Manifest { url: url.into() },
            &params("1.4.2"),
        );
        assert!(none.is_none(), "up to date");
    }

    #[test]
    fn a_manifest_without_a_semver_version_is_an_error() {
        let t = RecordedTransport::new().on("latest.json", 200, r#"{"version":"nightly"}"#);
        let spec = SourceSpec::Manifest {
            url: "https://m.example/latest.json".into(),
        };
        assert!(block(spec.check(&t, &params("1.0.0"))).is_err());
    }

    #[test]
    fn the_none_source_never_touches_the_network() {
        let t = RecordedTransport::new();
        assert!(run(&t, &SourceSpec::None, &params("0.0.1")).is_none());
        assert!(t.requests().is_empty());
    }

    #[test]
    fn the_pin_applies_to_every_source() {
        let pinned = |pin: &str| CheckParams {
            pin: Some(pin.into()),
            ..params("1.0.0")
        };
        let t = RecordedTransport::new().on("/releases", 200, &github_body());
        let gh = github("https://api.github.com");
        assert_eq!(run(&t, &gh, &pinned("1.4")).unwrap().version, "1.4.9");
        assert_eq!(run(&t, &gh, &pinned("1.4.2")).unwrap().version, "1.4.2");
        assert_eq!(run(&t, &gh, &pinned("1")).unwrap().version, "1.5.0");
        assert!(
            run(&t, &gh, &pinned("1.4.2-bad")).is_none(),
            "a bad pin offers nothing"
        );
        assert!(run(&t, &gh, &pinned("0.9")).is_none());
        let (t3, nuget) = nuget_v3();
        assert_eq!(run(&t3, &nuget, &pinned("1.4")).unwrap().version, "1.4.9");
        let tm = RecordedTransport::new().on("latest.json", 200, &manifest_body());
        let man = SourceSpec::Manifest {
            url: "https://m.example/latest.json".into(),
        };
        assert!(run(&tm, &man, &pinned("1.3")).is_none());
        assert_eq!(run(&tm, &man, &pinned("1.4")).unwrap().version, "1.4.2");
    }

    #[test]
    fn a_mirror_host_is_allowed_only_when_the_policy_gives_it() {
        let url = "https://mirror.example/gitpoulpi/a.tar.gz";
        let base = ["https://mirror.example/gitpoulpi/".to_string()];
        assert!(!download_host_allowed(url));
        assert!(!download_host_allowed_with(url, &[]));
        assert!(download_host_allowed_with(url, &base));
        assert!(download_host_allowed_with(
            url,
            &["mirror.example".to_string()]
        ));
        // the built-in hosts stay allowed, and the mirror grants nothing more
        assert!(download_host_allowed_with("https://github.com/a", &base));
        assert!(!download_host_allowed_with(
            "https://other.example/a",
            &base
        ));
        assert!(!download_host_allowed_with(
            "https://mirror.example.evil/a",
            &base
        ));
        assert!(!download_host_allowed_with(
            "https://mirror.example@evil.example/a",
            &base
        ));
        assert!(
            !download_host_allowed_with("http://mirror.example/a", &base),
            "https only"
        );
    }
}
