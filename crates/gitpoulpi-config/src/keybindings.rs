//! The `[keybindings]` table: which key triggers which action.
//!
//! This crate stores **strings only**. It knows nothing of actions,
//! contexts or key names, so a typo (or a table written by a newer
//! version) can never make `config.toml` unloadable for the CLI, the MCP
//! server or the daemon: anything that is not a context table of
//! `action = "chord"` / `action = ["chord", ...]` entries is dropped on
//! load. The TUI (`keymap::Keymap::from_config`) is the only thing that
//! interprets the strings and reports the bad ones.
//!
//! ```toml
//! [keybindings.global]
//! git_prefix = "Ctrl+B"          # one chord
//! settings   = ["Ctrl+,", "F2"]  # alternatives
//! quit       = []                # unbound
//!
//! [keybindings]
//! git_prefix.stage_file = "a"    # the same thing as dotted keys
//! ```

use crate::{Error, Result};
use std::collections::BTreeMap;
use std::path::Path;

/// Context name -> action name -> chords (text, as typed). An absent
/// action means "the default"; an empty list means "unbound".
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize)]
#[serde(transparent)]
pub struct KeybindingsConfig {
    entries: BTreeMap<String, BTreeMap<String, Vec<String>>>,
}

impl KeybindingsConfig {
    /// Every `(context, action, chords)` entry, in a stable order.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &str, &[String])> {
        self.entries.iter().flat_map(|(ctx, actions)| {
            actions
                .iter()
                .map(move |(action, chords)| (ctx.as_str(), action.as_str(), chords.as_slice()))
        })
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn get(&self, ctx: &str, action: &str) -> Option<&[String]> {
        self.entries
            .get(ctx)
            .and_then(|m| m.get(action))
            .map(Vec::as_slice)
    }

    /// Builds a table from `(context, action, chords)` triples.
    pub fn from_entries<'a>(
        entries: impl IntoIterator<Item = (&'a str, &'a str, Vec<String>)>,
    ) -> Self {
        let mut out = KeybindingsConfig::default();
        for (ctx, action, chords) in entries {
            out.entries
                .entry(ctx.to_string())
                .or_default()
                .insert(action.to_string(), chords);
        }
        out
    }

    fn from_value(value: serde_json::Value) -> Self {
        let mut out = KeybindingsConfig::default();
        let serde_json::Value::Object(contexts) = value else {
            return out;
        };
        for (ctx, actions) in contexts {
            let serde_json::Value::Object(actions) = actions else {
                continue;
            };
            for (action, spec) in actions {
                let chords = match spec {
                    serde_json::Value::String(s) if s.trim().is_empty() => Vec::new(),
                    serde_json::Value::String(s) => vec![s],
                    serde_json::Value::Array(items) => {
                        let strings: Vec<String> = items
                            .into_iter()
                            .filter_map(|v| match v {
                                serde_json::Value::String(s) if !s.trim().is_empty() => Some(s),
                                _ => None,
                            })
                            .collect();
                        strings
                    }
                    _ => continue,
                };
                out.entries
                    .entry(ctx.clone())
                    .or_default()
                    .insert(action, chords);
            }
        }
        out
    }
}

impl<'de> serde::Deserialize<'de> for KeybindingsConfig {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(d)?;
        Ok(KeybindingsConfig::from_value(value))
    }
}

/// The `[keybindings]` table as the loader sees it for `path` (the sibling
/// `tui.toml` when there is one), `None` when nothing is bound.
fn has_bindings(path: &Path) -> Result<bool> {
    Ok(crate::split::read_merged(path)?
        .and_then(|(doc, _)| doc.get("keybindings").cloned())
        .is_some_and(|kb| kb.as_table().is_some_and(|t| !t.is_empty())))
}

fn bad_bindings(key: &str) -> impl FnOnce(toml::de::Error) -> Error + '_ {
    move |err| Error::BadValue {
        key: key.to_string(),
        value: "<keybindings>".to_string(),
        expected: err.to_string(),
    }
}

fn check_name(key: &str, part: &str) -> Result<()> {
    if part.is_empty() || part.contains(['.', ' ', '[', ']', '"']) {
        return Err(Error::UnknownKey(key.to_string()));
    }
    Ok(())
}

