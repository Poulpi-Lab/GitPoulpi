//! The undo journal's backend primitives over the system `git`
//! (`ref_snapshot`, `update_refs`, `config_section`,
//! `set_config_section`). See the undo-redo design §2.7.

use std::collections::BTreeMap;

use gitpoulpi_core::domain::{
    HeadState, Oid, RefScope, RefSnapshot, RefUpdate, RepoRef, UndoRefusal,
};
use gitpoulpi_core::{Error, Result};

use crate::process::{run, run_bytes, run_lenient, run_with_stdin};

/// One `for-each-ref` (HEAD marked with `%(HEAD)`), plus a `symbolic-ref`
/// (and a `rev-parse` when detached) only when HEAD is not on a listed
/// branch: unborn, detached, or a scope without branches.
pub fn ref_snapshot(repo: &RepoRef, scope: &RefScope) -> Result<RefSnapshot> {
    let mut args: Vec<&str> = vec![
        "for-each-ref",
        "--format=%(HEAD)%00%(refname)%00%(objectname)%00%(symref)",
    ];
    let patterns: Vec<&str> = scope
        .prefixes
        .iter()
        .map(|p| p.trim_end_matches('/'))
        .collect();
    args.extend(patterns.iter().copied());
    let raw = run(&repo.root, &args)?;
    let mut refs = BTreeMap::new();
    let mut symrefs = BTreeMap::new();
    let mut head = None;
    for line in raw.lines().filter(|l| !l.is_empty()) {
        let mut parts = line.splitn(4, '\0');
        let (Some(marker), Some(name), Some(oid)) = (parts.next(), parts.next(), parts.next())
        else {
            continue;
        };
        if let Some(target) = parts.next().map(str::trim).filter(|t| !t.is_empty()) {
            symrefs.insert(name.to_string(), target.to_string());
        }
        let Some(oid) = Oid::from_hex(oid.trim()) else {
            continue;
        };
        if marker == "*" {
            if let Some(branch) = name.strip_prefix("refs/heads/") {
                head = Some(HeadState::Branch(branch.to_string()));
            }
        }
        refs.insert(name.to_string(), oid);
    }
    let head = match head {
        Some(head) => head,
        None => current_head(repo)?,
    };
    Ok(RefSnapshot {
        head,
        refs,
        symrefs,
    })
}

/// HEAD as it is right now.
fn current_head(repo: &RepoRef) -> Result<HeadState> {
    let (symbolic, target) = run_lenient(&repo.root, &["symbolic-ref", "-q", "HEAD"])?;
    if symbolic {
        let target = target.trim();
        return Ok(HeadState::Branch(
            target
                .strip_prefix("refs/heads/")
                .unwrap_or(target)
                .to_string(),
        ));
    }
    let raw = run(&repo.root, &["rev-parse", "--verify", "HEAD"])?;
    Oid::from_hex(raw.trim())
        .map(HeadState::Detached)
        .ok_or_else(|| Error::Backend("HEAD did not resolve to a commit".into()))
}

fn ref_value(repo: &RepoRef, name: &str) -> Option<Oid> {
    let (ok, out) = run_lenient(&repo.root, &["rev-parse", "--verify", "-q", name]).ok()?;
    if !ok {
        return None;
    }
    Oid::from_hex(out.trim())
}

