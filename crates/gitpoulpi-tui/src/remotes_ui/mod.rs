//! Remotes in the TUI: which remote is "the default", the REMOTE
//! section's per-remote menu, the Add / Edit remote popup, the toolbar's
//! Push and Pull dropdowns, Set upstream, and the dialog a diverged push
//! opens.
//!
//! Everything a mouse gesture here can do goes through one
//! [`RemoteUiAction`] and one [`apply_remote_ui_action`], so the
//! right-click, the keyboard (`.` on a remote row, the menus' own keys)
//! and the command palette cannot implement different subsets of it --
//! the same rule `ref_menu.rs` follows for refs.
//!
//! The decisions ("what does the Pull button do right now", "where does
//! Push go") are pure functions of plain data, tested without a terminal
//! or a daemon. `form` is the Add / Edit popup, `ui` the state the event
//! loop holds, `apply` the one place an action runs.

use super::*;

use crate::daemon_link::AutoStashNeeded;
use context_menu::{glyph_label, Entry};
use gitpoulpi_config::{PullButtonMode, PushBehavior};
use gitpoulpi_core::domain::{
    PullMode, PullOptions, PushOptions, PushRejectReason, PushRejection, Ref, Remote,
};
use gitpoulpi_i18n::{msg::remotes as msg, M};

// ---------------------------------------------------------------------
// The default remote
// ---------------------------------------------------------------------

/// The checked-out branch: its name and its upstream (`origin/main`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HeadBranch {
    pub(crate) name: String,
    pub(crate) upstream: Option<String>,
}

/// The checked-out *branch*, or `None` on a detached HEAD.
pub(crate) fn head_branch(refs: &[Ref]) -> Option<HeadBranch> {
    refs.iter().find_map(|r| match &r.kind {
        RefKind::LocalBranch { name } if r.is_head => Some(HeadBranch {
            name: name.clone(),
            upstream: r.upstream.clone(),
        }),
        _ => None,
    })
}

/// A local branch by name, as a [`HeadBranch`] (name + upstream).
pub(crate) fn local_branch(refs: &[Ref], branch: &str) -> Option<HeadBranch> {
    refs.iter().find_map(|r| match &r.kind {
        RefKind::LocalBranch { name } if name == branch => Some(HeadBranch {
            name: name.clone(),
            upstream: r.upstream.clone(),
        }),
        _ => None,
    })
}

/// The remote part of an upstream spelling (`grp/sub/main` -> `grp/sub`),
/// resolved against the actual remote list.
pub(crate) fn upstream_remote(upstream: &str, remotes: &[String]) -> Option<String> {
    gitpoulpi_core::domain::split_remote_ref(upstream, remotes).map(|(remote, _)| remote)
}

/// The repository's default remote, by the one ladder every frontend
/// uses (`gitpoulpi_session::resolve_default_remote`): the user's choice,
/// the checked-out branch's upstream remote, `origin`, the only remote.
pub(crate) fn default_remote(
    chosen: Option<&str>,
    refs: &[Ref],
    remotes: &[String],
) -> Option<String> {
    let upstream = head_branch(refs)
        .and_then(|h| h.upstream)
        .and_then(|u| upstream_remote(&u, remotes));
    gitpoulpi_session::resolve_default_remote(chosen, upstream.as_deref(), remotes)
}

pub(crate) fn repo_state_path(repo_root: &Path) -> PathBuf {
    let state_dir = gitpoulpi_config::Settings::resolve_ambient()
        .map(|s| s.config().session.per_repo_state_dir.clone())
        .unwrap_or_else(|_| gitpoulpi_config::DEFAULT_REPO_DATA_LOCATION.to_string());
    gitpoulpi_session::RepoState::path_for(repo_root, &state_dir)
}

/// The remote chosen with "Set as default remote" for this repository.
pub(crate) fn load_chosen_default_remote(repo_root: &Path) -> Option<String> {
    gitpoulpi_session::RepoState::load(&repo_state_path(repo_root))
        .ok()
        .and_then(|s| s.default_remote)
}

/// Load-modify-save, leaving every other field of `repo-state.toml`
/// alone (the same rule `refs_tree::save_collapsed` follows).
pub(crate) fn save_chosen_default_remote(repo_root: &Path, chosen: Option<&str>) {
    let path = repo_state_path(repo_root);
    let mut state = gitpoulpi_session::RepoState::load(&path).unwrap_or_default();
    state.default_remote = chosen.map(str::to_string);
    let _ = state.save(&path);
}

