//! Per-repository authentication (ADR-0010): the daemon reads the repository's
//! profile from `<private>/repo-auth.toml` before each transfer and turns it
//! into the [`NetworkAuth`] its git processes get, and serves the
//! `RepoAuthGet` / `Set` / `Clear` / `Test` operations.
//!
//! The rule that runs through all of it: a profile that cannot be used
//! **refuses the transfer** (`RepoAuthUnavailable`). It never falls back to the
//! machine's own credentials, because pushing to work with a personal account
//! is the failure this feature exists to prevent.
//!
//! The System and account (integration) HTTPS sources and the key-file and
//! integration-key SSH sources work. The helper and token-command HTTPS
//! sources are parsed and stored but refuse (they need the trust and hash
//! pinning of a later slice); `RepoAuthSet` does not accept them.

use std::path::Path;

use gitpoulpi_config::ConfigPaths;
use gitpoulpi_core::domain::{
    strip_userinfo, transport_of, AuthOverride, HttpsAuth, NetworkAuth, RemoteAccessOptions,
    RemoteAccessReport, RemoteIdentity, RemoteTransport, RepoAuthProfile, RepoAuthSetArgs,
    RepoAuthTestArgs, RepoAuthView, RepoRef, SshAuth,
};
use gitpoulpi_core::repo_config::sh_quote_path;
use gitpoulpi_core::{Error, GitBackend};
use gitpoulpi_ipc::Request;
use serde_json::Value;

use crate::dispatch::{err, failed, ok, Dispatched};

/// The key a status payload carries the summary under.
pub const STATUS_KEY: &str = "repo_auth";

fn unavailable(reason: impl Into<String>) -> Error {
    Error::RepoAuthUnavailable {
        reason: reason.into(),
    }
}

/// The stored profile of the repository at `root`, `None` when it has none.
/// `Err` (a refusal) when the file exists but cannot be read.
pub(crate) fn stored_profile(root: &Path) -> Result<Option<RepoAuthProfile>, Error> {
    let Some(paths) = ConfigPaths::discover() else {
        return Ok(None);
    };
    gitpoulpi_session::load_repo_auth(&paths, root)
        .map(|r| r.map(|r| r.profile()))
        .map_err(unavailable)
}

/// What resolving a profile gives a transfer: the `NetworkAuth` for the
/// repository's processes, and the grant (if the source has one) that must
/// stay alive until the transfer is over.
pub(crate) struct Resolved {
    pub auth: NetworkAuth,
    pub grant: Option<crate::grants::GrantGuard>,
}

/// What `profile` gives the repository's processes for a transfer of the
/// daemon's session `session`, or why it cannot be used. Reads the
/// configuration (read-only) for the cases that need it and, for an account
/// source, the account's token (which the grant then serves to the helper).
pub(crate) fn resolve(
    root: &Path,
    profile: &RepoAuthProfile,
    session: &str,
) -> Result<Resolved, Error> {
    resolve_inner(root, profile, Some(session))
}

/// Whether `profile` can be used, without reading a token or issuing a grant
/// (what a read of the profile reports).
pub(crate) fn check(root: &Path, profile: &RepoAuthProfile) -> Result<(), Error> {
    resolve_inner(root, profile, None).map(|_| ())
}

fn resolve_inner(
    root: &Path,
    profile: &RepoAuthProfile,
    session: Option<&str>,
) -> Result<Resolved, Error> {
    let mut auth = NetworkAuth::default();
    let mut grant = None;
    match &profile.https {
        None => https_system(root, &mut auth, None, false)?,
        Some(HttpsAuth::System {
            username,
            ignore_repo_helpers,
        }) => https_system(root, &mut auth, username.as_deref(), *ignore_repo_helpers)?,
        Some(HttpsAuth::Integration {
            integration,
            username,
        }) => {
            grant = crate::account_source::apply(
                root,
                &mut auth,
                integration,
                username.as_deref(),
                session,
            )?;
        }
        Some(HttpsAuth::Helper { .. }) => {
            return Err(unavailable(
                "the credential helper source is not available yet",
            ))
        }
        Some(HttpsAuth::TokenCommand { .. }) => {
            return Err(unavailable("the token command source is not available yet"))
        }
    }
    auth.ssh_command = match &profile.ssh {
        // The machine's default, as without a profile: the global key, when
        // one is configured and nothing else ranks above it.
        None | Some(SshAuth::System) => gitpoulpi_cli_backend::global_ssh_key_command(root),
        Some(SshAuth::KeyFile {
            key,
            known_hosts,
            strict_host_key,
        }) => Some(key_command(
            root,
            key,
            known_hosts.as_deref(),
            *strict_host_key,
        )?),
        Some(SshAuth::IntegrationKey {
            integration,
            known_hosts,
            strict_host_key,
        }) => {
            let key = integration_key(integration)?;
            Some(key_command(
                root,
                &key,
                known_hosts.as_deref(),
                *strict_host_key,
            )?)
        }
    };
    Ok(Resolved { auth, grant })
}

