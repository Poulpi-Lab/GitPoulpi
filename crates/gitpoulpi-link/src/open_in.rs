//! The "Open in" launcher, shared by every frontend: the facts a command
//! substitutes (`${repo}`, `${branch}`, `${web_url}`), turning a configured
//! entry into the program and arguments to run for a folder, and spawning it
//! without ever waiting on it or touching a terminal.
//!
//! A frontend owns the *list* of entries (`ui.open_in`, detection, the
//! default) and calls [`run_entry`] with the chosen one: it returns the line
//! to say at once, or the message of a failure known before spawning. A
//! launcher that fails shortly after starting is reported through the
//! `report` callback, on a watcher thread. [`prepare`] alone gives the
//! command without spawning it (a preview, a test).

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use gitpoulpi_config::open_in::{self, OpenInEntry};
use gitpoulpi_i18n::msg;

/// What `${repo}` and `${branch}` stand for on one tab.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RepoInfo {
    /// The repository root (differs from the tab folder for a worktree).
    pub repo: String,
    /// The checked-out branch, empty when detached or unknown.
    pub branch: String,
    /// The origin's web page when it is a GitHub / GitLab project, else empty.
    pub web_url: String,
}

/// Removes `.`/`..` components without touching the disk.
fn lexical_normalize(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for c in path.components() {
        match c {
            std::path::Component::ParentDir => {
                out.pop();
            }
            std::path::Component::CurDir => {}
            other => out.push(other.as_os_str()),
        }
    }
    out
}

/// Reads the facts from the files under `.git`: a plain repository, or a
/// linked worktree whose `.git` is a `gitdir: ...` file. `web_url_of` maps
/// the origin's URL to its web page (`None` for a host the frontend does not
/// know), which only the frontend can say: it holds the configured
/// integrations.
pub fn read_repo_info(path: &Path, web_url_of: &dyn Fn(&str) -> Option<String>) -> RepoInfo {
    let folder = path.to_string_lossy().to_string();
    let dot_git = path.join(".git");
    let (git_dir, repo) = if dot_git.is_dir() {
        (dot_git, folder.clone())
    } else if let Ok(text) = std::fs::read_to_string(&dot_git) {
        let Some(dir) = text.trim().strip_prefix("gitdir:").map(str::trim) else {
            return RepoInfo {
                repo: folder,
                ..RepoInfo::default()
            };
        };
        let git_dir = lexical_normalize(&path.join(dir));
        // `commondir` points from `.git/worktrees/<n>` back to the shared
        // `.git`; the repository root is its parent.
        let repo = std::fs::read_to_string(git_dir.join("commondir"))
            .ok()
            .map(|c| lexical_normalize(&git_dir.join(c.trim())))
            .and_then(|common| common.parent().map(|p| p.to_string_lossy().to_string()))
            .unwrap_or_else(|| folder.clone());
        (git_dir, repo)
    } else {
        return RepoInfo {
            repo: folder,
            ..RepoInfo::default()
        };
    };
    let branch = std::fs::read_to_string(git_dir.join("HEAD"))
        .ok()
        .and_then(|head| {
            head.trim()
                .strip_prefix("ref:")
                .map(|r| r.trim().trim_start_matches("refs/heads/").to_string())
        })
        .unwrap_or_default();
    let common = std::fs::read_to_string(git_dir.join("commondir"))
        .ok()
        .map(|c| lexical_normalize(&git_dir.join(c.trim())))
        .unwrap_or_else(|| git_dir.clone());
    let web_url = origin_url(&common.join("config"))
        .and_then(|url| web_url_of(&url))
        .unwrap_or_default();
    RepoInfo {
        repo,
        branch,
        web_url,
    }
}

/// The `url` of `[remote "origin"]` in a git config file.
fn origin_url(config: &Path) -> Option<String> {
    let text = std::fs::read_to_string(config).ok()?;
    let mut in_origin = false;
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_origin = line.replace(' ', "") == "[remote\"origin\"]"; // i18n-english
        } else if in_origin {
            if let Some(v) = line.strip_prefix("url") {
                if let Some(v) = v.trim_start().strip_prefix('=') {
                    return Some(v.trim().to_string());
                }
            }
        }
    }
    None
}

