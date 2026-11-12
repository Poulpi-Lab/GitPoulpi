//! The "Edit in" image editors: the runner behind the image view's
//! "Edit in" combobox, the palette entries and the context menus.
//!
//! The list comes from `ui.image_editors` (see
//! `gitpoulpi_config::image_editors`); while that key is absent it is what the
//! machine has installed, found by a background lookup started by [`install`]
//! that reuses the `PATH` listing "Open in" uses. Nothing here waits for it:
//! until it is done the list holds only the OS default app.
//!
//! Neither showing an image nor launching an editor changes the repository, so
//! there is no journal entry and the Undo button is unaffected. An image that
//! is not the working-tree file is handed over as a read-only temp copy
//! ([`materialise`]), so an editor can not write into the repository.
//!
//! The image view's "Edit in" header combobox (`image_view::edit_in`), the
//! palette and the context menus call [`entries`], [`default_index`] and
//! [`run`].
// Without the `image-view` feature the header, the palette and the menus are
// stubs and nothing asks for the list.
#![cfg_attr(not(feature = "image-view"), allow(dead_code))]

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use gitpoulpi_config::image_editors::{self, Detected, ImageVars};
use gitpoulpi_config::open_in::Os;
use gitpoulpi_config::os_apps;
use gitpoulpi_config::{OpenInEntry, UiConfig};
use gitpoulpi_i18n::msg;

use crate::tab_dir::{self, RepoInfo, Warmable};

#[derive(Debug, Clone, Default)]
struct State {
    /// `ui.image_editors` when the file has it; `None` = computed.
    explicit: Option<Vec<OpenInEntry>>,
    default_label: String,
}

static STATE: Mutex<State> = Mutex::new(State {
    explicit: None,
    default_label: String::new(),
});
static DETECTED: Warmable<Detected> = Warmable::new();

fn detect_machine() -> Detected {
    let lookup = crate::repo_menu::path_lookup();
    image_editors::detect(&|k| std::env::var(k).ok(), Os::current(), &lookup, &|p| {
        Path::new(p).exists()
    })
}

/// Publishes the live settings and starts the background detection (no-op once
/// started). Never blocks.
pub(crate) fn install(ui: &UiConfig) {
    DETECTED.warm(detect_machine);
    let state = State {
        explicit: ui.image_editors.clone(),
        default_label: ui.image_editor_default.clone(),
    };
    if let Ok(mut s) = STATE.lock() {
        *s = state;
    }
}

fn state() -> State {
    STATE
        .lock()
        .map(|s| s.clone())
        .unwrap_or_else(|e| e.into_inner().clone())
}

/// What is known while the lookup runs: the OS default app, always there.
fn default_app_only(os: Os) -> Detected {
    let ids = image_editors::presets_for(os)
        .into_iter()
        .filter(|p| p.id.starts_with("default-"))
        .map(|p| p.id)
        .collect();
    Detected { ids }
}

fn computed(os: Os, detected: Option<&Detected>) -> Vec<OpenInEntry> {
    match detected {
        Some(d) => image_editors::effective_list(d, os),
        None => image_editors::effective_list(&default_app_only(os), os),
    }
}

/// The detection result, if the background lookup has finished. The Settings
/// page asks this one, so it never waits on the input thread.
pub(crate) fn detected_ready() -> Option<&'static Detected> {
    DETECTED.get()
}

/// The effective list for `ui` without waiting for detection.
pub(crate) fn effective_entries_ready(ui: &UiConfig) -> Vec<OpenInEntry> {
    match &ui.image_editors {
        Some(list) => list.clone(),
        None => computed(Os::current(), detected_ready()),
    }
}

fn current_entries() -> Vec<OpenInEntry> {
    match state().explicit {
        Some(list) => list,
        None => computed(Os::current(), detected_ready()),
    }
}

