//! Key coverage of the translated `--help` catalogs
//! (`crates/gitpoulpi-commands/locales/help/*.toml`).

use clap::CommandFactory;
use gitpoulpi_commands::cli::{catalog, help_keys, Cli, HEADING_KEYS};
use gitpoulpi_i18n::Lang;

/// Writes the English source (`key = "text"`) for translators:
/// `GITPOULPI_DUMP_HELP_KEYS=/tmp/en.toml cargo test -p gitpoulpi-commands --test help_catalog dump`.
#[test]
fn dump_english_source_when_asked() {
    let Ok(path) = std::env::var("GITPOULPI_DUMP_HELP_KEYS") else {
        return;
    };
    let mut table = toml::Table::new();
    for k in help_keys(&Cli::command()) {
        table.insert(k.key, toml::Value::String(k.english));
    }
    std::fs::write(path, toml::to_string(&table).unwrap()).unwrap();
}

#[test]
fn shipped_languages_cover_every_help_key() {
    let keys = help_keys(&Cli::command());
    assert!(keys.len() > 100, "walker found {} keys", keys.len());
    for lang in gitpoulpi_i18n::SHIPPED
        .iter()
        .copied()
        .filter(|l| *l != Lang::En)
    {
        let cat = catalog(lang).unwrap_or_else(|| panic!("{} has no help catalog", lang.code()));
        let missing: Vec<&str> = keys
            .iter()
            .map(|k| k.key.as_str())
            .chain(HEADING_KEYS)
            .filter(|k| !cat.contains_key(*k))
            .collect();
        assert!(missing.is_empty(), "{}: missing {missing:?}", lang.code());
        let known: Vec<&str> = keys
            .iter()
            .map(|k| k.key.as_str())
            .chain(HEADING_KEYS)
            .collect();
        let stale: Vec<&String> = cat
            .keys()
            .filter(|k| !known.contains(&k.as_str()))
            .collect();
        // A custom build lacks the leaves of the features it left out, so
        // the catalogs legitimately hold keys for commands it has not got.
        if cfg!(feature = "full") {
            assert!(stale.is_empty(), "{}: stale keys {stale:?}", lang.code());
        }
    }
}

/// `locales/help/en.reference.toml` is the translators' source: it must match
/// the doc comments (regenerate with the dump test above, then re-append the
/// `_` heading entries).
#[test]
fn english_reference_matches_the_doc_comments() {
    let src = include_str!("../locales/help/en.reference.toml");
    let table: toml::Table = src.parse().unwrap();
    for k in help_keys(&Cli::command()) {
        assert_eq!(
            table.get(&k.key).and_then(|v| v.as_str()),
            Some(k.english.as_str()),
            "stale reference for {}",
            k.key
        );
    }
}
