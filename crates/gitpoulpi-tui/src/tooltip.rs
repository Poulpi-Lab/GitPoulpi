//! The single owner of hover-tooltip lifecycle.
//!
//! Every tooltip (the Log's per-column tooltips, the linked-PR chip's) goes
//! through [`TooltipLifecycle`]; nothing else decides whether one is drawn.
//! A tooltip is visible only when ALL of these hold:
//!
//! * the pointer is still over the same hover target it dwelled on for
//!   [`DWELL`] (a new target restarts the timer);
//! * nothing modal is open (palette, tab picker, help, Preferences, dialogs,
//!   any context menu or popup, a drag in flight) -- the caller passes that
//!   as `blocked` and it is evaluated at paint time, so a modal that opens
//!   between two moves hides the tooltip on the very next frame;
//! * no key, click, scroll, resize or focus loss has happened since the
//!   pointer last moved ([`TooltipLifecycle::dismiss`]).
//!
//! Hiding is not just "stop painting": the terminal can keep glyphs the
//! ratatui diff believes are gone (wide glyphs, graphics), so the rect the
//! tooltip covered is handed back once via [`TooltipLifecycle::take_repaint`]
//! and the event loop force-rewrites those cells from the finished frame.

use crate::th;
use std::time::{Duration, Instant};

use crate::wide_edge::Clear;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::widgets::{Block, BorderType, Borders, Paragraph};
use ratatui::Frame;

use crate::chrome::ToolbarButton;
use crate::log_table::LogColumn;
use crate::ui_kit::anchored_popup;
use crate::ui_kit::ClippedRender;
use gitpoulpi_core::domain::Oid;
use gitpoulpi_i18n::msg;

/// How long the pointer must rest on one target before its tooltip shows.
pub(crate) const DWELL: Duration = Duration::from_millis(600);

/// What a tooltip is about; a change of key is a change of hover target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TooltipKey {
    Commit(Oid, LogColumn),
    LinkedPrChip,
    /// A button of the diff / File View bar.
    DiffBar(crate::diff_view::DiffBarButton),
    /// A plain one-line hint for a button ([`hint_for`]).
    Hint(&'static str),
    /// The Undo / Redo toolbar buttons: their text is the journal's, read
    /// at paint time (`undo_ui::tooltip`), so it follows live changes.
    UndoRedo(gitpoulpi_core::domain::Direction),
    /// A row of the open remote menu (the Pull / Push dropdown items): the
    /// text is the row action's own, read at paint time.
    RemoteMenuRow(usize),
    /// A sidebar branch under a workflow: its type, parent and distance
    /// (`flow::view::hover_tooltip`); the value is a hash of the name.
    FlowBranch(u64),
    /// A sidebar tag: where it lives (`tag_where_ui::hover_key`).
    TagWhere(u64),
    /// The author's name in the commit-detail pane: its email.
    CommitAuthor,
    /// The remotes badge of a Refs row (`multi_remote_badge::hover_key`).
    RefRemotes(u64),
}

// The email behind the commit-detail author name, set by the paint walk
// that registers `HitTarget::CommitAuthorName`, read by the tooltip paint.
#[cfg(not(test))]
static AUTHOR_EMAIL: std::sync::Mutex<Option<String>> = std::sync::Mutex::new(None);
#[cfg(test)]
thread_local! {
    static AUTHOR_EMAIL: std::cell::RefCell<Option<String>> =
        const { std::cell::RefCell::new(None) };
}

/// The commit-detail author name now on screen has this email (`None`: no
/// name is drawn).
pub(crate) fn set_author_email(email: Option<String>) {
    #[cfg(not(test))]
    if let Ok(mut slot) = AUTHOR_EMAIL.lock() {
        *slot = email;
    }
    #[cfg(test)]
    AUTHOR_EMAIL.with(|e| *e.borrow_mut() = email);
}

