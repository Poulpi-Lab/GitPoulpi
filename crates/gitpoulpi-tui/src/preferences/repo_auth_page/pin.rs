//! Two rows of the SSH & credentials page that look at the machine rather than
//! at the profile: the permissions of the key file (OpenSSH refuses a key other
//! accounts can read) and an SSH key the repository pins in its own
//! `core.sshCommand`, which the page shows read-only and can move into
//! GitPoulpi.
//!
//! Neither runs on the render thread: the permission check and its fix run on
//! a thread (a key on a slow share would stall a frame), the move is two
//! daemon requests through the page's requester.
//!
//! Undo story of the move: the profile write is a setting, not undoable
//! (`RepoAuthSet`); the unset is `SetSshCommand`, which is not journaled either
//! (changing the SSH command never is: nothing is lost, the key is now in the
//! profile). The move saves the profile first, so a failed save leaves the pin
//! alone and a failed unset leaves both, which still sign in with the same key.

use std::path::PathBuf;
use std::sync::mpsc::Receiver;

use super::*;

/// What the page knows of the key file named by its draft.
#[derive(Default)]
pub(super) struct KeyAcl {
    /// The key text the last check was started for.
    path: Option<String>,
    scan: Option<Receiver<bool>>,
    /// Other accounts can read the key named by `path`.
    exposed: bool,
    fix: Option<Receiver<Result<(), String>>>,
}

/// A move in flight: the key being moved, and the request that removes the
/// pin once the profile is saved.
#[derive(Default)]
pub(super) struct PinMove {
    pub(super) key: Option<String>,
    pub(super) unset: Option<Reply<serde_json::Value>>,
}

/// `~` and `~/...` as the home folder (ssh does the same for `-i`).
fn expand(text: &str) -> PathBuf {
    let rest = text.strip_prefix("~/").or_else(|| text.strip_prefix("~\\"));
    match (rest, gitpoulpi_config::home_dir()) {
        (Some(rest), Some(home)) => home.join(rest),
        _ => PathBuf::from(text),
    }
}

impl PreferencesState {
    /// Starts the permission check when the draft's key changed and takes the
    /// answers of the check and of the fix. Never blocks.
    pub(super) fn ra_poll_key_acl(&mut self) {
        use crate::redraw::NotifySend as _;
        let ra = &mut self.repo_auth;
        if let Some(rx) = &ra.acl.scan {
            if let Ok(exposed) = rx.try_recv() {
                ra.acl.exposed = exposed;
                ra.acl.scan = None;
            }
        }
        if let Some(rx) = &ra.acl.fix {
            if let Ok(done) = rx.try_recv() {
                ra.acl.fix = None;
                match done {
                    Ok(()) => {
                        ra.status = Some(t::ACL_DONE.get().to_string());
                        // Look again: the answer decides whether the warning goes.
                        ra.acl.path = None;
                    }
                    Err(why) => ra.status = Some(t::acl_failed(why)),
                }
            }
        }
        let want = (ra.draft.ssh == SshKind::KeyFile && !ra.draft.key.trim().is_empty())
            .then(|| ra.draft.key.trim().to_string());
        match want {
            None => {
                ra.acl = KeyAcl {
                    fix: ra.acl.fix.take(),
                    ..KeyAcl::default()
                };
            }
            Some(key) if ra.acl.path.as_deref() != Some(&key) && ra.acl.scan.is_none() => {
                ra.acl.path = Some(key.clone());
                ra.acl.exposed = false;
                let (tx, rx) = std::sync::mpsc::channel();
                std::thread::spawn(move || {
                    let _ = tx.send_notify(gitpoulpi_config::secure_fs::readable_by_others(
                        &expand(&key),
                    ));
                });
                ra.acl.scan = Some(rx);
            }
            Some(_) => {}
        }
    }

    /// Whether the key named by the draft can be read by other accounts.
    pub(super) fn ra_key_exposed(&self) -> bool {
        let ra = &self.repo_auth;
        ra.acl.exposed
            && ra.acl.path.as_deref() == Some(ra.draft.key.trim())
            && ra.draft.ssh == SshKind::KeyFile
    }

