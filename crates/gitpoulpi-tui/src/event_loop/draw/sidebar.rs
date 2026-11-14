//! The sidebar block of the draw closure.

use super::super::*;

pub(in crate::event_loop) struct SidebarDrawCtx<'a> {
    pub(in crate::event_loop) focus: Focus,
    pub(in crate::event_loop) left_pane_collapsed: bool,
    pub(in crate::event_loop) hierarchical_refs: bool,
    pub(in crate::event_loop) active_ref_section: SidebarSection,
    pub(in crate::event_loop) refs_area: Rect,
    pub(in crate::event_loop) hovered_target: &'a Option<HitTarget>,
    pub(in crate::event_loop) hidden_from_graph: &'a std::collections::HashSet<String>,
    pub(in crate::event_loop) collapsed_ref_sections: &'a std::collections::HashSet<String>,
    pub(in crate::event_loop) collapsed_ref_folders:
        &'a std::collections::HashSet<(RefRowKind, String)>,
    pub(in crate::event_loop) hovered_ref_row: &'a Option<String>,
    pub(in crate::event_loop) hovered_ref_folder: &'a Option<String>,
    pub(in crate::event_loop) input_mode: &'a InputMode,
    pub(in crate::event_loop) ref_drag: &'a Option<RefDrag>,
    pub(in crate::event_loop) ref_drop_mark: &'a Option<(String, RefRowKind)>,
    pub(in crate::event_loop) refs_filter: &'a String,
    pub(in crate::event_loop) refs_result:
        &'a gitpoulpi_core::Result<Vec<gitpoulpi_core::domain::Ref>>,
    pub(in crate::event_loop) remote_names: &'a Vec<String>,
    /// Remote-tracking namespaces no remote is configured for: their root
    /// row in the REMOTE section carries a muted note.
    pub(in crate::event_loop) leftover_remotes: &'a std::collections::BTreeSet<String>,
    pub(in crate::event_loop) sidebar_states: &'a mut SidebarStates,
    pub(in crate::event_loop) solo_refs: &'a std::collections::HashSet<String>,
    pub(in crate::event_loop) submodules_result: &'a Vec<gitpoulpi_core::domain::Submodule>,
    pub(in crate::event_loop) unpushed_tags: &'a std::collections::HashSet<String>,
    pub(in crate::event_loop) worktrees_result:
        &'a gitpoulpi_core::Result<Vec<gitpoulpi_core::domain::Worktree>>,
    pub(in crate::event_loop) stashes_result:
        &'a gitpoulpi_core::Result<Vec<gitpoulpi_core::domain::Stash>>,
    pub(in crate::event_loop) pane_notice: &'a Option<(Vec<String>, Style)>,
    pub(in crate::event_loop) sidebar_heights: &'a Vec<u16>,
    pub(in crate::event_loop) hidden_sidebar_sections: &'a HashSet<SidebarSection>,
    pub(in crate::event_loop) hosting: &'a HostingUi,
    pub(in crate::event_loop) frame_hits: &'a mut Vec<(Rect, HitTarget)>,
    pub(in crate::event_loop) field_editor: &'a mut text_edit::FieldEditor<MainField>,
    pub(in crate::event_loop) refs_filter_area: &'a mut Rect,
    pub(in crate::event_loop) sidebar_layout_avail: &'a mut u16,
    pub(in crate::event_loop) sidebar_layout_specs: &'a mut Vec<sidebar_layout::SectionSpec>,
    pub(in crate::event_loop) sidebar_section_areas: &'a mut Vec<(SidebarSection, Rect)>,
    /// The refusal of a branch rename, and who holds the name field.
    pub(in crate::event_loop) ref_name_error: &'a Option<String>,
    pub(in crate::event_loop) name_slot: &'a mut NameSlot,
}

