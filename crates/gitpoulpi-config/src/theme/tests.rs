use super::*;
use crate::{ConfigPaths, UiConfig};

fn parse(files: &[(&str, &str)]) -> ThemeSet {
    let owned: Vec<_> = files
        .iter()
        .map(|(s, t)| (s.to_string(), t.to_string()))
        .collect();
    parse_user_themes(&owned)
}

/// The terminal's own colors cannot be measured, but a text drawn in the
/// very color of its background is invisible in every terminal.
#[test]
fn the_terminal_preset_never_draws_a_text_in_its_background_color() {
    let same: Vec<String> = preset("terminal")
        .unwrap()
        .contrast_issues()
        .into_iter()
        .filter(|i| i.ratio < 1.01)
        .map(|i| i.rule)
        .collect();
    assert!(same.is_empty(), "same color on both sides: {same:?}");
}

#[test]
fn presets_pass_the_contrast_rules() {
    let mut failures = Vec::new();
    for spec in presets() {
        if spec.slug == "terminal" {
            continue; // every color is the terminal's own: unknown
        }
        for i in spec.contrast_issues() {
            failures.push(format!(
                "{}: {} {:.2} < {}",
                spec.slug, i.rule, i.ratio, i.min
            ));
        }
    }
    assert!(failures.is_empty(), "{failures:#?}");
}

#[test]
fn a_deliberately_bad_spec_fails_the_contrast_rules() {
    let mut bad = preset("dark").unwrap();
    bad.ui.text = bad.ui.panel;
    bad.ui.hover = bad.ui.selection;
    bad.graph_palette = vec![bad.effective_bg()];
    let rules: Vec<_> = bad.contrast_issues().into_iter().map(|i| i.rule).collect();
    assert!(rules.contains(&"text/panel".to_string()), "{rules:?}");
    assert!(rules.iter().any(|r| r.contains("selection must differ")));
    assert!(rules.contains(&"graph_palette[0]/bg".to_string()));
}

#[test]
fn contrast_ratio_matches_wcag() {
    let r = contrast_ratio(ThemeColor::Rgb(0, 0, 0), ThemeColor::Rgb(255, 255, 255)).unwrap();
    assert!((r - 21.0).abs() < 1e-9);
    assert_eq!(
        contrast_ratio(ThemeColor::Default, ThemeColor::Rgb(0, 0, 0)),
        None
    );
}

#[test]
fn colors_parse_and_print() {
    assert_eq!("#1a2B3c".parse(), Ok(ThemeColor::Rgb(0x1a, 0x2b, 0x3c)));
    assert_eq!("ansi:12".parse(), Ok(ThemeColor::Ansi(12)));
    assert_eq!("default".parse(), Ok(ThemeColor::Default));
    for bad in ["#12345", "#gggggg", "ansi:300", "ansi:", "red", "123456"] {
        assert!(bad.parse::<ThemeColor>().is_err(), "{bad}");
    }
    assert_eq!(ThemeColor::Rgb(1, 2, 255).to_string(), "#0102ff");
}

#[test]
fn every_preset_has_a_palette_and_unique_slug() {
    let all = presets();
    for (i, a) in all.iter().enumerate() {
        assert!(a.graph_palette.len() >= 6, "{}", a.slug);
        assert_eq!(preset(&a.slug).as_ref(), Some(a));
        assert!(all[i + 1..].iter().all(|b| b.slug != a.slug));
    }
    assert_eq!(
        preset_slugs(),
        all.iter().map(|s| s.slug.as_str()).collect::<Vec<_>>()
    );
    assert_eq!(preset("dark").unwrap().ui.bg, ThemeColor::Default);
}

#[test]
fn lane_palettes_hold_the_four_originals_and_user_palettes() {
    let slugs: Vec<_> = lane_palettes().into_iter().map(|p| p.slug).collect();
    for s in ["gitpoulpi", "light", "material", "serie"] {
        assert!(slugs.iter().any(|x| x == s), "{s}");
    }
    assert_eq!(lane_palette("material").unwrap().colors.len(), 16);
    assert_eq!(lane_palette("serie").unwrap().colors.len(), 6);
    let user = parse(&[("mine", "graph_palette = [\"#112233\"]\n")]);
    let with = lane_palettes_with(&user.themes);
    assert_eq!(with.last().unwrap().slug, "mine");
}

#[test]
fn export_round_trips_through_the_parser() {
    let mut light = preset("light").unwrap();
    light.name = "Round".into();
    let set = parse(&[("round", &light.to_toml())]);
    assert!(set.warnings.is_empty(), "{:?}", set.warnings);
    let mut got = set.themes[0].clone();
    got.slug = "light".into();
    got.name = "Light".into();
    light.name = "Light".into();
    assert_eq!(got, light);
}

