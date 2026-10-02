//! Reading every spelling of an Azure DevOps remote.
//!
//! | Remote | Web base | Project path |
//! |---|---|---|
//! | `https://org@dev.azure.com/org/P/_git/R` | `https://dev.azure.com/org` | `P/R` |
//! | `https://dev.azure.com/org/_git/R` | `https://dev.azure.com/org` | `R/R` |
//! | `https://org.visualstudio.com[/DefaultCollection]/P/_git/R` | `https://dev.azure.com/org` | `P/R` |
//! | `git@ssh.dev.azure.com:v3/org/P/R` | `https://dev.azure.com/org` | `P/R` |
//! | `org@vs-ssh.visualstudio.com:v3/org/P/R` | `https://dev.azure.com/org` | `P/R` |
//! | Server `https://h/tfs/Coll/P/_git/R`, `ssh://h:22/tfs/Coll/P/_git/R` | `https://h/tfs/Coll` | `P/R` |
//!
//! Names are percent-decoded (`My%20Project` is `My Project`).
//!
//! On Server the project is the segment before `_git`; the short form
//! `https://h/tfs/Coll/_git/R` (no project segment) is read as collection
//! `tfs/Coll` when the first segment is `tfs`, and `https://h/Coll/_git/R`
//! as collection `Coll`; both give project `R`. A collection and a project
//! cannot otherwise be told apart from a URL alone.

use gitpoulpi_config::Integration;

use crate::spec::RemoteRef;

pub(super) fn parse_remote(remote: &str) -> Option<RemoteRef> {
    let remote = remote.trim();
    if remote.is_empty() {
        return None;
    }
    let (scheme, host, port, path) = split(remote)?;
    let host = host.to_ascii_lowercase();
    let segments: Vec<String> = path
        .split('/')
        .filter(|s| !s.is_empty())
        .map(decode)
        .collect();

    let services = |org: &str, rest: &[String]| -> Option<RemoteRef> {
        let (project, repo) = project_and_repo(rest)?;
        (!org.is_empty()).then(|| RemoteRef {
            web_base: format!("https://dev.azure.com/{org}"),
            project: format!("{project}/{repo}"),
        })
    };

    if host == "dev.azure.com" {
        let (org, rest) = segments.split_first()?;
        return services(org, rest);
    }
    if host == "ssh.dev.azure.com" || host == "vs-ssh.visualstudio.com" {
        // `v3/{org}/{project}/{repo}`
        let rest = segments.strip_prefix(&["v3".to_string()][..])?;
        let (org, rest) = rest.split_first()?;
        let (project, repo) = match rest {
            [project, repo] => (project, repo),
            _ => return None,
        };
        return services(org, &[project.clone(), "_git".into(), repo.clone()]);
    }
    if let Some(org) = host.strip_suffix(".visualstudio.com") {
        if org.contains('.') {
            return None;
        }
        let rest = match segments.first() {
            Some(first) if first.eq_ignore_ascii_case("DefaultCollection") => &segments[1..],
            _ => &segments[..],
        };
        return services(org, rest);
    }

    // Azure DevOps Server (or anything else with a `_git` segment).
    let git = segments.iter().position(|s| s == "_git")?;
    let repo = segments.get(git + 1)?.clone();
    let prefix = &segments[..git];
    let (collection, project) = match prefix {
        [] => return None,
        [only] => (vec![only.clone()], repo.clone()),
        [first, second] if first.eq_ignore_ascii_case("tfs") => {
            (vec![first.clone(), second.clone()], repo.clone())
        }
        _ => (
            prefix[..prefix.len() - 1].to_vec(),
            prefix[prefix.len() - 1].clone(),
        ),
    };
    // An `ssh://` port is the daemon's, not the web UI's.
    let web_scheme = if scheme == "http" { "http" } else { "https" };
    let authority = match (scheme.as_str(), port) {
        ("http" | "https", Some(port)) => format!("{host}:{port}"),
        _ => host.clone(),
    };
    Some(RemoteRef {
        web_base: format!("{web_scheme}://{authority}/{}", collection.join("/")),
        project: format!("{project}/{repo}"),
    })
}

/// `[P, "_git", R]` or `["_git", R]` (project named like its repository).
fn project_and_repo(rest: &[String]) -> Option<(String, String)> {
    match rest {
        [project, git, repo] if git == "_git" => Some((project.clone(), repo.clone())),
        [git, repo] if git == "_git" => Some((repo.clone(), repo.clone())),
        _ => None,
    }
}

/// `(scheme, host, port, path)` of any remote spelling.
fn split(remote: &str) -> Option<(String, String, Option<String>, String)> {
    if let Some((scheme, rest)) = remote.split_once("://") {
        let (authority, path) = rest.split_once('/').unwrap_or((rest, ""));
        let authority = authority.rsplit_once('@').map_or(authority, |(_, h)| h);
        let (host, port) = match authority.rsplit_once(':') {
            Some((h, p)) if !p.is_empty() && p.chars().all(|c| c.is_ascii_digit()) => {
                (h, Some(p.to_string()))
            }
            _ => (authority, None),
        };
        return (!host.is_empty()).then(|| {
            (
                scheme.to_ascii_lowercase(),
                host.to_string(),
                port,
                path.to_string(),
            )
        });
    }
    // scp-like: `user@host:path`.
    let after_user = remote.rsplit_once('@').map_or(remote, |(_, r)| r);
    let (host, path) = after_user.split_once(':')?;
    (!host.is_empty()).then(|| ("ssh".into(), host.to_string(), None, path.to_string()))
}

fn decode(segment: &str) -> String {
    fn hex(b: u8) -> Option<u8> {
        (b as char).to_digit(16).map(|d| d as u8)
    }
    let bytes = segment.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let (Some(hi), Some(lo)) = (hex(bytes[i + 1]), hex(bytes[i + 2])) {
                out.push(hi * 16 + lo);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// Identity of an instance root: lowercase `host/path`, no scheme, user,
/// port or trailing slash. Two Azure organizations on one host differ in
/// the path, which is the point.
fn root_key(url: &str) -> String {
    let rest = url.split_once("://").map_or(url, |(_, r)| r);
    let rest = rest.trim_end_matches('/');
    let (authority, path) = rest.split_once('/').unwrap_or((rest, ""));
    let authority = authority.rsplit_once('@').map_or(authority, |(_, h)| h);
    let host = match authority.rsplit_once(':') {
        Some((h, p)) if p.chars().all(|c| c.is_ascii_digit()) => h,
        _ => authority,
    };
    format!(
        "{}/{}",
        host.to_ascii_lowercase(),
        path.to_ascii_lowercase()
    )
    .trim_end_matches('/')
    .to_string()
}

/// A profile owns a remote when both name the same organization (or
/// Server collection), so two orgs on `dev.azure.com` are two profiles.
pub(super) fn owns_remote(integration: &Integration, remote: &RemoteRef) -> bool {
    if integration.url.trim().is_empty() {
        return false;
    }
    let root = gitpoulpi_config::normalize_azure_url(&integration.url);
    root_key(&root) == root_key(&remote.web_base)
}
