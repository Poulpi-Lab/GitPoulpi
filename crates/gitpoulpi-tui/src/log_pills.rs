//! Ref pills in the Log pane: which refs sit on a commit, how a pill is
//! drawn (icons, lane color, combined local+remote pill, `+n` overflow)
//! and the "other refs on this commit" popup.

use super::*;

/// The "other refs on this commit" reveal popup's state: a real menu the
/// user can move the mouse into and click, not a tooltip that vanishes when
/// the cursor leaves the triggering row. See its render site in the draw
/// closure and the `MouseEventKind::Moved`/keyboard-open arms for which
/// transitions open, retarget, or close it.
///
/// `anchor_{col,row}` is captured once, at open time, and never recomputed
/// from later mouse positions, so the popup stays put instead of tracking
/// the cursor.
pub(crate) struct RefPopupState {
    pub(crate) commit_id: Oid,
    pub(crate) selected: usize,
    /// A popup opened via Right/Space on the Log pane ignores mouse
    /// movement entirely (only Esc/Enter/an elsewhere-click closes it) --
    /// a hover-opened one still tracks the mouse leaving both the
    /// triggering row and the popup itself, see the `Moved` handler.
    pub(crate) opened_via_keyboard: bool,
    pub(crate) anchor_col: u16,
    pub(crate) anchor_row: u16,
}

/// A gh-dash-style ref badge: an icon+name pill, colored by
/// kind, with a `checked_out` flag (bold + a leading checkmark) for the
/// local branch that's currently HEAD, and `combined_remote` when a local
/// branch's remote-tracking counterpart points at the *same* commit - see
/// `ref_pills_for_commit`'s doc comment.
pub(crate) struct RefPill {
    pub(crate) name: String,
    pub(crate) kind: RefRowKind,
    pub(crate) checked_out: bool,
    pub(crate) combined_remote: bool,
    /// For a tag: where it lives (`tag_where_ui`); `None` for a branch.
    pub(crate) tag_where: Option<gitpoulpi_core::domain::TagWhere>,
}

impl RefPill {
    /// The icon(s) after the name: the kind's icon, with the remote icon
    /// beside it for a combined local+remote branch pill, and a tag's
    /// where-marker (`tag_where_ui::glyph`) after the tag icon. A blank cell
    /// separates them (see `ref_pill_spans`'s note on ambiguous widths).
    pub(crate) fn icons(&self) -> String {
        let (_, icon) = ref_style(self.kind);
        let mut out = if self.combined_remote {
            format!("{icon} {REMOTE_ICON}")
        } else {
            icon.to_string()
        };
        if let Some(place) = self.tag_where {
            out.push(' ');
            out.push_str(crate::tag_where_ui::glyph(place));
        }
        out
    }

    /// Sort/display priority: a checked-out local branch always wins (so
    /// it's the one shown as the full pill when several refs share a
    /// commit), then other local branches, then remotes, then tags --
    /// The ref-priority convention.
    fn priority(&self) -> u8 {
        match (self.kind, self.checked_out) {
            (RefRowKind::Local, true) => 0,
            (RefRowKind::Local, false) => 1,
            (RefRowKind::Remote, _) => 2,
            (RefRowKind::Tag, _) => 3,
        }
    }
}

