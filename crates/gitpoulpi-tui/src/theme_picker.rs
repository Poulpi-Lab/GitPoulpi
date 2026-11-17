//! `App: Change Theme...` (`Ctrl+N t`): the palette's second-level list of
//! every theme -- System first, the built-in presets in registry order, then
//! the user's `<config>/themes/*.toml`. Typing narrows it (the palette's
//! scorer), moving the highlight previews the theme live (the same
//! `install_theme` the Settings combo uses), Enter installs it and writes
//! `ui.theme`, Esc puts the previous theme back.

use gitpoulpi_config::theme::{self, ThemeSpec};
use gitpoulpi_config::{Config, ConfigPaths};
use gitpoulpi_i18n::msg;
use ratatui::style::Color;

/// The prefilter under the filter field.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum ThemeFilter {
    #[default]
    All,
    Dark,
    Light,
    Colored,
}

impl ThemeFilter {
    pub(crate) const ALL: [ThemeFilter; 4] = [
        ThemeFilter::All,
        ThemeFilter::Dark,
        ThemeFilter::Light,
        ThemeFilter::Colored,
    ];

    pub(crate) fn label(self) -> &'static str {
        match self {
            ThemeFilter::All => msg::overlays::THEME_FILTER_ALL.get(),
            ThemeFilter::Dark => msg::overlays::THEME_FILTER_DARK.get(),
            ThemeFilter::Light => msg::overlays::THEME_FILTER_LIGHT.get(),
            ThemeFilter::Colored => msg::overlays::THEME_FILTER_COLORED.get(),
        }
    }

    pub(crate) fn next(self, back: bool) -> Self {
        let i = Self::ALL.iter().position(|f| *f == self).unwrap_or(0);
        Self::ALL[(i + if back { 3 } else { 1 }) % 4]
    }
}

/// What the background of a theme says about it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Tone {
    pub(crate) light: bool,
    pub(crate) colored: bool,
}

/// Perceived brightness splits Dark from Light; a background is *colored*
/// when its chroma (max - min of the channels, 0..=255) shows a clear tint --
/// blue, violet, yellow... - while white, off-white, black and near-black
/// grays stay plain. A colored theme is also Dark or Light.
pub(crate) fn tone_of_rgb((r, g, b): (u8, u8, u8)) -> Tone {
    let (hi, lo) = (r.max(g).max(b), r.min(g).min(b));
    let luma = (299 * u32::from(r) + 587 * u32::from(g) + 114 * u32::from(b)) / 1000;
    Tone {
        light: luma >= 128,
        colored: hi - lo
            >= if luma >= 128 {
                COLORED_CHROMA_LIGHT
            } else {
                COLORED_CHROMA
            },
    }
}

/// Minimum channel spread of a colored background.
const COLORED_CHROMA: u8 = 12;
/// Light backgrounds are tinted a little by nature (cream, paper): it takes
/// more to count as colored.
const COLORED_CHROMA_LIGHT: u8 = 20;

/// The tone of a theme, `None` when its background is the terminal's own
/// (transparent): such a theme is neither dark nor light nor colored.
pub(crate) fn tone_of(spec: &ThemeSpec) -> Option<Tone> {
    spec.ui.bg.approx_rgb().map(tone_of_rgb)
}

impl ThemeFilter {
    fn admits(self, tone: Option<Tone>) -> bool {
        match (self, tone) {
            (ThemeFilter::All, _) => true,
            (_, None) => false,
            (ThemeFilter::Dark, Some(t)) => !t.light,
            (ThemeFilter::Light, Some(t)) => t.light,
            (ThemeFilter::Colored, Some(t)) => t.colored,
        }
    }
}

/// One row of the list.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ThemeChoice {
    pub(crate) name: String,
    pub(crate) slug: String,
    /// Background, text, accent and selection of the theme (none for System).
    pub(crate) chips: Vec<Color>,
    /// Dark / light / colored (none for System and transparent themes).
    pub(crate) tone: Option<Tone>,
}

