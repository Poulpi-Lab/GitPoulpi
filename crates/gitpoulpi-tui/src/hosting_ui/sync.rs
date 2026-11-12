//! The `HostingUi` lifecycle: construction, the worker, refresh policy, polling and applying a snapshot.

use super::*;

impl HostingUi {
    /// The folded PULL REQUESTS groups, as `fold_state` keys.
    pub(crate) fn folded_groups(&self) -> Vec<String> {
        self.collapsed.iter().map(|g| g.key().to_string()).collect()
    }

    /// Restores the remembered folds; `None` keeps the default.
    pub(crate) fn restore_folded_groups(&mut self, keys: Option<Vec<String>>) {
        if let Some(keys) = keys {
            self.collapsed = PrGroup::ALL
                .into_iter()
                .filter(|g| keys.iter().any(|k| k == g.key()))
                .collect();
        }
    }

    /// The production wiring: real worker, real config, on-disk cache.
    pub(crate) fn new(repo_root: &Path, interval: Option<Duration>) -> Self {
        let state_dir = gitpoulpi_config::Settings::resolve_ambient()
            .map(|s| s.config().session.per_repo_state_dir.clone())
            .unwrap_or_else(|_| gitpoulpi_config::DEFAULT_REPO_DATA_LOCATION.to_string());
        let mut ui = Self::with_parts(
            repo_root,
            interval,
            Some(HostingWorker::spawn(Arc::new(DefaultFactory))),
            crate::pull_request::load_integrations,
            Some(cache_path(repo_root, &state_dir)),
        );
        ui.factory = Some(Arc::new(DefaultFactory));
        ui.load_cache();
        ui.defer_open = true;
        ui
    }

    pub(crate) fn with_parts(
        repo_root: &Path,
        interval: Option<Duration>,
        worker: Option<HostingWorker>,
        integrations: fn() -> Vec<Integration>,
        cache: Option<PathBuf>,
    ) -> Self {
        let state =
            gitpoulpi_session::RepoState::load(&crate::remotes_ui::repo_state_path(repo_root)).ok();
        let override_id = state.as_ref().and_then(|s| s.hosting_integration.clone());
        let issues_source = state.as_ref().and_then(|s| s.issues_source.clone());
        let pulls_source = state.and_then(|s| s.pulls_source);
        Self {
            repo_root: repo_root.to_path_buf(),
            worker,
            policy: RefreshPolicy::new(interval),
            integrations,
            override_id,
            issues_source,
            pulls_source,
            inputs: None,
            snapshot: None,
            fresh: false,
            pr_view: PrView::default(),
            issue_view: IssueView::default(),
            cache,
            menu: None,
            picker: None,
            pr_query: String::new(),
            issue_query: String::new(),
            editing: None,
            editing_query: false,
            query_draft: String::new(),
            lists: None,
            lists_rx: None,
            collapsed: HashSet::from([PrGroup::All]),
            row_rects: RefCell::new(Vec::new()),
            projects: Default::default(),
            projects_rx: Vec::new(),
            notice: None,
            last_rate_note: None,
            token_bases: RefCell::new(None),
            commit_references: Vec::new(),
            factory: None,
            profiles: Vec::new(),
            local_jobs: Vec::new(),
            local_watch: Default::default(),
            in_flight: false,
            opened_once: false,
            now: unix_now,
            in_flight_since: None,
            defer_open: false,
            open_gate: None,
        }
    }

    /// A worker for tests, over any factory.
    #[cfg(all(test, feature = "github", feature = "gitlab"))]
    pub(crate) fn spawn_worker(
        factory: Arc<dyn crate::hosting_task::ClientFactory>,
    ) -> HostingWorker {
        HostingWorker::spawn(factory)
    }

    #[cfg(all(test, feature = "github", feature = "gitlab"))]
    pub(crate) fn set_factory(&mut self, factory: Arc<dyn crate::hosting_task::ClientFactory>) {
        self.factory = Some(factory);
    }

    pub(super) fn load_cache(&mut self) {
        if let Some(snapshot) = self.cache.as_deref().and_then(load_cache) {
            // What was cached may predate an unlink: an unlinked section
            // shows nothing from it.
            let mut snapshot = snapshot;
            if self.pulls_source.is_none() && self.override_id.is_none() {
                snapshot.profile_id = None;
                snapshot.pulls_target = None;
                snapshot.prs.clear();
                snapshot.head_pr = None;
                snapshot.can_pulls = false;
            }
            if self.issues_source.is_none() {
                snapshot.issues_target = None;
                snapshot.issues.clear();
            }
            self.pr_view = snapshot.pr_view;
            self.issue_view = snapshot.issue_view;
            self.snapshot = Some(snapshot);
            self.fresh = false;
        }
    }

