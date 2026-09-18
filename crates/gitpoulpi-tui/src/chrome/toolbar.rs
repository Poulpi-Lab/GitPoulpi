//! The toolbar: its buttons, priorities, overflow and painting.

use super::*;
use gitpoulpi_i18n::msg;

/// A clickable toolbar button, mirroring a `Ctrl+G`
/// keybinding one-for-one where one exists. `Undo`/`Redo` drive the
/// daemon's journal (`undo_ui`), lit only while there is something they
/// can do.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ToolbarButton {
    Undo,
    Redo,
    Pull,
    Push,
    Branch,
    Tag,
    Stash,
    /// Pops `stash@{0}`; grayed out while there is no stash.
    Pop,
    /// `LFS ▾`: one button that is the dropdown itself (a click
    /// anywhere on it opens the menu). Hidden when LFS is turned off.
    Lfs,
    /// `Flow ▾`: the branching workflow's menu (Start, Finish, Sync...). One
    /// button that is the dropdown itself, shown only while the repository
    /// has a workflow (`flow::view::configured`).
    Flow,
    Search,
}

// Ordered Undo, Redo, Pull, Push, Branch, Stash and then the rest --
// Fetch has no button of its own (it lives in Pull's dropdown;
// the `f` key and the palette still run it), Pop follows Stash, and Tag
// sits next to Branch since both are "create a ref at HEAD". Stash and Pop
// use the inbox tray (arrow going in) and the outbox tray (arrow coming
// out): U+1F4E5 / U+1F4E4, plain Unicode like the
// rest of the bar (the toolbar uses no Nerd Font glyphs).
pub(crate) const TOOLBAR_BUTTONS: &[(&str, gitpoulpi_i18n::M, ToolbarButton)] = &[
    ("\u{21B6}", msg::toolbar::UNDO, ToolbarButton::Undo),
    ("\u{21B7}", msg::toolbar::REDO, ToolbarButton::Redo),
    ("\u{21E9}", msg::toolbar::PULL, ToolbarButton::Pull),
    ("\u{21E7}", msg::toolbar::PUSH, ToolbarButton::Push),
    ("\u{2387}", msg::toolbar::BRANCH, ToolbarButton::Branch),
    ("\u{1F516}", msg::toolbar::TAG, ToolbarButton::Tag),
    ("\u{1F4E5}", msg::toolbar::STASH, ToolbarButton::Stash),
    ("\u{1F4E4}", msg::toolbar::POP, ToolbarButton::Pop),
    ("\u{25A3}", msg::toolbar::LFS, ToolbarButton::Lfs),
    (
        "\u{21C4}",
        gitpoulpi_i18n::msg::flow::TOOLBAR,
        ToolbarButton::Flow,
    ),
    // Search sits at the far right of the toolbar, away from
    // the action buttons -- it's a view control, not a git operation.
    ("\u{2315}", msg::toolbar::SEARCH, ToolbarButton::Search),
];

/// A toolbar button's icon glyph (code) and its label in the current
/// language and git vocabulary (the catalog's `[toolbar]` section).
pub(crate) fn toolbar_parts(button: ToolbarButton) -> (&'static str, &'static str) {
    TOOLBAR_BUTTONS
        .iter()
        .find(|(_, _, b)| *b == button)
        .map(|(glyph, label, _)| (*glyph, label.get()))
        .unwrap_or(("", ""))
}

/// `"<glyph> <label>"`, the button's text as the overflow menu lists it.
pub(crate) fn toolbar_text(button: ToolbarButton) -> String {
    let (glyph, label) = toolbar_parts(button);
    format!("{glyph} {label}")
}

