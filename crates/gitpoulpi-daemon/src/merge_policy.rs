//! The fast-forward policy of a merge the user starts (`git.merge_fast_forward`
//! and the repository's own override on the Merge options page).
//!
//! A merge request names its policy (`opts.ff`, the CLI's `--ff` / `--no-ff` /
//! `--ff-only`) or leaves it out; the daemon resolves a missing one here, so
//! the TUI, the CLI and MCP all follow the same setting. A request that sets
//! `no_ff` or `fast_forward_only` itself (the flows do) is taken as it is.
//!
//! `allowed` passes no flag at all: it is git's own behavior, so the
//! repository's `merge.ff` (when set) still applies there. An explicit
//! `never` / `only` overrides `merge.ff` with the flag.

use gitpoulpi_core::domain::{MergeFastForward, MergeOptions, RepoRef};
use gitpoulpi_session::RepoState;

/// The policy in force for `repo`: the repository's override, else the
/// global setting.
pub(crate) fn effective(repo: &RepoRef) -> MergeFastForward {
    let settings = gitpoulpi_config::Settings::resolve_ambient().ok();
    let global = settings
        .as_ref()
        .map(|s| s.config().git.merge_fast_forward)
        .unwrap_or_default();
    let repo_choice = RepoState::load(&crate::autostash::state_path(repo))
        .ok()
        .and_then(|s| s.preferences.merge_fast_forward);
    MergeFastForward::resolve(repo_choice.as_deref(), global)
}

/// `opts` with its fast-forward policy turned into git's flags.
pub(crate) fn apply(repo: &RepoRef, mut opts: MergeOptions) -> MergeOptions {
    if opts.no_ff || opts.fast_forward_only || opts.squash {
        return opts;
    }
    match opts.ff.unwrap_or_else(|| effective(repo)) {
        MergeFastForward::Never => opts.no_ff = true,
        MergeFastForward::Allowed => {}
        MergeFastForward::Only => opts.fast_forward_only = true,
    }
    opts
}
