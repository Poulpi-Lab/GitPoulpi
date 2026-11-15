use super::*;

/// How long after a click closed a combo popup the area it covered ignores
/// presses (a double click's second one).
const POPUP_SHIELD: std::time::Duration = std::time::Duration::from_millis(600);

/// The PR form's AI button label (the sparkle is prefixed when drawn).
pub(crate) const GENERATE_LABEL: gitpoulpi_i18n::M = msg::pull_request::GENERATE;

/// What the form says while the host's repositories are being fetched.
pub(crate) const LOADING_TEXT: gitpoulpi_i18n::M = msg::pull_request::LOADING;

/// One frame of the loading spinner, picked from the wall clock.
pub(super) fn spinner_glyph() -> char {
    const FRAMES: [char; 8] = [
        '\u{280B}', '\u{2819}', '\u{2839}', '\u{2838}', '\u{283C}', '\u{2834}', '\u{2826}',
        '\u{2827}',
    ];
    let ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_millis());
    FRAMES[(ms / 100 % FRAMES.len() as u128) as usize]
}

/// Whether a fresh form offers the AI `Generate title and description`
/// button. Off: the button exists only while `ai.enabled`, which
/// `RepoDialogs::open_pull_request` passes to
/// [`PullRequestDialog::set_ai_generate`].
pub(crate) const AI_GENERATE_HOOK: bool = false;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PrFocus {
    Tabs,
    FromRepo,
    FromBranch,
    ToRepo,
    ToBranch,
    Title,
    Body,
    /// The metadata buttons (GitHub, GitLab).
    MetaAssignees,
    MetaReviewers,
    MetaLabels,
    MetaMilestone,
    Draft,
    Generate,
    Web,
    Create,
    Cancel,
}

impl PrFocus {
    pub(super) fn meta_field(self) -> Option<MetaField> {
        match self {
            PrFocus::MetaAssignees => Some(MetaField::Assignees),
            PrFocus::MetaReviewers => Some(MetaField::Reviewers),
            PrFocus::MetaLabels => Some(MetaField::Labels),
            PrFocus::MetaMilestone => Some(MetaField::Milestone),
            _ => None,
        }
    }

    pub(super) fn of_meta(field: MetaField) -> Self {
        match field {
            MetaField::Assignees => PrFocus::MetaAssignees,
            MetaField::Reviewers => PrFocus::MetaReviewers,
            MetaField::Labels => PrFocus::MetaLabels,
            MetaField::Milestone => PrFocus::MetaMilestone,
        }
    }
}

/// A click target of the form.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PrHit {
    Tab(usize),
    Part(PrFocus),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(clippy::large_enum_variant)] // short-lived, one per submit
pub(crate) enum PullRequestOutcome {
    Stay,
    Cancel,
    /// "Create on GitHub / GitLab": open the provider's own prefilled
    /// form (the URL) in the browser; the dialog stays open.
    OpenWeb(String),
    Submit {
        integration: Integration,
        req: Box<NewPullRequest>,
    },
    /// The (future) AI button: ask for a title and description for this
    /// change, then hand them to [`PullRequestDialog::set_generated`].
    Generate(Box<NewPullRequest>),
}