/// The Pull button's mode and the Push behavior, read from
/// `config.toml` when a button is pressed rather than cached: the
/// Preferences screen and the Pull dropdown both write the file, and a
/// cached copy is a second place for the answer to live.
pub(crate) fn sync_settings() -> (PullButtonMode, PushBehavior) {
    gitpoulpi_config::Settings::resolve_ambient()
        .map(|s| (s.config().git.pull_mode, s.config().git.push_behavior))
        .unwrap_or_default()
}

// ---------------------------------------------------------------------
// URLs and names
// ---------------------------------------------------------------------

/// The web page of the repository a remote URL points at, or `None` when
/// the URL is not one a browser can open (a local path, `file://`).
///
/// `git@github.com:owner/repo.git`, `ssh://git@host:22/owner/repo.git`
/// and `https://user@host/owner/repo.git` all become
/// `https://host/owner/repo` - what "Copy link to remote"
/// puts on the clipboard. Pure string work: no hosting API, no network.
pub(crate) fn remote_web_url(url: &str) -> Option<String> {
    let url = url.trim();
    let (scheme, host, path) = split_remote_url(url)?;
    let path = path.trim_matches('/');
    let path = path
        .strip_suffix(".git")
        .unwrap_or(path)
        .trim_end_matches('/');
    if path.is_empty() {
        return None;
    }
    // Azure DevOps' SSH form names the org and project differently from
    // its web form: `git@ssh.dev.azure.com:v3/org/project/repo` is
    // browsed at `https://dev.azure.com/org/project/_git/repo`.
    if host == "ssh.dev.azure.com" {
        let parts: Vec<&str> = path.split('/').collect();
        if let ["v3", org, project, repo] = parts.as_slice() {
            return Some(format!("https://dev.azure.com/{org}/{project}/_git/{repo}"));
        }
    }
    let web_scheme = if scheme == "http" { "http" } else { "https" };
    Some(format!("{web_scheme}://{host}/{path}"))
}

