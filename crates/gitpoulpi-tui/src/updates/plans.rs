//! What this installation is and what GitPoulpi may do about an update (the
//! install-channels design): the facts read from disk once ([`Facts`]), the
//! plans derived from them ([`plans`]) and the check source the channel and
//! the policy choose ([`choose_source`]). Everything but [`Facts::detect`]
//! is pure, so the panel's per-procedure text and the source choice are
//! unit-tested without a file system.
//!
//! `Facts::detect` reads the marker, the machine policy and a few folders:
//! it runs on a background thread (`updates::load_facts`), never on the
//! render thread.

use gitpoulpi_config::policy::{self, CheckKind, EffectiveUpdates, Policy};
use gitpoulpi_config::release::{self, InstallChannel};
use gitpoulpi_config::{UpdateMode, UpdatesConfig};
use gitpoulpi_hosting::updates::sources::{SourceSpec, CHOCOLATEY_COMMUNITY_FEED};
use gitpoulpi_i18n::msg::updates as msg;
use gitpoulpi_update::channel::{self, Channel, RealEnv, Resolved};
use gitpoulpi_update::fsops::RealFs;
use gitpoulpi_update::plan::{self, PlanContext, Procedure};

/// What was read from the machine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Facts {
    pub(crate) resolved: Resolved,
    /// The machine policy (the file; the registry layer is the updater's).
    pub(crate) policy: Policy,
    /// `/etc/apt/sources.list.d/gitpoulpi*` exists.
    pub(crate) apt_source_present: bool,
    /// The first of `yay` / `paru` on `PATH`.
    pub(crate) aur_helper: Option<String>,
    /// `cargo-binstall` is on `PATH`.
    pub(crate) binstall: bool,
}

impl Facts {
    /// Reads the machine. Blocking (file reads, a write probe, a `PATH`
    /// walk): call from a background thread.
    #[cfg_attr(not(feature = "hosting"), allow(dead_code))]
    pub(crate) fn detect() -> Self {
        let exe = std::env::current_exe().unwrap_or_default();
        // The real Inno uninstall key on Windows: an old marker without
        // `channel=` resolves to inno-user / inno-machine through it.
        #[cfg(windows)]
        let registry = gitpoulpi_update::winreg::WindowsRegistry;
        #[cfg(not(windows))]
        let registry = channel::NoRegistry;
        let resolved =
            channel::resolve(&RealFs, &registry, &RealEnv, &exe, &release::host_target());
        let mut policy = Policy::default();
        for path in policy::policy_paths() {
            let loaded = policy::load(&policy::FilePolicySource(path));
            for warning in &loaded.warnings {
                tracing::warn!(target: "updates", "{warning}");
            }
            policy = loaded.policy;
        }
        let lookup = crate::repo_menu::path_lookup();
        let aur_helper = ["yay", "paru"]
            .into_iter()
            .find(|h| lookup(h))
            .map(str::to_string);
        Self {
            resolved,
            policy,
            apt_source_present: apt_source_present(),
            aur_helper,
            binstall: lookup("cargo-binstall"),
        }
    }

    /// The install channel the updater would swap (`Swap` procedures only
    /// exist for these).
    pub(crate) fn channel(&self) -> Channel {
        self.resolved.channel
    }
}

/// The folder apt keeps its sources in; a debug build lets a test point at
/// another one (`GITPOULPI_APT_SOURCES_DIR`).
#[cfg(unix)]
#[cfg_attr(not(feature = "hosting"), allow(dead_code))]
fn apt_sources_dir() -> std::path::PathBuf {
    #[cfg(debug_assertions)]
    if let Some(dir) = std::env::var_os("GITPOULPI_APT_SOURCES_DIR").filter(|d| !d.is_empty()) {
        return dir.into();
    }
    "/etc/apt/sources.list.d".into()
}

#[cfg(unix)]
#[cfg_attr(not(feature = "hosting"), allow(dead_code))]
fn apt_source_present() -> bool {
    std::fs::read_dir(apt_sources_dir())
        .map(|entries| {
            entries.flatten().any(|e| {
                let name = e.file_name().to_string_lossy().to_string();
                name.starts_with("gitpoulpi")
                    && (name.ends_with(".list") || name.ends_with(".sources"))
            })
        })
        .unwrap_or(false)
}

#[cfg(not(unix))]
#[cfg_attr(not(feature = "hosting"), allow(dead_code))]
fn apt_source_present() -> bool {
    false
}

