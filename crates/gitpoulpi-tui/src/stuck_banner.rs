//! The two strips of the "stuck states" slice (plan `stuck-states.md`, S8):
//! a `git am` stopped on a patch (Resolve / Continue / Skip / Abort), and a
//! stale `.git/index.lock` that makes every write fail (Remove). Same
//! painter and the same hit-test path as `rebase_banner`; each button has a
//! palette twin (`PaletteCommand::AmContinue` ...).
//!
//! Pure functions of an `OperationState::Am` / an `IndexLock`: unit tested
//! without a terminal.

use gitpoulpi_core::domain::{IndexLock, OperationState};
use gitpoulpi_i18n::msg::stuck_states as tr;

use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StuckBannerButton {
    /// Open the merge editor on the first conflicted file of the patch.
    AmResolve,
    AmContinue,
    AmSkip,
    AmAbort,
    /// Delete the stale `index.lock` (asks first).
    RemoveLock,
}

/// The `git am` banner's text.
pub(crate) fn am_text(state: &OperationState) -> String {
    match state {
        OperationState::Am { step, total } => tr::am_banner(*step, *total),
        _ => String::new(),
    }
}

/// The stale-lock banner's text.
pub(crate) fn lock_text(lock: &IndexLock) -> String {
    tr::lock_banner(lock.age_secs)
}

/// The `git am` strip; `conflicts` > 0 adds Resolve in front.
pub(crate) fn render_am(
    frame: &mut Frame,
    area: Rect,
    state: &OperationState,
    conflicts: usize,
    hovered: Option<StuckBannerButton>,
) -> Vec<(Rect, StuckBannerButton)> {
    let mut list = vec![
        (
            StuckBannerButton::AmContinue,
            gitpoulpi_i18n::msg::merge::BANNER_CONTINUE.get(),
        ),
        (
            StuckBannerButton::AmSkip,
            gitpoulpi_i18n::msg::rebase_banner::SKIP.get(),
        ),
        (
            StuckBannerButton::AmAbort,
            gitpoulpi_i18n::msg::merge::BANNER_ABORT.get(),
        ),
    ];
    if conflicts > 0 {
        list.insert(
            0,
            (
                StuckBannerButton::AmResolve,
                gitpoulpi_i18n::msg::merge::BANNER_RESOLVE.get(),
            ),
        );
    }
    paint(frame, area, am_text(state), list, hovered)
}

/// The stale-lock strip: only offered once the lock is stale.
pub(crate) fn render_lock(
    frame: &mut Frame,
    area: Rect,
    lock: &IndexLock,
    hovered: Option<StuckBannerButton>,
) -> Vec<(Rect, StuckBannerButton)> {
    let list = vec![(StuckBannerButton::RemoveLock, tr::LOCK_REMOVE.get())];
    paint(frame, area, lock_text(lock), list, hovered)
}

fn paint(
    frame: &mut Frame,
    area: Rect,
    text: String,
    list: Vec<(StuckBannerButton, &'static str)>,
    hovered: Option<StuckBannerButton>,
) -> Vec<(Rect, StuckBannerButton)> {
    let strip: Vec<crate::operation_banner::StripButton<'_>> = list
        .iter()
        .map(|(b, label)| crate::operation_banner::StripButton {
            label,
            danger: matches!(
                b,
                StuckBannerButton::AmAbort | StuckBannerButton::RemoveLock
            ),
            hovered: hovered == Some(*b),
            disabled: false,
        })
        .collect();
    crate::operation_banner::paint_strip(frame, area, &[text], &strip)
        .into_iter()
        .map(|(rect, i)| (rect, list[i].0))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    fn am() -> OperationState {
        OperationState::Am { step: 2, total: 3 }
    }

    fn lock() -> IndexLock {
        IndexLock {
            path: ".git/index.lock".into(),
            age_secs: 42,
            stale: true,
        }
    }

    #[test]
    fn the_am_text_names_the_patch_and_the_count() {
        let text = am_text(&am());
        assert!(text.contains("git am") && text.contains("2") && text.contains("3"));
    }

    #[test]
    fn the_lock_text_says_how_old_it_is() {
        assert!(lock_text(&lock()).contains("42"));
    }

    #[test]
    fn the_am_strip_draws_continue_skip_abort_and_resolve_on_conflicts() {
        let hits = |conflicts| {
            let mut terminal = Terminal::new(TestBackend::new(110, 2)).unwrap();
            let mut hits = Vec::new();
            terminal
                .draw(|f| hits = render_am(f, f.area(), &am(), conflicts, None))
                .unwrap();
            hits.into_iter().map(|(_, b)| b).collect::<Vec<_>>()
        };
        assert_eq!(
            hits(0),
            [
                StuckBannerButton::AmContinue,
                StuckBannerButton::AmSkip,
                StuckBannerButton::AmAbort
            ]
        );
        assert_eq!(hits(2)[0], StuckBannerButton::AmResolve);
    }

    #[test]
    fn the_lock_strip_draws_one_remove_button_and_it_lights_on_hover() {
        let paint = |hovered| {
            let mut terminal = Terminal::new(TestBackend::new(110, 2)).unwrap();
            let mut hits = Vec::new();
            terminal
                .draw(|f| hits = render_lock(f, f.area(), &lock(), hovered))
                .unwrap();
            assert_eq!(hits.len(), 1);
            terminal.backend().buffer()[(hits[0].0.x, hits[0].0.y)].bg
        };
        assert_ne!(paint(None), th().ui.button_hover);
        assert_eq!(
            paint(Some(StuckBannerButton::RemoveLock)),
            th().ui.button_hover
        );
    }
}
