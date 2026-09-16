//! Theme selection for custom builds. `GITPOULPI_THEMES` (unset or `all` =
//! every theme; `none`; or a comma list of families such as
//! `catppuccin,nord`) decides which presets are compiled in:
//!
//! * `OUT_DIR/theme_selection.rs`: the selected families as a constant, read
//!   by the hand-made presets (each arm is `const`-guarded, so unselected
//!   ones are dead code);
//! * `OUT_DIR/terminalcolors_selected.rs`: `src/theme/terminalcolors_data.rs`
//!   with the rows of unselected families removed.

#[path = "src/theme/family.rs"]
mod family;

use std::fmt::Write as _;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-env-changed=GITPOULPI_THEMES");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/theme/family.rs");
    println!("cargo:rerun-if-changed=src/theme/terminalcolors_data.rs");
    let value = std::env::var("GITPOULPI_THEMES").unwrap_or_default();
    let selection = family::parse_selection(&value);
    let out = PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR"));

    let mut sel = String::new();
    match &selection {
        None => sel.push_str("pub(super) const SELECTION: Option<&[&str]> = None;\n"),
        Some(list) => {
            let items: Vec<String> = list.iter().map(|f| format!("{f:?}")).collect();
            let _ = writeln!(
                sel,
                "pub(super) const SELECTION: Option<&[&str]> = Some(&[{}]);",
                items.join(", ")
            );
        }
    }
    std::fs::write(out.join("theme_selection.rs"), sel).expect("write theme_selection.rs");

    let data = std::fs::read_to_string("src/theme/terminalcolors_data.rs").expect("theme data");
    let families: Option<Vec<&str>> = selection
        .as_ref()
        .map(|l| l.iter().map(String::as_str).collect());
    let mut kept = String::new();
    for line in data.lines() {
        let slug = line
            .split_once("slug: \"")
            .and_then(|(_, r)| r.split_once('"'))
            .map(|(s, _)| s);
        match slug {
            Some(s) if !family::selected(s, families.as_deref()) => {}
            _ => {
                kept.push_str(line);
                kept.push('\n');
            }
        }
    }
    std::fs::write(out.join("terminalcolors_selected.rs"), kept).expect("write selected data");
}
