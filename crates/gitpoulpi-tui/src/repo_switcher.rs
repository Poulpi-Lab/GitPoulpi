//! The repository tab bar, and the types shared by everything that
//! lists repositories (`RepoTab`, `RepoFacts`) or moves between them
//! (`SwitcherAction`).
//!
//! There is no unified "Repositories" overlay: the Repository Management tab
//! lists the repositories, and its Browse / Init / Clone buttons open the
//! dialogs in `repo_dialogs` (the folder picker, `init_dialog`,
//! `clone_dialog`).
//!
//! Everything here is **pure**: state plus span builders plus hit-test
//! rect lists. Nothing touches the daemon or the terminal.

use crate::th;
use std::path::PathBuf;

use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};

/// One tab: a repository open in this session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepoTab {
    pub path: PathBuf,
    pub name: String,
    pub branch: Option<String>,
}

/// What a background probe found out about a repository, filled in after
/// the row is already on screen.
///
/// Every field costs a `git` subprocess, so none is read on the render path:
/// `repo_probe` runs off the input loop and the row is drawn the moment the
/// overlay opens, with `facts: None` meaning *still looking* (see "never
/// block the input loop on a git subprocess call" in
/// `docs/developer/where-we-are.md`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RepoFacts {
    /// The branch actually checked out now, which may differ from the one
    /// the recent list remembered.
    pub branch: Option<String>,
    /// Anything at all in the working tree or the index.
    pub dirty: bool,
    pub ahead: u32,
    pub behind: u32,
    /// Work-in-progress counters, `✎ n  + n`: files changed
    /// (modified, deleted, renamed, conflicted - anything tracked) and
    /// files added (untracked, or new in the index).
    pub modified: u32,
    pub added: u32,
    /// Who the repository belongs to: the owner or namespace from the default
    /// remote's URL (`Poulpi-Lab`, `group/sub`), `None` when there is no
    /// remote to read one from (the caller falls back to the parent folder's
    /// name).
    pub owner: Option<String>,
}

/// A move between repository tabs, from the tab bar, the management tab
/// or a dialog. Carried out by `tabs::apply_switcher_action`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SwitcherAction {
    /// Open (or focus, if already a tab) the repository at this path.
    Open(PathBuf),
    /// Focus the tab at this index.
    Focus(usize),
    /// Close the tab at this index. The repository is untouched.
    Close(usize),
}

// -- the tab bar ---------------------------------------------------------

/// Where a tab and its close button were drawn, so a click can be resolved.
///
/// The close button's rect is a *sub*-rect of the tab's. Callers must
/// register the close button **first**: `find_hit_target` returns the first
/// match, and a tab band registered first covers the button inside it,
/// making it unreachable while looking fine (see the conventions list in
/// `docs/developer/where-we-are.md`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TabHit {
    pub index: usize,
    pub tab: Rect,
    pub close: Rect,
}

/// The folder button's glyph, with a column of padding either side.
///
/// U+1F4C1 and friends are emoji-presentation and would render as a hole or
/// flip width under a different font. U+2302 (HOUSE) is text-presentation
/// and present in the monospace fonts a Windows terminal falls back to,
/// which U+20DD was not (see `docs/developer/commit-graph.md`).
pub const FOLDER_BUTTON: &str = " \u{2302} ";

/// The `+` button's glyph, a column of padding either side.
pub const NEW_TAB_BUTTON: &str = " + ";

/// The overflow button, drawn where the tabs ran out of room.
pub const OVERFLOW_BUTTON: &str = " \u{2026} ";
const OVERFLOW_BUTTON_W: u16 = 3;

/// The transient New tab and its `+` button, as one draw of the bar sees
/// them.
#[derive(Debug, Clone, Default)]
pub struct NewTabSlot {
    /// A New tab is open: drawn after the last repository tab as the
    /// selected one. Its [`TabHit`] has `index == tabs.len()`.
    pub open: bool,
    pub plus_hovered: bool,
    /// The overflow `...` button is under the mouse.
    pub more_hovered: bool,
    /// The workspace chip after the `⌂`; `None` = no chip (no workspace
    /// exists).
    pub chip: Option<ChipSlot>,
    /// The Radar tab after the house; `None` = this build has none.
    pub radar: Option<RadarSlot>,
    /// The Workspaces tab after the Radar; `Some` only while it is open
    /// (the `+` menu, `Alt+J` and the palette open it, its `x` closes it).
    pub workspaces: Option<RadarSlot>,
    /// The Release notes tab after the Radar; `Some` only while it is
    /// open (it is transient, like the New tab, and always the selected one).
    pub notes: Option<NotesSlot>,
    /// Where the scrolled window of repository tabs starts (`tab_scroll`).
    pub scroll: BarScroll,
}

/// The horizontal scroll of the repository tabs, as one draw sees it.
#[derive(Debug, Clone, Copy, Default)]
pub struct BarScroll {
    /// The first visible tab asked for (clamped by the draw).
    pub offset: usize,
    /// A tab to bring fully into view with the least movement.
    pub follow: Option<usize>,
    pub left_hovered: bool,
    pub right_hovered: bool,
}

/// One arrow button of the scrolled bar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScrollArrow {
    pub rect: Rect,
    /// There are tabs hidden on its side; a dimmed arrow is inert and not
    /// registered.
    pub enabled: bool,
}

/// The Release notes tab: right after Home (and the Radar), closable.
#[derive(Debug, Clone, Copy, Default)]
pub struct NotesSlot {
    pub close_hovered: bool,
}

/// The Radar's tab: right after Home, closable like a repository tab
/// (the session remembers; Home is the one that cannot close).
#[derive(Debug, Clone, Copy, Default)]
pub struct RadarSlot {
    /// The Radar is the view on screen: drawn as the selected tab.
    pub open: bool,
    pub hovered: bool,
    pub close_hovered: bool,
}

/// What the tab bar draws for the workspace chip.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChipSlot {
    pub dot: ratatui::style::Color,
    /// The workspace's name, or "No workspace".
    pub label: String,
    /// No workspace is active: the label is dimmed.
    pub none: bool,
    pub hovered: bool,
    /// Its combo is open.
    pub open: bool,
}

/// The longest a workspace's name is drawn on the chip, in cells.
const CHIP_LABEL_MAX: usize = 18;
/// Room the tabs and the `+` keep when the chip is drawn in full.
const CHIP_KEEP_FOR_TABS: u16 = 24;

