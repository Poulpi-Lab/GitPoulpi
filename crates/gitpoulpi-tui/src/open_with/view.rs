//! The "Open in" row of a binary file in the Diff view and the File View: the
//! file's name on the left, the combobox button on the right, the old notice
//! ("binary file - nothing to show") two rows below. Where the OS knows no
//! application for the type (or it is plain text) nothing is drawn here and
//! the caller keeps today's notice.
//!
//! What the last frame drew ([`shown`]) is what the keys and the palette act
//! on, so they exist exactly when the button does. The button's rect is the one
//! the paint produced, registered as `HitTarget::OpenWith`.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use gitpoulpi_i18n::msg::open_with as msg;

use super::{popup, Lookup, Subject};
use crate::chrome::HitTarget;
use crate::file_view::{FileInsight, FileInsightData};
use crate::ui_kit::ClippedRender;
use crate::{display_width, th};

static SHOWN: Mutex<Option<Subject>> = Mutex::new(None);
static HOVERED: AtomicBool = AtomicBool::new(false);

/// Call at the start of every draw: whatever the last frame showed is forgotten.
pub(crate) fn begin_frame() {
    if let Ok(mut s) = SHOWN.lock() {
        *s = None;
    }
    popup::forget_anchor();
}

/// The file the last frame offered "Open in" for, if it did.
pub(crate) fn shown() -> Option<Subject> {
    SHOWN.lock().ok().and_then(|s| s.clone())
}

/// Whether the pointer is over the button (lights it).
pub(crate) fn set_hovered(target: Option<&HitTarget>) {
    HOVERED.store(
        matches!(target, Some(HitTarget::OpenWith)),
        Ordering::Relaxed,
    );
}

/// Draws the row and the notice into `inner`; the button's hit target.
/// `None` when there is nothing to offer (the caller draws its own notice).
pub(crate) fn render(
    frame: &mut Frame,
    inner: Rect,
    subject: &Subject,
    notice: &str,
) -> Option<Vec<(Rect, HitTarget)>> {
    let name = subject.name();
    if super::looks_textual(&name) {
        return None;
    }
    let looking = match super::lookup(&name) {
        Lookup::Ready(found) if found.textual || found.apps.is_empty() => return None,
        Lookup::Ready(_) => false,
        // Quick lookups never show a flash of feedback.
        Lookup::Pending { since } if since.elapsed() < super::LOOKUP_FEEDBACK_AFTER => {
            return None;
        }
        Lookup::Pending { .. } => true,
    };
    let button = popup::button_width();
    if inner.height < 1 || inner.width < button + 2 {
        return None;
    }
    if let Ok(mut s) = SHOWN.lock() {
        *s = Some(subject.clone());
    }
    let muted = Style::default().fg(th().ui.muted);
    let mut hits = Vec::new();
    let row = Rect::new(inner.x, inner.y, inner.width, 1);
    let right_edge = inner.right().saturating_sub(1);
    if looking {
        let text = msg::LOOKING_UP.get();
        let w = (display_width(text) as u16).min(row.width.saturating_sub(2));
        frame.render_clipped(
            Paragraph::new(Line::from(Span::styled(
                text.to_string(),
                muted.add_modifier(Modifier::ITALIC),
            ))),
            Rect::new(right_edge.saturating_sub(w), row.y, w, 1),
        );
    } else {
        let rect = Rect::new(right_edge.saturating_sub(button), row.y, button, 1);
        popup::render_button(frame.buffer_mut(), rect, HOVERED.load(Ordering::Relaxed));
        hits.push((rect, HitTarget::OpenWith));
    }
    let room = row.width.saturating_sub(button + 3);
    frame.render_clipped(
        Paragraph::new(Line::from(Span::styled(format!(" {name}"), muted))),
        Rect::new(row.x, row.y, room, 1),
    );
    if inner.height > 2 {
        frame.render_clipped(
            Paragraph::new(Line::from(Span::styled(
                format!(" {notice}"),
                muted.add_modifier(Modifier::ITALIC),
            ))),
            Rect::new(inner.x, inner.y + 2, inner.width, 1),
        );
    }
    Some(hits)
}

/// The File View's row, for a binary file's content (or blame notice).
pub(crate) fn render_file(
    frame: &mut Frame,
    inner: Rect,
    insight: &FileInsight,
) -> Option<Vec<(Rect, HitTarget)>> {
    use crate::daemon_link::FileInsightKind;
    use gitpoulpi_core::domain::BlobContent;
    if insight.request.kind != FileInsightKind::Content {
        return None;
    }
    let FileInsightData::Content(blob) = &insight.data else {
        return None;
    };
    if !matches!(blob.content, BlobContent::Binary) {
        return None;
    }
    let subject = subject_of_insight(insight);
    let notice = gitpoulpi_i18n::msg::file_view::binary_show(blob.size);
    render(frame, inner, &subject, &notice)
}

/// Which version of the file a File View holds.
pub(crate) fn subject_of_insight(insight: &FileInsight) -> Subject {
    let req = &insight.request;
    let source = if req.worktree && req.staged {
        super::Source::Index
    } else if req.worktree {
        super::Source::WorkingTree
    } else {
        super::Source::Commit(req.at)
    };
    Subject {
        path: req.path.clone(),
        source,
    }
}

/// The Diff view's row, for a file with no hunks to show.
pub(crate) fn render_diff(
    frame: &mut Frame,
    inner: Rect,
    state: &crate::diff_view::DiffViewState,
) -> Option<Vec<(Rect, HitTarget)>> {
    use crate::diff_view::DiffSource;
    let source = match state.source {
        DiffSource::Unstaged => super::Source::WorkingTree,
        DiffSource::Staged => super::Source::Index,
        DiffSource::Commit => super::Source::Commit(state.commit),
        // A comparison's sides are two arbitrary revisions: not wired.
        DiffSource::Compare => return None,
    };
    let subject = Subject {
        path: state.path.clone(),
        source,
    };
    render(
        frame,
        inner,
        &subject,
        gitpoulpi_i18n::msg::diff_view::NOTICE_NO_CHANGES.get(),
    )
}
