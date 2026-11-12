use super::*;

impl IntegrationsPage {
    pub(super) fn write_config(env: &mut Env) {
        let Some(path) = env.config_path.clone() else {
            *env.message = Some(msg::NO_CONFIG_DIR.get().into());
            return;
        };
        if let Err(err) = set_list_in_file(&path, "integrations", &env.config.integrations) {
            *env.message = Some(crate::activity_log::report_problem_clear(
                crate::activity_log::ProblemSource::Save,
                &msg::config_write_failed(err),
            ));
        }
    }

    pub(super) fn open_url(&mut self, url: &str, env: &mut Env) {
        if let Err(err) = (self.opener)(url) {
            *env.message = Some(crate::activity_log::report_problem_clear(
                crate::activity_log::ProblemSource::Tool,
                &err,
            ));
        } else {
            *env.message = Some(msg::opened_url(url));
        }
    }

    pub(super) fn open_token_page(&mut self, env: &mut Env) {
        match self.token_url() {
            Some(url) => self.open_url(&url, env),
            None => {
                let why = match self.kind().capabilities().account {
                    Some(AccountField::ApiKey) => msg2::ERR_ACCOUNT_KEY,
                    Some(AccountField::Username) => msg2::ERR_ACCOUNT_USERNAME,
                    _ if self.kind().capabilities().org => msg2::ERR_ORG,
                    _ => msg::ENTER_HOST_FIRST,
                };
                *env.message = Some(why.get().into());
            }
        }
    }

    pub(super) fn connect(&mut self, env: &mut Env) {
        let Mode::Form(editing) = self.mode.clone() else {
            return;
        };
        let kind = self.kind();
        if let Some(why) = self.missing_input() {
            self.error = Some(why.get().into());
            return;
        }
        let Some(url) = self.form_url() else {
            return;
        };
        let typed = self.token.trim().to_string();
        if kind.capabilities().local_folder {
            // An absolute folder must exist; a relative one is read against
            // each repository, so it cannot be checked here.
            if std::path::Path::new(&url).is_absolute() && !std::path::Path::new(&url).is_dir() {
                self.error = Some(gitpoulpi_i18n::msg::local_issues::err_not_folder(&url));
                return;
            }
        } else if typed.is_empty() && !self.stored_token(env.config, env.presence) {
            self.error = Some(msg::ERR_TOKEN.get().into());
            return;
        }
        let mut integration = match editing
            .as_ref()
            .and_then(|id| env.config.integrations.iter().find(|i| &i.id == id))
        {
            Some(existing) => existing.clone(),
            None => {
                let id = Integration::new_id(kind, env.config.integrations.iter());
                Integration::new(id, kind, url.clone())
            }
        };
        // A changed address is only accepted together with a token typed for
        // it: the saved one must never travel to the new address (the
        // security check refuses that, and so does this form).
        let moved = editing.is_some()
            && !kind.capabilities().local_folder
            && gitpoulpi_hosting::trust::integration_origin(&integration).ok()
                != gitpoulpi_hosting::trust::normalize(&url);
        if moved && typed.is_empty() {
            self.error = Some(tmsg::URL_CHANGED_NEEDS_TOKEN.get().into());
            return;
        }
        integration.url = url;
        integration.account = self.account_text();
        self.error = None;
        let trust = TrustOptions {
            // A profile that was never connected has no token bound to its
            // old address: the form's address is the one to use.
            trust_new_origin: moved || integration.user.is_none(),
            allow_plain_http: self.plain_http && self.plain_http_offered(),
            strict: self.strict && self.strict_offered(),
        };

        let (tx, rx) = mpsc::channel();
        let tokens = Arc::clone(env.tokens);
        let transport = self.transport.clone();
        let job = integration.clone();
        let paths = self.trust_paths.clone();
        std::thread::spawn(move || {
            let gate = match paths {
                Some(p) => gitpoulpi_hosting::TrustGate::load(&p),
                None => gitpoulpi_hosting::TrustGate::ambient(),
            };
            let _ = tx.send_notify(run_check(&tokens, job, typed, transport, trust, &gate));
        });
        self.pending = Some(Pending {
            rx,
            integration,
            editing: editing.clone(),
            config_path: env.config_path.clone(),
            detach: false,
        });
        self.mode = Mode::Connecting(editing);
        self.item = 0;
    }

