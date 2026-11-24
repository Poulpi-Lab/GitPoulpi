use super::*;
use crate::theme::{override_theme, Theme};

fn key_of(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

/// A state on Appearance with the row of config key `key` selected, and the
/// thread's theme pinned so the preview never moves the global theme.
fn on_row(dir: &Path, section: PrefSection, key: &'static str) -> (PreferencesState, impl Sized) {
    let guard = override_theme(&Theme::dark());
    let mut state = state(dir);
    go(&mut state, section);
    state.focus = PrefFocus::Fields;
    state.field = state
        .fields()
        .iter()
        .position(|f| f.target == Target::Global(key))
        .unwrap();
    (state, guard)
}

fn options(state: &PreferencesState) -> Vec<String> {
    state
        .choice
        .as_ref()
        .unwrap()
        .1
        .options
        .iter()
        .map(|o| o.value.clone())
        .collect()
}

#[test]
fn the_theme_combo_lists_system_the_presets_and_user_themes_rescanned_on_open() {
    let dir = tempfile::tempdir().unwrap();
    let (mut state, _g) = on_row(dir.path(), PrefSection::Appearance, "ui.theme");
    // A theme file dropped in while Settings is already open.
    std::fs::create_dir_all(dir.path().join("themes")).unwrap();
    std::fs::write(
        dir.path().join("themes/x.toml"),
        "name = \"My X\"\nbase = \"nord\"\n",
    )
    .unwrap();
    state.on_key(key_of(KeyCode::Enter));
    let values = options(&state);
    assert_eq!(values[0], "system");
    for slug in gitpoulpi_config::theme::preset_slugs() {
        assert!(values.iter().any(|v| v == slug), "{slug} in {values:?}");
    }
    assert_eq!(values.last().unwrap(), "x");
    let labels: Vec<_> = state
        .choice
        .as_ref()
        .unwrap()
        .1
        .options
        .iter()
        .map(|o| o.label.clone())
        .collect();
    assert_eq!(labels[0], "System (follows terminal)");
    assert!(labels.contains(&"My X".to_string()));
}

#[test]
fn moving_the_theme_highlight_previews_it_and_esc_restores() {
    let dir = tempfile::tempdir().unwrap();
    let (mut state, _g) = on_row(dir.path(), PrefSection::Appearance, "ui.theme");
    state.on_key(key_of(KeyCode::Enter));
    assert_eq!(state.theme_preview.as_deref(), Some("system"));
    state.on_key(key_of(KeyCode::Down));
    assert_eq!(state.theme_preview.as_deref(), Some("dark"));
    state.on_key(key_of(KeyCode::Down));
    assert_eq!(state.theme_preview.as_deref(), Some("light"));
    state.on_key(key_of(KeyCode::Esc));
    assert_eq!(state.theme_preview, None, "Esc ends the preview");
    assert_eq!(state.config.ui.theme_slug(), "system", "nothing saved");
    assert!(!dir.path().join("config.toml").exists());
}

#[test]
fn enter_commits_the_highlighted_theme() {
    let dir = tempfile::tempdir().unwrap();
    let (mut state, _g) = on_row(dir.path(), PrefSection::Appearance, "ui.theme");
    state.on_key(key_of(KeyCode::Enter));
    state.on_key(key_of(KeyCode::Down));
    state.on_key(key_of(KeyCode::Down));
    state.on_key(key_of(KeyCode::Enter));
    // This (non-primary) installation keeps the pick in its own entry and
    // leaves the shared `ui.theme` alone.
    assert_eq!(state.own_config().ui.theme_slug(), "light");
    assert_eq!(state.config.ui.theme_slug(), "system");
    assert_eq!(state.theme_preview, None);
    let raw = std::fs::read_to_string(dir.path().join("config.toml")).unwrap();
    assert!(raw.contains("theme = \"light\""), "{raw}");
    assert!(!raw.contains("[ui]"), "{raw}");
}

#[test]
fn the_lane_combo_offers_follow_theme_and_every_palette_with_swatches() {
    let dir = tempfile::tempdir().unwrap();
    let (mut state, _g) = on_row(dir.path(), PrefSection::Appearance, "ui.graph_color_theme");
    state.on_key(key_of(KeyCode::Enter));
    let values = options(&state);
    assert_eq!(values[0], "theme");
    for p in gitpoulpi_config::theme::lane_palettes() {
        assert!(values.contains(&p.slug), "{}", p.slug);
    }
    assert_eq!(state.choice.as_ref().unwrap().1.selected(), Some("theme"));
    let opts = &state.choice.as_ref().unwrap().1.options;
    assert!(opts[0].swatch.is_empty());
    assert_eq!(opts[1].swatch.len(), 6);
    // Picking one stores its slug.
    state.on_key(key_of(KeyCode::Down));
    state.on_key(key_of(KeyCode::Enter));
    let raw = std::fs::read_to_string(dir.path().join("config.toml")).unwrap();
    assert!(raw.contains("graph_color_theme = \"gitpoulpi\""), "{raw}");
}

#[test]
fn the_swatch_row_sits_under_the_theme_row() {
    let dir = tempfile::tempdir().unwrap();
    let (state, _g) = on_row(dir.path(), PrefSection::Appearance, "ui.theme");
    let fields = state.fields();
    let Control::Swatches(chips) = &fields[state.field + 1].control else {
        panic!("no swatch row under the theme row");
    };
    assert_eq!(chips.len(), 8 + 6);
    assert!(!fields[state.field + 1].selectable());
}

#[test]
fn the_lane_combo_lists_user_themes_after_the_palettes_and_rescans_on_open() {
    let dir = tempfile::tempdir().unwrap();
    let (mut state, _g) = on_row(dir.path(), PrefSection::Appearance, "ui.graph_color_theme");
    // Dropped in while Settings is already open.
    std::fs::create_dir_all(dir.path().join("themes")).unwrap();
    std::fs::write(
        dir.path().join("themes/mine.toml"),
        "name = \"My Lanes\"\ngraph_palette = [\"#112233\", \"#445566\"]\n",
    )
    .unwrap();
    std::fs::write(
        dir.path().join("themes/plain.toml"),
        "name = \"Plain\"\nbase = \"nord\"\n",
    )
    .unwrap();
    state.on_key(key_of(KeyCode::Enter));
    let values = options(&state);
    let opts = &state.choice.as_ref().unwrap().1.options;
    let at = values
        .iter()
        .position(|v| v == "mine")
        .expect("custom listed");
    assert_eq!(opts[at].label, "My Lanes");
    assert_eq!(opts[at].swatch.len(), 2);
    let last_preset = values.iter().position(|v| v == "vscode-light").unwrap();
    assert!(at > last_preset, "custom themes come after the presets");
    // No lane colors of its own: it inherits its base's, like the main theme.
    let plain = values
        .iter()
        .position(|v| v == "plain")
        .expect("plain listed");
    assert!(!opts[plain].swatch.is_empty());
}

#[test]
fn a_configured_lane_theme_that_no_longer_exists_is_shown_as_missing() {
    let dir = tempfile::tempdir().unwrap();
    let state = state(dir.path());
    let options = state.lane_options("gone-theme");
    assert_eq!(options[0].1, "theme");
    assert_eq!(options[1].1, "gone-theme");
    assert!(options[1].0.contains("not found"), "{:?}", options[1]);
    // A known name or Follow theme adds no such row.
    for known in ["theme", "nord"] {
        assert!(state
            .lane_options(known)
            .iter()
            .all(|(l, _)| !l.contains("not found")));
    }
}
