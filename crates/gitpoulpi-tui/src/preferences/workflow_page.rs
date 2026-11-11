//! The Workflow page (Settings > Repository > Workflow): the branching
//! workflow of this repository (`.gitpoulpi/workflow.toml`, plan
//! `branching-workflows.md`).
//!
//! It edits a **draft** (the effective workflow: the shared file with this
//! user's override on top) and writes nothing until a Save button is used:
//! "Save to .gitpoulpi/workflow.toml (shared)" writes the draft to the
//! shared file (and drops this user's override, which it now contains),
//! "Only for me" writes the difference from the shared file into the repo
//! state. Bases and topic types are edited one at a time (a select row, then
//! the rows of the selected one) so every control is the same one the other
//! pages use: comboboxes, text rows, toggles, buttons.

use std::cell::OnceCell;
use std::sync::Arc;

use gitpoulpi_core::workflow::{
    detect, validate, BaseBranch, BranchPattern, Classification, Detection, FinishMode,
    FinishTarget, Hooks, Imported, IssueFlow, IssueFlowMode, MergeStrategy, Token, TopicType,
    Workflow, WorkflowOverride, WorkflowTemplate,
};
use gitpoulpi_i18n::msg::settings_workflow as t;
use gitpoulpi_session::{save_shared, workflow_path_for, LoadedWorkflow};

use super::editor_close::Act;
use super::persist::unique_id;

use super::*;
use crate::flow::view::FlowView;
use crate::th;

/// The rows of the page that are not plain config keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum WfTarget {
    Template,
    BaseName,
    BaseAliases,
    BaseParent,
    BaseProtected,
    BaseMerge,
    BaseTag,
    BaseEnvironment,
    TopicId,
    TopicLabel,
    TopicPattern,
    TopicMore,
    TopicRegex,
    TopicParent,
    FinishInto,
    FinishMerge,
    FinishTag,
    TopicFinishMode,
    TopicDelete,
    HookAfterStart,
    HookBeforeFinish,
    IssueMode,
    IssueTitle,
    IssueLabels,
    IssueBranch,
    IssueWaitFor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum WfButton {
    Detect,
    UseDetected,
    Import,
    SaveShared,
    SaveMine,
    Discard,
}

/// What the list's selection points at.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(super) enum WfKind {
    #[default]
    General,
    Base,
    Topic,
    /// A finish rule of the selected topic type.
    Finish,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EditorButton {
    Save,
    Cancel,
}

/// The multi-line hook editor.
struct HookEdit {
    target: WfTarget,
    text: String,
    field: crate::text_edit::FieldEditor<()>,
    close: super::editor_close::EditorClose,
}

#[derive(Default)]
pub(super) struct WorkflowPage {
    /// The workflow being edited; read from the files on first use.
    draft: OnceCell<Workflow>,
    /// The shared file's workflow (none when the file does not exist yet).
    shared: OnceCell<Option<Workflow>>,
    /// The draft as loaded: the page is dirty when the draft differs.
    baseline: OnceCell<Workflow>,
    pub(super) kind: WfKind,
    pub(super) base: usize,
    pub(super) topic: usize,
    pub(super) finish: usize,
    /// The window's report (branches, imports) and the version it had.
    view: Option<Arc<FlowView>>,
    view_version: u64,
    detections: Option<Vec<Detection>>,
    /// What the last action said (detect, save), shown on the page.
    notice: Option<String>,
    /// A status line for the window: a file was written.
    saved: Option<String>,
    editor: Option<HookEdit>,
    editor_hits: Vec<(Rect, EditorButton)>,
}

/// What `override_from` could not express: nothing yet, kept for clarity.
pub(super) fn override_from(shared: &Workflow, draft: &Workflow) -> WorkflowOverride {
    let mut o = WorkflowOverride::default();
    if draft.template != shared.template {
        o.template = Some(draft.template);
    }
    for b in &draft.bases {
        if shared.bases.iter().find(|s| s.name == b.name) != Some(b) {
            o.bases.push(b.clone());
        }
    }
    for s in &shared.bases {
        if !draft.bases.iter().any(|b| b.name == s.name) {
            o.disable_bases.push(s.name.clone());
        }
    }
    for tp in &draft.topics {
        if shared.topics.iter().find(|s| s.id == tp.id) != Some(tp) {
            o.topics.push(tp.clone());
        }
    }
    for s in &shared.topics {
        if !draft.topics.iter().any(|tp| tp.id == s.id) {
            o.disable_topics.push(s.id.clone());
        }
    }
    // The order is only stored when it differs from what the merge gives
    // without it: the shared order, then this user's own additions.
    let natural_bases: Vec<&str> = shared
        .bases
        .iter()
        .filter(|s| draft.bases.iter().any(|b| b.name == s.name))
        .map(|s| s.name.as_str())
        .chain(
            draft
                .bases
                .iter()
                .filter(|b| !shared.bases.iter().any(|s| s.name == b.name))
                .map(|b| b.name.as_str()),
        )
        .collect();
    let drafted: Vec<&str> = draft.bases.iter().map(|b| b.name.as_str()).collect();
    if drafted != natural_bases {
        o.base_order = draft.bases.iter().map(|b| b.name.clone()).collect();
    }
    let natural_topics: Vec<&str> = shared
        .topics
        .iter()
        .filter(|s| draft.topics.iter().any(|t| t.id == s.id))
        .map(|s| s.id.as_str())
        .chain(
            draft
                .topics
                .iter()
                .filter(|t| !shared.topics.iter().any(|s| s.id == t.id))
                .map(|t| t.id.as_str()),
        )
        .collect();
    let drafted: Vec<&str> = draft.topics.iter().map(|t| t.id.as_str()).collect();
    if drafted != natural_topics {
        o.topic_order = draft.topics.iter().map(|t| t.id.clone()).collect();
    }
    if draft.hooks != shared.hooks {
        o.hooks = Some(draft.hooks.clone());
    }
    if draft.issue_flow != shared.issue_flow {
        o.issue_flow = draft.issue_flow.clone();
    }
    o
}

fn list_text(items: &[String]) -> String {
    items.join(", ")
}

fn parse_list(text: &str) -> Vec<String> {
    text.split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect()
}

fn strategy_options() -> Vec<(String, String)> {
    vec![
        (t::DEFAULT.get().into(), String::new()),
        (t::MERGE_MERGE.get().into(), "merge".into()),
        (t::MERGE_SQUASH.get().into(), "squash".into()),
        (t::MERGE_REBASE.get().into(), "rebase".into()),
        (t::MERGE_FF.get().into(), "fast-forward".into()),
    ]
}

fn strategy_value(s: Option<MergeStrategy>) -> String {
    match s {
        None => String::new(),
        Some(MergeStrategy::Merge) => "merge".into(),
        Some(MergeStrategy::Squash) => "squash".into(),
        Some(MergeStrategy::Rebase) => "rebase".into(),
        Some(MergeStrategy::FastForward) => "fast-forward".into(),
    }
}

fn strategy_of(value: &str) -> Option<MergeStrategy> {
    match value {
        "merge" => Some(MergeStrategy::Merge),
        "squash" => Some(MergeStrategy::Squash),
        "rebase" => Some(MergeStrategy::Rebase),
        "fast-forward" => Some(MergeStrategy::FastForward),
        _ => None,
    }
}

