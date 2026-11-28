//! The programs a file's context menu hands work to: an external editor,
//! an external diff tool and the OS file explorer.
//!
//! Each is a *command template* from `[general]` in `config.toml`
//! (`external_editor`, `external_diff_tool`, `file_explorer`), with
//! `{placeholders}` replaced per file. Templates are published here by
//! `install` -- the same pattern `live_settings::install_auto_prune` uses --
//! because the menu code that launches them has no `Config` in hand.
//!
//! # Never blocking, never a console window
//!
//! Every launch is `Command::spawn` with null stdio and (on Windows)
//! `CREATE_NO_WINDOW`; the child is reaped on a throwaway thread. The input
//! loop never waits on a child. Launching that needs git work first (the
//! external diff tool's temp files) runs entirely on a worker thread and
//! reports failures through `take_late_errors`, which the loop drains onto
//! the status line.
//!
//! # Template syntax
//!
//! The first word is the program. The rest are arguments split on spaces,
//! with `"double quotes"` grouping a word that contains spaces. Placeholder
//! values are substituted *after* splitting, so a path with spaces stays one
//! argument. On Windows the argument text after the program is passed to
//! the process verbatim (`raw_arg`), which is what makes
//! `explorer.exe /select,"{path}"` work: Explorer parses its own command
//! line and rejects the re-quoted form Rust would otherwise build.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Mutex, RwLock};

use gitpoulpi_config::Config;
use gitpoulpi_i18n::msg::merge as tr;

pub(crate) use gitpoulpi_link::external_tools::{
    build_command, diff_launch, editor_launch, editor_name, explorer_launch, find_on_path,
    merge_launch, native_path, Launch,
};

/// The three templates, as configured.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct ExternalTools {
    pub(crate) editor: String,
    pub(crate) diff_tool: String,
    /// The conflict cards' merge tool (empty: `git mergetool`).
    pub(crate) merge_tool: String,
    pub(crate) file_explorer: String,
}

impl ExternalTools {
    pub(crate) fn from_config(config: &Config) -> Self {
        Self {
            editor: config.general.external_editor.clone(),
            diff_tool: config.general.external_diff_tool.clone(),
            merge_tool: config.general.external_merge_tool.clone(),
            file_explorer: config.general.file_explorer.clone(),
        }
    }
}

static TOOLS: RwLock<Option<ExternalTools>> = RwLock::new(None);

/// Publishes the configured templates (startup, and after a settings
/// write).
pub(crate) fn install(config: &Config) {
    if let Ok(mut slot) = TOOLS.write() {
        *slot = Some(ExternalTools::from_config(config));
    }
}

/// The templates in force; the built-in defaults before `install` ran.
pub(crate) fn current() -> ExternalTools {
    if let Ok(slot) = TOOLS.read() {
        if let Some(tools) = slot.as_ref() {
            return tools.clone();
        }
    }
    ExternalTools::from_config(&Config::default())
}

type SpawnHook = Box<dyn Fn(&Launch) -> std::io::Result<()> + Send + Sync>;
static SPAWN_HOOK: RwLock<Option<SpawnHook>> = RwLock::new(None);

/// Test seam: route every launch to `hook` instead of a real process.
#[cfg(test)]
pub(crate) fn set_spawn_hook(hook: Option<SpawnHook>) {
    *SPAWN_HOOK.write().unwrap() = hook;
}

/// Starts the command without waiting for it.
pub(crate) fn spawn(launch: &Launch) -> Result<(), String> {
    if let Ok(hook) = SPAWN_HOOK.read() {
        if let Some(hook) = hook.as_ref() {
            return hook(launch).map_err(|e| format!("{}: {e}", launch.program));
        }
    }
    gitpoulpi_link::external_tools::spawn(launch)
}

