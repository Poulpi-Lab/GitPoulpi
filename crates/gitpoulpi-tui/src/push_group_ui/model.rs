//! The data of the dialog's three steps and the pure decisions on it: which
//! action a row offers, the summary line, the rows a force confirmation
//! names, the request a Run sends. No painting and no I/O here, so the
//! rules are unit-tested without a terminal.

use std::cell::Cell;

use gitpoulpi_core::domain::{
    GroupAction, GroupMode, GroupRunStatus, Oid, PlannedTarget, Remote, RemoteClass,
    RemoteGroupPlan, RemoteGroupRunRequest, RemoteGroupStatus, RemoteRelation, RunTarget,
    TargetRef, TargetStatus,
};
use gitpoulpi_i18n::msg::push_group as msg;

use crate::combo::{Combo, ComboOption};
use crate::daemon_link::Reply;
use crate::scrollbar::{BarDrag, Track};

/// The things a click or a key can land on, from the paint walk.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Part {
    /// A row band (select the row).
    Row(u8),
    /// A selection row's checkbox.
    Check(u8),
    /// A plan row's action combo field.
    Combo(u8),
    Remember,
    Tags,
    Button(Btn),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Btn {
    Next,
    Cancel,
    FetchAgain,
    SkipProblems,
    Run,
    Retry,
    Close,
}

/// The scrolling state of one table.
#[derive(Default)]
pub(crate) struct Scroll {
    pub top: Cell<usize>,
    pub viewport: Cell<usize>,
    pub bar: Cell<Option<Track>>,
    pub drag: BarDrag,
}

// ---------------------------------------------------------------------
// Step 1: which remotes
// ---------------------------------------------------------------------

pub(crate) struct SelRow {
    pub remote: String,
    /// The branch's name on that remote.
    pub dest: String,
    pub checked: bool,
    /// The branch's upstream: it cannot be dropped from the config by this
    /// dialog, it only is (un)ticked for this push.
    pub primary: bool,
}

pub(crate) struct Select {
    pub remotes: Option<Reply<Vec<Remote>>>,
    pub status: Option<Reply<RemoteGroupStatus>>,
    names: Option<Vec<Remote>>,
    got: Option<RemoteGroupStatus>,
    pub error: Option<String>,
    pub ready: bool,
    pub rows: Vec<SelRow>,
    pub remember: bool,
    pub cursor: usize,
    pub scroll: Scroll,
}

impl Select {
    pub fn new(remotes: Reply<Vec<Remote>>, status: Reply<RemoteGroupStatus>) -> Self {
        Self {
            remotes: Some(remotes),
            status: Some(status),
            names: None,
            got: None,
            error: None,
            ready: false,
            rows: Vec::new(),
            remember: false,
            cursor: 0,
            scroll: Scroll::default(),
        }
    }

    /// Takes the answers that came; `true` when the rows were built.
    pub fn poll(&mut self, branch: &str) -> bool {
        if let Some(answer) = self.remotes.as_ref().and_then(Reply::take) {
            self.remotes = None;
            match answer {
                Ok(names) => self.names = Some(names),
                Err(e) => self.error = Some(crate::daemon_link::error_text::error_text(&e)),
            }
        }
        if let Some(answer) = self.status.as_ref().and_then(Reply::take) {
            self.status = None;
            match answer {
                Ok(s) => self.got = Some(s),
                Err(e) => self.error = Some(crate::daemon_link::error_text::error_text(&e)),
            }
        }
        if self.ready || self.error.is_some() {
            return false;
        }
        let (Some(names), Some(status)) = (&self.names, &self.got) else {
            return false;
        };
        let configured = |name: &str| -> Option<&TargetRef> {
            status
                .remotes
                .primary
                .iter()
                .chain(status.remotes.push_to.iter())
                .find(|t| t.remote == name)
        };
        self.rows = names
            .iter()
            .map(|r| {
                let t = configured(&r.name);
                SelRow {
                    remote: r.name.clone(),
                    dest: t.map_or_else(|| branch.to_string(), |t| t.branch.clone()),
                    checked: t.is_some(),
                    primary: status
                        .remotes
                        .primary
                        .as_ref()
                        .is_some_and(|p| p.remote == r.name),
                }
            })
            .collect();
        // A branch with no configured remote at all starts with the only
        // remote there is, so Next is not a dead end.
        if self.rows.iter().all(|r| !r.checked) && self.rows.len() == 1 {
            self.rows[0].checked = true;
        }
        self.ready = true;
        true
    }

