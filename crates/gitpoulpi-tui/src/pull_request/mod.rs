//! Start a pull request: from a local
//! branch's ref-menu row, either open the host's compare/new-request page
//! directly in a browser (no token, or no integration matches the
//! remote), or fill a small form and call the host's API through
//! `gitpoulpi-hosting`.
//!
//! No daemon involvement, by design (see the architect's plan): this is
//! the same shape as the Preferences screen's token store and
//! `gitpoulpi hosting` itself -- an adapter the TUI drives directly,
//! never state the other frontends need to agree on.
//!
//! # `[[integrations]]`
//!
//! `load_integrations` reads the same `config.toml` table
//! `gitpoulpi-cli`'s `hosting_cmd` does, with the same forward-compatible
//! parser (`gitpoulpi-config` does not carry the table yet -- see that
//! module's own comment). When the config crate grows the real type this
//! is replaced by `settings.config().integrations` and nothing else here
//! changes.

use super::*;
use crate::keymap::Action as Bind;
use crate::redraw::NotifySend as _;
use gitpoulpi_i18n::msg;

use std::sync::mpsc as std_mpsc;

use crossterm::event::MouseEvent;
use gitpoulpi_core::domain::{Ref, RefKind, Remote};
use gitpoulpi_hosting::{
    client_for, new_pull_request_url, remote_web_base, well_known_kind, CreatedPullRequest,
    DefaultTokenSource, Integration, NewPullRequest, TokenSource,
};

use crate::combo::{Combo, ComboOption, ComboOutcome};
use crate::dialog_kit::{self, ButtonState, CheckState};
use crate::hosting_task::{start_projects, ClientFactory, ProjectsReceiver, ProjectsState};
use crate::meta_picker::{self, MetaField, MetaFields, MetaSelection, MetaTarget};

/// The configured integrations, via the shared settings loader (legacy
/// kinds migrated). Empty rather than an error: nothing configured is a
/// legitimate, common state.
pub(crate) fn load_integrations() -> Vec<Integration> {
    gitpoulpi_config::Settings::resolve_ambient()
        .map(|settings| settings.config().integrations.clone())
        .unwrap_or_default()
}

/// `MenuContext::for_ref` plus `pr_reason`, computed from `remotes` and
/// the configured integrations. A thin wrapper rather than folding the
/// computation into `for_ref` itself: `for_ref` is defined in
/// `log_menu.rs`, shared with the commit menu, and has no reason to take
/// `remotes` or read `config.toml` for menus that are not about a
/// branch.
pub(crate) fn menu_context_for_ref(
    refs: &gitpoulpi_core::Result<Vec<Ref>>,
    log: &gitpoulpi_core::Result<gitpoulpi_core::domain::CommitGraph>,
    section: SidebarSection,
    name: &str,
    remotes: &[Remote],
) -> MenuContext {
    let mut ctx = MenuContext::for_ref(refs, log, section, name);
    if section.ref_kind() == Some(RefRowKind::Local) {
        ctx.pr_reason = pull_request_reason(name, refs, remotes, &load_integrations());
    }
    ctx
}

/// Why "Start a pull request" is disabled for `branch`, or `None` to
/// offer it: no upstream (never pushed), or a remote whose host
/// `gitpoulpi-hosting` cannot address (no matching `[[integrations]]`
/// entry and not a github.com/gitlab.com hostname).
///
/// Never looks a token up: this runs on the render thread every time a
/// local branch's menu opens, and the token lookup spawns the git
/// credential helper (Git Credential Manager on Windows: seconds). A
/// token only picks between the form and the direct URL, both offered.
pub(crate) fn pull_request_reason(
    branch: &str,
    refs: &gitpoulpi_core::Result<Vec<Ref>>,
    remotes: &[Remote],
    integrations: &[Integration],
) -> Option<String> {
    let refs = refs.as_ref().ok()?;
    locate(branch, refs, remotes, integrations).err()
}

/// One provider tab of the Create Pull Request form: a connected GitHub /
/// GitLab profile.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PrTab {
    pub(crate) integration: Integration,
    /// `GitHub`, or `GitLab · corp` when two tabs would read the same.
    pub(crate) label: String,
}

/// What opens the Create Pull Request form.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PullRequestSeed {
    /// The supported providers, one tab each.
    pub(crate) tabs: Vec<PrTab>,
    /// The tab the repository's remote belongs to.
    pub(crate) tab: usize,
    /// The local branch the form was opened for (may be empty: detached).
    pub(crate) local_branch: String,
    /// The repository the remote points at, when a tab answers for it.
    pub(crate) project: Option<String>,
    /// The branch as the remote spells it (the shorthand after the
    /// remote name in `Ref::upstream`), which is what the host's API
    /// wants as `head`/`source_branch`; `None` until it is pushed.
    pub(crate) source_branch: Option<String>,
    /// The remote's branches, for both branch dropdowns.
    pub(crate) branches: Vec<String>,
    /// The branch to merge into, by default.
    pub(crate) target: String,
}

