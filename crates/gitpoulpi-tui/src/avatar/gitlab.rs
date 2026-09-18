//! GitLab (gitlab.com or self-managed) as an avatar source: pure URL
//! building and parsing, no I/O (the fetching is `resolve`'s).
//!
//! Only used when a `[[integrations]]` entry of kind `gitlab` matches the
//! repository's remote host -- a GitLab host the user has not connected is
//! never asked about their authors' email addresses.
//!
//! Two endpoints, cheapest first:
//!
//! 1. `GET {base}/api/v4/avatar?email=<email>&size=<px>` -> `{"avatar_url":
//!    "..."}`. Works without a token. For an address GitLab does not know
//!    it answers with a **Gravatar** URL instead of an error, which is
//!    treated as "no account" here ([`is_gravatar`]): Gravatar is its own
//!    source with its own place in the order.
//! 2. With a token, `GET {base}/api/v4/users?search=<email>` -> an array
//!    of users, the first one's `avatar_url` (GitLab matches a public
//!    email, and, for a token with enough rights, any email).

use gitpoulpi_config::{Integration, IntegrationKind};

/// A connected GitLab instance a repository's remote points at.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct GitLabHost {
    /// `https://gitlab.example.com`, no trailing slash, no path.
    pub(crate) base: String,
}

/// An enabled gitlab.com or self-managed GitLab profile.
pub(crate) fn is_gitlab(integration: &Integration) -> bool {
    integration.enabled
        && matches!(
            integration.kind,
            IntegrationKind::Gitlab | IntegrationKind::GitlabSelfManaged
        )
}

/// `host[:port]` of a URL or scp-like remote, lowercased.
fn host_of(url: &str) -> Option<String> {
    let url = url.trim();
    let rest = match url.split_once("://") {
        Some((_, rest)) => rest,
        None => url,
    };
    let rest = rest.rsplit_once('@').map_or(rest, |(_, r)| r);
    let host = rest.split(['/', ':']).next()?;
    (!host.is_empty()).then(|| host.to_ascii_lowercase())
}

/// The connected GitLab integration whose host is the remote's, as the
/// host to ask. `origin` first, then any other remote.
pub(crate) fn pick_host<'a>(
    remotes: impl IntoIterator<Item = (&'a str, &'a str)>,
    integrations: &[Integration],
) -> Option<GitLabHost> {
    let connected: Vec<(String, &Integration)> = integrations
        .iter()
        .filter(|i| is_gitlab(i))
        .filter_map(|i| Some((host_of(&i.url)?, i)))
        .collect();
    let mut first = None;
    for (name, url) in remotes {
        let Some(remote_host) = host_of(url) else {
            continue;
        };
        if let Some((_, integration)) = connected.iter().find(|(h, _)| *h == remote_host) {
            let host = GitLabHost {
                base: base_of(&integration.url),
            };
            if name == "origin" {
                return Some(host);
            }
            first.get_or_insert(host);
        }
    }
    first
}

/// `scheme://host[:port]` of an integration URL (a trailing path or slash
/// is dropped), `https` when no scheme is given.
pub(crate) fn base_of(url: &str) -> String {
    let url = url.trim();
    let (scheme, rest) = url.split_once("://").unwrap_or(("https", url));
    let host = rest.split('/').next().unwrap_or(rest);
    format!("{scheme}://{host}")
}