fn chips_of(spec: &ThemeSpec) -> Vec<Color> {
    [spec.ui.bg, spec.ui.text, spec.ui.accent, spec.ui.selection]
        .into_iter()
        .map(crate::theme::color_of)
        .collect()
}

/// The list's content: System, the presets, then `user` themes.
pub(crate) fn choices(user: &[ThemeSpec]) -> Vec<ThemeChoice> {
    let mut out = vec![ThemeChoice {
        name: msg::settings_ui::THEME_SYSTEM.get().to_string(),
        slug: gitpoulpi_config::THEME_SYSTEM.to_string(),
        chips: Vec::new(),
        tone: None,
    }];
    out.extend(theme::presets().iter().chain(user).map(|t| ThemeChoice {
        name: t.name.clone(),
        slug: t.slug.clone(),
        chips: chips_of(t),
        tone: tone_of(t),
    }));
    out
}

/// The open picker: what `InputMode::ThemePicker` carries.
#[derive(Debug)]
pub(crate) struct ThemePicker {
    pub(crate) draft: String,
    /// Index into the *filtered* list.
    pub(crate) selected: usize,
    /// The prefilter under the filter field; combines with the draft.
    pub(crate) filter: ThemeFilter,
    /// The theme in force when the picker opened (ticked; Esc restores it).
    pub(crate) previous: String,
    /// The slug currently installed, preview included.
    shown: String,
    all: Vec<ThemeChoice>,
    user: Vec<ThemeSpec>,
    config_path: Option<std::path::PathBuf>,
}

impl ThemePicker {
    /// Opens against the real config (`config.toml` and `themes/`).
    pub(crate) fn open() -> Self {
        let paths = ConfigPaths::discover();
        let config_path = paths.as_ref().map(|p| p.config_file());
        let previous = config_path
            .as_deref()
            .and_then(|p| Config::load(p).ok())
            .map(|c| c.ui.theme_slug().to_string())
            .unwrap_or_else(|| gitpoulpi_config::THEME_SYSTEM.to_string());
        let user = paths
            .map(|p| theme::load_user_themes(&p.themes_dir()).themes)
            .unwrap_or_default();
        Self::with_parts(previous, user, config_path)
    }

    pub(crate) fn with_parts(
        previous: String,
        user: Vec<ThemeSpec>,
        config_path: Option<std::path::PathBuf>,
    ) -> Self {
        let all = choices(&user);
        let selected = all.iter().position(|c| c.slug == previous).unwrap_or(0);
        Self {
            draft: String::new(),
            selected,
            filter: ThemeFilter::All,
            shown: previous.clone(),
            previous,
            all,
            user,
            config_path,
        }
    }

    /// The rows matching the draft, best first (everything, in order, when
    /// the draft is empty).
    pub(crate) fn filtered(&self) -> Vec<&ThemeChoice> {
        let query = self.draft.trim().to_lowercase();
        let mut scored: Vec<(i32, usize, &ThemeChoice)> = self
            .all
            .iter()
            .enumerate()
            .filter(|(_, c)| self.filter.admits(c.tone))
            .filter_map(|(i, c)| crate::palette::score(&c.name, &query).map(|s| (s, i, c)))
            .collect();
        scored.sort_by_key(|(s, i, _)| (-*s, *i));
        scored.into_iter().map(|(_, _, c)| c).collect()
    }

    /// Switches the prefilter; the highlight goes back to the top.
    pub(crate) fn set_filter(&mut self, filter: ThemeFilter) {
        if filter != self.filter {
            self.filter = filter;
            self.selected = 0;
        }
    }

    pub(crate) fn highlighted(&self) -> Option<&ThemeChoice> {
        self.filtered().get(self.selected).copied()
    }