    /// Preferences is closing: a running token check finishes on its own
    /// and saves the profile (see `Pending::drop`).
    pub(crate) fn detach_pending(&mut self) {
        if let Some(pending) = self.pending.as_mut() {
            pending.detach = true;
        }
        self.pending = None;
    }

    /// Applies the background check's answer, if it has arrived.
    pub(crate) fn poll(&mut self, env: &mut Env) {
        // The security check's Disconnect rewrote the profile list on a
        // worker thread: read it again (nothing is being edited that it
        // could clobber, the page's own state is rebuilt from the file).
        if crate::trust_dialog::take_changed() {
            if let Some(path) = env.config_path.as_deref() {
                if let Ok(fresh) = Config::load(path) {
                    env.config.integrations = fresh.integrations;
                    self.pinned.clear();
                    self.rights.clear();
                    self.reset_view(env.config);
                }
            }
        }
        self.flush_radar(env);
        self.poll_ssh(env);
        self.pump_picker();
        let Some(pending) = &self.pending else {
            return;
        };
        let answer = match pending.rx.try_recv() {
            Ok(answer) => answer,
            Err(mpsc::TryRecvError::Empty) => return,
            Err(mpsc::TryRecvError::Disconnected) => Err(msg::CHECK_ENDED.get().to_string()),
        };
        let Some(pending) = self.pending.take() else {
            return;
        };
        let mut integration = pending.integration.clone();
        let editing = pending.editing.clone();
        match answer {
            Err(err) => {
                crate::activity_log::report_problem(
                    crate::activity_log::ProblemSource::Hosting,
                    &err,
                );
                self.mode = Mode::Form(editing);
                self.error = Some(err);
            }
            Ok(check) => {
                integration.user = Some(ConnectedUser {
                    login: check.user.login.clone(),
                    name: check.user.name.clone(),
                    id: check.user.id.clone(),
                });
                integration.enabled = true;
                env.presence
                    .insert(integration.token_ref.clone(), Some(true));
                if check.missing_scopes.is_empty() {
                    self.warnings.remove(&integration.id);
                } else {
                    self.warnings
                        .insert(integration.id.clone(), check.missing_scopes.clone());
                }
                let rights = Rights::of(&check);
                if rights.is_empty() {
                    self.rights.remove(&integration.id);
                } else {
                    self.rights.insert(integration.id.clone(), rights);
                }
                self.pinned.remove(&integration.id);
                *env.message = Some(if integration.kind.capabilities().local_folder {
                    gitpoulpi_i18n::msg::local_issues::folder_added(&integration.url)
                } else {
                    let connected = msg::connected_to(
                        integration.host().unwrap_or_default(),
                        &check.user.login,
                    );
                    // Only the process's memory holds a token the OS store
                    // refused: say so instead of a bare "connected".
                    match env.tokens.session_only_reason(&integration.token_ref) {
                        Some(why) => msg::connected_session_only(connected, why),
                        None => connected,
                    }
                });
                let id = integration.id.clone();
                match env.config.integrations.iter().position(|i| i.id == id) {
                    Some(at) => env.config.integrations[at] = integration,
                    None => env.config.integrations.push(integration),
                }
                Self::write_config(env);
                self.token.clear();
                self.host.clear();
                self.org.clear();
                self.account.clear();
                self.mode = Mode::Browse;
                self.sel = self
                    .profile_ids(env.config)
                    .iter()
                    .position(|i| env.config.integrations[*i].id == id)
                    .unwrap_or(0);
                self.item = 0;
                self.zone = Zone::Body;
            }
        }
    }

    /// Drops the trust record and the caches filled from `integration`.
    fn forget_trust(&mut self, integration: &Integration) {
        let paths = self
            .trust_paths
            .clone()
            .or_else(gitpoulpi_config::ConfigPaths::discover);
        if let Some(paths) = paths {
            gitpoulpi_hosting::trust::forget(&paths, integration);
        }
        self.pinned.remove(&integration.id);
        self.rights.remove(&integration.id);
    }

