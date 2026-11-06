//! Origin normalization: the one spelling a trust record compares.
//!
//! An origin is `scheme://host[:port]` plus the base path with no trailing
//! slash. The default port is dropped, the host is lowercased and IDNA
//! encoded (the `url` crate does both), and credentials, query and fragment
//! are cut off. Two spellings of one server therefore compare equal, and a
//! lookalike host never does.

use gitpoulpi_config::{ApiFamily, Integration, IntegrationKind};
use url::{Host, Url};

use crate::{Error, Result};

fn parse(url: &str) -> Option<Url> {
    let url = url.trim();
    let url = if url.contains("://") {
        Url::parse(url).ok()?
    } else {
        Url::parse(&format!("https://{url}")).ok()?
    };
    matches!(url.scheme(), "http" | "https").then_some(())?;
    url.host_str()?;
    Some(url)
}

fn origin_string(url: &Url) -> String {
    let mut out = format!("{}://{}", url.scheme(), url.host_str().unwrap_or(""));
    if let Some(port) = url.port() {
        out.push_str(&format!(":{port}"));
    }
    out
}

/// `scheme://host[:port]` plus the base path of `url`, normalized; `None`
/// when `url` is not an http(s) URL with a host.
pub fn normalize(url: &str) -> Option<String> {
    let url = parse(url)?;
    let mut out = origin_string(&url);
    let path = url.path().trim_end_matches('/');
    if !path.is_empty() {
        out.push_str(path);
    }
    Some(out)
}

/// Just `scheme://host[:port]` of `url` (what a request is checked against).
pub fn origin_of(url: &str) -> Option<String> {
    Some(origin_string(&parse(url)?))
}

/// The host (and nothing else) of `url`, lowercased.
pub fn host_of(url: &str) -> Option<String> {
    parse(url)?.host_str().map(str::to_string)
}

/// The normalized origin an integration is recorded under. Azure DevOps
/// Services first goes through the same rewrite the client applies, so
/// `org.visualstudio.com` and `dev.azure.com/org` are one record.
pub fn integration_origin(integration: &Integration) -> Result<String> {
    // An empty URL means the kind's public host (the clients do the same).
    let raw = match integration.url.trim() {
        "" => integration.kind.default_url().unwrap_or("").to_string(),
        url => url.to_string(),
    };
    let url = if integration.kind == IntegrationKind::AzureDevops {
        gitpoulpi_config::normalize_azure_url(&raw)
    } else {
        raw
    };
    normalize(&url).ok_or_else(|| Error::BadUrl(integration.url.clone()))
}

/// Whether `url` is a plain `http://` address that is not loopback.
pub fn is_plain_http_remote(url: &str) -> bool {
    parse(url).is_some_and(|u| u.scheme() == "http" && !host_is_loopback(&u))
}

/// Whether the host of `url` is a loopback address or `localhost`.
pub fn is_loopback(url: &str) -> bool {
    parse(url).is_some_and(|u| host_is_loopback(&u))
}

fn host_is_loopback(url: &Url) -> bool {
    match url.host() {
        Some(Host::Domain(d)) => d == "localhost" || d.ends_with(".localhost"),
        Some(Host::Ipv4(ip)) => ip.is_loopback(),
        Some(Host::Ipv6(ip)) => ip.is_loopback(),
        None => false,
    }
}

/// The public hosts that serve an API separate from the web address of a
/// SaaS kind (`github.com` -> `api.github.com`).
fn dot_com_api_host(kind: IntegrationKind, host: &str) -> Option<&'static str> {
    let (domain, api): (&str, &'static str) = match kind.family() {
        ApiFamily::Github => ("github.com", "api.github.com"),
        ApiFamily::Bitbucket if kind == IntegrationKind::Bitbucket => {
            ("bitbucket.org", "api.bitbucket.org")
        }
        ApiFamily::Trello => ("trello.com", "api.trello.com"),
        ApiFamily::Linear => ("linear.app", "api.linear.app"),
        _ => return None,
    };
    (host == domain || host == format!("www.{domain}") || host == format!("api.{domain}"))
        .then_some(api)
}

