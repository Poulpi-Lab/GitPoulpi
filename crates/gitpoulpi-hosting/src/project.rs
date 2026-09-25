//! The one project shape the UI renders, whichever host it came from.
//!
//! GitHub calls them repositories and keys them by `owner/name`; GitLab
//! calls them projects and keys them by a numeric id with a
//! `group/subgroup/name` path. Normalising here rather than in each
//! frontend is what lets the *clone from this list* flow be written once.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostedProject {
    /// The integration this came from, so a list merged across several
    /// hosts can still say where each row belongs - and so *clone this
    /// row* knows which token to use.
    pub integration_id: String,
    /// The host's own identifier, as a string: a number for GitLab, the
    /// `owner/name` path for GitHub. Opaque to everything except the
    /// client that produced it.
    pub id: String,
    /// The short name (`gitpoulpi`).
    pub name: String,
    /// The full path (`poulpi-lab/gitpoulpi`, `group/sub/thing`) - what
    /// the list is filtered on, because that is what a user types.
    pub full_name: String,
    pub description: Option<String>,
    pub default_branch: Option<String>,
    /// The two clone URLs, either of which can be missing when the host
    /// has that protocol disabled.
    pub ssh_url: Option<String>,
    pub https_url: Option<String>,
    /// The page a *Open in browser* action goes to.
    pub web_url: Option<String>,
    pub private: bool,
}

impl HostedProject {
    /// The owner section this project belongs to: the user, the GitHub
    /// organisation or the GitLab group / sub-group path (`group/sub` for
    /// `group/sub/thing`). Empty when the path has no owner.
    pub fn owner(&self) -> &str {
        self.full_name
            .rsplit_once('/')
            .map_or("", |(owner, _)| owner)
    }

    /// The URL a clone should use, given whether the user prefers SSH.
    /// Falls back to the other protocol rather than failing: a host with
    /// SSH disabled is legitimate.
    pub fn clone_url(&self, prefer_ssh: bool) -> Option<&str> {
        let (first, second) = if prefer_ssh {
            (&self.ssh_url, &self.https_url)
        } else {
            (&self.https_url, &self.ssh_url)
        };
        first.as_deref().or(second.as_deref())
    }

    /// Whether this project matches a filter the user typed:
    /// case-insensitive substring over the full path. Not fuzzy, because a
    /// list of several thousand rows is re-filtered on every keystroke.
    pub fn matches(&self, filter: &str) -> bool {
        let filter = filter.trim().to_lowercase();
        if filter.is_empty() {
            return true;
        }
        self.full_name.to_lowercase().contains(&filter)
            || self.name.to_lowercase().contains(&filter)
            || self
                .description
                .as_deref()
                .is_some_and(|d| d.to_lowercase().contains(&filter))
    }
}

/// An SSH public key as the host holds it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostedSshKey {
    pub id: String,
    pub title: String,
    /// The `ssh-ed25519 AAAA... comment` line. Public by definition --
    /// the private half never reaches this crate.
    pub key: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn project(ssh: Option<&str>, https: Option<&str>) -> HostedProject {
        HostedProject {
            integration_id: "h".into(),
            id: "1".into(),
            name: "thing".into(),
            full_name: "group/sub/thing".into(),
            description: Some("A useful Widget".into()),
            default_branch: Some("main".into()),
            ssh_url: ssh.map(str::to_string),
            https_url: https.map(str::to_string),
            web_url: None,
            private: false,
        }
    }

    #[test]
    fn the_preferred_protocol_wins_but_the_other_is_a_fallback() {
        let both = project(Some("git@h:g/t.git"), Some("https://h/g/t.git"));
        assert_eq!(both.clone_url(true), Some("git@h:g/t.git"));
        assert_eq!(both.clone_url(false), Some("https://h/g/t.git"));

        let https_only = project(None, Some("https://h/g/t.git"));
        assert_eq!(
            https_only.clone_url(true),
            Some("https://h/g/t.git"),
            "a host with ssh disabled must still be clonable"
        );

        assert_eq!(project(None, None).clone_url(true), None);
    }

    #[test]
    fn the_owner_is_the_path_without_the_last_segment() {
        assert_eq!(project(None, None).owner(), "group/sub");
        let mut p = project(None, None);
        p.full_name = "me/thing".into();
        assert_eq!(p.owner(), "me");
        p.full_name = "thing".into();
        assert_eq!(p.owner(), "");
    }

    #[test]
    fn filtering_is_case_insensitive_over_path_name_and_description() {
        let p = project(None, None);
        assert!(p.matches(""));
        assert!(p.matches("  "));
        assert!(p.matches("SUB/TH"));
        assert!(p.matches("thing"));
        assert!(p.matches("widget"), "description must be searchable");
        assert!(!p.matches("nothing-like-this"));
    }
}
