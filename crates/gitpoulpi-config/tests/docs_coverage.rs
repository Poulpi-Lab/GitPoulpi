//! Every config key is documented: each key of a default `Config`, and
//! every key some component reads (`LIVE_KEYS`), must be named in
//! `docs/user/config-reference.md`, so a new setting cannot ship
//! undocumented.

use gitpoulpi_config::{Config, LIVE_KEYS};

fn reference() -> String {
    std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../docs/user/config-reference.md"
    ))
    .unwrap()
}

/// The six `ai.actions.<action>` tables are documented once, generically.
fn doc_spelling(key: &str) -> String {
    let parts: Vec<&str> = key.split('.').collect();
    if parts.len() == 4 && parts[0] == "ai" && parts[1] == "actions" {
        return format!("ai.actions.<action>.{}", parts[3]);
    }
    // The ten `ai.sent_preview.<action>` switches are documented once too.
    if parts.len() == 3 && parts[0] == "ai" && parts[1] == "sent_preview" {
        return "ai.sent_preview.<action>".to_string();
    }
    key.to_string()
}

/// Dotted keys of a TOML table: leaves, empty tables and arrays, and
/// non-empty arrays of tables as `[[name]]` plus `[[name]] field` for each
/// field of their first entry.
fn keys(prefix: &str, table: &toml::Table, out: &mut Vec<String>) {
    for (name, value) in table {
        let key = if prefix.is_empty() {
            name.clone()
        } else {
            format!("{prefix}.{name}")
        };
        match value {
            toml::Value::Table(t) if !t.is_empty() => keys(&key, t, out),
            toml::Value::Array(items)
                if !items.is_empty() && items.iter().all(toml::Value::is_table) =>
            {
                out.push(format!("[[{key}]]"));
                if let Some(toml::Value::Table(first)) = items.first() {
                    for field in first.keys() {
                        out.push(format!("[[{key}]] {field}"));
                    }
                }
            }
            _ => out.push(key),
        }
    }
}

fn documented(doc: &str, key: &str) -> bool {
    if let Some(rest) = key.strip_prefix("[[") {
        return match rest.split_once("]] ") {
            // A field of an array of tables: shown as `field =` in its example.
            Some((_, field)) => doc.contains(&format!("\n{field} =")),
            None => doc.contains(key),
        };
    }
    let key = doc_spelling(key);
    doc.contains(&format!("`{key}`"))
        || doc.contains(&format!("`[{key}]`"))
        || doc.contains(&format!("[[{key}]]"))
}

#[test]
fn every_config_key_is_in_the_config_reference() {
    let doc = reference();
    let default = toml::Table::try_from(Config::default()).unwrap();
    let mut all = Vec::new();
    keys("", &default, &mut all);
    all.extend(LIVE_KEYS.iter().map(|k| k.to_string()));
    let mut missing: Vec<&String> = all.iter().filter(|k| !documented(&doc, k)).collect();
    missing.sort();
    missing.dedup();
    assert!(
        missing.is_empty(),
        "docs/user/config-reference.md does not document: {missing:?}"
    );
}

/// `desktop.*` keys (the `desktop.toml` of the desktop frontend) are
/// deliberately NOT required in `docs/user/config-reference.md` yet: their
/// documentation is a pending wave with native reviewers. This named allowance
/// keeps the list explicit; when the wave lands, delete it and check
/// `DESKTOP_LIVE_KEYS` and the `DesktopUiConfig` keys like the test above.
#[test]
fn desktop_keys_are_excused_until_the_documentation_wave() {
    let desktop_keys = gitpoulpi_config::desktop_ui_file::DESKTOP_LIVE_KEYS;
    assert!(!desktop_keys.is_empty());
    assert!(
        desktop_keys.iter().all(|k| k.starts_with("desktop.ui.")),
        "the allowance covers desktop.ui.* keys only"
    );
    assert!(
        LIVE_KEYS.iter().all(|k| !k.starts_with("desktop.")),
        "desktop keys have their own list, so the test above never asks for them"
    );
}
