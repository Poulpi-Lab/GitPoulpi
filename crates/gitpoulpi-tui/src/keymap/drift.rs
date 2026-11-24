//! The drift guard: UI text must show a registry action's *effective*
//! binding, never a key label typed into a string literal. This scans the
//! production source of the crate (outside `keymap/` and `#[cfg(test)]`
//! modules) for literals that spell a key and fails on any that is not an
//! explicitly allowed fixed key. Fix a hit with `keymap::current().shown(..)`
//! / `keymap::paren(..)`; allow it only for a key that is deliberately not
//! in the registry (text-field editing, mouse chords, Tab, ...).

use crate::source_scan::{literals, production_lines, production_source_files};
use std::path::Path;

/// Key spellings of keys that are fixed on purpose (not rebindable, not in
/// the registry): they may appear in literals. Each is stripped from a
/// literal before it is judged.
const FIXED_KEYS: &[&str] = &[
    // Text-field and editor caret chords (fixed by design).
    "Ctrl+U",
    "Ctrl+K",
    "Ctrl+A",
    "Ctrl+X",
    "Ctrl+C",
    "Ctrl+Left/Right",
    "Ctrl+Home/End",
    "Ctrl+Left",
    "Ctrl+Right",
    // Mouse chords and focus cycling.
    "Ctrl+click",
    "Shift+click",
    "Shift+wheel",
    "Shift+Tab",
    "Shift+Up/Down",
    "Shift+Enter",
    "Shift+(any move)",
    "Shift+Up",
    "Shift+Down",
    // Fixed folder-picker moves.
    "Alt+Up",
    "Alt+Left",
    "Alt+Right",
    // Compose review's fixed reorder hint is the registry's, see rebase.
    "Ctrl+J",
    // The Ctrl+Enter fallbacks the help explains for terminals that
    // cannot send it.
    "Alt+Enter",
    // The in-form hints that name the Ctrl+S fallback of a Ctrl+Enter
    // action are generated; the AI editor's own footer is fixed text.
    "Ctrl+S: save",
    // Exact fixed lines (the key is not a registry action there).
    "or Ctrl+F  search every section",
    "Alt+\\u{2191}\\u{2193} (or K/J)",
    "Bksp/Alt+\\u{2191}",
    "Alt+\\u{2190}/\\u{2192}",
    "Ctrl+Enter as a plain Enter",
    "Type the key (Ctrl+Shift+N, Alt+F2, ...)",
];

/// Whether `s` still spells a key: `Ctrl+x`, `Alt+x`, `Shift+x`, or a
/// bracketed / parenthesised single-key hint (` [m]`, ` (r)`, ` (/)`).
fn spells_a_key(s: &str) -> Option<String> {
    let b: Vec<char> = s.chars().collect();
    for i in 0..b.len() {
        let rest: String = b[i..].iter().take(8).collect();
        for p in ["Ctrl+", "Alt+", "Shift+"] {
            if rest.starts_with(p)
                && rest
                    .chars()
                    .nth(p.len())
                    .is_some_and(|c| !c.is_whitespace())
            {
                return Some(rest);
            }
        }
        if i > 0 && b[i - 1] == ' ' && i + 2 < b.len() {
            let (open, key, close) = (b[i], b[i + 1], b[i + 2]);
            let pair = matches!((open, close), ('[', ']') | ('(', ')'));
            if pair && (key.is_ascii_alphabetic() || "/*?".contains(key)) && key != 'x' {
                return Some(format!("{open}{key}{close}"));
            }
        }
    }
    None
}

#[test]
fn no_hard_coded_key_labels() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    // `keymap/` itself holds the registry the labels come from.
    let files = production_source_files(true);
    assert!(files.len() > 50, "the scan found {} files", files.len());
    let mut hits = Vec::new();
    for file in files {
        let name = file
            .strip_prefix(&root)
            .unwrap()
            .to_string_lossy()
            .to_string();
        let unix = name.replace('\\', "/");
        let text = std::fs::read_to_string(&file).unwrap();
        let lines = production_lines(&text);
        for (n, lit) in literals(&lines) {
            let mut rest = lit.clone();
            for fixed in FIXED_KEYS {
                rest = rest.replace(fixed, "");
            }
            // The palette catalogue's literals are lookup keys for the
            // effective binding; `palette_shortcuts_follow_the_keymap`
            // proves each one resolves to an action.
            if unix == "palette/catalog.rs" && lit.len() <= 20 {
                continue;
            }
            if let Some(found) = spells_a_key(&rest) {
                hits.push(format!("{name}:{n}: {found:?} in {lit:?}"));
            }
        }
    }
    assert!(
        hits.is_empty(),
        "hard-coded key labels (use keymap::current().shown / keymap::paren, or add a justified FIXED_KEYS entry):\n{}",
        hits.join("\n")
    );
}