#[test]
fn a_partial_override_keeps_the_rest_of_the_base() {
    let set = parse(&[(
        "mine",
        "name = \"My Dark\"\n[ui]\naccent = \"#ff0000\"\n[diff]\nadd_bg = \"ansi:22\"\n",
    )]);
    assert!(set.warnings.is_empty(), "{:?}", set.warnings);
    let t = &set.themes[0];
    let dark = preset("dark").unwrap();
    assert_eq!((t.slug.as_str(), t.name.as_str()), ("mine", "My Dark"));
    assert_eq!(t.ui.accent, ThemeColor::Rgb(255, 0, 0));
    assert_eq!(t.diff.add_bg, ThemeColor::Ansi(22));
    assert_eq!(t.ui.text, dark.ui.text);
    assert_eq!(t.syntax, dark.syntax);
    assert!(!t.light);
}

#[test]
fn name_defaults_to_the_file_stem_and_light_syntax_palette_override() {
    let set = parse(&[(
        "paper",
        "base = \"light\"\nlight = false\nsyntax = \"x\"\ngraph_palette = [\"#010203\", \"ansi:1\"]\n",
    )]);
    let t = &set.themes[0];
    assert_eq!(t.name, "paper");
    assert!(!t.light);
    assert_eq!(t.syntax, "x");
    assert_eq!(
        t.graph_palette,
        vec![ThemeColor::Rgb(1, 2, 3), ThemeColor::Ansi(1)]
    );
    assert_eq!(t.ui.bg, preset("light").unwrap().ui.bg);
}

#[test]
fn a_base_chain_inherits_through_user_themes() {
    let set = parse(&[
        ("a", "base = \"b\"\n[ui]\ntext = \"#010101\"\n"),
        (
            "b",
            "base = \"light\"\n[ui]\ntext = \"#020202\"\nmuted = \"#030303\"\n",
        ),
    ]);
    assert!(set.warnings.is_empty(), "{:?}", set.warnings);
    let a = set.themes.iter().find(|t| t.slug == "a").unwrap();
    assert_eq!(a.ui.text, ThemeColor::Rgb(1, 1, 1));
    assert_eq!(a.ui.muted, ThemeColor::Rgb(3, 3, 3));
    assert_eq!(a.ui.bg, preset("light").unwrap().ui.bg);
}

#[test]
fn a_cycle_is_rejected_with_a_warning() {
    let set = parse(&[("a", "base = \"b\"\n"), ("b", "base = \"a\"\n"), ("ok", "")]);
    assert_eq!(set.themes.len(), 1);
    assert_eq!(set.themes[0].slug, "ok");
    assert_eq!(
        set.warnings.iter().filter(|w| w.contains("cycle")).count(),
        2
    );
    let own = parse(&[("a", "base = \"a\"\n")]);
    assert!(own.themes.is_empty() && own.warnings[0].contains("cycle"));
}

#[test]
fn the_base_chain_depth_is_limited_to_four_links() {
    // t1 -> t2 -> t3 -> t4 -> dark is 4 links: fine. t0 -> t1 makes 5.
    let ok = parse(&[
        ("t1", "base = \"t2\"\n"),
        ("t2", "base = \"t3\"\n"),
        ("t3", "base = \"t4\"\n"),
        ("t4", "base = \"dark\"\n"),
    ]);
    assert_eq!(ok.themes.len(), 4, "{:?}", ok.warnings);
    let too_deep = parse(&[
        ("t0", "base = \"t1\"\n"),
        ("t1", "base = \"t2\"\n"),
        ("t2", "base = \"t3\"\n"),
        ("t3", "base = \"t4\"\n"),
        ("t4", "base = \"dark\"\n"),
    ]);
    assert!(too_deep.themes.iter().all(|t| t.slug != "t0"));
    assert!(too_deep.themes.iter().any(|t| t.slug == "t1"));
    assert!(too_deep
        .warnings
        .iter()
        .any(|w| w.contains("t0") && w.contains("longer")));
}

#[test]
fn unknown_keys_and_bad_values_warn_but_never_fail() {
    let set = parse(&[(
        "x",
        "wobble = 1\n[ui]\nnope = \"#000000\"\naccent = \"#12\"\ntext = 5\nmuted = \"#0a0b0c\"\n[bogus]\na = \"#000000\"\n",
    )]);
    let t = &set.themes[0];
    assert_eq!(t.ui.muted, ThemeColor::Rgb(10, 11, 12));
    assert_eq!(t.ui.accent, preset("dark").unwrap().ui.accent);
    let w = set.warnings.join("\n");
    for needle in [
        "`wobble`",
        "`ui.nope`",
        "`ui.accent`",
        "`ui.text`",
        "`bogus`",
    ] {
        assert!(w.contains(needle), "{needle} in {w}");
    }
}