/// The System HTTPS source. The process-wide `credentials.https_mode`
/// (already applied by `credentials::apply_from_config`) still decides
/// whether git may prompt and whether helpers run at all, because an entry
/// replaces that mode and must not silently lift it.
fn https_system(
    root: &Path,
    auth: &mut NetworkAuth,
    username: Option<&str>,
    ignore_repo_helpers: bool,
) -> Result<(), Error> {
    use gitpoulpi_cli_backend::CredentialMode;
    auth.credential_username = username
        .map(str::trim)
        .filter(|u| !u.is_empty())
        .map(str::to_string);
    match gitpoulpi_cli_backend::credential_mode() {
        CredentialMode::Helper if ignore_repo_helpers => {
            let entries = gitpoulpi_cli_backend::credential_helper_entries(root).map_err(|e| {
                unavailable(format!("the credential helpers could not be read: {e}"))
            })?;
            let mut plain = Vec::new();
            for (url, helper) in gitpoulpi_cli_backend::machine_wide_helpers(&entries) {
                match url {
                    None => plain.push(helper),
                    Some(url) => auth.credential_url_helpers.push((url, helper)),
                }
            }
            auth.credential_helpers = Some(plain);
        }
        CredentialMode::Helper => {}
        CredentialMode::Prompt => auth.credential_helpers = Some(Vec::new()),
        CredentialMode::None => {
            auth.credential_helpers = Some(Vec::new());
            auth.no_prompt = true;
        }
    }
    Ok(())
}

/// The key file of an integration's `IntegrationSsh`, read from the config
/// now (so a key changed in Settings applies to the next transfer).
fn integration_key(id: &str) -> Result<String, Error> {
    let settings = gitpoulpi_config::Settings::resolve_ambient()
        .map_err(|e| unavailable(format!("the settings could not be read: {e}")))?;
    let integration = settings
        .config()
        .integrations
        .iter()
        .find(|i| i.id == id)
        .ok_or_else(|| unavailable(format!("the account \"{id}\" no longer exists")))?;
    let ssh = integration
        .ssh
        .as_ref()
        .ok_or_else(|| unavailable(format!("the account \"{id}\" has no SSH key")))?;
    let key = [&ssh.private_key_path, &ssh.public_key_path]
        .into_iter()
        .flatten()
        .map(|p| p.trim())
        .find(|p| !p.is_empty())
        .map(str::to_string);
    key.ok_or_else(|| unavailable(format!("the account \"{id}\" has no SSH key")))
}

/// A word for the `sh` git runs `GIT_SSH_COMMAND` through: as is when it has
/// no character the shell would act on, else quoted.
fn shell_word(word: &str) -> String {
    let plain = !word.is_empty()
        && word
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "_-./:+@%".contains(c));
    if plain {
        word.to_string()
    } else {
        sh_quote_path(word)
    }
}

/// `<program> -i "<key>" -o IdentitiesOnly=yes [-o UserKnownHostsFile=...]
/// [-o StrictHostKeyChecking=yes]`. The program is the first word of the
/// effective global `core.sshCommand`, else `ssh`; the repository's own
/// `core.sshCommand` is what this replaces.
fn key_command(
    root: &Path,
    key: &str,
    known_hosts: Option<&str>,
    strict: bool,
) -> Result<String, Error> {
    let expanded = gitpoulpi_cli_backend::expand_home(key.trim());
    if !Path::new(&expanded).is_file() {
        return Err(unavailable(format!(
            "the SSH key file {} does not exist",
            key.trim()
        )));
    }
    let entries = gitpoulpi_cli_backend::ssh_command_entries(root)
        .map_err(|e| unavailable(format!("the ssh configuration could not be read: {e}")))?;
    let program = gitpoulpi_cli_backend::global_ssh_program(&entries);
    let mut command = format!(
        "{} -i {} -o IdentitiesOnly=yes",
        shell_word(&program),
        sh_quote_path(&expanded)
    );
    if let Some(hosts) = known_hosts.map(str::trim).filter(|h| !h.is_empty()) {
        // ssh_config words split on spaces unless quoted: the quotes are part
        // of the `-o` argument, the shell's own quoting wraps all of it.
        let value = format!(
            "UserKnownHostsFile=\"{}\"",
            gitpoulpi_cli_backend::expand_home(hosts)
        );
        command.push_str(&format!(" -o {}", sh_quote_path(&value)));
    }
    if strict {
        command.push_str(" -o StrictHostKeyChecking=yes");
    }
    Ok(command)
}

