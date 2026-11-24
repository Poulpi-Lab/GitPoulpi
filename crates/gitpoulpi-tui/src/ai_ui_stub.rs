//! `ai_ui` for a build without the `ai` feature (custom builds, see
//! the custom-builds design §7): the same `pub(crate)` API as
//! the real module, so the call sites compile unchanged, with AI never
//! enabled, nothing ever running and no popup ever open. The request and
//! answer types keep their shapes (with local stand-ins for the
//! `gitpoulpi-ai` types) so the `match`es on them still compile; none of
//! them is ever built.
#![allow(dead_code)]

use gitpoulpi_core::domain::{Oid, RepoRef, SplitPart, SplitSource};
use ratatui::layout::Rect;
use ratatui::Frame;

fn not_built() -> String {
    gitpoulpi_core::Error::NotBuilt { feature: "ai" }.to_string()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ExplainTarget {
    Commit {
        oid: Oid,
        message: String,
        short: String,
    },
    Wip,
}

/// Stands in for `gitpoulpi_ai::prompts::ConflictContext`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ConflictContext {
    pub(crate) path: String,
}

impl ConflictContext {
    /// Never asked: there is no AI to send to.
    pub(crate) fn size(&self) -> usize {
        0
    }

    /// Never: there is no AI to ask.
    pub(crate) fn from_document(
        _: &gitpoulpi_core::domain::MergeDocument,
        _: usize,
    ) -> Option<Self> {
        None
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum AiRequest {
    CommitMessage,
    Explain(ExplainTarget),
    StashMessage,
    PullRequest {
        source: String,
        target: String,
    },
    Compose,
    SplitCommit {
        oid: Oid,
        short: String,
    },
    ResolveConflict {
        path: String,
        conflict: usize,
        context: Box<ConflictContext>,
        oversize_ok: bool,
    },
    RewriteScript {
        instruction: String,
        sample: Vec<crate::rewrite_view::SampleRow>,
    },
    RewriteRules {
        description: String,
        context: crate::rewrite_view::RulesCtx,
    },
    WorktreeName {
        description: String,
        branches: Vec<String>,
    },
    SequenceDraft(Box<crate::preferences::SeqDraftRequest>),
}

/// Stands in for the real naming answer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AiNames {
    pub(crate) name: String,
    pub(crate) branch: String,
}

/// Stands in for `gitpoulpi_ai::assist::RewriteScriptOutcome`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RewriteScriptOutcome {
    pub(crate) script: String,
    pub(crate) problems: Vec<String>,
    pub(crate) checked: bool,
    pub(crate) retried: bool,
}

/// Stands in for the real preview: there is no AI to describe.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PromptPreview {
    pub(crate) system: String,
    pub(crate) via: String,
}

pub(crate) fn rewrite_prompt_preview(_: &str, _: &RepoRef) -> Result<PromptPreview, String> {
    Err(not_built())
}

pub(crate) fn rewrite_user_message(_: &str, _: &[crate::rewrite_view::SampleRow]) -> String {
    String::new()
}

pub(crate) fn rewrite_rules_prompt_preview(_: &str, _: &RepoRef) -> Result<PromptPreview, String> {
    Err(not_built())
}

pub(crate) fn rewrite_rules_user_message(_: &str, _: &crate::rewrite_view::RulesCtx) -> String {
    String::new()
}

/// Stands in for `gitpoulpi_ai::parse::CommitMessage`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CommitMessage {
    pub(crate) summary: String,
    pub(crate) description: String,
}

