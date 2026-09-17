//! The repository's commit message template (Settings > Repository > Commit
//! template): a text file in the repo data directory that `commit.template`
//! points at, so the system `commit` and other tools agree with the TUI.
//!
//! Everything here is pure paths and text; the git config side
//! (`commit.template`, `core.commentChar`) goes through the daemon.

use std::path::{Path, PathBuf};

use crate::Result;

/// The template's file name inside the repo data directory.
pub const TEMPLATE_FILE: &str = "commit-template.txt";

/// Where GitPoulpi keeps its own template: beside `repo-state.toml`.
pub fn own_template_path(repo_root: &Path, state_dir: &str) -> PathBuf {
    gitpoulpi_config::repo_data_file(repo_root, state_dir, TEMPLATE_FILE)
}

/// The file the template lives in: what `commit.template` names (`configured`,
/// `~/` expanded with `home`, a relative path taken from the work tree root)
/// when it is set, GitPoulpi's own file otherwise.
pub fn template_path(
    repo_root: &Path,
    state_dir: &str,
    configured: Option<&str>,
    home: Option<&Path>,
) -> PathBuf {
    let Some(value) = configured.map(str::trim).filter(|v| !v.is_empty()) else {
        return own_template_path(repo_root, state_dir);
    };
    if let Some(rest) = value.strip_prefix("~/") {
        if let Some(home) = home {
            return home.join(rest);
        }
    }
    let path = PathBuf::from(value);
    if path.is_absolute() {
        path
    } else {
        repo_root.join(path)
    }
}

/// The template's text; empty when there is no file (or it is unreadable).
pub fn read_template(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_default()
}

/// Writes the template, creating the directory (and, in GitPoulpi's own data
/// directory, the ignore that keeps it out of the work tree's status).
pub fn write_template(path: &Path, text: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
        crate::repo_state::ensure_state_gitignore(parent)?;
    }
    std::fs::write(path, text)?;
    Ok(())
}

/// The template as the commit panel's two fields: the first line is the
/// Summary, the rest (after the blank line that separates them) the
/// Description.
pub fn split_template(text: &str) -> (String, String) {
    let text = text.replace("\r\n", "\n");
    let (summary, rest) = text.split_once('\n').unwrap_or((text.as_str(), ""));
    let rest = rest.strip_prefix('\n').unwrap_or(rest);
    (summary.to_string(), rest.trim_end_matches('\n').to_string())
}

/// The inverse of [`split_template`]: what the file holds for two fields.
pub fn join_template(summary: &str, description: &str) -> String {
    if description.trim().is_empty() {
        if summary.is_empty() {
            String::new()
        } else {
            format!("{summary}\n")
        }
    } else {
        format!("{summary}\n\n{}\n", description.trim_end_matches('\n'))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_template_splits_into_summary_and_description_and_back() {
        let (s, d) = split_template("fix: what\n\nwhy\nmore\n");
        assert_eq!((s.as_str(), d.as_str()), ("fix: what", "why\nmore"));
        assert_eq!(join_template(&s, &d), "fix: what\n\nwhy\nmore\n");
        assert_eq!(split_template("only a summary\n").1, "");
        assert_eq!(join_template("only a summary", ""), "only a summary\n");
        assert_eq!(join_template("", ""), "");
        // A template that starts with a comment block keeps it as text.
        let (s, d) = split_template("\n# Say why\n# not what\n");
        assert_eq!((s.as_str(), d.as_str()), ("", "# Say why\n# not what"));
        assert_eq!(split_template("a\r\n\r\nb\r\n").1, "b");
    }

    #[test]
    fn the_template_path_follows_commit_template_or_the_repo_data_dir() {
        let root = Path::new("/work/repo");
        let own = template_path(root, ".gitpoulpi", None, None);
        assert_eq!(own, root.join(".gitpoulpi").join(TEMPLATE_FILE));
        assert_eq!(template_path(root, ".gitpoulpi", Some("  "), None), own);
        assert_eq!(
            template_path(root, ".gitpoulpi", Some("/etc/tpl.txt"), None),
            Path::new("/etc/tpl.txt")
        );
        assert_eq!(
            template_path(root, ".gitpoulpi", Some("docs/tpl.txt"), None),
            root.join("docs/tpl.txt")
        );
        assert_eq!(
            template_path(
                root,
                ".gitpoulpi",
                Some("~/tpl.txt"),
                Some(Path::new("/home/me"))
            ),
            Path::new("/home/me/tpl.txt")
        );
    }

    #[test]
    fn writing_a_template_creates_its_directory_and_reads_back() {
        let dir = tempfile::tempdir().unwrap();
        let path = own_template_path(dir.path(), ".gitpoulpi");
        assert_eq!(read_template(&path), "");
        write_template(&path, "a\n\nb\n").unwrap();
        assert_eq!(read_template(&path), "a\n\nb\n");
        assert!(path.parent().unwrap().join(".gitignore").exists());
    }
}
