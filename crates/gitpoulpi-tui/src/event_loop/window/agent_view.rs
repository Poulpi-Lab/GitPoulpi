//! An external agent's view intents, applied to this window (plan
//! the mcp-show-ui design). [`UiViewer`] queues what
//! the link delivers; once per tick [`WindowState::agent_view_tick`] decides:
//!
//! - the user's own choices first (Back, the chip's Show);
//! - what the previous window carried across a tab switch;
//! - then what arrived: the newest wins (the earlier ones are `superseded`),
//!   `ask` mode and a busy user (a modal, a text field, a key or click a
//!   moment ago, a drag) defer it to the footer chip, a dwell of 400 ms
//!   spaces two applied intents, and the rest is applied.
//!
//! Applying is view-only by construction: it opens or moves a view (a tab,
//! Home, Settings, the palette with a query it never submits, Help) through
//! the same state a click or a command changes, and pushes the view it
//! replaced on the Back stack first. See [`crate::ui_intent`] for what
//! survives a window restart.
//!
//! [`UiViewer`]: crate::ui_viewer::UiViewer

use crate::event_loop::*;
use crate::ui_intent::{self, ViewSnapshot};
use crate::ui_viewer::{HighlightReveal, Reveal, ShowReveal, ViewReveal};
use gitpoulpi_core::domain::UiControl;
use gitpoulpi_ipc::ui::{sanitize_with_limit, DeferReason, UiIntent, UiOrigin, UiOutcome};
use init::WindowState;

/// How long a carried `ShowRef` waits for the new window's refs to load.
const CARRIED_REFS_GRACE: std::time::Duration = std::time::Duration::from_secs(5);

/// Runs `f` with English as the language: what an agent reads (a `ui` intent's
/// reason or note) is English whatever the interface language is.
pub(super) fn en<R>(f: impl FnOnce() -> R) -> R {
    gitpoulpi_i18n::with_lang(gitpoulpi_i18n::Lang::En, f)
}

/// The result of applying one intent to this window.
pub(super) enum Applied {
    /// The view moved; the answer goes out after the next drawn frame.
    Done,
    /// The view moved, but not all of what was asked: `Partial` with this
    /// note.
    Partial(String),
    /// The view starts moving once an asynchronous step lands (the Log
    /// paging in a commit); the tick answers it.
    Waiting(Box<ShowReveal>),
    /// Like `Waiting`, for a diff or file on its way (several steps).
    Viewing(Box<ViewReveal>),
    /// A highlight widened the diff to the whole file and waits for its rows.
    Widening(Box<HighlightReveal>),
    /// The window is ending (a tab or Home switch): the next one applies the
    /// carried intent and answers it.
    Restarting,
    /// A proposal's dialog opened pre-filled (its id): answered `Proposed`
    /// after the next drawn frame.
    Proposed(String),
    /// Nothing moved.
    Not(Box<UiOutcome>),
}

fn focus_name(focus: Focus) -> &'static str {
    match focus {
        Focus::Refs => "refs",
        Focus::Log => "log",
        Focus::Status => "status",
    }
}

impl WindowState {
    /// Puts a merge editor in the central column. S6 calls this from the
    /// Status pane / palette entry points; until then nothing does, and the
    /// tests draw `MergeView` directly.
    #[allow(dead_code)]
    pub(in crate::event_loop) fn open_merge_view(&mut self, view: crate::merge_view::MergeView) {
        self.merge = Some(view);
        self.central_view = CentralView::Merge;
    }

    /// Whether a modal, a text field or a form owns the keyboard (the
    /// intents that arrive now are deferred, and `ui state` says so).
    pub(in crate::event_loop) fn agent_input_busy(&self) -> bool {
        self.preferences.is_some()
            || self.show_help
            || self.editor.is_some()
            || self.merge.is_some()
            || !matches!(self.input_mode, InputMode::Normal)
            || self.repo_dialogs.is_open()
            || self.remote_ui.is_open()
            || self.ai_ui.popup_open()
            || self.composer_focus.is_some()
            || self.commit_filter_focus
            || self.management.as_ref().is_some_and(|m| m.search_focused)
    }

