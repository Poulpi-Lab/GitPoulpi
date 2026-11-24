use super::*;

/// Renders one row of a working-changes sub-pane and registers its hover
/// button's hit target.
///
/// Split out of the draw closure because both sub-panes render rows the
/// same way and the closure is already long; `hits` is threaded through
/// rather than captured so the function stays callable per row.
#[allow(clippy::too_many_arguments)]
pub(crate) fn status_row_item<'a>(
    line: &'a StatusPaneLine,
    section: StatusSection,
    hovered: bool,
    on_screen: bool,
    list_area: Rect,
    row_y: u16,
    lfs: LfsMark<'_>,
    picked: bool,
    h_off: usize,
    hits: &mut Vec<(Rect, HitTarget)>,
) -> ListItem<'a> {
    // The per-row Stage/Unstage button is a hover affordance, exactly as
    // revealed on hover -- shown only for the row under the cursor so
    // the list isn't a wall of buttons.
    // The button is right-aligned and drawn OVER the end of the row (with a
    // solid background), so it is always whole, however long the name: the
    // head is clipped to what is left of the row, never the button.
    let push_button = |hits: &mut Vec<(Rect, HitTarget)>,
                       head: Vec<Span<'a>>,
                       label: &str,
                       target: HitTarget|
     -> Vec<Span<'a>> {
        let button_w = display_width(label) + 2;
        let room = (list_area.width as usize).saturating_sub(button_w);
        let mut spans = clip_spans(head, room);
        let head_w: usize = spans.iter().map(|s| display_width(&s.content)).sum();
        if on_screen {
            hits.push((
                Rect::new(
                    list_area.x + room as u16,
                    row_y,
                    (button_w as u16).min(list_area.width),
                    1,
                ),
                target,
            ));
        }
        spans.push(Span::raw(" ".repeat(room.saturating_sub(head_w))));
        spans.push(Span::styled(
            format!(" {label} "),
            Style::default().fg(th().ui.accent).bg(th().ui.hover),
        ));
        spans
    };

    let mut item = match line {
        StatusPaneLine::Folder {
            path,
            fold_key,
            depth,
            label,
            collapsed,
        } => {
            let chevron = crate::fold_mark::glyph(*collapsed);
            let head = format!("{}{chevron} {label}/", "  ".repeat(*depth));
            let head_style = Style::default()
                .fg(th().ui.muted)
                .add_modifier(Modifier::BOLD);
            // A folder row had no hover fill distinct from the reveal of
            // its own Stage/Unstage Folder button -- `hover_style` on the
            // name span itself, same as every other row's hover cue.
            let head_style = if hovered {
                hover_style(head_style)
            } else {
                head_style
            };
            // Scrolled sideways: the name, never the button below.
            let mut spans =
                crate::hscroll::cut_spans(vec![Span::styled(head.clone(), head_style)], h_off);
            if hovered {
                // A folder stages wholesale -- `StageTarget::Folder` exists
                // and both backends implement it.
                let label = match section {
                    StatusSection::Unstaged => gitpoulpi_i18n::msg::status_pane::STAGE_FOLDER.get(),
                    StatusSection::Staged => gitpoulpi_i18n::msg::status_pane::UNSTAGE_FOLDER.get(),
                };
                spans = push_button(
                    hits,
                    spans,
                    label,
                    HitTarget::StatusFolderButton {
                        section,
                        path: path.clone(),
                    },
                );
            }
            // After the button, never before: the row band covers the whole
            // width including the button, and `find_hit_target` takes the
            // first match -- registered first, the band swallowed every
            // click on `Stage Folder` and collapsed the folder instead.
            if on_screen {
                hits.push((
                    Rect::new(list_area.x, row_y, list_area.width, 1),
                    HitTarget::StatusFolderRow(fold_key.clone()),
                ));
            }
            ListItem::new(Line::from(spans))
        }
        StatusPaneLine::File {
            path,
            depth,
            label,
            status,
            ..
        } => {
            let marker = status.map(status_marker).unwrap_or(" ");
            let marker_color = status.map(file_status_color).unwrap_or(th().ui.muted);
            let indent = "  ".repeat(*depth);
            let mut spans = vec![
                Span::styled(
                    format!("{indent}{marker} "),
                    Style::default().fg(marker_color),
                ),
                Span::styled(label.clone(), Style::default().fg(th().ui.text)),
            ];
            // A file stored with Git LFS says so, in plain ASCII (no
            // glyph to measure), dim and after the name so an aligned
            // list of names stays aligned.
            if lfs.is_lfs {
                spans.push(Span::styled(
                    " LFS",
                    Style::default()
                        .fg(th().ui.accent)
                        .add_modifier(Modifier::BOLD),
                ));
            }
            if let Some(owner) = lfs.locked_by {
                spans.push(Span::styled(
                    gitpoulpi_i18n::msg::status_list::locked_by(owner),
                    Style::default().fg(th().status.warning),
                ));
            }
            let mut spans = crate::hscroll::cut_spans(spans, h_off);
            if hovered {
                spans = push_button(
                    hits,
                    spans,
                    section.row_label(),
                    HitTarget::StatusRowButton {
                        section,
                        path: path.clone(),
                    },
                );
            }
            ListItem::new(Line::from(spans))
        }
    };
    // Multi-selected (Ctrl/Shift+click): the same fill as the cursor row.
    if picked {
        item = item.style(Style::default().bg(th().ui.selection));
    }
    if hovered {
        item = item.style(Style::default().bg(th().ui.hover));
    }
    item
}

