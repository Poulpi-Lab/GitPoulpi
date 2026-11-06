//! The menu a repository answers with: right-click a repository tab
//! (the tab menu), or the `⋮` / right-click on a row of the
//! Repository Management tab. One action vocabulary for both, because
//! they are the same questions about the same thing -- a repository --
//! asked from two places.
//!
//! The decisions here are pure (which rows, which tabs a "close others"
//! closes); the only effects are [`open_in_file_manager`] and
//! [`open_in_terminal`], which hand a path to another program and never
//! wait for it.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::context_menu::Entry;
use gitpoulpi_i18n::msg::menu_repo as msg;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RepoMenuAction {
    /// Focus its tab, or open it as one.
    Open,
    Close,
    CloseOthers,
    CloseToTheRight,
    ToggleFavorite,
    RemoveFromRecent,
    /// Forget the repository in every stored list (recent, favorites).
    RemoveFromList,
    CopyPath,
    OpenInFileManager,
    OpenInTerminal,
    /// Entry `n` of the "Open in" list (`tab_dir::entries()`), run on this
    /// repository: the "Open in" submenu's rows.
    OpenIn(usize),
    /// Show this repository's `core.sshCommand` pin and choose an
    /// integration profile's SSH key for it (ADR-0008).
    SshKey,
    /// The Sparse checkout dialog of this repository.
    Sparse,
    /// A transient "New tab" (the tab bar's `+`); the tab menu's first row.
    NewTab,
    /// The `+` menu's Radar row, offered while its tab is closed.
    OpenRadar,
    /// The `+` menu's Workspaces row, offered while its tab is closed.
    OpenWorkspaces,
    /// Add this repository to the active workspace.
    AddToWorkspace,
    /// Take this repository out of the active workspace (it stays a tab).
    RemoveFromWorkspace,
    /// A row of a workspace's (or a workspace repository's) menu on the Home
    /// tab's Workspaces page.
    Ws(WsMenu),
}

/// The rows of the Workspaces page's menus (`workspace_ui::menu`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WsMenu {
    Open,
    Edit,
    Delete,
    FetchAll,
    PullAll,
    CloneMissing,
    OpenRepo,
    Locate,
    CloneRepo,
    RemoveRepo,
}

/// Where a menu was opened from: which tab, if it is one, and what the
/// repository is to the lists.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RepoMenuContext {
    /// Its index in the tab bar, when it is open as a tab.
    pub tab: Option<usize>,
    pub tab_count: usize,
    pub favorite: bool,
    pub recent: bool,
    /// Opened from the tab bar (the tab menu) rather than from a
    /// management row. The tab menu leads with the close group, as
    /// the tab menu does; a row leads with Open.
    pub from_tab_bar: bool,
    /// A workspace is active: `Some(true)` when this repository is in it.
    pub workspace: Option<bool>,
}

pub(crate) fn favorite_label(favorite: bool) -> &'static str {
    if favorite {
        msg::REMOVE_FAVORITE.get()
    } else {
        msg::ADD_FAVORITE.get()
    }
}

/// The tab bar's `+` menu: what a click on the `+` offers.
pub(crate) fn new_menu_entries(
    radar_closed: bool,
    workspaces_closed: bool,
) -> Vec<Entry<RepoMenuAction>> {
    let mut entries = vec![Entry::action(
        gitpoulpi_i18n::msg::tab_bar::PLUS_NEW_TAB.get(),
        RepoMenuAction::NewTab,
    )
    .with_key(crate::keymap::Action::GlobalNewTab)];
    if radar_closed {
        entries.push(
            Entry::action(
                gitpoulpi_i18n::msg::tab_bar::PLUS_RADAR.get(),
                RepoMenuAction::OpenRadar,
            )
            .with_key(crate::keymap::Action::GlobalRadar),
        );
    }
    if workspaces_closed {
        entries.push(
            Entry::action(
                gitpoulpi_i18n::msg::workspaces::PAGE_WORKSPACES.get(),
                RepoMenuAction::OpenWorkspaces,
            )
            .with_key(crate::keymap::Action::GlobalWorkspaces),
        );
    }
    entries
}

#[cfg(test)]
mod radar_row_tests {
    use super::*;

