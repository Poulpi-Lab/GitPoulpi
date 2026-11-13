//! Naming a new branch or tag *inline in the Log*:
//! a text box appears in the Ref column, on the row of the commit the new
//! ref will point at, drawn over whatever pills that commit already
//! carries. Enter confirms, Esc cancels and a name git would refuse (or a
//! ref that already exists) is reported on the same row, with the box
//! still open.
//!
//! The typing itself is still `InputMode::InlineName` -- same field,
//! same caret, same keys as before -- so the palette, the toolbar, the
//! `Ctrl+G b`/`t` chords and the commit context menu all reach it without
//! a second mode. What this module adds is the pure half: which commit the
//! box belongs on, whether a name is acceptable, and how the box is drawn.

use crate::wide_edge::Clear;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use super::*;
use gitpoulpi_i18n::msg::menu_inline as msg;

/// What the inline box is naming.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum InlineRefKind {
    Branch,
    Tag,
}

impl InlineRefKind {
    /// The placeholder shown while the box is empty (the conventional wording).
    pub(crate) fn placeholder(self) -> &'static str {
        match self {
            InlineRefKind::Branch => msg::PLACEHOLDER_BRANCH.get(),
            InlineRefKind::Tag => msg::PLACEHOLDER_TAG.get(),
        }
    }
}

/// The kind and the commit an in-progress name belongs to in the Log, or
/// `None` for the prompts that are not "a name on a commit's row" (the
/// Refs-row rename, create file, annotate, upstream...). `NewBranch`/`NewTag`
/// sit on HEAD's commit, the same commit `Action::BranchCreate` will use; a
/// rename from the pill (or the palette, as a second choice) sits on the
/// branch's tip.
pub(crate) fn inline_target(
    op: &RefNameOp,
    refs: &[gitpoulpi_core::domain::Ref],
) -> Option<(InlineRefKind, Oid)> {
    match op {
        RefNameOp::NewBranchAt(at) => Some((InlineRefKind::Branch, *at)),
        RefNameOp::NewTagAt(at) | RefNameOp::NewAnnotatedTagAt(at) => {
            Some((InlineRefKind::Tag, *at))
        }
        RefNameOp::NewBranch => head_oid(refs).map(|h| (InlineRefKind::Branch, h)),
        RefNameOp::NewTag => head_oid(refs).map(|h| (InlineRefKind::Tag, h)),
        RefNameOp::RenameBranch {
            name,
            spot: RenameSpot::Pill | RenameSpot::Auto,
        } => refs
            .iter()
            .find(|r| matches!(&r.kind, RefKind::LocalBranch { name: n } if n == name))
            .map(|r| (InlineRefKind::Branch, r.target)),
        RefNameOp::RenameBranch {
            spot: RenameSpot::Refs,
            ..
        }
        | RefNameOp::CreateFile(_) => None,
    }
}

/// The refs a rename must not collide with: every ref but the branch being
/// renamed (so `a` may become `a2`, and `a/b` may take `a`'s place).
pub(crate) fn refs_without_branch(
    refs: &[gitpoulpi_core::domain::Ref],
    old: &str,
) -> Vec<gitpoulpi_core::domain::Ref> {
    refs.iter()
        .filter(|r| !matches!(&r.kind, RefKind::LocalBranch { name } if name == old))
        .cloned()
        .collect()
}