    pub fn targets(&self) -> Vec<TargetRef> {
        self.rows
            .iter()
            .filter(|r| r.checked)
            .map(|r| TargetRef {
                remote: r.remote.clone(),
                branch: r.dest.clone(),
            })
            .collect()
    }

    /// What "Remember for this branch" writes: the ticked remotes that are
    /// not the upstream become the push targets; the pull sources stay as
    /// they were.
    pub fn remember_action(&self, branch: &str) -> Option<crate::daemon_link::Action> {
        let status = self.got.as_ref()?;
        let push_to: Vec<String> = self
            .rows
            .iter()
            .filter(|r| r.checked && !r.primary)
            .map(|r| {
                TargetRef {
                    remote: r.remote.clone(),
                    branch: r.dest.clone(),
                }
                .to_value(branch)
            })
            .collect();
        let pull_from: Vec<String> = status
            .remotes
            .pull_from
            .iter()
            .map(|t| t.to_value(branch))
            .collect();
        Some(crate::daemon_link::Action::BranchSetRemotes {
            branch: branch.to_string(),
            upstream: None,
            push_to,
            pull_from,
        })
    }
}

// ---------------------------------------------------------------------
// Step 2: the plan
// ---------------------------------------------------------------------

pub(crate) struct PlanRow {
    pub t: PlannedTarget,
    pub chosen: GroupAction,
    pub combo: Combo,
}

impl PlanRow {
    fn new(t: PlannedTarget) -> Self {
        let mut combo = Combo::new("").with_click_picks();
        combo.set_options(
            t.allowed
                .iter()
                .map(|a| ComboOption::new(action_label(*a, &t.remote), action_key(*a)))
                .collect(),
        );
        let chosen = t.suggested;
        combo.select(Some(action_key(chosen).to_string()));
        Self { t, chosen, combo }
    }

    pub fn choose(&mut self, action: GroupAction) {
        if self.t.allowed.contains(&action) {
            self.chosen = action;
            self.combo.select(Some(action_key(action).to_string()));
        }
    }

    /// Steps the choice through the allowed actions (`delta` -1 / +1).
    pub fn step(&mut self, delta: isize) {
        let at = self
            .t
            .allowed
            .iter()
            .position(|a| *a == self.chosen)
            .unwrap_or(0) as isize;
        let n = self.t.allowed.len() as isize;
        if n > 0 {
            let to = (at + delta).rem_euclid(n) as usize;
            self.choose(self.t.allowed[to]);
        }
    }

    /// The conflict line under the row, when the chosen action integrates.
    pub fn conflict_line(&self) -> Option<String> {
        let integrates = matches!(
            self.chosen,
            GroupAction::MergeFrom | GroupAction::RebaseOnto
        );
        (integrates && !self.t.conflicts.is_empty()).then(|| {
            msg::conflicts(
                self.t.conflicts.len() as u64,
                self.t.conflicts.iter().take(3).cloned().collect::<Vec<_>>().join(", "),
            )
        })
    }

    pub fn lines(&self) -> u16 {
        1 + u16::from(self.conflict_line().is_some())
    }

    /// Whether this row is a problem "Skip problems" sets aside.
    pub fn is_problem(&self) -> bool {
        self.t.error.is_some()
            || self.t.missing
            || matches!(
                self.t.relation,
                RemoteRelation::Diverged { .. } | RemoteRelation::Unrelated
            )
            || (self.t.allowed == [GroupAction::Skip]
                && self.t.relation != RemoteRelation::UpToDate)
    }

    /// Commits a force push takes off the remote: the ones it has and local
    /// lacks (for a rewritten branch they are older copies of local commits).
    pub fn replaced(&self) -> u32 {
        self.t.behind
    }
}

pub(crate) fn action_key(a: GroupAction) -> &'static str {
    match a {
        GroupAction::Push => "push",
        GroupAction::FastForwardFrom => "ff",
        GroupAction::MergeFrom => "merge",
        GroupAction::RebaseOnto => "rebase",
        GroupAction::ForceWithLease => "force",
        GroupAction::Skip => "skip",
    }
}

pub(crate) fn action_from_key(key: &str) -> Option<GroupAction> {
    [
        GroupAction::Push,
        GroupAction::FastForwardFrom,
        GroupAction::MergeFrom,
        GroupAction::RebaseOnto,
        GroupAction::ForceWithLease,
        GroupAction::Skip,
    ]
    .into_iter()
    .find(|a| action_key(*a) == key)
}