    /// Installs the highlighted theme (the live preview); a no-op while
    /// it is the one shown, or nothing matches.
    pub(crate) fn sync_preview(&mut self) {
        let Some(slug) = self.highlighted().map(|c| c.slug.clone()) else {
            return;
        };
        if slug != self.shown {
            crate::install_theme(&slug, &self.user);
            self.shown = slug;
        }
    }

    /// Esc, a click outside: the previous theme comes back.
    pub(crate) fn restore(&mut self) {
        if self.shown != self.previous {
            crate::install_theme(&self.previous, &self.user);
            self.shown = self.previous.clone();
        }
    }

    /// Enter: installs the highlighted theme and writes `ui.theme`. The
    /// status line message says what happened; a refused write restores.
    pub(crate) fn commit(&mut self) -> String {
        let Some(choice) = self.highlighted().cloned() else {
            self.restore();
            return msg::input::NO_MATCHING_THEME.get().to_string();
        };
        let Some(path) = self.config_path.clone() else {
            self.restore();
            return msg::settings_screen::NO_CONFIG_DIR.get().to_string();
        };
        match gitpoulpi_config::set_in_file(&path, "ui.theme", &choice.slug) {
            Ok(_) => {
                crate::install_theme(&choice.slug, &self.user);
                self.shown = choice.slug.clone();
                self.previous = choice.slug.clone();
                format!("ui.theme = {}", choice.slug)
            }
            Err(err) => {
                self.restore();
                format!("ui.theme: {err}")
            }
        }
    }
}

