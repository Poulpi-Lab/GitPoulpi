//! Path cleanup shared by every crate that canonicalises a filesystem
//! path and then has to show or store the result.
//!
//! `std::path::Path::canonicalize` on Windows answers with the
//! ["verbatim" prefix][verbatim] (`\\?\C:\x`, or `\\?\UNC\server\share`
//! for a network path). Left in, it leaks into title bars, path fields
//! and session files -- correct as a path, useless as text or as a
//! `current_dir` for another tool. This helper is the single place that
//! strips it.
//!
//! [verbatim]: https://learn.microsoft.com/en-us/dotnet/standard/io/file-path-formats#uncprefixed

use std::path::{Path, PathBuf};

/// Drops Windows' verbatim path prefix from an already-canonicalised
/// path. A no-op on every other platform, and a no-op for a path that
/// never had the prefix - safe to call on anything.
///
/// Both prefixes start with **two** backslashes: `canonicalize` answers
/// `\\?\C:\x`, not `\?\C:\x`. A UNC path becomes `\\server\share`, not
/// `\server\share` - the two backslashes UNC paths need are exactly
/// what `\\?\UNC\` already supplied.
pub fn strip_verbatim_prefix(path: &Path) -> PathBuf {
    const VERBATIM_UNC: &str = r"\\?\UNC\";
    const VERBATIM: &str = r"\\?\";
    let text = path.to_string_lossy();
    if let Some(rest) = text.strip_prefix(VERBATIM_UNC) {
        return PathBuf::from(format!(r"\\{rest}"));
    }
    match text.strip_prefix(VERBATIM) {
        Some(rest) => PathBuf::from(rest),
        None => path.to_path_buf(),
    }
}

/// `path.canonicalize()`, with the verbatim prefix stripped from a
/// successful result. Falls back to `path` unchanged when the filesystem
/// can't answer (deleted, unreadable, or never existed) - a stale
/// reference still has to be nameable.
pub fn canonicalize_display(path: &Path) -> PathBuf {
    match path.canonicalize() {
        Ok(canonical) => strip_verbatim_prefix(&canonical),
        Err(_) => path.to_path_buf(),
    }
}

/// `path` as given when absolute, else joined onto `base`.
fn resolve_against(base: &Path, path: &str) -> PathBuf {
    let path = Path::new(path);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        base.join(path)
    }
}

/// The per-worktree git directory of the checkout at `root`: `.git` when
/// it is a directory, else the target of a `gitdir: <path>` file (linked
/// worktree or submodule; a relative target is relative to `root`). `None`
/// when `.git` is missing or is not a gitdir file; a bare repository (no
/// `.git`) is left to the caller, which knows what marks one.
pub fn git_dir(root: &Path) -> Option<PathBuf> {
    let dot_git = root.join(".git");
    if dot_git.is_dir() {
        return Some(dot_git);
    }
    let text = std::fs::read_to_string(&dot_git).ok()?;
    let target = text.trim().lines().next()?.strip_prefix("gitdir:")?.trim();
    Some(resolve_against(root, target))
}

/// The directory shared by every worktree of the repository: a linked
/// worktree's git directory names it in a `commondir` file (absolute, or
/// relative to `git_dir`); without the file `git_dir` already is it.
pub fn common_dir(git_dir: &Path) -> PathBuf {
    match std::fs::read_to_string(git_dir.join("commondir")) {
        Ok(text) => resolve_against(git_dir, text.trim()),
        Err(_) => git_dir.to_path_buf(),
    }
}

/// Whether the repository at `root` stores its refs in the reftable format
/// (`extensions.refStorage = reftable`): no loose ref files, no `logs/`
/// directory, and libgit2 cannot open it, so everything that would read those
/// goes through the system `git` instead.
pub fn uses_reftable(root: &Path) -> bool {
    let Some(dir) = git_dir(root) else {
        return false;
    };
    let config = std::fs::read_to_string(common_dir(&dir).join("config")).unwrap_or_default();
    config.lines().any(|l| {
        let l: String = l.chars().filter(|c| !c.is_whitespace()).collect();
        l.eq_ignore_ascii_case("refstorage=reftable")
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_the_plain_verbatim_prefix() {
        assert_eq!(
            strip_verbatim_prefix(Path::new(r"\\?\C:\src\GitPoulpi")),
            PathBuf::from(r"C:\src\GitPoulpi")
        );
    }

    #[test]
    fn strips_the_unc_verbatim_prefix_keeping_two_leading_backslashes() {
        assert_eq!(
            strip_verbatim_prefix(Path::new(r"\\?\UNC\server\share\repo")),
            PathBuf::from(r"\\server\share\repo")
        );
    }

    #[test]
    fn leaves_a_path_without_the_prefix_untouched() {
        assert_eq!(
            strip_verbatim_prefix(Path::new(r"C:\src")),
            PathBuf::from(r"C:\src")
        );
        assert_eq!(
            strip_verbatim_prefix(Path::new("/home/user/repo")),
            PathBuf::from("/home/user/repo")
        );
    }

    #[test]
    fn canonicalize_display_falls_back_when_the_path_is_gone() {
        let gone = Path::new(if cfg!(windows) {
            r"Z:\definitely\not\here\at\all"
        } else {
            "/definitely/not/here/at/all"
        });
        assert_eq!(canonicalize_display(gone), gone.to_path_buf());
    }

    #[test]
    fn git_dir_of_a_plain_checkout_is_dot_git() {
        let root = tempfile::tempdir().unwrap();
        std::fs::create_dir(root.path().join(".git")).unwrap();
        assert_eq!(git_dir(root.path()), Some(root.path().join(".git")));
    }

    #[test]
    fn git_dir_follows_a_relative_gitdir_file() {
        let root = tempfile::tempdir().unwrap();
        std::fs::write(
            root.path().join(".git"),
            "gitdir: ../main/.git/worktrees/feature\n",
        )
        .unwrap();
        assert_eq!(
            git_dir(root.path()),
            Some(root.path().join("../main/.git/worktrees/feature"))
        );
    }

    #[test]
    fn git_dir_keeps_an_absolute_gitdir_target() {
        let root = tempfile::tempdir().unwrap();
        let target = root.path().join("elsewhere");
        std::fs::write(
            root.path().join(".git"),
            format!("gitdir: {}\n", target.display()),
        )
        .unwrap();
        assert_eq!(git_dir(root.path()), Some(target));
    }

    #[test]
    fn git_dir_rejects_a_missing_or_foreign_dot_git() {
        let root = tempfile::tempdir().unwrap();
        assert_eq!(git_dir(root.path()), None);
        std::fs::write(root.path().join(".git"), "not a gitdir file").unwrap();
        assert_eq!(git_dir(root.path()), None);
    }

    #[test]
    fn git_dir_of_a_bare_repository_is_left_to_the_caller() {
        let root = tempfile::tempdir().unwrap();
        std::fs::write(root.path().join("HEAD"), "ref: refs/heads/main\n").unwrap();
        std::fs::create_dir(root.path().join("refs")).unwrap();
        assert_eq!(git_dir(root.path()), None);
    }

    #[test]
    fn common_dir_is_the_git_dir_without_a_commondir_file() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(common_dir(dir.path()), dir.path());
    }

    #[test]
    fn common_dir_resolves_relative_and_absolute_commondir_files() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("commondir"), "../..\n").unwrap();
        assert_eq!(common_dir(dir.path()), dir.path().join("../.."));
        let abs = dir.path().join("shared");
        std::fs::write(dir.path().join("commondir"), abs.display().to_string()).unwrap();
        assert_eq!(common_dir(dir.path()), abs);
    }
}