/// Which refs point at `commit`, as pills, sorted by
/// `RefPill::priority` (checked-out local first).
///
/// A local branch and its remote-tracking counterpart (e.g. `main` /
/// `origin/main`) render as *one* combined pill (both icons, one label) when
/// they point at this same commit, the common up-to-date case. A diverged
/// pair points at two *different* commits, so each commit's own call only
/// ever sees its side and there is nothing to deduplicate.
pub(crate) fn ref_pills_for_commit(
    commit: Oid,
    refs: &[gitpoulpi_core::domain::Ref],
) -> Vec<RefPill> {
    let mut locals: Vec<(String, bool)> = Vec::new();
    let mut remotes: Vec<(String, String)> = Vec::new(); // (remote alias, branch name)
    let mut tags: Vec<(String, Option<gitpoulpi_core::domain::TagWhere>)> = Vec::new();
    for r in refs.iter().filter(|r| r.target == commit) {
        match &r.kind {
            RefKind::LocalBranch { name } => locals.push((name.clone(), r.is_head)),
            RefKind::RemoteBranch { remote, name } => remotes.push((remote.clone(), name.clone())),
            RefKind::Tag { name } => tags.push((name.clone(), r.tag_where())),
        }
    }

    let mut pills = Vec::with_capacity(locals.len() + remotes.len() + tags.len());
    for (name, is_head) in locals {
        let combined_remote = if let Some(pos) = remotes.iter().position(|(_, rn)| *rn == name) {
            remotes.remove(pos);
            true
        } else {
            false
        };
        pills.push(RefPill {
            name,
            kind: RefRowKind::Local,
            checked_out: is_head,
            combined_remote,
            tag_where: None,
        });
    }
    for (_remote, name) in remotes {
        // Just the branch name, no `origin/`-style prefix -- the cloud
        // icon (`REMOTE_ICON`) already says "this is remote", the same
        // logic already applied to dropping the LOCAL section's per-row
        // branch icon in favor of a computer icon. Repeating the remote
        // alias in the text was pure redundancy once the icon carries it.
        pills.push(RefPill {
            name,
            kind: RefRowKind::Remote,
            checked_out: false,
            combined_remote: false,
            tag_where: None,
        });
    }
    for (name, tag_where) in tags {
        pills.push(RefPill {
            name,
            kind: RefRowKind::Tag,
            checked_out: false,
            combined_remote: false,
            tag_where,
        });
    }
    pills.sort_by_key(|p| (p.priority(), p.name.clone()));
    pills
}

/// The ref-reveal popup's own list: everything `ref_pills_for_commit`
/// returns *except* the first pill, which is already shown inline in the row
/// (see `ref_pill_spans`) behind its `+n` badge. Shared by the render site
/// and the keyboard Up/Down/Enter handlers so both agree on what index
/// `state.selected` refers to.
pub(crate) fn popup_pills(commit: Oid, refs: &[gitpoulpi_core::domain::Ref]) -> Vec<RefPill> {
    let mut pills = ref_pills_for_commit(commit, refs);
    if !pills.is_empty() {
        pills.remove(0);
    }
    pills
}

/// One popup row's label, in the segment order a collapsed pill uses minus
/// the checkout-count badge: leading pad, checkout mark, name, type icon(s),
/// trailing pad, with the ref name truncated so the label fits `max_width`
/// display columns.
///
/// The popup fills the Ref column's full width (see
/// `ref_expand_block_width`), so a long branch name would otherwise be
/// hard-clipped with no indicator. The name is budgeted after the chrome
/// exactly as `ref_pill_spans` budgets the anchor row's name, so a name that
/// fits on the row also fits in the popup and both truncate at the same
/// point.
pub(crate) fn popup_row_label_within(pill: &RefPill, max_width: usize) -> String {
    let icon = pill.icons();
    let marker = if pill.checked_out { "\u{2713} " } else { "" };
    let chrome = display_width(" ")
        + display_width(marker)
        + display_width(" ")
        + display_width(&icon)
        + display_width(" ");
    let name_budget = max_width.saturating_sub(chrome).max(1);
    let name = truncate_with_ellipsis(&pill.name, name_budget);
    format!(" {marker}{name} {icon} ")
}

/// The width BOTH the anchor row's padded pill and every popup row fill to
/// while a commit's reveal popup is open: the Ref column's whole usable
/// width.
///
/// One source of truth shared by the two renderers, because they used to
/// disagree. The anchor pill is padded inside a table cell and can never
/// exceed the Ref column, while the popup is a free-floating overlay capped
/// only to the frame; sizing the popup to its own widest label left the two
/// blocks ending at different columns (short labels stopped short of the
/// column edge, long ones ran past it).
///
/// Rows whose label is shorter are padded out to this; longer ones are
/// truncated to it (see `popup_row_label_within`) rather than bleeding into
/// the neighboring column.
pub(crate) fn ref_expand_block_width(widths: &LogColumnWidths) -> usize {
    widths.width_of(LogColumn::Ref) as usize
}

