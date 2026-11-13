//! The chrome block of the draw closure.

use super::super::*;

/// The rows the chrome leaves for the panes below it, which the footer
/// and the management overlay paint into.
pub(in crate::event_loop) struct ChromeAreas {
    pub(in crate::event_loop) inner: Rect,
    pub(in crate::event_loop) tab_bar_area: Rect,
    pub(in crate::event_loop) toolbar_area: Rect,
    pub(in crate::event_loop) footer_area: Rect,
}

pub(in crate::event_loop) struct ChromeDrawCtx<'a> {
    /// Rows the footer needs: more than one while a long Ctrl+G / Ctrl+N hint wraps.
    pub(in crate::event_loop) footer_rows: u16,
    pub(in crate::event_loop) active_tab: usize,
    pub(in crate::event_loop) divider1_pct: u16,
    pub(in crate::event_loop) divider2_pct: u16,
    pub(in crate::event_loop) left_pane_collapsed: bool,
    pub(in crate::event_loop) right_pane_collapsed: bool,
    pub(in crate::event_loop) undo_avail: crate::undo_ui::Avail,
    pub(in crate::event_loop) redo_avail: crate::undo_ui::Avail,
    pub(in crate::event_loop) auto_stash_pending: bool,
    /// Git LFS is turned off for this repository: no `LFS` toolbar button.
    pub(in crate::event_loop) lfs_hidden: bool,
    pub(in crate::event_loop) hovered_close_tab: Option<usize>,
    pub(in crate::event_loop) hovered_tab_index: Option<usize>,
    pub(in crate::event_loop) title: &'a String,
    pub(in crate::event_loop) repo_tabs: &'a Vec<repo_switcher::RepoTab>,
    pub(in crate::event_loop) hovered_target: &'a Option<HitTarget>,
    pub(in crate::event_loop) management: &'a Option<crate::repo_manager::RepoManager>,
    pub(in crate::event_loop) hosting: &'a HostingUi,
    pub(in crate::event_loop) stashes_result:
        &'a gitpoulpi_core::Result<Vec<gitpoulpi_core::domain::Stash>>,
    pub(in crate::event_loop) frame_hits: &'a mut Vec<(Rect, HitTarget)>,
    pub(in crate::event_loop) overflow_slots: &'a mut Vec<(OverflowBar, Rect, Vec<OverflowItem>)>,
    pub(in crate::event_loop) remote_ui: &'a mut RemoteUi,
    pub(in crate::event_loop) tab_hits: &'a mut Vec<repo_switcher::TabHit>,
    pub(in crate::event_loop) panes_area: &'a mut Rect,
    pub(in crate::event_loop) refs_area: &'a mut Rect,
    pub(in crate::event_loop) log_area: &'a mut Rect,
    pub(in crate::event_loop) status_area: &'a mut Rect,
}

/// The main header's text: the app name and the repository path. The
/// session name lives in the status bar only (once on screen).
pub(in crate::event_loop) fn header_title(root: &std::path::Path) -> String {
    format!("GitPoulpi -- {}", root.display())
}

/// Between the path and the "Open in" menu, spaces included.
const OPEN_IN_SEPARATOR: &str = " - ";

