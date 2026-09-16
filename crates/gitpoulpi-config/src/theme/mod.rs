//! The theme as data: roles and colors with no ratatui dependency, the
//! built-in presets, the named lane palettes, user theme files and the
//! contrast checker. The TUI resolves a [`ThemeSpec`] into its runtime
//! `Theme`; the CLI and MCP list and validate theme names from here.

mod contrast;
mod family;
mod file;
mod presets;
mod spec;
mod terminalcolors;
mod themes;

pub use contrast::{
    contrast_ratio, hover_fill, lighten, ContrastIssue, BUTTON_HOVER_LIFT, PRIMARY_HOVER_LIFT,
    SEGMENTED_HOVER_LIFT,
};
pub use family::{family_of, in_family, parse_selection, ALWAYS as ALWAYS_THEMES};
pub use file::{load_user_themes, parse_user_themes, ThemeSet, MAX_BASE_DEPTH};
pub use presets::{
    canonical_slug, is_imported, lane_palette, lane_palettes, lane_palettes_with, preset,
    preset_slugs, presets, theme_set, with_theme_line, LanePalette, DEFAULT_SLUG,
};
pub use spec::{
    ChromeColors, DiffColors, EditorColors, GraphColors, HostingColors, MergeColors, MinimapColors,
    RefColors, ScrollbarColors, StatusColors, ThemeColor, ThemeColorError, ThemeSpec,
    ToolbarColors, UiColors,
};

#[cfg(test)]
mod tests;