pub(crate) fn action_label(a: GroupAction, remote: &str) -> String {
    match a {
        GroupAction::Push => msg::ACTION_PUSH.get().to_string(),
        GroupAction::FastForwardFrom => msg::action_fast_forward(remote),
        GroupAction::MergeFrom => msg::action_merge(remote),
        GroupAction::RebaseOnto => msg::action_rebase(remote),
        GroupAction::ForceWithLease => msg::ACTION_FORCE.get().to_string(),
        GroupAction::Skip => msg::ACTION_SKIP.get().to_string(),
    }
}

pub(crate) struct PlanView {
    /// `None` once the plan is in.
    pub reply: Option<Reply<RemoteGroupPlan>>,
    pub error: Option<String>,
    /// The explicit list this plan was asked for (Fetch again asks the
    /// same).
    pub targets: Option<Vec<TargetRef>>,
    pub local: Option<Oid>,
    pub rows: Vec<PlanRow>,
    pub tags: bool,
    pub cursor: usize,
    pub scroll: Scroll,
    /// `git.pull_mode` is rebase (read when the plan was asked).
    pub prefer_rebase: bool,
}

impl PlanView {
    pub fn loading(
        reply: Reply<RemoteGroupPlan>,
        targets: Option<Vec<TargetRef>>,
        prefer_rebase: bool,
    ) -> Self {
        Self {
            reply: Some(reply),
            error: None,
            targets,
            local: None,
            rows: Vec::new(),
            tags: false,
            cursor: 0,
            scroll: Scroll::default(),
            prefer_rebase,
        }
    }

    /// A plan that is already there (a `needs_resolution` answer).
    pub fn from_plan(plan: RemoteGroupPlan, prefer_rebase: bool) -> Self {
        let mut view = Self {
            reply: None,
            error: None,
            targets: None,
            local: None,
            rows: Vec::new(),
            tags: false,
            cursor: 0,
            scroll: Scroll::default(),
            prefer_rebase,
        };
        view.fill(plan);
        view
    }

    fn fill(&mut self, plan: RemoteGroupPlan) {
        self.local = Some(plan.local);
        self.rows = plan.targets.into_iter().map(PlanRow::new).collect();
        self.cursor = 0;
    }

    /// Takes the plan once it came; `true` when the rows changed.
    pub fn poll(&mut self) -> bool {
        let Some(answer) = self.reply.as_ref().and_then(Reply::take) else {
            return false;
        };
        self.reply = None;
        match answer {
            Ok(plan) => {
                self.error = None;
                self.fill(plan);
            }
            Err(e) => self.error = Some(crate::daemon_link::error_text::error_text(&e)),
        }
        true
    }

    pub fn ready(&self) -> bool {
        self.reply.is_none() && self.error.is_none() && !self.rows.is_empty()
    }

    pub fn skip_problems(&mut self) {
        for row in self.rows.iter_mut().filter(|r| r.is_problem()) {
            row.choose(GroupAction::Skip);
        }
    }

    /// The remotes a force push replaces commits on, and how many.
    pub fn force_rows(&self) -> Vec<(&str, u32)> {
        self.rows
            .iter()
            .filter(|r| r.chosen == GroupAction::ForceWithLease)
            .map(|r| (r.t.remote.as_str(), r.replaced()))
            .collect()
    }

    /// Remotes that already hold the local commits while a row rebases:
    /// the rebase rewrites them, so those remotes will need a force push.
    pub fn rebase_warning(&self) -> Vec<&str> {
        if !self
            .rows
            .iter()
            .any(|r| r.chosen == GroupAction::RebaseOnto)
        {
            return Vec::new();
        }
        self.rows
            .iter()
            .filter(|r| r.chosen != GroupAction::Skip && r.chosen != GroupAction::RebaseOnto)
            .filter(|r| {
                RemoteClass {
                    relation: r.t.relation,
                    rewritten: r.t.rewritten,
                }
                .holds_local(r.t.local_unmatched)
            })
            .map(|r| r.t.remote.as_str())
            .collect()
    }