#[test]
fn default_and_ansi_colors_are_accepted() {
    let set = parse(&[("t", "[ui]\nbg = \"default\"\ntext = \"ansi:15\"\n")]);
    assert!(set.warnings.is_empty());
    assert_eq!(set.themes[0].ui.text, ThemeColor::Ansi(15));
    let set = parse(&[("l", "base = \"light\"\n[ui]\nbg = \"default\"\n")]);
    assert_eq!(set.themes[0].ui.bg, ThemeColor::Default);
    // Contrast falls back to blend_bg when bg is default.
    assert_eq!(
        set.themes[0].effective_bg(),
        preset("light").unwrap().ui.blend_bg
    );
}

#[test]
fn builtin_names_invalid_toml_and_unknown_bases_are_handled() {
    let set = parse(&[
        ("dark", "[ui]\naccent = \"#000000\"\n"),
        ("system", ""),
        ("broken", "this is = = not toml"),
        ("lost", "base = \"nowhere\"\n[ui]\naccent = \"#0000ff\"\n"),
    ]);
    assert_eq!(set.themes.len(), 1);
    let lost = &set.themes[0];
    assert_eq!(lost.ui.text, preset("dark").unwrap().ui.text);
    assert_eq!(lost.ui.accent, ThemeColor::Rgb(0, 0, 255));
    let w = set.warnings.join("\n");
    assert!(w.contains("built-in") && w.contains("not valid TOML") && w.contains("unknown base"));
}

#[test]
fn themes_load_from_the_themes_dir() {
    let dir = tempfile::tempdir().unwrap();
    let paths = ConfigPaths::for_dir(dir.path());
    assert!(load_user_themes(&paths.themes_dir()).themes.is_empty());
    std::fs::create_dir_all(paths.themes_dir()).unwrap();
    std::fs::write(
        paths.themes_dir().join("b.toml"),
        "[ui]\naccent = \"#000001\"\n",
    )
    .unwrap();
    std::fs::write(paths.themes_dir().join("a.toml"), "").unwrap();
    std::fs::write(paths.themes_dir().join("notes.txt"), "x").unwrap();
    let set = load_user_themes(&paths.themes_dir());
    let slugs: Vec<_> = set.themes.iter().map(|t| t.slug.as_str()).collect();
    assert_eq!(slugs, ["a", "b"]);
}

#[test]
fn the_legacy_ui_theme_values_map_to_slugs() {
    let slug = |v: &str| {
        let ui: UiConfig = toml::from_str(&format!("theme = \"{v}\"\n")).unwrap();
        ui.theme_slug().to_string()
    };
    assert_eq!(slug("default"), "system");
    assert_eq!(slug("system"), "system");
    assert_eq!(slug("dark"), "dark");
    assert_eq!(slug("light"), "light");
    assert_eq!(UiConfig::default().theme_slug(), "system");
}

#[test]
fn fire_and_sunset_parse_from_config_and_are_colored_dark_presets() {
    for slug in ["fire", "sunset"] {
        let ui: UiConfig = toml::from_str(&format!("theme = \"{slug}\"\n")).unwrap();
        assert_eq!(ui.theme_slug(), slug);
        // A custom build (`GITPOULPI_THEMES`) may leave the preset out.
        let Some(spec) = preset(slug) else { continue };
        assert!(!spec.light);
        assert!(lane_palettes().iter().any(|p| p.slug == slug));
        // The picker's Colored prefilter needs a tinted background.
        let (r, g, b) = rgb(&spec, |t| t.ui.bg);
        assert!(r.max(g).max(b) - r.min(g).min(b) >= 12, "{slug}");
    }
}

/// The maintainer asked for a light Fire "with a really orange background".
#[test]
fn fire_light_is_a_light_preset_on_a_real_orange_background() {
    let ui: UiConfig = toml::from_str("theme = \"fire-light\"\n").unwrap();
    assert_eq!(ui.theme_slug(), "fire-light");
    let Some(spec) = preset("fire-light") else {
        return; // a custom build (`GITPOULPI_THEMES`) without it
    };
    assert!(spec.light);
    assert!(lane_palettes().iter().any(|p| p.slug == "fire-light"));
    let (r, g, b) = rgb(&spec, |t| t.ui.bg);
    assert!(
        r >= 0xe0 && (0x70..=0xd8).contains(&g) && b <= 0xb0,
        "{r:x}{g:x}{b:x}"
    );
}