/// The editors for the file called `name`: the configured or detected ones,
/// then the applications the OS registers for its type that are not already in
/// the list (`open_with`, found in the background; none until it is done).
/// An empty `name` is "no file": the list alone.
fn current_entries_for(name: &str) -> Vec<OpenInEntry> {
    let own = current_entries();
    if name.is_empty() {
        return own;
    }
    let apps = crate::open_with::ready_apps(name);
    os_apps::merge_after(&own, &apps)
}

/// The labels of the editors, in order.
#[cfg(test)]
pub(crate) fn entries() -> Vec<String> {
    entries_for("")
}

/// [`entries`] for the file called `name`, OS applications included.
pub(crate) fn entries_for(name: &str) -> Vec<String> {
    current_entries_for(name)
        .into_iter()
        .map(|e| e.label)
        .collect()
}

/// The index of the default editor (`ui.image_editor_default`, else the first).
#[cfg(test)]
pub(crate) fn default_index() -> usize {
    default_index_for("")
}

/// [`default_index`] for the file called `name`; the OS applications come
/// after the configured ones, so the index of a configured default is stable.
pub(crate) fn default_index_for(name: &str) -> usize {
    image_editors::default_index(&current_entries_for(name), &state().default_label)
}

// ------------------------------------------------------------------ the runner

/// How `gitpoulpi_config::image_editors::expand` words an unknown variable.
const UNKNOWN_VARIABLE_PREFIX: &str = "unknown variable "; // i18n-english

/// What a launch needs, ready to spawn.
struct Prepared {
    label: String,
    argv: Vec<String>,
    /// The verbatim tail for a program that parses its own command line.
    raw_args: Option<String>,
    dir: PathBuf,
    info: RepoInfo,
}

fn prepare(entries: &[OpenInEntry], index: usize, file: &Path) -> Result<Prepared, String> {
    let entry = entries
        .get(index)
        .ok_or_else(|| msg::open_in_run::no_entry(index + 1))?;
    let label = entry.label.clone();
    let dir = file.parent().map(Path::to_path_buf).unwrap_or_default();
    // The facts are read from the `.git` files (microseconds, no git call);
    // a temp copy is in no repository and gets empty ones.
    let info = tab_dir::repo_info(&dir);
    let name = file
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    let (file_text, dir_text) = (
        file.to_string_lossy().to_string(),
        dir.to_string_lossy().to_string(),
    );
    let argv = image_editors::expand(
        &entry.command,
        &ImageVars {
            file: &file_text,
            dir: &dir_text,
            name: &name,
            repo: &info.repo,
        },
        &|k| std::env::var(k).ok(),
    )
    .map_err(|e| match e.strip_prefix(UNKNOWN_VARIABLE_PREFIX) {
        Some(var) => msg::open_in_run::unknown_variable(&label, var),
        None => msg::open_in_run::bad_command(&label, e),
    })?;
    let raw_args = os_apps::verbatim_tail(&entry.command, &file_text);
    Ok(Prepared {
        label,
        argv,
        raw_args,
        dir,
        info,
    })
}

/// The argv entry `index` runs for `file`, without spawning.
#[cfg(test)]
fn argv_for(entries: &[OpenInEntry], index: usize, file: &Path) -> Result<Vec<String>, String> {
    prepare(entries, index, file).map(|p| p.argv)
}

/// Runs editor `index` on `file` (absolute): the status line's answer at once
/// ("Opening in <label>..."). Errors known before spawning come back as `Err`;
/// an editor that fails within 1.5 s reports through
/// `external_tools::take_late_errors`. Never waits.
pub(crate) fn run(index: usize, file: &Path) -> Result<String, String> {
    run_list(&current_entries_for(&file_name_of(file)), index, file)
}

fn file_name_of(file: &Path) -> String {
    file.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default()
}

