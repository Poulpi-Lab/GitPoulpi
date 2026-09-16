//! "Add GitPoulpi to PATH": the directory the GitPoulpi programs live in and
//! the alias `bin` directory ([`super::alias`]) go on the user's `PATH`
//! (Settings > General > Launcher, `gitpoulpi path ...`).
//!
//! | OS | Where | How it is known to be ours |
//! |---|---|---|
//! | Linux, macOS | a block in `~/.bashrc` / `~/.zshrc`, a file in fish's `conf.d` | the block's marker lines, the file's first line |
//! | Windows | the user `PATH` in `HKCU\Environment` | the value `GitPoulpiPathAdded` lists what we added |
//!
//! Only directories that are not on `PATH` yet are added, and Remove takes
//! out exactly what the marker says we added. The shell files and the
//! registry come through [`Env`] / [`UserPath`], so tests never touch the
//! real ones. Everything here is blocking (Windows spawns PowerShell).

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use crate::open_in::Os;

const BEGIN: &str = "# >>> GitPoulpi PATH (X-GitPoulpi=1) >>>";
const END: &str = "# <<< GitPoulpi PATH <<<";
const DIRS_PREFIX: &str = "# dirs: ";
const FISH_MARKER: &str = "# X-GitPoulpi=1";
const FISH_FILE: &str = "fish/conf.d/gitpoulpi-path.fish";
/// The registry value (next to `Path`) listing what we added.
pub const ADDED_VALUE: &str = "GitPoulpiPathAdded";

/// The machine `PATH` is changed on.
#[derive(Debug, Clone)]
pub struct Env {
    pub os: Os,
    pub home: PathBuf,
    /// `$XDG_CONFIG_HOME` (fish).
    pub config_home: PathBuf,
    /// What should be on `PATH`: the folder the programs live in, then the
    /// alias bin directory.
    pub wanted: Vec<PathBuf>,
    /// The current process `PATH`.
    pub path_var: Option<OsString>,
}

impl Env {
    pub fn discover(
        os: Os,
        env: &dyn Fn(&str) -> Option<String>,
        home: &Path,
        alias: &super::alias::Env,
    ) -> Env {
        let var = |k: &str| env(k).filter(|v| !v.is_empty()).map(PathBuf::from);
        let mut wanted = Vec::new();
        for dir in [
            alias.exe.parent().map(Path::to_path_buf),
            Some(alias.bin_dir.clone()),
        ]
        .into_iter()
        .flatten()
        {
            if !dir.as_os_str().is_empty() && !wanted.contains(&dir) {
                wanted.push(dir);
            }
        }
        Env {
            os,
            home: home.to_path_buf(),
            config_home: var("XDG_CONFIG_HOME").unwrap_or_else(|| home.join(".config")),
            wanted,
            path_var: env("PATH").map(OsString::from),
        }
    }

    fn on_path(&self, dir: &Path) -> bool {
        self.path_var
            .as_ref()
            .is_some_and(|p| std::env::split_paths(p).any(|e| same_dir(self.os, &e, dir)))
    }
}

fn norm(os: Os, s: &str) -> String {
    let t = s.trim().trim_end_matches(['/', '\\']);
    if os == Os::Windows {
        t.replace('/', "\\").to_lowercase()
    } else {
        t.to_string()
    }
}

fn same_dir(os: Os, a: &Path, b: &Path) -> bool {
    norm(os, &a.to_string_lossy()) == norm(os, &b.to_string_lossy())
}

/// What the user's `PATH` holds of ours.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    NotAdded,
    /// Our entry is there and covers every wanted directory.
    Added,
    /// Our entry is there but a wanted directory is missing (the install
    /// moved): Add fixes it.
    Outdated,
    /// Every wanted directory is on `PATH` already, not by us: nothing to do.
    OnPath,
}

impl State {
    pub fn as_str(self) -> &'static str {
        match self {
            State::NotAdded => "not_added",
            State::Added => "added",
            State::Outdated => "outdated",
            State::OnPath => "on_path",
        }
    }
}