/// Where `program` really is, for a spawn by full path. On Windows this is
/// the PATH + PATHEXT search `CreateProcess` does not do for `.cmd`/`.bat`
/// shims (`code`, `cursor`): `code` finds `code.cmd`. A name with a path
/// separator is taken relative to `cwd`. `windows` is a parameter so the
/// rules are testable on any OS; callers pass `cfg!(windows)`.
pub fn resolve_program(
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

/// How long the watcher thread waits for an early failure.
const WATCH: Duration = Duration::from_millis(1500);
/// How much of the launcher's stderr is kept.
const STDERR_KEEP: usize = 300;

/// One launch: what to run, in which folder, with which facts.
pub struct Launch<'a> {
    pub label: &'a str,
    pub program: &'a str,
    pub args: &'a [String],
    /// The command line after the program, passed verbatim on Windows when a
    /// program parses it itself (`--single-argument`); `args` otherwise.
    pub raw_args: Option<&'a str>,
    pub cwd: &'a Path,
    pub info: &'a RepoInfo,
}

fn program_stem(program: &str) -> String {
    let name = program.rsplit(['/', '\\']).next().unwrap_or(program);
    let lower = name.to_ascii_lowercase();
    lower.strip_suffix(".exe").unwrap_or(&lower).to_string()
}

/// `cmd /C start ...` opens its own window: it must not get
/// `CREATE_NO_WINDOW`.
#[cfg_attr(not(windows), allow(dead_code))] // only Windows spawns with it
pub fn is_cmd_start(program: &str, args: &[String]) -> bool {
    program_stem(program) == "cmd"
        && args
            .iter()
            .skip(1)
            .find(|a| !a.starts_with('/') || a.len() > 2)
            .is_some_and(|a| a.eq_ignore_ascii_case("start"))
}

/// Whether `program args` is a bare console shell (`cmd`, `pwsh`,
/// `powershell`, `wsl [--cd <dir>]`, `cmd /K <command>`): on Windows it is
/// started with `CREATE_NEW_CONSOLE` so it opens a window of its own in the
/// launch directory. Anything with other arguments (`cmd /C dir`) is a
/// command to run, not a terminal to show.
pub fn wants_new_console(program: &str, args: &[String]) -> bool {
    match program_stem(program).as_str() {
        "cmd" => args.is_empty() || args[0].eq_ignore_ascii_case("/K"),
        "pwsh" | "powershell" => args.is_empty(),
        "wsl" => args.is_empty() || (args.len() == 2 && args[0].eq_ignore_ascii_case("--cd")),
        _ => false,
    }
}

/// The late-error message for a launcher that ended with `code`, or `None`
/// when that is a success. `explorer.exe` always exits 1 even when it worked.
pub fn exit_failure(
    label: &str,
    program: &str,
    code: Option<i32>,
    stderr: &[u8],
) -> Option<String> {
    let Some(code) = code else {
        return Some(msg::open_in_run::stopped(label));
    };
    if code == 0 || (code == 1 && program_stem(program) == "explorer") {
        return None;
    }
    let text = String::from_utf8_lossy(stderr);
    let detail = text
        .lines()
        .map(str::trim)
        .find(|l| !l.is_empty())
        .unwrap_or_default();
    Some(if detail.is_empty() {
        msg::open_in_run::failed_silent(label, code)
    } else {
        msg::open_in_run::failed(label, code, detail)
    })
}

