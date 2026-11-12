use std::collections::BTreeMap;

use gitpoulpi_core::domain::{
    leftover_remote_refs, parse_remote_tag_mirror_ref, remote_tag_mirror_ref, Remote, RepoRef,
    REMOTE_TAG_MIRROR_PREFIX,
};
use gitpoulpi_core::{Error, Result};

use crate::process::{run, run_with_stdin};

/// See `GitBackend::leftover_remotes`.
pub fn leftover_remotes(repo: &RepoRef) -> Result<Vec<String>> {
    let configured: Vec<String> = remotes(repo)?.into_iter().map(|r| r.name).collect();
    let listed = run(
        &repo.root,
        &[
            "for-each-ref",
            "--format=%(refname)",
            "refs/remotes/",
            REMOTE_TAG_MIRROR_PREFIX,
        ],
    )?;
    let refnames: Vec<String> = listed.lines().map(str::to_string).collect();
    Ok(leftover_remote_refs(&refnames, &configured)
        .into_keys()
        .collect())
}

/// Every remote git itself knows by name (`git remote`): any
/// `remote.<name>.*` section, with or without a fetch address. A superset
/// of [`remotes`], which lists only the ones that have one.
fn declared_names(repo: &RepoRef) -> Result<Vec<String>> {
    let raw = run(&repo.root, &["remote"])?;
    Ok(raw
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(str::to_string)
        .collect())
}

fn require_remote(repo: &RepoRef, name: &str) -> Result<Vec<String>> {
    let names = declared_names(repo)?;
    if names.iter().any(|n| n == name) {
        Ok(names)
    } else {
        Err(Error::Backend(format!("no such remote: {name}")))
    }
}

/// `git remote rename`, then the part git knows nothing about: this
/// remote's tag mirror (`refs/gitpoulpi/remote-tags/<old>/...`) moves to
/// the new name in one `update-ref` transaction.
///
/// The mirror refs are listed *before* the rename and split against the
/// pre-rename remote list, so a sibling remote whose name merely starts
/// with the old one (`origin` vs `origin/sub`) keeps its own tags.
pub fn remote_rename(repo: &RepoRef, old_name: &str, new_name: &str) -> Result<()> {
    gitpoulpi_core::guard::check_remote_arg("remote name", new_name)?;
    let names = require_remote(repo, old_name)?;
    let mirror = mirror_refs_of(repo, old_name, &names)?;

    run(&repo.root, &["remote", "rename", "--", old_name, new_name])?;

    let mut script = String::new();
    for (old_ref, tag, oid) in &mirror {
        let new_ref = remote_tag_mirror_ref(new_name, tag);
        script.push_str(&format!("create {new_ref} {oid}\ndelete {old_ref} {oid}\n"));
    }
    apply_ref_script(repo, &script)
}

/// `git remote remove`, then this remote's tag mirror. Left behind, the
/// mirror would keep reporting its tags as pushed - to a remote that no
/// longer exists - and a new remote added under the same name would
/// inherit a history it never had.
///
/// A *leftover* name (refs, no config: see `leftover_remotes`) has no config
/// to remove, so only its remote-tracking branches and mirror are deleted -
/// the same refs a real removal ends with.
pub fn remote_remove(repo: &RepoRef, name: &str) -> Result<()> {
    let names = declared_names(repo)?;
    if !names.iter().any(|n| n == name) {
        let listed: Vec<String> = remotes(repo)?.into_iter().map(|r| r.name).collect();
        return sweep_leftover(repo, name, &listed)?
            .then_some(())
            .ok_or_else(|| Error::Backend(format!("no such remote: {name}")));
    }
    let mirror = mirror_refs_of(repo, name, &names)?;

    run(&repo.root, &["remote", "remove", name])?;

    let mut script = String::new();
    for (refname, _, oid) in &mirror {
        script.push_str(&format!("delete {refname} {oid}\n"));
    }
    apply_ref_script(repo, &script)?;
    // The removal only deletes what the remote's fetch refspec maps: a
    // remote declared without one (a lone `pushurl`) leaves its branches.
    let listed: Vec<String> = remotes(repo)?.into_iter().map(|r| r.name).collect();
    sweep_leftover(repo, name, &listed)?;
    Ok(())
}

