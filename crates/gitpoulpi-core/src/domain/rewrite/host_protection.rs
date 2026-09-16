//! What the host itself protects (S23), and how to get a rewrite through it.
//!
//! The workbench excludes the branches the host protects (GitHub branch
//! protection and rulesets, GitLab protected branches, Azure DevOps branch
//! policies, Bitbucket branch restrictions) exactly like the ones the
//! workflow and `rewrite.protected_branches` name: same per-branch checkbox,
//! same all/none toggle. When such a branch blocks (at the preview, or as
//! `rejected_protected_by_host` after a push) the three ways through are
//! *explained*, never done: lift the protection in the host's settings, push
//! the rewrite under a new branch name and switch the default branch, or as
//! the last resort delete and recreate the remote repository.
//!
//! Pure data and text. Reading the host is `gitpoulpi-hosting`'s job (a
//! client-side step: hosting stays out of the daemon); the answer travels in
//! [`ScopeOptions::host_protection`](super::ScopeOptions), where the daemon
//! adds its patterns to the protected names of the preview.

use serde::{Deserialize, Serialize};

use super::HostKind;

/// What is known about the host of one repository.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostProtection {
    pub kind: HostKind,
    /// The project as the host names it (`owner/name`, `group/sub/proj`,
    /// `Project/Repo` on Azure DevOps, `WORKSPACE/slug` or `KEY/slug` on
    /// Bitbucket).
    pub project: String,
    /// The web root of the instance (`https://github.com`).
    pub web_base: String,
    /// The branch names and `*` patterns the host protects (short names).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub patterns: Vec<String>,
    /// The host was asked (the repository is linked). `false`: the settings
    /// links are still built from the remote's URL, but nothing was read.
    #[serde(default)]
    pub asked: bool,
    /// Why the list could not be read (a token without the scope, an
    /// unsupported host). English; `patterns` may still hold what part of it
    /// could be read.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unknown: Option<String>,
}

/// `*` matches any run of characters (slashes included); everything else is
/// literal.
fn glob_match(pattern: &str, text: &str) -> bool {
    let parts: Vec<&str> = pattern.split('*').collect();
    let [first, middle @ .., last] = parts.as_slice() else {
        return pattern == text;
    };
    let Some(mut rest) = text.strip_prefix(first) else {
        return false;
    };
    for part in middle {
        match rest.find(part) {
            Some(at) => rest = &rest[at + part.len()..],
            None => return false,
        }
    }
    rest.ends_with(last)
}

impl HostProtection {
    pub fn new(kind: HostKind, project: impl Into<String>, web_base: impl Into<String>) -> Self {
        Self {
            kind,
            project: project.into(),
            web_base: web_base.into().trim_end_matches('/').to_string(),
            patterns: Vec::new(),
            asked: false,
            unknown: None,
        }
    }