/// `(scheme, host, path)` of a remote URL git accepts over the network:
/// `http(s)://`, `ssh://`, `git://` and the scp-like `[user@]host:path`.
/// The host loses any user and port.
pub(crate) fn split_remote_url(url: &str) -> Option<(&str, &str, &str)> {
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

/// Accepts what a remote URL typed into the popup may be: `http(s)://`,
/// `ssh://`, `git://`, scp-like `git@host:owner/repo.git`, or a local
/// `file://` URL. Deliberately simple - git is the real judge, and the
/// point here is to catch a paste of the wrong thing before it is saved.
pub(crate) fn validate_remote_url(url: &str) -> Result<(), String> {
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
pub(crate) fn validate_remote_name(
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

// ---------------------------------------------------------------------
// Actions and the menus that offer them
// ---------------------------------------------------------------------

/// Everything the remote UI can do, from any of its surfaces. Carries
/// its own data, so a menu row is self-contained and the palette can
/// build the same value without a menu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum RemoteUiAction {
    CopyRemoteName(String),
    CopyRemoteLink(String),
    SetDefaultRemote(String),
    EditRemote(String),
    RemoveRemote(String),
    /// A leftover namespace's menu row: delete its remote-tracking branches
    /// (what `RemoveRemote` of an unconfigured name does, asked more
    /// plainly).
    RemoveLeftover(String),
    /// A leftover namespace's menu row: the Add remote form with this name
    /// filled in.
    AddAsRemote(String),
    FetchRemote(String),
    AddRemote,
    /// The Pull dropdown's "Fetch All": runs at once.
    FetchAllRemotes,
    /// The Pull dropdown's pull items: run at once. The button itself
    /// stays "Pull" and keeps following `git.pull_mode`.
    RunPull(PullButtonMode),
    /// A row of the toolbar's `LFS` dropdown: runs at once.
    Lfs(crate::lfs_ui::LfsMenuItem),
    /// A row of the toolbar's `Flow` dropdown: the window opens the dialog
    /// (or runs the toggle) at its next notice pass.
    Flow(crate::flow::FlowCmd),
    /// Push the checked-out branch to this remote, once.
    PushTo(String),
    SetUpstream {
        branch: String,
        upstream: Option<String>,
    },
    /// Open a prompt to type the upstream by hand.
    TypeUpstream {
        branch: String,
    },
    /// The diverged-push dialog's three answers.
    PullThenPush {
        remote: String,
        branch: String,
    },
    /// The auto-stash dialog's yes: resend the refused operation, wrapped
    /// so the daemon stashes what blocks it, runs it and pops the stash.
    AutoStashContinue,
    /// The same, after writing `general.auto_stash = "always"` so the next
    /// blocked checkout, pull, merge or rebase does not ask again.
    AutoStashAlways,
    /// Always `--force-with-lease`. `after_fetch` is the diverged dialog's
    /// form: a rejected push leaves our remote-tracking ref behind the
    /// remote, so a lease against it could only ever be refused; the
    /// dialog fetches first and leases against what it just saw (which
    /// the confirmation spells out). The palette's plain force push
    /// leases against the last fetch, as `git push --force-with-lease`
    /// does.
    ForcePush {
        remote: String,
        branch: String,
        after_fetch: bool,
    },
    /// Hide the whole remote from the graph -- the remote-name row's eye.
    /// A pattern, not an expansion: the remote's branches fetched later are
    /// hidden too, and the commits only they reach leave the Log
    /// (`refs_tree::set_remote_hidden`).
    HideRemote(String),
    /// The complement: show the remote again, with every branch of it that
    /// was hidden one by one.
    ShowRemote(String),
    /// The push menu's "Signs in as: ..." row: open this repository's SSH &
    /// credentials page.
    OpenRepoAuth,
    Cancel,
}

impl RemoteUiAction {
    /// What hovering this menu row explains (the toolbar's Pull / Push
    /// dropdown items first). `None` for rows whose label says it all.
    pub(crate) fn tooltip(&self) -> Option<String> {
        Some(match self {
            RemoteUiAction::FetchAllRemotes => msg::TIP_FETCH_ALL.get().into(),
            RemoteUiAction::RunPull(PullButtonMode::Merge) => msg::TIP_PULL_MERGE.get().into(),
            RemoteUiAction::RunPull(PullButtonMode::FfOnly) => msg::TIP_PULL_FF_ONLY.get().into(),
            RemoteUiAction::RunPull(PullButtonMode::Rebase) => msg::TIP_PULL_REBASE.get().into(),
            RemoteUiAction::PushTo(remote) => msg::tip_push_to(remote),
            RemoteUiAction::FetchRemote(remote) => msg::tip_fetch_remote(remote),
            RemoteUiAction::SetDefaultRemote(remote) => msg::tip_set_default(remote),
            RemoteUiAction::AddRemote => msg::TIP_ADD_REMOTE.get().into(),
            _ => return None,
        })
    }
}

/// The remote-name row's menu: copy the name or the web
/// link, fetch, make it the default, edit, remove.
pub(crate) fn remote_menu_entries(
    name: &str,
    is_default: bool,
    hidden: bool,
) -> Vec<Entry<RemoteUiAction>> {
    let mut entries = vec![Entry::action(
        glyph_label("\u{27F2}", msg::fetch_remote(name)),
        RemoteUiAction::FetchRemote(name.to_string()),
    )];
    entries.push(Entry::Separator);
    entries.push(Entry::action(
        glyph_label("\u{2398}", msg::COPY_NAME.get()),
        RemoteUiAction::CopyRemoteName(name.to_string()),
    ));
    entries.push(Entry::action(
        glyph_label("\u{2398}", msg::COPY_LINK.get()),
        RemoteUiAction::CopyRemoteLink(name.to_string()),
    ));
    entries.push(Entry::Separator);
    if !is_default {
        entries.push(Entry::action(
            glyph_label("\u{2713}", msg::SET_DEFAULT.get()),
            RemoteUiAction::SetDefaultRemote(name.to_string()),
        ));
    }
    entries.push(Entry::action(
        glyph_label("\u{270E}", msg::edit_remote(name)),
        RemoteUiAction::EditRemote(name.to_string()),
    ));
    entries.push(Entry::Separator);
    // The remote row's eye, as a menu row: one row that flips with the
    // remote's state (a hidden remote is a remembered fact now, not a set
    // of branches, so there is a state to report).
    entries.push(if hidden {
        Entry::action(
            glyph_label(hide_in_graph_icon!(), msg::SHOW_REMOTE_IN_GRAPH.get()),
            RemoteUiAction::ShowRemote(name.to_string()),
        )
    } else {
        Entry::action(
            glyph_label(hide_in_graph_icon!(), msg::HIDE_REMOTE_IN_GRAPH.get()),
            RemoteUiAction::HideRemote(name.to_string()),
        )
    });
    entries.push(Entry::danger(
        glyph_label("\u{2717}", msg::remove_remote(name)),
        RemoteUiAction::RemoveRemote(name.to_string()),
    ));
    entries
}

/// The names the REMOTE section shows a root row for that no remote is
/// configured for: remote-tracking branches whose remote went (the
/// repository was a fork and is not one any more). Sorted. Empty until the
/// remote list has been read (`loaded`): before that every remote would look
/// like one.
pub(crate) fn leftover_names(
    refs: &[Ref],
    configured: &[String],
    loaded: bool,
) -> std::collections::BTreeSet<String> {
    if !loaded {
        return Default::default();
    }
    refs.iter()
        .filter_map(|r| match &r.kind {
            RefKind::RemoteBranch { remote, .. } if !configured.contains(remote) => {
                Some(remote.clone())
            }
            _ => None,
        })
        .collect()
}

/// The menu of a leftover namespace: nothing here can fetch, push or edit
/// an address that does not exist, so it offers the two things that make
/// sense - drop the branches, or make it a remote again.
pub(crate) fn leftover_menu_entries(name: &str, hidden: bool) -> Vec<Entry<RemoteUiAction>> {
    let mut entries = vec![
        Entry::action(
            glyph_label("\u{2398}", msg::COPY_NAME.get()),
            RemoteUiAction::CopyRemoteName(name.to_string()),
        ),
        Entry::Separator,
        Entry::action(
            glyph_label("\u{270E}", msg::add_as_remote(name)),
            RemoteUiAction::AddAsRemote(name.to_string()),
        ),
        Entry::Separator,
    ];
    entries.push(if hidden {
        Entry::action(
            glyph_label(hide_in_graph_icon!(), msg::SHOW_REMOTE_IN_GRAPH.get()),
            RemoteUiAction::ShowRemote(name.to_string()),
        )
    } else {
        Entry::action(
            glyph_label(hide_in_graph_icon!(), msg::HIDE_REMOTE_IN_GRAPH.get()),
            RemoteUiAction::HideRemote(name.to_string()),
        )
    });
    entries.push(Entry::danger(
        glyph_label("\u{2717}", msg::REMOVE_LEFTOVER.get()),
        RemoteUiAction::RemoveLeftover(name.to_string()),
    ));
    entries
}

/// A menu listing every remote, each row running `action(name)` - the
/// palette's "which remote?" step for the remote commands.
pub(crate) fn remote_chooser_entries(
    remotes: &[String],
    leftovers: &[String],
    default: Option<&str>,
    action: impl Fn(String) -> RemoteUiAction,
) -> Vec<Entry<RemoteUiAction>> {
    remotes
        .iter()
        .map(|r| {
            let label = if Some(r.as_str()) == default {
                format!("{r}  {}", msg::DEFAULT_MARK.get())
            } else {
                r.clone()
            };
            Entry::action(label, action(r.clone()))
        })
        .chain(leftovers.iter().map(|r| {
            Entry::action(
                format!("{r}  {}", msg::LEFTOVER_NOTE.get()),
                action(r.clone()),
            )
        }))
        .collect()
}

/// The radio mark a dropdown row carries. U+25C9 / U+25CB: both already
/// drawn elsewhere in this UI (solo, graph nodes), so their width in the
/// user's font is known.
fn radio(on: bool) -> &'static str {
    if on {
        "\u{25C9}"
    } else {
        "\u{25CB}"
    }
}

/// The dropdown's wording of a pull mode (the conventional wording, translated).
pub(crate) fn pull_mode_label(mode: PullButtonMode) -> M {
    match mode {
        PullButtonMode::Merge => msg::PULL_MERGE,
        PullButtonMode::FfOnly => msg::PULL_FF_ONLY,
        PullButtonMode::Rebase => msg::PULL_REBASE,
    }
}

/// The Pull dropdown: Fetch All and the pull variants, each of
/// which runs when chosen (the button is always "Pull").
pub(crate) fn pull_mode_entries() -> Vec<Entry<RemoteUiAction>> {
    std::iter::once(Entry::action(
        glyph_label("\u{21BB}", msg::FETCH_ALL.get()),
        RemoteUiAction::FetchAllRemotes,
    ))
    .chain(PullButtonMode::ALL.iter().map(|m| {
        Entry::action(
            glyph_label("\u{21E9}", pull_mode_label(*m).get()),
            RemoteUiAction::RunPull(*m),
        )
    }))
    .collect()
}

/// The toolbar's `Flow` dropdown: one row per command (`flow::menu_rows`).
pub(crate) fn flow_menu_entries() -> Vec<Entry<RemoteUiAction>> {
    crate::flow::menu_rows()
        .into_iter()
        .map(|(label, cmd)| Entry::action(label, RemoteUiAction::Flow(cmd)))
        .collect()
}

/// The toolbar's `LFS` dropdown.
pub(crate) fn lfs_menu_entries() -> Vec<Entry<RemoteUiAction>> {
    crate::lfs_ui::LfsMenuItem::ALL
        .iter()
        .map(|item| {
            Entry::action(
                glyph_label(lfs_menu_glyph(*item), item.label().get()),
                RemoteUiAction::Lfs(*item),
            )
        })
        .collect()
}

fn lfs_menu_glyph(item: crate::lfs_ui::LfsMenuItem) -> &'static str {
    use crate::lfs_ui::LfsMenuItem as I;
    match item {
        I::FetchAll => "\u{21BB}",
        I::Checkout => "\u{2713}",
        I::Pull => "\u{21E9}",
        I::PushAll => "\u{21E7}",
        I::Prune => "\u{2715}",
    }
}

