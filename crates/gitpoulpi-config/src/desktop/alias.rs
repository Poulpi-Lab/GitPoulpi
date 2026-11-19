//! The command alias (`gp` or any name): a launcher in a per-user `bin`
//! directory that starts `gitpoulpi` (Settings > Launcher,
//! `gitpoulpi alias ...`).
//!
//! | OS | What is written | How it is known to be ours |
//! |---|---|---|
//! | Linux, macOS | `~/.local/bin/<name>`, a symlink to `gitpoulpi` | the link's target is a file called `gitpoulpi` (or the older `gitpoulpi-tui`, then rewritten) |
//! | Windows | `%LOCALAPPDATA%\Programs\GitPoulpi\bin\<name>.cmd` | the file holds [`CMD_MARKER`] |
//!
//! The *add* variant (`gpa`: `gitpoulpi --add`, which opens the current folder
//! as a tab of the GitPoulpi that is already running) needs an argument, so it
//! cannot be a link. It is a two-line shim with the same marker idea: the
//! `.cmd` on Windows, and on Linux/macOS a `/bin/sh` script holding
//! [`UNIX_MARKER`] that `exec`s `gitpoulpi --add "$@"` (no extra process).
//!
//! A symlink cannot carry a marker, and a shim script would add a process
//! hop to every start, so on Unix the link itself is the proof: a file or a
//! link to anything else at that path is `Foreign` and never touched. The
//! directory is not on `PATH` by itself: [`super::path`] adds it. Every
//! directory comes through [`Env`], so tests never touch the real home.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use super::folder_menu::windows::{app_path_is_ours, app_path_ops, app_path_remove};
use super::registry::Registry;
use crate::open_in::Os;

/// Second line of every `.cmd` shim we write.
pub const CMD_MARKER: &str = "REM X-GitPoulpi=1";

/// The variable every `.cmd` shim exports: the command line of the `cmd.exe`
/// that runs it. Explorer's address bar runs a `.cmd` as `cmd.exe /c "..."`,
/// an interactive prompt has no `/c`, which is how the program tells them
/// apart (`gitpoulpi-tui`'s `relaunch::explorer_start`).
pub const CMDLINE_ENV: &str = "GITPOULPI_CMDCMDLINE";

/// Second line of the `/bin/sh` shim of the add variant on Linux and macOS.
pub const UNIX_MARKER: &str = "# X-GitPoulpi=1";

/// What an alias runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Kind {
    /// `gp`: GitPoulpi itself.
    #[default]
    Open,
    /// `gpa`: `gitpoulpi --add`, a tab in the GitPoulpi that is already open.
    Add,
}

impl Kind {
    pub fn as_str(self) -> &'static str {
        match self {
            Kind::Open => "open",
            Kind::Add => "add",
        }
    }
}

/// The program an alias starts: the one executable (the TUI is its default
/// mode).
const TUI_STEM: &str = "gitpoulpi";
/// The retired name aliases used to point at. Such an alias is still ours
/// (never `Foreign`) but stale: `create` rewrites it to [`TUI_STEM`].
pub const LEGACY_TUI_STEM: &str = "gitpoulpi-tui";
/// Every binary of ours starts with it, the TUI included.
const CLI_STEM: &str = "gitpoulpi";

/// Names PowerShell resolves before any file on `PATH`: an alias of one of
/// these would never run there.
const POWERSHELL_ALIASES: &[&str] = &[
    "gp", "gc", "gl", "gm", "gi", "gu", "gv", "ls", "cp", "mv", "rm", "ps", "cat", "diff", "sort",
    "sl", "sp", "sv", "man", "cd", "echo", "kill", "type",
];

/// The machine an alias is made on.
#[derive(Debug, Clone)]
pub struct Env {
    pub os: Os,
    /// Where aliases are written.
    pub bin_dir: PathBuf,
    /// The `gitpoulpi` executable an alias starts.
    pub exe: PathBuf,
    /// Newest-build mode ([`crate::dev_mode::launcher_newest`]): `create`
    /// points the alias at the newest build of `exe` as of creation (a `.cmd`
    /// shim or a symlink cannot choose at run time), and an alias naming the
    /// `dist` or `release` build of the same checkout reads `Installed`.
    pub newest: bool,
    /// The current `PATH`, to find names that already resolve.
    pub path_var: Option<OsString>,
    /// Windows `PATHEXT`.
    pub pathext: Option<String>,
}

impl Env {
    /// The real directories for `os`; `current_exe` is the running program
    /// (the CLI, the MCP server or the TUI), from which the TUI is found.
    pub fn discover(
        os: Os,
        env: &dyn Fn(&str) -> Option<String>,
        home: &Path,
        current_exe: &Path,
    ) -> Env {
        let var = |k: &str| env(k).filter(|v| !v.is_empty()).map(PathBuf::from);
        let bin_dir = match os {
            Os::Windows => var("LOCALAPPDATA")
                .unwrap_or_else(|| home.join("AppData/Local"))
                .join("Programs/GitPoulpi/bin"),
            Os::Linux | Os::Macos => home.join(".local/bin"),
        };
        Env {
            os,
            bin_dir,
            exe: tui_exe(current_exe),
            newest: false,
            path_var: env("PATH").map(OsString::from),
            pathext: env("PATHEXT"),
        }
    }

