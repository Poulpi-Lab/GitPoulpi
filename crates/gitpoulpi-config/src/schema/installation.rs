//! `[installation]`: one entry per GitPoulpi installation on the machine.
//!
//! Several installations (a Chocolatey package, a dev checkout, a cargo build,
//! an archive) share one `config.toml`. The shared `ui.*` keys belong to the
//! *primary* installation only; everyone else keeps its own look and names in
//! `[installation.settings.<id>]`, so one installation's settings never
//! recolour or rename another's. The identity rules live in
//! `crate::installation`; this file is just the stored shape.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

config_enum! {
    /// How an installation came to be, decided by its exe's location.
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub enum Kind {
        /// An exe under `<checkout>/target/<profile>/`.
        Dev => "dev",
        /// A release archive or package: a `.gitpoulpi-dist` marker sits next to the exe.
        Install => "install",
        /// The exe sits in `$CARGO_HOME/bin`.
        Cargo => "cargo",
        /// Anything else, such as an unpacked archive without a marker.
        #[default]
        Portable => "portable",
        /// The identity was named by `GITPOULPI_INSTALL_ID` (tests, scripts).
        Pinned => "pinned",
    }
}

/// The settings of one installation. Empty text means "not set": the value
/// then comes from `ui.*` (primary) or the kind's default.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct InstallationEntry {
    /// The installation's root folder, refreshed only when it changes.
    pub path: String,
    pub kind: Kind,
    /// The marker's `channel=`, informative; it is not part of the identity
    /// because a legacy marker gains one at its first update.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub channel: String,
    /// The base launcher name, frozen at first run so a second installation
    /// appearing never renames the first. Empty: the default name.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub name: String,
    /// Whether the launcher name carries `<major>.<minor>`; off by default
    /// because renaming breaks Start tiles and Dock items at each update.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub name_with_version: bool,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub logo_color: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub command_alias: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub command_alias_add: String,
    /// Folder-menu submenu this installation's entry goes into; empty: at the root.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub folder_menu_submenu: String,
    /// Environment (daemons, sessions, undo journal) this installation joins;
    /// empty: its own id.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub env: String,
    /// `mcp.bind` override, so two HTTP MCP servers need not fight for a port.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub mcp_bind: String,
}

/// The `[installation]` table.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct InstallationConfig {
    /// The id of the installation that owns the legacy names and the legacy
    /// `ui.*` keys. Absent by default: nobody is primary until the user says so.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary: Option<String>,
    /// Entries by installation id.
    pub settings: BTreeMap<String, InstallationEntry>,
}

impl InstallationConfig {
    /// Nothing recorded; the table is then left out of the file entirely.
    pub fn is_empty(&self) -> bool {
        self.primary.is_none() && self.settings.is_empty()
    }
}