/// Percent-encodes everything but unreserved characters.
fn encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b'~') {
            out.push(char::from(b));
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

/// `GET /api/v4/avatar?email=`. The address is lowercased: GitLab matches it
/// as stored, and a commit's `Ada@Example.com` is the account's
/// `ada@example.com` (email addresses compare without case in practice).
pub(crate) fn avatar_api_url(host: &GitLabHost, email: &str, size: u32) -> String {
    format!(
        "{}/api/v4/avatar?email={}&size={}",
        host.base,
        encode(&email.trim().to_lowercase()),
        size.clamp(1, 512)
    )
}

/// `GET /api/v4/users?search=`.
pub(crate) fn users_search_url(host: &GitLabHost, email: &str) -> String {
    format!(
        "{}/api/v4/users?search={}&per_page=1",
        host.base,
        encode(&email.trim().to_lowercase())
    )
}

/// Whether a picture URL is Gravatar's (GitLab's answer for "no such
/// user").
pub(crate) fn is_gravatar(url: &str) -> bool {
    let rest = url.split_once("://").map_or(url, |(_, r)| r);
    let host = rest.split(['/', ':']).next().unwrap_or("");
    let host = host.to_ascii_lowercase();
    host == "gravatar.com" || host.ends_with(".gravatar.com")
}

/// The picture URL out of either endpoint's body (an object, or the first
/// element of an array), made absolute against the host. `None` for no
/// user, for GitLab's Gravatar stand-in, or for a body that is not JSON.
pub(crate) fn avatar_url(host: &GitLabHost, body: &[u8]) -> Option<String> {
    let value: serde_json::Value = serde_json::from_slice(body).ok()?;
    let user = match &value {
        serde_json::Value::Array(users) => users.first()?,
        other => other,
    };
    let url = user.get("avatar_url")?.as_str()?.trim();
    if url.is_empty() {
        return None;
    }
    let url = if url.starts_with("http://") || url.starts_with("https://") {
        url.to_string()
    } else if url.starts_with('/') {
        format!("{}{url}", host.base)
    } else {
        return None;
    };
    (!is_gravatar(&url)).then_some(url)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn host() -> GitLabHost {
        GitLabHost {
            base: "https://git.example.com".into(),
        }
    }

    fn gitlab(url: &str) -> Integration {
        Integration::new("gl", IntegrationKind::Gitlab, url)
    }

    #[test]
    fn a_remote_matches_only_a_connected_gitlab_host() {
        let integrations = [gitlab("https://git.example.com/")];
        let pick = |url| pick_host([("origin", url)], &integrations);
        assert_eq!(pick("https://git.example.com/group/repo.git"), Some(host()));
        assert_eq!(pick("git@git.example.com:group/repo.git"), Some(host()));
        assert_eq!(pick("ssh://git@GIT.example.com:2222/g/r.git"), Some(host()));
        assert_eq!(pick("https://gitlab.com/g/r.git"), None);
        assert_eq!(pick("https://git.example.com.evil.net/g/r.git"), None);
        assert_eq!(
            pick_host([("origin", "https://git.example.com/g/r")], &[]),
            None
        );
        // A GitHub profile on that host does not count.
        let github = [Integration::new(
            "gh",
            IntegrationKind::Github,
            "https://git.example.com",
        )];
        assert_eq!(
            pick_host([("origin", "https://git.example.com/g/r")], &github),
            None
        );
    }

    #[test]
    fn origin_wins_over_another_remote() {
        let integrations = [
            gitlab("https://a.example.com"),
            gitlab("https://b.example.com"),
        ];
        let picked = pick_host(
            [
                ("up", "https://a.example.com/g/r"),
                ("origin", "https://b.example.com/g/r"),
            ],
            &integrations,
        );
        assert_eq!(picked.unwrap().base, "https://b.example.com");
    }

    #[test]
    fn the_urls_encode_the_email() {
        assert_eq!(
            avatar_api_url(&host(), " Ada+x@Example.com ", 128),
            "https://git.example.com/api/v4/avatar?email=ada%2Bx%40example.com&size=128"
        );
        assert!(users_search_url(&host(), "a@b.c").contains("search=a%40b.c"));
        // A commit's upper-case address finds the lower-case account.
        assert!(users_search_url(&host(), "Ada.Lovelace@Example.com")
            .contains("search=ada.lovelace%40example.com"));
    }

    #[test]
    fn the_avatar_url_is_read_from_either_endpoint() {
        assert_eq!(
            avatar_url(
                &host(),
                br#"{"avatar_url":"https://cdn.example.com/a.png"}"#
            )
            .as_deref(),
            Some("https://cdn.example.com/a.png")
        );
        assert_eq!(
            avatar_url(&host(), br#"[{"id":1,"avatar_url":"/uploads/a.png"}]"#).as_deref(),
            Some("https://git.example.com/uploads/a.png")
        );
        assert_eq!(avatar_url(&host(), b"[]"), None);
        assert_eq!(avatar_url(&host(), b"not json"), None);
        assert_eq!(avatar_url(&host(), br#"{"avatar_url":null}"#), None);
    }

    /// GitLab answers an unknown address with a Gravatar URL: that is
    /// "no account", not a picture from this source.
    #[test]
    fn gitlabs_gravatar_stand_in_is_no_account() {
        assert_eq!(
            avatar_url(
                &host(),
                br#"{"avatar_url":"https://www.gravatar.com/avatar/abc?s=80&d=identicon"}"#
            ),
            None
        );
        assert!(is_gravatar("https://secure.gravatar.com/avatar/x"));
        assert!(!is_gravatar("https://gravatar.com.evil.net/x"));
    }
}