/// `git check-ref-format`'s rules, as far as a short human message goes.
fn check_ref_format(name: &str) -> Result<(), String> {
    if name == "@" {
        return Err(msg::NAME_AT.get().into());
    }
    if name.starts_with('/') || name.ends_with('/') || name.contains("//") {
        return Err(msg::NAME_SLASH.get().into());
    }
    if name.ends_with('.') {
        return Err(msg::NAME_DOT_END.get().into());
    }
    if name.contains("..") {
        return Err(msg::NAME_DOTDOT.get().into());
    }
    if name.contains("@{") {
        return Err(msg::NAME_AT_BRACE.get().into());
    }
    if name.starts_with('-') {
        return Err(msg::NAME_DASH_START.get().into());
    }
    if let Some(bad) = name
        .chars()
        .find(|c| c.is_whitespace() || c.is_control() || "~^:?*[\\".contains(*c))
    {
        return Err(if bad.is_whitespace() {
            msg::NAME_SPACES.get().to_string()
        } else {
            msg::name_bad_char(bad.escape_default())
        });
    }
    for part in name.split('/') {
        if part.starts_with('.') {
            return Err(msg::NAME_PART_DOT.get().into());
        }
        if part.ends_with(".lock") {
            return Err(msg::NAME_PART_LOCK.get().into());
        }
    }
    Ok(())
}

/// The trimmed name if it can be created, otherwise the message to show
/// on the row. Checks git's format rules, an existing ref of the same
/// kind, and a directory/file clash (`a` next to `a/b`), which git also
/// refuses.
pub(crate) fn validate_new_ref(
    kind: InlineRefKind,
    raw: &str,
    refs: &[gitpoulpi_core::domain::Ref],
) -> Result<String, String> {
    let name = raw.trim();
    if name.is_empty() {
        return Err(match kind {
            InlineRefKind::Branch => msg::ENTER_BRANCH.get().to_string(),
            InlineRefKind::Tag => msg::ENTER_TAG.get().to_string(),
        });
    }
    check_ref_format(name)?;
    let existing = refs.iter().filter_map(|r| match (&r.kind, kind) {
        (RefKind::LocalBranch { name }, InlineRefKind::Branch) => Some(name.as_str()),
        (RefKind::Tag { name }, InlineRefKind::Tag) => Some(name.as_str()),
        _ => None,
    });
    for other in existing {
        if other == name {
            return Err(match kind {
                InlineRefKind::Branch => msg::exists_branch(name),
                InlineRefKind::Tag => msg::exists_tag(name),
            });
        }
        if other.starts_with(&format!("{name}/")) || name.starts_with(&format!("{other}/")) {
            return Err(msg::clashes(name, other));
        }
    }
    Ok(name.to_string())
}

/// The Ref cell of a commit's row, when it is wide enough to hold a
/// usable box. `None` (Ref column hidden, or squeezed away) sends the
/// caller back to the status-bar field so naming still works.
pub(crate) fn inline_box_rect(offsets: &[(LogColumn, u16, u16)], screen_row: u16) -> Option<Rect> {
    let (_, x, w) = *offsets.iter().find(|(c, _, _)| *c == LogColumn::Ref)?;
    (w >= 8).then(|| Rect::new(x, screen_row, w, 1))
}

/// Where the error is written: from the Message cell to the end of the
/// row (Message, Author, Date... - the row's own text is under it).
pub(crate) fn inline_error_rect(
    offsets: &[(LogColumn, u16, u16)],
    screen_row: u16,
) -> Option<Rect> {
    let (_, x, _) = *offsets.iter().find(|(c, _, _)| *c == LogColumn::Message)?;
    let end = offsets.iter().map(|(_, cx, cw)| cx + cw).max()?;
    (end >= x + 8).then(|| Rect::new(x, screen_row, end - x, 1))
}

fn box_bg() -> Color {
    crate::input_focus_bg()
}

/// One line of the box: caret first, then the placeholder while empty;
/// otherwise the draft through the shared field drawing, so caret and
/// selection behave as in every other field. `width` includes the one
/// blank column kept on the left.
#[cfg(test)]
pub(crate) fn box_line(
    kind: InlineRefKind,
    value: &str,
    caret: Option<&crate::text_edit::Caret>,
    width: usize,
    error: bool,
) -> Line<'static> {
    name_line(kind.placeholder(), value, caret, width, error)
}

