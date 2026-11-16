//! Responsive button bars: one layout rule for every row of buttons that
//! can run out of width.
//!
//! When a bar is too narrow for all its buttons, the ones that do not fit
//! collapse into a single overflow button (`⋯`, U+22EF) at the end of the
//! bar. Clicking it, or pressing Enter on a row of the menu it opens, runs
//! the *same* code the hidden button's own click would have run: each menu
//! row is an `OverflowItem` naming the `HitTarget` the button would have
//! registered, and the event loop re-dispatches the choice as a click on
//! that target (see `run_in_repo`'s overflow arm). There is no second copy
//! of any button's action.
//!
//! # Which buttons go first
//!
//! Each bar gives its buttons a `priority` (0 = most important). When the
//! bar is too narrow, buttons are dropped from the *least* important end
//! (highest priority number first, ties broken by dropping the one further
//! right) until the kept ones plus the `⋯` fit. The kept buttons stay in
//! their original left-to-right order: shrinking a bar must never reshuffle
//! what is left of it.
//!
//! # One walk, one set of rects
//!
//! `fit_bar` returns the x offset of every kept button and of the `⋯`, and
//! every caller both paints and registers its hit targets from that one
//! value (the `log_table` rule in `where-we-are.md`): two walks over the
//! same geometry is how a click lands a cell off what was painted. Widths
//! are always `display_width`, never `chars().count()`: the toolbar's
//! `🔖 Tag` is two cells wide, and counting it as one shifts the hit rects
//! of every button after it.

use super::*;

/// The overflow button as drawn: the glyph with a one-cell margin either
/// side, like every other bar button. U+22EF is a plain mathematical symbol
/// with no emoji presentation, in the same block as the `⋮` the repository
/// manager uses, so it cannot flip presentation under a different style.
pub(crate) const OVERFLOW_LABEL: &str = " \u{22EF} ";

/// The overflow button's width in cells.
pub(crate) fn overflow_width() -> u16 {
    display_width(OVERFLOW_LABEL) as u16
}

/// One button a bar wants to show.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BarItem<K> {
    pub(crate) key: K,
    /// The button's own width in cells, margins included.
    pub(crate) width: u16,
    /// Blank cells between this button and the one before it. Ignored for
    /// whichever button ends up first on the bar, so dropping the leading
    /// button never leaves a hole at the start.
    pub(crate) gap_before: u16,
    /// 0 is the most important: the last to be dropped.
    pub(crate) priority: u8,
}

/// Where a bar's buttons go, for one width.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BarFit<K> {
    /// The buttons drawn, left to right, each with its x offset from the
    /// bar's start (the gap before it already skipped).
    pub(crate) visible: Vec<(u16, BarItem<K>)>,
    /// The buttons that did not fit, in bar order - what the overflow
    /// menu lists.
    pub(crate) overflow: Vec<K>,
    /// `(x offset, width)` of the `⋯`, when anything overflowed and the `⋯`
    /// itself fits. `None` with a non-empty `overflow` means the bar is too
    /// narrow even for the `⋯`: those buttons are then reachable only by
    /// their shortcut and the palette.
    pub(crate) overflow_button: Option<(u16, u16)>,
}

impl<K: Copy> BarFit<K> {
    /// One past the last cell drawn (0 for an empty bar).
    pub(crate) fn end(&self) -> u16 {
        let buttons = self.visible.last().map(|(x, i)| x + i.width).unwrap_or(0);
        let more = self.overflow_button.map(|(x, w)| x + w).unwrap_or(0);
        buttons.max(more)
    }
}

/// Lays `items` out left to right in `width` cells, dropping the least
/// important ones behind a `⋯` when they do not all fit. `overflow_gap`
/// is the blank space between the last kept button and the `⋯`.
pub(crate) fn fit_bar<K: Copy>(items: &[BarItem<K>], width: u16, overflow_gap: u16) -> BarFit<K> {
    fit_bar_sized(items, width, overflow_gap, overflow_width())
}

