//! Palette popup rows, collapse strips and buttons, gear, close and tab-dir buttons, badges.

use super::*;

/// Whether the File Insight overlay owns the keyboard this frame.
///
/// The arm that consults this **must** sit above the `input_mode`
/// dispatch in the event `match`, exactly like the help overlay's does.
/// That ordering is what keeps the overlay's `Enter` (go to this line's
/// commit) and the Diff view's `Enter` (stage the selected lines) from
/// firing on the same keystroke: while the overlay is open the earlier
/// arm matches and the diff's key handling is never reached; once it is
/// closed this returns false and the diff behaves exactly as before.
///
/// The `InputMode::Normal` half matters too. A text field or the palette
/// can be opened over the overlay, and those own the keyboard - typing
/// "blame" into the palette must not have its `b` and `e` swallowed as
/// overlay navigation.
/// The command palette's geometry, as one frame drew it.
///
/// Captured during the draw rather than recomputed from
/// `centered_rect(62, 72, ..)` afterwards: a second, independent guess at
/// the same layout drifts the day either changes, which is the rule the
/// Log table and the repository overlay already follow.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PalettePopup {
    /// The whole popup, including its input box and hint band. A click
    /// outside this dismisses the palette; a click inside it that is on
    /// no row does nothing at all.
    pub(crate) popup: Rect,
    /// The list block. It has LEFT|RIGHT borders only, so every *row* of
    /// it is a content row while its first and last *columns* are not.
    pub(crate) list: Rect,
    /// The index the top visible row shows - the list scrolls, so row 0
    /// is not command 0.
    pub(crate) window_start: usize,
    /// How many commands the current filter matched.
    pub(crate) count: usize,
    /// The segmented prefilter's cells (the theme picker's All / Dark /
    /// Light / Colored), `Rect::ZERO` where there is none.
    pub(crate) segments: [Rect; 5],
}

thread_local! {
    /// Where the pointer rests while the palette or the tab picker is open
    /// (`None` otherwise): those popups consume every mouse event before the
    /// hit list is consulted, so their row hover reads this instead.
    static POPUP_POINTER: std::cell::Cell<Option<(u16, u16)>> = const { std::cell::Cell::new(None) };
}

/// Records the pointer for the open palette / tab picker (`None` clears it).
pub(crate) fn set_popup_pointer(pos: Option<(u16, u16)>) {
    POPUP_POINTER.with(|p| p.set(pos));
}

thread_local! {
    /// The close cross of the open palette / picker, as painted last frame.
    static POPUP_CROSS: std::cell::Cell<Option<Rect>> = const { std::cell::Cell::new(None) };
}

/// Paints the close cross on a list popup's top border (hover from the popup
/// pointer), remembers it for [`popup_cross_at`] and returns it for
/// `Win::decorate`.
pub(crate) fn paint_popup_cross(buf: &mut ratatui::buffer::Buffer, popup: Rect) -> Option<Rect> {
    let hovered = popup_pointer().is_some_and(|(x, y)| {
        Rect::new(popup.right().saturating_sub(5).max(popup.x), popup.y, 3, 1)
            .contains((x, y).into())
    });
    let cross = close_cross::paint(buf, popup, hovered);
    POPUP_CROSS.with(|c| c.set(cross));
    cross
}

/// Whether `(col, row)` is on the open list popup's close cross.
pub(crate) fn popup_cross_at(col: u16, row: u16) -> bool {
    POPUP_CROSS
        .with(|c| c.get())
        .is_some_and(|r| r.contains((col, row).into()))
}

/// Where the pointer rests over the open palette / tab picker.
pub(crate) fn popup_pointer() -> Option<(u16, u16)> {
    POPUP_POINTER.with(|p| p.get())
}

