//! `credentials.https_mode`, applied before every network operation (and
//! every other git command that can reach a remote): the setting is read
//! from `config.toml` and the environment each time, so a change in
//! Settings applies to the next fetch without restarting the daemon.

use gitpoulpi_cli_backend::{set_credential_mode, CredentialMode};
use gitpoulpi_config::HttpsMode;

/// Maps the setting onto the system-git backend's process-wide mode.
pub(crate) fn mode_for(mode: HttpsMode) -> CredentialMode {
    match mode {
        HttpsMode::CredentialHelper => CredentialMode::Helper,
        HttpsMode::Prompt => CredentialMode::Prompt,
        HttpsMode::None => CredentialMode::None,
    }
}

/// Reads `credentials.https_mode` now and makes it the mode of every git
/// command spawned from here on. An unreadable config keeps git's own
/// handling.
pub(crate) fn apply_from_config() {
    let settings = gitpoulpi_config::Settings::resolve_ambient().ok();
    let mode = settings
        .as_ref()
        .map(|s| s.config().credentials.https_mode)
        .unwrap_or_default();
    set_credential_mode(mode_for(mode));
    // The SSH key rides along: same moments, same live re-read.
    gitpoulpi_cli_backend::set_ssh_key(settings.and_then(|s| ssh_key_of(&s.config().ssh)));
}

/// [`apply_from_config`], then the repository's own authentication profile
/// (ADR-0010): read live from `repo-auth.toml`, resolved into a
/// `NetworkAuth` and handed to the backend, so a change applies to the next
/// transfer. A repository with no profile is exactly the global behaviour.
///
/// `Err` when the profile cannot be used (unreadable file, a missing account
/// or key, a source this version does not have): the caller **refuses the
/// transfer**. It never falls back to the machine's own credentials.
pub(crate) fn apply_for(
    backend: &dyn gitpoulpi_core::GitBackend,
    repo: &gitpoulpi_core::domain::RepoRef,
) -> gitpoulpi_core::Result<()> {
    apply_from_config();
    // Whatever was set for the previous transfer is gone, also when this one
    // is refused below.
    backend.set_network_auth(repo, None);
    let Some(profile) = crate::repo_auth::stored_profile(&repo.root)? else {
        return Ok(());
    };
    let auth = crate::repo_auth::resolve(&repo.root, &profile)?;
    backend.set_network_auth(repo, Some(auth));
    Ok(())
}

/// The key file `ssh.private_key_path` (else `ssh.public_key_path`, which ssh
/// reads as "the agent's key matching this public key") names, if any.
pub(crate) fn ssh_key_of(ssh: &gitpoulpi_config::SshConfig) -> Option<String> {
    [&ssh.private_key_path, &ssh.public_key_path]
        .into_iter()
        .flatten()
        .map(|p| p.trim())
        .find(|p| !p.is_empty())
        .map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_private_key_beats_the_public_one_and_blanks_are_ignored() {
        let ssh = |private: Option<&str>, public: Option<&str>| gitpoulpi_config::SshConfig {
            private_key_path: private.map(str::to_string),
            public_key_path: public.map(str::to_string),
            ..Default::default()
        };
        assert_eq!(ssh_key_of(&ssh(None, None)), None);
        assert_eq!(ssh_key_of(&ssh(Some(" "), Some(""))), None);
        assert_eq!(
            ssh_key_of(&ssh(Some("/k/id"), Some("/k/id.pub"))).as_deref(),
            Some("/k/id")
        );
        assert_eq!(
            ssh_key_of(&ssh(Some(""), Some(" /k/id.pub "))).as_deref(),
            Some("/k/id.pub")
        );
    }

    #[test]
    fn every_setting_has_its_own_mode() {
        assert_eq!(
            mode_for(HttpsMode::CredentialHelper),
            CredentialMode::Helper
        );
        assert_eq!(mode_for(HttpsMode::Prompt), CredentialMode::Prompt);
        assert_eq!(mode_for(HttpsMode::None), CredentialMode::None);
    }
}
