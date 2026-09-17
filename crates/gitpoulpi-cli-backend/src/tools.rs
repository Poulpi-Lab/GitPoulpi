//! Is a program usable here? One cached probe for every external tool a
//! history rewrite may use, the generalisation of the old `LfsProbe`.
//!
//! - [`ToolProbe`]: per-[`Tool`] answers with a 60 s cache, [`ToolProbe::fresh`]
//!   for explicit "Check again" actions, a constructor override for tests and
//!   the environment seam `GITPOULPI_TOOL_<NAME>` (see [`parse_override`]).
//! - Resolution is [`resolve_program`] (`PATH`, with `PATHEXT` on Windows) plus,
//!   on Windows, Git for Windows' own `usr/bin` found from
//!   `git --exec-path` ([`git_for_windows_usr_bin`]): `gpg`, `ssh-keygen`,
//!   `bash` and `sh` live there and are usually not on `PATH`.
//! - The version check runs the program with a 2 s timeout. Probing spawns
//!   processes, so the daemon runs it, never the render thread.
//!
//! Lua scripts are embedded in GitPoulpi: there is nothing to probe for them.
//! The interpreters of external scripts (shell, Python, an installed Lua) are
//! probed like the other tools, and [`ToolProbe::program`] says which program
//! to run. The test seam `GITPOULPI_TOOL_<NAME>_PATH` points a tool at one
//! exact program (a fixture standing in for Python), whatever is installed.

use std::collections::HashMap;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use gitpoulpi_core::domain::rewrite::{
    OsFamily, RewriteTools, SigningFormat, Tool, ToolCheck, ToolStatus,
};
use gitpoulpi_core::domain::{LfsBinary, RepoRef};
use gitpoulpi_core::Result;

use crate::process::{run, run_lenient};

/// The older, `git-lfs` only test seam. Still honoured: `=missing`.
pub const LEGACY_LFS_ENV: &str = "GITPOULPI_GIT_LFS";

/// How long an answer is trusted, so installing a tool while GitPoulpi runs
/// is noticed within a minute.
const PROBE_TTL: Duration = Duration::from_secs(60);

/// How long `<program> --version` may take. Generous on purpose: on macOS the
/// first launch of a new executable (and of the developer-tools shims) can take
/// several seconds while the system checks it. A program that is slower than
/// this is still found, never reported as missing.
const VERSION_TIMEOUT: Duration = Duration::from_secs(15);

/// A per-backend cache of tool probes.
#[derive(Default)]
pub struct ToolProbe {
    cached: Mutex<HashMap<Tool, (Instant, ToolStatus)>>,
    /// Constructor-level overrides (tests). Unlike the environment seam they
    /// cannot leak into a parallel test.
    forced: HashMap<Tool, ToolStatus>,
}

impl ToolProbe {
    /// A probe that answers `status` for `tool` and asks for the others.
    pub fn forced(tool: Tool, status: ToolStatus) -> Self {
        Self::default().with_forced(tool, status)
    }

    /// Also answers `status` for `tool`.
    pub fn with_forced(mut self, tool: Tool, status: ToolStatus) -> Self {
        self.forced.insert(tool, status);
        self
    }

    /// The tool's state, from the cache when it is under a minute old.
    pub fn get(&self, tool: Tool, repo_root: &Path) -> ToolStatus {
        self.lookup(tool, repo_root, false)
    }

    /// Always asks, for explicit user actions; refreshes the cache.
    pub fn fresh(&self, tool: Tool, repo_root: &Path) -> ToolStatus {
        self.lookup(tool, repo_root, true)
    }

    /// The program that runs `tool`, when it is usable: the exact program of
    /// `GITPOULPI_TOOL_<NAME>_PATH` when set, else the first default name that
    /// resolves (`PATH`, then on Windows Git for Windows' `usr/bin`).
    #[cfg(feature = "rewrite-scripts")]
    pub fn program(&self, tool: Tool, repo_root: &Path) -> Option<PathBuf> {
        if !self.get(tool, repo_root).is_usable() {
            return None;
        }
        let dirs = search_dirs(repo_root);
        let pathext = pathext_from_env(std::env::var("PATHEXT").ok().as_deref());
        let found: Vec<PathBuf> = default_names(tool)
            .iter()
            .filter_map(|name| resolve_program(name, &dirs, &pathext, cfg!(windows)))
            .collect();
        // Several names resolve (`pwsh` and `powershell` on Windows): the
        // first one that answers, as `probe_names` does, not a stub that
        // does not (the Windows Store's `pwsh.exe` alias).
        match found.len() {
            0 | 1 => found.into_iter().next(),
            _ => found
                .into_iter()
                .find(|p| probe_resolved(tool, p).is_usable()),
        }
    }

