use super::*;

/// The bulk stage/unstage button's rect on a working-changes sub-pane's
/// header row, or `None` when the section is too narrow to carry it.
///
/// The header row is shared: the `Block` paints `title` starting one cell
/// past the left border, and the button is anchored one cell short of the
/// right one. The button is dropped when it would overlap the title.
///
/// Both labels are measured with `display_width` (terminal columns, not
/// bytes): the title carries a `\u{25BE}` chevron and a file count.
/// Only the full-label case of `status_bulk_button_layout`, which is what
/// the rendering does whenever the label fits; kept as its own name for
/// the tests pinning the title-overlap bug.
#[cfg(test)]
pub(crate) fn status_bulk_button_rect(area: Rect, title: &str, bulk: &str) -> Option<Rect> {
    status_bulk_button_layout(area, title, bulk)
        .filter(|(rect, is_more)| !is_more && rect.width == display_width(bulk) as u16 + 2)
        .map(|(rect, _)| rect)
}

/// The global "Discard all changes" trash button's tooltip label.
pub(crate) fn discard_all_label() -> &'static str {
    gitpoulpi_i18n::msg::status_list::DISCARD_ALL.get()
}
/// The header's red discard button: a ballot cross (U+2717, the same glyph as
/// every delete/discard entry of the menus), padded by a cell each side (no
/// brackets): the button is 3 wide. Not U+2716: kitty drew it with the colour
/// emoji font (Noto Color Emoji ignores VS15 there), which left only a red sliver.
/// (A wastebasket was tried; it does not render in every terminal font.)
pub(crate) const DISCARD_GLYPH: &str = " \u{2717} ";

/// The glyph [`render_status_header_buttons`] paints for the button.
pub(crate) fn discard_glyph() -> &'static str {
    DISCARD_GLYPH
}
/// The header's right-hand AI sparkle button (U+2726, same family as the
/// composer's old Generate label).
pub(crate) const EXPLAIN_GLYPH: &str = " \u{2726} ";
/// Empty cells between the pane's own collapse chevron (one cell in from
/// the border, drawn by `render_pane_collapse_button`) and the discard
/// button: a click meant for the chevron must not land on Discard All.
pub(crate) const DISCARD_GAP: u16 = 2;
/// Columns the header reserves left of the title: the collapse chevron (1),
/// the gap and the discard button (3).
pub(crate) const HEADER_LEFT_RESERVE: u16 = 1 + DISCARD_GAP + 3;
/// The Explain sparkle is an icon already (3 cells): it stays until the pane
/// has no room left for it beside the trash and a clipped title.
const EXPLAIN_MIN_WIDTH: u16 = 20;

/// Paints the working-changes header's two buttons over the top border
/// row of `area` (the pane's outer rect): the red trash at the far left
/// and the AI "Explain working changes" sparkle at the far right.
///
/// The title is drawn by the `Block` and padded with
/// [`HEADER_LEFT_RESERVE`] columns so it never sits under the trash. The
/// hit rects come from this same paint walk. The trash is always live
/// (its confirmation says "nothing to discard" itself); Explain is not drawn
/// when AI is off, and keeps its place (the title is clipped before it).
pub(crate) fn render_status_header_buttons(
    frame: &mut Frame,
    area: Rect,
    ai_enabled: bool,
    hovered: Option<&HitTarget>,
    hits: &mut Vec<(Rect, HitTarget)>,
) {
    if area.width < 12 || area.height == 0 {
        return;
    }
    let paint = |frame: &mut Frame,
                 rect: Rect,
                 glyph: &'static str,
                 target: HitTarget,
                 base: Style,
                 hits: &mut Vec<(Rect, HitTarget)>| {
        let style = if hovered == Some(&target) {
            filled_hover_style(base)
        } else {
            base
        };
        frame.render_clipped(Paragraph::new(Span::styled(glyph, style)), rect);
        hits.push((rect, target));
    };
    let discard = Rect::new(
        area.x + 2 + DISCARD_GAP,
        area.y,
        display_width(discard_glyph()) as u16,
        1,
    );
    paint(
        frame,
        discard,
        discard_glyph(),
        HitTarget::StatusDiscardAllButton,
        Style::default()
            .fg(th().status.danger)
            .bg(crate::theme::button_bg(false)),
        hits,
    );
    if ai_enabled && area.width >= EXPLAIN_MIN_WIDTH {
        let w = display_width(EXPLAIN_GLYPH) as u16;
        let explain = Rect::new(area.right() - 1 - w, area.y, w, 1);
        let base = Style::default()
            .fg(th().status.ai)
            .bg(crate::theme::button_bg(false));
        paint(
            frame,
            explain,
            EXPLAIN_GLYPH,
            HitTarget::StatusExplainButton,
            base,
            hits,
        );
    }
}