/// What a status read found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Status {
    pub state: State,
    /// The directories a new shell would find GitPoulpi in.
    pub wanted: Vec<PathBuf>,
    /// What our entry added.
    pub added: Vec<PathBuf>,
    /// The files (or `HKCU\Environment`) holding our entry.
    pub files: Vec<PathBuf>,
}

// ---- Unix shells -------------------------------------------------------

fn posix_quote(dir: &str) -> String {
    let mut out = String::new();
    for c in dir.chars() {
        if matches!(c, '"' | '$' | '`' | '\\') {
            out.push('\\');
        }
        out.push(c);
    }
    out
}

fn fish_quote(dir: &str) -> String {
    format!("'{}'", dir.replace('\\', "\\\\").replace('\'', "\\'"))
}

/// The marked block for `.bashrc` / `.zshrc`.
pub fn rc_block(dirs: &[PathBuf]) -> String {
    let list: Vec<String> = dirs
        .iter()
        .map(|d| d.to_string_lossy().into_owned())
        .collect();
    let quoted: Vec<String> = list.iter().map(|d| posix_quote(d)).collect();
    format!(
        "{BEGIN}\n{DIRS_PREFIX}{}\nexport PATH=\"$PATH:{}\"\n{END}\n",
        list.join(":"),
        quoted.join(":")
    )
}

/// The fish `conf.d` file.
pub fn fish_text(dirs: &[PathBuf]) -> String {
    let list: Vec<String> = dirs
        .iter()
        .map(|d| d.to_string_lossy().into_owned())
        .collect();
    let quoted: Vec<String> = list.iter().map(|d| fish_quote(d)).collect();
    format!(
        "{FISH_MARKER}\n{DIRS_PREFIX}{}\nfish_add_path -a {}\n",
        list.join(":"),
        quoted.join(" ")
    )
}

/// `text` without our block, byte for byte otherwise; and the dirs the block
/// listed (`None`: there was no block).
pub fn strip_block(text: &str) -> (String, Option<Vec<PathBuf>>) {
    let mut out = String::new();
    let mut dirs = None;
    let mut inside = false;
    for line in text.split_inclusive('\n') {
        let bare = line.trim_end_matches(['\r', '\n']);
        if !inside && bare == BEGIN {
            inside = true;
            dirs = Some(Vec::new());
            // The blank line `with_block` put before the block goes with it.
            if out.ends_with("\n\n") {
                out.pop();
            }
            continue;
        }
        if inside {
            if let Some(list) = bare.strip_prefix(DIRS_PREFIX) {
                dirs = Some(
                    list.split(':')
                        .filter(|d| !d.is_empty())
                        .map(PathBuf::from)
                        .collect(),
                );
            }
            if bare == END {
                inside = false;
            }
            continue;
        }
        out.push_str(line);
    }
    (out, dirs)
}

/// `text` with our block added (or replaced in place).
pub fn with_block(text: &str, dirs: &[PathBuf]) -> String {
    let block = rc_block(dirs);
    let (rest, existing) = strip_block(text);
    if existing.is_some() {
        // Keep the block where it was.
        let mut out = String::new();
        let mut inside = false;
        for line in text.split_inclusive('\n') {
            let bare = line.trim_end_matches(['\r', '\n']);
            if !inside && bare == BEGIN {
                inside = true;
                out.push_str(&block);
            } else if inside {
                inside = bare != END;
            } else {
                out.push_str(line);
            }
        }
        return out;
    }
    let mut out = rest;
    if !out.is_empty() && !out.ends_with('\n') {
        out.push('\n');
    }
    if !out.is_empty() {
        out.push('\n');
    }
    out.push_str(&block);
    out
}

fn shell_files(env: &Env) -> (Vec<PathBuf>, Option<PathBuf>) {
    let bashrc = env.home.join(".bashrc");
    let zshrc = env.home.join(".zshrc");
    let mut rcs: Vec<PathBuf> = [bashrc.clone(), zshrc.clone()]
        .into_iter()
        .filter(|p| p.exists())
        .collect();
    if rcs.is_empty() {
        rcs.push(if env.os == Os::Macos { zshrc } else { bashrc });
    }
    let fish = env
        .config_home
        .join("fish")
        .is_dir()
        .then(|| env.config_home.join(FISH_FILE));
    (rcs, fish)
}