fn finish_mode_options() -> Vec<(String, String)> {
    vec![
        (t::DEFAULT.get().into(), String::new()),
        (t::MODE_LOCAL.get().into(), "local-merge".into()),
        (t::MODE_REQUEST.get().into(), "merge-request".into()),
        (t::MODE_ASK.get().into(), "ask".into()),
    ]
}

fn finish_mode_value(m: Option<FinishMode>) -> String {
    match m {
        None => String::new(),
        Some(FinishMode::LocalMerge) => "local-merge".into(),
        Some(FinishMode::MergeRequest) => "merge-request".into(),
        Some(FinishMode::Ask) => "ask".into(),
    }
}

fn finish_mode_of(value: &str) -> Option<FinishMode> {
    match value {
        "local-merge" => Some(FinishMode::LocalMerge),
        "merge-request" => Some(FinishMode::MergeRequest),
        "ask" => Some(FinishMode::Ask),
        _ => None,
    }
}

fn issue_mode_options() -> Vec<(String, String)> {
    vec![
        (t::ISSUE_LOCAL.get().into(), "local".into()),
        (t::ISSUE_HOST.get().into(), "host-creates-branch".into()),
        (t::ISSUE_BOT.get().into(), "bot".into()),
    ]
}

fn issue_mode_value(m: IssueFlowMode) -> String {
    match m {
        IssueFlowMode::Local => "local".into(),
        IssueFlowMode::HostCreatesBranch => "host-creates-branch".into(),
        IssueFlowMode::Bot => "bot".into(),
    }
}

fn issue_mode_of(value: &str) -> IssueFlowMode {
    match value {
        "host-creates-branch" => IssueFlowMode::HostCreatesBranch,
        "bot" => IssueFlowMode::Bot,
        _ => IssueFlowMode::Local,
    }
}

/// A value for each token, for "would create".
fn sample_fields(pattern: &BranchPattern) -> gitpoulpi_core::workflow::Fields {
    pattern
        .tokens()
        .into_iter()
        .map(|tok| {
            let v = match tok {
                Token::Slug => "add-login",
                Token::Issue => "123",
                Token::IssueKey => "ABC-123",
                Token::User => "ada",
                Token::Date => "2026-10-16",
                Token::Version => "1.4.0",
                Token::N => "1",
            };
            (tok.name().to_string(), v.to_string())
        })
        .collect()
}

impl PreferencesState {
    // -- loading ---------------------------------------------------------

    fn wf_load(&self) -> (Workflow, Option<Workflow>) {
        let dir = &self.config.session.per_repo_state_dir;
        match LoadedWorkflow::load(&self.repo_root, dir) {
            Ok(loaded) => {
                let shared = loaded.shared.as_ref().map(|s| s.workflow.clone());
                (loaded.effective(), shared)
            }
            Err(_) => (Workflow::default(), None),
        }
    }

    pub(super) fn wf_draft(&self) -> &Workflow {
        self.workflow.draft.get_or_init(|| {
            let (draft, shared) = self.wf_load();
            let _ = self.workflow.shared.set(shared);
            let _ = self.workflow.baseline.set(draft.clone());
            draft
        })
    }

    fn wf_shared(&self) -> Option<&Workflow> {
        self.wf_draft();
        self.workflow.shared.get().and_then(|s| s.as_ref())
    }

    fn wf_mut(&mut self) -> &mut Workflow {
        self.wf_draft();
        self.workflow.draft.get_mut().expect("initialised above")
    }

    fn wf_reload(&mut self) {
        self.workflow.draft = OnceCell::new();
        self.workflow.shared = OnceCell::new();
        self.workflow.baseline = OnceCell::new();
        self.workflow.kind = WfKind::General;
        self.workflow.base = 0;
        self.workflow.topic = 0;
        self.workflow.finish = 0;
        self.workflow.detections = None;
    }

    /// The daemon's report, pushed in by the window each frame.
    pub(crate) fn set_workflow_view(&mut self, view: Option<Arc<FlowView>>, version: u64) {
        if version != self.workflow.view_version {
            self.workflow.view_version = version;
            self.workflow.view = view;
        }
    }

    /// A status line when a workflow file was written since the last call.
    pub(crate) fn take_workflow_saved(&mut self) -> Option<String> {
        self.workflow.saved.take()
    }

    /// Lands on the Workflow page (the Flow menu's "Workflow settings...").
    pub(crate) fn show_workflow(&mut self) {
        self.section = PrefSection::ALL
            .iter()
            .position(|s| *s == PrefSection::Workflow)
            .unwrap_or(0);
        self.remember_sub_tab(PrefSection::Workflow, SubTab::General);
        self.focus = PrefFocus::Fields;
        self.field = self.first_selectable();
        self.scroll = 0;
    }

