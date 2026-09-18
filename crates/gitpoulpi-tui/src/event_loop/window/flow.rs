//! Flow commands at the window: the dropdown, the dialogs (seeded with the
//! window's branches, tags and issues), the grouping toggle and the
//! Settings page. Every surface (toolbar, palette, keys) reaches this
//! through `flow::request`, so none can drift.

use crate::event_loop::*;
use crate::flow::{FlowCmd, FlowKind, FlowSeed};
use init::WindowState;

impl WindowState {
    /// Keeps the workflow report fresh and runs a requested command.
    pub(in crate::event_loop) fn flow_tick(&mut self) {
        // Home has no repository to read a workflow from: the request would
        // go to the placeholder path and fail on every refs change.
        if self.repo.root != std::path::Path::new(crate::tabs::NO_REPOSITORY) {
            let requester = self.link.activity_handle().raw.requester();
            self.flow_source.tick(&requester, self.refs_gen);
        }
        if let Some(prefs) = self.preferences.as_mut() {
            prefs.set_workflow_view(crate::flow::view::current(), crate::flow::view::version());
            if let Some(path) = prefs.take_workflow_saved() {
                crate::flow::source::request_reload();
                self.last_message = Some(path);
            }
        }
        #[cfg(feature = "hosting")]
        self.issue_watch_tick();
        if let Some(cmd) = crate::flow::take_request() {
            self.run_flow_cmd(cmd);
        }
        self.open_flow_merge_request();
    }

    /// A merge-request finish pushed its branch: open the Create Pull
    /// Request form for it (target = the parent, title from the branch),
    /// or the compare URL when no integration answers for the remote.
    fn open_flow_merge_request(&mut self) {
        let Ok(refs) = self.refs_result.as_ref() else {
            return;
        };
        let pushed = |spec: &gitpoulpi_core::workflow::MergeRequestSpec| {
            refs.iter().any(|r| {
                matches!(&r.kind, RefKind::LocalBranch { name } if *name == spec.source)
                    && r.upstream.is_some()
            })
        };
        let Some(spec) = crate::flow::mr::take_when(|spec, waited| {
            pushed(spec) || waited > crate::flow::mr::WAIT
        }) else {
            return;
        };
        let mut seed = None;
        let said = crate::pull_request::start_pull_request_now(
            &spec.source,
            refs,
            &self.remote_ui.remotes,
            &mut seed,
        );
        #[cfg(feature = "hosting")]
        if let Some(mut seed) = seed {
            seed.target = spec.target.clone();
            self.repo_dialogs
                .open_pull_request(seed, &spec.title, self.ai_ui.enabled());
        }
        #[cfg(not(feature = "hosting"))]
        let _ = seed;
        self.last_message = Some(said);
    }

    /// Names of the local branches and of `remote`'s branches (without it).
    #[cfg(feature = "hosting")]
    fn flow_branch_names(&self, remote: &str) -> (Vec<String>, Vec<String>) {
        let (mut local, mut remote_names) = (Vec::new(), Vec::new());
        if let Ok(refs) = &self.refs_result {
            for r in refs {
                match &r.kind {
                    RefKind::LocalBranch { name } => local.push(name.clone()),
                    RefKind::RemoteBranch { remote: rm, name }
                        if rm == remote && name != "HEAD" =>
                    {
                        remote_names.push(name.clone())
                    }
                    _ => {}
                }
            }
        }
        (local, remote_names)
    }

    /// The remote host-driven flows fetch from and track: `origin`, else the
    /// only (first) one.
    #[cfg(feature = "hosting")]
    fn flow_remote(&self) -> String {
        if self.remote_names.iter().any(|r| r == "origin") {
            return "origin".to_string();
        }
        self.remote_names.first().cloned().unwrap_or_default()
    }

    /// Looks for the branch of a watched issue after every change of the refs
    /// (a fetch), and offers it once no dialog is in the way.
    #[cfg(feature = "hosting")]
    fn issue_watch_tick(&mut self) {
        let can_show = !self.repo_dialogs.is_open();
        let remote = self.flow_remote();
        let (local, remote_branches) = self.flow_branch_names(&remote);
        let state_path = crate::remotes_ui::repo_state_path(&self.repo_root);
        let appeared = self.issue_watch.tick(self.refs_gen, can_show, || {
            let view = crate::flow::view::configured()?;
            Some((
                view.report.workflow.clone(),
                state_path,
                remote_branches,
                local,
            ))
        });
        if let Some(a) = appeared {
            self.open_issue_flow(crate::flow::issue_flow::IssueFlowKind::Appeared {
                number: a.number,
                branches: a.branches,
            });
        }
    }

    /// The tracker and code host the flows talk to, from the repository's
    /// ISSUES and PULL REQUESTS links.
    #[cfg(feature = "hosting")]
    fn flow_targets(
        &self,
    ) -> (
        Option<crate::flow::issue_flow::Target>,
        Option<crate::flow::issue_flow::Target>,
        Vec<gitpoulpi_hosting::Integration>,
    ) {
        use crate::flow::issue_flow::Target;
        let integrations = crate::pull_request::load_integrations();
        let state = gitpoulpi_session::RepoState::load(&crate::remotes_ui::repo_state_path(
            &self.repo_root,
        ))
        .ok();
        let target = |source: Option<&gitpoulpi_session::IssuesSource>| {
            let s = source.filter(|s| !s.project.is_empty())?;
            let integration = integrations.iter().find(|i| i.id == s.integration)?.clone();
            Some(Target {
                integration,
                project: s.project.clone(),
            })
        };
        let issue = target(state.as_ref().and_then(|s| s.issues_source.as_ref()));
        let code =
            target(state.as_ref().and_then(|s| s.pulls_source.as_ref())).or_else(|| issue.clone());
        (issue, code, integrations)
    }