pub(crate) struct PullRequestDialog {
    pub(crate) seed: PullRequestSeed,
    pub(crate) tab: usize,
    pub(super) from_repo: Combo,
    pub(super) from_branch: Combo,
    pub(super) to_repo: Combo,
    pub(super) to_branch: Combo,
    pub(crate) title: String,
    /// The description: the shared Markdown editor.
    pub(crate) md: crate::md_editor::MdEditor,
    pub(crate) draft: bool,
    pub(crate) focus: PrFocus,
    pub(crate) message: Option<String>,
    pub(crate) busy: bool,
    pub(crate) hover: Option<PrHit>,
    pub(super) hits: Vec<(Rect, PrHit)>,
    pub(super) edit: crate::text_edit::FieldEditor<PrFocus>,
    /// Each provider's project list, fetched the first time its tab is
    /// shown.
    pub(super) projects: std::collections::HashMap<String, ProjectsState>,
    pub(super) projects_rx: Vec<ProjectsReceiver>,
    pub(super) factory: Option<std::sync::Arc<dyn ClientFactory>>,
    pub(super) ai_generate: bool,
    /// The chosen `(source, target)` branches as the description's "Write
    /// with AI" help reads them.
    pub(super) ai_branches: Option<crate::md_editor::ai::Shared<(String, String)>>,
    /// A generation is running: the button says so and does nothing.
    pub(super) generating: bool,
    /// Where a popup was when a click closed it. A popup lies over the
    /// form's other controls (the Generate button among them), so the
    /// rest of that gesture - a double click's second press - must not
    /// reach what is underneath: it is swallowed for a double-click
    /// interval, or until the pointer leaves.
    pub(super) popup_shield: Option<(Rect, std::time::Instant)>,
    /// Assignees, reviewers, labels and milestone, chosen in the picker.
    pub(crate) meta: MetaFields,
    /// The Title as prefilled: only a change to it counts as typed work.
    initial_title: String,
    pub(super) guard: crate::dialog_kit::DiscardGuard,
}

impl PullRequestDialog {
    /// What `Ctrl+N` badges: the tabs, fields, pickers and buttons; nothing while a list is open.
    pub(crate) fn nav_targets(&self) -> Vec<(Rect, (u16, u16))> {
        if self.open_combo().is_some() || self.meta.is_open() {
            return Vec::new();
        }
        dialog_kit::nav_cells(self.hits.iter().map(|(r, _)| *r).chain(self.md.nav_rects()))
    }

    /// Gives the description its "Write with AI" help (only while AI is on):
    /// the branch pair the form holds is what it describes.
    pub(crate) fn bind_ai(&mut self, scope: crate::md_editor::ai::AiScope) {
        let branches = crate::md_editor::ai::Shared::default();
        self.md.bind_ai(
            scope,
            Box::new(crate::md_editor::ai::PullRequestSource {
                branches: branches.clone(),
            }),
        );
        self.ai_branches = Some(branches);
    }

    /// Tells the description's AI help which branches are chosen now.
    pub(super) fn sync_ai_branches(&self) {
        let Some(cell) = &self.ai_branches else {
            return;
        };
        let pick = |c: &Combo| {
            c.selected()
                .map(|s| s.trim().to_string())
                .unwrap_or_default()
        };
        if let Ok(mut pair) = cell.lock() {
            *pair = (pick(&self.from_branch), pick(&self.to_branch));
        }
    }

    /// `tip_subject` prefills Title with the source branch's tip
    /// commit's subject line, per the plan.
    pub(crate) fn new(seed: PullRequestSeed, tip_subject: &str) -> Self {
        Self::with_factory(
            seed,
            tip_subject,
            std::sync::Arc::new(crate::hosting_task::DefaultFactory),
        )
    }

    /// [`new`](Self::new) over any client factory (the provider project
    /// lists come from it).
    pub(crate) fn with_factory(
        seed: PullRequestSeed,
        tip_subject: &str,
        factory: std::sync::Arc<dyn ClientFactory>,
    ) -> Self {
        let title = if tip_subject.is_empty() {
            seed.local_branch.clone()
        } else {
            tip_subject.to_string()
        };
        let mut dialog = Self {
            tab: seed.tab.min(seed.tabs.len().saturating_sub(1)),
            from_repo: Combo::new(msg::pull_request::SELECT.get()).with_custom(),
            from_branch: Combo::new(msg::pull_request::SELECT.get()).with_custom(),
            to_repo: Combo::new(msg::pull_request::SELECT.get()).with_custom(),
            to_branch: Combo::new(msg::pull_request::SELECT.get()).with_custom(),
            title: title.clone(),
            md: crate::md_editor::MdEditor::new(String::new(), Default::default()).with_draft(
                crate::md_editor::draft::key(&[
                    seed.tabs
                        .get(seed.tab)
                        .map_or("", |t| t.integration.id.as_str()),
                    seed.project.as_deref().unwrap_or(""),
                    &seed.local_branch,
                    "new-pr",
                ]),
            ),
            draft: false,
            focus: PrFocus::Title,
            message: None,
            busy: false,
            hover: None,
            hits: Vec::new(),
            edit: Default::default(),
            projects: Default::default(),
            projects_rx: Vec::new(),
            factory: Some(factory),
            ai_generate: AI_GENERATE_HOOK,
            ai_branches: None,
            generating: false,
            popup_shield: None,
            meta: MetaFields::default(),
            initial_title: title.clone(),
            guard: Default::default(),
            seed,
        };
        dialog.reset_choices();
        dialog
    }

