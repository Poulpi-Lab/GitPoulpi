use super::*;

/// The "Every palette command" list of `docs/user/keybindings.md` and of
/// every translated guide, between `<!-- palette:begin -->` and
/// `<!-- palette:end -->`: one row per catalogue entry, with its default key,
/// the command names in the guide's language. `UPDATE_KEYMAP_DOCS=1 cargo
/// test -p gitpoulpi-tui docs` rewrites them all (the header row stays as the
/// guide has it, see `crate::docs_tables`).
fn docs_table(lang: gitpoulpi_i18n::Lang, header: Option<&str>) -> String {
    let km = crate::keymap::Keymap::defaults();
    let mut out = header
        .map(str::to_string)
        .unwrap_or_else(|| String::from("| Command | Default key |\n|---|---|\n"));
    gitpoulpi_i18n::with_lang(lang, || {
        for e in PALETTE_COMMANDS {
            let key = entry_shortcut(&km, e);
            let key = if key.is_empty() {
                String::new()
            } else {
                format!("`{}`", key.replace('|', "\\|"))
            };
            out.push_str(&format!(
                "| {} | {} |\n",
                e.display_label().replace('|', "\\|"),
                key
            ));
        }
    });
    out
}

#[test]
fn docs_palette_list_is_current() {
    crate::docs_tables::assert_current(
        "the palette lists of the keybindings guides",
        "<!-- palette:begin -->\n",
        "<!-- palette:end -->",
        &docs_table,
    );
}
