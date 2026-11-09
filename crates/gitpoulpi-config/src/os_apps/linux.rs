//! The Linux "Open with" list, from the freedesktop.org specifications:
//!
//! 1. the MIME type of the file name, from `mime/globs2` (shared-mime-info)
//!    under `$XDG_DATA_HOME` and `$XDG_DATA_DIRS`;
//! 2. the default application, from `mimeapps.list` (`$XDG_CONFIG_HOME`,
//!    `$XDG_CONFIG_DIRS`, then `applications/` of the data dirs; the
//!    desktop-specific `<desktop>-mimeapps.list` first in each folder);
//! 3. every application whose `.desktop` file lists the MIME type in
//!    `MimeType`, plus the `[Added Associations]`, minus the
//!    `[Removed Associations]`. `NoDisplay`, `Hidden` and terminal programs
//!    are left out (a launch has no terminal to give them), except that a
//!    `NoDisplay` default stays: it is what the OS really opens.
//!
//! The file system is a parameter ([`Files`]), so the tests use fake roots.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use super::{exe_key, quote_arg, Found, OsApp};

/// What the parser reads of the machine.
pub trait Files {
    /// The text of a file, `None` when missing or unreadable.
    fn read(&self, path: &Path) -> Option<String>;
    /// The entries of a folder as `(name, is_dir)`; empty when it is missing.
    fn list(&self, dir: &Path) -> Vec<(String, bool)>;
}

/// The real file system.
pub struct StdFiles;

impl Files for StdFiles {
    fn read(&self, path: &Path) -> Option<String> {
        std::fs::read_to_string(path).ok()
    }

    fn list(&self, dir: &Path) -> Vec<(String, bool)> {
        let Ok(items) = std::fs::read_dir(dir) else {
            return Vec::new();
        };
        items
            .flatten()
            .map(|e| {
                // `is_dir` follows symlinks, as the data dirs use them.
                let is_dir = e.path().is_dir();
                (e.file_name().to_string_lossy().into_owned(), is_dir)
            })
            .collect()
    }
}

/// Where the specifications look, in priority order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XdgRoots {
    pub data_home: PathBuf,
    pub data_dirs: Vec<PathBuf>,
    pub config_home: PathBuf,
    pub config_dirs: Vec<PathBuf>,
    /// `$XDG_CURRENT_DESKTOP`, lower case.
    pub desktops: Vec<String>,
    /// How to start a terminal program (`Terminal=true`) in a window of its
    /// own, from `$TERMINAL`: `xdg-terminal-exec` takes the command as is,
    /// any other emulator gets `-e`. Empty: terminal programs are left out
    /// (`xdg-open` would start them in the TUI's own, captured terminal).
    pub terminal: Vec<String>,
}