    fn has_radar(closed: bool) -> bool {
        new_menu_entries(closed, false).iter().any(|e| {
            matches!(
                e,
                Entry::Action {
                    action: RepoMenuAction::OpenRadar,
                    ..
                }
            )
        })
    }

    fn has_workspaces(closed: bool) -> bool {
        new_menu_entries(false, closed).iter().any(|e| {
            matches!(
                e,
                Entry::Action {
                    action: RepoMenuAction::OpenWorkspaces,
                    ..
                }
            )
        })
    }

    /// The `+` menu offers Workspaces only while its tab is closed.
    #[test]
    fn the_plus_menu_offers_workspaces_only_when_the_tab_is_closed() {
        assert!(has_workspaces(true));
        assert!(!has_workspaces(false));
    }

    /// Every combination: each row follows its own flag only.
    #[test]
    fn the_plus_menu_rows_are_independent() {
        for (radar, ws) in [(false, false), (true, false), (false, true), (true, true)] {
            let entries = new_menu_entries(radar, ws);
            let has = |a: RepoMenuAction| {
                entries
                    .iter()
                    .any(|e| matches!(e, Entry::Action { action, .. } if *action == a))
            };
            assert_eq!(has(RepoMenuAction::OpenRadar), radar);
            assert_eq!(has(RepoMenuAction::OpenWorkspaces), ws);
            assert!(has(RepoMenuAction::NewTab));
        }
    }

    /// Every `+` menu row that has a global binding shows it (Radar too).
    #[test]
    fn the_plus_menu_rows_carry_their_key_hints() {
        use crate::keymap::Action;
        let want = |a: RepoMenuAction| match a {
            RepoMenuAction::NewTab => Some(Action::GlobalNewTab),
            RepoMenuAction::OpenRadar => Some(Action::GlobalRadar),
            RepoMenuAction::OpenWorkspaces => Some(Action::GlobalWorkspaces),
            _ => None,
        };
        for e in new_menu_entries(true, true) {
            if let Entry::Action { action, key, .. } = e {
                assert_eq!(key, want(action), "{action:?}");
            }
        }
    }

    /// The `+` menu offers the Radar only while its tab is closed.
    #[test]
    fn the_plus_menu_offers_the_radar_only_when_it_is_closed() {
        assert!(has_radar(true));
        assert!(!has_radar(false));
    }
}

pub(crate) fn repo_menu_entries(ctx: RepoMenuContext) -> Vec<Entry<RepoMenuAction>> {
    repo_menu_entries_with(ctx, &crate::tab_dir::entries())
}

/// [`repo_menu_entries`] with the "Open in" labels given: the submenu that
/// replaced the two "Open in file manager / terminal" rows lists them, and is
/// left out when there are none (the palette keeps the two old actions).
pub(crate) fn repo_menu_entries_with(
    ctx: RepoMenuContext,
    open_in: &[String],
) -> Vec<Entry<RepoMenuAction>> {
    use RepoMenuAction as A;
    let mut entries = Vec::new();
    if ctx.from_tab_bar {
        entries.push(
            Entry::action(msg::NEW_TAB.get(), A::NewTab)
                .with_key(crate::keymap::Action::GlobalNewTab),
        );
        entries.push(Entry::Separator);
    }
    if !ctx.from_tab_bar {
        entries.push(Entry::action(
            if ctx.tab.is_some() {
                msg::SWITCH_TO_TAB.get()
            } else {
                msg::OPEN.get()
            },
            A::Open,
        ));
        entries.push(Entry::Separator);
    }
    if let Some(tab) = ctx.tab {
        entries.push(
            Entry::action(msg::CLOSE_TAB.get(), A::Close)
                .with_key(crate::keymap::Action::GlobalCloseTab),
        );
        if ctx.tab_count > 1 {
            entries.push(Entry::action(msg::CLOSE_OTHERS.get(), A::CloseOthers));
        }
        if tab + 1 < ctx.tab_count {
            entries.push(Entry::action(msg::CLOSE_RIGHT.get(), A::CloseToTheRight));
        }
        entries.push(Entry::Separator);
    }
    entries.push(Entry::action(
        favorite_label(ctx.favorite),
        A::ToggleFavorite,
    ));
    if ctx.recent && !ctx.from_tab_bar {
        entries.push(Entry::action(msg::REMOVE_RECENT.get(), A::RemoveFromRecent));
    }
    if !ctx.from_tab_bar {
        entries.push(Entry::action(msg::REMOVE_LIST.get(), A::RemoveFromList));
    }
    match ctx.workspace {
        Some(false) => entries.push(Entry::action(
            gitpoulpi_i18n::msg::workspaces::MENU_ADD_TO.get(),
            A::AddToWorkspace,
        )),
        Some(true) => entries.push(Entry::action(
            gitpoulpi_i18n::msg::workspaces::MENU_REMOVE_FROM.get(),
            A::RemoveFromWorkspace,
        )),
        None => {}
    }
    entries.push(Entry::Separator);
    entries.push(Entry::action(msg::COPY_PATH.get(), A::CopyPath));
    if !open_in.is_empty() {
        entries.push(Entry::submenu(
            gitpoulpi_i18n::msg::open_in_bar::BUTTON.get(),
            open_in
                .iter()
                .enumerate()
                .map(|(i, label)| Entry::action(label.clone(), A::OpenIn(i)))
                .collect(),
        ));
    }
    entries.push(Entry::Separator);
    entries.push(Entry::action(msg::SSH_KEY.get(), A::SshKey));
    entries.push(Entry::action(
        gitpoulpi_i18n::msg::sparse::MENU.get(),
        A::Sparse,
    ));
    entries
}