/// Stands in for `gitpoulpi_ai::parse::PullRequestText`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PullRequestText {
    pub(crate) title: String,
    pub(crate) description: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum AiAnswer {
    CommitMessage(CommitMessage),
    Explanation(String),
    StashMessage(String),
    PullRequest(PullRequestText),
    Plan(Vec<(String, Vec<String>)>),
    Resolution(Vec<String>),
    Split {
        source: Box<SplitSource>,
        plan: Result<Vec<SplitPart>, String>,
    },
    RewriteScript(Box<RewriteScriptOutcome>),
    RewriteRules(Box<crate::rewrite_view::RulesProposal>),
    WorktreeName(AiNames),
    SequenceDraft(Box<crate::preferences::SeqDraftAnswer>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum AiOutcome {
    Done(AiRequest, Result<AiAnswer, String>),
    Canceled(AiRequest),
}

pub(crate) struct AiUi;

impl AiUi {
    pub(crate) fn new() -> Self {
        Self
    }

    pub(crate) fn reload(&mut self) {}

    /// Never: this binary has no AI.
    pub(crate) fn enabled(&self) -> bool {
        false
    }

    /// Never asked: nothing is sent.
    pub(crate) fn conflict_limit(&self) -> usize {
        usize::MAX
    }

    /// Never: this binary has no AI.
    pub(crate) fn has_local_agent(&self) -> bool {
        false
    }

    pub(crate) fn running(&self) -> bool {
        false
    }

    pub(crate) fn running_pull_request(&self) -> bool {
        false
    }

    pub(crate) fn running_worktree_name(&self) -> bool {
        false
    }

    pub(crate) fn running_conflict(&self) -> bool {
        false
    }

    pub(crate) fn running_conflict_info(&self) -> Option<(usize, String, u64)> {
        None
    }

    pub(crate) fn progress_line(&self) -> Option<String> {
        None
    }

    pub(crate) fn start_into(
        &mut self,
        _session: &str,
        _repo: &RepoRef,
        _request: AiRequest,
        last_message: &mut Option<String>,
    ) {
        *last_message = Some(not_built());
    }

    pub(crate) fn explain_into(
        &mut self,
        _session: &str,
        _repo: &RepoRef,
        _target: ExplainTarget,
        last_message: &mut Option<String>,
    ) {
        *last_message = Some(not_built());
    }

    pub(crate) fn adopt(&mut self, _: &std::path::Path) {}

    pub(crate) fn cancel(&mut self) {}

    pub(crate) fn poll(&mut self) -> Option<AiOutcome> {
        None
    }

    pub(crate) fn popup_open(&self) -> bool {
        false
    }

    pub(crate) fn popup_mut(&mut self) -> Option<&mut AiPopup> {
        None
    }

    pub(crate) fn close_popup(&mut self) {}

    pub(crate) fn fill_popup(&mut self, _result: Result<String, String>) {}

    pub(crate) fn render_popup(&self, _frame: &mut Frame, _area: Rect) {}
}

/// Never constructed.
pub(crate) struct AiPopup;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PopupKey {
    Nothing,
    Close,
    Copy(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PopupMouse {
    Nothing,
    Close,
}

impl AiPopup {
    pub(crate) fn on_mouse(&mut self, _mouse: crossterm::event::MouseEvent) -> PopupMouse {
        PopupMouse::Nothing
    }

    pub(crate) fn is_working(&self) -> bool {
        false
    }

    pub(crate) fn scroll_by(&mut self, _delta: i32) {}

    pub(crate) fn on_key(&mut self, _key: &crossterm::event::KeyEvent) -> PopupKey {
        PopupKey::Nothing
    }
}

/// How a request ended, for a waiter (a sequence step). Without the `ai`
/// feature no request ever runs, so nothing is ever recorded.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum AiEnd {
    Ok,
    Failed(String),
    Canceled,
}

/// Mirrors `ai_ui::finished_of`: the stub never sees an answer, so the end
/// carries nothing a waiter could act on.
pub(crate) struct Finished;

pub(crate) fn finished_of(_outcome: &AiOutcome) -> Finished {
    Finished
}

/// Mirrors `ai_ui::record_finished`; nothing to record without the feature.
pub(crate) fn record_finished(_finished: Finished) {}

/// Mirrors `ai_ui::take_finished`; no request ever finishes.
#[allow(dead_code)]
pub(crate) fn take_finished(_kind: gitpoulpi_config::AiAction) -> Option<AiEnd> {
    None
}
