//! The one-row strip drawn above the Log pane while a merge, a
//! cherry-pick or a revert is stopped (a rebase keeps `rebase_banner`,
//! which gained the same Resolve button): what is going on, how many
//! files still have conflicts, and the way forward -- Resolve conflicts
//! (opens the merge editor), Commit (or Continue) once nothing is left,
//! and Abort, which asks first.
//!
//! Purely a function of an `OperationState` and a conflict count -- no
//! daemon access, no state of its own -- so it is unit tested without a
//! terminal, as `rebase_banner` is.

use gitpoulpi_core::domain::OperationState;
use gitpoulpi_i18n::msg::merge as tr;

use super::*;

/// The banner's buttons, in the order they are drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OperationBannerButton {
    /// Open the merge editor on the first conflicted file.
    Resolve,
    /// Commit the merge (or continue the cherry-pick / revert): both are
    /// the commit, which concludes them.
    Commit,
    /// Give up on a cherry-pick or revert (empty because the change is
    /// already applied, or not worth resolving).
    Skip,
    Abort,
}

/// Whether `state` has a banner of this kind (the rebase has its own).
pub(crate) fn applies(state: &OperationState) -> bool {
    // A rebase and a `git am` keep their own strips.
    !matches!(state, OperationState::Rebase(_) | OperationState::Am { .. })
}

fn short(oid: gitpoulpi_core::domain::Oid) -> String {
    oid.to_hex()[..SHORT_SHA_LEN].to_string()
}

/// The banner's text, independent of layout.
pub(crate) fn banner_text(state: &OperationState, conflicts: usize) -> String {
    let head = match state {
        OperationState::Merge {
            head_name: Some(head),
            their_label,
        } => tr::banner_merge(their_label, head),
        OperationState::Merge { their_label, .. } => tr::banner_merge_detached(their_label),
        OperationState::CherryPick { commit } => tr::banner_pick(short(*commit)),
        OperationState::Revert { commit } => tr::banner_revert(short(*commit)),
        OperationState::Rebase(_) | OperationState::Am { .. } => String::new(),
    };
    let tail = if conflicts == 0 {
        tr::BANNER_ALL_RESOLVED.get().to_string()
    } else {
        tr::banner_files(conflicts as u64)
    };
    format!("{head} \u{b7} {tail}")
}

/// The text at every length it can be drawn at, longest first: names the
/// operation and what is left, then only the operation and what is left.
pub(crate) fn banner_texts(state: &OperationState, conflicts: usize) -> Vec<String> {
    let tail = if conflicts == 0 {
        tr::BANNER_ALL_RESOLVED.get().to_string()
    } else {
        tr::banner_files(conflicts as u64)
    };
    let name = match state {
        OperationState::Merge { .. } => tr::BANNER_MERGE_SHORT.get().to_string(),
        OperationState::CherryPick { commit } => tr::banner_pick(short(*commit)),
        OperationState::Revert { commit } => tr::banner_revert(short(*commit)),
        OperationState::Rebase(_) | OperationState::Am { .. } => String::new(),
    };
    vec![
        banner_text(state, conflicts),
        format!("{name} \u{b7} {tail}"),
        name,
    ]
}

/// The buttons that make sense now: Resolve while files are conflicted,
/// the commit (or Continue) once none is, Skip for a cherry-pick or
/// revert, and always Abort.
pub(crate) fn buttons(
    state: &OperationState,
    conflicts: usize,
) -> Vec<(OperationBannerButton, String)> {
    let mut out = Vec::new();
    if conflicts > 0 {
        out.push((
            OperationBannerButton::Resolve,
            tr::BANNER_RESOLVE.get().to_string(),
        ));
    } else {
        out.push((
            OperationBannerButton::Commit,
            match state {
                OperationState::Merge { .. } => tr::BANNER_COMMIT_MERGE.get(),
                _ => tr::BANNER_CONTINUE.get(),
            }
            .to_string(),
        ));
    }
    if matches!(
        state,
        OperationState::CherryPick { .. } | OperationState::Revert { .. }
    ) {
        out.push((
            OperationBannerButton::Skip,
            tr::BANNER_SKIP.get().to_string(),
        ));
    }
    out.push((
        OperationBannerButton::Abort,
        match state {
            OperationState::Merge { .. } => tr::BANNER_ABORT_MERGE.get(),
            _ => tr::BANNER_ABORT.get(),
        }
        .to_string(),
    ));
    out
}

/// One button of a banner strip.
pub(crate) struct StripButton<'a> {
    pub label: &'a str,
    /// Destructive (Abort): the red fill.
    pub danger: bool,
    pub hovered: bool,
}

/// The room the banner's words keep before a button is dropped to make some.
const MIN_TEXT_ROOM: u16 = 24;

