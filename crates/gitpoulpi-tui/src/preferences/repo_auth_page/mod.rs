//! The SSH & credentials page (Settings > Repository): how this
//! repository signs in over HTTPS and SSH, per the per-repository
//! authentication design. The profile lives in GitPoulpi's private
//! directory and is owned by the daemon (`RepoAuthGet/Set/Clear/Test`);
//! the page never writes it itself.
//!
//! The screen has no daemon link of its own, so the event loop hands it a
//! [`Requester`] each frame (`set_repo_auth_requester`) and the page sends
//! its requests through it: every call runs on the link thread and answers
//! into a [`Reply`] the page polls (`poll_repo_auth`), so nothing here ever
//! waits on the render thread. The rows edit a [`Draft`]; Save sends the
//! whole profile. A helper or token command is confirmed first, in the
//! hook-trust pattern (`panes::CommandConfirm`): the verbatim command, its
//! SHA-256, no default answer.
//!
//! The Remotes list is part of the page body, so the page's own scrollbar
//! scrolls it; the results panel and the confirmation have their own.

mod draft;
mod panes;
mod pin;
mod results;
#[cfg(test)]
mod tests;

use std::time::{Duration, Instant};

use crossterm::event::{KeyEvent, MouseEvent};
use gitpoulpi_core::domain::{
    HttpsAuth, OperationKind, RemoteAccessReport, RepoAuthProfile, RepoAuthSetArgs,
    RepoAuthTestArgs, RepoAuthView,
};
use gitpoulpi_i18n::msg::repo_auth as t;
use ratatui::layout::Rect;
use ratatui::Frame;

use crate::daemon_link::{error_text::error_text, Reply, Requester};

use self::draft::{Draft, HttpsKind, SshKind};
use self::panes::{CommandConfirm, ConfirmEvent, ResultsEvent, ResultsPane};
use self::pin::{KeyAcl, PinMove};
use super::*;

pub(super) use self::draft::split_command;

/// How long a test runs before its spinner shows.
const SPINNER_AFTER: Duration = Duration::from_millis(150);

/// The rows of this page that are not plain config keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RaTarget {
    HttpsSource,
    Username,
    IgnoreHelpers,
    Account,
    Helper,
    Command,
    Hosts,
    Cache,
    SshSource,
    Key,
    SshAccount,
    KnownHosts,
    Strict,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RaButton {
    Test,
    Save,
    Revert,
    /// Asks first, then [`RaButton::Clear`].
    UseDefaults,
    /// What the "Use defaults" question runs.
    Clear,
    ImportHelper,
    /// Moves the repository's own `core.sshCommand` key pin into the profile.
    MovePin,
    /// Makes the key file readable by this account only.
    RestrictKey,
}

/// A connection test in flight: the fetch check first, then the push check
/// (one after the other: the daemon keeps one credential outcome per
/// repository).
struct TestRun {
    started: Instant,
    stage: Stage,
    fetch: Vec<RemoteAccessReport>,
}

enum Stage {
    Fetch(Reply<Vec<RemoteAccessReport>>),
    Push(Reply<Vec<RemoteAccessReport>>),
}

/// The page's state.
#[derive(Default)]
pub(super) struct RepoAuthPage {
    requester: Option<Requester>,
    loaded: bool,
    get: Option<Reply<RepoAuthView>>,
    /// Whether the next answer of `get` replaces the unsaved edits (after a
    /// save, a clear or a revert), not only the first one.
    reset_on_get: bool,
    view: RepoAuthView,
    /// The stored profile with its commands (the daemon's view blanks them):
    /// read from the private file when a view arrives.
    stored: RepoAuthProfile,
    baseline: Draft,
    draft: Draft,
    set: Option<Reply<serde_json::Value>>,
    clear: Option<Reply<bool>>,
    test: Option<TestRun>,
    /// A palette command asked for a test as soon as the page has loaded.
    want_test: bool,
    keys_rx: Option<std::sync::mpsc::Receiver<Vec<String>>>,
    /// Private keys found in the home folder's `.ssh`, read once on a thread.
    keys: Option<Vec<String>>,
    status: Option<String>,
    confirm: Option<CommandConfirm>,
    /// The profile the open confirmation would save.
    pending: Option<RepoAuthProfile>,
    results: Option<ResultsPane>,
    /// The key file's permissions (checked and fixed off the render thread).
    acl: KeyAcl,
    /// A "Move into GitPoulpi" in flight.
    pin_move: PinMove,
}