/// Deletes the refs of the leftover namespace `name` (`configured` being
/// the remotes that own theirs). `false`: it owned none.
fn sweep_leftover(repo: &RepoRef, name: &str, configured: &[String]) -> Result<bool> {
    let listed = run(
        &repo.root,
        &[
            "for-each-ref",
            "--format=%(refname) %(objectname)",
            "refs/remotes/",
            REMOTE_TAG_MIRROR_PREFIX,
        ],
    )?;
    let refnames: Vec<String> = listed
        .lines()
        .filter_map(|l| l.split_once(' ').map(|(r, _)| r.to_string()))
        .collect();
    let Some(owned) = leftover_remote_refs(&refnames, configured).remove(name) else {
        return Ok(false);
    };
    let mut script = String::new();
    for line in listed.lines() {
        if let Some((refname, oid)) = line.split_once(' ') {
            if owned.iter().any(|r| r == refname) {
                // `refs/remotes/<r>/HEAD` is a symbolic ref: delete the alias,
                // not the branch it names.
                script.push_str(&format!("option no-deref\ndelete {refname} {oid}\n"));
            }
        }
    }
    apply_ref_script(repo, &script)?;
    Ok(true)
}

/// This remote's mirror refs as `(refname, tag, oid)`, split against the
/// repository's actual remote list (`names`) so a sibling remote whose
/// name merely starts with this one (`origin` vs `origin/sub`) is not
/// swept up with it.
fn mirror_refs_of(
    repo: &RepoRef,
    remote: &str,
    names: &[String],
) -> Result<Vec<(String, String, String)>> {
    let listed = run(
        &repo.root,
        &[
            "for-each-ref",
            "--format=%(refname) %(objectname)",
            REMOTE_TAG_MIRROR_PREFIX,
        ],
    )?;
    Ok(listed
        .lines()
        .filter_map(|line| {
            let (refname, oid) = line.split_once(' ')?;
            let (owner, tag) = parse_remote_tag_mirror_ref(refname, names)?;
            (owner == remote).then(|| (refname.to_string(), tag.to_string(), oid.to_string()))
        })
        .collect())
}

/// One `update-ref --stdin` transaction, or nothing for an empty script.
fn apply_ref_script(repo: &RepoRef, body: &str) -> Result<()> {
    if body.is_empty() {
        return Ok(());
    }
    let script = format!("start\n{body}commit\n");
    run_with_stdin(&repo.root, &["update-ref", "--stdin"], &script)?;
    Ok(())
}

/// `git remote set-url`, which already refuses an unknown remote.
pub fn remote_set_url(repo: &RepoRef, name: &str, url: &str) -> Result<()> {
    gitpoulpi_core::guard::check_remote_arg("remote address", url)?;
    require_remote(repo, name)?;
    run(&repo.root, &["remote", "set-url", "--", name, url])?;
    Ok(())
}

/// Sets or clears `remote.<name>.pushurl`. Clearing goes through
/// `git config --unset-all`, which *fails* when there is nothing to
/// unset, so it only runs when there is.
pub fn remote_set_push_url(repo: &RepoRef, name: &str, url: Option<&str>) -> Result<()> {
    if let Some(url) = url {
        gitpoulpi_core::guard::check_remote_arg("remote address", url)?;
    }
    require_remote(repo, name)?;
    let key = format!("remote.{name}.pushurl");
    match url {
        Some(url) => {
            run(&repo.root, &["config", "--replace-all", &key, url])?;
        }
        None => {
            let current = run(&repo.root, &["config", "--get-all", &key]).unwrap_or_default();
            if !current.trim().is_empty() {
                run(&repo.root, &["config", "--unset-all", &key])?;
            }
        }
    }
    Ok(())
}