/// Every shell file that already holds our entry, with its dirs.
fn unix_entries(env: &Env) -> Vec<(PathBuf, Vec<PathBuf>)> {
    let mut out = Vec::new();
    for rc in [".bashrc", ".zshrc"] {
        let path = env.home.join(rc);
        if let Ok(text) = std::fs::read_to_string(&path) {
            if let (_, Some(dirs)) = strip_block(&text) {
                out.push((path, dirs));
            }
        }
    }
    let fish = env.config_home.join(FISH_FILE);
    if let Ok(text) = std::fs::read_to_string(&fish) {
        if text.lines().next() == Some(FISH_MARKER) {
            let dirs = text
                .lines()
                .find_map(|l| l.strip_prefix(DIRS_PREFIX))
                .map(|l| {
                    l.split(':')
                        .filter(|d| !d.is_empty())
                        .map(PathBuf::from)
                        .collect()
                })
                .unwrap_or_default();
            out.push((fish, dirs));
        }
    }
    out
}

// ---- Windows user PATH -------------------------------------------------

/// What `HKCU\Environment` holds: the unexpanded `Path` and our record.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct UserPathValues {
    pub path: String,
    pub added: Option<String>,
}

/// The user `PATH` store: the registry in real life, memory in tests.
pub trait UserPath {
    fn read(&self) -> Result<UserPathValues, String>;
    /// Writes both values (`added: None` deletes the record) and tells the
    /// running programs (`WM_SETTINGCHANGE`).
    fn write(&self, path: &str, added: Option<&str>) -> Result<(), String>;
}

/// `HKCU\Environment` through PowerShell.
pub struct Registry;

fn run_powershell(script: &str, envs: &[(&str, &str)]) -> Result<String, String> {
    let mut cmd = std::process::Command::new("powershell.exe");
    cmd.args([
        "-NoProfile",
        "-NonInteractive",
        "-ExecutionPolicy",
        "Bypass",
        "-EncodedCommand",
        &super::windows::encode_command(script),
    ]);
    cmd.envs(envs.iter().copied());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
    }
    let out = cmd.output().map_err(|e| format!("PowerShell: {e}"))?;
    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).into_owned())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).trim().to_string())
    }
}

const READ_SCRIPT: &str = "$k=[Microsoft.Win32.Registry]::CurrentUser.OpenSubKey('Environment');\
$p=$k.GetValue('Path','',[Microsoft.Win32.RegistryValueOptions]::DoNotExpandEnvironmentNames);\
$a=$k.GetValue('GitPoulpiPathAdded',$null);\
[Console]::Out.Write('PATH='+$p+\"`n\");\
if($a -ne $null){[Console]::Out.Write('ADDED='+$a+\"`n\")}";

const WRITE_SCRIPT: &str = "$k=[Microsoft.Win32.Registry]::CurrentUser.OpenSubKey('Environment',$true);\
$k.SetValue('Path',$env:GP_NEW_PATH,[Microsoft.Win32.RegistryValueKind]::ExpandString);\
if($env:GP_ADDED){$k.SetValue('GitPoulpiPathAdded',$env:GP_ADDED)}else{$k.DeleteValue('GitPoulpiPathAdded',$false)};\
Add-Type -Namespace GP -Name N -MemberDefinition '[DllImport(\"user32.dll\",SetLastError=true,CharSet=CharSet.Auto)] public static extern IntPtr SendMessageTimeout(IntPtr h,uint m,UIntPtr w,string l,uint f,uint t,out UIntPtr r);';\
$r=[UIntPtr]::Zero;\
[void][GP.N]::SendMessageTimeout([IntPtr]0xffff,0x1A,[UIntPtr]::Zero,'Environment',2,5000,[ref]$r)";

/// Parses [`READ_SCRIPT`]'s output.
pub fn parse_registry_read(out: &str) -> UserPathValues {
    let mut values = UserPathValues::default();
    for line in out.lines() {
        if let Some(p) = line.strip_prefix("PATH=") {
            values.path = p.trim_end_matches('\r').to_string();
        } else if let Some(a) = line.strip_prefix("ADDED=") {
            values.added = Some(a.trim_end_matches('\r').to_string());
        }
    }
    values
}

