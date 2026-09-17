# 0008 - Per-profile SSH keys, generated in process

## Status
Accepted (2026-09-29).

## Context
Two accounts on one host (a work and a personal GitHub, or two self-managed GitLab instances) need two SSH identities, and ssh offers every agent key in turn, so the wrong account often answers first. Until now the Preferences SSH page shelled out to `ssh-keygen`, which is an optional Windows feature that a managed image often has off, and which prompts for a passphrase and overwrite confirmation (a hang in an alternate-screen TUI). `gitpoulpi-hosting::ssh_key` already generated Ed25519 pairs in process for the CLI.

## Decision
1. **One generator, in process.** `gitpoulpi_hosting::ssh_key::generate_and_write` (Ed25519, no passphrase, `0600` at creation on Unix, never overwrites) is the only key generator. The global Preferences > SSH "Generate key" action uses it (`~/.ssh/gitpoulpi_ed25519`) and no longer runs `ssh-keygen`.
2. **Per-profile key, stored as data.** `[[integrations]].ssh` (`IntegrationSsh`) holds `mode` (`default` | `generated` | `custom`), the key paths, the title and the provider-side `remote_key_id`. A generated key lives at `~/.ssh/gitpoulpi_<profile id>_ed25519{,.pub}`, so two profiles never share or overwrite a key.
3. **Upload is deduplicated and never fatal.** Before uploading, `list_ssh_keys` runs and a key the provider already holds (same type and body, comment ignored) is adopted rather than posted again. If the upload fails the key stays on disk and attached, with a note that it was not added.
4. **Remove vs Delete.** Remove detaches the key from the profile and deletes it on the provider (when `remote_key_id` is known); a provider failure aborts before anything local changes. Delete does the same and then unlinks the two local files, only after a confirmation that names both paths. Custom keys can be deleted the same way; the confirmation is the guard.
5. **Repository pinning is opt-in.** A new `GitBackend` pair, `ssh_command` / `set_ssh_command`, reads and writes the repository-local `core.sshCommand` only (deliberately not a general config write). The value comes from `ssh -i "<key>" -o IdentitiesOnly=yes`. Nothing sets it automatically: a profile's key is used by the repository only when the user asks, so a checked-out repository's behavior never changes because a profile was edited.
6. **No IPC for hosting.** Key generation, upload and deletion run client-side on a worker thread, like the rest of hosting; only the per-repo pin goes through the backend seam.

## Consequences
- Windows machines without OpenSSH tools can create and register keys.
- A profile's key file can outlive its profile if the profile is removed first; Remove/Delete the key before removing the profile (the profile-removal confirmation does not touch key files).
- The per-repo pin is wired end to end (2026-09-30): `OperationKind::SshCommand` (read) and `SetSshCommand`, the `ssh-command` CLI/MCP command, and the TUI's tab-menu row and palette entry. The `core.sshCommand` value is built by `gitpoulpi_hosting::ssh_key::ssh_command_for_key`, shared by the CLI and TUI.
- **Global key paths (2026-10-05).** `ssh.private_key_path` (else `ssh.public_key_path`) is read by the daemon before every network operation and handed to the system-git backend (`set_ssh_key`), which sets `GIT_SSH_COMMAND` on the transfer's git process only. Precedence, highest first: `GIT_SSH_COMMAND` / `GIT_SSH` already in the environment, `core.sshCommand` in the effective git config (the per-repository pin above, or a global one; checked with one `git config --get`, only when a key is set), the global key, then ssh's own defaults and the agent. The path is quoted for `sh` (`"`, `$` and the backtick escaped, backslashes become `/`) by `gitpoulpi_core::repo_config::ssh_command_for_key`, shared with the pin.
