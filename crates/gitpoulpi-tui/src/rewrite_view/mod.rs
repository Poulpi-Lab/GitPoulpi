//! The history rewrite workbench: the model (this slice) behind the view
//! that rewrites authors, dates, messages, content and paths over all
//! history or a selection (the history-rewrite design, "UX").
//!
//! [`RewriteView`] owns everything the screen shows and every decision a key
//! or a click makes; the renderer (slice S8) only reads it and calls its
//! methods, so keys, mouse and palette cannot drift. It is an
//! `InputMode::Rewrite`: it owns the keyboard while open.
//!
//! - **The recipe** (`recipe`) is edited in place: `rules` (list operations,
//!   `model.rs`), the detail pane of the rule under the cursor
//!   (`rules_form.rs`) and the scope bar (`scope.rs`). Anything that changes
//!   it makes the preview on screen out of date.
//! - **Preview, Apply** run in the daemon (cancellable jobs). Nothing here
//!   waits: every request is a [`Reply`] polled each frame by [`poll`].
//! - **Questions** that have no default and are asked each time (maintainer
//!   decisions, 2026-10-21): what happens to the signatures of signed
//!   commits that change, how a subdirectory is extracted, and the go-ahead
//!   for rewriting refs that are already pushed. Apply and Preview stay
//!   unsent until they are answered; answers are never saved in a recipe.
//! - **Recipes** are saved shared with the repository or only for this user
//!   (`gitpoulpi-session`); secret values are never written.
//!
//! [`poll`]: RewriteView::poll

// The renderer (slice S8) is the other half of this API: until it lands, part
// of what the model offers is only reached by the tests.
#![allow(dead_code, unused_imports)]

mod bars;
mod drill;
mod head;
mod host;
mod identities;
mod model;
mod paths;
mod popup;
mod preview;
mod purge;
mod push;
mod render;
mod rules_ai;
mod rules_form;
mod scope;
mod script;
mod script_ai;
mod secrets;
mod signing;
mod size;
mod tools;

#[cfg(test)]
mod tests;

use crate::keymap::term::TermKeymap;
use std::path::PathBuf;
use std::time::Instant;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use gitpoulpi_core::domain::rewrite::{
    ExtractMode, PathMove, PruneEmpty, Recipe, RewriteOutcome, RewritePlan, RewriteProgress,
    RewriteRefusal, RuleKind, SignaturePolicy, SigningFormat,
};
use gitpoulpi_core::domain::OperationKind;
use gitpoulpi_core::Error as CoreError;
use gitpoulpi_i18n::{msg::rewrite_view as t, M};
use gitpoulpi_ipc::rewrite::{ApplyArgs, DiscardArgs, RowFilter, RowsArgs, RowsPage, ToolsArgs};
use gitpoulpi_session::RecipePlace;
use ratatui::layout::Rect;

use crate::daemon_link::{Reply, Requester};
use crate::keymap::Action as Bind;
use crate::text_edit::{Caret, KeyEdit};

pub(crate) use bars::BarId;
pub(crate) use drill::{Drill, DrillDiff};
pub(crate) use host::{HostHandle, HostState};
pub(crate) use identities::{Candidate, IdentitiesState, MergePick};
pub(crate) use model::RuleTemplateExt;
pub(crate) use model::{
    detail_of, rows as rule_rows, template_of, RuleGroup, RuleList, RuleRow, RuleTemplate,
    Validation,
};
pub(crate) use popup::{PopupRow, ROW_FILTERS};
pub(crate) use purge::{PurgeConfirm, PurgeScreen};
pub(crate) use push::{PushPanel, PushPick};
pub(crate) use render::{rewrite_hit_at, RewriteHit};
pub(crate) use rules_ai::{ProposalProblem, ProposalSettings, RulesCtx, RulesProposal};
pub(crate) use rules_form::{DetailForm, Field, FieldId, FieldInput, FieldValue};
pub(crate) use scope::{ProtectedEntry, RefChoice, ScopeContext, ScopeCounts, ScopeKind};
pub(crate) use script::{CheckView, ScriptFocus, ScriptState, TestState, TEST_SAMPLE};
pub(crate) use script_ai::{AiSystem, AiWish, SampleRow, SAMPLE_MAX};
pub(crate) use secrets::{ScanResult, SecretsState};
pub(crate) use size::{SizeResult, SizeState};
pub(crate) use tools::{MissingTool, ToolsState};

/// Rows asked of the daemon per request.
const PAGE: usize = 200;
/// A row cursor this close to the end of what is loaded asks for more.
const LOOKAHEAD: usize = 40;

mod vocab;
pub(crate) use vocab::RewriteEntry;

/// The pane that has the keyboard (`Tab` cycles them).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Focus {
    Scope,
    Rules,
    Detail,
    Preview,
}

impl Focus {
    const ORDER: [Focus; 4] = [Focus::Scope, Focus::Rules, Focus::Detail, Focus::Preview];
}

/// The tabs of the preview pane.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PreviewTab {
    Summary,
    Commits,
    Refs,
    Impact,
    /// Scan the scope for secrets (S14); works without a preview.
    Secrets,
    /// Every name and email of the scope, and "Merge into..." (S16, the
    /// `.mailmap`); works without a preview.
    Identities,
    /// Sizes of the history and rows that become removal rules (S18).
    Size,
}

impl PreviewTab {
    pub(crate) const ALL: [PreviewTab; 7] = [
        PreviewTab::Summary,
        PreviewTab::Commits,
        PreviewTab::Refs,
        PreviewTab::Impact,
        PreviewTab::Secrets,
        PreviewTab::Identities,
        PreviewTab::Size,
    ];

    pub(crate) const fn label(self) -> M {
        match self {
            PreviewTab::Summary => t::TAB_SUMMARY,
            PreviewTab::Commits => t::TAB_COMMITS,
            PreviewTab::Refs => t::TAB_REFS,
            PreviewTab::Impact => t::TAB_IMPACT,
            PreviewTab::Secrets => gitpoulpi_i18n::msg::rewrite_secrets::TAB,
            PreviewTab::Identities => gitpoulpi_i18n::msg::rewrite_identities::TAB,
            PreviewTab::Size => gitpoulpi_i18n::msg::rewrite_size::TAB,
        }
    }
}

/// How the plan on screen relates to the recipe.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PlanState {
    /// No preview yet.
    None,
    /// The plan is the recipe's.
    Current,
    /// The recipe changed since: Preview again before Apply.
    OutOfDate,
    /// The repository changed since (the daemon said so, or the run is gone).
    Stale,
}

/// What the daemon is doing for this window.
#[derive(Default)]
pub(crate) enum Run {
    #[default]
    Idle,
    Previewing {
        reply: Reply<RewritePlan>,
        since: Instant,
        /// The daemon job, known from the first progress report.
        job: Option<u64>,
        progress: Option<RewriteProgress>,
    },
    Applying {
        reply: Reply<RewriteOutcome>,
        since: Instant,
    },
    /// An apply finished: the result panel replaces the preview.
    Done(RewriteOutcome),
}

impl Run {
    pub(crate) fn is_busy(&self) -> bool {
        matches!(self, Run::Previewing { .. } | Run::Applying { .. })
    }
}

/// The three signature choices, in the order they are offered.
pub(crate) const SIGNATURE_CHOICES: [(SignaturePolicy, M); 3] = [
    (SignaturePolicy::Strip, t::Q_SIGN_STRIP),
    (SignaturePolicy::ResignSigned, t::Q_SIGN_RESIGN_SIGNED),
    (SignaturePolicy::ResignAll, t::Q_SIGN_RESIGN_ALL),
];

/// The three ways to extract a subdirectory, in the order they are offered.
pub(crate) const EXTRACT_CHOICES: [(ExtractMode, M); 3] = [
    (ExtractMode::Clone, t::Q_EXTRACT_CLONE),
    (ExtractMode::InPlace, t::Q_EXTRACT_IN_PLACE),
    (ExtractMode::Submodule, t::Q_EXTRACT_SUBMODULE),
];