/// The procedures for the release the chip announces and for going back.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Plans {
    pub(crate) effective: EffectiveUpdates,
    pub(crate) procedure: Procedure,
    pub(crate) revert: Procedure,
}

/// The plans for `facts` (`None`: not read yet, which behaves like a build
/// with no install record). `user` is `config.toml [updates]`.
pub(crate) fn plans(
    facts: Option<&Facts>,
    user: &UpdatesConfig,
    to_version: &str,
    previous: Option<&str>,
) -> Plans {
    let policy = facts.map(|f| f.policy.clone()).unwrap_or_default();
    let effective = policy::merge(&policy, user);
    let Some(facts) = facts else {
        let nothing = Procedure::Nothing(plan::NothingReason::NoInstallRecord);
        return Plans {
            effective,
            procedure: nothing.clone(),
            revert: nothing,
        };
    };
    let ctx = PlanContext {
        to_version: to_version.to_string(),
        previous_version: previous.map(str::to_string),
        apt_source_present: facts.apt_source_present,
        aur_helper: facts.aur_helper.clone(),
        binstall: facts.binstall,
    };
    Plans {
        procedure: plan::plan(&facts.resolved, &effective, &ctx),
        revert: plan::revert_plan(&facts.resolved, &effective, &ctx),
        effective,
    }
}

/// The text the panel and the settings page show after "Installed with".
pub(crate) fn channel_label(channel: Channel) -> String {
    let text = match channel {
        Channel::Install(c) => match c {
            InstallChannel::Archive => msg::VIA_ARCHIVE,
            InstallChannel::InnoUser => msg::VIA_INNO_USER,
            InstallChannel::InnoMachine => msg::VIA_INNO_MACHINE,
            InstallChannel::Choco => msg::VIA_CHOCO,
            InstallChannel::ChocoInternal => msg::VIA_CHOCO_INTERNAL,
            InstallChannel::Winget => msg::VIA_WINGET,
            InstallChannel::Scoop => msg::VIA_SCOOP,
            InstallChannel::Homebrew => msg::VIA_HOMEBREW,
            InstallChannel::Apt => msg::VIA_APT,
            InstallChannel::Msi => msg::VIA_MSI,
            InstallChannel::Aur => msg::VIA_AUR,
            InstallChannel::Docker => msg::VIA_DOCKER,
        },
        Channel::Cargo => msg::VIA_CARGO,
        Channel::LegacyReadOnly => msg::VIA_LEGACY_READ_ONLY,
        Channel::Dev => msg::VIA_DEV,
    };
    text.get().to_string()
}

/// Which source answers "what is the latest release" (the install-channels
/// design, 1.5): the policy's `check` first, else the channel's own default.
/// `api_base` is the GitHub API root (the test override applies), `feed` the
/// marker's internal feed.
#[cfg_attr(not(feature = "hosting"), allow(dead_code))]
pub(crate) fn choose_source(
    facts: Option<&Facts>,
    eff: &EffectiveUpdates,
    api_base: String,
    token: Option<String>,
) -> SourceSpec {
    let github = |api_base: String| SourceSpec::GitHubReleases {
        api_base,
        repo: eff.repo.clone(),
        token: token.clone(),
    };
    let feed = facts.and_then(|f| f.resolved.feed()).map(str::to_string);
    let channel = facts.map(Facts::channel);
    let url = eff.check_url.clone();
    // A development build never asks anyone (it never updates).
    if channel == Some(Channel::Dev) {
        return SourceSpec::None;
    }
    if eff.mode == UpdateMode::Off {
        return SourceSpec::None;
    }
    match eff.check {
        Some(CheckKind::None) => return SourceSpec::None,
        Some(CheckKind::Github) => return github(url.unwrap_or(api_base)),
        Some(CheckKind::Nuget) => {
            if let Some(feed) = url.clone().or(feed.clone()) {
                return SourceSpec::NuGet { feed };
            }
        }
        Some(CheckKind::Manifest) => {
            if let Some(url) = url.clone() {
                return SourceSpec::Manifest { url };
            }
        }
        None => {}
    }
    match channel {
        Some(Channel::Install(InstallChannel::Choco)) => SourceSpec::NuGet {
            feed: CHOCOLATEY_COMMUNITY_FEED.to_string(),
        },
        Some(Channel::Install(InstallChannel::ChocoInternal)) => SourceSpec::NuGet {
            feed: url
                .or(feed)
                .unwrap_or_else(|| CHOCOLATEY_COMMUNITY_FEED.to_string()),
        },
        _ => github(api_base),
    }
}

#[cfg(test)]
mod tests;
