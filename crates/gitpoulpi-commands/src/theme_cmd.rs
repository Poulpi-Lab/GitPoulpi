//! `gitpoulpi theme list|export`, and the name checks `config set` uses
//! for `ui.theme` and `ui.graph_color_theme`.
//!
//! Everything comes from `gitpoulpi_config::theme`'s registry (the
//! presets, the lane palettes and the user files of `<config>/themes/`),
//! never from a hard-coded list, so a preset added there shows up here
//! (and in the MCP tools) by itself.

use gitpoulpi_config::theme::{self, ThemeSet, ThemeSpec};
use gitpoulpi_config::ConfigPaths;
use serde_json::json;

use crate::cli::ThemeAction;
use crate::commands::emitted;
use crate::repo_cmd::Emitted;

/// The user themes of the config directory (none when it cannot be found).
fn user_set() -> ThemeSet {
    ConfigPaths::discover()
        .map(|p| theme::load_user_themes(&p.themes_dir()))
        .unwrap_or_default()
}

/// Every theme: presets first, then user themes. `(spec, is_user)`.
fn all(set: &ThemeSet) -> Vec<(ThemeSpec, bool)> {
    theme::presets()
        .into_iter()
        .map(|t| (t, false))
        .chain(set.themes.iter().cloned().map(|t| (t, true)))
        .collect()
}

/// The name of the lane palette a theme brings: its own entry in the
/// registry when it has one, else the first palette with the same colors.
fn lane_palette_name(spec: &ThemeSpec, palettes: &[theme::LanePalette]) -> Option<String> {
    palettes
        .iter()
        .find(|p| p.slug == spec.slug && p.colors == spec.graph_palette)
        .or_else(|| palettes.iter().find(|p| p.colors == spec.graph_palette))
        .map(|p| p.slug.clone())
}

pub fn execute(action: ThemeAction) -> Result<Emitted, String> {
    let set = user_set();
    let themes = all(&set);
    match action {
        ThemeAction::List => {
            let specs: Vec<ThemeSpec> = themes.iter().map(|(t, _)| t.clone()).collect();
            let palettes = theme::lane_palettes_with(&specs);
            let rows: Vec<_> = themes
                .iter()
                .map(|(t, user)| {
                    json!({
                        "slug": t.slug,
                        "label": t.name,
                        "source": if *user { "user" } else { "preset" },
                        "light": t.light,
                        "lane_palette": lane_palette_name(t, &palettes),
                    })
                })
                .collect();
            let mut human = String::new();
            for (t, user) in &themes {
                human.push_str(&format!(
                    "{:<20} {}{}{}\n",
                    t.slug,
                    t.name,
                    if t.light { "  (light)" } else { "" },
                    if *user { "  [user]" } else { "" },
                ));
            }
            for w in &set.warnings {
                human.push_str(&format!("warning: {w}\n"));
            }
            Ok(emitted(
                json!({ "themes": rows, "warnings": set.warnings }),
                human.trim_end(),
            ))
        }
        ThemeAction::Export { slug } => {
            let (spec, _) = themes
                .iter()
                .find(|(t, _)| t.slug == slug)
                .ok_or_else(|| unknown_theme(&slug, &themes))?;
            let toml = spec.to_toml();
            Ok(emitted(
                json!({ "slug": spec.slug, "toml": toml }),
                toml.trim_end(),
            ))
        }
    }
}

fn unknown_theme(slug: &str, themes: &[(ThemeSpec, bool)]) -> String {
    let names: Vec<&str> = themes.iter().map(|(t, _)| t.slug.as_str()).collect();
    format!("unknown theme `{slug}`; valid names: {}", names.join(", "))
}

/// Checks a `ui.theme` value: `system`, the legacy `default`, a preset or
/// a user theme of `<config>/themes/`. The error lists the valid names.
pub fn validate_ui_theme(value: &str) -> Result<(), String> {
    if matches!(value, "system" | "default") {
        return Ok(());
    }
    let themes = all(&user_set());
    if themes.iter().any(|(t, _)| t.slug == value) {
        return Ok(());
    }
    let names: Vec<&str> = themes.iter().map(|(t, _)| t.slug.as_str()).collect();
    Err(format!(
        "unknown theme `{value}` for ui.theme; valid names: system, {}",
        names.join(", ")
    ))
}

/// Checks a `ui.graph_color_theme` value: `theme` (follow the theme) or
/// the name of any lane palette (built-in, per theme, or a user theme's
/// `graph_palette`).
pub fn validate_graph_color_theme(value: &str) -> Result<(), String> {
    if value == "theme" {
        return Ok(());
    }
    let specs: Vec<ThemeSpec> = all(&user_set()).into_iter().map(|(t, _)| t).collect();
    let palettes = theme::lane_palettes_with(&specs);
    if palettes.iter().any(|p| p.slug == value) {
        return Ok(());
    }
    let names: Vec<&str> = palettes.iter().map(|p| p.slug.as_str()).collect();
    Err(format!(
        "unknown lane palette `{value}` for ui.graph_color_theme; valid names: theme, {}",
        names.join(", ")
    ))
}