/// Runs entry `index` of `entries` on `file` (the same launch as [`run`]; the
/// binary view's "Open in" list goes through it).
pub(crate) fn run_list(
    entries: &[OpenInEntry],
    index: usize,
    file: &Path,
) -> Result<String, String> {
    run_with(entries, index, file, &|run| {
        tab_dir::start(
            &tab_dir::Launch {
                label: &run.label,
                program: &run.argv[0],
                args: &run.argv[1..],
                raw_args: run.raw_args.as_deref(),
                cwd: &run.dir,
                info: &run.info,
            },
            crate::external_tools::report_late_error,
        )
    })
}

/// [`run`] with the editor list and the spawner given (the tests record).
fn run_with(
    entries: &[OpenInEntry],
    index: usize,
    file: &Path,
    start: &dyn Fn(&Prepared) -> Result<(), String>,
) -> Result<String, String> {
    let run = prepare(entries, index, file)?;
    start(&run)?;
    Ok(msg::open_in_run::opening(&run.label))
}

// ------------------------------------------------------------------ temp copies

fn temp_root() -> PathBuf {
    std::env::temp_dir().join(format!("gitpoulpi-image-{}", std::process::id()))
}

/// A path component that cannot climb out of its folder.
fn safe_component(text: &str, fallback: &str) -> String {
    let cleaned: String = text
        .chars()
        .map(|c| {
            if c.is_control() || matches!(c, '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|') {
                '_'
            } else {
                c
            }
        })
        .collect();
    let cleaned = cleaned.trim_matches(['.', ' ']).to_string();
    if cleaned.is_empty() {
        fallback.to_string()
    } else {
        cleaned
    }
}

/// Writes `bytes` (already in memory: no extra daemon read) to
/// `temp/gitpoulpi-image-<pid>/<rev>/<name>` and sets the file read-only, so an
/// editor can not change what the repository holds. `rev` is a short sha or
/// `index`; `name` is the file's base name. A second call for the same
/// revision and name replaces the copy.
pub(crate) fn materialise(bytes: &[u8], rev: &str, name: &str) -> std::io::Result<PathBuf> {
    let base = Path::new(name)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    let dir = temp_root().join(safe_component(rev, "rev"));
    std::fs::create_dir_all(&dir)?;
    let path = dir.join(safe_component(&base, "image"));
    if let Ok(meta) = std::fs::metadata(&path) {
        // A read-only file can not be overwritten on Windows.
        let mut perms = meta.permissions();
        #[allow(clippy::permissions_set_readonly_false)]
        perms.set_readonly(false);
        let _ = std::fs::set_permissions(&path, perms);
        std::fs::remove_file(&path)?;
    }
    std::fs::write(&path, bytes)?;
    let mut perms = std::fs::metadata(&path)?.permissions();
    perms.set_readonly(true);
    std::fs::set_permissions(&path, perms)?;
    Ok(path)
}