/// Opens `path` with the OS's default program.
pub(crate) fn open_default(path: &Path) -> Result<(), String> {
    let target = native_path(path);
    let launch = if cfg!(windows) {
        Launch {
            program: "cmd".into(),
            args: vec!["/c".into(), "start".into(), String::new(), target.clone()],
            raw_args: format!("/c start \"\" \"{target}\""), // i18n-english
            cwd: None,
        }
    } else if cfg!(target_os = "macos") {
        Launch {
            program: "open".into(),
            args: vec![target],
            raw_args: String::new(),
            cwd: None,
        }
    } else {
        // Linux: the default application from the same lookup as "Open
        // with", on a worker thread (it reads the .desktop files). Not
        // `xdg-open` first: it starts a terminal program (`nvim.desktop`,
        // Omarchy's text default) in the terminal it was called from, here
        // the TUI's own with no stdin, so nothing ever shows. The lookup runs
        // such a program in `$TERMINAL`; `xdg-open` stays the fallback when
        // the lookup finds nothing.
        let path = path.to_path_buf();
        std::thread::spawn(move || {
            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            let result = match crate::open_with::default_app_blocking(&name) {
                Some(app) => crate::image_editor::run_list(&[app.entry()], 0, &path).map(|_| ()),
                None => spawn(&Launch {
                    program: "xdg-open".into(),
                    args: vec![target],
                    raw_args: String::new(),
                    cwd: None,
                }),
            };
            if let Err(why) = result {
                report_late_error(why);
            }
        });
        return Ok(());
    };
    spawn(&launch)
}

/// Opens `path` in the configured editor (VS Code when on `PATH`, else the
/// default program when the template is empty).
pub(crate) fn open_in_editor(template: &str, path: &Path) -> Result<(), String> {
    let template = template.trim();
    if !template.is_empty() {
        return spawn(&editor_launch(template, path)?);
    }
    match find_on_path("code") {
        Some(code) => {
            let program = code.to_string_lossy().into_owned();
            spawn(&editor_launch(&format!("\"{program}\""), path)?)
        }
        None => open_default(path),
    }
}

/// Shows `path` in the file explorer.
pub(crate) fn show_in_folder(template: &str, path: &Path) -> Result<(), String> {
    spawn(&explorer_launch(template, path)?)
}

static LATE_ERRORS: Mutex<Vec<String>> = Mutex::new(Vec::new());

/// Records a failure from a worker thread for the status line.
pub(crate) fn report_late_error(message: String) {
    if let Ok(mut errors) = LATE_ERRORS.lock() {
        errors.push(message);
    }
    crate::redraw::poke();
}

/// Drains the failures workers reported since the last call.
pub(crate) fn take_late_errors() -> Vec<String> {
    LATE_ERRORS
        .lock()
        .map(|mut e| std::mem::take(&mut *e))
        .unwrap_or_default()
}

fn run_git(root: &Path, args: &[&str]) -> Result<Vec<u8>, String> {
    let mut command = Command::new("git");
    command.current_dir(root).args(args);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x0800_0000);
    }
    let out = command
        .stdin(Stdio::null())
        .output()
        .map_err(gitpoulpi_i18n::msg::tool_error::git_failed)?;
    if out.status.success() {
        Ok(out.stdout)
    } else {
        Err(String::from_utf8_lossy(&out.stderr).trim().to_string())
    }
}

/// Opens the file's change in the external diff tool, entirely on a worker
/// thread (it reads blobs with git first). `staged` compares HEAD with the
/// index, otherwise the index with the working-tree file. Failures arrive
/// through `take_late_errors`.
pub(crate) fn open_external_diff(root: PathBuf, path: PathBuf, staged: bool, template: String) {
    std::thread::spawn(move || {
        if let Err(why) = open_external_diff_blocking(&root, &path, staged, &template) {
            report_late_error(why);
        }
    });
}