    /// Is `target` what this alias should start?
    fn is_current(&self, target: &Path) -> bool {
        target == self.exe || (self.newest && crate::dev_mode::same_build_family(target, &self.exe))
    }

    /// What `create` points at.
    fn wanted_exe(&self) -> PathBuf {
        if self.newest {
            crate::dev_mode::newest_build(&self.exe)
        } else {
            self.exe.clone()
        }
    }

    fn file(&self, name: &str) -> PathBuf {
        match self.os {
            Os::Windows => self.bin_dir.join(format!("{name}.cmd")),
            _ => self.bin_dir.join(name),
        }
    }
}

/// The retired `gitpoulpi-tui` spelling of `exe` when `exe` is the
/// `gitpoulpi` executable: what an item written by an older release names.
pub fn legacy_tui_exe(exe: &Path) -> Option<PathBuf> {
    // On the text, so a Windows path reads the same on any host.
    let text = exe.to_string_lossy();
    let cut = text.rfind(['/', '\\']).map_or(0, |i| i + 1);
    let (dir, file) = text.split_at(cut);
    let (stem, ext) = match file.rfind('.') {
        Some(i) => file.split_at(i),
        None => (file, ""),
    };
    stem.eq_ignore_ascii_case(TUI_STEM)
        .then(|| PathBuf::from(format!("{dir}{LEGACY_TUI_STEM}{ext}")))
}

/// The `gitpoulpi` executable next to (or being) `current`. A `gitpoulpi-tui`
/// or `gitpoulpi-mcp` stub running maps to the real `gitpoulpi` beside it.
pub fn tui_exe(current: &Path) -> PathBuf {
    let tui = format!("{TUI_STEM}{}", std::env::consts::EXE_SUFFIX);
    if current.file_name().is_some_and(|n| *n == *tui) {
        return current.to_path_buf();
    }
    let sibling = current.with_file_name(&tui);
    if sibling.is_file() {
        sibling
    } else {
        current.to_path_buf()
    }
}

/// Why a name cannot be an alias.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NameError {
    Empty,
    /// Holds whitespace.
    Space,
    /// Holds `/`, `\` or `:`.
    Separator,
    /// Another character a shell would treat specially.
    BadChar,
    /// Starts with `-` or `.`: it would read as an option or hide.
    BadStart,
    TooLong,
}

impl NameError {
    pub fn describe(self) -> &'static str {
        match self {
            NameError::Empty => "the alias name is empty",
            NameError::Space => "the alias name has a space",
            NameError::Separator => "the alias name has a path separator",
            NameError::BadChar => "the alias name may only hold letters, digits, - _ and .",
            NameError::BadStart => "the alias name cannot start with - or .",
            NameError::TooLong => "the alias name is longer than 64 characters",
        }
    }
}

/// Checks a name: letters, digits, `-`, `_` and `.` only.
pub fn validate_name(name: &str) -> Result<(), NameError> {
    if name.is_empty() {
        return Err(NameError::Empty);
    }
    if name.chars().any(char::is_whitespace) {
        return Err(NameError::Space);
    }
    if name.contains(['/', '\\', ':']) {
        return Err(NameError::Separator);
    }
    if !name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
    {
        return Err(NameError::BadChar);
    }
    if name.starts_with(['-', '.']) {
        return Err(NameError::BadStart);
    }
    if name.len() > 64 {
        return Err(NameError::TooLong);
    }
    Ok(())
}

/// What is at the alias path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    NotInstalled,
    /// Ours and starting the current exe.
    Installed,
    /// Ours, but the exe moved: Create fixes it.
    Outdated,
    /// A file of that name exists and is not ours; it is left alone.
    Foreign,
}