    /// Whether the user is typing or answering a form - unlike
    /// [`Self::agent_input_busy`], not merely looking at Settings, Help or the
    /// palette, which is what Back exists to close.
    fn agent_typing(&self) -> bool {
        self.editor.is_some()
            || self.merge.is_some()
            || self.composer_focus.is_some()
            || self.commit_filter_focus
            || self.repo_dialogs.is_open()
            || self.remote_ui.is_open()
            || self.ai_ui.popup_open()
            || self.preferences.as_ref().is_some_and(|p| p.typing())
            || !matches!(
                self.input_mode,
                InputMode::Normal | InputMode::CommandPalette { .. }
            )
            || self.management.as_ref().is_some_and(|m| m.search_focused)
    }

    fn agent_dragging(&self) -> bool {
        self.dragging_commit_message
            || self.dragging_sidebar_section.is_some()
            || self.dragging_divider.is_some()
            || self.dragging_log_boundary.is_some()
            || self.dragging_scrollbar.is_some()
            || self.dragging_log_header.is_some()
            || self.dragging_list_bar.is_some()
            || self.diff_line_drag.is_some()
            || self.text_drag.is_some()
            || self.ref_drag.is_some()
    }

    /// Why an intent from `origin` must wait for the user instead of moving
    /// the view now (plan section 4.3).
    fn agent_defer_reason(&self, intent: &UiIntent, origin: &UiOrigin) -> Option<DeferReason> {
        if crate::ui_viewer::is_local(origin) {
            // The user's own command (`gp -a`): never "ask", and only a form
            // being filled in holds it back (the footer chip then offers it).
            self.agent_typing().then_some(DeferReason::InputBusy)
        } else if origin.control == UiControl::Ask {
            Some(DeferReason::Ask)
        } else if matches!(intent, UiIntent::Back) {
            // Undoing the agent's own change: only typing holds it back.
            self.agent_typing().then_some(DeferReason::InputBusy)
        } else if self.agent_input_busy() {
            Some(DeferReason::InputBusy)
        } else if self.agent_dragging() {
            Some(DeferReason::Dragging)
        } else if self.ui_viewer.recent_input() {
            Some(DeferReason::RecentInput)
        } else {
            None
        }
    }

    /// Once per tick: the user's choices, the carried intent, the queue.
    pub(in crate::event_loop) fn agent_view_tick(&mut self) {
        // A proposal's dialog closing is settled whatever the link does.
        self.agent_propose_tick();
        // Answers are only accepted from the connection that attached.
        if !self.ui_viewer.is_attached(self.link.ui()) {
            return;
        }
        self.agent_reveal_tick();
        self.agent_tint_tick();
        if let Some(snapshot) = ui_intent::take_restore() {
            self.restore_view(&snapshot);
        }
        if let Some(carried) = ui_intent::take_carried() {
            // A new window ticks before its refs have loaded (quickly on a
            // fast machine, a good while later on a slow one), and a ref
            // that cannot be looked up yet is not "no such ref": hold the
            // intent a moment, then answer with whatever is there.
            if self.refs_result.is_err()
                && matches!(carried.intent, UiIntent::ShowRef { .. })
                && carried.age() < CARRIED_REFS_GRACE
            {
                ui_intent::hold_carried(carried);
            } else {
                self.run_agent_intent(carried.id, carried.intent, carried.origin, false);
            }
        }
        if self.ui_viewer.take_back_request() {
            self.run_agent_intent(None, UiIntent::Back, UiOrigin::cli(), true);
        }
        if let Some((intent, origin)) = self.ui_viewer.take_accepted() {
            self.run_agent_intent(None, intent, origin, true);
        }
        if let Some(intent) = self.ui_viewer.take_go() {
            self.run_agent_intent(None, intent, crate::ui_viewer::local_origin(), true);
        }

        let mut queued = self.ui_viewer.take_queue();
        let Some(newest) = queued.pop() else { return };
        // Waited too long (the dwell, a slow tick): the agent has given up
        // and the screen it meant has moved on.
        if newest.at.elapsed() > crate::ui_viewer::stale_after() {
            self.ui_viewer
                .report_now(self.link.ui(), newest.id, UiOutcome::refused("stale"));
            for older in queued {
                self.ui_viewer.report_now(
                    self.link.ui(),
                    older.id,
                    UiOutcome::refused("superseded"),
                );
            }
            return;
        }
        for older in queued {
            self.ui_viewer
                .report_now(self.link.ui(), older.id, UiOutcome::refused("superseded"));
        }
        if let Some(reason) = self.agent_defer_reason(&newest.intent, &newest.origin) {
            self.ui_viewer.offer(newest.intent, newest.origin);
            self.ui_viewer
                .report_now(self.link.ui(), newest.id, UiOutcome::Deferred { reason });
            return;
        }
        if self.ui_viewer.dwell_left().is_some() {
            // Too soon after the last one: wait for the deadline.
            self.ui_viewer.hold(newest);
            return;
        }
        self.run_agent_intent(Some(newest.id), newest.intent, newest.origin, true);
    }