impl RepoAuthPage {
    fn dirty(&self) -> bool {
        self.draft != self.baseline
    }
}

/// `host` of a remote URL, for ordering accounts.
fn host_of(url: &str) -> Option<String> {
    crate::remotes_ui::split_remote_url(url).map(|(_, host, _)| host.to_ascii_lowercase())
}

impl PreferencesState {
    // -- what the event loop pushes in -----------------------------------------

    /// The link's request handle (a clone, every frame).
    pub(crate) fn set_repo_auth_requester(&mut self, requester: Requester) {
        self.repo_auth.requester = Some(requester);
    }

    /// Asks for a connection test as soon as the page has loaded.
    pub(crate) fn repo_auth_test_command(&mut self) {
        self.open_repo_auth_page();
        self.repo_auth.want_test = true;
    }

    /// Opens the "Use the default sign-in?" question over the page.
    pub(crate) fn repo_auth_defaults_command(&mut self) {
        self.open_repo_auth_page();
        self.ra_ask_defaults();
    }

    /// Lands on the page, the keyboard in its rows.
    pub(crate) fn open_repo_auth_page(&mut self) {
        let _ = self.show_page(Some(gitpoulpi_ipc::ui::SettingsPage::SshCredentials), None);
    }

    /// A frame is drawn while a test runs (the spinner).
    pub(super) fn ra_animating(&self) -> bool {
        self.repo_auth.test.is_some()
    }

    /// Whether one of the page's panels is open over it.
    pub(super) fn ra_overlay_open(&self) -> bool {
        self.repo_auth.confirm.is_some() || self.repo_auth.results.is_some()
    }

    /// Unsaved edits that closing Settings would lose.
    pub(super) fn ra_dirty(&self) -> bool {
        self.repo_auth.loaded && self.repo_auth.dirty()
    }

    // -- the poll ---------------------------------------------------------------

    /// Sends what is due and takes the answers that came. Never blocks.
    pub(crate) fn poll_repo_auth(&mut self) {
        let Some(requester) = self.repo_auth.requester.clone() else {
            return;
        };
        let ra = &mut self.repo_auth;
        if !ra.loaded && ra.get.is_none() {
            ra.get = Some(requester.call(OperationKind::RepoAuthGet, serde_json::json!({})));
        }
        if ra.keys.is_none() && ra.keys_rx.is_none() {
            ra.keys_rx = Some(spawn_key_scan(
                // Tests never read the real home folder.
                if cfg!(test) {
                    None
                } else {
                    gitpoulpi_config::home_dir()
                },
                self.config
                    .integrations
                    .iter()
                    .filter_map(|i| i.ssh.as_ref()?.private_key_path.clone())
                    .collect(),
            ));
        }
        if let Some(rx) = &ra.keys_rx {
            if let Ok(found) = rx.try_recv() {
                ra.keys = Some(found);
                ra.keys_rx = None;
            }
        }
        // A view that lands while a row is being edited or a dropdown is
        // open would shift the rows under the person's hands: it waits.
        let holding = self.choice.is_some() || self.edit.is_some();
        let ra = &mut self.repo_auth;
        if let Some(answer) = ra.get.as_ref().filter(|_| !holding).and_then(Reply::take) {
            ra.get = None;
            match answer {
                Ok(view) => self.ra_view_arrived(view),
                Err(e) => ra.status = Some(t::load_failed(error_text(&e))),
            }
        }
        let ra = &mut self.repo_auth;
        if let Some(answer) = ra.set.as_ref().and_then(Reply::take) {
            ra.set = None;
            match answer {
                Ok(_) => {
                    ra.status = Some(t::SAVED.get().to_string());
                    ra.reset_on_get = true;
                    ra.get =
                        Some(requester.call(OperationKind::RepoAuthGet, serde_json::json!({})));
                    // A pin being moved: its key is in the profile now.
                    self.ra_pin_saved(&requester);
                }
                Err(e) => {
                    ra.pin_move.key = None;
                    ra.status = Some(t::refused(error_text(&e)));
                }
            }
        }
        self.ra_poll_pin_move(&requester);
        self.ra_poll_key_acl();
        let ra = &mut self.repo_auth;
        if let Some(answer) = ra.clear.as_ref().and_then(Reply::take) {
            ra.clear = None;
            match answer {
                Ok(had) => {
                    ra.status = Some(
                        if had {
                            t::CLEARED.get()
                        } else {
                            t::NOTHING_TO_CLEAR.get()
                        }
                        .to_string(),
                    );
                    ra.reset_on_get = true;
                    ra.get =
                        Some(requester.call(OperationKind::RepoAuthGet, serde_json::json!({})));
                }
                Err(e) => ra.status = Some(t::refused(error_text(&e))),
            }
        }
        self.ra_poll_test(&requester);
        let ra = &mut self.repo_auth;
        if ra.want_test && ra.loaded && ra.test.is_none() {
            ra.want_test = false;
            self.ra_start_test(&requester);
        }
    }

