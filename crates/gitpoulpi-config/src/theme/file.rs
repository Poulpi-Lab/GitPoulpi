//! User theme files: `<config>/themes/<slug>.toml`, partial overrides on a
//! base. Never fatal: whatever cannot be used is skipped and reported in
//! [`ThemeSet::warnings`].

use std::path::Path;

use super::presets::{preset, DEFAULT_SLUG};
use super::{ThemeColor, ThemeSpec};

/// The most `base` links a chain may follow (a user theme on a preset is 1).
pub const MAX_BASE_DEPTH: usize = 4;

/// The user themes that resolved, plus what was wrong with the rest.
#[derive(Debug, Clone, Default)]
pub struct ThemeSet {
    pub themes: Vec<ThemeSpec>,
    pub warnings: Vec<String>,
}

#[derive(Default)]
struct Raw {
    slug: String,
    name: Option<String>,
    base: Option<String>,
    light: Option<bool>,
    syntax: Option<String>,
    graph_palette: Option<Vec<ThemeColor>>,
    colors: Vec<(String, String, ThemeColor)>,
}

impl Raw {
    fn apply(&self, spec: &mut ThemeSpec) {
        spec.slug = self.slug.clone();
        spec.name = self.name.clone().unwrap_or_else(|| self.slug.clone());
        if let Some(light) = self.light {
            spec.light = light;
        }
        if let Some(syntax) = &self.syntax {
            spec.syntax = syntax.clone();
        }
        if let Some(palette) = &self.graph_palette {
            spec.graph_palette = palette.clone();
        }
        for (group, key, color) in &self.colors {
            spec.set(group, key, *color);
        }
    }
}

fn parse_raw(slug: &str, text: &str, warnings: &mut Vec<String>) -> Option<Raw> {
    let file = format!("themes/{slug}.toml");
    let table: toml::Table = match text.parse() {
        Ok(t) => t,
        Err(e) => {
            warnings.push(format!("{file}: not valid TOML, skipped ({e})"));
            return None;
        }
    };
    let mut raw = Raw {
        slug: slug.to_string(),
        ..Raw::default()
    };
    for (key, value) in &table {
        match (key.as_str(), value) {
            ("name", toml::Value::String(s)) => raw.name = Some(s.clone()),
            ("base", toml::Value::String(s)) => raw.base = Some(s.clone()),
            ("light", toml::Value::Boolean(b)) => raw.light = Some(*b),
            ("syntax", toml::Value::String(s)) => raw.syntax = Some(s.clone()),
            ("graph_palette", toml::Value::Array(items)) => {
                let mut colors = Vec::new();
                for item in items {
                    match item.as_str().map(str::parse::<ThemeColor>) {
                        Some(Ok(c)) => colors.push(c),
                        _ => warnings.push(format!(
                            "{file}: graph_palette entry {item} is not a color, ignored"
                        )),
                    }
                }
                if !colors.is_empty() {
                    raw.graph_palette = Some(colors);
                }
            }
            ("name" | "base" | "light" | "syntax" | "graph_palette", _) => {
                warnings.push(format!("{file}: `{key}` has the wrong type, ignored"));
            }
            (group, toml::Value::Table(fields)) if ThemeSpec::GROUPS.contains(&group) => {
                for (k, v) in fields {
                    let parsed = v.as_str().map(str::parse::<ThemeColor>);
                    match parsed {
                        Some(Ok(color)) => {
                            // Probe on a scratch spec so an unknown key is reported.
                            if super::presets::dark().set(group, k, color) {
                                raw.colors.push((group.to_string(), k.clone(), color));
                            } else {
                                warnings
                                    .push(format!("{file}: unknown key `{group}.{k}`, ignored"));
                            }
                        }
                        _ => warnings.push(format!(
                            "{file}: `{group}.{k}` = {v} is not a color, ignored"
                        )),
                    }
                }
            }
            _ => warnings.push(format!("{file}: unknown key `{key}`, ignored")),
        }
    }
    Some(raw)
}

/// Resolves `(slug, toml text)` pairs into themes.
pub fn parse_user_themes(sources: &[(String, String)]) -> ThemeSet {
    let mut set = ThemeSet::default();
    let mut raws: Vec<Raw> = Vec::new();
    for (slug, text) in sources {
        if preset(slug).is_some() || slug.eq_ignore_ascii_case("system") {
            set.warnings.push(format!(
                "themes/{slug}.toml: `{slug}` is a built-in theme name, skipped (rename the file)"
            ));
            continue;
        }
        if let Some(raw) = parse_raw(slug, text, &mut set.warnings) {
            raws.push(raw);
        }
    }

    for raw in &raws {
        let file = format!("themes/{}.toml", raw.slug);
        // Walk the base chain from this theme down to a preset.
        let mut chain: Vec<&Raw> = vec![raw];
        let mut root = DEFAULT_SLUG.to_string();
        let mut links = 0;
        let mut failed = None;
        loop {
            let current = chain[chain.len() - 1];
            let Some(base) = current.base.as_deref() else {
                break; // no base: the default preset
            };
            links += 1;
            if links > MAX_BASE_DEPTH {
                failed = Some(format!("`base` chain is longer than {MAX_BASE_DEPTH}"));
                break;
            }
            if preset(base).is_some() {
                root = base.to_string();
                break;
            }
            if chain.iter().any(|r| r.slug == base) {
                failed = Some(format!("`base` cycle through `{base}`"));
                break;
            }
            match raws.iter().find(|r| r.slug == base) {
                Some(next) => chain.push(next),
                None => {
                    set.warnings.push(format!(
                        "{file}: unknown base `{base}`, using `{DEFAULT_SLUG}`"
                    ));
                    break;
                }
            }
        }
        if let Some(why) = failed {
            set.warnings.push(format!("{file}: {why}, theme skipped"));
            continue;
        }
        let mut spec = preset(&root).unwrap_or_else(super::presets::dark);
        for link in chain.iter().rev() {
            link.apply(&mut spec);
        }
        // Inherited names must not leak: the label is this file's own.
        spec.slug = raw.slug.clone();
        spec.name = raw.name.clone().unwrap_or_else(|| raw.slug.clone());
        set.themes.push(spec);
    }
    set
}

/// Reads every `*.toml` of `dir` (sorted by name; a missing directory is
/// simply no themes) and resolves them.
pub fn load_user_themes(dir: &Path) -> ThemeSet {
    let mut sources = Vec::new();
    let mut warnings = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        let mut paths: Vec<_> = entries
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.extension().is_some_and(|x| x == "toml"))
            .collect();
        paths.sort();
        for path in paths {
            let Some(slug) = path.file_stem().and_then(|s| s.to_str()) else {
                continue;
            };
            match std::fs::read_to_string(&path) {
                Ok(text) => sources.push((slug.to_string(), text)),
                Err(e) => warnings.push(format!("themes/{slug}.toml: cannot read ({e})")),
            }
        }
    }
    let mut set = parse_user_themes(&sources);
    warnings.append(&mut set.warnings);
    set.warnings = warnings;
    set
}