/// The profile that answers for `remote`: the shared resolution rule
/// (`gitpoulpi_hosting::resolve_for_repo`), which ignores ports - an
/// instance's SSH and HTTPS remotes differ in them.
pub(crate) fn matching_integration<'a>(
    remote: &Remote,
    integrations: &'a [Integration],
) -> Option<&'a Integration> {
    gitpoulpi_hosting::resolve_for_repo(
        &[gitpoulpi_hosting::ResolveRemote::new(
            remote.name.clone(),
            remote.fetch_url.clone(),
        )],
        integrations,
        None,
    )
    .map(|r| r.integration)
}

/// The tabs the form offers: every enabled hosting profile with a client
/// (GitHub and GitLab families - nothing unsupported, not even grayed).
pub(crate) fn pr_tabs(integrations: &[Integration]) -> Vec<PrTab> {
    let usable: Vec<&Integration> = integrations
        .iter()
        .filter(|i| {
            let caps = i.kind.capabilities();
            i.enabled
                && caps.implemented
                && crate::build_features::kind_built(i.kind)
                && caps.pull_requests
        })
        .collect();
    usable
        .iter()
        .map(|i| {
            let name = crate::hosting_ui::provider_name(i.kind);
            let clash = usable
                .iter()
                .filter(|o| crate::hosting_ui::provider_name(o.kind) == name)
                .count()
                > 1;
            let label = if clash {
                let own = if i.name.trim().is_empty() {
                    i.host().unwrap_or_else(|| i.id.clone())
                } else {
                    i.name.clone()
                };
                format!("{name} \u{00B7} {own}")
            } else {
                name.to_string()
            };
            PrTab {
                integration: (*i).clone(),
                label,
            }
        })
        .collect()
}

/// A seed for `branch` (the checked-out one, or the one a ref menu was
/// opened on) over the configured `integrations`.
pub(crate) fn build_seed(
    branch: &str,
    refs: &[Ref],
    remotes: &[Remote],
    integrations: &[Integration],
) -> Result<PullRequestSeed, String> {
    let tabs = pr_tabs(integrations);
    if tabs.is_empty() {
        return Err(msg::pull_request::CONNECT_FIRST.get().to_string());
    }
    let remote_names: Vec<String> = remotes.iter().map(|r| r.name.clone()).collect();
    let upstream = refs
        .iter()
        .find(|r| matches!(&r.kind, RefKind::LocalBranch { name } if name == branch))
        .and_then(|r| r.upstream.as_deref())
        .and_then(|u| gitpoulpi_core::domain::split_remote_ref(u, &remote_names));
    // The remote the form is about: the branch's own, else `origin`, else
    // the first.
    let remote = upstream
        .as_ref()
        .and_then(|(name, _)| remotes.iter().find(|r| &r.name == name))
        .or_else(|| remotes.iter().find(|r| r.name == "origin"))
        .or_else(|| remotes.first());
    let (tab, project) = match remote {
        Some(remote) => {
            let project = remote_web_base(&remote.fetch_url).map(|(_, p)| p);
            let matched = matching_integration(remote, integrations);
            let tab = matched
                .and_then(|i| tabs.iter().position(|t| t.integration.id == i.id))
                .unwrap_or(0);
            (tab, project.filter(|_| matched.is_some()))
        }
        None => (0, None),
    };
    let remote_name = remote.map(|r| r.name.clone());
    let mut branches: Vec<String> = refs
        .iter()
        .filter_map(|r| match &r.kind {
            RefKind::RemoteBranch { remote, name }
                if Some(remote) == remote_name.as_ref() && name != "HEAD" =>
            {
                Some(name.clone())
            }
            _ => None,
        })
        .collect();
    branches.sort();
    branches.dedup();
    let target = remote_name
        .as_deref()
        .and_then(|r| default_target(refs, r))
        .filter(|t| t != "HEAD")
        .or_else(|| {
            ["main", "master", "develop"]
                .into_iter()
                .find(|n| branches.iter().any(|b| b == n))
                .map(str::to_string)
        })
        .unwrap_or_else(|| "main".to_string());
    Ok(PullRequestSeed {
        tabs,
        tab,
        local_branch: branch.to_string(),
        project,
        source_branch: upstream.as_ref().map(|(_, b)| b.to_string()),
        branches,
        target,
    })
}

/// What resolving a branch into a pull request found.
#[allow(clippy::large_enum_variant)] // short-lived, one per click
enum Resolution {
    /// No upstream, or a remote host `gitpoulpi-hosting` cannot address.
    Blocked(String),
    /// A matched integration with a usable token: open the form.
    Ready(PullRequestSeed),
    /// No integration matched, or it has no token: the compare/new-
    /// request URL to open directly instead.
    DirectUrl(String),
}

/// Where a branch's pull request would go, found without any I/O.
struct Located<'a> {
    integration: Option<Integration>,
    kind: gitpoulpi_config::IntegrationKind,
    web_base: String,
    project: String,
    source_branch: &'a str,
    target: String,
}

