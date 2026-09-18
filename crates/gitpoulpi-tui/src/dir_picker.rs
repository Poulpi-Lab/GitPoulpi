//! Path text helpers shared by the folder picker (`folder_picker`): how a
//! half-typed path splits into "folder to list" and "name fragment", how
//! a typed path is made absolute, which entries are shown, and what a
//! folder's parent is.
//!
//! The reading itself lives in `folder_listing` (off the input loop).
//!
//! There is no external-picker seam. If yazi, superfile, lf, nnn or the OS
//! folder dialog is added, it plugs in where
//! `repo_dialogs::RepoDialogs::new_picker` builds a `FolderPicker`, and it
//! has to solve what the built-in picker does not: hand the terminal over
//! (raw mode, alternate screen and mouse capture off before the child, back
//! on after, as an RAII guard so a picker that panics, is killed or exits
//! non-zero still gives the terminal back, then a full redraw); probe `PATH`
//! at runtime; and get the chosen folder back out (for yazi that is
//! `--cwd-file`, **not** `--chooser-file`, which reports selected *files*
//! and can never return a folder).

use std::path::{Path, PathBuf};

/// One subdirectory, as the overlay needs to show it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirEntryInfo {
    pub name: String,
    pub path: PathBuf,
    /// A git repository, so the overlay can say so before the user
    /// commits to opening it. Cheap - the same `.git` / bare-layout
    /// check `session_store::is_repository` makes, which is a handful
    /// of `stat` calls and no subprocess.
    pub is_repo: bool,
    /// A dot-folder (`.git`, `.cache`, ...) or, on Windows, one carrying the
    /// hidden file attribute. `read_folder` fills this in on every entry
    /// rather than dropping hidden ones, so the picker's "Show hidden
    /// folders" checkbox is a pure render-time filter with no re-read.
    pub is_hidden: bool,
}

/// How many entries one directory listing keeps. Ten times the tallest
/// list the panel can draw, so scrolling has somewhere to go while a
/// pathological directory still costs a bounded amount of work.
pub const LISTING_LIMIT: usize = 500;

// -- typed-path completion -----------------------------------------------

/// What a typed path splits into for completion: the directory being
/// listed, and the fragment of a name typed so far.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fragment {
    pub dir: PathBuf,
    pub prefix: String,
}

/// Splits a typed path into the directory to list and the name fragment to
/// match, resolving relative paths against `cwd`.
///
/// A **trailing separator means "inside this directory"**: `C:\work\` lists
/// `C:\work`, while `C:\work` lists `C:\` filtered to names starting with
/// `work` (as every shell does); getting it wrong makes the completion list
/// jump a level the moment the user types the separator.
///
/// Both separators are accepted on every platform, because Windows users
/// type both and `PathBuf` on Windows accepts both.
pub fn split_fragment(cwd: &Path, typed: &str) -> Fragment {
    let typed = typed.trim_start();
    if typed.is_empty() {
        return Fragment {
            dir: cwd.to_path_buf(),
            prefix: String::new(),
        };
    }
    // Split the **string the user typed**, at its last separator, not the
    // `Path` it parses into: `Path::parent`/`file_name` normalise. `/work/.`
    // has no file name as far as `Path` is concerned (the `.` is a `CurDir`
    // component), so a user typing a dot to reveal hidden folders would have
    // the whole fragment swallowed and get the parent's listing back.
    match typed.rfind(['/', '\\']) {
        Some(cut) => {
            let (head, tail) = typed.split_at(cut + 1);
            Fragment {
                // The trailing separator is kept rather than trimmed:
                // trimming turns `C:\` into `C:`, which on Windows is the
                // *drive-relative* current directory, not the drive root.
                // `Path` compares by components, so the separator costs
                // nothing.
                dir: absolutize(cwd, head),
                prefix: tail.to_string(),
            }
        }
        None => Fragment {
            dir: cwd.to_path_buf(),
            prefix: typed.to_string(),
        },
    }
}

/// A typed path made absolute against `cwd`.
///
/// Shared by completion and by the overlay's Enter handling so a path
/// cannot be resolved two different ways by two pieces of the same tab.
pub fn absolutize(cwd: &Path, typed: &str) -> PathBuf {
    let path = PathBuf::from(typed);
    if path.is_absolute() {
        path
    } else {
        cwd.join(path)
    }
}