    // -- refresh ------------------------------------------------------------

    /// A repository refresh landed: decide whether the host needs asking.
    pub(crate) fn on_refresh(
        &mut self,
        now: Instant,
        remotes: &[Remote],
        refs: &[gitpoulpi_core::domain::Ref],
    ) {
        let head = refs.iter().find_map(|r| match &r.kind {
            RefKind::LocalBranch { name } if r.is_head => {
                Some((name.clone(), r.upstream.is_some()))
            }
            _ => None,
        });
        let inputs = Inputs {
            remotes: remotes
                .iter()
                .map(|r| ResolveRemote::new(r.name.clone(), r.fetch_url.clone()))
                .collect(),
            head_branch: head.as_ref().map(|(name, _)| name.clone()),
            head_pushed: head.as_ref().is_some_and(|(_, pushed)| *pushed),
        };
        let head_name = inputs.head_branch.clone();
        self.inputs = Some(inputs);
        let observed = self
            .policy
            .observe(head_name.as_deref(), refs_signature(refs));
        if self.policy.should_send(now, observed).is_some() {
            if self.defer_open && !self.opened_once {
                // The very first request of this window: after the first
                // graph paint, not before it.
                self.open_gate.get_or_insert(0);
            } else {
                self.send();
            }
        }
    }

    /// The section shows a spinner, or the open gate counts frames: the
    /// render loop must keep drawing at the frame rate.
    pub(crate) fn animating(&self) -> bool {
        self.in_flight || self.open_gate.is_some()
    }

    /// When the periodic refresh falls due, for the render loop to wake at.
    pub(crate) fn next_due(&self) -> Option<Instant> {
        let watch = self.local_watch.deadline();
        let refresh = self.inputs.as_ref().and_then(|_| self.policy.next_due());
        match (refresh, watch) {
            (Some(a), Some(b)) => Some(a.min(b)),
            (a, b) => a.or(b),
        }
    }

    /// Per-frame: the periodic timer and a pending demand.
    pub(crate) fn tick(&mut self, now: Instant) {
        // The query field lost the keyboard to a click elsewhere: keep what
        // was typed.
        if self.editing_query && self.editing.is_none() {
            self.finish_query_edit(true);
        }
        if let Some(ticks) = self.open_gate {
            let ticks = ticks + 1;
            if ticks >= OPEN_TICKS_BEFORE_REQUEST {
                self.send();
            } else {
                self.open_gate = Some(ticks);
            }
            return;
        }
        if self.inputs.is_some() && self.policy.should_send(now, None).is_some() {
            self.send();
        }
        self.tick_local(now);
    }

    /// The user asked for a refresh (palette, filter change, Preferences
    /// closing).
    pub(crate) fn demand(&mut self) {
        self.policy.demand();
        // The next frame's `tick` sends it.
        crate::redraw::poke();
    }

    pub(crate) fn set_interval(&mut self, interval: Option<Duration>) {
        self.policy.set_interval(interval);
    }

    fn send(&mut self) {
        let (Some(inputs), Some(worker)) = (&self.inputs, &self.worker) else {
            return;
        };
        self.open_gate = None;
        self.opened_once = true;
        let integrations = (self.integrations)();
        self.profiles = integrations
            .iter()
            .filter(|i| {
                i.enabled && i.kind.category() == gitpoulpi_config::IntegrationCategory::Hosting
            })
            .map(|i| {
                let label = if i.name.trim().is_empty() {
                    i.host().unwrap_or_else(|| i.id.clone())
                } else {
                    i.name.clone()
                };
                (i.id.clone(), label)
            })
            .collect();
        worker.request(HostingRequest {
            remotes: inputs.remotes.clone(),
            integrations: integrations.clone(),
            override_id: self.override_id.clone(),
            head_branch: inputs.head_branch.clone(),
            pr_view: self.pr_view,
            issue_view: self.issue_view,
            issues_source: crate::hosting_task::resolve_local_source(
                &self.repo_root,
                &integrations,
                self.issues_source.clone(),
            ),
            pulls_source: self.pulls_source.clone(),
        });
        if !self.in_flight {
            self.in_flight_since = Some(Instant::now());
        }
        self.in_flight = true;
    }

