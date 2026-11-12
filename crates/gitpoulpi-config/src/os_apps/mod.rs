//! The applications the operating system registers for a file type: the
//! "Open with" list. Pure parsing, in the style of [`crate::image_editors`]:
//! nothing here spawns a process, reads the environment or touches the real
//! desktop configuration. The caller hands in the file system
//! ([`linux::Files`]) or the registry ([`windows::Registry`]), so every rule is
//! table-testable with a fake machine.
//!
//! - **Linux** ([`linux::apps_for`]): the MIME type from `mime/globs2`, the
//!   default from `mimeapps.list`, the full list from the `.desktop` files.
//! - **Windows** ([`windows::apps_for`]): the registry keys of the extension
//!   (`UserChoice`, `OpenWithProgids`, `OpenWithList`) and the `shell\open`
//!   commands they point at.
//! - **macOS**: the default app only, through `open`.
//!
//! An application is a command template in the `${file}` syntax of
//! [`crate::image_editors::expand`], so one runner launches it.

pub mod linux;
pub mod windows;

use crate::open_in::OpenInEntry;

/// One application that can open a file type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OsApp {
    /// What the list shows.
    pub name: String,
    /// An argv template with `${file}`, e.g. `evince ${file}`.
    pub command: String,
    /// What makes two entries the same program: the executable's normalized
    /// name ([`exe_key`]).
    pub key: String,
    /// The application the OS opens this type with.
    pub default: bool,
}

impl OsApp {
    pub fn entry(&self) -> OpenInEntry {
        OpenInEntry::new(self.name.clone(), self.command.clone())
    }
}

/// What the OS knows about one extension.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Found {
    /// The default application first, then the others by name. Empty when the
    /// OS knows none.
    pub apps: Vec<OsApp>,
    /// The type is plain text (a mode-only change of a `.txt` is no binary
    /// file): the binary view does not offer apps for it.
    pub textual: bool,
}

impl Found {
    /// Drops a command listed twice, puts the default first and the rest in
    /// name order, and leaves the mark on the first entry only. (Two entries
    /// of one program with different flags, `--writer` and `--calc`, stay.)
    pub fn finish(mut self) -> Self {
        // The default of a duplicated command must survive the dedupe.
        self.apps
            .sort_by_cached_key(|a| (!a.default, a.name.to_lowercase()));
        let mut seen: Vec<String> = Vec::new();
        self.apps.retain(|a| {
            if seen.contains(&a.command) {
                false
            } else {
                seen.push(a.command.clone());
                true
            }
        });
        for (i, a) in self.apps.iter_mut().enumerate() {
            a.default = i == 0;
        }
        self
    }
}

/// The lower-case extension of `name` without the dot (`"A.PDF"` -> `"pdf"`),
/// empty when it has none.
pub fn extension_of(name: &str) -> String {
    let base = name.rsplit(['/', '\\']).next().unwrap_or(name);
    match base.rfind('.') {
        Some(i) if i > 0 && i + 1 < base.len() => base[i + 1..].to_lowercase(),
        _ => String::new(),
    }
}

/// The macOS answer: the default app, opened through `open`.
pub fn macos_apps() -> Found {
    Found {
        apps: vec![OsApp {
            name: "Default app".into(), // i18n-english
            command: "open ${file}".into(),
            key: "open".into(),
            default: true,
        }],
        textual: false,
    }
}

/// Quotes `arg` so [`crate::launch::split_argv`] reads it back as one word.
pub fn quote_arg(arg: &str) -> String {
    let plain = !arg.is_empty()
        && arg
            .chars()
            .all(|c| !c.is_whitespace() && !matches!(c, '"' | '\'' | '\\'));
    if plain {
        return arg.to_string();
    }
    let mut out = String::from("\"");
    for c in arg.chars() {
        if matches!(c, '"' | '\\') {
            out.push('\\');
        }
        out.push(c);
    }
    out.push('"');
    out
}