    /// One check per tool, in [`Tool::ALL`] order.
    pub fn check_all(&self, repo_root: &Path, fresh: bool) -> RewriteTools {
        let os = OsFamily::current();
        RewriteTools {
            external_scripts: None,
            signing_format: signing_config(repo_root).format,
            tools: Tool::ALL
                .iter()
                .map(|&tool| ToolCheck::new(tool, self.lookup(tool, repo_root, fresh), os))
                .collect(),
        }
    }

    fn lookup(&self, tool: Tool, repo_root: &Path, fresh: bool) -> ToolStatus {
        if let Some(forced) = self
            .forced
            .get(&tool)
            .cloned()
            .or_else(|| env_override(tool))
        {
            return forced;
        }
        if !fresh {
            let cached = self.cached.lock().unwrap_or_else(|e| e.into_inner());
            if let Some((at, status)) = cached.get(&tool) {
                if at.elapsed() < PROBE_TTL {
                    return status.clone();
                }
            }
        }
        let status = probe(tool, repo_root);
        self.cached
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .insert(tool, (Instant::now(), status.clone()));
        status
    }
}

/// The backend's `rewrite_tools`.
pub fn rewrite_tools(repo: &RepoRef, probe: &ToolProbe, fresh: bool) -> Result<RewriteTools> {
    Ok(probe.check_all(&repo.root, fresh))
}

fn env_override(tool: Tool) -> Option<ToolStatus> {
    let var = format!("GITPOULPI_TOOL_{}", tool.env_suffix());
    if let Some(forced) = parse_override(tool, std::env::var(var).ok().as_deref()) {
        return Some(forced);
    }
    if tool == Tool::GitLfs {
        return parse_override(tool, std::env::var(LEGACY_LFS_ENV).ok().as_deref())
            .filter(|s| *s == ToolStatus::Missing);
    }
    None
}

/// A `GITPOULPI_TOOL_<NAME>` value to a forced answer:
///
/// - `missing`: not installed;
/// - `present`: installed, no version;
/// - `1.2.3` (digits and dots): installed at that version, or too old when
///   it is under the tool's minimum;
/// - `too_old`: too old;
/// - `not_configured`: installed, but its setup is incomplete.
///
/// An environment variable rather than a trimmed `PATH` because on Windows
/// Git for Windows finds `gpg.exe` and friends through its own `usr/bin`, so
/// no `PATH` edit can hide them, and because it crosses the daemon and pty
/// process boundaries.
pub fn parse_override(tool: Tool, value: Option<&str>) -> Option<ToolStatus> {
    let v = value?.trim();
    if v.eq_ignore_ascii_case("missing") {
        Some(ToolStatus::Missing)
    } else if v.eq_ignore_ascii_case("present") {
        Some(ToolStatus::Installed {
            version: String::new(),
        })
    } else if v.eq_ignore_ascii_case("too_old") {
        let (major, minor) = tool.min_version().unwrap_or((1, 0));
        Some(ToolStatus::TooOld {
            version: "0.0".into(),
            min: format!("{major}.{minor}"),
        })
    } else if v.eq_ignore_ascii_case("not_configured") {
        Some(ToolStatus::NotConfigured {
            reason: "forced by the test seam".into(),
        })
    } else if v.starts_with(|c: char| c.is_ascii_digit())
        && v.chars().all(|c| c.is_ascii_digit() || c == '.')
    {
        Some(ToolStatus::from_version(tool, v))
    } else {
        None
    }
}

// ---------------------------------------------------------------------
// Probing
// ---------------------------------------------------------------------

fn probe(tool: Tool, repo_root: &Path) -> ToolStatus {
    match tool {
        Tool::GitLfs => probe_git_lfs(repo_root),
        Tool::Gpg | Tool::SshKeygen | Tool::Gpgsm => probe_signing(tool, repo_root),
        _ => probe_names(tool, &default_names(tool), repo_root),
    }
}

fn probe_git_lfs(repo_root: &Path) -> ToolStatus {
    match run(repo_root, &["lfs", "version"]) {
        Ok(out) => status_of_lfs(&LfsBinary::from_version_output(&out)),
        Err(_) => ToolStatus::Missing,
    }
}