    /// The branch dropdowns' options and the repositories' defaults for
    /// the current tab.
    fn reset_choices(&mut self) {
        let branches: Vec<ComboOption> = self
            .seed
            .branches
            .iter()
            .map(|b| ComboOption::plain(b.clone()))
            .collect();
        self.from_branch.set_options(branches.clone());
        self.to_branch.set_options(branches);
        self.from_branch.select(self.seed.source_branch.clone());
        self.to_branch.select(Some(self.seed.target.clone()));
        // The repositories default to the remote's, on the tab that
        // answers for it; another tab starts empty.
        let on_home_tab = self.tab == self.seed.tab;
        let home = self.seed.project.clone().filter(|_| on_home_tab);
        self.from_repo.select(home.clone());
        self.to_repo.select(home);
        self.ensure_projects();
        self.refresh_repo_options();
    }

    pub(super) fn current_tab(&self) -> Option<&PrTab> {
        self.seed.tabs.get(self.tab)
    }

    /// What `#N` and `!N` in the description mean: the provider of the
    /// active tab, the repository the request goes to.
    pub(super) fn md_context(&self) -> crate::md_render::Context {
        let Some(tab) = self.current_tab() else {
            return Default::default();
        };
        let project = self
            .to_repo
            .selected()
            .map(str::to_string)
            .or_else(|| self.seed.project.clone())
            .unwrap_or_default();
        crate::md_render::Context {
            xrefs: gitpoulpi_hosting::xrefs::XrefContext {
                family: Some(tab.integration.kind.family()),
                web_base: gitpoulpi_hosting::integration_web_base(&tab.integration)
                    .unwrap_or_default(),
                project,
                ..Default::default()
            },
            ..Default::default()
        }
    }

    fn ensure_projects(&mut self) {
        let Some(tab) = self.current_tab().cloned() else {
            return;
        };
        let id = tab.integration.id.clone();
        if matches!(
            self.projects.get(&id),
            Some(ProjectsState::Loading | ProjectsState::Ready(_))
        ) {
            return;
        }
        let Some(factory) = self.factory.clone() else {
            return;
        };
        self.projects.insert(id, ProjectsState::Loading);
        let all: Vec<Integration> = self
            .seed
            .tabs
            .iter()
            .map(|t| t.integration.clone())
            .collect();
        self.projects_rx
            .push(start_projects(factory, tab.integration, all, None));
    }

    fn refresh_repo_options(&mut self) {
        let Some(id) = self.current_tab().map(|t| t.integration.id.clone()) else {
            return;
        };
        let mut names: Vec<String> = match self.projects.get(&id) {
            Some(ProjectsState::Ready(list)) => list.iter().map(|p| p.full_name.clone()).collect(),
            _ => Vec::new(),
        };
        if let Some(home) = self
            .seed
            .project
            .as_ref()
            .filter(|_| self.tab == self.seed.tab)
        {
            if !names.contains(home) {
                names.insert(0, home.clone());
            }
        }
        let loading = matches!(self.projects.get(&id), Some(ProjectsState::Loading));
        for combo in [&mut self.from_repo, &mut self.to_repo] {
            combo.set_options(
                names
                    .iter()
                    .map(|n| ComboOption::plain(n.clone()))
                    .collect(),
            );
            combo.loading = loading;
        }
    }

    /// Whether the current tab's provider takes assignees, reviewers,
    /// labels and a milestone (GitHub, GitLab).
    pub(super) fn uses_meta(&self) -> bool {
        self.current_tab()
            .is_some_and(|t| meta_picker::supports(t.integration.kind))
    }

