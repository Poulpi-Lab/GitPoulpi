//! The ✦ AI half of the review view: asking the model for suggested
//! comments, keeping them until the reviewer decides, and turning an
//! accepted one into a pending comment.
//!
//! Maintainer rules this follows: the AI runs only on the explicit "Suggest
//! review" action (button, `s`, palette), never on its own; the answer is
//! never sent anywhere - an accepted suggestion becomes an ordinary pending
//! comment ([`ReviewView::add_pending`], the one place those are born) and
//! nothing reaches the host before the reviewer submits the review; the run
//! happens on a worker thread with an in-flight line after 150 ms and Esc
//! cancels it.
//!
//! This file does not depend on the `ai` feature: the worker
//! (`ai_worker.rs`, a stub without the feature) is the only part that talks
//! to `gitpoulpi-ai`, and returns this file's own [`Item`]s.
#![cfg_attr(not(feature = "ai"), allow(dead_code))]

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};
use std::time::Instant;

use gitpoulpi_core::domain::Oid;
use gitpoulpi_hosting::{DiffSide, PendingComment, ReviewEvent};
use gitpoulpi_i18n::msg::review_ai as msg;
use gitpoulpi_session::{AiDraft, AiItem, AiPlace};

use super::model::{Item, Place, Sev};
use super::popups::ComposeFor;
use super::{ReviewEffect, ReviewView, SPIN_AFTER};

/// A thread the model should not repeat.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct BriefThread {
    pub(super) path: String,
    pub(super) line: Option<u32>,
    pub(super) resolved: bool,
    /// `(author, text)`.
    pub(super) comments: Vec<(String, String)>,
}

/// What the worker needs to ask.
#[derive(Clone)]
pub(super) struct AskInput {
    pub(super) session: String,
    /// `None`: the request is reviewed through the API, no clone.
    pub(super) repo_root: Option<std::path::PathBuf>,
    pub(super) source: super::source::CodeSource,
    /// A test's stand-in for the model: gets the files the worker read.
    #[cfg(test)]
    pub(super) fake: Option<FakeModel>,
    pub(super) title: String,
    pub(super) description: String,
    pub(super) threads: Vec<BriefThread>,
    pub(super) base: Oid,
    pub(super) head: Oid,
    pub(super) cancel: Arc<AtomicBool>,
    /// `host/project`, and `remote-reviews.toml` (no clone): where the
    /// project's own `[ai.review]` values live.
    pub(super) remote_key: String,
    pub(super) remote_file: Option<std::path::PathBuf>,
    /// Every configured profile: a context link goes through the one that
    /// matches its host.
    pub(super) integrations: Vec<gitpoulpi_hosting::Integration>,
    /// The choices of the "✦ Review options" dialog for this run, over the
    /// repository's and the global defaults.
    pub(super) per_run: gitpoulpi_config::RepoAiReview,
}

/// A test's stand-in for the model: gets the files the worker read.
#[cfg(test)]
pub(super) type FakeModel = Arc<dyn Fn(&[gitpoulpi_core::domain::FileDiff]) -> Done + Send + Sync>;

/// What the worker brings back.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Done {
    pub(super) items: Vec<Item>,
    pub(super) summary: String,
    /// The review text the model wrote (the summary when it wrote none).
    pub(super) body: String,
    pub(super) verdict: ReviewEvent,
    /// Files the model saw shortened / not at all.
    pub(super) cut: usize,
    pub(super) left_out: usize,
    /// What the context files could not give (an unreadable extra file) and
    /// which of them were shortened, for the draft card.
    pub(super) context_notes: Vec<String>,
}

/// `Err(String::new())`: canceled.
pub(super) type Answer = Result<Done, String>;

pub(super) struct Run {
    rx: mpsc::Receiver<Answer>,
    cancel: Arc<AtomicBool>,
    since: Instant,
}

