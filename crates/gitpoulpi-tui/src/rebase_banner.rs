//! The one-row strip drawn above the Log pane whenever
//! `RefreshOutcome::rebase_state_result` reports a rebase in progress:
//! "Rebasing <branch> onto <short onto> · step/total · stopped at <short>
//! (conflict|edit)" plus Continue/Skip/Abort buttons.
//!
//! Purely a function of a `RebaseState` -- no daemon access, no mutable
//! state of its own -- so it is unit tested the same way `reword_ui`'s
//! pure helpers are, without a terminal.

use gitpoulpi_core::domain::RebaseState;

use super::*;

/// The banner's three buttons, in the order they are drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RebaseBannerButton {
    /// Open the merge editor (only while the rebase stopped on a conflict).
    Resolve,
    Continue,
    Skip,
    Abort,
}

/// The banner's text, independent of layout - what `render` prints and
/// what a test can assert on without a `Buffer`.
pub(crate) fn banner_text(state: &RebaseState) -> String {
    let head = state.head_name.as_deref().unwrap_or("HEAD");
    let onto = state.onto.to_hex()[..SHORT_SHA_LEN].to_string();
    let stopped = state
        .stopped_at
        .map(|at| at.to_hex()[..SHORT_SHA_LEN].to_string());
    let kind = if state.conflicted {
        gitpoulpi_i18n::msg::rebase_banner::KIND_CONFLICT
    } else {
        gitpoulpi_i18n::msg::rebase_banner::KIND_EDIT
    };
    match stopped {
        Some(at) => gitpoulpi_i18n::msg::rebase_banner::stopped(
            head,
            onto,
            state.step,
            state.total,
            at,
            kind,
        ),
        None => gitpoulpi_i18n::msg::rebase_banner::running(head, onto, state.step, state.total),
    }
}

/// Draws the banner at the top of `area` (a single row) and returns its
/// button hit rects. `area` is expected to be exactly one row tall; the
/// caller (`event_loop.rs`) carves it off the top of the Log pane's rect
/// before laying out the table, the same way the commit filter field
/// does.
pub(crate) fn render(
    frame: &mut Frame,
    area: Rect,
    state: &RebaseState,
    hovered: Option<RebaseBannerButton>,
) -> Vec<(Rect, RebaseBannerButton)> {
    let resolve = gitpoulpi_i18n::msg::merge::BANNER_RESOLVE.get();
    let mut list = vec![
        (
            RebaseBannerButton::Continue,
            gitpoulpi_i18n::msg::merge::BANNER_CONTINUE.get(),
        ),
        (
            RebaseBannerButton::Skip,
            gitpoulpi_i18n::msg::rebase_banner::SKIP.get(),
        ),
        (
            RebaseBannerButton::Abort,
            gitpoulpi_i18n::msg::merge::BANNER_ABORT.get(),
        ),
    ];
    if state.conflicted {
        list.insert(0, (RebaseBannerButton::Resolve, resolve));
    }
    let strip: Vec<crate::operation_banner::StripButton<'_>> = list
        .iter()
        .map(|(b, label)| crate::operation_banner::StripButton {
            label,
            danger: *b == RebaseBannerButton::Abort,
            hovered: hovered == Some(*b),
            disabled: false,
        })
        .collect();
    crate::operation_banner::paint_strip(frame, area, &[banner_text(state)], &strip)
        .into_iter()
        .map(|(rect, i)| (rect, list[i].0))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::*;

    fn state(conflicted: bool, stopped: Option<u8>) -> RebaseState {
        RebaseState {
            head_name: Some("feature".into()),
            onto: oid(1),
            stopped_at: stopped.map(oid),
            step: 2,
            total: 5,
            conflicted,
            interactive: true,
        }
    }

    #[test]
    fn the_text_names_the_branch_the_step_and_a_stop_reason() {
        let s = state(true, Some(9));
        let text = banner_text(&s);
        assert!(text.contains("feature"), "{text}");
        assert!(text.contains("2/5"), "{text}");
        assert!(text.contains("conflict"), "{text}");

        let edit = state(false, Some(9));
        assert!(banner_text(&edit).contains("edit"));

        let running = state(false, None);
        assert!(!banner_text(&running).contains("stopped"));
    }

    #[test]
    fn render_draws_all_three_buttons() {
        use ratatui::backend::TestBackend;
        use ratatui::Terminal;
        let mut terminal = Terminal::new(TestBackend::new(100, 3)).unwrap();
        let s = state(true, Some(9));
        let mut hits = Vec::new();
        terminal
            .draw(|frame| hits = render(frame, frame.area(), &s, None))
            .unwrap();
        for b in [
            RebaseBannerButton::Continue,
            RebaseBannerButton::Skip,
            RebaseBannerButton::Abort,
        ] {
            assert!(hits.iter().any(|(_, h)| *h == b), "no hit for {b:?}");
        }
    }

    /// Hovering a button lights it with the app-wide strong hover style.
    #[test]
    fn a_hovered_button_takes_the_strong_hover_fill() {
        use ratatui::backend::TestBackend;
        use ratatui::Terminal;
        let s = state(true, Some(9));
        let paint = |hovered| {
            let mut terminal = Terminal::new(TestBackend::new(100, 3)).unwrap();
            let mut hits = Vec::new();
            terminal
                .draw(|frame| hits = render(frame, frame.area(), &s, hovered))
                .unwrap();
            let rect = hits
                .iter()
                .find(|(_, b)| *b == RebaseBannerButton::Skip)
                .unwrap()
                .0;
            terminal.backend().buffer()[(rect.x, rect.y)].bg
        };
        assert_ne!(paint(None), th().ui.button_hover);
        assert_eq!(paint(Some(RebaseBannerButton::Skip)), th().ui.button_hover);
    }
}