    /// The branch names the page previews against (local and remote, the
    /// remote part dropped, no duplicates).
    fn wf_branch_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self
            .workflow
            .view
            .as_ref()
            .map(|v| v.report.branches.iter().map(|b| b.name.clone()).collect())
            .unwrap_or_default();
        names.sort_unstable();
        names.dedup();
        names
    }

    fn wf_imports(&self) -> Vec<Imported> {
        self.workflow
            .view
            .as_ref()
            .map(|v| v.report.imports.clone())
            .unwrap_or_default()
    }

    // -- the rows --------------------------------------------------------

    fn wf_text(label: &str, value: &str, target: WfTarget, help: &str) -> Field {
        Field::plain(
            label,
            Control::Text(value.to_string()),
            Target::Workflow(target),
        )
        .with_help(help)
    }

    fn wf_combo(
        label: &str,
        options: Vec<(String, String)>,
        value: &str,
        custom: bool,
        target: WfTarget,
        help: &str,
    ) -> Field {
        Field::plain(
            label,
            Control::Combo {
                options,
                value: value.to_string(),
                custom,
            },
            Target::Workflow(target),
        )
        .with_help(help)
    }

    fn wf_toggle(label: &str, on: bool, target: WfTarget, help: &str) -> Field {
        Field::plain(label, Control::Toggle(on), Target::Workflow(target)).with_help(help)
    }

    fn wf_btn(label: &str, button: WfButton, help: &str) -> Field {
        let f = Field::button(label, PrefAction::Workflow(button));
        if help.is_empty() {
            f
        } else {
            f.with_help(help)
        }
    }

    fn wf_info(label: &str, text: String) -> Field {
        Field::plain(label, Control::Info(text), Target::None)
    }

    fn wf_base_options(&self, except: Option<usize>) -> Vec<(String, String)> {
        let mut out = vec![(t::NONE.get().to_string(), String::new())];
        for (i, b) in self.wf_draft().bases.iter().enumerate() {
            if Some(i) != except && !b.name.is_empty() {
                out.push((b.name.clone(), b.name.clone()));
            }
        }
        out
    }

    /// Workflow > General: the template, hooks, issue flow and where it is
    /// saved, none of them a rule's own.
    pub(super) fn workflow_general_fields(&self) -> Vec<Field> {
        let mut f = self.wf_general_rows(self.wf_draft());
        // The save bar is always the last row, and the page pins it.
        f.push(self.wf_save_bar());
        f
    }

    /// Workflow > the rules tab: the rows of the selected base, topic type
    /// or finish rule.
    pub(super) fn workflow_item_fields(&self) -> Vec<Field> {
        let wf = self.wf_draft();
        let mut f = match self.wf_kind() {
            WfKind::General => vec![Field::heading(t::NO_RULES.get())],
            WfKind::Base => self.wf_base_rows(wf),
            WfKind::Topic => self.wf_topic_rows(wf),
            WfKind::Finish => self.wf_finish_rows(wf),
        };
        f.push(self.wf_save_bar());
        f
    }

    /// How many trailing rows the page pins under the scrolling ones.
    pub(super) fn wf_sticky_rows() -> usize {
        1
    }

    /// Whether the page was ever opened (the draft is read on first use).
    pub(super) fn workflow_draft_loaded(&self) -> bool {
        self.workflow.draft.get().is_some()
    }

    /// Whether the draft differs from what was loaded.
    pub(super) fn wf_dirty(&self) -> bool {
        let draft = self.wf_draft();
        self.workflow.baseline.get().is_some_and(|b| b != draft)
    }

    fn wf_save_bar(&self) -> Field {
        let dirty = self.wf_dirty();
        let label = if dirty {
            t::UNSAVED.get()
        } else {
            t::NO_CHANGES.get()
        };
        let mut discard = Btn::new(t::DISCARD.get(), PrefAction::Workflow(WfButton::Discard));
        discard.enabled = dirty;
        Field::buttons(
            label,
            vec![
                Btn::new(
                    t::SAVE_SHARED.get(),
                    PrefAction::Workflow(WfButton::SaveShared),
                ),
                Btn::new(t::SAVE_MINE.get(), PrefAction::Workflow(WfButton::SaveMine)),
                discard,
            ],
        )
    }

    fn wf_general_rows(&self, wf: &Workflow) -> Vec<Field> {
        let mut f: Vec<Field> = Vec::new();
        f.push(Field::heading(t::HEADING_TEMPLATE.get()));
        f.push(
            Field::plain(
                t::TEMPLATE.get(),
                Control::Combo {
                    options: WorkflowTemplate::ALL
                        .iter()
                        .map(|tp| (tp.label().to_string(), tp.id().to_string()))
                        .collect(),
                    value: wf.template.id().to_string(),
                    custom: false,
                },
                Target::Workflow(WfTarget::Template),
            )
            .with_help(t::TEMPLATE_HELP.get()),
        );
        f.push(Self::wf_btn(
            t::DETECT.get(),
            WfButton::Detect,
            t::DETECT_HELP.get(),
        ));
        if let Some(found) = &self.workflow.detections {
            f.extend(self.wf_detection_rows(found));
        }
        let imports = self.wf_imports();
        if let Some(import) = imports.first() {
            f.push(Self::wf_info(t::IMPORT_FOUND.get(), import.source.clone()));
            for note in &import.notes {
                f.push(Self::wf_info("", note.clone()));
            }
            f.push(Self::wf_btn(
                t::IMPORT.get(),
                WfButton::Import,
                t::IMPORT_HELP.get(),
            ));
        }
        if let Some(note) = &self.workflow.notice {
            f.push(Self::wf_info(t::NOTICE.get(), note.clone()));
        }

        // -- hooks -----------------------------------------------------------
        f.push(Field::heading(t::HEADING_HOOKS.get()));
        f.push(Self::wf_info("", t::HOOKS_WARNING.get().to_string()));
        f.push(
            Field::plain(
                t::HOOK_AFTER_START.get(),
                Control::Area(wf.hooks.after_start.join("\n")),
                Target::Workflow(WfTarget::HookAfterStart),
            )
            .with_help(t::HOOK_HELP.get())
            .with_warning(t::HOOKS_WARNING.get()),
        );
        f.push(
            Field::plain(
                t::HOOK_BEFORE_FINISH.get(),
                Control::Area(wf.hooks.before_finish.join("\n")),
                Target::Workflow(WfTarget::HookBeforeFinish),
            )
            .with_help(t::HOOK_HELP.get())
            .with_warning(t::HOOKS_WARNING.get()),
        );

        // -- issue flow (carried; used by the host-driven flows) ----------------
        f.push(Field::heading(t::HEADING_ISSUE_FLOW.get()));
        let flow = wf.issue_flow.clone().unwrap_or_default();
        f.push(Self::wf_combo(
            t::ISSUE_MODE.get(),
            issue_mode_options(),
            &issue_mode_value(flow.mode),
            false,
            WfTarget::IssueMode,
            t::ISSUE_MODE_HELP.get(),
        ));
        f.push(Self::wf_text(
            t::ISSUE_TITLE.get(),
            flow.issue_title.as_deref().unwrap_or(""),
            WfTarget::IssueTitle,
            t::ISSUE_TITLE_HELP.get(),
        ));
        f.push(Self::wf_text(
            t::ISSUE_LABELS.get(),
            &list_text(&flow.issue_labels),
            WfTarget::IssueLabels,
            t::ISSUE_LABELS_HELP.get(),
        ));
        f.push(Self::wf_text(
            t::ISSUE_BRANCH.get(),
            flow.branch_from_issue.as_deref().unwrap_or(""),
            WfTarget::IssueBranch,
            t::ISSUE_BRANCH_HELP.get(),
        ));
        f.push(Self::wf_text(
            t::ISSUE_WAIT.get(),
            flow.wait_for.as_deref().unwrap_or(""),
            WfTarget::IssueWaitFor,
            t::ISSUE_WAIT_HELP.get(),
        ));

        // -- problems ----------------------------------------------------------
        let problems = validate(wf);
        if !problems.is_empty() {
            f.push(Field::heading(t::HEADING_PROBLEMS.get()));
            for p in problems {
                f.push(Self::wf_info("", p).with_warning(t::PROBLEM.get()));
            }
        }

        // -- where it is saved ---------------------------------------------------
        f.push(Field::heading(t::SHARED_FILE.get()));
        let path = workflow_path_for(&self.repo_root, &self.config.session.per_repo_state_dir);
        let state = if path.exists() {
            t::SHARED_EXISTS.get()
        } else {
            t::SHARED_MISSING.get()
        };
        f.push(Self::wf_info(
            "",
            format!("{} ({state})", self.wf_shared_label()),
        ));
        f.push(Self::wf_info("", t::SAVE_SHARED_HELP.get().to_string()));
        f.push(Self::wf_info("", t::SAVE_MINE_HELP.get().to_string()));
        f
    }

    fn wf_base_rows(&self, wf: &Workflow) -> Vec<Field> {
        let base = self.workflow.base.min(wf.bases.len().saturating_sub(1));
        let b = &wf.bases[base];
        vec![
            Field::heading(t::HEADING_BASES.get()),
            Self::wf_text(
                t::BASE_NAME.get(),
                &b.name,
                WfTarget::BaseName,
                t::BASE_NAME_HELP.get(),
            ),
            Self::wf_text(
                t::BASE_ALIASES.get(),
                &list_text(&b.names),
                WfTarget::BaseAliases,
                t::BASE_ALIASES_HELP.get(),
            ),
            Self::wf_combo(
                t::BASE_PARENT.get(),
                self.wf_base_options(Some(base)),
                b.parent.as_deref().unwrap_or(""),
                false,
                WfTarget::BaseParent,
                t::BASE_PARENT_HELP.get(),
            ),
            Self::wf_toggle(
                t::BASE_PROTECTED.get(),
                b.protected,
                WfTarget::BaseProtected,
                t::BASE_PROTECTED_HELP.get(),
            ),
            Self::wf_combo(
                t::BASE_MERGE.get(),
                strategy_options(),
                &strategy_value(b.merge),
                false,
                WfTarget::BaseMerge,
                t::BASE_MERGE_HELP.get(),
            ),
            Self::wf_toggle(
                t::BASE_TAG.get(),
                b.tag_on_merge,
                WfTarget::BaseTag,
                t::BASE_TAG_HELP.get(),
            ),
            Self::wf_toggle(
                t::BASE_ENVIRONMENT.get(),
                b.environment,
                WfTarget::BaseEnvironment,
                t::BASE_ENVIRONMENT_HELP.get(),
            ),
        ]
    }

    fn wf_topic_rows(&self, wf: &Workflow) -> Vec<Field> {
        let topic = self.workflow.topic.min(wf.topics.len().saturating_sub(1));
        let tp = &wf.topics[topic];
        let mut f = vec![
            Field::heading(t::HEADING_TOPICS.get()),
            Self::wf_text(
                t::TOPIC_ID.get(),
                &tp.id,
                WfTarget::TopicId,
                t::TOPIC_ID_HELP.get(),
            ),
            Self::wf_text(
                t::TOPIC_LABEL.get(),
                &tp.label,
                WfTarget::TopicLabel,
                t::TOPIC_LABEL_HELP.get(),
            ),
        ];
        let mut pattern = Self::wf_text(
            t::TOPIC_PATTERN.get(),
            &tp.pattern,
            WfTarget::TopicPattern,
            t::TOKEN_HELP.get(),
        );
        if let Err(e) = BranchPattern::parse(&tp.pattern) {
            pattern = pattern.with_warning(&e.to_string());
        }
        f.push(pattern);
        f.extend(self.wf_pattern_preview(wf, tp));
        f.push(Self::wf_text(
            t::TOPIC_MORE.get(),
            &list_text(&tp.patterns),
            WfTarget::TopicMore,
            t::TOPIC_MORE_HELP.get(),
        ));
        f.push(Self::wf_text(
            t::TOPIC_REGEX.get(),
            tp.regex.as_deref().unwrap_or(""),
            WfTarget::TopicRegex,
            t::TOPIC_REGEX_HELP.get(),
        ));
        f.push(Self::wf_combo(
            t::TOPIC_PARENT.get(),
            self.wf_base_options(None),
            tp.parent.as_deref().unwrap_or(""),
            true,
            WfTarget::TopicParent,
            t::TOPIC_PARENT_HELP.get(),
        ));
        f.push(Self::wf_combo(
            t::FINISH_MODE.get(),
            finish_mode_options(),
            &finish_mode_value(tp.finish_mode),
            false,
            WfTarget::TopicFinishMode,
            t::FINISH_MODE_HELP.get(),
        ));
        f.push(Self::wf_toggle(
            t::DELETE_AFTER.get(),
            tp.delete_after_finish,
            WfTarget::TopicDelete,
            t::DELETE_AFTER_HELP.get(),
        ));
        f
    }

    fn wf_finish_rows(&self, wf: &Workflow) -> Vec<Field> {
        let topic = self.workflow.topic.min(wf.topics.len().saturating_sub(1));
        let tp = &wf.topics[topic];
        let fin = self.workflow.finish.min(tp.finish.len().saturating_sub(1));
        let target = &tp.finish[fin];
        vec![
            Field::heading(t::GROUP_FINISH.get()),
            Self::wf_combo(
                t::FINISH_INTO.get(),
                self.wf_base_options(None)
                    .into_iter()
                    .filter(|(_, v)| !v.is_empty())
                    .collect(),
                &target.into,
                true,
                WfTarget::FinishInto,
                t::FINISH_INTO_HELP.get(),
            ),
            Self::wf_combo(
                t::FINISH_MERGE.get(),
                strategy_options(),
                &strategy_value(target.merge),
                false,
                WfTarget::FinishMerge,
                t::FINISH_MERGE_HELP.get(),
            ),
            Self::wf_text(
                t::FINISH_TAG.get(),
                target.tag.as_deref().unwrap_or(""),
                WfTarget::FinishTag,
                t::FINISH_TAG_HELP.get(),
            ),
        ]
    }

    fn wf_shared_label(&self) -> String {
        let dir = &self.config.session.per_repo_state_dir;
        format!("{dir}/{}", gitpoulpi_session::WORKFLOW_FILE)
    }

    /// The rows showing what Detect found.
    fn wf_detection_rows(&self, found: &[Detection]) -> Vec<Field> {
        let mut rows = Vec::new();
        let Some(best) = found.first() else {
            return rows;
        };
        rows.push(Self::wf_info(
            t::DETECT_BEST_ROW.get(),
            t::detect_best(best.template.label(), u64::from(best.score)),
        ));
        let bases: Vec<String> = best.bases.iter().filter_map(|b| b.found.clone()).collect();
        rows.push(Self::wf_info(
            t::DETECT_BASES.get(),
            if bases.is_empty() {
                t::NONE_FOUND.get().to_string()
            } else {
                list_text(&bases)
            },
        ));
        for topic in best.topics.iter().filter(|tp| tp.count > 0) {
            rows.push(Self::wf_info(
                t::DETECT_TOPIC_ROW.get(),
                t::detect_topic(topic.count as u64, &topic.id, list_text(&topic.examples)),
            ));
        }
        rows.push(Self::wf_info(
            t::DETECT_UNMATCHED.get(),
            best.unmatched.to_string(),
        ));
        let scores: Vec<String> = found
            .iter()
            .map(|d| format!("{} {}", d.template.label(), d.score))
            .collect();
        rows.push(Self::wf_info(t::DETECT_SCORES.get(), list_text(&scores)));
        rows.push(Self::wf_btn(
            t::USE_DETECTED.get(),
            WfButton::UseDetected,
            t::USE_DETECTED_HELP.get(),
        ));
        rows
    }

    /// The branches a pattern matches now and a name it would create.
    fn wf_pattern_preview(&self, wf: &Workflow, topic: &TopicType) -> Vec<Field> {
        let mut rows = Vec::new();
        let compiled = wf.compile();
        let matched: Vec<String> = self
            .wf_branch_names()
            .into_iter()
            .filter(|n| {
                matches!(
                    compiled.classify(n),
                    Classification::Topic { type_id, .. } if type_id == topic.id
                )
            })
            .collect();
        let shown: Vec<String> = matched.iter().take(4).cloned().collect();
        rows.push(Self::wf_info(
            t::PREVIEW_MATCHES.get(),
            if matched.is_empty() {
                t::NONE_FOUND.get().to_string()
            } else if matched.len() > shown.len() {
                t::preview_more(list_text(&shown), (matched.len() - shown.len()) as u64)
            } else {
                list_text(&shown)
            },
        ));
        if let Ok(pattern) = BranchPattern::parse(&topic.pattern) {
            if let Ok(name) = pattern.render(&sample_fields(&pattern)) {
                rows.push(Self::wf_info(t::PREVIEW_CREATES.get(), name));
            }
        }
        rows
    }

    // -- committing --------------------------------------------------------

    fn wf_touch(&mut self) {
        self.workflow.notice = None;
    }

    pub(super) fn wf_commit_bool(&mut self, target: WfTarget, on: bool) {
        self.wf_touch();
        let (bi, ti) = (self.workflow.base, self.workflow.topic);
        let wf = self.wf_mut();
        match target {
            WfTarget::BaseProtected => {
                if let Some(b) = wf.bases.get_mut(bi) {
                    b.protected = on;
                }
            }
            WfTarget::BaseTag => {
                if let Some(b) = wf.bases.get_mut(bi) {
                    b.tag_on_merge = on;
                }
            }
            WfTarget::BaseEnvironment => {
                if let Some(b) = wf.bases.get_mut(bi) {
                    b.environment = on;
                }
            }
            WfTarget::TopicDelete => {
                if let Some(tp) = wf.topics.get_mut(ti) {
                    tp.delete_after_finish = on;
                }
            }
            _ => {}
        }
    }

    pub(super) fn wf_commit_combo(&mut self, target: WfTarget, value: &str) {
        self.wf_touch();
        let (bi, ti, fi) = (
            self.workflow.base,
            self.workflow.topic,
            self.workflow.finish,
        );
        let value = value.trim().to_string();
        let opt = |v: &str| (!v.is_empty()).then(|| v.to_string());
        if target == WfTarget::Template {
            if let Some(template) = WorkflowTemplate::from_id(&value) {
                // A preset replaces the definition (nothing is saved yet);
                // the hooks and the issue flow are the user's own and stay.
                let preset = template.workflow();
                let wf = self.wf_mut();
                wf.template = template;
                wf.bases = preset.bases;
                wf.topics = preset.topics;
                self.workflow.base = 0;
                self.workflow.topic = 0;
                self.workflow.finish = 0;
            }
            return;
        }
        let wf = self.wf_mut();
        match target {
            WfTarget::BaseParent => {
                if let Some(b) = wf.bases.get_mut(bi) {
                    b.parent = opt(&value);
                }
            }
            WfTarget::BaseMerge => {
                if let Some(b) = wf.bases.get_mut(bi) {
                    b.merge = strategy_of(&value);
                }
            }
            WfTarget::TopicParent => {
                if let Some(tp) = wf.topics.get_mut(ti) {
                    tp.parent = opt(&value);
                }
            }
            WfTarget::FinishInto => {
                if let Some(x) = wf.topics.get_mut(ti).and_then(|tp| tp.finish.get_mut(fi)) {
                    x.into = value;
                }
            }
            WfTarget::FinishMerge => {
                if let Some(x) = wf.topics.get_mut(ti).and_then(|tp| tp.finish.get_mut(fi)) {
                    x.merge = strategy_of(&value);
                }
            }
            WfTarget::TopicFinishMode => {
                if let Some(tp) = wf.topics.get_mut(ti) {
                    tp.finish_mode = finish_mode_of(&value);
                }
            }
            WfTarget::IssueMode => {
                wf.issue_flow.get_or_insert_with(IssueFlow::default).mode = issue_mode_of(&value);
            }
            _ => {}
        }
    }

    /// A typed value. `false`: refused (the message says why).
    pub(super) fn wf_commit_text(&mut self, target: WfTarget, draft: &str) -> bool {
        self.wf_touch();
        let value = draft.trim().to_string();
        let (bi, ti, fi) = (
            self.workflow.base,
            self.workflow.topic,
            self.workflow.finish,
        );
        let opt = |v: &str| (!v.is_empty()).then(|| v.to_string());
        match target {
            WfTarget::BaseName => {
                if value.is_empty() {
                    self.message = Some(t::NAME_EMPTY.get().into());
                    return false;
                }
                let wf = self.wf_mut();
                if wf
                    .bases
                    .iter()
                    .enumerate()
                    .any(|(i, b)| i != bi && b.name == value)
                {
                    self.message = Some(t::name_taken(&value));
                    return false;
                }
                let Some(old) = wf.bases.get(bi).map(|b| b.name.clone()) else {
                    return true;
                };
                wf.bases[bi].name = value.clone();
                // Parents and finish targets follow the rename.
                for b in &mut wf.bases {
                    if b.parent.as_deref() == Some(old.as_str()) {
                        b.parent = Some(value.clone());
                    }
                }
                for tp in &mut wf.topics {
                    if tp.parent.as_deref() == Some(old.as_str()) {
                        tp.parent = Some(value.clone());
                    }
                    for x in &mut tp.finish {
                        if x.into == old {
                            x.into = value.clone();
                        }
                    }
                }
            }
            WfTarget::BaseAliases => {
                if let Some(b) = self.wf_mut().bases.get_mut(bi) {
                    b.names = parse_list(&value);
                }
            }
            WfTarget::TopicId => {
                if value.is_empty() {
                    self.message = Some(t::ID_EMPTY.get().into());
                    return false;
                }
                let wf = self.wf_mut();
                if wf
                    .topics
                    .iter()
                    .enumerate()
                    .any(|(i, tp)| i != ti && tp.id == value)
                {
                    self.message = Some(t::name_taken(&value));
                    return false;
                }
                if let Some(tp) = wf.topics.get_mut(ti) {
                    tp.id = value;
                }
            }
            WfTarget::TopicLabel => {
                if let Some(tp) = self.wf_mut().topics.get_mut(ti) {
                    tp.label = value;
                }
            }
            WfTarget::TopicPattern => {
                // Kept even when it does not parse (the row says why in red):
                // refusing would lose what was typed.
                if let Some(tp) = self.wf_mut().topics.get_mut(ti) {
                    tp.pattern = value;
                }
            }
            WfTarget::TopicMore => {
                if let Some(tp) = self.wf_mut().topics.get_mut(ti) {
                    tp.patterns = parse_list(&value);
                }
            }
            WfTarget::TopicRegex => {
                if let Some(tp) = self.wf_mut().topics.get_mut(ti) {
                    tp.regex = opt(&value);
                }
            }
            WfTarget::FinishTag => {
                if let Some(x) = self
                    .wf_mut()
                    .topics
                    .get_mut(ti)
                    .and_then(|tp| tp.finish.get_mut(fi))
                {
                    x.tag = opt(&value);
                }
            }
            WfTarget::IssueTitle => {
                self.wf_mut()
                    .issue_flow
                    .get_or_insert_with(IssueFlow::default)
                    .issue_title = opt(&value);
            }
            WfTarget::IssueLabels => {
                self.wf_mut()
                    .issue_flow
                    .get_or_insert_with(IssueFlow::default)
                    .issue_labels = parse_list(&value);
            }
            WfTarget::IssueBranch => {
                self.wf_mut()
                    .issue_flow
                    .get_or_insert_with(IssueFlow::default)
                    .branch_from_issue = opt(&value);
            }
            WfTarget::IssueWaitFor => {
                self.wf_mut()
                    .issue_flow
                    .get_or_insert_with(IssueFlow::default)
                    .wait_for = opt(&value);
            }
            _ => {}
        }
        true
    }

    pub(super) fn wf_button(&mut self, button: WfButton) {
        self.wf_touch();
        match button {
            WfButton::Detect => {
                let names = self.wf_branch_names();
                let found = detect(&names);
                self.workflow.notice = Some(if names.is_empty() {
                    t::DETECT_NO_BRANCHES.get().to_string()
                } else {
                    t::detect_done(names.len() as u64)
                });
                self.workflow.detections = Some(found);
            }
            WfButton::UseDetected => {
                if let Some(best) = self
                    .workflow
                    .detections
                    .as_ref()
                    .and_then(|d| d.first().cloned())
                {
                    *self.wf_mut() = best.workflow();
                    self.workflow.kind = WfKind::General;
                    self.workflow.base = 0;
                    self.workflow.topic = 0;
                    self.workflow.notice = Some(t::NOT_SAVED.get().to_string());
                }
            }
            WfButton::Import => {
                if let Some(import) = self.wf_imports().into_iter().next() {
                    *self.wf_mut() = import.workflow;
                    self.workflow.kind = WfKind::General;
                    self.workflow.base = 0;
                    self.workflow.topic = 0;
                    self.workflow.notice = Some(t::NOT_SAVED.get().to_string());
                }
            }
            WfButton::SaveShared => self.wf_save_shared(),
            WfButton::SaveMine => self.wf_save_mine(),
            WfButton::Discard => {
                self.wf_reload();
                self.workflow.notice = Some(t::DISCARDED.get().to_string());
            }
        }
    }

    fn wf_save_shared(&mut self) {
        let path = workflow_path_for(&self.repo_root, &self.config.session.per_repo_state_dir);
        let draft = self.wf_draft().clone();
        match save_shared(&path, &draft, true) {
            Ok(_) => {
                // The shared file now holds what the draft says, this user's
                // layer included: it is dropped so it cannot shadow a
                // teammate's later edit of the file.
                self.wf_set_override(None);
                let said = t::saved_shared(path.display());
                self.workflow.saved = Some(said.clone());
                self.message = Some(said);
                self.wf_reload();
            }
            Err(err) => self.message = Some(t::save_failed(err)),
        }
    }

    /// Writes this user's override into the repo state (`None` drops it),
    /// starting from what is on disk so nothing else in the file is lost.
    fn wf_set_override(&mut self, ovr: Option<WorkflowOverride>) {
        let mut state = RepoState::load(&self.repo_state_path).unwrap_or_default();
        if state.workflow == ovr && self.repo_state.workflow == ovr {
            return;
        }
        state.workflow = ovr;
        self.repo_state = state;
        self.write_repo_state();
    }

    fn wf_save_mine(&mut self) {
        let shared = self.wf_shared().cloned().unwrap_or_default();
        let draft = self.wf_draft().clone();
        let ovr = override_from(&shared, &draft);
        self.wf_set_override((!ovr.is_empty()).then_some(ovr));
        let said = t::SAVED_MINE.get().to_string();
        self.workflow.saved = Some(said.clone());
        self.message = Some(said);
        self.wf_reload();
    }

    // -- the list (master-detail) -----------------------------------------------

    /// The selected kind, falling back to General when what the indexes
    /// point at is gone (removed, or the draft was replaced).
    pub(super) fn wf_kind(&self) -> WfKind {
        let wf = self.wf_draft();
        let w = &self.workflow;
        match w.kind {
            WfKind::Base if w.base < wf.bases.len() => WfKind::Base,
            WfKind::Topic if w.topic < wf.topics.len() => WfKind::Topic,
            WfKind::Finish
                if wf
                    .topics
                    .get(w.topic)
                    .is_some_and(|tp| w.finish < tp.finish.len()) =>
            {
                WfKind::Finish
            }
            // Nothing valid selected: the first rule, or none at all.
            _ if !wf.bases.is_empty() => WfKind::Base,
            _ if !wf.topics.is_empty() => WfKind::Topic,
            _ => WfKind::General,
        }
    }

    /// The topic type whose finish rules the list shows.
    fn wf_current_topic(&self) -> Option<usize> {
        let n = self.wf_draft().topics.len();
        (n > 0).then(|| self.workflow.topic.min(n - 1))
    }

    /// List index -> (kind, index within its group).
    fn wf_decode(&self, i: usize) -> (WfKind, usize) {
        let wf = self.wf_draft();
        let (nb, nt) = (wf.bases.len(), wf.topics.len());
        if i < nb {
            (WfKind::Base, i)
        } else if i < nb + nt {
            (WfKind::Topic, i - nb)
        } else {
            (WfKind::Finish, i - nb - nt)
        }
    }

    /// Replaces a selection that points at nothing by the first rule (the
    /// list has no General row: that is the other tab).
    pub(super) fn wf_normalise(&mut self) {
        let kind = self.wf_kind();
        let wf = self.wf_draft();
        let (nb, nt) = (wf.bases.len(), wf.topics.len());
        self.workflow.kind = kind;
        match kind {
            WfKind::Base => self.workflow.base = self.workflow.base.min(nb.saturating_sub(1)),
            WfKind::Topic => self.workflow.topic = self.workflow.topic.min(nt.saturating_sub(1)),
            WfKind::General | WfKind::Finish => {}
        }
    }

    /// The list index of the current selection.
    pub(super) fn wf_list_selected(&self) -> usize {
        let wf = self.wf_draft();
        let (nb, nt) = (wf.bases.len(), wf.topics.len());
        match self.wf_kind() {
            WfKind::General => 0,
            WfKind::Base => self.workflow.base.min(nb.saturating_sub(1)),
            WfKind::Topic => nb + self.workflow.topic.min(nt.saturating_sub(1)),
            WfKind::Finish => nb + nt + self.workflow.finish,
        }
    }

    pub(super) fn wf_select_index(&mut self, i: usize) {
        let (kind, at) = self.wf_decode(i);
        self.workflow.kind = kind;
        match kind {
            WfKind::General => {}
            WfKind::Base => self.workflow.base = at,
            WfKind::Topic => {
                if self.workflow.topic != at {
                    self.workflow.finish = 0;
                }
                self.workflow.topic = at;
            }
            WfKind::Finish => self.workflow.finish = at,
        }
    }

    /// The list: the bases, the topic types, and the finish rules of
    /// the topic type in play, each group under its header.
    pub(super) fn wf_list_spec(&self) -> super::list_editor::ListSpec {
        use super::list_editor::{AddKind, ListItem, ListSpec, Mark};
        let wf = self.wf_draft();
        let group = |g: &str| Some(g.to_string());
        let mut items = Vec::new();
        for b in &wf.bases {
            items.push(ListItem {
                label: if b.name.is_empty() {
                    t::UNNAMED.get().to_string()
                } else {
                    b.name.clone()
                },
                sub: b.parent.clone().filter(|p| !p.is_empty()),
                group: group(t::HEADING_BASES.get()),
                ..ListItem::default()
            });
        }
        for tp in &wf.topics {
            let problem = BranchPattern::parse(&tp.pattern).err();
            items.push(ListItem {
                label: tp.display_label().to_string(),
                sub: Some(tp.pattern.clone()).filter(|p| !p.is_empty()),
                mark: problem.map_or(Mark::None, |e| Mark::Problem(e.to_string())),
                group: group(t::HEADING_TOPICS.get()),
                ..ListItem::default()
            });
        }
        if let Some(tp) = self.wf_current_topic().and_then(|i| wf.topics.get(i)) {
            for x in &tp.finish {
                items.push(ListItem {
                    label: if x.into.is_empty() {
                        t::UNNAMED.get().to_string()
                    } else {
                        x.into.clone()
                    },
                    sub: Some(tp.display_label().to_string()),
                    group: group(t::GROUP_FINISH.get()),
                    ..ListItem::default()
                });
            }
        }
        ListSpec {
            title: t::LIST_TITLE.get().to_string(),
            items,
            add: AddKind::Menu(vec![
                t::ADD_KIND_BASE.get().to_string(),
                t::ADD_KIND_TOPIC.get().to_string(),
                t::ADD_KIND_FINISH.get().to_string(),
            ]),
            duplicate: false,
            reorder: true,
            min_len: 0,
        }
    }

    /// One list operation on the draft (nothing is written: Save does that).
    pub(super) fn wf_list_op(&mut self, op: super::list_editor::ListOp) {
        use super::list_editor::ListOp;
        self.wf_touch();
        match op {
            ListOp::Add(Some(0)) => {
                let wf = self.wf_mut();
                let name = unique_id("branch", |c| wf.bases.iter().any(|b| b.name == c));
                wf.bases.push(BaseBranch {
                    name,
                    ..Default::default()
                });
                self.workflow.base = self.wf_draft().bases.len() - 1;
                self.workflow.kind = WfKind::Base;
            }
            ListOp::Add(Some(1)) => {
                let wf = self.wf_mut();
                let id = unique_id("topic", |c| wf.topics.iter().any(|tp| tp.id == c));
                let parent = wf.bases.first().map(|b| b.name.clone());
                wf.topics.push(TopicType {
                    pattern: format!("{id}/{{slug}}"),
                    id,
                    parent,
                    ..Default::default()
                });
                self.workflow.topic = self.wf_draft().topics.len() - 1;
                self.workflow.finish = 0;
                self.workflow.kind = WfKind::Topic;
            }
            ListOp::Add(Some(2)) => {
                let Some(ti) = self.wf_current_topic() else {
                    self.message = Some(t::NEED_TOPIC.get().to_string());
                    return;
                };
                let wf = self.wf_mut();
                let into = wf
                    .topics
                    .get(ti)
                    .and_then(|tp| tp.parent.clone())
                    .or_else(|| wf.bases.first().map(|b| b.name.clone()))
                    .unwrap_or_default();
                if let Some(tp) = wf.topics.get_mut(ti) {
                    tp.finish.push(FinishTarget {
                        into,
                        ..Default::default()
                    });
                    self.workflow.finish = tp.finish.len() - 1;
                    self.workflow.topic = ti;
                    self.workflow.kind = WfKind::Finish;
                }
            }
            ListOp::Remove(i) => self.wf_remove(i),
            ListOp::Move { from, to } => self.wf_move(from, to),
            _ => {}
        }
    }

    fn wf_remove(&mut self, i: usize) {
        let (kind, at) = self.wf_decode(i);
        let ti = self.wf_current_topic().unwrap_or(0);
        match kind {
            WfKind::General => {}
            WfKind::Base => {
                let wf = self.wf_mut();
                if at < wf.bases.len() {
                    let gone = wf.bases.remove(at);
                    for b in &mut wf.bases {
                        if b.parent.as_deref() == Some(gone.name.as_str()) {
                            b.parent = None;
                        }
                    }
                    for tp in &mut wf.topics {
                        if tp.parent.as_deref() == Some(gone.name.as_str()) {
                            tp.parent = None;
                        }
                    }
                    self.workflow.base = at.saturating_sub(1);
                    self.workflow.kind = if self.wf_draft().bases.is_empty() {
                        WfKind::General
                    } else {
                        WfKind::Base
                    };
                }
            }
            WfKind::Topic => {
                let wf = self.wf_mut();
                if at < wf.topics.len() {
                    wf.topics.remove(at);
                    self.workflow.topic = at.saturating_sub(1);
                    self.workflow.finish = 0;
                    self.workflow.kind = if self.wf_draft().topics.is_empty() {
                        WfKind::General
                    } else {
                        WfKind::Topic
                    };
                }
            }
            WfKind::Finish => {
                let mut left = None;
                if let Some(tp) = self.wf_mut().topics.get_mut(ti) {
                    if at < tp.finish.len() {
                        tp.finish.remove(at);
                        left = Some(tp.finish.len());
                    }
                }
                if let Some(left) = left {
                    self.workflow.finish = at.saturating_sub(1);
                    self.workflow.kind = if left == 0 {
                        WfKind::Topic
                    } else {
                        WfKind::Finish
                    };
                }
            }
        }
    }

    /// Moves within a group only (the list never offers a cross-group move).
    fn wf_move(&mut self, from: usize, to: usize) {
        let (kind, a) = self.wf_decode(from);
        let (kind_to, b) = self.wf_decode(to);
        if kind != kind_to {
            return;
        }
        let ti = self.wf_current_topic().unwrap_or(0);
        fn shift<T>(v: &mut Vec<T>, a: usize, b: usize) -> bool {
            if a >= v.len() || b >= v.len() {
                return false;
            }
            let item = v.remove(a);
            v.insert(b, item);
            true
        }
        match kind {
            WfKind::General => {}
            WfKind::Base => {
                if shift(&mut self.wf_mut().bases, a, b) {
                    self.workflow.base = b;
                }
            }
            WfKind::Topic => {
                if shift(&mut self.wf_mut().topics, a, b) {
                    self.workflow.topic = b;
                }
            }
            WfKind::Finish => {
                let moved = self
                    .wf_mut()
                    .topics
                    .get_mut(ti)
                    .is_some_and(|tp| shift(&mut tp.finish, a, b));
                if moved {
                    self.workflow.finish = b;
                }
            }
        }
    }

    // -- the hook editor (multi-line) ------------------------------------------

    pub(super) fn wf_open_editor(&mut self, target: WfTarget) {
        let wf = self.wf_draft();
        let text = match target {
            WfTarget::HookAfterStart => wf.hooks.after_start.join("\n"),
            WfTarget::HookBeforeFinish => wf.hooks.before_finish.join("\n"),
            _ => return,
        };
        self.workflow.editor = Some(HookEdit {
            target,
            close: super::editor_close::EditorClose::new(&text),
            text,
            field: {
                let mut field = crate::text_edit::FieldEditor::default();
                field.caret_mut(());
                field
            },
        });
    }

    pub(super) fn wf_editor_open(&self) -> bool {
        self.workflow.editor.is_some()
    }

    fn wf_editor_save(&mut self) {
        self.workflow.editor_hits.clear();
        let Some(edit) = self.workflow.editor.take() else {
            return;
        };
        let lines: Vec<String> = edit
            .text
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .map(str::to_string)
            .collect();
        self.wf_touch();
        let hooks: &mut Hooks = &mut self.wf_mut().hooks;
        match edit.target {
            WfTarget::HookAfterStart => hooks.after_start = lines,
            WfTarget::HookBeforeFinish => hooks.before_finish = lines,
            _ => {}
        }
    }

    fn wf_editor_cancel(&mut self) {
        self.workflow.editor = None;
        self.workflow.editor_hits.clear();
    }

    pub(super) fn wf_editor_key(&mut self, key: KeyEvent) -> bool {
        let Some(edit) = self.workflow.editor.as_mut() else {
            return true;
        };
        match edit.close.key(key, &edit.text) {
            Act::Close => {
                self.wf_editor_cancel();
                return true;
            }
            Act::Spent => return true,
            Act::Pass => {}
        }
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        match key.code {
            KeyCode::Char('s' | 'S' | '\u{13}') if ctrl || key.code == KeyCode::Char('\u{13}') => {
                self.wf_editor_save()
            }
            KeyCode::Enter => edit.field.caret_mut(()).insert(&mut edit.text, "\n"),
            _ => {
                edit.field.caret_mut(()).on_key(&mut edit.text, &key, true);
            }
        }
        true
    }

    pub(super) fn wf_editor_paste(&mut self, text: &str) {
        if let Some(edit) = self.workflow.editor.as_mut() {
            edit.field.caret_mut(()).insert(&mut edit.text, text);
        }
    }

    pub(super) fn wf_editor_mouse(
        &mut self,
        kind: MouseEventKind,
        col: u16,
        row: u16,
        shift: bool,
    ) {
        let Some(edit) = self.workflow.editor.as_mut() else {
            return;
        };
        let event = crossterm::event::MouseEvent {
            kind,
            column: col,
            row,
            modifiers: KeyModifiers::NONE,
        };
        match edit.close.mouse(event, &edit.text) {
            Act::Close => return self.wf_editor_cancel(),
            Act::Spent => return,
            Act::Pass => {}
        }
        if edit.close.asking() {
            return;
        }
        if edit
            .field
            .area_mouse((), &edit.text, kind, (col, row), shift)
        {
            return;
        }
        if kind != MouseEventKind::Down(MouseButton::Left) {
            return;
        }
        match crate::dialog_kit::hit_at(&self.workflow.editor_hits, col, row) {
            Some(EditorButton::Save) => self.wf_editor_save(),
            Some(EditorButton::Cancel) if edit.close.cancel_button(&edit.text) == Act::Close => {
                self.wf_editor_cancel()
            }
            _ => {}
        }
    }

    /// Draws the hook editor over the page, when it is open.
    pub(super) fn render_workflow_editor(&mut self, frame: &mut Frame, area: Rect) {
        self.workflow.editor_hits.clear();
        let mouse = self.mouse_pos;
        let Some(edit) = self.workflow.editor.as_mut() else {
            return;
        };
        let width = (area.width * 4 / 5).max(30).min(area.width);
        let height = (area.height / 2).max(8).min(area.height);
        let rect = WIN.place(
            area,
            Rect::new(
                area.x + (area.width - width) / 2,
                area.y + (area.height - height) / 2,
                width,
                height,
            ),
        );
        let title = match edit.target {
            WfTarget::HookAfterStart => t::HOOK_AFTER_START.get(),
            _ => t::HOOK_BEFORE_FINISH.get(),
        };
        edit.close
            .framed(frame.buffer_mut(), rect, title, &WIN, true);
        super::editor_close::footer(
            frame.buffer_mut(),
            rect,
            &t::editor_footer(crate::dialog_kit::ctrl_key('s')),
        );
        let inner = rect.inner(ratatui::layout::Margin::new(1, 1));
        let text_area = Rect::new(
            inner.x,
            inner.y + 1,
            inner.width,
            inner.height.saturating_sub(3),
        );
        // The warning stays on screen while editing.
        crate::dialog_kit::put(
            frame.buffer_mut(),
            inner.x,
            inner.y,
            t::HOOKS_WARNING.get(),
            inner.width,
            Style::default().fg(th().status.warning),
        );
        let look = crate::text_field::EditLook::new(
            Style::default()
                .fg(th().ui.text)
                .bg(crate::input_focus_bg()),
            "\u{2588}",
        );
        let lines = crate::text_field::edit_lines(
            &edit.text,
            edit.field.caret(()),
            text_area.width.saturating_sub(1) as usize,
            text_area.height as usize,
            true,
            look,
        );
        frame.render_widget(Paragraph::new(lines), text_area);
        // The field's click rect: the same width `edit_lines` wrapped to.
        edit.field.begin_frame();
        edit.field.drawn(
            (),
            Rect::new(
                text_area.x,
                text_area.y,
                text_area.width.saturating_sub(1),
                text_area.height,
            ),
            true,
        );
        // Save / Cancel.
        use crate::dialog_kit::{self, ButtonState};
        let y = inner.bottom().saturating_sub(1);
        let cancel_label = gitpoulpi_i18n::msg::dialog::CANCEL.get();
        let save_label = t::EDITOR_SAVE.get();
        let cancel_w = dialog_kit::button_width(cancel_label);
        let save_w = dialog_kit::button_width(save_label);
        let cancel_x = inner.right().saturating_sub(cancel_w);
        let save_x = cancel_x.saturating_sub(save_w + 1);
        let hovered = |r: Rect| mouse.is_some_and(|(c, row)| r.contains((c, row).into()));
        let save_rect = Rect::new(save_x, y, save_w, 1);
        let cancel_rect = Rect::new(cancel_x, y, cancel_w, 1);
        let buf = frame.buffer_mut();
        dialog_kit::button(
            buf,
            save_x,
            y,
            save_label,
            ButtonState::Primary {
                hovered: hovered(save_rect),
                focused: false,
            },
        );
        dialog_kit::button(
            buf,
            cancel_x,
            y,
            cancel_label,
            ButtonState::Normal {
                hovered: hovered(cancel_rect),
                focused: false,
            },
        );
        self.workflow.editor_hits = vec![
            (save_rect, EditorButton::Save),
            (cancel_rect, EditorButton::Cancel),
        ];
        if let Some(edit) = self.workflow.editor.as_mut() {
            edit.close.render(frame.buffer_mut());
        }
    }
}