/// `git update-ref --stdin -z`: one transaction, so a stale expectation
/// anywhere changes nothing. A failure is turned into the typed
/// `RefMoved` by re-reading the refs involved.
pub fn update_refs(repo: &RepoRef, updates: &[RefUpdate], reflog_message: &str) -> Result<()> {
    // Keep-alive pins are the hot case (every journaled operation writes
    // some): a loose ref is a one-line file, so write it ourselves rather
    // than spend a process on it.
    if write_pins_directly(repo, updates) || delete_pins_directly(repo, updates) {
        return Ok(());
    }
    // HEAD moves are verified first, applied after the ref transaction.
    let mut head_moves: Vec<(&HeadState, &HeadState)> = Vec::new();
    let mut symbolic: Vec<(&String, &Option<String>)> = Vec::new();
    let mut input = String::from("start\0");
    let mut any_ref = false;
    for update in updates {
        match update {
            RefUpdate::SetHead { old, new } => {
                let actual = current_head(repo)?;
                if &actual != old {
                    return Err(Error::UndoRefused(UndoRefusal::HeadMoved {
                        expected: old.clone(),
                        actual,
                    }));
                }
                match (old, new) {
                    // Atomic with the rest.
                    (HeadState::Detached(o), HeadState::Detached(n)) => {
                        input.push_str(&format!("update HEAD\0{n}\0{o}\0"));
                        any_ref = true;
                    }
                    _ => head_moves.push((old, new)),
                }
            }
            RefUpdate::SetSymbolic { name, new, .. } => symbolic.push((name, new)),
            RefUpdate::Create { name, new } => {
                input.push_str(&format!("create {name}\0{new}\0"));
                any_ref = true;
            }
            RefUpdate::Update { name, old, new } => {
                input.push_str(&format!("update {name}\0{new}\0{old}\0"));
                any_ref = true;
            }
            RefUpdate::Delete { name, old } => {
                input.push_str(&format!("delete {name}\0{old}\0"));
                any_ref = true;
            }
            RefUpdate::Verify { name, old } => {
                input.push_str(&format!("verify {name}\0{old}\0"));
                any_ref = true;
            }
        }
    }
    if any_ref {
        input.push_str("prepare\0commit\0");
        let result = run_with_stdin(
            &repo.root,
            &[
                "update-ref",
                "--no-deref",
                "-m",
                reflog_message,
                "--stdin",
                "-z",
            ],
            &input,
        );
        if let Err(e) = result {
            return Err(explain_failure(repo, updates, e));
        }
    }
    for (name, target) in symbolic {
        match target {
            Some(target) => {
                run(
                    &repo.root,
                    &["symbolic-ref", "-m", reflog_message, name, target],
                )?;
            }
            None => {
                run_lenient(&repo.root, &["symbolic-ref", "--delete", "-q", name])?;
            }
        }
    }
    for (_, new) in head_moves {
        match new {
            HeadState::Branch(name) => {
                run(
                    &repo.root,
                    &[
                        "symbolic-ref",
                        "-m",
                        reflog_message,
                        "HEAD",
                        &format!("refs/heads/{name}"),
                    ],
                )?;
            }
            HeadState::Detached(oid) => {
                run(
                    &repo.root,
                    &[
                        "update-ref",
                        "--no-deref",
                        "-m",
                        reflog_message,
                        "HEAD",
                        &oid.to_hex(),
                    ],
                )?;
            }
        }
    }
    Ok(())
}

/// A failed transaction: name the first ref that is not as expected, or
/// pass git's own message through when every expectation holds.
fn explain_failure(repo: &RepoRef, updates: &[RefUpdate], original: Error) -> Error {
    for update in updates {
        let (name, expected) = match update {
            RefUpdate::Create { name, .. } => (name, None),
            RefUpdate::Update { name, old, .. }
            | RefUpdate::Delete { name, old }
            | RefUpdate::Verify { name, old } => (name, Some(*old)),
            RefUpdate::SetHead { .. } | RefUpdate::SetSymbolic { .. } => continue,
        };
        let actual = ref_value(repo, name);
        if actual != expected {
            return Error::UndoRefused(UndoRefusal::RefMoved {
                name: name.clone(),
                expected,
                actual,
                restore: None,
            });
        }
    }
    original
}

