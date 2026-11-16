//! Keys, mouse and the commands the palette shares with them. Every mouse
//! action here has a key, and every key and button a [`ReviewCmd`] the
//! command palette lists (maintainer rules 4 and 6): `run` is the one place an
//! action happens, however it was asked for.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use gitpoulpi_hosting::{DiffSide, PendingComment};
use gitpoulpi_i18n::msg::review_view as msg;

use super::diffrows::{code_line, RowKind};
use super::draw::Hit;
use super::load::Call;
use super::model::Focus;
use super::popups::{
    ComposeBox, ComposeFor, ComposeOutcome, Confirm, ConfirmOutcome, FinishOutcome,
};
use super::{ConfirmFor, PaneId, ReviewEffect, ReviewView};
use crate::diff_view::DiffRow;
use crate::diff_view::{hunk_jump, DiffBarButton, DiffViewToggle};
use crate::meta_picker::MetaField;
use crate::review_cmd::{ReviewCmd, Tab};

/// Cells one sideways step scrolls (the Diff view's `h` / `l`).
const H_STEP: isize = 4;

/// The command a plain key stands for on `tab`.
pub(super) fn key_cmd(tab: Tab, key: &KeyEvent) -> Option<ReviewCmd> {
    let alt = key.modifiers.contains(KeyModifiers::ALT);
    let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
    match key.code {
        KeyCode::Char('1') if alt => return Some(ReviewCmd::TabOverview),
        KeyCode::Char('2') if alt => return Some(ReviewCmd::TabFiles),
        KeyCode::Char('3') if alt => return Some(ReviewCmd::TabCommits),
        KeyCode::F(5) => return Some(ReviewCmd::Refresh),
        KeyCode::Char('r') if ctrl => return Some(ReviewCmd::Refresh),
        // The Diff view's copy: the selected text, else the line.
        KeyCode::Char('c') if ctrl && tab != Tab::Overview => return Some(ReviewCmd::CopyText),
        _ => {}
    }
    if alt || ctrl {
        return None;
    }
    Some(match key.code {
        KeyCode::Char('c') if tab == Tab::Overview => ReviewCmd::ConversationComment,
        KeyCode::Char('c') => ReviewCmd::AddComment,
        KeyCode::Char('C') => ReviewCmd::ConversationComment,
        KeyCode::Char('r') => ReviewCmd::Reply,
        KeyCode::Char('x') => ReviewCmd::ToggleResolve,
        KeyCode::Char('z') => ReviewCmd::ToggleThread,
        KeyCode::Char('e') => ReviewCmd::EditPending,
        KeyCode::Char('d') => ReviewCmd::DeletePending,
        KeyCode::Char('v') => ReviewCmd::ToggleViewed,
        KeyCode::Char(']') => ReviewCmd::NextFile,
        KeyCode::Char('[') => ReviewCmd::PrevFile,
        KeyCode::Char('f') => ReviewCmd::FinishReview,
        KeyCode::Char('o') => ReviewCmd::OpenBrowser,
        KeyCode::Char('b') => ReviewCmd::Checkout,
        KeyCode::Char('a') => ReviewCmd::EditAssignees,
        KeyCode::Char('p') => ReviewCmd::EditReviewers,
        KeyCode::Char('l') => ReviewCmd::EditLabels,
        KeyCode::Char('m') => ReviewCmd::SetMilestone,
        KeyCode::Char('P') => ReviewCmd::PickPatchSet,
        KeyCode::Char('D') => ReviewCmd::ComparePatchSet,
        KeyCode::Char('U') => ReviewCmd::SubmitChange,
        KeyCode::Char('Q') => ReviewCmd::AbandonChange,
        // The ✦ suggestions.
        KeyCode::Char('s') => ReviewCmd::SuggestReview,
        KeyCode::Char('y') => ReviewCmd::AcceptSuggestion,
        KeyCode::Char('w') => ReviewCmd::EditSuggestion,
        KeyCode::Char('n') => ReviewCmd::DismissSuggestion,
        KeyCode::Char('A') => ReviewCmd::AcceptAllSuggestions,
        KeyCode::Char('N') => ReviewCmd::DismissAllSuggestions,
        KeyCode::Char('i') => ReviewCmd::UseSummary,
        KeyCode::Char('u') => ReviewCmd::UseDraft,
        KeyCode::Char('t') => ReviewCmd::EditDraftText,
        // The Diff view's toggles on the diff pane. Its own letters are the
        // review's (n, p, m, w, s), so these are the capitals: V(iew) layout,
        // H(unks), S(pace), W(rap), and J / K for the next / previous hunk.
        KeyCode::Char('V') if tab != Tab::Overview => ReviewCmd::ToggleLayout,
        KeyCode::Char('H') if tab != Tab::Overview => ReviewCmd::ToggleHunksOnly,
        KeyCode::Char('S') if tab != Tab::Overview => ReviewCmd::ToggleWhitespace,
        KeyCode::Char('W') if tab != Tab::Overview => ReviewCmd::ToggleWrap,
        KeyCode::Char('J') if tab != Tab::Overview => ReviewCmd::NextHunk,
        KeyCode::Char('K') if tab != Tab::Overview => ReviewCmd::PrevHunk,
        KeyCode::Char('E') if tab != Tab::Overview => ReviewCmd::ChooseEncoding,
        KeyCode::Char('M') if tab != Tab::Overview => ReviewCmd::ToggleMinimap,
        _ => return None,
    })
}

impl ReviewView {
    // -- where the cursor is ------------------------------------------------------

    pub(super) fn pane(&self) -> &super::pane::DiffPane {
        match self.tab {
            Tab::Commits => &self.commits_pane,
            _ => &self.files_pane,
        }
    }

    fn pane_of(&mut self, id: PaneId) -> &mut super::pane::DiffPane {
        match id {
            PaneId::Files => &mut self.files_pane,
            PaneId::Commits => &mut self.commits_pane,
        }
    }

    fn active_pane(&mut self) -> &mut super::pane::DiffPane {
        match self.tab {
            Tab::Commits => &mut self.commits_pane,
            _ => &mut self.files_pane,
        }
    }

    /// The thread the cursor is in, or on the line right above.
    pub(super) fn thread_at_cursor(&self) -> Option<usize> {
        let p = &self.files_pane;
        match p.kind(p.cursor)? {
            RowKind::ThreadHead { thread } | RowKind::ThreadBody { thread, .. } => Some(thread),
            RowKind::Line { .. } => match p.kind(p.cursor + 1)? {
                RowKind::ThreadHead { thread } => Some(thread),
                _ => None,
            },
            _ => None,
        }
    }

