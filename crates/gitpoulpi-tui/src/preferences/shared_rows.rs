//! The terminal UI's side of the shared Settings rows
//! (`gitpoulpi_app::settings`): the facts it supplies to the row builders and
//! the conversion of a shared row into this screen's [`Field`].
//!
//! The builders of the plain `config.toml` rows live in the app crate; what
//! stays here is what only this window knows: the theme list (built-in and
//! user themes), the update policy read in the background, the installation's
//! own look, and the colours a swatch names.

use super::*;
use gitpoulpi_app::settings::{self as shared, Action, Facts, PoolCtx, Swatch, UpdateFacts};
use ratatui::style::Color;

/// The ratatui colour a swatch names.
pub(crate) fn swatch_color(swatch: Swatch) -> Color {
    match swatch {
        Swatch::Reset => Color::Reset, // color-data
        Swatch::Ansi(n) => match n {
            0 => Color::Black,         // color-data
            1 => Color::Red,           // color-data
            2 => Color::Green,         // color-data
            3 => Color::Yellow,        // color-data
            4 => Color::Blue,          // color-data
            5 => Color::Magenta,       // color-data
            6 => Color::Cyan,          // color-data
            7 => Color::Gray,          // color-data
            8 => Color::DarkGray,      // color-data
            9 => Color::LightRed,      // color-data
            10 => Color::LightGreen,   // color-data
            11 => Color::LightYellow,  // color-data
            12 => Color::LightBlue,    // color-data
            13 => Color::LightMagenta, // color-data
            14 => Color::LightCyan,    // color-data
            _ => Color::White,         // color-data
        },
        Swatch::Indexed(i) => Color::Indexed(i), // color-data
        Swatch::Rgb(r, g, b) => Color::Rgb(r, g, b), // color-data
    }
}

/// The swatch that names `color`.
pub(crate) fn swatch_of(color: Color) -> Swatch {
    match color {
        Color::Reset => Swatch::Reset,               // color-data
        Color::Black => Swatch::Ansi(0),             // color-data
        Color::Red => Swatch::Ansi(1),               // color-data
        Color::Green => Swatch::Ansi(2),             // color-data
        Color::Yellow => Swatch::Ansi(3),            // color-data
        Color::Blue => Swatch::Ansi(4),              // color-data
        Color::Magenta => Swatch::Ansi(5),           // color-data
        Color::Cyan => Swatch::Ansi(6),              // color-data
        Color::Gray => Swatch::Ansi(7),              // color-data
        Color::DarkGray => Swatch::Ansi(8),          // color-data
        Color::LightRed => Swatch::Ansi(9),          // color-data
        Color::LightGreen => Swatch::Ansi(10),       // color-data
        Color::LightYellow => Swatch::Ansi(11),      // color-data
        Color::LightBlue => Swatch::Ansi(12),        // color-data
        Color::LightMagenta => Swatch::Ansi(13),     // color-data
        Color::LightCyan => Swatch::Ansi(14),        // color-data
        Color::White => Swatch::Ansi(15),            // color-data
        Color::Indexed(i) => Swatch::Indexed(i),     // color-data
        Color::Rgb(r, g, b) => Swatch::Rgb(r, g, b), // color-data
    }
}

/// What the shared row builders ask this window.
struct TuiFacts<'a>(&'a PreferencesState);

impl Facts for TuiFacts<'_> {
    fn theme_options(&self) -> Vec<(String, String)> {
        self.0.theme_options()
    }

    fn lane_options(&self, current: &str) -> Vec<(String, String)> {
        self.0.lane_options(current)
    }

    fn theme_swatches(&self) -> Vec<Swatch> {
        self.0.theme_swatches()
    }

    fn hosted_sources(
        &self,
        integrations: &[gitpoulpi_config::Integration],
    ) -> Vec<(String, String)> {
        crate::clone_dialog::hosted_sources(integrations)
            .into_iter()
            .map(|h| (h.label, h.profile_id))
            .collect()
    }

    fn format_date_now(&self, pattern: &str) -> String {
        crate::live_settings::format_date(pattern, self.0.now)
    }

    fn updates(&self) -> UpdateFacts {
        let view = crate::updates::settings_view();
        UpdateFacts {
            mode_locked: view.mode_locked,
            channel_locked: view.channel_locked,
            installed_with: view.installed_with,
        }
    }

    #[cfg(feature = "mcp")]
    fn mcp_bind_in_force(&self, _config: &Config) -> String {
        self.0.mcp_bind_in_force()
    }
}

impl From<shared::Row> for Field {
    /// A shared row as this screen's field. The rows the terminal UI builds
    /// itself (the shared descriptors of the installation, launcher and MCP
    /// security rows) are never converted.
    fn from(row: shared::Row) -> Self {
        let action = |a: Action| match a {
            Action::ForgetCredentials => PrefAction::ForgetCredentials,
            Action::GenerateSshKey => PrefAction::GenerateSshKey,
            Action::Host(name) => {
                unreachable!("the terminal UI builds the `{name}` row itself")
            }
        };
        let target = match row.target {
            shared::Target::Global(key) => Target::Global(key),
            shared::Target::Action(a) => Target::Action(action(a)),
            shared::Target::None => Target::None,
        };
        Field {
            label: row.label,
            control: row.control.map_action(action),
            help: row.help,
            warning: row.warning,
            danger: row.danger,
            target,
            live: row.live,
            coming: row.coming,
            placeholder: row.placeholder,
            ellipsize: row.ellipsize,
        }
    }
}

impl PreferencesState {
    /// Runs `build` over the shared builders' context. `own` is the
    /// configuration as this installation shows it (the `ui.*` rows).
    fn with_pool<R>(&self, own: &Config, build: impl FnOnce(&PoolCtx) -> R) -> R {
        build(&PoolCtx {
            config: &self.config,
            own,
            facts: &TuiFacts(self),
        })
    }

    fn shared_fields(
        &self,
        own: &Config,
        build: impl FnOnce(&PoolCtx) -> Vec<shared::Row>,
    ) -> Vec<Field> {
        self.with_pool(own, build)
            .into_iter()
            .map(Field::from)
            .collect()
    }

    /// Every `general.*` / `repos.*` / `log.*` / `updates.*` row, unplaced.
    pub(super) fn general_rows(&self) -> Vec<Field> {
        self.shared_fields(&self.config, shared::general_rows)
    }

    /// The `ui.*` rows, unplaced: this installation's own look keys.
    pub(super) fn ui_rows(&self) -> Vec<Field> {
        let own = self.own_config();
        self.shared_fields(&own, shared::ui_rows)
    }

    /// The Diff & editor page's rows, unplaced.
    pub(super) fn editor_rows(&self) -> Vec<Field> {
        self.shared_fields(&self.config, shared::editor_rows)
    }

    /// The MCP switch and port, unplaced, then the suggest-port button.
    #[cfg(feature = "mcp")]
    pub(super) fn mcp_rows(&self) -> Vec<Field> {
        let mut rows = self.shared_fields(&self.config, shared::mcp_rows);
        rows.push(self.mcp_suggest_row());
        rows
    }
}