    pub(super) fn disconnect(&mut self, env: &mut Env) {
        let Some(i) = self.selected_profile(env.config) else {
            return;
        };
        let (id, token_ref) = {
            let e = &env.config.integrations[i];
            (e.id.clone(), e.token_ref.clone())
        };
        let before = env.config.integrations[i].clone();
        match env.tokens.delete(&token_ref) {
            Ok(()) => {
                // Disconnecting forgets what was trusted for the profile too:
                // a later connection starts from nothing.
                self.forget_trust(&before);
                env.config.integrations[i].user = None;
                env.presence.insert(token_ref, Some(false));
                self.warnings.remove(&id);
                Self::write_config(env);
                *env.message = Some(msg::disconnected(&id));
                self.begin_form(env.config, Some(i));
            }
            Err(err) => {
                *env.message = Some(crate::activity_log::report_problem_clear(
                    crate::activity_log::ProblemSource::Save,
                    &msg::token_not_removed(err),
                ))
            }
        }
    }

    pub(super) fn remove(&mut self, env: &mut Env) {
        let Some(i) = self.selected_profile(env.config) else {
            return;
        };
        let removed = env.config.integrations.remove(i);
        // The token outlives the config entry unless it is removed here,
        // and an orphaned secret in a keychain can no longer be seen.
        let _ = env.tokens.delete(&removed.token_ref);
        self.forget_trust(&removed);
        env.presence.remove(&removed.token_ref);
        self.warnings.remove(&removed.id);
        Self::write_config(env);
        *env.message = Some(msg::removed(&removed.id));
        self.sel = self.sel.saturating_sub(1);
        self.reset_view(env.config);
        self.zone = Zone::Profiles;
    }

    fn spawn_ssh<F>(&mut self, env: &mut Env, what: M, work: F)
    where
        F: FnOnce(
                Tokens,
                Option<Arc<dyn HttpTransport>>,
                Integration,
            ) -> Result<SshOutcome, String>
            + Send
            + 'static,
    {
        if self.ssh.job.is_some() {
            return;
        }
        let Some(i) = self.selected_profile(env.config) else {
            return;
        };
        let integration = env.config.integrations[i].clone();
        let id = integration.id.clone();
        let (tx, rx) = mpsc::channel();
        let tokens = Arc::clone(env.tokens);
        let transport = self.transport.clone();
        std::thread::spawn(move || {
            let _ = tx.send_notify(work(tokens, transport, integration));
        });
        self.ssh.error = None;
        self.ssh.job = Some(SshJob { rx, id, what });
    }

    pub(super) fn start_generate(&mut self, env: &mut Env) {
        let Some(i) = self.selected_profile(env.config) else {
            return;
        };
        let host = env.config.integrations[i].host().unwrap_or_default();
        let title = match self.ssh.title.trim() {
            "" => gitpoulpi_hosting::ssh_key::profile_key_title(&host),
            t => t.to_string(),
        };
        let Some(ssh_dir) = self
            .ssh
            .ssh_dir
            .clone()
            .or_else(|| gitpoulpi_hosting::ssh_key::ssh_dir().ok())
        else {
            self.ssh.error = Some(msg::NO_SSH_HOME.get().into());
            return;
        };
        let upload = self.ssh.upload;
        self.spawn_ssh(
            env,
            msg::JOB_GENERATING,
            move |tokens, transport, integration| {
                ssh_io::generate(
                    &tokens,
                    transport,
                    GenerateRequest {
                        integration,
                        ssh_dir,
                        title,
                        upload,
                    },
                )
            },
        );
    }

    pub(super) fn start_remove_key(&mut self, env: &mut Env, delete_files: bool) {
        let what = if delete_files {
            msg::JOB_DELETING
        } else {
            msg::JOB_REMOVING
        };
        self.spawn_ssh(env, what, move |tokens, transport, integration| {
            ssh_io::remove(&tokens, transport, &integration, delete_files)
        });
    }

    pub(super) fn use_custom_key(&mut self, env: &mut Env) {
        let Some(i) = self.selected_profile(env.config) else {
            return;
        };
        match ssh_io::attach_custom(&self.ssh.path, gitpoulpi_config::home_dir().as_deref()) {
            Ok(ssh) => {
                *env.message = Some(msg::now_uses(
                    &env.config.integrations[i].id,
                    ssh.private_key_path.as_deref().unwrap_or_default(),
                ));
                env.config.integrations[i].ssh = Some(ssh);
                self.ssh.path.clear();
                self.ssh.error = None;
                Self::write_config(env);
            }
            Err(err) => self.ssh.error = Some(err),
        }
    }