impl State {
    pub fn as_str(self) -> &'static str {
        match self {
            State::NotInstalled => "not_installed",
            State::Installed => "installed",
            State::Outdated => "outdated",
            State::Foreign => "foreign",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Status {
    pub state: State,
    /// Not installed here, but the name already resolves on `PATH` to this
    /// (a shell alias or function cannot be seen from here).
    pub shadowed_by: Option<PathBuf>,
    /// PowerShell resolves this name to a built-in alias first (Windows only).
    pub powershell_builtin: bool,
}

/// The `.cmd` text: the exe with the arguments passed on, and a marker line.
pub fn cmd_shim_text(exe: &Path) -> String {
    cmd_shim_text_for(exe, Kind::Open)
}

/// [`cmd_shim_text`] for either kind: the add variant passes `--add` first.
pub fn cmd_shim_text_for(exe: &Path, kind: Kind) -> String {
    let flag = match kind {
        Kind::Open => "",
        Kind::Add => " --add",
    };
    format!(
        "@echo off\r\n{CMD_MARKER}\r\nset \"{CMDLINE_ENV}=%CMDCMDLINE%\"\r\n\"{}\"{flag} %*\r\n",
        exe.display().to_string().replace('"', "")
    )
}

/// The `/bin/sh` text of the add variant: a marker and an `exec`.
pub fn sh_shim_text(exe: &Path) -> String {
    format!(
        "#!/bin/sh\n{UNIX_MARKER}\nexec {} --add \"$@\"\n",
        sh_quote(&exe.display().to_string())
    )
}

fn sh_quote(text: &str) -> String {
    format!("'{}'", text.replace('\'', "'\\''"))
}

/// The word `sh_quote` wrote at the start of `text`, and what follows it.
fn sh_unquote(text: &str) -> Option<(String, &str)> {
    let mut out = String::new();
    let mut rest = text;
    loop {
        rest = rest.strip_prefix('\'')?;
        let end = rest.find('\'')?;
        out.push_str(&rest[..end]);
        rest = &rest[end + 1..];
        match rest.strip_prefix("\\'") {
            Some(more) => {
                out.push('\'');
                rest = more;
            }
            None => return Some((out, rest)),
        }
    }
}

/// The exe and kind of a `.cmd` shim of ours.
fn shim_parts(text: &str) -> Option<(PathBuf, Kind)> {
    if !text.lines().any(|l| l.trim() == CMD_MARKER) {
        return None;
    }
    let line = text
        .lines()
        .find_map(|l| l.trim().strip_suffix("%*"))?
        .trim();
    let (exe, rest) = match line.strip_prefix('"') {
        Some(inner) => {
            let end = inner.find('"')?;
            (&inner[..end], inner[end + 1..].trim())
        }
        None => (line, ""),
    };
    let kind = if rest == "--add" {
        Kind::Add
    } else {
        Kind::Open
    };
    Some((PathBuf::from(exe), kind))
}

/// The exe a `.cmd` shim of ours starts.
#[cfg(test)]
fn shim_target(text: &str) -> Option<PathBuf> {
    shim_parts(text).map(|(exe, _)| exe)
}

/// The exe and kind of a `/bin/sh` shim of ours.
fn sh_parts(text: &str) -> Option<(PathBuf, Kind)> {
    if !text.lines().any(|l| l.trim() == UNIX_MARKER) {
        return None;
    }
    let line = text.lines().find_map(|l| l.trim().strip_prefix("exec "))?;
    let (exe, rest) = sh_unquote(line.trim())?;
    (rest.trim() == "--add \"$@\"").then(|| (PathBuf::from(exe), Kind::Add))
}

/// `Some((target, kind))` when `path` is an alias of ours.
fn ours(env: &Env, path: &Path) -> Option<(PathBuf, Kind)> {
    match env.os {
        Os::Windows => shim_parts(&std::fs::read_to_string(path).ok()?),
        _ => {
            if let Ok(target) = std::fs::read_link(path) {
                let name = target.file_name()?.to_string_lossy().into_owned();
                // The CLI itself when no TUI sits next to it (`exe_for`).
                return name.starts_with(CLI_STEM).then_some((target, Kind::Open));
            }
            // A regular file: only ever a small script of ours.
            let meta = std::fs::metadata(path).ok()?;
            if !meta.is_file() || meta.len() > 4096 {
                return None;
            }
            sh_parts(&std::fs::read_to_string(path).ok()?)
        }
    }
}

/// The first file `name` resolves to on `PATH`, or none.
fn resolve_on_path(env: &Env, name: &str) -> Option<PathBuf> {
    let exts: Vec<String> = if env.os == Os::Windows {
        let ext = env
            .pathext
            .clone()
            .unwrap_or_else(|| ".COM;.EXE;.BAT;.CMD".into());
        std::iter::once(String::new())
            .chain(ext.split(';').filter(|e| !e.is_empty()).map(String::from))
            .collect()
    } else {
        vec![String::new()]
    };
    std::env::split_paths(env.path_var.as_ref()?)
        .filter(|d| !d.as_os_str().is_empty())
        .flat_map(|d| exts.iter().map(move |e| d.join(format!("{name}{e}"))))
        .find(|p| p.is_file() || p.is_symlink())
}

/// Windows: a shim of ours whose text is not what this version writes (an
/// older one lacks the `GITPOULPI_CMDCMDLINE` line) is `Outdated`. Other
/// systems have no text to compare.
fn shim_text_current(env: &Env, path: &Path) -> bool {
    if env.os != Os::Windows {
        return true;
    }
    std::fs::read_to_string(path)
        .ok()
        .and_then(|t| shim_parts(&t).map(|(exe, kind)| t == cmd_shim_text_for(&exe, kind)))
        .unwrap_or(true)
}

pub fn status(env: &Env, name: &str) -> Status {
    status_kind(env, name, Kind::Open)
}

/// [`status`] for the alias `name` as it should be of `kind`: one of the other
/// kind (or pointing at another exe) is ours but `Outdated`, and Create fixes it.
pub fn status_kind(env: &Env, name: &str, kind: Kind) -> Status {
    let path = env.file(name);
    let state = if std::fs::symlink_metadata(&path).is_err() {
        State::NotInstalled
    } else {
        match ours(env, &path) {
            Some((target, k))
                if k == kind && env.is_current(&target) && shim_text_current(env, &path) =>
            {
                State::Installed
            }
            Some(_) => State::Outdated,
            None => State::Foreign,
        }
    };
    let shadowed_by = (state == State::NotInstalled)
        .then(|| resolve_on_path(env, name))
        .flatten();
    Status {
        state,
        shadowed_by,
        powershell_builtin: env.os == Os::Windows
            && POWERSHELL_ALIASES.contains(&name.to_ascii_lowercase().as_str()),
    }
}

/// Creates or rewrites the alias; returns the file written.
pub fn create(env: &Env, name: &str) -> Result<PathBuf, String> {
    create_kind(env, name, Kind::Open)
}

/// [`create`] for either kind.
pub fn create_kind(env: &Env, name: &str, kind: Kind) -> Result<PathBuf, String> {
    validate_name(name).map_err(|e| e.describe().to_string())?;
    let path = env.file(name);
    if std::fs::symlink_metadata(&path).is_ok() {
        if ours(env, &path).is_none() {
            return Err(format!("{} is not ours; not replacing it", path.display()));
        }
        std::fs::remove_file(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    }
    std::fs::create_dir_all(&env.bin_dir).map_err(|e| format!("{}: {e}", env.bin_dir.display()))?;
    let exe = env.wanted_exe();
    match (env.os, kind) {
        (Os::Windows, kind) => std::fs::write(&path, cmd_shim_text_for(&exe, kind))
            .map_err(|e| format!("{}: {e}", path.display()))?,
        (_, Kind::Open) => symlink(&exe, &path)?,
        (_, Kind::Add) => write_script(&path, &sh_shim_text(&exe))?,
    }
    Ok(path)
}

#[cfg(unix)]
fn write_script(path: &Path, text: &str) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::write(path, text).map_err(|e| format!("{}: {e}", path.display()))?;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755))
        .map_err(|e| format!("{}: {e}", path.display()))
}