/// A question the run waits on. There is no default answer to any of them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Question {
    /// Signed commits change: strip, re-sign the signed ones, or re-sign all.
    /// `count` is how many, when the daemon said.
    Signatures { count: Option<u64>, cursor: usize },
    /// How the subdirectory is extracted.
    ExtractMode { cursor: usize },
    /// The run moves refs that are already on a remote.
    ConfirmPushed { prompt: String },
    /// An external script (not sandboxed) that nobody trusted yet: trust it
    /// (kept for its exact text) or not. No answer is preselected.
    TrustScript {
        hash: String,
        then: script::AfterTrust,
        cursor: Option<usize>,
    },
}

/// What the renderer draws for the open question.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct QuestionView {
    pub title: M,
    pub prompt: String,
    pub options: Vec<M>,
    pub cursor: usize,
}

/// The answers given for the run on screen. Cleared when the run ends or the
/// window closes; never part of a saved recipe.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct Choices {
    pub signatures: Option<SignaturePolicy>,
    pub extract_mode: Option<ExtractMode>,
}

/// A line under the preview: information or a refusal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Notice {
    pub text: String,
    pub error: bool,
}

/// A refusal that has more to say than a line: what is in the way, as a
/// list, shown in the preview pane until something changes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Refusal {
    pub kind: RewriteRefusal,
    /// The paths or names the daemon listed, one per line.
    pub lines: Vec<String>,
}

/// What a key asks of the caller.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RewriteKeyOutcome {
    Stay,
    Close,
    /// The result panel's Undo (the caller runs the journal's undo).
    Undo,
    /// A rejected push row's "Fetch and review": the caller closes the
    /// workbench and fetches the remotes, so the Log shows what moved.
    FetchReview,
}

/// A saved recipe offered by Load.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SavedRecipe {
    pub name: String,
    pub place: RecipePlace,
    pub path: PathBuf,
}

/// A popup list over the workbench.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Popup {
    /// The Add rule menu, a cursor over `RuleTemplate::ALL`.
    AddRule { cursor: usize },
    Load {
        items: Vec<SavedRecipe>,
        cursor: usize,
    },
    /// The values of choice field `field` of the detail pane.
    Choice { field: usize, cursor: usize },
    /// The scope kinds.
    Scope { cursor: usize },
    /// Where Save writes: the repository (shared) or only for the user.
    Save { cursor: usize },
    /// Which commits the Commits tab lists.
    Filter { cursor: usize },
    /// The checklist of branches and tags of a refs scope.
    Refs { cursor: usize },
}

/// Where recipes are saved.
#[derive(Debug, Clone, Default)]
pub(crate) struct RecipeStore {
    pub repo_root: PathBuf,
    /// The user's config dir (`None` when the OS gave none: only-for-me
    /// saves are then refused).
    pub config_dir: Option<PathBuf>,
}

impl RecipeStore {
    fn dir(&self, place: RecipePlace) -> Option<PathBuf> {
        match place {
            RecipePlace::Shared => Some(gitpoulpi_session::shared_recipes_dir(&self.repo_root)),
            RecipePlace::Mine => self
                .config_dir
                .as_deref()
                .map(gitpoulpi_session::my_recipes_dir),
        }
    }
}

/// The commits listed on the Commits tab, paged in from the daemon.
#[derive(Default)]
pub(crate) struct RowsState {
    pub filter: RowFilter,
    /// Rows matching the filter in the whole plan (as the daemon counted).
    pub total: usize,
    pub loaded: Vec<gitpoulpi_core::domain::rewrite::CommitChange>,
    /// The cursor, an index into the rows.
    pub cursor: usize,
    pending: Option<Reply<RowsPage>>,
}

/// What is being typed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EditTarget {
    /// Field `.0` of the detail pane.
    Field(usize),
    /// The recipe's name.
    Name,
}

pub(crate) struct Editing {
    pub target: EditTarget,
    pub text: String,
    pub caret: Caret,
    multiline: bool,
    /// What Esc puts back.
    snapshot: Snapshot,
}

enum Snapshot {
    Rule(Box<gitpoulpi_core::domain::rewrite::Rule>),
    Name(String),
}

pub(crate) struct RewriteView {
    pub recipe: Recipe,
    pub rules: RuleList,
    pub detail: DetailForm,
    pub scope_ctx: ScopeContext,
    pub focus: Focus,
    pub tab: PreviewTab,
    pub tools: ToolsState,
    /// A tool check on its way (the last answer stays in `tools`).
    tools_wait: Option<Reply<gitpoulpi_core::domain::rewrite::RewriteTools>>,
    /// The "Check signing" smoke test on its way, and its last answer.
    sign_wait: Option<Reply<gitpoulpi_core::domain::rewrite::SigningCheck>>,
    sign_check: Option<gitpoulpi_core::domain::rewrite::SigningCheck>,
    pub run: Run,
    pub plan: Option<RewritePlan>,
    pub plan_state: PlanState,
    pub rows: RowsState,
    /// The cursor of the Refs tab's protected-branch list.
    pub refs_cursor: usize,
    pub question: Option<Question>,
    pub choices: Choices,
    pub editing: Option<Editing>,
    pub popup: Option<Popup>,
    pub notice: Option<Notice>,
    /// A refusal shown in the preview pane (dirty files, a branch checked out
    /// elsewhere, a partial clone).
    pub refusal: Option<Refusal>,
    /// The blob diff overlay.
    pub drill: Option<Drill>,
    /// The Script rule's editor overlay, check and test.
    pub script: ScriptState,
    /// The ✦ "Create Rules with AI" dialog (`rules_ai.rs`).
    rules_ai: rules_ai::RulesAiState,
    /// The result panel's push list, note and maps (S13).
    pub push: PushPanel,
    /// What the host protects, and the how-to overlay (S23).
    pub host: HostState,
    /// The Secrets tab: the scan and its findings (S14).
    pub secrets: SecretsState,
    /// The Identities tab: the names and emails of the scope and the
    /// `.mailmap` merge (S16).
    pub identities: IdentitiesState,
    /// The Size tab: the analysis and its rows (S18).
    pub size: SizeState,
    /// The purge screen over the workbench (S15).
    pub purge: Option<PurgeScreen>,
    /// Where an extraction into a new repository goes (S20).
    paths: paths::PathsState,
    /// The last press on a Commits row, for the double click.
    last_row_press: Option<(usize, Instant)>,
    /// The repository's `gpg.format`, for the tools a re-sign needs.
    pub signing_format: SigningFormat,
    store: RecipeStore,
    requester: Requester,
    /// What the last paint learned (windows, the frame).
    paint: render::Paint,
    /// `Ctrl+N` was pressed: the controls show a badge and the next key is
    /// one (a press is a click on the control).
    pub badges: bool,
}

impl RewriteView {
    /// A new workbench on a fresh recipe whose scope is the current branch.
    /// Asks the daemon for the tool check.
    pub(crate) fn open(requester: Requester, store: RecipeStore, scope_ctx: ScopeContext) -> Self {
        let mut view = Self {
            recipe: Recipe::new(
                "",
                gitpoulpi_core::domain::rewrite::Scope::CurrentBranch { since: None },
                Vec::new(),
            ),
            rules: RuleList::default(),
            detail: DetailForm::default(),
            scope_ctx,
            focus: Focus::Rules,
            tab: PreviewTab::Summary,
            tools: ToolsState::Unknown,
            tools_wait: None,
            sign_wait: None,
            sign_check: None,
            run: Run::Idle,
            plan: None,
            plan_state: PlanState::None,
            rows: RowsState::default(),
            refs_cursor: 0,
            question: None,
            choices: Choices::default(),
            editing: None,
            popup: None,
            notice: None,
            refusal: None,
            drill: None,
            script: ScriptState::default(),
            rules_ai: rules_ai::RulesAiState::default(),
            push: PushPanel::default(),
            host: HostState::default(),
            secrets: SecretsState::default(),
            identities: IdentitiesState::default(),
            size: SizeState::default(),
            purge: None,
            paths: paths::PathsState::default(),
            last_row_press: None,
            signing_format: SigningFormat::from_config(None),
            store,
            requester,
            paint: render::Paint::default(),
            badges: false,
        };
        view.request_tools(false);
        if view.scope_ctx.use_selection {
            scope::set_kind(&mut view.recipe, &view.scope_ctx, ScopeKind::Selection);
        } else if view.scope_ctx.since.is_some() {
            scope::set_since(&mut view.recipe, view.scope_ctx.since);
        }
        view
    }