impl XdgRoots {
    /// The roots `env` describes, with the specifications' defaults.
    pub fn from_env(env: &dyn Fn(&str) -> Option<String>) -> Self {
        let get = |k: &str| env(k).filter(|v| !v.is_empty());
        let home = get("HOME").map(PathBuf::from).unwrap_or_default();
        let list = |v: Option<String>, default: &str| -> Vec<PathBuf> {
            v.unwrap_or_else(|| default.to_string())
                .split(':')
                .filter(|s| !s.is_empty())
                .map(PathBuf::from)
                .collect()
        };
        Self {
            data_home: get("XDG_DATA_HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|| home.join(".local/share")),
            data_dirs: list(get("XDG_DATA_DIRS"), "/usr/local/share:/usr/share"),
            config_home: get("XDG_CONFIG_HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|| home.join(".config")),
            config_dirs: list(get("XDG_CONFIG_DIRS"), "/etc/xdg"),
            desktops: get("XDG_CURRENT_DESKTOP")
                .map(|d| d.split(':').map(str::to_lowercase).collect())
                .unwrap_or_default(),
            terminal: get("TERMINAL")
                .map(|t| {
                    let takes_command = Path::new(&t)
                        .file_name()
                        .is_some_and(|n| n == "xdg-terminal-exec");
                    let mut argv = vec![t];
                    if !takes_command {
                        argv.push("-e".to_string());
                    }
                    argv
                })
                .unwrap_or_default(),
        }
    }

    fn data(&self) -> impl Iterator<Item = &PathBuf> {
        std::iter::once(&self.data_home).chain(self.data_dirs.iter())
    }

    fn config(&self) -> impl Iterator<Item = &PathBuf> {
        std::iter::once(&self.config_home).chain(self.config_dirs.iter())
    }
}

// ------------------------------------------------------------------ MIME type

/// `*` and `?` over `text`; every other character is literal.
fn glob_matches(glob: &str, text: &str) -> bool {
    fn go(g: &[char], t: &[char]) -> bool {
        match g.split_first() {
            None => t.is_empty(),
            Some(('*', rest)) => (0..=t.len()).any(|i| go(rest, &t[i..])),
            Some(('?', rest)) => !t.is_empty() && go(rest, &t[1..]),
            Some((c, rest)) => t.first() == Some(c) && go(rest, &t[1..]),
        }
    }
    let g: Vec<char> = glob.chars().collect();
    let t: Vec<char> = text.chars().collect();
    go(&g, &t)
}

/// The MIME type `file_name` matches: the longest glob wins, then the
/// heaviest.
pub fn mime_of(file_name: &str, roots: &XdgRoots, files: &dyn Files) -> Option<String> {
    let name = file_name.rsplit(['/', '\\']).next().unwrap_or(file_name);
    let lower = name.to_lowercase();
    let mut best: Option<(usize, u32, String)> = None;
    let mut consider = |mime: &str, glob: &str, weight: u32, case_sensitive: bool| {
        let hit = if case_sensitive {
            glob_matches(glob, name)
        } else {
            glob_matches(&glob.to_lowercase(), &lower)
        };
        if hit
            && best
                .as_ref()
                .is_none_or(|(l, w, _)| (glob.len(), weight) > (*l, *w))
        {
            best = Some((glob.len(), weight, mime.to_string()));
        }
    };
    let mut any_globs2 = false;
    for dir in roots.data() {
        let Some(text) = files.read(&dir.join("mime/globs2")) else {
            continue;
        };
        any_globs2 = true;
        for line in text.lines().filter(|l| !l.starts_with('#')) {
            // weight:mime:glob[:flags]
            let mut parts = line.splitn(4, ':');
            let (Some(w), Some(mime), Some(glob)) = (parts.next(), parts.next(), parts.next())
            else {
                continue;
            };
            let flags = parts.next().unwrap_or("");
            if glob == "__NOGLOBS__" {
                continue;
            }
            let weight = w.trim().parse().unwrap_or(50);
            consider(mime, glob, weight, flags.split(',').any(|f| f == "cs"));
        }
    }
    if !any_globs2 {
        for dir in roots.data() {
            let Some(text) = files.read(&dir.join("mime/globs")) else {
                continue;
            };
            for line in text.lines().filter(|l| !l.starts_with('#')) {
                if let Some((mime, glob)) = line.split_once(':') {
                    consider(mime, glob, 50, false);
                }
            }
        }
    }
    best.map(|(_, _, mime)| mime)
}

/// Types whose files are text although their MIME type is not `text/*`.
const TEXT_LIKE: &[&str] = &[
    "application/json",
    "application/xml",
    "application/javascript",
    "application/x-shellscript",
    "application/toml",
    "application/x-yaml",
    "application/sql",
    "application/x-perl",
    "application/x-php",
    "application/x-ruby",
    "application/x-subrip",
    "application/x-desktop",
];

/// Whether files of `mime` are plain text.
pub fn is_textual(mime: &str) -> bool {
    mime.starts_with("text/") || TEXT_LIKE.contains(&mime)
}

// ----------------------------------------------------------------- mimeapps.list

#[derive(Debug, Default)]
struct Associations {
    default: Vec<String>,
    added: Vec<String>,
    removed: Vec<String>,
}

fn id_list(value: &str) -> Vec<String> {
    value
        .split(';')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect()
}

/// One `mimeapps.list`, reduced to the lines for `mime`.
fn parse_mimeapps(text: &str, mime: &str) -> Associations {
    let mut out = Associations::default();
    let mut section = "";
    for line in text.lines().map(str::trim) {
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        if let Some(name) = line.strip_prefix('[').and_then(|l| l.strip_suffix(']')) {
            section = match name {
                "Default Applications" => "default",
                "Added Associations" => "added",
                "Removed Associations" => "removed",
                _ => "",
            };
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        if key.trim() != mime {
            continue;
        }
        let ids = id_list(value);
        match section {
            "default" => out.default.extend(ids),
            "added" => out.added.extend(ids),
            "removed" => out.removed.extend(ids),
            _ => {}
        }
    }
    out
}

/// The `mimeapps.list` files in priority order.
fn mimeapps_paths(roots: &XdgRoots) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let names: Vec<String> = roots
        .desktops
        .iter()
        .map(|d| format!("{d}-mimeapps.list"))
        .chain(std::iter::once("mimeapps.list".to_string()))
        .collect();
    for dir in roots.config() {
        out.extend(names.iter().map(|n| dir.join(n)));
    }
    for dir in roots.data() {
        out.extend(names.iter().map(|n| dir.join("applications").join(n)));
    }
    out
}

// ------------------------------------------------------------------- .desktop

/// A `[Desktop Entry]` reduced to what the list needs.
#[derive(Debug, Default, Clone)]
struct DesktopEntry {
    name: String,
    exec: String,
    mime_types: Vec<String>,
    hidden: bool,
    no_display: bool,
    terminal: bool,
    is_application: bool,
}

/// The escapes of a `.desktop` string value (`\s`, `\n`, `\t`, `\r`, `\\`).
fn unescape_value(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut chars = value.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('s') => out.push(' '),
            Some('n') => out.push('\n'),
            Some('t') => out.push('\t'),
            Some('r') => out.push('\r'),
            Some('\\') => out.push('\\'),
            Some(other) => {
                out.push('\\');
                out.push(other);
            }
            None => out.push('\\'),
        }
    }
    out
}

