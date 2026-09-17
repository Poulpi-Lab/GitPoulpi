//! The files git reads rules from, written on the user's behalf:
//! `.gitignore` / `.git/info/exclude` lines (`gitignore_add`) and
//! `.gitattributes` / `.git/info/attributes` treatments
//! (`gitattributes_treat`), plus the question the second one answers
//! (`binary_paths`).

use std::path::{Path, PathBuf};

use gitpoulpi_core::domain::{
    attributes_set, rule_pattern, AttributesWrite, RepoRef, RuleKind, RuleScope, Treat,
};
use gitpoulpi_core::{Error, Result};

use crate::process::{run, run_bytes_with_stdin};
use crate::snapshot_ops::normalize;

/// How many leading bytes git looks at to call a file binary.
const SNIFF_BYTES: usize = 8000;

/// The repo-relative name of the rule file asked for and where it is on
/// disk. `None` is the root file of `kind`; the per-clone file is resolved
/// by git (`.git` may be a file in a linked worktree, and the file is
/// shared by all of a repository's worktrees); any other name must be a
/// `kind` file inside the repository.
fn resolve(repo: &RepoRef, kind: RuleKind, file: Option<&str>) -> Result<(String, PathBuf)> {
    let rel = match file {
        None => kind.file_name().to_string(),
        Some(f) if f.replace('\\', "/") == kind.local_file() => {
            let name = kind.local_file().trim_start_matches(".git/");
            let path = run(&repo.root, &["rev-parse", "--git-path", name])?;
            return Ok((kind.local_file().to_string(), repo.root.join(path.trim())));
        }
        Some(f) => normalize(f)?,
    };
    if rel.rsplit('/').next() != Some(kind.file_name()) {
        return Err(Error::Backend(format!(
            "{rel} is not a {} file",
            kind.file_name()
        )));
    }
    let full = repo.root.join(&rel);
    Ok((rel, full))
}

pub(crate) fn read_or_empty(full: &Path, rel: &str) -> Result<String> {
    match std::fs::read_to_string(full) {
        Ok(text) => Ok(text),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(String::new()),
        Err(e) => Err(Error::Backend(format!("cannot read {rel}: {e}"))),
    }
}

pub(crate) fn write(full: &Path, rel: &str, text: String) -> Result<()> {
    if let Some(parent) = full.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| Error::Backend(format!("cannot create {}: {e}", parent.display())))?;
    }
    std::fs::write(full, text).map_err(|e| Error::Backend(format!("cannot write {rel}: {e}")))
}

/// Appends `pattern` as its own line to a `.gitignore` or to
/// `.git/info/exclude`; see `GitBackend::gitignore_add`.
pub fn gitignore_add(repo: &RepoRef, pattern: &str, file: Option<&str>) -> Result<bool> {
    let pattern = pattern.trim();
    if pattern.is_empty() || pattern.contains(['\n', '\r']) {
        return Err(Error::Backend(
            "an ignore pattern is one non-empty line".into(),
        ));
    }
    let (rel, full) = resolve(repo, RuleKind::Ignore, file)?;
    let existing = read_or_empty(&full, &rel)?;
    if existing.lines().any(|l| l.trim() == pattern) {
        return Ok(false);
    }
    let mut next = existing;
    if !next.is_empty() && !next.ends_with('\n') {
        next.push('\n');
    }
    next.push_str(pattern);
    next.push('\n');
    write(&full, &rel, next)?;
    Ok(true)
}

/// See `GitBackend::gitattributes_treat`.
pub fn gitattributes_treat(
    repo: &RepoRef,
    paths: &[String],
    scope: RuleScope,
    file: Option<&str>,
    treat: Treat,
) -> Result<AttributesWrite> {
    if paths.is_empty() {
        return Err(Error::Backend("no path to treat".into()));
    }
    let (rel, full) = resolve(repo, RuleKind::Attributes, file)?;
    let mut patterns: Vec<String> = Vec::new();
    let mut files: Vec<String> = Vec::new();
    for path in paths {
        let path = normalize(path)?;
        let pattern = rule_pattern(RuleKind::Attributes, &rel, &path, scope).ok_or_else(|| {
            Error::Backend(match scope {
                RuleScope::Extension => format!("{path} has no extension"),
                _ => format!("{path} is not inside the folder of {rel}"),
            })
        })?;
        if !patterns.contains(&pattern) {
            patterns.push(pattern);
        }
        files.push(path);
    }
    let existing = read_or_empty(&full, &rel)?;
    let mut text = existing.clone();
    for pattern in &patterns {
        text = attributes_set(&text, pattern, treat).0;
    }
    let changed = text != existing;
    if changed {
        write(&full, &rel, text)?;
    }
    // A folder is not a file git can be asked about.
    let effective = scope == RuleScope::Folder
        || binary_paths(repo, &files)?
            .iter()
            .all(|&b| b == (treat == Treat::Binary));
    Ok(AttributesWrite {
        file: rel,
        patterns,
        changed,
        effective,
    })
}

/// See `GitBackend::binary_paths`.
pub fn binary_paths(repo: &RepoRef, paths: &[String]) -> Result<Vec<bool>> {
    if paths.is_empty() {
        return Ok(Vec::new());
    }
    let paths: Vec<String> = paths.iter().map(|p| normalize(p)).collect::<Result<_>>()?;
    let mut input = Vec::new();
    for p in &paths {
        input.extend_from_slice(p.as_bytes());
        input.push(0);
    }
    let raw = run_bytes_with_stdin(&repo.root, &["check-attr", "-z", "--stdin", "diff"], &input)?;
    // `path NUL attribute NUL value NUL`, one record per path.
    let mut fields = raw.split(|&b| b == 0);
    let mut values: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    while let (Some(path), Some(_attr), Some(value)) = (fields.next(), fields.next(), fields.next())
    {
        values.insert(
            String::from_utf8_lossy(path).into_owned(),
            String::from_utf8_lossy(value).into_owned(),
        );
    }
    Ok(paths
        .iter()
        .map(|p| match values.get(p).map(String::as_str) {
            Some("unset") => true,
            Some("unspecified") | None => sniff_binary(repo, p),
            // `set`, or a driver (`diff=lfs`): diffed as text.
            Some(_) => false,
        })
        .collect())
}

fn sniff_binary(repo: &RepoRef, path: &str) -> bool {
    use std::io::Read;
    let Ok(mut file) = std::fs::File::open(repo.root.join(path)) else {
        return false;
    };
    let mut buf = vec![0u8; SNIFF_BYTES];
    let n = file.read(&mut buf).unwrap_or(0);
    buf[..n].contains(&0)
}
