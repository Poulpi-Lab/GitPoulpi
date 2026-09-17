//! Generates `$OUT_DIR/help_catalogs.rs`: the `catalog_source` match of
//! `src/cli/help_lang.rs`, with an `include_str!` only for the languages built
//! into this binary. `gitpoulpi-i18n`'s build script publishes the built
//! languages as `DEP_GITPOULPI_I18N_LANGS` (a comma list of codes), so this
//! crate needs no `lang-*` features of its own.

use std::path::PathBuf;
use std::{env, fs};

/// (code, `Lang` variant). English has no catalog (the clap doc comments are
/// the English source) and `qps` is not a help language.
const HELP_LANGS: &[(&str, &str)] = &[
    ("fr", "Fr"),
    ("es", "Es"),
    ("pt-BR", "PtBr"),
    ("pt-PT", "PtPt"),
    ("it", "It"),
    ("de", "De"),
    ("zh-CN", "ZhCn"),
    ("ja", "Ja"),
    ("ko", "Ko"),
    ("ru", "Ru"),
    ("uk", "Uk"),
    ("pl", "Pl"),
    ("cs", "Cs"),
    ("nl", "Nl"),
    ("sv", "Sv"),
    ("tr", "Tr"),
];

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=locales/help");
    let built = env::var("DEP_GITPOULPI_I18N_LANGS").unwrap_or_default();
    let built: Vec<&str> = built.split(',').collect();
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let any = HELP_LANGS.iter().any(|(code, _)| built.contains(code));
    // No language built (English only): a `match` with only a wildcard arm
    // trips clippy's `match_single_binding`.
    let mut o = if any {
        String::from("match lang {\n")
    } else {
        String::from("{\n    let _ = lang;\n    return None;\n}\n")
    };
    for (code, variant) in HELP_LANGS {
        if built.contains(code) {
            let path = manifest.join("locales/help").join(format!("{code}.toml"));
            o.push_str(&format!(
                "    Lang::{variant} => include_str!({:?}),\n",
                path.to_str().expect("utf-8 path")
            ));
        }
    }
    // English, the pseudo-locale, and every language not built in.
    if any {
        o.push_str("    _ => return None,\n}\n");
    }
    let out = PathBuf::from(env::var("OUT_DIR").unwrap()).join("help_catalogs.rs");
    fs::write(out, o).unwrap();
}