/// The suggestions of one open review.
pub(super) struct Suggest {
    /// AI is on (Settings > AI): nothing of this shows otherwise.
    enabled: bool,
    session: String,
    #[cfg(test)]
    pub(super) fake: Option<FakeModel>,
    pub(super) run: Option<Run>,
    pub(super) items: Vec<Item>,
    pub(super) summary: String,
    /// The review text of the ✦ draft, editable before it is used.
    pub(super) body: String,
    /// The verdict the model recommends.
    pub(super) verdict: ReviewEvent,
    /// The head the draft was written against (empty: unknown).
    pub(super) head_sha: String,
    pub(super) notes: Vec<String>,
    /// A run finished (so "nothing to point out" can be said).
    pub(super) done: bool,
    /// Asked while the request's code was still being fetched: it starts
    /// by itself the moment the code is here (still the reviewer's own
    /// action, just asked a little early).
    queued: bool,
    /// The options the queued (or last) run was asked with.
    pub(super) per_run: gitpoulpi_config::RepoAiReview,
    /// A test's stand-in for the files the options dialog reads.
    #[cfg(test)]
    pub(super) fake_defaults: Option<super::options::Defaults>,
    next_id: u64,
}

impl Suggest {
    pub(super) fn new(enabled: bool, session: String) -> Self {
        Self {
            enabled: enabled && crate::build_features::AI,
            session,
            #[cfg(test)]
            fake: None,
            run: None,
            items: Vec::new(),
            summary: String::new(),
            body: String::new(),
            verdict: ReviewEvent::Comment,
            head_sha: String::new(),
            notes: Vec::new(),
            done: false,
            queued: false,
            per_run: Default::default(),
            #[cfg(test)]
            fake_defaults: None,
            next_id: 1,
        }
    }

    pub(super) fn enabled(&self) -> bool {
        self.enabled
    }

    pub(super) fn session(&self) -> &str {
        &self.session
    }

    pub(super) fn running(&self) -> bool {
        self.run.is_some() || self.queued
    }

    /// Seconds so far, once the run is slow enough to show feedback.
    pub(super) fn progress(&self) -> Option<u64> {
        self.run
            .as_ref()
            .filter(|r| r.since.elapsed() >= SPIN_AFTER)
            .map(|r| r.since.elapsed().as_secs())
            .or_else(|| self.queued.then_some(0))
    }

    fn cancel(&mut self) -> bool {
        let queued = std::mem::take(&mut self.queued);
        match self.run.take() {
            Some(run) => {
                run.cancel.store(true, Ordering::Relaxed);
                true
            }
            None => queued,
        }
    }
}

impl Drop for Suggest {
    fn drop(&mut self) {
        self.cancel();
    }
}

impl ReviewView {
    /// The ✦ controls exist.
    pub(super) fn ai_on(&self) -> bool {
        self.sug.enabled()
    }

    /// Esc while a suggestion is being made: stop it instead of closing.
    pub(super) fn cancel_suggest(&mut self) -> bool {
        let was = self.sug.cancel();
        if was {
            self.message = Some(msg::CANCELED.get().to_string());
        }
        was
    }

    /// "✦ Draft review...": the options dialog, prefilled from the defaults.
    /// The run itself starts when the reviewer confirms it.
    pub(super) fn open_options(&mut self) {
        if !self.sug.enabled() {
            self.message = Some(msg::WHY_OFF.get().to_string());
            return;
        }
        if self.sug.running() {
            self.message = Some(msg::WHY_BUSY.get().to_string());
            return;
        }
        #[cfg(test)]
        if let Some(defaults) = self.sug.fake_defaults.clone() {
            self.options = Some(super::options::OptionsDialog::new(defaults));
            return;
        }
        let defaults = super::options::load_defaults(
            self.target.repo_root.as_deref(),
            self.remote_file().as_deref(),
            &self.target.remote_key(),
        );
        self.options = Some(super::options::OptionsDialog::new(defaults));
    }

    /// `remote-reviews.toml`, for a request reviewed without a clone.
    fn remote_file(&self) -> Option<std::path::PathBuf> {
        match &self.store {
            super::model::DraftStore::Remote(file) => file.clone(),
            super::model::DraftStore::Repo(_) => None,
        }
    }