    fn ra_view_arrived(&mut self, view: RepoAuthView) {
        let stored = read_stored(&self.repo_root).unwrap_or_else(|| view.profile.clone());
        self.ra_take_view(view, stored);
    }

    /// Installs a view and the stored profile. The first one, and the one
    /// after our own save, replace the draft; a later one keeps the edits.
    pub(super) fn ra_take_view(&mut self, view: RepoAuthView, stored: RepoAuthProfile) {
        let ra = &mut self.repo_auth;
        let replace = !ra.loaded || ra.reset_on_get || !ra.dirty();
        ra.view = view;
        ra.stored = stored;
        ra.baseline = Draft::from_profile(&ra.stored);
        if replace {
            ra.draft = ra.baseline.clone();
        }
        ra.loaded = true;
        ra.reset_on_get = false;
    }

    // -- buttons ----------------------------------------------------------------

    pub(super) fn ra_button(&mut self, button: RaButton) {
        match button {
            RaButton::Test => {
                if self.repo_auth.dirty() {
                    self.repo_auth.status = Some(t::TEST_SAVE_FIRST.get().to_string());
                } else {
                    self.repo_auth.want_test = true;
                }
            }
            RaButton::Save => self.ra_save(),
            RaButton::Revert => {
                self.repo_auth.draft = self.repo_auth.baseline.clone();
                self.repo_auth.status = Some(t::REVERTED.get().to_string());
            }
            RaButton::UseDefaults => self.ra_ask_defaults(),
            RaButton::Clear => {
                if let Some(requester) = self.repo_auth.requester.clone() {
                    self.repo_auth.clear =
                        Some(requester.call(OperationKind::RepoAuthClear, serde_json::json!({})));
                    self.repo_auth.status = Some(t::SAVING.get().to_string());
                }
            }
            RaButton::ImportHelper => self.ra_import_helper(),
            RaButton::MovePin => self.ra_move_pin(),
            RaButton::RestrictKey => self.ra_restrict_key(),
        }
    }

    fn ra_ask_defaults(&mut self) {
        let confirm = crate::dialog_kit::Confirm::new(
            t::DEFAULTS_TITLE.get(),
            t::DEFAULTS_BODY.get(),
            vec![
                (
                    gitpoulpi_i18n::msg::settings_screen::CANCEL
                        .get()
                        .to_string(),
                    false,
                ),
                (t::DEFAULTS_CONFIRM.get().to_string(), true),
            ],
            false,
        )
        .primary(true);
        self.confirm_action = Some((confirm, PrefAction::RepoAuth(RaButton::Clear)));
    }

    /// The credential helpers this repository sets for itself, from the
    /// view's overrides (local or worktree scope), with their origin.
    pub(super) fn ra_repo_helpers(&self) -> Vec<(String, String)> {
        self.repo_auth
            .view
            .overrides
            .iter()
            .filter(|o| {
                o.key == "credential.helper" && matches!(o.scope.as_str(), "local" | "worktree")
            })
            .map(|o| (o.value.clone(), o.origin.clone()))
            .collect()
    }

    fn ra_import_helper(&mut self) {
        let Some((value, _)) = self.ra_repo_helpers().into_iter().next() else {
            self.repo_auth.status = Some(t::IMPORT_NONE.get().to_string());
            return;
        };
        let d = &mut self.repo_auth.draft;
        d.https = HttpsKind::Helper;
        d.helper = value;
        self.repo_auth.status = Some(t::IMPORT_DONE.get().to_string());
    }