    /// Whether the target repository blocks assignment: the assignee picker
    /// is left out and nothing is assigned.
    pub(super) fn assignment_blocked(&self) -> bool {
        self.current_tab().is_some_and(|t| {
            gitpoulpi_hosting::scope::assignment_blocked(
                &t.integration.id,
                self.to_repo.selected().unwrap_or(""),
            )
        })
    }

    /// What the metadata choices were made for: the provider tab and the
    /// target repository. A form that moves on starts them over.
    fn meta_scope(&self) -> String {
        format!(
            "{}|{}",
            self.current_tab().map_or("", |t| t.integration.id.as_str()),
            self.to_repo.selected().unwrap_or("")
        )
    }

    fn meta_scope_is_current(&self) -> bool {
        self.meta.scope_is(&self.meta_scope())
    }

    pub(super) fn sync_meta_scope(&mut self) {
        let scope = self.meta_scope();
        self.meta.set_scope(scope);
    }

    fn open_meta(&mut self, field: MetaField) {
        self.sync_meta_scope();
        let (Some(factory), Some(tab)) = (self.factory.clone(), self.current_tab().cloned()) else {
            return;
        };
        let Some(project) = self
            .to_repo
            .selected()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
        else {
            self.message = Some(msg::meta_picker::PICK_REPO_FIRST.get().to_string());
            return;
        };
        let all: Vec<Integration> = self
            .seed
            .tabs
            .iter()
            .map(|t| t.integration.clone())
            .collect();
        let target = MetaTarget {
            integration: tab.integration,
            all,
            project,
            kind: gitpoulpi_hosting::MetaKind::PullRequest,
            number: None,
            caption: msg::meta_picker::CREATE_CAPTION.get().to_string(),
        };
        self.meta.open(factory, target, field);
    }

    /// A picker is waiting on the host (the spinner moves).
    pub(crate) fn animating(&self) -> bool {
        self.meta.busy()
    }

    /// Whether the current tab's repository list is still being fetched.
    pub(crate) fn is_loading(&self) -> bool {
        self.current_tab().is_some_and(|t| {
            matches!(
                self.projects.get(&t.integration.id),
                Some(ProjectsState::Loading)
            )
        })
    }

    /// Drains the project-list fetches. True when the form changed.
    pub(crate) fn poll(&mut self) -> bool {
        let picked = self.meta.poll();
        let mut landed = Vec::new();
        self.projects_rx.retain(|rx| match rx.try_recv() {
            Some(answer) => {
                landed.push(answer);
                false
            }
            None => true,
        });
        let changed = !landed.is_empty();
        for answer in landed {
            match answer.result {
                Ok(list) => {
                    self.projects
                        .insert(answer.profile_id, ProjectsState::Ready(list));
                }
                Err(why) => {
                    crate::activity_log::report_problem(
                        crate::activity_log::ProblemSource::Hosting,
                        &why,
                    );
                    self.message = Some(msg::pull_request::repos_list_failed(&why));
                    self.projects
                        .insert(answer.profile_id, ProjectsState::Failed(why));
                }
            }
        }
        if changed {
            self.refresh_repo_options();
        }
        changed || picked
    }

    fn select_tab(&mut self, index: usize) {
        if index >= self.seed.tabs.len() || index == self.tab {
            return;
        }
        self.tab = index;
        self.message = None;
        self.reset_choices();
    }

    /// Fills Title and Description (the AI generator's answer).
    pub(crate) fn set_generated(&mut self, title: String, body: String) {
        self.title = title;
        self.md.set_text(body);
        self.message = None;
        self.generating = false;
    }

    /// The generator failed or was canceled: say why, keep the fields.
    pub(crate) fn generation_failed(&mut self, why: String) {
        self.message = Some(why);
        self.generating = false;
    }

    /// Shows or hides the `Generate title and description` button
    /// (`ai.enabled`): hidden, not grayed out, while AI is off.
    pub(crate) fn set_ai_generate(&mut self, on: bool) {
        self.ai_generate = on;
        if !on && self.focus == PrFocus::Generate {
            self.focus = PrFocus::Title;
        }
    }

