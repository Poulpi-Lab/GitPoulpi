//! The overlays of the draw closure that sit over the panes: the management tab, navigation badges, the palette, tab picker and form popups.

use super::super::*;

pub(in crate::event_loop) struct OverlaysDrawCtx<'a> {
    pub(in crate::event_loop) active_tab: usize,
    pub(in crate::event_loop) central_view: CentralView,
    pub(in crate::event_loop) commit_files_area: Rect,
    pub(in crate::event_loop) editor: &'a Option<crate::editor::Editor>,
    pub(in crate::event_loop) merge: &'a Option<crate::merge_view::MergeView>,
    pub(in crate::event_loop) field_editor: &'a crate::text_edit::FieldEditor<MainField>,
    pub(in crate::event_loop) file_insight: &'a Option<FileInsight>,
    pub(in crate::event_loop) file_insight_area: &'a mut Rect,
    pub(in crate::event_loop) frame_hits: &'a mut Vec<(Rect, HitTarget)>,
    pub(in crate::event_loop) home_mode_hint: &'a Option<String>,
    pub(in crate::event_loop) hovered_target: &'a Option<HitTarget>,
    pub(in crate::event_loop) inner: Rect,
    pub(in crate::event_loop) input_mode: &'a InputMode,
    pub(in crate::event_loop) journal_state: &'a gitpoulpi_core::domain::JournalState,
    pub(in crate::event_loop) left_pane_collapsed: bool,
    pub(in crate::event_loop) log_area: Rect,
    /// The commit column's rect (empty when not drawn) and what the view got
    /// of the central column beside it.
    pub(in crate::event_loop) commit_column_area: Rect,
    pub(in crate::event_loop) view_area: Rect,
    pub(in crate::event_loop) management: &'a mut Option<repo_manager::RepoManager>,
    pub(in crate::event_loop) management_panel: &'a mut Option<(Rect, repo_manager::ManagerRender)>,
    pub(in crate::event_loop) nav_badge_targets: &'a mut Vec<(char, FocusTarget)>,
    pub(in crate::event_loop) palette_ctx: PaletteContext,
    /// A repository dialog is open: the palette (opened from the Undo
    /// history panel) is painted over it by `draw_overlay_menus`.
    pub(in crate::event_loop) palette_over_dialog: bool,
    /// The Settings screen is open: the palette and the pickers are painted
    /// over it by `draw_overlay_menus` (`draw_picker_popups`), not here.
    pub(in crate::event_loop) settings_open: bool,
    pub(in crate::event_loop) palette_popup: &'a mut Option<PalettePopup>,
    pub(in crate::event_loop) rebase_state: &'a Option<gitpoulpi_core::domain::RebaseState>,
    /// The refusal shown inside the name box, and who holds the name field.
    pub(in crate::event_loop) ref_name_error: &'a Option<String>,
    pub(in crate::event_loop) name_slot: &'a mut NameSlot,
    pub(in crate::event_loop) operation_state: &'a Option<gitpoulpi_core::domain::OperationState>,
    pub(in crate::event_loop) conflict_count: usize,
    pub(in crate::event_loop) index_lock: &'a Option<gitpoulpi_core::domain::IndexLock>,
    pub(in crate::event_loop) pending_auto_stash: &'a Option<gitpoulpi_session::PendingAutoStash>,
    /// The predicted conflicts to badge (`conflict_badge`), already checked
    /// against the HEAD that is checked out.
    pub(in crate::event_loop) conflict_prediction:
        &'a Option<gitpoulpi_core::domain::conflict_prevention::ConflictPrediction>,
    pub(in crate::event_loop) refs_filter_area: Rect,
    pub(in crate::event_loop) repo_tabs: &'a Vec<repo_switcher::RepoTab>,
    pub(in crate::event_loop) right_pane_collapsed: bool,
    pub(in crate::event_loop) right_panel: RightPanel,
    pub(in crate::event_loop) sidebar_section_areas: &'a Vec<(SidebarSection, Rect)>,
    pub(in crate::event_loop) status_area: Rect,
    pub(in crate::event_loop) status_section_areas: &'a Vec<(StatusSection, Rect)>,
    pub(in crate::event_loop) tab_bar_area: Rect,
    pub(in crate::event_loop) tab_picker_popup: &'a mut Option<PalettePopup>,
    pub(in crate::event_loop) ui_graph_theme: crate::graph::LaneTheme,
    pub(in crate::event_loop) undo_busy: bool,
}

/// The confirmation moves (never resizes: its size follows its text).
/// The tab picker moves and resizes: the list grows with the height.
const TAB_PICKER_WIN: crate::dialog_kit::Win =
    crate::dialog_kit::Win::resizable("tab_picker", 30, 8);

const CONFIRM_WIN: crate::dialog_kit::Win = crate::dialog_kit::Win::movable("confirm_prompt");