/// The working-changes pane's action row (Path / Tree, then Collapse all
/// or Expand all in Tree mode), laid out for `area` by the shared
/// `fit_bar` rule: what does not fit goes behind a `⋯`.
/// `collapse` is `None` in Path mode (no Collapse all button), else what
/// the button would do. Returns the line, the hit targets from the same
/// walk, and the `⋯` with what it hides.
///
/// The key hints (`Tree [m]`, `View all files [a]`) are shown only when
/// everything still fits with them: they must never push a control behind
/// the `⋯` or truncate its label. The keys are in the palette, the help and
/// the overflow menu's labels regardless.
pub(crate) fn status_toolbar_row(
    area: Rect,
    mode: FileListMode,
    collapse: Option<bool>,
    view_all: Option<bool>,
    lfs_only: Option<bool>,
    hovered: Option<&HitTarget>,
    _has_unstaged_changes: bool,
) -> (Line<'static>, Vec<(Rect, HitTarget)>, Option<OverflowSlot>) {
    let with = status_toolbar_row_hinted(
        area,
        mode,
        collapse,
        view_all,
        lfs_only,
        hovered,
        _has_unstaged_changes,
        true,
    );
    if with.2.is_none() {
        return with;
    }
    status_toolbar_row_hinted(
        area,
        mode,
        collapse,
        view_all,
        lfs_only,
        hovered,
        _has_unstaged_changes,
        false,
    )
}

