//! The right panel in working-changes mode: the Unstaged/Staged sub-panes, the LFS banner and the commit composer.

use super::*;

pub(super) fn draw_wip(frame: &mut Frame, ctx: &mut RightPanelDrawCtx<'_>) {
    let active_status_section = ctx.active_status_section;
    let ai_ui = ctx.ai_ui;
    let collapsed_status_folders = ctx.collapsed_status_folders;
    let collapsed_status_sections = ctx.collapsed_status_sections;
    let commit_amend = ctx.commit_amend;
    let commit_skip_hooks = ctx.commit_skip_hooks;
    let commit_description = ctx.commit_description;
    let commit_summary = ctx.commit_summary;
    let composer_focus = ctx.composer_focus;
    let conflicts = ctx.conflicts;
    let field_editor = ctx.field_editor;
    let focus = ctx.focus;
    let file_sel = ctx.file_sel;
    let frame_hits = &mut *ctx.frame_hits;
    let issue_ref_enabled = ctx.hosting.issue_ref_available();
    let hovered_status_row = ctx.hovered_status_row;
    let hovered_target = ctx.hovered_target;
    let lfs = ctx.lfs;
    let overflow_slots = &mut *ctx.overflow_slots;
    let refs_result = ctx.refs_result;
    let status_area = ctx.status_area;
    let status_list_mode = ctx.status_list_mode;
    let status_result = ctx.status_result;
    let status_section_areas = &mut *ctx.status_section_areas;
    let status_section_lines = &mut *ctx.status_section_lines;
    let status_states = &mut *ctx.status_states;
    let view_all_files = ctx.view_all_files;
    let wip_all = &mut *ctx.wip_all;
    let input_mode = ctx.input_mode;
    let ref_name_error = ctx.ref_name_error;
    let name_slot = &mut *ctx.name_slot;
    // `Unstaged Files` and `Staged Files` are two real sub-panes (own border,
    // chevron+count title and scroll), shaped like the Refs sidebar's sections.
    let status_border_style = if focus == Focus::Status {
        Style::default().fg(th().ui.border_focused)
    } else {
        Style::default()
    };
    let branch = checked_out_branch_name(refs_result.as_deref().unwrap_or(&[]))
        .unwrap_or(gitpoulpi_i18n::msg::sidebar::DETACHED.get());
    let change_count = status_result.as_ref().map(|t| t.files.len()).unwrap_or(0);
    let staged_count = status_result
        .as_ref()
        .map(|t| section_paths(t, StatusSection::Staged).len())
        .unwrap_or(0);
    let outer = Block::default()
        .title(format!(
            "{}{}",
            " ".repeat(crate::status_list::HEADER_LEFT_RESERVE as usize),
            working_changes_title(change_count, branch, conflicts.len())
        ))
        .border_type(BorderType::Rounded)
        .borders(Borders::ALL)
        .border_style(status_border_style);
    let inner = outer.inner(status_area);
    frame.render_clipped(outer, status_area);
    // Header buttons over the top border: red trash far left,
    // AI Explain far right.
    crate::status_list::render_status_header_buttons(
        frame,
        status_area,
        ai_ui.enabled(),
        hovered_target.as_ref(),
        frame_hits,
    );

    // The composer grows up to `COMPOSER_MAX_HEIGHT` with the room left once
    // the toolbar row and a 3-row file list are set aside, and disappears
    // below `COMPOSER_MIN_HEIGHT`. Computed once here, not inside
    // `render_commit_composer`, so the outer `Layout` and the composer's own
    // can never disagree about the band's height.
    let composer_budget = inner.height.saturating_sub(1 + 3);
    let composer_height = composer_height_for(composer_budget);
    // "Git LFS is not installed" strip, only for a repository
    // that uses LFS and only when the pane can spare the rows.
    let lfs_banner = lfs.banner_text();
    let banner_rows = lfs_banner
        .as_ref()
        .map(|text| lfs_ui::banner_rows(text, inner.width))
        .unwrap_or_default();
    let banner_height = if lfs_ui::banner_height_ok(inner.height) {
        banner_rows.len() as u16
    } else {
        0
    };
    let [toolbar_row, lfs_band, sections_band, composer_band] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(banner_height),
        Constraint::Min(0),
        Constraint::Length(composer_height),
    ])
    .areas(inner);
    if banner_height > 0 {
        let hovered = hovered_target.as_ref() == Some(&HitTarget::LfsInstallLink);
        let style = if hovered {
            hover_style(Style::default().fg(th().status.warning))
        } else {
            Style::default().fg(th().status.warning)
        };
        let lines: Vec<Line> = banner_rows
            .iter()
            .map(|row| Line::styled(row.clone(), style))
            .collect();
        frame.render_clipped(Paragraph::new(lines), lfs_band);
        frame_hits.push((lfs_band, HitTarget::LfsInstallLink));
    }

    // -- toolbar: Path/Tree segmented control + Collapse/Expand All --
    // Laid out by `status_toolbar_row`: what
    // does not fit goes behind a `⋯`.
    {
        let collapse =
            (status_list_mode == FileListMode::Tree).then_some(collapsed_status_folders.is_empty());
        let has_unstaged_changes = status_result
            .as_ref()
            .is_ok_and(|t| !section_paths(t, StatusSection::Unstaged).is_empty());
        let (line, hits, overflow) = status_toolbar_row(
            toolbar_row,
            status_list_mode,
            collapse,
            Some(view_all_files),
            lfs.has_files().then(|| lfs.only()),
            hovered_target.as_ref(),
            has_unstaged_changes,
        );
        frame_hits.extend(hits);
        if let Some((rect, items)) = overflow {
            overflow_slots.push((OverflowBar::StatusToolbar, rect, items));
        }
        frame.render_clipped(Paragraph::new(line), toolbar_row);
    }

    // -- rows per section, then the two sub-panes --
    let empty_tree = WorkingTree::default();
    let tree_ref = status_result.as_ref().unwrap_or(&empty_tree);
    *status_section_lines = StatusSection::ORDER
        .iter()
        .map(|&section| {
            let lines = build_status_section_lines(
                tree_ref,
                section,
                status_list_mode,
                collapsed_status_folders,
                "",
            );
            (section, lines)
        })
        .collect();

    let rows_of = |section: StatusSection| -> usize {
        status_section_lines
            .iter()
            .find(|(s, _)| *s == section)
            .map(|(_, l)| l.len())
            .unwrap_or(0)
    };
    // Natural height, same rule as the sidebar: borders plus
    // the rows, or borders alone when collapsed.
    let natural = |section: StatusSection| -> u16 {
        let expanded = !collapsed_status_sections.contains(&section);
        2 + if expanded { rows_of(section) as u16 } else { 0 }
    };
    let wanted: Vec<u16> = StatusSection::ORDER.iter().map(|&s| natural(s)).collect();
    let avail = sections_band.height;
    // If both fit, each gets its natural height. If not, split
    // proportionally rather than letting the second sub-pane be
    // clipped out of existence: the sidebar can assume short
    // lists, a working tree can't.
    let heights: Vec<u16> = if wanted.iter().sum::<u16>() <= avail {
        wanted.clone()
    } else {
        let total: u16 = wanted.iter().sum::<u16>().max(1);
        let mut hs: Vec<u16> = wanted
            .iter()
            .map(|w| (((*w as u32 * avail as u32) / total as u32) as u16).max(3))
            .collect();
        let over = hs.iter().sum::<u16>().saturating_sub(avail);
        if over > 0 {
            if let Some(biggest) = hs.iter_mut().max_by_key(|h| **h) {
                *biggest = biggest.saturating_sub(over).max(3);
            }
        }
        hs
    };
    let areas = Layout::vertical(
        heights
            .iter()
            .map(|h| Constraint::Length(*h))
            .collect::<Vec<_>>(),
    )
    .split(sections_band);
    // The ONE list of (section, area) every mouse site reuses, so click,
    // right-click, hover and both scroll directions agree.
    *status_section_areas = StatusSection::ORDER
        .iter()
        .copied()
        .zip(areas.iter().copied())
        .collect();

    // View all files replaces the two
    // sub-panes with the whole working tree, changes marked.
    // No section areas then, so every Unstaged/Staged mouse
    // path below finds nothing to act on.
    if view_all_files {
        status_section_areas.clear();
        wip_all.render(
            frame,
            sections_band,
            focus == Focus::Status,
            tree_ref,
            status_list_mode,
            collapsed_status_folders,
            file_sel,
        );
    }

    for (section, area) in status_section_areas.clone() {
        if area.height < 2 {
            continue;
        }
        let expanded = !collapsed_status_sections.contains(&section);
        let is_active = focus == Focus::Status && active_status_section == section;
        let chevron = crate::fold_mark::glyph_open(expanded);
        let files = section_paths(tree_ref, section).len();
        let title = format!("{chevron} {} ({files})", section.title());
        let header_style = if is_active {
            Style::default()
                .fg(th().ui.accent)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
                .fg(th().ui.muted)
                .add_modifier(Modifier::BOLD)
        };
        // A Status section header gets the same `hover_style` on its title span
        // as the Refs sidebar's section headers.
        let header_title_style = if *hovered_target == Some(HitTarget::StatusSectionHeader(section))
        {
            header_style.patch(plain_button_style(true))
        } else {
            header_style
        };
        let block = Block::default()
            .title(Span::styled(title.clone(), header_title_style))
            .border_type(BorderType::Rounded)
            .borders(Borders::ALL)
            .border_style(header_style);
        let list_area = block.inner(area);
        frame.render_clipped(block, area);

        // The header row: clicking it collapses the sub-pane, and its right end
        // carries the bulk stage/unstage button.
        //
        // **The button is registered FIRST.** `find_hit_target` returns the first
        // registered rect containing the point and the header band spans the full
        // width including the button, so the other order makes `Stage All
        // Changes` collapse the section. Too narrow for the label, it becomes a
        // `⋯` whose menu runs the same bulk action.
        if let Some((rect, items)) = render_status_bulk_button(
            frame,
            area,
            &title,
            section,
            hovered_target.as_ref(),
            frame_hits,
        ) {
            overflow_slots.push((OverflowBar::StatusBulk(section), rect, items));
        }
        frame_hits.push((
            Rect::new(area.x, area.y, area.width, 1),
            HitTarget::StatusSectionHeader(section),
        ));

        if !expanded {
            continue;
        }

        let lines: &[StatusPaneLine] = status_section_lines
            .iter()
            .find(|(s, _)| *s == section)
            .map(|(_, l)| l.as_slice())
            .unwrap_or_default();

        // A new file's name box sits at the unscrolled column.
        let naming = section == StatusSection::Unstaged
            && matches!(
                input_mode,
                InputMode::InlineName {
                    op: RefNameOp::CreateFile(_),
                    ..
                }
            );
        let h_off = if lines.is_empty() {
            crate::hscroll::claim(crate::hscroll::HPane::Status(section), area, 0)
        } else {
            let width = status_list::status_content_width(
                lines,
                status_states.state(section),
                status_states.follow(section),
                list_area.height as usize,
                lfs,
                section,
                list_area.width as usize,
            );
            let off = crate::hscroll::claim(crate::hscroll::HPane::Status(section), area, width);
            if naming {
                0
            } else {
                off
            }
        };
        let items: Vec<ListItem> = if lines.is_empty() {
            vec![ListItem::new(Span::styled(
                format!("  {}", gitpoulpi_i18n::msg::sidebar::NONE.get()),
                Style::default().fg(th().ui.muted),
            ))]
        } else {
            status_list::status_row_items(
                lines,
                section,
                hovered_status_row
                    .filter(|(s, _)| *s == section)
                    .map(|(_, i)| i),
                status_states.state(section),
                status_states.follow(section),
                list_area,
                lfs,
                file_sel,
                h_off,
                frame_hits,
            )
        };

        let highlight = if is_active {
            Style::default()
                .bg(th().ui.selection)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        // Same reason as the commit-detail files list above:
        // `List` never symbol-clears rows past its last item.
        frame.render_clipped(Clear, list_area);
        // Render through a throwaway clone so
        // a wheel-scrolled offset survives the frame instead
        // of snapping back to the selection -- see
        // `ui_kit::list_render_state`. `items` above was
        // already built against `state_offset`, so this must
        // not change what offset actually gets drawn.
        let visible = list_area.height.max(1) as usize;
        let mut render_state = list_render_state(
            status_states.state(section),
            status_states.follow(section),
            visible,
        );
        frame.render_stateful_clipped(
            List::new(items).highlight_style(highlight),
            list_area,
            &mut render_state,
        );
        *status_states.state_mut(section).offset_mut() = render_state.offset();
        // A scrollbar over the
        // sub-pane's own right border, rendering-only (see the
        // Log's own site above for why).
        if let Some(track) =
            scrollbar::Track::vertical(area, lines.len(), status_states.state(section).offset())
        {
            scrollbar::render(frame, track, true, false);
        }
        crate::hscroll::paint_bar(
            frame,
            crate::hscroll::HPane::Status(section),
            frame_hits,
            None,
        );
        // A file being named: the box over the row under its folder.
        if section == StatusSection::Unstaged {
            if let InputMode::InlineName {
                op: RefNameOp::CreateFile(folder),
                value,
            } = input_mode
            {
                paint_new_file_box(
                    frame,
                    list_area,
                    lines,
                    status_states.state(section).offset(),
                    folder,
                    value,
                    field_editor,
                    ref_name_error.as_deref(),
                    name_slot,
                );
            }
        }
    }

    if let Some((rect, items)) = render_commit_composer(
        frame,
        composer_band,
        frame_hits,
        commit_summary,
        commit_description,
        commit_amend,
        composer_focus,
        staged_count,
        ctx.merging,
        ComposerHover {
            conflicts: conflicts.len(),
            more: hover(hovered_target).composer_more(),
            amend_checkbox: hover(hovered_target).amend_checkbox(),
            skip_hooks: commit_skip_hooks,
            skip_hooks_checkbox: hover(hovered_target).skip_hooks_checkbox(),
            commit_button: hover(hovered_target).commit_button(),
            ai: ComposerAi {
                enabled: ai_ui.enabled(),
                running: ai_ui.running(),
                hovered: hover(hovered_target).ai_generate(),
                compose_hovered: hover(hovered_target).ai_compose(),
            },
            issue_ref: crate::composer::ComposerIssueRef {
                shown: cfg!(feature = "hosting"),
                enabled: issue_ref_enabled,
                hovered: hover(hovered_target).issue_ref(),
            },
        },
        field_editor,
    ) {
        overflow_slots.push((OverflowBar::Composer, rect, items));
    }
}

