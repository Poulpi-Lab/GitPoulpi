//! Submodules through the system `git`: the list is `.gitmodules` (what is
//! declared) joined with `git ls-files --stage` (what the superproject
//! records) and `git submodule status` (what is checked out).

use std::collections::BTreeMap;

use gitpoulpi_core::domain::{Oid, RepoRef, Submodule, SubmoduleState, SubmoduleUpdateOptions};
use gitpoulpi_core::Result;

use crate::process::{run, run_lenient, run_network};

/// One `.gitmodules` section's keys.
#[derive(Default)]
struct Declared {
    name: String,
    path: String,
    url: String,
    branch: Option<String>,
}

/// Parses `git config -z --get-regexp ^submodule\.` output: records are
/// `key\nvalue\0`, a key being `submodule.<name>.<variable>` where the
/// name may itself contain dots.
fn parse_declared(raw: &str) -> Vec<Declared> {
    let mut by_name: BTreeMap<String, Declared> = BTreeMap::new();
    for record in raw.split('\0').filter(|r| !r.is_empty()) {
        let (key, value) = record.split_once('\n').unwrap_or((record, ""));
        let Some(rest) = key.strip_prefix("submodule.") else {
            continue;
        };
        let Some((name, variable)) = rest.rsplit_once('.') else {
            continue;
        };
        let entry = by_name.entry(name.to_string()).or_insert_with(|| Declared {
            name: name.to_string(),
            ..Declared::default()
        });
        match variable {
            "path" => entry.path = value.to_string(),
            "url" => entry.url = value.to_string(),
            "branch" => entry.branch = Some(value.to_string()),
            _ => {}
        }
    }
    by_name
        .into_values()
        .filter(|d| !d.path.is_empty())
        .collect()
}

/// `git submodule status` lines: `<flag><sha> <path>[ (<describe>)]`.
fn parse_status(raw: &str, declared: &[Declared]) -> BTreeMap<String, (char, Option<Oid>)> {
    let mut out = BTreeMap::new();
    for line in raw.lines() {
        let Some(flag) = line.chars().next() else {
            continue;
        };
        let rest = &line[flag.len_utf8()..];
        let Some((sha, after)) = rest.split_once(' ') else {
            continue;
        };
        let Some(path) = declared
            .iter()
            .map(|d| d.path.as_str())
            .find(|p| after == *p || after.starts_with(&format!("{p} (")))
        else {
            continue;
        };
        out.insert(path.to_string(), (flag, Oid::from_hex(sha)));
    }
    out
}

pub fn submodules(repo: &RepoRef) -> Result<Vec<Submodule>> {
    if !repo.root.join(".gitmodules").is_file() {
        return Ok(Vec::new());
    }
    let (found, raw) = run_lenient(
        &repo.root,
        &[
            "config",
            "-z",
            "-f",
            ".gitmodules",
            "--get-regexp",
            r"^submodule\.",
        ],
    )?;
    if !found {
        return Ok(Vec::new());
    }
    let declared = parse_declared(&raw);
    if declared.is_empty() {
        return Ok(Vec::new());
    }

    // What the superproject records: gitlink entries (mode 160000).
    let mut args: Vec<&str> = vec!["ls-files", "--stage", "-z", "--"];
    args.extend(declared.iter().map(|d| d.path.as_str()));
    let staged = run(&repo.root, &args)?;
    let mut expected: BTreeMap<String, Oid> = BTreeMap::new();
    for record in staged.split('\0').filter(|r| !r.is_empty()) {
        // `<mode> <sha> <stage>\t<path>`
        let Some((meta, path)) = record.split_once('\t') else {
            continue;
        };
        let mut fields = meta.split(' ');
        if fields.next() != Some("160000") {
            continue;
        }
        if let Some(oid) = fields.next().and_then(Oid::from_hex) {
            expected.entry(path.to_string()).or_insert(oid);
        }
    }

    let status = parse_status(&run(&repo.root, &["submodule", "status"])?, &declared);
    let mut out: Vec<Submodule> = declared
        .into_iter()
        .map(|d| {
            let (flag, head) = status.get(&d.path).copied().unwrap_or(('-', None));
            let state = match flag {
                '-' => SubmoduleState::Uninitialized,
                '+' => SubmoduleState::Modified,
                'U' => SubmoduleState::Conflicted,
                _ => SubmoduleState::UpToDate,
            };
            Submodule {
                expected: expected.get(&d.path).copied(),
                head: head.filter(|_| state != SubmoduleState::Uninitialized),
                name: d.name,
                url: d.url,
                branch: d.branch,
                state,
                path: d.path,
            }
        })
        .collect();
    out.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(out)
}

pub fn submodule_init(repo: &RepoRef, paths: &[String]) -> Result<()> {
    let mut args: Vec<&str> = vec!["submodule", "init"];
    if !paths.is_empty() {
        args.push("--");
        args.extend(paths.iter().map(String::as_str));
    }
    run(&repo.root, &args)?;
    Ok(())
}

pub fn submodule_update(repo: &RepoRef, opts: &SubmoduleUpdateOptions) -> Result<()> {
    let mut args: Vec<&str> = vec!["submodule", "update"];
    if opts.init {
        args.push("--init");
    }
    if opts.recursive {
        args.push("--recursive");
    }
    if !opts.paths.is_empty() {
        args.push("--");
        args.extend(opts.paths.iter().map(String::as_str));
    }
    run_network(&repo.root, &args)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn declared_keys_split_on_the_last_dot_so_names_may_contain_dots() {
        let raw = "submodule.libs/a.b.path\nlibs/a.b\0submodule.libs/a.b.url\n../a\0\
                   submodule.libs/a.b.branch\ndev\0";
        let declared = parse_declared(raw);
        assert_eq!(declared.len(), 1);
        assert_eq!(declared[0].name, "libs/a.b");
        assert_eq!(declared[0].path, "libs/a.b");
        assert_eq!(declared[0].url, "../a");
        assert_eq!(declared[0].branch.as_deref(), Some("dev"));
    }

    #[test]
    fn status_lines_are_matched_to_declared_paths_even_with_spaces() {
        let declared = vec![Declared {
            name: "my lib".into(),
            path: "my lib".into(),
            ..Declared::default()
        }];
        let sha = "a".repeat(40);
        let parsed = parse_status(&format!("+{sha} my lib (heads/main)\n"), &declared);
        assert_eq!(parsed["my lib"].0, '+');
        assert!(parsed["my lib"].1.is_some());
    }
}
