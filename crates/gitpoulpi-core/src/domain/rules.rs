//! Where an ignore or attribute rule can be written, and the exact line to
//! write there. Pure text work shared by the backends (which write the
//! files), the TUI (which lists the candidate files in its picker) and the
//! CLI, so all three agree on what `src/a/b.txt` becomes inside
//! `src/.gitignore`.

use serde::{Deserialize, Serialize};

/// `.git/info/exclude`: the per-clone `.gitignore`, never committed.
pub const INFO_EXCLUDE: &str = ".git/info/exclude";
/// `.git/info/attributes`: the per-clone `.gitattributes`, never committed.
pub const INFO_ATTRIBUTES: &str = ".git/info/attributes";

/// Which family of rule file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuleKind {
    Ignore,
    Attributes,
}

impl RuleKind {
    /// The committed file's name (`.gitignore` / `.gitattributes`).
    pub fn file_name(self) -> &'static str {
        match self {
            RuleKind::Ignore => ".gitignore",
            RuleKind::Attributes => ".gitattributes",
        }
    }

    /// The per-clone file (`.git/info/exclude` / `.git/info/attributes`).
    pub fn local_file(self) -> &'static str {
        match self {
            RuleKind::Ignore => INFO_EXCLUDE,
            RuleKind::Attributes => INFO_ATTRIBUTES,
        }
    }
}

/// What a rule names.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuleScope {
    /// Exactly this file.
    File,
    /// Every file with this file's extension (`*.ext`).
    Extension,
    /// Everything inside this folder (the path is the folder).
    Folder,
}

/// Whether git should diff a path as text or as binary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Treat {
    Binary,
    Text,
}

impl Treat {
    /// The attributes written after the pattern. `binary` is git's macro for
    /// `-diff -merge -text`. There is no "text" macro: `diff` makes `git
    /// diff` print hunks even when the content has NUL bytes, and `text`
    /// marks it for line-ending normalisation like any text file.
    pub fn attributes(self) -> &'static [&'static str] {
        match self {
            Treat::Binary => &["binary"],
            Treat::Text => &["text", "diff"],
        }
    }
}

/// What `GitBackend::gitattributes_treat` did.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttributesWrite {
    /// The attributes file written, repo-relative (`.gitattributes`,
    /// `src/.gitattributes` or `.git/info/attributes`).
    pub file: String,
    /// The patterns it now holds a rule for.
    pub patterns: Vec<String>,
    /// `false`: every rule was already there, nothing was written.
    pub changed: bool,
    /// Whether git now treats every path as asked (`false`: another rule
    /// overrides the one just written).
    pub effective: bool,
}

fn dir_of(path: &str) -> &str {
    path.rsplit_once('/').map_or("", |(dir, _)| dir)
}

/// The folder a rule file governs, repo-relative (`""` for the root).
/// `.git/info/*` files govern the whole repository.
pub fn rule_dir(file: &str) -> &str {
    if file.starts_with(".git/") {
        ""
    } else {
        dir_of(file)
    }
}

/// The files a rule for `path` can go to, in the picker's order: the
/// repository's root file first (the default), then the file of every
/// folder down to the one holding the path, and last the per-clone file.
/// For a folder path, its own file is not offered (a rule inside what it
/// governs), so both a file and a folder stop at the path's parent.
pub fn candidate_files(kind: RuleKind, path: &str) -> Vec<String> {
    let path = path.replace('\\', "/");
    let path = path.trim_matches('/');
    let mut out = vec![kind.file_name().to_string()];
    let mut acc = String::new();
    for part in dir_of(path).split('/').filter(|p| !p.is_empty()) {
        if !acc.is_empty() {
            acc.push('/');
        }
        acc.push_str(part);
        out.push(format!("{acc}/{}", kind.file_name()));
    }
    out.push(kind.local_file().to_string());
    out
}

/// Escapes what git's glob would read as magic in a file name.
fn escape_glob(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    for c in name.chars() {
        if matches!(c, '\\' | '*' | '?' | '[') {
            out.push('\\');
        }
        out.push(c);
    }
    if out.ends_with(' ') {
        out.pop();
        out.push_str("\\ ");
    }
    out
}