/// The editor resizes: its text area takes the whole rect, so it grows and
/// shrinks with it.
const WIN: crate::dialog_kit::Win =
    crate::dialog_kit::Win::resizable("settings_workflow_editor", 40, 10);

#[cfg(test)]
mod tests {
    use super::*;
    use gitpoulpi_core::workflow::WorkflowTemplate;

    fn gitflow() -> Workflow {
        WorkflowTemplate::GitFlow.workflow()
    }

    #[test]
    fn an_unchanged_draft_is_an_empty_override() {
        let shared = gitflow();
        assert!(override_from(&shared, &shared).is_empty());
    }

    #[test]
    fn the_override_is_what_merged_with_turns_the_shared_into_the_draft() {
        let shared = gitflow();
        let mut draft = shared.clone();
        draft.bases[1].protected = true;
        draft.topics[0].pattern = "feat/{slug}".into();
        draft.topics.retain(|t| t.id != "support");
        draft.topics.push(TopicType {
            id: "chore".into(),
            pattern: "chore/{slug}".into(),
            ..Default::default()
        });
        draft.hooks.before_finish = vec!["cargo test".into()];
        let ovr = override_from(&shared, &draft);
        assert_eq!(ovr.disable_topics, ["support"]);
        assert_eq!(ovr.bases.len(), 1);
        let merged = shared.merged_with(&ovr);
        assert_eq!(merged, draft);
    }

    #[test]
    fn lists_round_trip() {
        assert_eq!(parse_list(" a, b ,,c "), ["a", "b", "c"]);
        assert_eq!(list_text(&parse_list("x,y")), "x, y");
    }
}