    /// Applies one intent and answers it. `snapshot`: whether the view it
    /// replaces goes on the Back stack (not when a restarted window finishes
    /// what the previous one started).
    fn run_agent_intent(
        &mut self,
        id: Option<u64>,
        intent: UiIntent,
        origin: UiOrigin,
        snapshot: bool,
    ) {
        self.supersede_reveal();
        // A proposal opens a dialog over the view: nothing for Back to undo.
        let pushed = snapshot
            && !matches!(intent, UiIntent::Back | UiIntent::Propose { .. })
            && self.agent_moves_view(&intent);
        if pushed {
            ui_intent::push(self.capture_view());
        }
        let result = if matches!(intent, UiIntent::Back) && !snapshot {
            // The previous window popped the snapshot and handed it over
            // (restored on arrival); this finishes the answer.
            Applied::Done
        } else {
            self.apply_agent_intent(id, &intent, &origin)
        };
        self.conclude_agent_intent(id, &origin, &ui_intent::describe(&intent), pushed, result);
    }

    /// Reports how applying an intent went: the toast, the answer, and the
    /// Back snapshot dropped when nothing moved. `what` is the toast's name
    /// for it.
    pub(super) fn conclude_agent_intent(
        &mut self,
        id: Option<u64>,
        origin: &UiOrigin,
        what: &str,
        pushed: bool,
        result: Applied,
    ) {
        match result {
            Applied::Done => {
                self.ui_viewer.note_applied();
                self.agent_changed_view(origin);
                self.ui_viewer.shown(origin, what);
                if let Some(id) = id {
                    self.ui_viewer.report_after_frame(id);
                }
            }
            Applied::Partial(note) => {
                self.ui_viewer.note_applied();
                self.agent_changed_view(origin);
                self.ui_viewer.shown(origin, what);
                match id {
                    Some(id) => self.ui_viewer.report_partial_after_frame(id, note),
                    None => self
                        .ui_viewer
                        .say(gitpoulpi_i18n::msg::ui_intent::agent_shown_but(what, &note)),
                }
            }
            Applied::Waiting(mut show) => {
                show.pushed = pushed;
                self.ui_viewer.set_reveal(Some(Reveal::Show(show)));
            }
            Applied::Viewing(mut view) => {
                view.pushed = pushed;
                self.ui_viewer.set_reveal(Some(Reveal::View(view)));
            }
            Applied::Widening(mut reveal) => {
                reveal.pushed = pushed;
                // The cursor moves when the whole-file rows land: the band
                // is watched again once they have.
                self.ui_viewer.note_tint(None);
                self.ui_viewer.set_reveal(Some(Reveal::Highlight(reveal)));
            }
            Applied::Restarting => {}
            Applied::Proposed(proposal) => {
                // The dialog's own "Proposed by" banner says who; no toast.
                self.ui_viewer.note_applied();
                if self.management.is_none() {
                    ui_intent::mark_tab(&self.repo_root);
                }
                if let Some(id) = id {
                    self.ui_viewer.report_proposed_after_frame(id, proposal);
                }
            }
            Applied::Not(outcome) => {
                if pushed {
                    ui_intent::pop();
                }
                match id {
                    Some(id) => self.ui_viewer.report_now(self.link.ui(), id, *outcome),
                    None => {
                        self.ui_viewer
                            .say(gitpoulpi_i18n::msg::ui_intent::agent_could_not_show(
                                what,
                                match &*outcome {
                                    UiOutcome::Refused { reason } => reason.clone(),
                                    UiOutcome::NotFound { what } => what.clone(),
                                    _ => gitpoulpi_i18n::msg::ui_intent::NOT_POSSIBLE
                                        .get()
                                        .to_string(),
                                },
                            ))
                    }
                }
            }
        }
    }

