//! Guards the custom-build feature contract
//! (the custom-builds design): every `locales/*.toml` has
//! a `lang-*` feature, and every binary crate forwards each `lang-*` and each feature in `full`.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf()
}

fn features(krate: &str) -> toml::Table {
    let path = root().join("crates").join(krate).join("Cargo.toml");
    let doc: toml::Table = fs::read_to_string(&path)
        .unwrap()
        .parse()
        .unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    doc.get("features")
        .and_then(|f| f.as_table())
        .unwrap_or_else(|| panic!("{krate} has no [features]"))
        .clone()
}

fn list(t: &toml::Table, key: &str) -> Vec<String> {
    t.get(key)
        .unwrap_or_else(|| panic!("feature `{key}` missing"))
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect()
}

/// `lang-<stem lowercased>` for every locale file, plus `lang-qps`.
fn lang_features() -> BTreeSet<String> {
    let dir = root().join("crates/gitpoulpi-i18n/locales");
    let mut out: BTreeSet<String> = fs::read_dir(dir)
        .unwrap()
        .filter_map(|e| {
            let p = e.unwrap().path();
            (p.extension()? == "toml").then(|| p.file_stem().unwrap().to_str().unwrap().to_string())
        })
        .filter(|stem| stem != "en")
        .map(|stem| format!("lang-{}", stem.to_lowercase()))
        .collect();
    out.insert("lang-qps".into());
    out
}

const BINARIES: [&str; 3] = ["gitpoulpi-cli", "gitpoulpi-tui", "gitpoulpi-mcp"];

#[test]
fn every_locale_has_a_feature_in_i18n() {
    let f = features("gitpoulpi-i18n");
    let all: BTreeSet<String> = list(&f, "all-langs").into_iter().collect();
    assert_eq!(all, lang_features(), "`all-langs` must list every locale");
    for l in lang_features() {
        assert!(f.contains_key(&l), "gitpoulpi-i18n lacks feature `{l}`");
    }
}

#[test]
fn every_binary_forwards_every_lang_to_i18n() {
    for bin in BINARIES {
        let f = features(bin);
        for l in lang_features() {
            let fwd = list(&f, &l);
            assert!(
                fwd.contains(&format!("gitpoulpi-i18n/{l}")),
                "{bin}: `{l}` must forward to gitpoulpi-i18n/{l}, has {fwd:?}"
            );
        }
        let all: BTreeSet<String> = list(&f, "all-langs").into_iter().collect();
        assert_eq!(all, lang_features(), "{bin}: `all-langs`");
    }
}

#[test]
fn every_binary_has_the_same_full_and_declares_each_member() {
    let expected = list(&features("gitpoulpi-cli"), "full");
    assert!(expected.contains(&"all-langs".to_string()));
    for bin in BINARIES {
        let f = features(bin);
        assert_eq!(list(&f, "default"), ["full"], "{bin}: default");
        let full = list(&f, "full");
        assert_eq!(
            full.iter().collect::<BTreeSet<_>>(),
            expected.iter().collect::<BTreeSet<_>>(),
            "{bin}: `full`"
        );
        for member in &full {
            assert!(f.contains_key(member), "{bin} lacks feature `{member}`");
        }
    }
}

#[test]
fn the_cli_forwards_each_feature_to_the_other_binaries() {
    let cli = features("gitpoulpi-cli");
    for member in list(&cli, "full") {
        if member == "all-langs" || member == "avatar-fetch" {
            continue;
        }
        let fwd = list(&cli, &member);
        for other in ["gitpoulpi-tui", "gitpoulpi-mcp"] {
            assert!(
                fwd.contains(&format!("{other}/{member}"))
                    || fwd.contains(&format!("{other}?/{member}")),
                "gitpoulpi-cli: `{member}` must forward to {other}/{member}"
            );
        }
    }
}