/// "Push <branch> to ...": every remote, the default one first.
pub(crate) fn push_to_entries(
    remotes: &[String],
    default: Option<&str>,
) -> Vec<Entry<RemoteUiAction>> {
    let mut ordered: Vec<&String> = remotes.iter().collect();
    ordered.sort_by_key(|r| Some(r.as_str()) != default);
    ordered
        .into_iter()
        .map(|r| {
            let label = if Some(r.as_str()) == default {
                format!("\u{21E7} {r}  {}", msg::DEFAULT_MARK.get())
            } else {
                format!("\u{21E7} {r}")
            };
            Entry::action(label, RemoteUiAction::PushTo(r.clone()))
        })
        .collect()
}

/// How many remote branches one remote's submenu lists. The menu does
/// not scroll, so a remote with hundreds of branches would run off the
/// screen; the rest are reachable through "Type a remote branch...".
const UPSTREAM_MENU_LIMIT: usize = 20;

/// "Set upstream of <branch>": one submenu per remote listing its
/// branches (the same-named one first), a way to type one, and "Unset"
/// when there is an upstream to remove.
pub(crate) fn upstream_entries(
    branch: &str,
    refs: &[Ref],
    remotes: &[String],
    current: Option<&str>,
) -> Vec<Entry<RemoteUiAction>> {
    let mut entries = Vec::new();
    for remote in remotes {
        let mut names: Vec<&str> = refs
            .iter()
            .filter_map(|r| match &r.kind {
                RefKind::RemoteBranch { remote: rm, name } if rm == remote && name != "HEAD" => {
                    Some(name.as_str())
                }
                _ => None,
            })
            .collect();
        names.sort_by_key(|n| (*n != branch, n.to_string()));
        let items: Vec<Entry<RemoteUiAction>> = names
            .into_iter()
            .take(UPSTREAM_MENU_LIMIT)
            .map(|n| {
                let full = format!("{remote}/{n}");
                let label = format!("{} {full}", radio(current == Some(full.as_str())));
                Entry::action(
                    label,
                    RemoteUiAction::SetUpstream {
                        branch: branch.to_string(),
                        upstream: Some(full),
                    },
                )
            })
            .collect();
        if !items.is_empty() {
            entries.push(Entry::submenu(remote.clone(), items));
        }
    }
    entries.push(Entry::Separator);
    entries.push(Entry::action(
        glyph_label("\u{270E}", msg::TYPE_REMOTE_BRANCH.get()),
        RemoteUiAction::TypeUpstream {
            branch: branch.to_string(),
        },
    ));
    if current.is_some() {
        entries.push(Entry::action(
            glyph_label("\u{2717}", msg::UNSET_UPSTREAM.get()),
            RemoteUiAction::SetUpstream {
                branch: branch.to_string(),
                upstream: None,
            },
        ));
    }
    entries
}