// Ref rows built by the paint loop (test counter: must track the
// visible window, not the branch count).
#[cfg(test)]
thread_local! {
    pub(super) static REF_ROWS_BUILT: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// The rows a Refs `List` paints this frame. Unlike the status pane's
/// wider `status_paint_window`, a followed selection outside the view
/// is resolved the way ratatui will (offset moved just far enough to
/// show it; rows are one line high), so a first frame with a deep
/// selection builds one screenful, not everything up to the selection.
fn ref_paint_window(
    len: usize,
    state: &ListState,
    follow: bool,
    visible: usize,
) -> std::ops::Range<usize> {
    let visible = visible.max(1);
    let mut lo = state.offset();
    if let Some(sel) = state.selected().filter(|_| follow && len > 0) {
        let sel = sel.min(len - 1);
        if sel >= lo + visible {
            lo = sel + 1 - visible;
        } else if sel < lo {
            lo = sel;
        }
    }
    let lo = lo.min(len.saturating_sub(1));
    lo..(lo + visible).min(len)
}

pub(in crate::event_loop) fn draw_sidebar(frame: &mut Frame, ctx: &mut SidebarDrawCtx<'_>) {
    let focus = ctx.focus;
    let left_pane_collapsed = ctx.left_pane_collapsed;
    let hierarchical_refs = ctx.hierarchical_refs;
    let active_ref_section = ctx.active_ref_section;
    let refs_area = ctx.refs_area;
    let hovered_target = ctx.hovered_target;
    let hidden_from_graph = ctx.hidden_from_graph;
    let collapsed_ref_sections = ctx.collapsed_ref_sections;
    let collapsed_ref_folders = ctx.collapsed_ref_folders;
    let hovered_ref_row = ctx.hovered_ref_row;
    let hovered_ref_folder = ctx.hovered_ref_folder;
    let input_mode = ctx.input_mode;
    let ref_drag = ctx.ref_drag;
    let ref_drop_mark = ctx.ref_drop_mark;
    let refs_filter = ctx.refs_filter;
    let refs_result = ctx.refs_result;
    let remote_names = ctx.remote_names;
    let leftover_remotes = ctx.leftover_remotes;
    let sidebar_states = &mut *ctx.sidebar_states;
    let solo_refs = ctx.solo_refs;
    let submodules_result = ctx.submodules_result;
    let unpushed_tags = ctx.unpushed_tags;
    let worktrees_result = ctx.worktrees_result;
    let stashes_result = ctx.stashes_result;
    let pane_notice = ctx.pane_notice;
    let sidebar_heights = ctx.sidebar_heights;
    let hosting = ctx.hosting;
    let frame_hits = &mut *ctx.frame_hits;
    let field_editor = &mut *ctx.field_editor;
    let refs_filter_area = &mut *ctx.refs_filter_area;
    let sidebar_layout_avail = &mut *ctx.sidebar_layout_avail;
    let sidebar_layout_specs = &mut *ctx.sidebar_layout_specs;
    let sidebar_section_areas = &mut *ctx.sidebar_section_areas;
    let ref_name_error = ctx.ref_name_error;
    let name_slot = &mut *ctx.name_slot;
    let empty_refs_for_head = Vec::new();
    let checked_out = checked_out_branch_name(refs_result.as_ref().unwrap_or(&empty_refs_for_head));
    // The local branches with extra push targets, by name: the remotes badge.
    let multi_by_name: std::collections::HashMap<
        &str,
        &[gitpoulpi_core::domain::RemoteTargetState],
    > = refs_result
        .as_ref()
        .ok()
        .map(|refs| {
            refs.iter()
                .filter_map(|r| match (&r.kind, &r.multi_remote) {
                    (gitpoulpi_core::domain::RefKind::LocalBranch { name }, Some(t)) => {
                        Some((name.as_str(), t.as_slice()))
                    }
                    _ => None,
                })
                .collect()
        })
        .unwrap_or_default();
    let empty_groups = RefsGroups::default();
    let ref_groups = refs_result
        .as_ref()
        .ok()
        .map(|refs| group_refs(refs, refs_filter))
        .unwrap_or(empty_groups);

    let refs_border_style = if focus == Focus::Refs {
        Style::default().fg(th().ui.accent)
    } else {
        Style::default()
    };
    // While refs are soloed and/or hidden, the sidebar title swaps from plain
    // "Refs" to a count of whichever is active plus a right-aligned button
    // that clears the louder one in a click; see `refs_title` for why Stop
    // Soloing wins the slot (Show All stays reachable from the palette and
    // `Shift+H`).
    let soloing = !solo_refs.is_empty();
    // Hidden refs: the set's own entries, plus every remote branch a hidden
    // remote covers (a hidden remote key is not itself a ref).
    let hidden_count = hidden_from_graph
        .iter()
        .filter(|k| !k.ends_with('/'))
        .count()
        + ref_groups
            .names(RefRowKind::Remote)
            .iter()
            .filter(|n| {
                !hidden_from_graph.contains(*n)
                    && crate::refs_tree::hidden_through_remote(hidden_from_graph, n)
            })
            .count();
    let total_refs: usize = [RefRowKind::Local, RefRowKind::Remote, RefRowKind::Tag]
        .iter()
        .map(|&k| ref_groups.names(k).len())
        .sum();
    let (refs_title_text, refs_title_button) =
        refs_title(solo_refs.len(), total_refs, hidden_count);
    let refs_outer = if let Some(button_label) = refs_title_button {
        // The Refs title's Show All and Stop Soloing share this slot. At rest the
        // text color is left as it was; on hover it takes the strong
        // `plain_button_style` fill of the gear and the close cross.
        let button_style = if hover(hovered_target).refs_title_button() {
            Style::default().patch(plain_button_style(true))
        } else {
            Style::default()
        };
        Block::default()
            .title(Line::from(refs_title_text.clone()).left_aligned())
            .title(Line::from(Span::styled(button_label, button_style)).right_aligned())
            .border_type(BorderType::Rounded)
            .borders(Borders::ALL)
            .border_style(refs_border_style)
    } else {
        Block::default()
            .title(refs_title_text.clone())
            .border_type(BorderType::Rounded)
            .borders(Borders::ALL)
            .border_style(refs_border_style)
    };
    let refs_inner = refs_outer.inner(refs_area);
    frame.render_clipped(refs_outer, refs_area);
    if let Some(button_label) = refs_title_button {
        let label_width = display_width(button_label) as u16;
        let hit_x = (refs_area.x + refs_area.width)
            .saturating_sub(1)
            .saturating_sub(label_width);
        frame_hits.push((
            Rect::new(hit_x, refs_area.y, label_width, 1),
            if soloing {
                HitTarget::StopSoloing
            } else {
                HitTarget::ShowAllHiddenRefs
            },
        ));
    }

    // A real, always-visible filter textbox at the top of the sidebar; opened
    // for editing with `/` or a click.
    let [computed_refs_filter_area, refs_sections_area] =
        Layout::vertical([Constraint::Length(3), Constraint::Min(0)]).areas(refs_inner);
    *refs_filter_area = computed_refs_filter_area;
    let filter_editing = matches!(&input_mode, InputMode::FilteringRefs(_));
    // The box's two borders and the " / " prefix are chrome; the draft
    // scrolls inside what is left (`text_field::field_window`). Drawn
    // verbatim it outgrows the border and ratatui clips the caret with it.
    let refs_field_w = refs_filter_area.width.saturating_sub(2 + 3) as usize;
    // The text itself: inside the border, after " / ".
    field_editor.drawn(
        MainField::RefsFilter,
        Rect::new(
            refs_filter_area.x + 4,
            refs_filter_area.y + 1,
            refs_field_w as u16,
            1,
        ),
        false,
    );
    let prefix_fg = if filter_editing || !refs_filter.is_empty() {
        th().ui.text
    } else {
        th().ui.muted
    };
    let filter_bg = crate::input_bg(filter_editing, false);
    let mut filter_line = vec![Span::styled(
        " / ",
        Style::default().fg(prefix_fg).bg(filter_bg),
    )];
    if let InputMode::FilteringRefs(draft) = &input_mode {
        filter_line.extend(
            text_field::edit_line(
                draft,
                field_editor.caret(MainField::RefsFilter),
                refs_field_w,
                true,
                text_field::EditLook::new(Style::default().fg(th().ui.text).bg(filter_bg), "_"),
            )
            .spans,
        );
    } else if refs_filter.is_empty() {
        let hint = gitpoulpi_i18n::msg::sidebar::FILTER_HINT.get();
        filter_line.push(Span::styled(
            hint,
            Style::default().fg(th().ui.muted).bg(filter_bg),
        ));
        filter_line.push(Span::styled(
            " ".repeat(refs_field_w.saturating_sub(display_width(hint))),
            Style::default().bg(filter_bg),
        ));
    } else {
        filter_line.extend(text_field::resting_spans(
            refs_filter,
            refs_field_w,
            Style::default().fg(th().ui.text).bg(filter_bg),
        ));
    }
    frame.render_clipped(
        Paragraph::new(Line::from(filter_line)).block(
            Block::default()
                .title(gitpoulpi_i18n::msg::sidebar::FILTER_TITLE.get())
                .border_type(BorderType::Rounded)
                .borders(Borders::ALL)
                .border_style(if filter_editing {
                    Style::default().fg(th().ui.accent)
                } else {
                    Style::default().fg(th().ui.muted)
                }),
        ),
        *refs_filter_area,
    );
    let refs_inner = refs_sections_area;

    // Three separate collapsible sub-panes, each a bordered `Block` (title =
    // chevron + name + count) with its own `ListState`. A collapsed section
    // shrinks to its title bar (height 2); an expanded one asks for header +
    // every visible row, and `Layout::vertical` shrinks all three
    // proportionally if that exceeds `refs_area`. Each List still
    // auto-scrolls to keep its selection visible, so nothing becomes
    // unreachable.
    let worktrees: &[gitpoulpi_core::domain::Worktree] = worktrees_result.as_deref().unwrap_or(&[]);
    let stashes: &[gitpoulpi_core::domain::Stash] = stashes_result.as_deref().unwrap_or(&[]);
    let sections = visible_sections_with(
        linked_worktree_count(worktrees),
        stashes.len(),
        hosting.sections_visible(),
        submodules_result.len(),
        ctx.hidden_sidebar_sections,
    );

    // Each kind's display rows, built once per frame: the section heights
    // below and the paint loop both read them.
    let ref_rows: Vec<(RefRowKind, Vec<RefTreeRow>)> =
        [RefRowKind::Local, RefRowKind::Remote, RefRowKind::Tag]
            .into_iter()
            .map(|kind| {
                (
                    kind,
                    ref_display_rows(
                        ref_groups.names(kind),
                        kind,
                        hierarchical_refs,
                        collapsed_ref_folders,
                        remote_names,
                    ),
                )
            })
            .collect();
    let rows_of = |kind: RefRowKind| -> &[RefTreeRow] {
        ref_rows
            .iter()
            .find(|(k, _)| *k == kind)
            .map(|(_, r)| r.as_slice())
            .unwrap_or_default()
    };
    let section_row_count = |section: SidebarSection| -> u16 {
        match section.ref_kind() {
            Some(kind) => rows_of(kind).len() as u16,
            None => match section {
                SidebarSection::Worktrees if linked_worktree_count(worktrees) == 0 => 1,
                SidebarSection::Worktrees => worktrees.len() as u16,
                SidebarSection::Stashes => stashes.len() as u16,
                SidebarSection::Submodules => submodules_result.len() as u16,
                SidebarSection::PullRequests => hosting.row_counts().0 as u16,
                SidebarSection::Issues => hosting.row_counts().1 as u16,
                _ => 0,
            },
        }
    };
    // Each section's height: its own content by default, or what the
    // user dragged it to (`sidebar_layout`); a section that gets less
    // than its content scrolls (its own scrollbar, drawn below).
    *sidebar_layout_specs = sections
        .iter()
        .map(|s| sidebar_layout::SectionSpec {
            expanded: !collapsed_ref_sections.contains(s.title()),
            rows: section_row_count(*s),
            user: sidebar_layout::stored_height(sidebar_heights, s.index()),
        })
        .collect();
    *sidebar_layout_avail = refs_inner.height;
    let heights = sidebar_layout::section_heights(refs_inner.height, sidebar_layout_specs);
    let mut section_y = refs_inner.y;
    *sidebar_section_areas = sections
        .iter()
        .copied()
        .zip(heights.iter().copied())
        .map(|(s, h)| {
            let y = section_y;
            section_y = section_y.saturating_add(h);
            let h = h.min(
                refs_inner
                    .y
                    .saturating_add(refs_inner.height)
                    .saturating_sub(y),
            );
            (s, Rect::new(refs_inner.x, y, refs_inner.width, h))
        })
        .collect();

    for (section, area) in sidebar_section_areas.clone() {
        // The resize handle: the section's bottom border row, minus
        // its corners, for an expanded section that has a bottom
        // border of its own. Registered ahead of everything else
        // the section registers (inner first); it shares no cell
        // with the header band or the rows.
        if !collapsed_ref_sections.contains(section.title())
            && area.height >= sidebar_layout::MIN_SECTION_HEIGHT
            && area.width > 2
        {
            frame_hits.push((
                Rect::new(area.x + 1, area.y + area.height - 1, area.width - 2, 1),
                HitTarget::SidebarSectionBoundary(section),
            ));
        }
        // The hosting sections paint themselves (`hosting_ui`).
        if let Some(list) = HostingList::from_section(section) {
            hosting.render_section(
                frame,
                area,
                list,
                !collapsed_ref_sections.contains(section.title()),
                focus == Focus::Refs && active_ref_section == section,
                hovered_target.as_ref(),
                &mut *sidebar_states,
                &mut *frame_hits,
            );
            continue;
        }
        let kind_opt = section.ref_kind();
        let expanded = !collapsed_ref_sections.contains(section.title());
        let empty_names: Vec<String> = Vec::new();
        let names: &[String] = match kind_opt {
            Some(kind) => ref_groups.names(kind),
            None => &empty_names,
        };
        let rows: &[RefTreeRow] = match kind_opt {
            Some(kind) => rows_of(kind),
            None => &[],
        };
        // The List's inner height: the bordered rect minus top+bottom.
        let sidebar_visible = area.height.saturating_sub(2) as usize;
        let chevron = crate::fold_mark::glyph_open(expanded);
        let is_active = focus == Focus::Refs && active_ref_section == section;
        let header_style = if is_active {
            Style::default()
                .fg(th().ui.accent)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
                .fg(th().ui.muted)
                .add_modifier(Modifier::BOLD)
        };
        // `(row index, column offset from the section's left edge, width,
        // branch)` of every remotes badge drawn this frame.
        let mut badge_hits: Vec<(usize, u16, u16, String)> = Vec::new();
        let items: Vec<ListItem> = if !expanded {
            Vec::new()
        } else if section == SidebarSection::Worktrees && linked_worktree_count(worktrees) == 0 {
            // Shown on request in a repository with no linked worktree: the
            // hint, and the header's `+` creates the first one.
            vec![ListItem::new(format!(
                "  {}",
                gitpoulpi_i18n::msg::worktree_manage::EMPTY_HINT.get()
            ))
            .style(Style::default().fg(th().ui.muted))]
        } else if section == SidebarSection::Worktrees {
            // Read-only listing. The main worktree is marked
            // rather than hidden -- it's the one you're looking
            // at, and omitting it would make a single-worktree
            // repo look like it had none.
            worktrees
                .iter()
                .map(|wt| {
                    let branch = wt
                        .branch
                        .as_deref()
                        .unwrap_or(gitpoulpi_i18n::msg::sidebar::DETACHED.get());
                    let label = if wt.is_main {
                        format!(
                            "{branch}  {}",
                            gitpoulpi_i18n::msg::sidebar::WORKTREE_MAIN.get()
                        )
                    } else {
                        branch.to_string()
                    };
                    let mut style = Style::default().fg(th().refs.local);
                    if wt.is_main {
                        style = style.add_modifier(Modifier::BOLD);
                    }
                    let lock = if wt.is_locked { "\u{1F512} " } else { "" };
                    ListItem::new(Line::from(vec![
                        Span::raw("  "),
                        Span::raw(lock),
                        Span::styled(
                            truncate_with_ellipsis(&label, area.width.saturating_sub(4) as usize),
                            style,
                        ),
                    ]))
                })
                .collect()
        } else if section == SidebarSection::Submodules {
            submodule_items(submodules_result, area.width)
        } else if section == SidebarSection::Stashes {
            stashes
                .iter()
                .map(|st| {
                    // `stash@{n}` is the handle the user sees in
                    // git's own output, so lead with it, then the
                    // message, then the branch it was taken on.
                    let label = format!("stash@{{{}}}  {}", st.id.0, st.message);
                    ListItem::new(Line::from(vec![
                        Span::raw("  "),
                        Span::styled(
                            truncate_with_ellipsis(&label, area.width.saturating_sub(4) as usize),
                            Style::default().fg(th().status.warning),
                        ),
                    ]))
                })
                .collect()
        } else if rows.is_empty() {
            vec![
                ListItem::new(format!("  {}", gitpoulpi_i18n::msg::sidebar::NONE.get()))
                    .style(Style::default().fg(th().ui.muted)),
            ]
        } else if let Some(kind) = kind_opt {
            // Which row the List will highlight, so the name on
            // it can carry the selection's BOLD itself -- the
            // highlight style below deliberately does not, or it
            // would bold the eye too (see `HIDE_IN_GRAPH_ICON`).
            let selected_row = sidebar_states.state_mut(section).selected();
            // Only the rows the List can paint get real items; every
            // row is one line high, so the rest are blank one-liners
            // and scrolling is unchanged (see `status_paint_window`).
            let window = ref_paint_window(
                rows.len(),
                sidebar_states.state(section),
                sidebar_states.follow(section),
                sidebar_visible,
            );
            rows.iter()
                .enumerate()
                .map(|(row_index, row)| {
                    if !window.contains(&row_index) {
                        return ListItem::new("");
                    }
                    #[cfg(test)]
                    REF_ROWS_BUILT.with(|n| n.set(n.get() + 1));
                    // Depth 0 gets no baseline indent: there is no parent folder to align
                    // under.
                    let indent = "  ".repeat(row.depth() as usize);
                    match row {
                        RefTreeRow::Folder {
                            label,
                            path,
                            is_remote_root,
                            ..
                        } => {
                            let folder_expanded =
                                !collapsed_ref_folders.contains(&(kind, path.clone()));
                            let fchevron = crate::fold_mark::glyph_open(folder_expanded);
                            // Every ref this folder/remote-root row
                            // holds -- the same set "hide/solo this
                            // whole folder" (the eye gutter's click,
                            // its context menu entry, and the `o`
                            // key) bulk-toggles. Ignoring collapse
                            // deliberately: whether the children are
                            // currently drawn has nothing to do with
                            // whether they're hidden.
                            let under = refs_under_folder(names, path);
                            // A whole remote hidden (its root row, and every
                            // folder under it) reads hidden whatever its
                            // branches say.
                            let all_hidden = folder_fully_in(hidden_from_graph, &under)
                                || (kind == RefRowKind::Remote
                                    && crate::refs_tree::hidden_through_remote(
                                        hidden_from_graph,
                                        path,
                                    ));
                            let is_row_hovered =
                                hovered_ref_folder.as_deref() == Some(path.as_str());
                            // Same gutter-icon shape as a leaf row's
                            // (see below): blank unless hovered or
                            // already hidden, a solo dot pair while
                            // Solo mode owns the gutter.
                            let eye = if soloing {
                                if folder_fully_in(solo_refs, &under) {
                                    SOLO_FULL_MARK // ◎ every descendant soloed
                                } else {
                                    "\u{25CB}" // ○ not (fully) soloed
                                }
                            } else if all_hidden {
                                HIDDEN_IN_GRAPH_MARK
                            } else if is_row_hovered {
                                HIDE_IN_GRAPH_ICON
                            } else {
                                " "
                            };
                            // No checkmark column for a folder (a
                            // folder is never checked out) -- a
                            // blank cell in its place so the eye
                            // still lands in the same column a leaf
                            // row's does, which is also what
                            // `gutter_end`'s width assumes.
                            let label_text = if crate::flow::view::is_group_path(path) {
                                group_folder_label("", fchevron, label, folder_expanded)
                            } else {
                                ref_folder_label(
                                    "",
                                    fchevron,
                                    label,
                                    *is_remote_root,
                                    folder_expanded,
                                )
                            };
                            let mut style = Style::default()
                                .fg(th().ui.muted)
                                .add_modifier(Modifier::BOLD);
                            if all_hidden {
                                style = style.add_modifier(Modifier::DIM);
                            }
                            let mut spans = vec![
                                Span::raw(indent),
                                Span::raw(" "),
                                Span::raw(" "),
                                Span::raw(eye),
                                Span::raw(" "),
                                Span::styled(label_text, style),
                            ];
                            if *is_remote_root
                                && kind == RefRowKind::Remote
                                && leftover_remotes.contains(path)
                            {
                                spans.push(Span::styled(
                                    format!(
                                        " {}",
                                        gitpoulpi_i18n::msg::remotes::LEFTOVER_NOTE.get()
                                    ),
                                    Style::default().fg(th().ui.muted),
                                ));
                            }
                            ListItem::new(Line::from(spans))
                        }
                        RefTreeRow::Leaf {
                            full_name,
                            display_name,
                            ..
                        } => {
                            let is_checked_out = kind == RefRowKind::Local
                                && checked_out == Some(full_name.as_str());
                            let is_hidden = crate::refs_tree::hidden_in_graph(
                                hidden_from_graph,
                                full_name,
                                kind == RefRowKind::Remote,
                            );
                            // No per-kind icon (the row already sits inside its LOCAL/REMOTE/TAGS
                            // sub-pane). The checked-out mark gets a small fixed gutter on the left,
                            // so names stay aligned whichever rows are checked out.
                            let checkmark = if is_checked_out {
                                CHECKED_OUT_MARK
                            } else {
                                " "
                            };
                            // Hide/show-in-graph eye icon: blank
                            // for a visible ref unless the row is
                            // currently hovered (the "click to
                            // hide" affordance); once hidden, the
                            // eye stays regardless of hover, so
                            // the user can see at a glance which
                            // branches are hidden.
                            let is_row_hovered =
                                hovered_ref_row.as_deref() == Some(full_name.as_str());
                            // While soloing, this gutter slot swaps from the hide-in-graph eye to a
                            // solo glyph: filled dot for a soloed ref, hollow for one that isn't
                            // (click toggles). Hide and Solo stay independent; this is only about
                            // which owns the gutter icon.
                            let eye = if soloing {
                                if solo_refs.contains(full_name) {
                                    SOLO_FULL_MARK // ◎ soloed
                                } else {
                                    "\u{25CB}" // ○ not soloed
                                }
                            } else if is_hidden {
                                // The *state* mark: a hidden ref keeps the circled slash, with the name
                                // dimmed. The eye below is the *action* (the glyph of the menu's "Hide in
                                // graph" entry), so the two differ on purpose.
                                HIDDEN_IN_GRAPH_MARK
                            } else if is_row_hovered {
                                // The "click to hide" affordance on
                                // a visible row: the context menu's
                                // own eye. Never bold: see
                                // `HIDE_IN_GRAPH_ICON`.
                                HIDE_IN_GRAPH_ICON
                            } else {
                                " "
                            };
                            // A branch with extra push targets keeps room for the
                            // compact badge (`⇉3`): the name gives way, not the badge.
                            let multi = (kind == RefRowKind::Local)
                                .then(|| multi_by_name.get(full_name.as_str()).copied())
                                .flatten();
                            let prefix = 4 + row.depth() as usize * 2;
                            let inner_room =
                                (area.width.saturating_sub(2) as usize).saturating_sub(prefix);
                            let name_room =
                                area.width.saturating_sub(4 + row.depth() as u16 * 2) as usize;
                            let name = match multi {
                                Some(t) => truncate_with_ellipsis(
                                    display_name,
                                    inner_room
                                        .saturating_sub(
                                            display_width(
                                                &crate::multi_remote_badge::compact_form(t),
                                            ) + 1,
                                        )
                                        .clamp(1, name_room),
                                ),
                                None => truncate_with_ellipsis(display_name, name_room),
                            };
                            let name_width = display_width(&name);
                            let (kind_color, _) = ref_style(kind);
                            let mut style = if is_hidden {
                                Style::default().fg(th().ui.muted)
                            } else {
                                Style::default().fg(kind_color)
                            };
                            if is_checked_out || (is_active && selected_row == Some(row_index)) {
                                style = style.add_modifier(Modifier::BOLD);
                            }
                            // The ref currently in the air reads as picked up: dimmed and italic
                            // where it normally sits, so the row the drag left behind does not claim
                            // to be where the ref is.
                            if ref_drag
                                .as_ref()
                                .is_some_and(|d| d.started && d.source == *full_name)
                            {
                                style = style.add_modifier(Modifier::DIM | Modifier::ITALIC);
                            }
                            // A tag says where it lives with a trailing marker
                            // (`tag_where_ui`): up arrow = only here (the amber the stash
                            // rows use for "local only"), down arrow = only on a remote (the
                            // row is also dimmed and italic: it is not in this repository),
                            // left-right = both, up-down = both at different commits. The
                            // glyphs are text-presentation and present in every monospace
                            // font measured for this project -- the standing glyph rule; an
                            // emoji arrow would flip presentation on the selected row, which
                            // has bitten this sidebar once. A marker rather than a color on
                            // its own: the tag rows are already colored by kind.
                            let place = (kind == RefRowKind::Tag)
                                .then(|| {
                                    crate::tag_where_ui::lookup(full_name).map(|i| i.place).or(
                                        unpushed_tags
                                            .contains(full_name.as_str())
                                            .then_some(gitpoulpi_core::domain::TagWhere::Local),
                                    )
                                })
                                .flatten();
                            // The ref marked for a keyboard drop (`ref_drop_mark`).
                            let is_marked = ref_drop_mark
                                .as_ref()
                                .is_some_and(|(n, k)| n == full_name && *k == kind);
                            if is_marked {
                                style = style.add_modifier(Modifier::UNDERLINED);
                            }
                            if place == Some(gitpoulpi_core::domain::TagWhere::Remote) && !is_hidden
                            {
                                style = style.fg(th().ui.muted).add_modifier(Modifier::ITALIC);
                            }
                            let mut spans = vec![
                                Span::raw(indent),
                                Span::raw(checkmark),
                                Span::raw(" "),
                                Span::raw(eye),
                                Span::raw(" "),
                                Span::styled(name, style),
                            ];
                            if let Some(targets) = multi {
                                let room = inner_room.saturating_sub(name_width + 1);
                                if let Some(text) = crate::multi_remote_badge::fit(targets, room) {
                                    badge_hits.push((
                                        row_index,
                                        (1 + prefix + name_width + 1) as u16,
                                        display_width(&text) as u16,
                                        full_name.clone(),
                                    ));
                                    let tint = crate::multi_remote_badge::color(
                                        crate::multi_remote_badge::health(targets),
                                    );
                                    spans.push(Span::raw(" "));
                                    spans.push(Span::styled(text, Style::default().fg(tint)));
                                }
                            }
                            if let Some(place) = place {
                                spans.push(Span::styled(
                                    format!(" {}", crate::tag_where_ui::glyph(place)),
                                    Style::default().fg(crate::tag_where_ui::color(place)),
                                ));
                            }
                            if is_marked {
                                spans.push(Span::styled(
                                    format!(" {}", crate::ref_drop::MARK_GLYPH),
                                    Style::default()
                                        .fg(th().ui.accent)
                                        .add_modifier(Modifier::BOLD),
                                ));
                            }
                            ListItem::new(Line::from(spans))
                        }
                    }
                })
                .collect()
        } else {
            Vec::new()
        };
        // Per-section soloed/total count, shown instead of the
        // plain total while soloing mode is active -- e.g.
        // "LOCAL (1/5)" meaning 1 of this section's 5 refs is
        // currently soloed.
        // Icon between the chevron and the name.
        // The count is the section's own row count so worktrees
        // and stashes report theirs too, not just ref sections.
        let icon = section.icon();
        let total = match kind_opt {
            Some(_) => names.len(),
            None if section == SidebarSection::Worktrees
                && linked_worktree_count(worktrees) == 0 =>
            {
                0
            }
            None => section_row_count(section) as usize,
        };
        let section_title = if soloing && kind_opt.is_some() {
            let soloed = names.iter().filter(|n| solo_refs.contains(*n)).count();
            format!(
                "{chevron} {icon} {} ({soloed}/{total})",
                section.display_title()
            )
        } else {
            format!("{chevron} {icon} {} ({total})", section.display_title())
        };
        let section_title_width = display_width(&section_title) as u16;
        // The title span gets `hover_style` (`theme.rs`): the same "lift the bg,
        // keep the caller's fg/bold" treatment every title-bar text button uses.
        let header_title_style = if *hovered_target == Some(HitTarget::RefsSectionHeader(section)) {
            header_style.patch(plain_button_style(true))
        } else {
            header_style
        };
        // The rows the pointer can light: none for a collapsed section or the
        // "none" placeholder line.
        let hover_len = if !expanded
            || (kind_opt.is_some() && rows.is_empty())
            || (section == SidebarSection::Worktrees && linked_worktree_count(worktrees) == 0)
        {
            0
        } else {
            items.len()
        };
        // Render through a throwaway clone so a
        // wheel-scrolled offset survives the frame instead of
        // snapping back to the selection -- see
        // `ui_kit::list_render_state`. `area` is the bordered
        // rect; the border top+bottom are what `list_render_state`
        // must not count as content rows.
        let mut sidebar_render_state = list_render_state(
            sidebar_states.state(section),
            sidebar_states.follow(section),
            sidebar_visible,
        );
        // The rows the list really has after the folders are folded: the
        // scrollbar measures these, never the section's ref count in the title.
        let visible_len = items.len();
        frame.render_stateful_clipped(
            List::new(items)
                .block(
                    Block::default()
                        .title(Span::styled(section_title, header_title_style))
                        .border_type(BorderType::Rounded)
                        .borders(Borders::ALL)
                        .border_style(header_style),
                )
                // Only the focused sub-pane paints its selection highlight: each keeps
                // its own `ListState`, but three highlights at once read as three
                // selected rows.
                //
                // No BOLD in a ref section's highlight: it would patch the gutter's eye
                // too and flip it to its color presentation (`HIDE_IN_GRAPH_ICON`); the
                // selected leaf bolds its own name instead.
                .highlight_style(match (is_active, kind_opt) {
                    (false, _) => Style::default(),
                    (true, Some(_)) => Style::default().bg(th().ui.selection),
                    (true, None) => Style::default()
                        .bg(th().ui.selection)
                        .add_modifier(Modifier::BOLD),
                }),
            area,
            &mut sidebar_render_state,
        );
        *sidebar_states.state_mut(section).offset_mut() = sidebar_render_state.offset();
        // The row under the pointer lights across its whole width, in every
        // section alike (the selected row of the focused section keeps its
        // own, stronger band).
        if let Some((index, band)) =
            crate::refs_sidebar::hovered_row_of(area, sidebar_render_state.offset(), hover_len)
        {
            if !(is_active && sidebar_states.state(section).selected() == Some(index)) {
                crate::refs_sidebar::paint_row_hover(frame.buffer_mut(), band);
            }
        }
        // A scrollbar over the
        // section's own right border, rendering-only (see the
        // Log's own site for why). `visible_len` is this section's own
        // row count after the folders are folded (`total` is the count of
        // refs, the title's).
        if let Some(track) =
            scrollbar::Track::vertical(area, visible_len, sidebar_states.state(section).offset())
        {
            scrollbar::render(frame, track, true, false);
        }
        // A branch rename from the Refs row (or the palette): the name box
        // over the branch's own row, from its label to the row's end, with
        // the refusal on the row below (above on the last visible row).
        if let (
            Some(RefRowKind::Local),
            true,
            InputMode::InlineName {
                op:
                    RefNameOp::RenameBranch {
                        name: renaming,
                        spot: RenameSpot::Refs | RenameSpot::Auto,
                    },
                value,
            },
        ) = (kind_opt, expanded, input_mode)
        {
            let offset = sidebar_render_state.offset();
            let leaf = rows.iter().position(
                |row| matches!(row, RefTreeRow::Leaf { full_name, .. } if full_name == renaming),
            );
            if let Some(index) = leaf.filter(|i| {
                name_slot.may_draw(NameSpot::Refs) && *i >= offset && *i < offset + sidebar_visible
            }) {
                // indent, checkmark, space, eye, space: the label's column.
                let x = area.x + 1 + rows[index].depth() as u16 * 2 + 4;
                let y = area.y + 1 + (index - offset) as u16;
                let end = area.x + area.width.saturating_sub(1);
                if end >= x + 8 {
                    let below = index + 1 < offset + sidebar_visible;
                    let error_y = if below { y + 1 } else { y.saturating_sub(1) };
                    let error_rect = (below || index > offset)
                        .then(|| Rect::new(area.x + 1, error_y, end - area.x - 1, 1));
                    let text_rect = render_name_box(
                        frame,
                        Rect::new(x, y, end - x, 1),
                        error_rect,
                        InlineRefKind::Branch.placeholder(),
                        value,
                        field_editor.caret(MainField::RefName),
                        ref_name_error.as_deref(),
                    );
                    field_editor.drawn(MainField::RefName, text_rect, false);
                    name_slot.mark(NameSpot::Refs);
                }
            }
        }
        // The remotes badges of the visible rows: hit targets registered
        // before the section header's band (inner first), over the row.
        {
            let offset = sidebar_render_state.offset();
            for (index, dx, width, branch) in badge_hits.drain(..) {
                if index >= offset && index < offset + sidebar_visible {
                    frame_hits.push((
                        Rect::new(area.x + dx, area.y + 1 + (index - offset) as u16, width, 1),
                        HitTarget::RefRemotesBadge(branch),
                    ));
                }
            }
        }
        // The REMOTE header's `+` (Add remote), as in most Git clients. On the
        // header's right end, only when it cannot cover the title
        // (the shared-header rule), and registered *before* the
        // header band that contains it -- inner first. Hovered, it
        // spells out what it does, with a tooltip.
        if section == SidebarSection::Remote {
            render_add_remote_button(
                frame,
                area,
                section_title_width,
                hover(hovered_target).add_remote(),
                *hovered_target == Some(HitTarget::RefsSectionHeader(SidebarSection::Remote)),
                &mut *frame_hits,
            );
        }
        if section == SidebarSection::Worktrees && crate::build_features::WORKTREE {
            render_add_worktree_button(
                frame,
                area,
                section_title_width,
                hover(hovered_target).add_worktree(),
                *hovered_target == Some(HitTarget::RefsSectionHeader(SidebarSection::Worktrees)),
                &mut *frame_hits,
            );
        }
        frame_hits.push((
            Rect::new(area.x, area.y, area.width, 1),
            HitTarget::RefsSectionHeader(section),
        ));
    }

    // Neutral loading text over the sidebar's sections, which the loop above
    // drew as empty because `refs_result` is still the placeholder `Err`.
    // Drawn last so it sits on top of them.
    if let Some((lines, style)) = &pane_notice {
        render_centered_message(frame, refs_inner, lines, *style);
    }

    // The Refs pane's own collapse affordance, painted last: the collapsed
    // sidebar's near-zero-width render pass above could not have drawn over
    // it. See `render_collapsed_pane_strip`.
    if left_pane_collapsed {
        render_collapsed_pane_strip(
            frame,
            refs_area,
            PaneSide::Refs,
            hover(hovered_target).pane_restore() == Some(PaneSide::Refs),
            &mut *frame_hits,
        );
    } else {
        render_pane_collapse_button(
            frame,
            refs_area,
            PaneSide::Refs,
            hover(hovered_target).pane_collapse() == Some(PaneSide::Refs),
            &mut *frame_hits,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gitpoulpi_core::domain::{Oid, Ref, RefKind, RefVisibility};
    use ratatui::{backend::TestBackend, Terminal};

    /// 2,000 local branches: the paint loop builds only the rows the
    /// List can show, and scrolling / selection are unchanged.
    #[test]
    fn ref_rows_built_track_the_window_not_the_branch_count() {
        let refs: Vec<Ref> = (0..2000)
            .map(|i| Ref {
                kind: RefKind::LocalBranch {
                    name: format!("branch-{i:04}"),
                },
                target: Oid([0; 20]),
                visibility: RefVisibility::Shown,
                is_head: false,
                upstream: None,
                tag_sync: Default::default(),
                multi_remote: None,
            })
            .collect();
        let dir = tempfile::tempdir().unwrap();
        #[cfg(feature = "hosting")]
        let hosting = HostingUi::with_parts(dir.path(), None, None, Vec::new, None);
        #[cfg(not(feature = "hosting"))]
        let hosting = HostingUi::new(dir.path(), None);
        let refs_result = Ok(refs);
        let worktrees_result = Ok(Vec::new());
        let stashes_result = Ok(Vec::new());
        let (hidden, collapsed_sections, collapsed_folders, solo, unpushed) = (
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
        );
        let (hovered, hovered_row, hovered_folder, mode, drag, filter) =
            (None, None, None, InputMode::Normal, None, String::new());
        let (remote_names, submodules, notice, heights) =
            (Vec::new(), Vec::new(), None, Vec::new());
        let mut states = SidebarStates::default();
        states.local.select(Some(1_500));
        let (mut hits, mut editor, mut filter_area, mut avail, mut specs, mut areas) = (
            Vec::new(),
            Default::default(),
            Rect::default(),
            0,
            Vec::new(),
            Vec::new(),
        );
        let mut terminal = Terminal::new(TestBackend::new(40, 30)).unwrap();
        REF_ROWS_BUILT.with(|n| n.set(0));
        terminal
            .draw(|frame| {
                draw_sidebar(
                    frame,
                    &mut SidebarDrawCtx {
                        focus: Focus::Refs,
                        left_pane_collapsed: false,
                        hierarchical_refs: false,
                        active_ref_section: SidebarSection::Local,
                        refs_area: Rect::new(0, 0, 40, 30),
                        hovered_target: &hovered,
                        hidden_from_graph: &hidden,
                        collapsed_ref_sections: &collapsed_sections,
                        collapsed_ref_folders: &collapsed_folders,
                        hovered_ref_row: &hovered_row,
                        hovered_ref_folder: &hovered_folder,
                        input_mode: &mode,
                        ref_drag: &drag,
                        ref_drop_mark: &None,
                        refs_filter: &filter,
                        refs_result: &refs_result,
                        remote_names: &remote_names,
                        leftover_remotes: &Default::default(),
                        sidebar_states: &mut states,
                        solo_refs: &solo,
                        submodules_result: &submodules,
                        unpushed_tags: &unpushed,
                        worktrees_result: &worktrees_result,
                        stashes_result: &stashes_result,
                        pane_notice: &notice,
                        sidebar_heights: &heights,
                        hidden_sidebar_sections: &Default::default(),
                        hosting: &hosting,
                        frame_hits: &mut hits,
                        field_editor: &mut editor,
                        refs_filter_area: &mut filter_area,
                        sidebar_layout_avail: &mut avail,
                        sidebar_layout_specs: &mut specs,
                        sidebar_section_areas: &mut areas,
                        ref_name_error: &None,
                        name_slot: &mut NameSlot::default(),
                    },
                );
            })
            .unwrap();
        let built = REF_ROWS_BUILT.with(|n| n.get());
        assert!(built > 0 && built < 40, "built {built} rows of 2000");
        // The followed selection scrolled into view and is painted.
        let text: String = terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|c| c.symbol())
            .collect();
        assert!(text.contains("branch-1500"), "selection scrolled in view");
        // A build with no provider paints neither hosting section.
        #[cfg(not(feature = "hosting"))]
        {
            assert!(!text.contains("PULL REQUESTS"), "{text}");
            assert!(!text.contains("ISSUES"), "{text}");
        }
    }

    /// The TAGS section draws `releases/v1.0.0` as the folder `releases/`
    /// holding the leaf `v1.0.0`; a collapsed folder hides its leaves.
    #[test]
    fn tags_section_draws_folders_and_honors_collapse() {
        let refs: Vec<Ref> = ["releases/v1.0.0", "releases/v1.1.0", "plain"]
            .iter()
            .map(|n| Ref {
                kind: RefKind::Tag {
                    name: n.to_string(),
                },
                target: Oid([0; 20]),
                visibility: RefVisibility::Shown,
                is_head: false,
                upstream: None,
                tag_sync: Default::default(),
                multi_remote: None,
            })
            .collect();
        let draw = |collapsed: crate::refs_tree::CollapsedFolders| -> String {
            let dir = tempfile::tempdir().unwrap();
            #[cfg(feature = "hosting")]
            let hosting = HostingUi::with_parts(dir.path(), None, None, Vec::new, None);
            #[cfg(not(feature = "hosting"))]
            let hosting = HostingUi::new(dir.path(), None);
            let refs_result = Ok(refs.clone());
            let worktrees_result = Ok(Vec::new());
            let stashes_result = Ok(Vec::new());
            let (hidden, sections, solo, unpushed) = (
                Default::default(),
                Default::default(),
                Default::default(),
                Default::default(),
            );
            let (hovered, hovered_row, hovered_folder, mode, drag, filter) =
                (None, None, None, InputMode::Normal, None, String::new());
            let (remote_names, submodules, notice, heights) =
                (Vec::new(), Vec::new(), None, Vec::new());
            let mut states = SidebarStates::default();
            let (mut hits, mut editor, mut filter_area, mut avail, mut specs, mut areas) = (
                Vec::new(),
                Default::default(),
                Rect::default(),
                0,
                Vec::new(),
                Vec::new(),
            );
            let mut terminal = Terminal::new(TestBackend::new(40, 30)).unwrap();
            terminal
                .draw(|frame| {
                    draw_sidebar(
                        frame,
                        &mut SidebarDrawCtx {
                            focus: Focus::Refs,
                            left_pane_collapsed: false,
                            hierarchical_refs: true,
                            active_ref_section: SidebarSection::Tags,
                            refs_area: Rect::new(0, 0, 40, 30),
                            hovered_target: &hovered,
                            hidden_from_graph: &hidden,
                            collapsed_ref_sections: &sections,
                            collapsed_ref_folders: &collapsed,
                            hovered_ref_row: &hovered_row,
                            hovered_ref_folder: &hovered_folder,
                            input_mode: &mode,
                            ref_drag: &drag,
                            ref_drop_mark: &None,
                            refs_filter: &filter,
                            refs_result: &refs_result,
                            remote_names: &remote_names,
                            leftover_remotes: &Default::default(),
                            sidebar_states: &mut states,
                            solo_refs: &solo,
                            submodules_result: &submodules,
                            unpushed_tags: &unpushed,
                            worktrees_result: &worktrees_result,
                            stashes_result: &stashes_result,
                            pane_notice: &notice,
                            sidebar_heights: &heights,
                            hidden_sidebar_sections: &Default::default(),
                            hosting: &hosting,
                            frame_hits: &mut hits,
                            field_editor: &mut editor,
                            refs_filter_area: &mut filter_area,
                            sidebar_layout_avail: &mut avail,
                            sidebar_layout_specs: &mut specs,
                            sidebar_section_areas: &mut areas,
                            ref_name_error: &None,
                            name_slot: &mut NameSlot::default(),
                        },
                    );
                })
                .unwrap();
            terminal
                .backend()
                .buffer()
                .content()
                .iter()
                .map(|c| c.symbol())
                .collect()
        };
        let open = draw(Default::default());
        assert!(open.contains("TAGS (3)"), "{open}");
        assert!(open.contains("releases/"), "{open}");
        assert!(open.contains("v1.0.0") && open.contains("v1.1.0"), "{open}");
        assert!(!open.contains("releases/v1.0.0"), "{open}");
        let closed = draw([(RefRowKind::Tag, "releases".to_string())].into());
        assert!(closed.contains("TAGS (3)"), "{closed}");
        assert!(closed.contains("releases/"), "{closed}");
        assert!(!closed.contains("v1.0.0"), "{closed}");
        assert!(closed.contains("plain"), "{closed}");
    }

    /// The one row rule: a Refs row and a Worktrees row light the same way
    /// under the pointer (the whole row, `ui.hover`), the rows beside them
    /// do not, and the arrow is the pointer over both.
    #[test]
    fn a_refs_row_and_a_worktrees_row_light_identically_under_the_pointer() {
        use gitpoulpi_core::domain::Worktree;
        let refs: Vec<Ref> = ["alpha", "beta"]
            .iter()
            .map(|n| Ref {
                kind: RefKind::LocalBranch {
                    name: n.to_string(),
                },
                target: Oid([0; 20]),
                visibility: RefVisibility::Shown,
                is_head: false,
                upstream: None,
                tag_sync: Default::default(),
                multi_remote: None,
            })
            .collect();
        let worktrees: Vec<Worktree> = ["main", "feature"]
            .iter()
            .enumerate()
            .map(|(i, b)| Worktree {
                path: format!("/w/{b}").into(),
                branch: Some(b.to_string()),
                is_main: i == 0,
                is_locked: false,
            })
            .collect();
        let draw = |pointer: Option<(u16, u16)>| -> (ratatui::buffer::Buffer, Vec<(SidebarSection, Rect)>) {
            crate::refs_sidebar::set_row_pointer(pointer);
            let dir = tempfile::tempdir().unwrap();
            #[cfg(feature = "hosting")]
            let hosting = HostingUi::with_parts(dir.path(), None, None, Vec::new, None);
            #[cfg(not(feature = "hosting"))]
            let hosting = HostingUi::new(dir.path(), None);
            let refs_result = Ok(refs.clone());
            let worktrees_result = Ok(worktrees.clone());
            let stashes_result = Ok(Vec::new());
            let (hidden, collapsed_sections, collapsed_folders, solo, unpushed) = (
                Default::default(),
                Default::default(),
                Default::default(),
                Default::default(),
                Default::default(),
            );
            let (hovered, hovered_row, hovered_folder, mode, drag, filter) =
                (None, None, None, InputMode::Normal, None, String::new());
            let (remote_names, submodules, notice, heights) =
                (Vec::new(), Vec::new(), None, Vec::new());
            let mut states = SidebarStates::default();
            let (mut hits, mut editor, mut filter_area, mut avail, mut specs, mut areas) = (
                Vec::new(),
                Default::default(),
                Rect::default(),
                0,
                Vec::new(),
                Vec::new(),
            );
            let mut terminal = Terminal::new(TestBackend::new(40, 40)).unwrap();
            terminal
                .draw(|frame| {
                    draw_sidebar(
                        frame,
                        &mut SidebarDrawCtx {
                            focus: Focus::Log,
                            left_pane_collapsed: false,
                            hierarchical_refs: false,
                            active_ref_section: SidebarSection::Local,
                            refs_area: Rect::new(0, 0, 40, 40),
                            hovered_target: &hovered,
                            hidden_from_graph: &hidden,
                            collapsed_ref_sections: &collapsed_sections,
                            collapsed_ref_folders: &collapsed_folders,
                            hovered_ref_row: &hovered_row,
                            hovered_ref_folder: &hovered_folder,
                            input_mode: &mode,
                            ref_drag: &drag,
                            ref_drop_mark: &None,
                            refs_filter: &filter,
                            refs_result: &refs_result,
                            remote_names: &remote_names,
                            leftover_remotes: &Default::default(),
                            sidebar_states: &mut states,
                            solo_refs: &solo,
                            submodules_result: &submodules,
                            unpushed_tags: &unpushed,
                            worktrees_result: &worktrees_result,
                            stashes_result: &stashes_result,
                            pane_notice: &notice,
                            sidebar_heights: &heights,
                            hidden_sidebar_sections: &Default::default(),
                            hosting: &hosting,
                            frame_hits: &mut hits,
                            field_editor: &mut editor,
                            refs_filter_area: &mut filter_area,
                            sidebar_layout_avail: &mut avail,
                            sidebar_layout_specs: &mut specs,
                            sidebar_section_areas: &mut areas,
                            ref_name_error: &None,
                            name_slot: &mut NameSlot::default(),
                        },
                    );
                })
                .unwrap();
            (terminal.backend().buffer().clone(), areas)
        };
        let (_, areas) = draw(None);
        let area_of = |s: SidebarSection| areas.iter().find(|(x, _)| *x == s).expect("section").1;
        let hover_bg = th().ui.hover;
        for section in [SidebarSection::Local, SidebarSection::Worktrees] {
            let area = area_of(section);
            let (first, second) = (area.y + 1, area.y + 2);
            let x = area.x + 6;
            let (lit, _) = draw(Some((x, first)));
            // Every cell of the hovered row lights, the next row does not.
            for cx in area.x + 1..area.right() - 1 {
                assert_eq!(lit[(cx, first)].bg, hover_bg, "{section:?} col {cx}");
                assert_ne!(lit[(cx, second)].bg, hover_bg, "{section:?} next row");
            }
            let (rest, _) = draw(None);
            assert_ne!(rest[(x, first)].bg, hover_bg, "{section:?} at rest");
        }
        crate::refs_sidebar::set_row_pointer(None);
    }
}

#[cfg(test)]
mod badge_tests;