/// [`LfsBinary`] to [`ToolStatus`].
pub fn status_of_lfs(binary: &LfsBinary) -> ToolStatus {
    match binary {
        LfsBinary::Installed { version } => ToolStatus::Installed {
            version: version.clone(),
        },
        LfsBinary::TooOld { version } => ToolStatus::TooOld {
            version: version.clone(),
            min: "3.0".into(),
        },
        LfsBinary::Missing => ToolStatus::Missing,
    }
}

/// [`ToolStatus`] to [`LfsBinary`] (anything unusable but too old is
/// missing).
pub fn lfs_of_status(status: &ToolStatus) -> LfsBinary {
    match status {
        ToolStatus::Installed { version } => LfsBinary::Installed {
            version: version.clone(),
        },
        ToolStatus::TooOld { version, .. } => LfsBinary::TooOld {
            version: version.clone(),
        },
        ToolStatus::Missing | ToolStatus::NotConfigured { .. } => LfsBinary::Missing,
    }
}

/// `GITPOULPI_TOOL_<NAME>_PATH`: one exact program for `tool` (a test seam).
fn path_override(tool: Tool) -> Option<String> {
    std::env::var(format!("GITPOULPI_TOOL_{}_PATH", tool.env_suffix()))
        .ok()
        .filter(|v| !v.trim().is_empty())
}

fn default_names(tool: Tool) -> Vec<String> {
    if let Some(exact) = path_override(tool) {
        return vec![exact];
    }
    let names: &[&str] = match tool {
        Tool::Gpg => &["gpg"],
        Tool::SshKeygen => &["ssh-keygen"],
        Tool::Gpgsm => &["gpgsm"],
        Tool::GitLfs => &["git-lfs"],
        Tool::Python if cfg!(windows) => &["python3", "python", "py"],
        Tool::Python => &["python3", "python"],
        Tool::Lua => &["lua5.4", "lua54", "lua", "luajit"],
        Tool::Bash => &["bash"],
        Tool::Pwsh if cfg!(windows) => &["pwsh", "powershell"],
        Tool::Pwsh => &["pwsh"],
        Tool::Sh => &["sh"],
    };
    names.iter().map(|n| (*n).to_string()).collect()
}

/// What the signing probe needs from git config.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct SigningConfig {
    pub format: SigningFormat,
    pub key: Option<String>,
    /// `gpg.program`.
    pub gpg_program: Option<String>,
    /// `gpg.openpgp.program`.
    pub openpgp_program: Option<String>,
    /// `gpg.ssh.program`.
    pub ssh_program: Option<String>,
    /// `gpg.x509.program`.
    pub x509_program: Option<String>,
}

/// Parses `git config --get-regexp` lines (`key value`) of `gpg.*` and
/// `user.signingkey`. Keys are matched case-insensitively.
pub fn parse_signing_config(output: &str) -> SigningConfig {
    let mut cfg = SigningConfig::default();
    let mut format = None;
    for line in output.lines() {
        let Some((key, value)) = line.split_once(' ') else {
            continue;
        };
        let value = value.trim();
        if value.is_empty() {
            continue;
        }
        let value = Some(value.to_string());
        match key.to_ascii_lowercase().as_str() {
            "gpg.format" => format = value,
            "user.signingkey" => cfg.key = value,
            "gpg.program" => cfg.gpg_program = value,
            "gpg.openpgp.program" => cfg.openpgp_program = value,
            "gpg.ssh.program" => cfg.ssh_program = value,
            "gpg.x509.program" => cfg.x509_program = value,
            _ => {}
        }
    }
    cfg.format = SigningFormat::from_config(format.as_deref());
    cfg
}

impl SigningConfig {
    /// The program git runs for `tool`'s format, when configured.
    fn program_for(&self, tool: Tool) -> Option<&str> {
        match tool {
            Tool::Gpg => self
                .openpgp_program
                .as_deref()
                .or(self.gpg_program.as_deref()),
            Tool::SshKeygen => self.ssh_program.as_deref(),
            Tool::Gpgsm => self.x509_program.as_deref(),
            _ => None,
        }
    }
}

/// The repository's signing config (`gpg.*`, `user.signingkey`), one spawn.
pub(crate) fn signing_config(repo_root: &Path) -> SigningConfig {
    let (_, out) = run_lenient(
        repo_root,
        &["config", "--get-regexp", r"^(gpg\..*|user\.signingkey)$"],
    )
    .unwrap_or((false, String::new()));
    parse_signing_config(&out)
}