/// The tab indices a close action closes, given the tab it was asked
/// about. Pure, so "close others" and "close to the right" are tested
/// without a tab bar.
pub(crate) fn tabs_to_close(action: RepoMenuAction, index: usize, count: usize) -> Vec<usize> {
    match action {
        RepoMenuAction::Close if index < count => vec![index],
        RepoMenuAction::CloseOthers if index < count => {
            (0..count).filter(|&i| i != index).collect()
        }
        RepoMenuAction::CloseToTheRight => (index + 1..count).collect(),
        _ => Vec::new(),
    }
}

/// The confirm modal's question for closing several tabs at once. Names
/// how many, and says the repositories are untouched - the same promise
/// the single-tab prompt makes.
pub(crate) fn close_tabs_prompt(names: &[String]) -> String {
    let listed = match names {
        [] => String::new(),
        [one] => msg::name_one(one),
        [a, b] => msg::names_two(a, b),
        [a, b, rest @ ..] => msg::names_more(rest.len() as u64, a, b),
    };
    msg::close_prompt(names.len() as u64, listed)
}

/// The platform's file manager as the "open in" row and its status line
/// word it (two messages per platform: grammar differs by language).
#[derive(Clone, Copy)]
enum FileManager {
    Explorer,
    Finder,
    Other,
}

impl FileManager {
    fn opened(self, path: &Path) -> String {
        let path = path.display();
        match self {
            FileManager::Explorer => msg::opened_in_explorer(path),
            FileManager::Finder => msg::opened_in_finder(path),
            FileManager::Other => msg::opened_in_file_manager(path),
        }
    }
}

fn file_manager_name() -> FileManager {
    if cfg!(windows) {
        FileManager::Explorer
    } else if cfg!(target_os = "macos") {
        FileManager::Finder
    } else {
        FileManager::Other
    }
}

/// The program and arguments that show `path` in the platform's file
/// manager: `explorer` on Windows, `open` on macOS, `xdg-open` elsewhere.
pub(crate) fn file_manager_command(path: &Path) -> (String, Vec<String>) {
    let arg = path.display().to_string();
    if cfg!(windows) {
        ("explorer".into(), vec![arg])
    } else if cfg!(target_os = "macos") {
        ("open".into(), vec![arg])
    } else {
        ("xdg-open".into(), vec![arg])
    }
}