#[test]
fn the_scanner_flags_what_it_should() {
    assert!(spells_a_key("Stage (Ctrl+G s)").is_some());
    assert!(spells_a_key("Tree [m]").is_some());
    assert!(spells_a_key("refresh (r)").is_some());
    assert!(spells_a_key("Search repositories (/)").is_some());
    assert!(spells_a_key("a [x] checkbox and 2 line(s)").is_none());
    assert!(spells_a_key("Shift+").is_none());
}

/// Every palette shortcut that spells a key resolves to a registry action
/// (so it is replaced by the effective binding), or is a fixed key.
#[test]
fn palette_shortcuts_follow_the_keymap() {
    let defaults = crate::palette::default_shortcut_actions();
    let fixed = ["Ctrl+F", "Ctrl+C", "Shift+Tab", "+"];
    for e in crate::palette::PALETTE_COMMANDS
        .iter()
        .filter(|e| crate::build_features::offered(e.requires))
    {
        if crate::palette::command_action(e).is_some() {
            continue;
        }
        for alt in e.shortcut.split(" / ").map(str::trim) {
            let alt = alt
                .strip_suffix(crate::palette::BADGE_SUFFIX)
                .unwrap_or(alt);
            if spells_a_key(alt).is_none() && !alt.contains(['+']) {
                continue;
            }
            assert!(
                defaults.iter().any(|(l, _)| l == alt) || fixed.contains(&alt),
                "{:?}: shortcut {alt:?} is not tied to a registry action",
                e.label
            );
        }
    }
}

/// The generated "Default keybindings" table of `docs/user/keybindings.md`
/// and of every translated guide, between `<!-- keymap:begin -->` and
/// `<!-- keymap:end -->`: one row per registry action, the "What it does"
/// column in the guide's language. `UPDATE_KEYMAP_DOCS=1 cargo test -p
/// gitpoulpi-tui docs` rewrites them all; the header row stays as the guide
/// has it (`crate::docs_tables`).
fn docs_table(lang: gitpoulpi_i18n::Lang, header: Option<&str>) -> String {
    let km = crate::keymap::Keymap::defaults();
    let english = lang == gitpoulpi_i18n::Lang::En;
    let mut out = header.map(str::to_string).unwrap_or_else(|| {
        String::from("| Context | Action id | What it does | Default keys |\n|---|---|---|---|\n")
    });
    // The palette commands (`palette.<id>`, all unbound) are listed in
    // Settings, not here: one row each would bury the table.
    gitpoulpi_i18n::with_lang(lang, || {
        for def in crate::keymap::ACTIONS.iter().filter(|d| {
            !matches!(
                d.ctx,
                crate::keymap::Ctx::Palette | crate::keymap::Ctx::Sequence
            )
        }) {
            let keys = km
                .labels(def.action)
                .iter()
                .map(|l| format!("`{l}`"))
                .collect::<Vec<_>>()
                .join(", ");
            out.push_str(&format!(
                "| `{}` | `{}` | {} | {} |\n",
                def.ctx.name(),
                def.config_id()
                    .split_once('.')
                    .map_or(def.config_id(), |(_, a)| a),
                def.display_label(),
                match (keys.is_empty(), english) {
                    (false, _) => keys,
                    (true, true) => "(none)".to_string(),
                    (true, false) => "-".to_string(),
                }
            ));
        }
    });
    out
}

#[test]
fn docs_table_is_current() {
    crate::docs_tables::assert_current(
        "the keymap tables of the keybindings guides",
        "<!-- keymap:begin -->\n",
        "<!-- keymap:end -->",
        &docs_table,
    );
}

/// An action's id and label are read through `ActionDef::config_id` and
/// `display_label`/`english_label` everywhere outside `keymap/`: a sequence
/// slot's are the user's, which the raw fields do not know.
#[test]
fn nothing_outside_the_keymap_reads_the_raw_id_or_label() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut hits = Vec::new();
    for file in production_source_files(true) {
        let name = file
            .strip_prefix(&root)
            .unwrap()
            .to_string_lossy()
            .to_string();
        let text = std::fs::read_to_string(&file).unwrap();
        for (n, line) in production_lines(&text) {
            if line.contains(".static_id") || line.contains(".static_label") {
                hits.push(format!("{name}:{n}: {}", line.trim()));
            }
        }
    }
    assert!(
        hits.is_empty(),
        "read `config_id()` / `display_label()` / `english_label()` instead:\n{}",
        hits.join("\n")
    );
}