/// The email the author-name tooltip shows.
pub(crate) fn author_email() -> Option<String> {
    #[cfg(not(test))]
    {
        AUTHOR_EMAIL.lock().ok().and_then(|e| e.clone())
    }
    #[cfg(test)]
    {
        AUTHOR_EMAIL.with(|e| e.borrow().clone())
    }
}

/// The one-line tooltip of the working-changes panel's icon buttons.
pub(crate) fn hint_for(target: &crate::chrome::HitTarget) -> Option<&'static str> {
    use crate::chrome::HitTarget as H;
    match target {
        H::StatusDiscardAllButton => Some(crate::status_list::discard_all_label()),
        H::StatusExplainButton => Some(msg::tooltip::EXPLAIN_CHANGES.get()),
        H::AiGenerate => Some(msg::tooltip::AI_GENERATE.get()),
        H::AiCompose => Some(msg::tooltip::AI_COMPOSE.get()),
        H::SkipHooksCheckbox => Some(msg::tooltip::SKIP_HOOKS.get()),
        H::ComposerIssueRef => Some(msg::tooltip::INSERT_ISSUE_REF.get()),
        H::CommitReference(_) => Some(msg::tooltip::COMMIT_REFERENCE.get()),
        H::Toolbar(b) => toolbar_hint(*b),
        H::ToolbarDropdown(ToolbarButton::Pull) => Some(msg::toolbar::PULL_MENU_TIP.get()),
        H::ToolbarDropdown(ToolbarButton::Push) => Some(msg::toolbar::PUSH_MENU_TIP.get()),
        H::ToolbarDropdown(ToolbarButton::Undo) => Some(msg::undo_safety::TIP_CHIP.get()),
        H::ToolbarDropdown(_) => None,
        H::Merge(crate::merge_view::MergeHit::ResetAll) => Some(msg::merge::TIP_RESET.get()),
        H::Merge(crate::merge_view::MergeHit::TakeAllA) => Some(msg::merge::TIP_TAKE_ALL_A.get()),
        H::Merge(crate::merge_view::MergeHit::TakeAllB) => Some(msg::merge::TIP_TAKE_ALL_B.get()),
        H::Gear => Some(msg::toolbar::SETTINGS_TIP.get()),
        H::About => Some(gitpoulpi_i18n::msg::about::TOOLTIP.get()),
        H::ProfileField => Some(crate::profile_bar::tooltip()),
        H::CloseApp => Some(msg::tooltip::QUIT.get()),
        H::CommitColumn(crate::commit_column::ColumnHit::Reopen)
        | H::History(crate::history_view::HistoryHit::Reopen) => {
            Some(msg::commit_column::REOPEN_TIP.get())
        }
        H::NewTabButton => Some(msg::tooltip::NEW_TAB.get()),
        H::TabOverflow => Some(msg::tooltip::TAB_OVERFLOW.get()),
        H::TabScrollLeft => Some(msg::tooltip::TAB_SCROLL_LEFT.get()),
        H::TabScrollRight => Some(msg::tooltip::TAB_SCROLL_RIGHT.get()),
        H::RadarTab => Some(msg::tooltip::RADAR_TAB.get()),
        H::RadarClose => Some(msg::tooltip::RADAR_CLOSE.get()),
        H::NotesTab | H::NotesClose => Some(gitpoulpi_i18n::msg::updates::TAB_CLOSE_TIP.get()),
        H::GlobalSearchButton => Some(msg::tooltip::GLOBAL_SEARCH.get()),
        H::WorkspaceChip => Some(gitpoulpi_i18n::msg::workspaces::CHIP_TOOLTIP.get()),
        H::OpenRepositoryButton => Some(msg::tooltip::REPO_MANAGEMENT.get()),
        H::TabDir(crate::tab_dir::TabDirButton::QuitCd) => Some(msg::tooltip::QUIT_CD.get()),
        H::TabDir(crate::tab_dir::TabDirButton::Terminal) => Some(crate::open_in_bar::tooltip()),
        _ => None,
    }
}