/// [`box_line`] with any placeholder (the new-file box names a file).
pub(crate) fn name_line(
    placeholder: &str,
    value: &str,
    caret: Option<&crate::text_edit::Caret>,
    width: usize,
    error: bool,
) -> Line<'static> {
    let bg = if error {
        th().chrome.error_bg
    } else {
        box_bg()
    };
    let base = Style::default().fg(th().ui.text).bg(bg);
    let caret_style = Style::default().fg(th().ui.accent).bg(bg);
    let mut spans = vec![Span::styled(" ", base)];
    let inner = width.saturating_sub(1);
    if value.is_empty() {
        let hint = truncate_with_ellipsis(placeholder, inner.saturating_sub(1));
        let hint_w = display_width(&hint);
        spans.push(Span::styled("\u{2588}", caret_style));
        spans.push(Span::styled(
            hint,
            Style::default()
                .fg(th().ui.muted)
                .bg(bg)
                .add_modifier(Modifier::ITALIC),
        ));
        spans.push(Span::styled(
            " ".repeat(inner.saturating_sub(1 + hint_w)),
            base,
        ));
    } else {
        spans.extend(
            text_field::edit_line(
                value,
                caret,
                inner,
                true,
                text_field::EditLook::new(base, "\u{2588}").caret_style(caret_style),
            )
            .spans,
        );
    }
    Line::from(spans)
}

/// Paints the box over the commit's Ref cell, and the error (if any) in
/// its Message cell. Returns the rect of the *text* (for the caret's
/// mouse mapping), which starts one column in.
pub(crate) fn render_inline_box(
    frame: &mut Frame,
    box_rect: Rect,
    error_rect: Option<Rect>,
    kind: InlineRefKind,
    value: &str,
    caret: Option<&crate::text_edit::Caret>,
    error: Option<&str>,
) -> Rect {
    render_name_box(
        frame,
        box_rect,
        error_rect,
        kind.placeholder(),
        value,
        caret,
        error,
    )
}