fn parse_desktop(text: &str) -> DesktopEntry {
    let mut entry = DesktopEntry::default();
    let mut in_entry = false;
    for line in text.lines().map(str::trim) {
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        if line.starts_with('[') {
            in_entry = line == "[Desktop Entry]";
            continue;
        }
        if !in_entry {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let (key, value) = (key.trim(), value.trim());
        let flag = value == "true";
        match key {
            "Type" => entry.is_application = value == "Application",
            "Name" => entry.name = unescape_value(value),
            "Exec" => entry.exec = value.to_string(),
            "MimeType" => entry.mime_types = id_list(value),
            "NoDisplay" => entry.no_display = flag,
            "Hidden" => entry.hidden = flag,
            "Terminal" => entry.terminal = flag,
            _ => {}
        }
    }
    entry
}

/// Splits an `Exec` value into arguments (the quoting of the Desktop Entry
/// specification: double quotes group, `\"`, `` \` ``, `\$` and `\\` escape
/// inside them).
fn split_exec(exec: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut cur = String::new();
    let mut started = false;
    let mut quoted = false;
    let mut chars = exec.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '"' => {
                quoted = !quoted;
                started = true;
            }
            '\\' if quoted && matches!(chars.peek(), Some('"' | '`' | '$' | '\\')) => {
                cur.extend(chars.next());
            }
            c if c.is_whitespace() && !quoted => {
                if started {
                    args.push(std::mem::take(&mut cur));
                    started = false;
                }
            }
            c => {
                cur.push(c);
                started = true;
            }
        }
    }
    if started {
        args.push(cur);
    }
    args
}