/// Lights the palette / tab-picker row under the pointer (not the selected
/// one: that already carries the selection band).
pub(crate) fn paint_popup_row_hover(
    buf: &mut ratatui::buffer::Buffer,
    popup: &PalettePopup,
    selected: usize,
) {
    let Some((col, row)) = POPUP_POINTER.with(|p| p.get()) else {
        return;
    };
    let Some(index) = palette_row_at(popup, col, row) else {
        return;
    };
    if index != selected {
        buf.set_style(
            Rect::new(popup.list.x + 1, row, popup.list.width.saturating_sub(2), 1),
            Style::default().bg(th().ui.hover),
        );
    }
}

/// The command under `(col, row)`, if any.
///
/// Both axes, like every other hit test in this file: a click at a row's
/// height but outside the popup is not a click on that row.
pub(crate) fn palette_row_at(popup: &PalettePopup, col: u16, row: u16) -> Option<usize> {
    if popup.count == 0 || popup.list.width < 3 {
        return None;
    }
    // The block's own border columns are not rows.
    if col <= popup.list.x || col + 1 >= popup.list.right() {
        return None;
    }
    if row < popup.list.y || row >= popup.list.bottom() {
        return None;
    }
    let index = popup.window_start + (row - popup.list.y) as usize;
    if index < popup.count {
        Some(index)
    } else {
        None
    }
}

/// The gear in the toolbar's right-hand corner, and its hit target.
///
/// A function rather than ten lines inside the draw closure so the one
/// thing that can silently go wrong here is testable: the gear is
/// *right-anchored* while the button row grows from the left, so at a
/// narrow width the two collide. `buttons_end_x` is one column past the
/// last toolbar button; when the gear would not clear it, nothing is
/// drawn **and nothing is registered** - a hit target with no glyph
/// under it is a click that appears to do nothing, which is worse than
/// no gear at all.
/// Paints a collapsed pane's restore strip: `PANE_COLLAPSED_WIDTH` (1)
/// column, full height, with a chevron pointing toward the center --
/// a thin strip with a chevron. Registered as
/// `HitTarget::RestorePane` over the whole strip, not just the chevron's
/// cell: it is one column wide already, so the whole thing IS the
/// affordance.
///
/// Called *after* the pane's own (now near-zero-width, since `area` is
/// only `PANE_COLLAPSED_WIDTH` wide) render pass, so this is the last
/// thing painted into that column and nothing the collapsed pane tried to
/// draw underneath survives - see this function's call sites for why
/// suppressing that render pass outright was not worth the churn: a
/// bordered block one column wide already has an inner width of zero
/// (`Block::inner` saturates), so nothing it draws can register a hit
/// target with a non-zero width.
pub(crate) fn render_collapsed_pane_strip(
    frame: &mut Frame,
    area: Rect,
    side: PaneSide,
    hovered: bool,
    hits: &mut Vec<(Rect, HitTarget)>,
) {
    if area.width == 0 || area.height == 0 {
        return;
    }
    // Points toward the center it would restore into: Refs collapses to
    // the left edge so its chevron points right; Status collapses to the
    // right edge so its chevron points left. Text-presentation BMP
    // arrows, not the box-drawing double angle quotes -- the project's
    // standing glyph rule (no VS16 dependence, checked against the same
    // font table `commit-graph.md` keeps).
    let glyph = match side {
        PaneSide::Refs => '\u{203A}',   // "›"
        PaneSide::Status => '\u{2039}', // "‹"
    };
    let bg = button_bg(hovered);
    let buf = frame.buffer_mut();
    let frame_area = buf.area;
    for y in area.y..area.y.saturating_add(area.height) {
        if area.x >= frame_area.x + frame_area.width || y >= frame_area.y + frame_area.height {
            continue;
        }
        let mid = area.y + area.height / 2;
        let symbol = if y == mid {
            glyph.to_string()
        } else {
            " ".to_string()
        };
        buf[(area.x, y)]
            .set_symbol(&symbol)
            .set_style(Style::default().bg(bg).fg(th().ui.on_button));
    }
    hits.push((area, HitTarget::RestorePane(side)));
}