pub fn remotes(repo: &RepoRef) -> Result<Vec<Remote>> {
    let raw = run(&repo.root, &["remote", "-v"])?;

    let mut fetch_urls: BTreeMap<String, String> = BTreeMap::new();
    let mut push_urls: BTreeMap<String, String> = BTreeMap::new();

    for line in raw.lines() {
        let mut parts = line.splitn(2, '\t');
        let name = match parts.next() {
            Some(n) if !n.is_empty() => n.to_string(),
            _ => continue,
        };
        let Some(rest) = parts.next() else { continue };
        let Some((url, kind)) = rest.rsplit_once(' ') else {
            continue;
        };
        match kind {
            "(fetch)" => {
                fetch_urls.insert(name, url.to_string());
            }
            "(push)" => {
                push_urls.insert(name, url.to_string());
            }
            _ => {}
        }
    }

    Ok(fetch_urls
        .into_iter()
        .map(|(name, fetch_url)| {
            let push_url = push_urls.get(&name).cloned();
            Remote {
                name,
                fetch_url,
                push_url,
            }
        })
        .collect())
}

/// The repository-local value of an allow-listed key, or `None`.
pub fn repo_config(repo: &RepoRef, key: &str) -> Result<Option<String>> {
    gitpoulpi_core::repo_config::check_repo_config_key(key)?;
    let out = run(&repo.root, &["config", "--local", "--get-all", key]).unwrap_or_default();
    let value = out.lines().last().unwrap_or("").trim().to_string();
    Ok((!value.is_empty()).then_some(value))
}

/// See `GitBackend::config_entries`.
pub fn config_entries(repo: &RepoRef, prefixes: &[&str]) -> Result<Vec<(String, String)>> {
    let raw = run(&repo.root, &["config", "--local", "--list", "-z"])?;
    let prefixes: Vec<String> = prefixes.iter().map(|p| p.to_ascii_lowercase()).collect();
    Ok(raw
        .split('\0')
        // `key\nvalue`; a key with no value has no newline.
        .filter_map(|entry| entry.split_once('\n'))
        .filter(|(key, _)| {
            let lower = key.to_ascii_lowercase();
            prefixes.iter().any(|p| lower.starts_with(p.as_str()))
        })
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect())
}

/// Sets or clears an allow-listed repository-local key.
pub fn set_repo_config(repo: &RepoRef, key: &str, value: Option<&str>) -> Result<()> {
    match value {
        Some(v) => {
            gitpoulpi_core::repo_config::check_repo_config_value(key, v)?;
            run(&repo.root, &["config", "--local", "--replace-all", key, v])?;
        }
        None => {
            gitpoulpi_core::repo_config::check_repo_config_key(key)?;
            if repo_config(repo, key)?.is_some() {
                run(&repo.root, &["config", "--local", "--unset-all", key])?;
            }
        }
    }
    Ok(())
}

/// The repository-local `core.sshCommand`, or `None`.
pub fn ssh_command(repo: &RepoRef) -> Result<Option<String>> {
    let out = run(
        &repo.root,
        &["config", "--local", "--get-all", "core.sshCommand"],
    )
    .unwrap_or_default();
    let value = out.lines().last().unwrap_or("").trim().to_string();
    Ok((!value.is_empty()).then_some(value))
}

/// Sets or clears the repository-local `core.sshCommand`. `--unset-all`
/// fails when there is nothing to unset, so it only runs when there is.
pub fn set_ssh_command(repo: &RepoRef, command: Option<&str>) -> Result<()> {
    match command {
        Some(cmd) => {
            run(
                &repo.root,
                &["config", "--local", "--replace-all", "core.sshCommand", cmd],
            )?;
        }
        None => {
            if ssh_command(repo)?.is_some() {
                run(
                    &repo.root,
                    &["config", "--local", "--unset-all", "core.sshCommand"],
                )?;
            }
        }
    }
    Ok(())
}