    fn ra_save(&mut self) {
        let ra = &mut self.repo_auth;
        let profile = match ra.draft.to_profile(&ra.stored) {
            Ok(p) => p,
            Err(why) => {
                ra.status = Some(t::invalid(why));
                return;
            }
        };
        if ra.draft.needs_confirm(&ra.stored) {
            let (helper, shown, hash) = match &profile.https {
                Some(HttpsAuth::Helper { helper, .. }) => {
                    (true, helper.clone(), command_hash(&profile))
                }
                Some(HttpsAuth::TokenCommand { argv, .. }) => {
                    (false, draft::join_command(argv), command_hash(&profile))
                }
                _ => {
                    self.ra_send_set(profile);
                    return;
                }
            };
            ra.confirm = Some(CommandConfirm::new(helper, &shown, &hash));
            ra.pending = Some(profile);
            return;
        }
        self.ra_send_set(profile);
    }

    fn ra_send_set(&mut self, profile: RepoAuthProfile) {
        let Some(requester) = self.repo_auth.requester.clone() else {
            return;
        };
        let confirm = matches!(
            profile.https,
            Some(HttpsAuth::Helper { .. }) | Some(HttpsAuth::TokenCommand { .. })
        );
        let args = RepoAuthSetArgs { profile, confirm };
        self.repo_auth.set = Some(requester.call(
            OperationKind::RepoAuthSet,
            serde_json::to_value(&args).unwrap_or_default(),
        ));
        self.repo_auth.status = Some(t::SAVING.get().to_string());
    }

    // -- the test ---------------------------------------------------------------

    fn ra_start_test(&mut self, requester: &Requester) {
        let ra = &mut self.repo_auth;
        ra.status = None;
        ra.test = Some(TestRun {
            started: Instant::now(),
            stage: Stage::Fetch(
                requester.call(
                    OperationKind::RepoAuthTest,
                    serde_json::to_value(RepoAuthTestArgs {
                        remote: None,
                        push: false,
                    })
                    .unwrap_or_default(),
                ),
            ),
            fetch: Vec::new(),
        });
    }

    fn ra_poll_test(&mut self, requester: &Requester) {
        let ra = &mut self.repo_auth;
        let Some(run) = ra.test.as_mut() else {
            return;
        };
        let answer = match &run.stage {
            Stage::Fetch(reply) | Stage::Push(reply) => reply.take(),
        };
        let Some(answer) = answer else {
            return;
        };
        let reports = match answer {
            Ok(reports) => reports,
            Err(e) => {
                let text = results::error_lines(&error_text(&e));
                ra.test = None;
                ra.results = Some(ResultsPane::new(text));
                return;
            }
        };
        match &run.stage {
            Stage::Fetch(_) => {
                run.fetch = reports;
                run.stage = Stage::Push(
                    requester.call(
                        OperationKind::RepoAuthTest,
                        serde_json::to_value(RepoAuthTestArgs {
                            remote: None,
                            push: true,
                        })
                        .unwrap_or_default(),
                    ),
                );
            }
            Stage::Push(_) => {
                let fetch = std::mem::take(&mut run.fetch);
                ra.test = None;
                ra.results = Some(ResultsPane::new(results::lines(
                    &ra.view.remotes,
                    &fetch,
                    &reports,
                )));
            }
        }
    }

    // -- the rows' commits ---------------------------------------------------------

    pub(super) fn ra_commit_text(&mut self, target: RaTarget, text: &str) -> bool {
        let d = &mut self.repo_auth.draft;
        let text = text.trim().to_string();
        match target {
            RaTarget::Username => d.username = text,
            RaTarget::Helper => d.helper = text,
            RaTarget::Command => d.command = text,
            RaTarget::Hosts => d.hosts = text,
            RaTarget::Cache => d.cache = text,
            RaTarget::Key => d.key = text,
            RaTarget::KnownHosts => d.known_hosts = text,
            _ => {}
        }
        true
    }

    pub(super) fn ra_commit_bool(&mut self, target: RaTarget, value: bool) {
        let d = &mut self.repo_auth.draft;
        match target {
            RaTarget::IgnoreHelpers => d.ignore_repo_helpers = value,
            RaTarget::Strict => d.strict = value,
            _ => {}
        }
    }

    pub(super) fn ra_commit_choice(&mut self, target: RaTarget, next: usize) {
        match target {
            RaTarget::HttpsSource => {
                if let Some(kind) = self.ra_https_kinds().get(next) {
                    self.repo_auth.draft.https = *kind;
                }
            }
            RaTarget::SshSource => {
                if let Some(kind) = self.ra_ssh_kinds().get(next) {
                    self.repo_auth.draft.ssh = *kind;
                }
            }
            RaTarget::Account | RaTarget::SshAccount => {
                let id = next
                    .checked_sub(1)
                    .and_then(|i| self.ra_accounts().into_iter().nth(i))
                    .map(|(id, _)| id)
                    .unwrap_or_default();
                if target == RaTarget::Account {
                    self.repo_auth.draft.account = id;
                } else {
                    self.repo_auth.draft.ssh_account = id;
                }
            }
            _ => {}
        }
    }

