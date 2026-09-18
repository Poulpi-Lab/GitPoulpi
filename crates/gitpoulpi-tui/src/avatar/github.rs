//! GitHub as an avatar source: pure parsing and URL building, no I/O
//! (the fetching is `resolve`'s, over an injected `Fetch`).
//!
//! Two ways an author's GitHub picture is found, cheapest first:
//!
//! 1. **A GitHub noreply address** names the account outright:
//!    `<id>+<login>@users.noreply.github.com` (accounts since 2017) ->
//!    `https://avatars.githubusercontent.com/u/<id>`, or the older
//!    `<login>@users.noreply.github.com` -> `https://github.com/<login>.png`.
//!    No API call at all.
//! 2. **Any other address, in a repository with a github.com remote**:
//!    the REST "get a commit" endpoint
//!    (`GET /repos/{owner}/{repo}/commits/{sha}`) answers with
//!    `author.avatar_url` when GitHub has linked the commit's email to an
//!    account (and `author: null` when it has not). One call per *author*
//!    (not per commit), cached on disk (`cache::AvatarCache::get_github`),
//!    because unauthenticated calls are limited to 60 an hour.
//!
//! Verified against docs.github.com (REST "Commits" and "Rate limits"
//! pages): the commit object's top-level `author` is the
//! GitHub user (or `null`), distinct from `commit.author` (the raw git
//! identity); an unknown sha answers 422, an invisible repository 404;
//! an exhausted rate limit answers 403 or 429 with
//! `x-ratelimit-remaining: 0` and `x-ratelimit-reset` in epoch seconds.
//! `avatars.githubusercontent.com` honours `s=<px>` for the size.

/// The GitHub account a noreply address names.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum NoReply {
    /// `<id>+<login>@users.noreply.github.com`: the numeric id is stable
    /// across renames, so it is what the picture URL uses.
    Id(u64),
    /// `<login>@users.noreply.github.com`, the pre-2017 form.
    Login(String),
}

const NOREPLY_DOMAIN: &str = "@users.noreply.github.com";

/// The account a GitHub noreply `email` names, if it is one.
pub(crate) fn parse_noreply(email: &str) -> Option<NoReply> {
    let email = email.trim().to_ascii_lowercase();
    let local = email.strip_suffix(NOREPLY_DOMAIN)?;
    if let Some((id, login)) = local.split_once('+') {
        // `41898282+github-actions[bot]@...` is a bot: its login is not a
        // valid user name but its id is a perfectly good account id.
        if login.is_empty() || id.is_empty() || !id.bytes().all(|b| b.is_ascii_digit()) {
            return None;
        }
        return id.parse().ok().map(NoReply::Id);
    }
    is_login(local).then(|| NoReply::Login(local.to_string()))
}

/// GitHub's user-name rule: 1-39 ASCII letters, digits or single inner
/// hyphens.
fn is_login(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 39
        && s.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
        && !s.starts_with('-')
        && !s.ends_with('-')
        && !s.contains("--")
}

/// The picture URL for a noreply address, `size` pixels square.
pub(crate) fn noreply_avatar_url(email: &str, size: u32) -> Option<String> {
    let size = size.clamp(1, 460);
    Some(match parse_noreply(email)? {
        NoReply::Id(id) => format!("https://avatars.githubusercontent.com/u/{id}?s={size}&v=4"),
        NoReply::Login(login) => format!("https://github.com/{login}.png?size={size}"),
    })
}

/// A repository on github.com.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct GitHubRepo {
    pub(crate) owner: String,
    pub(crate) name: String,
}

impl GitHubRepo {
    /// `owner/name`, lowercased: the key a repository that answered 404
    /// is remembered under for the session.
    pub(crate) fn slug(&self) -> String {
        format!("{}/{}", self.owner, self.name).to_ascii_lowercase()
    }
}