/// The switch of Chromium browsers (Chrome, Edge) whose registered command is
/// `chrome.exe --single-argument %1`: everything after it is ONE verbatim
/// argument, quotes included.
const SINGLE_ARGUMENT: &str = "--single-argument";

/// The command line after the program for a command that reads its tail
/// verbatim (`--single-argument`), with `${file}` replaced by `file` as it is:
/// a standard argument vector would wrap a path with spaces in quotes, and the
/// browser would take the quote for the first character of the address
/// (`http://"c/...`). `None` for any other command, which is run with
/// ordinary arguments. Pure: the Windows spawn passes the answer to `raw_arg`.
pub fn verbatim_tail(command: &str, file: &str) -> Option<String> {
    let words = crate::launch::split_argv(command).ok()?;
    if !words.iter().skip(1).any(|w| w == SINGLE_ARGUMENT) {
        return None;
    }
    let text = command.trim_start();
    let end = if let Some(rest) = text.strip_prefix('"') {
        rest.find('"').map(|i| i + 2)?
    } else {
        text.find(char::is_whitespace).unwrap_or(text.len())
    };
    let tail = text[end..]
        .trim()
        .replace("\"${file}\"", "${file}")
        .replace("${file}", file);
    Some(tail)
}

/// The normalized name of the program a command runs: the file name without
/// directory, `.exe` or a trailing version (`gimp-2.10` -> `gimp`). A
/// `flatpak run <id>` is identified by its id.
pub fn exe_key(argv: &[String]) -> String {
    let Some(first) = argv.first() else {
        return String::new();
    };
    let base = |s: &str| -> String {
        let name = s.rsplit(['/', '\\']).next().unwrap_or(s).to_lowercase();
        let name = name.strip_suffix(".exe").unwrap_or(&name).to_string();
        // `gimp-2.10`, `libreoffice7.6`: the version does not make a program.
        let trimmed = name.trim_end_matches(|c: char| c.is_ascii_digit() || c == '.');
        let trimmed = trimmed.trim_end_matches('-');
        if trimmed.is_empty() {
            name
        } else {
            trimmed.to_string()
        }
    };
    if base(first) == "flatpak" {
        if let Some(id) = argv
            .iter()
            .skip(1)
            .find(|a| !a.starts_with('-') && *a != "run")
        {
            return id.to_lowercase();
        }
    }
    base(first)
}

/// [`exe_key`] of a command template; empty when it does not split.
pub fn command_key(command: &str) -> String {
    crate::launch::split_argv(command)
        .map(|argv| exe_key(&argv))
        .unwrap_or_default()
}