/// Paints the picker (see [`crate::picker_view`]).
pub(crate) fn render(
    frame: &mut ratatui::Frame,
    picker: &ThemePicker,
    field_editor: &crate::text_edit::FieldEditor<crate::modes::MainField>,
) -> crate::chrome::PalettePopup {
    use crate::picker_view::{render, PickerRow, PickerView};
    let found = picker.filtered();
    let rows: Vec<PickerRow> = found
        .iter()
        .map(|c| PickerRow {
            label: &c.name,
            ticked: c.slug == picker.previous,
            chips: &c.chips,
        })
        .collect();
    let view = PickerView {
        field: crate::modes::MainField::ThemePicker,
        win: "picker_theme",
        title: msg::overlays::change_theme_title(found.len(), picker.all.len()),
        draft: &picker.draft,
        selected: picker.selected,
        empty: msg::overlays::NO_MATCHING_THEME.get(),
        hint: msg::overlays::THEME_PICKER_HINT.get(),
        segments: Some((
            ThemeFilter::ALL.iter().map(|f| f.label()).collect(),
            ThemeFilter::ALL
                .iter()
                .position(|f| *f == picker.filter)
                .unwrap_or(0),
        )),
    };
    render(frame, &view, &rows, field_editor)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn user(slug: &str, name: &str) -> ThemeSpec {
        let mut spec = theme::preset(theme::DEFAULT_SLUG).expect("default preset");
        spec.slug = slug.into();
        spec.name = name.into();
        spec
    }

    #[test]
    fn system_comes_first_then_presets_in_registry_order_then_user_themes() {
        let list = choices(&[user("mine", "Mine")]);
        assert_eq!(list[0].slug, gitpoulpi_config::THEME_SYSTEM);
        let presets: Vec<String> = theme::presets().into_iter().map(|t| t.slug).collect();
        let slugs: Vec<&str> = list.iter().map(|c| c.slug.as_str()).collect();
        assert_eq!(&slugs[1..=presets.len()], presets.as_slice());
        assert_eq!(*slugs.last().unwrap(), "mine");
        assert_eq!(slugs.len(), presets.len() + 2);
        assert!(list[0].chips.is_empty() && list[1].chips.len() == 4);
    }

    fn tone(name: &str) -> Tone {
        let spec = theme::presets()
            .into_iter()
            .find(|t| t.name == name)
            .unwrap_or_else(|| panic!("{name}"));
        tone_of(&spec).expect("an opaque background")
    }

    #[test]
    fn the_classifier_splits_dark_light_and_colored_on_known_presets() {
        // Tinted backgrounds: colored, and still dark or light.
        for dark in [
            "Solarized Dark",
            "Dracula",
            "Nord",
            "Tokyo Night",
            "Shades of Purple",
        ] {
            assert_eq!(
                tone(dark),
                Tone {
                    light: false,
                    colored: true
                },
                "{dark}"
            );
        }
        for light in ["Solarized Light", "Gruvbox Light", "Noctis Lux"] {
            assert_eq!(
                tone(light),
                Tone {
                    light: true,
                    colored: true
                },
                "{light}"
            );
        }
        // Grays and (off-)whites are plain.
        for dark in [
            "GitHub Dark",
            "VS Code Dark",
            "Monokai",
            "Tomorrow Night Bright",
        ] {
            assert_eq!(
                tone(dark),
                Tone {
                    light: false,
                    colored: false
                },
                "{dark}"
            );
        }
        for light in [
            "GitHub Light",
            "Catppuccin Latte",
            "One Light",
            "Rosé Pine Dawn",
        ] {
            assert_eq!(
                tone(light),
                Tone {
                    light: true,
                    colored: false
                },
                "{light}"
            );
        }
    }

    #[test]
    fn system_is_only_under_all_and_the_prefilter_combines_with_the_draft() {
        let mut picker = ThemePicker::with_parts("dark".into(), Vec::new(), None);
        let all = picker.filtered().len();
        let slugs = |p: &ThemePicker| {
            p.filtered()
                .iter()
                .map(|c| c.slug.clone())
                .collect::<Vec<_>>()
        };
        assert!(slugs(&picker).contains(&gitpoulpi_config::THEME_SYSTEM.to_string()));
        for f in [ThemeFilter::Dark, ThemeFilter::Light, ThemeFilter::Colored] {
            picker.set_filter(f);
            assert!(!slugs(&picker).contains(&gitpoulpi_config::THEME_SYSTEM.to_string()));
        }
        picker.set_filter(ThemeFilter::Dark);
        let dark = picker.filtered().len();
        picker.set_filter(ThemeFilter::Light);
        let light = picker.filtered().len();
        assert!(dark > 0 && light > 0 && dark + light <= all);
        picker.set_filter(ThemeFilter::Colored);
        picker.draft = "solarized".into();
        let names: Vec<_> = picker.filtered().iter().map(|c| c.name.clone()).collect();
        assert!(names.contains(&"Solarized Dark".to_string()), "{names:?}");
        picker.draft = "github dark".into();
        assert!(picker.filtered().iter().all(|c| c.name != "GitHub Dark"));
        assert_eq!(ThemeFilter::All.next(true), ThemeFilter::Colored);
        assert_eq!(ThemeFilter::Colored.next(false), ThemeFilter::All);
    }

    #[test]
    fn it_opens_on_the_current_theme_and_narrows_by_typing() {
        let mut picker = ThemePicker::with_parts("light".into(), vec![user("mine", "Mine")], None);
        assert_eq!(picker.highlighted().unwrap().slug, "light");
        picker.draft = "mine".into();
        picker.selected = 0;
        let rows = picker.filtered();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].slug, "mine");
    }

    #[test]
    fn commit_writes_ui_theme_and_without_a_config_dir_says_so() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        let mut picker = ThemePicker::with_parts("dark".into(), Vec::new(), Some(path.clone()));
        // "light" alone also matches Gruvbox Light: the label is exact.
        picker.draft = "gitpoulpi light".into();
        picker.selected = 0;
        let _guard = crate::theme::override_theme(crate::theme::th());
        assert_eq!(picker.commit(), "ui.theme = light");
        assert!(std::fs::read_to_string(path).unwrap().contains("light"));
        let mut none = ThemePicker::with_parts("dark".into(), Vec::new(), None);
        none.draft = "light".into();
        assert_ne!(none.commit(), "ui.theme = light");
    }
}