/// Renders the primary pill as two SEPARATE, individually-contiguous
/// colored chips:
///
///   [bg start] checkmark  name  icon(s) [bg end]  ' '  [bg start] +n [bg end]
///
/// The checkout mark + name + type icon(s) are ONE contiguous background
/// span (a background painted only under the label's own characters would
/// leave the icon/padding cells transparent), then a plain uncolored space
/// separates the name pill from the overflow badge, then the `+n` badge is
/// its OWN contiguous chip. At rest neither chip is widened to fill the
/// column; see `commit_row`'s `LogColumn::Ref` arm, which deliberately
/// applies no cell-wide background.
///
/// The exception is `popup_fill_width`: while THIS commit's reveal popup is
/// open, the name chip is padded out to the popup's own width (and the `+n`
/// badge is hidden, since the popup lists what it hid), so the row and the
/// rows expanded below it end at the same column and read as one block. That
/// widening is scoped to the hover.
///
/// `lane_color` is the commit's own graph-lane color (`muted_pill_bg` blends
/// it down for the chip; `contrasting_fg` picks readable text). The Refs
/// sidebar has its own backgroundless row renderer (`ref_sidebar_line`);
/// this coloring is Log-column-only.
///
/// `max_width` is the *total* display-column budget for everything returned
/// (pill + overflow badge). The ref name is truncated with an ellipsis to
/// fit, reserving room for the checkmark/icon/overflow-badge chrome first
/// (measured by display width, not char count; see `display_width`).
pub(crate) fn ref_pill_spans(
    pills: &[RefPill],
    max_width: usize,
    lane_color: Color,
    popup_fill_width: Option<usize>,
) -> Vec<Span<'static>> {
    let Some(first) = pills.first() else {
        return Vec::new();
    };
    // A combined local+remote pill (see `ref_pills_for_commit`) shows both
    // icons together in one badge. A blank cell between them matters: these
    // symbol/pictograph glyphs (⎇/☁/🏷) have Unicode's "Ambiguous" East Asian
    // width, which `unicode-width` (and so ratatui's buffer model) counts as 1
    // column, but many terminal fonts paint across 2, so adjacent glyphs eat
    // into each other. The gap cell gives that overflow somewhere to land.
    let icon = first.icons();
    // Hidden while the reveal popup for this exact commit is open -- the
    // popup already lists what `+n` was hiding, so the badge is redundant
    // (and visually doubled up) once expanded.
    let overflow = if pills.len() > 1 && popup_fill_width.is_none() {
        format!("+{}", pills.len() - 1)
    } else {
        String::new()
    };
    let checkmark = if first.checked_out { "\u{2713} " } else { "" };
    // Fixed segments: leading space, checkmark, name (budgeted below),
    // separator space, icon, trailing space, a plain separator space, then
    // the overflow badge's own chip (' +n ').
    let overflow_chrome = if overflow.is_empty() {
        0
    } else {
        display_width(" ") + display_width(" ") + display_width(&overflow) + display_width(" ")
    };
    let chrome_width = display_width(" ")
        + display_width(checkmark)
        + display_width(" ")
        + display_width(&icon)
        + display_width(" ")
        + overflow_chrome;
    let name_budget = max_width.saturating_sub(chrome_width).max(1);
    let name = truncate_with_ellipsis(&first.name, name_budget);
    let label = format!(" {checkmark}{name} {icon} ");
    // Padded to the popup's width only while this commit's popup is open
    // (see the doc comment). `.min(max_width)` keeps it inside the Ref
    // column, and `saturating_sub` leaves a pill that already fills the
    // column as it was. Still ONE span, so the fill stays contiguous.
    let label = match popup_fill_width {
        Some(target) => {
            let pad = target.min(max_width).saturating_sub(display_width(&label));
            format!("{label}{}", " ".repeat(pad))
        }
        None => label,
    };
    let bg = muted_pill_bg(lane_color);
    let fg = contrasting_fg(bg);
    let mut style = Style::default().bg(bg).fg(fg);
    if first.checked_out {
        style = style.add_modifier(Modifier::BOLD);
    }
    let mut spans = vec![Span::styled(label, style)];
    if !overflow.is_empty() {
        spans.push(Span::raw(" ")); // uncolored gap between the two chips
        spans.push(Span::styled(
            format!(" {overflow} "),
            Style::default().bg(bg).fg(fg),
        ));
    }
    spans
}