/// The collapse chevron drawn on an expanded pane's top border, at the
/// edge nearest the center - the corner where collapse arrows usually
/// sit. Suppressed when the pane is too narrow to draw it without
/// overlapping the block's own corner glyph, the gear's own rule ("too
/// narrow to draw means not registered" - a hit target with no glyph
/// under it is a click that appears to do nothing).
pub(crate) fn render_pane_collapse_button(
    frame: &mut Frame,
    pane_area: Rect,
    side: PaneSide,
    hovered: bool,
    hits: &mut Vec<(Rect, HitTarget)>,
) {
    if pane_area.width < 4 || pane_area.height == 0 {
        return;
    }
    // Points AWAY from the center -- the direction the pane will travel
    // when it collapses -- mirroring the restore strip's chevron, which
    // points back toward it.
    let glyph = match side {
        PaneSide::Refs => '\u{2039}',   // "‹"
        PaneSide::Status => '\u{203A}', // "›"
    };
    let x = match side {
        // One cell in from the pane's own right border, so it never
        // collides with the rounded corner glyph.
        PaneSide::Refs => pane_area.right().saturating_sub(2),
        // One cell in from the left border, symmetrically.
        PaneSide::Status => pane_area.x + 1,
    };
    let rect = Rect::new(x, pane_area.y, 1, 1);
    let fg = if hovered {
        th().ui.accent
    } else {
        th().ui.muted
    };
    frame.render_clipped(
        Paragraph::new(Span::styled(glyph.to_string(), Style::default().fg(fg))),
        rect,
    );
    hits.push((rect, HitTarget::CollapsePane(side)));
}

/// The REMOTE section header's `+` (Add remote): plain at rest
/// (`plain_button_style`), and hovered it grows into a "+ Add Remote"
/// tooltip-style label.
///
/// **Reported bug, fixed here**: hovering made the `+` vanish
/// entirely. The cause wasn't a color collision (`plain_button_style`'s
/// hover fill and text color are never equal) - it was that the
/// *whole button, glyph included*, was gated on there being room for
/// whichever label was about to be drawn, and the *hovered* label
/// (`" + Add Remote "`, 15 columns) is much wider than the resting one
/// (`" + "`, 3 columns). On a sidebar wide enough for the short glyph
/// but not the long tooltip, the gate passed at rest and failed the
/// instant the mouse arrived, so the button - not just the extra text --
/// disappeared under the pointer. Fixed by gating the *glyph* on room for
/// the short label only, and falling back to the short label (still
/// hover-styled) when the wide one doesn't fit; the tooltip text is a
/// nice-to-have that degrades gracefully instead of taking the whole
/// button down with it. Pinned by
/// `the_add_remote_button_never_disappears_on_hover_even_when_narrow`.
pub(crate) fn render_add_remote_button(
    frame: &mut Frame,
    area: Rect,
    section_title_width: u16,
    hovered: bool,
    header_hovered: bool,
    hits: &mut Vec<(Rect, HitTarget)>,
) {
    render_header_plus(
        frame,
        area,
        section_title_width,
        &format!(" + {} ", gitpoulpi_i18n::msg::sidebar::ADD_REMOTE.get()),
        hovered,
        header_hovered,
        HitTarget::AddRemoteButton,
        hits,
    );
}

/// The WORKTREES header's `+` (create a worktree), drawn exactly like the
/// REMOTE one.
pub(crate) fn render_add_worktree_button(
    frame: &mut Frame,
    area: Rect,
    section_title_width: u16,
    hovered: bool,
    header_hovered: bool,
    hits: &mut Vec<(Rect, HitTarget)>,
) {
    render_header_plus(
        frame,
        area,
        section_title_width,
        &format!(
            " + {} ",
            gitpoulpi_i18n::msg::worktree_manage::PLUS_TIP.get()
        ),
        hovered,
        header_hovered,
        HitTarget::AddWorktreeButton,
        hits,
    );
}