/// `git reset --keep`: the checked-out `branch` moves from `from` to `to`
/// and the index and files follow, refusing (nothing changed) when a
/// local change would be overwritten.
pub fn move_head_keep(repo: &RepoRef, branch: &str, from: Oid, to: Oid) -> Result<()> {
    let expected = HeadState::Branch(branch.to_string());
    let actual = current_head(repo)?;
    if actual != expected {
        return Err(Error::UndoRefused(UndoRefusal::HeadMoved {
            expected,
            actual,
        }));
    }
    let name = format!("refs/heads/{branch}");
    let tip = ref_value(repo, &name);
    if tip != Some(from) {
        return Err(Error::UndoRefused(UndoRefusal::RefMoved {
            name,
            expected: Some(from),
            actual: tip,
            restore: None,
        }));
    }
    release_submodules(repo, from, to)?;
    run(&repo.root, &["reset", "--keep", "--quiet", &to.to_hex()]).map_err(|e| {
        Error::UndoRefused(UndoRefusal::WouldOverwrite {
            detail: e.to_string(),
        })
    })?;
    Ok(())
}

/// A checked-out submodule that the move turns back into a plain folder
/// (undoing "extract as a submodule") is de-initialised first: git would
/// otherwise leave its files in the way of the folder's own. Edits inside
/// it refuse the move, nothing changed.
fn release_submodules(repo: &RepoRef, from: Oid, to: Oid) -> Result<()> {
    let raw = run_bytes(
        &repo.root,
        &[
            "diff-tree",
            "-r",
            "-z",
            "--no-renames",
            "--raw",
            &from.to_hex(),
            &to.to_hex(),
        ],
    )?;
    let mut fields = raw.split(|b| *b == 0);
    let mut gone: Vec<String> = Vec::new();
    while let Some(meta) = fields.next() {
        if meta.is_empty() {
            continue;
        }
        let Some(path) = fields.next() else { break };
        let meta = String::from_utf8_lossy(meta);
        let mut parts = meta.trim_start_matches(':').split(' ');
        let (old_mode, new_mode) = (parts.next(), parts.next());
        if old_mode == Some("160000") && new_mode != Some("160000") {
            gone.push(String::from_utf8_lossy(path).into_owned());
        }
    }
    for path in gone {
        let dir = repo.root.join(&path);
        if !dir.join(".git").exists() {
            continue;
        }
        let dirty = run(&dir, &["status", "--porcelain"])?;
        if !dirty.trim().is_empty() {
            return Err(Error::UndoRefused(UndoRefusal::WouldOverwrite {
                detail: format!("{path} is a submodule with local changes"),
            }));
        }
        run(
            &repo.root,
            &["submodule", "deinit", "--force", "--quiet", "--", &path],
        )
        .map_err(|e| {
            Error::UndoRefused(UndoRefusal::WouldOverwrite {
                detail: e.to_string(),
            })
        })?;
    }
    Ok(())
}

/// `branch.<name>` and `remote.<name>` only: this is an undo helper, not
/// a config API.
pub fn check_section(section: &str) -> Result<()> {
    // The allow-listed repo-config keys (`core.autocrlf`, `core.eol`) are
    // one-variable pseudo-sections, so the undo journal restores them with
    // the same steps; `core` as a whole is never accepted.
    if is_key_section(section) {
        return Ok(());
    }
    match section
        .strip_prefix("branch.")
        .or_else(|| section.strip_prefix("remote."))
    {
        Some(name) if !name.is_empty() => Ok(()),
        _ => Err(Error::Backend(format!(
            "only `branch.<name>` and `remote.<name>` config sections can be read or \
             restored, not `{section}`"
        ))),
    }
}

/// The local config as `(key, value)` pairs, in file order.
fn local_entries(repo: &RepoRef) -> Result<Vec<(String, String)>> {
    let (_, raw) = run_lenient(&repo.root, &["config", "--local", "--list", "-z"])?;
    Ok(raw
        .split('\0')
        .filter(|entry| !entry.is_empty())
        .map(|entry| {
            let (key, value) = entry.split_once('\n').unwrap_or((entry, ""));
            (key.to_string(), value.to_string())
        })
        .collect())
}

