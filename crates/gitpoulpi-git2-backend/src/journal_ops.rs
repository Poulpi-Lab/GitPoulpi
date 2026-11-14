//! The undo journal's backend primitives over libgit2. See
//! the undo-redo design §2.7.

use std::collections::BTreeMap;

use gitpoulpi_core::domain::{HeadState, RefScope, RefSnapshot, RefUpdate, RepoRef, UndoRefusal};
use gitpoulpi_core::{Error, Result};

use crate::repo::{map_err, open, to_domain_oid, to_git2_oid};

fn head_state(repo: &git2::Repository) -> Result<HeadState> {
    let head = repo.find_reference("HEAD").map_err(map_err)?;
    if let Some(target) = head.symbolic_target().ok().flatten() {
        return Ok(HeadState::Branch(
            target
                .strip_prefix("refs/heads/")
                .unwrap_or(target)
                .to_string(),
        ));
    }
    match head.target() {
        Some(oid) => Ok(HeadState::Detached(to_domain_oid(oid)?)),
        None => Err(Error::Backend("HEAD points at nothing".into())),
    }
}

pub fn ref_snapshot(repo: &RepoRef, scope: &RefScope) -> Result<RefSnapshot> {
    let git_repo = open(repo)?;
    let mut refs = BTreeMap::new();
    let mut symrefs = BTreeMap::new();
    for reference in git_repo.references().map_err(map_err)? {
        let reference = reference.map_err(map_err)?;
        if let (Some(name), Some(target)) = (
            reference.name().ok(),
            reference.symbolic_target().ok().flatten(),
        ) {
            if scope.prefixes.iter().any(|p| name.starts_with(p.as_str())) {
                symrefs.insert(name.to_string(), target.to_string());
            }
        }
        let (Some(name), Some(target)) = (reference.name().ok(), reference.target()) else {
            continue;
        };
        if scope.prefixes.iter().any(|p| name.starts_with(p.as_str())) {
            refs.insert(name.to_string(), to_domain_oid(target)?);
        }
    }
    Ok(RefSnapshot {
        head: head_state(&git_repo)?,
        refs,
        symrefs,
    })
}

fn current(repo: &git2::Repository, name: &str) -> Option<gitpoulpi_core::domain::Oid> {
    repo.find_reference(name)
        .ok()
        .and_then(|r| r.target())
        .and_then(|oid| to_domain_oid(oid).ok())
}

/// Locks every ref, checks every expectation while the locks are held,
/// then applies and commits - libgit2's transaction is the atomic part,
/// the lock is what makes the check-then-write race free.
pub fn update_refs(repo: &RepoRef, updates: &[RefUpdate], reflog_message: &str) -> Result<()> {
    let git_repo = open(repo)?;

    for update in updates {
        if let RefUpdate::SetHead { old, .. } = update {
            let actual = head_state(&git_repo)?;
            if &actual != old {
                return Err(Error::UndoRefused(UndoRefusal::HeadMoved {
                    expected: old.clone(),
                    actual,
                }));
            }
        }
    }

    // libgit2's transaction writes whatever id it is given; `git update-ref`
    // refuses an object the repository does not have, and so must we.
    let odb = git_repo.odb().map_err(map_err)?;
    for update in updates {
        if let RefUpdate::Create { name, new } | RefUpdate::Update { name, new, .. } = update {
            if !odb.exists(to_git2_oid(*new)?) {
                return Err(Error::Backend(format!(
                    "cannot point {name} at {new}: the object is not in the repository"
                )));
            }
        }
    }

    let sig = git_repo
        .signature()
        .or_else(|_| git2::Signature::now("GitPoulpi", "gitpoulpi@localhost"))
        .map_err(map_err)?;
    let mut tx = git_repo.transaction().map_err(map_err)?;
    let mut locked: Vec<&str> = Vec::new();
    for update in updates {
        let name = match update {
            RefUpdate::Create { name, .. }
            | RefUpdate::Update { name, .. }
            | RefUpdate::Delete { name, .. }
            | RefUpdate::Verify { name, .. } => name.as_str(),
            RefUpdate::SetHead { .. } | RefUpdate::SetSymbolic { .. } => continue,
        };
        if !locked.contains(&name) {
            tx.lock_ref(name).map_err(map_err)?;
            locked.push(name);
        }
    }
    // Every expectation, with the locks held.
    for update in updates {
        let (name, expected) = match update {
            RefUpdate::Create { name, .. } => (name, None),
            RefUpdate::Update { name, old, .. }
            | RefUpdate::Delete { name, old }
            | RefUpdate::Verify { name, old } => (name, Some(*old)),
            RefUpdate::SetHead { .. } | RefUpdate::SetSymbolic { .. } => continue,
        };
        let actual = current(&git_repo, name);
        if actual != expected {
            return Err(Error::UndoRefused(UndoRefusal::RefMoved {
                name: name.clone(),
                expected,
                actual,
                restore: None,
            }));
        }
    }
    for update in updates {
        match update {
            RefUpdate::Create { name, new } | RefUpdate::Update { name, new, .. } => {
                tx.set_target(name, to_git2_oid(*new)?, Some(&sig), reflog_message)
                    .map_err(map_err)?;
            }
            RefUpdate::Delete { name, .. } => tx.remove(name).map_err(map_err)?,
            RefUpdate::Verify { .. }
            | RefUpdate::SetHead { .. }
            | RefUpdate::SetSymbolic { .. } => {}
        }
    }
    tx.commit().map_err(map_err)?;

    for update in updates {
        if let RefUpdate::SetSymbolic { name, new, .. } = update {
            match new {
                Some(target) => {
                    git_repo
                        .reference_symbolic(name, target, true, reflog_message)
                        .map_err(map_err)?;
                }
                None => {
                    if let Ok(mut reference) = git_repo.find_reference(name) {
                        reference.delete().map_err(map_err)?;
                    }
                }
            }
        }
    }
    for update in updates {
        if let RefUpdate::SetHead { new, .. } = update {
            match new {
                HeadState::Branch(name) => git_repo
                    .set_head(&format!("refs/heads/{name}"))
                    .map_err(map_err)?,
                HeadState::Detached(oid) => git_repo
                    .set_head_detached(to_git2_oid(*oid)?)
                    .map_err(map_err)?,
            }
        }
    }
    Ok(())
}