pub(in crate::event_loop) fn draw_overlays(frame: &mut Frame, ctx: &mut OverlaysDrawCtx<'_>) {
    let active_tab = ctx.active_tab;
    let central_view = ctx.central_view;
    let commit_files_area = ctx.commit_files_area;
    let editor = ctx.editor;
    let merge = ctx.merge;
    let field_editor = ctx.field_editor;
    let file_insight = ctx.file_insight;
    let mut file_insight_area = *ctx.file_insight_area;
    let frame_hits = &mut *ctx.frame_hits;
    let home_mode_hint = ctx.home_mode_hint;
    let hovered_target = ctx.hovered_target;
    let inner = ctx.inner;
    let input_mode = ctx.input_mode;
    let journal_state = ctx.journal_state;
    let left_pane_collapsed = ctx.left_pane_collapsed;
    let log_area = ctx.log_area;
    let commit_column_area = ctx.commit_column_area;
    let view_area = ctx.view_area;
    let management = &mut *ctx.management;
    let management_panel = &mut *ctx.management_panel;
    let nav_badge_targets = &mut *ctx.nav_badge_targets;
    let palette_ctx = ctx.palette_ctx;
    let palette_over_dialog = ctx.palette_over_dialog;
    let settings_open = ctx.settings_open;
    let palette_popup = &mut *ctx.palette_popup;
    let rebase_state = ctx.rebase_state;
    let ref_name_error = ctx.ref_name_error;
    let name_slot = &mut *ctx.name_slot;
    let operation_state = ctx.operation_state;
    let conflict_count = ctx.conflict_count;
    let index_lock = ctx.index_lock;
    let pending_auto_stash = ctx.pending_auto_stash;
    let conflict_prediction = ctx.conflict_prediction;
    let refs_filter_area = ctx.refs_filter_area;
    let repo_tabs = ctx.repo_tabs;
    let right_pane_collapsed = ctx.right_pane_collapsed;
    let right_panel = ctx.right_panel;
    let sidebar_section_areas = ctx.sidebar_section_areas;
    let status_area = ctx.status_area;
    let status_section_areas = ctx.status_section_areas;
    let tab_bar_area = ctx.tab_bar_area;
    let tab_picker_popup = &mut *ctx.tab_picker_popup;
    let ui_graph_theme = ctx.ui_graph_theme;
    let undo_busy = ctx.undo_busy;
    // The Repository Management tab: everything under the tab
    // bar, painted over the (empty) panes of the no-repository
    // window. Drawn before every popup (palette, confirm, menus,
    // the repository overlay) so those still sit on top of it.
    // The panes' hit targets under it are dropped, so no hover,
    // click or resize strip can reach them through the view.
    *management_panel = None;
    crate::activity_log::set_home(false);
    if let Some(manager) = management.as_mut() {
        // Everything the tab bar leaves: under it, above it or right of it.
        let tabs = crate::live_settings::chrome_layout()
            .effective(frame.area().width, true)
            .tabs;
        let body = crate::chrome::home_body(inner, tab_bar_area, tabs);
        frame.render_clipped(Clear, body);
        manager.mode_hint = home_mode_hint.clone();
        manager.radar.set_mode_hint(home_mode_hint.clone());
        // Home's bottom row is a status bar like a repository tab's: the
        // Activity Log button at its right end (it opens the Global tab here,
        // `activity_log::set_home`), the jobs indicator left of it.
        crate::activity_log::set_home(true);
        let bar = Rect::new(body.x, body.bottom().saturating_sub(1), body.width, 1);
        let button_w = crate::activity_log::status_button_width();
        let covered = manager.radar.is_open() || manager.notes.is_open();
        let listed = (!covered).then(|| repo_manager::render(manager, body));
        let hints_w = listed
            .as_ref()
            .and_then(|r| r.lines.last())
            .map_or(0, |l| l.width() as u16);
        let paint_buttons = |frame: &mut Frame| {
            crate::activity_log::paint_status_button(frame.buffer_mut(), bar);
        };
        let render = if manager.radar.is_open() || manager.notes.is_open() {
            // The Radar and the Release notes cover the list; each keeps
            // its own hit rects. The list's bar is not there to press.
            manager.bar = None;
            if manager.notes.is_open() {
                manager.notes.render(frame, body);
            } else {
                manager.radar.render(frame, body);
            }
            crate::jobs_ui::clear_indicator();
            paint_buttons(frame);
            repo_manager::ManagerRender {
                lines: Vec::new(),
                hits: Vec::new(),
            }
        } else {
            let render = listed.expect("laid out above when not covered");
            frame.render_clipped(Paragraph::new(render.lines.clone()), body);
            if let Some(track) = manager.bar {
                crate::scrollbar::render(frame, track, true, false);
            }
            paint_buttons(frame);
            // The jobs indicator ends the key-hint line, where room is left.
            let reserved = if bar.width >= button_w + 40 {
                button_w
            } else {
                0
            };
            crate::jobs_ui::paint_indicator(
                frame.buffer_mut(),
                bar,
                bar.right().saturating_sub(reserved + 1),
                body.x + hints_w + 1,
            );
            render
        };
        crate::theme::paint_statusbar(frame.buffer_mut(), bar);
        // The footer is under this tab: a failure is painted on its bar.
        crate::activity_log::status_error::note_covered(bar);
        frame_hits.retain(|(rect, _)| !rect.intersects(body));
        *management_panel = Some((body, render));

        // No gear of its own: the tab row's (the window's) is drawn on Home too.
    }

    // The Status pane's own collapse affordance -- same reasoning
    // as the Refs pane's, painted last over whatever the (also
    // near-zero-width once collapsed) status render pass above
    // attempted. Not on the Repository Management tab, which has
    // no panes to collapse and is drawn above.
    if management.is_some() {
    } else if right_pane_collapsed {
        render_collapsed_pane_strip(
            frame,
            status_area,
            PaneSide::Status,
            hover(hovered_target).pane_restore() == Some(PaneSide::Status),
            frame_hits,
        );
    } else {
        render_pane_collapse_button(
            frame,
            status_area,
            PaneSide::Status,
            hover(hovered_target).pane_collapse() == Some(PaneSide::Status),
            frame_hits,
        );
    }

    // `Ctrl+N` navigation mode's badges -- the visual half of "press a badge
    // to focus that sub-pane" (see `FocusTarget`, `NavigationAction::FocusByBadge`
    // and the `InputMode::Navigation` arm). Every focusable sub-pane in *this*
    // frame gets one badge, built in a fixed order (Refs filter, each visible
    // Refs section, Log, each visible Status section) and assigned by
    // `assign_navigation_badges`, the one place that decides "1-9 then
    // letters", so the painted glyph and the key that acts can never
    // disagree. A collapsed side pane contributes nothing. `nav_badge_targets`
    // is captured for the key handler: the same "computed while drawing,
    // consulted on the next key/click" shape as `frame_hits`/`hit_targets`.
    // Settings covers the window and badges itself (`draw_overlay_menus`, after
    // it is painted): nothing of the panes under it may be badged.
    // A repository dialog or the Activity Log is over the panes the same way (each
    // badges itself, after it is painted).
    if matches!(input_mode, InputMode::Navigation)
        && !settings_open
        && !palette_over_dialog
        && !crate::activity_log::is_open()
    {
        let mut pairs: Vec<(Rect, FocusTarget)> = Vec::new();
        // The Repository Management tab is painted *over* the (empty)
        // Refs/Log/Status panes of the no-repository window; badging those too
        // would draw badges through the tab's content, so only one of these two
        // branches contributes pane targets.
        if management.is_none() {
            if !left_pane_collapsed {
                pairs.push((refs_filter_area, FocusTarget::RefsFilter));
                for (section, area) in sidebar_section_areas.iter() {
                    pairs.push((*area, FocusTarget::RefsSection(*section)));
                }
            }
            // The Diff view replaces the Log table in the same `log_area` rect, so it
            // is badged there: the badge sits on the pane that is on screen. The File
            // View badges itself below, from its own rect (the central column). The
            // Editor renders into the same rect too (`ed.render(frame, log_area, ..)`
            // above) and is just as focusable, so it is badged here as well.
            // The commit column, left of the view that replaced the Log: its own
            // badge, and the view's badge moves to the part the view got.
            if commit_column_area.width > 0 {
                pairs.push((commit_column_area, FocusTarget::CommitColumn));
            }
            let view_area = if commit_column_area.width > 0 {
                view_area
            } else {
                log_area
            };
            match central_view {
                CentralView::Diff => pairs.push((view_area, FocusTarget::DiffView)),
                CentralView::Log => pairs.push((log_area, FocusTarget::Log)),
                CentralView::Editor if editor.is_some() => {
                    pairs.push((view_area, FocusTarget::Editor));
                }
                CentralView::Merge => {
                    if let Some(view) = merge {
                        for (rect, pane) in view.nav_targets() {
                            pairs.push((rect, FocusTarget::MergePane(pane)));
                        }
                    }
                }
                CentralView::Editor | CentralView::File => {}
            }
            if !right_pane_collapsed {
                for (section, area) in status_section_areas.iter() {
                    pairs.push((*area, FocusTarget::StatusSection(*section)));
                }
                // The commit composer's Summary field, when the
                // pane has room to show it (`composer_height_for`
                // returns `0` and hides the whole box below a
                // minimum height) -- reusing the exact rect its
                // own render pass already registered
                // (`HitTarget::CommitSummaryField`), the click
                // target's own source of truth, rather than
                // recomputing the composer's layout a second time
                // here.
                if let Some((rect, _)) = frame_hits
                    .iter()
                    .find(|(_, t)| *t == HitTarget::CommitSummaryField)
                {
                    pairs.push((*rect, FocusTarget::ComposerSummary));
                }
                // The commit-detail pane's own sub-controls (a
                // COMMIT is selected), mirroring what the Status
                // sections get on the WIP row. Rects come from this
                // frame's hit list, so the badge sits on the
                // control that is really there. The badge is drawn
                // at `area.x + 1`, so each `area` starts one (or
                // three, or two) cells early to land on blank
                // cells: the Path button's leading spaces, the
                // gap before the checkbox -- never on a label.
                if right_panel == RightPanel::CommitDetail {
                    if commit_files_area.height > 0 {
                        pairs.push((commit_files_area, FocusTarget::CommitFiles));
                    }
                    let hit_rect = |want: HitTarget| {
                        frame_hits.iter().find(|(_, t)| *t == want).map(|(r, _)| *r)
                    };
                    if let Some(r) = hit_rect(HitTarget::FileListModeButton(FileListMode::Path)) {
                        pairs.push((
                            Rect::new(r.x.saturating_sub(3), r.y, r.width, 1),
                            FocusTarget::CommitPathTree,
                        ));
                    }
                    if let Some(r) = hit_rect(HitTarget::CommitSortButton) {
                        pairs.push((Rect::new(r.x, r.y, r.width, 1), FocusTarget::CommitSort));
                    }
                    if let Some(r) = hit_rect(HitTarget::ViewAllFilesCheckbox) {
                        pairs.push((
                            Rect::new(r.x.saturating_sub(2), r.y, r.width, 1),
                            FocusTarget::CommitViewAll,
                        ));
                    }
                    if let Some(r) = hit_rect(HitTarget::CommitFilterField) {
                        pairs.push((
                            Rect::new(r.x.saturating_sub(1), r.y, r.width, 1),
                            FocusTarget::CommitFilter,
                        ));
                    }
                }
            }
        }
        // The File Insight overlay, when open -- `file_insight_area`
        // is the previous frame's popup rect (this block draws
        // ahead of the overlay itself), which only matters on the
        // exact frame the window is resized while `Ctrl+N` is
        // held open, same tolerance every other "computed while
        // drawing" rect in this loop accepts.
        if file_insight.is_some() {
            pairs.push((file_insight_area, FocusTarget::FileView));
        }
        // The Repository Management tab's search field and each
        // of its four sections -- reusing the exact rects its own
        // render pass already registered for hit-testing
        // (`ManagerRender::hits`), rather than a second,
        // independently-computed guess at the same layout.
        //
        // Handed out in a fixed order: the search field, the four
        // sections, the toolbar buttons (and the Workspaces
        // tab's own buttons), so the search stays badge `1`.
        if let Some((_, render)) = management_panel.as_ref() {
            use repo_manager::{ManagerHit as H, WsHit};
            let pick = |want: &dyn Fn(&H) -> bool| -> Vec<(Rect, H)> {
                render
                    .hits
                    .iter()
                    .filter(|(_, hit)| want(hit))
                    .map(|(rect, hit)| (*rect, *hit))
                    .collect()
            };
            for (rect, _) in pick(&|h| *h == H::Search) {
                pairs.push((rect, FocusTarget::RepoManagerSearch));
            }
            for (rect, hit) in pick(&|h| matches!(h, H::Header(_))) {
                if let H::Header(section) = hit {
                    pairs.push((rect, FocusTarget::RepoManagerSection(section)));
                }
            }
            // A button's badge starts on the gap before it, so it hides
            // fewer letters of the label.
            for (rect, hit) in pick(&|h| {
                matches!(h, H::Button(_)) || matches!(h, H::Ws(WsHit::New | WsHit::FromTabs))
            }) {
                let rect = Rect::new(rect.x.saturating_sub(1), rect.y, rect.width + 1, 1);
                pairs.push((rect, FocusTarget::HomeControl(hit)));
            }
        }
        // The Radar covers the list and keeps its own hit rects.
        if let Some(manager) = management.as_ref() {
            for (rect, spot) in manager.radar.nav_targets() {
                pairs.push((rect, FocusTarget::Radar(spot)));
            }
        }
        // The tab bar's enabled scroll arrows, from this frame's hit list.
        // The badge is drawn one cell in, so the area starts on the gap
        // before the arrow and the badge covers the arrow exactly.
        for (rect, target) in frame_hits.iter() {
            let right = match target {
                HitTarget::TabScrollLeft => false,
                HitTarget::TabScrollRight => true,
                _ => continue,
            };
            let area = Rect::new(rect.x.saturating_sub(1), rect.y, rect.width + 1, 1);
            pairs.push((area, FocusTarget::TabScroll(right)));
        }
        let targets: Vec<FocusTarget> = pairs.iter().map(|(_, t)| *t).collect();
        let assigned = assign_navigation_badges(&targets);
        let badge_panes: Vec<(Rect, char)> = pairs
            .iter()
            .zip(assigned.iter())
            .map(|((area, _), (c, _))| (*area, *c))
            .collect();
        render_navigation_badges(frame, &badge_panes);
        // The flag under the title bar's octopus: `Ctrl+N a` opens About. Not a
        // focus badge (it is a fixed, rebindable key), so it is not in
        // `nav_badge_targets`; a key that is not one plain character has no flag.
        let about_key = keymap::current().shown_key(keymap::Action::NavOpenAbout);
        let mut about_chars = about_key.chars();
        if let (Some(c), None) = (about_chars.next(), about_chars.next()) {
            if let Some((rect, _)) = frame_hits.iter().find(|(_, t)| *t == HitTarget::About) {
                let below = Rect::new(rect.x.saturating_sub(1), rect.y + 1, 3, 1);
                render_navigation_badges(frame, &[(below, c)]);
            }
        }
        *nav_badge_targets = assigned;
    }

    // The File View is the central column (`file_panel::render_file_panel`
    // above draws it and registers its scrollbars). Nothing drew it this
    // frame when it is closed -- or when the Log is showing because the
    // window has no central column -- so its rect must not keep catching the
    // mouse.
    if central_view != CentralView::File || file_insight.is_none() {
        file_insight_area = Rect::new(0, 0, 0, 0);
    }

    if !settings_open {
        draw_command_palette(
            frame,
            input_mode,
            palette_over_dialog,
            palette_ctx,
            field_editor,
            journal_state,
            undo_busy,
            palette_popup,
        );
    }

    // The tab picker (`Ctrl+N t`, `Repository: Select Tab...`): the
    // multi-repository counterpart of the command palette above, reusing its
    // `PalettePopup`/`palette_row_at` geometry and hit test.
    if let InputMode::TabPicker { draft, selected } = &input_mode {
        let indices = filter_tab_indices(repo_tabs, draft);
        // Wide enough for the title and its close cross on a narrow screen.
        let area = frame.area();
        let mut default = centered_rect(46, 50, area);
        let min_w = 28.min(area.width);
        if default.width < min_w {
            default.x = area.x + (area.width - min_w) / 2;
            default.width = min_w;
        }
        let popup = TAB_PICKER_WIN.place(area, default);
        frame.render_clipped(Clear, popup);
        let [input_area, list_area, hint_area] = Layout::vertical([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(2),
        ])
        .areas(popup);
        let draft_w = input_area.width.saturating_sub(2);
        field_editor.drawn(
            MainField::TabPicker,
            Rect::new(input_area.x + 1, input_area.y + 1, draft_w, 1),
            false,
        );
        let draft_line = text_field::edit_line(
            draft,
            field_editor.caret(MainField::TabPicker),
            draft_w as usize,
            true,
            text_field::EditLook::new(Style::default().bg(crate::input_focus_bg()), "_"),
        );
        frame.render_clipped(
            Paragraph::new(draft_line).block(
                Block::default()
                    .title(format!(
                        " {} ",
                        gitpoulpi_i18n::msg::overlays::select_tab_title(
                            indices.len(),
                            repo_tabs.len()
                        )
                    ))
                    .border_type(BorderType::Rounded)
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(th().ui.accent)),
            ),
            input_area,
        );

        let rows = list_area.height as usize;
        let window_start = crate::popup_list::View::window_start(*selected, rows, indices.len());
        let inner_w = list_area.width.saturating_sub(2) as usize;
        let items: Vec<ListItem> = if indices.is_empty() {
            vec![ListItem::new(format!(
                "  {}",
                gitpoulpi_i18n::msg::overlays::NO_MATCHING_TAB.get()
            ))
            .style(Style::default().fg(th().ui.muted))]
        } else {
            indices
                .iter()
                .enumerate()
                .skip(window_start)
                .take(rows)
                .map(|(i, &tab_index)| {
                    let tab = &repo_tabs[tab_index];
                    let is_selected = i == *selected;
                    // A filled dot marks the tab currently on
                    // screen, the same "you are here" the tab bar
                    // itself shows, so the picker doesn't ask the
                    // owner to guess which row is a no-op.
                    let marker = if tab_index == active_tab {
                        "\u{25cf} "
                    } else {
                        "  "
                    };
                    let mut base = Style::default().fg(th().ui.text);
                    if is_selected {
                        base = base.bg(th().ui.selection).add_modifier(Modifier::BOLD);
                    }
                    let label = truncate_with_ellipsis(&tab.name, inner_w.saturating_sub(2));
                    ListItem::new(Line::from(Span::styled(format!("{marker}{label}"), base)))
                })
                .collect()
        };
        frame.render_clipped(
            List::new(items).block(
                Block::default()
                    .border_type(BorderType::Rounded)
                    .borders(Borders::LEFT | Borders::RIGHT)
                    .border_style(Style::default().fg(th().ui.accent)),
            ),
            list_area,
        );
        crate::popup_list::View::paint_bar(frame.buffer_mut(), list_area, indices.len());
        // Captured from the walk that just drew the rows, same
        // "computed while drawing, consulted on the next click"
        // rule `palette_popup` above already follows.
        *tab_picker_popup = Some(PalettePopup {
            popup,
            list: list_area,
            window_start,
            count: indices.len(),
            segments: [Rect::ZERO; 4],
        });
        if let Some(popup) = tab_picker_popup.as_ref() {
            crate::chrome::paint_popup_row_hover(frame.buffer_mut(), popup, *selected);
        }
        frame.render_clipped(
            Paragraph::new(gitpoulpi_i18n::msg::overlays::TAB_PICKER_HINT.get())
                .style(Style::default().fg(th().ui.muted))
                .block(
                    Block::default()
                        .border_type(BorderType::Rounded)
                        .borders(Borders::LEFT | Borders::RIGHT | Borders::BOTTOM)
                        .border_style(Style::default().fg(th().ui.accent)),
                ),
            hint_area,
        );
        let cross = crate::chrome::paint_popup_cross(frame.buffer_mut(), popup);
        TAB_PICKER_WIN.decorate(frame.buffer_mut(), popup, cross, true);
        crate::dialog_kit::frame::offer_list_popup(input_area, list_area);
    }

    if !settings_open {
        draw_picker_popups(frame, input_mode, field_editor, tab_picker_popup);
    }

    // The name box: where a name is typed when no inline spot (the Log's Ref
    // cell, the Refs row, the Status list) could draw it this frame.
    if let InputMode::InlineName { op, value } = &input_mode {
        if name_slot.may_draw(NameSpot::Box) {
            let hovered_part = match &hovered_target {
                Some(HitTarget::RefNameBox(part)) => Some(*part),
                _ => None,
            };
            let front: Vec<(Rect, HitTarget)> = crate::ref_name_box::paint_name_box(
                frame,
                frame.area(),
                op,
                value,
                ref_name_error.as_deref(),
                field_editor,
                hovered_part,
            )
            .into_iter()
            .map(|(rect, part)| (rect, HitTarget::RefNameBox(part)))
            .collect();
            if !front.is_empty() {
                name_slot.mark(NameSpot::Box);
                frame_hits.splice(0..0, front);
            }
        }
    }

    // A real modal for a pending destructive action: a bordered box with the
    // prompt and two clickable [Delete]/[Cancel] buttons, using the same
    // `Clear` + centered-box mechanism as the palette. Keyboard (y/Enter,
    // n/Esc) works too.
    if let InputMode::Confirm { prompt, action } = &input_mode {
        let confirm_action_label = action.label();
        let confirm_bg = if action.is_destructive() {
            th().chrome.confirm_danger
        } else {
            th().ui.accent
        };
        let confirm_fg = if action.is_destructive() {
            th().ui.on_button
        } else {
            th().ui.on_accent
        };
        // Wide enough for both buttons in any language (inside border
        // and one cell of margin each side).
        let buttons_w = crate::ui_kit::display_width(&confirm_action_label)
            + 2
            + crate::ui_kit::display_width(gitpoulpi_i18n::msg::dialog::CANCEL.get())
            + 2;
        let area_w = u32::from(frame.area().width.max(1));
        let needed_pct = ((buttons_w as u32 + 4) * 100).div_ceil(area_w).min(100) as u16;
        let popup = CONFIRM_WIN.place(
            frame.area(),
            centered_rect(needed_pct.max(44), 18, frame.area()),
        );
        frame.render_clipped(Clear, popup);
        frame.render_clipped(
            Block::default()
                .title(gitpoulpi_i18n::msg::overlays::CONFIRM_TITLE.get())
                .border_type(BorderType::Rounded)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(th().status.warning))
                .style(Style::default().bg(th().ui.panel)),
            popup,
        );
        let inner = Rect::new(
            popup.x + 2,
            popup.y + 1,
            popup.width.saturating_sub(4),
            popup.height.saturating_sub(2),
        );
        let [message_area, buttons_area] =
            Layout::vertical([Constraint::Min(1), Constraint::Length(1)]).areas(inner);
        frame.render_clipped(
            Paragraph::new(prompt.as_str()).wrap(ratatui::widgets::Wrap { trim: true }),
            message_area,
        );
        let confirm_label = confirm_action_label;
        let cancel_label = format!(" {} ", gitpoulpi_i18n::msg::dialog::CANCEL.get());
        let confirm_w = crate::ui_kit::display_width(&confirm_label) as u16;
        let cancel_w = crate::ui_kit::display_width(&cancel_label) as u16;
        let gap = 2u16;
        let total = confirm_w + gap + cancel_w;
        let start_x = buttons_area.x + buttons_area.width.saturating_sub(total) / 2;
        let confirm_rect = Rect::new(start_x, buttons_area.y, confirm_w, 1);
        let cancel_rect = Rect::new(start_x + confirm_w + gap, buttons_area.y, cancel_w, 1);
        // Both are solid filled bands, so hovering lightens the
        // fill (`lighten`, shared with the commit composer's
        // button) rather than lifting a background that's
        // already there.
        let confirm_fill = if hover(hovered_target).confirm() == Some(true) {
            crate::hover_fill(confirm_bg, confirm_fg, 24)
        } else {
            confirm_bg
        };
        let cancel_fill = if hover(hovered_target).confirm() == Some(false) {
            crate::hover_fill(th().ui.button, th().ui.on_button, 24)
        } else {
            th().ui.button
        };
        frame.render_clipped(
            Paragraph::new(confirm_label.as_str()).style(
                Style::default()
                    .bg(confirm_fill)
                    .fg(confirm_fg)
                    .add_modifier(Modifier::BOLD),
            ),
            confirm_rect,
        );
        frame.render_clipped(
            Paragraph::new(cancel_label.as_str())
                .style(Style::default().bg(cancel_fill).fg(th().ui.on_button)),
            cancel_rect,
        );
        frame_hits.insert(0, (confirm_rect, HitTarget::ConfirmButton(true)));
        frame_hits.insert(0, (cancel_rect, HitTarget::ConfirmButton(false)));
        let cross = crate::chrome::close_cross::paint(
            frame.buffer_mut(),
            popup,
            matches!(hovered_target, Some(HitTarget::ConfirmClose)),
        );
        if let Some(cross) = cross {
            frame_hits.insert(0, (cross, HitTarget::ConfirmClose));
        }
        CONFIRM_WIN.decorate(frame.buffer_mut(), popup, cross, true);
    }

    // A guiding dialog (conflicts first / input too large): it paints its
    // own buttons and cross and keeps their rects for its mouse handler.
    if let InputMode::Guide(dialog) = &input_mode {
        let area = frame.area();
        dialog.render(frame.buffer_mut(), area);
    }

    // The Edit commit message popup, registered from the walk
    // that drew it.
    if let InputMode::EditingCommitMessage(form) = &input_mode {
        let hovered_part = match &hovered_target {
            Some(HitTarget::RewordForm(part)) => Some(*part),
            _ => None,
        };
        let front: Vec<(Rect, HitTarget)> = form
            .render(frame, frame.area(), hovered_part)
            .into_iter()
            .map(|(rect, part)| (rect, HitTarget::RewordForm(part)))
            .collect();
        frame_hits.splice(0..0, front);
    }

    // A prompt dialog (stash message, tag message...), same pattern.
    if let InputMode::Prompt(dialog) = &input_mode {
        let hovered_part = match &hovered_target {
            Some(HitTarget::Prompt(part)) => Some(*part),
            _ => None,
        };
        let front: Vec<(Rect, HitTarget)> = dialog
            .render(frame, frame.area(), hovered_part)
            .into_iter()
            .map(|(rect, part)| (rect, HitTarget::Prompt(part)))
            .collect();
        frame_hits.splice(0..0, front);
    }

    // The interactive rebase / cherry-pick popup, same pattern.
    // It takes over the whole central column (the Log's area),
    // a full view, not a small popup; the message
    // editor it embeds is still a popup over the frame.
    if let InputMode::InteractiveRebase(form) = &input_mode {
        let hovered_part = match &hovered_target {
            Some(HitTarget::RebaseForm(part)) => Some(*part),
            _ => None,
        };
        let view_area = if log_area.width >= 40 && log_area.height >= 10 {
            log_area
        } else {
            frame.area()
        };
        let front: Vec<(Rect, HitTarget)> = form
            .render(frame, view_area, frame.area(), hovered_part, ui_graph_theme)
            .into_iter()
            .map(|(rect, part)| (rect, HitTarget::RebaseForm(part)))
            .collect();
        frame_hits.splice(0..0, front);
    }

    // The AI commit-composer review, same pattern: it takes over the
    // Log's column.
    if let InputMode::ComposeCommits(form) = &input_mode {
        // The right pane lists the selected proposal's files.
        let hovered_part = match &hovered_target {
            Some(HitTarget::ComposeForm(part)) => Some(*part),
            _ => None,
        };
        if status_area.width >= 12 && status_area.height >= 4 {
            let pane_hits = form.render_files_pane(frame, status_area, hovered_part);
            frame_hits.splice(
                0..0,
                pane_hits
                    .into_iter()
                    .map(|(rect, part)| (rect, HitTarget::ComposeForm(part))),
            );
        }
        let hovered_part = match &hovered_target {
            Some(HitTarget::ComposeForm(part)) => Some(*part),
            _ => None,
        };
        let view_area = if log_area.width >= 50 && log_area.height >= 12 {
            log_area
        } else {
            frame.area()
        };
        let front: Vec<(Rect, HitTarget)> = form
            .render(frame, view_area, hovered_part)
            .into_iter()
            .map(|(rect, part)| (rect, HitTarget::ComposeForm(part)))
            .collect();
        frame_hits.splice(0..0, front);
    }

    // The split-commit editor, same pattern: it takes over the Log's column.
    if let InputMode::SplitCommit(form) = &input_mode {
        let hovered_part = match &hovered_target {
            Some(HitTarget::SplitForm(part)) => Some(*part),
            _ => None,
        };
        let view_area = if log_area.width >= 50 && log_area.height >= 12 {
            log_area
        } else {
            frame.area()
        };
        let front: Vec<(Rect, HitTarget)> = form
            .render(frame, view_area, hovered_part)
            .into_iter()
            .map(|(rect, part)| (rect, HitTarget::SplitForm(part)))
            .collect();
        frame_hits.splice(0..0, front);
    }

    // The history rewrite workbench takes the Log's column the same way.
    if let InputMode::Rewrite(view) = &input_mode {
        let hovered_part = match &hovered_target {
            Some(HitTarget::Rewrite(part)) => Some(*part),
            _ => None,
        };
        let view_area = if log_area.width >= 60 && log_area.height >= 16 {
            log_area
        } else {
            frame.area()
        };
        let front: Vec<(Rect, HitTarget)> = view
            .render(frame, view_area, hovered_part)
            .into_iter()
            .map(|(rect, part)| (rect, HitTarget::Rewrite(part)))
            .collect();
        frame_hits.splice(0..0, front);
    }

    // The rebase-in-progress banner: a one-row strip painted over
    // the Log pane's own top row (an overlay, not a resize of
    // `log_area` -- the row-index arithmetic every Log offset
    // depends on is exactly what `where-we-are.md` warns is easy
    // to get subtly wrong, so this deliberately never touches it).
    // Spliced to the *front* of `frame_hits` so its buttons win
    // over whatever row happens to be underneath, the same
    // precedence a popup gets.
    if central_view == CentralView::Log && log_area.height > 0 {
        if let Some(state) = &rebase_state {
            let strip = Rect::new(log_area.x, log_area.y, log_area.width, 1);
            let hovered_button = match &hovered_target {
                Some(HitTarget::RebaseBanner(button)) => Some(*button),
                _ => None,
            };
            let banner_hits = crate::rebase_banner::render(frame, strip, state, hovered_button);
            let front: Vec<(Rect, HitTarget)> = banner_hits
                .into_iter()
                .map(|(rect, button)| (rect, HitTarget::RebaseBanner(button)))
                .collect();
            frame_hits.splice(0..0, front);
        }
    }
    // The merge / cherry-pick / revert banner takes the same row (only one
    // operation runs at a time; a rebase has its own banner above).
    let operation_banner = operation_state
        .as_ref()
        .filter(|s| crate::operation_banner::applies(s));
    if central_view == CentralView::Log && log_area.height > 0 {
        if let Some(state) = operation_banner {
            let strip = Rect::new(log_area.x, log_area.y, log_area.width, 1);
            let hovered_button = match &hovered_target {
                Some(HitTarget::OperationBanner(button)) => Some(*button),
                _ => None,
            };
            let front: Vec<(Rect, HitTarget)> = crate::operation_banner::render(
                frame,
                strip,
                state,
                conflict_count,
                hovered_button,
            )
            .into_iter()
            .map(|(rect, button)| (rect, HitTarget::OperationBanner(button)))
            .collect();
            frame_hits.splice(0..0, front);
        }
    }
    // A `git am` stopped on a patch takes the same row (it is none of the
    // above: `operation_banner::applies` leaves it out).
    let am_banner = operation_state
        .as_ref()
        .filter(|s| matches!(s, gitpoulpi_core::domain::OperationState::Am { .. }));
    if central_view == CentralView::Log && log_area.height > 0 {
        if let Some(state) = am_banner {
            let strip = Rect::new(log_area.x, log_area.y, log_area.width, 1);
            let hovered_button = match &hovered_target {
                Some(HitTarget::StuckBanner(button)) => Some(*button),
                _ => None,
            };
            let front: Vec<(Rect, HitTarget)> =
                crate::stuck_banner::render_am(frame, strip, state, conflict_count, hovered_button)
                    .into_iter()
                    .map(|(rect, button)| (rect, HitTarget::StuckBanner(button)))
                    .collect();
            frame_hits.splice(0..0, front);
        }
    }
    let top_taken = rebase_state.is_some() || operation_banner.is_some() || am_banner.is_some();
    // The pending auto-stash strip: the same kind of overlay, on the Log's
    // bottom border row when another banner has the top one.
    if central_view == CentralView::Log {
        // The top border row, or the bottom one when another banner has
        // the top: never the column-title row under it.
        let slot = usize::from(top_taken);
        if let (Some(pending), Some(strip)) = (
            &pending_auto_stash,
            crate::operation_banner::strip_rect(log_area, slot),
        ) {
            let hovered_button = match &hovered_target {
                Some(HitTarget::AutoStashBanner(button)) => Some(*button),
                _ => None,
            };
            let front: Vec<(Rect, HitTarget)> =
                crate::autostash_banner::render(frame, strip, pending, hovered_button)
                    .into_iter()
                    .map(|(rect, button)| (rect, HitTarget::AutoStashBanner(button)))
                    .collect();
            frame_hits.splice(0..0, front);
        }
    }
    // The stale `index.lock` strip takes the next free row (a second banner
    // is the last one the Log's borders have room for).
    if central_view == CentralView::Log {
        let slot = usize::from(top_taken) + usize::from(pending_auto_stash.is_some());
        if let (Some(lock), Some(strip)) = (
            index_lock.as_ref().filter(|l| l.stale),
            crate::operation_banner::strip_rect(log_area, slot),
        ) {
            let hovered_button = match &hovered_target {
                Some(HitTarget::StuckBanner(button)) => Some(*button),
                _ => None,
            };
            let front: Vec<(Rect, HitTarget)> =
                crate::stuck_banner::render_lock(frame, strip, lock, hovered_button)
                    .into_iter()
                    .map(|(rect, button)| (rect, HitTarget::StuckBanner(button)))
                    .collect();
            frame_hits.splice(0..0, front);
        }
    }
    // The proactive conflict badge takes the next free row, after every
    // banner above: it is a warning, and the others are in-progress operations.
    if central_view == CentralView::Log {
        let slot = usize::from(top_taken)
            + usize::from(pending_auto_stash.is_some())
            + usize::from(index_lock.as_ref().is_some_and(|l| l.stale));
        if let (Some(prediction), Some(strip)) = (
            conflict_prediction,
            crate::operation_banner::strip_rect(log_area, slot),
        ) {
            let hovered = matches!(&hovered_target, Some(HitTarget::ConflictBadge));
            let front: Vec<(Rect, HitTarget)> =
                crate::conflict_badge::render(frame, strip, prediction, hovered)
                    .into_iter()
                    .map(|rect| (rect, HitTarget::ConflictBadge))
                    .collect();
            frame_hits.splice(0..0, front);
        }
    }
    *ctx.file_insight_area = file_insight_area;
}

