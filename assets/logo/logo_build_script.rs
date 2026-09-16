//! The build-script body of the binary crates (included with `#[path]`, next
//! to `logo_build.rs`): the colorway table of the About window and the
//! Windows executable icon, both driven by the `GITPOULPI_LOGO_*` /
//! `GITPOULPI_ICON_COLOR` environment variables of a custom build.

#![allow(dead_code)]

use crate::logo_build as lb;
use std::path::PathBuf;

fn fail(msg: String) -> ! {
    eprintln!("error: {msg}");
    std::process::exit(1);
}

/// `write_table`: also write `OUT_DIR/colourways.json` (the crate that
/// draws the logo). The icon is embedded on Windows targets only, and a
/// missing resource compiler only warns.
///
/// `embed_on_gnu`: the GNU toolchain links the resource as a static library,
/// which propagates to every crate depending on this one (the `gitpoulpi`
/// exe links the tui and mcp libraries), so only that exe's crate embeds on
/// `*-windows-gnu`. MSVC passes the resource as a per-package link argument
/// and every binary crate embeds.
pub fn run(write_table: bool, embed_on_gnu: bool) {
    for var in [lb::COLORS_ENV, lb::DEFAULT_ENV, lb::ICON_ENV] {
        println!("cargo:rerun-if-env-changed={var}");
    }
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=../../assets/logo/colourways.json");
    let table: serde_json::Value = serde_json::from_str(include_str!("colourways.json"))
        .unwrap_or_else(|e| fail(format!("assets/logo/colourways.json: {e}")));
    let env = |k: &str| std::env::var(k).unwrap_or_default();
    let colors =
        lb::parse_colors(&env(lb::COLORS_ENV), &lb::slugs(&table)).unwrap_or_else(|e| fail(e));
    let default =
        lb::resolve_default(&table, &colors, &env(lb::DEFAULT_ENV)).unwrap_or_else(|e| fail(e));
    let icon = lb::resolve_icon(&table, &env(lb::ICON_ENV), &default).unwrap_or_else(|e| fail(e));
    let out = PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR"));

    if write_table {
        let filtered = lb::filter_table(&table, &colors, &default);
        std::fs::write(
            out.join("colourways.json"),
            serde_json::to_string(&filtered).expect("serialise"),
        )
        .expect("write colourways.json");
    }

    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let gnu = std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("gnu");
    if gnu && !embed_on_gnu {
        return;
    }
    let path = out.join("gitpoulpi.ico");
    let bytes = lb::ico(&table, &icon).unwrap_or_else(|e| fail(e));
    std::fs::write(&path, bytes).expect("write gitpoulpi.ico");
    let result = std::panic::catch_unwind(|| {
        let mut res = winresource::WindowsResource::new();
        res.set_icon(path.to_str().expect("utf-8 path"));
        res.compile()
    });
    match result {
        Ok(Ok(())) => {}
        Ok(Err(e)) => {
            println!("cargo:warning=executable icon not embedded (no resource compiler?): {e}")
        }
        Err(_) => println!("cargo:warning=executable icon not embedded (resource compiler failed)"),
    }
}