    /// Drains the workers. True when something on screen changed.
    pub(crate) fn poll(&mut self) -> bool {
        let mut changed = self.poll_local();
        if ISSUES_REFRESH.swap(false, std::sync::atomic::Ordering::SeqCst) {
            self.demand();
        }
        while let Some(snapshot) = self.worker.as_ref().and_then(HostingWorker::try_recv) {
            // A refresh answered `304 Not Modified` for everything comes
            // back identical: nothing to redraw.
            let was_fresh = self.fresh;
            let same = self.snapshot.as_ref().is_some_and(|old| {
                let mut probe = snapshot.clone();
                probe.fetched_at = old.fetched_at;
                *old == probe
            });
            self.apply(snapshot);
            changed |= !(same && was_fresh);
            let note = gitpoulpi_hosting::shared_rate_note();
            if note != self.last_rate_note {
                if note.is_some() {
                    self.notice = note.clone();
                }
                self.last_rate_note = note;
            }
        }
        let mut landed = Vec::new();
        self.projects_rx.retain(|rx| match rx.try_recv() {
            Some(answer) => {
                landed.push(answer);
                false
            }
            None => true,
        });
        if let Some((profile, board, rx)) = self.lists_rx.as_ref() {
            if let Some(answer) = rx.try_recv() {
                let names = match answer {
                    Ok(options) => Some(options.lists.into_iter().map(|c| c.label).collect()),
                    Err(why) => {
                        crate::activity_log::report_problem_as(
                            crate::activity_log::ProblemSource::Hosting,
                            "Hosting request: list board columns", // i18n-english
                            "",
                            &why,
                        );
                        None
                    }
                };
                self.lists = Some((profile.clone(), board.clone(), names));
                self.lists_rx = None;
                self.refresh_lists_picker();
                changed = true;
            }
        }
        for answer in landed {
            self.projects.insert(
                answer.profile_id.clone(),
                match answer.result {
                    Ok(list) => ProjectsState::Ready(list),
                    Err(why) => {
                        crate::activity_log::report_problem_as(
                            crate::activity_log::ProblemSource::Hosting,
                            "Hosting request: list projects", // i18n-english
                            &format!("profile={}", answer.profile_id),
                            &why,
                        );
                        ProjectsState::Failed(why)
                    }
                },
            );
            self.refresh_repo_picker();
            changed = true;
        }
        changed
    }

    /// Folds a worker answer into the current snapshot. A failed fetch
    /// keeps the previous lists (stale beats blank) and records the
    /// problem; a good one replaces everything and is cached.
    pub(crate) fn apply(&mut self, mut new: HostingSnapshot) {
        self.in_flight = false;
        self.in_flight_since = None;
        if new.problem.is_some() {
            if let Some(old) = self
                .snapshot
                .as_ref()
                .filter(|o| o.profile_id == new.profile_id)
            {
                if new.prs.is_empty() && old.pr_view == new.pr_view {
                    new.prs = old.prs.clone();
                }
                if new.head_pr.is_none() && old.head_branch == new.head_branch {
                    new.head_pr = old.head_pr.clone();
                }
                new.fetched_at = old.fetched_at;
            }
        }
        if new.issues_problem.is_some() {
            if let Some(old) = self
                .snapshot
                .as_ref()
                .filter(|o| o.issues_target == new.issues_target)
            {
                if new.issues.is_empty() && old.issue_view == new.issue_view {
                    new.issues = old.issues.clone();
                }
                new.fetched_at = old.fetched_at;
            }
        }
        if new.problem.is_none() && new.issues_problem.is_none() {
            if let Some(path) = &self.cache {
                save_cache(path, &new);
            }
        }
        self.fresh = true;
        if new.issues_problem.is_none() {
            bump_issues_generation();
        }
        self.snapshot = Some(new);
    }

    #[cfg(all(test, feature = "github", feature = "gitlab"))]
    pub(crate) fn snapshot(&self) -> Option<&HostingSnapshot> {
        self.snapshot.as_ref()
    }

    #[cfg(all(test, feature = "github", feature = "gitlab"))]
    pub(crate) fn set_clock(&mut self, now: fn() -> i64) {
        self.now = now;
    }
}
