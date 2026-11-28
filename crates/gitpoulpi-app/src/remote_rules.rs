//! What a remote's name and URL may be, as the Add / Edit remote form checks
//! them, and how a remote URL splits into scheme, host and path.
//!
//! A frontend calls [`validate_remote_name`] and [`validate_remote_url`] on
//! the typed fields and shows the message next to the field; git is still the
//! real judge, the point is to catch a paste of the wrong thing before it is
//! saved.

use gitpoulpi_i18n::msg::remotes as msg;

/// `(scheme, host, path)` of a remote URL git accepts over the network:
/// `http(s)://`, `ssh://`, `git://` and the scp-like `[user@]host:path`.
/// The host loses any user and port.
pub fn split_remote_url(url: &str) -> Option<(&str, &str, &str)> {
    if url.chars().any(char::is_whitespace) {
        return None;
    }
    if let Some((scheme, rest)) = url.split_once("://") {
        let scheme = match scheme.to_ascii_lowercase().as_str() {
            "http" => "http",
            "https" => "https",
            "ssh" | "git+ssh" | "ssh+git" => "ssh",
            "git" => "git",
            _ => return None,
        };
        let (authority, path) = rest.split_once('/').unwrap_or((rest, ""));
        let host = authority.rsplit('@').next()?;
        let host = host.split(':').next()?;
        if host.is_empty() {
            return None;
        }
        return Some((scheme, host, path));
    }
    // scp-like: `[user@]host:path`. The colon must come before any `/`
    // (`./a:b` is a path), and a one-letter "host" is a Windows drive
    // (`C:\repo`, `C:/repo`), not a machine.
    let (authority, path) = url.split_once(':')?;
    if authority.contains('/') || authority.contains('\\') {
        return None;
    }
    let host = authority.rsplit('@').next()?;
    if host.len() <= 1 || path.is_empty() || path.starts_with('\\') {
        return None;
    }
    Some(("ssh", host, path))
}

/// Accepts what a remote URL typed into the form may be: `http(s)://`,
/// `ssh://`, `git://`, scp-like `git@host:owner/repo.git`, or a local
/// `file://` URL. Deliberately simple.
pub fn validate_remote_url(url: &str) -> Result<(), String> {
    let url = url.trim();
    if url.is_empty() {
        return Err(msg::URL_EMPTY.get().into());
    }
    if url.starts_with("file://") && url.len() > "file://".len() {
        return Ok(());
    }
    match split_remote_url(url) {
        Some((_, _, path)) if !path.trim_matches('/').is_empty() => Ok(()),
        _ => Err(msg::not_remote_url(url)),
    }
}

/// Git's own refname rules, as they apply to a remote name, plus "not
/// already taken" (the name being kept by an edit does not count).
pub fn validate_remote_name(
    name: &str,
    taken: &[String],
    keeping: Option<&str>,
) -> Result<(), String> {
    if name.is_empty() {
        return Err(msg::NAME_EMPTY.get().into());
    }
    if name.chars().any(|c| c.is_whitespace() || c.is_control())
        || name.chars().any(|c| "~^:?*[\\".contains(c))
        || name.contains("..")
        || name.contains("@{")
        || name.starts_with('-')
        || name.starts_with('/')
        || name.ends_with('/')
        || name.ends_with('.')
        || name.ends_with(".lock")
    {
        return Err(msg::invalid_name(name));
    }
    if taken.iter().any(|t| t == name) && keeping != Some(name) {
        return Err(msg::name_taken(name));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn urls_split_and_validate() {
        assert_eq!(
            split_remote_url("git@host.example:owner/repo.git"),
            Some(("ssh", "host.example", "owner/repo.git"))
        );
        assert_eq!(
            split_remote_url("https://user@host.example:8443/a/b"),
            Some(("https", "host.example", "a/b"))
        );
        assert_eq!(split_remote_url("C:\\repo"), None);
        for ok in [
            "https://h.example/a/b.git",
            "git@h.example:a/b",
            "file:///srv/r",
        ] {
            assert!(validate_remote_url(ok).is_ok(), "{ok}");
        }
        for bad in ["", "   ", "not a url", "https://h.example/", "ftp://h/x"] {
            assert!(validate_remote_url(bad).is_err(), "{bad:?}");
        }
    }

    #[test]
    fn names_follow_ref_rules_and_may_keep_their_own() {
        let taken = vec!["origin".to_string()];
        assert!(validate_remote_name("upstream", &taken, None).is_ok());
        assert!(validate_remote_name("grp/sub", &taken, None).is_ok());
        assert!(validate_remote_name("origin", &taken, None).is_err());
        assert!(validate_remote_name("origin", &taken, Some("origin")).is_ok());
        for bad in ["", "a b", "a..b", "-x", "x.lock", "a:b"] {
            assert!(validate_remote_name(bad, &taken, None).is_err(), "{bad:?}");
        }
    }
}