/// The program and arguments that open a terminal in `path`, best
/// effort per platform:
///
/// - **Windows**: Windows Terminal (`wt -d <path>`) when `wt` is on
///   `PATH`, else a new console window running `cmd`, started with `path`
///   as its working directory (no arguments, see
///   `tab_dir::wants_new_console`).
/// - **macOS**: `open -a Terminal <path>`.
/// - **Linux and the rest**: `$TERMINAL` when set, else
///   `xdg-terminal-exec` when installed, else `x-terminal-emulator`
///   (Debian's alternative), started with `path` as its working directory --
///   which every terminal emulator inherits.
///
/// This is the `auto` of `ui.terminal_launch` when `ui.launch` is off; a
/// preset or custom template (see `crate::tab_dir`) goes before it.
pub(crate) fn terminal_command(
    path: &Path,
    on_path: impl Fn(&str) -> bool,
    env_terminal: Option<String>,
) -> (String, Vec<String>) {
    let dir = path.display().to_string();
    if cfg!(windows) {
        if on_path("wt") {
            ("wt".into(), vec!["-d".into(), dir])
        } else {
            // No arguments: the launcher starts it in `path` with its own
            // console window, so the folder name never reaches a cmd line.
            ("cmd".into(), Vec::new())
        }
    } else if cfg!(target_os = "macos") {
        ("open".into(), vec!["-a".into(), "Terminal".into(), dir])
    } else {
        match env_terminal.filter(|t| !t.trim().is_empty()) {
            Some(term) => (term, Vec::new()),
            None if on_path("xdg-terminal-exec") => ("xdg-terminal-exec".into(), Vec::new()),
            None => ("x-terminal-emulator".into(), Vec::new()),
        }
    }
}

/// Whether `program` can be found on `PATH` (with Windows' executable
/// extensions). A filesystem walk, no process spawned.
pub(crate) fn on_path(program: &str) -> bool {
    let Some(paths) = std::env::var_os("PATH") else {
        return false;
    };
    let exts = path_exts();
    std::env::split_paths(&paths).any(|dir| {
        exts.iter()
            .any(|ext| dir.join(format!("{program}{ext}")).is_file())
    })
}

/// `on_path` for many names at once: each `PATH` directory is listed once,
/// then every question is a set lookup. Probing name by name costs
/// programs x directories x `PATHEXT` file checks - tens of thousands on a
/// Windows `PATH`, which took seconds on the input thread before the first
/// frame. A snapshot: a program installed after the call is not seen.
pub(crate) fn path_lookup() -> impl Fn(&str) -> bool {
    // NTFS and APFS compare names without case, like the `is_file` probes do.
    let fold_case = !cfg!(unix) || cfg!(target_os = "macos");
    path_lookup_in(std::env::var_os("PATH"), path_exts(), fold_case)
}

fn path_lookup_in(
    path_var: Option<std::ffi::OsString>,
    exts: Vec<String>,
    fold_case: bool,
) -> impl Fn(&str) -> bool {
    let fold = move |name: &str| {
        if fold_case {
            name.to_lowercase()
        } else {
            name.to_string()
        }
    };
    let mut files = std::collections::HashSet::new();
    for dir in path_var.iter().flat_map(std::env::split_paths) {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let Ok(kind) = entry.file_type() else {
                continue;
            };
            // `is_file` follows a link; the listing does not.
            if kind.is_file() || (kind.is_symlink() && entry.path().is_file()) {
                if let Some(name) = entry.file_name().to_str() {
                    files.insert(fold(name));
                }
            }
        }
    }
    move |program: &str| {
        exts.iter()
            .any(|ext| files.contains(&fold(&format!("{program}{ext}"))))
    }
}

fn path_exts() -> Vec<String> {
    if cfg!(windows) {
        std::env::var("PATHEXT")
            .unwrap_or_else(|_| ".EXE;.CMD;.BAT".into())
            .split(';')
            .map(str::to_string)
            .collect()
    } else {
        vec![String::new()]
    }
}