/// One draw of the tab bar: the spans, where each tab landed, and where
/// the folder button landed when there was room for it.
pub struct TabBar {
    pub line: Line<'static>,
    /// The vertical column's rows (`tab_column`): each with the rect it is
    /// painted at. Empty for the horizontal bar, which paints `line`.
    pub rows: Vec<(Rect, Line<'static>)>,
    pub hits: Vec<TabHit>,
    /// `None` when the bar was too narrow to draw it.
    pub folder: Option<Rect>,
    /// The `+` after the last tab; `None` when it did not fit.
    pub plus: Option<Rect>,
    /// The overflow button drawn when tabs did not fit (opens the tab
    /// picker); `None` when every tab fit.
    pub more: Option<Rect>,
    /// The workspace chip; `None` when there is none or it did not fit.
    pub chip: Option<Rect>,
    /// The Radar tab; `None` when there is none or it did not fit.
    pub radar: Option<Rect>,
    /// The Radar tab's `x`; `None` exactly when `radar` is.
    pub radar_close: Option<Rect>,
    /// The Workspaces tab; `None` when it is not open or did not fit.
    pub workspaces: Option<Rect>,
    /// The Workspaces tab's `x`; `None` when `workspaces` is.
    pub workspaces_close: Option<Rect>,
    /// The Release notes tab; `None` when it is not open or did not fit.
    pub notes: Option<Rect>,
    /// The Release notes tab's `x`; `None` exactly when `notes` is.
    pub notes_close: Option<Rect>,
    /// The scroll arrows at both ends of the repository tabs; `None` when
    /// every tab fits (or the bar is too narrow for them).
    pub scroll_left: Option<ScrollArrow>,
    pub scroll_right: Option<ScrollArrow>,
    /// The first visible repository tab the draw settled on.
    pub offset: usize,
}

/// The tab bar's spans, plus where each tab landed. One function produces
/// both, as `log_table` does for the Log table: a second walk computing the
/// rects would drift from the one that drew the glyphs and clicks would land
/// a cell or two off.
#[cfg(test)]
pub fn tab_bar(tabs: &[RepoTab], active: usize, area: Rect, folder_hovered: bool) -> TabBar {
    tab_bar_with(
        tabs,
        Some(active),
        area,
        folder_hovered,
        false,
        None,
        None,
        NewTabSlot::default(),
    )
}

/// [`tab_bar`], for a window that may be the Repository Management tab.
///
/// `management` says the `⌂` *is* the tab on screen: it is drawn as the
/// selected tab (an accent block, since it is not a repository tab) and no
/// repository tab is highlighted (`active` is `None`). A repository window
/// passes its tab and `false`.
///
/// `hovered_tab` is the index (into `tabs`) the mouse is over (inactive tabs
/// get a hover style).
#[allow(clippy::too_many_arguments)]
pub fn tab_bar_with(
    tabs: &[RepoTab],
    active: Option<usize>,
    area: Rect,
    folder_hovered: bool,
    management: bool,
    hovered_tab: Option<usize>,
    hovered_close: Option<usize>,
    new_tab: NewTabSlot,
) -> TabBar {
    let active = if management { None } else { active };
    // The house is the selected tab only on the Home view, not on a New tab
    // nor the Radar.
    let management = management
        && !new_tab.open
        && !new_tab.radar.is_some_and(|l| l.open)
        && !new_tab.workspaces.is_some_and(|l| l.open)
        && new_tab.notes.is_none();
    let mut spans = Vec::new();
    let mut hits = Vec::new();
    let mut x = area.x;

    // The folder button leads the strip. It is drawn first
    // so the tabs start after it, and its rect comes out of this same walk:
    // a second computation of "two columns at the left edge" is how a button
    // ends up one cell off the glyph that advertises it.
    let folder = Rect::new(
        area.x,
        area.y,
        crate::ui_kit::display_width(FOLDER_BUTTON) as u16,
        1,
    );
    let folder = if folder.width < area.width {
        let style = if management {
            Style::default()
                .fg(crate::th().ui.on_accent)
                .bg(th().ui.accent)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
                .fg(if folder_hovered {
                    th().ui.text
                } else {
                    th().ui.accent
                })
                .bg(if folder_hovered {
                    th().ui.selection
                } else {
                    th().ui.hover
                })
        };
        spans.push(Span::styled(FOLDER_BUTTON, style));
        x += folder.width;
        Some(folder)
    } else {
        // No room: not drawn, and therefore not registered. A hit
        // target with no glyph under it is a click that appears to do
        // nothing, which is the rule the Preferences gear already
        // carries.
        None
    };

    // The workspace chip, right after the `⌂`: dot and name, or just the dot
    // when the bar is short; neither drawn nor registered when even the dot
    // would crowd the tabs out.
    let end = area.x + area.width;
    let chip = new_tab.chip.as_ref().and_then(|c| {
        let name = crate::fit(&c.label, CHIP_LABEL_MAX);
        let full = format!(" {} {name} ", crate::workspace_ui::DOT);
        let dot = format!(" {} ", crate::workspace_ui::DOT);
        let text = if x + crate::ui_kit::display_width(&full) as u16 + CHIP_KEEP_FOR_TABS <= end {
            full
        } else if x + crate::ui_kit::display_width(&dot) as u16 + 8 <= end {
            dot
        } else {
            return None;
        };
        let bg = if c.hovered || c.open {
            th().ui.selection
        } else {
            th().ui.hover
        };
        let width = crate::ui_kit::display_width(&text) as u16;
        let rect = Rect::new(x, area.y, width, 1);
        let label_style = Style::default()
            .fg(if c.none { th().ui.muted } else { th().ui.text })
            .bg(bg);
        let mut parts = text.splitn(3, ' ');
        // " ● name " -> "", "●", "name ".
        let _lead = parts.next();
        let glyph = parts.next().unwrap_or("");
        let rest = parts.next().unwrap_or("");
        spans.push(Span::styled(" ", label_style));
        spans.push(Span::styled(
            glyph.to_string(),
            Style::default().fg(c.dot).bg(bg),
        ));
        spans.push(Span::styled(format!(" {rest}"), label_style));
        x += width;
        spans.push(Span::styled(" ", Style::default()));
        x += 1;
        Some(rect)
    });

    // The Radar, pinned right after Home (and its workspace chip): a tab
    // like the others, `x` included. Neither drawn nor registered when the
    // bar has no room left for the repository tabs.
    let mut pinned = |slot: Option<RadarSlot>, title: &str| -> Option<(Rect, Option<Rect>)> {
        let l = slot?;
        let text = format!(" {title} ");
        let label_w = crate::ui_kit::display_width(&text) as u16;
        let width = label_w + 2;
        if x + width + CHIP_KEEP_FOR_TABS > end {
            return None;
        }
        let style = if l.open {
            Style::default()
                .fg(th().ui.text)
                .bg(crate::theme::selected_tab_bg())
                .add_modifier(Modifier::BOLD)
        } else if l.hovered {
            Style::default().fg(th().ui.text).bg(th().ui.hover)
        } else {
            Style::default().fg(th().ui.muted)
        };
        spans.push(Span::styled(text, style));
        let close_style = if l.close_hovered {
            style.patch(crate::theme::plain_button_style(true))
        } else {
            style.fg(if l.open {
                th().ui.accent
            } else {
                th().ui.muted
            })
        };
        // Like the repository tabs: the cross only on the open or hovered
        // tab, a blank cell (and no rect) otherwise.
        let close_shown = l.open || l.hovered || l.close_hovered;
        spans.push(Span::styled(
            if close_shown { "x" } else { " " },
            close_style,
        ));
        spans.push(Span::styled(" ", style));
        let close = close_shown.then(|| Rect::new(x + label_w, area.y, 1, 1));
        let rect = Rect::new(x, area.y, width, 1);
        x += width;
        Some((rect, close))
    };
    let (radar, radar_close) = pinned(new_tab.radar, gitpoulpi_i18n::msg::radar::TITLE.get())
        .map_or((None, None), |(r, c)| (Some(r), c));
    // The Workspaces tab, right after the Radar: the same kind of tab.
    let (workspaces, workspaces_close) = pinned(
        new_tab.workspaces,
        gitpoulpi_i18n::msg::workspaces::PAGE_WORKSPACES.get(),
    )
    .map_or((None, None), |(r, c)| (Some(r), c));

    // The Release notes tab: transient, right after the Radar, selected
    // while it exists. Like the Radar's, `x` included.
    let mut notes_close = None;
    let notes = new_tab.notes.and_then(|n| {
        let text = format!(" {} ", gitpoulpi_i18n::msg::updates::TAB_TITLE.get());
        let label_w = crate::ui_kit::display_width(&text) as u16;
        let width = label_w + 2;
        if x + width + CHIP_KEEP_FOR_TABS > end {
            return None;
        }
        let style = Style::default()
            .fg(th().ui.text)
            .bg(crate::theme::selected_tab_bg())
            .add_modifier(Modifier::BOLD);
        spans.push(Span::styled(text, style));
        let close_style = if n.close_hovered {
            style.patch(crate::theme::plain_button_style(true))
        } else {
            style.fg(th().ui.accent)
        };
        spans.push(Span::styled("x", close_style));
        spans.push(Span::styled(" ", style));
        notes_close = Some(Rect::new(x + label_w, area.y, 1, 1));
        let rect = Rect::new(x, area.y, width, 1);
        x += width;
        Some(rect)
    });

    // Room kept at the right end for the New tab and the `+`, so they stay
    // visible when the repository tabs overflow (the `+` follows the last
    // tab that fits, then the ellipsis that says more exist).
    let plus_w = crate::ui_kit::display_width(NEW_TAB_BUTTON) as u16;
    let new_label = format!(" {} ", gitpoulpi_i18n::msg::tab_bar::NEW_TAB.get());
    let new_w = (crate::ui_kit::display_width(&new_label) + 2) as u16;
    let wanted = plus_w + 1 + if new_tab.open { new_w } else { 0 };
    let reserve = if x + wanted + 8 <= area.x + area.width {
        wanted
    } else {
        0
    };
    let limit = area.x + area.width - reserve;

    // When the tabs overflow, they scroll: a window of them between two
    // arrow buttons (`tab_scroll`), then the `...` button (the tab picker)
    // when there is room for it too. The `...` gives way first, then the
    // arrows; the remaining tabs are still reachable by keyboard and through
    // the tab picker, so the bar scrolls rather than shrinking every label
    // into illegibility.
    let widths: Vec<u16> = tabs
        .iter()
        .map(|t| crate::ui_kit::display_width(&format!(" {} x ", t.name)) as u16)
        .collect();
    let all_w: u32 = widths.iter().map(|w| u32::from(*w)).sum();
    let room = limit.saturating_sub(x);
    let overflow = all_w > u32::from(room);
    let arrow_w = crate::tab_scroll::ARROW_W;
    let arrows = overflow && room >= 2 * arrow_w + 4;
    let with_more = arrows && room >= 2 * arrow_w + OVERFLOW_BUTTON_W + 8;
    let strip_w =
        room - if arrows { 2 * arrow_w } else { 0 } - if with_more { OVERFLOW_BUTTON_W } else { 0 };
    let offset = if overflow {
        crate::tab_scroll::window_start(
            &widths,
            strip_w,
            new_tab.scroll.offset,
            new_tab.scroll.follow,
        )
    } else {
        0
    };
    let arrow_style = |enabled: bool, hovered: bool| {
        if enabled {
            Style::default()
                .fg(if hovered {
                    th().ui.text
                } else {
                    th().ui.accent
                })
                .bg(if hovered {
                    th().ui.selection
                } else {
                    th().ui.hover
                })
        } else {
            crate::dialog_kit::button_style(crate::dialog_kit::ButtonState::Disabled)
        }
    };
    let mut scroll_left = None;
    if arrows {
        let enabled = offset > 0;
        spans.push(Span::styled(
            crate::tab_scroll::LEFT_BUTTON,
            arrow_style(enabled, new_tab.scroll.left_hovered),
        ));
        scroll_left = Some(ScrollArrow {
            rect: Rect::new(x, area.y, arrow_w, 1),
            enabled,
        });
        x += arrow_w;
    }
    let strip_end = x + strip_w;
    let mut end = offset;
    for (index, tab) in tabs.iter().enumerate().skip(offset) {
        let is_active = Some(index) == active;
        // Every tab carries its own close button, including the active
        // one: a bar where you have to focus a tab before you can close
        // it takes two clicks for every close.
        let close = "x ";
        let mut label = format!(" {} ", tab.name);
        let mut total =
            (crate::ui_kit::display_width(&label) + crate::ui_kit::display_width(close)) as u16;
        if x + total > strip_end {
            // The first tab of the window wider than the whole strip: its
            // name is cut rather than the bar showing no tab at all.
            let cells = strip_end.saturating_sub(x);
            if index != offset || cells < 6 {
                if !arrows && cells >= 1 {
                    spans.push(Span::styled("\u{2026}", Style::default().fg(th().ui.muted)));
                    x += 1;
                }
                break;
            }
            label = format!(" {} ", crate::fit(&tab.name, usize::from(cells - 4)));
            total =
                (crate::ui_kit::display_width(&label) + crate::ui_kit::display_width(close)) as u16;
        }

        // The same selection treatment and palette as every other list:
        // explicit RGB, never the palette-relative `Cyan`/`Gray`/`DarkGray`
        // slots, which the user's terminal theme may render into their
        // background. An inactive tab gets a hover style; only the active
        // tab paints `SELECTION_BG`.
        let style = tab_style(is_active, hovered_tab == Some(index));
        spans.push(Span::styled(label.clone(), style));
        // The cross itself gets the strong Settings-gear hover.
        let close_style = close_style(style, is_active, hovered_close == Some(index));
        // Only the `x` cell; the trailing space keeps the tab's own style.
        // The cross is painted on the selected tab and the hovered one only
        // (owner, 2026-10-20); elsewhere its cell is blank, so the tab keeps
        // its width and does not jump on hover, and its rect is empty: a
        // click needs a glyph under it.
        let close_shown = is_active || hovered_tab == Some(index) || hovered_close == Some(index);
        spans.push(Span::styled(
            if close_shown { "x" } else { " " },
            close_style,
        ));
        spans.push(Span::styled(" ", style));

        hits.push(TabHit {
            index,
            tab: Rect {
                x,
                y: area.y,
                width: total,
                height: 1,
            },
            close: Rect {
                x: x + crate::ui_kit::display_width(&label) as u16,
                y: area.y,
                width: u16::from(close_shown),
                height: 1,
            },
        });
        x += total;
        end = index + 1;
    }
    let mut scroll_right = None;
    let mut more = None;
    if arrows {
        // The right arrow stays at the strip's end, so it does not move
        // while the window scrolls.
        if x < strip_end {
            spans.push(Span::raw(" ".repeat(usize::from(strip_end - x))));
            x = strip_end;
        }
        let enabled = end < tabs.len();
        spans.push(Span::styled(
            crate::tab_scroll::RIGHT_BUTTON,
            arrow_style(enabled, new_tab.scroll.right_hovered),
        ));
        scroll_right = Some(ScrollArrow {
            rect: Rect::new(x, area.y, arrow_w, 1),
            enabled,
        });
        x += arrow_w;
    }
    if with_more {
        // A real button (it opens the tab picker): padded like the `+`.
        let style = Style::default()
            .fg(if new_tab.more_hovered {
                th().ui.text
            } else {
                th().ui.accent
            })
            .bg(if new_tab.more_hovered {
                th().ui.selection
            } else {
                th().ui.hover
            });
        spans.push(Span::styled(OVERFLOW_BUTTON, style));
        more = Some(Rect::new(x, area.y, OVERFLOW_BUTTON_W, 1));
        x += OVERFLOW_BUTTON_W;
    }

    if tabs.is_empty() && !new_tab.open {
        let text = format!(" {} ", gitpoulpi_i18n::msg::tab_bar::NO_REPO.get());
        x += crate::ui_kit::display_width(&text) as u16;
        spans.push(Span::styled(text, Style::default().fg(th().ui.muted)));
    }

    // The transient New tab: a tab like the others, selected, with its `x`.
    if reserve > 0 && new_tab.open {
        let style = Style::default()
            .fg(th().ui.text)
            .bg(crate::theme::selected_tab_bg())
            .add_modifier(Modifier::BOLD);
        let label_w = crate::ui_kit::display_width(&new_label) as u16;
        spans.push(Span::styled(new_label.clone(), style));
        spans.push(Span::styled("x", style.fg(th().ui.accent)));
        spans.push(Span::styled(" ", style));
        hits.push(TabHit {
            index: tabs.len(),
            tab: Rect::new(x, area.y, new_w, 1),
            close: Rect::new(x + label_w, area.y, 1, 1),
        });
        x += new_w;
    }

    // The `+`, right after the last tab. Drawn only when it fits, and so
    // registered only then.
    let plus = if reserve > 0 {
        let style = Style::default()
            .fg(if new_tab.plus_hovered {
                th().ui.text
            } else {
                th().ui.accent
            })
            .bg(if new_tab.plus_hovered {
                th().ui.selection
            } else {
                th().ui.hover
            });
        spans.push(Span::styled(" ", Style::default()));
        spans.push(Span::styled(NEW_TAB_BUTTON, style));
        Some(Rect::new(x + 1, area.y, plus_w, 1))
    } else {
        None
    };

    TabBar {
        line: Line::from(spans),
        rows: Vec::new(),
        hits,
        folder,
        plus,
        more,
        chip,
        radar,
        radar_close,
        workspaces,
        workspaces_close,
        notes,
        notes_close,
        scroll_left,
        scroll_right,
        offset,
    }
}

/// The look of a tab: selected, hovered or resting. Shared by the horizontal
/// bar and the vertical column.
pub(crate) fn tab_style(active: bool, hovered: bool) -> Style {
    if active {
        Style::default()
            .fg(th().ui.text)
            .bg(crate::theme::selected_tab_bg())
            .add_modifier(Modifier::BOLD)
    } else if hovered {
        Style::default().fg(th().ui.text).bg(th().ui.hover)
    } else {
        Style::default().fg(th().ui.muted)
    }
}

/// The look of a tab's close cross, given the tab's own style.
pub(crate) fn close_style(tab: Style, active: bool, hovered: bool) -> Style {
    if hovered {
        tab.patch(crate::theme::plain_button_style(true))
    } else {
        tab.fg(if active {
            th().ui.accent
        } else {
            th().ui.muted
        })
    }
}

/// Resolves a click on the tab bar.
///
/// The close button is tested first, by construction - see [`TabHit`].
pub fn tab_bar_click(hits: &[TabHit], col: u16, row: u16) -> Option<SwitcherAction> {
    for hit in hits {
        if contains(hit.close, col, row) {
            return Some(SwitcherAction::Close(hit.index));
        }
    }
    for hit in hits {
        if contains(hit.tab, col, row) {
            return Some(SwitcherAction::Focus(hit.index));
        }
    }
    None
}

/// The tab under a *right* click, close button included - the tab
/// menu is about the tab, whichever part of it was clicked.
pub fn tab_at(hits: &[TabHit], col: u16, row: u16) -> Option<usize> {
    hits.iter()
        .find(|hit| contains(hit.tab, col, row))
        .map(|hit| hit.index)
}

fn contains(rect: Rect, col: u16, row: u16) -> bool {
    col >= rect.x && col < rect.x + rect.width && row >= rect.y && row < rect.y + rect.height
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text(lines: &[Line<'_>]) -> String {
        lines
            .iter()
            .map(|l| {
                l.spans
                    .iter()
                    .map(|s| s.content.as_ref())
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// The button leads the strip, the tabs start after it, and its
    /// rect comes from the same walk that drew it.
    #[test]
    fn the_folder_button_leads_the_tab_strip() {
        let bar = tab_bar(&tabs(), 0, bar_area(), false);
        let folder = bar.folder.expect("there is room for it");
        assert_eq!(folder.x, bar_area().x);
        assert_eq!(
            folder.width,
            crate::ui_kit::display_width(FOLDER_BUTTON) as u16
        );
        assert!(
            bar.hits[0].tab.x >= folder.x + folder.width,
            "the first tab starts after the button: {:?}",
            bar.hits[0]
        );
        // ...and the glyph is actually on screen at that rect.
        let drawn: String = bar.line.spans.iter().map(|s| s.content.as_ref()).collect();
        assert!(drawn.starts_with(FOLDER_BUTTON), "{drawn:?}");
    }

    /// On the Repository Management tab the `⌂` *is* the selected tab --
    /// an accent block - and no repository tab claims the selection;
    /// in a repository window the `⌂` is a plain button and the active
    /// tab is selected.
    #[test]
    fn on_the_management_tab_the_house_is_selected_and_no_repository_tab_is() {
        let selected_bg = |bar: &TabBar, needle: &str| {
            bar.line
                .spans
                .iter()
                .find(|s| s.content.contains(needle))
                .and_then(|s| s.style.bg)
        };
        let managing = tab_bar_with(
            &tabs(),
            Some(0),
            bar_area(),
            false,
            true,
            None,
            None,
            NewTabSlot::default(),
        );
        assert_eq!(selected_bg(&managing, "\u{2302}"), Some(th().ui.accent));
        assert!(
            managing
                .line
                .spans
                .iter()
                .all(|s| s.style.bg != Some(th().ui.selection)),
            "no repository tab is highlighted"
        );
        let in_repo = tab_bar_with(
            &tabs(),
            Some(0),
            bar_area(),
            false,
            false,
            None,
            None,
            NewTabSlot::default(),
        );
        assert_ne!(selected_bg(&in_repo, "\u{2302}"), Some(th().ui.accent));
        assert!(in_repo
            .line
            .spans
            .iter()
            .any(|s| s.style.bg == Some(th().ui.selection)));
    }

    /// The tab's close cross lights with the strong hover fill, only
    /// that cell, only for the hovered tab.
    #[test]
    fn the_hovered_close_cross_gets_the_strong_hover_and_nothing_else_does() {
        let strong = crate::theme::plain_button_style(true).bg;
        let area = bar_area();
        let render = |hc: Option<usize>| {
            let bar = tab_bar_with(
                &tabs(),
                Some(0),
                area,
                false,
                false,
                None,
                hc,
                NewTabSlot::default(),
            );
            let mut buf = ratatui::buffer::Buffer::empty(area);
            ratatui::widgets::Widget::render(
                ratatui::widgets::Paragraph::new(bar.line),
                area,
                &mut buf,
            );
            (bar.hits, buf)
        };
        let (hits, hot) = render(Some(1));
        let (_, rest) = render(None);
        for x in area.x..area.x + area.width {
            let is_cross = x == hits[1].close.x;
            assert_eq!(Some(hot[(x, area.y)].bg) == strong, is_cross, "hot x={x}");
            assert_ne!(Some(rest[(x, area.y)].bg), strong, "rest x={x}");
        }
    }

    fn drawn_text(bar: TabBar) -> (Vec<TabHit>, String) {
        let area = bar_area();
        let mut buf = ratatui::buffer::Buffer::empty(area);
        ratatui::widgets::Widget::render(
            ratatui::widgets::Paragraph::new(bar.line),
            area,
            &mut buf,
        );
        let text = (0..area.width)
            .map(|x| buf[(x, 0)].symbol().to_string())
            .collect();
        (bar.hits, text)
    }

    /// The close cross is painted on the selected tab and the hovered one
    /// only; the others keep their width (blank cell) and register no close
    /// rect, so tabs do not jump on hover and a click needs a glyph.
    #[test]
    fn the_close_cross_shows_only_on_the_active_and_the_hovered_tab() {
        let three: Vec<RepoTab> = ["alpha", "beta", "gamma"]
            .iter()
            .map(|n| RepoTab {
                path: PathBuf::from(format!("/repos/{n}")),
                name: n.to_string(),
                branch: None,
            })
            .collect();
        let draw = |hover: Option<usize>| {
            drawn_text(tab_bar_with(
                &three,
                Some(0),
                bar_area(),
                false,
                false,
                hover,
                None,
                NewTabSlot::default(),
            ))
        };
        let (rest_hits, rest) = draw(None);
        let (hot_hits, hot) = draw(Some(2));
        let cross = |text: &str, hit: &TabHit| text.chars().nth(hit.close.x as usize) == Some('x');
        assert!(cross(&rest, &rest_hits[0]), "active");
        assert!(!cross(&rest, &rest_hits[1]) && !cross(&rest, &rest_hits[2]));
        assert!(
            cross(&hot, &hot_hits[0]) && !cross(&hot, &hot_hits[1]) && cross(&hot, &hot_hits[2])
        );
        // Stable widths: every tab sits where it did.
        for (a, b) in rest_hits.iter().zip(&hot_hits) {
            assert_eq!(a.tab, b.tab);
        }
        // Hit rects follow the paint: empty where blank.
        assert_eq!(rest_hits[1].close.width, 0);
        assert_eq!(
            tab_bar_click(&rest_hits, rest_hits[1].close.x, 0),
            Some(SwitcherAction::Focus(1))
        );
        assert_eq!(
            tab_bar_click(&hot_hits, hot_hits[2].close.x, 0),
            Some(SwitcherAction::Close(2))
        );
    }

    /// A right click anywhere on a tab, its `x` included, is about that
    /// tab.
    #[test]
    fn a_right_click_resolves_to_the_tab_under_it() {
        let bar = tab_bar(&tabs(), 0, bar_area(), false);
        for hit in &bar.hits {
            assert_eq!(tab_at(&bar.hits, hit.tab.x, hit.tab.y), Some(hit.index));
            assert_eq!(tab_at(&bar.hits, hit.close.x, hit.close.y), Some(hit.index));
        }
        let folder = bar.folder.unwrap();
        assert_eq!(tab_at(&bar.hits, folder.x, folder.y), None);
    }

    /// A bar with no room for the button does not draw it - and
    /// therefore does not register it. A hit target with no glyph under
    /// it is a click that appears to do nothing, which is the rule the
    /// Preferences gear already carries.
    #[test]
    fn a_bar_too_narrow_for_the_folder_button_does_not_pretend_to_have_one() {
        let cramped = Rect {
            x: 0,
            y: 0,
            width: 2,
            height: 1,
        };
        let bar = tab_bar(&tabs(), 0, cramped, false);
        assert!(bar.folder.is_none());
        let drawn: String = bar.line.spans.iter().map(|s| s.content.as_ref()).collect();
        assert!(!drawn.contains('\u{2302}'), "{drawn:?}");
    }

    /// The button is never inside a tab's band, so the two hit lists
    /// can never claim the same cell.
    #[test]
    fn the_folder_button_never_overlaps_a_tab() {
        let bar = tab_bar(&tabs(), 0, bar_area(), false);
        let folder = bar.folder.unwrap();
        for hit in &bar.hits {
            assert!(
                hit.tab.x >= folder.x + folder.width,
                "{hit:?} overlaps {folder:?}"
            );
        }
        assert_eq!(tab_bar_click(&bar.hits, folder.x, 0), None);
    }

    // -- tab bar --

    fn tabs() -> Vec<RepoTab> {
        ["alpha", "beta"]
            .iter()
            .map(|n| RepoTab {
                path: PathBuf::from(format!("/repos/{n}")),
                name: n.to_string(),
                branch: None,
            })
            .collect()
    }

    fn bar_area() -> Rect {
        Rect {
            x: 0,
            y: 0,
            width: 80,
            height: 1,
        }
    }

    /// A repository named in CJK is two cells per character: the tab, its
    /// close button and the next tab are laid out in cells, not chars.
    #[test]
    fn cjk_tab_names_are_measured_in_cells() {
        let tabs: Vec<RepoTab> = ["\u{9879}\u{76ee}", "\u{30ea}\u{30dd}", "ab"]
            .iter()
            .map(|n| RepoTab {
                path: PathBuf::from(format!("/repos/{n}")),
                name: n.to_string(),
                branch: None,
            })
            .collect();
        let bar = tab_bar(&tabs, 0, bar_area(), false);
        assert_eq!(bar.hits.len(), 3);
        // " name " + "x ": two CJK chars are 4 cells.
        assert_eq!(bar.hits[0].tab.width, 1 + 4 + 1 + 2);
        assert_eq!(bar.hits[0].close.x, bar.hits[0].tab.x + 6);
        assert_eq!(bar.hits[1].tab.x, bar.hits[0].tab.x + bar.hits[0].tab.width);
        assert_eq!(bar.hits[2].tab.x, bar.hits[1].tab.x + bar.hits[1].tab.width);
        let drawn: usize = bar
            .line
            .spans
            .iter()
            .map(|s| crate::ui_kit::display_width(&s.content))
            .sum();
        // The walk ends with the `+`: its extent is the drawn width.
        let plus = bar.plus.unwrap();
        assert_eq!(
            drawn as u16,
            plus.x + plus.width,
            "drawn width = hit extent"
        );
        let last = bar.hits.last().unwrap().tab;
        assert_eq!(plus.x, last.x + last.width + 1);
    }

    #[test]
    fn clicking_a_tab_focuses_it_and_clicking_its_x_closes_it() {
        // Tab 1 is hovered, so its cross is painted and live.
        let hits = tab_bar_with(
            &tabs(),
            Some(0),
            bar_area(),
            false,
            false,
            Some(1),
            None,
            NewTabSlot::default(),
        )
        .hits;
        assert_eq!(hits.len(), 2);

        // The body of the second tab focuses it...
        assert_eq!(
            tab_bar_click(&hits, hits[1].tab.x + 1, 0),
            Some(SwitcherAction::Focus(1))
        );
        // ...and its close cell closes it, even though that cell is
        // inside the tab's own band. This is the inner-first rule; the
        // test fails if `tab_bar_click` tests the bands first.
        assert_eq!(
            tab_bar_click(&hits, hits[1].close.x, 0),
            Some(SwitcherAction::Close(1))
        );
    }

    #[test]
    fn a_click_past_the_last_tab_hits_nothing() {
        let hits = tab_bar(&tabs(), 0, bar_area(), false).hits;
        let past = hits.last().unwrap().tab.x + hits.last().unwrap().tab.width + 3;
        assert_eq!(tab_bar_click(&hits, past, 0), None);
    }

    /// Every close button must be inside its own tab's band, or the
    /// inner-first ordering above is testing nothing.
    #[test]
    fn each_close_button_is_inside_its_own_tab() {
        let hits = tab_bar(&tabs(), 0, bar_area(), false).hits;
        for hit in &hits {
            assert!(
                hit.close.x >= hit.tab.x && hit.close.x < hit.tab.x + hit.tab.width,
                "{hit:?}"
            );
        }
    }

    /// A bar narrower than its tabs must truncate rather than draw past
    /// its area - and must not register a hit for a tab it did not draw.
    #[test]
    fn a_narrow_bar_truncates_instead_of_overflowing() {
        let narrow = Rect {
            x: 0,
            y: 0,
            width: 10,
            height: 1,
        };
        let bar = tab_bar(&tabs(), 0, narrow, false);
        let (line, hits) = (bar.line, bar.hits);
        assert!(hits.len() < 2, "the second tab does not fit: {hits:?}");
        let width: usize = line.spans.iter().map(|s| s.content.chars().count()).sum();
        assert!(width <= narrow.width as usize + 1, "drew {width} cells");
        for hit in &hits {
            assert!(hit.tab.x + hit.tab.width <= narrow.x + narrow.width);
        }
    }

    #[test]
    fn an_empty_bar_says_so() {
        let bar = tab_bar(&[], 0, bar_area(), false);
        let (line, hits) = (bar.line, bar.hits);
        assert!(hits.is_empty());
        assert!(text(&[line]).contains("no repository open"));
    }

    fn drawn(bar: &TabBar, area: Rect) -> ratatui::buffer::Buffer {
        let mut buf = ratatui::buffer::Buffer::empty(area);
        ratatui::widgets::Widget::render(
            ratatui::widgets::Paragraph::new(bar.line.clone()),
            area,
            &mut buf,
        );
        buf
    }

    fn row_text(buf: &ratatui::buffer::Buffer, area: Rect) -> String {
        (area.x..area.x + area.width)
            .map(|x| buf[(x, area.y)].symbol().to_string())
            .collect()
    }

    fn chip(label: &str, none: bool) -> NewTabSlot {
        NewTabSlot {
            chip: Some(ChipSlot {
                dot: ratatui::style::Color::Green,
                label: label.into(),
                none,
                hovered: false,
                open: false,
            }),
            ..NewTabSlot::default()
        }
    }

    fn bar_with_chip(width: u16, slot: NewTabSlot) -> (TabBar, Rect) {
        let area = Rect::new(0, 0, width, 1);
        (
            tab_bar_with(&tabs(), Some(0), area, false, false, None, None, slot),
            area,
        )
    }

    /// The chip sits right after the house, before the first tab: a dot and the
    /// workspace's name, its rect the glyphs' own cells.
    #[test]
    fn the_chip_follows_the_house_and_precedes_the_tabs() {
        let (bar, area) = bar_with_chip(80, chip("Acme", false));
        let rect = bar.chip.expect("room for the chip");
        let buf = drawn(&bar, area);
        let row = row_text(&buf, area);
        let painted: String = row
            .chars()
            .skip(rect.x as usize)
            .take(rect.width as usize)
            .collect();
        assert!(
            painted.contains("Acme") && painted.contains('\u{25CF}'),
            "{painted:?}"
        );
        assert!(rect.x >= bar.folder.unwrap().right(), "{row}");
        assert!(rect.right() <= bar.hits[0].tab.x, "{row}");
        // The dot wears the workspace's color.
        let dot_x = row.chars().position(|c| c == '\u{25CF}').expect("a dot") as u16;
        assert_eq!(buf[(dot_x, 0)].fg, ratatui::style::Color::Green);
    }

    /// "No workspace" is dimmed.
    #[test]
    fn no_workspace_is_dimmed() {
        let (bar, area) = bar_with_chip(80, chip("No workspace", true));
        let rect = bar.chip.unwrap();
        let buf = drawn(&bar, area);
        let name_x = rect.x + 4;
        assert_eq!(
            buf[(name_x, 0)].fg,
            th().ui.muted,
            "{}",
            row_text(&buf, area)
        );
    }

    /// A short bar keeps the dot; a shorter one draws and registers neither.
    #[test]
    fn the_chip_shrinks_to_a_dot_and_then_is_neither_drawn_nor_registered() {
        let full = bar_with_chip(80, chip("Acme", false)).0.chip.unwrap();
        // Room for the dot and a few tabs, not for the name.
        let (short, area) = bar_with_chip(26, chip("Acme", false));
        let dot = short.chip.expect("the dot still fits");
        assert!(dot.width < full.width, "{dot:?} vs {full:?}");
        let row = row_text(&drawn(&short, area), area);
        assert!(row.contains('\u{25CF}') && !row.contains("Acme"), "{row}");
        // Barely room for the house.
        let (tiny, area) = bar_with_chip(9, chip("Acme", false));
        assert!(tiny.chip.is_none());
        assert!(!row_text(&drawn(&tiny, area), area).contains('\u{25CF}'));
        // No chip asked for, none drawn.
        let (none, _) = bar_with_chip(80, NewTabSlot::default());
        assert!(none.chip.is_none());
    }

    /// The `+` is drawn right after the last tab, and its rect is the
    /// glyph's own cells.
    #[test]
    fn the_plus_follows_the_last_tab() {
        let area = bar_area();
        let bar = tab_bar(&tabs(), 0, area, false);
        let plus = bar.plus.expect("room for the +");
        let last = bar.hits.last().unwrap();
        assert!(plus.x >= last.tab.x + last.tab.width, "{plus:?} {last:?}");
        assert!(plus.x - (last.tab.x + last.tab.width) <= 2);
        let buf = drawn(&bar, area);
        assert_eq!(buf[(plus.x + 1, plus.y)].symbol(), "+");
    }

    /// When the tabs overflow, the `+` is still drawn (the tabs give way
    /// to it) and stays inside the bar.
    #[test]
    fn the_plus_survives_an_overflowing_bar() {
        let area = Rect::new(0, 0, 30, 1);
        let bar = tab_bar(&tabs(), 0, area, false);
        let plus = bar.plus.expect("the + is kept when the tabs overflow");
        assert!(plus.x + plus.width <= area.x + area.width, "{plus:?}");
        let buf = drawn(&bar, area);
        assert_eq!(buf[(plus.x + 1, plus.y)].symbol(), "+");
        for hit in &bar.hits {
            assert!(hit.tab.x + hit.tab.width <= plus.x, "{hit:?} {plus:?}");
        }
    }

    /// Overflowing tabs leave a drawn, registered `...` button after the
    /// right scroll arrow and before the `+`; a bar where everything fits
    /// has none.
    #[test]
    fn overflowing_tabs_end_in_a_registered_ellipsis_button() {
        let area = Rect::new(0, 0, 40, 1);
        let bar = tab_bar(&many(8), 0, area, false);
        let more = bar.more.expect("the ... button");
        let buf = drawn(&bar, area);
        assert_eq!(buf[(more.x + 1, more.y)].symbol(), "\u{2026}");
        for hit in &bar.hits {
            assert!(hit.tab.x + hit.tab.width <= more.x, "{hit:?} {more:?}");
        }
        assert!(bar.scroll_right.unwrap().rect.right() <= more.x);
        assert!(more.x + more.width <= bar.plus.unwrap().x);
        let wide = tab_bar(&tabs(), 0, bar_area(), false);
        assert!(wide.more.is_none());
        // Short of room, the `...` gives way first and the arrows stay.
        let short = tab_bar(&tabs(), 0, Rect::new(0, 0, 20, 1), false);
        assert!(short.more.is_none() && short.scroll_right.is_some());
    }

    fn many(n: usize) -> Vec<RepoTab> {
        (0..n)
            .map(|i| RepoTab {
                path: PathBuf::from(format!("/repos/repo{i}")),
                name: format!("repo{i}"),
                branch: None,
            })
            .collect()
    }

    fn scrolled(tabs: &[RepoTab], active: usize, width: u16, scroll: BarScroll) -> (TabBar, Rect) {
        let area = Rect::new(0, 0, width, 1);
        let slot = NewTabSlot {
            scroll,
            ..NewTabSlot::default()
        };
        (
            tab_bar_with(tabs, Some(active), area, false, false, None, None, slot),
            area,
        )
    }

    /// Tabs that do not fit sit between two arrow buttons: at the start the
    /// left one is dimmed and inert, the right one live, and the glyphs are
    /// painted on the rects the walk returned.
    #[test]
    fn overflowing_tabs_scroll_between_two_arrows() {
        let (bar, area) = scrolled(&many(8), 0, 50, BarScroll::default());
        let left = bar.scroll_left.expect("a left arrow");
        let right = bar.scroll_right.expect("a right arrow");
        assert!(!left.enabled && right.enabled);
        let buf = drawn(&bar, area);
        assert_eq!(
            buf[(left.rect.x + 1, 0)].symbol(),
            crate::tab_scroll::LEFT_GLYPH
        );
        assert_eq!(
            buf[(right.rect.x + 1, 0)].symbol(),
            crate::tab_scroll::RIGHT_GLYPH
        );
        assert_eq!(buf[(left.rect.x + 1, 0)].fg, th().ui.disabled, "dimmed");
        assert_ne!(buf[(right.rect.x + 1, 0)].fg, th().ui.disabled);
        assert_eq!(bar.hits[0].index, 0);
        for hit in &bar.hits {
            assert!(hit.tab.x >= left.rect.right() && hit.tab.right() <= right.rect.x);
        }
        let row = row_text(&buf, area);
        assert!(row.contains("repo0") && !row.contains("repo7"), "{row}");
        // Everything fits: no arrows at all.
        let (wide, _) = scrolled(&tabs(), 0, 80, BarScroll::default());
        assert!(wide.scroll_left.is_none() && wide.scroll_right.is_none());
    }

    /// Following the last tab brings it just into view at the right end of
    /// the strip; the left arrow comes alive and the right one is dimmed.
    #[test]
    fn following_the_last_tab_shows_it_whole() {
        let tabs = many(8);
        let follow = BarScroll {
            follow: Some(7),
            ..BarScroll::default()
        };
        let (bar, area) = scrolled(&tabs, 7, 50, follow);
        let last = bar.hits.last().expect("tabs drawn");
        assert_eq!(last.index, 7);
        let right = bar.scroll_right.unwrap();
        assert!(last.tab.right() <= right.rect.x);
        assert!(bar.scroll_left.unwrap().enabled && !right.enabled);
        let row = row_text(&drawn(&bar, area), area);
        assert!(row.contains("repo7 x"), "{row}");
        // A tab already in view does not move the window.
        let (same, _) = scrolled(
            &tabs,
            bar.hits[1].index,
            50,
            BarScroll {
                offset: bar.offset,
                follow: Some(bar.hits[1].index),
                ..BarScroll::default()
            },
        );
        assert_eq!(same.offset, bar.offset);
        // Following the first tab again scrolls back to it.
        let (back, _) = scrolled(
            &tabs,
            0,
            50,
            BarScroll {
                offset: bar.offset,
                follow: Some(0),
                ..BarScroll::default()
            },
        );
        assert_eq!(back.offset, 0);
        assert_eq!(back.hits[0].index, 0);
    }

    /// No room: not drawn, not registered.
    #[test]
    fn a_bar_too_narrow_for_the_plus_neither_draws_nor_registers_it() {
        let area = Rect::new(0, 0, 8, 1);
        let bar = tab_bar(&tabs(), 0, area, false);
        assert!(bar.plus.is_none());
        let line = row_text(&drawn(&bar, area), area);
        assert!(!line.contains('+'), "{line:?}");
    }

    /// An open New tab is a selected tab after the last repository tab (its
    /// hit has the index one past them), the `+` follows it, and the house
    /// is not the selected one.
    #[test]
    fn an_open_new_tab_sits_between_the_last_tab_and_the_plus() {
        let area = Rect::new(0, 0, 80, 1);
        let slot = NewTabSlot {
            open: true,
            plus_hovered: false,
            more_hovered: false,
            chip: None,
            radar: None,
            markup: Default::default(),
            workspaces: None,
            notes: None,
            scroll: BarScroll::default(),
        };
        let bar = tab_bar_with(&tabs(), None, area, false, true, None, None, slot);
        assert_eq!(bar.hits.len(), tabs().len() + 1);
        let new = bar.hits.last().unwrap();
        assert_eq!(new.index, tabs().len());
        let plus = bar.plus.unwrap();
        assert!(new.tab.x + new.tab.width <= plus.x);
        let buf = drawn(&bar, area);
        assert!(row_text(&buf, area).contains("New tab"));
        // The `x` is inside the tab's band and closes it.
        assert_eq!(
            tab_bar_click(&bar.hits, new.close.x, 0),
            Some(SwitcherAction::Close(tabs().len()))
        );
        // The house is the plain button, not the accent block.
        let folder = bar.folder.unwrap();
        assert_ne!(buf[(folder.x, 0)].bg, th().ui.accent);
    }

    /// The Radar is a pinned tab right after the house (and its chip),
    /// before the first repository tab, with no close button of its own; it
    /// is the selected one when open, and then the house is not.
    #[test]
    fn the_radar_is_pinned_after_the_house_and_selects_instead_of_it() {
        let area = Rect::new(0, 0, 100, 1);
        let slot = |open| NewTabSlot {
            radar: Some(RadarSlot {
                open,
                hovered: false,
                close_hovered: false,
            }),
            ..NewTabSlot::default()
        };
        let bar = tab_bar_with(&tabs(), None, area, false, true, None, None, slot(true));
        let pad = bar.radar.expect("room for the Radar tab");
        let folder = bar.folder.unwrap();
        assert_eq!(pad.x, folder.x + folder.width, "right after the house");
        assert!(bar.hits.iter().all(|h| h.tab.x >= pad.x + pad.width));
        let buf = drawn(&bar, area);
        assert_eq!(
            buf[(pad.x + 1, 0)].bg,
            crate::theme::selected_tab_bg(),
            "selected while open"
        );
        assert_ne!(
            buf[(folder.x, 0)].bg,
            th().ui.accent,
            "the house is not selected on the Radar"
        );
        let closed = tab_bar_with(&tabs(), None, area, false, true, None, None, slot(false));
        assert_eq!(closed.radar, Some(pad));
        let buf = drawn(&closed, area);
        assert_eq!(buf[(folder.x, 0)].bg, th().ui.accent, "Home selected again");
        let none = tab_bar_with(
            &tabs(),
            None,
            area,
            false,
            true,
            None,
            None,
            NewTabSlot::default(),
        );
        assert_eq!(none.radar, None, "a build without it has no tab");
        assert_eq!(none.radar_close, None);
    }

    /// The Workspaces tab sits after the Radar, shows an `x` while open, and
    /// takes the selection from the house; absent, it has no rects.
    #[test]
    fn the_workspaces_tab_follows_the_radar_and_has_a_close_button() {
        let area = Rect::new(0, 0, 100, 1);
        let with = |open: bool| NewTabSlot {
            radar: Some(RadarSlot::default()),
            workspaces: Some(RadarSlot {
                open,
                ..RadarSlot::default()
            }),
            ..NewTabSlot::default()
        };
        let bar = tab_bar_with(&tabs(), None, area, false, true, None, None, with(true));
        let radar = bar.radar.expect("the Radar tab");
        let ws = bar.workspaces.expect("the Workspaces tab");
        assert_eq!(ws.x, radar.x + radar.width, "right after the Radar");
        let close = bar.workspaces_close.expect("an x on the open tab");
        assert!(close.x >= ws.x && close.x < ws.x + ws.width);
        let idle = tab_bar_with(&tabs(), None, area, false, true, None, None, with(false));
        assert_eq!(idle.workspaces_close, None, "no x on an idle tab");
        let none = tab_bar_with(
            &tabs(),
            None,
            area,
            false,
            true,
            None,
            None,
            NewTabSlot::default(),
        );
        assert_eq!((none.workspaces, none.workspaces_close), (None, None));
    }

    /// The Radar tab closes like a repository tab: an `x` inside the tab,
    /// drawn by the same walk, one cell, painted as "x", only while the tab
    /// is open or hovered (blank cell, same width, no rect otherwise).
    #[test]
    fn the_radar_tab_has_a_close_button_inside_it() {
        let area = Rect::new(0, 0, 100, 1);
        let with = |l: RadarSlot| NewTabSlot {
            radar: Some(l),
            ..NewTabSlot::default()
        };
        let idle = tab_bar_with(
            &tabs(),
            None,
            area,
            false,
            true,
            None,
            None,
            with(RadarSlot::default()),
        );
        assert_eq!(idle.radar_close, None, "no x on an idle tab");
        let idle_pad = idle.radar.unwrap();
        assert!(!drawn(&idle, area)
            .content
            .iter()
            .skip(usize::from(idle_pad.x))
            .take(usize::from(idle_pad.width))
            .any(|c| c.symbol() == "x"));
        let hovered = RadarSlot {
            hovered: true,
            ..RadarSlot::default()
        };
        let bar = tab_bar_with(&tabs(), None, area, false, true, None, None, with(hovered));
        let pad = bar.radar.unwrap();
        assert_eq!(
            pad.width, idle_pad.width,
            "the width does not jump on hover"
        );
        let close = bar.radar_close.expect("an x");
        assert!(
            close.x > pad.x && close.x < pad.x + pad.width,
            "inside the tab"
        );
        assert_eq!(close.width, 1);
        assert_eq!(drawn(&bar, area)[(close.x, 0)].symbol(), "x");
        assert!(
            bar.hits.iter().all(|h| h.tab.x >= pad.x + pad.width),
            "repository tabs still start after it"
        );
    }
}
