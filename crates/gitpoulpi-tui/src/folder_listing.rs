//! Reading folders for the folder picker, **off the input loop**.
//!
//! The picker (`folder_picker.rs`) is pure: it says which folder it wants
//! listed and is handed the answer later. This module is the other half:
//! [`read_folder`] does the actual `read_dir`, and [`Lister`] runs it on
//! a thread per request and reports on an `std::sync::mpsc` channel the
//! event loop drains with `try_recv` -- the rule every other background
//! answer in this TUI follows (`repo_probe`, `clone_task`, `init_task`).
//!
//! **Why a thread per request rather than one worker.** A folder on a slow
//! or dead network share can take tens of seconds to answer (or never
//! answer). With one worker, every later request would queue behind it and
//! the picker would look frozen after the user navigated elsewhere. A
//! thread per request costs nothing noticeable at human navigation rates,
//! and a hung one only delays an answer nobody is waiting for (the picker
//! drops answers for folders it is no longer showing).
//!
//! The quick places (Home, Desktop, drives, ...) go through the same
//! channel for the same reason: probing `D:\` for existence can block on
//! an unplugged network drive letter.

use crate::redraw::NotifySend as _;
use std::path::{Path, PathBuf};
use std::sync::mpsc;

use crate::dir_picker::DirEntryInfo;
use gitpoulpi_i18n::msg;

/// How many entries one listing keeps, folders and files together
/// (`dir_picker::LISTING_LIMIT`): a `node_modules` with ten thousand
/// children is read, allocated and sorted once per navigation, never per
/// keystroke, but it is still bounded.
pub const FOLDER_LIMIT: usize = crate::dir_picker::LISTING_LIMIT;

/// A file, as the picker shows it - shown dimmed, never chosen.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FileEntry {
    pub name: String,
    /// A dot-file or, on Windows, one carrying the hidden attribute.
    pub is_hidden: bool,
}

/// One folder's content, as the picker shows it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FolderListing {
    /// Every subfolder, sorted case-insensitively - hidden ones
    /// (dot-folders, `.git`, Windows hidden-attribute folders) included,
    /// marked [`DirEntryInfo::is_hidden`] rather than left out, so the
    /// picker's "Show hidden folders" checkbox is a render-time filter
    /// and flipping it never needs a re-read.
    pub dirs: Vec<DirEntryInfo>,
    /// Every file, sorted the same way.
    pub files: Vec<FileEntry>,
    /// The folder itself is a git repository.
    pub is_repo: bool,
    /// There were more entries than [`FOLDER_LIMIT`].
    pub truncated: bool,
    /// The folder could not be read, and why.
    pub error: Option<String>,
}

/// Reads `dir`: its subfolders (with which of them are repositories), its
/// files, and whether it is a repository itself. Blocking - call it on
/// a thread ([`Lister`]) or from a test.
///
/// Hidden entries are read and marked, never dropped - see
/// [`FolderListing::dirs`].
pub fn read_folder(dir: &Path, limit: usize) -> FolderListing {
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(err) => {
            return FolderListing {
                error: Some(friendly_io_error(&err)),
                ..FolderListing::default()
            }
        }
    };
    let mut dirs = Vec::new();
    let mut files = Vec::new();
    let mut truncated = false;
    for entry in entries.flatten() {
        if dirs.len() + files.len() >= limit {
            truncated = true;
            break;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        let hidden = is_hidden_entry(&name, &entry);
        // `file_type` comes out of the directory read itself on Windows
        // and Linux: no `stat` per child.
        let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
        if is_dir {
            let path = entry.path();
            // `.git` itself happens to satisfy the bare-repository layout
            // check (`HEAD` + `objects` + `refs` right inside it) -- never
            // mark it a repository in its own right.
            let is_repo = name != ".git" && crate::session_store::is_repository(&path);
            dirs.push(DirEntryInfo {
                is_repo,
                is_hidden: hidden,
                name,
                path,
            });
        } else {
            files.push(FileEntry {
                name,
                is_hidden: hidden,
            });
        }
    }
    dirs.sort_by_key(|e| e.name.to_lowercase());
    files.sort_by_key(|f| f.name.to_lowercase());
    FolderListing {
        dirs,
        files,
        is_repo: crate::session_store::is_repository(dir),
        truncated,
        error: None,
    }
}