/// The pattern line that names `path` (repo-relative, `/` separated; `\`
/// is accepted) from inside `file`'s folder, or `None` when it cannot:
/// the path is outside that folder, or `Extension` of a file without one.
/// Anchored with a leading `/` for a file or a folder, so `a/b.txt` in
/// `src/.gitignore` is `/a/b.txt` and never matches `other/a/b.txt`.
pub fn rule_pattern(kind: RuleKind, file: &str, path: &str, scope: RuleScope) -> Option<String> {
    let path = path.replace('\\', "/");
    let path = path.trim_matches('/');
    if path.is_empty() {
        return None;
    }
    let dir = rule_dir(file);
    let rel = if dir.is_empty() {
        path
    } else {
        path.strip_prefix(dir)?.strip_prefix('/')?
    };
    if rel.is_empty() {
        return None;
    }
    match scope {
        RuleScope::File => Some(format!("/{}", escape_glob(rel))),
        RuleScope::Extension => {
            let name = rel.rsplit('/').next()?;
            let ext = name
                .rsplit_once('.')
                .filter(|(stem, _)| !stem.is_empty())?
                .1;
            if ext.is_empty() {
                return None;
            }
            Some(format!("*.{}", escape_glob(ext)))
        }
        // `dir/` is not a valid attribute pattern (attributes never match
        // a directory), `dir/**` is; in a `.gitignore` the trailing slash
        // is the idiom.
        RuleScope::Folder => Some(match kind {
            RuleKind::Ignore => format!("/{}/", escape_glob(rel)),
            RuleKind::Attributes => format!("/{}/**", escape_glob(rel)),
        }),
    }
}

/// Quotes `pattern` for a `.gitattributes` line when it holds whitespace
/// (git reads a C-style quoted first field).
pub fn attribute_field(pattern: &str) -> String {
    if pattern.chars().any(char::is_whitespace) {
        format!("\"{}\"", pattern.replace('\\', "\\\\").replace('"', "\\\""))
    } else {
        pattern.to_string()
    }
}

/// The first field of an attributes line, unquoted, and the rest of it.
fn split_attribute_line(line: &str) -> Option<(String, &str)> {
    let line = line.trim_start();
    if line.is_empty() || line.starts_with('#') {
        return None;
    }
    if let Some(quoted) = line.strip_prefix('"') {
        let mut out = String::new();
        let mut chars = quoted.char_indices();
        while let Some((i, c)) = chars.next() {
            match c {
                '\\' => {
                    if let Some((_, n)) = chars.next() {
                        out.push(n);
                    }
                }
                '"' => return Some((out, &quoted[i + 1..])),
                c => out.push(c),
            }
        }
        return None;
    }
    let end = line.find(char::is_whitespace).unwrap_or(line.len());
    Some((line[..end].to_string(), &line[end..]))
}

/// Whether an attribute token sets, unsets or resets one of the attributes
/// a [`Treat`] decides, so it must go when the rule is rewritten.
fn is_treatment_token(token: &str) -> bool {
    let name = token.trim_start_matches(['-', '!']);
    // `diff=lfs` and `merge=lfs` name a driver, not a treatment: left alone.
    if let Some((name, _)) = name.split_once('=') {
        return name == "text";
    }
    matches!(name, "binary" | "text" | "diff" | "merge")
}

