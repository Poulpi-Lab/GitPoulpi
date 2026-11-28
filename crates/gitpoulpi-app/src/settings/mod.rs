//! The Settings screen as data, shared by the terminal UI and the desktop UI.
//!
//! A frontend draws Settings with its own widgets; what it must not restate is
//! what Settings *are*: which rows exist, their keys, labels, kinds, choices,
//! limits, notes and order, which of them an interface shows, and the rules
//! that make a row unavailable in this build.
//!
//! - [`Row`] / [`Control`]: one row, as data. Colour strips are
//!   [`Swatch`] references, the frontend resolves them.
//! - [`pool`]: every plain config row, unplaced, built from a [`PoolCtx`]
//!   (the configuration plus the [`Facts`] only the frontend knows: its theme
//!   list, the update policy, ...). The rows of the machine and the running
//!   installation have their descriptors in [`host`]; the frontend builds
//!   them with their values.
//! - [`rows`], [`setting_rows`], [`rows_for`]: the flat list of settings, one
//!   per key, and the part of it an [`Interface`] shows (a per-interface row of
//!   the desktop targets its `desktop.ui.*` key, a terminal-only row is
//!   absent).
//! - [`palette_rows`]: the generated `Settings: ...` palette entries.
//!
//! Writing stays with the frontend: a frontend turns an edit into a key and a
//! value and hands it to its own writer (the terminal UI's is the
//! `launcher_sync` / `gitpoulpi_config` writer, the desktop's
//! `DesktopConfig::set_in_file`).

pub mod choice_labels;
mod ctx;
pub mod host;
pub mod notes;
pub mod pages;
pub mod palette_rows;
mod pool_general;
mod pool_quantity;
mod pool_ui;
mod pool_updates;
mod row;
mod rows;
pub mod search;

pub use ctx::{Facts, NoFacts, PoolCtx, UpdateFacts};
pub use gitpoulpi_config::split::{Home, Interface};
pub use pages::{Entry, Page};
#[cfg(feature = "mcp")]
pub use pool_general::mcp_rows;
pub use pool_general::{editor_rows, general_rows};
pub use pool_quantity::{
    auto_fetch_field, hosting_refresh_field, image_max_field, log_max_file_field,
    script_timeout_field, script_total_timeout_field,
};
pub use pool_ui::{language_choices, language_options, ui_rows};
pub use pool_updates::{updates_channel_field, updates_installed_with_field, updates_mode_field};
pub use row::{value_text, Action, Btn, Control, QuantityUnit, Row, Swatch, Target};
pub use rows::{
    desktop_key, offered_to, rows, rows_for, setting_row, setting_rows, InterfaceRow, SettingKind,
    SettingRow,
};

/// The colour names the input colour row offers: the theme's own, a few
/// terminal-palette fixes, then a custom value.
pub const INPUT_PRESETS: &[&str] = &["theme", "gray", "blue", "slate", "dark", "custom"];

/// Whether this build has the row of the config key `key`: a row of a
/// feature the build lacks is never listed (nor a palette or search hit).
pub fn key_built(key: &str) -> bool {
    match key {
        "general.keep_submodules_up_to_date" => crate::features::SUBMODULE,
        "general.radar_enabled" => crate::features::RADAR,
        _ => true,
    }
}

/// Every plain config row, unplaced, in the Settings order: General, the
/// interface, the editor, the installation, the launcher and (with `mcp`)
/// the MCP server, then the date patterns' live previews (each right after
/// its pattern). The rows a build lacks are left out.
pub fn pool(cx: &PoolCtx) -> Vec<Row> {
    let mut pool = general_rows(cx);
    pool.extend(ui_rows(cx));
    pool.extend(editor_rows(cx));
    pool.extend(host::installation_rows(cx));
    pool.extend(host::launcher_rows(cx));
    #[cfg(feature = "mcp")]
    {
        pool.extend(mcp_rows(cx));
        pool.push(host::mcp_suggest_row(cx));
        pool.extend(host::mcp_security_rows(cx));
    }
    pool.retain(|f| f.key().is_none_or(key_built));
    pool
}
