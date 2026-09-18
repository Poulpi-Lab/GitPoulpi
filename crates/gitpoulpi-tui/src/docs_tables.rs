//! Test support for the tables generated into the keybindings guide, in
//! every language that has one: `docs/user/keybindings.md` (English) and
//! `docs/user/<code>/keybindings.md`. A table sits between two HTML-comment
//! markers; the generator writes the rows in the guide's own language (the
//! catalog's labels), but keeps the table's header row as the translators
//! wrote it, so a heading is edited in the guide, not in a catalog.
//!
//! `UPDATE_KEYMAP_DOCS=1 cargo test -p gitpoulpi-tui docs` rewrites every
//! stale table of every language; without it a stale table fails its test.

use gitpoulpi_i18n::{Lang, SHIPPED};
use std::path::{Path, PathBuf};

/// Every shipped language with its keybindings guide (English first).
pub(crate) fn guides() -> Vec<(Lang, PathBuf)> {
    let docs = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/user");
    SHIPPED
        .iter()
        .copied()
        .filter(|l| *l != Lang::Qps)
        .map(|l| {
            let path = if l == Lang::En {
                docs.join("keybindings.md")
            } else {
                docs.join(l.code()).join("keybindings.md")
            };
            (l, path)
        })
        .collect()
}

/// The two header lines (titles and the `|---|` rule) of the table the guide
/// holds between the markers, or `None` when it has none yet (English then
/// uses its own).
fn header_of(block: &str) -> Option<String> {
    let mut lines = block.lines();
    let (a, b) = (lines.next()?, lines.next()?);
    (a.starts_with('|') && b.starts_with("|---")).then(|| format!("{a}\n{b}\n"))
}

/// Brings the table of `path` between `begin` and `end` up to date.
/// `make(header)` renders it, given the guide's current header lines.
/// `Ok(true)` when it was stale and has been rewritten (`UPDATE_KEYMAP_DOCS`),
/// `Err` when it is stale and nothing may be written.
pub(crate) fn sync(
    path: &Path,
    begin: &str,
    end: &str,
    make: &dyn Fn(Option<&str>) -> String,
) -> Result<bool, String> {
    let shown = path.display();
    let doc = std::fs::read_to_string(path).map_err(|e| format!("{shown}: {e}"))?;
    let from = doc
        .find(begin)
        .map(|i| i + begin.len())
        .ok_or_else(|| format!("{shown}: marker {begin:?} is missing"))?;
    let to = doc[from..]
        .find(end)
        .map(|i| i + from)
        .ok_or_else(|| format!("{shown}: marker {end:?} is missing"))?;
    let table = make(header_of(&doc[from..to]).as_deref());
    if doc[from..to] == table {
        return Ok(false);
    }
    if std::env::var_os("UPDATE_KEYMAP_DOCS").is_some() {
        let new = format!("{}{}{}", &doc[..from], table, &doc[to..]);
        std::fs::write(path, new).map_err(|e| format!("{shown}: {e}"))?;
        return Ok(true);
    }
    Err(format!(
        "{shown}: the table between {begin:?} and {end:?} is stale"
    ))
}

/// Runs [`sync`] on the guide of every language and fails listing the stale
/// ones (all of them at once, so one `UPDATE_KEYMAP_DOCS=1` run fixes all).
pub(crate) fn assert_current(
    what: &str,
    begin: &str,
    end: &str,
    make: &dyn Fn(Lang, Option<&str>) -> String,
) {
    let guides = guides();
    assert!(
        guides.iter().any(|(l, _)| *l == Lang::En),
        "the English guide is part of the scan"
    );
    let mut errors = Vec::new();
    for (lang, path) in guides {
        if let Err(e) = sync(&path, begin, end, &|header| make(lang, header)) {
            errors.push(e);
        }
    }
    assert!(
        errors.is_empty(),
        "{what} stale: run UPDATE_KEYMAP_DOCS=1 cargo test -p gitpoulpi-tui docs\n{}",
        errors.join("\n")
    );
}

#[test]
fn every_shipped_language_has_a_keybindings_guide() {
    for (lang, path) in guides() {
        assert!(
            path.exists(),
            "{}: {} is missing",
            lang.code(),
            path.display()
        );
    }
}

#[test]
fn the_header_row_is_read_from_the_guide() {
    let block = "| Contexte | Id |\n|---|---|\n| `global` | `x` |\n";
    assert_eq!(
        header_of(block).as_deref(),
        Some("| Contexte | Id |\n|---|---|\n")
    );
    assert_eq!(header_of(""), None);
    assert_eq!(header_of("not a table\nat all\n"), None);
}

/// The freshness check is language-agnostic: a table that differs is reported
/// whatever the language of the guide (and the header the guide wrote stays).
#[test]
fn a_stale_table_is_reported_and_the_guides_header_is_kept() {
    if std::env::var_os("UPDATE_KEYMAP_DOCS").is_some() {
        return; // the switch would rewrite the temp file instead of failing
    }
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("keybindings.md");
    let doc =
        "intro\n<!-- t:begin -->\n| Contexte | Id |\n|---|---|\n| old |  |\n<!-- t:end -->\noutro\n";
    std::fs::write(&path, doc).unwrap();
    let make =
        |header: Option<&str>| format!("{}| new |  |\n", header.unwrap_or("| English |\n|---|\n"));
    let (begin, end) = ("<!-- t:begin -->\n", "<!-- t:end -->");
    let err = sync(&path, begin, end, &make).unwrap_err();
    assert!(err.contains("stale"), "{err}");
    assert_eq!(std::fs::read_to_string(&path).unwrap(), doc, "not written");
    // A current table passes, with the guide's own header.
    let fresh = doc.replace("| old |  |", "| new |  |");
    std::fs::write(&path, &fresh).unwrap();
    assert_eq!(sync(&path, begin, end, &make), Ok(false));
    // A missing marker is an error, not a silent pass.
    std::fs::write(&path, "no markers here\n").unwrap();
    assert!(sync(&path, begin, end, &make).is_err());
}