    pub(super) fn pending_at_cursor(&self) -> Option<usize> {
        let p = &self.files_pane;
        match p.kind(p.cursor)? {
            RowKind::PendingHead { pending } | RowKind::PendingBody { pending } => Some(pending),
            RowKind::Line { .. } => {
                // A pending comment sits under its line, after any thread.
                let mut at = p.cursor + 1;
                while let Some(kind) = p.kind(at) {
                    match kind {
                        RowKind::ThreadHead { .. } | RowKind::ThreadBody { .. } => at += 1,
                        RowKind::PendingHead { pending } => return Some(pending),
                        _ => return None,
                    }
                }
                None
            }
            _ => None,
        }
    }

    /// What a new comment would be about, or why there is none.
    pub(super) fn comment_target(&self) -> Result<ComposeFor, &'static str> {
        if self.tab != Tab::Files {
            return Err(msg::WHY_NOT_ON_FILES.get());
        }
        if self.git.is_none() {
            return Err(msg::NOT_FETCHED.get());
        }
        let path = self.current_path().ok_or(msg::WHY_NOT_ON_CODE.get())?;
        let p = &self.files_pane;
        if p.is_empty() {
            return Err(msg::WHY_NOT_ON_CODE.get());
        }
        let (lo, hi) = p.selection();
        let side = p.side;
        let line_of = |row: &DiffRow| code_line(row, side);
        let mut lines: Vec<(DiffSide, u32, Option<u32>)> = p
            .rows()
            .get(lo..=hi.min(p.len().saturating_sub(1)))
            .unwrap_or(&[])
            .iter()
            .filter_map(line_of)
            .collect();
        if lines.is_empty() {
            // On a thread or a pending comment: the line above it.
            if let Some(found) = p.rows()[..=p.cursor.min(p.len().saturating_sub(1))]
                .iter()
                .rev()
                .find_map(line_of)
            {
                lines.push(found);
            }
        }
        let Some(&(side, line, other)) = lines.last() else {
            return Err(msg::WHY_NOT_ON_CODE.get());
        };
        let start_line = if self.can_range() && lo != hi {
            lines
                .iter()
                .find(|(s, _, _)| *s == side)
                .map(|(_, l, _)| *l)
                .filter(|l| *l != line)
        } else {
            None
        };
        let previous_path = self
            .host
            .as_ref()
            .and_then(|h| h.files.iter().find(|f| f.path == path))
            .and_then(|f| f.previous_path.clone());
        Ok(ComposeFor::Line {
            path,
            previous_path,
            side,
            line,
            start_line,
            other_line: other,
        })
    }

    // -- commands -----------------------------------------------------------------------

    pub(crate) fn run(&mut self, cmd: ReviewCmd) -> ReviewEffect {
        if let Some(why) = cmd.unavailable(Some(self.avail())) {
            self.message = Some(why.to_string());
            return ReviewEffect::None;
        }
        match cmd {
            ReviewCmd::TabOverview => self.set_tab(Tab::Overview),
            ReviewCmd::TabFiles => self.set_tab(Tab::Files),
            ReviewCmd::TabCommits => self.set_tab(Tab::Commits),
            ReviewCmd::BackToComment => self.set_tab(Tab::Overview),
            ReviewCmd::AddComment => match self.comment_target() {
                Ok(purpose) => self.open_compose(purpose, String::new()),
                Err(why) => self.message = Some(why.to_string()),
            },
            ReviewCmd::ConversationComment => {
                // The conversation is the Overview's: its composer opens.
                self.set_tab(Tab::Overview);
                let effect = self.ov.run(crate::item_cmd::ItemCmd::NewComment);
                return self.on_overview_effect(effect);
            }
            ReviewCmd::Reply => {
                if let Some(thread) = self.thread_at_cursor() {
                    let id = self
                        .host
                        .as_ref()
                        .and_then(|h| h.threads.get(thread))
                        .map(|t| t.id.clone());
                    if let Some(thread_id) = id {
                        self.open_compose(ComposeFor::Reply { thread_id }, String::new());
                    }
                }
            }
            ReviewCmd::ToggleResolve => {
                let found = self.thread_at_cursor().and_then(|i| {
                    self.host
                        .as_ref()
                        .and_then(|h| h.threads.get(i))
                        .map(|t| (t.id.clone(), t.resolved))
                });
                if let Some((thread_id, resolved)) = found {
                    self.start_call(
                        Call::Resolve {
                            thread_id,
                            resolved: !resolved,
                        },
                        false,
                        None,
                    );
                }
            }
            ReviewCmd::ToggleThread => {
                let id = self.thread_at_cursor().and_then(|i| {
                    self.host
                        .as_ref()
                        .and_then(|h| h.threads.get(i))
                        .map(|t| t.id.clone())
                });
                if let Some(id) = id {
                    if !self.collapsed.remove(&id) {
                        self.collapsed.insert(id);
                    }
                    self.revision += 1;
                }
            }
            ReviewCmd::EditPending => {
                if let Some(i) = self.pending_at_cursor() {
                    let text = self.pending.get(i).map(|c| c.body.clone());
                    if let Some(text) = text {
                        self.open_compose(ComposeFor::EditPending(i), text);
                    }
                }
            }
            ReviewCmd::DeletePending => {
                if let Some(i) = self.pending_at_cursor() {
                    self.delete_pending(i);
                }
            }
            ReviewCmd::ToggleViewed => {
                if let Some(path) = self.current_path() {
                    self.toggle_viewed(&path);
                }
            }
            ReviewCmd::NextFile => self.step_file(1),
            ReviewCmd::PrevFile => self.step_file(-1),
            ReviewCmd::FinishReview => self.open_finish(),
            ReviewCmd::DiscardReview => {
                self.confirm = Some((
                    ConfirmFor::DiscardReview,
                    Confirm::new(
                        msg::CONFIRM_TITLE.get(),
                        msg::discard_prompt(self.pending.len() as u64),
                        msg::DISCARD_YES.get(),
                    ),
                ));
            }
            ReviewCmd::Refresh => {
                self.refresh();
                self.message = Some(msg::REFRESHING.get().to_string());
            }
            ReviewCmd::OpenBrowser => {
                self.message = Some(match crate::repo_menu::open_url(&self.target.web_url) {
                    Ok(()) => msg::opened_in_browser(&self.target.iid_label),
                    Err(e) => msg::browser_failed(&e),
                });
            }
            ReviewCmd::Checkout => return ReviewEffect::Checkout(self.target.number),
            ReviewCmd::Close => return ReviewEffect::Close,
            ReviewCmd::EditAssignees => self.open_meta(MetaField::Assignees),
            ReviewCmd::EditReviewers => self.open_meta(MetaField::Reviewers),
            ReviewCmd::EditLabels => self.open_meta(MetaField::Labels),
            ReviewCmd::SetMilestone => self.open_meta(MetaField::Milestone),
            ReviewCmd::PickPatchSet => self.open_patch_sets(),
            ReviewCmd::ComparePatchSet => self.open_compare(),
            ReviewCmd::SubmitChange => self.ask_change(ConfirmFor::SubmitChange),
            ReviewCmd::AbandonChange => self.ask_change(ConfirmFor::AbandonChange),
            ReviewCmd::SuggestReview => self.open_options(),
            ReviewCmd::AcceptSuggestion => {
                if let Some(i) = self.current_suggestion() {
                    self.accept_suggestion(i, None);
                }
            }
            ReviewCmd::EditSuggestion => {
                if let Some(i) = self.current_suggestion() {
                    self.edit_suggestion(i);
                }
            }
            ReviewCmd::DismissSuggestion => {
                if let Some(i) = self.current_suggestion() {
                    self.dismiss_suggestion(i);
                }
            }
            ReviewCmd::AcceptAllSuggestions => self.accept_all_suggestions(),
            ReviewCmd::DismissAllSuggestions => self.dismiss_all_suggestions(),
            ReviewCmd::UseSummary => return self.use_ai_summary(),
            ReviewCmd::UseDraft => self.use_draft(),
            ReviewCmd::EditDraftText => self.edit_draft_text(),
            ReviewCmd::DiscardDraft => self.discard_draft(),
            ReviewCmd::ToggleLayout => {
                let next = self.diff_opts.mode.next();
                self.apply_toggle(DiffViewToggle::Mode(next));
            }
            ReviewCmd::ToggleHunksOnly => self.apply_toggle(DiffViewToggle::ToggleHunksOnly),
            ReviewCmd::ToggleWhitespace => self.apply_toggle(DiffViewToggle::ToggleWhitespace),
            ReviewCmd::ToggleWrap => self.apply_toggle(DiffViewToggle::ToggleWordWrap),
            ReviewCmd::NextHunk => self.jump_hunk(true),
            ReviewCmd::PrevHunk => self.jump_hunk(false),
            ReviewCmd::ChooseEncoding => self.open_encoding_menu(),
            ReviewCmd::ToggleMinimap => {
                let path = crate::live_settings::diff_config_path();
                self.message = Some(crate::diff_actions::toggle_minimap(&path));
                self.revision += 1;
            }
            ReviewCmd::ScrollLeft => self.scroll_h(-H_STEP),
            ReviewCmd::ScrollRight => self.scroll_h(H_STEP),
            ReviewCmd::CopyText => self.copy_text(),
        }
        ReviewEffect::None
    }

    /// One of the Diff view's display toggles: the same state change, the same
    /// preference published for the next diff and written to `config.toml`
    /// (`persist_diff_view_toggle`), then the rows of both panes are rebuilt
    /// from the top (a cursor row of the old layout means nothing in the new).
    pub(super) fn apply_toggle(&mut self, toggle: DiffViewToggle) {
        let path = crate::live_settings::diff_config_path();
        crate::diff_actions::persist_diff_view_toggle(toggle, &mut self.diff_opts, &path);
        for pane in [&mut self.files_pane, &mut self.commits_pane] {
            pane.cursor = 0;
            pane.offset = 0;
            pane.anchor = None;
        }
        self.revision += 1;
    }

    /// The encoding dropdown, hung under the bar's button (or under the `⋯`
    /// that hides it).
    fn open_encoding_menu(&mut self) {
        if self.source.is_api() {
            self.message = Some(msg::NO_CLONE_ENCODING.get().to_string());
            return;
        }
        let Some(path) = self.enc_file() else {
            self.message = Some(msg::WHY_NOT_ON_CODE.get().to_string());
            return;
        };
        let anchor = self
            .bar_slots
            .encoding
            .or_else(|| self.bar_slots.more.as_ref().map(|(r, _)| *r))
            .unwrap_or(self.pane().area);
        let current = crate::diff_recode::remembered(std::path::Path::new(&path));
        self.bar_menu = Some(super::barmenu::BarMenu::encoding(current, anchor));
    }

    /// A choice in the encoding dropdown: remembered for the file (shared
    /// with the Diff view), then the file is read again.
    fn pick_encoding(&mut self, pick: Option<crate::editor_charset::Charset>) {
        let Some(path) = self.enc_file() else {
            return;
        };
        crate::diff_recode::remember(std::path::Path::new(&path), pick);
        self.recodes.forget(&path);
        for pane in [&mut self.files_pane, &mut self.commits_pane] {
            pane.cursor = 0;
            pane.offset = 0;
            pane.anchor = None;
            pane.text_sel = None;
        }
        self.diff_opts.h_offset = 0;
        self.revision += 1;
        self.message = Some(match pick {
            Some(c) => gitpoulpi_i18n::msg::diff_view::enc_chosen(c.name()),
            None => gitpoulpi_i18n::msg::diff_view::ENC_AUTO_CHOSEN
                .get()
                .to_string(),
        });
    }

    /// A choice in the `⋯` list: the hidden button runs as if it had been
    /// clicked in place.
    fn run_bar_button(&mut self, button: DiffBarButton) -> ReviewEffect {
        let mouse = MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 0,
            row: 0,
            modifiers: KeyModifiers::NONE,
        };
        self.click(Hit::Bar(button), mouse)
    }

    /// The text selection, or the line (the `v` range) under the cursor, onto
    /// the clipboard: the Diff view's own copy.
    fn copy_text(&mut self) {
        let state = self.pane().view_state(&self.diff_opts);
        let Some(render) = self.pane().render.clone() else {
            self.message = Some(
                gitpoulpi_i18n::msg::diff_view::NO_TEXT_ROW
                    .get()
                    .to_string(),
            );
            return;
        };
        self.message = Some(match crate::diff_view::yank(&render, &state) {
            Ok((what, text)) => crate::clipboard::copy_shared(what, &text),
            Err(why) => why.to_string(),
        });
    }

    /// Scrolls the diff's text sideways by `delta` cells (the Diff view's
    /// `h` / `l`), within what the open render can scroll.
    pub(super) fn scroll_h(&mut self, delta: isize) {
        let max = self.pane().h_max;
        let to = (self.diff_opts.h_offset as isize + delta).clamp(0, max as isize) as usize;
        self.set_h_offset(to);
    }

    pub(super) fn set_h_offset(&mut self, to: usize) {
        if self.diff_opts.word_wrap || to == self.diff_opts.h_offset {
            return;
        }
        self.diff_opts.h_offset = to;
        self.revision += 1;
    }

    /// The cursor onto the next / previous hunk header of the open diff.
    fn jump_hunk(&mut self, forward: bool) {
        let pane = self.active_pane();
        let starts = pane
            .render
            .as_ref()
            .map(|r| r.hunk_starts.clone())
            .unwrap_or_default();
        let visible = pane.offset..pane.offset + pane.visible();
        let from = if visible.contains(&pane.cursor) {
            pane.cursor
        } else {
            pane.offset
        };
        match hunk_jump(&starts, from, forward) {
            Some(row) => {
                pane.jump(row);
                pane.offset = row.min(pane.len().saturating_sub(pane.visible()));
            }
            None => {
                self.message = Some(
                    if forward {
                        gitpoulpi_i18n::msg::diff_view::NO_FURTHER_HUNK.get()
                    } else {
                        gitpoulpi_i18n::msg::diff_view::FIRST_HUNK.get()
                    }
                    .to_string(),
                );
            }
        }
    }

    pub(super) fn set_tab(&mut self, tab: Tab) {
        // Leaving the code a comment opened: back on that comment.
        if tab != Tab::Files {
            self.goto = None;
            if let Some(id) = self.ov_return.take() {
                if tab == Tab::Overview {
                    self.ov.select_thread(&id);
                }
            }
        }
        self.tab = tab;
        self.focus = Focus::List;
        self.message = None;
    }

    fn step_file(&mut self, delta: isize) {
        let n = self.file_count();
        if n == 0 {
            return;
        }
        let at = (self.file_sel as isize + delta).clamp(0, (n as isize - 1).max(0)) as usize;
        self.select_file(at);
    }

    pub(super) fn select_file(&mut self, at: usize) {
        if at != self.file_sel {
            self.file_sel = at;
            self.diff_err = None;
        }
        let rows = self.visible_files();
        if at < self.file_offset {
            self.file_offset = at;
        } else if at >= self.file_offset + rows {
            self.file_offset = at + 1 - rows;
        }
    }

    pub(super) fn select_commit(&mut self, at: usize) {
        let n = self.git.as_ref().map_or(0, |g| g.commits.len());
        if n == 0 {
            return;
        }
        let at = at.min(n - 1);
        if at != self.commit_sel {
            self.commit_sel = at;
            self.diff_err = None;
        }
        let rows = self.visible_files();
        if at < self.commit_offset {
            self.commit_offset = at;
        } else if at >= self.commit_offset + rows {
            self.commit_offset = at + 1 - rows;
        }
    }

    fn visible_files(&self) -> usize {
        // The list's rows as last painted; before the first paint, a screenful.
        self.list_rows.max(1)
    }

    pub(super) fn open_compose(&mut self, purpose: ComposeFor, text: String) {
        let (title, about) = match &purpose {
            ComposeFor::Line {
                path,
                line,
                start_line,
                ..
            } => (
                msg::COMPOSE_TITLE_LINE.get().to_string(),
                match start_line {
                    Some(s) => format!("{path}:{s}-{line}"),
                    None => format!("{path}:{line}"),
                },
            ),
            ComposeFor::EditPending(i) => {
                let about = self
                    .pending
                    .get(*i)
                    .map(|c| format!("{}:{}", c.path, c.line))
                    .unwrap_or_default();
                (msg::COMPOSE_TITLE_EDIT.get().to_string(), about)
            }
            ComposeFor::Reply { thread_id } => {
                let who = self
                    .host
                    .as_ref()
                    .and_then(|h| h.threads.iter().find(|t| &t.id == thread_id))
                    .and_then(|t| t.comments.first())
                    .map(|c| c.author.clone())
                    .unwrap_or_default();
                (
                    msg::COMPOSE_TITLE_REPLY.get().to_string(),
                    msg::reply_about(&who),
                )
            }
            ComposeFor::DraftText => (
                gitpoulpi_i18n::msg::review_ai::COMPOSE_TITLE_DRAFT
                    .get()
                    .to_string(),
                self.target.caption(),
            ),
            ComposeFor::Suggestion(id) => {
                let about = self
                    .sug
                    .items
                    .iter()
                    .find(|i| i.id == *id)
                    .map(|i| match i.place {
                        Some(p) => format!("{}:{}", i.path, p.line),
                        None => i.path.clone(),
                    })
                    .unwrap_or_default();
                (
                    gitpoulpi_i18n::msg::review_ai::COMPOSE_TITLE_SUGGESTION
                        .get()
                        .to_string(),
                    about,
                )
            }
        };
        // New texts keep a draft per target; edits of an existing text do not.
        let target_key = match &purpose {
            ComposeFor::Reply { thread_id } => Some(format!("reply:{thread_id}")),
            ComposeFor::Line { path, line, .. } => Some(format!("line:{path}:{line}")),
            _ => None,
        };
        let draft_key = target_key.filter(|_| text.is_empty()).map(|k| {
            crate::md_editor::draft::key(&[
                &self.target.integration.id,
                &self.target.project,
                &format!("pr#{}", self.target.number),
                &k,
            ])
        });
        let ctx = self.target.md_context();
        let ai = self.composer_ai(Some(&purpose));
        self.compose = Some(
            ComposeBox::new(purpose, title, about, text)
                .for_target(ctx, draft_key)
                .with_ai(ai),
        );
    }

    fn finish_compose(&mut self, purpose: ComposeFor, text: String) {
        match purpose {
            ComposeFor::Line {
                path,
                previous_path,
                side,
                line,
                start_line,
                other_line,
            } => {
                self.add_pending(PendingComment {
                    path,
                    previous_path,
                    line,
                    side,
                    start_line,
                    other_line,
                    body: text,
                });
                self.message = Some(msg::pending_added(self.pending.len() as u64));
            }
            ComposeFor::EditPending(i) => {
                if let Some(c) = self.pending.get_mut(i) {
                    c.body = text;
                    self.revision += 1;
                    self.save_draft();
                    self.message = Some(msg::PENDING_EDITED.get().to_string());
                }
            }
            ComposeFor::Reply { thread_id } => {
                let restore = Some((
                    ComposeFor::Reply {
                        thread_id: thread_id.clone(),
                    },
                    text.clone(),
                ));
                self.start_call(
                    Call::Reply {
                        thread_id,
                        body: text,
                    },
                    false,
                    restore,
                );
            }
            ComposeFor::Suggestion(id) => self.finish_suggestion_edit(id, text),
            ComposeFor::DraftText => self.finish_draft_text(text),
        }
    }

    // -- keys ---------------------------------------------------------------------------

    pub(crate) fn paste(&mut self, text: &str) {
        if let Some(menu) = self.bar_menu.as_mut() {
            menu.paste(text);
        } else if let Some(meta) = self.meta.as_mut() {
            meta.paste(text);
        } else if let Some(c) = self.compose.as_mut() {
            c.paste(text);
        } else if let Some(f) = self.finish.as_mut() {
            f.paste(text);
        } else if let Some(o) = self.options.as_mut() {
            o.paste(text);
        } else if self.tab == Tab::Overview {
            self.ov.paste(text);
        }
    }

    pub(crate) fn on_key(&mut self, key: KeyEvent) -> ReviewEffect {
        // Popups own the keyboard, innermost first.
        if self.patch_sets_open() {
            self.ps_key(key);
            return ReviewEffect::None;
        }
        if let Some(menu) = self.bar_menu.as_mut() {
            let outcome = menu.on_key(key);
            return self.on_bar_menu(outcome);
        }
        if let Some(meta) = self.meta.as_mut() {
            let outcome = meta.on_key(key);
            return self.on_meta(outcome).unwrap_or(ReviewEffect::None);
        }
        if let Some(compose) = self.compose.as_mut() {
            match compose.on_key(key) {
                ComposeOutcome::Stay => {}
                ComposeOutcome::Cancel => self.compose = None,
                ComposeOutcome::Submit(text) => {
                    if let Some(c) = self.compose.take() {
                        self.finish_compose(c.purpose, text);
                    }
                }
            }
            return ReviewEffect::None;
        }
        if self.confirm.is_some() {
            let outcome = self.confirm.as_mut().map(|(_, c)| c.on_key(key));
            return self.on_confirm(outcome.unwrap_or(ConfirmOutcome::Stay));
        }
        if let Some(finish) = self.finish.as_mut() {
            let outcome = finish.on_key(key);
            return self.on_finish(outcome);
        }
        if let Some(options) = self.options.as_mut() {
            let outcome = options.on_key(key);
            return self.on_options(outcome);
        }
        self.message = None;
        if self.tab == Tab::Overview {
            if let Some(effect) = self.overview_key(key) {
                return effect;
            }
        }
        if let Some(cmd) = key_cmd(self.tab, &key) {
            // `N` throws the whole ✦ draft away when there is one.
            let cmd = if cmd == ReviewCmd::DismissAllSuggestions && self.has_draft() {
                ReviewCmd::DiscardDraft
            } else {
                cmd
            };
            return self.run(cmd);
        }
        self.navigate(key)
    }

    fn on_bar_menu(&mut self, outcome: super::barmenu::BarMenuOutcome) -> ReviewEffect {
        use super::barmenu::BarMenuOutcome as O;
        match outcome {
            O::Stay => {}
            O::Closed => self.bar_menu = None,
            O::Encoding(pick) => {
                self.bar_menu = None;
                self.pick_encoding(pick);
            }
            O::Button(button) => {
                self.bar_menu = None;
                return self.run_bar_button(button);
            }
        }
        ReviewEffect::None
    }

    fn on_finish(&mut self, outcome: FinishOutcome) -> ReviewEffect {
        match outcome {
            FinishOutcome::Stay => {}
            FinishOutcome::Cancel => self.finish = None,
            FinishOutcome::Submit { event, body, votes } => self.submit_review(event, body, votes),
            FinishOutcome::Discard => {
                return self.run(ReviewCmd::DiscardReview);
            }
        }
        ReviewEffect::None
    }

    fn on_confirm(&mut self, outcome: ConfirmOutcome) -> ReviewEffect {
        match outcome {
            ConfirmOutcome::Stay => {}
            ConfirmOutcome::No => self.confirm = None,
            ConfirmOutcome::Yes => match self.confirm.take() {
                Some((ConfirmFor::DiscardReview, _)) => {
                    self.pending.clear();
                    self.revision += 1;
                    self.save_draft();
                    self.finish = None;
                    self.message = Some(msg::DISCARDED.get().to_string());
                }
                Some((ConfirmFor::SubmitChange, _)) => {
                    self.start_call(Call::SubmitChange, false, None);
                }
                Some((ConfirmFor::AbandonChange, _)) => {
                    self.start_call(Call::AbandonChange, false, None);
                }
                None => {}
            },
        }
        ReviewEffect::None
    }

    /// The page's own movement keys.
    fn navigate(&mut self, key: KeyEvent) -> ReviewEffect {
        let shift = key.modifiers.contains(KeyModifiers::SHIFT);
        if key.code == KeyCode::Esc {
            // A suggestion being made is stopped first; the view stays.
            if self.cancel_suggest() {
                return ReviewEffect::None;
            }
            let tab = self.tab;
            let returning = tab == Tab::Files && self.ov_return.is_some();
            let pane = self.active_pane();
            if tab != Tab::Overview && pane.anchor.is_some() {
                pane.anchor = None;
            } else if returning {
                self.set_tab(Tab::Overview);
            } else {
                return ReviewEffect::Close;
            }
            return ReviewEffect::None;
        }
        match self.tab {
            // The conversation has the keys (`overview_key`).
            Tab::Overview => {}
            Tab::Files | Tab::Commits => {
                let tab = self.tab;
                match (self.focus, key.code) {
                    (Focus::List, KeyCode::Up) => self.step_list(-1),
                    (Focus::List, KeyCode::Down) => self.step_list(1),
                    (Focus::List, KeyCode::PageUp) => self.step_list(-(self.list_rows as isize)),
                    (Focus::List, KeyCode::PageDown) => self.step_list(self.list_rows as isize),
                    (Focus::List, KeyCode::Home) => self.step_list(isize::MIN / 2),
                    (Focus::List, KeyCode::End) => self.step_list(isize::MAX / 2),
                    (Focus::List, KeyCode::Char(' ')) if tab == Tab::Files => {
                        return self.run(ReviewCmd::ToggleViewed);
                    }
                    (Focus::List, KeyCode::Enter | KeyCode::Right | KeyCode::Tab) => {
                        self.focus = Focus::Diff;
                    }
                    // Sideways first, back to the list once at the left edge.
                    (Focus::Diff, KeyCode::Left) if self.diff_opts.h_offset > 0 => {
                        self.scroll_h(-H_STEP)
                    }
                    (Focus::Diff, KeyCode::Right) => self.scroll_h(H_STEP),
                    (Focus::Diff, KeyCode::Left | KeyCode::BackTab) => self.focus = Focus::List,
                    (Focus::Diff, KeyCode::Tab) => self.focus = Focus::List,
                    (Focus::Diff, KeyCode::Up) => self.active_pane().move_cursor(-1, shift),
                    (Focus::Diff, KeyCode::Down) => self.active_pane().move_cursor(1, shift),
                    (Focus::Diff, KeyCode::PageUp) => {
                        let n = self.active_pane().visible() as isize;
                        self.active_pane().move_cursor(-n, shift);
                    }
                    (Focus::Diff, KeyCode::PageDown) => {
                        let n = self.active_pane().visible() as isize;
                        self.active_pane().move_cursor(n, shift);
                    }
                    (Focus::Diff, KeyCode::Home) => self.active_pane().jump(0),
                    (Focus::Diff, KeyCode::End) => {
                        let last = self.active_pane().len();
                        self.active_pane().jump(last);
                    }
                    (Focus::Diff, KeyCode::Enter) if tab == Tab::Files => {
                        return self.run(ReviewCmd::ToggleThread);
                    }
                    _ => {}
                }
                // A selection that grows past one line needs a host that
                // takes ranges.
                if shift && !self.can_range() && self.focus == Focus::Diff {
                    self.active_pane().anchor = None;
                    self.message = Some(msg::NO_RANGES.get().to_string());
                }
            }
        }
        ReviewEffect::None
    }

    fn step_list(&mut self, delta: isize) {
        match self.tab {
            Tab::Files => {
                let n = self.file_count();
                if n > 0 {
                    let at = (self.file_sel as isize)
                        .saturating_add(delta)
                        .clamp(0, (n as isize - 1).max(0));
                    self.select_file(at as usize);
                }
            }
            Tab::Commits => {
                let at = (self.commit_sel as isize).saturating_add(delta).max(0) as usize;
                self.select_commit(at);
            }
            Tab::Overview => {}
        }
    }

    fn scroll_overview(&mut self, delta: isize) {
        let max = self.ov_len.saturating_sub(self.ov_rows);
        self.ov_offset = (self.ov_offset as isize)
            .saturating_add(delta)
            .clamp(0, max as isize) as usize;
    }

    // -- mouse --------------------------------------------------------------------------

    pub(crate) fn on_mouse(&mut self, mouse: MouseEvent) -> ReviewEffect {
        // Over a popup nothing behind it lights up.
        if !self.takes_palette() {
            self.hover = None;
        }
        if self.ps_mouse(mouse) {
            return ReviewEffect::None;
        }
        if let Some(menu) = self.bar_menu.as_mut() {
            let outcome = menu.on_mouse(mouse);
            return self.on_bar_menu(outcome);
        }
        if let Some(meta) = self.meta.as_mut() {
            let outcome = meta.on_mouse(mouse);
            return self.on_meta(outcome).unwrap_or(ReviewEffect::None);
        }
        if self.compose.is_some() {
            if let Some(compose) = self.compose.as_mut() {
                match compose.on_mouse(mouse) {
                    ComposeOutcome::Stay => {}
                    ComposeOutcome::Cancel => self.compose = None,
                    ComposeOutcome::Submit(text) => {
                        if let Some(c) = self.compose.take() {
                            self.finish_compose(c.purpose, text);
                        }
                    }
                }
            }
            return ReviewEffect::None;
        }
        if self.confirm.is_some() {
            let outcome = self.confirm.as_mut().map(|(_, c)| c.on_mouse(mouse));
            return self.on_confirm(outcome.unwrap_or(ConfirmOutcome::Stay));
        }
        if let Some(finish) = self.finish.as_mut() {
            let outcome = finish.on_mouse(mouse);
            return self.on_finish(outcome);
        }
        if let Some(options) = self.options.as_mut() {
            let outcome = options.on_mouse(mouse);
            return self.on_options(outcome);
        }
        if self.bars_consume(mouse) {
            return ReviewEffect::None;
        }
        if self.tab == Tab::Overview {
            if let Some(effect) = self.overview_mouse(mouse) {
                return effect;
            }
        }
        let hit = self
            .hits
            .iter()
            .find(|(r, _)| r.contains((mouse.column, mouse.row).into()))
            .map(|(_, h)| *h);
        match mouse.kind {
            MouseEventKind::Moved => {
                self.hover = hit;
            }
            MouseEventKind::ScrollLeft => self.wheel_h(mouse, -H_STEP),
            MouseEventKind::ScrollRight => self.wheel_h(mouse, H_STEP),
            MouseEventKind::ScrollDown if mouse.modifiers.contains(KeyModifiers::SHIFT) => {
                self.wheel_h(mouse, H_STEP)
            }
            MouseEventKind::ScrollUp if mouse.modifiers.contains(KeyModifiers::SHIFT) => {
                self.wheel_h(mouse, -H_STEP)
            }
            MouseEventKind::ScrollUp => self.wheel(mouse, -3),
            MouseEventKind::ScrollDown => self.wheel(mouse, 3),
            MouseEventKind::Down(MouseButton::Left) => {
                if let Some(hit) = hit {
                    self.message = None;
                    return self.click(hit, mouse);
                }
            }
            MouseEventKind::Drag(MouseButton::Left) => {
                if self.drag_text(mouse) {
                    return ReviewEffect::None;
                }
                if let Some(Hit::Row(id, row)) = hit {
                    self.drag_to(id, row);
                }
            }
            MouseEventKind::Up(MouseButton::Left) => {
                for pane in [&mut self.files_pane, &mut self.commits_pane] {
                    pane.press = None;
                    pane.text_drag = None;
                }
            }
            _ => {}
        }
        ReviewEffect::None
    }

    /// The scrollbars' thumb drag, track jump and end arrows.
    fn bars_consume(&mut self, mouse: MouseEvent) -> bool {
        let ov = (self.ov_bar, self.ov_len, self.ov_offset);
        if let Some(top) = self.ov_drag.on_mouse(ov.0, mouse, ov.1, ov.2) {
            self.ov_offset = top.min(self.ov_len.saturating_sub(self.ov_rows));
            return true;
        }
        let n = self.file_count();
        if let Some(top) = self
            .file_drag
            .on_mouse(self.file_bar, mouse, n, self.file_offset)
        {
            self.file_offset = top.min(n.saturating_sub(self.list_rows));
            return true;
        }
        let c = self.git.as_ref().map_or(0, |g| g.commits.len());
        if let Some(top) = self
            .commit_drag
            .on_mouse(self.commit_bar, mouse, c, self.commit_offset)
        {
            self.commit_offset = top.min(c.saturating_sub(self.list_rows));
            return true;
        }
        if self.h_bar_consume(mouse) || self.minimap_consume(mouse) {
            return true;
        }
        for id in [PaneId::Files, PaneId::Commits] {
            let pane = self.pane_of(id);
            let (bar, len, top) = (pane.bar, pane.len(), pane.offset);
            if let Some(new) = pane.drag.on_mouse(bar, mouse, len, top) {
                let max = len.saturating_sub(pane.visible());
                pane.offset = new.min(max);
                return true;
            }
        }
        false
    }

    /// The horizontal scrollbar's end arrows, thumb and track.
    fn h_bar_consume(&mut self, mouse: MouseEvent) -> bool {
        use crate::scrollbar::TrackPress;
        let tab_pane = match self.tab {
            Tab::Commits => PaneId::Commits,
            _ => PaneId::Files,
        };
        let pane = self.pane_of(tab_pane);
        let Some(track) = pane.h_bar else {
            pane.h_grab = None;
            return false;
        };
        let (content, max) = (pane.h_content_len(), pane.h_max);
        let pos = i32::from(mouse.column) - i32::from(track.rect.x);
        let width = track.rect.width as usize;
        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left)
                if track.rect.contains((mouse.column, mouse.row).into()) =>
            {
                match track.press(pos as u16) {
                    TrackPress::Arrow(step) => self.scroll_h(step * H_STEP),
                    TrackPress::Grab { grab, jump } => {
                        self.pane_of(tab_pane).h_grab = Some(grab);
                        if jump {
                            let to = track.drag_offset(content, width, grab, pos).min(max);
                            self.set_h_offset(to);
                        }
                    }
                }
                true
            }
            MouseEventKind::Drag(MouseButton::Left) if pane.h_grab.is_some() => {
                let grab = pane.h_grab.unwrap_or(0);
                let to = track.drag_offset(content, width, grab, pos).min(max);
                self.set_h_offset(to);
                true
            }
            MouseEventKind::Up(MouseButton::Left) => {
                pane.h_grab = None;
                false
            }
            _ => false,
        }
    }

    /// A press or drag on the minimap puts the line under it mid-view.
    fn minimap_consume(&mut self, mouse: MouseEvent) -> bool {
        let tab_pane = match self.tab {
            Tab::Commits => PaneId::Commits,
            _ => PaneId::Files,
        };
        let pane = self.pane_of(tab_pane);
        let Some(rect) = pane.map else {
            pane.map_drag = false;
            return false;
        };
        let over = rect.contains((mouse.column, mouse.row).into());
        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) if over => pane.map_drag = true,
            MouseEventKind::Drag(MouseButton::Left) if pane.map_drag => {}
            MouseEventKind::Up(MouseButton::Left) => {
                pane.map_drag = false;
                return false;
            }
            _ => return false,
        }
        let (total, view) = (pane.len(), pane.visible());
        let line = crate::minimap::line_at(rect, total, mouse.row);
        pane.offset = crate::minimap::top_for(line, total, view);
        true
    }

    /// A sideways wheel notch over the diff (or any of it): the text scrolls.
    fn wheel_h(&mut self, mouse: MouseEvent, delta: isize) {
        if matches!(self.tab, Tab::Files | Tab::Commits)
            && self.pane().area.contains((mouse.column, mouse.row).into())
        {
            self.scroll_h(delta);
        }
    }

    /// A drag that began in the code sweeps a text selection on the side it
    /// began on, row by row, clamped into the drawn rows (the Diff view's).
    fn drag_text(&mut self, mouse: MouseEvent) -> bool {
        for id in [PaneId::Files, PaneId::Commits] {
            let pane = self.pane_of(id);
            let (Some(drag), Some(render)) = (pane.text_drag, pane.render.clone()) else {
                continue;
            };
            if let Some(head) = crate::diff_view::text_pos_at(
                &render,
                pane.area,
                pane.offset,
                drag.side,
                mouse.column,
                mouse.row,
            ) {
                pane.text_sel = Some(drag.to(head));
            }
            return true;
        }
        false
    }

    fn wheel(&mut self, mouse: MouseEvent, delta: isize) {
        let at = (mouse.column, mouse.row).into();
        match self.tab {
            Tab::Overview => self.scroll_overview(delta),
            Tab::Files | Tab::Commits => {
                if self.pane().area.contains(at) {
                    self.active_pane().scroll(delta);
                } else if self.tab == Tab::Files {
                    let max = self.file_count().saturating_sub(self.list_rows);
                    self.file_offset =
                        (self.file_offset as isize + delta).clamp(0, max as isize) as usize;
                } else {
                    let n = self.git.as_ref().map_or(0, |g| g.commits.len());
                    let max = n.saturating_sub(self.list_rows);
                    self.commit_offset =
                        (self.commit_offset as isize + delta).clamp(0, max as isize) as usize;
                }
            }
        }
    }

    fn drag_to(&mut self, id: PaneId, row: usize) {
        let ranges = self.can_range();
        let pane = self.pane_of(id);
        if let Some(start) = pane.press {
            if ranges && row != start {
                pane.anchor = Some(start);
                pane.cursor = row;
            }
        }
    }

    fn click(&mut self, hit: Hit, mouse: MouseEvent) -> ReviewEffect {
        let shift = mouse.modifiers.contains(KeyModifiers::SHIFT);
        match hit {
            Hit::Close => return ReviewEffect::Close,
            Hit::Bar(button) => match button {
                DiffBarButton::Mode(mode) => {
                    if self.diff_opts.mode != mode {
                        self.apply_toggle(DiffViewToggle::Mode(mode));
                    }
                }
                DiffBarButton::ToggleHunksOnly => return self.run(ReviewCmd::ToggleHunksOnly),
                DiffBarButton::ToggleWhitespace => return self.run(ReviewCmd::ToggleWhitespace),
                DiffBarButton::ToggleWordWrap => return self.run(ReviewCmd::ToggleWrap),
                DiffBarButton::NextHunk => return self.run(ReviewCmd::NextHunk),
                DiffBarButton::PrevHunk => return self.run(ReviewCmd::PrevHunk),
                DiffBarButton::Encoding => return self.run(ReviewCmd::ChooseEncoding),
                DiffBarButton::ToggleMinimap => return self.run(ReviewCmd::ToggleMinimap),
                _ => {}
            },
            Hit::BarMore => {
                if let Some((rect, hidden)) = self.bar_slots.more.clone() {
                    let state = self.diff_opts.clone();
                    self.bar_menu = Some(super::barmenu::BarMenu::overflow(hidden, &state, rect));
                }
            }
            Hit::Tab(tab) => self.set_tab(tab),
            Hit::BackToComment => return self.run(ReviewCmd::BackToComment),
            Hit::Meta(field) => self.open_meta(field),
            Hit::Checkout => return self.run(ReviewCmd::Checkout),
            Hit::Browser => return self.run(ReviewCmd::OpenBrowser),
            Hit::Refresh => return self.run(ReviewCmd::Refresh),
            Hit::Finish => return self.run(ReviewCmd::FinishReview),
            Hit::SubmitChange => return self.run(ReviewCmd::SubmitChange),
            Hit::AbandonChange => return self.run(ReviewCmd::AbandonChange),
            Hit::MdLink(i) => {
                if let Some(url) = self.md_links.get(i).cloned() {
                    crate::link_confirm::ask(vec![url]);
                }
            }
            Hit::File(i) => {
                self.select_file(i);
                self.focus = Focus::List;
            }
            Hit::Viewed(i) => {
                let path = self
                    .host
                    .as_ref()
                    .and_then(|h| h.files.get(i))
                    .map(|f| f.path.clone());
                if let Some(path) = path {
                    self.toggle_viewed(&path);
                }
            }
            Hit::Commit(i) => {
                self.select_commit(i);
                self.focus = Focus::List;
            }
            Hit::Row(id, row) => {
                let tab_matches = matches!(
                    (id, self.tab),
                    (PaneId::Files, Tab::Files) | (PaneId::Commits, Tab::Commits)
                );
                if tab_matches {
                    self.focus = Focus::Diff;
                }
                let ranges = self.can_range();
                let double = self.double_click(mouse);
                let pane = self.pane_of(id);
                // In Split, the half that was clicked is the side a comment goes on.
                let mut in_text = false;
                if let Some(render) = pane.render.clone() {
                    let col = mouse.column.saturating_sub(pane.area.x);
                    pane.side = crate::text_select::side_at(render.split_x, col);
                    // Where a press lands decides what a drag does: in the code
                    // it selects text (to copy), in the gutter (the numbers, the
                    // marker, the `+`) it selects lines (to comment on).
                    in_text = render
                        .segs
                        .get(row)
                        .and_then(|s| s[pane.side])
                        .is_some_and(|seg| col >= seg.x);
                    if in_text && !(shift && ranges) {
                        let side = pane.side;
                        let pos = crate::diff_view::text_pos_at(
                            &render,
                            pane.area,
                            pane.offset,
                            side,
                            mouse.column,
                            mouse.row,
                        );
                        pane.anchor = None;
                        pane.cursor = row;
                        pane.press = None;
                        pane.text_drag = crate::text_select::press(
                            crate::text_select::TextView::Diff,
                            side,
                            pos,
                            double,
                            &render.text.side(side),
                            &mut pane.text_sel,
                        );
                        return ReviewEffect::None;
                    }
                }
                pane.text_sel = None;
                pane.text_drag = None;
                if shift && ranges {
                    pane.anchor.get_or_insert(pane.cursor);
                    pane.cursor = row;
                } else {
                    pane.anchor = None;
                    pane.cursor = row;
                    pane.press = Some(row);
                }
                if double && id == PaneId::Files && !in_text {
                    return self.run(ReviewCmd::AddComment);
                }
            }
            Hit::Add(id, row) => {
                let pane = self.pane_of(id);
                pane.anchor = None;
                pane.cursor = row;
                return self.run(ReviewCmd::AddComment);
            }
            Hit::Fold(row) => {
                self.files_pane.cursor = row;
                return self.run(ReviewCmd::ToggleThread);
            }
            Hit::ThreadReply(row) => {
                self.files_pane.cursor = row;
                return self.run(ReviewCmd::Reply);
            }
            Hit::ThreadResolve(row) => {
                self.files_pane.cursor = row;
                return self.run(ReviewCmd::ToggleResolve);
            }
            Hit::PendingEdit(row) => {
                self.files_pane.cursor = row;
                return self.run(ReviewCmd::EditPending);
            }
            Hit::PendingDelete(row) => {
                self.files_pane.cursor = row;
                return self.run(ReviewCmd::DeletePending);
            }
            Hit::Suggest => {
                // Running: the same button cancels it.
                if self.cancel_suggest() {
                    return ReviewEffect::None;
                }
                return self.run(ReviewCmd::SuggestReview);
            }
            Hit::GhostAccept(row) => {
                self.files_pane.cursor = row;
                return self.run(ReviewCmd::AcceptSuggestion);
            }
            Hit::GhostEdit(row) => {
                self.files_pane.cursor = row;
                return self.run(ReviewCmd::EditSuggestion);
            }
            Hit::GhostDismiss(row) => {
                self.files_pane.cursor = row;
                return self.run(ReviewCmd::DismissSuggestion);
            }
            Hit::SugAccept(i) => self.accept_suggestion(i, None),
            Hit::SugEdit(i) => self.edit_suggestion(i),
            Hit::SugDismiss(i) => self.dismiss_suggestion(i),
            Hit::UseDraft => return self.run(ReviewCmd::UseDraft),
            Hit::EditDraft => return self.run(ReviewCmd::EditDraftText),
            Hit::DiscardDraft => return self.run(ReviewCmd::DiscardDraft),
            Hit::SugAcceptAll => return self.run(ReviewCmd::AcceptAllSuggestions),
            Hit::SugDismissAll => return self.run(ReviewCmd::DismissAllSuggestions),
        }
        ReviewEffect::None
    }

    fn double_click(&mut self, mouse: MouseEvent) -> bool {
        let now = std::time::Instant::now();
        let double = self.last_click.is_some_and(|(t, c, r)| {
            now.duration_since(t) < crate::pane_layout::DOUBLE_CLICK_WINDOW
                && c == mouse.column
                && r == mouse.row
        });
        self.last_click = (!double).then_some((now, mouse.column, mouse.row));
        double
    }
}