/// The cheap half of `resolve`: everything but the token, so the ref menu
/// can ask "blocked?" without spawning the credential helper. `Err` is the
/// reason the pull request is blocked.
fn locate<'a>(
    branch: &str,
    refs: &'a [Ref],
    remotes: &[Remote],
    integrations: &[Integration],
) -> Result<Located<'a>, String> {
    let Some(this) = refs
        .iter()
        .find(|r| matches!(&r.kind, RefKind::LocalBranch { name } if name == branch))
    else {
        return Err(crate::git_actions::gone(branch));
    };
    let Some(upstream) = &this.upstream else {
        return Err(msg::pull_request::push_first(branch));
    };
    let remote_names: Vec<String> = remotes.iter().map(|r| r.name.clone()).collect();
    let Some((remote_name, source_branch)) =
        gitpoulpi_core::domain::split_remote_ref(upstream, &remote_names)
    else {
        return Err(msg::pull_request::push_first(branch));
    };
    let Some(remote) = remotes.iter().find(|r| r.name == remote_name) else {
        return Err(msg::pull_request::push_first(branch));
    };
    let Some((web_base, project)) = remote_web_base(&remote.fetch_url) else {
        return Err(msg::pull_request::HOST_UNKNOWN.get().into());
    };
    let integration = matching_integration(remote, integrations).cloned();
    let web_base = gitpoulpi_hosting::web_base_for(integration.as_ref(), web_base);
    let kind = integration
        .as_ref()
        .map(|i| i.kind)
        .or_else(|| well_known_kind(&web_base));
    let Some(kind) = kind else {
        return Err(msg::pull_request::HOST_UNKNOWN.get().into());
    };
    let target = default_target(refs, &remote_name)
        .filter(|t| t != "HEAD")
        .unwrap_or_else(|| "main".to_string());
    Ok(Located {
        integration,
        kind,
        web_base,
        project,
        source_branch,
        target,
    })
}

fn resolve(
    branch: &str,
    refs: &[Ref],
    remotes: &[Remote],
    integrations: &[Integration],
    token_source: &dyn TokenSource,
) -> Resolution {
    let located = match locate(branch, refs, remotes, integrations) {
        Ok(located) => located,
        Err(reason) => return Resolution::Blocked(reason),
    };
    if let Some(integration) = &located.integration {
        if token_source.token(integration).is_ok() {
            if let Ok(seed) = build_seed(branch, refs, remotes, integrations) {
                return Resolution::Ready(seed);
            }
        }
    }
    Resolution::DirectUrl(new_pull_request_url(
        located.kind,
        &located.web_base,
        &located.project,
        located.source_branch,
        Some(&located.target),
    ))
}

/// Guesses the remote's default branch from the refs already loaded: a
/// remote-tracking `HEAD` when the backend surfaces one, else
/// `main`/`master` among that remote's branches, in that order.
fn default_target(refs: &[Ref], remote: &str) -> Option<String> {
    let has = |name: &str| {
        refs.iter().any(|r| {
            matches!(&r.kind, RefKind::RemoteBranch { remote: rm, name: n } if rm == remote && n == name)
        })
    };
    if has("HEAD") {
        return Some("HEAD".to_string());
    }
    ["main", "master"]
        .into_iter()
        .find(|n| has(n))
        .map(str::to_string)
}

/// Runs "Start a pull request" for `branch`: opens a browser directly, or
/// leaves `seed` for `repo_dialogs` to open the form from. Always
/// returns a status-line message.
pub(crate) fn start_pull_request_now(
    branch: &str,
    refs: &[Ref],
    remotes: &[Remote],
    seed: &mut Option<PullRequestSeed>,
) -> String {
    let integrations = load_integrations();
    match resolve(
        branch,
        refs,
        remotes,
        &integrations,
        &DefaultTokenSource::for_integrations(&integrations),
    ) {
        Resolution::Blocked(reason) => reason,
        Resolution::DirectUrl(url) => match crate::repo_menu::open_url(&url) {
            Ok(()) => msg::pull_request::opened_in_browser(&url),
            Err(e) => crate::activity_log::report_problem_clear(
                crate::activity_log::ProblemSource::Tool,
                &e,
            ),
        },
        Resolution::Ready(found) => {
            *seed = Some(found);
            msg::pull_request::OPENING_FORM.get().to_string()
        }
    }
}

/// The header's `+` / `n` / palette: the Create Pull Request form for the
/// checked-out branch, whatever its state (not pushed yet is said in the
/// form), over every connected provider.
pub(crate) fn create_pull_request_now(
    refs: &[Ref],
    remotes: &[Remote],
    seed: &mut Option<PullRequestSeed>,
) -> String {
    let head = refs.iter().find_map(|r| match &r.kind {
        RefKind::LocalBranch { name } if r.is_head => Some(name.clone()),
        _ => None,
    });
    match build_seed(
        head.as_deref().unwrap_or(""),
        refs,
        remotes,
        &load_integrations(),
    ) {
        Ok(found) => {
            *seed = Some(found);
            msg::pull_request::OPENING_FORM.get().to_string()
        }
        Err(reason) => reason,
    }
}

mod dialog;
mod render;

pub(crate) use dialog::*;
pub(crate) use render::*;

#[cfg(test)]
#[cfg(all(feature = "github", feature = "gitlab"))]
mod tests;