/// A dot-name, or (Windows) an entry carrying the hidden file attribute.
fn is_hidden_entry(name: &str, entry: &std::fs::DirEntry) -> bool {
    if name.starts_with('.') {
        return true;
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        const FILE_ATTRIBUTE_HIDDEN: u32 = 0x2;
        if let Ok(meta) = entry.metadata() {
            if meta.file_attributes() & FILE_ATTRIBUTE_HIDDEN != 0 {
                return true;
            }
        }
    }
    #[cfg(not(windows))]
    let _ = entry;
    false
}

/// `io::Error`'s own text on Windows ends in "(os error 5)", which tells
/// the user nothing they can act on.
fn friendly_io_error(err: &std::io::Error) -> String {
    match err.kind() {
        std::io::ErrorKind::NotFound => msg::folder_picker::NO_SUCH_FOLDER.get().into(),
        std::io::ErrorKind::PermissionDenied => msg::folder_picker::PERMISSION_DENIED.get().into(),
        _ => {
            let text = err.to_string();
            match text.split_once(" (os error") {
                Some((head, _)) => head.to_string(),
                None => text,
            }
        }
    }
}

/// A quick place in the picker's left column.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Place {
    pub label: String,
    pub path: PathBuf,
    pub kind: PlaceKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaceKind {
    /// Home, Desktop, Documents, Downloads, the clone folder.
    Standard,
    /// A drive root (Windows) or `/`.
    Drive,
    /// The parent folder of a favourite or recent repository.
    Recent,
}

/// The quick places, given where home is and the extra folders the
/// caller knows about (the clone root, favourite / recent repositories'
/// parents). Blocking (existence checks) - run by [`Lister`].
pub fn quick_places(home: Option<&Path>, extra: &[(String, PathBuf)]) -> Vec<Place> {
    fn push(label: String, path: PathBuf, kind: PlaceKind, places: &mut Vec<Place>) {
        // Windows' verbatim prefix is stripped before the comparison (and
        // before the path is kept): two spellings of one folder, "C:\src"
        // from a favourite's parent and "\\?\C:\src" from one built off a
        // canonicalized `RepoRef::root`, must collapse to one QUICK ACCESS
        // row.
        let path = gitpoulpi_core::paths::strip_verbatim_prefix(&path);
        if path.is_dir() && !places.iter().any(|p| same_place(&p.path, &path)) {
            places.push(Place { label, path, kind });
        }
    }

    /// Windows paths are case-insensitive; two quick places built from
    /// different labels for the same folder ("src" the clone root's
    /// parent, "src" a favourite's parent) must be recognized as one
    /// even when they were typed or cased differently.
    fn same_place(a: &Path, b: &Path) -> bool {
        if cfg!(windows) {
            a.as_os_str()
                .to_string_lossy()
                .eq_ignore_ascii_case(&b.as_os_str().to_string_lossy())
        } else {
            a == b
        }
    }
    let mut places = Vec::new();
    if let Some(home) = home {
        push(
            msg::folder_picker::PLACE_HOME.get().into(),
            home.to_path_buf(),
            PlaceKind::Standard,
            &mut places,
        );
        // The folder's name on disk, and what the list calls it.
        for (name, label) in [
            ("Desktop", msg::folder_picker::PLACE_DESKTOP),
            ("Documents", msg::folder_picker::PLACE_DOCUMENTS),
            ("Downloads", msg::folder_picker::PLACE_DOWNLOADS),
        ] {
            push(
                label.get().into(),
                home.join(name),
                PlaceKind::Standard,
                &mut places,
            );
        }
    }
    for (label, path) in extra {
        let kind = if label == msg::folder_picker::CLONE_FOLDER.get() {
            PlaceKind::Standard
        } else {
            PlaceKind::Recent
        };
        push(label.clone(), path.clone(), kind, &mut places);
    }
    for root in drive_roots() {
        let label = root.display().to_string();
        push(label, root, PlaceKind::Drive, &mut places);
    }
    places
}