#[cfg(not(unix))]
fn write_script(_path: &Path, _text: &str) -> Result<(), String> {
    Err("shell scripts are not available on this system".into())
}

#[cfg(unix)]
fn symlink(exe: &Path, path: &Path) -> Result<(), String> {
    std::os::unix::fs::symlink(exe, path).map_err(|e| format!("{}: {e}", path.display()))
}

#[cfg(not(unix))]
fn symlink(_exe: &Path, _path: &Path) -> Result<(), String> {
    Err("symbolic links are not available on this system".into())
}

/// Deletes the alias if it is ours. `Ok(false)`: there was none.
pub fn remove(env: &Env, name: &str) -> Result<bool, String> {
    validate_name(name).map_err(|e| e.describe().to_string())?;
    let path = env.file(name);
    if std::fs::symlink_metadata(&path).is_err() {
        return Ok(false);
    }
    if ours(env, &path).is_none() {
        return Err(format!("{} is not ours; not removing it", path.display()));
    }
    std::fs::remove_file(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(true)
}

/// The App Paths program name for the alias `name`: `gp.exe`.
pub fn app_path_name(name: &str) -> String {
    format!("{name}.exe")
}

/// [`create_kind`], and on Windows for the Open alias also App Paths
/// `<name>.exe` pointing at the console-less launcher beside the exe, so that
/// typing the alias in the File Explorer address bar finds it even where the
/// bin directory is not on `PATH`. No launcher beside the exe: no key, no
/// error. A key of that name that is not ours is left alone.
pub fn create_with_app_path(
    env: &Env,
    registry: &dyn Registry,
    name: &str,
    kind: Kind,
) -> Result<PathBuf, String> {
    create_with_app_path_in(env, registry, name, kind, &|p| p.is_file())
}

fn create_with_app_path_in(
    env: &Env,
    registry: &dyn Registry,
    name: &str,
    kind: Kind,
    exists: &dyn Fn(&Path) -> bool,
) -> Result<PathBuf, String> {
    let path = create_kind(env, name, kind)?;
    if env.os == Os::Windows && kind == Kind::Open {
        let key = app_path_name(name);
        let ours = app_path_is_ours(registry, &key)?;
        if let (Some(helper), true) = (
            super::terminals::folder_helper(env.os, &env.wanted_exe(), exists),
            ours != Some(false),
        ) {
            registry.apply(&app_path_ops(&key, &helper))?;
        }
    }
    Ok(path)
}

/// [`remove`], and on Windows the App Paths key of that name when it is ours.
pub fn remove_with_app_path(
    env: &Env,
    registry: &dyn Registry,
    name: &str,
) -> Result<bool, String> {
    let removed = remove(env, name)?;
    if removed && env.os == Os::Windows {
        let key = app_path_name(name);
        if app_path_is_ours(registry, &key)? == Some(true) {
            registry.apply(&[app_path_remove(&key)])?;
        }
    }
    Ok(removed)
}

/// Every alias of ours in the bin directory, by name.
pub fn list(env: &Env) -> Vec<(String, State)> {
    list_kinds(env)
        .into_iter()
        .map(|(name, _, state)| (name, state))
        .collect()
}

/// [`list`] with what each one runs.
pub fn list_kinds(env: &Env) -> Vec<(String, Kind, State)> {
    let Ok(read) = std::fs::read_dir(&env.bin_dir) else {
        return Vec::new();
    };
    let mut out: Vec<(String, Kind, State)> = read
        .flatten()
        .filter_map(|e| {
            let file = e.file_name().to_string_lossy().into_owned();
            let name = match env.os {
                Os::Windows => file.strip_suffix(".cmd")?.to_string(),
                _ => file,
            };
            let (target, kind) = ours(env, &e.path())?;
            let state = if env.is_current(&target) && shim_text_current(env, &e.path()) {
                State::Installed
            } else {
                State::Outdated
            };
            Some((name, kind, state))
        })
        .collect();
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::desktop::folder_menu::windows::app_path_key;

    fn env(root: &Path, os: Os) -> Env {
        Env {
            os,
            bin_dir: root.join("bin"),
            exe: PathBuf::from("/opt/gp/gitpoulpi"),
            newest: false,
            path_var: None,
            pathext: None,
        }
    }

    #[test]
    fn names_are_checked() {
        assert_eq!(validate_name("gp"), Ok(()));
        assert_eq!(validate_name("git-poulpi_2.x"), Ok(()));
        assert_eq!(validate_name(""), Err(NameError::Empty));
        assert_eq!(validate_name("g p"), Err(NameError::Space));
        assert_eq!(validate_name("a/b"), Err(NameError::Separator));
        assert_eq!(validate_name("a\\b"), Err(NameError::Separator));
        assert_eq!(validate_name("c:x"), Err(NameError::Separator));
        assert_eq!(validate_name("a;b"), Err(NameError::BadChar));
        assert_eq!(validate_name("-rf"), Err(NameError::BadStart));
        assert_eq!(validate_name(&"a".repeat(65)), Err(NameError::TooLong));
    }

    #[test]
    fn aliases_start_gitpoulpi_even_when_a_stub_runs() {
        let tmp = tempfile::tempdir().unwrap();
        let sfx = std::env::consts::EXE_SUFFIX;
        let cli = tmp.path().join(format!("gitpoulpi{sfx}"));
        let stub = tmp.path().join(format!("gitpoulpi-tui{sfx}"));
        assert_eq!(tui_exe(&stub), stub, "no sibling: the program itself");
        std::fs::write(&cli, "").unwrap();
        assert_eq!(tui_exe(&cli), cli);
        assert_eq!(tui_exe(&stub), cli, "the stub maps to the real program");
        let mcp = tmp.path().join(format!("gitpoulpi-mcp{sfx}"));
        assert_eq!(tui_exe(&mcp), cli);
    }

    #[test]
    fn the_retired_spelling_of_an_exe_is_known() {
        assert_eq!(
            legacy_tui_exe(Path::new("/o/gitpoulpi")),
            Some(PathBuf::from("/o/gitpoulpi-tui"))
        );
        assert_eq!(
            legacy_tui_exe(Path::new("C:/GP/gitpoulpi.exe")),
            Some(PathBuf::from("C:/GP/gitpoulpi-tui.exe"))
        );
        assert_eq!(legacy_tui_exe(Path::new("/o/gitpoulpi-tui")), None);
        assert_eq!(legacy_tui_exe(Path::new("/o/other")), None);
    }

    #[cfg(unix)]
    #[test]
    fn an_alias_to_the_retired_tui_is_ours_stale_and_rewritten() {
        let tmp = tempfile::tempdir().unwrap();
        let e = Env {
            exe: PathBuf::from("/opt/gp/gitpoulpi"),
            ..env(tmp.path(), Os::Linux)
        };
        std::fs::create_dir_all(&e.bin_dir).unwrap();
        let (open, add) = (e.bin_dir.join("gp"), e.bin_dir.join("gpa"));
        std::os::unix::fs::symlink("/opt/gp/gitpoulpi-tui", &open).unwrap();
        std::fs::write(
            &add,
            format!("#!/bin/sh\n{UNIX_MARKER}\nexec '/opt/gp/gitpoulpi-tui' --add \"$@\"\n"),
        )
        .unwrap();
        assert_eq!(status(&e, "gp").state, State::Outdated, "never Foreign");
        assert_eq!(
            status_kind(&e, "gpa", Kind::Add).state,
            State::Outdated,
            "never Foreign"
        );
        create(&e, "gp").unwrap();
        create_kind(&e, "gpa", Kind::Add).unwrap();
        assert_eq!(std::fs::read_link(&open).unwrap(), e.exe);
        assert_eq!(status(&e, "gp").state, State::Installed);
        assert_eq!(status_kind(&e, "gpa", Kind::Add).state, State::Installed);
        assert!(std::fs::read_to_string(&add)
            .unwrap()
            .contains("exec '/opt/gp/gitpoulpi' --add"));
    }

    #[test]
    fn a_cmd_shim_of_the_retired_tui_is_ours_stale_and_rewritten() {
        let tmp = tempfile::tempdir().unwrap();
        let mut e = env(tmp.path(), Os::Windows);
        e.exe = PathBuf::from("C:/GP/gitpoulpi.exe");
        std::fs::create_dir_all(&e.bin_dir).unwrap();
        let path = e.bin_dir.join("gp.cmd");
        std::fs::write(&path, cmd_shim_text(Path::new("C:/GP/gitpoulpi-tui.exe"))).unwrap();
        assert_eq!(status(&e, "gp").state, State::Outdated, "never Foreign");
        create(&e, "gp").unwrap();
        assert_eq!(status(&e, "gp").state, State::Installed);
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(text.contains("\"C:/GP/gitpoulpi.exe\""), "{text}");
        assert!(!text.contains("gitpoulpi-tui"), "{text}");
    }

    #[cfg(unix)]
    #[test]
    fn a_symlink_is_created_recognised_and_removed_only_when_ours() {
        let tmp = tempfile::tempdir().unwrap();
        let e = env(tmp.path(), Os::Linux);
        assert_eq!(status(&e, "gp").state, State::NotInstalled);
        let path = create(&e, "gp").unwrap();
        assert_eq!(std::fs::read_link(&path).unwrap(), e.exe);
        assert_eq!(status(&e, "gp").state, State::Installed);
        assert_eq!(list(&e), [("gp".to_string(), State::Installed)]);
        // The exe moved: outdated, and Create repoints it.
        let moved = Env {
            exe: PathBuf::from("/elsewhere/gitpoulpi-tui"),
            newest: false,
            ..e.clone()
        };
        assert_eq!(status(&moved, "gp").state, State::Outdated);
        create(&moved, "gp").unwrap();
        assert_eq!(std::fs::read_link(&path).unwrap(), moved.exe);
        assert!(remove(&moved, "gp").unwrap());
        assert!(!remove(&moved, "gp").unwrap(), "already gone");
        assert!(std::fs::symlink_metadata(&path).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn a_link_to_the_cli_is_ours_when_no_tui_sits_next_to_it() {
        let tmp = tempfile::tempdir().unwrap();
        let e = Env {
            exe: PathBuf::from("/opt/gp/gitpoulpi"),
            ..env(tmp.path(), Os::Linux)
        };
        create(&e, "gp").unwrap();
        assert_eq!(list(&e), [("gp".to_string(), State::Installed)]);
        assert!(remove(&e, "gp").unwrap());
    }

    #[cfg(unix)]
    #[test]
    fn a_file_or_a_link_that_is_not_ours_is_left_alone() {
        let tmp = tempfile::tempdir().unwrap();
        let e = env(tmp.path(), Os::Linux);
        std::fs::create_dir_all(&e.bin_dir).unwrap();
        let file = e.bin_dir.join("gp");
        std::fs::write(&file, "#!/bin/sh\n").unwrap();
        assert_eq!(status(&e, "gp").state, State::Foreign);
        assert!(create(&e, "gp").is_err());
        assert!(remove(&e, "gp").is_err());
        assert_eq!(std::fs::read_to_string(&file).unwrap(), "#!/bin/sh\n");
        std::fs::remove_file(&file).unwrap();
        std::os::unix::fs::symlink("/usr/bin/git", &file).unwrap();
        assert_eq!(status(&e, "gp").state, State::Foreign);
        assert!(remove(&e, "gp").is_err());
        assert!(std::fs::symlink_metadata(&file).is_ok());
        assert!(list(&e).is_empty());
    }

    #[cfg(unix)]
    #[test]
    fn a_name_already_on_path_is_reported_not_blocked() {
        let tmp = tempfile::tempdir().unwrap();
        let other = tmp.path().join("other");
        std::fs::create_dir_all(&other).unwrap();
        std::fs::write(other.join("gp"), "").unwrap();
        let e = Env {
            path_var: Some(other.clone().into_os_string()),
            ..env(tmp.path(), Os::Linux)
        };
        let st = status(&e, "gp");
        assert_eq!(st.shadowed_by, Some(other.join("gp")));
        assert!(!st.powershell_builtin);
        assert_eq!(status(&e, "zz").shadowed_by, None);
        create(&e, "gp").unwrap();
        assert_eq!(status(&e, "gp").shadowed_by, None, "ours now");
    }

    #[test]
    fn the_windows_shim_carries_the_marker_and_round_trips() {
        let tmp = tempfile::tempdir().unwrap();
        let mut e = env(tmp.path(), Os::Windows);
        e.exe = PathBuf::from("C:/Program Files/GP/gitpoulpi-tui.exe");
        let text = cmd_shim_text(&e.exe);
        assert!(text.contains("\r\nREM X-GitPoulpi=1\r\n"));
        assert!(text.ends_with("gitpoulpi-tui.exe\" %*\r\n"), "{text}");
        let path = create(&e, "gp").unwrap();
        assert_eq!(path, e.bin_dir.join("gp.cmd"));
        let st = status(&e, "gp");
        assert_eq!(st.state, State::Installed);
        assert!(st.powershell_builtin, "gp is Get-ItemProperty there");
        assert!(!status(&e, "gpx").powershell_builtin);
        assert_eq!(list(&e), [("gp".to_string(), State::Installed)]);
        // A .cmd without the marker is foreign.
        std::fs::write(e.bin_dir.join("other.cmd"), "@echo hi\r\n").unwrap();
        assert_eq!(status(&e, "other").state, State::Foreign);
        assert!(remove(&e, "other").is_err());
        assert!(remove(&e, "gp").unwrap());
        assert!(!path.exists());
    }

    #[test]
    fn discover_picks_the_per_user_bin_dirs() {
        let env = |k: &str| (k == "LOCALAPPDATA").then(|| "/u/AppData/Local".to_string());
        let w = Env::discover(
            Os::Windows,
            &env,
            Path::new("/h"),
            Path::new("/x/gitpoulpi.exe"),
        );
        assert_eq!(
            w.bin_dir,
            Path::new("/u/AppData/Local/Programs/GitPoulpi/bin")
        );
        let l = Env::discover(Os::Linux, &env, Path::new("/h"), Path::new("/x/gitpoulpi"));
        assert_eq!(l.bin_dir, Path::new("/h/.local/bin"));
    }

    #[cfg(unix)]
    #[test]
    fn the_add_variant_is_a_marked_script_that_execs_with_add() {
        let tmp = tempfile::tempdir().unwrap();
        let e = env(tmp.path(), Os::Linux);
        assert_eq!(status_kind(&e, "gpa", Kind::Add).state, State::NotInstalled);
        let path = create_kind(&e, "gpa", Kind::Add).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        assert_eq!(
            text,
            format!("#!/bin/sh\n{UNIX_MARKER}\nexec '/opt/gp/gitpoulpi' --add \"$@\"\n")
        );
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            std::fs::metadata(&path).unwrap().permissions().mode() & 0o111,
            0o111,
            "executable"
        );
        assert_eq!(status_kind(&e, "gpa", Kind::Add).state, State::Installed);
        // Asked for as the open kind it is ours but wrong: Create fixes it.
        assert_eq!(status_kind(&e, "gpa", Kind::Open).state, State::Outdated);
        assert_eq!(
            list_kinds(&e),
            [("gpa".to_string(), Kind::Add, State::Installed)]
        );
        // Both kinds side by side, each listed with what it runs.
        create(&e, "gp").unwrap();
        let kinds: Vec<_> = list_kinds(&e).into_iter().map(|(n, k, _)| (n, k)).collect();
        assert_eq!(
            kinds,
            [
                ("gp".to_string(), Kind::Open),
                ("gpa".to_string(), Kind::Add)
            ]
        );
        // Moved exe: outdated; Create repoints; Remove deletes only ours.
        let moved = Env {
            exe: PathBuf::from("/elsewhere/gitpoulpi-tui"),
            ..e.clone()
        };
        assert_eq!(status_kind(&moved, "gpa", Kind::Add).state, State::Outdated);
        create_kind(&moved, "gpa", Kind::Add).unwrap();
        assert_eq!(
            status_kind(&moved, "gpa", Kind::Add).state,
            State::Installed
        );
        assert!(remove(&moved, "gpa").unwrap());
        assert!(std::fs::symlink_metadata(&path).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn a_script_without_the_marker_is_foreign_and_a_quote_in_the_path_round_trips() {
        let tmp = tempfile::tempdir().unwrap();
        let e = env(tmp.path(), Os::Linux);
        std::fs::create_dir_all(&e.bin_dir).unwrap();
        std::fs::write(e.bin_dir.join("gpa"), "#!/bin/sh\necho hi\n").unwrap();
        assert_eq!(status_kind(&e, "gpa", Kind::Add).state, State::Foreign);
        assert!(create_kind(&e, "gpa", Kind::Add).is_err());
        assert!(remove(&e, "gpa").is_err());
        assert!(list_kinds(&e).is_empty());
        let odd = Path::new("/it's here/gitpoulpi-tui");
        let text = sh_shim_text(odd);
        assert_eq!(sh_parts(&text), Some((odd.to_path_buf(), Kind::Add)));
    }

    #[test]
    fn the_windows_add_shim_passes_add_and_round_trips() {
        let tmp = tempfile::tempdir().unwrap();
        let mut e = env(tmp.path(), Os::Windows);
        e.exe = PathBuf::from("C:/Program Files/GP/gitpoulpi-tui.exe");
        let text = cmd_shim_text_for(&e.exe, Kind::Add);
        assert!(text.ends_with("gitpoulpi-tui.exe\" --add %*\r\n"), "{text}");
        assert_eq!(shim_parts(&text), Some((e.exe.clone(), Kind::Add)));
        assert_eq!(
            shim_parts(&cmd_shim_text(&e.exe)),
            Some((e.exe.clone(), Kind::Open))
        );
        let path = create_kind(&e, "gpa", Kind::Add).unwrap();
        assert_eq!(path, e.bin_dir.join("gpa.cmd"));
        assert_eq!(status_kind(&e, "gpa", Kind::Add).state, State::Installed);
        assert_eq!(status_kind(&e, "gpa", Kind::Open).state, State::Outdated);
        assert!(!status_kind(&e, "gpa", Kind::Add).powershell_builtin);
        assert_eq!(
            list_kinds(&e),
            [("gpa".to_string(), Kind::Add, State::Installed)]
        );
    }

    #[test]
    fn newest_mode_points_at_the_newest_build_and_accepts_either() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("co");
        std::fs::create_dir_all(root.join("crates/gitpoulpi-tui")).unwrap();
        std::fs::write(root.join("Cargo.toml"), "").unwrap();
        let mk = |profile: &str, age: u64| {
            let dir = root.join("target").join(profile);
            std::fs::create_dir_all(&dir).unwrap();
            let p = dir.join("gitpoulpi-tui");
            let f = std::fs::File::create(&p).unwrap();
            f.set_modified(std::time::SystemTime::now() - std::time::Duration::from_secs(age))
                .unwrap();
            p
        };
        let dist = mk("dist", 100);
        let release = mk("release", 1);
        let from_dist = Env {
            exe: dist.clone(),
            newest: true,
            ..env(tmp.path(), Os::Windows)
        };
        let path = create(&from_dist, "gp").unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        assert_eq!(shim_target(&text), Some(release.clone()), "the newest");
        // Starting either build leaves it alone.
        assert_eq!(status(&from_dist, "gp").state, State::Installed);
        let from_release = Env {
            exe: release,
            ..from_dist.clone()
        };
        assert_eq!(status(&from_release, "gp").state, State::Installed);
        // Outside newest mode it is the old exact comparison.
        let plain = Env {
            newest: false,
            ..from_dist
        };
        assert_eq!(status(&plain, "gp").state, State::Outdated);
    }

    #[test]
    fn the_windows_shim_exports_the_cmd_command_line_and_keeps_args_last() {
        let exe = Path::new("C:/GP/gitpoulpi-tui.exe");
        for (kind, flag) in [(Kind::Open, ""), (Kind::Add, " --add")] {
            let text = cmd_shim_text_for(exe, kind);
            assert_eq!(
                text,
                format!(
                    "@echo off\r\nREM X-GitPoulpi=1\r\nset \"GITPOULPI_CMDCMDLINE=%CMDCMDLINE%\"\r\n\"C:/GP/gitpoulpi-tui.exe\"{flag} %*\r\n"
                )
            );
            assert_eq!(shim_parts(&text), Some((exe.to_path_buf(), kind)));
        }
    }

    #[test]
    fn an_old_shim_reads_outdated_and_create_rewrites_it() {
        let tmp = tempfile::tempdir().unwrap();
        let mut e = env(tmp.path(), Os::Windows);
        e.exe = PathBuf::from("C:/GP/gitpoulpi-tui.exe");
        std::fs::create_dir_all(&e.bin_dir).unwrap();
        let old = "@echo off\r\nREM X-GitPoulpi=1\r\n\"C:/GP/gitpoulpi-tui.exe\" %*\r\n";
        std::fs::write(e.bin_dir.join("gp.cmd"), old).unwrap();
        assert_eq!(status(&e, "gp").state, State::Outdated);
        assert_eq!(
            list_kinds(&e),
            [("gp".to_string(), Kind::Open, State::Outdated)]
        );
        create(&e, "gp").unwrap();
        assert_eq!(status(&e, "gp").state, State::Installed);
    }

    fn windows_env_with_helper(root: &Path) -> (Env, PathBuf) {
        let mut e = env(root, Os::Windows);
        e.exe = PathBuf::from("C:/GP/gitpoulpi-tui.exe");
        (e, PathBuf::from("C:/GP/gitpoulpi-launch.exe"))
    }

    #[test]
    fn app_paths_follow_the_open_alias() {
        use crate::desktop::registry::MemoryRegistry;
        let tmp = tempfile::tempdir().unwrap();
        let (e, helper) = windows_env_with_helper(tmp.path());
        let reg = MemoryRegistry::new();
        let has = |p: &Path| p == helper;
        create_with_app_path_in(&e, &reg, "gp", Kind::Open, &has).unwrap();
        let key = app_path_key("gp.exe");
        let dump = reg.dump();
        assert!(
            dump.iter().any(|(k, n, v)| k.eq_ignore_ascii_case(&key)
                && n.is_empty()
                && v.ends_with("gitpoulpi-launch.exe")),
            "{dump:?}"
        );
        assert_eq!(app_path_is_ours(&reg, "gp.exe").unwrap(), Some(true));
        // The add variant has no App Paths key.
        create_with_app_path_in(&e, &reg, "gpa", Kind::Add, &has).unwrap();
        assert_eq!(app_path_is_ours(&reg, "gpa.exe").unwrap(), None);
        // Remove deletes the key with the shim.
        assert!(remove_with_app_path(&e, &reg, "gp").unwrap());
        assert_eq!(app_path_is_ours(&reg, "gp.exe").unwrap(), None);
    }

    #[test]
    fn app_paths_skip_without_a_launcher_and_spare_a_foreign_key() {
        use crate::desktop::registry::{MemoryRegistry, RegOp};
        let tmp = tempfile::tempdir().unwrap();
        let (e, helper) = windows_env_with_helper(tmp.path());
        let reg = MemoryRegistry::new();
        create_with_app_path_in(&e, &reg, "gp", Kind::Open, &|_| false).unwrap();
        assert!(reg.dump().is_empty(), "no launcher: no key");
        // A key that is not ours stays as it is, on create and on remove.
        let key = app_path_key("gp.exe");
        reg.apply(&[RegOp::Set {
            key: key.clone(),
            name: String::new(),
            value: "D:/other.exe".into(),
        }])
        .unwrap();
        create_with_app_path_in(&e, &reg, "gp", Kind::Open, &|p| p == helper).unwrap();
        assert_eq!(app_path_is_ours(&reg, "gp.exe").unwrap(), Some(false));
        assert!(remove_with_app_path(&e, &reg, "gp").unwrap());
        assert_eq!(app_path_is_ours(&reg, "gp.exe").unwrap(), Some(false));
        // Other systems never touch the registry.
        let lin = env(tmp.path(), Os::Linux);
        let reg2 = MemoryRegistry::new();
        let _ = create_with_app_path_in(&lin, &reg2, "gp", Kind::Open, &|_| true);
        assert!(reg2.dump().is_empty());
    }
}