/// The program that signs for `cfg`'s format: the configured one when it
/// resolves, else the default name (`gpg`, `ssh-keygen`, `gpgsm`).
pub(crate) fn signing_program(cfg: &SigningConfig, repo_root: &Path) -> Option<PathBuf> {
    let tool = cfg.format.tool();
    let dirs = search_dirs(repo_root);
    let pathext = pathext_from_env(std::env::var("PATHEXT").ok().as_deref());
    let mut names: Vec<String> = cfg
        .program_for(tool)
        .map(str::to_string)
        .into_iter()
        .collect();
    names.extend(default_names(tool));
    names
        .iter()
        .find_map(|name| resolve_program(name, &dirs, &pathext, cfg!(windows)))
}

fn probe_signing(tool: Tool, repo_root: &Path) -> ToolStatus {
    let (_, out) = run_lenient(
        repo_root,
        &["config", "--get-regexp", r"^(gpg\..*|user\.signingkey)$"],
    )
    .unwrap_or((false, String::new()));
    let cfg = parse_signing_config(&out);
    let mut names = Vec::new();
    if let Some(p) = cfg.program_for(tool) {
        names.push(p.to_string());
    }
    names.extend(default_names(tool));
    let status = probe_names(tool, &names, repo_root);
    // An SSH or X.509 signature cannot be made without a configured key; a
    // missing OpenPGP key falls back to the committer identity.
    if status.is_usable() && cfg.format.tool() == tool && tool != Tool::Gpg && cfg.key.is_none() {
        return ToolStatus::NotConfigured {
            reason: "user.signingkey is not set".into(),
        };
    }
    status
}

/// The first of `names` that resolves, probed for its version.
fn probe_names(tool: Tool, names: &[String], repo_root: &Path) -> ToolStatus {
    let dirs = search_dirs(repo_root);
    let pathext = pathext_from_env(std::env::var("PATHEXT").ok().as_deref());
    for name in names {
        if let Some(path) = resolve_program(name, &dirs, &pathext, cfg!(windows)) {
            // A name that resolves but does not answer (an app-execution
            // alias stub) does not hide the next name.
            let status = probe_resolved(tool, &path);
            if status != ToolStatus::Missing {
                return status;
            }
        }
    }
    ToolStatus::Missing
}

/// `PATH`, then on Windows Git for Windows' `usr/bin`.
fn search_dirs(repo_root: &Path) -> Vec<PathBuf> {
    let mut dirs: Vec<PathBuf> = std::env::var_os("PATH")
        .map(|p| std::env::split_paths(&p).collect())
        .unwrap_or_default();
    if cfg!(windows) {
        if let Ok(exec_path) = run(repo_root, &["--exec-path"]) {
            if let Some(usr_bin) = git_for_windows_usr_bin(exec_path.trim()) {
                dirs.push(usr_bin);
            }
        }
    }
    dirs
}

/// Asks the resolved program for its version. `ssh-keygen` and `sh` have no
/// version output (as does `powershell.exe`): resolving is enough. A program
/// that exits non-zero counts as missing. One that has not answered within
/// [`VERSION_TIMEOUT`] is slow, not absent: it is reported installed with no
/// version, as for the tools that have none.
fn probe_resolved(tool: Tool, path: &Path) -> ToolStatus {
    probe_resolved_within(tool, path, VERSION_TIMEOUT)
}

/// [`probe_resolved`] with an explicit budget (tests shrink it).
fn probe_resolved_within(tool: Tool, path: &Path, budget: Duration) -> ToolStatus {
    // Windows PowerShell 5.1 (`powershell.exe`) rejects `--version`.
    let is_windows_powershell = path
        .file_stem()
        .is_some_and(|s| s.eq_ignore_ascii_case("powershell"));
    if matches!(tool, Tool::SshKeygen | Tool::Sh) || is_windows_powershell {
        return ToolStatus::Installed {
            version: String::new(),
        };
    }
    // `lua` has no `--version`, only `-v`.
    let flag = if tool == Tool::Lua { "-v" } else { "--version" };
    match run_version(path, flag, budget) {
        VersionAnswer::Output(out) => {
            ToolStatus::from_version(tool, &first_version(&out).unwrap_or_default())
        }
        VersionAnswer::Failed => ToolStatus::Missing,
        // A timeout is never "not installed": the program exists, it is slow.
        VersionAnswer::TimedOut => ToolStatus::Installed {
            version: String::new(),
        },
    }
}