/// Every drive root: `A:\` .. `Z:\` that exist on Windows, `/` elsewhere.
fn drive_roots() -> Vec<PathBuf> {
    if cfg!(windows) {
        ('A'..='Z')
            .map(|letter| PathBuf::from(format!("{letter}:\\")))
            .filter(|root| root.exists())
            .collect()
    } else {
        vec![PathBuf::from("/")]
    }
}

/// What the lister reports.
#[derive(Debug, Clone)]
pub enum ListerEvent {
    Listed(PathBuf, FolderListing),
    Places(Vec<Place>),
}

/// Runs folder reads on threads and hands the answers back through a
/// channel. See the module comment for why a thread per request.
pub struct Lister {
    tx: mpsc::Sender<ListerEvent>,
    rx: mpsc::Receiver<ListerEvent>,
}

impl Default for Lister {
    fn default() -> Self {
        let (tx, rx) = mpsc::channel();
        Self { tx, rx }
    }
}

impl Lister {
    /// Lists `dir` in the background.
    pub fn request(&self, dir: PathBuf) {
        let tx = self.tx.clone();
        std::thread::spawn(move || {
            let listing = read_folder(&dir, FOLDER_LIMIT);
            let _ = tx.send_notify(ListerEvent::Listed(dir, listing));
        });
    }

    /// Works out the quick places in the background.
    pub fn request_places(&self, home: Option<PathBuf>, extra: Vec<(String, PathBuf)>) {
        let tx = self.tx.clone();
        std::thread::spawn(move || {
            let _ = tx.send_notify(ListerEvent::Places(quick_places(home.as_deref(), &extra)));
        });
    }

    /// Every answer that has arrived. Never waits.
    pub fn drain(&self) -> Vec<ListerEvent> {
        std::iter::from_fn(|| self.rx.try_recv().ok()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_listing_separates_folders_from_files_and_marks_repositories() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join("repo").join(".git")).unwrap();
        std::fs::create_dir(tmp.path().join("Plain")).unwrap();
        std::fs::create_dir(tmp.path().join(".hidden")).unwrap();
        std::fs::write(tmp.path().join("notes.txt"), "x").unwrap();
        let listing = read_folder(tmp.path(), FOLDER_LIMIT);
        let names: Vec<(&str, bool)> = listing
            .dirs
            .iter()
            .map(|d| (d.name.as_str(), d.is_repo))
            .collect();
        // `.hidden` is read, not dropped -- it is marked hidden instead
        // so the picker can filter it at render time.
        assert_eq!(
            names,
            [(".hidden", false), ("Plain", false), ("repo", true)]
        );
        let hidden: Vec<&str> = listing
            .dirs
            .iter()
            .filter(|d| d.is_hidden)
            .map(|d| d.name.as_str())
            .collect();
        assert_eq!(hidden, [".hidden"]);
        let files: Vec<&str> = listing.files.iter().map(|f| f.name.as_str()).collect();
        assert_eq!(files, ["notes.txt"]);
        assert!(!listing.is_repo);
        assert!(read_folder(&tmp.path().join("repo"), FOLDER_LIMIT).is_repo);
        // `.git` itself is never marked a repository, even though its
        // own layout (HEAD, objects/, refs/) would otherwise pass the
        // bare-repository check.
        let git_entry = read_folder(&tmp.path().join("repo"), FOLDER_LIMIT)
            .dirs
            .into_iter()
            .find(|d| d.name == ".git");
        assert_eq!(git_entry.map(|d| d.is_repo), Some(false));
    }

