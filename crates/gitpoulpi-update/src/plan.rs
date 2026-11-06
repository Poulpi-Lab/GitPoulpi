//! What GitPoulpi does about an update, per install channel: the table of the
//! install-channels design as one pure function.
//!
//! The four outcomes:
//! - [`Procedure::Swap`]: the updater replaces GitPoulpi's own files.
//! - [`Procedure::Run`]: the updater runs the package manager itself.
//! - [`Procedure::Show`]: the panel shows the exact command with Copy and
//!   Open in terminal. The app never runs these itself, and never behind a
//!   hidden `sudo`.
//! - [`Procedure::Announce`]: nothing to do from here (managed, containers,
//!   a folder this user cannot write); the panel says why.
//!
//! The policy overrides apply last: `allow_self_update = false` turns every
//! Swap into an announcement and every Run into a Show, `mode = off` hides
//! everything. `auto` never restarts anything and only stages in the
//! background for [`Procedure::Swap`] ([`stages_in_background`]).

use gitpoulpi_config::policy::EffectiveUpdates;
use gitpoulpi_config::release::{InstallChannel, DOCKER_IMAGE};
use gitpoulpi_config::UpdateMode;

use crate::channel::{Channel, Hive, Resolved};

/// The AUR package of the binary release.
const AUR_PACKAGE: &str = "gitpoulpi-bin";
/// Where the source repository is for `cargo install --git`.
const REPO_URL: &str = "https://github.com/Poulpi-Lab/GitPoulpi";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Elevation {
    None,
    /// Windows UAC (the updater's elevated worker, or `runas` on the manager).
    Uac,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NothingReason {
    /// `mode = off`: the check is hidden too.
    ModeOff,
    /// A development build or an unrecorded install: version only.
    NoInstallRecord,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnnounceReason {
    /// The administrator disabled self-update (`allow_self_update = false`).
    PolicyBlocksSelfUpdate,
    /// An MSI install: deployed by the administrator.
    Managed,
    /// A container image: pull a new one.
    Container,
    /// A pre-channel install in a folder this user cannot write.
    LegacyReadOnly,
    /// apt without GitPoulpi's repository configured: download the .deb.
    AptWithoutRepository,
    /// Nothing to go back to (no previous version known, or no way to).
    NoRevert,
}

/// What to do. Commands are display strings in `Show` and `Announce`, argv
/// in `Run`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Procedure {
    Nothing(NothingReason),
    /// Replace the install's own files (also the revert swap). `registry` is
    /// the hive whose uninstall key follows with the new `DisplayVersion`.
    Swap {
        elevation: Elevation,
        registry: Option<Hive>,
    },
    Run {
        argv: Vec<String>,
        elevation: Elevation,
    },
    Show {
        commands: Vec<String>,
    },
    Announce {
        reason: AnnounceReason,
        /// Commands worth showing next to the announcement.
        commands: Vec<String>,
        /// The administrator's `message`, shown in the panel.
        message: Option<String>,
    },
}

/// Facts the table needs that are neither the channel nor the policy.
#[derive(Debug, Clone, Default)]
pub struct PlanContext {
    /// The version to go to (a revert plan ignores it).
    pub to_version: String,
    /// What a revert goes back to; `None` when unknown.
    pub previous_version: Option<String>,
    /// `/etc/apt/sources.list.d/gitpoulpi*.list` (or `.sources`) exists.
    pub apt_source_present: bool,
    /// The first of `yay` / `paru` on PATH.
    pub aur_helper: Option<String>,
    /// `cargo-binstall` is on PATH.
    pub binstall: bool,
}

/// A command as the panel shows and copies it: arguments with spaces or
/// shell characters are double-quoted.
pub fn display_command(argv: &[String]) -> String {
    argv.iter()
        .map(|a| {
            if a.is_empty()
                || a.contains(|c: char| c.is_whitespace() || "\"'$`\\&|;<>()".contains(c))
            {
                format!("\"{}\"", a.replace('"', "\\\""))
            } else {
                a.clone()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn argv(parts: &[&str]) -> Vec<String> {
    parts.iter().map(|p| p.to_string()).collect()
}

/// The feed an internal Chocolatey package uses: the policy's `check_url`
/// first, then the marker's `feed=`.
fn choco_feed(resolved: &Resolved, eff: &EffectiveUpdates) -> Option<String> {
    eff.check_url
        .clone()
        .or_else(|| resolved.feed().map(str::to_string))
}

fn announce(reason: AnnounceReason, commands: Vec<String>, eff: &EffectiveUpdates) -> Procedure {
    Procedure::Announce {
        reason,
        commands,
        message: eff.message.clone(),
    }
}

/// The policy's last word over a plan.
fn apply_policy(p: Procedure, eff: &EffectiveUpdates) -> Procedure {
    if eff.allow_self_update {
        return p;
    }
    match p {
        Procedure::Swap { .. } => announce(AnnounceReason::PolicyBlocksSelfUpdate, vec![], eff),
        Procedure::Run { argv, .. } => Procedure::Show {
            commands: vec![display_command(&argv)],
        },
        other => other,
    }
}

/// The procedure that brings `ctx.to_version` in.
pub fn plan(resolved: &Resolved, eff: &EffectiveUpdates, ctx: &PlanContext) -> Procedure {
    if eff.mode == UpdateMode::Off {
        return Procedure::Nothing(NothingReason::ModeOff);
    }
    let v = ctx.to_version.as_str();
    let procedure = match resolved.channel {
        Channel::Dev => Procedure::Nothing(NothingReason::NoInstallRecord),
        Channel::LegacyReadOnly => announce(AnnounceReason::LegacyReadOnly, vec![], eff),
        Channel::Cargo => Procedure::Show {
            commands: vec![cargo_command(v, ctx.binstall)],
        },
        Channel::Install(c) => match c {
            InstallChannel::Archive => Procedure::Swap {
                elevation: Elevation::None,
                registry: None,
            },
            InstallChannel::InnoUser => Procedure::Swap {
                elevation: Elevation::None,
                registry: Some(Hive::CurrentUser),
            },
            InstallChannel::InnoMachine => Procedure::Swap {
                elevation: Elevation::Uac,
                registry: Some(Hive::LocalMachine),
            },
            InstallChannel::Choco => Procedure::Run {
                argv: argv(&["choco", "upgrade", "gitpoulpi", "-y"]),
                elevation: Elevation::Uac,
            },
            InstallChannel::ChocoInternal => {
                // No `--params`: the deployed configuration stays untouched.
                let mut a = argv(&["choco", "upgrade", "gitpoulpi", "-y"]);
                if let Some(feed) = choco_feed(resolved, eff) {
                    a.push(format!("--source={feed}"));
                }
                Procedure::Run {
                    argv: a,
                    elevation: Elevation::Uac,
                }
            }
            InstallChannel::Winget => Procedure::Run {
                argv: argv(&[
                    "winget",
                    "upgrade",
                    "--id",
                    "GitPoulpi.GitPoulpi",
                    "--exact",
                    "--silent",
                    "--accept-source-agreements",
                    "--accept-package-agreements",
                ]),
                // winget asks for the machine scope itself.
                elevation: Elevation::None,
            },
            InstallChannel::Scoop => Procedure::Run {
                argv: argv(&["scoop", "update", "gitpoulpi"]),
                elevation: Elevation::None,
            },
            InstallChannel::Homebrew => Procedure::Run {
                argv: argv(&["brew", "upgrade", "gitpoulpi"]),
                elevation: Elevation::None,
            },
            InstallChannel::Apt if ctx.apt_source_present => Procedure::Show {
                commands: vec![
                    "sudo apt update && sudo apt install --only-upgrade gitpoulpi".into(),
                ],
            },
            InstallChannel::Apt => announce(AnnounceReason::AptWithoutRepository, vec![], eff),
            InstallChannel::Msi => announce(AnnounceReason::Managed, vec![], eff),
            InstallChannel::Aur => Procedure::Show {
                commands: aur_commands(ctx.aur_helper.as_deref()),
            },
            InstallChannel::Docker => announce(
                AnnounceReason::Container,
                vec![
                    format!("docker pull {DOCKER_IMAGE}:{v}"),
                    format!("docker pull {DOCKER_IMAGE}:latest"),
                ],
                eff,
            ),
        },
    };
    apply_policy(procedure, eff)
}

fn cargo_command(version: &str, binstall: bool) -> String {
    if binstall {
        "cargo binstall gitpoulpi-cli".into()
    } else {
        format!("cargo install --locked --git {REPO_URL} --tag releases/v{version} gitpoulpi-cli")
    }
}

fn aur_commands(helper: Option<&str>) -> Vec<String> {
    match helper {
        // Never `pacman -Sy <pkg>`: a partial upgrade breaks Arch.
        Some(h) => vec![format!("{h} -S {AUR_PACKAGE}")],
        None => vec![
            format!("git clone https://aur.archlinux.org/{AUR_PACKAGE}.git"),
            format!("cd {AUR_PACKAGE} && makepkg -si"),
        ],
    }
}

/// The procedure that goes back to `ctx.previous_version`.
pub fn revert_plan(resolved: &Resolved, eff: &EffectiveUpdates, ctx: &PlanContext) -> Procedure {
    if eff.mode == UpdateMode::Off {
        return Procedure::Nothing(NothingReason::ModeOff);
    }
    let Some(prev) = ctx.previous_version.as_deref() else {
        return match plan(resolved, eff, ctx) {
            // The `.old` files are the previous version: the updater knows
            // whether they exist.
            swap @ Procedure::Swap { .. } => swap,
            Procedure::Nothing(r) => Procedure::Nothing(r),
            _ => announce(AnnounceReason::NoRevert, vec![], eff),
        };
    };
    let procedure = match resolved.channel {
        Channel::Dev => Procedure::Nothing(NothingReason::NoInstallRecord),
        Channel::LegacyReadOnly => announce(AnnounceReason::LegacyReadOnly, vec![], eff),
        Channel::Cargo => Procedure::Show {
            commands: vec![format!(
                "cargo install --locked --git {REPO_URL} --tag releases/v{prev} gitpoulpi-cli"
            )],
        },
        Channel::Install(c) => match c {
            InstallChannel::Archive | InstallChannel::InnoUser | InstallChannel::InnoMachine => {
                return plan(resolved, eff, ctx)
            }
            InstallChannel::Choco | InstallChannel::ChocoInternal => {
                let mut a = argv(&[
                    "choco",
                    "install",
                    "gitpoulpi",
                    "--version",
                    prev,
                    "--allow-downgrade",
                    "-y",
                ]);
                if c == InstallChannel::ChocoInternal {
                    if let Some(feed) = choco_feed(resolved, eff) {
                        a.push(format!("--source={feed}"));
                    }
                }
                Procedure::Show {
                    commands: vec![display_command(&a)],
                }
            }
            InstallChannel::Winget => Procedure::Show {
                commands: vec![format!(
                    "winget install --id GitPoulpi.GitPoulpi --version {prev}"
                )],
            },
            InstallChannel::Scoop => Procedure::Run {
                argv: vec!["scoop".into(), "reset".into(), format!("gitpoulpi@{prev}")],
                elevation: Elevation::None,
            },
            // Homebrew has no downgrade; a versioned formula is the nearest
            // thing, shown rather than run.
            InstallChannel::Homebrew => Procedure::Show {
                commands: vec![format!("brew install gitpoulpi@{prev}")],
            },
            InstallChannel::Apt if ctx.apt_source_present => Procedure::Show {
                commands: vec![format!("sudo apt install gitpoulpi={prev}")],
            },
            InstallChannel::Apt => announce(AnnounceReason::AptWithoutRepository, vec![], eff),
            InstallChannel::Msi => announce(AnnounceReason::Managed, vec![], eff),
            InstallChannel::Aur => announce(AnnounceReason::NoRevert, vec![], eff),
            InstallChannel::Docker => Procedure::Show {
                commands: vec![format!("docker pull {DOCKER_IMAGE}:{prev}")],
            },
        },
    };
    apply_policy(procedure, eff)
}

/// Whether `auto` mode may download and stage in the background: only where
/// the updater swaps files. A manager channel behaves like `notify`, and
/// nothing ever restarts without the user's action.
pub fn stages_in_background(procedure: &Procedure, eff: &EffectiveUpdates) -> bool {
    eff.mode == UpdateMode::Auto && matches!(procedure, Procedure::Swap { .. })
}

#[cfg(test)]
mod tests;