/// Paints a one-row banner on `area`'s first row: the first of `texts` that
/// fits on the left (the last one, truncated with an ellipsis, when none
/// does), never reaching the buttons (one space is kept between them), the
/// buttons right-aligned. The row is cleared first: it replaces the Log's
/// border line, whose glyphs would show through the gaps. Buttons that do not
/// fit (leaving [`MIN_TEXT_ROOM`] for the words) are dropped from the left, so the last one (Abort, Drop) stays.
/// Returns the rect of each button drawn, in order, with its index in
/// `buttons`. Shared by the operation, rebase and auto-stash banners.
pub(crate) fn paint_strip(
    frame: &mut Frame,
    area: Rect,
    texts: &[String],
    buttons: &[StripButton<'_>],
) -> Vec<(Rect, usize)> {
    if area.height == 0 {
        return Vec::new();
    }
    let row = Rect::new(area.x, area.y, area.width, 1);
    frame.render_widget(crate::wide_edge::Clear, row);
    frame.render_clipped(
        Block::default().style(Style::default().bg(th().chrome.banner_bg)),
        row,
    );
    let width_of = |b: &StripButton<'_>| display_width(&format!(" {} ", b.label)) as u16 + 1;
    let mut first = 0;
    while first + 1 < buttons.len()
        && buttons[first..].iter().map(width_of).sum::<u16>() + MIN_TEXT_ROOM > row.width
    {
        first += 1;
    }
    let shown = &buttons[first..];
    let buttons_width: u16 = shown.iter().map(width_of).sum();
    // One column of padding on the left, one space before the buttons.
    let text_room = row.width.saturating_sub(buttons_width + 2);
    let text = match texts
        .iter()
        .find(|t| display_width(t) <= usize::from(text_room))
    {
        Some(fits) => fits.clone(),
        None => truncate_with_ellipsis(
            texts.last().map_or("", String::as_str),
            usize::from(text_room),
        ),
    };
    frame.render_clipped(
        Paragraph::new(Span::styled(text, Style::default().fg(th().status.warning))),
        Rect::new(row.x + 1, row.y, text_room, 1),
    );
    let mut hits = Vec::new();
    let mut x = row.x + row.width.saturating_sub(buttons_width);
    for (i, b) in shown.iter().enumerate() {
        let label = format!(" {} ", b.label);
        let w = display_width(&label) as u16;
        let style = if b.hovered {
            crate::theme::plain_button_style(true)
        } else {
            Style::default().fg(th().ui.on_button).bg(if b.danger {
                th().chrome.banner_danger_bg
            } else {
                th().ui.button
            })
        };
        let rect = Rect::new(x, row.y, w, 1);
        frame.render_clipped(Paragraph::new(Span::styled(label, style)), rect);
        hits.push((rect, first + i));
        x += w + 1;
    }
    hits
}

/// Where the `slot`th banner goes on the Log pane `log`: the first on its
/// top border row, a second one on its bottom border row. Never the
/// column-title row below the top border, which the Log needs, and never
/// a content row (the Log's row arithmetic stays untouched).
pub(crate) fn strip_rect(log: Rect, slot: usize) -> Option<Rect> {
    match slot {
        0 if log.height > 0 => Some(Rect::new(log.x, log.y, log.width, 1)),
        1 if log.height >= 5 => Some(Rect::new(log.x, log.bottom() - 1, log.width, 1)),
        _ => None,
    }
}

/// Draws the banner on `area`'s first row and returns its button rects.
pub(crate) fn render(
    frame: &mut Frame,
    area: Rect,
    state: &OperationState,
    conflicts: usize,
    hovered: Option<OperationBannerButton>,
) -> Vec<(Rect, OperationBannerButton)> {
    let list = buttons(state, conflicts);
    let strip: Vec<StripButton<'_>> = list
        .iter()
        .map(|(b, label)| StripButton {
            label,
            danger: *b == OperationBannerButton::Abort,
            hovered: hovered == Some(*b),
        })
        .collect();
    paint_strip(frame, area, &banner_texts(state, conflicts), &strip)
        .into_iter()
        .map(|(rect, i)| (rect, list[i].0))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use gitpoulpi_core::domain::Oid;

    fn merge() -> OperationState {
        OperationState::Merge {
            head_name: Some("main".into()),
            their_label: "feature/login".into(),
        }
    }

    #[test]
    fn the_text_names_the_operation_and_what_is_left() {
        let t = banner_text(&merge(), 2);
        assert!(t.contains("feature/login") && t.contains("main"), "{t}");
        assert!(t.contains("2 conflicted files"), "{t}");
        assert!(banner_text(&merge(), 1).contains("1 conflicted file"));
        assert!(banner_text(&merge(), 0).contains("all conflicts resolved"));
        let pick = OperationState::CherryPick {
            commit: Oid([0xab; 20]),
        };
        assert!(banner_text(&pick, 1).contains("abababa"));
    }

    #[test]
    fn the_way_forward_follows_the_conflicts() {
        let labels = |n| {
            buttons(&merge(), n)
                .into_iter()
                .map(|(b, _)| b)
                .collect::<Vec<_>>()
        };
        assert_eq!(
            labels(2),
            [OperationBannerButton::Resolve, OperationBannerButton::Abort]
        );
        assert_eq!(
            labels(0),
            [OperationBannerButton::Commit, OperationBannerButton::Abort]
        );
    }

    /// A cherry-pick or revert can be skipped (empty, or not worth
    /// resolving); a merge has nothing to skip.
    #[test]
    fn skip_is_offered_for_a_cherry_pick_and_a_revert_only() {
        let kinds = |state: &OperationState| {
            buttons(state, 0)
                .into_iter()
                .map(|(b, _)| b)
                .collect::<Vec<_>>()
        };
        let pick = OperationState::CherryPick {
            commit: Oid([0xab; 20]),
        };
        let revert = OperationState::Revert {
            commit: Oid([0xcd; 20]),
        };
        for state in [&pick, &revert] {
            assert_eq!(
                kinds(state),
                [
                    OperationBannerButton::Commit,
                    OperationBannerButton::Skip,
                    OperationBannerButton::Abort
                ]
            );
        }
        assert!(!kinds(&merge()).contains(&OperationBannerButton::Skip));
    }

    /// The banner's text never runs into its buttons: at every width the
    /// text is cut with an ellipsis, a space is kept before the first
    /// button, and no button overlaps another or leaves the row.
    #[test]
    fn the_label_never_runs_into_the_buttons_at_80_100_and_160_columns() {
        use ratatui::backend::TestBackend;
        use ratatui::Terminal;
        let rebase = gitpoulpi_core::domain::RebaseState {
            head_name: Some("feature/a-rather-long-branch-name".into()),
            onto: Oid([1; 20]),
            stopped_at: Some(Oid([9; 20])),
            step: 2,
            total: 5,
            conflicted: true,
            interactive: true,
        };
        let pending = gitpoulpi_session::PendingAutoStash {
            stash_id: gitpoulpi_core::domain::StashId(0),
            stash_commit: None,
            reason: gitpoulpi_core::domain::OperationKind::Pull,
            created_at: 0,
            state: gitpoulpi_session::PendingAutoStashState::Conflicted,
            error: None,
            after_operation: false,
        };
        for width in [80u16, 100, 160] {
            for kind in 0..3 {
                let mut terminal = Terminal::new(TestBackend::new(width, 1)).unwrap();
                let mut rects: Vec<Rect> = Vec::new();
                terminal
                    .draw(|f| {
                        rects = match kind {
                            0 => render(f, f.area(), &merge(), 2, None)
                                .into_iter()
                                .map(|h| h.0)
                                .collect(),
                            1 => crate::rebase_banner::render(f, f.area(), &rebase, None)
                                .into_iter()
                                .map(|h| h.0)
                                .collect(),
                            _ => crate::autostash_banner::render(f, f.area(), &pending, None)
                                .into_iter()
                                .map(|h| h.0)
                                .collect(),
                        };
                    })
                    .unwrap();
                let buf = terminal.backend().buffer();
                let first = rects.iter().map(|r| r.x).min().expect("a button");
                assert!(rects.iter().all(|r| r.right() <= width), "{kind}@{width}");
                for (i, a) in rects.iter().enumerate() {
                    for b in &rects[i + 1..] {
                        assert!(a.right() <= b.x || b.right() <= a.x, "{kind}@{width}");
                    }
                }
                // A space between the text and the first button.
                assert_eq!(buf[(first - 1, 0)].symbol(), " ", "{kind}@{width}");
                let text: String = (0..first - 1).map(|x| buf[(x, 0)].symbol()).collect();
                let candidates = match kind {
                    0 => banner_texts(&merge(), 2),
                    1 => vec![crate::rebase_banner::banner_text(&rebase)],
                    _ => vec![crate::autostash_banner::banner_text(&pending)],
                };
                // One of the forms, whole, or the shortest cut with an
                // ellipsis.
                assert!(
                    candidates.iter().any(|c| text.contains(c.as_str()))
                        || text.trim_end().ends_with('\u{2026}'),
                    "{kind}@{width}: {text:?}"
                );
            }
        }
    }

    /// A second banner goes on the Log's bottom border, never on the row of
    /// column titles under the top border.
    #[test]
    fn a_banner_never_takes_the_column_title_row() {
        let log = Rect::new(10, 3, 80, 20);
        assert_eq!(strip_rect(log, 0), Some(Rect::new(10, 3, 80, 1)));
        assert_eq!(strip_rect(log, 1), Some(Rect::new(10, 22, 80, 1)));
        for slot in 0..2 {
            let r = strip_rect(log, slot).unwrap();
            assert_ne!(r.y, log.y + 1, "the column titles");
            assert_ne!(r.y, log.y + 2, "the first Log row");
        }
        // Too short for a second one: it is not drawn rather than overlaid.
        assert_eq!(strip_rect(Rect::new(0, 0, 80, 4), 1), None);
        assert_eq!(strip_rect(Rect::new(0, 0, 80, 0), 0), None);
    }

    #[test]
    fn the_rebase_has_its_own_banner() {
        assert!(applies(&merge()));
    }
}