/// Sets `[keybindings.<ctx>] <action>` to `chords` in the document at
/// `path`, leaving every other key as the file had it. One chord is
/// written as a string, several (or none: "unbound") as an array.
pub fn set_keybinding_in_file(
    path: &Path,
    ctx: &str,
    action: &str,
    chords: &[String],
) -> Result<()> {
    let key = format!("keybindings.{ctx}.{action}");
    check_name(&key, ctx)?;
    check_name(&key, action)?;
    let value = match chords {
        [one] => toml::Value::String(one.clone()),
        many => toml::Value::Array(many.iter().cloned().map(toml::Value::String).collect()),
    };
    // The bindings are interface settings: `tui.toml` (see `crate::split`).
    crate::split::edit_config_document(
        path,
        "keybindings",
        |document| {
            let root = document.as_table_mut().ok_or(Error::UnsettableKey {
                key: key.clone(),
                kind: "value",
            })?;
            let kb = root
                .entry("keybindings")
                .or_insert_with(|| toml::Value::Table(toml::map::Map::new()));
            let kb = kb.as_table_mut().ok_or(Error::UnsettableKey {
                key: "keybindings".into(),
                kind: "value",
            })?;
            let table = kb
                .entry(ctx)
                .or_insert_with(|| toml::Value::Table(toml::map::Map::new()));
            let table = table.as_table_mut().ok_or(Error::UnsettableKey {
                key: format!("keybindings.{ctx}"),
                kind: "value",
            })?;
            table.insert(action.to_string(), value);
            Ok(())
        },
        bad_bindings("keybindings"),
    )
    .map(|_| ())
}

/// Removes one binding (back to its default). An emptied context table,
/// and then an emptied `[keybindings]`, is dropped too.
pub fn reset_keybinding_in_file(path: &Path, ctx: &str, action: &str) -> Result<()> {
    if !has_bindings(path)? {
        return Ok(());
    }
    crate::split::edit_config_document(
        path,
        "keybindings",
        |document| {
            let Some(root) = document.as_table_mut() else {
                return Ok(());
            };
            if let Some(kb) = root
                .get_mut("keybindings")
                .and_then(toml::Value::as_table_mut)
            {
                if let Some(table) = kb.get_mut(ctx).and_then(toml::Value::as_table_mut) {
                    table.remove(action);
                    if table.is_empty() {
                        kb.remove(ctx);
                    }
                }
                if kb.is_empty() {
                    root.remove("keybindings");
                }
            }
            Ok(())
        },
        bad_bindings("keybindings"),
    )
    .map(|_| ())
}

/// Removes the whole `[keybindings]` table.
pub fn reset_all_keybindings_in_file(path: &Path) -> Result<()> {
    if !has_bindings(path)? {
        return Ok(());
    }
    crate::split::edit_config_document(
        path,
        "keybindings",
        |document| {
            if let Some(root) = document.as_table_mut() {
                root.remove("keybindings");
            }
            Ok(())
        },
        bad_bindings("keybindings"),
    )
    .map(|_| ())
}