/// The toolbar buttons' one-liners. Undo and Redo have their own live text
/// (`undo_ui::tooltip`), so they return `None` here.
fn toolbar_hint(button: ToolbarButton) -> Option<&'static str> {
    Some(match button {
        ToolbarButton::Undo | ToolbarButton::Redo => return None,
        ToolbarButton::Pull => msg::toolbar::PULL_TIP.get(),
        ToolbarButton::Push => msg::toolbar::PUSH_TIP.get(),
        ToolbarButton::Branch => msg::toolbar::BRANCH_TIP.get(),
        ToolbarButton::Tag => msg::toolbar::TAG_TIP.get(),
        ToolbarButton::Stash => msg::toolbar::STASH_TIP.get(),
        ToolbarButton::Pop => msg::toolbar::POP_TIP.get(),
        ToolbarButton::Lfs => msg::toolbar::LFS_TIP.get(),
        ToolbarButton::Flow => msg::flow::TOOLBAR_TIP.get(),
        ToolbarButton::Search => msg::toolbar::SEARCH_TIP.get(),
    })
}

/// [`hint_for`] with the effective binding of the button's key twin
/// appended (`" (Ctrl+G w)"`), nothing when it has none or is unbound.
pub(crate) fn hint_text(target: &crate::chrome::HitTarget) -> Option<String> {
    use crate::chrome::HitTarget as H;
    use crate::keymap::Action as K;
    let base = hint_for(target)?;
    let key = match target {
        H::StatusDiscardAllButton => Some(K::GitDiscardAll),
        H::StatusExplainButton => Some(K::GitAiExplain),
        H::AiGenerate => Some(K::GitAiCommitMessage),
        H::AiCompose => Some(K::GitAiCompose),
        H::SkipHooksCheckbox => Some(K::StatusSkipHooks),
        H::ComposerIssueRef => Some(K::ComposerInsertIssueRef),
        H::ToolbarDropdown(ToolbarButton::Undo) => Some(K::GlobalUndoHistory),
        H::Toolbar(ToolbarButton::Pull) => Some(K::GitPull),
        H::Toolbar(ToolbarButton::Push) => Some(K::GitPush),
        H::Toolbar(ToolbarButton::Branch) => Some(K::GitNewBranch),
        H::Toolbar(ToolbarButton::Tag) => Some(K::GitNewTag),
        H::Toolbar(ToolbarButton::Stash) => Some(K::GitStashPush),
        H::Toolbar(ToolbarButton::Pop) => Some(K::GitStashPop),
        H::Toolbar(ToolbarButton::Lfs) => Some(K::GitLfsMenu),
        H::Toolbar(ToolbarButton::Flow) => Some(K::GitFlowMenu),
        H::Toolbar(ToolbarButton::Search) => Some(K::GlobalFindInLog),
        H::Merge(crate::merge_view::MergeHit::ResetAll) => Some(K::MergeResetAll),
        H::Merge(crate::merge_view::MergeHit::TakeAllA) => Some(K::MergeTakeAllA),
        H::Merge(crate::merge_view::MergeHit::TakeAllB) => Some(K::MergeTakeAllB),
        H::GlobalSearchButton => Some(K::GlobalWorkspaceSearch),
        H::Gear => Some(K::GlobalSettings),
        H::CloseApp => Some(K::GlobalQuit),
        H::CommitColumn(crate::commit_column::ColumnHit::Reopen)
        | H::History(crate::history_view::HistoryHit::Reopen) => Some(K::GlobalCommitColumn),
        H::TabDir(crate::tab_dir::TabDirButton::Terminal) => Some(K::TabOpenInPopup),
        _ => None,
    };
    Some(match key {
        Some(a) => format!(
            "{base}{}",
            crate::keymap::paren(&crate::keymap::current(), a)
        ),
        None => base.to_string(),
    })
}

