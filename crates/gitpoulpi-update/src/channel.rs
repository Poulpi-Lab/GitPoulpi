//! Which install channel the running build belongs to (the install-channels
//! design). The answer comes from recorded facts only, never from a guess
//! about the path: a marker's `channel=`, an Inno uninstall key that names
//! this folder, cargo's own install record. Anything else is not ours to
//! replace.
//!
//! Everything the resolution reads comes through [`FileOps`],
//! [`RegistryOps`] and [`ChannelEnv`], so the tests need no registry, no
//! real `CARGO_HOME` and no read-only directory.
//!
//! Traps:
//! - A marker for another platform (files copied around) is `Dev`, not
//!   Legacy: the same rule as [`crate::detect`].
//! - `Legacy` (a marker without `channel=`) resolves to `archive` only when
//!   this user can write the folder; otherwise it is announce-only, because
//!   a swap would fail halfway or need rights nobody granted.
//! - The first successful update rewrites the marker with the resolved
//!   channel (`install::apply_with_channel`).

use std::path::{Path, PathBuf};

use gitpoulpi_config::release::{self, InstallChannel, Marker, MARKER_FILE};

use crate::fsops::FileOps;

/// A Windows registry hive an Inno uninstall key lives in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hive {
    CurrentUser,
    LocalMachine,
}

/// The one registry question channel resolution asks. The Windows
/// implementation (the updater crate) reads the Inno `..._is1` uninstall
/// key; everywhere else [`NoRegistry`].
pub trait RegistryOps {
    /// `InstallLocation` of GitPoulpi's Inno uninstall key in `hive`.
    fn inno_install_location(&self, hive: Hive) -> Option<String>;
}

pub struct NoRegistry;

impl RegistryOps for NoRegistry {
    fn inno_install_location(&self, _hive: Hive) -> Option<String> {
        None
    }
}

/// The rest of the machine facts resolution needs.
pub trait ChannelEnv {
    /// `$CARGO_HOME` (default `~/.cargo`).
    fn cargo_home(&self) -> Option<PathBuf>;
    /// Whether this user can create files in `dir`.
    fn dir_writable(&self, dir: &Path) -> bool;
}

/// The real environment.
pub struct RealEnv;

impl ChannelEnv for RealEnv {
    fn cargo_home(&self) -> Option<PathBuf> {
        if let Some(h) = std::env::var_os("CARGO_HOME").filter(|h| !h.is_empty()) {
            return Some(PathBuf::from(h));
        }
        let home = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE"))?;
        Some(PathBuf::from(home).join(".cargo"))
    }

    fn dir_writable(&self, dir: &Path) -> bool {
        let probe = dir.join(format!(".gitpoulpi-write-test-{}", std::process::id()));
        match std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&probe)
        {
            Ok(_) => {
                let _ = std::fs::remove_file(&probe);
                true
            }
            Err(_) => false,
        }
    }
}

/// What the running build is, as far as updating goes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Channel {
    /// An install with a known channel (recorded, or Legacy resolved to one).
    Install(InstallChannel),
    /// `cargo install` / `cargo binstall`: no marker, cargo's record only.
    Cargo,
    /// A marker without a channel in a folder this user cannot write.
    LegacyReadOnly,
    /// A development build (or anything else with no record): never updates.
    Dev,
}

/// The fact the channel was concluded from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum How {
    Marker,
    CargoRecord,
    InnoUninstallKey,
    Legacy,
    Dev,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Resolved {
    pub channel: Channel,
    pub how: How,
    pub dir: PathBuf,
    pub exe_name: String,
    /// The marker next to the exe, when it is this platform's and parses.
    pub marker: Option<Marker>,
}

impl Resolved {
    /// The internal feed an internalised package recorded.
    pub fn feed(&self) -> Option<&str> {
        self.marker.as_ref().and_then(|m| m.feed.as_deref())
    }
}

/// Lower-case, forward slashes, no trailing separator: Inno writes
/// `InstallLocation` with a trailing backslash and the OS compares paths
/// case-insensitively.
fn same_dir(a: &str, b: &str) -> bool {
    let norm = |s: &str| s.replace('\\', "/").trim_end_matches('/').to_lowercase();
    norm(a) == norm(b)
}

fn cargo_recorded(fs: &dyn FileOps, cargo_home: &Path) -> bool {
    let Ok(text) = fs.read_to_string(&cargo_home.join(".crates2.json")) else {
        return false;
    };
    let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) else {
        return false;
    };
    json.get("installs")
        .and_then(|i| i.as_object())
        .is_some_and(|installs| installs.keys().any(|k| k.starts_with("gitpoulpi-cli ")))
}

/// Resolves the channel of the build running as `exe`.
pub fn resolve(
    fs: &dyn FileOps,
    reg: &dyn RegistryOps,
    env: &dyn ChannelEnv,
    exe: &Path,
    host_target: &str,
) -> Resolved {
    let dir = exe.parent().map(Path::to_path_buf).unwrap_or_default();
    let exe_name = exe
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_default()
        .to_string();
    let make = |channel, how, marker| Resolved {
        channel,
        how,
        dir: dir.clone(),
        exe_name: exe_name.clone(),
        marker,
    };

    let marker = fs
        .read_to_string(&dir.join(MARKER_FILE))
        .ok()
        .and_then(|t| Marker::parse(&t));
    if let Some(m) = marker {
        if m.target != host_target {
            return make(Channel::Dev, How::Dev, None);
        }
        if let Some(c) = m.channel {
            return make(Channel::Install(c), How::Marker, Some(m));
        }
        // Legacy: a marker from before channels existed.
        if release::is_windows(host_target) {
            let dir_text = dir.to_string_lossy();
            for (hive, channel) in [
                (Hive::CurrentUser, InstallChannel::InnoUser),
                (Hive::LocalMachine, InstallChannel::InnoMachine),
            ] {
                if reg
                    .inno_install_location(hive)
                    .is_some_and(|loc| same_dir(&loc, &dir_text))
                {
                    return make(Channel::Install(channel), How::InnoUninstallKey, Some(m));
                }
            }
        }
        return if env.dir_writable(&dir) {
            make(
                Channel::Install(InstallChannel::Archive),
                How::Legacy,
                Some(m),
            )
        } else {
            make(Channel::LegacyReadOnly, How::Legacy, Some(m))
        };
    }

    if let Some(home) = env.cargo_home() {
        if dir == home.join("bin") && cargo_recorded(fs, &home) {
            return make(Channel::Cargo, How::CargoRecord, None);
        }
    }
    make(Channel::Dev, How::Dev, None)
}

#[cfg(test)]
mod tests;
