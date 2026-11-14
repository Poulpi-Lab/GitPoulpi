//! The strip drawn over the Log's top row while a repository has a pending
//! auto-stash: GitPoulpi stashed local changes to run an operation and has
//! not put them back (a conflicted pop, or a crash in between). "Pop now",
//! "Show stash", "Start over" and "Drop" are its buttons; each has a
//! palette command (`AutoStashPop`, `AutoStashShow`, `AutoStashStartOver`,
//! `AutoStashDrop`).
//!
//! The record lives in the repo's `repo-state.toml`, written by the daemon;
//! the window reads it at open and again on `Event::PendingAutoStashChanged`
//! (see `Activity::mark_pending_auto_stash_changed`).

use gitpoulpi_core::domain::OperationKind;
use gitpoulpi_session::{PendingAutoStash, PendingAutoStashState, RepoState};

use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AutoStashBannerButton {
    /// Open the merge editor (only while the pop left conflicts).
    Resolve,
    Pop,
    /// After a conflicted pop: keep the working tree, drop the stash.
    Finish,
    Show,
    Drop,
    /// Discard the stash's files' changes, undo the operation, pop again.
    StartOver,
}

/// The pending auto-stash record of the repository at `repo_root`, if any.
pub(crate) fn load(repo_root: &std::path::Path) -> Option<PendingAutoStash> {
    RepoState::load(&crate::remotes_ui::repo_state_path(repo_root))
        .ok()
        .and_then(|s| s.pending_auto_stash)
}

fn reason_word(reason: OperationKind) -> &'static str {
    use gitpoulpi_i18n::msg::autostash as m;
    match reason {
        OperationKind::Checkout => m::REASON_CHECKOUT,
        OperationKind::Merge => m::REASON_MERGE,
        OperationKind::Rebase => m::REASON_REBASE,
        OperationKind::Pull => m::REASON_PULL,
        _ => m::REASON_OPERATION,
    }
    .get()
}

pub(crate) fn banner_text(pending: &PendingAutoStash) -> String {
    let word = reason_word(pending.reason);
    let id = pending.stash_id.0;
    match pending.state {
        PendingAutoStashState::Conflicted => {
            gitpoulpi_i18n::msg::autostash::banner_conflicted(word, id)
        }
        PendingAutoStashState::PopFailed => {
            let text = gitpoulpi_i18n::msg::autostash::banner_pop_failed(word, id);
            match pending
                .error
                .as_deref()
                .map(str::trim)
                .filter(|e| !e.is_empty())
            {
                // git's own words, first line only: the strip is one row.
                Some(reason) => format!("{text} ({})", reason.lines().next().unwrap_or(reason)),
                None => text,
            }
        }
        PendingAutoStashState::Applied => gitpoulpi_i18n::msg::autostash::banner_applied(word, id),
    }
}

pub(crate) fn render(
    frame: &mut Frame,
    area: Rect,
    pending: &PendingAutoStash,
    hovered: Option<AutoStashBannerButton>,
) -> Vec<(Rect, AutoStashBannerButton)> {
    let resolve = gitpoulpi_i18n::msg::merge::BANNER_RESOLVE.get();
    let conflicted = pending.state == PendingAutoStashState::Conflicted;
    let pop_label = if pending.state == PendingAutoStashState::PopFailed {
        gitpoulpi_i18n::msg::autostash::RETRY_POP.get()
    } else {
        gitpoulpi_i18n::msg::autostash::POP_NOW.get()
    };
    let mut list = vec![
        (AutoStashBannerButton::Pop, pop_label),
        (
            AutoStashBannerButton::Show,
            gitpoulpi_i18n::msg::autostash::SHOW_STASH.get(),
        ),
        (
            AutoStashBannerButton::StartOver,
            gitpoulpi_i18n::msg::autostash::START_OVER.get(),
        ),
        (
            AutoStashBannerButton::Drop,
            gitpoulpi_i18n::msg::autostash::DROP.get(),
        ),
    ];
    if conflicted {
        // Pop stays: after a reset to where the stash applies cleanly it is
        // the way to start over.
        list.insert(
            0,
            (
                AutoStashBannerButton::Finish,
                gitpoulpi_i18n::msg::autostash::FINISH.get(),
            ),
        );
        list.insert(0, (AutoStashBannerButton::Resolve, resolve));
    }
    let strip: Vec<crate::operation_banner::StripButton<'_>> = list
        .iter()
        .map(|(b, label)| crate::operation_banner::StripButton {
            label,
            danger: *b == AutoStashBannerButton::Drop,
            hovered: hovered == Some(*b),
            disabled: false,
        })
        .collect();
    crate::operation_banner::paint_strip(frame, area, &[banner_text(pending)], &strip)
        .into_iter()
        .map(|(rect, i)| (rect, list[i].0))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use gitpoulpi_core::domain::StashId;

    fn pending(state: PendingAutoStashState) -> PendingAutoStash {
        PendingAutoStash {
            stash_id: StashId(0),
            stash_commit: None,
            reason: OperationKind::Pull,
            created_at: 0,
            state,
            error: None,
            after_operation: false,
        }
    }

    #[test]
    fn the_text_says_what_happened_and_where_the_changes_are() {
        let t = banner_text(&pending(PendingAutoStashState::Conflicted));
        assert!(t.contains("pull") && t.contains("stash@{0}") && t.contains("conflicted"));
        let t = banner_text(&pending(PendingAutoStashState::Applied));
        assert!(t.contains("not restored"));
    }

    #[test]
    fn a_failed_pop_says_nothing_was_restored_with_git_s_reason_and_offers_a_retry() {
        let mut p = pending(PendingAutoStashState::PopFailed);
        p.error = Some("u.txt already exists, no checkout\nsecond line".into());
        let t = banner_text(&p);
        assert!(t.contains("failed") && t.contains("stash@{0}"), "{t}");
        assert!(
            t.contains("u.txt already exists") && !t.contains("second line"),
            "{t}"
        );
        use ratatui::backend::TestBackend;
        use ratatui::Terminal;
        let mut terminal = Terminal::new(TestBackend::new(200, 2)).unwrap();
        terminal
            .draw(|frame| {
                render(frame, frame.area(), &p, None);
            })
            .unwrap();
        let text: String = terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|c| c.symbol())
            .collect();
        assert!(text.contains("Retry pop") && !text.contains("Pop now"));
    }

    #[test]
    fn render_draws_all_three_buttons_with_hit_rects() {
        use ratatui::backend::TestBackend;
        use ratatui::Terminal;
        let mut terminal = Terminal::new(TestBackend::new(120, 2)).unwrap();
        let p = pending(PendingAutoStashState::Conflicted);
        let mut hits = Vec::new();
        terminal
            .draw(|frame| hits = render(frame, frame.area(), &p, None))
            .unwrap();
        let text: String = terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|c| c.symbol())
            .collect();
        for label in [
            "Resolve conflicts",
            "Keep result",
            "Pop now",
            "Show stash",
            "Start over",
            "Drop",
        ] {
            assert!(text.contains(label), "{label} missing");
        }
        assert_eq!(hits.len(), 6);
    }
}