#[derive(Debug, Default)]
pub(crate) struct TooltipLifecycle {
    target: Option<TooltipKey>,
    since: Option<Instant>,
    last_pos: (u16, u16),
    /// Set by [`Self::pointer_moved`], cleared by [`Self::take_fed`]: lets the
    /// router tell a motion that reached the tooltip rule from one a surface
    /// swallowed.
    fed: bool,
    suppressed: bool,
    shown: Option<Rect>,
    repaint: Option<Rect>,
}

impl TooltipLifecycle {
    /// The pointer moved. `key` is the tooltip target under it, if any.
    pub(crate) fn pointer_moved(&mut self, key: Option<TooltipKey>, pos: (u16, u16), now: Instant) {
        if key != self.target {
            self.target = key;
            self.since = key.map(|_| now);
            self.suppressed = false;
        } else if self.suppressed && pos != self.last_pos {
            // Same target, but the user moved again after a dismissal.
            self.suppressed = false;
            self.since = Some(now);
        }
        self.last_pos = pos;
        self.fed = true;
    }

    /// Whether [`Self::pointer_moved`] ran since the last call; clears it.
    pub(crate) fn take_fed(&mut self) -> bool {
        std::mem::take(&mut self.fed)
    }

    /// Key press, click, scroll, resize, modal open: hide until the pointer
    /// moves again.
    pub(crate) fn dismiss(&mut self) {
        self.suppressed = true;
    }

    /// Focus lost: forget the target too.
    pub(crate) fn reset(&mut self) {
        self.target = None;
        self.since = None;
        self.suppressed = true;
    }

    /// True when a tooltip for `key` may be painted right now.
    pub(crate) fn visible(&mut self, key: TooltipKey, now: Instant, blocked: bool) -> bool {
        if blocked {
            // A modal opening counts as a dismissal: the tooltip must not
            // pop back the instant the modal closes.
            self.suppressed = true;
            return false;
        }
        !self.suppressed
            && self.target == Some(key)
            && self.since.is_some_and(|s| now.duration_since(s) >= DWELL)
    }

    /// Call once per frame after painting: records what was shown and, when
    /// a tooltip that was on screen is gone, remembers its rect for repaint.
    pub(crate) fn end_frame(&mut self, painted: Option<Rect>) {
        if let Some(prev) = self.shown {
            if painted != Some(prev) {
                self.repaint = Some(prev);
            }
        }
        self.shown = painted;
    }

    /// When the dwell on the current target ends and the tooltip may appear,
    /// if that is still ahead: the render loop wakes for it.
    pub(crate) fn next_deadline(&self, now: Instant) -> Option<Instant> {
        if self.suppressed || self.target.is_none() {
            return None;
        }
        let due = self.since? + DWELL;
        (due > now).then_some(due)
    }

    /// The rect a just-hidden tooltip covered, once.
    pub(crate) fn take_repaint(&mut self) -> Option<Rect> {
        self.repaint.take()
    }

    /// Whether a tooltip is currently on screen.
    #[cfg(test)]
    pub(crate) fn is_shown(&self) -> bool {
        self.shown.is_some()
    }
}

/// Any input mode but `Normal` (Confirm, palette, tab picker, find bars,
/// reword, rebase, leader/navigation prefixes...) is modal to tooltips.
pub(crate) fn blocked_by_mode(mode: &crate::modes::InputMode) -> bool {
    !matches!(mode, crate::modes::InputMode::Normal)
}