/// Whether the profile is a source this version can use (`RepoAuthSet`).
fn validate(profile: &RepoAuthProfile) -> Result<(), String> {
    let text = |what: &str, v: &str| -> Result<(), String> {
        if v.trim().is_empty() {
            return Err(format!("{what} is empty"));
        }
        if v.chars().any(char::is_control) {
            return Err(format!("{what} holds a control character"));
        }
        Ok(())
    };
    match &profile.https {
        Some(HttpsAuth::System { username, .. }) => {
            if let Some(u) = username.as_deref().filter(|u| !u.is_empty()) {
                text("the user name", u)?;
            }
        }
        Some(HttpsAuth::Integration {
            integration,
            username,
        }) => {
            text("the account", integration)?;
            if let Some(u) = username.as_deref().filter(|u| !u.is_empty()) {
                text("the user name", u)?;
            }
        }
        // Refused by the caller with its own typed error.
        Some(HttpsAuth::Helper { .. }) | Some(HttpsAuth::TokenCommand { .. }) | None => {}
    }
    match &profile.ssh {
        Some(SshAuth::KeyFile {
            key, known_hosts, ..
        }) => {
            text("the key file", key)?;
            if let Some(h) = known_hosts.as_deref().filter(|h| !h.is_empty()) {
                text("the known_hosts file", h)?;
            }
        }
        Some(SshAuth::IntegrationKey {
            integration,
            known_hosts,
            ..
        }) => {
            text("the account", integration)?;
            if let Some(h) = known_hosts.as_deref().filter(|h| !h.is_empty()) {
                text("the known_hosts file", h)?;
            }
        }
        Some(SshAuth::System) | None => {}
    }
    Ok(())
}

/// `RepoAuthGet`.
pub(crate) fn get(backend: &dyn GitBackend, repo: &RepoRef, request: &Request) -> Dispatched {
    let view = build_view(backend, repo);
    match serde_json::to_value(view) {
        Ok(json) => ok(request.id, json, false),
        Err(e) => err(request.id, format!("failed to encode result: {e}"), false),
    }
}

/// A command-running source's command is not part of a view: nothing in it
/// may leak, and showing it is a decision for the surfaces that can confirm.
fn blanked(mut profile: RepoAuthProfile) -> RepoAuthProfile {
    match &mut profile.https {
        Some(HttpsAuth::Helper { helper, .. }) => helper.clear(),
        Some(HttpsAuth::TokenCommand { argv, .. }) => argv.clear(),
        _ => {}
    }
    profile
}

pub(crate) fn build_view(backend: &dyn GitBackend, repo: &RepoRef) -> RepoAuthView {
    let mut view = RepoAuthView::default();
    let record = match ConfigPaths::discover() {
        None => None,
        Some(paths) => match gitpoulpi_session::load_repo_auth(&paths, &repo.root) {
            Ok(r) => r,
            Err(e) => {
                view.unavailable = Some(e);
                None
            }
        },
    };
    let profile = record.as_ref().map(|r| r.profile()).unwrap_or_default();
    view.configured = record.is_some();
    view.git_common_dir = record.as_ref().map(|r| r.git_common_dir.clone());
    if let Some(r) = &record {
        let now = gitpoulpi_config::git_common_dir(&repo.root)
            .map(|d| d.to_string_lossy().replace('\\', "/"));
        if now.as_deref().is_some_and(|n| n != r.git_common_dir) {
            view.warnings.push(
                "the repository moved since this profile was saved (its git directory differs)"
                    .to_string(),
            );
        }
    }
    if view.unavailable.is_none() && record.is_some() {
        if let Err(Error::RepoAuthUnavailable { reason }) = check(&repo.root, &profile) {
            view.unavailable = Some(reason);
        }
    }
    if let Some(reason) = &view.unavailable {
        view.warnings.push(format!(
            "transfers of this repository are refused: {reason}"
        ));
    }
    view.overrides = overrides(&repo.root, &profile);
    view.remotes = backend
        .remotes(repo)
        .unwrap_or_default()
        .into_iter()
        .map(|r| {
            let transport = transport_of(&r.fetch_url);
            RemoteIdentity {
                identity: identity_of(&profile, transport),
                url: strip_userinfo(&r.fetch_url),
                transport,
                remote: r.name,
            }
        })
        .collect();
    view.summary = profile.summary();
    view.profile = blanked(profile);
    view
}

