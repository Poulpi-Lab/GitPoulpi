//! "Open in" for binary files: the applications the OS registers for a file's
//! type, offered where the Diff view and the File View only had a "binary
//! file" notice, from the file's context menu and from the palette.
//!
//! Which applications is the OS's answer ([`gitpoulpi_config::os_apps`]), found
//! on a background thread the first time an extension is asked about and kept
//! per extension: the input loop never waits for it (a registry query costs
//! hundreds of milliseconds on Windows). Until it is done the view shows
//! "looking up apps..." (after 150 ms) and the list has that one row.
//!
//! Neither showing the list nor launching an application changes the
//! repository: no journal entry, the Undo button is unaffected. The file handed
//! over is the working-tree file when the view shows the working tree; for a
//! commit's (or the index's) version it is a read-only temp copy
//! ([`crate::image_editor::materialise`]) written from the bytes the daemon
//! sends, so no application writes into the repository.
//!
//! Layout: this file is the lookup cache, the subject and the launch; `popup`
//! is the combobox list; `view` paints the button in the two views.

pub(crate) mod popup;
pub(crate) mod view;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

use gitpoulpi_config::os_apps::{self, Found, OsApp};
use gitpoulpi_core::domain::{BlobBody, Oid};
use gitpoulpi_i18n::msg::open_with as msg;

use crate::daemon_link::{
    DaemonLink, DiffContentSide, FileInsightKind, FileInsightOutcome, FileInsightPayload,
    FileInsightRequest,
};

/// How long a lookup runs before the view says so.
pub(crate) const LOOKUP_FEEDBACK_AFTER: Duration = Duration::from_millis(150);

// ------------------------------------------------------------------- the cache

/// The answer for one extension.
#[derive(Debug, Clone)]
pub(crate) enum Lookup {
    Pending { since: Instant },
    Ready(Arc<Found>),
}

/// What the lookups are keyed on: the extension, else the whole (lower-case)
/// file name (`Makefile`).
pub(crate) fn key_of(name: &str) -> String {
    let ext = os_apps::extension_of(name);
    if ext.is_empty() {
        name.rsplit(['/', '\\'])
            .next()
            .unwrap_or(name)
            .to_lowercase()
    } else {
        ext
    }
}

type Discover = fn(&str) -> Found;

/// Lookups by key; each is started once.
#[derive(Default)]
pub(crate) struct Cache {
    slots: Arc<Mutex<BTreeMap<String, Lookup>>>,
}

impl Cache {
    /// The answer for `name`'s type; the first ask starts `discover` on a
    /// thread and answers `Pending`. Never waits.
    pub(crate) fn lookup(&self, name: &str, discover: Discover) -> Lookup {
        let key = key_of(name);
        let Ok(mut slots) = self.slots.lock() else {
            return Lookup::Ready(Arc::new(Found::default()));
        };
        if let Some(found) = slots.get(&key) {
            return found.clone();
        }
        let pending = Lookup::Pending {
            since: Instant::now(),
        };
        slots.insert(key.clone(), pending.clone());
        // The name the discovery sees keeps the extension (a `Makefile` has none).
        let probe = if os_apps::extension_of(name).is_empty() {
            key.clone()
        } else {
            format!("file.{key}")
        };
        let shared = Arc::clone(&self.slots);
        std::thread::spawn(move || {
            let found = Arc::new(discover(&probe));
            if let Ok(mut slots) = shared.lock() {
                slots.insert(key, Lookup::Ready(found));
            }
            crate::redraw::poke();
        });
        pending
    }

    pub(crate) fn any_pending(&self) -> bool {
        self.slots
            .lock()
            .is_ok_and(|s| s.values().any(|l| matches!(l, Lookup::Pending { .. })))
    }
}

fn cache() -> &'static Cache {
    static CACHE: OnceLock<Cache> = OnceLock::new();
    CACHE.get_or_init(Cache::default)
}

/// What this machine's OS knows about `probe`'s type. Reads the environment
/// (`XDG_*`) and the file system (Linux) or the registry (Windows); only ever
/// called from a lookup thread.
fn discover_machine(probe: &str) -> Found {
    // Unit tests never read the real desktop: the fake machine, else nothing.
    #[cfg(test)]
    return match TEST_DISCOVER.lock().ok().and_then(|f| *f) {
        Some(fake) => fake(probe),
        None => Found::default(),
    };
    #[cfg(not(test))]
    discover_real(probe)
}

#[cfg(not(test))]
fn discover_real(probe: &str) -> Found {
    use gitpoulpi_config::open_in::Os;
    let env = |k: &str| std::env::var(k).ok();
    match Os::current() {
        Os::Linux => {
            let roots = os_apps::linux::XdgRoots::from_env(&env);
            os_apps::linux::apps_for(probe, &roots, &os_apps::linux::StdFiles)
        }
        Os::Windows => windows_apps(probe, &env),
        Os::Macos => os_apps::macos_apps(),
    }
}