/// A focused text box on Fire Light is pale yellow, not the white the other
/// light themes derive; the other light presets keep their own.
#[test]
fn fire_light_focuses_text_boxes_in_pale_yellow() {
    let Some(spec) = preset("fire-light") else {
        return;
    };
    let (r, g, b) = rgb(&spec, |t| t.ui.input_focus);
    assert!(
        r >= 0xf0 && g >= 0xe0 && (0x80..=0xc8).contains(&b),
        "{r:x}{g:x}{b:x}"
    );
    let (tr, tg, tb) = rgb(&spec, |t| t.ui.text);
    let lum = |(r, g, b): (u8, u8, u8)| {
        let f = |v: u8| {
            let v = f64::from(v) / 255.0;
            if v <= 0.03928 {
                v / 12.92
            } else {
                ((v + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * f(r) + 0.7152 * f(g) + 0.0722 * f(b)
    };
    let (a, b2) = (lum((r, g, b)), lum((tr, tg, tb)));
    assert!((a + 0.05) / (b2 + 0.05) >= 7.0, "contrast with the text");
    // Another light preset still derives the whitish focus.
    if let Some(other) = preset("github-light").or_else(|| {
        presets()
            .into_iter()
            .find(|p| p.light && p.slug != "fire-light")
    }) {
        let (r, g, b) = rgb(&other, |t| t.ui.input_focus);
        assert!(
            !(r >= 0xf0 && g >= 0xe0 && (0x80..=0xc8).contains(&b) && r > b + 40),
            "{}",
            other.slug
        );
    }
}

#[test]
fn every_preset_palette_is_in_the_lane_registry() {
    let registry = lane_palettes();
    for spec in presets() {
        assert!(
            registry.iter().any(|p| p.colors == spec.graph_palette),
            "{}: its lane palette is not listed",
            spec.slug
        );
    }
    let slugs: Vec<_> = registry.iter().map(|p| p.slug.as_str()).collect();
    for s in [
        "solarized",
        "catppuccin-mocha",
        "catppuccin-latte",
        "gruvbox",
        "nord",
    ] {
        // A custom build may leave a theme family out (`GITPOULPI_THEMES`).
        if preset_slugs().iter().any(|p| in_family(p, s)) {
            assert!(slugs.contains(&s), "{s}");
        }
    }
}

#[test]
fn the_terminal_preset_only_uses_the_terminals_colors() {
    let t = preset("terminal").unwrap();
    let toml = t.to_toml();
    for line in toml.lines() {
        // The lane palette and `blend_bg` are the only RGB values.
        if line.contains("\"#") && !line.starts_with("graph_palette") && !line.contains("blend_bg")
        {
            panic!("terminal preset has an RGB role: {line}");
        }
    }
}

#[test]
fn theme_and_lane_lists_put_gitpoulpi_first_then_sort_by_label() {
    let sorted_tail = |labels: Vec<String>| {
        let mut s = labels.clone();
        s.sort_by_key(|l| l.to_lowercase());
        assert_eq!(labels, s);
    };
    let themes = presets();
    assert_eq!(themes[0].slug, "dark");
    assert_eq!(themes[0].name, "GitPoulpi Dark");
    assert_eq!(themes[1].slug, "light");
    assert_eq!(themes[1].name, "GitPoulpi Light");
    sorted_tail(themes[2..].iter().map(|t| t.name.clone()).collect());
    let lanes = lane_palettes();
    assert_eq!(lanes[0].label, "GitPoulpi Dark");
    assert_eq!(lanes[1].label, "GitPoulpi Light");
    sorted_tail(lanes[2..].iter().map(|p| p.label.clone()).collect());
}

#[test]
fn theme_families_group_slugs_and_parse_selections() {
    assert_eq!(family_of("catppuccin-latte"), "catppuccin");
    assert_eq!(family_of("rose-pine-moon"), "rose-pine");
    assert_eq!(family_of("shades-of-purple-super-dark"), "shades-of-purple");
    assert_eq!(family_of("nord"), "nord");
    assert!(in_family("nightfox-carbonfox", "nightfox"));
    assert!(!in_family("nightfoxy", "nightfox"));
    assert!(!in_family("nightfly", "nightfox"));
    assert_eq!(parse_selection(""), None);
    assert_eq!(parse_selection("ALL"), None);
    assert_eq!(parse_selection("none"), Some(vec![]));
    assert_eq!(
        parse_selection("nord, catppuccin,nord"),
        Some(vec!["catppuccin".to_string(), "nord".to_string()])
    );
}

#[test]
fn the_compiled_set_is_the_fixed_themes_plus_the_selected_families() {
    let slugs = preset_slugs();
    for fixed in ["dark", "light", "terminal"] {
        assert!(slugs.contains(&fixed), "{fixed}");
    }
    let set = theme_set();
    if set == "all" {
        assert!(slugs.len() > 100);
        return;
    }
    let families: Vec<&str> = set
        .split(' ')
        .filter(|f| !ALWAYS_THEMES.contains(f))
        .collect();
    for slug in &slugs {
        assert!(
            ALWAYS_THEMES.contains(slug) || families.contains(&family_of(slug)),
            "{slug} is compiled but its family is not selected ({set})"
        );
    }
    for f in &families {
        assert!(slugs.iter().any(|s| in_family(s, f)), "family {f} empty");
    }
}

fn rgb(spec: &ThemeSpec, f: impl Fn(&ThemeSpec) -> ThemeColor) -> (u8, u8, u8) {
    f(spec).approx_rgb().expect("an RGB role")
}

#[test]
fn vscode_presets_have_the_blue_status_bar_with_white_text() {
    for slug in ["vscode-dark", "vscode-light"] {
        let Some(t) = preset(slug) else { continue };
        assert_eq!(
            t.ui.statusbar_bg,
            ThemeColor::Rgb(0x00, 0x7a, 0xcc),
            "{slug}"
        );
        assert_eq!(t.ui.statusbar_fg, ThemeColor::Rgb(255, 255, 255), "{slug}");
        assert_ne!(t.ui.toolbar_bg, t.ui.bg, "{slug}");
    }
}

#[test]
fn derived_presets_get_a_toolbar_a_step_off_the_background() {
    // Every preset with a real background paints a toolbar that differs from
    // it, and a readable status bar text.
    for spec in presets() {
        if spec.ui.bg == ThemeColor::Default {
            assert_eq!(spec.ui.toolbar_bg, ThemeColor::Default, "{}", spec.slug);
            continue;
        }
        let (b, t) = (rgb(&spec, |s| s.ui.bg), rgb(&spec, |s| s.ui.toolbar_bg));
        assert_ne!(b, t, "{}: toolbar must differ from bg", spec.slug);
        let r = contrast_ratio(spec.effective_statusbar_fg(), spec.effective_statusbar_bg());
        assert!(r.unwrap() >= 4.5, "{}: status bar text {r:?}", spec.slug);
    }
}

#[test]
fn the_new_roles_round_trip_through_a_theme_file() {
    let t = preset("vscode-dark").or_else(|| preset("dark")).unwrap();
    let toml = t.to_toml();
    for key in ["toolbar_bg", "statusbar_bg", "statusbar_fg"] {
        assert!(toml.contains(&format!("{key} = ")), "{key} exported");
    }
    let set = parse(&[(
        "mine",
        "base = \"dark\"\n[ui]\ntoolbar_bg = \"#102030\"\nstatusbar_bg = \"#003366\"\nstatusbar_fg = \"#eeeeee\"\n",
    )]);
    assert!(set.warnings.is_empty(), "{:?}", set.warnings);
    let u = &set.themes[0].ui;
    assert_eq!(u.toolbar_bg, ThemeColor::Rgb(0x10, 0x20, 0x30));
    assert_eq!(u.statusbar_bg, ThemeColor::Rgb(0, 0x33, 0x66));
    assert_eq!(u.statusbar_fg, ThemeColor::Rgb(0xee, 0xee, 0xee));
}

#[test]
fn unreadable_status_bar_text_is_a_contrast_issue() {
    let mut t = preset("dark").unwrap();
    t.ui.statusbar_bg = ThemeColor::Rgb(0, 0x7a, 0xcc);
    t.ui.statusbar_fg = ThemeColor::Rgb(0x30, 0x80, 0xd0);
    assert!(t
        .contrast_issues()
        .iter()
        .any(|i| i.rule == "statusbar_fg/statusbar_bg"));
}

/// A config naming a theme this build does not have (a removed one, a typo)
/// still parses; the lookup finds nothing and the TUI falls back to Dark.
#[test]
fn a_config_naming_a_missing_theme_parses_and_finds_no_preset() {
    for slug in ["poulpi-big-bro", "poulpi-big-bro-light", "nope"] {
        let ui: UiConfig = toml::from_str(&format!("theme = \"{slug}\"\n")).unwrap();
        assert_eq!(ui.theme_slug(), slug);
        assert!(preset(slug).is_none(), "{slug}");
        assert!(lane_palette(slug).is_none(), "{slug}");
    }
}