    pub(super) fn plan(&self) -> Result<NewPullRequest, String> {
        self.plan_inner(self.title.trim())
    }

    fn plan_inner(&self, title: &str) -> Result<NewPullRequest, String> {
        let choice = |combo: &Combo, why: gitpoulpi_i18n::M| -> Result<String, String> {
            combo
                .selected()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .ok_or_else(|| why.get().to_string())
        };
        // Only what was chosen for this provider and repository.
        let meta = if self.uses_meta() && self.meta_scope_is_current() {
            self.meta.selection.clone()
        } else {
            MetaSelection::default()
        };
        let to_repo = choice(&self.to_repo, msg::pull_request::PICK_TO_REPO)?;
        let from_repo = choice(&self.from_repo, msg::pull_request::PICK_FROM_REPO)?;
        let from_branch = choice(&self.from_branch, msg::pull_request::PICK_FROM_BRANCH)?;
        let to_branch = choice(&self.to_branch, msg::pull_request::PICK_TO_BRANCH)?;
        if title.is_empty() {
            return Err(msg::pull_request::PICK_TITLE.get().into());
        }
        if from_repo.eq_ignore_ascii_case(&to_repo) && from_branch == to_branch {
            return Err(msg::pull_request::SAME_BRANCHES.get().into());
        }
        Ok(NewPullRequest {
            project: to_repo.clone(),
            source_project: (!from_repo.eq_ignore_ascii_case(&to_repo)).then_some(from_repo),
            source_branch: from_branch,
            target_branch: to_branch,
            title: title.to_string(),
            body: self.md.text().trim().to_string(),
            draft: self.draft,
            assignees: if self.assignment_blocked() {
                Vec::new()
            } else {
                meta.assignees.clone()
            },
            reviewers: meta.reviewers.clone(),
            labels: meta.labels.clone(),
            milestone: meta.milestone.as_ref().map(|m| m.id),
        })
    }

    /// The order Tab walks the form in.
    pub(super) fn order(&self) -> Vec<PrFocus> {
        let mut order = vec![
            PrFocus::Tabs,
            PrFocus::FromRepo,
            PrFocus::FromBranch,
            PrFocus::ToRepo,
            PrFocus::ToBranch,
            PrFocus::Title,
        ];
        if self.ai_generate {
            order.push(PrFocus::Generate);
        }
        order.push(PrFocus::Body);
        if self.uses_meta() {
            if !self.assignment_blocked() {
                order.push(PrFocus::MetaAssignees);
            }
            order.extend([
                PrFocus::MetaReviewers,
                PrFocus::MetaLabels,
                PrFocus::MetaMilestone,
            ]);
        }
        order.extend([
            PrFocus::Draft,
            PrFocus::Web,
            PrFocus::Create,
            PrFocus::Cancel,
        ]);
        order
    }

    pub(super) fn combo_mut(&mut self, focus: PrFocus) -> Option<&mut Combo> {
        match focus {
            PrFocus::FromRepo => Some(&mut self.from_repo),
            PrFocus::FromBranch => Some(&mut self.from_branch),
            PrFocus::ToRepo => Some(&mut self.to_repo),
            PrFocus::ToBranch => Some(&mut self.to_branch),
            _ => None,
        }
    }

    /// The combobox whose popup is open, if any (modal).
    fn open_combo(&self) -> Option<PrFocus> {
        [
            PrFocus::FromRepo,
            PrFocus::FromBranch,
            PrFocus::ToRepo,
            PrFocus::ToBranch,
        ]
        .into_iter()
        .find(|f| match f {
            PrFocus::FromRepo => self.from_repo.is_open(),
            PrFocus::FromBranch => self.from_branch.is_open(),
            PrFocus::ToRepo => self.to_repo.is_open(),
            _ => self.to_branch.is_open(),
        })
    }

    fn submit(&mut self) -> PullRequestOutcome {
        if self.busy {
            return PullRequestOutcome::Stay;
        }
        match self.plan() {
            Ok(req) => match self.current_tab() {
                Some(tab) => PullRequestOutcome::Submit {
                    integration: tab.integration.clone(),
                    req: Box::new(req),
                },
                None => PullRequestOutcome::Stay,
            },
            Err(reason) => {
                self.message = Some(reason);
                PullRequestOutcome::Stay
            }
        }
    }