    /// "Restrict to my account": `chmod 600` (Unix) or a user-only DACL
    /// (Windows) on the key file, on a thread.
    pub(super) fn ra_restrict_key(&mut self) {
        use crate::redraw::NotifySend as _;
        let ra = &mut self.repo_auth;
        if ra.acl.fix.is_some() {
            return;
        }
        let key = ra.draft.key.trim().to_string();
        if key.is_empty() {
            return;
        }
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let done = gitpoulpi_config::secure_fs::restrict_to_user(&expand(&key))
                .map_err(|e| e.to_string());
            let _ = tx.send_notify(done);
        });
        ra.acl.fix = Some(rx);
    }

    /// The `core.sshCommand` this repository sets for itself, from the view's
    /// overrides (local or worktree scope).
    pub(super) fn ra_pin(&self) -> Option<String> {
        self.repo_auth
            .view
            .overrides
            .iter()
            .find(|o| {
                o.key.eq_ignore_ascii_case("core.sshCommand")
                    && matches!(o.scope.as_str(), "local" | "worktree")
            })
            .map(|o| o.value.clone())
    }

    /// The rows of the pin: its command, read-only, and the button that moves
    /// it into GitPoulpi (or why it cannot be moved).
    pub(super) fn ra_pin_rows(&self) -> Vec<Field> {
        let Some(pin) = self.ra_pin() else {
            return Vec::new();
        };
        let mut rows =
            vec![
                Field::plain(t::PIN_LABEL.get(), Control::Info(pin.clone()), Target::None)
                    .with_help(t::PIN_HELP.get()),
            ];
        if gitpoulpi_core::repo_config::key_of_ssh_command(&pin).is_some() {
            let ra = &self.repo_auth;
            let mut button = Btn::new(t::PIN_MOVE.get(), PrefAction::RepoAuth(RaButton::MovePin));
            button.enabled = ra.set.is_none() && ra.pin_move.unset.is_none();
            rows.push(Field::buttons("", vec![button]));
        } else {
            rows.push(Field::plain(
                t::PIN_OTHER.get(),
                Control::Info(String::new()),
                Target::None,
            ));
        }
        rows
    }

    /// "Move into GitPoulpi": saves the equivalent key-file profile, then
    /// (when that is saved) unsets the repository's own `core.sshCommand`.
    pub(super) fn ra_move_pin(&mut self) {
        let Some(key) = self
            .ra_pin()
            .and_then(|pin| gitpoulpi_core::repo_config::key_of_ssh_command(&pin))
        else {
            return;
        };
        let ra = &mut self.repo_auth;
        if ra.dirty() {
            ra.status = Some(t::PIN_SAVE_FIRST.get().to_string());
            return;
        }
        if !matches!(
            ra.stored.ssh,
            None | Some(gitpoulpi_core::domain::SshAuth::System)
        ) {
            ra.status = Some(t::PIN_SSH_TAKEN.get().to_string());
            return;
        }
        let mut profile = ra.stored.clone();
        profile.ssh = Some(gitpoulpi_core::domain::SshAuth::KeyFile {
            key: key.clone(),
            known_hosts: None,
            strict_host_key: false,
        });
        ra.pin_move.key = Some(key);
        self.ra_send_set(profile);
    }

    /// The profile is saved: remove the pin (the move's second half).
    pub(super) fn ra_pin_saved(&mut self, requester: &Requester) {
        let ra = &mut self.repo_auth;
        if ra.pin_move.key.is_some() {
            ra.pin_move.unset = Some(requester.call(
                OperationKind::SetSshCommand,
                serde_json::json!({ "command": null }),
            ));
        }
    }

    /// Takes the answer of the pin removal.
    pub(super) fn ra_poll_pin_move(&mut self, requester: &Requester) {
        let ra = &mut self.repo_auth;
        let Some(answer) = ra.pin_move.unset.as_ref().and_then(Reply::take) else {
            return;
        };
        ra.pin_move.unset = None;
        let key = ra.pin_move.key.take().unwrap_or_default();
        ra.status = Some(match answer {
            Ok(_) => t::pin_moved(key),
            Err(e) => t::pin_unset_failed(error_text(&e)),
        });
        ra.reset_on_get = true;
        ra.get = Some(requester.call(OperationKind::RepoAuthGet, serde_json::json!({})));
    }
}