/// A sidebar section header's `+`: the REMOTE one (Add remote), and the
/// PULL REQUESTS / ISSUES ones (New pull request / New issue), all drawn
/// by this one function so the glyph, color, alignment and hover
/// tooltip are identical. `full` is the hovered tooltip label, spaces
/// included (`" + Add Remote "`).
#[allow(clippy::too_many_arguments)]
pub(crate) fn render_header_plus(
    frame: &mut Frame,
    area: Rect,
    section_title_width: u16,
    full: &str,
    hovered: bool,
    header_hovered: bool,
    target: HitTarget,
    hits: &mut Vec<(Rect, HitTarget)>,
) {
    render_header_button(
        frame,
        area,
        section_title_width,
        (" + ", full),
        0,
        hovered,
        header_hovered,
        target,
        hits,
    );
}

/// A header button: `labels.0` at rest, `labels.1` (the tooltip, spaces
/// included) hovered. `skip` is the width of the buttons already to its
/// right (the ISSUES header's Board button sits left of the `+`).
#[allow(clippy::too_many_arguments)]
pub(crate) fn render_header_button(
    frame: &mut Frame,
    area: Rect,
    section_title_width: u16,
    labels: (&str, &str),
    skip: u16,
    hovered: bool,
    header_hovered: bool,
    target: HitTarget,
    hits: &mut Vec<(Rect, HitTarget)>,
) {
    let (minimal, full) = labels;
    let minimal_w = display_width(minimal) as u16;
    // Too narrow to sit beside the title: the bare glyph is drawn OVER the
    // end of the header text instead (always reachable); only a pane too
    // small to hold the glyph between its borders loses it.
    if area.width < minimal_w + skip + 2 {
        return;
    }
    let full_w = display_width(full) as u16;
    // Hovered, the tooltip is spelled out wherever the borders leave room
    // for it, over the end of the title when need be: it is a tooltip, so
    // covering the title while the pointer is on it is fine, and a
    // sidebar is rarely wide enough for title and label side by side.
    let (label, w) = if hovered && area.width >= full_w + skip + 2 {
        (full, full_w)
    } else {
        (minimal, minimal_w)
    };
    let rect = Rect::new(area.x + area.width - w - skip - 1, area.y, w, 1);
    // Over the title (no room beside it): drawn while the header or the
    // button is hovered so the title stays whole otherwise; always
    // clickable.
    let overlay = area.width <= section_title_width + minimal_w + skip + 4;
    if !overlay || hovered || header_hovered {
        frame.render_clipped(
            Paragraph::new(Span::styled(label, plain_button_style(hovered))),
            rect,
        );
    }
    hits.push((rect, target));
}

/// Blank cells kept between the gear and the quit button when the title bar
/// has room (one when it is tight), so a click aimed at the gear never quits.
const GEAR_GAP: u16 = 2;

/// Cells the gear and its gap take left of the quit button.
pub(crate) fn gear_reserve() -> u16 {
    display_width(gear_label()) as u16 + GEAR_GAP
}

/// The gear sits immediately left of the quit button (`close`, what
/// [`render_close_app_button`] returned), at least one blank cell away from
/// it. `left_limit_x` is the first column it may use (the end of the title
/// and of "Open in"); when it does not fit there, nothing is drawn and
/// nothing is registered.
pub(crate) fn render_gear(
    frame: &mut Frame,
    close: Rect,
    left_limit_x: u16,
    hovered: bool,
    hits: &mut Vec<(Rect, HitTarget)>,
) -> Option<Rect> {
    // Text-presentation, like every other toolbar glyph (see
    // `toolbar_icon_color`): nothing here relies on a VS16 selector, so
    // it cannot flip to an emoji hole under a different font.
    let gear = gear_label();
    let width = display_width(gear) as u16;
    let x = [GEAR_GAP, 1]
        .into_iter()
        .filter_map(|gap| close.x.checked_sub(gap + width))
        .find(|x| *x >= left_limit_x)?;
    let rect = Rect::new(x, close.y, width, 1);
    frame.render_clipped(
        Paragraph::new(Line::from(Span::styled(gear, plain_button_style(hovered)))),
        rect,
    );
    hits.push((rect, HitTarget::Gear));
    Some(rect)
}