fn open_external_diff_blocking(
    root: &Path,
    path: &Path,
    staged: bool,
    template: &str,
) -> Result<(), String> {
    let rel = path.to_string_lossy().replace('\\', "/");
    if template.trim().is_empty() {
        let mut args = vec!["difftool", "--no-prompt"];
        if staged {
            args.push("--cached");
        }
        args.push("--");
        args.push(&rel);
        // `difftool` outlives this call, so it is a detached child.
        return spawn(&Launch {
            program: "git".into(),
            args: args.iter().map(|s| s.to_string()).collect(),
            raw_args: String::new(),
            cwd: Some(root.to_path_buf()),
        });
    }
    let dir = std::env::temp_dir().join(format!("gitpoulpi-diff-{}", std::process::id()));
    std::fs::create_dir_all(&dir).map_err(gitpoulpi_i18n::msg::tool_error::temp_dir)?;
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "file".into());
    let local = dir.join(format!("LOCAL_{name}"));
    let blob = if staged {
        format!("HEAD:{rel}")
    } else {
        format!(":{rel}")
    };
    // A file new to the index/HEAD has nothing to compare against.
    let local_bytes = run_git(root, &["show", &blob]).unwrap_or_default();
    std::fs::write(&local, local_bytes).map_err(gitpoulpi_i18n::msg::tool_error::temp_file)?;
    let remote = if staged {
        let remote = dir.join(format!("INDEX_{name}"));
        let bytes = run_git(root, &["show", &format!(":{rel}")])?;
        std::fs::write(&remote, bytes).map_err(gitpoulpi_i18n::msg::tool_error::temp_file)?;
        remote
    } else {
        root.join(path)
    };
    let mut launch = diff_launch(template, &local, &remote)?;
    launch.cwd = Some(root.to_path_buf());
    spawn(&launch)
}

/// Hands a conflicted file to the merge tool, entirely on a worker thread.
/// With no template that is `git mergetool` (the tool configured in git,
/// which stages the file when it succeeds); with one, the stages are
/// written to temp files, the tool runs on them and, when it exits
/// successfully, `{merged}` replaces the working-tree file and is staged.
/// Failures arrive through `take_late_errors`.
pub(crate) fn open_external_merge(root: PathBuf, path: PathBuf, template: String) {
    std::thread::spawn(move || {
        if let Err(why) = open_external_merge_blocking(&root, &path, &template) {
            report_late_error(why);
        }
        crate::redraw::poke();
    });
}

