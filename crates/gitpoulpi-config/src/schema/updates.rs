//! The `updates` part of the config schema: whether and how GitPoulpi looks
//! for a newer release (the updates-and-install design).

use serde::{Deserialize, Serialize};

/// The GitHub repository whose releases are checked.
pub const DEFAULT_UPDATES_REPO: &str = "Poulpi-Lab/GitPoulpi";

/// What GitPoulpi does about a newer release: `updates.mode`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum UpdateMode {
    /// Download and stage the update on its own. Until the self-updater
    /// exists this behaves exactly like `notify`.
    #[serde(rename = "auto")]
    Auto,
    /// Check once a day and say so in the status bar when a newer release exists.
    #[default]
    #[serde(rename = "notify")]
    Notify,
    /// Never check: no request is made.
    #[serde(rename = "off")]
    Off,
}

impl UpdateMode {
    pub const ALL: &'static [UpdateMode] = &[UpdateMode::Auto, UpdateMode::Notify, UpdateMode::Off];

    pub fn as_str(self) -> &'static str {
        match self {
            UpdateMode::Auto => "auto",
            UpdateMode::Notify => "notify",
            UpdateMode::Off => "off",
        }
    }
}

/// Which releases count: `updates.channel`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum UpdateChannel {
    /// Only full releases (no `-rc.1` style pre-release).
    #[default]
    #[serde(rename = "stable")]
    Stable,
    /// Pre-releases too.
    #[serde(rename = "prerelease")]
    Prerelease,
}

impl UpdateChannel {
    pub const ALL: &'static [UpdateChannel] = &[UpdateChannel::Stable, UpdateChannel::Prerelease];

    pub fn as_str(self) -> &'static str {
        match self {
            UpdateChannel::Stable => "stable",
            UpdateChannel::Prerelease => "prerelease",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct UpdatesConfig {
    /// Live: `auto`, `notify` (default) or `off`.
    pub mode: UpdateMode,
    /// Live: `stable` (default) or `prerelease`.
    pub channel: UpdateChannel,
    /// `owner/name` on github.com whose releases are checked.
    pub repo: String,
}

impl Default for UpdatesConfig {
    fn default() -> Self {
        Self {
            mode: UpdateMode::default(),
            channel: UpdateChannel::default(),
            repo: DEFAULT_UPDATES_REPO.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_notify_on_the_stable_channel() {
        let u = UpdatesConfig::default();
        assert_eq!(u.mode, UpdateMode::Notify);
        assert_eq!(u.channel, UpdateChannel::Stable);
        assert_eq!(u.repo, "Poulpi-Lab/GitPoulpi");
    }

    #[test]
    fn every_spelling_parses_and_a_bad_one_is_refused() {
        for m in UpdateMode::ALL {
            let u: UpdatesConfig = toml::from_str(&format!("mode = \"{}\"", m.as_str())).unwrap();
            assert_eq!(u.mode, *m);
        }
        for c in UpdateChannel::ALL {
            let u: UpdatesConfig =
                toml::from_str(&format!("channel = \"{}\"", c.as_str())).unwrap();
            assert_eq!(u.channel, *c);
        }
        assert!(toml::from_str::<UpdatesConfig>("mode = \"sometimes\"").is_err());
    }
}