/// The global Search button, the magnifier and its label (` ⌕ Global search `),
/// flush against the gear's left edge (`gear`, what [`render_gear`]
/// returned). It gives way first: when the label does not fit after
/// `left_limit_x` (the end of the title and of "Open in") the bare magnifier
/// is drawn, and when that does not fit either nothing is drawn and nothing
/// is registered, so the gear and the quit button keep their room. The palette's
/// `Search: Workspace...` and `Ctrl+Alt+F` always reach it.
pub(crate) fn render_search_button(
    frame: &mut Frame,
    gear: Rect,
    left_limit_x: u16,
    hovered: bool,
    hits: &mut Vec<(Rect, HitTarget)>,
) {
    let full = format!(
        " {} {} ",
        crate::log_search::MAGNIFIER,
        gitpoulpi_i18n::msg::tab_bar::SEARCH.get()
    );
    let icon = format!(" {} ", crate::log_search::MAGNIFIER);
    let Some((label, width, x)) = [full, icon].into_iter().find_map(|label| {
        let width = display_width(&label) as u16;
        let x = gear.x.checked_sub(width).filter(|x| *x >= left_limit_x)?;
        Some((label, width, x))
    }) else {
        return;
    };
    let rect = Rect::new(x, gear.y, width, 1);
    frame.render_clipped(
        Paragraph::new(Line::from(Span::styled(label, plain_button_style(hovered)))),
        rect,
    );
    hits.push((rect, HitTarget::GlobalSearchButton));
}

/// The octopus at the title bar's left, before the title: opens About.
/// Emoji presentation on purpose (the maintainer asked for the color 🐙); two
/// cells wide with one blank on each side (like the other title bar buttons'
/// padding), which the hit rect covers.
pub(crate) const ABOUT_LABEL: &str = " \u{1F419} ";

/// Paints the octopus at `(x, y)` and registers it; returns the column after
/// it.
pub(crate) fn render_about_button(
    frame: &mut Frame,
    x: u16,
    y: u16,
    hovered: bool,
    hits: &mut Vec<(Rect, HitTarget)>,
) -> u16 {
    let width = display_width(ABOUT_LABEL) as u16;
    let rect = Rect::new(x, y, width, 1);
    if frame.area().right() >= x + width
        && crate::title_logo::render(
            frame.buffer_mut(),
            Rect::new(x + 1, y, 2, 1),
            crate::about::way_index(),
            hovered.then(|| plain_button_style(true).bg).flatten(),
        )
    {
        // The picture takes the emoji's two cells (and paints the hover
        // background under itself); a blank on each side keeps the highlight.
        for bx in [x, x + 3] {
            frame.render_clipped(
                Paragraph::new(Span::styled(" ", plain_button_style(hovered))),
                Rect::new(bx, y, 1, 1),
            );
        }
    } else {
        frame.render_clipped(
            Paragraph::new(Line::from(Span::styled(
                ABOUT_LABEL,
                plain_button_style(hovered),
            ))),
            rect,
        );
    }
    hits.push((rect, HitTarget::About));
    x + width
}

/// The `\u{D7}` quit button in the outer title bar's top-right corner --
/// mouse parity for `q`. Anchored to `title_area`'s right
/// edge one column shy of the block's own corner glyph, the same
/// "registered rect must not collide with anything else" rule `render_gear`
/// already follows (the gear sits just left of this button). `title_end_x` is one column past the last character of
/// the rendered title (left border, then the title text `Block` draws by
/// default) - passed in rather than recomputed from the title string a
/// second time, so this can never disagree with what ratatui actually put
/// on screen; when the button would land at or before it, nothing is drawn
/// and nothing is registered, exactly the gear's own narrow-width rule.
pub(crate) fn render_close_app_button(
    frame: &mut Frame,
    title_area: Rect,
    title_end_x: u16,
    hovered: bool,
    hits: &mut Vec<(Rect, HitTarget)>,
) -> Option<Rect> {
    const CLOSE: &str = " \u{D7} ";
    let width = display_width(CLOSE) as u16;
    if title_area.width <= width + 1 {
        return None;
    }
    // One column left of the border's own right corner glyph.
    let right_edge = title_area.right().saturating_sub(1);
    // A title that runs into the button is covered, not the button dropped:
    // the quit button keeps its place at any width (`Ctrl+Q` is its twin).
    let _ = title_end_x;
    let rect = Rect::new(right_edge - width, title_area.y, width, 1);
    frame.render_clipped(
        Paragraph::new(Line::from(Span::styled(CLOSE, plain_button_style(hovered)))),
        rect,
    );
    hits.push((rect, HitTarget::CloseApp));
    Some(rect)
}