/// Toolbar icon glyphs (⇩⇧↻⎘⎇↶↷) are plain box-drawing/technical symbols,
/// not color emoji - they only carry color when we set it as a `Style::fg`
/// explicitly. Semantic per-action colors, not one flat white, per "toutes
/// tes icone en couleur" (every icon should be colored, not just some).
pub(crate) fn toolbar_icon_color(button: ToolbarButton) -> Color {
    // Softened/pastel, not the full-saturation `LANE_PALETTE` hues: a live
    // screenshot review flagged the first (bold, fully-saturated) attempt
    // as "trop pétant" (too flashy) against the toolbar's own smooth
    // background, which stays untouched -- the user explicitly said it was
    // the backgrounds they preferred as-is, not the icon color itself.
    match button {
        ToolbarButton::Pull => th().toolbar.pull, // soft blue, echoes the Remote pane's color
        ToolbarButton::Push => th().toolbar.push, // soft green, echoes Local
        ToolbarButton::Stash | ToolbarButton::Pop => th().toolbar.stash, // soft amber
        ToolbarButton::Branch => th().toolbar.branch, // soft purple
        ToolbarButton::Lfs => th().toolbar.lfs,   // soft lime: large files
        ToolbarButton::Flow => th().toolbar.branch, // the same purple as Branch: it makes branches
        ToolbarButton::Tag => th().toolbar.tag,   // soft gold, echoes the Tags pane's color
        ToolbarButton::Search => th().toolbar.search, // soft neutral: a view control, not an action
        ToolbarButton::Undo | ToolbarButton::Redo => th().toolbar.undo, // soft cyan
    }
}

/// Which toolbar buttons are live right now; the others are grayed
/// (icon and label) like Pop with no stash. Clicking a grayed one still
/// runs its handler, which says why it cannot.
#[derive(Debug, Clone, Copy)]
pub(crate) struct ToolbarEnabled {
    pub(crate) pop: bool,
    pub(crate) undo: bool,
    pub(crate) redo: bool,
    /// `general.undo_enabled` is off: Undo and Redo are left out of the
    /// bar (and so of its overflow) and the rest reflow.
    pub(crate) undo_hidden: bool,
    /// A pending auto-stash blocks Pull, Stash and Redo (the daemon
    /// refuses them; the banner holds the way out). Undo stays: with the
    /// operation that stashed on top of the journal it is *Start over*
    /// (the daemon asks first), otherwise the daemon says why not.
    pub(crate) auto_stash_blocks: bool,
    /// `git.lfs_enabled` resolves to off for this repository: the LFS
    /// button is left out (and so of the overflow) and the rest reflow.
    pub(crate) lfs_hidden: bool,
    /// Cells kept free at the bar's right end (plus one gap): the profile
    /// combobox when it is shown, `0` for nothing.
    pub(crate) right_reserve: u16,
}

impl ToolbarEnabled {
    #[cfg(test)]
    pub(crate) const ALL: ToolbarEnabled = ToolbarEnabled {
        pop: true,
        undo: true,
        redo: true,
        undo_hidden: false,
        auto_stash_blocks: false,
        lfs_hidden: false,
        // The width of the gear this reserve used to be.
        right_reserve: 3,
    };

    fn is_off(self, button: ToolbarButton) -> bool {
        if self.auto_stash_blocks
            && matches!(
                button,
                ToolbarButton::Pull | ToolbarButton::Stash | ToolbarButton::Redo
            )
        {
            return true;
        }
        match button {
            ToolbarButton::Pop => !self.pop,
            ToolbarButton::Undo => !self.undo,
            ToolbarButton::Redo => !self.redo,
            _ => false,
        }
    }
}

/// How important each toolbar button is when the toolbar runs out of
/// width (0 = kept longest; `button_bar.rs`). Pull and Push are what the
/// toolbar is for, so it stays; Branch next, then Undo and Redo (they
/// work now and are the buttons people reach for after a slip); Stash and
/// Pop; Search and Tag have shortcuts nobody forgets, so they go first.
pub(crate) fn toolbar_priority(button: ToolbarButton) -> u8 {
    match button {
        ToolbarButton::Pull => 0,
        ToolbarButton::Push => 1,
        ToolbarButton::Branch => 2,
        ToolbarButton::Undo => 3,
        ToolbarButton::Redo => 4,
        ToolbarButton::Stash => 5,
        ToolbarButton::Pop => 6,
        ToolbarButton::Search => 7,
        ToolbarButton::Tag => 8,
        ToolbarButton::Lfs => 9,
        ToolbarButton::Flow => 10,
    }
}

/// The gear's label, pinned to the toolbar's right edge.
pub(crate) const GEAR_LABEL: &str = " \u{2699} ";

/// The gear for kitty and Ghostty: GEAR WITHOUT HUB. kitty draws U+2699 as
/// an empty cell (a text-default emoji: the font fallback goes to the colour
/// emoji font and nothing shows), with or without a VS15/VS16 selector;
/// U+26ED is not emoji-capable and draws as a plain symbol. Same width as
/// [`GEAR_LABEL`], so no layout reads which one is painted.
pub(crate) const GEAR_LABEL_KITTY: &str = " \u{26ED} ";

