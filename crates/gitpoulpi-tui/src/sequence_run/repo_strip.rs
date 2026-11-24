//! The notice over the Log when the repository carries its own sequences
//! (`.gitpoulpi/sequences.toml`) nobody trusted yet, or that changed since:
//! "This repository defines N sequences" with `[Review] [Trust] [Ignore]`.
//!
//! It is a strip like the others (`operation_banner::paint_strip`): one row on
//! a free border row of the Log, never a layout of its own, never blocking.
//! Each button is the twin of a palette command (`Sequence: Review This
//! Repository's Sequences`, `... Trust ...`, `... Ignore ...`).

use super::repo::Notice;
use crate::operation_banner::{paint_strip, StripButton as Button};
use gitpoulpi_i18n::msg::sequence_run as m;
use ratatui::layout::Rect;
use ratatui::Frame;

/// The notice's buttons.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NoticeButton {
    Review,
    Trust,
    Ignore,
}

/// The words, longest first.
pub(crate) fn texts(notice: &Notice) -> Vec<String> {
    let n = notice.count as u64;
    let long = if notice.changed {
        m::repo_notice_changed(n)
    } else {
        m::repo_notice(n)
    };
    vec![format!("\u{25b6} {long}")]
}

/// The buttons, in order.
pub(crate) fn buttons() -> Vec<(NoticeButton, String)> {
    vec![
        (
            NoticeButton::Review,
            m::REPO_BUTTON_REVIEW.get().to_string(),
        ),
        (NoticeButton::Trust, m::REPO_BUTTON_TRUST.get().to_string()),
        (
            NoticeButton::Ignore,
            m::REPO_BUTTON_IGNORE.get().to_string(),
        ),
    ]
}

/// Draws the notice on `area`'s row; returns each button's rect.
pub(crate) fn render(
    frame: &mut Frame,
    area: Rect,
    notice: &Notice,
    hovered: Option<NoticeButton>,
) -> Vec<(Rect, NoticeButton)> {
    let list = buttons();
    let drawn: Vec<Button<'_>> = list
        .iter()
        .map(|(b, label)| Button {
            label,
            danger: false,
            hovered: hovered == Some(*b),
            disabled: false,
        })
        .collect();
    paint_strip(frame, area, &texts(notice), &drawn)
        .into_iter()
        .map(|(rect, i)| (rect, list[i].0))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    fn drawn(width: u16, notice: &Notice) -> (String, Vec<(Rect, NoticeButton)>) {
        let mut terminal = Terminal::new(TestBackend::new(width, 1)).unwrap();
        let mut hits = Vec::new();
        terminal
            .draw(|f| hits = render(f, Rect::new(0, 0, width, 1), notice, None))
            .unwrap();
        let row: String = (0..width)
            .map(|x| terminal.backend().buffer()[(x, 0)].symbol().to_string())
            .collect();
        (row, hits)
    }

    #[test]
    fn the_notice_counts_the_sequences_and_offers_the_three_answers() {
        let (row, hits) = drawn(
            100,
            &Notice {
                count: 3,
                changed: false,
            },
        );
        assert!(row.contains("This repository defines 3 sequences"), "{row}");
        let kinds: Vec<_> = hits.iter().map(|(_, b)| *b).collect();
        assert_eq!(
            kinds,
            [
                NoticeButton::Review,
                NoticeButton::Trust,
                NoticeButton::Ignore
            ]
        );
        let (row, _) = drawn(
            100,
            &Notice {
                count: 1,
                changed: false,
            },
        );
        assert!(row.contains("defines 1 sequence "), "{row}");
        let (row, _) = drawn(
            100,
            &Notice {
                count: 2,
                changed: true,
            },
        );
        assert!(row.contains("2 sequences changed"), "{row}");
    }

    #[test]
    fn the_notice_reads_in_french() {
        let _fr = gitpoulpi_i18n::override_lang(gitpoulpi_i18n::Lang::Fr);
        let (row, _) = drawn(
            100,
            &Notice {
                count: 2,
                changed: false,
            },
        );
        assert!(row.contains("Ce dépôt définit 2 séquences"), "{row}");
        assert!(
            row.contains("Examiner") && row.contains("Approuver"),
            "{row}"
        );
    }
}
