//! The file context menu's commands (ignore, treat as binary or text, stash this file, open in the editor
//! or the default program, show in the folder, copy the path, history and blame) and the external
//! programs they start. The ids map to a [`FileOp`] here; the work is `Workspace::run_file_op`
//! (`app/workspace/file_menu.rs`).
//!
//! External programs are the user's configuration: `general.external_editor` and `general.file_explorer`
//! are command templates (`{file}`, `{path}`, `{dir}`), the worktree's "Open in" entries are
//! `ui.open_in` (`${cwd}`), all through `gitpoulpi-config`. Nothing waits on a child and nothing opens a
//! console window; tests replace the spawner with a recorder ([`set_spawner`]).

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Mutex, OnceLock};

use gitpoulpi_app::palette::vocab::status_pane::FileContextAction;
use gitpoulpi_config::open_in::{self, OpenInEntry, OpenInVars, Os};

/// What a file command does, on the file a menu names or the selected one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileOp {
    Stage,
    Unstage,
    Discard,
    IgnoreFile,
    IgnoreExtension,
    IgnoreFolder,
    TreatFileBinary,
    TreatFileText,
    TreatExtensionBinary,
    TreatExtensionText,
    StashFile,
    OpenInEditor,
    OpenInDefault,
    ShowInFolder,
    CopyPath,
    /// The file's history (the commits that changed it) as a read-only list. Blame is not here yet.
    History,
}

/// The file op of a file menu action the desktop runs. Stage, Unstage and Discard stay the working-changes
/// pane's own ops (`ops.rs`); the rest of the menu is here.
pub fn for_file_action(action: FileContextAction) -> Option<FileOp> {
    use FileContextAction as A;
    Some(match action {
        A::IgnoreFile => FileOp::IgnoreFile,
        A::IgnoreExtension => FileOp::IgnoreExtension,
        A::TreatFileBinary => FileOp::TreatFileBinary,
        A::TreatFileText => FileOp::TreatFileText,
        A::TreatExtensionBinary => FileOp::TreatExtensionBinary,
        A::TreatExtensionText => FileOp::TreatExtensionText,
        A::StashFile => FileOp::StashFile,
        A::FileHistory => FileOp::History,
        A::OpenInEditor => FileOp::OpenInEditor,
        A::OpenInDefault => FileOp::OpenInDefault,
        A::ShowInFolder => FileOp::ShowInFolder,
        A::CopyPath => FileOp::CopyPath,
        _ => return None,
    })
}

// -- spawning --------------------------------------------------------------------------------------------

type Spawner = Box<dyn Fn(&[String], Option<&Path>) -> Result<(), String> + Send + Sync>;

static SPAWNER: OnceLock<Mutex<Option<Spawner>>> = OnceLock::new();

fn slot() -> &'static Mutex<Option<Spawner>> {
    SPAWNER.get_or_init(|| Mutex::new(None))
}

/// Replaces what starts programs (tests record the command lines instead of running anything).
pub fn set_spawner(
    f: impl Fn(&[String], Option<&Path>) -> Result<(), String> + Send + Sync + 'static,
) {
    *slot().lock().unwrap_or_else(|e| e.into_inner()) = Some(Box::new(f));
}

/// Starts `argv` detached: null stdio, no console window, reaped on a thread.
pub fn spawn(argv: &[String], cwd: Option<&Path>) -> Result<(), String> {
    if let Some(f) = slot().lock().unwrap_or_else(|e| e.into_inner()).as_ref() {
        return f(argv, cwd);
    }
    let (program, args) = argv.split_first().ok_or("the command is empty")?;
    let mut cmd = Command::new(program);
    cmd.args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    if let Some(cwd) = cwd {
        cmd.current_dir(cwd);
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt as _;
        cmd.creation_flags(0x0800_0000);
    }
    let mut child = cmd.spawn().map_err(|e| e.to_string())?;
    std::thread::spawn(move || {
        let _ = child.wait();
    });
    Ok(())
}

/// `template` split into words, then every `{name}` of `vars` replaced inside each word (so a path with
/// spaces stays one argument).
pub fn expand_template(template: &str, vars: &[(&str, &str)]) -> Result<Vec<String>, String> {
    let words = gitpoulpi_config::launch::split_argv(template)?;
    if words.is_empty() {
        return Err("the command is empty".into());
    }
    Ok(words
        .into_iter()
        .map(|w| {
            vars.iter()
                .fold(w, |w, (k, v)| w.replace(&format!("{{{k}}}"), v))
        })
        .collect())
}

/// The user's choice of external programs, read from the common config (`general.external_editor`,
/// `general.file_explorer`, `ui.open_in`).
#[derive(Clone, Debug, Default)]
pub struct External {
    pub editor: String,
    pub explorer: String,
    pub open_in: Option<Vec<OpenInEntry>>,
}

impl External {
    pub fn of(config: &gitpoulpi_config::Config) -> Self {
        Self {
            editor: config.general.external_editor.clone(),
            explorer: config.general.file_explorer.clone(),
            open_in: config.ui.open_in.clone(),
        }
    }