    /// An intent moved this window's view: mark its tab with a ✦ (until the
    /// user's next key or click) and remember where the highlight band sits.
    fn agent_changed_view(&mut self, origin: &UiOrigin) {
        if self.management.is_none() && !crate::ui_viewer::is_local(origin) {
            ui_intent::mark_tab(&self.repo_root);
        }
        let cursor = self.tint_cursor_now();
        self.ui_viewer.note_tint(cursor);
    }

    /// The cursor of the open diff / file while it carries a highlight band.
    fn tint_cursor_now(&self) -> Option<usize> {
        match self.central_view {
            CentralView::Diff => self
                .diff_view_state
                .as_ref()
                .filter(|s| s.highlight.is_some())
                .map(|s| s.cursor),
            CentralView::File => self
                .file_insight
                .as_ref()
                .filter(|i| i.highlight.is_some())
                .map(|i| i.selected),
            _ => None,
        }
    }

    /// An agent's highlight band goes when the user moves the cursor out of
    /// it, or presses Esc (documented in `docs/user/mcp-and-agents.md`).
    fn agent_tint_tick(&mut self) {
        let esc = self.ui_viewer.take_esc();
        let Some(placed) = self.ui_viewer.tint_cursor() else {
            return;
        };
        if esc || self.tint_cursor_now() != Some(placed) {
            if let Some(state) = self.diff_view_state.as_mut() {
                state.highlight = None;
            }
            if let Some(insight) = self.file_insight.as_mut() {
                insight.highlight = None;
            }
            self.ui_viewer.note_tint(None);
            crate::redraw::poke();
        }
    }

    /// Whether applying `intent` to this window changes what is shown (a
    /// tab switch to the tab already open does not, so Back has nothing to
    /// undo).
    fn agent_moves_view(&self, intent: &UiIntent) -> bool {
        match intent {
            UiIntent::ShowTab { repo } => {
                self.management.is_some()
                    || gitpoulpi_core::domain::RepoId::from_path(std::path::Path::new(&repo.0))
                        != self.repo.id
            }
            _ => true,
        }
    }

    /// Switches to the tab of `path` (opening it); the window ends, and the
    /// next one applies `intent` (carried) and answers it.
    fn agent_open_tab(
        &mut self,
        id: Option<u64>,
        intent: &UiIntent,
        origin: &UiOrigin,
        path: std::path::PathBuf,
    ) -> Applied {
        let message = self
            .tab_ctx()
            .switcher_action(repo_switcher::SwitcherAction::Open(path));
        if self.switch_to.is_some() {
            ui_intent::carry(id, intent.clone(), origin.clone());
            return Applied::Restarting;
        }
        match message {
            // "already on {path}": the tab switcher's own sentence, in the
            // language it was worded in.
            Some(what)
                if what.starts_with(gitpoulpi_i18n::msg::tabs::already_on("").trim_end()) =>
            {
                Applied::Done
            }
            Some(what) => Applied::Not(Box::new(UiOutcome::NotFound { what })),
            None => Applied::Done,
        }
    }