    pub(super) fn ra_commit_combo(&mut self, target: RaTarget, value: &str) {
        if target == RaTarget::Key {
            self.repo_auth.draft.key = value.trim().to_string();
        }
    }

    // -- the panels -------------------------------------------------------------

    pub(super) fn ra_overlay_key(&mut self, key: KeyEvent) {
        let ra = &mut self.repo_auth;
        if let Some(confirm) = ra.confirm.as_mut() {
            match confirm.key(key) {
                ConfirmEvent::Handled => {}
                ConfirmEvent::Save => self.ra_confirmed(true),
                ConfirmEvent::Cancel => self.ra_confirmed(false),
            }
        } else if let Some(results) = ra.results.as_mut() {
            if results.key(key) == ResultsEvent::Close {
                ra.results = None;
            }
        }
    }

    pub(super) fn ra_overlay_mouse(&mut self, mouse: MouseEvent) {
        let ra = &mut self.repo_auth;
        if let Some(confirm) = ra.confirm.as_mut() {
            match confirm.mouse(mouse) {
                ConfirmEvent::Handled => {}
                ConfirmEvent::Save => self.ra_confirmed(true),
                ConfirmEvent::Cancel => self.ra_confirmed(false),
            }
        } else if let Some(results) = ra.results.as_mut() {
            if results.mouse(mouse) == ResultsEvent::Close {
                ra.results = None;
            }
        }
    }

    fn ra_confirmed(&mut self, save: bool) {
        self.repo_auth.confirm = None;
        let pending = self.repo_auth.pending.take();
        if let (true, Some(profile)) = (save, pending) {
            self.ra_send_set(profile);
        }
    }

    pub(super) fn render_repo_auth_overlays(&mut self, frame: &mut Frame, area: Rect) {
        let ra = &mut self.repo_auth;
        if let Some(results) = ra.results.as_mut() {
            results.render(frame.buffer_mut(), area);
        }
        if let Some(confirm) = ra.confirm.as_mut() {
            confirm.render(frame.buffer_mut(), area);
        }
    }

    // -- the rows ---------------------------------------------------------------

    fn ra_https_kinds(&self) -> Vec<HttpsKind> {
        let mut kinds = vec![HttpsKind::System];
        if self.ra_has_accounts() {
            kinds.push(HttpsKind::Account);
        }
        kinds.push(HttpsKind::Helper);
        kinds.push(HttpsKind::TokenCommand);
        kinds
    }

    fn ra_ssh_kinds(&self) -> Vec<SshKind> {
        let mut kinds = vec![SshKind::System, SshKind::KeyFile];
        if self.ra_has_accounts() {
            kinds.push(SshKind::AccountKey);
        }
        kinds
    }

    /// The account choice needs the hosting feature (the integrations).
    fn ra_has_accounts(&self) -> bool {
        crate::build_features::offered(Some(crate::build_features::Feature::Hosting))
    }

    /// The accounts to pick from: those whose host is one of this
    /// repository's remotes first.
    pub(super) fn ra_accounts(&self) -> Vec<(String, String)> {
        let hosts: Vec<String> = self
            .repo_auth
            .view
            .remotes
            .iter()
            .filter_map(|r| host_of(&r.url))
            .collect();
        let mut accounts: Vec<(bool, String, String)> = self
            .config
            .integrations
            .iter()
            .filter_map(|i| {
                let host = i.host()?;
                let matches = hosts.iter().any(|h| {
                    let bare = host.split(':').next().unwrap_or(&host).to_ascii_lowercase();
                    *h == bare
                });
                Some((matches, i.id.clone(), t::account_option(&i.id, &host)))
            })
            .collect();
        accounts.sort_by_key(|(matches, ..)| !*matches);
        accounts
            .into_iter()
            .map(|(_, id, label)| (id, label))
            .collect()
    }

    fn ra_account_choice(&self, selected: &str, target: RaTarget, label: &str) -> Field {
        let accounts = self.ra_accounts();
        let mut options = vec![t::ACCOUNT_NONE.get().to_string()];
        options.extend(accounts.iter().map(|(_, l)| l.clone()));
        let index = accounts
            .iter()
            .position(|(id, _)| id == selected)
            .map_or(0, |i| i + 1);
        Field::plain(
            label,
            Control::Choice { options, index },
            Target::RepoAuth(target),
        )
    }