/// One process for any number of sections.
pub fn config_sections(
    repo: &RepoRef,
    sections: &[String],
) -> Result<Vec<gitpoulpi_core::backend::ConfigSection>> {
    for section in sections {
        check_section(section)?;
    }
    if sections.is_empty() {
        return Ok(Vec::new());
    }
    let all = local_entries(repo)?;
    Ok(sections
        .iter()
        .map(|section| {
            let entries = all
                .iter()
                .filter(|(key, _)| in_section(section, key))
                .cloned()
                .collect();
            (section.clone(), entries)
        })
        .collect())
}

/// The `branch.<name>` sections whose `remote` is `remote`, sorted.
pub fn remote_dependents(repo: &RepoRef, remote: &str) -> Result<Vec<String>> {
    Ok(dependents_in(&local_entries(repo)?, remote))
}

/// [`remote_dependents`] over already-read config entries (git2 shares it).
pub fn dependents_in(entries: &[(String, String)], remote: &str) -> Vec<String> {
    let mut out: Vec<String> = entries
        .iter()
        .filter(|(key, value)| {
            key.starts_with("branch.") && key.ends_with(".remote") && value == remote
        })
        .map(|(key, _)| key[..key.len() - ".remote".len()].to_string())
        .collect();
    out.sort();
    out.dedup();
    out
}

/// Whether `key` is a variable of `section` (`branch.a` owns
/// `branch.a.remote` but not `branch.a.b.remote`, which is branch `a.b`'s).
pub fn in_section(section: &str, key: &str) -> bool {
    if is_key_section(section) {
        return key.eq_ignore_ascii_case(section);
    }
    key.strip_prefix(section)
        .and_then(|rest| rest.strip_prefix('.'))
        .is_some_and(|variable| !variable.is_empty() && !variable.contains('.'))
}

/// `section` is one allow-listed key standing for itself (see `check_section`).
pub fn is_key_section(section: &str) -> bool {
    gitpoulpi_core::repo_config::REPO_CONFIG_KEYS
        .iter()
        .any(|(k, _)| *k == section)
}

pub fn config_section(repo: &RepoRef, section: &str) -> Result<Vec<(String, String)>> {
    check_section(section)?;
    Ok(local_entries(repo)?
        .into_iter()
        .filter(|(key, _)| in_section(section, key))
        .collect())
}

pub fn set_config_section(
    repo: &RepoRef,
    section: &str,
    entries: &[(String, String)],
) -> Result<()> {
    check_section(section)?;
    if let Some((key, _)) = entries.iter().find(|(key, _)| !in_section(section, key)) {
        return Err(Error::Backend(format!(
            "`{key}` is not a variable of `{section}`"
        )));
    }
    if is_key_section(section) {
        for (key, value) in entries {
            gitpoulpi_core::repo_config::check_repo_config_value(key, value)?;
        }
        if !config_section(repo, section)?.is_empty() {
            run(&repo.root, &["config", "--local", "--unset-all", section])?;
        }
    } else if !config_section(repo, section)?.is_empty() {
        run(
            &repo.root,
            &["config", "--local", "--remove-section", section],
        )?;
    }
    for (key, value) in entries {
        run(&repo.root, &["config", "--local", "--add", key, value])?;
    }
    Ok(())
}

/// The directory holding `refs/` for this repository (the common git dir
/// of a linked worktree), only for the files ref backend: a reftable
/// repository does not read loose ref files.
fn loose_refs_root(repo: &RepoRef) -> Option<std::path::PathBuf> {
    let dir = gitpoulpi_core::paths::common_dir(&gitpoulpi_core::paths::git_dir(&repo.root)?);
    if !dir.join("refs").is_dir() {
        return None;
    }
    let config = std::fs::read_to_string(dir.join("config")).unwrap_or_default();
    if config.to_ascii_lowercase().contains("reftable") {
        return None;
    }
    Some(dir)
}