    /// The Title was edited or a Description typed.
    fn typed(&self) -> bool {
        self.title != self.initial_title || !self.md.is_blank()
    }

    /// The cross, Esc, Cancel and a click outside all ask the one rule.
    fn request_close(&mut self, kind: dialog_kit::DismissKind) -> PullRequestOutcome {
        use dialog_kit::{dismiss, DismissAction};
        match dismiss(&*self, kind) {
            DismissAction::Close => PullRequestOutcome::Cancel,
            DismissAction::Swallow => {
                self.guard.close.request_flash();
                PullRequestOutcome::Stay
            }
            DismissAction::Confirm => {
                self.guard.ask();
                PullRequestOutcome::Stay
            }
        }
    }

    fn verdict(&self, v: dialog_kit::Verdict) -> PullRequestOutcome {
        match v {
            dialog_kit::Verdict::Discard => PullRequestOutcome::Cancel,
            dialog_kit::Verdict::Stay => PullRequestOutcome::Stay,
        }
    }

    fn activate(&mut self, part: PrFocus) -> PullRequestOutcome {
        match part {
            PrFocus::Cancel => self.request_close(dialog_kit::DismissKind::Cancel),
            PrFocus::Draft => {
                self.draft = !self.draft;
                PullRequestOutcome::Stay
            }
            PrFocus::Generate if self.generating => PullRequestOutcome::Stay,
            PrFocus::Generate => match self.plan_for_generation() {
                Some(req) => {
                    self.generating = true;
                    PullRequestOutcome::Generate(Box::new(req))
                }
                None => PullRequestOutcome::Stay,
            },
            PrFocus::Create => self.submit(),
            PrFocus::Web => self.open_web(),
            other => {
                if let Some(field) = other.meta_field() {
                    self.open_meta(field);
                }
                PullRequestOutcome::Stay
            }
        }
    }