    /// What the options dialog answered.
    pub(super) fn on_options(&mut self, outcome: super::options::OptionsOutcome) -> ReviewEffect {
        use super::options::OptionsOutcome as O;
        match outcome {
            O::Stay => {}
            O::Cancel => self.options = None,
            O::Run(choice) => {
                self.options = None;
                let saved = choice.save.as_ref().map(|values| {
                    super::options::save_defaults(
                        self.target.repo_root.as_deref(),
                        self.remote_file().as_deref(),
                        &self.target.remote_key(),
                        &choice.state_dir,
                        values,
                    )
                });
                self.start_suggest(choice.per_run);
                match saved {
                    Some(Ok(())) => {
                        self.message = Some(
                            gitpoulpi_i18n::msg::review_options::DEFAULTS_SAVED
                                .get()
                                .into(),
                        );
                    }
                    Some(Err(why)) => {
                        let text = gitpoulpi_i18n::msg::review_options::save_failed(why);
                        crate::activity_log::report_problem(
                            crate::activity_log::ProblemSource::Save,
                            &text,
                        );
                        self.message = Some(text);
                    }
                    None => {}
                }
            }
        }
        ReviewEffect::None
    }

    /// One explicit run, with the choices of the options dialog.
    pub(super) fn start_suggest(&mut self, per_run: gitpoulpi_config::RepoAiReview) {
        if !self.sug.enabled() {
            self.message = Some(msg::WHY_OFF.get().to_string());
            return;
        }
        if self.sug.running() {
            self.message = Some(msg::WHY_BUSY.get().to_string());
            return;
        }
        self.sug.per_run = per_run;
        let (Some(host), Some(git)) = (&self.host, &self.git) else {
            // The code is still coming: ask as soon as it is here.
            if self.host.is_some() && self.git_err.is_none() {
                self.sug.queued = true;
                self.message = Some(super::msg::FETCHING.get().to_string());
            } else {
                self.message = Some(msg::WHY_NO_DIFF.get().to_string());
            }
            return;
        };
        let threads = host
            .threads
            .iter()
            .map(|t| BriefThread {
                path: t.path.clone(),
                line: t.line.or(t.original_line),
                resolved: t.resolved,
                comments: t
                    .comments
                    .iter()
                    .map(|c| (c.author.clone(), c.body.clone()))
                    .collect(),
            })
            .collect();
        let cancel = Arc::new(AtomicBool::new(false));
        let input = AskInput {
            session: self.sug.session.clone(),
            repo_root: self.target.repo_root.clone(),
            source: self.source.clone(),
            #[cfg(test)]
            fake: self.sug.fake.clone(),
            title: self.target.title.clone(),
            description: host.detail.body.clone(),
            threads,
            base: git.diff_base,
            head: git.head,
            cancel: cancel.clone(),
            remote_key: self.target.remote_key(),
            remote_file: match &self.store {
                super::model::DraftStore::Remote(file) => file.clone(),
                super::model::DraftStore::Repo(_) => None,
            },
            integrations: self.target.all.clone(),
            per_run: self.sug.per_run.clone(),
        };
        let rx = super::ai_worker::start(input);
        self.message = None;
        self.sug.queued = false;
        self.sug.run = Some(Run {
            rx,
            cancel,
            since: Instant::now(),
        });
    }

    /// Drains the worker. Called once per frame.
    pub(super) fn poll_suggest(&mut self) {
        if self.sug.queued {
            if self.git.is_some() {
                self.sug.queued = false;
                self.start_suggest(self.sug.per_run.clone());
            } else if self.git_err.is_some() {
                self.sug.queued = false;
                self.message = Some(msg::WHY_NO_DIFF.get().to_string());
            }
        }
        let Some(run) = &self.sug.run else { return };
        let answer = match run.rx.try_recv() {
            Ok(answer) => answer,
            Err(mpsc::TryRecvError::Empty) => return,
            Err(mpsc::TryRecvError::Disconnected) => Err(super::msg::LOAD_STOPPED.get().into()),
        };
        self.sug.run = None;
        match answer {
            Ok(done) => self.on_suggestions(done),
            // Canceled: the message was said when it was.
            Err(why) if why.is_empty() => {}
            Err(why) => {
                crate::activity_log::report_problem(
                    crate::activity_log::ProblemSource::Hosting,
                    &why,
                );
                self.message = Some(msg::failed(&why));
            }
        }
    }