/// [`render_inline_box`] with any placeholder.
pub(crate) fn render_name_box(
    frame: &mut Frame,
    box_rect: Rect,
    error_rect: Option<Rect>,
    placeholder: &str,
    value: &str,
    caret: Option<&crate::text_edit::Caret>,
    error: Option<&str>,
) -> Rect {
    frame.render_widget(Clear, box_rect);
    frame.render_widget(
        Paragraph::new(name_line(
            placeholder,
            value,
            caret,
            usize::from(box_rect.width),
            error.is_some(),
        )),
        box_rect,
    );
    if let (Some(msg), Some(rect)) = (error, error_rect) {
        let text = truncate_with_ellipsis(&format!(" \u{2717} {msg}"), usize::from(rect.width));
        frame.render_widget(Clear, rect);
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                text,
                Style::default()
                    .fg(th().status.danger)
                    .add_modifier(Modifier::BOLD),
            ))),
            rect,
        );
    }
    Rect::new(box_rect.x + 1, box_rect.y, box_rect.width - 1, 1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::*;

    fn tag_ref(name: &str) -> gitpoulpi_core::domain::Ref {
        gitpoulpi_core::domain::Ref {
            kind: RefKind::Tag { name: name.into() },
            ..local_branch_at("x", oid(1))
        }
    }

    #[test]
    fn accepts_plain_and_slashed_names() {
        assert_eq!(
            validate_new_ref(InlineRefKind::Branch, " feat/x ", &[]),
            Ok("feat/x".into())
        );
        assert_eq!(
            validate_new_ref(InlineRefKind::Tag, "v1.0", &[]),
            Ok("v1.0".into())
        );
    }

    #[test]
    fn refuses_what_git_refuses() {
        for bad in [
            "", "  ", "a b", "a..b", "a~", "a^", "a:b", "a?", "a*", "a[", "a\\b", "/a", "a/",
            "a//b", "a.", ".a", "a/.b", "x.lock", "@", "a@{b", "-a",
        ] {
            assert!(
                validate_new_ref(InlineRefKind::Branch, bad, &[]).is_err(),
                "{bad:?}"
            );
        }
    }

    #[test]
    fn refuses_existing_refs_of_the_same_kind_only() {
        let refs = vec![local_branch_at("main", oid(1)), tag_ref("v1")];
        assert!(validate_new_ref(InlineRefKind::Branch, "main", &refs).is_err());
        assert!(validate_new_ref(InlineRefKind::Tag, "main", &refs).is_ok());
        assert!(validate_new_ref(InlineRefKind::Tag, "v1", &refs).is_err());
        assert!(validate_new_ref(InlineRefKind::Branch, "v1", &refs).is_ok());
        // Directory/file clash.
        assert!(validate_new_ref(InlineRefKind::Branch, "main/x", &refs).is_err());
    }

    #[test]
    fn inline_target_covers_the_commit_bound_prompts_only() {
        let mut head = local_branch_at("main", oid(2));
        head.is_head = true;
        let refs = vec![head, local_branch_at("feat", oid(5))];
        assert_eq!(
            inline_target(&RefNameOp::NewBranchAt(oid(3)), &[]),
            Some((InlineRefKind::Branch, oid(3)))
        );
        assert_eq!(
            inline_target(&RefNameOp::NewTag, &refs),
            Some((InlineRefKind::Tag, oid(2)))
        );
        assert_eq!(inline_target(&RefNameOp::NewTag, &[]), None);
        // A rename sits on the Refs row, or on the branch's tip in the Log.
        let rename = |spot| RefNameOp::RenameBranch {
            name: "feat".into(),
            spot,
        };
        assert_eq!(inline_target(&rename(RenameSpot::Refs), &refs), None);
        assert_eq!(
            inline_target(&rename(RenameSpot::Pill), &refs),
            Some((InlineRefKind::Branch, oid(5)))
        );
        assert_eq!(
            inline_target(&rename(RenameSpot::Auto), &refs),
            Some((InlineRefKind::Branch, oid(5)))
        );
    }

    #[test]
    fn a_rename_may_reuse_its_own_name_and_its_folder_slot() {
        let refs = vec![local_branch_at("a", oid(1)), local_branch_at("b", oid(1))];
        let others = refs_without_branch(&refs, "a");
        assert_eq!(others.len(), 1);
        assert!(validate_new_ref(InlineRefKind::Branch, "a/x", &others).is_ok());
        assert!(validate_new_ref(InlineRefKind::Branch, "b", &others).is_err());
    }

    #[test]
    fn empty_box_shows_the_placeholder_and_a_typed_one_the_text() {
        let text = |l: Line| {
            l.spans
                .iter()
                .map(|s| s.content.to_string())
                .collect::<String>()
        };
        let empty = text(box_line(InlineRefKind::Branch, "", None, 24, false));
        assert!(empty.contains("enter branch name"), "{empty:?}");
        assert_eq!(display_width(&empty), 24);
        let tag = text(box_line(InlineRefKind::Tag, "", None, 24, false));
        assert!(tag.contains("enter tag name"), "{tag:?}");
        let typed = text(box_line(InlineRefKind::Tag, "v2", None, 24, true));
        assert!(
            typed.contains("v2") && !typed.contains("enter"),
            "{typed:?}"
        );
        assert_eq!(display_width(&typed), 24);
    }

    #[test]
    fn box_needs_a_usable_ref_column() {
        let offs = vec![(LogColumn::Ref, 1, 12), (LogColumn::Message, 13, 30)];
        assert_eq!(inline_box_rect(&offs, 5), Some(Rect::new(1, 5, 12, 1)));
        assert_eq!(inline_error_rect(&offs, 5), Some(Rect::new(13, 5, 30, 1)));
        let wide = vec![
            (LogColumn::Ref, 1, 12),
            (LogColumn::Message, 13, 30),
            (LogColumn::Date, 43, 10),
        ];
        assert_eq!(inline_error_rect(&wide, 5), Some(Rect::new(13, 5, 40, 1)));
        assert_eq!(inline_box_rect(&[(LogColumn::Ref, 1, 6)], 5), None);
        assert_eq!(inline_box_rect(&[(LogColumn::Message, 1, 30)], 5), None);
    }
}
