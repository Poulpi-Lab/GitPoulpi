//! The strip on the Log's border while a grouped push is paused on the
//! conflicts of one remote. The second banner of the Log's borders, drawn by
//! the one strip painter (`operation_banner::paint_strip`).
//!
//! Continue is always there (owner decision): greyed while the daemon says
//! the resolution is not concluded (`PendingGroup::ready_to_continue`), and
//! always pressed by hand: nothing continues on its own. The strip's words
//! say what is missing. Abort group aborts the merge or rebase the group
//! started and drops the saved run (pushes already done stay).

use std::path::Path;

use gitpoulpi_core::domain::GroupMode;
use gitpoulpi_i18n::msg::push_group as msg;
use ratatui::layout::Rect;
use ratatui::Frame;

use super::{with, Effect};
use crate::modes::PendingDestructive;
use crate::operation_banner::{paint_strip, StripButton as Button};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StripButton {
    Continue,
    Abort,
}

/// Whether a paused group of the repository at `root` wants the strip.
pub(crate) fn visible(root: &Path) -> bool {
    super::CORE.lock().ok().is_some_and(|c| {
        c.as_ref()
            .is_some_and(|c| c.paused.as_ref().is_some_and(|p| p.root == root))
    })
}

/// The words of the strip and whether Continue is on.
pub(crate) fn text(root: &Path) -> Option<(String, bool)> {
    with(|c| {
        c.paused.as_ref().filter(|p| p.root == root).map(|p| {
            (
                match (p.mode, p.ready) {
                    (GroupMode::Push, true) => msg::strip_ready(&p.remote),
                    (GroupMode::Push, false) => msg::strip_waiting(&p.remote),
                    (GroupMode::Pull, true) => msg::strip_ready_pull(&p.remote),
                    (GroupMode::Pull, false) => msg::strip_waiting_pull(&p.remote),
                },
                p.ready,
            )
        })
    })
}

/// Paints the strip on `area` and returns its buttons' rects.
pub(crate) fn render(
    frame: &mut Frame,
    area: Rect,
    root: &Path,
    hovered: Option<StripButton>,
) -> Vec<(Rect, StripButton)> {
    let Some((text, ready)) = text(root) else {
        return Vec::new();
    };
    let list: [(StripButton, &str); 2] = [
        (StripButton::Continue, msg::STRIP_CONTINUE.get()),
        (StripButton::Abort, msg::STRIP_ABORT.get()),
    ];
    let buttons: Vec<Button<'_>> = list
        .iter()
        .map(|(b, label)| Button {
            label,
            danger: *b == StripButton::Abort,
            hovered: hovered == Some(*b),
            disabled: *b == StripButton::Continue && !ready,
        })
        .collect();
    paint_strip(frame, area, &[text], &buttons)
        .into_iter()
        .map(|(rect, i)| (rect, list[i].0))
        .collect()
}

/// A press on a strip button.
pub(crate) fn click(root: &Path, button: StripButton) -> Effect {
    let Some(paused) = with(|c| c.paused.clone()).filter(|p| p.root == root) else {
        return Effect::None;
    };
    match button {
        StripButton::Continue => {
            if !paused.ready {
                return Effect::None;
            }
            Effect::Submit(vec![crate::daemon_link::Action::RemoteGroupContinue {
                expect_run_id: Some(paused.run_id),
                targets: Vec::new(),
                prefer_rebase: paused.prefer_rebase,
            }])
        }
        StripButton::Abort => Effect::Confirm {
            prompt: msg::abort_confirm(&paused.branch, &paused.remote),
            pending: PendingDestructive::GroupAbort {
                run_id: paused.run_id,
            },
        },
    }
}