#[allow(clippy::too_many_arguments)]
fn status_toolbar_row_hinted(
    area: Rect,
    mode: FileListMode,
    collapse: Option<bool>,
    view_all: Option<bool>,
    lfs_only: Option<bool>,
    hovered: Option<&HitTarget>,
    _has_unstaged_changes: bool,
    hints: bool,
) -> (Line<'static>, Vec<(Rect, HitTarget)>, Option<OverflowSlot>) {
    let km = crate::keymap::current();
    let mode_label = |m: FileListMode| -> String {
        match (m, hints) {
            (FileListMode::Path, _) => {
                format!(" {} ", gitpoulpi_i18n::msg::commit_detail::MODE_PATH.get())
            }
            (FileListMode::Tree, true) => format!(
                " {}{} ",
                gitpoulpi_i18n::msg::commit_detail::MODE_TREE.get(),
                crate::keymap::bracket(&km, crate::keymap::Action::StatusToggleFileMode)
            ),
            (FileListMode::Tree, false) => {
                format!(" {} ", gitpoulpi_i18n::msg::commit_detail::MODE_TREE.get())
            }
        }
    };
    let collapse_label = |c: bool| {
        format!(
            " {} ",
            if c {
                gitpoulpi_i18n::msg::button_bar::COLLAPSE_ALL.get()
            } else {
                gitpoulpi_i18n::msg::button_bar::EXPAND_ALL.get()
            }
        )
    };
    // The WIP row's own View all files checkbox -- the
    // same toggle (and the same `HitTarget`) as the commit-detail one.
    let view_all_label = |on: bool| -> String {
        let hint = if hints {
            crate::keymap::bracket(&km, crate::keymap::Action::StatusViewAllFiles)
        } else {
            String::new()
        };
        format!(
            "{} {}{hint}",
            if on { "[x]" } else { "[ ]" },
            gitpoulpi_i18n::msg::commit_detail::VIEW_ALL_FILES.get()
        )
    };
    // The `LFS only` pre-filter: offered only for a repository with LFS files.
    let lfs_only_label = |on: bool| -> String {
        let hint = if hints {
            crate::keymap::bracket(&km, crate::keymap::Action::StatusLfsOnly)
        } else {
            String::new()
        };
        format!(
            "{} {}{hint}",
            if on { "[x]" } else { "[ ]" },
            gitpoulpi_i18n::msg::commit_detail::LFS_ONLY.get()
        )
    };
    let mut items: Vec<BarItem<OverflowItem>> = [FileListMode::Path, FileListMode::Tree]
        .iter()
        .enumerate()
        .map(|(i, &m)| BarItem {
            key: OverflowItem::FileListMode(m),
            width: display_width(&mode_label(m)) as u16,
            // A segmented control: the two halves touch.
            gap_before: 0,
            priority: i as u8,
        })
        .collect();
    if let Some(c) = collapse {
        items.push(BarItem {
            key: OverflowItem::CollapseAll { collapse: c },
            width: display_width(&collapse_label(c)) as u16,
            gap_before: 2,
            priority: 2,
        });
    }
    if let Some(on) = view_all {
        items.push(BarItem {
            key: OverflowItem::ViewAllFiles,
            width: display_width(&view_all_label(on)) as u16,
            gap_before: 2,
            priority: 3,
        });
    }
    if let Some(on) = lfs_only {
        items.push(BarItem {
            key: OverflowItem::LfsOnly,
            width: display_width(&lfs_only_label(on)) as u16,
            gap_before: 2,
            priority: 4,
        });
    }
    let fit = fit_bar(&items, area.width, 1);
    let mut spans: Vec<Span<'static>> = Vec::new();
    let mut hits = Vec::new();
    let mut cursor = 0u16;
    for (x, item) in &fit.visible {
        if *x > cursor {
            spans.push(Span::raw(" ".repeat((x - cursor) as usize)));
        }
        let target = item.key.hit_target();
        let (label, style) = match item.key {
            OverflowItem::FileListMode(m) => (
                mode_label(m),
                segmented_style(mode == m, hovered == Some(&target)),
            ),
            OverflowItem::CollapseAll { collapse } => {
                // `Collapse all`/`Expand all` get hover feedback too.
                let base = Style::default().fg(th().ui.accent);
                let style = if hovered == Some(&target) {
                    hover_style(base)
                } else {
                    base
                };
                (collapse_label(collapse), style)
            }
            OverflowItem::ViewAllFiles => {
                let base = Style::default().fg(th().ui.muted);
                let style = if hovered == Some(&target) {
                    hover_style(base)
                } else {
                    base
                };
                (view_all_label(view_all.unwrap_or(false)), style)
            }
            OverflowItem::LfsOnly => {
                let base = Style::default().fg(th().ui.muted);
                let style = if hovered == Some(&target) {
                    hover_style(base)
                } else {
                    base
                };
                (lfs_only_label(lfs_only.unwrap_or(false)), style)
            }
            _ => (String::new(), Style::default()),
        };
        spans.push(Span::styled(label, style));
        hits.push((Rect::new(area.x + x, area.y, item.width, 1), target));
        cursor = x + item.width;
    }
    let mut overflow = None;
    if let Some((x, w)) = fit.overflow_button {
        if x > cursor {
            spans.push(Span::raw(" ".repeat((x - cursor) as usize)));
        }
        let target = HitTarget::BarOverflow(OverflowBar::StatusToolbar);
        let base = Style::default().fg(th().ui.accent);
        let style = if hovered == Some(&target) {
            hover_style(base)
        } else {
            base
        };
        spans.push(Span::styled(OVERFLOW_LABEL, style));
        let rect = Rect::new(area.x + x, area.y, w, 1);
        hits.push((rect, target));
        overflow = Some((rect, fit.overflow.clone()));
    }
    (Line::from(spans), hits, overflow)
}