    /// "GitHub", "GitLab", "Azure DevOps", "Bitbucket"; the neutral "the
    /// host" for a server that is none of them.
    pub fn provider(&self) -> &'static str {
        match self.kind {
            HostKind::GitHub => "GitHub",
            HostKind::GitLab => "GitLab",
            HostKind::AzureDevOps => "Azure DevOps",
            HostKind::Bitbucket => "Bitbucket",
            HostKind::Other => "the host",
        }
    }

    /// Whether the host protects the branch `name` (short or full).
    pub fn covers(&self, name: &str) -> bool {
        let short = name.strip_prefix("refs/heads/").unwrap_or(name);
        self.patterns.iter().any(|p| glob_match(p, short))
    }

    fn data_center(&self) -> bool {
        self.kind == HostKind::Bitbucket && !self.web_base.contains("bitbucket.org")
    }

    fn azure_split(&self) -> (String, String) {
        match self.project.split_once('/') {
            Some((p, r)) => (p.to_string(), r.to_string()),
            None => (self.project.clone(), self.project.clone()),
        }
    }

    /// The pages where the protection is lifted, most specific first. Empty
    /// for a host GitPoulpi cannot name.
    pub fn settings_urls(&self) -> Vec<String> {
        let (base, project) = (&self.web_base, &self.project);
        match self.kind {
            HostKind::GitHub => vec![
                format!("{base}/{project}/settings/branches"),
                format!("{base}/{project}/settings/rules"),
            ],
            HostKind::GitLab => {
                vec![format!(
                    "{base}/{project}/-/settings/repository#js-protected-branches-settings"
                )]
            }
            HostKind::AzureDevOps => {
                let (p, _) = self.azure_split();
                vec![format!("{base}/{}/_settings/repositories", url_part(&p))]
            }
            HostKind::Bitbucket if self.data_center() => {
                let (key, slug) = self.project.split_once('/').unwrap_or((project, project));
                vec![format!("{base}/projects/{key}/repos/{slug}/settings")]
            }
            HostKind::Bitbucket => vec![format!("{base}/{project}/admin/branch-restrictions")],
            HostKind::Other => Vec::new(),
        }
    }

    /// Where the default branch is switched (way 2).
    pub fn default_branch_url(&self) -> Option<String> {
        let (base, project) = (&self.web_base, &self.project);
        match self.kind {
            HostKind::GitHub => Some(format!("{base}/{project}/settings/branches")),
            HostKind::GitLab => Some(format!(
                "{base}/{project}/-/settings/repository#branch-defaults-settings"
            )),
            HostKind::AzureDevOps => {
                let (p, r) = self.azure_split();
                Some(format!(
                    "{base}/{}/_git/{}/branches",
                    url_part(&p),
                    url_part(&r)
                ))
            }
            HostKind::Bitbucket if self.data_center() => {
                let (key, slug) = self.project.split_once('/').unwrap_or((project, project));
                Some(format!("{base}/projects/{key}/repos/{slug}/settings"))
            }
            HostKind::Bitbucket => Some(format!("{base}/{project}/admin")),
            HostKind::Other => None,
        }
    }

    /// Where the repository is deleted (way 3).
    pub fn delete_url(&self) -> Option<String> {
        let (base, project) = (&self.web_base, &self.project);
        match self.kind {
            HostKind::GitHub => Some(format!("{base}/{project}/settings")),
            HostKind::GitLab => Some(format!("{base}/{project}/-/edit")),
            HostKind::AzureDevOps => self.settings_urls().into_iter().next(),
            HostKind::Bitbucket if self.data_center() => {
                let (key, slug) = self.project.split_once('/').unwrap_or((project, project));
                Some(format!("{base}/projects/{key}/repos/{slug}/settings"))
            }
            HostKind::Bitbucket => Some(format!("{base}/{project}/admin")),
            HostKind::Other => None,
        }
    }

    /// The three ways through, for `branches` (what blocks; short names).
    pub fn howto(&self, branches: &[String]) -> Howto {
        let kind = self.kind;
        let lift = HowtoWay {
            way: Way::LiftProtection,
            steps: vec![HowStep::lift(kind)],
            links: self.settings_urls(),
        };
        let rename = HowtoWay {
            way: Way::NewBranchName,
            steps: vec![HowStep::PushNewName, HowStep::switch(kind)],
            links: self.default_branch_url().into_iter().collect(),
        };
        let recreate = HowtoWay {
            way: Way::Recreate,
            steps: vec![HowStep::Recreate],
            links: self.delete_url().into_iter().collect(),
        };
        Howto {
            provider: self.provider().to_string(),
            branches: branches.to_vec(),
            ways: vec![lift, rename, recreate],
            losses: Loss::ALL.to_vec(),
        }
    }
}

/// A path segment, percent-encoded where a space or a slash would break it.
fn url_part(text: &str) -> String {
    let mut out = String::new();
    for b in text.bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b'~') {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

/// The three ways, from the least to the most drastic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Way {
    /// Temporarily lift the protection in the host's settings.
    LiftProtection,
    /// Push the rewrite under a new branch name, then switch the default
    /// branch (or rename).
    NewBranchName,
    /// Delete and recreate the remote repository, then push everything.
    Recreate,
}

impl Way {
    /// The i18n key suffix.
    pub fn code(self) -> &'static str {
        match self {
            Self::LiftProtection => "way_lift",
            Self::NewBranchName => "way_rename",
            Self::Recreate => "way_recreate",
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            Self::LiftProtection => "1. Temporarily lift the protection in the host's settings",
            Self::NewBranchName => {
                "2. Push the rewrite under a new branch name, then switch the default branch"
            }
            Self::Recreate => "3. Last resort: delete and recreate the remote repository",
        }
    }
}

/// One instruction of a way. `code()` is the i18n key suffix.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HowStep {
    LiftGithub,
    LiftGitlab,
    LiftAzure,
    LiftBitbucket,
    LiftOther,
    PushNewName,
    SwitchGithub,
    SwitchGitlab,
    SwitchAzure,
    SwitchBitbucket,
    SwitchOther,
    Recreate,
}

impl HowStep {
    fn lift(kind: HostKind) -> Self {
        match kind {
            HostKind::GitHub => Self::LiftGithub,
            HostKind::GitLab => Self::LiftGitlab,
            HostKind::AzureDevOps => Self::LiftAzure,
            HostKind::Bitbucket => Self::LiftBitbucket,
            HostKind::Other => Self::LiftOther,
        }
    }

