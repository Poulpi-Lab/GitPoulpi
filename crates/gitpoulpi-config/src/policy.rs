//! The machine-wide update policy an administrator (or an installer property)
//! sets: a separate file, not a `config.toml` key, so a user's own config can
//! never loosen it (the install-channels design).
//!
//! Precedence, strongest first: registry policy (Windows), the policy file,
//! the user's `config.toml [updates]`, the marker's `feed=`, built-in
//! defaults. Everything here is pure: the file and the registry come in
//! through [`PolicySource`], so tests never read a real `/etc` or
//! `ProgramData`.
//!
//! Traps:
//! - A policy that cannot be parsed **fails closed** (`mode = notify`, no
//!   self-update): a typo must never leave a managed fleet updating itself.
//!   The worker that holds elevated rights reads the machine policy only
//!   ([`merge`] with the user's config is for the unprivileged side).
//! - `mode` is a ceiling (`off < notify < auto`): the user may pick lower,
//!   never higher.

use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::{UpdateChannel, UpdateMode, UpdatesConfig};

/// Which source answers "what is the latest release": `check`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckKind {
    Github,
    Nuget,
    Manifest,
    None,
}

impl CheckKind {
    fn parse(text: &str) -> Option<Self> {
        Some(match text {
            "github" => Self::Github,
            "nuget" => Self::Nuget,
            "manifest" => Self::Manifest,
            "none" => Self::None,
            _ => return None,
        })
    }
}

/// The highest version offered. `1.4` allows any `1.4.x`, `1.4.2` allows at
/// most `1.4.2`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pin(Vec<u64>);

impl std::fmt::Display for Pin {
    /// `1.4` or `1.4.2`: the text [`Pin::parse`] reads back.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, n) in self.0.iter().enumerate() {
            if i > 0 {
                f.write_str(".")?;
            }
            write!(f, "{n}")?;
        }
        Ok(())
    }
}

impl Pin {
    /// One to three dot-separated numbers; `None` otherwise.
    pub fn parse(text: &str) -> Option<Self> {
        let parts: Option<Vec<u64>> = text
            .trim()
            .split('.')
            .map(|p| p.parse::<u64>().ok())
            .collect();
        parts.filter(|p| (1..=3).contains(&p.len())).map(Self)
    }

    /// Whether `version` (`1.4.9`, `v1.5.0-rc.1`) is at or below the pin. A
    /// version that is not `major.minor.patch` is never allowed.
    pub fn allows(&self, version: &str) -> bool {
        let Some(v) = core_version(version) else {
            return false;
        };
        if self.0.len() < 3 {
            v[..self.0.len()] == self.0[..]
        } else {
            v <= [self.0[0], self.0[1], self.0[2]]
        }
    }
}

/// `major.minor.patch` of a version, ignoring a `v` prefix and any
/// pre-release or build suffix.
fn core_version(version: &str) -> Option<[u64; 3]> {
    let core = version.trim().trim_start_matches('v');
    let core = core.split(['-', '+']).next()?;
    let mut it = core.split('.');
    let mut out = [0u64; 3];
    for slot in &mut out {
        *slot = it.next()?.parse().ok()?;
    }
    it.next().is_none().then_some(out)
}

/// What one layer (registry or file) sets. `None` = not set by policy.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Policy {
    pub mode: Option<UpdateMode>,
    pub release_channel: Option<UpdateChannel>,
    pub pin: Option<Pin>,
    pub check: Option<CheckKind>,
    pub check_url: Option<String>,
    pub repo: Option<String>,
    pub download_base: Option<String>,
    pub allow_self_update: Option<bool>,
    pub message: Option<String>,
    /// This layer could not be read and stands in as "notify, no
    /// self-update".
    pub failed_closed: bool,
}

fn rank(mode: UpdateMode) -> u8 {
    match mode {
        UpdateMode::Off => 0,
        UpdateMode::Notify => 1,
        UpdateMode::Auto => 2,
    }
}

fn lower(a: UpdateMode, b: UpdateMode) -> UpdateMode {
    if rank(a) <= rank(b) {
        a
    } else {
        b
    }
}