/// The dialog a push rejected as non-fast-forward opens:
/// pull first, force push (always with lease, and confirmed), or cancel.
pub(crate) fn diverged_entries(rejection: &PushRejection) -> Vec<Entry<RemoteUiAction>> {
    let remote = rejection.remote.clone();
    let branch = rejection.branch.clone();
    vec![
        Entry::action(
            glyph_label("\u{21E9}", msg::PULL_THEN_PUSH.get()),
            RemoteUiAction::PullThenPush {
                remote: remote.clone(),
                branch: branch.clone(),
            },
        ),
        Entry::danger(
            glyph_label("\u{21E7}", msg::FORCE_PUSH_LEASE.get()),
            RemoteUiAction::ForcePush {
                remote,
                branch,
                after_fetch: true,
            },
        ),
        Entry::Separator,
        Entry::action(
            gitpoulpi_i18n::msg::dialog::CANCEL.get(),
            RemoteUiAction::Cancel,
        ),
    ]
}

// ---------------------------------------------------------------------
// What the buttons do
// ---------------------------------------------------------------------

/// The action the Pull button (and `Ctrl+G p`) runs in `mode`, with its
/// status line - or why it cannot run.
///
/// A branch with no upstream pulls its same-named branch from the
/// default remote, rather than failing the way a bare `git pull` would.
pub(crate) fn pull_action(
    mode: PullButtonMode,
    head: Option<&HeadBranch>,
    default_remote: Option<&str>,
) -> Result<(Action, String), String> {
    let mode = match mode {
        PullButtonMode::Merge => PullMode::Merge,
        PullButtonMode::FfOnly => PullMode::FastForwardOnly,
        PullButtonMode::Rebase => PullMode::Rebase,
    };
    let head = head.ok_or_else(|| msg::DETACHED_PULL.get().to_string())?;
    let remote = if head.upstream.is_some() {
        None
    } else {
        Some(
            default_remote
                .map(str::to_string)
                .ok_or_else(|| msg::no_upstream_no_default(&head.name))?,
        )
    };
    let from = match (&head.upstream, &remote) {
        (Some(up), _) => up.clone(),
        (None, Some(r)) => format!("{r}/{}", head.name),
        (None, None) => unreachable!("handled above"),
    };
    Ok((
        Action::Pull(PullOptions { mode, remote }),
        msg::pulling_from(from, &head.name),
    ))
}