/// Runs `attempt`, retrying a few times on ETXTBSY ("Text file busy"). A
/// program installed or written just before it is probed can still be open for
/// writing in a concurrently forked child for a moment; this clears on its own
/// within milliseconds and must not read as "tool missing".
pub(crate) fn retry_busy<T>(mut attempt: impl FnMut() -> std::io::Result<T>) -> std::io::Result<T> {
    const MAX_ATTEMPTS: u32 = 8;
    let mut tries = 0;
    loop {
        match attempt() {
            Err(e) if tries + 1 < MAX_ATTEMPTS && is_text_file_busy(&e) => {
                tries += 1;
                std::thread::sleep(Duration::from_millis(25 * u64::from(tries)));
            }
            other => return other,
        }
    }
}

fn is_text_file_busy(e: &std::io::Error) -> bool {
    // ETXTBSY is 26 on Linux, macOS and the BSDs.
    cfg!(unix) && e.raw_os_error() == Some(26)
}

/// What `<program> --version` came to.
enum VersionAnswer {
    /// Exit 0, with this output (stdout, else stderr).
    Output(String),
    /// It would not start or exited non-zero.
    Failed,
    /// It did not answer within the budget.
    TimedOut,
}

/// Runs `<path> --version`, waiting at most `budget`.
fn run_version(path: &Path, flag: &str, budget: Duration) -> VersionAnswer {
    let mut child = match retry_busy(|| {
        Command::new(path)
            .arg(flag)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }) {
        Ok(child) => child,
        Err(_) => return VersionAnswer::Failed,
    };
    let started = Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if started.elapsed() < budget => {
                std::thread::sleep(Duration::from_millis(10));
            }
            Ok(None) => {
                let _ = child.kill();
                let _ = child.wait();
                return VersionAnswer::TimedOut;
            }
            Err(_) => {
                let _ = child.kill();
                let _ = child.wait();
                return VersionAnswer::Failed;
            }
        }
    };
    if !status.success() {
        return VersionAnswer::Failed;
    }
    let mut out = String::new();
    if let Some(mut stdout) = child.stdout.take() {
        let _ = stdout.read_to_string(&mut out);
    }
    if out.trim().is_empty() {
        if let Some(mut stderr) = child.stderr.take() {
            let _ = stderr.read_to_string(&mut out);
        }
    }
    VersionAnswer::Output(out)
}

/// The first `digits.digits...` token of `output`
/// (`gpg (GnuPG) 2.4.4`, `GNU bash, version 5.2.26(1)-release`,
/// `PowerShell 7.4.1`).
pub fn first_version(output: &str) -> Option<String> {
    output.split_whitespace().find_map(|token| {
        let version: String = token
            .chars()
            .take_while(|c| c.is_ascii_digit() || *c == '.')
            .collect();
        let version = version.trim_end_matches('.');
        (version.starts_with(|c: char| c.is_ascii_digit()) && version.contains('.'))
            .then(|| version.to_string())
    })
}

// ---------------------------------------------------------------------
// Pure path resolution
// ---------------------------------------------------------------------

/// `PATHEXT` split into extensions (`.EXE`), the Windows default when unset.
pub fn pathext_from_env(value: Option<&str>) -> Vec<String> {
    let value = value
        .filter(|v| !v.trim().is_empty())
        .unwrap_or(".COM;.EXE;.BAT;.CMD");
    value
        .split(';')
        .map(str::trim)
        .filter(|e| !e.is_empty())
        .map(|e| {
            if e.starts_with('.') {
                e.to_string()
            } else {
                format!(".{e}")
            }
        })
        .collect()
}

/// Finds `name` the way the OS shell would: an absolute or relative path
/// with a separator is checked as is, a bare name is looked up in `dirs` in
/// order. With `windows_style`, a name without one of the `pathext`
/// extensions is tried with each (an extensionless file is not runnable
/// there); otherwise the file must carry an execute bit (Unix).
pub fn resolve_program(
    name: &str,
    dirs: &[PathBuf],
    pathext: &[String],
    windows_style: bool,
) -> Option<PathBuf> {
    let candidates = candidate_names(name, pathext, windows_style);
    if name.contains('/') || (windows_style && name.contains('\\')) {
        return candidates
            .iter()
            .map(PathBuf::from)
            .find(|p| is_runnable(p, windows_style));
    }
    dirs.iter()
        .filter(|d| !d.as_os_str().is_empty())
        .flat_map(|dir| candidates.iter().map(move |c| dir.join(c)))
        .find(|p| is_runnable(p, windows_style))
}