/// `existing` with `pattern` set to `treat`: the line already naming that
/// pattern has its contradicting attributes (`binary`, `-diff`, `text`,
/// `-text`, `diff`, `-merge`, ...) replaced, others kept (`filter=lfs`); a
/// new line is appended when there is none. The second value is whether
/// the text changed.
pub fn attributes_set(existing: &str, pattern: &str, treat: Treat) -> (String, bool) {
    let wanted = treat.attributes();
    let mut out: Vec<String> = Vec::new();
    let mut done = false;
    for line in existing.lines() {
        let Some((field, rest)) = split_attribute_line(line) else {
            out.push(line.to_string());
            continue;
        };
        if field != pattern {
            out.push(line.to_string());
            continue;
        }
        let mut tokens: Vec<&str> = rest
            .split_whitespace()
            .filter(|t| !is_treatment_token(t))
            .collect();
        if !done {
            tokens.extend(wanted.iter());
            done = true;
        }
        // A duplicate line left with no attribute at all is dropped.
        if tokens.is_empty() {
            continue;
        }
        out.push(format!("{} {}", attribute_field(pattern), tokens.join(" ")));
    }
    if !done {
        out.push(format!("{} {}", attribute_field(pattern), wanted.join(" ")));
    }
    let mut text = out.join("\n");
    text.push('\n');
    let changed = text != existing;
    (text, changed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use RuleKind::{Attributes, Ignore};

    #[test]
    fn candidates_run_root_first_then_each_folder_then_the_local_file() {
        assert_eq!(
            candidate_files(Ignore, "src/a/b.txt"),
            [
                ".gitignore",
                "src/.gitignore",
                "src/a/.gitignore",
                ".git/info/exclude"
            ]
        );
        assert_eq!(
            candidate_files(Attributes, "top.bin"),
            [".gitattributes", ".git/info/attributes"]
        );
        // A folder is not offered its own file.
        assert_eq!(
            candidate_files(Ignore, "src/a"),
            [".gitignore", "src/.gitignore", ".git/info/exclude"]
        );
        assert_eq!(
            candidate_files(Ignore, "src\\a\\b.txt")[2],
            "src/a/.gitignore"
        );
    }

    #[test]
    fn a_pattern_is_relative_to_the_rule_files_folder() {
        let p = |file, path, scope| rule_pattern(Ignore, file, path, scope);
        let file = RuleScope::File;
        assert_eq!(
            p(".gitignore", "src/a/b.txt", file).as_deref(),
            Some("/src/a/b.txt")
        );
        assert_eq!(
            p("src/.gitignore", "src/a/b.txt", file).as_deref(),
            Some("/a/b.txt")
        );
        assert_eq!(
            p("src/a/.gitignore", "src/a/b.txt", file).as_deref(),
            Some("/b.txt")
        );
        assert_eq!(
            p(INFO_EXCLUDE, "src/a/b.txt", file).as_deref(),
            Some("/src/a/b.txt")
        );
        assert_eq!(p("other/.gitignore", "src/a/b.txt", file), None);
        // `srcx/` is not inside `src/`.
        assert_eq!(p("src/.gitignore", "srcx/b.txt", file), None);
        let ext = RuleScope::Extension;
        assert_eq!(
            p("src/.gitignore", "src/a/b.txt", ext).as_deref(),
            Some("*.txt")
        );
        assert_eq!(p(".gitignore", "Makefile", ext), None);
        assert_eq!(p(".gitignore", ".env", ext), None);
        let dir = RuleScope::Folder;
        assert_eq!(p(".gitignore", "src/a", dir).as_deref(), Some("/src/a/"));
        assert_eq!(p("src/.gitignore", "src/a", dir).as_deref(), Some("/a/"));
        assert_eq!(
            rule_pattern(Attributes, "src/.gitattributes", "src/a", dir).as_deref(),
            Some("/a/**")
        );
        assert_eq!(
            p(".gitignore", "a*b[1].txt", file).as_deref(),
            Some("/a\\*b\\[1].txt")
        );
    }

    #[test]
    fn setting_a_treatment_replaces_a_contradicting_rule_for_the_same_pattern() {
        let (t, c) = attributes_set("", "*.dat", Treat::Binary);
        assert_eq!((t.as_str(), c), ("*.dat binary\n", true));
        // The same again: nothing to write.
        assert!(!attributes_set(&t, "*.dat", Treat::Binary).1);
        // Flip it: the line is replaced, not doubled.
        let (t, c) = attributes_set(&t, "*.dat", Treat::Text);
        assert_eq!((t.as_str(), c), ("*.dat text diff\n", true));
        let (t, _) = attributes_set(&t, "*.dat", Treat::Binary);
        assert_eq!(t, "*.dat binary\n");
        // Another pattern, a comment and an unrelated attribute survive.
        let src = "# keep\n*.psd filter=lfs diff=lfs -text\n*.dat -diff -merge\nREADME eol=lf";
        let (t, _) = attributes_set(src, "*.dat", Treat::Text);
        assert_eq!(
            t,
            "# keep\n*.psd filter=lfs diff=lfs -text\n*.dat text diff\nREADME eol=lf\n"
        );
        // Other attributes of the same pattern stay.
        let (t, _) = attributes_set("*.psd filter=lfs diff=lfs -text\n", "*.psd", Treat::Binary);
        assert_eq!(t, "*.psd filter=lfs diff=lfs binary\n");
        // A quoted pattern is found again.
        let (t, _) = attributes_set("\"a b.dat\" text\n", "a b.dat", Treat::Binary);
        assert_eq!(t, "\"a b.dat\" binary\n");
        let (t, _) = attributes_set("", "/a b.dat", Treat::Binary);
        assert_eq!(t, "\"/a b.dat\" binary\n");
    }
}