fn identity_of(profile: &RepoAuthProfile, transport: RemoteTransport) -> String {
    match transport {
        RemoteTransport::Https | RemoteTransport::Http => match &profile.https {
            None => "system git".to_string(),
            Some(HttpsAuth::System {
                username,
                ignore_repo_helpers,
            }) => {
                let mut s = "system git".to_string();
                if let Some(u) = username.as_deref().filter(|u| !u.is_empty()) {
                    s.push_str(&format!(" (user {u})"));
                }
                if *ignore_repo_helpers {
                    s.push_str(", the repository's own helpers ignored");
                }
                s
            }
            Some(HttpsAuth::Integration { integration, .. }) => {
                format!("account {integration} (token)")
            }
            Some(HttpsAuth::Helper { .. }) => "credential helper".to_string(),
            Some(HttpsAuth::TokenCommand { .. }) => "token command".to_string(),
        },
        RemoteTransport::Ssh => match &profile.ssh {
            None | Some(SshAuth::System) => "system ssh (agent or default key)".to_string(),
            Some(SshAuth::KeyFile { key, .. }) => format!("ssh key {}", key_name(key)),
            Some(SshAuth::IntegrationKey { integration, .. }) => {
                format!("ssh key of account {integration}")
            }
        },
        RemoteTransport::Git | RemoteTransport::File | RemoteTransport::Other => {
            "no authentication".to_string()
        }
    }
}

fn key_name(key: &str) -> &str {
    key.rsplit(['/', '\\'])
        .find(|s| !s.is_empty())
        .unwrap_or(key)
}

/// The repository's own settings that compete with the profile, with where
/// they come from: its `core.sshCommand` and credential helpers.
fn overrides(root: &Path, profile: &RepoAuthProfile) -> Vec<AuthOverride> {
    let ssh_replaced = matches!(
        profile.ssh,
        Some(SshAuth::KeyFile { .. }) | Some(SshAuth::IntegrationKey { .. })
    );
    let helpers_replaced = match &profile.https {
        None => false,
        Some(HttpsAuth::System {
            ignore_repo_helpers,
            ..
        }) => *ignore_repo_helpers,
        Some(_) => true,
    };
    let mut out = Vec::new();
    let mut add = |entries: Result<Vec<gitpoulpi_cli_backend::ConfigEntry>, String>,
                   replaced: bool| {
        for e in entries.unwrap_or_default() {
            if e.is_machine_wide() || e.value.is_empty() {
                continue;
            }
            out.push(AuthOverride {
                key: e.key,
                value: e.value,
                scope: e.scope,
                origin: e.origin,
                overridden: replaced,
            });
        }
    };
    add(
        gitpoulpi_cli_backend::ssh_command_entries(root),
        ssh_replaced,
    );
    add(
        gitpoulpi_cli_backend::credential_helper_entries(root),
        helpers_replaced,
    );
    out
}

/// `RepoAuthSet`. Not journaled, not undoable: a choice of identity is a
/// setting, not a change to the repository.
pub(crate) fn set(repo: &RepoRef, request: &Request) -> Dispatched {
    let args: RepoAuthSetArgs = match serde_json::from_value(request.args.clone()) {
        Ok(a) => a,
        Err(e) => return err(request.id, format!("invalid arguments: {e}"), false),
    };
    if matches!(
        args.profile.https,
        Some(HttpsAuth::Helper { .. }) | Some(HttpsAuth::TokenCommand { .. })
    ) {
        // Whatever `confirm` says: running a command needs the trust and
        // hash pinning of a later slice.
        let error =
            unavailable("the credential helper and token command sources are not available yet");
        return failed(request, error.to_string(), error.typed(), None);
    }
    if let Err(message) = validate(&args.profile) {
        return err(request.id, format!("invalid profile: {message}"), false);
    }
    let Some(paths) = ConfigPaths::discover() else {
        return err(request.id, "no configuration directory".into(), false);
    };
    match gitpoulpi_session::save_repo_auth(
        &paths,
        &repo.root,
        &args.profile,
        gitpoulpi_core::time::unix_now(),
    ) {
        Ok(()) => ok(request.id, Value::Null, true),
        Err(message) => err(request.id, message, false),
    }
}