/// Creates the pin refs of `updates` as loose ref files, when `updates`
/// is nothing but `Create`s under the pin namespace. All or nothing: any
/// trouble (the ref exists, no permission) removes what was written and
/// reports `false`, and the caller takes the ordinary `update-ref` path,
/// which words the failure.
fn write_pins_directly(repo: &RepoRef, updates: &[RefUpdate]) -> bool {
    use gitpoulpi_core::domain::UNDO_PIN_PREFIX;
    use std::io::Write;
    let creates: Vec<(&String, &Oid)> = updates
        .iter()
        .filter_map(|u| match u {
            RefUpdate::Create { name, new } if name.starts_with(UNDO_PIN_PREFIX) => {
                Some((name, new))
            }
            _ => None,
        })
        .collect();
    if creates.is_empty() || creates.len() != updates.len() {
        return false;
    }
    if creates
        .iter()
        .any(|(name, _)| name.contains("..") || name.contains('\\') || name.ends_with('/'))
    {
        return false;
    }
    let Some(root) = loose_refs_root(repo) else {
        return false;
    };
    // `git update-ref` refuses an id the repository does not have; a file
    // written here would not. Only a loose object is checked without a
    // process: anything else (packed, in an alternate, missing) takes the
    // `update-ref` path, which verifies it.
    let objects = root.join("objects");
    if creates.iter().any(|(_, oid)| {
        let hex = oid.to_string();
        !objects.join(&hex[..2]).join(&hex[2..]).is_file()
    }) {
        return false;
    }
    let mut written: Vec<std::path::PathBuf> = Vec::new();
    for (name, oid) in &creates {
        let path = root.join(name);
        let ok = (|| {
            std::fs::create_dir_all(path.parent()?).ok()?;
            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&path)
                .ok()?;
            written.push(path.clone());
            file.write_all(format!("{oid}\n").as_bytes()).ok()
        })();
        if ok.is_none() {
            for path in &written {
                let _ = std::fs::remove_file(path);
            }
            return false;
        }
    }
    true
}

/// Releases pin refs as loose ref files, when `updates` is nothing but
/// `Delete`s under the pin namespace whose loose files hold the expected
/// oid. A pin that is not a loose file (packed by `git pack-refs`, or
/// already gone) sends the whole batch down the ordinary `update-ref`
/// path, which also words the failure; nothing is removed in that case.
fn delete_pins_directly(repo: &RepoRef, updates: &[RefUpdate]) -> bool {
    use gitpoulpi_core::domain::UNDO_PIN_PREFIX;
    let deletes: Vec<(&String, &Oid)> = updates
        .iter()
        .filter_map(|u| match u {
            RefUpdate::Delete { name, old } if name.starts_with(UNDO_PIN_PREFIX) => {
                Some((name, old))
            }
            _ => None,
        })
        .collect();
    if deletes.is_empty() || deletes.len() != updates.len() {
        return false;
    }
    if deletes
        .iter()
        .any(|(name, _)| name.contains("..") || name.contains('\\'))
    {
        return false;
    }
    let Some(root) = loose_refs_root(repo) else {
        return false;
    };
    let paths: Vec<std::path::PathBuf> = deletes.iter().map(|(n, _)| root.join(n)).collect();
    for ((_, oid), path) in deletes.iter().zip(&paths) {
        match std::fs::read_to_string(path) {
            Ok(text) if text.trim() == oid.to_hex() => {}
            _ => return false,
        }
    }
    for path in &paths {
        if std::fs::remove_file(path).is_err() {
            return false;
        }
        // The entry's directory goes with its last pin (best effort).
        if let Some(dir) = path.parent() {
            let _ = std::fs::remove_dir(dir);
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_dotted_branch_name_does_not_claim_its_prefix_branch() {
        assert!(in_section("branch.a", "branch.a.remote"));
        assert!(!in_section("branch.a", "branch.a.b.remote"));
        assert!(in_section("branch.a.b", "branch.a.b.remote"));
        assert!(!in_section("branch.a", "branch.ab.remote"));
    }

    #[test]
    fn only_branch_sections_are_accepted() {
        assert!(check_section("branch.main").is_ok());
        assert!(check_section("branch.").is_err());
        assert!(check_section("core").is_err());
        assert!(check_section("remote.origin").is_ok());
        assert!(check_section("remote.").is_err());
    }
}