/// The gear label this terminal draws, from its own variables. The unit
/// tests always get the standard one (a developer's own kitty must not turn
/// their buffer assertions red); `gear_label_for` and the PTY tests cover
/// the choice.
pub(crate) fn gear_label() -> &'static str {
    if cfg!(test) {
        return GEAR_LABEL;
    }
    static LABEL: std::sync::OnceLock<&'static str> = std::sync::OnceLock::new();
    LABEL.get_or_init(|| gear_label_for(&|name| std::env::var(name).ok()))
}

pub(crate) fn gear_label_for<F: Fn(&str) -> Option<String>>(get: &F) -> &'static str {
    let set = |name: &str| get(name).is_some_and(|v| !v.is_empty());
    let term = get("TERM").unwrap_or_default().to_ascii_lowercase();
    let program = get("TERM_PROGRAM").unwrap_or_default().to_ascii_lowercase();
    let kitty = set("KITTY_WINDOW_ID")
        || set("GHOSTTY_RESOURCES_DIR")
        || term == "xterm-kitty"
        || term == "xterm-ghostty"
        || program.contains("ghostty");
    if kitty {
        GEAR_LABEL_KITTY
    } else {
        GEAR_LABEL
    }
}

/// The toolbar, laid out for one width (or height): what to paint, the hit
/// targets (from the same walk), where the Push / Pull dropdowns hang, and
/// the `⋯` with the buttons it hides when they did not all fit.
pub(crate) struct ToolbarRender {
    /// A row toolbar's one line (empty for a column, see `rows`).
    pub(crate) line: Line<'static>,
    /// A column toolbar's lines, one per row of its area (empty for a row).
    pub(crate) rows: Vec<Line<'static>>,
    pub(crate) hits: Vec<(Rect, HitTarget)>,
    pub(crate) push_anchor: (u16, u16),
    pub(crate) pull_anchor: (u16, u16),
    /// Where the LFS dropdown hangs (under its button).
    pub(crate) lfs_anchor: (u16, u16),
    /// Where the Flow dropdown hangs (under its button).
    pub(crate) flow_anchor: (u16, u16),
    /// One column past the last drawn cell plus its trailing gap - what
    /// `render_gear` checks it would collide with.
    #[allow(dead_code)] // read by the chrome tests
    pub(crate) end_x: u16,
    pub(crate) overflow: Option<(Rect, Vec<OverflowItem>)>,
}

impl ToolbarRender {
    /// Paints the line or the rows into `area`.
    pub(crate) fn paint(&self, frame: &mut Frame, area: Rect) {
        if self.rows.is_empty() {
            frame.render_clipped(Paragraph::new(self.line.clone()), area);
        } else {
            frame.render_clipped(Paragraph::new(self.rows.clone()), area);
        }
    }
}

/// How the toolbar's buttons are placed along it, from the layout options.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ToolbarPlacement {
    pub(crate) align: gitpoulpi_config::ToolbarAlign,
    /// The pull request button sits at the far end instead of flowing with
    /// the other buttons.
    pub(crate) pr_pinned: bool,
    /// The side of the window the toolbar is on: dropdowns open away from it.
    pub(crate) edge: gitpoulpi_config::ToolbarPosition,
}

impl Default for ToolbarPlacement {
    fn default() -> Self {
        let l = ChromeLayout::default();
        Self {
            align: l.align,
            pr_pinned: l.pr_pinned,
            edge: l.toolbar,
        }
    }
}

impl ToolbarPlacement {
    /// The previous look: a row at the top, centered, the chip first.
    #[cfg(test)]
    pub(crate) const LEGACY: ToolbarPlacement = ToolbarPlacement {
        align: gitpoulpi_config::ToolbarAlign::Center,
        pr_pinned: false,
        edge: gitpoulpi_config::ToolbarPosition::Top,
    };

    /// The row a dropdown hangs below (`anchored_popup` adds one): under a
    /// top toolbar the popup starts one row below the bar, as it always
    /// did; on any other edge it hangs from the bar's own row, so a popup
    /// without room below flips above it.
    fn popup_row(self, bar_y: u16) -> u16 {
        match self.edge {
            gitpoulpi_config::ToolbarPosition::Top => bar_y + 1,
            _ => bar_y,
        }
    }
}

/// A toolbar row's items: the buttons and, when it flows, the linked pull
/// request chip.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Slot {
    Button(ToolbarButton),
    Chip,
}