fn candidate_names(name: &str, pathext: &[String], windows_style: bool) -> Vec<String> {
    if !windows_style {
        return vec![name.to_string()];
    }
    let lower = name.to_ascii_lowercase();
    if pathext
        .iter()
        .any(|ext| lower.ends_with(&ext.to_ascii_lowercase()))
    {
        return vec![name.to_string()];
    }
    pathext.iter().map(|ext| format!("{name}{ext}")).collect()
}

fn is_runnable(path: &Path, windows_style: bool) -> bool {
    let Ok(meta) = std::fs::metadata(path) else {
        return false;
    };
    if !meta.is_file() {
        return false;
    }
    #[cfg(unix)]
    if !windows_style {
        use std::os::unix::fs::PermissionsExt;
        return meta.permissions().mode() & 0o111 != 0;
    }
    let _ = windows_style;
    true
}

/// Git for Windows' `usr/bin` (where `gpg`, `ssh-keygen`, `bash` and `sh`
/// live) from `git --exec-path`, which reads
/// `<root>\mingw64\libexec\git-core` (`mingw32`, `clangarm64`, `clang64`, `ucrt64` alike): the
/// directory above the `mingw*` one, then `usr\bin`. A string function, so
/// it is the same on every OS and testable anywhere.
pub fn git_for_windows_usr_bin(exec_path: &str) -> Option<PathBuf> {
    let sep = if exec_path.contains('\\') { '\\' } else { '/' };
    let segments: Vec<&str> = exec_path.split(['/', '\\']).collect();
    let at = segments.iter().position(|s| {
        ["mingw64", "mingw32", "clangarm64", "clang64", "ucrt64"]
            .iter()
            .any(|m| s.eq_ignore_ascii_case(m))
    })?;
    if at == 0 {
        return None;
    }
    let mut root = segments[..at].join(&sep.to_string());
    if root.is_empty() {
        root.push(sep);
    }
    let sep = sep.to_string();
    Some(PathBuf::from(
        [root.as_str(), "usr", "bin"].join(sep.as_str()),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(unix)]
    fn write_exec(path: &Path, body: &str, mode: u32) {
        use std::os::unix::fs::PermissionsExt;
        std::fs::write(path, body).unwrap();
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode)).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn a_busy_text_file_is_retried_but_other_errors_are_not() {
        let mut calls = 0;
        let r = retry_busy(|| {
            calls += 1;
            if calls < 3 {
                Err(std::io::Error::from_raw_os_error(26))
            } else {
                Ok(calls)
            }
        });
        assert_eq!(r.unwrap(), 3);
        let mut calls = 0;
        let r: std::io::Result<()> = retry_busy(|| {
            calls += 1;
            Err(std::io::Error::from_raw_os_error(2))
        });
        assert!(r.is_err());
        assert_eq!(calls, 1);
        let mut calls = 0;
        let r: std::io::Result<()> = retry_busy(|| {
            calls += 1;
            Err(std::io::Error::from_raw_os_error(26))
        });
        assert!(r.is_err());
        assert_eq!(calls, 8);
    }

    #[test]
    fn overrides_parse() {
        assert_eq!(
            parse_override(Tool::Gpg, Some(" MISSING ")),
            Some(ToolStatus::Missing)
        );
        assert!(parse_override(Tool::Gpg, Some("present"))
            .unwrap()
            .is_usable());
        assert_eq!(
            parse_override(Tool::GitLfs, Some("2.9.0")),
            Some(ToolStatus::TooOld {
                version: "2.9.0".into(),
                min: "3.0".into()
            })
        );
        assert_eq!(
            parse_override(Tool::GitLfs, Some("3.5.1")),
            Some(ToolStatus::Installed {
                version: "3.5.1".into()
            })
        );
        assert!(matches!(
            parse_override(Tool::Python, Some("too_old")),
            Some(ToolStatus::TooOld { .. })
        ));
        assert!(matches!(
            parse_override(Tool::SshKeygen, Some("not_configured")),
            Some(ToolStatus::NotConfigured { .. })
        ));
        assert_eq!(parse_override(Tool::Gpg, Some("banana")), None);
        assert_eq!(parse_override(Tool::Gpg, None), None);
    }

    #[test]
    fn a_forced_probe_answers_without_asking_and_for_that_tool_only() {
        let probe = ToolProbe::forced(Tool::Gpg, ToolStatus::Missing).with_forced(
            Tool::Bash,
            ToolStatus::Installed {
                version: "5.2".into(),
            },
        );
        let root = Path::new(".");
        assert_eq!(probe.get(Tool::Gpg, root), ToolStatus::Missing);
        assert_eq!(probe.fresh(Tool::Gpg, root), ToolStatus::Missing);
        assert!(probe.get(Tool::Bash, root).is_usable());
    }

    #[test]
    fn check_all_lists_every_tool_with_os_hints() {
        let mut probe = ToolProbe::default();
        for tool in Tool::ALL {
            probe = probe.with_forced(tool, ToolStatus::Missing);
        }
        let all = probe.check_all(Path::new("."), false);
        assert_eq!(all.tools.len(), Tool::ALL.len());
        let lfs = all.get(Tool::GitLfs).unwrap();
        assert_eq!(lfs.status, ToolStatus::Missing);
        assert!(lfs.hint_key.starts_with("tool_hint_git_lfs_"));
        assert!(!all.is_usable(Tool::Gpg));
    }

    #[test]
    fn lfs_conversions_round_trip() {
        for b in [
            LfsBinary::Missing,
            LfsBinary::Installed {
                version: "3.5.1".into(),
            },
            LfsBinary::TooOld {
                version: "2.9".into(),
            },
        ] {
            assert_eq!(lfs_of_status(&status_of_lfs(&b)), b);
        }
    }

    #[test]
    fn versions_are_found_in_real_outputs() {
        assert_eq!(
            first_version("gpg (GnuPG) 2.4.4\nlibgcrypt 1.10"),
            Some("2.4.4".into())
        );
        assert_eq!(
            first_version("GNU bash, version 5.2.26(1)-release (x86_64-pc-linux-gnu)"),
            Some("5.2.26".into())
        );
        assert_eq!(first_version("PowerShell 7.4.1"), Some("7.4.1".into()));
        assert_eq!(first_version("Python 3.12.1"), Some("3.12.1".into()));
        assert_eq!(first_version("no digits here"), None);
    }

    #[test]
    fn signing_config_is_parsed() {
        let cfg = parse_signing_config(
            "gpg.format ssh\nuser.signingkey ~/.ssh/id.pub\ngpg.ssh.program C:/x/ssh-keygen.exe\n\
             gpg.program gpg2\n",
        );
        assert_eq!(cfg.format, SigningFormat::Ssh);
        assert_eq!(cfg.key.as_deref(), Some("~/.ssh/id.pub"));
        assert_eq!(
            cfg.program_for(Tool::SshKeygen),
            Some("C:/x/ssh-keygen.exe")
        );
        assert_eq!(cfg.program_for(Tool::Gpg), Some("gpg2"));
        assert_eq!(cfg.program_for(Tool::Gpgsm), None);
        assert_eq!(parse_signing_config("").format, SigningFormat::OpenPgp);
    }

    #[test]
    fn pathext_defaults_and_normalises() {
        assert_eq!(pathext_from_env(None), vec![".COM", ".EXE", ".BAT", ".CMD"]);
        assert_eq!(pathext_from_env(Some("exe; .cmd;")), vec![".exe", ".cmd"]);
    }

    #[test]
    fn windows_style_resolution_appends_pathext() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("gpg.EXE"), "").unwrap();
        std::fs::write(dir.path().join("plain"), "").unwrap();
        let dirs = vec![PathBuf::from("/nonexistent"), dir.path().to_path_buf()];
        let ext = pathext_from_env(None);
        assert_eq!(
            resolve_program("gpg", &dirs, &ext, true),
            Some(dir.path().join("gpg.EXE"))
        );
        // Already carries an extension.
        assert_eq!(
            resolve_program("gpg.EXE", &dirs, &ext, true),
            Some(dir.path().join("gpg.EXE"))
        );
        // An extensionless file is not runnable on Windows.
        assert_eq!(resolve_program("plain", &dirs, &ext, true), None);
        assert_eq!(resolve_program("absent", &dirs, &ext, true), None);
    }

    #[cfg(unix)]
    #[test]
    fn unix_resolution_needs_the_execute_bit_and_honours_path_order() {
        let first = tempfile::tempdir().unwrap();
        let second = tempfile::tempdir().unwrap();
        write_exec(&first.path().join("tool"), "", 0o644);
        write_exec(&second.path().join("tool"), "", 0o755);
        let dirs = vec![first.path().to_path_buf(), second.path().to_path_buf()];
        assert_eq!(
            resolve_program("tool", &dirs, &[], false),
            Some(second.path().join("tool"))
        );
        write_exec(&first.path().join("tool"), "", 0o755);
        assert_eq!(
            resolve_program("tool", &dirs, &[], false),
            Some(first.path().join("tool"))
        );
        // A path with a separator skips the search.
        let direct = second.path().join("tool");
        assert_eq!(
            resolve_program(direct.to_str().unwrap(), &[], &[], false),
            Some(direct)
        );
    }

    #[cfg(unix)]
    #[test]
    fn a_resolved_program_is_asked_for_its_version() {
        let dir = tempfile::tempdir().unwrap();
        let gpg = dir.path().join("gpg");
        write_exec(&gpg, "#!/bin/sh\necho 'gpg (GnuPG) 2.4.4'\n", 0o755);
        assert_eq!(
            probe_resolved(Tool::Gpg, &gpg),
            ToolStatus::Installed {
                version: "2.4.4".into()
            }
        );
        // Too old against the tool's minimum.
        let lfs = dir.path().join("lfs");
        write_exec(&lfs, "#!/bin/sh\necho 'git-lfs/2.1.0'\n", 0o755);
        assert!(matches!(
            probe_resolved(Tool::GitLfs, &lfs),
            ToolStatus::TooOld { .. }
        ));
        // A program that fails its version check is not usable.
        let broken = dir.path().join("broken");
        write_exec(&broken, "#!/bin/sh\nexit 3\n", 0o755);
        assert_eq!(probe_resolved(Tool::Bash, &broken), ToolStatus::Missing);
        // No version output expected: resolving is enough.
        assert!(probe_resolved(Tool::SshKeygen, &broken).is_usable());
    }

    #[cfg(unix)]
    #[test]
    fn a_slow_version_check_is_not_reported_missing() {
        let dir = tempfile::tempdir().unwrap();
        let slow = dir.path().join("slow");
        write_exec(&slow, "#!/bin/sh\nexec sleep 30\n", 0o755);
        let started = Instant::now();
        // Past the budget: slow, so installed with no known version.
        assert_eq!(
            probe_resolved_within(Tool::Bash, &slow, Duration::from_millis(300)),
            ToolStatus::Installed {
                version: String::new()
            }
        );
        assert!(started.elapsed() < Duration::from_secs(10));
        // Within a generous budget the same shape of program still answers.
        let late = dir.path().join("late");
        write_exec(
            &late,
            "#!/bin/sh\nsleep 1\necho 'gpg (GnuPG) 2.4.4'\n",
            0o755,
        );
        assert_eq!(
            probe_resolved_within(Tool::Gpg, &late, Duration::from_secs(30)),
            ToolStatus::Installed {
                version: "2.4.4".into()
            }
        );
    }

    #[test]
    fn git_for_windows_usr_bin_comes_from_the_exec_path() {
        assert_eq!(
            git_for_windows_usr_bin(r"C:\Program Files\Git\mingw64\libexec\git-core"),
            Some(PathBuf::from(r"C:\Program Files\Git\usr\bin"))
        );
        assert_eq!(
            git_for_windows_usr_bin("C:/Program Files/Git/mingw64/libexec/git-core"),
            Some(PathBuf::from("C:/Program Files/Git/usr/bin"))
        );
        assert_eq!(
            git_for_windows_usr_bin(r"D:\Tools\Git\CLANGARM64\libexec\git-core"),
            Some(PathBuf::from(r"D:\Tools\Git\usr\bin"))
        );
        assert_eq!(
            git_for_windows_usr_bin("C:/Program Files/Git/ucrt64/libexec/git-core"),
            Some(PathBuf::from("C:/Program Files/Git/usr/bin")),
            "recent Git for Windows builds use ucrt64"
        );
        assert_eq!(
            git_for_windows_usr_bin("/usr/lib/git-core"),
            None,
            "a Linux exec path has no Git for Windows layout"
        );
        assert_eq!(git_for_windows_usr_bin(""), None);
    }
}

#[cfg(test)]
mod backend_tests {
    use super::*;
    use crate::SystemGitBackend;
    use gitpoulpi_core::GitBackend;

    #[test]
    fn the_backend_reports_every_tool_and_honours_without_lfs() {
        let backend = SystemGitBackend::without_lfs();
        let repo = RepoRef::from_root(Path::new("."));
        let tools = backend.rewrite_tools(&repo, false).unwrap();
        assert_eq!(tools.tools.len(), Tool::ALL.len());
        assert_eq!(tools.get(Tool::GitLfs).unwrap().status, ToolStatus::Missing);
        // The embedded Lua is not a tool; only an installed one (external
        // Lua scripts) is probed.
        assert!(tools.get(Tool::Lua).is_some());
    }
}
