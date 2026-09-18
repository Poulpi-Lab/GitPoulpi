//! "Create File": validating the typed name and creating the empty file.
//!
//! The name prompt (`RefNameOp::CreateFile`) hands the typed text and the
//! folder it is relative to (empty = the repository root) here. Intermediate
//! folders are created; an existing file or folder of that name is refused.

use gitpoulpi_i18n::msg::new_file as msg;
use std::path::{Path, PathBuf};

/// Characters no file name may contain (the Windows set, which is the
/// strictest: a name valid there is valid everywhere).
const INVALID: &[char] = &['<', '>', ':', '"', '|', '?', '*'];

/// The repository-relative path `folder/name` for a typed `name`, or why
/// the name is refused. Does not touch the disk except to check the target
/// does not exist yet.
pub(crate) fn validate(root: &Path, folder: &Path, name: &str) -> Result<PathBuf, String> {
    let name = name.trim();
    if name.is_empty() {
        return Err(msg::EMPTY_NAME.get().into());
    }
    if let Some(c) = name.chars().find(|c| INVALID.contains(c) || c.is_control()) {
        return Err(msg::bad_char(c));
    }
    // A backslash is a separator too, whatever the platform.
    let name = name.replace('\\', "/");
    if name.starts_with('/') {
        return Err(msg::ABSOLUTE.get().into());
    }
    if name.ends_with('/') {
        return Err(msg::ENDS_FOLDER.get().into());
    }
    // Built as one forward-slash string, the spelling every other path in
    // the Status pane and the editor uses.
    let mut parts: Vec<String> = Vec::new();
    let folder_text = folder.to_string_lossy().replace('\\', "/");
    if !folder_text.is_empty() {
        parts.push(folder_text);
    }
    for (i, part) in name.split('/').enumerate() {
        if part.is_empty() {
            return Err(msg::EMPTY_FOLDER.get().into());
        }
        if part == "." || part == ".." {
            return Err(msg::bad_char(part));
        }
        if part.ends_with('.') || part.ends_with(' ') {
            return Err(msg::dot_space(part));
        }
        if i == 0 && folder.as_os_str().is_empty() && part.eq_ignore_ascii_case(".git") {
            return Err(msg::GIT_RESERVED.get().into());
        }
        parts.push(part.to_string());
    }
    let text = parts.join("/");
    let relative = PathBuf::from(&text);
    if root.join(&relative).exists() {
        return Err(msg::exists(&text));
    }
    Ok(relative)
}

/// Validates then creates the empty file (and its folders). Returns the
/// repository-relative path.
pub(crate) fn create(root: &Path, folder: &Path, name: &str) -> Result<PathBuf, String> {
    let relative = validate(root, folder, name)?;
    let absolute = root.join(&relative);
    if let Some(parent) = absolute.parent() {
        std::fs::create_dir_all(parent).map_err(|e| msg::mkdir_failed(parent.display(), e))?;
    }
    std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&absolute)
        .map_err(|e| msg::create_failed(relative.display(), e))?;
    Ok(relative)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_empty_or_blank_name_is_refused() {
        let dir = tempfile::tempdir().unwrap();
        assert!(validate(dir.path(), Path::new(""), "").is_err());
        assert!(validate(dir.path(), Path::new(""), "   ").is_err());
    }

    #[test]
    fn invalid_characters_and_odd_components_are_refused() {
        let dir = tempfile::tempdir().unwrap();
        for bad in [
            "a*b.txt", "a?.txt", "x:y", "a|b", "../x.txt", "/abs.txt", "d/", "a//b", "x.",
        ] {
            assert!(
                validate(dir.path(), Path::new(""), bad).is_err(),
                "{bad:?} should be refused"
            );
        }
        assert!(validate(dir.path(), Path::new(""), ".git/config").is_err());
    }

    #[test]
    fn intermediate_folders_are_allowed_and_prefixed_by_the_folder() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(
            validate(dir.path(), Path::new(""), "notes/new.md").unwrap(),
            Path::new("notes").join("new.md")
        );
        assert_eq!(
            validate(dir.path(), Path::new("src"), "deep\\er/x.rs").unwrap(),
            Path::new("src").join("deep").join("er").join("x.rs")
        );
    }

    #[test]
    fn an_existing_file_or_folder_is_refused() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("a.txt"), "x").unwrap();
        std::fs::create_dir(dir.path().join("d")).unwrap();
        assert!(validate(dir.path(), Path::new(""), "a.txt")
            .unwrap_err()
            .contains("already exists"));
        assert!(validate(dir.path(), Path::new(""), "d").is_err());
    }

    #[test]
    fn create_makes_the_empty_file_and_its_folders() {
        let dir = tempfile::tempdir().unwrap();
        let rel = create(dir.path(), Path::new(""), "notes/new.md").unwrap();
        let abs = dir.path().join(&rel);
        assert!(abs.is_file());
        assert_eq!(std::fs::metadata(abs).unwrap().len(), 0);
        assert!(create(dir.path(), Path::new(""), "notes/new.md").is_err());
    }
}
