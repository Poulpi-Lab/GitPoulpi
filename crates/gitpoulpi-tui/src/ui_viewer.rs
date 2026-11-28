//! The TUI as a *viewer* for an external agent's view intents (`gitpoulpi ui
//! ...`, the MCP `ui_*` tools). Design: the mcp-show-ui design.
//!
//! Two halves, joined by [`UiBridge`] (one per `DaemonLink`, held in
//! `Activity` so the link's reader and select loop already reach it):
//!
//! - **link side** (`daemon_link`): the reader drops each `Event::UiIntent` /
//!   `Event::UiStateRequest` addressed to this process into the inbox; the
//!   select loop wakes on the bridge and sends `UiAttach` / `UiPresence`
//!   (what the daemon routes by) and the answers (`UiReport`) on its own
//!   connection, which is the one the daemon addresses.
//! - **render side** ([`UiViewer`]): once per frame, [`UiViewer::frame`]
//!   publishes the window's presence, answers state requests (the text of the
//!   central pane is read from the frame just drawn, so what an agent reads
//!   is what the user sees, secret fields masked as on screen), shows a
//!   `notify` toast and, for every other intent, either refuses cleanly or
//!   (`ask` mode) offers a chip the user accepts or dismisses.
//!
//! The view-moving intents are applied by the window itself
//! (`event_loop/window/agent_view.rs`), which takes them from [`UiViewer`]'s
//! queue once per tick, after the deferral policy, coalescing and dwell.
//! Intents the build cannot show answer "not supported by this build".
//! Nothing here can mutate a repository.

use crate::th;
use std::sync::OnceLock;
use std::time::{Duration, Instant};

use crate::wide_edge::Clear;
use crossterm::event::{Event, KeyCode, MouseButton, MouseEventKind};
use gitpoulpi_core::domain::{RepoId, UiControl};
use gitpoulpi_ipc::ui::{
    sanitize_display, ProposalStatus, UiIntent, UiOrigin, UiOutcome, UiReportArgs, UiReportBody,
    UiState, UiTab, ViewerInfo,
};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::Paragraph;
use ratatui::Frame;