impl UserPath for Registry {
    fn read(&self) -> Result<UserPathValues, String> {
        run_powershell(READ_SCRIPT, &[]).map(|o| parse_registry_read(&o))
    }

    fn write(&self, path: &str, added: Option<&str>) -> Result<(), String> {
        run_powershell(
            WRITE_SCRIPT,
            &[("GP_NEW_PATH", path), ("GP_ADDED", added.unwrap_or(""))],
        )
        .map(|_| ())
    }
}

fn split_entries(value: &str) -> Vec<&str> {
    value.split(';').filter(|e| !e.trim().is_empty()).collect()
}

fn contains(os: Os, entries: &[&str], dir: &str) -> bool {
    entries.iter().any(|e| norm(os, e) == norm(os, dir))
}

fn win_dir(dir: &Path) -> String {
    dir.to_string_lossy().replace('/', "\\")
}

/// The new `Path` value and record for Add: what we added before is taken
/// out first, then every wanted directory that is missing goes on the end.
pub fn windows_add(values: &UserPathValues, wanted: &[PathBuf]) -> (String, Option<String>) {
    let before = values
        .added
        .as_deref()
        .map(split_entries)
        .unwrap_or_default();
    let mut entries: Vec<&str> = split_entries(&values.path)
        .into_iter()
        .filter(|e| !contains(Os::Windows, &before, e))
        .collect();
    let wanted: Vec<String> = wanted.iter().map(|d| win_dir(d)).collect();
    let missing: Vec<&str> = wanted
        .iter()
        .map(String::as_str)
        .filter(|d| !contains(Os::Windows, &entries, d))
        .collect();
    entries.extend(&missing);
    let added = (!missing.is_empty()).then(|| missing.join(";"));
    (entries.join(";"), added)
}

/// The new `Path` value for Remove: exactly what the record lists goes.
pub fn windows_remove(values: &UserPathValues) -> String {
    let added = values
        .added
        .as_deref()
        .map(split_entries)
        .unwrap_or_default();
    split_entries(&values.path)
        .into_iter()
        .filter(|e| !contains(Os::Windows, &added, e))
        .collect::<Vec<_>>()
        .join(";")
}

// ---- The three operations ----------------------------------------------

/// Reads what is there. `user` is only used for [`Os::Windows`].
pub fn status(env: &Env, user: &dyn UserPath) -> Result<Status, String> {
    let wanted = env.wanted.clone();
    if env.os == Os::Windows {
        let values = user.read()?;
        let entries = split_entries(&values.path);
        let added: Vec<PathBuf> = values
            .added
            .as_deref()
            .map(split_entries)
            .unwrap_or_default()
            .into_iter()
            .map(PathBuf::from)
            .collect();
        let covered = |d: &PathBuf| contains(Os::Windows, &entries, &win_dir(d));
        let state = if values.added.is_some() {
            if wanted.iter().all(covered) {
                State::Added
            } else {
                State::Outdated
            }
        } else if wanted.iter().all(|d| covered(d) || env.on_path(d)) {
            State::OnPath
        } else {
            State::NotAdded
        };
        let files = if values.added.is_some() {
            vec![PathBuf::from("HKCU\\Environment")]
        } else {
            Vec::new()
        };
        return Ok(Status {
            state,
            wanted,
            added,
            files,
        });
    }
    let entries = unix_entries(env);
    let state = if let Some((_, dirs)) = entries.first() {
        let covered = |d: &PathBuf| {
            entries
                .iter()
                .any(|(_, ds)| ds.iter().any(|x| same_dir(env.os, x, d)))
                || env.on_path(d)
        };
        let _ = dirs;
        if wanted.iter().all(covered) {
            State::Added
        } else {
            State::Outdated
        }
    } else if wanted.iter().all(|d| env.on_path(d)) {
        State::OnPath
    } else {
        State::NotAdded
    };
    let added = entries.first().map(|(_, d)| d.clone()).unwrap_or_default();
    Ok(Status {
        state,
        wanted,
        added,
        files: entries.into_iter().map(|(p, _)| p).collect(),
    })
}