/// The command palette popup (not while a repository dialog hosts it).
#[allow(clippy::too_many_arguments)]
pub(in crate::event_loop) fn draw_command_palette(
    frame: &mut Frame,
    input_mode: &InputMode,
    palette_over_dialog: bool,
    palette_ctx: PaletteContext,
    field_editor: &crate::text_edit::FieldEditor<MainField>,
    journal_state: &gitpoulpi_core::domain::JournalState,
    undo_busy: bool,
    palette_popup: &mut Option<PalettePopup>,
) {
    if let (InputMode::CommandPalette { draft, selected }, false) =
        (input_mode, palette_over_dialog)
    {
        super::palette_overlay::render_command_palette(
            frame,
            draft,
            *selected,
            palette_ctx,
            field_editor,
            journal_state,
            undo_busy,
            palette_popup,
        );
    }
}

/// The theme, language / graph color and workspace-search pickers. Painted
/// over the panes, or over the Settings screen when it is open, so a theme
/// preview shows on Settings itself.
pub(in crate::event_loop) fn draw_picker_popups(
    frame: &mut Frame,
    input_mode: &InputMode,
    field_editor: &crate::text_edit::FieldEditor<MainField>,
    tab_picker_popup: &mut Option<PalettePopup>,
) {
    // The theme picker (`Ctrl+N y`, `App: Change Theme...`).
    if let InputMode::ThemePicker(picker) = input_mode {
        *tab_picker_popup = Some(crate::theme_picker::render(frame, picker, field_editor));
    }

    if let InputMode::LanguagePicker(picker) = input_mode {
        *tab_picker_popup = Some(crate::language_picker::render(frame, picker, field_editor));
    }

    // The workspace search (`Ctrl+Alt+F`).
    if let InputMode::WorkspaceSearch(search) = input_mode {
        *tab_picker_popup = Some(crate::workspace_search::render(frame, search, field_editor));
    }
}