pub(in crate::event_loop) fn draw_chrome(
    frame: &mut Frame,
    ctx: &mut ChromeDrawCtx<'_>,
) -> ChromeAreas {
    let active_tab = ctx.active_tab;
    let divider1_pct = ctx.divider1_pct;
    let divider2_pct = ctx.divider2_pct;
    let left_pane_collapsed = ctx.left_pane_collapsed;
    let right_pane_collapsed = ctx.right_pane_collapsed;
    let undo_avail = ctx.undo_avail;
    let redo_avail = ctx.redo_avail;
    let hovered_close_tab = ctx.hovered_close_tab;
    let hovered_tab_index = ctx.hovered_tab_index;
    let title = ctx.title;
    let repo_tabs = ctx.repo_tabs;
    let hovered_target = ctx.hovered_target;
    let management = ctx.management;
    let hosting = ctx.hosting;
    let stashes_result = ctx.stashes_result;
    let frame_hits = &mut *ctx.frame_hits;
    let overflow_slots = &mut *ctx.overflow_slots;
    let remote_ui = &mut *ctx.remote_ui;
    let tab_hits = &mut *ctx.tab_hits;
    let panes_area = &mut *ctx.panes_area;
    let refs_area = &mut *ctx.refs_area;
    let log_area = &mut *ctx.log_area;
    let status_area = &mut *ctx.status_area;
    // The title row's tab-folder button ("Open in"): next to the repository
    // path, when the active tab is not the launch folder.
    let active_path = repo_tabs
        .get(active_tab)
        .map(|t| t.path.as_path())
        .filter(|_| management.is_none());
    let tab_dir_buttons = crate::tab_dir::visible_buttons(active_path);
    let open_in_visible = tab_dir_buttons.contains(&crate::tab_dir::TabDirButton::Terminal);
    // Where the title bar's "Open in" goes this frame; painted (and made the
    // popup's anchor) after the fallback anchor below is set.
    let mut open_in_button = None;
    crate::updates::chip::clear();
    let inner = {
        let outer = Block::default().borders(Borders::ALL);
        let area = outer.inner(frame.area());
        frame.render_clipped(outer, frame.area());
        // The title is painted by hand, after the octopus that opens About:
        // `┌🐙 GitPoulpi -- /path - [Open in]`. Hit rects and the close
        // button's room come from the same positions.
        let title_y = frame.area().y;
        let after_octopus = render_about_button(
            frame,
            frame.area().x + 1,
            title_y,
            hover(hovered_target).about(),
            &mut *frame_hits,
        );
        // `GitPoulpi`, the version chip (it opens the Release notes), then
        // `-- /path`. The chip's own padding is the spaces around it, and it
        // gives way (full label, bare version, none) before the path, the
        // quit button and the gear lose their room.
        let right_limit = frame.area().right().saturating_sub(1);
        let (name, path) = match title.split_once(' ') {
            Some((name, rest)) => (name, rest),
            None => (title.as_str(), ""),
        };
        let name_w = display_width(name) as u16;
        frame.render_clipped(
            Paragraph::new(name),
            Rect::new(
                after_octopus,
                title_y,
                name_w.min(right_limit.saturating_sub(after_octopus)),
                1,
            ),
        );
        let chip_x = after_octopus + name_w;
        let tail_w = 3 + crate::chrome::gear_reserve() + 12;
        let chip_w = crate::updates::chip::fit(right_limit.saturating_sub(chip_x + tail_w));
        crate::updates::chip::paint(frame.buffer_mut(), chip_x, title_y, chip_w);
        let path_x = if chip_w > 0 {
            chip_x + chip_w
        } else {
            chip_x + 1
        };
        let path_w = display_width(path) as u16;
        frame.render_clipped(
            Paragraph::new(path),
            Rect::new(
                path_x,
                title_y,
                path_w.min(right_limit.saturating_sub(path_x)),
                1,
            ),
        );
        let title_end_x = path_x + path_w;
        // "Open in", after a " - " separator, only when it fits before the
        // close button (the palette's `Tab: Open in...` reaches it always).
        if open_in_visible {
            let w = crate::open_in_bar::button_width();
            let sep_w = display_width(OPEN_IN_SEPARATOR) as u16;
            let x = title_end_x + sep_w;
            // The quit button, and the gear with its gap, are to its right.
            let close_w = 3 + crate::chrome::gear_reserve();
            if x + w + close_w < frame.area().right() {
                frame.render_clipped(
                    Paragraph::new(OPEN_IN_SEPARATOR),
                    Rect::new(title_end_x, title_y, sep_w, 1),
                );
                let rect = Rect::new(x, title_y, w, 1);
                open_in_button = Some(rect);
                frame_hits.push((
                    rect,
                    HitTarget::TabDir(crate::tab_dir::TabDirButton::Terminal),
                ));
            }
        }
        let title_clear_x = open_in_button.map_or(title_end_x, |r| r.right());
        let close = render_close_app_button(
            frame,
            frame.area(),
            title_end_x,
            hover(hovered_target).close_app(),
            &mut *frame_hits,
        );
        // The settings gear sits just left of the quit button, with a gap
        // so a click aimed at it never quits.
        if let Some(close) = close {
            if let Some(gear) = render_gear(
                frame,
                close,
                title_clear_x,
                hover(hovered_target).gear(),
                &mut *frame_hits,
            ) {
                // Search, flush left of the gear; it gives way first. A build
                // without the global search draws nothing here (nothing else
                // reserves room for it, so the title bar keeps its layout).
                if crate::build_features::GLOBAL_SEARCH {
                    crate::chrome::render_search_button(
                        frame,
                        gear,
                        title_clear_x,
                        hover(hovered_target).global_search(),
                        &mut *frame_hits,
                    );
                }
            }
        }
        area
    };

    // The bands: the repository tab bar scopes everything below it, so it
    // stays at the top (under the status line when that is on top); the
    // toolbar is a row above or below the panes, or a column beside them
    // (`ui.toolbar_position`); the status line is the first or the last row
    // (`ui.statusbar_position`). `chrome::split` is the one place that knows.
    let profile_w = if crate::profile_bar::visible() && management.is_none() {
        crate::profile_bar::field_width()
    } else {
        0
    };
    let hosting_chip = hosting.chip();
    let enabled = crate::chrome::ToolbarEnabled {
        pop: stashes_result.as_ref().is_ok_and(|s| !s.is_empty()),
        undo: undo_avail == crate::undo_ui::Avail::Ready,
        redo: redo_avail == crate::undo_ui::Avail::Ready,
        undo_hidden: undo_avail == crate::undo_ui::Avail::Off,
        auto_stash_blocks: ctx.auto_stash_pending,
        lfs_hidden: ctx.lfs_hidden,
        right_reserve: profile_w,
    };
    let layout =
        crate::live_settings::chrome_layout().effective(frame.area().width, management.is_some());
    let column_w = if layout.is_vertical() {
        crate::chrome::toolbar_column_width(enabled)
    } else {
        0
    };
    let crate::chrome::Regions {
        tab_bar: tab_bar_area,
        toolbar: toolbar_area,
        panes: computed_panes_area,
        footer: footer_area,
    } = crate::chrome::split(inner, ctx.footer_rows, column_w, layout);
    *panes_area = computed_panes_area;

    // The "Quit & cd here" button takes the tab row's right-hand end when
    // the active tab is not the launch folder; the tabs get the rest. "Open
    // in", the global search button and the settings gear live in the title
    // bar.
    let tab_dir_buttons: Vec<_> = tab_dir_buttons
        .into_iter()
        .filter(|b| *b != crate::tab_dir::TabDirButton::Terminal)
        .collect();
    // Where the "Open in" popup hangs from when the button itself is not drawn
    // (the launch folder, a narrow bar, no title bar) and the palette opened
    // the list.
    crate::open_in_bar::set_bar(Rect::new(
        frame.area().x,
        frame.area().y,
        frame.area().width,
        1,
    ));
    // After the fallback: a drawn button is the popup's anchor.
    if let Some(rect) = open_in_button {
        crate::open_in_bar::render_button(
            frame.buffer_mut(),
            rect,
            hover(hovered_target).tab_dir() == Some(crate::tab_dir::TabDirButton::Terminal),
        );
    }
    let column = layout.tabs == gitpoulpi_config::TabsPosition::Left;
    // Where the folder buttons go, and the part of the tab band left for the
    // tabs themselves: the row's end on top and at the bottom, the column's
    // last row on the left.
    let mut dirs: Option<(Rect, u16)> = None;
    let tabs_area = if column {
        let col = tab_bar_area;
        let mut list = col;
        if col.height >= 8 && col.width >= 4 {
            if let Some(w) =
                crate::chrome::tab_dir_buttons_width(&tab_dir_buttons, Rect::new(0, 0, u16::MAX, 1))
            {
                if w <= col.width {
                    dirs = Some((Rect::new(col.x, col.bottom() - 1, col.width, 1), w));
                    list.height -= 1;
                }
            }
        }
        list
    } else {
        let tab_row = tab_bar_area;
        match crate::chrome::tab_dir_buttons_width(&tab_dir_buttons, tab_row) {
            Some(w) => {
                dirs = Some((tab_row, w));
                Rect {
                    width: tab_row.width - w - 1,
                    ..tab_row
                }
            }
            None => tab_row,
        }
    };
    // The horizontal bar's scroll: the active tab is brought into view when
    // it, the number of tabs or the bar's width changed (`tab_scroll`).
    let (scroll_offset, scroll_follow) = crate::tab_scroll::scroll_for_draw(
        (!column && management.is_none()).then_some(active_tab),
        repo_tabs.len(),
        tabs_area.width,
    );
    let slot = repo_switcher::NewTabSlot {
        scroll: repo_switcher::BarScroll {
            offset: scroll_offset,
            follow: scroll_follow,
            left_hovered: hover(hovered_target).tab_scroll_left(),
            right_hovered: hover(hovered_target).tab_scroll_right(),
        },
        open: management.as_ref().is_some_and(|m| m.new_tab),
        plus_hovered: hover(hovered_target).new_tab(),
        more_hovered: hover(hovered_target).tab_overflow(),
        workspaces: crate::build_features::workspaces_tab_shown().then(|| {
            repo_switcher::RadarSlot {
                open: management
                    .as_ref()
                    .is_some_and(|m| m.page == crate::repo_manager::HomePage::Workspaces),
                hovered: hover(hovered_target).workspaces_tab(),
                close_hovered: hover(hovered_target).workspaces_close(),
            }
        }),
        radar: crate::build_features::radar_tab_shown().then(|| repo_switcher::RadarSlot {
            open: management.as_ref().is_some_and(|m| m.radar.is_open()),
            hovered: hover(hovered_target).radar_tab(),
            close_hovered: hover(hovered_target).radar_close(),
        }),
        notes: management
            .as_ref()
            .is_some_and(|m| m.notes.is_open())
            .then(|| repo_switcher::NotesSlot {
                close_hovered: hover(hovered_target).notes_close(),
            }),
        chip: crate::workspace_ui::chip::look().map(|l| repo_switcher::ChipSlot {
            dot: l.dot,
            label: l.label,
            none: l.none,
            hovered: hover(hovered_target).workspace_chip(),
            open: crate::workspace_ui::chip::is_open(),
        }),
    };
    let bar = if column {
        let (scroll, follow) =
            crate::tab_column::scroll_for_draw(management.is_none().then_some(active_tab));
        let col = crate::tab_column::tab_column(
            tabs_area,
            crate::tab_column::ColumnArgs {
                tabs: repo_tabs,
                active: Some(active_tab),
                management: management.is_some(),
                folder_hovered: hover(hovered_target).folder(),
                hovered_tab: hovered_tab_index,
                hovered_close: hovered_close_tab,
                new_tab: slot,
                scroll,
                follow,
            },
        );
        crate::tab_column::store_scroll(col.offset);
        crate::tab_column::set_area(Some(tab_bar_area));
        col.bar
    } else {
        crate::tab_column::set_area(None);
        repo_switcher::tab_bar_with(
            repo_tabs,
            Some(active_tab),
            tabs_area,
            hover(hovered_target).folder(),
            management.is_some(),
            hovered_tab_index,
            hovered_close_tab,
            slot,
        )
    };
    // The chip's popup hangs from it (or from the bar's left end when the
    // bar was too short to draw it).
    crate::workspace_ui::chip::set_anchor(bar.chip.or(Some(Rect::new(
        tabs_area.x,
        tabs_area.y,
        1,
        1,
    ))));
    if column {
        for (rect, line) in &bar.rows {
            frame.render_clipped(Paragraph::new(line.clone()), *rect);
        }
    } else {
        frame.render_clipped(Paragraph::new(bar.line), tabs_area);
    }
    // The tab an agent's intent changed the view of carries a purple ✦ on
    // its leading pad cell, until the user's next key or click.
    if let Some(marked) = crate::ui_intent::marked_tab() {
        for hit in &bar.hits {
            let marked_here = repo_tabs.get(hit.index).is_some_and(|t| t.path == marked);
            if marked_here && hit.tab.width > 2 {
                if let Some(cell) = frame.buffer_mut().cell_mut((hit.tab.x, hit.tab.y)) {
                    cell.set_symbol("\u{2726}").set_fg(th().chrome.marker);
                }
            }
        }
    }
    *tab_hits = bar.hits;
    if let Some((area, width)) = dirs {
        crate::chrome::render_tab_dir_buttons(
            frame,
            area,
            &tab_dir_buttons,
            width,
            hover(hovered_target).tab_dir(),
            &mut *frame_hits,
        );
    }

    // First into the list, and the only thing on its row: the
    // inner-first rule has nothing to fight over here, but a
    // rect registered after a band that covered it is how this
    // project has lost a button four times.
    if let Some(folder) = bar.folder {
        frame_hits.push((folder, HitTarget::OpenRepositoryButton));
    }
    if let Some(plus) = bar.plus {
        frame_hits.push((plus, HitTarget::NewTabButton));
    }
    if let Some(more) = bar.more {
        frame_hits.push((more, HitTarget::TabOverflow));
    }
    // The scroll arrows: registered (and auto-repeating while held) only
    // while there are tabs to show on their side.
    let live = |a: Option<repo_switcher::ScrollArrow>| a.filter(|a| a.enabled).map(|a| a.rect);
    let (left, right) = (live(bar.scroll_left), live(bar.scroll_right));
    if let Some(rect) = left {
        frame_hits.push((rect, HitTarget::TabScrollLeft));
        crate::scrollbar::note_repeat_button(rect, crate::tab_scroll::LEFT_GLYPH);
    }
    if let Some(rect) = right {
        frame_hits.push((rect, HitTarget::TabScrollRight));
        crate::scrollbar::note_repeat_button(rect, crate::tab_scroll::RIGHT_GLYPH);
    }
    if !column {
        crate::tab_scroll::store(bar.offset, Some(tabs_area), left, right);
    } else {
        crate::tab_scroll::store(0, None, None, None);
    }
    if let Some(chip) = bar.chip {
        frame_hits.push((chip, HitTarget::WorkspaceChip));
    }
    crate::build_features::install_tabs_drawn(bar.radar.is_some(), bar.workspaces.is_some());
    // The `x` first: the first rect containing the point wins.
    if let Some(close) = bar.radar_close {
        frame_hits.push((close, HitTarget::RadarClose));
    }
    if let Some(radar) = bar.radar {
        frame_hits.push((radar, HitTarget::RadarTab));
    }
    if let Some(close) = bar.workspaces_close {
        frame_hits.push((close, HitTarget::WorkspacesClose));
    }
    if let Some(ws) = bar.workspaces {
        frame_hits.push((ws, HitTarget::WorkspacesTab));
    }
    if let Some(close) = bar.notes_close {
        frame_hits.push((close, HitTarget::NotesClose));
    }
    if let Some(notes) = bar.notes {
        frame_hits.push((notes, HitTarget::NotesTab));
    }
    // The toolbar, laid out by `toolbar_render_with` / `toolbar_column`
    // (`chrome/toolbar.rs`): buttons that do not fit go behind a `⋯`, and
    // every rect registered here comes from the same walk that produced the
    // painted line.
    let place = crate::chrome::ToolbarPlacement {
        align: layout.align,
        pr_pinned: layout.pr_pinned,
        edge: layout.toolbar,
    };
    let toolbar = if layout.is_vertical() {
        crate::chrome::toolbar_column(
            toolbar_area,
            hovered_target.as_ref(),
            hosting_chip.as_ref(),
            enabled,
            place,
        )
    } else {
        toolbar_render_with(
            toolbar_area,
            hovered_target.as_ref(),
            hosting_chip.as_ref(),
            enabled,
            place,
        )
    };
    frame_hits.extend(toolbar.hits.iter().cloned());
    if let Some((rect, items)) = toolbar.overflow.clone() {
        overflow_slots.push((OverflowBar::Toolbar, rect, items));
    }
    remote_ui.set_toolbar_anchors(
        toolbar.push_anchor,
        toolbar.pull_anchor,
        toolbar.lfs_anchor,
        toolbar.flow_anchor,
    );
    toolbar.paint(frame, toolbar_area);

    // The per-repository profile combobox, pinned to the toolbar's right
    // end (only with two or more profiles), or to the last row of a column.
    // The buttons give way to it.
    let profile_rect = if layout.is_vertical() {
        (profile_w > 0 && toolbar_area.height >= 6 && toolbar_area.width >= 8).then(|| {
            Rect::new(
                toolbar_area.x,
                toolbar_area.bottom() - 1,
                toolbar_area.width,
                1,
            )
        })
    } else {
        (profile_w > 0 && toolbar_area.width > profile_w + 1).then(|| {
            Rect::new(
                toolbar_area.right() - profile_w,
                toolbar_area.y,
                profile_w,
                1,
            )
        })
    };
    if let Some(rect) = profile_rect {
        crate::profile_bar::render_field(
            frame.buffer_mut(),
            rect,
            hover(hovered_target).profile_field(),
        );
        frame_hits.push((rect, HitTarget::ProfileField));
    }

    let [computed_refs_area, computed_log_area, computed_status_area] = pane_areas(
        *panes_area,
        divider1_pct,
        divider2_pct,
        left_pane_collapsed,
        right_pane_collapsed,
    );
    *refs_area = computed_refs_area;
    *log_area = computed_log_area;
    *status_area = computed_status_area;

    ChromeAreas {
        inner,
        tab_bar_area,
        toolbar_area,
        footer_area,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_header_names_the_path_not_the_session_and_open_in_follows_a_spaced_hyphen() {
        let title = header_title(std::path::Path::new("/work/repo"));
        assert_eq!(title, "GitPoulpi -- /work/repo");
        assert!(!title.contains("session"));
        assert_eq!(OPEN_IN_SEPARATOR, " - ");
    }
}