/// Where `program` really is, for a spawn by full path. On Windows this is
/// the PATH + PATHEXT search `CreateProcess` does not do for `.cmd`/`.bat`
/// shims (`code`, `cursor`): `code` finds `code.cmd`. A name with a path
/// separator is taken relative to `cwd`. `windows` is a parameter so the
/// rules are testable on any OS; callers pass `cfg!(windows)`.
pub(crate) fn resolve_program(
    program: &str,
    path_var: Option<&std::ffi::OsStr>,
    pathext: Option<&str>,
    windows: bool,
    cwd: &Path,
) -> Option<PathBuf> {
    let mut names: Vec<String> = Vec::new();
    if windows {
        let exts: Vec<String> = pathext
            .unwrap_or(".COM;.EXE;.BAT;.CMD")
            .split(';')
            .map(str::trim)
            .filter(|e| !e.is_empty())
            .map(str::to_string)
            .collect();
        let lower = program.to_ascii_lowercase();
        if exts
            .iter()
            .any(|e| lower.ends_with(&e.to_ascii_lowercase()))
        {
            names.push(program.to_string());
        }
        names.extend(exts.iter().map(|e| format!("{program}{e}")));
    } else {
        names.push(program.to_string());
    }
    let has_separator = program.contains('/') || (windows && program.contains('\\'));
    if has_separator {
        return names.iter().find_map(|n| {
            let candidate = Path::new(n);
            let full = if candidate.is_absolute() {
                candidate.to_path_buf()
            } else {
                cwd.join(candidate)
            };
            full.is_file().then_some(full)
        });
    }
    let paths = path_var?;
    std::env::split_paths(paths).find_map(|dir| {
        names.iter().find_map(|n| {
            let full = dir.join(n);
            full.is_file().then_some(full)
        })
    })
}

/// Starts `program` detached: no stdin/stdout/stderr tied to this
/// terminal (which is in raw mode and belongs to the TUI), and never
/// waited on - the rule the whole input loop lives by.
fn spawn_detached(program: &str, args: &[String], cwd: &Path) -> Result<(), String> {
    Command::new(program)
        .args(args)
        .current_dir(cwd)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map(|_| ())
        .map_err(|err| msg::start_failed(program, err))
}

pub(crate) fn open_in_file_manager(path: &Path) -> Result<String, String> {
    if !path.exists() {
        return Err(msg::gone(path.display()));
    }
    let (program, args) = file_manager_command(path);
    spawn_detached(&program, &args, path)?;
    Ok(file_manager_name().opened(path))
}

pub(crate) fn open_in_terminal(path: &Path) -> Result<String, String> {
    crate::tab_dir::open_terminal(path)
}

/// The program and arguments that open `url` in the OS default browser:
/// `rundll32 url.dll,FileProtocolHandler <url>` on Windows (`cmd /C
/// start` breaks on a URL containing `&`, which a compare/merge-request
/// URL always does - it is a query string), `open` on macOS,
/// `xdg-open` elsewhere.
pub(crate) fn url_command(url: &str) -> (String, Vec<String>) {
    if cfg!(windows) {
        (
            "rundll32".into(),
            vec!["url.dll,FileProtocolHandler".into(), url.to_string()],
        )
    } else if cfg!(target_os = "macos") {
        ("open".into(), vec![url.to_string()])
    } else {
        ("xdg-open".into(), vec![url.to_string()])
    }
}

/// Opens `url` in the OS default browser, detached and never waited on --
/// Start a pull request's no-token fallback, and what a token-holding
/// flow opens on success too.
pub(crate) fn open_url(url: &str) -> Result<(), String> {
    // `GITPOULPI_OPEN_LOG=<file>`: a test stub for the browser -- append
    // the URL to that file instead of launching anything.
    if let Some(log) = std::env::var_os("GITPOULPI_OPEN_LOG").filter(|v| !v.is_empty()) {
        use std::io::Write as _;
        return std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(log)
            .and_then(|mut f| writeln!(f, "{url}"))
            .map_err(|e| e.to_string());
    }
    // `GITPOULPI_NO_BROWSER=1`: never launch one (the pty tests, headless
    // sessions); the caller still reports the URL on the status line.
    if std::env::var_os("GITPOULPI_NO_BROWSER").is_some_and(|v| !v.is_empty()) {
        return Ok(());
    }
    let (program, args) = url_command(url);
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    Command::new(program)
        .args(args)
        .current_dir(cwd)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map(|_| ())
        .map_err(msg::no_browser)
}

/// A path for the clipboard: as the user would type it.
pub(crate) fn path_text(path: &Path) -> String {
    PathBuf::from(path).display().to_string()
}

#[cfg(test)]
mod tests {