    /// Dot-files are read and marked hidden too, not
    /// dropped - the "Show hidden folders" checkbox covers files that
    /// happen to sit in the list (`.gitignore` and the like), not only
    /// folders.
    #[test]
    fn hidden_files_are_read_and_marked_hidden() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join(".gitignore"), "x").unwrap();
        std::fs::write(tmp.path().join("readme.md"), "x").unwrap();
        let listing = read_folder(tmp.path(), FOLDER_LIMIT);
        let files: Vec<(&str, bool)> = listing
            .files
            .iter()
            .map(|f| (f.name.as_str(), f.is_hidden))
            .collect();
        assert_eq!(files, [(".gitignore", true), ("readme.md", false)]);
    }

    #[test]
    fn an_unreadable_folder_is_an_error_not_an_empty_one() {
        let tmp = tempfile::tempdir().unwrap();
        let listing = read_folder(&tmp.path().join("missing"), FOLDER_LIMIT);
        assert_eq!(listing.error.as_deref(), Some("no such folder"));
    }

    #[test]
    fn the_quick_places_keep_existing_folders_once_and_offer_a_drive() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir(tmp.path().join("Desktop")).unwrap();
        let places = quick_places(
            Some(tmp.path()),
            &[
                ("Clone folder".into(), tmp.path().to_path_buf()),
                ("work".into(), tmp.path().join("nope")),
            ],
        );
        let labels: Vec<&str> = places.iter().map(|p| p.label.as_str()).collect();
        assert_eq!(labels[..2], ["Home", "Desktop"]);
        // The clone folder is Home again: listed once. A missing folder
        // is not offered at all.
        assert!(!labels.contains(&"Clone folder") && !labels.contains(&"work"));
        assert!(places.iter().any(|p| p.kind == PlaceKind::Drive));
    }

    /// Two spellings of one folder - one with the
    /// Windows verbatim prefix, one without - must collapse to a single
    /// QUICK ACCESS row rather than listing "src" twice.
    #[test]
    fn a_verbatim_prefixed_duplicate_of_an_already_listed_place_is_dropped() {
        let tmp = tempfile::tempdir().unwrap();
        let verbatim = PathBuf::from(format!(r"\\?\{}", tmp.path().display()));
        let places = quick_places(
            None,
            &[
                ("Clone folder".into(), tmp.path().to_path_buf()),
                ("src".into(), verbatim),
            ],
        );
        let matches = places.iter().filter(|p| p.kind != PlaceKind::Drive).count();
        assert_eq!(
            matches, 1,
            "the verbatim spelling is the same folder: {places:?}"
        );
        assert!(!places[0].path.display().to_string().contains(r"\\?\"));
    }

    #[test]
    #[cfg(windows)]
    fn two_differently_cased_spellings_of_one_folder_collapse_on_windows() {
        let tmp = tempfile::tempdir().unwrap();
        let upper = PathBuf::from(tmp.path().display().to_string().to_uppercase());
        let places = quick_places(
            None,
            &[
                ("work".into(), tmp.path().to_path_buf()),
                ("WORK".into(), upper),
            ],
        );
        let matches = places.iter().filter(|p| p.kind != PlaceKind::Drive).count();
        assert_eq!(matches, 1, "{places:?}");
    }

    #[test]
    fn the_lister_answers_off_thread() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir(tmp.path().join("a")).unwrap();
        let lister = Lister::default();
        lister.request(tmp.path().to_path_buf());
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        loop {
            if let Some(ListerEvent::Listed(dir, listing)) = lister.drain().into_iter().next() {
                assert_eq!(dir, tmp.path());
                assert_eq!(listing.dirs.len(), 1);
                break;
            }
            assert!(std::time::Instant::now() < deadline, "no answer");
            std::thread::yield_now();
        }
    }
}