/// The shared visibility rule: a prefix match, hidden entries only once a
/// dot is typed, never `.git`. One function because the browser and the
/// completion list must agree on what is offered.
pub fn filter_entries(
    entries: Vec<DirEntryInfo>,
    prefix: &str,
) -> impl Iterator<Item = DirEntryInfo> + '_ {
    let wanted = prefix.to_lowercase();
    let show_hidden = prefix.starts_with('.');
    entries.into_iter().filter(move |e| {
        e.name.to_lowercase().starts_with(&wanted)
            && (show_hidden || !e.name.starts_with('.'))
            && e.name != ".git"
    })
}

// -- drive roots and parents ---------------------------------------------

/// The directory one level up, or `None` at a root.
///
/// `Path::parent` is *almost* this: for `C:\` it answers `None` (correct),
/// but for a path built from a typed draft it can answer `Some("")`, which
/// is not a directory anyone can list. Everything that walks upwards goes
/// through here.
pub fn parent_of(dir: &Path) -> Option<PathBuf> {
    let parent = dir.parent()?;
    if parent.as_os_str().is_empty() {
        return None;
    }
    Some(parent.to_path_buf())
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    fn entries(names: &[(&str, bool)]) -> Vec<DirEntryInfo> {
        names
            .iter()
            .map(|(name, is_repo)| DirEntryInfo {
                name: name.to_string(),
                path: PathBuf::from("/work").join(name),
                is_repo: *is_repo,
                is_hidden: name.starts_with('.'),
            })
            .collect()
    }

    fn work() -> Vec<DirEntryInfo> {
        entries(&[
            ("alpha", true),
            ("Albatross", false),
            ("beta", true),
            (".hidden", false),
            (".git", false),
        ])
    }

    #[test]
    fn a_trailing_separator_lists_inside_the_directory() {
        let f = split_fragment(Path::new("/work"), "/work/");
        assert_eq!(f.dir, PathBuf::from("/work"));
        assert_eq!(f.prefix, "");
    }

    /// Without the trailing separator the last segment is a *fragment*
    /// to match, not a directory to list. Getting this backwards makes
    /// the completion list jump a level the moment the user types the
    /// separator that should have taken them into the folder.
    #[test]
    fn no_trailing_separator_matches_the_last_segment_in_its_parent() {
        let f = split_fragment(Path::new("/work"), "/work/al");
        assert_eq!(f.dir, PathBuf::from("/work"));
        assert_eq!(f.prefix, "al");
    }

    #[test]
    fn a_relative_fragment_is_resolved_against_the_working_directory() {
        let f = split_fragment(Path::new("/work"), "al");
        assert_eq!(f.dir, PathBuf::from("/work"));
        assert_eq!(f.prefix, "al");
    }

    /// A backslash is a separator on Windows and users type it. The
    /// draft is a string the user typed, not a `Path` the OS built.
    #[test]
    #[cfg(windows)]
    fn a_backslash_ends_a_directory_on_windows() {
        let f = split_fragment(Path::new(r"C:\work"), r"C:\work\");
        assert_eq!(f.dir, PathBuf::from(r"C:\work"));
        assert_eq!(f.prefix, "");
    }

    /// A drive root has no parent, and `..` must not be offered from
    /// one. `Path::parent` gets this right; a hand-rolled "chop at the
    /// last separator" does not, and would offer a `C:` that lists
    /// nothing.
    #[test]
    #[cfg(windows)]
    fn a_drive_root_has_no_parent() {
        assert_eq!(parent_of(Path::new(r"C:\")), None);
        assert_eq!(
            parent_of(Path::new(r"C:\work\repo")),
            Some(PathBuf::from(r"C:\work"))
        );
    }

    #[test]
    #[cfg(not(windows))]
    fn the_filesystem_root_has_no_parent() {
        assert_eq!(parent_of(Path::new("/")), None);
        assert_eq!(
            parent_of(Path::new("/work/repo")),
            Some(PathBuf::from("/work"))
        );
    }

    /// Windows paths are case-insensitive and users type lowercase.
    #[test]
    fn matching_is_case_insensitive() {
        let names: Vec<String> = filter_entries(work(), "al").map(|e| e.name).collect();
        assert_eq!(names, ["alpha", "Albatross"]);
    }

    #[test]
    fn hidden_directories_appear_only_once_a_dot_is_typed() {
        let plain: Vec<String> = filter_entries(work(), "").map(|e| e.name).collect();
        assert_eq!(plain, ["alpha", "Albatross", "beta"]);
        let dotted: Vec<String> = filter_entries(work(), ".").map(|e| e.name).collect();
        // `.git` is never offered.
        assert_eq!(dotted, [".hidden"]);
    }
}