    /// The settings of the running application.
    pub fn current(cx: &gpui_kit::App) -> Self {
        Self::of(crate::prefs::PrefsStore::get(cx).common())
    }
}

/// The name the menu's "Open in" row shows: the configured editor's program, `None` when there is none.
pub fn editor_name(ext: &External) -> Option<String> {
    let word = gitpoulpi_config::launch::split_argv(&ext.editor)
        .ok()?
        .into_iter()
        .next()?;
    let name = Path::new(&word).file_stem()?.to_string_lossy().into_owned();
    (!name.is_empty()).then_some(name)
}

/// The program the operating system opens `path` with.
pub fn default_program_argv(path: &Path) -> Vec<String> {
    let p = path.to_string_lossy().into_owned();
    if cfg!(windows) {
        vec!["cmd".into(), "/C".into(), "start".into(), "".into(), p]
    } else if cfg!(target_os = "macos") {
        vec!["open".into(), p]
    } else {
        vec!["xdg-open".into(), p]
    }
}

/// Opens `file` in the configured external editor, else in the default program.
pub fn open_in_editor(ext: &External, file: &Path) -> Result<String, String> {
    let editor = &ext.editor;
    let file_text = file.to_string_lossy().into_owned();
    if editor.trim().is_empty() {
        spawn(&default_program_argv(file), None)?;
        return Ok(file_text);
    }
    let argv = expand_template(editor, &[("file", &file_text), ("path", &file_text)])?;
    spawn(&argv, None)?;
    Ok(argv[0].clone())
}

/// Opens `file` with the default program.
pub fn open_in_default(file: &Path) -> Result<String, String> {
    spawn(&default_program_argv(file), None)?;
    Ok(file.to_string_lossy().into_owned())
}

/// Shows `file` in the operating system's file manager (`general.file_explorer`).
pub fn show_in_folder(ext: &External, file: &Path) -> Result<String, String> {
    let explorer = &ext.explorer;
    let path = file.to_string_lossy().into_owned();
    let dir = file
        .parent()
        .map(|d| d.to_string_lossy().into_owned())
        .unwrap_or_default();
    if explorer.trim().is_empty() {
        let target = file.parent().unwrap_or(file);
        spawn(&default_program_argv(target), None)?;
        return Ok(dir);
    }
    let argv = expand_template(explorer, &[("path", &path), ("dir", &dir), ("file", &path)])?;
    spawn(&argv, None)?;
    Ok(dir)
}

// -- the worktree's "Open in" entries -------------------------------------------------------------------

fn on_path(program: &str) -> bool {
    std::env::var_os("PATH").is_some_and(|paths| {
        std::env::split_paths(&paths).any(|dir| {
            let f = dir.join(program);
            f.is_file() || (cfg!(windows) && dir.join(format!("{program}.exe")).is_file())
        })
    })
}

/// The "Open in" entries the worktree menu offers: the configured list (`ui.open_in`), else the detected
/// one, keeping those marked "Can open a worktree".
pub fn open_in_entries(ext: &External) -> Vec<OpenInEntry> {
    let all = ext.open_in.clone().unwrap_or_else(|| {
        let d = open_in::detect(&|k| std::env::var(k).ok(), Os::current(), &on_path);
        open_in::effective_list(None, &d, Os::current())
    });
    all.into_iter().filter(|e| e.can_open_worktree()).collect()
}

/// Runs `entry` with `folder` as `${cwd}`; the entry's label on success.
pub fn run_open_in(entry: &OpenInEntry, folder: &Path, branch: &str) -> Result<String, String> {
    let cwd = folder.to_string_lossy().into_owned();
    let name = folder
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let command = open_in::modernized_command(&entry.command).unwrap_or(&entry.command);
    let argv = open_in::expand(
        command,
        &OpenInVars {
            cwd: &cwd,
            repo: &cwd,
            name: &name,
            branch,
            web_url: "",
        },
        &|k| std::env::var(k).ok(),
    )?;
    spawn(&argv, Some(folder))?;
    Ok(entry.label.clone())
}

/// The path of a repository-relative file under `root`.
pub fn absolute(root: &Path, rel: &str) -> PathBuf {
    root.join(rel)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_template_keeps_a_path_with_spaces_as_one_word() {
        let argv = expand_template("code -r {file}", &[("file", "/a b/c.txt")]).unwrap();
        assert_eq!(argv, ["code", "-r", "/a b/c.txt"]);
        assert!(expand_template("  ", &[]).is_err());
    }

    #[test]
    fn the_menu_actions_map_to_file_ops() {
        use FileContextAction as A;
        assert_eq!(for_file_action(A::IgnoreFile), Some(FileOp::IgnoreFile));
        assert_eq!(for_file_action(A::FileHistory), Some(FileOp::History));
        assert_eq!(for_file_action(A::FileBlame), None);
        assert_eq!(for_file_action(A::Stage), None);
        assert_eq!(for_file_action(A::DeleteFile), None);
    }
}