/// `RepoAuthClear`: back to the machine's defaults.
pub(crate) fn clear(repo: &RepoRef, request: &Request) -> Dispatched {
    let Some(paths) = ConfigPaths::discover() else {
        return ok(request.id, Value::Bool(false), false);
    };
    match gitpoulpi_session::clear_repo_auth(&paths, &repo.root) {
        // Nothing to clear is not a change: nothing is broadcast.
        Ok(had) => ok(request.id, Value::Bool(had), had),
        Err(message) => err(request.id, message, false),
    }
}

/// `RepoAuthTest`: `ls-remote` of one remote or all of them with the
/// profile applied (the caller has applied it), asking ssh to name the key
/// the server accepted.
pub(crate) async fn test(
    backend: &dyn GitBackend,
    repo: &RepoRef,
    args: &RepoAuthTestArgs,
) -> gitpoulpi_core::Result<Vec<RemoteAccessReport>> {
    let names: Vec<String> = match &args.remote {
        Some(name) => vec![name.clone()],
        None => backend.remotes(repo)?.into_iter().map(|r| r.name).collect(),
    };
    let opts = RemoteAccessOptions {
        push: args.push,
        ssh_verbose: true,
    };
    let mut reports = Vec::with_capacity(names.len());
    // Whatever an earlier operation of this repository left is not this test's.
    let _ = crate::grants::take_outcome(&repo.root);
    for name in names {
        let mut report = backend.remote_access_check(repo, &name, &opts).await?;
        // The transport reports `store` for a credential that worked: that
        // names the identity (the account), which no backend can see.
        report.accepted_identity = crate::grants::take_outcome(&repo.root).accepted;
        reports.push(report);
    }
    Ok(reports)
}

/// The short label for the status payload, computed from the file (no spawn).
/// `None` when everything is default or the file is not readable (the
/// transfer itself says so then).
pub(crate) fn summary_for(root: &Path) -> Option<String> {
    stored_profile(root).ok().flatten()?.summary()
}

/// Adds the summary to a serialized status payload.
pub(crate) fn add_summary(root: &Path, status: &mut Value) {
    if let (Some(label), Some(map)) = (summary_for(root), status.as_object_mut()) {
        map.insert(STATUS_KEY.to_string(), Value::String(label));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gitpoulpi_core::domain::SshAuth;

    #[test]
    fn a_word_with_a_space_is_quoted_and_a_plain_one_is_not() {
        assert_eq!(shell_word("ssh"), "ssh");
        assert_eq!(
            shell_word("C:/Windows/System32/OpenSSH/ssh.exe"),
            "C:/Windows/System32/OpenSSH/ssh.exe"
        );
        assert_eq!(
            shell_word("C:/Program Files/OpenSSH/ssh.exe"),
            "\"C:/Program Files/OpenSSH/ssh.exe\""
        );
    }

    #[test]
    fn a_key_file_that_does_not_exist_is_unavailable() {
        let dir = tempfile::tempdir().unwrap();
        let profile = RepoAuthProfile {
            https: None,
            ssh: Some(SshAuth::KeyFile {
                key: dir.path().join("nope").to_string_lossy().into_owned(),
                known_hosts: None,
                strict_host_key: false,
            }),
        };
        let error = match resolve(dir.path(), &profile, "s") {
            Err(e) => e,
            Ok(_) => panic!("a missing key file must not resolve"),
        };
        assert!(
            matches!(error, Error::RepoAuthUnavailable { ref reason } if reason.contains("does not exist")),
            "{error:?}"
        );
    }

    #[test]
    fn the_key_command_quotes_paths_and_adds_the_options() {
        let dir = tempfile::tempdir().unwrap();
        let key = dir.path().join("my key");
        std::fs::write(&key, "k").unwrap();
        let hosts = dir.path().join("known hosts");
        let command = key_command(
            dir.path(),
            &key.to_string_lossy(),
            Some(&hosts.to_string_lossy()),
            true,
        )
        .unwrap();
        let key_text = key.to_string_lossy().replace('\\', "/");
        let hosts_text = hosts.to_string_lossy().replace('\\', "/");
        assert!(command.contains(&format!("-i \"{key_text}\" -o IdentitiesOnly=yes")));
        assert!(
            command.contains(&format!("-o \"UserKnownHostsFile=\\\"{hosts_text}\\\"\"")),
            "{command}"
        );
        assert!(command.ends_with("-o StrictHostKeyChecking=yes"));
    }
}