    /// The workbench open on an agent's recipe (a proposal, S25): the rules
    /// are filled in and nothing else happens. The user previews and applies
    /// (`apply` is the only path that tells the proposal it was accepted).
    pub(crate) fn with_proposed(mut self, recipe: Recipe, client: &str) -> Self {
        self.recipe = recipe;
        self.rules.reset();
        self.detail = DetailForm::default();
        self.sync_detail();
        self.notice = Some(Notice {
            text: t::proposed_notice(client),
            error: false,
        });
        self
    }

    /// The workbench on its Identities tab (the palette's
    /// `Rewrite: Merge Identities (.mailmap)...`).
    pub(crate) fn with_identities(mut self) -> Self {
        self.focus = Focus::Preview;
        self.tab = PreviewTab::Identities;
        self.load_identities();
        self
    }

    /// The workbench with one rule already added and its fields focused (the
    /// palette's `Rewrite: ...` rows).
    pub(crate) fn with_rule(mut self, template: RuleTemplate) -> Self {
        if template.disabled_reason().is_some() {
            self.notice = template.disabled_reason().map(|why| Notice {
                text: format!("{}: {}", template.label().get(), why.get()),
                error: true,
            });
            return self;
        }
        self.add_rule(template);
        self.focus = Focus::Detail;
        if template == RuleTemplate::Script {
            self.open_script_editor();
        }
        self
    }

    // ------------------------------------------------------------ queries

    /// The question the run waits on, ready to draw.
    pub(crate) fn question_view(&self) -> Option<QuestionView> {
        Some(match self.question.as_ref()? {
            Question::Signatures { count, cursor } => QuestionView {
                title: t::Q_SIGN_TITLE,
                prompt: {
                    let base = match count {
                        Some(n) => format!("{}  {}", t::q_sign_count(n), t::Q_SIGN_PROMPT.get()),
                        None => t::Q_SIGN_PROMPT.get().to_string(),
                    };
                    match self.sign_check_text(*count) {
                        Some(extra) => format!("{base}  {extra}"),
                        None => base,
                    }
                },
                // The last row only checks that signing works.
                options: SIGNATURE_CHOICES
                    .iter()
                    .map(|(_, m)| *m)
                    .chain([t::Q_SIGN_CHECK])
                    .collect(),
                cursor: *cursor,
            },
            Question::ExtractMode { cursor } => QuestionView {
                title: t::Q_EXTRACT_TITLE,
                prompt: paths::extract_prompt(*cursor),
                options: EXTRACT_CHOICES.iter().map(|(_, m)| *m).collect(),
                cursor: *cursor,
            },
            Question::ConfirmPushed { prompt } => QuestionView {
                title: t::Q_PUSHED_TITLE,
                prompt: prompt.clone(),
                options: vec![t::Q_PUSHED_YES, t::Q_PUSHED_NO],
                cursor: 0,
            },
            Question::TrustScript { hash, cursor, .. } => QuestionView {
                title: t::Q_TRUST_TITLE,
                prompt: format!(
                    "{}  (sha256 {})",
                    t::Q_TRUST_PROMPT.get(),
                    hash.get(..12).unwrap_or(hash)
                ),
                options: vec![t::Q_TRUST_YES, t::Q_TRUST_NO],
                // Past the last row: nothing is highlighted, so Enter has
                // nothing to pick until the user moves or presses 1 or 2.
                cursor: cursor.unwrap_or(usize::MAX),
            },
        })
    }

    /// The rows of the rules list.
    pub(crate) fn rule_rows(&self) -> Vec<RuleRow> {
        model::rows(&self.recipe.rules)
    }

    /// What the recipe lacks, worded for the footer.
    pub(crate) fn validation(&self) -> Validation {
        model::validation(&self.recipe)
    }

    /// Whether the rule at `index` needs a tool the machine lacks.
    pub(crate) fn missing_tools(&self, index: usize) -> Vec<MissingTool> {
        self.recipe
            .rules
            .get(index)
            .map(|r| tools::missing_for_rule(&self.tools, r))
            .unwrap_or_default()
    }

    /// The tools the whole recipe needs that are missing.
    pub(crate) fn missing_tools_for_run(&self) -> Vec<MissingTool> {
        tools::missing_for_recipe(&self.tools, &self.request_recipe(), self.signing_format)
    }

    /// A daemon job of this window is running (preview, apply, scan, push).
    pub(crate) fn busy(&self) -> bool {
        self.run.is_busy()
            || self.scanning()
            || self.analysing()
            || self.pushing()
            || self.purging()
    }

    /// The protected branches of the plan on screen, ticked as the recipe
    /// says.
    pub(crate) fn protected(&self) -> Vec<ProtectedEntry> {
        self.plan
            .as_ref()
            .map(|p| scope::protected_entries(&self.recipe, p))
            .unwrap_or_default()
    }

    /// The numbers of the scope bar's tail, from the last preview.
    pub(crate) fn scope_counts(&self) -> Option<ScopeCounts> {
        self.plan.as_ref().map(scope::counts)
    }

    pub(crate) fn scope_kind(&self) -> ScopeKind {
        ScopeKind::of(&self.recipe.scope)
    }

    pub(crate) fn scope_kinds(&self) -> Vec<ScopeKind> {
        scope::kinds_offered(&self.scope_ctx, &self.recipe)
    }

    pub(crate) fn ref_choices(&self) -> Vec<RefChoice> {
        scope::ref_choices(&self.recipe, &self.scope_ctx)
    }

    /// The first reason Apply is not available, `None` when it is. The
    /// footer shows it while Apply is disabled.
    pub(crate) fn apply_blocker(&self) -> Option<String> {
        if self.busy() {
            return Some(t::B_BUSY.get().to_string());
        }
        if let Validation::Invalid { text, .. } = self.validation() {
            return Some(text);
        }
        match (&self.plan, self.plan_state) {
            (None, _) | (_, PlanState::None) => Some(t::B_PREVIEW_FIRST.get().to_string()),
            (Some(_), PlanState::OutOfDate) => Some(t::B_PREVIEW_AGAIN.get().to_string()),
            (Some(_), PlanState::Stale) => Some(t::B_STALE.get().to_string()),
            (Some(plan), PlanState::Current) => {
                (plan.counts.commits_changed == 0).then(|| t::B_NOTHING_CHANGES.get().to_string())
            }
        }
    }

    pub(crate) fn can_apply(&self) -> bool {
        self.apply_blocker().is_none() && !matches!(self.run, Run::Done(_))
    }

    pub(crate) fn can_preview(&self) -> bool {
        !self.busy()
            && matches!(
                self.validation(),
                Validation::Ok | Validation::NeedsExtractMode
            )
    }

    /// The recipe as the daemon gets it: the answers of this run filled in.
    pub(crate) fn request_recipe(&self) -> Recipe {
        let mut recipe = self.recipe.clone();
        // S23: what the host protects (never saved with the recipe).
        recipe.scope_options.host_protection = self.host().filter(|h| h.asked).cloned();
        if recipe.options.signatures.is_none() {
            recipe.options.signatures = self.choices.signatures;
        }
        if let Some(mode) = self.choices.extract_mode {
            for rule in &mut recipe.rules {
                if let RuleKind::MovePaths(PathMove::ExtractSubdir { mode: m, .. }) = &mut rule.kind
                {
                    m.get_or_insert(mode);
                }
            }
        }
        recipe
    }

    /// The recipe as it is written to disk: no secret, no answer of a run.
    fn saved_recipe(&self) -> Recipe {
        let mut recipe = self.recipe.saved();
        recipe.options.signatures = None;
        for rule in &mut recipe.rules {
            if let RuleKind::MovePaths(PathMove::ExtractSubdir { mode, .. }) = &mut rule.kind {
                *mode = None;
            }
        }
        recipe
    }

    // ------------------------------------------------------- recipe edits

    /// A recipe edit: the plan on screen no longer matches it.
    fn touch(&mut self) {
        if self.plan.is_some() && self.plan_state == PlanState::Current {
            self.plan_state = PlanState::OutOfDate;
        }
        if matches!(self.run, Run::Done(_)) {
            self.run = Run::Idle;
        }
        self.notice = None;
        self.refusal = None;
    }