    /// Opens New issue / Pick up / the appeared card.
    #[cfg(feature = "hosting")]
    pub(in crate::event_loop) fn open_issue_flow(
        &mut self,
        kind: crate::flow::issue_flow::IssueFlowKind,
    ) {
        use crate::flow::issue_flow::{DaemonPickUp, IssueFlowKind, IssueFlowSeed};
        let Some(view) = crate::flow::view::configured() else {
            self.last_message = Some(
                gitpoulpi_i18n::msg::flow::WHY_NOT_CONFIGURED
                    .get()
                    .to_string(),
            );
            return;
        };
        let (issue_target, code_target, integrations) = self.flow_targets();
        if matches!(kind, IssueFlowKind::New(_)) && issue_target.is_none() {
            self.last_message = Some(
                gitpoulpi_i18n::msg::flow::ISSUE_WHY_NO_TRACKER
                    .get()
                    .to_string(),
            );
            return;
        }
        let remote = self.flow_remote();
        let (_, remote_branches) = self.flow_branch_names(&remote);
        let requester = self.link.activity_handle().raw.requester();
        self.repo_dialogs.open_issue_flow(IssueFlowSeed {
            view,
            kind,
            issue_target,
            code_target,
            integrations,
            factory: std::sync::Arc::new(crate::hosting_task::DefaultFactory),
            picker: std::sync::Arc::new(DaemonPickUp { requester }),
            remote,
            remote_branches,
            repo_root: self.repo_root.clone(),
            state_path: crate::remotes_ui::repo_state_path(&self.repo_root),
            user: crate::live_settings::active_profile_author().map(|a| a.name),
        });
    }

    fn selected_local_branch(&self) -> Option<String> {
        if self.active_ref_section != SidebarSection::Local {
            return None;
        }
        selected_ref_leaf(
            &self.refs_result,
            &self.refs_filter,
            self.active_ref_section,
            self.sidebar_states.local.selected(),
            self.hierarchical_refs,
            &self.collapsed_ref_folders,
            &self.remote_names,
        )
    }

    fn flow_seed(&self, kind: FlowKind) -> Option<FlowSeed> {
        let view = crate::flow::view::configured()?;
        let refs = self.refs_result.as_ref().ok()?;
        let mut branches = Vec::new();
        let mut tags = Vec::new();
        for r in refs {
            match &r.kind {
                RefKind::LocalBranch { name } => branches.push(name.clone()),
                RefKind::Tag { name } => tags.push(name.clone()),
                _ => {}
            }
        }
        branches.sort_unstable();
        Some(FlowSeed {
            view,
            kind,
            current: checked_out_branch_name(refs).map(str::to_string),
            selected: self.selected_local_branch(),
            branches,
            tags,
            issues: self.hosting.issue_choices(),
            user: crate::live_settings::active_profile_author().map(|a| a.name),
        })
    }

    pub(in crate::event_loop) fn run_flow_cmd(&mut self, cmd: FlowCmd) {
        if let Some(why) = cmd.unavailable() {
            self.last_message = Some(why.to_string());
            return;
        }
        let kind = match cmd {
            FlowCmd::Menu => {
                let (col, row) = self.remote_ui.flow_anchor();
                self.remote_ui.open_flow_menu(col, row);
                return;
            }
            FlowCmd::ToggleGrouping => {
                let plain = !crate::flow::view::plain();
                crate::flow::view::set_plain(plain);
                self.last_message = Some(if plain {
                    gitpoulpi_i18n::msg::flow::MENU_GROUP_ON.get().to_string()
                } else {
                    gitpoulpi_i18n::msg::flow::MENU_GROUP_OFF.get().to_string()
                });
                return;
            }
            FlowCmd::Settings => {
                let mut state = PreferencesState::open(&self.repo_root);
                state.show_workflow();
                self.preferences = Some(state);
                return;
            }
            FlowCmd::NewIssueAny | FlowCmd::NewIssue(_) | FlowCmd::PickUp => {
                #[cfg(feature = "hosting")]
                {
                    use crate::flow::issue_flow::IssueFlowKind;
                    self.open_issue_flow(match cmd {
                        FlowCmd::NewIssue(i) => IssueFlowKind::New(Some(usize::from(i))),
                        FlowCmd::NewIssueAny => IssueFlowKind::New(None),
                        _ => IssueFlowKind::PickUp,
                    });
                }
                return;
            }
            FlowCmd::StartAny => FlowKind::Start(None),
            FlowCmd::Start(i) => FlowKind::Start(Some(usize::from(i))),
            FlowCmd::Finish => FlowKind::Finish,
            FlowCmd::Sync => FlowKind::Sync,
            FlowCmd::Promote => FlowKind::Promote,
            FlowCmd::SetParent => FlowKind::SetParent,
            FlowCmd::Resume => FlowKind::Resume,
        };
        match self.flow_seed(kind) {
            Some(seed) => {
                let requester = self.link.activity_handle().raw.requester();
                self.repo_dialogs
                    .open_flow(seed, crate::flow::default_actions(requester))
            }
            None => {
                self.last_message = Some(
                    gitpoulpi_i18n::msg::flow::WHY_NOT_CONFIGURED
                        .get()
                        .to_string(),
                )
            }
        }
    }
}