    fn ra_scope_line(&self) -> String {
        match self.integrations.scope_notes(&self.repo_auth.draft.account) {
            None => t::SCOPE_UNKNOWN.get().to_string(),
            Some((missing, excess)) => {
                let mut parts = Vec::new();
                if !missing.is_empty() {
                    parts.push(t::scope_missing(missing.join(", ")));
                }
                if !excess.is_empty() {
                    parts.push(t::scope_excess(excess.join(", ")));
                }
                if parts.is_empty() {
                    t::SCOPE_OK.get().to_string()
                } else {
                    parts.join("; ")
                }
            }
        }
    }

    fn ra_ssh_program(&self) -> String {
        let want_global = self.repo_auth.draft.ssh != SshKind::System;
        self.repo_auth
            .view
            .overrides
            .iter()
            .filter(|o| o.key == "core.sshCommand")
            .find(|o| !want_global || matches!(o.scope.as_str(), "global" | "system"))
            .map_or_else(
                || t::SSH_PROGRAM_DEFAULT.get().to_string(),
                |o| o.value.clone(),
            )
    }

    pub(super) fn repo_auth_fields(&self) -> Vec<Field> {
        let ra = &self.repo_auth;
        let d = &ra.draft;
        let target = Target::RepoAuth;
        let mut f = vec![Field::plain(
            t::INTRO.get(),
            Control::Info(String::new()),
            Target::None,
        )];

        // -- HTTPS ----------------------------------------------------------------
        f.push(Field::heading(t::HEADING_HTTPS.get()));
        let kinds = self.ra_https_kinds();
        let labels: Vec<String> = kinds
            .iter()
            .map(|k| {
                match k {
                    HttpsKind::System => t::HTTPS_SYSTEM,
                    HttpsKind::Account => t::HTTPS_ACCOUNT,
                    HttpsKind::Helper => t::HTTPS_HELPER,
                    HttpsKind::TokenCommand => t::HTTPS_TOKEN_COMMAND,
                }
                .get()
                .to_string()
            })
            .collect();
        f.push(Field::plain(
            t::HTTPS_SOURCE.get(),
            Control::Choice {
                options: labels,
                index: kinds.iter().position(|k| *k == d.https).unwrap_or(0),
            },
            target(RaTarget::HttpsSource),
        ));
        let username = |label: &str| {
            Field::plain(
                label,
                Control::Text(d.username.clone()),
                target(RaTarget::Username),
            )
            .with_help(t::USERNAME_HELP.get())
        };
        match d.https {
            HttpsKind::System => {
                f.push(username(t::USERNAME.get()));
                f.push(
                    Field::plain(
                        t::IGNORE_HELPERS.get(),
                        Control::Toggle(d.ignore_repo_helpers),
                        target(RaTarget::IgnoreHelpers),
                    )
                    .with_help(t::IGNORE_HELPERS_HELP.get()),
                );
                let helpers = self.ra_repo_helpers();
                if helpers.is_empty() {
                    f.push(Field::plain(
                        t::NO_REPO_HELPERS.get(),
                        Control::Info(String::new()),
                        Target::None,
                    ));
                }
                for (value, origin) in helpers {
                    f.push(Field::plain(
                        t::REPO_HELPER_LABEL.get(),
                        Control::Info(t::repo_helper_line(value, origin)),
                        Target::None,
                    ));
                }
            }
            HttpsKind::Account => {
                f.push(self.ra_account_choice(&d.account, RaTarget::Account, t::ACCOUNT.get()));
                f.push(username(t::USERNAME.get()));
                f.push(Field::plain(
                    t::SCOPE_LABEL.get(),
                    Control::Info(self.ra_scope_line()),
                    Target::None,
                ));
            }
            HttpsKind::Helper => {
                f.push(
                    Field::plain(
                        t::HELPER_LABEL.get(),
                        Control::Text(d.helper.clone()),
                        target(RaTarget::Helper),
                    )
                    .with_help(t::HELPER_HELP.get())
                    .with_danger(t::COMMAND_WARNING.get()),
                );
                f.push(Field::buttons(
                    "",
                    vec![Btn::new(
                        t::IMPORT_HELPER.get(),
                        PrefAction::RepoAuth(RaButton::ImportHelper),
                    )],
                ));
                f.push(username(t::USERNAME.get()));
                f.push(
                    Field::plain(
                        t::HOSTS_LABEL.get(),
                        Control::Text(d.hosts.clone()),
                        target(RaTarget::Hosts),
                    )
                    .with_help(t::HOSTS_HELP.get()),
                );
            }
            HttpsKind::TokenCommand => {
                f.push(
                    Field::plain(
                        t::COMMAND_LABEL.get(),
                        Control::Text(d.command.clone()),
                        target(RaTarget::Command),
                    )
                    .with_help(t::COMMAND_HELP.get())
                    .with_danger(t::COMMAND_WARNING.get()),
                );
                let preview = match split_command(&d.command) {
                    Ok(argv) if !argv.is_empty() => argv
                        .iter()
                        .map(|a| format!("[{a}]"))
                        .collect::<Vec<_>>()
                        .join(" "),
                    _ => String::new(),
                };
                f.push(Field::plain(
                    t::ARGV_LABEL.get(),
                    Control::Info(preview),
                    Target::None,
                ));
                f.push(
                    Field::plain(
                        t::HOSTS_LABEL.get(),
                        Control::Text(d.hosts.clone()),
                        target(RaTarget::Hosts),
                    )
                    .with_help(t::HOSTS_HELP.get()),
                );
                f.push(
                    Field::plain(
                        t::CACHE_LABEL.get(),
                        Control::Text(d.cache.clone()),
                        target(RaTarget::Cache),
                    )
                    .with_help(t::CACHE_HELP.get()),
                );
                f.push(username(t::USERNAME.get()));
            }
        }

        // -- SSH ------------------------------------------------------------------
        f.push(Field::heading(t::HEADING_SSH.get()));
        let kinds = self.ra_ssh_kinds();
        let labels: Vec<String> = kinds
            .iter()
            .map(|k| {
                match k {
                    SshKind::System => t::SSH_SYSTEM,
                    SshKind::KeyFile => t::SSH_KEY_FILE,
                    SshKind::AccountKey => t::SSH_ACCOUNT_KEY,
                }
                .get()
                .to_string()
            })
            .collect();
        f.push(Field::plain(
            t::SSH_SOURCE.get(),
            Control::Choice {
                options: labels,
                index: kinds.iter().position(|k| *k == d.ssh).unwrap_or(0),
            },
            target(RaTarget::SshSource),
        ));
        match d.ssh {
            SshKind::System => {}
            SshKind::KeyFile => {
                let mut options: Vec<(String, String)> = Vec::new();
                for key in ra.keys.iter().flatten() {
                    options.push((key.clone(), key.clone()));
                }
                let exposed = self.ra_key_exposed();
                let mut key_row = Field::plain(
                    t::KEY_PATH.get(),
                    Control::Combo {
                        options,
                        value: d.key.clone(),
                        custom: true,
                    },
                    target(RaTarget::Key),
                )
                .with_help(t::KEY_HELP.get());
                if exposed {
                    key_row = key_row.with_danger(t::ACL_WARNING.get());
                }
                f.push(key_row);
                if exposed {
                    f.push(Field::buttons(
                        "",
                        vec![Btn::new(
                            t::ACL_BUTTON.get(),
                            PrefAction::RepoAuth(RaButton::RestrictKey),
                        )],
                    ));
                }
            }
            SshKind::AccountKey => {
                f.push(self.ra_account_choice(
                    &d.ssh_account,
                    RaTarget::SshAccount,
                    t::ACCOUNT.get(),
                ));
            }
        }
        if d.ssh != SshKind::System {
            f.push(
                Field::plain(
                    t::KNOWN_HOSTS.get(),
                    Control::Text(d.known_hosts.clone()),
                    target(RaTarget::KnownHosts),
                )
                .with_help(t::KNOWN_HOSTS_HELP.get()),
            );
            f.push(
                Field::plain(
                    t::STRICT_HOST.get(),
                    Control::Toggle(d.strict),
                    target(RaTarget::Strict),
                )
                .with_help(t::STRICT_HOST_HELP.get()),
            );
        }
        f.push(Field::plain(
            t::SSH_PROGRAM_LABEL.get(),
            Control::Info(self.ra_ssh_program()),
            Target::None,
        ));

        f.extend(self.ra_pin_rows());

        // -- Remotes ----------------------------------------------------------------
        f.push(Field::heading(t::HEADING_REMOTES.get()));
        if ra.view.remotes.is_empty() {
            f.push(Field::plain(
                t::NO_REMOTES.get(),
                Control::Info(String::new()),
                Target::None,
            ));
        }
        for r in &ra.view.remotes {
            f.push(Field::plain(
                &r.remote,
                Control::Info(t::remote_value(
                    format!("{:?}", r.transport).to_ascii_lowercase(),
                    &r.identity,
                )),
                Target::None,
            ));
        }
        for o in &ra.view.overrides {
            let line = if o.overridden {
                t::override_overridden(&o.key, &o.value, &o.origin)
            } else {
                t::override_competes(&o.key, &o.value, &o.origin)
            };
            f.push(Field::plain(
                &line,
                Control::Info(String::new()),
                Target::None,
            ));
        }
        for w in &ra.view.warnings {
            f.push(Field::plain(w, Control::Info(String::new()), Target::None));
        }
        if let Some(why) = &ra.view.unavailable {
            f.push(Field::plain(
                t::UNAVAILABLE_LABEL.get(),
                Control::Info(why.clone()),
                Target::None,
            ));
        }

        // -- Buttons and the status line -----------------------------------------------
        let dirty = ra.dirty();
        let busy = ra.set.is_some() || ra.clear.is_some() || ra.test.is_some();
        let mut test = Btn::new(t::BUTTON_TEST.get(), PrefAction::RepoAuth(RaButton::Test));
        test.enabled = !busy;
        let mut save = Btn::new(t::BUTTON_SAVE.get(), PrefAction::RepoAuth(RaButton::Save));
        save.enabled = dirty && !busy;
        let mut revert = Btn::new(
            t::BUTTON_REVERT.get(),
            PrefAction::RepoAuth(RaButton::Revert),
        );
        revert.enabled = dirty;
        let defaults = Btn::new(
            t::BUTTON_DEFAULTS.get(),
            PrefAction::RepoAuth(RaButton::UseDefaults),
        );
        f.push(Field::buttons("", vec![test, save, revert, defaults]));
        let status = match &ra.test {
            Some(run) if run.started.elapsed() >= SPINNER_AFTER => Some(t::test_running(
                crate::activity::spinner_frame(run.started.elapsed()),
            )),
            Some(_) => None,
            None if !ra.loaded => Some(t::LOADING.get().to_string()),
            None => ra
                .status
                .clone()
                .or_else(|| dirty.then(|| t::DIRTY_NOTE.get().to_string())),
        };
        if let Some(text) = status {
            f.push(Field::plain(
                &text,
                Control::Info(String::new()),
                Target::None,
            ));
        }
        f
    }
}

