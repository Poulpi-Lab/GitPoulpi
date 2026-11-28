//! What building the Settings rows needs besides the configuration.

use super::row::Swatch;
use gitpoulpi_config::{Config, Integration};

/// The update policy as this installation sees it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct UpdateFacts {
    /// A machine policy fixes the check mode.
    pub mode_locked: bool,
    /// A machine policy fixes the channel.
    pub channel_locked: bool,
    /// How this installation was installed, once read.
    pub installed_with: Option<String>,
}

/// The facts about the running frontend and the machine that rows are built
/// from. A frontend implements it once; the defaults describe a frontend with
/// nothing special to say, which is what the shared descriptors use
/// ([`NoFacts`]).
pub trait Facts {
    /// The Theme row's choices as (name, slug): "System", the presets, the
    /// user's own themes.
    fn theme_options(&self) -> Vec<(String, String)> {
        Vec::new()
    }

    /// The graph lane palette row's choices as (label, slug); `current` is the
    /// configured value.
    fn lane_options(&self, _current: &str) -> Vec<(String, String)> {
        Vec::new()
    }

    /// The chips of the theme swatch strip.
    fn theme_swatches(&self) -> Vec<Swatch> {
        Vec::new()
    }

    /// The profiles the Clone dialog could start on, as (label, profile id).
    fn hosted_sources(&self, _integrations: &[Integration]) -> Vec<(String, String)> {
        Vec::new()
    }

    /// `pattern` applied to the current time, for the date format previews.
    fn format_date_now(&self, _pattern: &str) -> String {
        String::new()
    }

    /// The update policy.
    fn updates(&self) -> UpdateFacts {
        UpdateFacts::default()
    }

    /// The MCP bind this installation serves on (its own override, else
    /// `[mcp] bind`).
    fn mcp_bind_in_force(&self, config: &Config) -> String {
        config.mcp.bind.clone()
    }
}

/// [`Facts`] for a caller with nothing to add: the descriptor of the rows.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoFacts;

impl Facts for NoFacts {}

/// What the shared row builders read.
#[derive(Clone, Copy)]
pub struct PoolCtx<'a> {
    /// The configuration as stored.
    pub config: &'a Config,
    /// The configuration as this installation shows it: its own look keys
    /// over the shared `ui.*` (the same as `config` when there is no
    /// per-installation look).
    pub own: &'a Config,
    pub facts: &'a dyn Facts,
}
