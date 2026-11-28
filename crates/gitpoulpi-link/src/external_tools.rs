//! Command templates for the programs a file's context menu hands work to
//! (an external editor, an external diff or merge tool, the OS file explorer),
//! expanded into a [`Launch`] a frontend spawns.
//!
//! The first word of a template is the program. The rest are arguments split
//! on spaces, with `"double quotes"` grouping a word that contains spaces.
//! `{placeholder}` values are substituted *after* splitting, so a path with
//! spaces stays one argument. On Windows the argument text after the program
//! is kept verbatim in [`Launch::raw_args`], which is what makes
//! `explorer.exe /select,"{path}"` work. [`spawn`] starts a launch without
//! waiting for it and without a console window.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// A command ready to run: program, split-and-substituted arguments, and
/// the same arguments as one verbatim string for Windows' `raw_arg`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Launch {
    pub program: String,
    pub args: Vec<String>,
    pub raw_args: String,
    pub cwd: Option<PathBuf>,
}

/// Splits `template` into words: spaces separate, a `"..."` span groups,
/// quotes are dropped. `raw` additionally keeps the original text of
/// everything after the first word, verbatim.
pub fn split_template(template: &str) -> (Vec<String>, String) {
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
pub fn expand(template: &str, vars: &[(&str, &str)]) -> Result<Launch, String> {
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
pub fn native_path(path: &Path) -> String {
    let text = path.to_string_lossy().into_owned();
    if cfg!(windows) {
        text.replace('/', "\\")
    } else {
        text
    }
}

/// `{file}` is appended when the editor template never mentions it, so
/// `code` and `code -r` both work.
pub fn editor_launch(template: &str, file: &Path) -> Result<Launch, String> {
    let template = if template.contains("{file}") {
        template.to_string()
    } else {
        format!("{} \"{{file}}\"", template.trim())
    };
    expand(&template, &[("file", &native_path(file))])
}

pub fn explorer_launch(template: &str, file: &Path) -> Result<Launch, String> {
    let dir = file.parent().map(native_path).unwrap_or_default();
    expand(template, &[("path", &native_path(file)), ("dir", &dir)])
}

pub fn diff_launch(template: &str, local: &Path, remote: &Path) -> Result<Launch, String> {
    expand(
        template,
        &[
            ("local", &native_path(local)),
            ("remote", &native_path(remote)),
        ],
    )
}

/// The first `name` (or `name.exe`/`.cmd`/... on Windows) found on `PATH`.
pub fn find_on_path(name: &str) -> Option<PathBuf> {
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
pub fn editor_name(template: &str) -> String {
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

/// The process for `launch`: arguments (verbatim on Windows), working
/// directory, no console window.
pub fn build_command(launch: &Launch) -> Command {
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

/// The merge tool's command: `{base}`, `{local}` (A), `{remote}` (B) and
/// `{merged}` (where the result goes).
pub fn merge_launch(
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

/// Starts the command without waiting for it: null stdio, no console window,
/// the child reaped on a throwaway thread.
pub fn spawn(launch: &Launch) -> Result<(), String> {
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
}