    /// The "Then: ..." line.
    pub fn then_line(&self) -> String {
        let mut steps: Vec<String> = Vec::new();
        for r in &self.rows {
            match r.chosen {
                GroupAction::FastForwardFrom => {
                    steps.push(msg::step_fast_forward(&r.t.remote));
                }
                GroupAction::MergeFrom => steps.push(msg::step_merge(&r.t.remote)),
                GroupAction::RebaseOnto => steps.push(msg::step_rebase(&r.t.remote)),
                _ => {}
            }
        }
        let sends = self
            .rows
            .iter()
            .filter(|r| r.chosen != GroupAction::Skip)
            .count();
        if sends > 0 {
            steps.push(msg::step_push(sends as u64));
        }
        if steps.is_empty() {
            return msg::THEN_NOTHING.get().to_string();
        }
        msg::then_line(steps.join(msg::THEN_JOIN.get()))
    }

    /// The request a Run sends: the decisions in plan order.
    pub fn request(&self, branch: &str) -> RemoteGroupRunRequest {
        RemoteGroupRunRequest {
            branch: branch.to_string(),
            mode: GroupMode::Push,
            targets: self
                .rows
                .iter()
                .map(|r| RunTarget::from_planned(&r.t, r.chosen))
                .collect(),
            tags: self.tags,
            prefer_rebase: self.prefer_rebase,
            ..Default::default()
        }
    }
}