    pub(super) fn on_suggestions(&mut self, done: Done) {
        let mut items = done.items;
        for item in &mut items {
            item.id = self.sug.next_id;
            self.sug.next_id += 1;
        }
        self.message = Some(if items.is_empty() {
            msg::NOTHING.get().to_string()
        } else {
            msg::ready(items.len() as u64)
        });
        self.sug.notes.clear();
        if done.cut > 0 {
            self.sug.notes.push(msg::cut_note(done.cut as u64));
        }
        if done.left_out > 0 {
            self.sug
                .notes
                .push(msg::left_out_note(done.left_out as u64));
        }
        self.sug.notes.extend(done.context_notes);
        self.sug.items = items;
        self.sug.summary = done.summary;
        self.sug.body = done.body;
        self.sug.verdict = done.verdict;
        self.sug.head_sha = self
            .host
            .as_ref()
            .map(|h| h.detail.head_sha.clone())
            .unwrap_or_default();
        self.sug.done = true;
        self.ov_offset = 0;
        self.revision += 1;
        self.save_draft();
    }

    // -- the ✦ draft ---------------------------------------------------------------

    /// The Overview shows the draft card: the model wrote a text or
    /// suggestions the reviewer has not dealt with.
    pub(super) fn has_draft(&self) -> bool {
        self.sug.enabled()
            && self.sug.done
            && (!self.sug.body.trim().is_empty() || !self.sug.items.is_empty())
    }

    /// The request's head moved since the draft was written.
    pub(super) fn draft_stale(&self) -> bool {
        !self.sug.head_sha.is_empty()
            && self
                .host
                .as_ref()
                .is_some_and(|h| h.detail.head_sha != self.sug.head_sha)
    }

    /// What `RepoState` keeps of the draft (`None`: nothing to keep).
    pub(super) fn ai_draft(&self) -> Option<AiDraft> {
        let s = &self.sug;
        if !s.done || (s.body.trim().is_empty() && s.items.is_empty()) {
            return None;
        }
        Some(AiDraft {
            body: s.body.clone(),
            verdict: s.verdict.as_str().to_string(),
            summary: s.summary.clone(),
            items: s.items.iter().map(item_to_session).collect(),
            head_sha: s.head_sha.clone(),
        })
    }

    /// Takes back a draft kept by an earlier visit.
    pub(super) fn restore_draft(&mut self, draft: AiDraft) {
        let mut items: Vec<Item> = draft.items.iter().map(item_from_session).collect();
        for item in &mut items {
            item.id = self.sug.next_id;
            self.sug.next_id += 1;
        }
        self.sug.items = items;
        self.sug.body = draft.body;
        self.sug.summary = draft.summary;
        self.sug.verdict = ReviewEvent::parse(&draft.verdict).unwrap_or(ReviewEvent::Comment);
        self.sug.head_sha = draft.head_sha;
        self.sug.done = true;
    }

    /// The draft is spent (submitted) or thrown away.
    pub(super) fn clear_draft(&mut self) {
        self.sug.items.clear();
        self.sug.body.clear();
        self.sug.summary.clear();
        self.sug.notes.clear();
        self.sug.verdict = ReviewEvent::Comment;
        self.sug.head_sha.clear();
        self.sug.done = false;
        self.revision += 1;
    }

    /// "Discard the ✦ draft": the text, the verdict and the suggestions.
    pub(super) fn discard_draft(&mut self) {
        if !self.has_draft() {
            self.message = Some(msg::WHY_NO_DRAFT.get().to_string());
            return;
        }
        self.clear_draft();
        self.save_draft();
        self.message = Some(msg::DRAFT_DISCARDED.get().to_string());
    }

    /// "✦ Use the draft": the open suggestions become pending comments and
    /// the Finish review form opens with the draft's text and verdict. The
    /// host decides what it can take (GitLab has no Request changes, Gerrit
    /// takes a vote). Nothing is sent; cancelling keeps the draft.
    pub(super) fn use_draft(&mut self) {
        if !self.has_draft() {
            self.message = Some(msg::WHY_NO_DRAFT.get().to_string());
            return;
        }
        if !self
            .host
            .as_ref()
            .is_some_and(|h| h.detail.capabilities.review)
        {
            // Says why (not loaded / no review on this host).
            self.open_finish();
            return;
        }
        if !self.sug.items.is_empty() {
            self.accept_all_suggestions();
        }
        self.open_finish();
        let (body, verdict) = (self.sug.body.clone(), self.sug.verdict);
        if let Some(f) = self.finish.as_mut() {
            f.message = f.prefill(&body, verdict);
        }
    }