/// Adds the missing directories. Returns the files (or `HKCU\Environment`)
/// written; empty when every directory was on `PATH` already.
pub fn add(env: &Env, user: &dyn UserPath) -> Result<Vec<PathBuf>, String> {
    if env.os == Os::Windows {
        let values = user.read()?;
        let (path, added) = windows_add(&values, &env.wanted);
        if added.is_none() && values.added.is_none() {
            return Ok(Vec::new());
        }
        user.write(&path, added.as_deref())?;
        return Ok(vec![PathBuf::from("HKCU\\Environment")]);
    }
    let missing: Vec<PathBuf> = env
        .wanted
        .iter()
        .filter(|d| !env.on_path(d) || unix_entries(env).iter().any(|(_, ds)| ds.contains(d)))
        .cloned()
        .collect();
    if missing.is_empty() {
        return Ok(Vec::new());
    }
    let (rcs, fish) = shell_files(env);
    let mut written = Vec::new();
    for rc in rcs {
        let text = std::fs::read_to_string(&rc).unwrap_or_default();
        let new = with_block(&text, &missing);
        if new != text {
            std::fs::write(&rc, new).map_err(|e| format!("{}: {e}", rc.display()))?;
        }
        written.push(rc);
    }
    if let Some(fish) = fish {
        if let Some(parent) = fish.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
        }
        std::fs::write(&fish, fish_text(&missing))
            .map_err(|e| format!("{}: {e}", fish.display()))?;
        written.push(fish);
    }
    Ok(written)
}