#[cfg(windows)]
#[cfg_attr(test, allow(dead_code))]
fn windows_apps(probe: &str, env: &dyn Fn(&str) -> Option<String>) -> Found {
    os_apps::windows::apps_for(
        &os_apps::extension_of(probe),
        &reg_exe::RegExe::default(),
        env,
    )
}

#[cfg(not(windows))]
#[cfg_attr(test, allow(dead_code))]
fn windows_apps(_probe: &str, _env: &dyn Fn(&str) -> Option<String>) -> Found {
    Found::default()
}

#[cfg(windows)]
mod reg_exe;

/// The application the OS opens `name`'s type with, waiting for the lookup
/// when it is not cached yet: only ever called from a worker thread (the
/// Linux "Open file in default program", see `external_tools::open_default`).
pub(crate) fn default_app_blocking(name: &str) -> Option<OsApp> {
    let found = match ready(name) {
        Some(found) => found.apps.clone(),
        None => discover_machine(name).apps,
    };
    found.into_iter().find(|a| a.default)
}

/// Tests that need the module's own lookups replace the machine's answer.
#[cfg(test)]
pub(crate) static TEST_DISCOVER: Mutex<Option<Discover>> = Mutex::new(None);

/// The lookup for `name`'s type (started if it was not).
pub(crate) fn lookup(name: &str) -> Lookup {
    cache().lookup(name, discover_machine)
}

/// Starts the lookup for `name`'s type without caring about the answer yet
/// (the picture views do, for their "Edit in" list).
#[cfg_attr(not(feature = "image-view"), allow(dead_code))]
pub(crate) fn warm(name: &str) {
    let _ = lookup(name);
}

/// The finished answer, if there is one; starts the lookup otherwise.
pub(crate) fn ready(name: &str) -> Option<Arc<Found>> {
    match lookup(name) {
        Lookup::Ready(found) => Some(found),
        Lookup::Pending { .. } => None,
    }
}

/// The OS applications for `name`'s type once known; empty until then.
pub(crate) fn ready_apps(name: &str) -> Vec<OsApp> {
    ready(name).map(|f| f.apps.clone()).unwrap_or_default()
}

/// Whether a lookup is running (the loop keeps drawing so the "looking up"
/// feedback appears on time).
pub(crate) fn animating() -> bool {
    cache().any_pending()
}

/// Whether a file's context menu should offer "Open with...": the OS knows an
/// application for its type and it is not plain text, or the answer is still
/// on its way (the popup then says it is looking).
pub(crate) fn menu_offered(name: &str) -> bool {
    // Decided by the name alone, before and after the OS answers: a desktop
    // that lists apps for `.txt` (macOS) must not add the row late, which
    // would shift every row under a click aimed below it.
    if looks_textual(name) {
        return false;
    }
    match lookup(name) {
        Lookup::Pending { .. } => !looks_textual(name),
        Lookup::Ready(found) => !found.textual && !found.apps.is_empty(),
    }
}

/// Extensions that are text for sure; the menu does not wait for the OS to
/// say so (a source file's menu must not flash an "Open with" row).
pub(crate) fn looks_textual(name: &str) -> bool {
    const TEXT: &[&str] = &[
        "txt",
        "md",
        "markdown",
        "rst",
        "rs",
        "toml",
        "json",
        "yaml",
        "yml",
        "xml",
        "html",
        "htm",
        "css",
        "scss",
        "js",
        "mjs",
        "cjs",
        "ts",
        "tsx",
        "jsx",
        "py",
        "rb",
        "go",
        "java",
        "kt",
        "c",
        "h",
        "cc",
        "cpp",
        "hpp",
        "cs",
        "sh",
        "bash",
        "zsh",
        "ps1",
        "bat",
        "cmd",
        "sql",
        "ini",
        "cfg",
        "conf",
        "lock",
        "log",
        "csv",
        "tsv",
        "gitignore",
        "svg",
        "lua",
        "php",
        "swift",
        "vue",
        "svelte",
        "tex",
        "diff",
        "patch",
    ];
    let key = key_of(name);
    TEXT.contains(&key.as_str())
        || matches!(
            key.as_str(),
            "makefile" | "dockerfile" | "license" | "readme"
        )
}

// ------------------------------------------------------------------ the subject

/// Which version of the file to hand over.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Source {
    WorkingTree,
    Index,
    Commit(Oid),
}

/// The file an "Open in" acts on: a path in the repository and a version.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Subject {
    /// Relative to the repository root.
    pub(crate) path: PathBuf,
    pub(crate) source: Source,
}

impl Subject {
    pub(crate) fn name(&self) -> String {
        self.path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default()
    }