    /// "Edit the draft text": the comment box on the review text.
    pub(super) fn edit_draft_text(&mut self) {
        if !self.has_draft() {
            self.message = Some(msg::WHY_NO_DRAFT.get().to_string());
            return;
        }
        let body = self.sug.body.clone();
        self.open_compose(ComposeFor::DraftText, body);
    }

    pub(super) fn finish_draft_text(&mut self, text: String) {
        self.sug.body = text.trim().to_string();
        self.revision += 1;
        self.save_draft();
        self.message = Some(msg::DRAFT_TEXT_SAVED.get().to_string());
    }

    // -- acting on suggestions -----------------------------------------------------

    /// Index in the list of the suggestion the keyboard acts on: the one under
    /// the cursor on the Files page, else the first of the list.
    pub(super) fn current_suggestion(&self) -> Option<usize> {
        if self.tab == crate::review_cmd::Tab::Files {
            self.ghost_at_cursor()
        } else if self.tab == crate::review_cmd::Tab::Overview {
            (!self.sug.items.is_empty()).then_some(0)
        } else {
            None
        }
    }

    /// `(index, suggestion)` of the suggestions about `path`.
    pub(super) fn ghosts_of(&self, path: &str) -> Vec<(usize, &Item)> {
        self.sug
            .items
            .iter()
            .enumerate()
            .filter(|(_, i)| i.path == path)
            .collect()
    }

    /// The suggestion the Files cursor is on: in its rows, or on the line
    /// right above it (after any thread and pending comment of that line).
    pub(super) fn ghost_at_cursor(&self) -> Option<usize> {
        use super::diffrows::RowKind;
        let p = &self.files_pane;
        match p.kind(p.cursor)? {
            RowKind::GhostHead { item } | RowKind::GhostBody { item } => Some(item),
            RowKind::Line { .. } => {
                let mut at = p.cursor + 1;
                while let Some(kind) = p.kind(at) {
                    match kind {
                        RowKind::ThreadHead { .. }
                        | RowKind::ThreadBody { .. }
                        | RowKind::PendingHead { .. }
                        | RowKind::PendingBody { .. } => at += 1,
                        RowKind::GhostHead { item } => return Some(item),
                        _ => return None,
                    }
                }
                None
            }
            _ => None,
        }
    }

    /// Accepts suggestion `index` as a pending comment, with `text` instead
    /// of the model's when the reviewer edited it.
    pub(super) fn accept_suggestion(&mut self, index: usize, text: Option<String>) {
        let Some(item) = self.sug.items.get(index).cloned() else {
            self.message = Some(msg::WHY_NO_SUGGESTION.get().to_string());
            return;
        };
        let Some(comment) = self.comment_of(&item, text.as_deref()) else {
            self.message = Some(msg::WHY_NO_PLACE.get().to_string());
            return;
        };
        self.sug.items.remove(index);
        self.add_pending(comment);
        self.message = Some(msg::ACCEPTED.get().to_string());
    }

    /// The pending comment an accepted `item` becomes.
    fn comment_of(&self, item: &Item, text: Option<&str>) -> Option<PendingComment> {
        let place = item.place?;
        let previous_path = self
            .host
            .as_ref()
            .and_then(|h| h.files.iter().find(|f| f.path == item.path))
            .and_then(|f| f.previous_path.clone());
        Some(PendingComment {
            path: item.path.clone(),
            previous_path,
            line: place.line,
            side: place.side,
            start_line: place.start_line.filter(|_| self.can_range()),
            other_line: place.other_line,
            body: format!("{}{}", item.prefix, text.unwrap_or(&item.body).trim()),
        })
    }

    pub(super) fn dismiss_suggestion(&mut self, index: usize) {
        if index < self.sug.items.len() {
            self.sug.items.remove(index);
            self.revision += 1;
            self.save_draft();
            self.message = Some(msg::DISMISSED.get().to_string());
        }
    }