/// What `gitpoulpi config set keybindings.<ctx>.<action> <value>` stores:
/// `value` is one chord or a whitespace-separated list (a chord never
/// contains a space; the space key is `Space`); empty means unbound.
pub(crate) fn chords_from_cli(value: &str) -> Vec<String> {
    value.split_whitespace().map(str::to_string).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Config;

    fn parse(text: &str) -> Config {
        toml::from_str(text).unwrap()
    }

    #[test]
    fn dotted_and_table_spellings_parse_equal() {
        let a = parse("[keybindings.git_prefix]\nstage_file = \"a\"\n[keybindings.global]\nsettings = [\"Ctrl+,\", \"F2\"]\n");
        let b = parse("[keybindings]\ngit_prefix.stage_file = \"a\"\nglobal.settings = [\"Ctrl+,\", \"F2\"]\n");
        assert_eq!(a.keybindings, b.keybindings);
        assert_eq!(
            a.keybindings.get("global", "settings").unwrap(),
            ["Ctrl+,", "F2"]
        );
        assert_eq!(
            a.keybindings.get("git_prefix", "stage_file").unwrap(),
            ["a"]
        );
    }

    #[test]
    fn empty_list_and_empty_string_mean_unbound() {
        let c = parse("[keybindings.global]\nquit = []\nhelp = \"\"\n");
        assert_eq!(c.keybindings.get("global", "quit").unwrap().len(), 0);
        assert_eq!(c.keybindings.get("global", "help").unwrap().len(), 0);
    }

    #[test]
    fn garbage_never_stops_the_config_loading() {
        let c = parse(
            "[keybindings]\nglobal = 5\n[keybindings.log]\nfind = 7\nok = \"x\"\nlist = [1, \"y\"]\n",
        );
        assert_eq!(c.keybindings.get("log", "ok").unwrap(), ["x"]);
        assert_eq!(c.keybindings.get("log", "list").unwrap(), ["y"]);
        assert!(c.keybindings.get("log", "find").is_none());
        let c = parse("keybindings = \"nope\"\n");
        assert!(c.keybindings.is_empty());
        // Unknown contexts and actions survive: only the TUI judges them.
        let c = parse("[keybindings.nowhere]\nthing = \"Ctrl+Nope\"\n");
        assert_eq!(
            c.keybindings.get("nowhere", "thing").unwrap(),
            ["Ctrl+Nope"]
        );
    }

    #[test]
    fn config_round_trips_and_get_walks_the_table() {
        let c = parse("[keybindings.global]\ngit_prefix = \"Ctrl+B\"\n");
        let v = c.get("keybindings.global.git_prefix").unwrap();
        assert_eq!(v, serde_json::json!(["Ctrl+B"]));
        assert!(Config::default().keybindings.is_empty());
    }

    #[test]
    fn writer_leaves_other_keys_and_reset_cleans_up() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        std::fs::write(&path, "[ui]\nminimap = false\n").unwrap();
        set_keybinding_in_file(&path, "global", "git_prefix", &["Ctrl+B".into()]).unwrap();
        set_keybinding_in_file(&path, "git_prefix", "stage_file", &["a".into(), "z".into()])
            .unwrap();
        set_keybinding_in_file(&path, "global", "quit", &[]).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(text.contains("minimap = false"));
        let cfg = Config::load(&path).unwrap();
        assert!(!cfg.ui.minimap);
        assert_eq!(
            cfg.keybindings.get("global", "git_prefix").unwrap(),
            ["Ctrl+B"]
        );
        assert_eq!(
            cfg.keybindings.get("git_prefix", "stage_file").unwrap(),
            ["a", "z"]
        );
        assert_eq!(cfg.keybindings.get("global", "quit").unwrap().len(), 0);

        reset_keybinding_in_file(&path, "git_prefix", "stage_file").unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(
            !text.contains("git_prefix]"),
            "emptied table dropped: {text}"
        );
        reset_keybinding_in_file(&path, "global", "git_prefix").unwrap();
        reset_keybinding_in_file(&path, "global", "quit").unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(!text.contains("keybindings"), "{text}");
        assert!(text.contains("minimap = false"));

        set_keybinding_in_file(&path, "global", "help", &["F1".into()]).unwrap();
        reset_all_keybindings_in_file(&path).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(!text.contains("keybindings") && text.contains("minimap"));
    }

    #[test]
    fn config_set_routes_keybindings_to_the_writer() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        crate::set_in_file(&path, "keybindings.global.git_prefix", "Ctrl+B").unwrap();
        crate::set_in_file(&path, "keybindings.global.settings", "Ctrl+, F2").unwrap();
        crate::set_in_file(&path, "keybindings.global.quit", "").unwrap();
        let cfg = Config::load(&path).unwrap();
        assert_eq!(
            cfg.keybindings.get("global", "git_prefix").unwrap(),
            ["Ctrl+B"]
        );
        assert_eq!(cfg.keybindings.get("global", "quit").unwrap().len(), 0);
        assert_eq!(
            cfg.keybindings.get("global", "settings").unwrap(),
            ["Ctrl+,", "F2"]
        );
        assert!(crate::set_in_file(&path, "keybindings.global", "x").is_err());
        assert!(crate::Settings::key_is_implemented(
            "keybindings.global.git_prefix"
        ));
    }
}