    /// Re-lists the detail pane when the cursor moved to another rule.
    fn sync_detail(&mut self) {
        self.rules.clamp(&self.recipe.rules);
        match self.recipe.rules.get(self.rules.cursor) {
            Some(rule) => {
                let prune = self.recipe.options.prune_empty;
                if self.detail.rule != self.rules.cursor || self.detail.fields.is_empty() {
                    self.detail = DetailForm::for_rule(self.rules.cursor, rule);
                }
                self.detail.run_prune = prune;
                self.detail.run_hashes = self.recipe.options.update_message_hashes;
                self.detail.rebuild(rule);
            }
            None => self.detail = DetailForm::default(),
        }
    }

    pub(crate) fn add_rule(&mut self, template: RuleTemplate) {
        self.rules.add(&mut self.recipe.rules, template);
        self.detail = DetailForm::default();
        self.sync_detail();
        self.touch();
    }

    pub(crate) fn toggle_rule(&mut self) {
        if self.rules.toggle(&mut self.recipe.rules) {
            self.touch();
        }
    }

    pub(crate) fn move_rule(&mut self, delta: isize) {
        if self.rules.move_rule(&mut self.recipe.rules, delta) {
            self.detail = DetailForm::default();
            self.sync_detail();
            self.touch();
        }
    }

    pub(crate) fn delete_rule(&mut self) {
        if self.rules.remove(&mut self.recipe.rules) {
            self.detail = DetailForm::default();
            self.sync_detail();
            self.touch();
        }
    }

    pub(crate) fn restore_rule(&mut self) {
        if self.rules.restore(&mut self.recipe.rules) {
            self.detail = DetailForm::default();
            self.sync_detail();
            self.touch();
        }
    }

    pub(crate) fn duplicate_rule(&mut self) {
        if self.rules.duplicate(&mut self.recipe.rules) {
            self.detail = DetailForm::default();
            self.sync_detail();
            self.touch();
        }
    }

    pub(crate) fn move_rule_cursor(&mut self, delta: isize) {
        self.rules.move_cursor(&self.recipe.rules, delta);
        self.detail = DetailForm::default();
        self.sync_detail();
    }

    /// Puts `input` into field `at` of the detail pane.
    pub(crate) fn edit_field(&mut self, at: usize, input: FieldInput) {
        if self.language_refused(at, &input) {
            return;
        }
        if self.detail.fields.get(at).map(|f| f.id) == Some(FieldId::PruneEmpty) {
            if let FieldInput::Choice(i) = input {
                self.recipe.options.prune_empty = if i == 0 {
                    PruneEmpty::Auto
                } else {
                    PruneEmpty::Never
                };
                self.sync_detail();
                self.touch();
            }
            return;
        }
        if self.detail.fields.get(at).map(|f| f.id) == Some(FieldId::UpdateHashes) {
            if let FieldInput::Toggle(on) = input {
                self.recipe.options.update_message_hashes = on;
                self.sync_detail();
                self.touch();
            }
            return;
        }
        let Some(rule) = self.recipe.rules.get_mut(self.rules.cursor) else {
            return;
        };
        if self.detail.edit(rule, at, input) {
            self.touch();
        }
    }

    /// Cycles a choice field by `delta` (wrapping), or flips a toggle.
    pub(crate) fn step_field(&mut self, at: usize, delta: isize) {
        let input = match self.detail.fields.get(at).map(|f| &f.value) {
            Some(FieldValue::Choice { options, index }) => {
                let n = options.len() as isize;
                FieldInput::Choice(((*index as isize + delta).rem_euclid(n)) as usize)
            }
            Some(FieldValue::Toggle(on)) => FieldInput::Toggle(!on),
            _ => return,
        };
        self.edit_field(at, input);
    }

    pub(crate) fn set_name(&mut self, name: &str) {
        self.recipe.name = name.to_string();
    }

    pub(crate) fn set_scope_kind(&mut self, kind: ScopeKind) {
        if scope::set_kind(&mut self.recipe, &self.scope_ctx, kind) {
            self.touch();
        }
    }

    /// Steps the scope kind through the ones on offer.
    pub(crate) fn step_scope_kind(&mut self, delta: isize) {
        let kinds = self.scope_kinds();
        let at = kinds
            .iter()
            .position(|k| *k == self.scope_kind())
            .unwrap_or(0) as isize;
        let next = kinds[(at + delta).rem_euclid(kinds.len() as isize) as usize];
        self.set_scope_kind(next);
    }

    pub(crate) fn toggle_tags(&mut self) {
        scope::toggle_tags(&mut self.recipe);
        self.touch();
    }

    pub(crate) fn toggle_ref(&mut self, name: &str) {
        if scope::toggle_ref(&mut self.recipe, name) {
            self.touch();
        }
    }

    /// Ticks or unticks protected branch `name`.
    pub(crate) fn toggle_protected(&mut self, name: &str) {
        scope::toggle_protected(&mut self.recipe, name);
        self.touch();
    }

    /// "Check all": every protected branch of the preview is included.
    pub(crate) fn check_all_protected(&mut self) {
        if let Some(plan) = &self.plan {
            if scope::check_all_protected(&mut self.recipe, plan) {
                self.touch();
            }
        }
    }

    /// "Uncheck all": every protected branch is left out again.
    pub(crate) fn uncheck_all_protected(&mut self) {
        if scope::uncheck_all_protected(&mut self.recipe) {
            self.touch();
        }
    }

    // --------------------------------------------------------- questions

    /// Answers the open question with its `choice`-th option.
    pub(crate) fn answer(&mut self, choice: usize) {
        if matches!(self.question, Some(Question::Signatures { .. }))
            && choice == SIGNATURE_CHOICES.len()
        {
            self.request_sign_check();
            return;
        }
        match self.question.take() {
            Some(Question::Signatures { .. }) => {
                if let Some((policy, _)) = SIGNATURE_CHOICES.get(choice) {
                    self.choices.signatures = Some(*policy);
                    self.preview();
                }
            }
            Some(Question::ExtractMode { .. }) => {
                if let Some((mode, _)) = EXTRACT_CHOICES.get(choice) {
                    self.choose_extract_mode(*mode);
                }
            }
            Some(Question::ConfirmPushed { .. }) if choice == 0 => self.send_apply(true),
            Some(Question::ConfirmPushed { .. }) => {}
            Some(Question::TrustScript { hash, then, .. }) if choice == 0 => {
                self.trust_script(hash, then)
            }
            Some(Question::TrustScript { .. }) => {}
            None => {}
        }
    }

    /// Leaves the question unanswered; nothing was sent.
    pub(crate) fn dismiss_question(&mut self) {
        self.question = None;
    }

    // ------------------------------------------------------ daemon calls

    fn request_tools(&mut self, fresh: bool) {
        self.tools_wait = Some(self.requester.call(
            OperationKind::RewriteTools,
            serde_json::to_value(ToolsArgs { fresh }).unwrap_or_default(),
        ));
    }

    /// A tool check is on its way.
    pub(crate) fn tools_loading(&self) -> bool {
        self.tools_wait.is_some()
    }

    /// "Check again": probes the tools afresh.
    pub(crate) fn check_tools_again(&mut self) {
        self.request_tools(true);
    }

    /// Runs the preview, unless a question has to be answered first.
    pub(crate) fn preview(&mut self) {
        if self.busy() {
            return;
        }
        self.notice = None;
        self.refusal = None;
        self.host.refused = false;
        match self.validation() {
            Validation::Invalid { text, .. } => {
                self.notice = Some(Notice { text, error: true });
                return;
            }
            Validation::NeedsExtractMode if self.choices.extract_mode.is_none() => {
                self.question = Some(Question::ExtractMode { cursor: 0 });
                return;
            }
            _ => {}
        }
        let missing = self.missing_tools_for_run();
        if let Some(first) = missing.first() {
            self.notice = Some(Notice {
                text: t::tool_missing(first.tool.name(), first.hint.get()),
                error: true,
            });
            return;
        }
        let reply = self.requester.call(
            OperationKind::RewritePreview,
            serde_json::json!({
                "recipe": self.request_recipe(),
                "vouched": self.vouched_hashes(),
            }),
        );
        self.run = Run::Previewing {
            reply,
            since: Instant::now(),
            job: None,
            progress: None,
        };
        self.rows = RowsState::default();
    }