    fn switch(kind: HostKind) -> Self {
        match kind {
            HostKind::GitHub => Self::SwitchGithub,
            HostKind::GitLab => Self::SwitchGitlab,
            HostKind::AzureDevOps => Self::SwitchAzure,
            HostKind::Bitbucket => Self::SwitchBitbucket,
            HostKind::Other => Self::SwitchOther,
        }
    }

    pub fn code(self) -> &'static str {
        match self {
            Self::LiftGithub => "step_lift_github",
            Self::LiftGitlab => "step_lift_gitlab",
            Self::LiftAzure => "step_lift_azure",
            Self::LiftBitbucket => "step_lift_bitbucket",
            Self::LiftOther => "step_lift_other",
            Self::PushNewName => "step_push_new_name",
            Self::SwitchGithub => "step_switch_github",
            Self::SwitchGitlab => "step_switch_gitlab",
            Self::SwitchAzure => "step_switch_azure",
            Self::SwitchBitbucket => "step_switch_bitbucket",
            Self::SwitchOther => "step_switch_other",
            Self::Recreate => "step_recreate",
        }
    }

    /// The step in English (CLI, MCP, `rewrite note`).
    pub fn text(self) -> &'static str {
        match self {
            Self::LiftGithub => {
                "Open the repository's Settings > Branches (branch protection rules) and Settings > Rules (rulesets), remove or relax the rule that covers the branch, push the rewrite, then put the rule back."
            }
            Self::LiftGitlab => {
                "Open Settings > Repository > Protected branches, unprotect the branch (or turn on \"Allowed to force push\" for it), push the rewrite, then protect it again."
            }
            Self::LiftAzure => {
                "Open Project settings > Repositories > the repository > Policies, remove the branch policy on the branch and, under Security, allow \"Force push (rewrite history and delete branches)\" for yourself, push the rewrite, then restore both."
            }
            Self::LiftBitbucket => {
                "Open the repository's settings > Branch restrictions (Branch permissions on Data Center), remove the restriction that covers the branch, push the rewrite, then restore it."
            }
            Self::LiftOther => {
                "Remove or relax the branch protection rule in the host's repository settings, push the rewrite, then restore the rule."
            }
            Self::PushNewName => {
                "Push the rewrite to a branch that is not protected: `git push <remote> <branch>:<new-name>` (GitPoulpi leaves the protected ref alone)."
            }
            Self::SwitchGithub => {
                "Then in Settings > Branches set the new branch as the default branch (or rename the old one first with Settings > Branches > the pencil icon > Rename), and delete the old branch once nobody uses it."
            }
            Self::SwitchGitlab => {
                "Then in Settings > Repository > Branch defaults set the new branch as the default branch, and delete the old one (Code > Branches) once nobody uses it."
            }
            Self::SwitchAzure => {
                "Then in Repos > Branches use the \"...\" menu of the new branch > Set as default branch, and delete the old one once nobody uses it."
            }
            Self::SwitchBitbucket => {
                "Then in the repository's settings > Repository details set the new branch as the main branch (the default branch on Data Center), and delete the old one once nobody uses it."
            }
            Self::SwitchOther => {
                "Then make the new branch the default branch in the host's repository settings, and delete the old one once nobody uses it."
            }
            Self::Recreate => {
                "Delete the remote repository in its settings, create an empty one with the same name, then push every branch and tag (`git push --all` and `git push --tags`). Everyone must re-clone."
            }
        }
    }
}

/// What deleting and recreating the remote repository loses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Loss {
    PullRequests,
    Issues,
    Forks,
    Stars,
    CiSettings,
    Webhooks,
}

impl Loss {
    pub const ALL: [Loss; 6] = [
        Loss::PullRequests,
        Loss::Issues,
        Loss::Forks,
        Loss::Stars,
        Loss::CiSettings,
        Loss::Webhooks,
    ];

    pub fn code(self) -> &'static str {
        match self {
            Self::PullRequests => "loss_pull_requests",
            Self::Issues => "loss_issues",
            Self::Forks => "loss_forks",
            Self::Stars => "loss_stars",
            Self::CiSettings => "loss_ci",
            Self::Webhooks => "loss_webhooks",
        }
    }

    pub fn text(self) -> &'static str {
        match self {
            Self::PullRequests => "pull / merge requests and their review history",
            Self::Issues => "issues and their discussions",
            Self::Forks => "forks, which stay attached to the old history",
            Self::Stars => "stars and watchers",
            Self::CiSettings => "CI settings, secrets and variables",
            Self::Webhooks => "webhooks and deploy keys",
        }
    }
}