    /// The provider's name (`GitHub` / `GitLab`) of the current tab.
    pub(crate) fn provider_name(&self) -> &'static str {
        self.current_tab()
            .map_or("", |t| crate::hosting_ui::provider_name(t.integration.kind))
    }

    /// The provider's own new pull / merge request form for the chosen
    /// repositories and branches, prefilled with the title and description
    /// as typed. Needs no token and ignores the title being empty.
    pub(crate) fn web_url(&self) -> Result<String, String> {
        let typed = self.title.trim();
        // `plan` insists on a title; the web form does not.
        let mut req = self.plan_inner(if typed.is_empty() { "-" } else { typed })?;
        if typed.is_empty() {
            req.title.clear();
        }
        let integration = &self.current_tab().ok_or("no provider")?.integration;
        let base = gitpoulpi_hosting::integration_web_base(integration)
            .ok_or_else(|| msg::issue_form::NO_WEB.get().to_string())?;
        Ok(gitpoulpi_hosting::new_pull_request_form_url(
            integration.kind,
            &base,
            &gitpoulpi_hosting::PullRequestForm {
                project: &req.project,
                source_project: req.source_project.as_deref(),
                source_branch: &req.source_branch,
                target_branch: Some(&req.target_branch),
                title: &req.title,
                body: &req.body,
                draft: req.draft,
            },
        ))
    }

    fn open_web(&mut self) -> PullRequestOutcome {
        match self.web_url() {
            Ok(url) => PullRequestOutcome::OpenWeb(url),
            Err(reason) => {
                self.message = Some(reason);
                PullRequestOutcome::Stay
            }
        }
    }

    /// The change the generator should describe: the repositories and
    /// branches, whatever the text fields hold yet.
    fn plan_for_generation(&mut self) -> Option<NewPullRequest> {
        let saved = std::mem::replace(&mut self.title, "-".into());
        let plan = self.plan();
        self.title = saved;
        match plan {
            Ok(req) => Some(req),
            Err(reason) => {
                self.message = Some(reason);
                None
            }
        }
    }

    pub(crate) fn paste(&mut self, text: &str) {
        if self.meta.paste(text) {
            return;
        }
        if let Some(open) = self.open_combo() {
            if let Some(c) = self.combo_mut(open) {
                c.paste(text);
            }
            return;
        }
        match self.focus {
            PrFocus::Title => {
                let clean: String = text.chars().filter(|c| !c.is_control()).collect();
                self.edit.sync(Some(PrFocus::Title));
                self.edit
                    .caret_mut(PrFocus::Title)
                    .insert(&mut self.title, &clean);
            }
            PrFocus::Body => self.md.paste(text),
            _ => {}
        }
    }

    pub(crate) fn on_key(&mut self, key: KeyEvent) -> PullRequestOutcome {
        let shift = key.modifiers.contains(KeyModifiers::SHIFT);
        if let Some(v) = self.guard.key(key) {
            return self.verdict(v);
        }
        // An open picker is modal: it takes every key.
        if self.meta.is_open() {
            self.meta.on_key(key);
            return PullRequestOutcome::Stay;
        }
        // An open popup is modal: it takes every key.
        if let Some(open) = self.open_combo() {
            if let Some(c) = self.combo_mut(open) {
                c.on_key(key);
            }
            self.message = None;
            return PullRequestOutcome::Stay;
        }
        if crate::keymap::current().is(Bind::DialogWeb, &key)
            && !(self.focus == PrFocus::Body && self.md.takes_ai_key(&key))
        {
            return self.open_web();
        }
        // The text fields' editing keys, through the shared core.
        self.edit
            .sync((self.focus == PrFocus::Title).then_some(self.focus));
        match self.focus {
            PrFocus::Title => {
                if let Some((result, copied)) =
                    self.edit.dialog_key(PrFocus::Title, &mut self.title, &key)
                {
                    if result == crate::text_edit::KeyEdit::Edited {
                        self.message = None;
                    }
                    if copied.is_some() {
                        self.message = copied;
                    }
                    return PullRequestOutcome::Stay;
                }
            }
            PrFocus::Body => match self.md.on_key(key) {
                crate::md_editor::KeyUse::Handled => {
                    self.message = None;
                    return PullRequestOutcome::Stay;
                }
                crate::md_editor::KeyUse::Submit => return self.submit(),
                crate::md_editor::KeyUse::Pass => {}
            },
            _ => {}
        }
        // A focused, closed combobox opens on Enter / Space / Down.
        let focus = self.focus;
        if let Some(c) = self.combo_mut(focus) {
            if !matches!(key.code, KeyCode::Tab | KeyCode::BackTab | KeyCode::Up)
                && c.on_key(key) == ComboOutcome::Handled
            {
                self.message = None;
                return PullRequestOutcome::Stay;
            }
        }
        match key.code {
            KeyCode::Esc => return self.request_close(dialog_kit::DismissKind::Esc),
            KeyCode::Left | KeyCode::Right if self.focus == PrFocus::Tabs => {
                let len = self.seed.tabs.len();
                if len > 0 {
                    let next = if key.code == KeyCode::Right {
                        (self.tab + 1) % len
                    } else {
                        (self.tab + len - 1) % len
                    };
                    self.select_tab(next);
                }
            }
            KeyCode::Tab | KeyCode::BackTab | KeyCode::Down | KeyCode::Up => {
                let forward =
                    matches!(key.code, KeyCode::Down) || (key.code == KeyCode::Tab && !shift);
                let order = self.order();
                let at = order.iter().position(|f| *f == self.focus).unwrap_or(0);
                let next = if forward {
                    (at + 1) % order.len()
                } else {
                    (at + order.len() - 1) % order.len()
                };
                self.focus = order[next];
            }
            _ if crate::keymap::current().is(Bind::DialogSubmit, &key) => return self.submit(),
            KeyCode::Enter => {
                return match self.focus {
                    // Enter in Title moves on, like a form.
                    PrFocus::Title => {
                        self.focus = PrFocus::Body;
                        PullRequestOutcome::Stay
                    }
                    other => self.activate(other),
                };
            }
            KeyCode::Char(' ') if !matches!(self.focus, PrFocus::Title | PrFocus::Body) => {
                return self.activate(self.focus)
            }
            _ => {}
        }
        PullRequestOutcome::Stay
    }

    fn field_mouse(&mut self, mouse: MouseEvent) -> bool {
        let focus = self.focus;
        let (id, text) = match focus {
            PrFocus::Title => (Some(PrFocus::Title), self.title.clone()),
            _ => (None, String::new()),
        };
        self.edit.sync(id);
        self.edit
            .dialog_mouse(id, &text, mouse.kind, mouse.column, mouse.row)
    }

    pub(crate) fn on_mouse(&mut self, mouse: MouseEvent) -> PullRequestOutcome {
        if let Some(v) = self.guard.mouse(mouse) {
            return self.verdict(v);
        }
        // An open picker owns the pointer: nothing behind it hovers.
        if self.meta.is_open() {
            self.meta.on_mouse(mouse);
            return PullRequestOutcome::Stay;
        }
        // An open popup is modal: it takes every mouse event.
        if let Some(open) = self.open_combo() {
            let mut covered = Rect::default();
            if let Some(c) = self.combo_mut(open) {
                covered = c.popup_rect();
                c.on_mouse(mouse);
            }
            if matches!(mouse.kind, MouseEventKind::Down(_)) && self.open_combo().is_none() {
                self.popup_shield = Some((covered, std::time::Instant::now()));
            }
            return PullRequestOutcome::Stay;
        }
        if let Some((shield, at)) = self.popup_shield {
            let inside = shield.contains((mouse.column, mouse.row).into());
            match mouse.kind {
                _ if at.elapsed() > POPUP_SHIELD => self.popup_shield = None,
                MouseEventKind::Down(_) if inside => return PullRequestOutcome::Stay,
                MouseEventKind::Up(_) | MouseEventKind::Drag(_) if inside => {
                    return PullRequestOutcome::Stay
                }
                _ if inside => {}
                _ => self.popup_shield = None,
            }
        }
        if let Some(kind) = self.guard.pointer(mouse) {
            return self.request_close(kind);
        }
        if self.md.on_mouse(mouse) == crate::md_editor::MouseUse::Inside {
            if matches!(mouse.kind, MouseEventKind::Down(MouseButton::Left)) {
                self.focus = PrFocus::Body;
            }
            return PullRequestOutcome::Stay;
        }
        let hit = dialog_kit::hit_at(&self.hits, mouse.column, mouse.row);
        if !matches!(mouse.kind, MouseEventKind::Down(_)) && self.field_mouse(mouse) {
            return PullRequestOutcome::Stay;
        }
        match mouse.kind {
            MouseEventKind::Moved | MouseEventKind::Drag(_) => self.hover = hit,
            MouseEventKind::Down(MouseButton::Left) => match hit {
                Some(PrHit::Tab(i)) => {
                    self.focus = PrFocus::Tabs;
                    self.select_tab(i);
                }
                Some(PrHit::Part(part)) => {
                    self.focus = part;
                    match part {
                        PrFocus::FromRepo
                        | PrFocus::FromBranch
                        | PrFocus::ToRepo
                        | PrFocus::ToBranch => {
                            if let Some(c) = self.combo_mut(part) {
                                c.on_mouse(mouse);
                            }
                        }
                        PrFocus::Title => {
                            self.field_mouse(mouse);
                        }
                        PrFocus::Body => {}
                        PrFocus::Tabs => {}
                        _ => return self.activate(part),
                    }
                }
                None => {}
            },
            _ => {}
        }
        PullRequestOutcome::Stay
    }
}

impl dialog_kit::Dismiss for PullRequestDialog {
    fn is_dirty(&self) -> bool {
        self.typed()
    }

    fn guard(&self) -> dialog_kit::Guard {
        dialog_kit::Guard::ConfirmWhenDirty
    }

    fn close_hit(&self) -> &dialog_kit::CloseHit {
        &self.guard.close
    }

    fn close_hit_mut(&mut self) -> &mut dialog_kit::CloseHit {
        &mut self.guard.close
    }
}