    /// Cancels the running preview or scan (the job is the daemon's).
    pub(crate) fn cancel(&mut self) {
        if let Run::Previewing { job: Some(job), .. } = &self.run {
            crate::jobs_ui::cancel(*job);
        }
        self.cancel_scan();
        self.cancel_analysis();
    }

    /// Applies the previewed plan. A plan that moves pushed refs first
    /// becomes the question "rewrite them anyway?" (the daemon insists).
    pub(crate) fn apply(&mut self) {
        if let Some(why) = self.apply_blocker() {
            self.notice = Some(Notice {
                text: why,
                error: true,
            });
            return;
        }
        self.send_apply(false);
    }

    fn send_apply(&mut self, confirmed: bool) {
        let Some(plan) = &self.plan else {
            return;
        };
        let (run, digest) = (plan.run.clone(), plan.digest.clone());
        // The user's own Apply: tells an open proposal what was accepted.
        crate::ui_intent::proposal::confirmed(
            crate::ui_intent::proposal::Dialog::Rewrite,
            &serde_json::to_string(&self.recipe).unwrap_or_default(),
        );
        let args = ApplyArgs {
            run,
            digest,
            confirmed,
            target: self.extract_target(),
        };
        let reply = self.requester.call(
            OperationKind::RewriteApply,
            serde_json::to_value(args).unwrap_or_default(),
        );
        self.run = Run::Applying {
            reply,
            since: Instant::now(),
        };
        self.notice = None;
    }

    /// Drops the plan on screen and its run in the daemon.
    pub(crate) fn discard(&mut self) {
        if let Some(plan) = self.plan.take() {
            let _: Reply<()> = self.requester.call(
                OperationKind::RewriteDiscard,
                serde_json::to_value(DiscardArgs { run: plan.run }).unwrap_or_default(),
            );
        }
        self.plan_state = PlanState::None;
        self.rows = RowsState::default();
        self.run = Run::Idle;
    }

    /// Chooses which commits the Commits tab lists.
    pub(crate) fn set_row_filter(&mut self, filter: RowFilter) {
        if self.rows.filter == filter {
            return;
        }
        self.rows = RowsState {
            filter,
            ..RowsState::default()
        };
        self.want_rows(0);
    }

    /// Asks for more rows when `upto` (a row index the screen is about to
    /// show) is near the end of what is loaded. Call it from the frame that
    /// draws the Commits tab.
    pub(crate) fn want_rows(&mut self, upto: usize) {
        let Some(plan) = &self.plan else {
            return;
        };
        let loaded = self.rows.loaded.len();
        let more = loaded == 0 && self.rows.total == 0 || loaded < self.rows.total;
        if self.rows.pending.is_some() || !more || upto + LOOKAHEAD < loaded {
            return;
        }
        let args = RowsArgs {
            run: plan.run.clone(),
            filter: self.rows.filter,
            offset: loaded,
            limit: PAGE,
        };
        self.rows.pending = Some(self.requester.call(
            OperationKind::RewriteRows,
            serde_json::to_value(args).unwrap_or_default(),
        ));
    }

    /// Collects every answer that has come. Called once per frame; never
    /// waits.
    pub(crate) fn poll(&mut self, progress: Option<(u64, RewriteProgress)>) {
        self.page_rows();
        self.poll_drill();
        self.poll_host();
        self.poll_script(Instant::now());
        self.poll_push();
        self.poll_purge();
        self.poll_secrets(progress);
        self.poll_identities();
        self.poll_size(progress);
        if let Some((job, report)) = progress {
            if let Run::Previewing {
                job: j,
                progress: p,
                ..
            } = &mut self.run
            {
                *j = Some(job);
                *p = Some(report);
            }
        }
        self.poll_sign_check();
        self.pump_paths();
        if let Some(reply) = &self.tools_wait {
            if let Some(answer) = reply.take() {
                self.tools_wait = None;
                if let Ok(tools) = answer {
                    self.signing_format = tools.signing_format;
                    self.tools = ToolsState::Known(tools);
                }
            }
        }
        let previewed = match &self.run {
            Run::Previewing { reply, .. } => reply.take(),
            _ => None,
        };
        if let Some(answer) = previewed {
            self.run = Run::Idle;
            match answer {
                Ok(plan) => self.take_plan(plan),
                Err(error) => self.on_error(&error),
            }
        }
        let applied = match &self.run {
            Run::Applying { reply, .. } => reply.take(),
            _ => None,
        };
        if let Some(answer) = applied {
            self.run = Run::Idle;
            match answer {
                Ok(outcome) => self.finish_apply(outcome),
                Err(error) => {
                    self.paths_apply_failed();
                    self.on_error(&error)
                }
            }
        }
        if let Some(reply) = &self.rows.pending {
            if let Some(answer) = reply.take() {
                self.rows.pending = None;
                if let Ok(page) = answer {
                    let same_run = self.plan.as_ref().is_some_and(|p| p.run == page.run);
                    if same_run
                        && page.filter == self.rows.filter
                        && page.offset == self.rows.loaded.len()
                    {
                        self.rows.total = page.total;
                        self.rows.loaded.extend(page.rows);
                    }
                }
            }
        }
    }

    /// An apply finished: the result panel replaces the preview.
    pub(crate) fn finish_apply(&mut self, outcome: RewriteOutcome) {
        let lfs = self.plan.as_ref().is_some_and(|p| p.counts.lfs_run);
        self.push = PushPanel::of(&outcome);
        self.push.lfs = lfs;
        self.paths_applied(&outcome);
        self.run = Run::Done(outcome);
        // The history the findings were read in is gone.
        self.secrets.stale = true;
        self.identities.loaded = None;
        self.identities.done = None;
        self.size.stale = true;
        self.plan_state = PlanState::None;
        self.choices = Choices::default();
        self.notice = None;
    }

    fn take_plan(&mut self, plan: RewritePlan) {
        self.rows = RowsState {
            total: plan.counts.commits_changed as usize,
            ..RowsState::default()
        };
        self.plan = Some(plan);
        self.plan_state = PlanState::Current;
        // Previewing the rule "Remove from history" added: the counts are on
        // the Summary, not under the findings.
        if matches!(self.tab, PreviewTab::Secrets | PreviewTab::Size) {
            self.tab = PreviewTab::Summary;
        }
        self.refs_cursor = 0;
        self.notice = None;
        self.want_rows(0);
    }

    /// What a refusal or failure does to the screen: a question, the stale
    /// marker, or a line with the reason.
    fn on_error(&mut self, error: &CoreError) {
        use gitpoulpi_core::TypedError as T;
        match error.typed() {
            Some(T::RewriteRefused(RewriteRefusal::SignatureChoiceNeeded, detail)) => {
                self.sign_check = None;
                self.question = Some(Question::Signatures {
                    count: leading_count(&detail),
                    cursor: 0,
                });
            }
            Some(T::RewriteRefused(RewriteRefusal::ExtractModeNeeded, _)) => {
                self.question = Some(Question::ExtractMode { cursor: 0 });
            }
            Some(T::RewriteRefused(RewriteRefusal::ScriptNotTrusted, detail)) => {
                self.ask_trust(detail, script::AfterTrust::Preview);
            }
            Some(T::RewriteRefused(RewriteRefusal::Stale, _)) => {
                self.plan_state = PlanState::Stale;
                self.notice = Some(Notice {
                    text: t::B_STALE.get().to_string(),
                    error: true,
                });
            }
            Some(T::RewriteRefused(
                kind @ (RewriteRefusal::DirtyPathsTouched
                | RewriteRefusal::CheckedOutElsewhere
                | RewriteRefusal::LfsObjectsMissing
                | RewriteRefusal::PartialClone
                | RewriteRefusal::PathCollision),
                detail,
            )) => {
                self.refusal = Some(Refusal {
                    kind,
                    lines: detail
                        .lines()
                        // The collision's first line only counts them.
                        .skip(usize::from(kind == RewriteRefusal::PathCollision))
                        .map(str::trim)
                        .filter(|l| !l.is_empty())
                        .map(str::to_string)
                        .collect(),
                });
                self.notice = Some(Notice {
                    text: crate::daemon_link::error_text::rewrite_refusal(kind)
                        .get()
                        .to_string(),
                    error: true,
                });
            }
            Some(T::RewriteRefused(RewriteRefusal::ScriptFailed, detail)) => {
                let (_, lines) = script::failure_lines(&detail);
                self.refusal = Some(Refusal {
                    kind: RewriteRefusal::ScriptFailed,
                    lines,
                });
                self.notice = Some(Notice {
                    text: crate::daemon_link::error_text::error_text(error),
                    error: true,
                });
            }
            Some(T::RewriteRefused(RewriteRefusal::InvalidRule, _))
                if self.secrets.result.is_some() =>
            {
                // A finding id the history no longer holds: the scan is old.
                self.secrets.stale = true;
                self.notice = Some(Notice {
                    text: crate::daemon_link::error_text::error_text(error),
                    error: true,
                });
            }
            Some(T::NeedsConfirmation(prompt)) => {
                self.question = Some(Question::ConfirmPushed { prompt });
            }
            Some(T::RewriteRefused(RewriteRefusal::ProtectedRef, _)) => {
                // S23: when the host protects them, the how-to is the way out.
                self.host.refused = true;
                self.notice = Some(Notice {
                    text: crate::daemon_link::error_text::error_text(error),
                    error: true,
                });
            }
            Some(T::Canceled { .. }) => {
                self.notice = Some(Notice {
                    text: t::PREVIEW_CANCELED.get().to_string(),
                    error: false,
                });
            }
            _ => {
                self.notice = Some(Notice {
                    text: crate::daemon_link::error_text::error_text(error),
                    error: true,
                });
            }
        }
    }

