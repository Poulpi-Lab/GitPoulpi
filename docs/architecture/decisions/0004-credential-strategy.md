# 0004 - Credential strategy: system ssh-agent + git credential-helper

## Status
Accepted

## Context
SSH and HTTPS credentials need to work across GitHub and one or more self-managed GitLab instances, without asking the user to re-enter or re-manage secrets they've already configured for daily `git` use, while still covering instances a credential helper doesn't know about.

## Decision
- **SSH**: rely entirely on the platform's standard ssh-agent / `~/.ssh` discovery (Windows OpenSSH agent service, `ssh-agent`/`gpg-agent` on Linux/macOS). No GitPoulpi-specific SSH key store (a per-profile/per-repo key *path* pin was added later, ADR 0008).
- **HTTPS**: default to shelling out to the user's existing `git credential.helper` (`credentials.https_mode = "credential_helper"` in config) - this already works today via Git Credential Manager (Windows), libsecret/pass-based helpers (Linux), or Keychain (macOS), and needs zero new secret-management UI.
- **Update:** the planned encrypted local secrets file was dropped (maintainer decision, quality sweep). Tokens use the OS credential helper; with no helper they stay in memory for the session.
- Explicitly rejected: an OS-keyring-based approach (`keyring` crate) as the primary mechanism - the user preferred reusing what's already configured over introducing a new credential store.

## Consequences
- Zero new UI needed for the common case (GitHub + one GitLab instance already covered by the user's credential helper).
- `gitpoulpi-secrets` stays small: the credential-helper adapter plus an in-memory session store.
- Tokens must never be logged or written to `config.toml`.
