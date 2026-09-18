//! What is at the clone destination, found **off the render thread**.
//!
//! The dialog used to `read_dir` the destination on every frame; on a slow
//! or network drive that stalls the frame. Now one probe per (folder, URL)
//! runs on a thread, its answer is cached on the dialog, and an edit of
//! the folder or the URL starts a new one. The Clone press waits for the
//! answer (`CloneDialog::poll` carries it on) instead of guessing.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// What the destination folder holds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum DestKind {
    /// Not there (or not readable as a folder): git creates it.
    Missing,
    /// There and empty: git clones into it.
    Empty,
    /// A git repository whose remote is the URL being cloned.
    SameRepo,
    /// Anything else with files in it: git refuses. `suggestion` is the
    /// first free `<name>-2`, `-3`... beside it.
    NotEmpty { suggestion: String },
}

/// One probe: the question, and the answer once the thread has it.
#[derive(Debug, Clone)]
pub(crate) struct Probe {
    pub(crate) dest: PathBuf,
    pub(crate) url: String,
    slot: Arc<Mutex<Option<DestKind>>>,
    result: Option<DestKind>,
}

impl Probe {
    /// Starts the thread.
    pub(crate) fn start(dest: PathBuf, url: String) -> Self {
        let slot = Arc::new(Mutex::new(None));
        let (d, u, s) = (dest.clone(), url.clone(), slot.clone());
        std::thread::spawn(move || {
            let kind = inspect(&d, &u);
            if let Ok(mut guard) = s.lock() {
                *guard = Some(kind);
            }
            crate::redraw::poke();
        });
        Self {
            dest,
            url,
            slot,
            result: None,
        }
    }

    /// A probe whose answer is already known (the dialog just chose a
    /// free name itself).
    pub(crate) fn known(dest: PathBuf, url: String, kind: DestKind) -> Self {
        Self {
            dest,
            url,
            slot: Arc::new(Mutex::new(None)),
            result: Some(kind),
        }
    }

    pub(crate) fn is_for(&self, dest: &Path, url: &str) -> bool {
        self.dest == dest && self.url == url
    }

    /// The answer, if it came. `true` when it arrived with this call.
    pub(crate) fn resolve(&mut self) -> bool {
        if self.result.is_some() {
            return false;
        }
        let taken = self.slot.lock().ok().and_then(|mut g| g.take());
        self.result = taken;
        self.result.is_some()
    }

    pub(crate) fn kind(&self) -> Option<&DestKind> {
        self.result.as_ref()
    }
}

/// Reads the disk: blocking, so only ever called from the probe thread.
pub(crate) fn inspect(dest: &Path, url: &str) -> DestKind {
    let Ok(mut entries) = std::fs::read_dir(dest) else {
        return DestKind::Missing;
    };
    if entries.next().is_none() {
        return DestKind::Empty;
    }
    if remote_urls(dest).iter().any(|u| same_url(u, url)) {
        return DestKind::SameRepo;
    }
    let parent = dest.parent().unwrap_or(dest);
    let name = dest
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    DestKind::NotEmpty {
        suggestion: first_free_name(parent, &name),
    }
}

/// `name-2`, else `name-3`...: the first that does not exist in `parent`.
pub(crate) fn first_free_name(parent: &Path, name: &str) -> String {
    (2u32..)
        .map(|n| format!("{name}-{n}"))
        .find(|candidate| std::fs::symlink_metadata(parent.join(candidate)).is_err())
        .unwrap_or_else(|| format!("{name}-2"))
}

/// The `url = ...` lines of the repository's `.git/config`.
fn remote_urls(dest: &Path) -> Vec<String> {
    let Ok(text) = std::fs::read_to_string(dest.join(".git").join("config")) else {
        return Vec::new();
    };
    text.lines()
        .filter_map(|l| {
            let l = l.trim();
            let rest = l.strip_prefix("url")?.trim_start().strip_prefix('=')?;
            Some(rest.trim().to_string())
        })
        .collect()
}

/// The same remote spelled alike: trailing `/` and `.git`, case and
/// separators ignored.
pub(crate) fn same_url(a: &str, b: &str) -> bool {
    fn norm(u: &str) -> String {
        let u = u.trim().replace('\\', "/").to_lowercase();
        let u = u.trim_end_matches('/');
        u.strip_suffix(".git")
            .unwrap_or(u)
            .trim_end_matches('/')
            .to_string()
    }
    let (a, b) = (norm(a), norm(b));
    !a.is_empty() && a == b
}