/// What a Status row says about Git LFS: the `LFS` tag for a file stored
/// with LFS, and `locked by X` for a file somebody has locked.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct LfsMark<'a> {
    pub(crate) is_lfs: bool,
    /// `Some("you")` / `Some("bob")` when the file is locked.
    pub(crate) locked_by: Option<&'a str>,
}

impl<'a> LfsMark<'a> {
    /// The mark for `path` under the current LFS state.
    pub(crate) fn of(lfs: &'a crate::lfs_ui::LfsUi, path: &std::path::Path) -> Self {
        Self {
            is_lfs: lfs.is_lfs(path),
            locked_by: lfs.lock_on(path).map(|l| {
                if l.ours {
                    gitpoulpi_i18n::msg::status_list::LOCK_YOU.get()
                } else {
                    l.owner
                        .as_deref()
                        .unwrap_or(gitpoulpi_i18n::msg::status_list::LOCK_SOMEONE.get())
                }
            }),
        }
    }
}

/// The half-open row range a sub-pane's `List` can actually paint this
/// frame: the `visible` rows from the offset, widened to take in the
/// selection when ratatui may scroll to it (`follow`, or already in view)
/// and to the last row when the offset is past the end (ratatui clamps it).
pub(crate) fn status_paint_window(
    len: usize,
    state: &ListState,
    follow: bool,
    visible: usize,
) -> std::ops::Range<usize> {
    let visible = visible.max(1);
    let mut lo = state.offset();
    let mut hi = lo.saturating_add(visible);
    if let Some(sel) = state.selected() {
        if follow || (sel >= lo && sel < hi) {
            lo = lo.min(sel);
            hi = hi.max(sel + 1);
        }
    }
    lo = lo.min(len.saturating_sub(1));
    lo..hi.min(len)
}