/// `entries` followed by the OS applications whose program is not already in
/// the list (the user's own entries win, so `ui.image_editors` overrides keep
/// working). Nothing is removed from `entries`.
pub fn merge_after(entries: &[OpenInEntry], apps: &[OsApp]) -> Vec<OpenInEntry> {
    let mut out = entries.to_vec();
    let mut keys: Vec<String> = entries.iter().map(|e| command_key(&e.command)).collect();
    for app in apps {
        if keys.contains(&app.key) || out.iter().any(|e| e.label == app.name) {
            continue;
        }
        keys.push(app.key.clone());
        out.push(app.entry());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn app(name: &str, command: &str, default: bool) -> OsApp {
        OsApp {
            name: name.into(),
            command: command.into(),
            key: command_key(command),
            default,
        }
    }

    #[test]
    fn the_extension_is_lower_case_and_needs_a_base_name() {
        assert_eq!(extension_of("Report.PDF"), "pdf");
        assert_eq!(extension_of("a/b\\c.tar.gz"), "gz");
        assert_eq!(extension_of(".gitignore"), "");
        assert_eq!(extension_of("noext"), "");
        assert_eq!(extension_of("trailing."), "");
    }

    #[test]
    fn a_program_is_identified_without_version_directory_or_exe() {
        let key = |c: &str| command_key(c);
        assert_eq!(key("gimp-2.10 ${file}"), "gimp");
        assert_eq!(key("/usr/bin/GIMP ${file}"), "gimp");
        assert_eq!(
            key("\"C:\\\\Program Files\\\\X\\\\Viewer.exe\" \"${file}\""),
            "viewer"
        );
        assert_eq!(key("flatpak run org.gimp.GIMP ${file}"), "org.gimp.gimp");
        assert_eq!(
            key("flatpak run --branch=stable org.gimp.GIMP ${file}"),
            "org.gimp.gimp"
        );
        assert_eq!(key(""), "");
    }

    #[test]
    fn finish_puts_the_default_first_and_marks_only_it() {
        let found = Found {
            apps: vec![
                app("Zed", "zed ${file}", false),
                app("Alpha", "alpha ${file}", false),
                app("Mid", "mid ${file}", true),
                app("Mid again", "mid ${file}", false),
                app("Mid flag", "mid --flag ${file}", false),
            ],
            textual: false,
        }
        .finish();
        let names: Vec<_> = found.apps.iter().map(|a| a.name.as_str()).collect();
        assert_eq!(names, ["Mid", "Alpha", "Mid flag", "Zed"]);
        assert!(found.apps[0].default && found.apps[1..].iter().all(|a| !a.default));
    }

    #[test]
    fn merging_keeps_the_users_entries_and_drops_the_same_program() {
        let own = vec![
            OpenInEntry::new("My GIMP", "gimp --new-instance ${file}"),
            OpenInEntry::new("Default app", "xdg-open ${file}"),
        ];
        let os = vec![
            app("GNU Image Manipulation Program", "gimp-2.10 ${file}", true),
            app("Krita", "krita ${file}", false),
            app("Default app", "other ${file}", false),
        ];
        let merged = merge_after(&own, &os);
        let labels: Vec<_> = merged.iter().map(|e| e.label.as_str()).collect();
        assert_eq!(labels, ["My GIMP", "Default app", "Krita"]);
        // An explicitly empty list is extended, not replaced.
        assert_eq!(merge_after(&[], &os[1..2]).len(), 1);
    }

    #[test]
    fn a_command_is_quoted_so_it_splits_back() {
        for arg in ["plain", "with space", "q\"uote", "back\\slash", "it's", ""] {
            let line = format!("x {}", quote_arg(arg));
            assert_eq!(
                crate::launch::split_argv(&line).unwrap(),
                ["x", arg],
                "{arg:?}"
            );
        }
    }

    #[test]
    fn macos_offers_the_default_app_only() {
        let found = macos_apps();
        assert_eq!(found.apps.len(), 1);
        assert!(found.apps[0].default);
        assert_eq!(found.apps[0].command, "open ${file}");
    }

    #[test]
    fn a_single_argument_tail_is_verbatim_and_unquoted() {
        let file = r"C:\Users\Jo Doe\a b.html";
        for command in [
            r#""C:\Program Files\Google\Chrome\Application\chrome.exe" --single-argument ${file}"#,
            r#""C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe" --single-argument ${file}"#,
            r#"msedge.exe --single-argument "${file}""#,
        ] {
            assert_eq!(
                verbatim_tail(command, file).as_deref(),
                Some(format!("--single-argument {file}").as_str()),
                "{command}"
            );
        }
    }

    #[test]
    fn other_commands_keep_ordinary_arguments() {
        for command in [
            r#""C:\Program Files\Mozilla Firefox\firefox.exe" -osint -url "${file}""#,
            r#""C:\Adobe\Acrobat.exe" "${file}""#,
            "notepad.exe ${file}",
            r#""C:\x\viewer.exe" --kiosk ${file}"#,
            "x.exe",
            "--single-argument ${file}",
        ] {
            assert_eq!(verbatim_tail(command, r"C:\a b\c.txt"), None, "{command}");
        }
    }
}