/// Spawns the launcher and returns at once. Immediate errors: the program is
/// not found, or the system refuses to start it. Later ones (a non-zero exit
/// within [`WATCH`]) go to `report`; the child is reaped either way.
pub fn start(launch: &Launch, report: fn(String)) -> Result<(), String> {
    use std::process::{Command, Stdio};
    let resolved = resolve_program(
        launch.program,
        std::env::var_os("PATH").as_deref(),
        std::env::var("PATHEXT").ok().as_deref(),
        cfg!(windows),
        launch.cwd,
    )
    .ok_or_else(|| msg::open_in_run::program_not_found(launch.label, launch.program))?;
    let mut command = Command::new(&resolved);
    command.current_dir(launch.cwd);
    match launch.raw_args {
        #[cfg(windows)]
        Some(raw) => {
            use std::os::windows::process::CommandExt;
            command.raw_arg(raw);
        }
        _ => {
            command.args(launch.args);
        }
    }
    command
        .env("GITPOULPI_CWD", launch.cwd)
        .env("GITPOULPI_REPO", &launch.info.repo)
        .env("GITPOULPI_BRANCH", &launch.info.branch);
    // A console shell opened "here" gets its own console window, with the
    // folder as its working directory: no `cmd /C start` in between, which
    // would parse the folder name as a command line.
    let own_console = cfg!(windows) && wants_new_console(launch.program, launch.args);
    if own_console {
        // Null handles would be kept by the new console and end the shell at
        // once; inherited console handles are replaced by the new console's.
        command
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());
    } else {
        command
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped());
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        if own_console {
            command.creation_flags(0x0000_0010); // CREATE_NEW_CONSOLE
        } else if !is_cmd_start(launch.program, launch.args) {
            command.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
        }
    }
    let mut child = command
        .spawn()
        .map_err(|err| msg::open_in_run::could_not_start(launch.label, launch.program, err))?;

    // Keep the first bytes of stderr, drain the rest so the child never blocks
    // on a full pipe (a GUI program holds it open for as long as it lives).
    let captured = Arc::new(Mutex::new(Vec::<u8>::new()));
    let drained = Arc::new(AtomicBool::new(false));
    if let Some(mut stderr) = child.stderr.take() {
        let (captured, drained) = (captured.clone(), drained.clone());
        std::thread::spawn(move || {
            use std::io::Read;
            let mut buf = [0u8; 4096];
            while let Ok(n) = stderr.read(&mut buf) {
                if n == 0 {
                    break;
                }
                if let Ok(mut kept) = captured.lock() {
                    let room = STDERR_KEEP.saturating_sub(kept.len());
                    kept.extend_from_slice(&buf[..n.min(room)]);
                }
            }
            drained.store(true, Ordering::Relaxed);
        });
    } else {
        drained.store(true, Ordering::Relaxed);
    }

    let (label, program) = (launch.label.to_string(), launch.program.to_string());
    std::thread::spawn(move || {
        let deadline = Instant::now() + WATCH;
        loop {
            match child.try_wait() {
                Ok(Some(status)) => {
                    // Give the stderr reader a moment to see the end.
                    let until = Instant::now() + Duration::from_millis(150);
                    while !drained.load(Ordering::Relaxed) && Instant::now() < until {
                        std::thread::sleep(Duration::from_millis(10));
                    }
                    let kept = captured.lock().map(|k| k.clone()).unwrap_or_default();
                    if let Some(message) = exit_failure(&label, &program, status.code(), &kept) {
                        report(message);
                    }
                    return;
                }
                Ok(None) if Instant::now() < deadline => {
                    std::thread::sleep(Duration::from_millis(20));
                }
                Ok(None) => break,
                Err(_) => return,
            }
        }
        // Still running: a terminal or editor that stays up. Reap it
        // whenever it ends.
        let _ = child.wait();
    });
    Ok(())
}

/// How `gitpoulpi_config::open_in::expand` words an unknown variable.
const UNKNOWN_VARIABLE_PREFIX: &str = "unknown variable "; // i18n-english

/// A run, prepared: the entry's label, its expanded argv and the facts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenInCommand {
    pub label: String,
    /// The program, then its arguments (never empty).
    pub argv: Vec<String>,
    pub info: RepoInfo,
}

/// Expands `entry`'s command for the folder `path` (placeholders `${cwd}`,
/// `${repo}`, `${name}`, `${branch}`, `${web_url}` and environment variables),
/// with `info` from [`read_repo_info`]. The result is what to spawn, without
/// spawning it. Errors name the entry: an unknown variable, a command that
/// does not parse, a `${web_url}` with no web page.
pub fn prepare(entry: &OpenInEntry, path: &Path, info: RepoInfo) -> Result<OpenInCommand, String> {
    let label = entry.label.clone();
    let cwd = path.to_string_lossy().to_string();
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    // A saved config may still hold an old `cmd /C start` preset text.
    let command = open_in::modernized_command(&entry.command).unwrap_or(&entry.command);
    let argv = open_in::expand(
        command,
        &open_in::OpenInVars {
            cwd: &cwd,
            repo: &info.repo,
            name: &name,
            branch: &info.branch,
            web_url: &info.web_url,
        },
        &|k| std::env::var(k).ok(),
    )
    .map_err(|e| match e.strip_prefix(UNKNOWN_VARIABLE_PREFIX) {
        Some(var) => msg::open_in_run::unknown_variable(&label, var),
        None => msg::open_in_run::bad_command(&label, e),
    })?;
    if argv.is_empty() {
        return Err(msg::open_in_run::bad_command(&label, String::new()));
    }
    if info.web_url.is_empty() && entry.command.contains("${web_url}") {
        return Err(msg::open_in_run::no_web_url(&label));
    }
    Ok(OpenInCommand { label, argv, info })
}