impl Policy {
    fn closed() -> Self {
        Self {
            mode: Some(UpdateMode::Notify),
            allow_self_update: Some(false),
            failed_closed: true,
            ..Self::default()
        }
    }

    /// `self` over `weaker`: every field `self` sets wins. A layer that
    /// failed closed never raises a `mode` the other layer lowered.
    fn over(self, weaker: Policy) -> Policy {
        let mode = match (self.mode, weaker.mode) {
            (Some(a), Some(b)) if self.failed_closed || weaker.failed_closed => Some(lower(a, b)),
            (a, b) => a.or(b),
        };
        Policy {
            mode,
            release_channel: self.release_channel.or(weaker.release_channel),
            pin: self.pin.or(weaker.pin),
            check: self.check.or(weaker.check),
            check_url: self.check_url.or(weaker.check_url),
            repo: self.repo.or(weaker.repo),
            download_base: self.download_base.or(weaker.download_base),
            allow_self_update: self.allow_self_update.or(weaker.allow_self_update),
            message: self.message.or(weaker.message),
            failed_closed: self.failed_closed || weaker.failed_closed,
        }
    }

    /// Reads `[updates]` values given as text (a registry key, or a TOML
    /// table flattened to strings). An unknown key lands in `warnings`; a
    /// bad value is an error.
    fn from_values(
        values: &BTreeMap<String, String>,
        warnings: &mut Vec<String>,
    ) -> Result<Self, String> {
        let mut p = Policy::default();
        for (key, value) in values {
            let v = value.trim();
            match key.as_str() {
                "mode" => {
                    p.mode = Some(match v {
                        "off" => UpdateMode::Off,
                        "notify" => UpdateMode::Notify,
                        "auto" => UpdateMode::Auto,
                        _ => return Err(format!("mode = {v:?} is not off, notify or auto")),
                    })
                }
                "release_channel" => {
                    p.release_channel = Some(match v {
                        "stable" => UpdateChannel::Stable,
                        "prerelease" => UpdateChannel::Prerelease,
                        _ => {
                            return Err(format!(
                                "release_channel = {v:?} is not stable or prerelease"
                            ))
                        }
                    })
                }
                "pin" => {
                    p.pin =
                        Some(Pin::parse(v).ok_or_else(|| format!("pin = {v:?} is not a version"))?)
                }
                "check" => {
                    p.check = Some(CheckKind::parse(v).ok_or_else(|| {
                        format!("check = {v:?} is not github, nuget, manifest or none")
                    })?)
                }
                "check_url" => p.check_url = non_empty(v),
                "repo" => p.repo = non_empty(v),
                "download_base" => p.download_base = non_empty(v),
                "allow_self_update" => {
                    p.allow_self_update = Some(match v.to_ascii_lowercase().as_str() {
                        "true" | "1" => true,
                        "false" | "0" => false,
                        _ => return Err(format!("allow_self_update = {v:?} is not true or false")),
                    })
                }
                "message" => p.message = non_empty(v),
                other => warnings.push(format!("unknown update policy key {other:?} ignored")),
            }
        }
        Ok(p)
    }
}

fn non_empty(v: &str) -> Option<String> {
    (!v.is_empty()).then(|| v.to_string())
}

/// Where the policy text comes from. The real file reader is
/// [`FilePolicySource`]; the Windows registry reader is another
/// implementation, and tests pass fixed text.
pub trait PolicySource {
    /// The policy file's text, `None` when there is no file.
    fn file_text(&self) -> Option<String>;
    /// The `Updates` registry key's values as text (names as in the file),
    /// `None` when there is no such key or this is not Windows.
    fn registry_values(&self) -> Option<BTreeMap<String, String>>;
}

/// Reads the policy file at a path; no registry.
pub struct FilePolicySource(pub PathBuf);

impl PolicySource for FilePolicySource {
    fn file_text(&self) -> Option<String> {
        std::fs::read_to_string(&self.0).ok()
    }

    fn registry_values(&self) -> Option<BTreeMap<String, String>> {
        None
    }
}

/// The machine's policy and what to log about reading it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Loaded {
    pub policy: Policy,
    /// One English line each (unknown key, unreadable layer): the caller logs
    /// them.
    pub warnings: Vec<String>,
}