/// [`fit_bar`] with the overflow button's own extent: a vertical bar counts
/// rows, so its `⋯` takes one, not the three cells of a row bar's.
pub(crate) fn fit_bar_sized<K: Copy>(
    items: &[BarItem<K>],
    width: u16,
    overflow_gap: u16,
    more_extent: u16,
) -> BarFit<K> {
    let layout = |kept: &[bool]| -> (Vec<(u16, BarItem<K>)>, u32) {
        let mut x: u32 = 0;
        let mut out = Vec::new();
        for (item, keep) in items.iter().zip(kept) {
            if !keep {
                continue;
            }
            if !out.is_empty() {
                x += u32::from(item.gap_before);
            }
            out.push((x as u16, *item));
            x += u32::from(item.width);
        }
        (out, x)
    };

    let mut kept = vec![true; items.len()];
    let (visible, end) = layout(&kept);
    if end <= u32::from(width) {
        return BarFit {
            visible,
            overflow: Vec::new(),
            overflow_button: None,
        };
    }

    // Least important first; among equals, the one further right.
    let mut drop_order: Vec<usize> = (0..items.len()).collect();
    drop_order.sort_by(|&a, &b| {
        items[b]
            .priority
            .cmp(&items[a].priority)
            .then_with(|| b.cmp(&a))
    });
    let more_w = u32::from(more_extent);
    for index in drop_order {
        kept[index] = false;
        let (visible, end) = layout(&kept);
        let more_x = if visible.is_empty() {
            0
        } else {
            end + u32::from(overflow_gap)
        };
        if more_x + more_w <= u32::from(width) {
            return BarFit {
                visible,
                overflow: overflowed(items, &kept),
                overflow_button: Some((more_x as u16, more_w as u16)),
            };
        }
    }
    BarFit {
        visible: Vec::new(),
        overflow: items.iter().map(|i| i.key).collect(),
        overflow_button: None,
    }
}

fn overflowed<K: Copy>(items: &[BarItem<K>], kept: &[bool]) -> Vec<K> {
    items
        .iter()
        .zip(kept)
        .filter(|(_, keep)| !**keep)
        .map(|(i, _)| i.key)
        .collect()
}

/// Which bar an overflow button belongs to - carried by
/// `HitTarget::BarOverflow` so a click knows whose hidden buttons to list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum OverflowBar {
    /// The top toolbar (Undo ... Search).
    Toolbar,
    /// The Diff view's button bar.
    DiffBar,
    /// The working-changes pane's Path / Tree / Collapse All row.
    StatusToolbar,
    /// A working-changes section header's bulk button.
    StatusBulk(StatusSection),
    /// The commit composer's summary row: Generate and `#` when it is narrow.
    Composer,
}

/// One row of an overflow menu: a hidden button. Running it means
/// dispatching `hit_target()` exactly as if the button had been clicked.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OverflowItem {
    Toolbar(ToolbarButton),
    /// The `▾` beside Push / Pull, hidden along with its button.
    ToolbarDropdown(ToolbarButton),
    Diff(DiffBarButton),
    FileListMode(FileListMode),
    CollapseAll {
        collapse: bool,
    },
    StatusBulk(StatusSection),
    /// The working-changes toolbar's View all files checkbox.
    ViewAllFiles,
    /// The composer's AI Generate button, folded into the summary row's `⋯`.
    AiGenerate,
    /// The composer's `#` issue-reference button, folded the same way.
    ComposerIssueRef,
    /// The toolbar's pull request chip, when the bar had no room for it.
    HostingChip {
        /// The `GL !53` chip (a linked request), else the dim `+ PR` one.
        linked: bool,
    },
}

impl OverflowItem {
    /// The click this row stands for.
    pub(crate) fn hit_target(self) -> HitTarget {
        match self {
            OverflowItem::Toolbar(b) => HitTarget::Toolbar(b),
            OverflowItem::ToolbarDropdown(b) => HitTarget::ToolbarDropdown(b),
            OverflowItem::Diff(b) => HitTarget::DiffBarButton(b),
            OverflowItem::FileListMode(m) => HitTarget::FileListModeButton(m),
            OverflowItem::CollapseAll { collapse } => HitTarget::CollapseAllButton { collapse },
            OverflowItem::StatusBulk(s) => HitTarget::StatusBulkButton(s),
            OverflowItem::ViewAllFiles => HitTarget::ViewAllFilesCheckbox,
            OverflowItem::AiGenerate => HitTarget::AiGenerate,
            OverflowItem::ComposerIssueRef => HitTarget::ComposerIssueRef,
            OverflowItem::HostingChip { linked: true } => {
                HitTarget::Hosting(crate::hosting_ui::HostingHit::LinkedPrChip)
            }
            OverflowItem::HostingChip { linked: false } => {
                HitTarget::Hosting(crate::hosting_ui::HostingHit::NewPrChip)
            }
        }
    }