/// The origins a request of an integration may reach: its own recorded
/// origin and, for a SaaS kind, the API host that serves it. Anything else
/// is refused before it is sent.
pub fn api_origins(kind: IntegrationKind, recorded_origin: &str) -> Vec<String> {
    let mut out = Vec::new();
    let Some(own) = origin_of(recorded_origin) else {
        return out;
    };
    if let Some(host) = host_of(recorded_origin) {
        if let Some(api) = dot_com_api_host(kind, &host) {
            out.push(format!("https://{api}"));
        }
    }
    out.push(own);
    out.dedup();
    out
}

/// The default hosts of the SaaS kinds: no TLS record is kept for them, the
/// system roots already vouch for them.
pub fn is_public_saas_host(host: &str) -> bool {
    let host = host.to_ascii_lowercase();
    const EXACT: &[&str] = &[
        "github.com",
        "api.github.com",
        "gitlab.com",
        "codeberg.org",
        "bitbucket.org",
        "api.bitbucket.org",
        "dev.azure.com",
        "api.trello.com",
        "trello.com",
        "api.linear.app",
        "linear.app",
    ];
    EXACT.contains(&host.as_str())
        || host.ends_with(".atlassian.net")
        || host.ends_with(".youtrack.cloud")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spellings_of_one_server_compare_equal_and_lookalikes_do_not() {
        let same = [
            ("https://GitLab.Example.com/", "https://gitlab.example.com"),
            (
                "https://gitlab.example.com:443",
                "https://gitlab.example.com",
            ),
            ("http://gitlab.example.com:80/", "http://gitlab.example.com"),
            (
                "https://user:pw@gitlab.example.com/sub/?q=1#f",
                "https://gitlab.example.com/sub",
            ),
            ("gitlab.example.com", "https://gitlab.example.com"),
            (
                "https://git.example.com:8443/a/b/",
                "https://git.example.com:8443/a/b",
            ),
            ("https://BÜCHER.example", "https://xn--bcher-kva.example"),
        ];
        for (input, want) in same {
            assert_eq!(normalize(input).as_deref(), Some(want), "{input}");
        }
        let a = normalize("https://gitlab.example.com").unwrap();
        for other in [
            "https://gitlab.example.com.evil.test",
            "https://gitlab-example.com",
            "http://gitlab.example.com",
            "https://gitlab.example.com:8443",
            "https://gitlab.example.com/other",
        ] {
            assert_ne!(normalize(other).unwrap(), a, "{other}");
        }
        assert_eq!(normalize("ftp://x.example"), None);
        assert_eq!(normalize(""), None);
    }

    #[test]
    fn loopback_and_plain_http() {
        assert!(is_loopback("http://localhost:3000"));
        assert!(is_loopback("http://127.0.0.1:8080"));
        assert!(is_loopback("http://[::1]:11434"));
        assert!(!is_loopback("http://192.168.1.4"));
        assert!(!is_plain_http_remote("http://localhost:11434"));
        assert!(is_plain_http_remote("http://nas.local:3000"));
        assert!(!is_plain_http_remote("https://nas.local"));
    }

    #[test]
    fn saas_kinds_may_reach_their_api_host_and_nothing_else() {
        let gh = api_origins(IntegrationKind::Github, "https://github.com");
        assert!(gh.contains(&"https://api.github.com".to_string()));
        assert!(gh.contains(&"https://github.com".to_string()));
        let ghe = api_origins(
            IntegrationKind::GithubEnterprise,
            "https://ghe.corp.example",
        );
        assert_eq!(ghe, vec!["https://ghe.corp.example".to_string()]);
        let gl = api_origins(
            IntegrationKind::GitlabSelfManaged,
            "https://git.corp.example/sub",
        );
        assert_eq!(gl, vec!["https://git.corp.example".to_string()]);
        assert!(api_origins(IntegrationKind::Trello, "https://trello.com")
            .contains(&"https://api.trello.com".to_string()));
    }

    #[test]
    fn public_saas_hosts() {
        assert!(is_public_saas_host("GitHub.com"));
        assert!(is_public_saas_host("acme.atlassian.net"));
        assert!(!is_public_saas_host("git.corp.example"));
        assert!(!is_public_saas_host("evilgithub.com"));
    }
}
