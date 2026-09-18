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

/// A command ready to run: program, split-and-substituted arguments, and
/// the same arguments as one verbatim string for Windows' `raw_arg`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Launch {
    pub(crate) program: String,
    pub(crate) args: Vec<String>,
    pub(crate) raw_args: String,
    pub(crate) cwd: Option<PathBuf>,
}

/// Splits `template` into words: spaces separate, a `"..."` span groups,
/// quotes are dropped. `raw` additionally keeps the original text of
/// everything after the first word, verbatim.
fn split_template(template: &str) -> (Vec<String>, String) {
    let mut words: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut in_word = false;
    let mut in_quotes = false;
    let mut raw_start: Option<usize> = None;
    for (i, c) in template.char_indices() {
        match c {
            '"' => {
                in_quotes = !in_quotes;
                in_word = true;
            }
            c if c.is_whitespace() && !in_quotes => {
                if in_word {
                    words.push(std::mem::take(&mut current));
                    in_word = false;
                    if words.len() == 1 {
                        raw_start = Some(i);
                    }
                }
            }
            c => {
                current.push(c);
                in_word = true;
            }
        }
    }
    if in_word {
        words.push(current);
    }
    let raw = raw_start
        .map(|i| template[i..].trim().to_string())
        .unwrap_or_default();
    (words, raw)
}

fn substitute(text: &str, vars: &[(&str, &str)]) -> String {
    let mut out = text.to_string();
    for (name, value) in vars {
        out = out.replace(&format!("{{{name}}}"), value);
    }
    out
}

/// Expands `template` with `vars` (`("file", "C:\\x\\a.txt")` fills
/// `{file}`). An empty template is an error the caller words for its own
/// setting.
pub(crate) fn expand(template: &str, vars: &[(&str, &str)]) -> Result<Launch, String> {
    let (words, raw) = split_template(template);
    let Some((program, rest)) = words.split_first() else {
        return Err(gitpoulpi_i18n::msg::tool_error::TEMPLATE_EMPTY.get().into());
    };
    Ok(Launch {
        program: substitute(program, vars),
        args: rest.iter().map(|w| substitute(w, vars)).collect(),
        raw_args: substitute(&raw, vars),
        cwd: None,
    })
}

/// A path as the tools want it: with the platform's separator.
pub(crate) fn native_path(path: &Path) -> String {
    let text = path.to_string_lossy().into_owned();
    if cfg!(windows) {
        text.replace('/', "\\")
    } else {
        text
    }
}

/// `{file}` is appended when the editor template never mentions it, so
/// `code` and `code -r` both work.
pub(crate) fn editor_launch(template: &str, file: &Path) -> Result<Launch, String> {
    let template = if template.contains("{file}") {
        template.to_string()
    } else {
        format!("{} \"{{file}}\"", template.trim())
    };
    expand(&template, &[("file", &native_path(file))])
}

pub(crate) fn explorer_launch(template: &str, file: &Path) -> Result<Launch, String> {
    let dir = file.parent().map(native_path).unwrap_or_default();
    expand(template, &[("path", &native_path(file)), ("dir", &dir)])
}

pub(crate) fn diff_launch(template: &str, local: &Path, remote: &Path) -> Result<Launch, String> {
    expand(
        template,
        &[
            ("local", &native_path(local)),
            ("remote", &native_path(remote)),
        ],
    )
}