/// How long a `notify` toast stays up.
const TOAST_FOR: Duration = Duration::from_secs(6);
/// How long the "Shown by ..." toast of an applied intent stays up.
const SHOWN_FOR: Duration = Duration::from_secs(4);
/// A deferred intent offered on the chip expires after this.
pub(crate) const CHIP_FOR: Duration = Duration::from_secs(60);
/// Two applied intents are at least this far apart, so the screen never strobes.
pub(crate) const DWELL: Duration = Duration::from_millis(400);
/// An intent that arrives this soon after the user's last key or click is
/// deferred: the user is in the middle of something.
pub(crate) const RECENT_INPUT: Duration = Duration::from_millis(1500);
/// An intent still unapplied this long after it arrived (held back by the
/// dwell, or the window was busy) is dropped as `Refused("stale")`: the agent
/// gave up waiting (the daemon's own timeout is 8 s) and the screen it
/// meant has moved on. `GITPOULPI_UI_STALE_MS` overrides it (tests).
pub(crate) fn stale_after() -> Duration {
    static MS: OnceLock<u64> = OnceLock::new();
    Duration::from_millis(*MS.get_or_init(|| {
        std::env::var("GITPOULPI_UI_STALE_MS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(8000)
    }))
}
/// The most lines `ui state --text` returns.
const TEXT_LINES: usize = 200;

pub(crate) use crate::ui_viewer_pure::*;

/// What the window knows about itself this frame, as plain data.
pub(crate) struct Facts {
    pub(crate) active_repo: RepoId,
    /// Every tab's repository and whether it is the active one.
    pub(crate) tabs: Vec<(RepoId, bool)>,
    pub(crate) central: &'static str,
    pub(crate) focus: &'static str,
    pub(crate) right_panel: &'static str,
    pub(crate) overlay: Option<&'static str>,
    pub(crate) input_busy: bool,
    pub(crate) log_selected: Option<usize>,
    pub(crate) selected_commit: Option<String>,
    /// The central pane, for `ui state --text`.
    pub(crate) central_area: Rect,
    /// The Settings screen's page id and the row an agent landed on.
    pub(crate) settings: Option<(&'static str, Option<String>)>,
    /// The open diff / file, for `ui state` (`agent_look.rs`).
    pub(crate) diff: Option<serde_json::Value>,
    pub(crate) file: Option<serde_json::Value>,
    /// The open merge editor, for `ui state`.
    pub(crate) merge: Option<serde_json::Value>,
}

/// A deferred intent offered on the footer chip.
struct Chip {
    what: String,
    client: String,
    intent: UiIntent,
    origin: UiOrigin,
    at: Instant,
    rect: Rect,
}

/// An intent waiting for the window's next tick.
pub(crate) struct Queued {
    pub(crate) id: u64,
    pub(crate) intent: UiIntent,
    pub(crate) origin: UiOrigin,
    /// When the window received it (the stale-drop clock).
    pub(crate) at: Instant,
}

/// The answer an applied intent gets after the next drawn frame.
enum Report {
    Applied,
    Partial(String),
    /// A proposal's dialog opened (its id).
    Proposed(String),
}

/// What the user chose on the chip or from the palette, for the next tick.
#[derive(Default)]
struct Wishes {
    accept: bool,
    back: bool,
    /// A view the user chose from the window itself (the workspace search's
    /// Enter): applied like an intent, but not announced as an agent's.
    go: Option<UiIntent>,
}

/// The `UiOrigin::client` of a view the user chose in this window: applied
/// through the intent machinery (tab switch carried across the window
/// restart, Log paging, Back) without the "shown by" toast or the tab mark.
pub(crate) const LOCAL_CLIENT: &str = "workspace-search";

/// Whether `origin` is the user's own choice: made in this window (see
/// [`LOCAL_CLIENT`]) or typed in a terminal (`gitpoulpi --add`).
pub(crate) fn is_local(origin: &UiOrigin) -> bool {
    origin.client == LOCAL_CLIENT || origin.client == gitpoulpi_ipc::ui::ADD_CLIENT
}

/// The origin of a view the user chose in this window.
pub(crate) fn local_origin() -> UiOrigin {
    UiOrigin {
        client: LOCAL_CLIENT.into(),
        label: None,
        control: UiControl::Auto,
    }
}

/// An asynchronous follow-up of a view change: ticked once per window tick
/// until it lands or its deadline passes.
pub(crate) enum Reveal {
    /// Put the Log's selection back on a commit once the Log has it.
    SelectCommit {
        oid: gitpoulpi_core::domain::Oid,
        until: Instant,
    },
    /// An agent's `show commit` / `show ref` waiting for the Log to page in
    /// the commit.
    Show(Box<ShowReveal>),
    /// An agent's `show diff` / `show file`: select the commit, wait for
    /// the file and its rows, then place the view.
    View(Box<ViewReveal>),
    /// An agent's `highlight` on a diff, waiting for the whole-file layout
    /// it widened the diff to (the line was outside every hunk).
    Highlight(Box<HighlightReveal>),
}

/// How often a pending reveal looks at the window again (the Log page it
/// waits for lands on a channel the tick does not own).
pub(crate) const REVEAL_POLL: Duration = Duration::from_millis(50);

/// The commit an agent asked to see, and what goes with it.
pub(crate) struct CommitGoal {
    pub(crate) oid: gitpoulpi_core::domain::Oid,
    pub(crate) also: Vec<gitpoulpi_core::domain::Oid>,
    pub(crate) files: bool,
    pub(crate) focus: Option<gitpoulpi_ipc::ui::PaneId>,
}

/// An intent whose answer waits for an asynchronous step.
pub(crate) struct ShowReveal {
    /// The daemon's id when the agent is still waiting for the answer.
    pub(crate) id: Option<u64>,
    pub(crate) origin: UiOrigin,
    /// What the toast calls it.
    pub(crate) what: String,
    pub(crate) goal: CommitGoal,
    /// Already decided to answer `Partial` with this note when the rest
    /// works out (a ref row selected, its commit unreachable).
    pub(crate) note: Option<String>,
    /// The view already moved (a ref row is selected): failing to reach the
    /// commit is a `Partial`, not a `NotFound`.
    pub(crate) moved: bool,
    /// The Back stack got a snapshot for this intent.
    pub(crate) pushed: bool,
    pub(crate) started: Instant,
    /// The current step's deadline.
    pub(crate) until: Instant,
    pub(crate) poll_at: Instant,
    pub(crate) pages: u32,
}

/// An agent's `show diff` / `show file` on its way to the screen.
pub(crate) struct ViewReveal {
    pub(crate) id: Option<u64>,
    pub(crate) origin: UiOrigin,
    pub(crate) what: String,
    /// The intent being shown (`ShowDiff` or `ShowFile`).
    pub(crate) goal: UiIntent,
    /// The commit a diff of a commit is shown from, while it is paged in
    /// and selected in the Log.
    pub(crate) select: Option<Box<ShowReveal>>,
    /// A `Partial` note to answer with once the rest works out.
    pub(crate) note: Option<String>,
    /// The Diff view / File View is open (the view moved).
    pub(crate) opened: bool,
    /// The diff was switched to the whole-file layout to reach a line.
    pub(crate) widened: bool,
    pub(crate) pushed: bool,
    pub(crate) started: Instant,
    /// The current step's deadline.
    pub(crate) until: Instant,
    pub(crate) poll_at: Instant,
    /// Test hook (`GITPOULPI_UI_REVEAL_HOLD_MS`): do nothing before this, so
    /// a test can press a key while the reveal is pending.
    pub(crate) hold_until: Option<Instant>,
}

/// A `highlight` that widened the open diff and waits for its rows.
pub(crate) struct HighlightReveal {
    pub(crate) id: Option<u64>,
    pub(crate) origin: UiOrigin,
    pub(crate) what: String,
    /// The side (0 old, 1 new) and first line of the band.
    pub(crate) side: usize,
    pub(crate) from: u32,
    pub(crate) pushed: bool,
    pub(crate) started: Instant,
    pub(crate) until: Instant,
    pub(crate) poll_at: Instant,
}

pub(crate) struct UiViewer {
    allowed: bool,
    focused: Option<bool>,
    published: Option<(u64, ViewerInfo)>,
    toast: Option<(String, Instant, Duration)>,
    chip: Option<Chip>,
    /// Whether the last frame had a modal up (the chip's keys wait for it).
    busy: bool,
    /// Intents received and not yet decided on (coalesced by the tick).
    queue: Vec<Queued>,
    last_input: Option<Instant>,
    last_applied: Option<Instant>,
    /// Applied intents whose `Applied` answer waits for the next drawn
    /// frame, so the state it carries is the one on screen.
    to_report: Vec<(u64, Report)>,
    wishes: Wishes,
    reveal: Option<Reveal>,
    /// Where the cursor was when an agent's highlight band was placed: the
    /// band goes when the user moves it away (or presses Esc).
    tint_cursor: Option<usize>,
    esc_pressed: bool,
    /// Esc pressed since the window last looked: dismisses a proposed
    /// sequence when nothing else owned the key.
    banner_esc: bool,
}

impl UiViewer {
    /// `allowed`: `[ui] allow_agent_view_control`. Off, this window never
    /// attaches, so the daemon finds no viewer to route to.
    pub(crate) fn new(allowed: bool) -> Self {
        Self {
            allowed,
            focused: None,
            published: None,
            toast: None,
            chip: None,
            busy: false,
            queue: Vec::new(),
            last_input: None,
            last_applied: None,
            to_report: Vec::new(),
            wishes: Wishes::default(),
            reveal: None,
            tint_cursor: None,
            esc_pressed: false,
            banner_esc: false,
        }
    }

    /// Terminal focus is tracked, and the time of the user's last key or
    /// click (the deferral policy). `Esc` and a click on the chip answer it;
    /// its keyboard twins are the `Ctrl+N` actions and the palette entries.
    /// Returns `true` when the event was consumed.
    pub(crate) fn handle_event(&mut self, event: &Event) -> bool {
        match event {
            Event::FocusGained => self.focused = Some(true),
            Event::FocusLost => self.focused = Some(false),
            Event::Key(key) => {
                self.last_input = Some(Instant::now());
                crate::ui_intent::clear_marked_tab();
                if key.code == KeyCode::Esc {
                    self.esc_pressed = true;
                    self.banner_esc = true;
                }
                if self.chip.is_some() && !self.busy && key.code == KeyCode::Esc {
                    self.drop_chip(ProposalStatus::Rejected);
                    return true;
                }
            }
            Event::Mouse(m) if m.kind != MouseEventKind::Moved => {
                self.last_input = Some(Instant::now());
                crate::ui_intent::clear_marked_tab();
                if m.kind == MouseEventKind::Down(MouseButton::Left) {
                    match crate::ui_intent::proposal::banner_hit(m.column, m.row) {
                        Some(crate::ui_intent::proposal::BannerButton::Run) => {
                            crate::ui_intent::proposal::run_sequence();
                            return true;
                        }
                        Some(crate::ui_intent::proposal::BannerButton::Dismiss) => {
                            crate::ui_intent::proposal::dismiss_sequence();
                            return true;
                        }
                        None => {}
                    }
                    let hit = self.chip.as_ref().is_some_and(|c| {
                        c.rect
                            .contains(ratatui::layout::Position::new(m.column, m.row))
                    });
                    if hit {
                        self.accept_pending();
                        return true;
                    }
                }
            }
            Event::Paste(_) => {
                self.last_input = Some(Instant::now());
                crate::ui_intent::clear_marked_tab();
            }
            _ => {}
        }
        false
    }

    /// The user chose to show the offered view (chip click, `Ctrl+N Enter`,
    /// the palette): the next tick applies it.
    /// The terminal said it lost focus (unknown counts as focused).
    pub(crate) fn unfocused(&self) -> bool {
        self.focused == Some(false)
    }

    pub(crate) fn accept_pending(&mut self) {
        if crate::ui_intent::proposal::sequence_pending() {
            crate::ui_intent::proposal::run_sequence();
            return;
        }
        if self.chip.is_some() {
            self.wishes.accept = true;
            crate::redraw::poke();
        }
    }

    /// The user dropped the offered view.
    pub(crate) fn dismiss_pending(&mut self) {
        crate::ui_intent::proposal::dismiss_sequence();
        self.drop_chip(ProposalStatus::Rejected);
    }

    /// Removes the chip; a proposal on it is settled with `status`.
    fn drop_chip(&mut self, status: ProposalStatus) {
        if let Some(Chip {
            intent: UiIntent::Propose { id, .. },
            ..
        }) = self.chip.take()
        {
            crate::ui_intent::proposal::settle(&id, status);
        }
    }

    /// Whether a deferred intent waits behind the chip.
    pub(crate) fn has_pending(&self) -> bool {
        self.chip.is_some() || crate::ui_intent::proposal::sequence_pending()
    }

    /// Whether Esc was pressed since the last call (once).
    pub(crate) fn take_banner_esc(&mut self) -> bool {
        std::mem::take(&mut self.banner_esc)
    }

    /// The user chose to go to `intent`'s view (the workspace search): the
    /// next tick applies it.
    pub(crate) fn go(&mut self, intent: UiIntent) {
        self.wishes.go = Some(intent);
        crate::redraw::poke();
    }

    pub(crate) fn take_go(&mut self) -> Option<UiIntent> {
        self.wishes.go.take()
    }

    /// The user chose Back (`Ctrl+N Backspace`, the palette).
    pub(crate) fn request_back(&mut self) {
        self.wishes.back = true;
        crate::redraw::poke();
    }

    /// Whether the user's Back would do something (the palette's rule).
    pub(crate) fn can_go_back(&self) -> bool {
        crate::ui_intent::depth() > 0
    }

    /// Whether this window's presence reached the daemon on its current
    /// link: only then are answers accepted (the daemon takes a report from
    /// the connection that attached).
    pub(crate) fn is_attached(&self, bridge: &UiBridge) -> bool {
        self.published
            .as_ref()
            .is_some_and(|(serial, _)| *serial == bridge.serial)
    }

    pub(crate) fn take_back_request(&mut self) -> bool {
        std::mem::take(&mut self.wishes.back)
    }

    /// The intent the user accepted from the chip, with who offered it.
    pub(crate) fn take_accepted(&mut self) -> Option<(UiIntent, UiOrigin)> {
        if !std::mem::take(&mut self.wishes.accept) {
            return None;
        }
        self.chip.take().map(|c| (c.intent, c.origin))
    }

    /// Everything that arrived since the last tick.
    pub(crate) fn take_queue(&mut self) -> Vec<Queued> {
        std::mem::take(&mut self.queue)
    }

    /// Puts an intent back to wait out the dwell.
    pub(crate) fn hold(&mut self, held: Queued) {
        self.queue.push(held);
    }

    pub(crate) fn recent_input(&self) -> bool {
        self.last_input
            .is_some_and(|at| at.elapsed() < RECENT_INPUT)
    }

    /// Time left before another intent may be applied.
    pub(crate) fn dwell_left(&self) -> Option<Duration> {
        let at = self.last_applied?;
        DWELL.checked_sub(at.elapsed()).filter(|d| !d.is_zero())
    }

    pub(crate) fn note_applied(&mut self) {
        self.last_applied = Some(Instant::now());
    }

    /// The `Applied` answer for `id` is sent from the next drawn frame.
    pub(crate) fn report_after_frame(&mut self, id: u64) {
        self.to_report.push((id, Report::Applied));
        crate::redraw::poke();
    }

    /// The `Partial` answer for `id` (the view moved, with `note` to say what
    /// did not), sent from the next drawn frame.
    pub(crate) fn report_partial_after_frame(&mut self, id: u64, note: String) {
        self.to_report.push((id, Report::Partial(note)));
        crate::redraw::poke();
    }

    /// The `Proposed` answer for `id` (proposal `proposal`'s dialog opened),
    /// sent from the next drawn frame.
    pub(crate) fn report_proposed_after_frame(&mut self, id: u64, proposal: String) {
        self.to_report.push((id, Report::Proposed(proposal)));
        crate::redraw::poke();
    }

    /// Whether the user pressed a key or clicked since `at`.
    pub(crate) fn input_since(&self, at: Instant) -> bool {
        self.last_input.is_some_and(|input| input > at)
    }

    /// Answers `id` now (anything but `Applied`).
    pub(crate) fn report_now(&self, bridge: &UiBridge, id: u64, outcome: UiOutcome) {
        bridge.report(UiReportArgs {
            id,
            body: UiReportBody::Outcome { outcome },
        });
    }

    /// Offers `intent` on the chip instead of applying it.
    pub(crate) fn offer(&mut self, intent: UiIntent, origin: UiOrigin) {
        // A proposal the chip held is replaced: it expires.
        self.drop_chip(ProposalStatus::Expired);
        if let UiIntent::Propose { id, proposal, .. } = &intent {
            crate::ui_intent::proposal::record(
                id,
                proposal.kind(),
                &sanitize_display(&origin.client),
                ProposalStatus::Offered,
            );
        }
        self.chip = Some(Chip {
            what: crate::ui_intent::describe(&intent),
            client: sanitize_display(&origin.client),
            intent,
            origin,
            at: Instant::now(),
            rect: Rect::default(),
        });
        crate::redraw::poke();
    }

    /// The "Shown by <client>: <what>" toast of an applied intent.
    pub(crate) fn shown(&mut self, origin: &UiOrigin, what: &str) {
        if is_local(origin) {
            return;
        }
        let km = crate::keymap::current();
        let back = format!(
            "{} {}",
            km.shown(crate::keymap::Action::GlobalNavPrefix),
            km.shown_key(crate::keymap::Action::NavAgentBack)
        );
        self.toast = Some((
            format!(
                "{} {}",
                crate::theme::AI_SPARK,
                gitpoulpi_i18n::msg::ui_intent::shown(sanitize_display(&origin.client), what, back)
            ),
            Instant::now(),
            SHOWN_FOR,
        ));
    }

    /// A plain line in the toast (a refusal the user should see).
    pub(crate) fn say(&mut self, text: String) {
        self.toast = Some((text, Instant::now(), SHOWN_FOR));
    }

    /// An agent's highlight band now sits with the cursor at `cursor`
    /// (`None`: no band).
    pub(crate) fn note_tint(&mut self, cursor: Option<usize>) {
        self.tint_cursor = cursor;
        self.esc_pressed = false;
    }

    /// Where the cursor was when the band was placed, if there is one.
    pub(crate) fn tint_cursor(&self) -> Option<usize> {
        self.tint_cursor
    }

    /// Whether Esc was pressed since the last look (consumed).
    pub(crate) fn take_esc(&mut self) -> bool {
        std::mem::take(&mut self.esc_pressed)
    }

    pub(crate) fn set_reveal(&mut self, reveal: Option<Reveal>) {
        self.reveal = reveal;
    }

    pub(crate) fn take_reveal(&mut self) -> Option<Reveal> {
        self.reveal.take()
    }

    /// When something time-driven here becomes visible or due, for the
    /// render loop to wake at: the toast and chip expiring, an intent
    /// waiting out the dwell, a reveal giving up.
    pub(crate) fn next_deadline(&self) -> Option<Instant> {
        let toast = self.toast.as_ref().map(|(_, at, for_)| *at + *for_);
        let chip = self.chip.as_ref().map(|c| c.at + CHIP_FOR);
        let dwell = (!self.queue.is_empty())
            .then(|| self.last_applied.map(|at| at + DWELL))
            .flatten();
        let reveal = self.reveal.as_ref().map(|r| match r {
            Reveal::SelectCommit { until, .. } => *until,
            Reveal::Show(show) => show.until.min(show.poll_at),
            Reveal::View(view) => view.until.min(view.poll_at),
            Reveal::Highlight(h) => h.until.min(h.poll_at),
        });
        [toast, chip, dwell, reveal].into_iter().flatten().min()
    }

    /// Once per frame, right after it is drawn.
    pub(crate) fn frame(&mut self, bridge: &UiBridge, facts: &Facts, buffer: &Buffer) {
        self.busy = facts.input_busy;
        self.publish_presence(bridge, facts);
        if self
            .toast
            .as_ref()
            .is_some_and(|(_, at, for_)| at.elapsed() > *for_)
        {
            self.toast = None;
            // Cleared after this frame was drawn: one more to show it gone.
            crate::redraw::poke();
        }
        if self
            .chip
            .as_ref()
            .is_some_and(|c| c.at.elapsed() > CHIP_FOR)
        {
            self.drop_chip(ProposalStatus::Expired);
            crate::redraw::poke();
        }
        for (id, note) in std::mem::take(&mut self.to_report) {
            let state = self.state(facts);
            let outcome = match note {
                Report::Partial(note) => UiOutcome::Partial { state, note },
                Report::Applied => UiOutcome::Applied { state },
                Report::Proposed(id) => UiOutcome::Proposed { id, state },
            };
            bridge.report(UiReportArgs {
                id,
                body: UiReportBody::Outcome { outcome },
            });
        }
        for request in bridge.take_inbound() {
            match request {
                Inbound::State { id, include_text } => {
                    let mut state = self.state(facts);
                    if include_text {
                        state.text = Some(visible_text(buffer, facts.central_area));
                    }
                    bridge.report(UiReportArgs {
                        id,
                        body: UiReportBody::State { state },
                    });
                }
                Inbound::Intent { id, intent, origin } => {
                    // The daemon re-sends an unanswered intent when a window
                    // re-attaches; the one the previous window carried over
                    // is already in hand.
                    if crate::ui_intent::is_handled(id) {
                        continue;
                    }
                    let intent = *intent;
                    if let Some(outcome) = self.accept_inbound(&intent, &origin, facts) {
                        bridge.report(UiReportArgs {
                            id,
                            body: UiReportBody::Outcome { outcome },
                        });
                    } else {
                        self.queue.push(Queued {
                            id,
                            intent,
                            origin,
                            at: Instant::now(),
                        });
                        crate::redraw::poke();
                    }
                }
            }
        }
    }

    fn publish_presence(&mut self, bridge: &UiBridge, facts: &Facts) {
        // Off, a window never announces itself: the daemon finds no viewer.
        if !self.allowed {
            return;
        }
        let info = ViewerInfo {
            id: viewer_id().clone(),
            pid: std::process::id(),
            active_repo: Some(facts.active_repo.clone()),
            tabs: facts.tabs.iter().map(|(r, _)| r.clone()).collect(),
            focused: self.focused,
            agent_control: self.allowed,
            last_focus_at: None,
        };
        let key = bridge.serial;
        if self.published.as_ref() == Some(&(key, info.clone())) {
            return;
        }
        bridge.publish(info.clone());
        // The window may hold an intent carried over from the one it
        // replaced, waiting for this attach: tick again.
        crate::redraw::poke();
        self.published = Some((key, info));
    }

    fn state(&self, facts: &Facts) -> UiState {
        UiState {
            viewer: Some(viewer_id().clone()),
            focused: self.focused,
            repo: Some(facts.active_repo.0.clone()),
            tabs: facts
                .tabs
                .iter()
                .map(|(r, active)| UiTab {
                    path: r.0.clone(),
                    active: *active,
                })
                .collect(),
            central: Some(facts.central.into()),
            focus: Some(facts.focus.into()),
            right_panel: Some(facts.right_panel.into()),
            overlay: facts.overlay.map(Into::into),
            input_busy: facts.input_busy,
            log: Some(serde_json::json!({
                "selected_row": facts.log_selected,
                "selected_commit": facts.selected_commit,
            })),
            settings: facts
                .settings
                .as_ref()
                .map(|(page, setting)| serde_json::json!({ "page": page, "setting": setting })),
            diff: facts.diff.clone(),
            file: facts.file.clone(),
            merge: facts.merge.clone(),
            proposals: crate::ui_intent::proposal::records(),
            ..UiState::default()
        }
    }

    /// What can be answered at once: a switched-off window, a toast. `None`
    /// means the intent goes to the window's tick (policy, then applying).
    fn accept_inbound(
        &mut self,
        intent: &UiIntent,
        origin: &UiOrigin,
        facts: &Facts,
    ) -> Option<UiOutcome> {
        if !self.allowed {
            return Some(UiOutcome::refused(
                "the user turned agent view control off in this window", // i18n-english
            ));
        }
        match intent {
            UiIntent::Notify { message } => {
                let client = sanitize_display(&origin.client);
                let message = sanitize_display(message);
                self.toast = Some((
                    format!("\u{2726} {client}: {message}"),
                    Instant::now(),
                    TOAST_FOR,
                ));
                Some(UiOutcome::Applied {
                    state: self.state(facts),
                })
            }
            _ => None,
        }
    }

    /// Draws the chip, else the toast, over the bottom of the frame. Called
    /// last in the draw so nothing covers it; the chip's rect is the click
    /// target, taken from this paint.
    pub(crate) fn render(&mut self, frame: &mut Frame<'_>) {
        let area = frame.area();
        if area.height < 3 || area.width < 8 {
            return;
        }
        crate::ui_intent::proposal::render_banner(frame, area);
        let (text, style, is_chip) = if let Some(chip) = &self.chip {
            let km = crate::keymap::current();
            let accept = format!(
                "{} {}",
                km.shown(crate::keymap::Action::GlobalNavPrefix),
                km.shown_key(crate::keymap::Action::NavAgentShow)
            );
            (
                if matches!(chip.intent, UiIntent::Propose { .. }) {
                    format!(
                        " {} {} ",
                        crate::theme::AI_SPARK,
                        gitpoulpi_i18n::msg::ui_intent::chip_propose(
                            &chip.client,
                            &chip.what,
                            accept
                        )
                    )
                } else {
                    format!(
                        " {} {} ",
                        crate::theme::AI_SPARK,
                        gitpoulpi_i18n::msg::ui_intent::chip_show(&chip.client, &chip.what, accept)
                    )
                },
                Style::default()
                    .fg(th().chrome.on_bright)
                    .bg(th().chrome.marker),
                true,
            )
        } else if let Some((text, _, _)) = &self.toast {
            (
                format!(" {text} "),
                Style::default()
                    .fg(th().ui.on_button)
                    .bg(th().chrome.toast_bg)
                    .add_modifier(Modifier::BOLD),
                false,
            )
        } else {
            return;
        };
        let width = (crate::ui_kit::display_width(&text) as u16).min(area.width);
        let rect = Rect::new(area.right() - width, area.bottom() - 2, width, 1);
        frame.render_widget(Clear, rect);
        frame.render_widget(Paragraph::new(Line::from(text)).style(style), rect);
        if is_chip {
            if let Some(chip) = self.chip.as_mut() {
                chip.rect = rect;
            }
        }
    }
}

/// The rows of `area` as plain text, trailing blanks trimmed, at most
/// [`TEXT_LINES`]. Read from the drawn frame, so a masked field is masked.
fn visible_text(buffer: &Buffer, area: Rect) -> Vec<String> {
    let area = area.intersection(buffer.area);
    (area.top()..area.bottom())
        .take(TEXT_LINES)
        .map(|y| {
            let mut row = String::new();
            for x in area.left()..area.right() {
                let cell = &buffer[(x, y)];
                if cell.diff_option != ratatui::buffer::CellDiffOption::Skip {
                    row.push_str(cell.symbol());
                }
            }
            row.trim_end().to_string()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn facts() -> Facts {
        Facts {
            active_repo: RepoId("/r".into()),
            tabs: vec![(RepoId("/r".into()), true)],
            central: "log",
            focus: "log",
            right_panel: "commit_detail",
            overlay: None,
            input_busy: false,
            log_selected: Some(0),
            selected_commit: None,
            central_area: Rect::new(0, 0, 10, 2),
            settings: None,
            diff: None,
            file: None,
            merge: None,
        }
    }

    fn origin(control: UiControl) -> UiOrigin {
        UiOrigin {
            client: "claude-code".into(),
            label: None,
            control,
        }
    }

    #[test]
    fn viewer_id_is_stable_for_the_process() {
        assert_eq!(viewer_id(), viewer_id());
    }

    #[test]
    fn notify_is_a_sanitized_toast_and_other_intents_go_to_the_tick() {
        let mut v = UiViewer::new(true);
        let f = facts();
        let out = v.accept_inbound(
            &UiIntent::Notify {
                message: "hi\u{1b}[31m".into(),
            },
            &origin(UiControl::Ask),
            &f,
        );
        assert!(matches!(out, Some(UiOutcome::Applied { .. })));
        assert_eq!(v.toast.as_ref().unwrap().0, "\u{2726} claude-code: hi[31m");
        assert!(v
            .accept_inbound(&UiIntent::Back, &origin(UiControl::Auto), &f)
            .is_none());
    }

    #[test]
    fn an_offered_intent_waits_on_the_chip_until_accepted_or_dismissed() {
        let mut v = UiViewer::new(true);
        v.offer(UiIntent::ShowHelp { topic: None }, origin(UiControl::Ask));
        assert!(v.has_pending());
        assert!(v.take_accepted().is_none(), "nothing chosen yet");
        v.accept_pending();
        let (intent, _) = v.take_accepted().expect("accepted");
        assert_eq!(intent, UiIntent::ShowHelp { topic: None });
        assert!(!v.has_pending());
        v.offer(UiIntent::Back, origin(UiControl::Ask));
        v.dismiss_pending();
        assert!(!v.has_pending());
    }

    #[test]
    fn recent_input_and_the_dwell_are_tracked() {
        let mut v = UiViewer::new(true);
        assert!(!v.recent_input());
        v.handle_event(&Event::Key(crossterm::event::KeyEvent::new(
            KeyCode::Char('x'),
            crossterm::event::KeyModifiers::NONE,
        )));
        assert!(v.recent_input());
        assert!(v.dwell_left().is_none());
        v.note_applied();
        assert!(v.dwell_left().is_some());
    }

    #[test]
    fn a_window_with_the_switch_off_refuses_and_never_announces_itself() {
        let mut v = UiViewer::new(false);
        let bridge = UiBridge::default();
        v.frame(&bridge, &facts(), &Buffer::empty(Rect::new(0, 0, 10, 2)));
        assert!(bridge.take_outgoing().0.is_none());
        let out = v.accept_inbound(
            &UiIntent::Notify {
                message: "x".into(),
            },
            &origin(UiControl::Auto),
            &facts(),
        );
        assert!(matches!(out, Some(UiOutcome::Refused { .. })));
    }

    #[test]
    fn a_chip_expires_after_sixty_seconds_and_wakes_the_loop_for_it() {
        let mut v = UiViewer::new(true);
        let bridge = UiBridge::default();
        v.offer(UiIntent::ShowHelp { topic: None }, origin(UiControl::Ask));
        let due = v.next_deadline().expect("the chip has a deadline");
        assert!(due > Instant::now() + CHIP_FOR - Duration::from_secs(2));
        let buffer = Buffer::empty(Rect::new(0, 0, 10, 2));
        v.frame(&bridge, &facts(), &buffer);
        assert!(v.has_pending(), "still fresh");
        v.chip.as_mut().unwrap().at = Instant::now() - CHIP_FOR - Duration::from_secs(1);
        v.frame(&bridge, &facts(), &buffer);
        assert!(!v.has_pending(), "expired");
    }

    #[test]
    fn the_french_chip_and_toast_fit_one_line() {
        use ratatui::backend::TestBackend;
        use ratatui::Terminal;
        let _fr = gitpoulpi_i18n::override_lang(gitpoulpi_i18n::Lang::Fr);
        let mut v = UiViewer::new(true);
        v.offer(UiIntent::ShowHelp { topic: None }, origin(UiControl::Ask));
        let mut terminal = Terminal::new(TestBackend::new(100, 8)).unwrap();
        terminal.draw(|f| v.render(f)).unwrap();
        let buffer = terminal.backend().buffer().clone();
        let row: String = (0..100)
            .map(|x| buffer[(x, 6)].symbol().to_string())
            .collect();
        let row = row.trim();
        assert!(row.contains("claude-code veut afficher l'aide"), "{row}");
        assert!(
            row.ends_with("ignorer"),
            "the whole chip fits at 100 columns: {row}"
        );
        v.shown(&origin(UiControl::Ask), "l'aide");
        let toast = v.toast.as_ref().unwrap().0.clone();
        assert!(
            toast.contains("affiché par claude-code\u{a0}: l'aide"),
            "{toast}"
        );
    }

    #[test]
    fn the_highlight_watch_is_set_and_cleared() {
        let mut v = UiViewer::new(true);
        assert_eq!(v.tint_cursor(), None);
        v.note_tint(Some(4));
        assert_eq!(v.tint_cursor(), Some(4));
        v.handle_event(&Event::Key(crossterm::event::KeyEvent::new(
            KeyCode::Esc,
            crossterm::event::KeyModifiers::NONE,
        )));
        assert!(v.take_esc());
        assert!(!v.take_esc(), "consumed");
    }

    #[test]
    fn state_text_comes_from_the_drawn_frame() {
        let mut buffer = Buffer::empty(Rect::new(0, 0, 10, 3));
        buffer.set_string(0, 0, "hello", Style::default());
        buffer.set_string(0, 1, "***", Style::default());
        let mut v = UiViewer::new(true);
        let bridge = UiBridge::default();
        bridge.push_inbound(Inbound::State {
            id: 7,
            include_text: true,
        });
        v.frame(&bridge, &facts(), &buffer);
        let (info, reports) = bridge.take_outgoing();
        assert_eq!(info.unwrap().id, *viewer_id());
        let UiReportBody::State { state } = &reports[0].body else {
            panic!("a state report")
        };
        assert_eq!(state.text.as_deref().unwrap(), ["hello", "***"]);
        assert_eq!(state.central.as_deref(), Some("log"));
    }
}
