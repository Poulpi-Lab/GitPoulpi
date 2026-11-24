//! The Theme and lane-palette rows of Settings: their option lists, the
//! swatch strip under the Theme row, the live preview while the Theme
//! combo's highlight moves, and the user-theme rescan.

use super::*;
use crate::th;
use gitpoulpi_config::theme;
use gitpoulpi_i18n::msg;
use ratatui::style::Color;

/// How many lane colors the swatch strip and a lane option show.
const LANE_CHIPS: usize = 6;

impl PreferencesState {
    /// `<config>/themes`, next to the `config.toml` this screen edits.
    fn themes_dir(&self) -> Option<PathBuf> {
        self.config_path
            .as_deref()
            .and_then(Path::parent)
            .map(|dir| dir.join("themes"))
    }

    /// Re-reads the user theme files (the Theme combo calls this as it
    /// opens, so a file added while Settings was open shows up).
    pub(super) fn rescan_user_themes(&mut self) {
        self.user_themes = self
            .themes_dir()
            .map(|dir| theme::load_user_themes(&dir))
            .unwrap_or_default();
        crate::theme::refresh_user_lanes(&self.user_themes.themes);
    }

    /// The Theme combo's options: System, the presets, then user themes.
    pub(super) fn theme_options(&self) -> Vec<(String, String)> {
        let mut out = vec![(
            msg::settings_ui::THEME_SYSTEM.get().to_string(),
            gitpoulpi_config::THEME_SYSTEM.to_string(),
        )];
        out.extend(theme::presets().into_iter().map(|t| (t.name, t.slug)));
        out.extend(
            self.user_themes
                .themes
                .iter()
                .map(|t| (t.name.clone(), t.slug.clone())),
        );
        out
    }

    /// The lane palette combo's options: Follow theme, then every palette
    /// of the registry (built-ins, per theme, user `graph_palette`s).
    /// `current` is the configured value: a name that is no palette any more
    /// (a deleted user theme) gets its own row saying so; the graph then uses
    /// the active theme's lanes.
    pub(super) fn lane_options(&self, current: &str) -> Vec<(String, String)> {
        let mut out = vec![(
            msg::settings_ui::FOLLOW_THEME.get().to_string(),
            gitpoulpi_config::GRAPH_COLOR_FOLLOW_THEME.to_string(),
        )];
        let palettes = self.lane_palettes();
        if current != gitpoulpi_config::GRAPH_COLOR_FOLLOW_THEME
            && !palettes.iter().any(|p| p.slug == current)
        {
            out.push((
                msg::settings_ui::graph_color_missing(current),
                current.to_string(),
            ));
        }
        out.extend(palettes.into_iter().map(|p| (p.label, p.slug)));
        out
    }

    fn lane_palettes(&self) -> Vec<theme::LanePalette> {
        let mut specs = theme::presets();
        specs.extend(self.user_themes.themes.iter().cloned());
        theme::lane_palettes_with(&specs)
    }

    /// Chips for the lane palette option `slug` (its first colors).
    pub(super) fn lane_swatch(&self, slug: &str) -> Vec<Color> {
        self.lane_palettes()
            .into_iter()
            .find(|p| p.slug == slug)
            .map(|p| {
                p.colors
                    .into_iter()
                    .take(LANE_CHIPS)
                    .map(crate::theme::color_of)
                    .collect()
            })
            .unwrap_or_default()
    }

    /// The strip under the Theme row: bg, text, muted, accent, selection,
    /// danger, diff add, diff del, then the first lane colors - of the
    /// theme currently installed, so it follows the live preview.
    pub(super) fn theme_swatch_row(&self) -> Field {
        let t = th();
        let bg = if t.ui.bg == Color::Reset {
            t.ui.blend_bg
        } else {
            t.ui.bg
        };
        let mut chips = vec![
            bg,
            t.ui.text,
            t.ui.muted,
            t.ui.accent,
            t.ui.selection,
            t.status.danger,
            t.diff.add_bg,
            t.diff.del_bg,
        ];
        chips.extend(t.graph.palette.iter().take(LANE_CHIPS).copied());
        Field {
            label: format!("  {}", msg::settings_ui::THEME_SWATCHES.get()),
            control: Control::Swatches(chips),
            help: None,
            warning: None,
            danger: false,
            target: Target::None,
            live: true,
            coming: None,
            placeholder: None,
            ellipsize: None,
        }
    }

    /// The slug the Theme combo is highlighting right now, if it is open.
    fn highlighted_theme(&self) -> Option<String> {
        let (index, combo) = self.choice.as_ref()?;
        let field = self.fields().into_iter().nth(*index)?;
        if field.target != Target::Global("ui.theme") {
            return None;
        }
        combo.highlighted_value().map(str::to_string)
    }

    /// Keeps the installed theme in step with the combo: the highlighted
    /// theme while the Theme combo is open (the live preview), the stored
    /// one otherwise (Esc, a click away, a committed pick all land here).
    pub(super) fn sync_theme_preview(&mut self) {
        let wanted = self.highlighted_theme();
        if wanted == self.theme_preview {
            return;
        }
        let slug = wanted
            .clone()
            .unwrap_or_else(|| self.own_config().ui.theme_slug().to_string());
        crate::install_theme(&slug, &self.user_themes.themes);
        self.theme_preview = wanted;
    }

    /// Installs the stored theme (after a commit, with the freshest user
    /// theme list).
    pub(super) fn install_stored_theme(&mut self) {
        crate::install_theme(self.own_config().ui.theme_slug(), &self.user_themes.themes);
    }
}