    /// Every suggestion that has a place becomes a pending comment (the
    /// others stay listed). Nothing goes to the host.
    pub(super) fn accept_all_suggestions(&mut self) {
        let items = std::mem::take(&mut self.sug.items);
        let mut keep = Vec::new();
        let mut comments = Vec::new();
        for item in items {
            match self.comment_of(&item, None) {
                Some(c) => comments.push(c),
                None => keep.push(item),
            }
        }
        self.sug.items = keep;
        let n = comments.len();
        self.add_pending_batch(comments);
        self.save_draft();
        self.message = Some(if self.sug.items.is_empty() || n > 0 {
            msg::accepted_all(n as u64)
        } else {
            msg::WHY_NO_PLACE.get().to_string()
        });
    }

    pub(super) fn dismiss_all_suggestions(&mut self) {
        self.sug.items.clear();
        self.revision += 1;
        self.save_draft();
        self.message = Some(msg::DISMISSED_ALL.get().to_string());
    }

    /// Opens the comment box on suggestion `index`; submitting accepts it
    /// with the edited words.
    pub(super) fn edit_suggestion(&mut self, index: usize) {
        let Some(item) = self.sug.items.get(index) else {
            self.message = Some(msg::WHY_NO_SUGGESTION.get().to_string());
            return;
        };
        if item.place.is_none() {
            self.message = Some(msg::WHY_NO_PLACE.get().to_string());
            return;
        }
        let (id, body) = (item.id, item.body.clone());
        self.open_compose(ComposeFor::Suggestion(id), body);
    }

    /// The comment box of suggestion `id` was submitted.
    pub(super) fn finish_suggestion_edit(&mut self, id: u64, text: String) {
        match self.sug.items.iter().position(|i| i.id == id) {
            Some(index) => self.accept_suggestion(index, Some(text)),
            None => self.message = Some(msg::WHY_NO_SUGGESTION.get().to_string()),
        }
    }

    /// "Use AI summary": puts the model's summary in the Finish review
    /// text, opening the dialog when it is not open.
    pub(super) fn use_ai_summary(&mut self) -> ReviewEffect {
        if self.sug.summary.trim().is_empty() {
            self.message = Some(msg::WHY_NO_SUMMARY.get().to_string());
            return ReviewEffect::None;
        }
        if self.finish.is_none() {
            self.open_finish();
        }
        if let Some(f) = self.finish.as_mut() {
            f.use_summary();
            self.message = Some(msg::SUMMARY_INSERTED.get().to_string());
        }
        ReviewEffect::None
    }

    /// The text of the in-flight line.
    pub(super) fn suggest_status(&self) -> Option<String> {
        self.sug.progress().map(|secs| {
            if self.sug.run.is_some() {
                format!("{} \u{2726} {}", self.spinner(), msg::working(secs))
            } else {
                format!("{} \u{2726} {}", self.spinner(), super::msg::FETCHING.get())
            }
        })
    }
}

fn side_key(side: DiffSide) -> String {
    side.as_str().to_string()
}

fn item_to_session(i: &Item) -> AiItem {
    AiItem {
        path: i.path.clone(),
        line: i.line,
        severity: match i.severity {
            Sev::Info => "info",
            Sev::Warning => "warning",
            Sev::Issue => "issue",
        }
        .to_string(),
        body: i.body.clone(),
        anchored: i.anchored,
        place: i.place.map(|p| AiPlace {
            side: side_key(p.side),
            line: p.line,
            other_line: p.other_line,
            start_line: p.start_line,
        }),
        prefix: i.prefix.clone(),
    }
}

fn item_from_session(i: &AiItem) -> Item {
    Item {
        id: 0,
        path: i.path.clone(),
        line: i.line,
        severity: match i.severity.as_str() {
            "issue" => Sev::Issue,
            "warning" => Sev::Warning,
            _ => Sev::Info,
        },
        body: i.body.clone(),
        anchored: i.anchored,
        place: i.place.as_ref().map(|p| Place {
            side: DiffSide::parse(&p.side).unwrap_or_default(),
            line: p.line,
            other_line: p.other_line,
            start_line: p.start_line,
        }),
        prefix: i.prefix.clone(),
    }
}