/// The github.com repository a remote URL points at, for every spelling
/// a clone leaves in `.git/config`: `https://github.com/o/r(.git)`,
/// `git@github.com:o/r.git`, `ssh://git@github.com(:22)/o/r.git`,
/// `ssh.github.com` (port 443 SSH). `None` for any other host - a GitHub
/// Enterprise server has its own API base and its own accounts.
pub(crate) fn repo_of_remote(url: &str) -> Option<GitHubRepo> {
    let url = url.trim();
    let (host, path) = if let Some((_, rest)) = url.split_once("://") {
        let rest = rest.rsplit_once('@').map(|(_, r)| r).unwrap_or(rest);
        let (host, path) = rest.split_once('/')?;
        (host.split(':').next()?, path)
    } else {
        // scp-like: `user@host:path`.
        let rest = url.rsplit_once('@').map(|(_, r)| r).unwrap_or(url);
        rest.split_once(':')?
    };
    let host = host.to_ascii_lowercase();
    if !matches!(
        host.as_str(),
        "github.com" | "www.github.com" | "ssh.github.com"
    ) {
        return None;
    }
    let path = path.trim_matches('/');
    let path = path.strip_suffix(".git").unwrap_or(path);
    let mut parts = path.split('/');
    let (owner, name) = (parts.next()?, parts.next()?);
    if parts.next().is_some() || owner.is_empty() || name.is_empty() {
        return None;
    }
    Some(GitHubRepo {
        owner: owner.to_string(),
        name: name.to_string(),
    })
}

/// The repository to ask, from the window's remotes as `(name, url)`:
/// `origin` when it is on github.com, else the first remote that is.
pub(crate) fn pick_repo<'a>(
    remotes: impl IntoIterator<Item = (&'a str, &'a str)>,
) -> Option<GitHubRepo> {
    let mut first = None;
    for (name, url) in remotes {
        if let Some(repo) = repo_of_remote(url) {
            if name == "origin" {
                return Some(repo);
            }
            first.get_or_insert(repo);
        }
    }
    first
}

/// `GET /repos/{owner}/{repo}/commits/{sha}`.
pub(crate) fn commit_api_url(repo: &GitHubRepo, sha: &str) -> String {
    format!(
        "https://api.github.com/repos/{}/{}/commits/{sha}",
        repo.owner, repo.name
    )
}

/// `author.avatar_url` out of a "get a commit" response. `None` when
/// GitHub has not linked the commit's email to an account (`author:
/// null`) or the body is not what the endpoint documents.
pub(crate) fn author_avatar_url(body: &[u8]) -> Option<String> {
    let value: serde_json::Value = serde_json::from_slice(body).ok()?;
    let url = value.get("author")?.get("avatar_url")?.as_str()?;
    url.starts_with("https://").then(|| url.to_string())
}

/// `GET /search/users?q=<email> in:email`: the account whose *public*
/// email is this address. Used only with a connected integration's token
/// (the search API has its own, small, rate limit), after the commits API
/// found nothing.
pub(crate) fn search_users_url(email: &str) -> String {
    let mut q = String::new();
    for b in email.trim().bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b'~') {
            q.push(char::from(b));
        } else {
            q.push_str(&format!("%{b:02X}"));
        }
    }
    format!("https://api.github.com/search/users?q={q}+in%3Aemail&per_page=1") // i18n-english
}

/// The first hit's `avatar_url` out of a user-search response.
pub(crate) fn search_avatar_url(body: &[u8]) -> Option<String> {
    let value: serde_json::Value = serde_json::from_slice(body).ok()?;
    let url = value
        .get("items")?
        .as_array()?
        .first()?
        .get("avatar_url")?
        .as_str()?;
    url.starts_with("https://").then(|| url.to_string())
}