/// The first `name` (or `name.exe`/`.cmd`/... on Windows) found on `PATH`.
pub(crate) fn find_on_path(name: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    let exts: Vec<String> = if cfg!(windows) {
        std::env::var("PATHEXT")
            .unwrap_or_else(|_| ".COM;.EXE;.BAT;.CMD".into())
            .split(';')
            .map(|e| e.to_ascii_lowercase())
            .collect()
    } else {
        Vec::new()
    };
    for dir in std::env::split_paths(&path) {
        let plain = dir.join(name);
        if cfg!(not(windows)) && plain.is_file() {
            return Some(plain);
        }
        for ext in &exts {
            let candidate = dir.join(format!("{name}{ext}"));
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}

/// What "Open in <editor>" is called, from the configured template: the
/// program's file name without its extension (`code` becomes "VS Code").
pub(crate) fn editor_name(template: &str) -> String {
    let template = template.trim();
    if template.is_empty() {
        return if find_on_path("code").is_some() {
            "VS Code".into()
        } else {
            gitpoulpi_i18n::msg::tool_error::SYSTEM_EDITOR.get().into()
        };
    }
    let (words, _) = split_template(template);
    let program = words.first().cloned().unwrap_or_default();
    // A config shared across machines may hold a Windows path on Unix (or the
    // reverse), and `Path` only splits on the host's separator: take the last
    // segment on both `/` and `\\`.
    let file = program.rsplit(['/', '\\']).next().unwrap_or(&program);
    let stem = match file.rfind('.') {
        Some(dot) if dot > 0 => &file[..dot],
        _ => file,
    }
    .to_string();
    if stem.eq_ignore_ascii_case("code") {
        "VS Code".into()
    } else if stem.is_empty() {
        gitpoulpi_i18n::msg::tool_error::SYSTEM_EDITOR.get().into()
    } else {
        stem
    }
}

type SpawnHook = Box<dyn Fn(&Launch) -> std::io::Result<()> + Send + Sync>;
static SPAWN_HOOK: RwLock<Option<SpawnHook>> = RwLock::new(None);

/// Test seam: route every launch to `hook` instead of a real process.
#[cfg(test)]
pub(crate) fn set_spawn_hook(hook: Option<SpawnHook>) {
    *SPAWN_HOOK.write().unwrap() = hook;
}

/// The process for `launch`: arguments (verbatim on Windows), working
/// directory, no console window.
fn build_command(launch: &Launch) -> Command {
    let mut command = Command::new(&launch.program);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        if launch.raw_args.is_empty() {
            command.args(&launch.args);
        } else {
            command.raw_arg(&launch.raw_args);
        }
        // CREATE_NO_WINDOW: no console flashes up for a GUI launch.
        command.creation_flags(0x0800_0000);
    }
    #[cfg(not(windows))]
    command.args(&launch.args);
    if let Some(cwd) = &launch.cwd {
        command.current_dir(cwd);
    }
    command
}

/// Starts the command without waiting for it.
pub(crate) fn spawn(launch: &Launch) -> Result<(), String> {
    if let Ok(hook) = SPAWN_HOOK.read() {
        if let Some(hook) = hook.as_ref() {
            return hook(launch).map_err(|e| format!("{}: {e}", launch.program));
        }
    }
    let mut command = build_command(launch);
    command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    match command.spawn() {
        Ok(mut child) => {
            // Reap it so it does not linger as a zombie; nobody waits for
            // the answer.
            std::thread::spawn(move || {
                let _ = child.wait();
            });
            Ok(())
        }
        Err(e) => Err(gitpoulpi_i18n::msg::tool_error::start_failed(
            &launch.program,
            e,
        )),
    }
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

/// The merge tool's command: `{base}`, `{local}` (A), `{remote}` (B) and
/// `{merged}` (where the result goes).
pub(crate) fn merge_launch(
    template: &str,
    base: &Path,
    local: &Path,
    remote: &Path,
    merged: &Path,
) -> Result<Launch, String> {
    expand(
        template,
        &[
            ("base", &native_path(base)),
            ("local", &native_path(local)),
            ("remote", &native_path(remote)),
            ("merged", &native_path(merged)),
        ],
    )
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

    #[test]
    fn expands_placeholders_after_splitting_so_spaces_stay_one_argument() {
        let l = expand(
            "meld {local} \"{remote}\"",
            &[("local", "/tmp/a b/L.txt"), ("remote", "/r/c d.txt")],
        )
        .unwrap();
        assert_eq!(l.program, "meld");
        assert_eq!(l.args, ["/tmp/a b/L.txt", "/r/c d.txt"]);
    }

    #[test]
    fn explorer_select_template_keeps_its_quotes_in_the_raw_form() {
        let l = expand(
            "explorer.exe /select,\"{path}\"",
            &[("path", "C:\\my repo\\a.txt")],
        )
        .unwrap();
        assert_eq!(l.program, "explorer.exe");
        assert_eq!(l.raw_args, "/select,\"C:\\my repo\\a.txt\"");
        assert_eq!(l.args, ["/select,C:\\my repo\\a.txt"]);
    }

    #[test]
    fn an_empty_template_is_an_error_and_a_quoted_program_is_one_word() {
        assert!(expand("   ", &[]).is_err());
        let l = expand(
            "\"C:\\Program Files\\x\\code.cmd\" -r {file}",
            &[("file", "f")],
        )
        .unwrap();
        assert_eq!(l.program, "C:\\Program Files\\x\\code.cmd");
        assert_eq!(l.args, ["-r", "f"]);
    }

    #[test]
    fn the_editor_template_gets_the_file_appended_when_it_has_no_placeholder() {
        let l = editor_launch("code -r", Path::new("a/b.txt")).unwrap();
        assert_eq!(l.program, "code");
        assert_eq!(l.args.len(), 2);
        assert_eq!(l.args[0], "-r");
        assert!(l.args[1].ends_with("b.txt"));
        let l = editor_launch("subl {file}:1", Path::new("x.txt")).unwrap();
        assert_eq!(l.args, ["x.txt:1"]);
    }

    #[test]
    fn the_editor_is_named_from_its_program() {
        assert_eq!(editor_name("code -r {file}"), "VS Code");
        assert_eq!(
            editor_name("\"C:\\tools\\notepad++.exe\" {file}"),
            "notepad++"
        );
        assert_eq!(editor_name("nvim-qt"), "nvim-qt");
    }

    #[test]
    fn the_merge_tool_template_gets_the_four_files() {
        let l = merge_launch(
            "meld {local} {base} {remote} --output \"{merged}\"",
            Path::new("/t/BASE x"),
            Path::new("/t/LOCAL"),
            Path::new("/t/REMOTE"),
            Path::new("/t/MERGED"),
        )
        .unwrap();
        assert_eq!(l.program, "meld");
        // Windows renders the paths with backslashes.
        let args: Vec<String> = l.args.iter().map(|a| a.replace('\\', "/")).collect();
        assert_eq!(
            args,
            [
                "/t/LOCAL",
                "/t/BASE x",
                "/t/REMOTE",
                "--output",
                "/t/MERGED"
            ]
        );
    }

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