/// Removes this process's temp copies, best effort (read-only files first made
/// writable, which Windows needs).
pub(crate) fn clean_temp() {
    fn unlock(dir: &Path) {
        let Ok(items) = std::fs::read_dir(dir) else {
            return;
        };
        for item in items.flatten() {
            let path = item.path();
            if path.is_dir() {
                unlock(&path);
            } else if let Ok(meta) = std::fs::metadata(&path) {
                let mut perms = meta.permissions();
                #[allow(clippy::permissions_set_readonly_false)]
                perms.set_readonly(false);
                let _ = std::fs::set_permissions(&path, perms);
            }
        }
    }
    let root = temp_root();
    unlock(&root);
    let _ = std::fs::remove_dir_all(root);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::external_tools::Launch;
    use std::sync::Arc;

    /// The editor statics are process-wide: tests that publish them take turns.
    static SERIAL: Mutex<()> = Mutex::new(());

    fn turn() -> std::sync::MutexGuard<'static, ()> {
        SERIAL.lock().unwrap_or_else(|e| e.into_inner())
    }

    fn ui_with(list: Option<Vec<OpenInEntry>>, default: &str) -> UiConfig {
        UiConfig {
            image_editors: list,
            image_editor_default: default.to_string(),
            ..UiConfig::default()
        }
    }

    /// Runs `index` on `file` with a recording spawner shaped like the
    /// `external_tools::set_spawn_hook` one: the launches it saw.
    fn recorded(
        entries: &[OpenInEntry],
        index: usize,
        file: &Path,
    ) -> (Result<String, String>, Vec<Launch>) {
        let seen = Arc::new(Mutex::new(Vec::<Launch>::new()));
        let sink = Arc::clone(&seen);
        let result = run_with(entries, index, file, &move |run| {
            sink.lock().unwrap().push(Launch {
                program: run.argv[0].clone(),
                args: run.argv[1..].to_vec(),
                raw_args: String::new(),
                cwd: Some(run.dir.clone()),
            });
            Ok(())
        });
        let launches = seen.lock().unwrap().clone();
        (result, launches)
    }

    fn gimp() -> Vec<OpenInEntry> {
        vec![
            OpenInEntry::new("GIMP", "gimp ${file}"),
            OpenInEntry::new("Tool", "tool --dir ${dir} --name ${name} ${file}"),
        ]
    }

    #[test]
    fn running_an_entry_launches_the_expanded_command_with_the_file_as_one_argument() {
        let file = Path::new("/tmp/my pics").join("a b.png");
        let (result, launches) = recorded(&gimp(), 0, &file);
        assert!(result.unwrap().contains("GIMP"));
        assert_eq!(launches.len(), 1);
        assert_eq!(launches[0].program, "gimp");
        assert_eq!(launches[0].args, vec![file.to_string_lossy().to_string()]);
        assert_eq!(launches[0].cwd.as_deref(), file.parent());

        let (_, launches) = recorded(&gimp(), 1, &file);
        assert_eq!(
            launches[0].args,
            vec![
                "--dir".to_string(),
                "/tmp/my pics".into(),
                "--name".into(),
                "a b.png".into(),
                file.to_string_lossy().to_string()
            ]
        );
    }

    #[test]
    fn a_bad_index_an_unknown_variable_and_a_bad_quote_fail_before_spawning() {
        let file = Path::new("/tmp/a.png");
        let (result, launches) = recorded(&gimp(), 9, file);
        assert!(result.is_err() && launches.is_empty());
        let bad = vec![OpenInEntry::new("Bad", "x ${cwd}")];
        let (result, launches) = recorded(&bad, 0, file);
        let why = result.unwrap_err();
        assert!(why.contains("Bad") && why.contains("cwd"), "{why}");
        assert!(launches.is_empty());
        let quote = vec![OpenInEntry::new("Q", "x 'oops ${file}")];
        assert!(argv_for(&quote, 0, file).is_err());
    }

    #[test]
    fn a_spawn_error_comes_back_to_the_caller() {
        let result = run_with(&gimp(), 0, Path::new("/tmp/a.png"), &|_| {
            Err("no such program".into())
        });
        assert_eq!(result.unwrap_err(), "no such program");
    }

    #[test]
    fn an_explicit_list_is_the_list_and_the_default_label_picks_the_index() {
        let _turn = turn();
        install(&ui_with(Some(gimp()), "Tool"));
        assert_eq!(entries(), vec!["GIMP", "Tool"]);
        assert_eq!(default_index(), 1);
        install(&ui_with(Some(gimp()), "gone"));
        assert_eq!(default_index(), 0, "a stale label falls back quietly");
        install(&ui_with(Some(vec![]), ""));
        assert!(
            entries().is_empty(),
            "Some(vec![]) is a deliberately empty list"
        );
    }

    #[test]
    fn an_absent_list_is_computed_and_never_empty_while_detection_runs() {
        let _turn = turn();
        install(&ui_with(None, ""));
        let list = entries();
        assert!(!list.is_empty(), "at least the OS default app");
        assert!(list.iter().any(|l| l == "Default app"));
        // The Settings view of it agrees on the explicit/computed split.
        assert_eq!(effective_entries_ready(&ui_with(Some(gimp()), "")), gimp());
        assert!(!effective_entries_ready(&ui_with(None, "")).is_empty());
    }

    #[test]
    fn the_os_apps_of_the_type_follow_the_configured_ones_without_duplicates() {
        let _turn = turn();
        crate::open_with::tests::install_fake();
        crate::open_with::tests::wait_ready("p.fakepng");
        // The user's own list wins: their `gimp` hides the OS's GIMP.
        install(&ui_with(Some(gimp()), "Tool"));
        assert_eq!(entries_for("p.fakepng"), vec!["GIMP", "Tool", "Fake Krita"]);
        assert_eq!(entries_for("p.other"), vec!["GIMP", "Tool"]);
        assert_eq!(entries_for(""), vec!["GIMP", "Tool"]);
        assert_eq!(default_index_for("p.fakepng"), 1, "the configured default");
        // An explicit empty list still gets the OS's apps for a file.
        install(&ui_with(Some(vec![]), ""));
        assert_eq!(
            entries_for("p.fakepng"),
            vec!["GNU Image Manipulation Program", "Fake Krita"]
        );
        assert!(entries_for("").is_empty());
    }

    #[test]
    fn the_until_detected_list_is_the_default_app_of_this_os() {
        for os in [Os::Linux, Os::Windows, Os::Macos] {
            let list = computed(os, None);
            assert_eq!(list.len(), 1, "{os:?}");
            assert_eq!(list[0].label, "Default app");
        }
    }

    #[test]
    fn a_temp_copy_holds_the_bytes_and_is_read_only() {
        let _turn = turn();
        let bytes = [137u8, 80, 78, 71, 1, 2, 3];
        let path = materialise(&bytes, "ab12cd3", "logo one.png").unwrap();
        assert!(path.starts_with(temp_root()));
        assert!(path.ends_with(Path::new("ab12cd3").join("logo one.png")));
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
        assert!(std::fs::metadata(&path).unwrap().permissions().readonly());
        assert!(
            std::fs::OpenOptions::new().write(true).open(&path).is_err() || cfg!(unix) && is_root(),
            "an editor can not write the copy"
        );
        // Again for the same revision and name: replaced, still read-only.
        let again = materialise(&[9, 9], "ab12cd3", "logo one.png").unwrap();
        assert_eq!(again, path);
        assert_eq!(std::fs::read(&again).unwrap(), [9, 9]);
        assert!(std::fs::metadata(&again).unwrap().permissions().readonly());
        // Another revision keeps its own copy.
        let other = materialise(&bytes, "index", "logo one.png").unwrap();
        assert_ne!(other, path);
        std::fs::remove_dir_all(path.parent().unwrap()).ok();
        std::fs::remove_dir_all(other.parent().unwrap()).ok();
    }

    #[cfg(unix)]
    fn is_root() -> bool {
        std::env::var("USER").is_ok_and(|u| u == "root")
    }
    #[cfg(not(unix))]
    fn is_root() -> bool {
        false
    }

    #[test]
    fn a_name_or_revision_can_not_climb_out_of_the_temp_folder() {
        let _turn = turn();
        let path = materialise(&[1], "../../etc", "../../evil.png").unwrap();
        assert!(path.starts_with(temp_root()), "{}", path.display());
        assert_eq!(path.file_name().unwrap(), "evil.png");
        std::fs::remove_dir_all(path.parent().unwrap()).ok();
        let weird = materialise(&[1], "", "").unwrap();
        assert!(weird.starts_with(temp_root()));
        std::fs::remove_dir_all(weird.parent().unwrap()).ok();
    }

    #[test]
    fn clean_temp_removes_read_only_copies() {
        let _turn = turn();
        let path = materialise(&[1, 2], "clean01", "x.png").unwrap();
        clean_temp();
        assert!(!path.exists());
    }
}