/// The ghost pill (`ui.graph_ghost_ref`): the same shape and width rules as
/// [`ref_pill_spans`], in faded colours - the muted ink, italic, on the lane's
/// chip colour blended a second time toward the background. A `deleted`
/// branch (inferred from a merge subject) is also struck through.
pub(crate) fn ghost_pill_spans(
    pill: &RefPill,
    max_width: usize,
    lane_color: Color,
    deleted: bool,
) -> Vec<Span<'static>> {
    let target = pill_blend_target();
    let bg = match muted_pill_bg(lane_color) {
        Color::Rgb(r, g, b) => {
            let fade = |c: u8, t: u8| ((u16::from(c) * 35 + u16::from(t) * 65) / 100) as u8;
            Color::Rgb(fade(r, target.0), fade(g, target.1), fade(b, target.2))
        }
        other => other,
    };
    let mut style = Style::default()
        .bg(bg)
        .fg(th().ui.muted)
        .add_modifier(Modifier::ITALIC);
    if deleted {
        style = style.add_modifier(Modifier::CROSSED_OUT);
    }
    ref_pill_spans(std::slice::from_ref(pill), max_width, lane_color, None)
        .into_iter()
        .map(|s| Span::styled(s.content, style))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::LANE_PALETTE;

    fn pill(name: &str, kind: RefRowKind, checked_out: bool) -> RefPill {
        RefPill {
            name: name.to_string(),
            kind,
            checked_out,
            combined_remote: false,
            tag_where: None,
        }
    }

    /// Renders the spans the way ratatui would: the plain text, plus a
    /// parallel mask of which columns actually carry a background. Both
    /// are measured in display columns, so wide icon glyphs are counted
    /// the way the terminal paints them.
    fn render_spans(spans: &[Span<'static>]) -> (String, Vec<bool>) {
        let mut text = String::new();
        let mut filled = Vec::new();
        for span in spans {
            let has_bg = span.style.bg.is_some();
            text.push_str(&span.content);
            filled.extend(std::iter::repeat_n(has_bg, display_width(&span.content)));
        }
        (text, filled)
    }

    #[test]
    fn a_pill_is_unchanged_while_its_reveal_popup_is_closed() {
        let pills = vec![
            pill("main", RefRowKind::Local, true),
            pill("v1.0", RefRowKind::Tag, false),
        ];
        let (text, filled) = render_spans(&ref_pill_spans(&pills, 40, LANE_PALETTE[0], None));
        // The `+n` badge is present, and the pill does NOT stretch to the
        // column -- the tight, at-rest chip the user explicitly asked for.
        assert!(text.contains("+1"), "closed popup keeps the overflow badge");
        assert!(
            filled.len() < 40,
            "at rest the pill must not fill the column, got {} of 40",
            filled.len()
        );
        // The uncolored separator between the name chip and the `+n` chip.
        assert!(
            filled.iter().any(|f| !f),
            "the two chips stay separated by an uncolored gap"
        );
    }

    /// While the popup is open the anchor pill fills the Ref column's WHOLE
    /// width, not just the popup's own widest label.
    #[test]
    fn an_open_popup_pads_the_anchor_pill_out_to_the_whole_ref_column() {
        let pills = vec![
            pill("main", RefRowKind::Local, true),
            pill("feature/some-longer-branch", RefRowKind::Local, false),
        ];
        let col_width = 60;

        let (text, filled) = render_spans(&ref_pill_spans(
            &pills,
            col_width,
            LANE_PALETTE[0],
            Some(col_width),
        ));

        assert!(
            !text.contains('+'),
            "the +n badge is hidden while the popup lists what it was hiding"
        );
        assert_eq!(
            filled.len(),
            col_width,
            "the anchor pill fills the Ref column edge to edge"
        );
        assert!(
            filled.iter().all(|f| *f),
            "the fill is contiguous - no holes inside the colored block"
        );
    }

    /// The fill is the column's width, so it can never spill into the
    /// neighboring column - `ref_pill_spans` still clamps independently
    /// of what it is handed.
    #[test]
    fn an_open_popup_never_pads_past_the_ref_column() {
        let pills = vec![
            pill("main", RefRowKind::Local, true),
            pill(&"x".repeat(200), RefRowKind::Local, false),
        ];

        let max_width = 24;
        // Deliberately over-wide: even then the pill stays in its column.
        let (_, filled) = render_spans(&ref_pill_spans(
            &pills,
            max_width,
            LANE_PALETTE[0],
            Some(max_width * 4),
        ));
        assert!(
            filled.len() <= max_width,
            "padding must stay inside the Ref column, got {} of {max_width}",
            filled.len()
        );
    }

    #[test]
    fn an_already_full_pill_is_left_alone_when_its_popup_opens() {
        // Name long enough that the chrome + truncated name already eat
        // the whole column: the padding must be a no-op, not an overflow.
        let pills = vec![
            pill(&"long-branch-name".repeat(4), RefRowKind::Local, false),
            pill("other", RefRowKind::Local, false),
        ];
        let max_width = 20;
        let (_, closed) = render_spans(&ref_pill_spans(&pills, max_width, LANE_PALETTE[0], None));
        let (_, open) = render_spans(&ref_pill_spans(
            &pills,
            max_width,
            LANE_PALETTE[0],
            Some(max_width),
        ));
        assert!(closed.len() <= max_width && open.len() <= max_width);
    }

    /// The shared width IS the Ref column's width - not the popup's widest
    /// label, not a constant. This pins the source, which is what actually
    /// regressed: both renderers were the right color and each was
    /// internally consistent, they just measured against different things.
    #[test]
    fn the_expand_block_width_is_the_ref_columns_own_width() {
        for area_width in [60u16, 120, 200] {
            let widths = LogColumnWidths::for_area_width(
                area_width,
                &std::collections::HashSet::new(),
                &std::collections::HashMap::new(),
            );
            assert_eq!(
                ref_expand_block_width(&widths),
                widths.width_of(LogColumn::Ref) as usize,
                "the expand fill must span the Ref column at area width {area_width}"
            );
        }
    }

    /// The shared width comes from the Ref column, and a popup label longer
    /// than it is truncated to fit rather than running past it - that
    /// overrun is exactly what left the two blocks ending in different
    /// places.
    #[test]
    fn popup_labels_are_truncated_to_the_shared_column_width() {
        let target = 24usize;
        let long = pill(
            &"release/very-long-branch-name".repeat(3),
            RefRowKind::Local,
            false,
        );
        let short = pill("v0.1.0", RefRowKind::Tag, false);

        assert!(
            display_width(&popup_row_label_within(&long, target)) <= target,
            "a long popup label is truncated to the column, not clipped past it"
        );
        assert!(
            display_width(&popup_row_label_within(&short, target)) <= target,
            "a short label is left alone (padding is the renderer's job)"
        );
        // Untruncated, that same label really is wider -- otherwise the
        // assertion above would pass for the wrong reason.
        assert!(
            display_width(&popup_row_label_within(&long, usize::MAX)) > target,
            "precondition: this label genuinely overflows the column"
        );
    }

    /// The columns of `row` in `buf` that actually carry a background, i.e.
    /// what a screenshot would show as the colored block.
    ///
    /// Two ratatui details this accounts for:
    ///
    /// - `bg.is_some()` is NOT the test: an untouched cell carries
    ///   `Some(Color::Reset)`, which would call every column painted and
    ///   make the assertion vacuous.
    /// - The cell *after* a double-width grapheme (the 💻/🔖 pill icons) is
    ///   blanked to a reset style and never emitted; the terminal paints the
    ///   glyph and its background across both columns. Scoring that
    ///   continuation cell on its own reports a hole that is not on screen,
    ///   so it inherits its predecessor.
    fn filled_columns(buf: &ratatui::buffer::Buffer, row: u16, width: u16) -> Vec<bool> {
        let mut out: Vec<bool> = Vec::with_capacity(width as usize);
        let mut skip_next = false;
        for x in 0..width {
            let cell = &buf[(x, row)];
            let painted = if skip_next {
                out.last().copied().unwrap_or(false)
            } else {
                !matches!(cell.style().bg, None | Some(ratatui::style::Color::Reset))
            };
            skip_next = display_width(cell.symbol()) == 2;
            out.push(painted);
        }
        out
    }

    /// The closest thing to a screenshot that works headlessly: render the
    /// anchor row and the popup rows into one real `Buffer` and compare the
    /// painted cells. Span-level assertions cannot catch a cell that fails
    /// to paint its padding.
    #[test]
    fn rendered_anchor_row_and_popup_rows_share_one_right_edge() {
        use ratatui::buffer::Buffer;
        use ratatui::widgets::Widget;

        let lane = LANE_PALETTE[2];
        let popup_rows = [
            pill("release/1.0", RefRowKind::Local, false),
            pill("v0.2.0", RefRowKind::Tag, false),
        ];
        let anchor = vec![
            pill("main", RefRowKind::Local, true),
            pill("release/1.0", RefRowKind::Local, false),
            pill("v0.2.0", RefRowKind::Tag, false),
        ];

        // Derived the way production derives it: ONE width, taken from the
        // real column allocator, handed to both renderers. Modelling this
        // faithfully is the point -- the defect was the two sides sourcing
        // their width from different places, so a test that hands both the
        // same hand-picked number could never have caught it.
        let widths = LogColumnWidths::for_area_width(
            120,
            &std::collections::HashSet::new(),
            &std::collections::HashMap::new(),
        );
        let target = ref_expand_block_width(&widths);
        let width = target as u16;
        let area = Rect::new(0, 0, width, 3);
        let mut buf = Buffer::empty(area);

        // Row 0: the anchor pill, popup open.
        Paragraph::new(Line::from(ref_pill_spans(
            &anchor,
            width as usize,
            lane,
            Some(target),
        )))
        .render(Rect::new(0, 0, width, 1), &mut buf);

        // Rows 1..: the popup's own bars, built exactly as the renderer does.
        let bg = muted_pill_bg(lane);
        for (i, p) in popup_rows.iter().enumerate() {
            let label = popup_row_label_within(p, target);
            let padded = format!(
                "{label}{}",
                " ".repeat(target.saturating_sub(display_width(&label)))
            );
            Paragraph::new(Line::from(Span::styled(
                padded,
                Style::default().bg(bg).fg(contrasting_fg(bg)),
            )))
            .render(Rect::new(0, 1 + i as u16, width, 1), &mut buf);
        }

        let last_filled = |row: u16| {
            filled_columns(&buf, row, width)
                .iter()
                .rposition(|f| *f)
                .expect("row should have a colored block")
        };

        let anchor_edge = last_filled(0);
        assert_eq!(
            anchor_edge,
            last_filled(1),
            "anchor row and first popup row must end at the same column"
        );
        assert_eq!(
            anchor_edge,
            last_filled(2),
            "anchor row and second popup row must end at the same column"
        );

        // And the anchor row's block is solid all the way there -- the
        // regression that produced "les trous" would show up right here.
        let anchor_fill = filled_columns(&buf, 0, width);
        assert!(
            anchor_fill[..=anchor_edge].iter().all(|f| *f),
            "no holes inside the anchor row's colored block: {anchor_fill:?}"
        );
    }
}