fn file_values(text: &str) -> Result<BTreeMap<String, String>, String> {
    let table: toml::Table = text.parse().map_err(|e: toml::de::Error| e.to_string())?;
    let mut out = BTreeMap::new();
    let Some(section) = table.get("updates") else {
        return Ok(out);
    };
    let section = section
        .as_table()
        .ok_or("[updates] is not a table".to_string())?;
    for (key, value) in section {
        let text = match value {
            toml::Value::String(s) => s.clone(),
            toml::Value::Boolean(b) => b.to_string(),
            _ => return Err(format!("{key} must be a string or a boolean")),
        };
        out.insert(key.clone(), text);
    }
    Ok(out)
}

/// Reads both layers, registry over file. A layer that cannot be parsed or
/// holds a bad value fails closed (see the module docs).
pub fn load(source: &dyn PolicySource) -> Loaded {
    let mut warnings = Vec::new();
    let mut layer = |name: &str, values: Result<BTreeMap<String, String>, String>| {
        values
            .and_then(|v| Policy::from_values(&v, &mut warnings))
            .unwrap_or_else(|e| {
                warnings.push(format!(
                    "update policy {name} is invalid ({e}): failing closed"
                ));
                Policy::closed()
            })
    };
    let file = source.file_text().map(|t| layer("file", file_values(&t)));
    let registry = source.registry_values().map(|v| layer("registry", Ok(v)));
    let policy = match (registry, file) {
        (Some(r), Some(f)) => r.over(f),
        (Some(p), None) | (None, Some(p)) => p,
        (None, None) => Policy::default(),
    };
    Loaded { policy, warnings }
}

/// Where the policy file lives on `os` (`windows`, `macos`, anything else is
/// Linux style). `program_data` is `%ProgramData%`.
pub fn policy_path_for(os: &str, program_data: Option<&str>) -> PathBuf {
    match os {
        "windows" => PathBuf::from(program_data.unwrap_or(r"C:\ProgramData"))
            .join("GitPoulpi")
            .join("policy.toml"),
        "macos" => PathBuf::from("/Library/Application Support/GitPoulpi/policy.toml"),
        _ => PathBuf::from("/etc/gitpoulpi/policy.toml"),
    }
}

/// The policy file path(s) of the running OS.
pub fn policy_paths() -> Vec<PathBuf> {
    let program_data = std::env::var("ProgramData").ok();
    vec![policy_path_for(
        std::env::consts::OS,
        program_data.as_deref(),
    )]
}

/// Policy and the user's `config.toml [updates]` combined: what the app
/// actually does, and which fields the policy set (the UI shows those as
/// "Set by your administrator").
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectiveUpdates {
    pub mode: UpdateMode,
    pub release_channel: UpdateChannel,
    pub repo: String,
    pub pin: Option<Pin>,
    pub check: Option<CheckKind>,
    pub check_url: Option<String>,
    pub download_base: Option<String>,
    pub allow_self_update: bool,
    pub message: Option<String>,
    pub mode_locked: bool,
    pub release_channel_locked: bool,
    pub repo_locked: bool,
}

/// The mode is the lower of policy and user (a ceiling); every other field
/// the policy sets replaces the user's. `allow_self_update` defaults to true
/// when no policy says otherwise.
pub fn merge(policy: &Policy, user: &UpdatesConfig) -> EffectiveUpdates {
    EffectiveUpdates {
        mode: policy.mode.map_or(user.mode, |p| lower(p, user.mode)),
        release_channel: policy.release_channel.unwrap_or(user.channel),
        repo: policy.repo.clone().unwrap_or_else(|| user.repo.clone()),
        pin: policy.pin.clone(),
        check: policy.check,
        check_url: policy.check_url.clone(),
        download_base: policy.download_base.clone(),
        allow_self_update: policy.allow_self_update.unwrap_or(true),
        message: policy.message.clone(),
        mode_locked: policy.mode.is_some(),
        release_channel_locked: policy.release_channel.is_some(),
        repo_locked: policy.repo.is_some(),
    }
}

#[cfg(test)]
mod tests;