/// The dropdown arrow after a button's label, and its width in cells.
const DROPDOWN: &str = "\u{25BE} ";

fn dropdown_w() -> u16 {
    display_width(DROPDOWN) as u16
}

/// Push, Pull and Undo: a button plus a `▾` chip with its own hit rect.
fn is_split(b: ToolbarButton) -> bool {
    matches!(
        b,
        ToolbarButton::Push | ToolbarButton::Pull | ToolbarButton::Undo
    )
}

/// Buttons that paint a `▾`: the split ones, and LFS and Flow, which carry
/// the arrow but are one hit rect (the whole button opens the dropdown).
fn has_arrow(b: ToolbarButton) -> bool {
    is_split(b) || matches!(b, ToolbarButton::Lfs | ToolbarButton::Flow)
}

/// The buttons this toolbar shows, with their widths and priorities.
fn button_items(enabled: ToolbarEnabled) -> Vec<BarItem<ToolbarButton>> {
    TOOLBAR_BUTTONS
        .iter()
        .filter(|(_, _, button)| {
            !(enabled.undo_hidden && matches!(button, ToolbarButton::Undo | ToolbarButton::Redo))
                && !((enabled.lfs_hidden || !crate::build_features::LFS)
                    && *button == ToolbarButton::Lfs)
                && !(crate::flow::view::configured().is_none() && *button == ToolbarButton::Flow)
        })
        .map(|(glyph, label, button)| BarItem {
            key: *button,
            width: display_width(&format!(" {glyph} {} ", label.get())) as u16
                + if has_arrow(*button) { dropdown_w() } else { 0 },
            gap_before: 1,
            priority: toolbar_priority(*button),
        })
        .collect()
}

/// A button's chip background: lighter while hovered.
fn button_bg(hovered: Option<&HitTarget>, button: ToolbarButton) -> Color {
    if hovered == Some(&HitTarget::Toolbar(button)) {
        th().chrome.toolbar_bg_hover
    } else {
        th().chrome.toolbar_bg
    }
}

/// A button's icon and label colors: grayed while it has nothing to do.
fn button_fgs(enabled: ToolbarEnabled, button: ToolbarButton) -> (Color, Color) {
    if enabled.is_off(button) {
        (
            th().chrome.toolbar_disabled_fg,
            th().chrome.toolbar_disabled_fg,
        )
    } else {
        (toolbar_icon_color(button), th().chrome.toolbar_fg)
    }
}

/// The linked pull request chip's style.
fn chip_style(chip: &ToolbarChip, hovered: Option<&HitTarget>) -> Style {
    let target = HitTarget::Hosting(chip.hit.clone());
    if hovered == Some(&target) {
        plain_button_style(true)
    } else {
        let mut style = Style::default().fg(chip.fg);
        match chip.bg {
            Some(bg) => style = style.bg(bg).add_modifier(Modifier::BOLD),
            None => style = style.add_modifier(Modifier::DIM),
        }
        style
    }
}

/// Lays the toolbar out in `area` (its row), keeping room for the profile
/// combobox at the right end. `hovered` is the event loop's `hovered_target`.
///
/// Split buttons (Push, Pull) are one `BarItem` including their `▾` chip,
/// so a button and its dropdown are always kept or hidden together; the
/// overflow menu then lists both the button and its dropdown.
#[cfg(test)]
pub(crate) fn toolbar_render(
    area: Rect,
    hovered: Option<&HitTarget>,
    enabled: ToolbarEnabled,
) -> ToolbarRender {
    toolbar_render_with(area, hovered, None, enabled, ToolbarPlacement::LEGACY)
}

/// The room the buttons keep before the linked-PR chip is dropped: the
/// gear, the `⋯` and one button.
pub(super) const TOOLBAR_MIN_BUTTON_ROOM: u16 = 16;