    // -------------------------------------------------------- save, load

    /// Writes the recipe to the shared or the user's recipes. A recipe needs
    /// a name; secret values and the answers of a run are not written.
    pub(crate) fn save(&mut self, place: RecipePlace) {
        if self.recipe.name.trim().is_empty() {
            self.notice = Some(Notice {
                text: t::E_NAME_NEEDED.get().to_string(),
                error: true,
            });
            return;
        }
        let Some(dir) = self.store.dir(place) else {
            self.notice = Some(Notice {
                text: t::E_NO_CONFIG_DIR.get().to_string(),
                error: true,
            });
            return;
        };
        let holds_secrets = self.recipe.holds_secrets();
        let text = match gitpoulpi_session::save_recipe(&dir, place, &self.saved_recipe()) {
            Ok(path) => {
                let shown = path.display();
                if holds_secrets {
                    t::saved_without_secrets(shown)
                } else {
                    match place {
                        RecipePlace::Shared => t::saved_shared(shown),
                        RecipePlace::Mine => t::saved_mine(shown),
                    }
                }
            }
            Err(error) => {
                self.notice = Some(Notice {
                    text: t::save_failed(error),
                    error: true,
                });
                return;
            }
        };
        self.notice = Some(Notice { text, error: false });
    }

    /// Every saved recipe: the repository's, then the user's.
    pub(crate) fn saved_recipes(&self) -> Vec<SavedRecipe> {
        let mut out = Vec::new();
        for place in [RecipePlace::Shared, RecipePlace::Mine] {
            if let Some(dir) = self.store.dir(place) {
                out.extend(gitpoulpi_session::list_recipes(&dir).into_iter().map(|e| {
                    SavedRecipe {
                        name: e.name,
                        place,
                        path: e.path,
                    }
                }));
            }
        }
        out
    }

    /// Replaces the recipe with the saved one. A rule that needs secret
    /// values asks for them again (they were never saved).
    pub(crate) fn load(&mut self, saved: &SavedRecipe) {
        match gitpoulpi_session::load_recipe(&saved.path) {
            Ok(recipe) => {
                self.recipe = recipe;
                self.rules.reset();
                self.detail = DetailForm::default();
                self.sync_detail();
                self.choices = Choices::default();
                self.discard();
                self.notice = Some(Notice {
                    text: t::loaded(&self.recipe.name),
                    error: false,
                });
            }
            Err(error) => {
                self.notice = Some(Notice {
                    text: t::load_failed(error),
                    error: true,
                });
            }
        }
    }

    // --------------------------------------------------------------- keys

    pub(crate) fn open_add_menu(&mut self) {
        self.popup = Some(Popup::AddRule { cursor: 0 });
    }

    pub(crate) fn open_load_menu(&mut self) {
        let items = self.saved_recipes();
        if items.is_empty() {
            self.notice = Some(Notice {
                text: t::E_NO_SAVED.get().to_string(),
                error: false,
            });
        } else {
            self.popup = Some(Popup::Load { items, cursor: 0 });
        }
    }

    /// Starts typing into field `at` of the detail pane (a text field), or
    /// flips/cycles any other kind.
    pub(crate) fn activate_field(&mut self, at: usize) {
        let Some(field) = self.detail.fields.get(at) else {
            return;
        };
        // The script's text is edited in its own overlay: the pane is a few
        // rows high.
        if field.id == FieldId::ScriptText {
            self.open_script_editor();
            return;
        }
        match &field.value {
            FieldValue::Text(text) => {
                let Some(rule) = self.recipe.rules.get(self.rules.cursor) else {
                    return;
                };
                self.editing = Some(Editing {
                    target: EditTarget::Field(at),
                    text: text.clone(),
                    caret: Caret::default(),
                    multiline: field.multiline,
                    snapshot: Snapshot::Rule(Box::new(rule.clone())),
                });
            }
            _ => self.step_field(at, 1),
        }
    }

    /// Starts typing the recipe's name.
    pub(crate) fn edit_name(&mut self) {
        self.editing = Some(Editing {
            target: EditTarget::Name,
            text: self.recipe.name.clone(),
            caret: Caret::default(),
            multiline: false,
            snapshot: Snapshot::Name(self.recipe.name.clone()),
        });
    }

    /// Pasted text goes into the field being typed.
    pub(crate) fn paste(&mut self, text: &str) {
        if self.has_target_picker() {
            return self.target_paste(text);
        }
        if self.rules_ai.is_open() {
            return self.paste_rules_ai(text);
        }
        if self.script.ai.dialog.is_some() {
            return self.paste_ai(text);
        }
        if self.script.editor.is_some() {
            return self.paste_script(text);
        }
        self.paste_editing(text);
    }

    /// Pasted text goes into the field being typed (not the script editor).
    fn paste_editing(&mut self, text: &str) {
        let Some(ed) = self.editing.as_mut() else {
            return;
        };
        let text = if ed.multiline {
            text.to_string()
        } else {
            text.replace(['\r', '\n'], " ")
        };
        ed.caret.insert(&mut ed.text, &text);
        self.push_edit();
    }

    /// Puts what is typed into the recipe now, so the footer and the row
    /// follow every key.
    fn push_edit(&mut self) {
        let Some(ed) = &self.editing else {
            return;
        };
        let (target, text) = (ed.target, ed.text.clone());
        match target {
            EditTarget::Field(at) => self.edit_field(at, FieldInput::Text(text)),
            EditTarget::Name => self.recipe.name = text,
        }
    }

    fn stop_editing(&mut self, keep: bool) {
        let Some(ed) = self.editing.take() else {
            return;
        };
        if keep {
            return;
        }
        match ed.snapshot {
            Snapshot::Rule(rule) => {
                if let Some(slot) = self.recipe.rules.get_mut(self.rules.cursor) {
                    *slot = *rule;
                }
                self.detail = DetailForm::default();
                self.sync_detail();
                self.touch();
            }
            Snapshot::Name(name) => self.recipe.name = name,
        }
    }

    fn on_edit_key(&mut self, key: KeyEvent) {
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        let Some(ed) = self.editing.as_mut() else {
            return;
        };
        match key.code {
            KeyCode::Esc => self.stop_editing(false),
            KeyCode::Enter if !ed.multiline || ctrl => self.stop_editing(true),
            KeyCode::Tab if ed.multiline => self.stop_editing(true),
            _ => {
                if ed.caret.on_key(&mut ed.text, &key, ed.multiline) == KeyEdit::Edited {
                    self.push_edit();
                }
            }
        }
    }