    /// The Browse button of the local notes folder.
    pub(super) fn open_folder_picker(&mut self) {
        let home = gitpoulpi_config::home_dir();
        let typed = PathBuf::from(self.host.trim());
        let start = if typed.is_absolute() && typed.is_dir() {
            typed
        } else {
            home.clone().unwrap_or_else(|| PathBuf::from("."))
        };
        let picker = FolderPicker::new(
            gitpoulpi_i18n::msg::local_issues::PICKER_TITLE.get(),
            gitpoulpi_i18n::msg::local_issues::USE_THIS_FOLDER.get(),
            start,
        );
        self.ssh.lister.request_places(home, Vec::new());
        self.ssh.picking_folder = true;
        self.ssh.picker = Some(picker);
    }

    pub(super) fn open_key_picker(&mut self) {
        let home = gitpoulpi_config::home_dir();
        let start = home
            .as_ref()
            .map(|h| h.join(".ssh"))
            .filter(|p| p.is_dir())
            .or_else(|| home.clone())
            .unwrap_or_else(|| PathBuf::from("."));
        let mut picker = FolderPicker::new(msg::PICKER_TITLE.get(), msg::USE_THIS_KEY.get(), start);
        picker.set_file_mode();
        self.ssh.lister.request_places(home, Vec::new());
        self.ssh.picker = Some(picker);
    }

    /// Feeds the picker the folder listings it asked for and hands it the
    /// ones that arrived. Never waits.
    pub(super) fn pump_picker(&mut self) {
        let Some(picker) = self.ssh.picker.as_mut() else {
            return;
        };
        for event in self.ssh.lister.drain() {
            match event {
                ListerEvent::Listed(dir, listing) => picker.deliver(&dir, listing),
                ListerEvent::Places(places) => picker.set_places(places),
            }
        }
        for dir in picker.wants() {
            self.ssh.lister.request(dir);
        }
    }

    pub(super) fn picker_outcome(&mut self, outcome: PickerOutcome) {
        match outcome {
            PickerOutcome::Stay
            | PickerOutcome::ShowHiddenChanged(_)
            | PickerOutcome::ShowSiblingsChanged(_) => {}
            PickerOutcome::Cancel => {
                self.ssh.picker = None;
                self.ssh.picking_folder = false;
            }
            PickerOutcome::Choose(path) if self.ssh.picking_folder => {
                self.host = path.display().to_string();
                self.error = None;
                self.ssh.picker = None;
                self.ssh.picking_folder = false;
            }
            PickerOutcome::Choose(path) => {
                self.ssh.path = path.display().to_string();
                self.ssh.error = None;
                self.ssh.picker = None;
            }
            PickerOutcome::CreateFolder(_) => {
                if let Some(p) = self.ssh.picker.as_mut() {
                    p.folder_create_failed(msg::NO_FOLDER_CREATE.get());
                }
            }
        }
    }

    /// Applies a finished key job's answer, if it has arrived.
    fn poll_ssh(&mut self, env: &mut Env) {
        let Some(job) = &self.ssh.job else {
            return;
        };
        let answer = match job.rx.try_recv() {
            Ok(answer) => answer,
            Err(mpsc::TryRecvError::Empty) => return,
            Err(mpsc::TryRecvError::Disconnected) => Err(msg::KEY_JOB_ENDED.get().to_string()),
        };
        let Some(SshJob { id, .. }) = self.ssh.job.take() else {
            return;
        };
        match answer {
            Err(err) => {
                crate::activity_log::report_problem(
                    crate::activity_log::ProblemSource::Hosting,
                    &err,
                );
                self.ssh.error = Some(err);
            }
            Ok(outcome) => {
                let Some(at) = env.config.integrations.iter().position(|i| i.id == id) else {
                    return;
                };
                match outcome {
                    SshOutcome::Attached { ssh, note } => {
                        env.config.integrations[at].ssh = Some(ssh);
                        *env.message = Some(note);
                    }
                    SshOutcome::Detached { note } => {
                        env.config.integrations[at].ssh = None;
                        *env.message = Some(note);
                    }
                }
                self.ssh.mode = SshMode::Default;
                self.ssh.fingerprints.clear();
                Self::write_config(env);
                self.mode = Mode::Browse;
                self.item = 0;
            }
        }
    }
}

