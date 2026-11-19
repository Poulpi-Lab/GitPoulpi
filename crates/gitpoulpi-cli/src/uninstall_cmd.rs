//! `gitpoulpi uninstall`: what the Linux/macOS `install.sh --uninstall` and the
//! Windows (Inno Setup) uninstaller both call, so the two share one
//! implementation with Settings. It undoes, in this order, the command
//! aliases, the PATH entries and the shortcuts (each through the code that made
//! them, so only what is ours goes), then the program files.
//!
//! It refuses unless the folder of the running exe carries the
//! `.gitpoulpi-dist` marker: a `cargo` build (the aliases of which point into
//! `target/`) must not delete itself or its checkout. The files removed are
//! only the names a release ships plus their `.old`/`.new` leftovers, never
//! the config, sessions or repositories. On Windows a running exe cannot be
//! deleted, so the files that are in use are reported in `files_failed` and
//! the uninstaller (which runs after this process ends) deletes them.

use std::path::{Path, PathBuf};

use gitpoulpi_commands::cli::{AliasAction, PathAction};
use gitpoulpi_config::release::{self, MARKER_FILE};
use serde_json::{json, Value};

pub fn run(keep_files: bool, json_output: bool) {
    match uninstall(keep_files) {
        Ok(value) => {
            if json_output {
                println!("{value}");
            } else {
                println!("{}", human(&value));
            }
        }
        Err(message) => {
            if json_output {
                println!("{}", json!({ "error": message }));
            } else {
                eprintln!("gitpoulpi: {message}");
            }
            std::process::exit(1);
        }
    }
}

fn uninstall(keep_files: bool) -> Result<Value, String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let dir = exe.parent().ok_or("the exe has no folder")?.to_path_buf();
    if !dir.join(MARKER_FILE).is_file() {
        return Err(format!(
            "{} is not a release install ({MARKER_FILE} is missing); not uninstalling a development build",
            dir.display()
        ));
    }
    let mut warnings: Vec<String> = Vec::new();

    let aliases = match gitpoulpi_commands::alias_cmd::alias(AliasAction::Remove { name: None }) {
        Ok(e) => e.json["removed"].clone(),
        Err(e) => {
            warnings.push(format!("aliases: {e}"));
            json!([])
        }
    };
    let path = match gitpoulpi_commands::alias_cmd::path(PathAction::Remove) {
        Ok(e) => e.json["changed"].clone(),
        Err(e) => {
            warnings.push(format!("PATH: {e}"));
            json!([])
        }
    };
    let shortcuts = match gitpoulpi_tui::desktop_shortcuts::remove(None) {
        Ok(lines) => lines,
        Err(e) => {
            warnings.push(format!("shortcuts: {e}"));
            Vec::new()
        }
    };

    // The file-manager menu entries (and the App Paths key of the folder menu).
    let folder_menu = match gitpoulpi_tui::desktop_shortcuts::folder_menu_remove() {
        Ok(lines) => lines,
        Err(e) => {
            warnings.push(format!("folder menu: {e}"));
            Vec::new()
        }
    };

    let (files_removed, files_failed, dir_removed) = if keep_files {
        (Vec::new(), Vec::new(), false)
    } else {
        remove_files(&dir)
    };
    Ok(json!({
        "dir": dir.display().to_string(),
        "aliases_removed": aliases,
        "path_changed": path,
        "shortcuts": shortcuts,
        "folder_menu": folder_menu,
        "kept_files": keep_files,
        "files_removed": files_removed,
        "files_failed": files_failed,
        "dir_removed": dir_removed,
        "warnings": warnings,
    }))
}

/// The files of a release install in `dir`: every shipped binary and the
/// marker and every retired program, plus the updater's `.old`, `.new` and `.swap` leftovers.
fn owned_files(dir: &Path) -> Vec<PathBuf> {
    owned_files_with(dir, &release::RETIRED)
}

/// [`owned_files`] with an explicit retired list (a retired program's files
/// are removed too).
fn owned_files_with(dir: &Path, retired: &[&str]) -> Vec<PathBuf> {
    let target = release::host_target();
    let mut names: Vec<String> = release::BINARIES
        .iter()
        .map(|b| release::exe_name(b, &target))
        .collect();
    names.extend(release::retired_names_in(retired, &target));
    names.push(MARKER_FILE.to_string());
    let mut files = Vec::new();
    for name in &names {
        for suffix in ["", ".old", ".new", ".swap"] {
            files.push(dir.join(format!("{name}{suffix}")));
        }
    }
    // The marker goes last: it is what makes the folder removable.
    files.sort_by_key(|f| f.file_name().is_some_and(|n| n == MARKER_FILE));
    files
}

fn remove_files(dir: &Path) -> (Vec<String>, Vec<String>, bool) {
    let mut removed = Vec::new();
    let mut failed = Vec::new();
    for file in owned_files(dir) {
        if std::fs::symlink_metadata(&file).is_err() {
            continue;
        }
        let name = file.file_name().unwrap_or_default().to_string_lossy();
        match std::fs::remove_file(&file) {
            Ok(()) => removed.push(name.into_owned()),
            Err(e) => failed.push(format!("{name}: {e}")),
        }
    }
    // Only when empty: a folder the user also keeps other things in stays.
    let _ = std::fs::remove_dir(dir.join("bin"));
    let dir_removed = std::fs::remove_dir(dir).is_ok();
    (removed, failed, dir_removed)
}

fn human(v: &Value) -> String {
    let list = |k: &str| -> usize { v[k].as_array().map_or(0, Vec::len) };
    format!(
        "removed {} alias(es), {} PATH change(s), {} shortcut result(s), {} file(s){}",
        list("aliases_removed"),
        list("path_changed"),
        list("shortcuts"),
        list("files_removed"),
        if list("files_failed") > 0 {
            format!("; {} file(s) in use were left", list("files_failed"))
        } else {
            String::new()
        }
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_retired_program_is_part_of_what_uninstall_removes() {
        let dir = Path::new("/gp");
        let target = release::host_target();
        let retired = release::exe_name("gitpoulpi-old", &target);
        let has = |files: &[PathBuf], name: &str| {
            files
                .iter()
                .any(|f| f.file_name().is_some_and(|n| n == name))
        };
        assert!(!has(&owned_files_with(dir, &[]), &retired));
        let files = owned_files_with(dir, &["gitpoulpi-old"]);
        for suffix in ["", ".old", ".new", ".swap"] {
            assert!(has(&files, &format!("{retired}{suffix}")), "{suffix}");
        }
        // Shipped names stay, the marker still goes last.
        assert!(has(&files, &release::exe_name("gitpoulpi", &target)));
        assert_eq!(files.last().unwrap().file_name().unwrap(), MARKER_FILE);
    }
}