    fn on_question_key(&mut self, key: KeyEvent) {
        let options = match &self.question {
            Some(Question::Signatures { .. }) => SIGNATURE_CHOICES.len() + 1,
            Some(Question::ExtractMode { .. }) => EXTRACT_CHOICES.len(),
            // Confirm (0) or leave it (1).
            Some(Question::ConfirmPushed { .. }) => 2,
            // Trust it (1) or not (2).
            Some(Question::TrustScript { .. }) => 2,
            None => return,
        };
        if let Some(Question::TrustScript { cursor, .. }) = &mut self.question {
            // No default: the first arrow key only starts the highlight.
            match key.code {
                KeyCode::Up | KeyCode::Char('k') => {
                    *cursor = Some(cursor.map_or(0, |c| c.saturating_sub(1)));
                    return;
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    *cursor = Some(cursor.map_or(0, |c| (c + 1).min(1)));
                    return;
                }
                KeyCode::Enter => {
                    if let Some(c) = *cursor {
                        self.answer(c);
                    }
                    return;
                }
                _ => {}
            }
        }
        let cursor = match &mut self.question {
            Some(Question::Signatures { cursor, .. } | Question::ExtractMode { cursor }) => {
                Some(cursor)
            }
            _ => None,
        };
        match key.code {
            KeyCode::Esc => self.question = None,
            // A long prompt scrolls by pages; the arrows keep moving the answer.
            KeyCode::PageUp | KeyCode::PageDown => {
                let page = self.paint.question_room.get().saturating_sub(1).max(1);
                let top = self.paint.question_top.get();
                self.paint.question_top.set(if key.code == KeyCode::PageUp {
                    top.saturating_sub(page)
                } else {
                    top + page
                });
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if let Some(c) = cursor {
                    *c = c.saturating_sub(1);
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if let Some(c) = cursor {
                    *c = (*c + 1).min(options - 1);
                }
            }
            KeyCode::Char(d @ '1'..='9') => {
                let n = d as usize - '1' as usize;
                if n < options {
                    self.answer(n);
                }
            }
            KeyCode::Enter => {
                let at = match &self.question {
                    Some(
                        Question::Signatures { cursor, .. } | Question::ExtractMode { cursor },
                    ) => *cursor,
                    _ => 0,
                };
                self.answer(at);
            }
            _ => {}
        }
    }

    fn on_popup_key(&mut self, key: KeyEvent) {
        let len = self.popup_len();
        let at = self.popup_cursor();
        match key.code {
            KeyCode::Esc => self.popup = None,
            KeyCode::Up | KeyCode::Char('k') => self.popup_set_cursor(at.saturating_sub(1)),
            KeyCode::Down | KeyCode::Char('j') => {
                self.popup_set_cursor((at + 1).min(len.saturating_sub(1)))
            }
            KeyCode::Home => self.popup_set_cursor(0),
            KeyCode::End => self.popup_set_cursor(len.saturating_sub(1)),
            KeyCode::Enter => self.popup_pick(at),
            KeyCode::Char(' ') if matches!(self.popup, Some(Popup::Refs { .. })) => {
                self.popup_pick(at)
            }
            _ => {}
        }
    }

    fn cycle_focus(&mut self, delta: isize) {
        let order: Vec<Focus> = Focus::ORDER
            .into_iter()
            .filter(|f| *f != Focus::Detail || !self.recipe.rules.is_empty())
            .collect();
        let at = order.iter().position(|f| *f == self.focus).unwrap_or(0) as isize;
        self.focus = order[(at + delta).rem_euclid(order.len() as isize) as usize];
    }

    fn step_tab(&mut self, delta: isize) {
        self.refusal = None;
        let at = PreviewTab::ALL
            .iter()
            .position(|t| *t == self.tab)
            .unwrap_or(0) as isize;
        self.tab =
            PreviewTab::ALL[(at + delta).rem_euclid(PreviewTab::ALL.len() as isize) as usize];
    }

    /// One key. The workbench owns the keyboard while it is open.
    /// Whether `Ctrl+N` may badge the controls: only on the workbench itself.
    /// A dropdown, a question, a text being typed or an overlay (the folder
    /// picker, purge, the AI dialogs, the script editor, the blob diff, the
    /// host how-to, the merge picker) owns the next key, so none is badged.
    pub(crate) fn badge_ready(&self) -> bool {
        !(self.has_target_picker()
            || self.purge.is_some()
            || self.identities.pick.is_some()
            || self.question.is_some()
            || self.rules_ai.is_open()
            || self.script.ai.dialog.is_some()
            || self.script.editor.is_some()
            || self.drill.is_some()
            || self.host.howto.is_some()
            || self.editing.is_some()
            || self.popup.is_some())
    }

    /// What `Ctrl+N` badges, from the click targets the last paint registered
    /// (in click order): the buttons, tabs, fields, checks and rules; not the
    /// commit rows, the empty room of a pane or a shield.
    pub(crate) fn nav_targets(hits: &[(Rect, RewriteHit)]) -> Vec<(Rect, (u16, u16))> {
        crate::dialog_kit::nav_cells_in(hits, |hit| hit.badged())
    }

    pub(crate) fn on_key(&mut self, key: KeyEvent) -> RewriteKeyOutcome {
        if self.has_target_picker() {
            return self.on_target_key(key);
        }
        if self.purge.is_some() {
            return self.on_purge_key(key);
        }
        if self.identities.pick.is_some() {
            return self.on_identity_pick_key(key);
        }
        if self.question.is_some() {
            self.on_question_key(key);
            return RewriteKeyOutcome::Stay;
        }
        if self.rules_ai.is_open() {
            self.on_rules_ai_key(key);
            return RewriteKeyOutcome::Stay;
        }
        if self.script.ai.dialog.is_some() {
            self.on_ai_key(key);
            return RewriteKeyOutcome::Stay;
        }
        if self.script.editor.is_some() {
            self.on_script_key(key);
            return RewriteKeyOutcome::Stay;
        }
        if self.drill.is_some() {
            self.on_drill_key(key);
            return RewriteKeyOutcome::Stay;
        }
        if self.host.howto.is_some() {
            self.on_howto_key(key);
            return RewriteKeyOutcome::Stay;
        }
        if self.editing.is_some() {
            self.on_edit_key(key);
            return RewriteKeyOutcome::Stay;
        }
        if self.popup.is_some() {
            self.on_popup_key(key);
            return RewriteKeyOutcome::Stay;
        }
        let km = crate::keymap::current();
        let plain = !key
            .modifiers
            .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT);
        // The result panel: the journal's Undo chord undoes the rewrite.
        if matches!(self.run, Run::Done(_)) && km.is(Bind::GlobalUndo, &key) {
            return RewriteKeyOutcome::Undo;
        }
        if matches!(self.run, Run::Done(_)) && key.code != KeyCode::Esc {
            if let Some(outcome) = self.on_result_key(key, plain, &km) {
                return outcome;
            }
        }
        if key.code == KeyCode::Esc {
            // While a preview or a scan runs, Esc stops it; a second Esc
            // closes.
            if matches!(self.run, Run::Previewing { .. }) || self.scanning() || self.analysing() {
                self.cancel();
                return RewriteKeyOutcome::Stay;
            }
            if self.busy() {
                return RewriteKeyOutcome::Stay;
            }
            return RewriteKeyOutcome::Close;
        }
        // The Identities tab after a merge: `u` takes the `.mailmap` back
        // (the keyboard twin of its Undo button).
        if plain
            && key.code == KeyCode::Char('u')
            && self.focus == Focus::Preview
            && self.tab == PreviewTab::Identities
        {
            return self.identities_undo();
        }
        match key.code {
            KeyCode::Tab => self.cycle_focus(1),
            KeyCode::BackTab => self.cycle_focus(-1),
            _ if km.is(Bind::RewritePreview, &key) => self.preview(),
            _ if km.is(Bind::RewriteApply, &key) => self.apply(),
            _ if km.is(Bind::RewriteCancel, &key) => self.cancel(),
            _ if km.is(Bind::RewriteMoveUp, &key) => self.move_rule(-1),
            _ if km.is(Bind::RewriteMoveDown, &key) => self.move_rule(1),
            _ if km.is(Bind::RewriteRestoreRule, &key) => self.restore_rule(),
            _ if km.is(Bind::RewriteDuplicateRule, &key) => self.duplicate_rule(),
            _ if km.is(Bind::RewriteNextTab, &key) => self.step_tab(1),
            _ if km.is(Bind::RewritePrevTab, &key) => self.step_tab(-1),
            _ if km.is(Bind::RewriteSaveShared, &key) => self.save(RecipePlace::Shared),
            _ if km.is(Bind::RewriteSaveMine, &key) => self.save(RecipePlace::Mine),
            _ if km.is(Bind::RewriteLoad, &key) => self.open_load_menu(),
            _ if km.is(Bind::RewriteProtectedAll, &key) => self.check_all(),
            _ if km.is(Bind::RewriteProtectedNone, &key) => self.uncheck_all(),
            _ if km.is(Bind::RewriteCheckTools, &key) => self.check_tools_again(),
            _ if km.is(Bind::RewriteAnalyze, &key) => self.analyse_from_anywhere(),
            _ if km.is(Bind::RewriteHostHowto, &key) => self.open_howto(),
            _ if km.is(Bind::RewriteAiRules, &key) => self.open_rules_ai(),
            _ if km.is(Bind::RewriteName, &key) => self.edit_name(),
            // Last of the global keys: its bare-letter default also matches
            // the same letter with Alt or Ctrl (the keymap is lenient), so the
            // chords that carry a modifier go first.
            _ if km.is(Bind::RewriteAddRule, &key) => self.open_add_menu(),
            _ => match self.focus {
                Focus::Scope => self.on_scope_key(key, plain, &km),
                Focus::Rules => self.on_rules_key(key, plain, &km),
                Focus::Detail => self.on_detail_key(key, plain),
                Focus::Preview => self.on_preview_key(key, plain, &km),
            },
        }
        RewriteKeyOutcome::Stay
    }