    /// The menu row's text: the button's own label where that reads on
    /// its own, spelled out where the bar relies on its neighbors (a bare
    /// `↑` means nothing out of the Diff bar).
    pub(crate) fn label(self) -> String {
        match self {
            OverflowItem::Toolbar(b) => crate::chrome::toolbar_text(b),
            OverflowItem::ToolbarDropdown(ToolbarButton::Push) => {
                format!(
                    "\u{21E7} {}",
                    gitpoulpi_i18n::msg::toolbar::OVERFLOW_PUSH_MENU.get()
                )
            }
            OverflowItem::ToolbarDropdown(ToolbarButton::Undo) => {
                format!(
                    "\u{21B6} {}",
                    gitpoulpi_i18n::msg::undo_safety::OVERFLOW_UNDO_MENU.get()
                )
            }
            OverflowItem::ToolbarDropdown(_) => {
                format!(
                    "\u{21E9} {}",
                    gitpoulpi_i18n::msg::toolbar::OVERFLOW_PULL_MENU.get()
                )
            }
            OverflowItem::Diff(DiffBarButton::PrevHunk) => {
                format!(
                    "\u{2191} {}",
                    gitpoulpi_i18n::msg::button_bar::PREV_HUNK.get()
                )
            }
            OverflowItem::Diff(DiffBarButton::NextHunk) => {
                format!(
                    "\u{2193} {}",
                    gitpoulpi_i18n::msg::button_bar::NEXT_HUNK.get()
                )
            }
            OverflowItem::Diff(b) => crate::diff_view::DIFF_BAR_BUTTONS
                .iter()
                .find(|t| **t == b)
                .map(|b| b.bar_label())
                .unwrap_or_default(),
            OverflowItem::FileListMode(m) => {
                let hint = crate::keymap::hint(
                    &crate::keymap::current(),
                    crate::keymap::Action::StatusToggleFileMode,
                    gitpoulpi_i18n::msg::button_bar::TOGGLES.get(),
                );
                if m == FileListMode::Path {
                    gitpoulpi_i18n::msg::button_bar::path_view(hint)
                } else {
                    gitpoulpi_i18n::msg::button_bar::tree_view(hint)
                }
            }
            OverflowItem::CollapseAll { collapse: true } => {
                gitpoulpi_i18n::msg::button_bar::COLLAPSE_ALL.get().into()
            }
            OverflowItem::CollapseAll { collapse: false } => {
                gitpoulpi_i18n::msg::button_bar::EXPAND_ALL.get().into()
            }
            OverflowItem::StatusBulk(s) => s.bulk_label().into(),
            OverflowItem::AiGenerate => format!(
                "{} {}",
                crate::composer::SPARK,
                gitpoulpi_i18n::msg::tooltip::AI_GENERATE.get()
            ),
            OverflowItem::ComposerIssueRef => {
                format!("# {}", gitpoulpi_i18n::msg::tooltip::INSERT_ISSUE_REF.get())
            }
            OverflowItem::HostingChip { linked: true } => {
                gitpoulpi_i18n::msg::hosting_ui::PR_ABBR.get().into()
            }
            OverflowItem::HostingChip { linked: false } => {
                gitpoulpi_i18n::msg::hosting_ui::NEW_PR.get().into()
            }
            OverflowItem::ViewAllFiles => {
                gitpoulpi_i18n::msg::button_bar::view_all_files(crate::keymap::paren(
                    &crate::keymap::current(),
                    crate::keymap::Action::StatusViewAllFiles,
                ))
            }
        }
    }
}

/// The overflow menu for `items`, hanging under the `⋯` at `button`.
/// Uses the one `context_menu::Menu` every other popup menu here uses,
/// so it takes the same keys and the same hover.
pub(crate) fn overflow_menu_for(items: &[OverflowItem], button: Rect) -> ContextMenu<OverflowItem> {
    let entries = items
        .iter()
        .map(|&i| context_menu::Entry::action(i.label(), i))
        .collect();
    ContextMenu::new(
        gitpoulpi_i18n::msg::button_bar::MORE.get(),
        entries,
        button.x,
        button.y + 1,
    )
}

/// What an open overflow menu needs to remember: the menu itself and the
/// `⋯` it hangs from. A choice is re-dispatched as a left click *at* the
/// `⋯`'s cell with the hidden button's target forced (see the event
/// loop), so the click lands on the same pane the button lives in.
#[derive(Debug, Clone)]
pub(crate) struct OverflowMenuState {
    pub(crate) button: Rect,
    pub(crate) menu: ContextMenu<OverflowItem>,
}

/// One drawn `⋯`: where it is and the buttons it hides.
pub(crate) type OverflowSlot = (Rect, Vec<OverflowItem>);

/// The overflow buttons one frame drew: which bar, where, and what it
/// hides. Rebuilt every frame, like `hit_targets`.
pub(crate) type OverflowSlots = Vec<(OverflowBar, Rect, Vec<OverflowItem>)>;

