//! Obsidian integration: vault detection, the `obsidian://` link and the
//! optional Base file. Pure file-system and string work; nothing here
//! launches anything.

use std::path::{Path, PathBuf};

use super::store::write_atomic;
use crate::{Error, Result};

/// The vault root above `folder` (the closest ancestor, `folder` included,
/// that holds an `.obsidian/` directory), if any.
pub fn vault_root(folder: &Path) -> Option<PathBuf> {
    folder
        .ancestors()
        .find(|dir| dir.join(".obsidian").is_dir())
        .map(Path::to_path_buf)
}

/// `obsidian://open?path=<absolute path>`, only when `file` lies inside a
/// vault. The path is percent-encoded as one component, which is what
/// Obsidian's URI handler expects.
pub fn open_url(file: &Path) -> Option<String> {
    let dir = file.parent()?;
    vault_root(dir)?;
    Some(format!(
        "obsidian://open?path={}",
        crate::percent_encode_component(&file.to_string_lossy().replace('\\', "/"))
    ))
}

/// The file name of the Base GitPoulpi writes.
pub const BASE_FILE: &str = "issues.base";

/// The Base: a table of the notes and a kanban grouped by `status`, in the
/// order of `statuses`. Format per Obsidian's Bases syntax
/// (<https://obsidian.md/help/bases/syntax>, checked 2026-10-14): top-level
/// `filters` / `properties` / `views`; a `table` view with `order` and
/// `sort`; a `kanban` view with `groupBy.property` and `groupOrder`.
pub fn base_text(statuses: &[String]) -> String {
    let mut out = String::new();
    out.push_str(
        "filters:\n  and:\n    - file.folder == this.file.folder\n    - file.ext == \"md\"\n",
    );
    out.push_str("properties:\n  note.id:\n    displayName: \"#\"\n  note.title:\n    displayName: Title\n  note.status:\n    displayName: Status\n");
    out.push_str("views:\n  - type: table\n    name: Issues\n    order:\n      - note.id\n      - note.title\n      - note.status\n      - note.assignees\n      - note.labels\n      - note.milestone\n      - note.created\n    sort:\n      - property: note.id\n        direction: DESC\n");
    out.push_str(
        "  - type: kanban\n    name: Board\n    groupBy:\n      property: note.status\n      direction: ASC\n",
    );
    if !statuses.is_empty() {
        out.push_str("    groupOrder:\n");
        for s in statuses {
            let quoted = format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""));
            out.push_str(&format!("      - {quoted}\n"));
        }
    }
    out
}

/// Writes `issues.base` next to the notes, once. An existing file (ours or
/// the user's) is never touched: the error says so.
pub fn create_base(folder: &Path, statuses: &[String]) -> Result<PathBuf> {
    let path = folder.join(BASE_FILE);
    if path.exists() {
        return Err(Error::Local(format!(
            "`{}` already exists; GitPoulpi never edits a Base",
            path.display()
        )));
    }
    if !folder.is_dir() {
        return Err(Error::Local(format!(
            "`{}` is not a folder",
            folder.display()
        )));
    }
    write_atomic(&path, &base_text(statuses))?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_vault_is_found_up_the_tree() {
        let dir = tempfile::tempdir().unwrap();
        let notes = dir.path().join("Projects").join("Issues");
        std::fs::create_dir_all(&notes).unwrap();
        assert!(vault_root(&notes).is_none());
        assert!(open_url(&notes.join("1-a.md")).is_none());
        std::fs::create_dir(dir.path().join(".obsidian")).unwrap();
        assert_eq!(vault_root(&notes).as_deref(), Some(dir.path()));
        let url = open_url(&notes.join("1-a b.md")).unwrap();
        // An absolute path: `/tmp/...` on unix, `C:/Users/...` on Windows.
        let encoded_root = if cfg!(windows) { "%3A%2F" } else { "%2F" };
        assert!(url.starts_with("obsidian://open?path="), "{url}");
        assert!(url.contains(encoded_root), "{url}");
        assert!(!url.contains("%5C"), "no backslash survives: {url}");
        assert!(url.ends_with("Issues%2F1-a%20b.md"), "{url}");
    }

    #[test]
    fn the_base_is_written_once_and_never_overwritten() {
        let dir = tempfile::tempdir().unwrap();
        let statuses = ["todo".to_string(), "doing".to_string(), "done".to_string()];
        let path = create_base(dir.path(), &statuses).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(text.contains("type: table") && text.contains("type: kanban"));
        assert!(text.contains("property: note.status"));
        assert!(text.contains("      - \"doing\"\n"));
        let err = create_base(dir.path(), &statuses).unwrap_err().to_string();
        assert!(err.contains("already exists"), "{err}");
        std::fs::write(&path, "mine").unwrap();
        assert!(create_base(dir.path(), &statuses).is_err());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "mine");
    }
}