/// The background half of Connect: check the token, and only then keep it.
/// `gate` is where the check is trusted (the real store in the app; tests pass
/// one over a temporary private directory, never the user's).
pub(super) fn run_check(
    tokens: &Tokens,
    integration: Integration,
    typed: String,
    transport: Option<Arc<dyn HttpTransport>>,
    trust: TrustOptions,
    gate: &gitpoulpi_hosting::TrustGate,
) -> Result<TokenCheck, String> {
    // A folder of notes has no token and no host: nothing to check.
    if integration.kind.capabilities().local_folder {
        let user = gitpoulpi_hosting::HostedUser {
            login: integration
                .local
                .as_ref()
                .map(|l| l.me.clone())
                .filter(|m| !m.is_empty())
                .unwrap_or_else(|| integration.display_name().to_string()),
            ..Default::default()
        };
        return Ok(TokenCheck {
            user,
            ..Default::default()
        });
    }
    let secret = if typed.is_empty() {
        tokens
            .load(&integration.token_ref)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| msg::NO_TOKEN_STORED.get().to_string())?
    } else {
        typed.clone()
    };
    let transport: Arc<dyn HttpTransport> = match transport {
        Some(t) => t,
        None => Arc::new(gitpoulpi_hosting::ReqwestTransport::new().map_err(|e| e.to_string())?),
    };
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(msg::check_start_failed)?;
    // What the person asked of the security check in the form comes first: an
    // address changed together with a typed token, a plain http address
    // allowed. A refusal after that is queued for the dialog (the gate does it)
    // and is an ordinary error text here.
    if trust.trust_new_origin {
        let current = gitpoulpi_hosting::trust::integration_origin(&integration).ok();
        let recorded = gate.record_of(&integration.id).map(|r| r.origin);
        if recorded.is_some() && recorded != current {
            gate.trust_origin(&integration).map_err(|e| e.to_string())?;
        }
    }
    let was_allowed = gate
        .record_of(&integration.id)
        .is_some_and(|r| r.allow_plain_http);
    if trust.allow_plain_http != was_allowed {
        gate.set_allow_plain_http(&integration, trust.allow_plain_http)
            .map_err(|e| e.to_string())?;
    }
    let client = client_for(integration.clone(), secret.clone(), transport, gate)
        .map_err(|e| e.to_string())?;
    let check = runtime
        .block_on(client.check_token())
        .map_err(|e| e.to_string())?;
    // The token must belong to the account this integration is trusted for:
    // a typed token of another account is refused before it is stored.
    gate.check_account(
        &integration,
        &gitpoulpi_hosting::Account {
            login: check.user.login.clone(),
            id: check.user.id.clone(),
        },
    )
    .map_err(|e| e.to_string())?;
    apply_pinning(gate, &integration, trust.strict);
    if !typed.is_empty() {
        tokens
            .store(&integration.token_ref, &secret)
            .map_err(msg::token_not_stored)?;
    }
    Ok(check)
}

/// Applies the form's "Strict certificate pinning" toggle after a passing
/// check: switch an existing pin between first-use and strict, or pin what the
/// verifier saw. Without a certificate to read (a recorded transport, a plain
/// http address) there is nothing to pin.
fn apply_pinning(gate: &gitpoulpi_hosting::TrustGate, integration: &Integration, strict: bool) {
    use gitpoulpi_hosting::trust::{self, TlsMode};
    let pinned = gate.record_of(&integration.id).and_then(|r| r.tls);
    match pinned {
        Some(tls) if (tls.mode == TlsMode::Strict) != strict => {
            let _ = gate.set_strict(integration, strict);
        }
        None if strict => {
            let observed = trust::integration_origin(integration)
                .ok()
                .and_then(|o| trust::origin::host_of(&o))
                .and_then(|host| trust::tls::observed(&host));
            if let Some(cert) = observed {
                let _ = gate.trust_certificate(integration, &cert, TlsMode::Strict);
            }
        }
        _ => {}
    }
}