/// The SHA-256 the confirmation shows for the command `profile` would run.
fn command_hash(profile: &RepoAuthProfile) -> String {
    profile
        .https
        .as_ref()
        .and_then(HttpsAuth::command_sha256)
        .unwrap_or_default()
}

/// The stored profile with its commands, from the private file (the daemon's
/// view blanks them). A small local file, read when a view arrives.
fn read_stored(repo_root: &std::path::Path) -> Option<RepoAuthProfile> {
    // Tests never read the real private directory.
    if cfg!(test) {
        return None;
    }
    let paths = gitpoulpi_config::ConfigPaths::discover()?;
    gitpoulpi_session::load_repo_auth(&paths, repo_root)
        .ok()
        .flatten()
        .map(|record| record.profile())
}

/// Lists the private keys of `<home>/.ssh` (a file with a `.pub` beside it,
/// or named `id_*`) plus `extra` (the integrations' own keys) on a thread.
fn spawn_key_scan(
    home: Option<std::path::PathBuf>,
    extra: Vec<String>,
) -> std::sync::mpsc::Receiver<Vec<String>> {
    use crate::redraw::NotifySend as _;
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let mut found: Vec<String> = Vec::new();
        if let Some(dir) = home.map(|h| h.join(".ssh")) {
            if let Ok(entries) = std::fs::read_dir(&dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    let name = entry.file_name().to_string_lossy().into_owned();
                    let is_pub = name.ends_with(".pub");
                    let is_private = path.is_file()
                        && !is_pub
                        && !matches!(
                            name.as_str(),
                            "known_hosts" | "known_hosts.old" | "config" | "authorized_keys"
                        )
                        && (name.starts_with("id_")
                            || path.with_file_name(format!("{name}.pub")).is_file());
                    if is_private {
                        found.push(path.to_string_lossy().into_owned());
                    }
                }
            }
        }
        found.extend(extra);
        found.sort();
        found.dedup();
        let _ = tx.send_notify(found);
    });
    rx
}