/// Paints one bordered tooltip box anchored at (`col`, `row`) inside
/// `bounds`, fully `Clear`ed first. Returns the rect it covered.
pub(crate) fn paint_box(
    frame: &mut Frame,
    bounds: Rect,
    col: u16,
    row: u16,
    lines: Vec<Line<'static>>,
) -> Rect {
    let width = lines
        .iter()
        .map(|l| l.width() as u16)
        .max()
        .unwrap_or(20)
        .min(bounds.width.saturating_sub(4))
        .max(20.min(bounds.width.saturating_sub(4)))
        + 2;
    let height = (lines.len() as u16 + 2).min(bounds.height.saturating_sub(2));
    let popup = anchored_popup(bounds, col, row, width, height);
    frame.render_clipped(Clear, popup);
    frame.render_clipped(
        Paragraph::new(lines).block(
            Block::default()
                .border_type(BorderType::Rounded)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(th().ui.accent))
                .style(Style::default().bg(th().ui.panel)),
        ),
        popup,
    );
    popup
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    fn key() -> TooltipKey {
        TooltipKey::Commit(Oid([1; 20]), LogColumn::Message)
    }

    #[test]
    fn needs_dwell_and_matching_target() {
        let t0 = Instant::now();
        let mut t = TooltipLifecycle::default();
        t.pointer_moved(Some(key()), (5, 5), t0);
        assert!(!t.visible(key(), t0, false));
        assert!(t.visible(key(), t0 + DWELL, false));
        assert!(!t.visible(TooltipKey::LinkedPrChip, t0 + DWELL, false));
    }

    #[test]
    fn leave_key_scroll_and_modal_hide_until_the_pointer_moves() {
        let t0 = Instant::now();
        let late = t0 + DWELL * 2;
        let mut t = TooltipLifecycle::default();
        t.pointer_moved(Some(key()), (5, 5), t0);
        assert!(t.visible(key(), late, false));
        t.dismiss(); // key press / scroll / click / resize
        assert!(!t.visible(key(), late, false));
        t.pointer_moved(Some(key()), (5, 5), late); // no real movement
        assert!(!t.visible(key(), late + DWELL * 4, false));
        t.pointer_moved(Some(key()), (6, 5), late); // moved: dwell restarts
        assert!(!t.visible(key(), late, false));
        assert!(t.visible(key(), late + DWELL, false));
        // Leaving the target.
        t.pointer_moved(None, (0, 0), late);
        assert!(!t.visible(key(), late + DWELL * 2, false));
        // A modal hides it, and it stays hidden after the modal closes.
        t.pointer_moved(Some(key()), (5, 5), late);
        assert!(!t.visible(key(), late + DWELL, true));
        assert!(!t.visible(key(), late + DWELL * 3, false));
        // Focus loss forgets the target.
        t.pointer_moved(Some(key()), (9, 9), late);
        t.reset();
        assert!(!t.visible(key(), late + DWELL * 5, false));
    }

    #[test]
    fn confirm_and_palette_modes_block_tooltips() {
        use crate::modes::{ConfirmAction, InputMode};
        assert!(!blocked_by_mode(&InputMode::Normal));
        let confirm = InputMode::Confirm {
            prompt: "Close the tab for repo?".into(),
            action: ConfirmAction::CloseTab(0),
        };
        assert!(blocked_by_mode(&confirm));
        let palette = InputMode::CommandPalette {
            draft: String::new(),
            selected: 0,
        };
        assert!(blocked_by_mode(&palette));
        let tabs = InputMode::TabPicker {
            draft: String::new(),
            selected: 0,
        };
        assert!(blocked_by_mode(&tabs));
        // And a blocked mode really keeps the tooltip off a dwelled target.
        let t0 = Instant::now();
        let mut life = TooltipLifecycle::default();
        life.pointer_moved(Some(key()), (3, 2), t0);
        assert!(!life.visible(key(), t0 + DWELL * 2, blocked_by_mode(&confirm)));
    }

    fn base(frame: &mut Frame) {
        let area = frame.area();
        let text = "x".repeat(area.width as usize);
        let lines: Vec<Line> = (0..area.height).map(|_| Line::from(text.clone())).collect();
        frame.render_widget(Paragraph::new(lines), area);
    }

    /// Draws the base UI then the tooltip pass exactly as the event loop
    /// does (the modal state decides via `blocked`).
    fn draw(
        term: &mut Terminal<TestBackend>,
        life: &mut TooltipLifecycle,
        now: Instant,
        blocked: bool,
    ) {
        term.draw(|frame| {
            base(frame);
            let painted = if life.visible(key(), now, blocked) {
                Some(paint_box(
                    frame,
                    frame.area(),
                    3,
                    2,
                    vec![Line::from("a long commit message")],
                ))
            } else {
                None
            };
            life.end_frame(painted);
        })
        .unwrap();
    }

    #[test]
    fn a_modal_open_paints_no_tooltip_cells_and_hiding_restores_the_plain_render() {
        let t0 = Instant::now();
        let late = t0 + DWELL * 2;
        let mut life = TooltipLifecycle::default();
        life.pointer_moved(Some(key()), (3, 2), t0);

        let mut plain = Terminal::new(TestBackend::new(40, 10)).unwrap();
        draw(&mut plain, &mut TooltipLifecycle::default(), late, false);

        let mut live = Terminal::new(TestBackend::new(40, 10)).unwrap();
        draw(&mut live, &mut life, late, false);
        assert!(life.is_shown());
        assert_ne!(live.backend().buffer(), plain.backend().buffer());

        // A modal opens: the very next frame carries no tooltip cells.
        draw(&mut live, &mut life, late, true);
        assert!(!life.is_shown());
        assert_eq!(live.backend().buffer(), plain.backend().buffer());
        // The hide reported the rect to force-repaint, exactly once.
        assert!(life.take_repaint().is_some());
        assert!(life.take_repaint().is_none());

        // Modal closed: stays hidden (no pop-back) and the rect is clean.
        draw(&mut live, &mut life, late + DWELL, false);
        assert_eq!(live.backend().buffer(), plain.backend().buffer());
    }

    /// A button's tooltip carries the effective binding of its key twin,
    /// and follows a rebinding.
    #[test]
    fn tooltips_show_the_effective_key() {
        use crate::chrome::HitTarget;
        let t = hint_text(&HitTarget::StatusExplainButton).unwrap();
        assert_eq!(t, "Explain working changes (Ctrl+G w)");
        assert_eq!(
            hint_text(&HitTarget::AiGenerate).unwrap(),
            "Generate a commit message with AI (Ctrl+G i)"
        );
        let km = crate::keymap::test_keymap(&[("git_prefix", "ai_explain", &["z"])]);
        crate::keymap::with_test_keymap(km, || {
            assert_eq!(
                hint_text(&HitTarget::StatusExplainButton).unwrap(),
                "Explain working changes (Ctrl+G z)"
            );
            assert!(crate::diff_view::DiffBarButton::NextHunk
                .tooltip()
                .ends_with("(n)"));
        });
        let km = crate::keymap::test_keymap(&[("diff", "next_hunk", &["]"])]);
        crate::keymap::with_test_keymap(km, || {
            assert_eq!(
                crate::diff_view::DiffBarButton::NextHunk.tooltip(),
                "Next hunk (])"
            );
        });
        // The title bar's Search button names the global search key.
        assert_eq!(
            hint_text(&HitTarget::GlobalSearchButton).unwrap(),
            "Search every open repository (Ctrl+Alt+F)"
        );
        // The commit column's strip names its toggle key.
        assert_eq!(
            hint_text(&HitTarget::CommitColumn(
                crate::commit_column::ColumnHit::Reopen
            ))
            .unwrap(),
            "Show the commit column (Alt+C)"
        );
        // A button without a key twin gets no suffix.
        assert_eq!(
            hint_text(&HitTarget::CommitReference(0)).unwrap(),
            "Open in your browser (palette: Open Commit Reference)"
        );
    }
}