    fn on_scope_key(&mut self, key: KeyEvent, plain: bool, _km: &crate::keymap::Keymap) {
        match key.code {
            KeyCode::Left if plain => self.step_scope_kind(-1),
            KeyCode::Right if plain => self.step_scope_kind(1),
            KeyCode::Enter | KeyCode::Down if plain => self.open_scope_menu(),
            KeyCode::Char(' ') if plain => self.toggle_tags(),
            KeyCode::Char('r') if plain => self.open_refs_menu(),
            KeyCode::Char('o') if plain => self.toggle_since(),
            _ => {}
        }
    }

    fn on_rules_key(&mut self, key: KeyEvent, plain: bool, km: &crate::keymap::Keymap) {
        match key.code {
            _ if km.is(Bind::RewriteToggleRule, &key) => self.toggle_rule(),
            _ if km.is(Bind::RewriteDeleteRule, &key) => self.delete_rule(),
            KeyCode::Up | KeyCode::Char('k') if plain => self.move_rule_cursor(-1),
            KeyCode::Down | KeyCode::Char('j') if plain => self.move_rule_cursor(1),
            KeyCode::Enter if plain && !self.recipe.rules.is_empty() => self.focus = Focus::Detail,
            _ => {}
        }
    }

    fn on_detail_key(&mut self, key: KeyEvent, plain: bool) {
        let at = self.detail.cursor;
        match key.code {
            KeyCode::Up | KeyCode::Char('k') if plain => self.detail.move_cursor(-1),
            KeyCode::Down | KeyCode::Char('j') if plain => self.detail.move_cursor(1),
            KeyCode::Left if plain => self.step_field(at, -1),
            KeyCode::Right if plain => self.step_field(at, 1),
            KeyCode::Enter | KeyCode::Char(' ') if plain => self.open_field(at),
            _ => {}
        }
    }

    /// "Check all" of the tab on screen: the protected branches of the Refs
    /// tab, or the findings of the Secrets tab.
    fn check_all(&mut self) {
        if self.tab == PreviewTab::Secrets {
            self.check_all_findings();
        } else if self.tab == PreviewTab::Identities {
            self.check_all_identities();
        } else if self.tab == PreviewTab::Size {
            self.check_all_size();
        } else {
            self.check_all_protected();
        }
    }

    fn uncheck_all(&mut self) {
        if self.tab == PreviewTab::Secrets {
            self.uncheck_all_findings();
        } else if self.tab == PreviewTab::Identities {
            self.uncheck_all_identities();
        } else if self.tab == PreviewTab::Size {
            self.uncheck_all_size();
        } else {
            self.uncheck_all_protected();
        }
    }

    fn on_preview_key(&mut self, key: KeyEvent, plain: bool, km: &crate::keymap::Keymap) {
        if self.tab == PreviewTab::Identities {
            return match key.code {
                KeyCode::Left if plain => self.step_tab(-1),
                KeyCode::Right if plain => self.step_tab(1),
                _ => self.on_identities_key(key, plain),
            };
        }
        if self.tab == PreviewTab::Secrets {
            return match key.code {
                KeyCode::Left if plain => self.step_tab(-1),
                KeyCode::Right if plain => self.step_tab(1),
                _ => self.on_secrets_key(key, plain),
            };
        }
        if self.tab == PreviewTab::Size {
            return match key.code {
                KeyCode::Left if plain => self.step_tab(-1),
                KeyCode::Right if plain => self.step_tab(1),
                _ => self.on_size_key(key, plain),
            };
        }
        match key.code {
            KeyCode::Left if plain => self.step_tab(-1),
            KeyCode::Right if plain => self.step_tab(1),
            KeyCode::Up | KeyCode::Char('k') if plain => self.move_preview_cursor(-1),
            KeyCode::Down | KeyCode::Char('j') if plain => self.move_preview_cursor(1),
            KeyCode::Char('f') if plain && self.tab == PreviewTab::Commits => {
                self.open_filter_menu()
            }
            // The Summary's "Use .mailmap" button (S16).
            KeyCode::Char('m')
                if plain && self.tab == PreviewTab::Summary && self.mailmap_offer() =>
            {
                self.use_mailmap_instead()
            }
            KeyCode::Enter if plain && self.tab == PreviewTab::Commits => {
                self.open_drill(self.rows.cursor)
            }
            // The Refs tab's list of moving refs has no cursor: its pages are
            // the keys' twin of its scrollbar (Up and Down move the protected
            // list's).
            KeyCode::PageUp | KeyCode::PageDown
                if self.tab == PreviewTab::Refs
                    && self.page_bar(
                        bars::BarId::RefsMoving,
                        if key.code == KeyCode::PageUp { -1 } else { 1 },
                    ) => {}
            KeyCode::PageUp => self.move_preview_cursor(-10),
            KeyCode::PageDown => self.move_preview_cursor(10),
            KeyCode::Char(' ') | KeyCode::Enter if plain && self.tab == PreviewTab::Refs => {
                let entries = self.protected();
                if let Some(entry) = entries.get(self.refs_cursor) {
                    self.toggle_protected(&entry.name.clone());
                }
            }
            _ if km.is(Bind::RewriteToggleRule, &key) && self.tab == PreviewTab::Refs => {
                let entries = self.protected();
                if let Some(entry) = entries.get(self.refs_cursor) {
                    self.toggle_protected(&entry.name.clone());
                }
            }
            _ => {}
        }
    }

    fn move_preview_cursor(&mut self, delta: isize) {
        if matches!(self.run, Run::Done(_)) {
            return self.move_push_cursor(delta);
        }
        match self.tab {
            PreviewTab::Commits => {
                let last = self
                    .rows
                    .total
                    .max(self.rows.loaded.len())
                    .saturating_sub(1);
                self.rows.cursor =
                    (self.rows.cursor as isize + delta).clamp(0, last as isize) as usize;
                self.want_rows(self.rows.cursor);
            }
            PreviewTab::Refs => {
                let last = self.protected().len().saturating_sub(1);
                self.refs_cursor =
                    (self.refs_cursor as isize + delta).clamp(0, last as isize) as usize;
            }
            PreviewTab::Secrets => self.move_secret_cursor(delta),
            PreviewTab::Identities => self.move_identity_cursor(delta),
            PreviewTab::Size => self.move_size_cursor(delta),
            // Summary and Impact are text: the keys scroll it.
            PreviewTab::Summary | PreviewTab::Impact => self.scroll_text(delta),
        }
    }

    /// Housekeeping when the workbench closes: a running preview is
    /// canceled (a finished plan stays in the daemon until the next
    /// preview), answers given for the run are forgotten.
    pub(crate) fn close(&mut self) {
        self.cancel();
        self.choices = Choices::default();
    }
}

/// The leading number of the daemon's "N signed commits would be rewritten".
fn leading_count(detail: &str) -> Option<u64> {
    detail.split_whitespace().next()?.parse().ok()
}