/// The `Exec` line as an argv template: `%f %F %u %U` become `${file}`, the
/// other field codes are dropped, and a command with no file code gets the
/// file as its last argument. `None` when it does not make a command.
fn exec_template(exec: &str, name: &str) -> Option<String> {
    let mut out: Vec<String> = Vec::new();
    for arg in split_exec(&unescape_value(exec)) {
        let mut text = String::new();
        let mut chars = arg.chars();
        while let Some(c) = chars.next() {
            if c != '%' {
                text.push(c);
                continue;
            }
            match chars.next() {
                Some('f' | 'F' | 'u' | 'U') => text.push_str("${file}"),
                Some('c') => text.push_str(name),
                Some('%') => text.push('%'),
                // %i %k %d %D %n %N %v %m: nothing a one-file launch needs.
                Some(_) | None => {}
            }
        }
        if !text.is_empty() || arg.is_empty() {
            out.push(text);
        }
    }
    if out.is_empty() || out[0].is_empty() || out[0].contains("${file}") {
        return None;
    }
    if !out.iter().any(|a| a.contains("${file}")) {
        out.push("${file}".to_string());
    }
    // A literal `${` would read as a variable of ours.
    if out.iter().any(|a| a.replace("${file}", "").contains("${")) {
        return None;
    }
    // `quote_arg` leaves `${file}` bare (no space or quote in it).
    Some(
        out.iter()
            .map(|a| quote_arg(a))
            .collect::<Vec<_>>()
            .join(" "),
    )
}

/// Every `.desktop` file of the data dirs by desktop id (`a/b.desktop` is
/// `a-b.desktop`); the first folder in priority order shadows the others.
fn desktop_files(roots: &XdgRoots, files: &dyn Files) -> HashMap<String, DesktopEntry> {
    fn walk(
        files: &dyn Files,
        dir: &Path,
        prefix: &str,
        depth: usize,
        out: &mut HashMap<String, DesktopEntry>,
    ) {
        for (name, is_dir) in files.list(dir) {
            let path = dir.join(&name);
            if is_dir {
                if depth < 4 {
                    walk(files, &path, &format!("{prefix}{name}-"), depth + 1, out);
                }
            } else if name.ends_with(".desktop") {
                let id = format!("{prefix}{name}");
                if out.contains_key(&id) {
                    continue;
                }
                if let Some(text) = files.read(&path) {
                    out.insert(id, parse_desktop(&text));
                }
            }
        }
    }
    let mut out = HashMap::new();
    for dir in roots.data() {
        walk(files, &dir.join("applications"), "", 0, &mut out);
    }
    out
}

// ------------------------------------------------------------------- the answer