/// The state cell of a plan row: its words and what colour says about it.
pub(crate) fn state_text(t: &PlannedTarget) -> (String, StateTone) {
    let counts = |a: u32, b: u32| {
        let mut out = String::new();
        if a > 0 {
            out.push_str(&format!("\u{2191}{a}"));
        }
        if b > 0 {
            if !out.is_empty() {
                out.push(' ');
            }
            out.push_str(&format!("\u{2193}{b}"));
        }
        out
    };
    if t.missing {
        return (msg::STATE_MISSING.get().to_string(), StateTone::Bad);
    }
    if t.error.is_some() {
        return (msg::STATE_UNREACHABLE.get().to_string(), StateTone::Bad);
    }
    let (words, tone) = match t.relation {
        RemoteRelation::UpToDate => (msg::STATE_UP_TO_DATE.get().to_string(), StateTone::Good),
        RemoteRelation::LocalAhead(n) => {
            let word = if t.allowed == [GroupAction::Skip] {
                (msg::STATE_REFUSED.get(), StateTone::Bad)
            } else {
                (msg::STATE_READY.get(), StateTone::Warn)
            };
            return (format!("\u{2191}{n} {}", word.0), word.1);
        }
        RemoteRelation::RemoteAhead(n) => {
            return (
                format!("\u{2193}{n} {}", msg::STATE_BEHIND.get()),
                StateTone::Warn,
            )
        }
        RemoteRelation::Diverged { ahead, behind } => {
            let word = if t.rewritten {
                msg::STATE_REWRITTEN.get()
            } else {
                msg::STATE_DIVERGED.get()
            };
            return (
                format!("{} {word}", counts(ahead, behind)),
                if t.rewritten {
                    StateTone::Warn
                } else {
                    StateTone::Bad
                },
            );
        }
        RemoteRelation::Absent => (msg::STATE_NEW.get().to_string(), StateTone::Muted),
        RemoteRelation::Unrelated => (msg::STATE_UNRELATED.get().to_string(), StateTone::Bad),
    };
    (words, tone)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StateTone {
    Good,
    Warn,
    Bad,
    Muted,
}

// ---------------------------------------------------------------------
// Step 3: the run
// ---------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct RunRow {
    pub remote: String,
    pub branch: String,
    /// `None` until the first progress report: waiting.
    pub status: Option<TargetStatus>,
    pub detail: Option<String>,
    /// What the integration did (a merge, a rebase, a fast-forward).
    pub integrated: Option<gitpoulpi_core::domain::Integration>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Phase {
    Going,
    Ended(GroupRunStatus),
    /// The run request itself was refused (a pending run elsewhere ...).
    Failed(String),
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct RunState {
    pub branch: String,
    pub mode: GroupMode,
    pub run_id: Option<String>,
    pub request: RemoteGroupRunRequest,
    pub rows: Vec<RunRow>,
    pub phase: Phase,
    pub message: String,
}

impl RunState {
    pub fn from_request(request: &RemoteGroupRunRequest) -> Self {
        Self {
            branch: request.branch.clone(),
            mode: request.mode,
            run_id: None,
            request: request.clone(),
            rows: request
                .targets
                .iter()
                .map(|t| RunRow {
                    remote: t.remote.clone(),
                    branch: t.branch.clone(),
                    status: None,
                    detail: None,
                    integrated: None,
                })
                .collect(),
            phase: Phase::Going,
            message: String::new(),
        }
    }

    pub fn going(&self) -> bool {
        self.phase == Phase::Going
    }

    /// Some target failed, was rejected, or is unknown.
    pub fn has_retryable(&self) -> bool {
        self.rows.iter().any(|r| {
            matches!(
                r.status,
                Some(
                    TargetStatus::Failed { .. }
                        | TargetStatus::Rejected { .. }
                        | TargetStatus::Unknown
                        | TargetStatus::NotAttempted
                )
            )
        }) || (matches!(
            self.phase,
            Phase::Ended(GroupRunStatus::Problems | GroupRunStatus::Canceled)
        ))
    }
}

/// One progress row's words: (glyph, text, tone).
pub(crate) fn run_row_text(row: &RunRow) -> (&'static str, String, StateTone) {
    let (glyph, text, tone) = run_row_base(row);
    match row.integrated {
        Some(i) if tone == StateTone::Good && row.status != Some(TargetStatus::UpToDate) => {
            (glyph, format!("{}, {text}", integrated_text(i)), tone)
        }
        _ => (glyph, text, tone),
    }
}

fn run_row_base(row: &RunRow) -> (&'static str, String, StateTone) {
    let why = |d: &Option<String>| d.clone().unwrap_or_default();
    match &row.status {
        None | Some(TargetStatus::NotAttempted) => (
            "\u{00B7}",
            msg::RUN_WAITING.get().to_string(),
            StateTone::Muted,
        ),
        Some(TargetStatus::Integrating) => (
            "\u{2026}",
            msg::RUN_MERGING.get().to_string(),
            StateTone::Warn,
        ),
        Some(TargetStatus::Pushing) => (
            "\u{2026}",
            msg::RUN_PUSHING.get().to_string(),
            StateTone::Warn,
        ),
        Some(TargetStatus::Integrated) => (
            "\u{2713}",
            msg::RUN_INTEGRATED.get().to_string(),
            StateTone::Good,
        ),
        Some(TargetStatus::Pushed { created: true }) => (
            "\u{2713}",
            msg::RUN_CREATED.get().to_string(),
            StateTone::Good,
        ),
        Some(TargetStatus::Pushed { created: false }) => (
            "\u{2713}",
            msg::RUN_PUSHED.get().to_string(),
            StateTone::Good,
        ),
        Some(TargetStatus::UpToDate) => (
            "\u{2713}",
            msg::RUN_UP_TO_DATE.get().to_string(),
            StateTone::Good,
        ),
        Some(TargetStatus::Skipped) => (
            "\u{00B7}",
            msg::RUN_SKIPPED.get().to_string(),
            StateTone::Muted,
        ),
        Some(TargetStatus::Rejected { reason }) => (
            "\u{2717}",
            msg::run_rejected(row.detail.clone().unwrap_or_else(|| format!("{reason:?}"))),
            StateTone::Bad,
        ),
        Some(TargetStatus::Failed { kind }) => (
            "\u{2717}",
            msg::run_failed(row.detail.clone().unwrap_or_else(|| format!("{kind:?}"))),
            StateTone::Bad,
        ),
        Some(TargetStatus::RemoteMissing) => (
            "\u{2717}",
            msg::RUN_MISSING.get().to_string(),
            StateTone::Bad,
        ),
        Some(TargetStatus::Unknown) => (
            "?",
            format!(
                "{}{}",
                msg::RUN_UNKNOWN.get(),
                if why(&row.detail).is_empty() {
                    String::new()
                } else {
                    format!(" - {}", why(&row.detail))
                }
            ),
            StateTone::Warn,
        ),
    }
}

/// The integration word for a final status where the target was combined
/// (a merge, a rebase, a fast-forward) rather than only pushed.
pub(crate) fn integrated_text(i: gitpoulpi_core::domain::Integration) -> String {
    use gitpoulpi_core::domain::Integration as I;
    match i {
        I::FastForwarded => msg::RUN_FAST_FORWARDED.get().to_string(),
        I::Merged => msg::RUN_MERGED.get().to_string(),
        I::Rebased => msg::RUN_REBASED.get().to_string(),
        I::UpToDate => msg::RUN_UP_TO_DATE.get().to_string(),
    }
}