    fn apply_agent_intent(
        &mut self,
        id: Option<u64>,
        intent: &UiIntent,
        origin: &UiOrigin,
    ) -> Applied {
        match intent {
            UiIntent::ShowTab { repo } => {
                if !self.agent_moves_view(intent) {
                    return Applied::Done;
                }
                self.agent_open_tab(id, intent, origin, std::path::PathBuf::from(&repo.0))
            }
            UiIntent::ShowCommit { repo, .. }
            | UiIntent::ShowRef { repo, .. }
            | UiIntent::ShowSearch { repo, .. }
            | UiIntent::ShowHosting { repo, .. }
            | UiIntent::ShowWip { repo, .. }
            | UiIntent::ShowDiff { repo, .. }
            | UiIntent::ShowFile { repo, .. }
            | UiIntent::ShowConflict { repo, .. }
            | UiIntent::Propose { repo, .. } => {
                // Another tab (or Home) first: the next window applies it.
                if self.agent_moves_view(&UiIntent::ShowTab { repo: repo.clone() }) {
                    return self.agent_open_tab(
                        id,
                        intent,
                        origin,
                        std::path::PathBuf::from(&repo.0),
                    );
                }
                match intent {
                    UiIntent::ShowDiff { .. } | UiIntent::ShowFile { .. } => {
                        self.apply_look(id, intent, origin)
                    }
                    UiIntent::ShowConflict { path, conflict, .. } => {
                        self.apply_show_conflict(path, *conflict)
                    }
                    UiIntent::Propose {
                        id: proposal_id,
                        proposal,
                        ..
                    } => self.apply_propose(proposal_id, proposal, origin),
                    _ => self.apply_show(id, intent, origin),
                }
            }
            UiIntent::Highlight { range, scroll } => {
                self.apply_highlight(id, origin, *range, *scroll)
            }
            UiIntent::ShowHome { filter } => {
                if let Some(manager) = self.management.as_mut() {
                    if let Some(filter) = filter {
                        manager.query = sanitize_with_limit(filter, 100);
                        manager.selected = 0;
                    }
                    return Applied::Done;
                }
                ui_intent::carry(id, intent.clone(), origin.clone());
                self.show_management = Some(ManagementEntry::Show);
                Applied::Restarting
            }
            UiIntent::ShowSettings { page, setting } => {
                let mut state = PreferencesState::open(&self.repo_root);
                match state.show_page(*page, setting.as_deref()) {
                    Ok(()) => {
                        self.preferences = Some(state);
                        Applied::Done
                    }
                    Err(what) => Applied::Not(Box::new(UiOutcome::NotFound { what })),
                }
            }
            UiIntent::ShowPalette { query } => {
                // Opened with the query typed in; Enter is the user's.
                self.input_mode = InputMode::CommandPalette {
                    draft: sanitize_with_limit(query, 100),
                    selected: 0,
                };
                Applied::Done
            }
            UiIntent::ShowHelp { topic } => {
                self.help_filter = topic
                    .as_deref()
                    .map(|t| sanitize_with_limit(t, 100))
                    .unwrap_or_default();
                self.help_scroll = 0;
                // A topic is a filter on the key bindings.
                crate::help_view::set_tab(0);
                self.show_help = true;
                Applied::Done
            }
            UiIntent::Back => self.agent_back(id, origin),
            // Notify never gets here.
            _ => Applied::Not(Box::new(UiOutcome::refused(en(|| {
                gitpoulpi_i18n::msg::agent_reason::NOT_SUPPORTED
                    .get()
                    .to_string()
            })))),
        }
    }

    /// The view as it is now, for Back.
    fn capture_view(&self) -> ViewSnapshot {
        ViewSnapshot {
            repo_root: self.repo_root.clone(),
            home: self.management.is_some(),
            focus: focus_name(self.focus),
            left_collapsed: self.left_pane_collapsed,
            right_collapsed: self.right_pane_collapsed,
            selected_commit: selected_log_commit(
                &self.log_result,
                &self.log_state,
                &self.log_order,
            ),
            central_log: self.central_view == CentralView::Log,
        }
    }

    fn agent_back(&mut self, id: Option<u64>, origin: &UiOrigin) -> Applied {
        let Some(snapshot) = ui_intent::pop() else {
            return Applied::Not(Box::new(UiOutcome::refused(en(|| {
                gitpoulpi_i18n::msg::agent_reason::NOTHING_TO_GO_BACK
                    .get()
                    .to_string()
            }))));
        };
        let here =
            !snapshot.home && self.management.is_none() && snapshot.repo_root == self.repo_root
                || snapshot.home && self.management.is_some();
        if here {
            self.restore_view(&snapshot);
            return Applied::Done;
        }
        // Another tab (or Home): the next window restores on arrival.
        ui_intent::hand_over_restore(snapshot.clone());
        ui_intent::carry(id, UiIntent::Back, origin.clone());
        if snapshot.home {
            self.show_management = Some(ManagementEntry::Show);
            return Applied::Restarting;
        }
        let message = self
            .tab_ctx()
            .switcher_action(repo_switcher::SwitcherAction::Open(snapshot.repo_root));
        if self.switch_to.is_some() {
            Applied::Restarting
        } else {
            ui_intent::take_restore();
            ui_intent::take_carried();
            Applied::Not(Box::new(UiOutcome::NotFound {
                what: message.unwrap_or_else(|| {
                    en(|| {
                        gitpoulpi_i18n::msg::agent_reason::TAB_GONE
                            .get()
                            .to_string()
                    })
                }),
            }))
        }
    }