/// One way and its instructions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HowtoWay {
    pub way: Way,
    pub steps: Vec<HowStep>,
    /// The exact settings pages, copyable and openable.
    pub links: Vec<String>,
}

/// The three ways through a host's protection, structured (the TUI words it
/// in its own language) and as English text (CLI, MCP, `rewrite note`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Howto {
    pub provider: String,
    /// The protected branches that block.
    pub branches: Vec<String>,
    pub ways: Vec<HowtoWay>,
    /// What way 3 loses.
    pub losses: Vec<Loss>,
}

impl Howto {
    /// The English text of the explanation.
    pub fn text(&self) -> String {
        let names = if self.branches.is_empty() {
            "A branch".to_string()
        } else {
            self.branches.join(", ")
        };
        let mut out = format!(
            "{names} {} protected on {}, so the host refuses the rewritten history. \
             GitPoulpi never changes the host's settings for you; there are three ways \
             through, from the least to the most drastic.",
            if self.branches.len() > 1 { "are" } else { "is" },
            self.provider
        );
        for way in &self.ways {
            out.push_str("\n\n");
            out.push_str(way.way.title());
            for step in &way.steps {
                out.push_str("\n   ");
                out.push_str(step.text());
            }
            for link in &way.links {
                out.push_str("\n   Link: ");
                out.push_str(link);
            }
            if way.way == Way::Recreate {
                out.push_str("\n   This loses: ");
                out.push_str(
                    &self
                        .losses
                        .iter()
                        .map(|l| l.text())
                        .collect::<Vec<_>>()
                        .join("; "),
                );
                out.push('.');
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn github() -> HostProtection {
        HostProtection::new(HostKind::GitHub, "octo/thing", "https://github.com/")
    }

    #[test]
    fn patterns_cover_names_short_or_full_with_a_star_for_any_run() {
        let mut host = github();
        host.patterns = vec!["main".into(), "release/*".into()];
        assert!(host.covers("main") && host.covers("refs/heads/main"));
        assert!(host.covers("release/1.2/x"));
        assert!(!host.covers("mainline") && !host.covers("feature/main"));
        assert!(!github().covers("main"), "nothing known, nothing covered");
    }

    #[test]
    fn the_settings_pages_are_exact_per_provider() {
        assert_eq!(
            github().settings_urls(),
            [
                "https://github.com/octo/thing/settings/branches",
                "https://github.com/octo/thing/settings/rules"
            ]
        );
        let gitlab = HostProtection::new(HostKind::GitLab, "g/sub/p", "https://git.corp.example");
        assert_eq!(
            gitlab.settings_urls(),
            ["https://git.corp.example/g/sub/p/-/settings/repository#js-protected-branches-settings"]
        );
        let azure = HostProtection::new(
            HostKind::AzureDevOps,
            "My Project/Repo",
            "https://dev.azure.com/org",
        );
        assert_eq!(
            azure.settings_urls(),
            ["https://dev.azure.com/org/My%20Project/_settings/repositories"]
        );
        let cloud = HostProtection::new(HostKind::Bitbucket, "ws/repo", "https://bitbucket.org");
        assert_eq!(
            cloud.settings_urls(),
            ["https://bitbucket.org/ws/repo/admin/branch-restrictions"]
        );
        let dc = HostProtection::new(HostKind::Bitbucket, "KEY/slug", "https://bb.corp.example");
        assert_eq!(
            dc.settings_urls(),
            ["https://bb.corp.example/projects/KEY/repos/slug/settings"]
        );
        assert!(HostProtection::new(HostKind::Other, "a/b", "https://x")
            .settings_urls()
            .is_empty());
    }

    #[test]
    fn the_howto_has_three_ways_each_with_its_link_and_the_losses() {
        let howto = github().howto(&["main".to_string()]);
        let ways: Vec<Way> = howto.ways.iter().map(|w| w.way).collect();
        assert_eq!(
            ways,
            [Way::LiftProtection, Way::NewBranchName, Way::Recreate]
        );
        assert_eq!(howto.ways[0].steps, [HowStep::LiftGithub]);
        assert_eq!(
            howto.ways[2].links,
            ["https://github.com/octo/thing/settings"]
        );
        assert_eq!(howto.losses.len(), 6);
        let text = howto.text();
        assert!(text.starts_with("main is protected on GitHub"));
        assert!(text.contains("https://github.com/octo/thing/settings/branches"));
        assert!(text.contains("pull / merge requests") && text.contains("webhooks"));
    }
}