/// The `⋯`'s style for a bar whose buttons are filled chips (the toolbar,
/// the Diff bar): the same chip colors, lighter when hovered, using the
/// `PLAIN_BUTTON_HOVER_BG` every other hoverable header button/chip uses.
pub(crate) fn chip_overflow_style(hovered: bool) -> Style {
    let bg = if hovered {
        th().ui.button_hover
    } else {
        th().ui.button
    };
    Style::default().bg(bg).fg(th().ui.on_button)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(key: char, width: u16, priority: u8) -> BarItem<char> {
        BarItem {
            key,
            width,
            gap_before: 1,
            priority,
        }
    }

    fn keys(fit: &BarFit<char>) -> String {
        fit.visible.iter().map(|(_, i)| i.key).collect()
    }

    #[test]
    fn everything_fits_so_nothing_overflows_and_there_is_no_more_button() {
        let items = [item('a', 3, 0), item('b', 3, 1), item('c', 3, 2)];
        let fit = fit_bar(&items, 11, 1);
        assert_eq!(keys(&fit), "abc");
        assert!(fit.overflow.is_empty());
        assert_eq!(fit.overflow_button, None);
        assert_eq!(fit.visible[1].0, 4);
        assert_eq!(fit.end(), 11);
    }

    #[test]
    fn the_least_important_buttons_go_first_and_the_rest_keep_their_order() {
        // c is least important, then a. All three need 3+1+3+1+5 = 13.
        // Width 10: "a b" + gap + ⋯ = 3+1+3+1+3 = 11 > 10, so a goes too.
        let items = [item('a', 3, 1), item('b', 3, 0), item('c', 5, 2)];
        let fit = fit_bar(&items, 10, 1);
        assert_eq!(keys(&fit), "b");
        assert_eq!(fit.overflow, vec!['a', 'c']);
        // The first kept button starts at 0 even though it had a gap.
        assert_eq!(fit.visible[0].0, 0);
        assert_eq!(fit.overflow_button, Some((4, 3)));
        // At 12 only c has to go.
        let fit = fit_bar(&items, 12, 1);
        assert_eq!(keys(&fit), "ab");
        assert_eq!(fit.overflow, vec!['c']);
    }

    #[test]
    fn ties_drop_the_rightmost_first() {
        let items = [item('a', 3, 0), item('b', 3, 0), item('c', 3, 0)];
        let fit = fit_bar(&items, 10, 1);
        assert_eq!(keys(&fit), "a");
        assert_eq!(fit.overflow, vec!['b', 'c']);
    }

    #[test]
    fn nothing_drawn_ever_crosses_the_width_and_rects_never_overlap() {
        let items: Vec<_> = (0..8)
            .map(|i| item((b'a' + i) as char, 2 + u16::from(i % 3), i * 7 % 5))
            .collect();
        for width in 0..40 {
            let fit = fit_bar(&items, width, 1);
            assert!(fit.end() <= width, "width {width}: {fit:?}");
            let mut spans: Vec<(u16, u16)> =
                fit.visible.iter().map(|(x, i)| (*x, x + i.width)).collect();
            spans.extend(fit.overflow_button.map(|(x, w)| (x, x + w)));
            for pair in spans.windows(2) {
                assert!(pair[0].1 <= pair[1].0, "width {width}");
            }
            // Every button is either drawn or listed, exactly once.
            assert_eq!(fit.visible.len() + fit.overflow.len(), items.len());
        }
    }

    #[test]
    fn too_narrow_even_for_the_more_button_draws_nothing() {
        let items = [item('a', 5, 0)];
        let fit = fit_bar(&items, 2, 1);
        assert!(fit.visible.is_empty());
        assert_eq!(fit.overflow, vec!['a']);
        assert_eq!(fit.overflow_button, None);
        // A single button that does not fit becomes the ⋯ on its own.
        let fit = fit_bar(&items, 3, 1);
        assert_eq!(fit.overflow_button, Some((0, 3)));
    }

    #[test]
    fn every_overflow_item_names_the_click_its_button_would_have_made() {
        assert_eq!(
            OverflowItem::Toolbar(ToolbarButton::Push).hit_target(),
            HitTarget::Toolbar(ToolbarButton::Push)
        );
        assert_eq!(
            OverflowItem::Diff(DiffBarButton::ToggleWordWrap).hit_target(),
            HitTarget::DiffBarButton(DiffBarButton::ToggleWordWrap)
        );
        for b in TOOLBAR_BUTTONS.iter().map(|(_, _, b)| *b) {
            assert!(!OverflowItem::Toolbar(b).label().is_empty());
        }
        for b in crate::diff_view::DIFF_BAR_BUTTONS.iter().copied() {
            assert!(!OverflowItem::Diff(b).label().is_empty());
        }
    }
}