fn local_config(git_repo: &git2::Repository) -> Result<git2::Config> {
    git_repo
        .config()
        .map_err(map_err)?
        .open_level(git2::ConfigLevel::Local)
        .map_err(map_err)
}

fn read_section(config: &git2::Config, section: &str) -> Result<Vec<(String, String)>> {
    let mut out = Vec::new();
    let mut entries = config.entries(None).map_err(map_err)?;
    while let Some(entry) = entries.next() {
        let entry = entry.map_err(map_err)?;
        let Some(key) = entry.name().ok() else {
            continue;
        };
        if gitpoulpi_cli_backend::config_key_in_section(section, key) {
            out.push((key.to_string(), entry.value().unwrap_or("").to_string()));
        }
    }
    Ok(out)
}

/// The `branch.<name>` sections whose remote is `remote`, sorted.
pub fn remote_dependents(repo: &RepoRef, remote: &str) -> Result<Vec<String>> {
    let config = local_config(&open(repo)?)?;
    let mut all = Vec::new();
    let mut entries = config.entries(None).map_err(map_err)?;
    while let Some(entry) = entries.next() {
        let entry = entry.map_err(map_err)?;
        if let Ok(key) = entry.name() {
            all.push((key.to_string(), entry.value().unwrap_or("").to_string()));
        }
    }
    Ok(gitpoulpi_cli_backend::remote_dependents_in(&all, remote))
}

pub fn config_section(repo: &RepoRef, section: &str) -> Result<Vec<(String, String)>> {
    gitpoulpi_cli_backend::check_config_section(section)?;
    read_section(&local_config(&open(repo)?)?, section)
}

pub fn set_config_section(
    repo: &RepoRef,
    section: &str,
    entries: &[(String, String)],
) -> Result<()> {
    gitpoulpi_cli_backend::check_config_section(section)?;
    if let Some((key, _)) = entries
        .iter()
        .find(|(key, _)| !gitpoulpi_cli_backend::config_key_in_section(section, key))
    {
        return Err(Error::Backend(format!(
            "`{key}` is not a variable of `{section}`"
        )));
    }
    let git_repo = open(repo)?;
    let mut config = local_config(&git_repo)?;
    if gitpoulpi_cli_backend::is_config_key_section(section) {
        for (key, value) in entries {
            gitpoulpi_core::repo_config::check_repo_config_value(key, value)?;
        }
    }
    for (key, _) in read_section(&config, section)? {
        // A multivar key is not unique, so a plain `remove` refuses it:
        // drop every value by pattern. A second removal of the same key
        // (listed once per value) is then a harmless "not found".
        let _ = config.remove_multivar(&key, ".*");
        let _ = config.remove(&key);
    }
    let mut seen: Vec<&str> = Vec::new();
    for (key, value) in entries {
        if seen.contains(&key.as_str()) {
            // A repeated key is a multivar: append, never replace.
            config.set_multivar(key, "a^", value).map_err(map_err)?;
        } else {
            config.set_str(key, value).map_err(map_err)?;
            seen.push(key);
        }
    }
    Ok(())
}

/// The tree of the index as it is on disk right now, written to the
/// object database; no process.
pub fn index_tree(repo: &RepoRef) -> Result<gitpoulpi_core::domain::Oid> {
    let git_repo = open(repo)?;
    let mut index = git_repo.index().map_err(map_err)?;
    if index.has_conflicts() {
        return Err(Error::Backend(
            "git2: the index has unresolved conflicts, it has no tree".into(),
        ));
    }
    let oid = index.write_tree().map_err(map_err)?;
    to_domain_oid(oid)
}

/// Makes the index equal `tree` and writes it; the working tree is not
/// touched.
pub fn index_restore(repo: &RepoRef, tree: gitpoulpi_core::domain::Oid) -> Result<()> {
    let git_repo = open(repo)?;
    let tree = git_repo.find_tree(to_git2_oid(tree)?).map_err(map_err)?;
    let mut index = git_repo.index().map_err(map_err)?;
    index.read_tree(&tree).map_err(map_err)?;
    index.write().map_err(map_err)
}