/// Runs the "Open in" entry for the folder `path`: the line to say at once
/// ("Opening in <label>..."). Errors that can be known before spawning come
/// back as `Err` (the folder is gone, the command does not expand, the program
/// is not found); a launcher that fails afterwards goes to `report` from a
/// watcher thread. Never waits, never calls git.
pub fn run_entry(
    entry: &OpenInEntry,
    path: &Path,
    info: RepoInfo,
    report: fn(String),
) -> Result<String, String> {
    if !path.is_dir() {
        return Err(msg::open_in_run::folder_gone(&entry.label, path.display()));
    }
    let run = prepare(entry, path, info)?;
    start(
        &Launch {
            label: &run.label,
            program: &run.argv[0],
            args: &run.argv[1..],
            raw_args: None,
            cwd: path,
            info: &run.info,
        },
        report,
    )?;
    Ok(msg::open_in_run::opening(&run.label))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(command: &str) -> OpenInEntry {
        OpenInEntry {
            label: "Editor".into(),
            command: command.into(),
            ..OpenInEntry::default()
        }
    }

    fn info() -> RepoInfo {
        RepoInfo {
            repo: "/src/repo".into(),
            branch: "main".into(),
            web_url: String::new(),
        }
    }

    #[test]
    fn placeholders_expand_into_one_argv() {
        let cmd = prepare(
            &entry("code --folder ${cwd} --repo ${repo} --name ${name} -b ${branch}"),
            Path::new("/src/repo-wt"),
            info(),
        )
        .unwrap();
        assert_eq!(cmd.label, "Editor");
        assert_eq!(
            cmd.argv,
            [
                "code",
                "--folder",
                "/src/repo-wt",
                "--repo",
                "/src/repo",
                "--name",
                "repo-wt",
                "-b",
                "main"
            ]
        );
    }

    #[test]
    fn a_bad_entry_names_itself() {
        let e = prepare(&entry("x ${nope}"), Path::new("/a"), info()).unwrap_err();
        assert!(e.contains("Editor") && e.contains("nope"), "{e}");
        let e = prepare(&entry("open ${web_url}"), Path::new("/a"), info()).unwrap_err();
        assert!(e.contains("Editor"), "{e}");
        let ok = RepoInfo {
            web_url: "https://host.example/o/r".into(),
            ..info()
        };
        let cmd = prepare(&entry("open ${web_url}"), Path::new("/a"), ok).unwrap();
        assert_eq!(cmd.argv, ["open", "https://host.example/o/r"]);
    }

    #[test]
    fn a_gone_folder_is_refused_before_spawning() {
        let e = run_entry(
            &entry("true"),
            Path::new("/definitely/not/here"),
            info(),
            |_| {},
        )
        .unwrap_err();
        assert!(e.contains("Editor"), "{e}");
    }

    #[test]
    fn console_shells_get_their_own_window_and_commands_do_not() {
        let args = |a: &[&str]| a.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        assert!(wants_new_console("cmd.exe", &[]));
        assert!(wants_new_console("pwsh", &[]));
        assert!(wants_new_console("wsl", &args(&["--cd", "x"])));
        assert!(!wants_new_console("cmd", &args(&["/C", "dir"])));
        assert!(!wants_new_console("code", &[]));
    }

    #[test]
    fn explorer_exiting_with_one_is_a_success() {
        assert_eq!(exit_failure("E", "explorer.exe", Some(1), b""), None);
        assert!(exit_failure("E", "code", Some(1), b"boom\n").is_some());
        assert!(exit_failure("E", "code", None, b"").is_some());
        assert_eq!(exit_failure("E", "code", Some(0), b""), None);
    }

    #[test]
    fn repo_facts_come_from_the_git_files() {
        let dir = tempfile::tempdir().unwrap();
        let git = dir.path().join(".git");
        std::fs::create_dir(&git).unwrap();
        std::fs::write(git.join("HEAD"), "ref: refs/heads/feat/x\n").unwrap();
        std::fs::write(
            git.join("config"),
            "[remote \"origin\"]\n\turl = git@host.example:o/r.git\n",
        )
        .unwrap();
        let facts = read_repo_info(dir.path(), &|url| Some(format!("web:{url}")));
        assert_eq!(facts.branch, "feat/x");
        assert_eq!(facts.web_url, "web:git@host.example:o/r.git");
        assert_eq!(facts.repo, dir.path().to_string_lossy());
        let plain = read_repo_info(dir.path(), &|_| None);
        assert_eq!(plain.web_url, "");
    }
}