/// The applications for a file called `file_name`.
pub fn apps_for(file_name: &str, roots: &XdgRoots, files: &dyn Files) -> Found {
    let Some(mime) = mime_of(file_name, roots, files) else {
        return Found::default();
    };
    let textual = is_textual(&mime);
    let desktop = desktop_files(roots, files);

    // Associations, highest priority first.
    let mut default_ids: Vec<String> = Vec::new();
    let mut added: Vec<String> = Vec::new();
    let mut removed: HashSet<String> = HashSet::new();
    for path in mimeapps_paths(roots) {
        let Some(text) = files.read(&path) else {
            continue;
        };
        let assoc = parse_mimeapps(&text, &mime);
        default_ids.extend(assoc.default);
        for id in assoc.added {
            if !removed.contains(&id) && !added.contains(&id) {
                added.push(id);
            }
        }
        removed.extend(assoc.removed);
    }
    let default_id = default_ids
        .iter()
        .find(|id| desktop.contains_key(*id) && !removed.contains(*id))
        .cloned();

    let mut ids: Vec<String> = Vec::new();
    ids.extend(default_id.clone());
    ids.extend(added);
    let mut by_mime: Vec<&String> = desktop
        .iter()
        .filter(|(_, e)| e.mime_types.contains(&mime))
        .map(|(id, _)| id)
        .collect();
    by_mime.sort();
    ids.extend(by_mime.into_iter().cloned());

    let mut apps = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    for id in ids {
        if !seen.insert(id.clone()) || (removed.contains(&id) && default_id.as_ref() != Some(&id)) {
            continue;
        }
        let Some(entry) = desktop.get(&id) else {
            continue;
        };
        let is_default = default_id.as_ref() == Some(&id);
        if !entry.is_application || entry.hidden || (entry.terminal && roots.terminal.is_empty()) {
            continue;
        }
        if entry.no_display && !is_default {
            continue;
        }
        let name = if entry.name.is_empty() {
            id.trim_end_matches(".desktop").to_string()
        } else {
            entry.name.clone()
        };
        let Some(program) = exec_template(&entry.exec, &name) else {
            continue;
        };
        // The program decides the dedupe key, not the terminal around it.
        let key = crate::launch::split_argv(&program)
            .map(|argv| exe_key(&argv))
            .unwrap_or_default();
        let command = if entry.terminal {
            let mut parts: Vec<String> = roots.terminal.iter().map(|a| quote_arg(a)).collect();
            parts.push(program);
            parts.join(" ")
        } else {
            program
        };
        apps.push(OsApp {
            name,
            command,
            key,
            default: is_default,
        });
    }
    Found { apps, textual }.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A fake machine: files by path.
    #[derive(Default)]
    struct Fake(HashMap<PathBuf, String>);

    impl Fake {
        fn with(mut self, path: &str, text: &str) -> Self {
            self.0.insert(PathBuf::from(path), text.to_string());
            self
        }
    }

    impl Files for Fake {
        fn read(&self, path: &Path) -> Option<String> {
            self.0.get(path).cloned()
        }

        fn list(&self, dir: &Path) -> Vec<(String, bool)> {
            let mut out: Vec<(String, bool)> = Vec::new();
            for key in self.0.keys() {
                let Ok(rest) = key.strip_prefix(dir) else {
                    continue;
                };
                let mut parts = rest.components();
                let Some(first) = parts.next() else {
                    continue;
                };
                let entry = (
                    first.as_os_str().to_string_lossy().into_owned(),
                    parts.next().is_some(),
                );
                if !out.contains(&entry) {
                    out.push(entry);
                }
            }
            out
        }
    }

    fn roots() -> XdgRoots {
        XdgRoots::from_env(&|k| match k {
            "HOME" => Some("/h".into()),
            "XDG_DATA_DIRS" => Some("/d1:/d2".into()),
            "XDG_CONFIG_DIRS" => Some("/c1".into()),
            "XDG_CURRENT_DESKTOP" => Some("Hyprland:Other".into()),
            _ => None,
        })
    }

    fn desktop(name: &str, exec: &str, mime: &str) -> String {
        format!("[Desktop Entry]\nType=Application\nName={name}\nExec={exec}\nMimeType={mime}\n")
    }

    const GLOBS: &str = "# comment\n50:application/pdf:*.pdf\n50:image/png:*.png\n\
        50:application/gzip:*.gz\n50:application/x-compressed-tar:*.tar.gz\n\
        50:text/plain:*.txt\n50:text/x-makefile:Makefile:cs\n10:application/vnd.x:*.PDF:cs\n";

    #[test]
    fn the_roots_follow_the_environment_and_the_specifications_defaults() {
        let r = roots();
        assert_eq!(r.data_home, Path::new("/h/.local/share"));
        assert_eq!(r.config_home, Path::new("/h/.config"));
        assert_eq!(r.data_dirs, [PathBuf::from("/d1"), PathBuf::from("/d2")]);
        assert_eq!(r.desktops, ["hyprland", "other"]);
        let d = XdgRoots::from_env(&|k| (k == "HOME").then(|| "/home/u".to_string()));
        assert_eq!(
            d.data_dirs,
            [
                PathBuf::from("/usr/local/share"),
                PathBuf::from("/usr/share")
            ]
        );
        assert_eq!(d.config_dirs, [PathBuf::from("/etc/xdg")]);
    }

    #[test]
    fn the_mime_type_is_the_longest_matching_glob_without_regard_to_case() {
        let fs = Fake::default().with("/d1/mime/globs2", GLOBS);
        let r = roots();
        assert_eq!(
            mime_of("Report.PDF", &r, &fs).as_deref(),
            Some("application/pdf")
        );
        assert_eq!(
            mime_of("a.tar.gz", &r, &fs).as_deref(),
            Some("application/x-compressed-tar")
        );
        assert_eq!(
            mime_of("a.gz", &r, &fs).as_deref(),
            Some("application/gzip")
        );
        assert_eq!(
            mime_of("Makefile", &r, &fs).as_deref(),
            Some("text/x-makefile")
        );
        assert_eq!(mime_of("makefile", &r, &fs), None, "the cs flag keeps case");
        assert_eq!(mime_of("noglob.zzz", &r, &fs), None);
        assert_eq!(
            mime_of("dir/sub/pic.png", &r, &fs).as_deref(),
            Some("image/png")
        );
    }

    #[test]
    fn the_old_globs_file_is_the_fallback() {
        let fs = Fake::default().with("/d2/mime/globs", "application/pdf:*.pdf\n");
        assert_eq!(
            mime_of("a.pdf", &roots(), &fs).as_deref(),
            Some("application/pdf")
        );
    }

    #[test]
    fn the_list_has_the_default_first_and_every_handler_once() {
        let fs = Fake::default()
            .with("/d1/mime/globs2", GLOBS)
            .with(
                "/h/.config/mimeapps.list",
                "[Default Applications]\napplication/pdf=okular.desktop;evince.desktop;\n",
            )
            .with(
                "/d1/applications/evince.desktop",
                &desktop("Evince", "evince %U", "application/pdf;"),
            )
            .with(
                "/d1/applications/okular.desktop",
                &desktop("Okular", "okular %f", "application/pdf;"),
            )
            .with(
                "/d2/applications/firefox.desktop",
                &desktop(
                    "Firefox",
                    "firefox --new-window %u",
                    "text/html;application/pdf;",
                ),
            )
            .with(
                "/d2/applications/gimp.desktop",
                &desktop("GIMP", "gimp %U", "image/png;"),
            );
        let found = apps_for("doc.pdf", &roots(), &fs);
        let names: Vec<_> = found.apps.iter().map(|a| a.name.as_str()).collect();
        assert_eq!(names, ["Okular", "Evince", "Firefox"]);
        assert!(found.apps[0].default && !found.apps[1].default);
        assert_eq!(found.apps[0].command, "okular ${file}");
        assert_eq!(found.apps[2].command, "firefox --new-window ${file}");
        assert!(!found.textual);
    }

    #[test]
    fn a_higher_priority_mimeapps_list_decides_the_default() {
        let fs = Fake::default()
            .with("/d1/mime/globs2", GLOBS)
            // The desktop-specific list of the config home beats the plain one.
            .with(
                "/h/.config/hyprland-mimeapps.list",
                "[Default Applications]\napplication/pdf=b.desktop\n",
            )
            .with(
                "/h/.config/mimeapps.list",
                "[Default Applications]\napplication/pdf=a.desktop\n",
            )
            .with(
                "/c1/mimeapps.list",
                "[Default Applications]\napplication/pdf=a.desktop\n",
            )
            .with(
                "/d1/applications/a.desktop",
                &desktop("A", "a %f", "application/pdf"),
            )
            .with(
                "/d1/applications/b.desktop",
                &desktop("B", "b %f", "application/pdf"),
            );
        let found = apps_for("x.pdf", &roots(), &fs);
        assert_eq!(found.apps[0].name, "B");
    }

    #[test]
    fn a_default_that_is_not_installed_falls_to_the_next_installed_one() {
        let fs = Fake::default()
            .with("/d1/mime/globs2", GLOBS)
            .with(
                "/h/.config/mimeapps.list",
                "[Default Applications]\napplication/pdf=gone.desktop;b.desktop\n",
            )
            .with(
                "/d1/applications/a.desktop",
                &desktop("A", "a %f", "application/pdf"),
            )
            .with(
                "/d1/applications/b.desktop",
                &desktop("B", "b %f", "application/pdf"),
            );
        assert_eq!(apps_for("x.pdf", &roots(), &fs).apps[0].name, "B");
    }

    #[test]
    fn added_and_removed_associations_change_the_list() {
        let fs = Fake::default()
            .with("/d1/mime/globs2", GLOBS)
            .with(
                "/h/.config/mimeapps.list",
                "[Added Associations]\napplication/pdf=extra.desktop;\n\
                 [Removed Associations]\napplication/pdf=noisy.desktop;\n",
            )
            .with(
                "/d1/applications/extra.desktop",
                &desktop("Extra", "extra %f", "image/png"),
            )
            .with(
                "/d1/applications/noisy.desktop",
                &desktop("Noisy", "noisy %f", "application/pdf"),
            )
            .with(
                "/d1/applications/plain.desktop",
                &desktop("Plain", "plain %f", "application/pdf"),
            );
        let names: Vec<String> = apps_for("x.pdf", &roots(), &fs)
            .apps
            .into_iter()
            .map(|a| a.name)
            .collect();
        assert_eq!(names, ["Extra", "Plain"]);
    }

    #[test]
    fn hidden_nodisplay_terminal_and_other_types_are_left_out() {
        let entry = |extra: &str| format!("{}{extra}\n", desktop("X", "x %f", "application/pdf"));
        let fs = Fake::default()
            .with("/d1/mime/globs2", GLOBS)
            .with("/d1/applications/hidden.desktop", &entry("Hidden=true"))
            .with(
                "/d1/applications/nodisplay.desktop",
                &entry("NoDisplay=true"),
            )
            .with("/d1/applications/term.desktop", &entry("Terminal=true"))
            .with(
                "/d1/applications/link.desktop",
                "[Desktop Entry]\nType=Link\nName=L\nExec=l %f\nMimeType=application/pdf\n",
            )
            .with(
                "/d1/applications/ok.desktop",
                &desktop("Fine", "fine %f", "application/pdf"),
            );
        let names: Vec<String> = apps_for("x.pdf", &roots(), &fs)
            .apps
            .into_iter()
            .map(|a| a.name)
            .collect();
        assert_eq!(names, ["Fine"]);
        // A NoDisplay application the user made the default is still the default.
        let fs = fs.with(
            "/h/.config/mimeapps.list",
            "[Default Applications]\napplication/pdf=nodisplay.desktop\n",
        );
        let found = apps_for("x.pdf", &roots(), &fs);
        assert_eq!(found.apps[0].name, "X");
        assert!(found.apps[0].default);
    }

    #[test]
    fn a_terminal_program_opens_in_terminal_when_one_is_known() {
        let fs = Fake::default()
            .with("/d1/mime/globs2", GLOBS)
            .with(
                "/d1/applications/nvim.desktop",
                &format!(
                    "{}Terminal=true\n",
                    desktop("Neovim", "nvim %F", "text/plain")
                ),
            )
            .with(
                "/h/.config/mimeapps.list",
                "[Default Applications]\ntext/plain=nvim.desktop\n",
            );
        let with_term = |t: &str| {
            let t = t.to_string();
            XdgRoots::from_env(&move |k| match k {
                "HOME" => Some("/h".into()),
                "XDG_DATA_DIRS" => Some("/d1".into()),
                "TERMINAL" => Some(t.clone()),
                _ => None,
            })
        };
        // xdg-terminal-exec (Omarchy's $TERMINAL) takes the command as is.
        let found = apps_for("notes.txt", &with_term("xdg-terminal-exec"), &fs);
        assert_eq!(found.apps[0].command, "xdg-terminal-exec nvim ${file}");
        assert!(found.apps[0].default);
        assert_eq!(found.apps[0].key, "nvim");
        // Any other emulator gets -e.
        let found = apps_for("notes.txt", &with_term("/usr/bin/alacritty"), &fs);
        assert_eq!(found.apps[0].command, "/usr/bin/alacritty -e nvim ${file}");
        // No terminal known: left out, as before.
        assert!(apps_for("notes.txt", &roots(), &fs).apps.is_empty());
    }

    #[test]
    fn an_earlier_data_dir_shadows_a_later_desktop_id_and_subfolders_join_with_dashes() {
        let fs = Fake::default()
            .with("/d1/mime/globs2", GLOBS)
            .with(
                "/d1/applications/same.desktop",
                &desktop("Mine", "mine %f", "application/pdf"),
            )
            .with(
                "/d2/applications/same.desktop",
                &desktop("System", "system %f", "application/pdf"),
            )
            .with(
                "/d2/applications/kde/viewer.desktop",
                &desktop("Viewer", "viewer %f", "application/pdf"),
            )
            .with(
                "/h/.config/mimeapps.list",
                "[Default Applications]\napplication/pdf=kde-viewer.desktop\n",
            );
        let found = apps_for("x.pdf", &roots(), &fs);
        let names: Vec<_> = found.apps.iter().map(|a| a.name.as_str()).collect();
        assert_eq!(names, ["Viewer", "Mine"]);
    }

    #[test]
    fn exec_lines_expand_field_codes_and_keep_quoted_words_whole() {
        let t = |e: &str| exec_template(e, "App");
        assert_eq!(t("evince %U").as_deref(), Some("evince ${file}"));
        assert_eq!(
            t("app --open=%f -x").as_deref(),
            Some("app --open=${file} -x")
        );
        assert_eq!(t("app %i %c %k %f").as_deref(), Some("app App ${file}"));
        assert_eq!(t("app").as_deref(), Some("app ${file}"));
        assert_eq!(t("app 100%% %F").as_deref(), Some("app 100% ${file}"));
        assert_eq!(
            t(r#""/opt/My App/run" --flag "a \"b\" c" %f"#).as_deref(),
            Some(r#""/opt/My App/run" --flag "a \"b\" c" ${file}"#)
        );
        assert_eq!(
            t("env FOO=bar app %f").as_deref(),
            Some("env FOO=bar app ${file}")
        );
        assert_eq!(t(""), None);
        assert_eq!(t("%f"), None, "no program");
        assert_eq!(t("app ${HOME}"), None, "a literal variable would be ours");
        // The result splits back into the same words.
        let argv = crate::launch::split_argv(&t(r#""/opt/My App/run" "a b" %f"#).unwrap()).unwrap();
        assert_eq!(argv, ["/opt/My App/run", "a b", "${file}"]);
    }

    #[test]
    fn text_types_are_flagged_and_unknown_types_have_no_apps() {
        let fs = Fake::default().with("/d1/mime/globs2", GLOBS).with(
            "/d1/applications/ed.desktop",
            &desktop("Ed", "ed %f", "text/plain"),
        );
        let found = apps_for("notes.txt", &roots(), &fs);
        assert!(found.textual);
        assert_eq!(found.apps.len(), 1);
        assert!(apps_for("blob.zzz", &roots(), &fs).apps.is_empty());
        assert!(
            apps_for("blob.pdf", &roots(), &fs).apps.is_empty(),
            "known type, no handler"
        );
    }
}