/// `avatar_url` with the size parameter added (`avatar_url` already
/// carries `?v=4`).
pub(crate) fn sized(avatar_url: &str, size: u32) -> String {
    let sep = if avatar_url.contains('?') { '&' } else { '?' };
    format!("{avatar_url}{sep}s={}", size.clamp(1, 460))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Falsified: accepted any local part as a login (`is_login` skipped:
    /// `-bad-`, `a--b` and `github-actions[bot]` read as accounts, red);
    /// restored.
    #[test]
    fn noreply_addresses_name_the_account_both_spellings() {
        assert_eq!(
            parse_noreply("12345+octocat@users.noreply.github.com"),
            Some(NoReply::Id(12345))
        );
        assert_eq!(
            parse_noreply(" Octocat@Users.NoReply.GitHub.com "),
            Some(NoReply::Login("octocat".into()))
        );
        assert_eq!(
            parse_noreply("41898282+github-actions[bot]@users.noreply.github.com"),
            Some(NoReply::Id(41898282))
        );
        for not_one in [
            "kai@example.com",
            "abc+me@users.noreply.github.com",
            "+me@users.noreply.github.com",
            "12+@users.noreply.github.com",
            "-bad-@users.noreply.github.com",
            "a--b@users.noreply.github.com",
            "github-actions[bot]@users.noreply.github.com",
            "me@users.noreply.github.com.evil.example",
            "",
        ] {
            assert_eq!(parse_noreply(not_one), None, "{not_one}");
        }
    }

    #[test]
    fn a_noreply_address_becomes_a_picture_url_with_no_api_call() {
        assert_eq!(
            noreply_avatar_url("583231+octocat@users.noreply.github.com", 42).as_deref(),
            Some("https://avatars.githubusercontent.com/u/583231?s=42&v=4")
        );
        assert_eq!(
            noreply_avatar_url("octocat@users.noreply.github.com", 42).as_deref(),
            Some("https://github.com/octocat.png?size=42")
        );
        assert_eq!(noreply_avatar_url("ada@example.com", 42), None);
    }

    #[test]
    fn every_remote_spelling_of_a_github_repo_is_recognized() {
        let want = Some(GitHubRepo {
            owner: "Poulpi-Lab".into(),
            name: "GitPoulpi".into(),
        });
        for url in [
            "https://github.com/Poulpi-Lab/GitPoulpi.git",
            "https://github.com/Poulpi-Lab/GitPoulpi",
            "https://github.com/Poulpi-Lab/GitPoulpi/",
            "https://user:tok@github.com/Poulpi-Lab/GitPoulpi.git",
            "git@github.com:Poulpi-Lab/GitPoulpi.git",
            "ssh://git@github.com/Poulpi-Lab/GitPoulpi.git",
            "ssh://git@ssh.github.com:443/Poulpi-Lab/GitPoulpi.git",
            "HTTPS://GitHub.com/Poulpi-Lab/GitPoulpi.git",
        ] {
            assert_eq!(repo_of_remote(url), want, "{url}");
        }
        for url in [
            "https://gitlab.com/Poulpi-Lab/GitPoulpi.git",
            "https://github.example.com/o/r.git",
            "git@github.com:onlyowner.git",
            "https://github.com/o/r/extra",
            "C:\\Projets\\GitPoulpi",
            "",
        ] {
            assert_eq!(repo_of_remote(url), None, "{url}");
        }
    }

    #[test]
    fn origin_is_preferred_then_the_first_github_remote() {
        let remotes = [
            ("upstream", "https://github.com/up/r.git"),
            ("fork", "https://github.com/me/r.git"),
            ("origin", "https://github.com/o/r.git"),
        ];
        assert_eq!(pick_repo(remotes).unwrap().owner, "o");
        let no_origin = [
            ("mirror", "https://gitlab.com/x/y.git"),
            ("upstream", "https://github.com/up/r.git"),
        ];
        assert_eq!(pick_repo(no_origin).unwrap().owner, "up");
        assert_eq!(pick_repo([("origin", "https://gitlab.com/x/y.git")]), None);
    }

    /// The shape docs.github.com documents: the GitHub user at the top
    /// level (`author`), the raw git identity under `commit.author`.
    #[test]
    fn the_avatar_url_is_the_top_level_author_not_the_git_identity() {
        let linked = br#"{"sha":"abc","commit":{"author":{"name":"S","email":"s@x"}},
            "author":{"login":"poulpi-lab","avatar_url":"https://avatars.githubusercontent.com/u/9?v=4"}}"#;
        assert_eq!(
            author_avatar_url(linked).as_deref(),
            Some("https://avatars.githubusercontent.com/u/9?v=4")
        );
        let unlinked = br#"{"sha":"abc","commit":{"author":{"name":"S"}},"author":null}"#;
        assert_eq!(author_avatar_url(unlinked), None);
        assert_eq!(author_avatar_url(b"<html>"), None);
        let not_https = br#"{"author":{"avatar_url":"http://evil.example/a.png"}}"#;
        assert_eq!(author_avatar_url(not_https), None);
    }

    #[test]
    fn the_size_is_appended_to_a_url_that_already_has_a_query() {
        assert_eq!(
            sized("https://avatars.githubusercontent.com/u/9?v=4", 42),
            "https://avatars.githubusercontent.com/u/9?v=4&s=42"
        );
        assert_eq!(sized("https://a.example/p", 42), "https://a.example/p?s=42");
        assert_eq!(
            commit_api_url(
                &GitHubRepo {
                    owner: "o".into(),
                    name: "r".into()
                },
                "abc"
            ),
            "https://api.github.com/repos/o/r/commits/abc"
        );
    }
}