/// Pushing `branch` to `remote`, setting the upstream when the branch
/// has none (the first push of a new branch).
pub(crate) fn push_action(remote: &str, branch: &HeadBranch, force_with_lease: bool) -> Action {
    Action::Push {
        remote: remote.to_string(),
        opts: PushOptions {
            force_with_lease,
            set_upstream: branch.upstream.is_none(),
            branch: Some(branch.name.clone()),
        },
    }
}

/// Where a branch's own Push goes: its upstream's remote, else the
/// default remote.
pub(crate) fn remote_for_branch(
    branch: &HeadBranch,
    remotes: &[String],
    default_remote: Option<&str>,
) -> Option<String> {
    branch
        .upstream
        .as_deref()
        .and_then(|u| upstream_remote(u, remotes))
        .or_else(|| default_remote.map(str::to_string))
}

/// What the Push button (and `Ctrl+G P`) does.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum PushPlan {
    /// Push now.
    Direct(Action, String),
    /// Open the push-to menu, the default remote preselected.
    Ask,
    /// Nothing can be pushed; say why.
    Refuse(String),
}

pub(crate) fn push_plan(
    behavior: PushBehavior,
    head: Option<&HeadBranch>,
    default_remote: Option<&str>,
    has_remotes: bool,
) -> PushPlan {
    let Some(head) = head else {
        return PushPlan::Refuse(msg::DETACHED_PUSH.get().into());
    };
    if !has_remotes {
        return PushPlan::Refuse(msg::NO_REMOTE_ADD.get().into());
    }
    match (behavior, default_remote) {
        (PushBehavior::Direct, Some(remote)) => PushPlan::Direct(
            push_action(remote, head, false),
            msg::pushing_to(&head.name, remote),
        ),
        // No default (several remotes, none `origin`): ask rather than
        // guess, whatever the setting says.
        _ => PushPlan::Ask,
    }
}

mod apply;
mod form;
mod ui;

pub(crate) use apply::*;
pub(crate) use form::*;
pub(crate) use ui::*;

#[cfg(test)]
mod tests;