/// Where a working-changes section header's bulk button goes: the full
/// ` Stage all changes ` when it fits beside the title, else the `⋯`
/// alone when *that* fits, else nothing. Returns the rect and
/// whether it is the `⋯`. Right-aligned one cell shy of the border.
pub(crate) fn status_bulk_button_layout(
    area: Rect,
    title: &str,
    bulk: &str,
) -> Option<(Rect, bool)> {
    // +1 for the left border the title starts after, +1 for the right
    // border the button stops short of.
    let room = area.width.saturating_sub(display_width(title) as u16 + 2);
    let item = BarItem {
        key: (),
        width: display_width(bulk) as u16 + 2,
        gap_before: 0,
        priority: 0,
    };
    let fit = fit_bar(&[item], room, 0);
    let (w, is_more) = match (fit.visible.first(), fit.overflow_button) {
        (Some((_, i)), _) => (i.width, false),
        (None, Some((_, w))) => (w, true),
        // Not even the `⋯` fits beside the title: the compact glyph is
        // drawn OVER the end of the title instead (always reachable); only
        // a pane too small to hold it between its borders loses it.
        (None, None) if area.width >= COMPACT_BULK_W + 2 => {
            return Some((
                Rect::new(
                    area.x + area.width - COMPACT_BULK_W - 1,
                    area.y,
                    COMPACT_BULK_W,
                    1,
                ),
                false,
            ))
        }
        (None, None) => return None,
    };
    Some((
        Rect::new(area.x + area.width - w - 1, area.y, w, 1),
        is_more,
    ))
}

/// Width of the compact bulk button (` + ` / ` - `) overlaid on a header
/// too narrow for the label or the `⋯`.
const COMPACT_BULK_W: u16 = 3;

/// The compact glyph standing for a section's bulk action.
fn compact_bulk_label(section: StatusSection) -> &'static str {
    match section {
        StatusSection::Unstaged => " + ",
        StatusSection::Staged => " - ",
    }
}

/// Draws a working-changes section header's bulk button (or its `⋯`, see
/// `status_bulk_button_layout`) and registers its hit target. Must run
/// before the header band's own target is registered: the inner-first
/// rule. Returns the `⋯`'s rect and what it hides when it drew one.
pub(crate) fn render_status_bulk_button(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    section: StatusSection,
    hovered: Option<&HitTarget>,
    hits: &mut Vec<(Rect, HitTarget)>,
) -> Option<OverflowSlot> {
    let bulk = section.bulk_label();
    let (button, is_more) = status_bulk_button_layout(area, title, bulk)?;
    let target = if is_more {
        HitTarget::BarOverflow(OverflowBar::StatusBulk(section))
    } else {
        HitTarget::StatusBulkButton(section)
    };
    // `Stage all changes`/`Unstage all changes` get hover feedback too.
    let style = if hovered == Some(&target) {
        Style::default()
            .fg(th().ui.accent)
            .patch(plain_button_style(true))
    } else {
        Style::default().fg(th().ui.accent)
    };
    let label = if is_more {
        OVERFLOW_LABEL.to_string()
    } else if button.width == COMPACT_BULK_W && display_width(bulk) as u16 + 2 != COMPACT_BULK_W {
        compact_bulk_label(section).to_string()
    } else {
        format!(" {bulk} ")
    };
    frame.render_clipped(
        Paragraph::new(Line::from(Span::styled(label, style))),
        button,
    );
    hits.push((button, target));
    is_more.then(|| (button, vec![OverflowItem::StatusBulk(section)]))
}