/// [`toolbar_render`] with the hosting chip (`GL !53`, `GH #12`, `+ PR`).
/// Pinned (the default) it ends the row, left of the profile combobox;
/// otherwise it is the first of the group, which `place.align` then places
/// as a whole. A bar too narrow to keep [`TOOLBAR_MIN_BUTTON_ROOM`] for the
/// buttons drops it. Its hit rect comes from the same walk that paints it,
/// like every other bar item.
///
/// The group is packed at the start, centered in the row or packed at the
/// end; when some buttons overflow it stays packed at the start against the
/// `⋯`.
pub(crate) fn toolbar_render_with(
    area: Rect,
    hovered: Option<&HitTarget>,
    chip: Option<&ToolbarChip>,
    enabled: ToolbarEnabled,
    place: ToolbarPlacement,
) -> ToolbarRender {
    let buttons = button_items(enabled);
    let chip_label = chip.map(|c| format!(" {} ", c.text));
    let chip_w = chip_label.as_deref().map_or(0, |l| display_width(l) as u16);
    let show_chip = chip.is_some() && area.width >= chip_w + 1 + TOOLBAR_MIN_BUTTON_ROOM;
    let pinned = show_chip && place.pr_pinned;
    let flows = show_chip && !place.pr_pinned;
    // Room for what is pinned at the right end (the profile combobox) and
    // the gap before it: the buttons are what give way.
    let gear_w = enabled.right_reserve;
    let limit = if gear_w > 0 && area.width > gear_w {
        area.width - gear_w - 1
    } else {
        area.width
    };
    let group_limit = if pinned {
        limit.saturating_sub(chip_w + 1)
    } else {
        limit
    };
    let mut items: Vec<BarItem<Slot>> = Vec::with_capacity(buttons.len() + 1);
    if flows {
        items.push(BarItem {
            key: Slot::Chip,
            width: chip_w,
            gap_before: 1,
            // Kept after Pull and Push, dropped before everything else.
            priority: 1,
        });
    }
    items.extend(buttons.iter().map(|b| BarItem {
        key: Slot::Button(b.key),
        width: b.width,
        gap_before: b.gap_before,
        priority: b.priority,
    }));
    let fit = fit_bar(&items, group_limit, 1);
    let lead: u16 = if fit.overflow_button.is_none() && !fit.visible.is_empty() {
        let group_w = fit.end();
        match place.align {
            gitpoulpi_config::ToolbarAlign::Start => 0,
            gitpoulpi_config::ToolbarAlign::Center => {
                let want = area.width.saturating_sub(group_w) / 2;
                want.min(group_limit.saturating_sub(group_w))
            }
            gitpoulpi_config::ToolbarAlign::End => group_limit.saturating_sub(group_w),
        }
    } else {
        0
    };

    let row = place.popup_row(area.y);
    let mut spans: Vec<Span<'static>> = Vec::new();
    let mut hits = Vec::new();
    let mut push_anchor = (area.x, row);
    let mut pull_anchor = (area.x, row);
    let mut lfs_anchor = (area.x, row);
    let mut flow_anchor = (area.x, row);
    let mut cursor = 0u16;
    for (x, item) in &fit.visible {
        let x = &(*x + lead);
        if *x > cursor {
            spans.push(Span::raw(" ".repeat((x - cursor) as usize)));
        }
        let button = match item.key {
            Slot::Chip => {
                // `flows` implies a chip and its label.
                if let (Some(chip), Some(label)) = (chip, chip_label.clone()) {
                    spans.push(Span::styled(label, chip_style(chip, hovered)));
                    hits.push((
                        Rect::new(area.x + x, area.y, chip_w, 1),
                        HitTarget::Hosting(chip.hit.clone()),
                    ));
                }
                cursor = x + item.width;
                continue;
            }
            Slot::Button(b) => b,
        };
        let (icon, label) = toolbar_parts(button);
        // Toolbar buttons keep a background chip -- a button needs to
        // read as a button regardless of selection state. Both values are
        // deliberately far brighter than a dark terminal theme's own
        // background can plausibly be: the previous, subtler pair was
        // indistinguishable from WezTerm's near-navy default in a real
        // screenshot.
        let bg = button_bg(hovered, button);
        // Icon glyph colored per action (`toolbar_icon_color`), label
        // text white -- "toutes tes icone en couleur".
        // A button with nothing to do (Pop with no stash, Undo with no
        // history) is grayed: icon and label.
        let (icon_fg, label_fg) = button_fgs(enabled, button);
        spans.push(Span::styled(" ", Style::default().bg(bg)));
        spans.push(Span::styled(icon, Style::default().bg(bg).fg(icon_fg)));
        spans.push(Span::styled(
            format!(" {label} "),
            Style::default().bg(bg).fg(label_fg),
        ));
        let main_w = item.width - if has_arrow(button) { dropdown_w() } else { 0 };
        if matches!(button, ToolbarButton::Lfs | ToolbarButton::Flow) {
            // One rect over the label and its arrow, one background.
            spans.push(Span::styled(
                DROPDOWN,
                Style::default().bg(bg).fg(th().chrome.toolbar_fg),
            ));
            hits.push((
                Rect::new(area.x + x, area.y, item.width, 1),
                HitTarget::Toolbar(button),
            ));
            if button == ToolbarButton::Lfs {
                lfs_anchor = (area.x + x, row);
            } else {
                flow_anchor = (area.x + x, row);
            }
            cursor = x + item.width;
            continue;
        }
        hits.push((
            Rect::new(area.x + x, area.y, main_w, 1),
            HitTarget::Toolbar(button),
        ));
        // Push and Pull are split buttons: the `▾` chip
        // beside each opens its dropdown. A separate rect right after the
        // button's own, so neither covers the other.
        if is_split(button) {
            let chip_x = area.x + x + main_w;
            let arrow_bg = if hovered == Some(&HitTarget::ToolbarDropdown(button)) {
                th().chrome.toolbar_bg_hover
            } else {
                th().chrome.toolbar_arrow_bg
            };
            spans.push(Span::styled(
                DROPDOWN,
                Style::default().bg(arrow_bg).fg(th().chrome.toolbar_fg),
            ));
            hits.push((
                Rect::new(chip_x, area.y, dropdown_w(), 1),
                HitTarget::ToolbarDropdown(button),
            ));
            // A dropdown opened from the keyboard or the palette hangs
            // under the same chip a click would use.
            if button == ToolbarButton::Push {
                push_anchor = (chip_x, row);
            } else {
                pull_anchor = (chip_x, row);
            }
        }
        cursor = x + item.width;
    }
    let mut overflow = None;
    if let Some((x, w)) = fit.overflow_button {
        if x > cursor {
            spans.push(Span::raw(" ".repeat((x - cursor) as usize)));
        }
        let hovered_more = hovered == Some(&HitTarget::BarOverflow(OverflowBar::Toolbar));
        spans.push(Span::styled(
            OVERFLOW_LABEL,
            Style::default()
                .bg(if hovered_more {
                    th().chrome.toolbar_bg_hover
                } else {
                    th().chrome.toolbar_bg
                })
                .fg(th().chrome.toolbar_fg),
        ));
        let rect = Rect::new(area.x + x, area.y, w, 1);
        hits.push((rect, HitTarget::BarOverflow(OverflowBar::Toolbar)));
        cursor = x + w;
        let hidden: Vec<ToolbarButton> = fit
            .overflow
            .iter()
            .filter_map(|s| match s {
                Slot::Button(b) => Some(*b),
                Slot::Chip => None,
            })
            .collect();
        // A hidden split button's dropdown, opened from the keyboard or
        // the palette, hangs under the `⋯` instead of the left edge.
        for b in &hidden {
            match b {
                ToolbarButton::Push => push_anchor = (rect.x, row),
                ToolbarButton::Pull => pull_anchor = (rect.x, row),
                ToolbarButton::Lfs => lfs_anchor = (rect.x, row),
                ToolbarButton::Flow => flow_anchor = (rect.x, row),
                _ => {}
            }
        }
        let items = hidden
            .iter()
            .flat_map(|b| {
                let mut v = vec![OverflowItem::Toolbar(*b)];
                if is_split(*b) {
                    v.push(OverflowItem::ToolbarDropdown(*b));
                }
                v
            })
            .collect();
        overflow = Some((rect, items));
    }
    if pinned {
        if let (Some(chip), Some(label)) = (chip, chip_label) {
            let x = limit - chip_w;
            if x > cursor {
                spans.push(Span::raw(" ".repeat((x - cursor) as usize)));
            }
            spans.push(Span::styled(label, chip_style(chip, hovered)));
            hits.push((
                Rect::new(area.x + x, area.y, chip_w, 1),
                HitTarget::Hosting(chip.hit.clone()),
            ));
        }
    }
    ToolbarRender {
        line: Line::from(spans),
        rows: Vec::new(),
        hits,
        push_anchor,
        pull_anchor,
        lfs_anchor,
        flow_anchor,
        end_x: area.x + fit.end() + lead + 1,
        overflow,
    }
}