fn open_external_merge_blocking(root: &Path, path: &Path, template: &str) -> Result<(), String> {
    let rel = path.to_string_lossy().replace('\\', "/");
    if template.trim().is_empty() {
        return spawn(&Launch {
            program: "git".into(),
            args: ["mergetool", "--no-prompt", "--", &rel]
                .iter()
                .map(|s| s.to_string())
                .collect(),
            raw_args: String::new(),
            cwd: Some(root.to_path_buf()),
        });
    }
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "file".into());
    let dir = std::env::temp_dir().join(format!("gitpoulpi-merge-{}", std::process::id()));
    std::fs::create_dir_all(&dir).map_err(tr::ext_prepare_failed)?;
    // A stage that does not exist (added on one side only) is an empty
    // file, as `git mergetool` does.
    let stage = |n: u8, prefix: &str| -> Result<PathBuf, String> {
        let file = dir.join(format!("{prefix}_{name}"));
        let bytes = run_git(root, &["show", &format!(":{n}:{rel}")]).unwrap_or_default();
        std::fs::write(&file, bytes).map_err(tr::ext_prepare_failed)?;
        Ok(file)
    };
    let (base, local, remote) = (stage(1, "BASE")?, stage(2, "LOCAL")?, stage(3, "REMOTE")?);
    let merged = dir.join(format!("MERGED_{name}"));
    let worktree = root.join(path);
    std::fs::copy(&worktree, &merged)
        .or_else(|_| std::fs::write(&merged, b"").map(|()| 0))
        .map_err(tr::ext_prepare_failed)?;
    let mut launch = merge_launch(template, &base, &local, &remote, &merged)?;
    launch.cwd = Some(root.to_path_buf());
    let status = build_command(&launch)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|e| tr::ext_start_failed(&launch.program, e))?;
    if !status.success() {
        return Err(tr::ext_failed(&launch.program, status, &rel));
    }
    std::fs::copy(&merged, &worktree).map_err(tr::ext_prepare_failed)?;
    run_git(root, &["add", "--", &rel])?;
    let _ = std::fs::remove_dir_all(&dir);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A configured tool resolves a real conflicted file: it writes `{merged}`
    /// from `{remote}` and exits 0, so the file is B's and staged; a tool that
    /// fails leaves it conflicted.
    #[cfg(unix)]
    #[test]
    fn the_merge_tool_resolves_the_file_and_a_failing_one_leaves_it_conflicted() {
        let fixture = gitpoulpi_testkit::ConflictFixture::text();
        let path = Path::new(fixture.path());
        let theirs = fixture.stage_bytes(3, fixture.path()).unwrap();
        let root = fixture.root();
        let err = open_external_merge_blocking(root, path, "sh -c \"exit 3\"").unwrap_err();
        assert!(err.contains("sh"), "{err}");
        assert!(!fixture
            .git(&["diff", "--name-only", "--diff-filter=U"])
            .is_empty());
        open_external_merge_blocking(root, path, "sh -c \"cat '{remote}' > '{merged}'\"").unwrap();
        assert_eq!(std::fs::read(root.join(path)).unwrap(), theirs);
        assert_eq!(fixture.git(&["diff", "--name-only", "--diff-filter=U"]), "");
    }

    /// The spawn hook is process-global, so the tests that set it take turns.
    static HOOK_LOCK: Mutex<()> = Mutex::new(());

    fn recorded(run: impl FnOnce() -> Result<(), String>) -> (Result<(), String>, Vec<Launch>) {
        let _turn = HOOK_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let seen = std::sync::Arc::new(Mutex::new(Vec::new()));
        let sink = std::sync::Arc::clone(&seen);
        set_spawn_hook(Some(Box::new(move |l| {
            sink.lock().unwrap().push(l.clone());
            Ok(())
        })));
        let result = run();
        set_spawn_hook(None);
        let launches = seen.lock().unwrap().clone();
        (result, launches)
    }

    #[test]
    fn show_in_folder_and_the_editor_launch_the_expanded_templates() {
        let file = Path::new("repo").join("a b.txt");
        let (result, launches) =
            recorded(|| show_in_folder("opener --reveal \"{path}\" {dir}", &file));
        result.unwrap();
        assert_eq!(launches.len(), 1);
        assert_eq!(launches[0].program, "opener");
        assert_eq!(launches[0].args[0], "--reveal");
        assert!(launches[0].args[1].ends_with("a b.txt"));
        assert!(launches[0].args[2].ends_with("repo"));

        let (result, launches) = recorded(|| open_in_editor("myedit -g", &file));
        result.unwrap();
        assert_eq!(launches[0].program, "myedit");
        assert_eq!(launches[0].args[0], "-g");
        assert!(launches[0].args[1].ends_with("a b.txt"));
    }

    #[test]
    fn a_failing_launch_is_reported_with_the_program_name() {
        let _turn = HOOK_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        set_spawn_hook(Some(Box::new(|_| {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "no such program",
            ))
        })));
        let result = show_in_folder("nope {path}", Path::new("x"));
        set_spawn_hook(None);
        let why = result.unwrap_err();
        assert!(
            why.contains("nope") && why.contains("no such program"),
            "{why}"
        );
    }

    #[test]
    fn the_default_explorer_template_matches_the_platform() {
        let t = ExternalTools::from_config(&Config::default()).file_explorer;
        if cfg!(windows) {
            assert_eq!(t, "explorer.exe /select,\"{path}\"");
        } else if cfg!(target_os = "macos") {
            assert!(t.starts_with("open -R"));
        } else {
            assert!(t.starts_with("xdg-open"));
        }
    }
}