/// Removes what we added. Returns the files (or `HKCU\Environment`) changed;
/// empty when there was nothing of ours.
pub fn remove(env: &Env, user: &dyn UserPath) -> Result<Vec<PathBuf>, String> {
    if env.os == Os::Windows {
        let values = user.read()?;
        if values.added.is_none() {
            return Ok(Vec::new());
        }
        user.write(&windows_remove(&values), None)?;
        return Ok(vec![PathBuf::from("HKCU\\Environment")]);
    }
    let mut changed = Vec::new();
    for (path, _) in unix_entries(env) {
        if path.ends_with(FISH_FILE) {
            std::fs::remove_file(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        } else {
            let text = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
            let (rest, _) = strip_block(&text);
            std::fs::write(&path, rest).map_err(|e| format!("{}: {e}", path.display()))?;
        }
        changed.push(path);
    }
    Ok(changed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    #[derive(Default)]
    struct Memory(RefCell<UserPathValues>);

    impl UserPath for Memory {
        fn read(&self) -> Result<UserPathValues, String> {
            Ok(self.0.borrow().clone())
        }
        fn write(&self, path: &str, added: Option<&str>) -> Result<(), String> {
            *self.0.borrow_mut() = UserPathValues {
                path: path.to_string(),
                added: added.map(String::from),
            };
            Ok(())
        }
    }

    fn unix_env(root: &Path, path_var: &str) -> Env {
        Env {
            os: Os::Linux,
            home: root.to_path_buf(),
            config_home: root.join(".config"),
            wanted: vec![PathBuf::from("/opt/gp"), root.join(".local/bin")],
            path_var: Some(path_var.into()),
        }
    }

    #[test]
    fn the_block_is_added_replaced_and_removed_byte_for_byte() {
        let dirs = [PathBuf::from("/opt/gp"), PathBuf::from("/h/my \"bin\"")];
        let original = "export A=1\nalias ll='ls -l'\n";
        let added = with_block(original, &dirs);
        assert!(
            added.contains("export PATH=\"$PATH:/opt/gp:/h/my \\\"bin\\\"\"\n"),
            "{added}"
        );
        let (rest, found) = strip_block(&added);
        assert_eq!(rest, original);
        assert_eq!(found.unwrap(), dirs);
        // Idempotent: adding again changes nothing.
        assert_eq!(with_block(&added, &dirs), added);
        // A new list replaces the block in place, text after it is kept.
        let tail = format!("{added}export B=2\n");
        let swapped = with_block(&tail, &dirs[..1]);
        let (rest, found) = strip_block(&swapped);
        assert_eq!(rest, format!("{original}export B=2\n"));
        assert_eq!(found.unwrap(), dirs[..1]);
        assert!(swapped.find("/opt/gp").unwrap() < swapped.find("export B").unwrap());
        // No block: strip is a no-op.
        assert_eq!(strip_block(original), (original.to_string(), None));
        // A file with no final newline still gets a clean block.
        assert!(with_block("x", &dirs).starts_with("x\n\n# >>>"));
    }

    // The rc files and the unix `PATH` use `:`, which a Windows temp dir
    // (`C:\...`) contains: the unix flow is only meaningful on unix paths.
    #[cfg(unix)]
    #[test]
    fn unix_add_status_remove_touch_only_marked_text() {
        let tmp = tempfile::tempdir().unwrap();
        let e = unix_env(tmp.path(), "/usr/bin");
        std::fs::write(tmp.path().join(".bashrc"), "export A=1\n").unwrap();
        std::fs::write(tmp.path().join(".zshrc"), "export Z=1\n").unwrap();
        std::fs::create_dir_all(tmp.path().join(".config/fish")).unwrap();
        let user = Memory::default();
        assert_eq!(status(&e, &user).unwrap().state, State::NotAdded);
        let files = add(&e, &user).unwrap();
        assert_eq!(files.len(), 3, "bash, zsh and fish: {files:?}");
        let fish = std::fs::read_to_string(tmp.path().join(".config").join(FISH_FILE)).unwrap();
        assert!(
            fish.starts_with("# X-GitPoulpi=1\n") && fish.contains("fish_add_path -a '/opt/gp'")
        );
        let before = std::fs::read_to_string(tmp.path().join(".bashrc")).unwrap();
        assert_eq!(status(&e, &user).unwrap().state, State::Added);
        // Idempotent.
        add(&e, &user).unwrap();
        assert_eq!(
            std::fs::read_to_string(tmp.path().join(".bashrc")).unwrap(),
            before
        );
        // A new shell has the dirs on PATH: still Added, and nothing re-added.
        let sourced = unix_env(tmp.path(), "/usr/bin:/opt/gp:/h/.local/bin");
        assert_eq!(status(&sourced, &user).unwrap().state, State::Added);
        // The install moved: outdated until Add rewrites the block.
        let mut moved = unix_env(tmp.path(), "/usr/bin");
        moved.wanted[0] = PathBuf::from("/new/gp");
        assert_eq!(status(&moved, &user).unwrap().state, State::Outdated);
        add(&moved, &user).unwrap();
        assert_eq!(status(&moved, &user).unwrap().state, State::Added);
        // Remove: the user's lines stay, the fish file goes.
        let changed = remove(&moved, &user).unwrap();
        assert_eq!(changed.len(), 3);
        assert_eq!(
            std::fs::read_to_string(tmp.path().join(".bashrc")).unwrap(),
            "export A=1\n"
        );
        assert_eq!(
            std::fs::read_to_string(tmp.path().join(".zshrc")).unwrap(),
            "export Z=1\n"
        );
        assert!(!tmp.path().join(".config").join(FISH_FILE).exists());
        assert!(remove(&moved, &user).unwrap().is_empty());
    }

    // Same reason as the test above: `:`-separated `PATH` and unix paths.
    #[cfg(unix)]
    #[test]
    fn dirs_already_on_path_are_left_alone_and_a_fish_file_not_ours_is_kept() {
        let tmp = tempfile::tempdir().unwrap();
        let on = format!(
            "/usr/bin:/opt/gp:{}",
            tmp.path().join(".local/bin").display()
        );
        let e = unix_env(tmp.path(), &on);
        let user = Memory::default();
        assert_eq!(status(&e, &user).unwrap().state, State::OnPath);
        assert!(add(&e, &user).unwrap().is_empty());
        assert!(!tmp.path().join(".bashrc").exists(), "nothing written");
        // Only one is missing: only that one is written.
        let e = unix_env(tmp.path(), "/usr/bin:/opt/gp");
        add(&e, &user).unwrap();
        let (_, dirs) = strip_block(&std::fs::read_to_string(tmp.path().join(".bashrc")).unwrap());
        assert_eq!(dirs.unwrap(), [tmp.path().join(".local/bin")]);
        // A fish file that is not ours is never removed.
        let fish = tmp.path().join(".config").join(FISH_FILE);
        std::fs::create_dir_all(fish.parent().unwrap()).unwrap();
        std::fs::write(&fish, "set -x A 1\n").unwrap();
        remove(&e, &user).unwrap();
        assert!(fish.exists());
    }

    #[test]
    fn macos_with_no_rc_file_gets_zshrc() {
        let tmp = tempfile::tempdir().unwrap();
        let mut e = unix_env(tmp.path(), "/usr/bin");
        e.os = Os::Macos;
        add(&e, &Memory::default()).unwrap();
        assert!(tmp.path().join(".zshrc").exists());
        assert!(!tmp.path().join(".bashrc").exists());
    }

    #[test]
    fn windows_user_path_adds_what_is_missing_and_removes_only_that() {
        let wanted = [
            PathBuf::from("C:/Apps/GP"),
            PathBuf::from("C:/Users/me/AppData/Local/Programs/GitPoulpi/bin"),
        ];
        let values = UserPathValues {
            path: "C:\\Windows;%USERPROFILE%\\bin;c:\\apps\\gp\\".into(),
            added: None,
        };
        let (path, added) = windows_add(&values, &wanted);
        // C:\Apps\GP is there already (case and slash differ): not repeated.
        assert_eq!(
            path,
            "C:\\Windows;%USERPROFILE%\\bin;c:\\apps\\gp\\;C:\\Users\\me\\AppData\\Local\\Programs\\GitPoulpi\\bin"
        );
        assert_eq!(
            added.as_deref(),
            Some("C:\\Users\\me\\AppData\\Local\\Programs\\GitPoulpi\\bin")
        );
        let after = UserPathValues { path, added };
        let back = windows_remove(&after);
        assert_eq!(back, "C:\\Windows;%USERPROFILE%\\bin;c:\\apps\\gp\\");
        // Add again: what we added is replaced, never doubled.
        let (again, added2) = windows_add(&after, &wanted);
        assert_eq!(again, after.path);
        assert_eq!(added2, after.added);
    }

    #[test]
    fn windows_flow_through_the_store() {
        let tmp = tempfile::tempdir().unwrap();
        let e = Env {
            os: Os::Windows,
            home: tmp.path().to_path_buf(),
            config_home: tmp.path().to_path_buf(),
            wanted: vec![PathBuf::from("C:/Apps/GP")],
            path_var: Some("C:\\Windows".into()),
        };
        let user = Memory(RefCell::new(UserPathValues {
            path: "C:\\Windows".into(),
            added: None,
        }));
        assert_eq!(status(&e, &user).unwrap().state, State::NotAdded);
        assert_eq!(add(&e, &user).unwrap().len(), 1);
        assert_eq!(user.0.borrow().path, "C:\\Windows;C:\\Apps\\GP");
        assert_eq!(status(&e, &user).unwrap().state, State::Added);
        assert_eq!(remove(&e, &user).unwrap().len(), 1);
        assert_eq!(
            *user.0.borrow(),
            UserPathValues {
                path: "C:\\Windows".into(),
                added: None
            }
        );
        assert!(
            remove(&e, &user).unwrap().is_empty(),
            "nothing of ours: untouched"
        );
        // Already there, not by us: nothing is written and Remove stays out.
        let there = Memory(RefCell::new(UserPathValues {
            path: "C:\\Apps\\GP".into(),
            added: None,
        }));
        assert_eq!(status(&e, &there).unwrap().state, State::OnPath);
        assert!(add(&e, &there).unwrap().is_empty());
        assert!(remove(&e, &there).unwrap().is_empty());
        assert_eq!(there.0.borrow().path, "C:\\Apps\\GP");
    }

    #[test]
    fn the_registry_answer_is_parsed() {
        let v = parse_registry_read("PATH=C:\\a;%X%\\b\r\nADDED=C:\\a\r\n");
        assert_eq!(v.path, "C:\\a;%X%\\b");
        assert_eq!(v.added.as_deref(), Some("C:\\a"));
        assert_eq!(parse_registry_read("PATH=\n"), UserPathValues::default());
    }
}