/// The new-file name box of the Status list (`ref_name_box.rs`): over the row
/// just below the target folder's own row, one level deeper; when the folder
/// has no row (the root, a folder with no changes, a flat list) over the
/// first row, behind a muted `folder/` prefix. Nothing is drawn when the row
/// is not on screen or another spot already holds the field: the name box
/// takes over.
#[allow(clippy::too_many_arguments)]
fn paint_new_file_box(
    frame: &mut Frame,
    list_area: Rect,
    lines: &[StatusPaneLine],
    offset: usize,
    folder: &std::path::Path,
    value: &str,
    field_editor: &crate::text_edit::FieldEditor<MainField>,
    error: Option<&str>,
    name_slot: &mut NameSlot,
) {
    if !name_slot.may_draw(NameSpot::Status) {
        return;
    }
    let folder_row = (!folder.as_os_str().is_empty())
        .then(|| {
            lines.iter().position(|l| {
                matches!(l, StatusPaneLine::Folder { path, collapsed: false, .. } if path == folder)
            })
        })
        .flatten();
    let (index, indent, prefix) = match folder_row {
        Some(i) => {
            let depth = match &lines[i] {
                StatusPaneLine::Folder { depth, .. } | StatusPaneLine::File { depth, .. } => *depth,
            };
            (i + 1, 2 * (depth + 1) + 2, String::new())
        }
        None if folder.as_os_str().is_empty() => (0, 2, String::new()),
        None => (
            0,
            2,
            format!("{}/", folder.to_string_lossy().replace('\\', "/")),
        ),
    };
    let visible = usize::from(list_area.height);
    if index < offset || index >= offset + visible {
        return;
    }
    let y = list_area.y + (index - offset) as u16;
    let x = list_area.x + indent as u16;
    let prefix_w = if prefix.is_empty() {
        0
    } else {
        display_width(&prefix) as u16 + 1
    };
    let right = list_area.right();
    if right < x + prefix_w + 8 {
        return;
    }
    if !prefix.is_empty() {
        frame.render_widget(Clear, Rect::new(x, y, prefix_w, 1));
        frame.render_widget(
            Paragraph::new(Span::styled(
                format!(" {prefix}"),
                Style::default()
                    .fg(th().ui.muted)
                    .bg(crate::input_focus_bg()),
            )),
            Rect::new(x, y, prefix_w, 1),
        );
    }
    let below = index + 1 < offset + visible;
    let error_rect = (below || index > offset).then(|| {
        let ey = if below { y + 1 } else { y - 1 };
        Rect::new(list_area.x, ey, list_area.width, 1)
    });
    let text_rect = render_name_box(
        frame,
        Rect::new(x + prefix_w, y, right - x - prefix_w, 1),
        error_rect,
        crate::ref_name_box::box_placeholder(&RefNameOp::CreateFile(folder.to_path_buf())),
        value,
        field_editor.caret(MainField::RefName),
        error,
    );
    field_editor.drawn(MainField::RefName, text_rect, false);
    name_slot.mark(NameSpot::Status);
}