    /// The short label a copy's folder and notice carry.
    fn rev(&self) -> String {
        match self.source {
            Source::Commit(oid) => gitpoulpi_core::domain::short(oid),
            Source::WorkingTree | Source::Index => "index".to_string(),
        }
    }
}

// --------------------------------------------------------------------- launching

/// A read of a version's bytes sent to the daemon, waiting for the answer.
struct Awaiting {
    request: FileInsightRequest,
    apps: Vec<OsApp>,
    index: usize,
    subject: Subject,
}

static AWAITING: Mutex<Option<Awaiting>> = Mutex::new(None);
/// What a finished read wants the status line to say.
static NOTICE: Mutex<Option<String>> = Mutex::new(None);

/// The status line's word from a read that finished off the input path.
pub(crate) fn take_notice() -> Option<String> {
    NOTICE.lock().ok().and_then(|mut n| n.take())
}

fn set_notice(text: String) {
    if let Ok(mut n) = NOTICE.lock() {
        *n = Some(text);
    }
    crate::redraw::poke();
}

/// Runs application `index` of `apps` on the subject: the status line's answer.
/// The working-tree file is handed over as it is; another version is read from
/// the daemon first and handed over as a read-only copy when the bytes arrive
/// ([`offer`]).
pub(crate) fn run(
    apps: &[OsApp],
    index: usize,
    subject: &Subject,
    repo_root: &Path,
    link: &DaemonLink,
) -> String {
    run_with(apps, index, subject, repo_root, &|r| {
        link.request_file_insight(r)
    })
}

/// [`run`] with the daemon read given (the tests record it).
fn run_with(
    apps: &[OsApp],
    index: usize,
    subject: &Subject,
    repo_root: &Path,
    ask: &dyn Fn(FileInsightRequest),
) -> String {
    let name = subject.name();
    if apps.get(index).is_none() {
        return msg::no_app(&name);
    }
    let (side, at) = match subject.source {
        Source::WorkingTree => {
            let file = repo_root.join(&subject.path);
            if !file.exists() {
                return msg::gone(&name);
            }
            return report(launch(apps, index, &file));
        }
        Source::Index => (DiffContentSide::Index, Oid([0u8; 20])),
        Source::Commit(oid) => (DiffContentSide::Commit, oid),
    };
    let request = FileInsightRequest {
        path: subject.path.clone(),
        at,
        kind: FileInsightKind::ImageBytes(side),
        worktree: false,
        staged: false,
    };
    if let Ok(mut waiting) = AWAITING.lock() {
        *waiting = Some(Awaiting {
            request: request.clone(),
            apps: apps.to_vec(),
            index,
            subject: subject.clone(),
        });
    }
    ask(request);
    msg::reading(&name)
}

fn launch(apps: &[OsApp], index: usize, file: &Path) -> Result<String, String> {
    let entries: Vec<_> = apps.iter().map(OsApp::entry).collect();
    crate::image_editor::run_list(&entries, index, file)
}

/// A launch's status line; a failure also goes to the failure card, like the
/// other launches.
fn report(result: Result<String, String>) -> String {
    match result {
        Ok(opening) => opening,
        Err(why) => crate::activity_log::report_problem_clear(
            crate::activity_log::ProblemSource::Tool,
            &why,
        ),
    }
}

/// Takes the answer of the read [`run`] sent; anything else is handed back.
pub(crate) fn offer(outcome: FileInsightOutcome) -> Option<FileInsightOutcome> {
    let Ok(mut waiting) = AWAITING.lock() else {
        return Some(outcome);
    };
    if waiting
        .as_ref()
        .map_or(true, |w| w.request != outcome.request)
    {
        return Some(outcome);
    }
    let Some(wait) = waiting.take() else {
        return Some(outcome);
    };
    drop(waiting);
    let name = wait.subject.name();
    let bytes = match outcome.result {
        Ok(FileInsightPayload::ImageBytes(bytes)) => bytes,
        Ok(_) => return None,
        Err(e) => {
            set_notice(msg::read_failed(&name, e));
            return None;
        }
    };
    let data = match bytes.body {
        BlobBody::Bytes(b) => b,
        BlobBody::Lfs {
            object: Some(b), ..
        } => b,
        BlobBody::TooLarge | BlobBody::Lfs { object: None, .. } => {
            set_notice(msg::too_large(
                &name,
                gitpoulpi_core::domain::human_size(bytes.size),
            ));
            return None;
        }
    };
    let rev = wait.subject.rev();
    match crate::image_editor::materialise(&data, &rev, &name) {
        Ok(copy) => {
            set_notice(match launch(&wait.apps, wait.index, &copy) {
                Ok(_) => gitpoulpi_i18n::msg::image_edit_in::copy_notice(&name, &rev),
                Err(why) => report(Err(why)),
            });
        }
        Err(e) => set_notice(gitpoulpi_i18n::msg::image_edit_in::copy_failed(e)),
    }
    None
}

#[cfg(test)]
pub(crate) mod tests;