/// Cells a tab-folder button takes: "Quit & cd here" its label, "Open in" a
/// combobox field.
fn button_width(button: crate::tab_dir::TabDirButton) -> u16 {
    match button {
        crate::tab_dir::TabDirButton::Terminal => crate::open_in_bar::button_width(),
        other => display_width(&other.label()) as u16,
    }
}

/// The tab bar's room for the tab-folder buttons: how much of `area` they
/// need (labels plus one blank column between), or `None` when the bar is too
/// narrow to give up that much and still show a few tabs.
pub(crate) fn tab_dir_buttons_width(
    buttons: &[crate::tab_dir::TabDirButton],
    area: Rect,
) -> Option<u16> {
    const MIN_TABS_ROOM: u16 = 30;
    if buttons.is_empty() {
        return None;
    }
    let width = buttons.iter().map(|b| button_width(*b)).sum::<u16>() + buttons.len() as u16;
    (area.width >= width + MIN_TABS_ROOM).then_some(width)
}

/// Paints the tab-folder buttons at the right edge of `area` (the tab bar
/// row), registering each from the rect it was painted at. `hovered` is the
/// button under the pointer.
pub(crate) fn render_tab_dir_buttons(
    frame: &mut Frame,
    area: Rect,
    buttons: &[crate::tab_dir::TabDirButton],
    width: u16,
    hovered: Option<crate::tab_dir::TabDirButton>,
    hits: &mut Vec<(Rect, HitTarget)>,
) {
    let mut x = area.right().saturating_sub(width);
    for button in buttons {
        let w = button_width(*button);
        let rect = Rect::new(x, area.y, w, 1);
        if *button == crate::tab_dir::TabDirButton::Terminal {
            // "Open in" is a combobox like every other choice field.
            crate::open_in_bar::render_button(frame.buffer_mut(), rect, hovered == Some(*button));
        } else {
            frame.render_clipped(
                Paragraph::new(Line::from(Span::styled(
                    button.label(),
                    plain_button_style(hovered == Some(*button)),
                ))),
                rect,
            );
        }
        hits.push((rect, HitTarget::TabDir(*button)));
        x += w + 1;
    }
}

/// The badges `Ctrl+N` navigation mode draws over each focusable sub-pane -
/// the visual half of "press a badge to focus that sub-pane" (see
/// `FocusTarget` and `NavigationAction::FocusByBadge`). `panes` pairs each
/// sub-pane's area with the character that focuses it, built by zipping
/// `assign_navigation_badges`'s output back onto the `Rect`s the same
/// frame just drew - a sub-pane with nothing to show (a collapsed side
/// pane, an empty Refs section) is simply left out by the caller, the
/// same way a click on it has nothing to focus. Areas too narrow for the
/// 3-cell badge are skipped rather than drawn truncated, the gear's own
/// "no room, no glyph" rule.
pub(crate) fn render_navigation_badges(frame: &mut Frame, panes: &[(Rect, char)]) {
    for (area, digit) in panes {
        if area.width < 3 || area.height == 0 {
            continue;
        }
        let rect = Rect::new(area.x + 1, area.y, 3, 1);
        frame.render_clipped(
            Paragraph::new(Span::styled(
                format!(" {digit} "),
                Style::default()
                    .bg(th().ui.accent)
                    .fg(th().chrome.on_bright)
                    .add_modifier(Modifier::BOLD),
            )),
            rect,
        );
    }
}
