//! The `git` part of the config schema.

use gitpoulpi_core::domain::MergeFastForward;
use serde::{Deserialize, Serialize};

config_enum! {
/// Line-ending normalisation, mirroring git's own `core.autocrlf`.
/// `Unset` means "do not touch it", i.e. defer to the repository's or the
/// machine's own git configuration.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutoCrlf {
    #[default]
    Unset => "unset",
    True => "true",
    Input => "input",
    False => "false",
}
}

config_enum! {
/// Mirrors git's `core.eol`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum EolMode {
    #[default]
    Native => "native",
    Lf => "lf",
    Crlf => "crlf",
}
}

config_enum! {
/// What the toolbar's Pull button does when clicked, chosen from its
/// dropdown.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum PullButtonMode {
    /// Pull, fast-forwarding when possible and merging otherwise.
    /// `fetch_all`, the value an older config may hold (Fetch All is
    /// now only a dropdown item, not what the button does), reads as this.
    #[default]
    #[serde(alias = "fetch_all")]
    Merge => "merge",
    /// Pull, fast-forward only; a diverged branch is refused.
    FfOnly => "ff_only",
    /// Pull with rebase.
    Rebase => "rebase",
}
}

impl PullButtonMode {
    /// The dropdown's wording.
    pub fn label(self) -> &'static str {
        match self {
            PullButtonMode::Merge => "Pull (fast-forward if possible)",
            PullButtonMode::FfOnly => "Pull (fast-forward only)",
            PullButtonMode::Rebase => "Pull (rebase)",
        }
    }
}

config_enum! {
/// What the toolbar's Push button (and `Ctrl+G P`) does: push straight
/// to the default remote, or open a small dialog with the default remote
/// preselected so another can be picked.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum PushBehavior {
    #[default]
    Direct => "direct",
    Ask => "ask",
}
}

/// Defaults for the per-repository git settings. A repository that says
/// nothing inherits these; one that does overrides them in its own
/// `.gitpoulpi/repo-state.toml` (see `gitpoulpi_session::RepoPreferences`),
/// which is why there are two homes for the same three names.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GitConfig {
    pub default_encoding: String,
    pub autocrlf: AutoCrlf,
    pub eol: EolMode,
    pub lfs_enabled: bool,
    /// Live: before a pull or push of a repository that uses LFS, install
    /// its (repository-local) LFS filters and pre-push hook when they are
    /// missing, so the transfer is LFS-aware.
    pub lfs_autofix_filters: bool,
    /// Live: what the toolbar's Pull button does (its dropdown writes it).
    pub pull_mode: PullButtonMode,
    /// Live: whether Push goes straight to the default remote or asks.
    pub push_behavior: PushBehavior,
    /// Live: whether a merge the user starts may fast-forward: `never`
    /// (always a merge commit, the default), `allowed` (git's own default)
    /// or `only`. A repository's Merge options page overrides it.
    pub merge_fast_forward: MergeFastForward,
}

impl Default for GitConfig {
    fn default() -> Self {
        Self {
            default_encoding: "utf-8".into(),
            autocrlf: AutoCrlf::default(),
            eol: EolMode::default(),
            lfs_enabled: true,
            lfs_autofix_filters: true,
            pull_mode: PullButtonMode::default(),
            push_behavior: PushBehavior::default(),
            merge_fast_forward: MergeFastForward::default(),
        }
    }
}