/// The narrowest a toolbar column gets, in cells.
const COLUMN_MIN_WIDTH: u16 = 8;

/// The width a vertical toolbar needs: its widest button (the label and a
/// `▾`), at most [`super::COLUMN_MAX_WIDTH`]. The linked pull request chip
/// is clipped to it.
pub(crate) fn toolbar_column_width(enabled: ToolbarEnabled) -> u16 {
    button_items(enabled)
        .iter()
        .map(|i| i.width)
        .max()
        .unwrap_or(0)
        .clamp(COLUMN_MIN_WIDTH, super::COLUMN_MAX_WIDTH)
}

/// `text` cut to `width` cells (display width, never splitting a glyph),
/// then padded with spaces to exactly `width`.
fn fit_cells(text: &str, width: u16) -> String {
    let width = width as usize;
    let mut out = String::new();
    let mut used = 0;
    for ch in text.chars() {
        let w = display_width(ch.encode_utf8(&mut [0; 4]));
        if used + w > width {
            break;
        }
        out.push(ch);
        used += w;
    }
    out.push_str(&" ".repeat(width - used));
    out
}

/// Lays the toolbar out as a column in `area`: one button per row, split
/// buttons keeping their `▾` chip at the row's end. Buttons that do not fit
/// in the height go behind a `⋯` row (the same overflow menu). The linked
/// pull request chip is pinned to the bottom (or first in the group when it
/// flows), above the profile combobox, which takes the last row.
///
/// `place.edge` is `Left` or `Right`; dropdowns open beside the column.
pub(crate) fn toolbar_column(
    area: Rect,
    hovered: Option<&HitTarget>,
    chip: Option<&ToolbarChip>,
    enabled: ToolbarEnabled,
    place: ToolbarPlacement,
) -> ToolbarRender {
    let w = area.width;
    let h = area.height;
    let arrow_w = dropdown_w().min(w);
    let buttons = button_items(enabled);
    let chip_label = chip.map(|c| format!(" {} ", c.text));
    // The profile field and the pinned chip each take a row at the bottom,
    // when the column is tall enough to also keep a few buttons.
    let profile_row = enabled.right_reserve > 0 && h >= 6 && w >= COLUMN_MIN_WIDTH;
    let show_chip = chip.is_some() && h >= 6;
    let pinned = show_chip && place.pr_pinned;
    let flows = show_chip && !place.pr_pinned;
    let bottom_rows = u16::from(profile_row) + u16::from(pinned);
    let group_limit = h.saturating_sub(bottom_rows);

    let mut items: Vec<BarItem<Slot>> = Vec::with_capacity(buttons.len() + 1);
    if flows {
        items.push(BarItem {
            key: Slot::Chip,
            width: 1,
            gap_before: 0,
            priority: 1,
        });
    }
    items.extend(buttons.iter().map(|b| BarItem {
        key: Slot::Button(b.key),
        width: 1,
        gap_before: 0,
        priority: b.priority,
    }));
    let fit = fit_bar_sized(&items, group_limit, 0, 1);
    let lead: u16 = if fit.overflow_button.is_none() && !fit.visible.is_empty() {
        let n = fit.end();
        match place.align {
            gitpoulpi_config::ToolbarAlign::Start => 0,
            gitpoulpi_config::ToolbarAlign::Center => group_limit.saturating_sub(n) / 2,
            gitpoulpi_config::ToolbarAlign::End => group_limit.saturating_sub(n),
        }
    } else {
        0
    };

    // Dropdowns open beside the column, level with their button.
    let side_x = match place.edge {
        gitpoulpi_config::ToolbarPosition::Right => area.x,
        _ => area.right(),
    };
    let anchor_at = |y: u16| (side_x, y.saturating_sub(1));
    let mut rows: Vec<Line<'static>> = vec![Line::raw(""); h as usize];
    let mut hits = Vec::new();
    let mut push_anchor = anchor_at(area.y);
    let mut pull_anchor = push_anchor;
    let mut lfs_anchor = push_anchor;
    let mut flow_anchor = push_anchor;

    let chip_row = |chip: &ToolbarChip, label: &str| -> Line<'static> {
        Line::from(Span::styled(fit_cells(label, w), chip_style(chip, hovered)))
    };
    for (offset, item) in &fit.visible {
        let r = (offset + lead) as usize;
        let y = area.y + offset + lead;
        let button = match item.key {
            Slot::Chip => {
                if let (Some(chip), Some(label)) = (chip, chip_label.as_deref()) {
                    rows[r] = chip_row(chip, label);
                    hits.push((
                        Rect::new(area.x, y, w, 1),
                        HitTarget::Hosting(chip.hit.clone()),
                    ));
                }
                continue;
            }
            Slot::Button(b) => b,
        };
        let (icon, label) = toolbar_parts(button);
        let bg = button_bg(hovered, button);
        let (icon_fg, label_fg) = button_fgs(enabled, button);
        let main_w = if has_arrow(button) { w - arrow_w } else { w };
        // " <icon> <label>", the label clipped to what the row leaves and
        // padded so the chip's background fills the row.
        let icon_w = display_width(icon) as u16;
        let label_text = fit_cells(label, main_w.saturating_sub(icon_w + 2));
        let mut spans = vec![
            Span::styled(" ", Style::default().bg(bg)),
            Span::styled(icon, Style::default().bg(bg).fg(icon_fg)),
            Span::styled(
                format!(" {label_text}"),
                Style::default().bg(bg).fg(label_fg),
            ),
        ];
        match button {
            ToolbarButton::Lfs | ToolbarButton::Flow => {
                spans.push(Span::styled(
                    DROPDOWN,
                    Style::default().bg(bg).fg(th().chrome.toolbar_fg),
                ));
                hits.push((Rect::new(area.x, y, w, 1), HitTarget::Toolbar(button)));
                if button == ToolbarButton::Lfs {
                    lfs_anchor = anchor_at(y);
                } else {
                    flow_anchor = anchor_at(y);
                }
            }
            _ => {
                hits.push((Rect::new(area.x, y, main_w, 1), HitTarget::Toolbar(button)));
                if is_split(button) {
                    let arrow_bg = if hovered == Some(&HitTarget::ToolbarDropdown(button)) {
                        th().chrome.toolbar_bg_hover
                    } else {
                        th().chrome.toolbar_arrow_bg
                    };
                    spans.push(Span::styled(
                        DROPDOWN,
                        Style::default().bg(arrow_bg).fg(th().chrome.toolbar_fg),
                    ));
                    hits.push((
                        Rect::new(area.x + main_w, y, arrow_w, 1),
                        HitTarget::ToolbarDropdown(button),
                    ));
                    if button == ToolbarButton::Push {
                        push_anchor = anchor_at(y);
                    } else if button == ToolbarButton::Pull {
                        pull_anchor = anchor_at(y);
                    }
                }
            }
        }
        rows[r] = Line::from(spans);
    }
    let mut overflow = None;
    if let Some((offset, _)) = fit.overflow_button {
        let y = area.y + offset;
        let hovered_more = hovered == Some(&HitTarget::BarOverflow(OverflowBar::Toolbar));
        let bg = if hovered_more {
            th().chrome.toolbar_bg_hover
        } else {
            th().chrome.toolbar_bg
        };
        rows[offset as usize] = Line::from(Span::styled(
            fit_cells(OVERFLOW_LABEL, w),
            Style::default().bg(bg).fg(th().chrome.toolbar_fg),
        ));
        let rect = Rect::new(area.x, y, w, 1);
        hits.push((rect, HitTarget::BarOverflow(OverflowBar::Toolbar)));
        let hidden: Vec<ToolbarButton> = fit
            .overflow
            .iter()
            .filter_map(|s| match s {
                Slot::Button(b) => Some(*b),
                Slot::Chip => None,
            })
            .collect();
        for b in &hidden {
            match b {
                ToolbarButton::Push => push_anchor = anchor_at(y),
                ToolbarButton::Pull => pull_anchor = anchor_at(y),
                ToolbarButton::Lfs => lfs_anchor = anchor_at(y),
                ToolbarButton::Flow => flow_anchor = anchor_at(y),
                _ => {}
            }
        }
        let list = hidden
            .iter()
            .flat_map(|b| {
                let mut v = vec![OverflowItem::Toolbar(*b)];
                if is_split(*b) {
                    v.push(OverflowItem::ToolbarDropdown(*b));
                }
                v
            })
            .collect();
        overflow = Some((rect, list));
    }
    if pinned {
        if let (Some(chip), Some(label)) = (chip, chip_label.as_deref()) {
            let r = h - u16::from(profile_row) - 1;
            rows[r as usize] = chip_row(chip, label);
            hits.push((
                Rect::new(area.x, area.y + r, w, 1),
                HitTarget::Hosting(chip.hit.clone()),
            ));
        }
    }
    ToolbarRender {
        line: Line::default(),
        rows,
        hits,
        push_anchor,
        pull_anchor,
        lfs_anchor,
        flow_anchor,
        end_x: area.x + w,
        overflow,
    }
}