    #[test]
    fn path_lookup_lists_each_directory_once_and_answers_like_on_path() {
        let a = tempfile::tempdir().unwrap();
        let b = tempfile::tempdir().unwrap();
        std::fs::write(a.path().join("code.CMD"), "").unwrap();
        std::fs::write(b.path().join("nvim"), "").unwrap();
        std::fs::create_dir(b.path().join("zed")).unwrap();
        let path = std::env::join_paths([a.path(), b.path()]).unwrap();
        let exts = vec![String::new(), ".cmd".to_string()];

        let folded = path_lookup_in(Some(path.clone()), exts.clone(), true);
        assert!(folded("code"), "an extension from PATHEXT, any case");
        assert!(folded("nvim"), "a second directory");
        assert!(!folded("zed"), "a directory is not a program");
        assert!(!folded("vim"));

        let exact = path_lookup_in(Some(path), exts, false);
        assert!(!exact("code"), "case kept where the filesystem keeps it");
        assert!(exact("nvim"));
        assert!(!path_lookup_in(None, vec![String::new()], true)("nvim"));
    }
    use super::*;
    use crate::context_menu::Entry;

    fn labels(entries: &[Entry<RepoMenuAction>]) -> Vec<String> {
        entries
            .iter()
            .filter_map(|e| match e {
                Entry::Action { label, .. } => Some(label.clone()),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn resolve_program_applies_pathext_on_windows_rules() {
        let bin = tempfile::tempdir().unwrap();
        let other = tempfile::tempdir().unwrap();
        std::fs::write(bin.path().join("code.cmd"), "").unwrap();
        std::fs::write(bin.path().join("tool.exe"), "").unwrap();
        std::fs::write(other.path().join("tool"), "").unwrap();
        let path = std::env::join_paths([bin.path(), other.path()]).unwrap();
        let cwd = Path::new(".");
        // `code` is a .cmd shim: found through PATHEXT, case-insensitively named.
        let found = resolve_program("code", Some(&path), Some(".exe;.cmd"), true, cwd);
        assert_eq!(found, Some(bin.path().join("code.cmd")));
        // With an extension already: taken as is.
        let found = resolve_program("tool.exe", Some(&path), Some(".exe;.cmd"), true, cwd);
        assert_eq!(found, Some(bin.path().join("tool.exe")));
        // PATHEXT order decides, and an extensionless file is not a program.
        assert_eq!(
            resolve_program("tool", Some(&path), Some(".exe"), true, cwd),
            Some(bin.path().join("tool.exe"))
        );
        assert_eq!(
            resolve_program("tool", Some(&path), Some(".cmd"), true, cwd),
            None
        );
        // Default PATHEXT (upper case) when the variable is unset; the file
        // is named in upper case too so a case-sensitive filesystem agrees.
        std::fs::write(bin.path().join("shim.CMD"), "").unwrap();
        assert!(resolve_program("shim", Some(&path), None, true, cwd).is_some());
        assert!(resolve_program("missing", Some(&path), None, true, cwd).is_none());
        // Non-Windows rules: the exact name.
        assert_eq!(
            resolve_program("tool", Some(&path), None, false, cwd),
            Some(other.path().join("tool"))
        );
        assert!(resolve_program("code", Some(&path), None, false, cwd).is_none());
        // A separator means "this file", relative to the folder it runs in.
        let rel = resolve_program("./tool.exe", None, None, false, bin.path());
        assert_eq!(rel, Some(bin.path().join("./tool.exe")));
        let abs = bin.path().join("code");
        let abs = abs.to_string_lossy();
        assert_eq!(
            resolve_program(&abs, None, Some(".cmd"), true, cwd),
            Some(bin.path().join("code.cmd"))
        );
    }

    #[test]
    fn close_others_and_close_to_the_right_pick_the_right_tabs() {
        use RepoMenuAction as A;
        assert_eq!(tabs_to_close(A::Close, 1, 4), [1]);
        assert_eq!(tabs_to_close(A::CloseOthers, 1, 4), [0, 2, 3]);
        assert_eq!(tabs_to_close(A::CloseToTheRight, 1, 4), [2, 3]);
        assert!(tabs_to_close(A::CloseToTheRight, 3, 4).is_empty());
        assert!(tabs_to_close(A::CloseOthers, 0, 1).is_empty());
        assert!(tabs_to_close(A::Close, 9, 4).is_empty(), "a stale index");
        assert!(tabs_to_close(A::CopyPath, 1, 4).is_empty());
    }

    /// The tab menu: the close group, the favorite toggle, then
    /// the path group. A close entry that would close nothing is not
    /// offered.
    #[test]
    fn the_tab_menu_offers_only_the_closes_that_would_close_something() {
        let ctx = RepoMenuContext {
            tab: Some(1),
            tab_count: 3,
            favorite: false,
            recent: true,
            from_tab_bar: true,
            workspace: None,
        };
        assert_eq!(
            labels(&repo_menu_entries_with(ctx, &[])),
            [
                "New tab",
                "Close tab",
                "Close other tabs...",
                "Close tabs to the right...",
                "Add to favorites",
                "Copy path",
                "SSH key...",
                "Sparse checkout...",
            ]
        );
        // The "Open in" list is one submenu, between the path and SSH groups.
        let with = repo_menu_entries_with(ctx, &["Terminal".into(), "Editor".into()]);
        let Some(Entry::Submenu { label, items }) =
            with.iter().find(|e| matches!(e, Entry::Submenu { .. }))
        else {
            panic!("no Open in submenu");
        };
        assert_eq!(label, "Open in");
        assert_eq!(items.len(), 2);
        let last = RepoMenuContext {
            tab: Some(2),
            favorite: true,
            ..ctx
        };
        let shown = labels(&repo_menu_entries(last));
        assert!(!shown.iter().any(|l| l == "Close tabs to the right..."));
        assert!(shown.iter().any(|l| l == "Remove from favorites"));
        let only = RepoMenuContext {
            tab: Some(0),
            tab_count: 1,
            ..ctx
        };
        let shown = labels(&repo_menu_entries(only));
        assert!(!shown.iter().any(|l| l == "Close other tabs..."));
    }

    #[test]
    fn a_management_row_menu_leads_with_open_and_offers_remove_from_recent() {
        let ctx = RepoMenuContext {
            tab: None,
            tab_count: 2,
            favorite: false,
            recent: true,
            from_tab_bar: false,
            workspace: None,
        };
        let shown = labels(&repo_menu_entries(ctx));
        assert_eq!(shown[0], "Open");
        assert!(shown.iter().any(|l| l == "Remove from recent"));
        assert!(!shown.iter().any(|l| l.starts_with("Close")));
    }

    #[test]
    fn the_multi_close_prompt_names_the_tabs_and_promises_the_disk() {
        let prompt = close_tabs_prompt(&["a".into(), "b".into(), "c".into(), "d".into()]);
        assert!(
            prompt.starts_with("Close 4 tabs: 'a', 'b' and 2 more?"),
            "{prompt}"
        );
        assert!(prompt.contains("not touched on disk"));
        assert!(close_tabs_prompt(&["solo".into()]).starts_with("Close 1 tab: 'solo'?"));
    }

    #[test]
    fn the_terminal_ladder_prefers_windows_terminal_then_falls_back() {
        let dir = Path::new("/work/repo");
        let (with_wt, args) = terminal_command(dir, |p| p == "wt", Some("kitty".into()));
        let (without, without_args) = terminal_command(dir, |_| false, None);
        if cfg!(windows) {
            assert_eq!(with_wt, "wt");
            assert_eq!(args[0], "-d");
            assert_eq!(without, "cmd");
            assert!(
                without_args.is_empty(),
                "the folder is the cwd, never an argument"
            );
        } else if cfg!(target_os = "macos") {
            assert_eq!(with_wt, "open");
        } else {
            assert_eq!(with_wt, "kitty", "$TERMINAL wins");
            assert_eq!(without, "x-terminal-emulator");
        }
    }

    /// While a workspace is active the tab menu offers to add the repository
    /// to it, or to take it out; with none active, neither.
    #[test]
    fn the_tab_menu_offers_workspace_membership_only_while_a_workspace_is_active() {
        let base = RepoMenuContext {
            tab: Some(0),
            tab_count: 2,
            favorite: false,
            recent: false,
            from_tab_bar: true,
            workspace: None,
        };
        let has = |w: Option<bool>, label: &str| {
            labels(&repo_menu_entries(RepoMenuContext {
                workspace: w,
                ..base
            }))
            .iter()
            .any(|l| l == label)
        };
        assert!(has(Some(false), "Add to workspace"));
        assert!(!has(Some(false), "Remove from workspace"));
        assert!(has(Some(true), "Remove from workspace"));
        assert!(!has(None, "Add to workspace") && !has(None, "Remove from workspace"));
    }
}