    /// Puts `snapshot`'s view back in this window: closes what an intent
    /// opened over it, then the focus, the pane collapse and the Log's
    /// selected commit (once the Log has it).
    fn restore_view(&mut self, snapshot: &ViewSnapshot) {
        self.close_agent_overlays();
        if snapshot.central_log
            && matches!(self.central_view, CentralView::Diff | CentralView::File)
        {
            // What an intent opened over the Log: a diff or a file.
            self.diff_view_state = None;
            self.file_insight = None;
            self.central_view = CentralView::Log;
        }
        // What an intent opened over the Log: the merge editor, unless it
        // holds the user's own work.
        if snapshot.central_log
            && self.central_view == CentralView::Merge
            && self.merge.as_ref().is_some_and(|v| !v.model.is_dirty())
        {
            self.merge = None;
            self.central_view = CentralView::Log;
        }
        self.left_pane_collapsed = snapshot.left_collapsed;
        self.right_pane_collapsed = snapshot.right_collapsed;
        self.focus = match snapshot.focus {
            "refs" if !self.left_pane_collapsed => Focus::Refs,
            "status" if !self.right_pane_collapsed => Focus::Status,
            _ => Focus::Log,
        };
        let Some(oid) = snapshot.selected_commit else {
            return;
        };
        let now = selected_log_commit(&self.log_result, &self.log_state, &self.log_order);
        if now != Some(oid)
            && !select_log_commit(&self.log_result, &self.log_order, &mut self.log_state, oid)
        {
            self.ui_viewer.set_reveal(Some(Reveal::SelectCommit {
                oid,
                until: Instant::now() + Duration::from_secs(6),
            }));
        }
    }

    /// Closes Settings, Help and the palette, the overlays an intent opens.
    fn close_agent_overlays(&mut self) {
        if self.preferences.is_some() {
            // Like leaving Settings by hand: what the TUI reads from the
            // file is republished.
            let cx = &mut self.command_ctx();
            *cx.preferences = None;
            cx.link.request_refresh();
            if let Some(load) = apply_live_settings(
                &mut *cx.last_message,
                &mut *cx.hierarchical_refs,
                &mut *cx.graph_show_stashes,
                &mut *cx.ref_connector,
                &mut *cx.avatar_placement,
                &mut *cx.avatar_live,
                &mut *cx.ui_lane_background,
                &mut *cx.ui_graph_theme,
                &mut *cx.ui_tint_strength,
            ) {
                cx.lazy_log.apply_settings(load);
            }
        }
        self.show_help = false;
        if matches!(self.input_mode, InputMode::CommandPalette { .. }) {
            self.input_mode = InputMode::Normal;
        }
    }

    /// Ticks the follow-up of a restore (the Log may not have the commit
    /// yet: a tab just opened) or of an agent's `show commit`. Gives up after
    /// its deadline.
    fn agent_reveal_tick(&mut self) {
        match self.ui_viewer.take_reveal() {
            Some(Reveal::SelectCommit { oid, until }) => {
                if Instant::now() < until
                    && !select_log_commit(
                        &self.log_result,
                        &self.log_order,
                        &mut self.log_state,
                        oid,
                    )
                {
                    self.ui_viewer
                        .set_reveal(Some(Reveal::SelectCommit { oid, until }));
                }
            }
            Some(Reveal::Show(show)) => self.tick_show_reveal(show),
            Some(Reveal::View(view)) => self.tick_view_reveal(view),
            Some(Reveal::Highlight(h)) => self.tick_highlight_reveal(h),
            None => {}
        }
    }
}