// Rows built by `status_row_items` (test counter: rows built must equal
// the paint window, not the file count).
#[cfg(test)]
thread_local! {
    pub(super) static STATUS_ROWS_BUILT: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// One sub-pane's `ListItem`s, with a real row (and its hit targets) only
/// inside `status_paint_window` and a one-line blank elsewhere - every
/// row has height 1, so the `List` scrolls identically while a 2,000-file
/// tree does not style 2,000 rows every frame.
#[allow(clippy::too_many_arguments)]
pub(crate) fn status_row_items<'a>(
    lines: &'a [StatusPaneLine],
    section: StatusSection,
    hovered_row: Option<usize>,
    state: &ListState,
    follow: bool,
    list_area: Rect,
    lfs: &'a crate::lfs_ui::LfsUi,
    sel: &FileSelection,
    h_off: usize,
    hits: &mut Vec<(Rect, HitTarget)>,
) -> Vec<ListItem<'a>> {
    let list = match section {
        StatusSection::Unstaged => FileList::Unstaged,
        StatusSection::Staged => FileList::Staged,
    };
    let offset = state.offset();
    let window = status_paint_window(lines.len(), state, follow, list_area.height as usize);
    lines
        .iter()
        .enumerate()
        .map(|(i, line)| {
            if !window.contains(&i) {
                return ListItem::new("");
            }
            #[cfg(test)]
            STATUS_ROWS_BUILT.with(|n| n.set(n.get() + 1));
            let row_y = list_area.y + (i.saturating_sub(offset)) as u16;
            let on_screen = i >= offset && row_y < list_area.y + list_area.height;
            let mark = match line {
                StatusPaneLine::File { path, .. } => LfsMark::of(lfs, path),
                StatusPaneLine::Folder { .. } => LfsMark::default(),
            };
            let picked = match line {
                StatusPaneLine::File { path, .. } => sel.contains(list, path, false),
                StatusPaneLine::Folder { path, .. } => sel.contains(list, path, true),
            };
            status_row_item(
                line,
                section,
                hovered_row == Some(i),
                on_screen,
                list_area,
                row_y,
                mark,
                picked,
                h_off,
                hits,
            )
        })
        .collect()
}

/// How wide a row is before it scrolls: the indent, the marker, the name and
/// the tags after it.
fn row_width(line: &StatusPaneLine, mark: LfsMark<'_>) -> usize {
    match line {
        StatusPaneLine::Folder { depth, label, .. } => {
            // chevron, space, name, slash
            depth * 2
                + crate::cell_text::cells(crate::fold_mark::glyph(false))
                + 1
                + crate::cell_text::cells(label)
                + 1
        }
        StatusPaneLine::File { depth, label, .. } => {
            let mut w = depth * 2 + 2 + crate::cell_text::cells(label);
            if mark.is_lfs {
                w += 4;
            }
            if let Some(owner) = mark.locked_by {
                w += crate::cell_text::cells(&gitpoulpi_i18n::msg::status_list::locked_by(owner));
            }
            w
        }
    }
}

/// The widest row the sub-pane paints this frame (its paint window), the
/// width the sideways scroll runs over.
pub(crate) fn status_content_width(
    lines: &[StatusPaneLine],
    state: &ListState,
    follow: bool,
    visible: usize,
    lfs: &crate::lfs_ui::LfsUi,
    section: StatusSection,
    width: usize,
) -> usize {
    // A row wider than the list (`width`) gets room past its end for the hover
    // button, which covers it: scrolled to the end, the name's last cell is
    // still in view. A row that fits needs no bar.
    let button = crate::cell_text::cells(section.row_label())
        .max(crate::cell_text::cells(
            gitpoulpi_i18n::msg::status_pane::STAGE_FOLDER.get(),
        ))
        .max(crate::cell_text::cells(
            gitpoulpi_i18n::msg::status_pane::UNSTAGE_FOLDER.get(),
        ))
        + 3;
    status_paint_window(lines.len(), state, follow, visible)
        .filter_map(|i| lines.get(i))
        .map(|line| {
            let mark = match line {
                StatusPaneLine::File { path, .. } => LfsMark::of(lfs, path),
                StatusPaneLine::Folder { .. } => LfsMark::default(),
            };
            row_width(line, mark)
        })
        .max()
        .map_or(0, |w| if w > width { w + button } else { w })
}
