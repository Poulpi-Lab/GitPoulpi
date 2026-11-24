//! The right panel in commit-detail mode: the selected commit's message, author, toolbar and changed-files list.

use super::*;

pub(super) fn draw_commit_detail(frame: &mut Frame, ctx: &mut RightPanelDrawCtx<'_>) {
    let avatar_capability = ctx.avatar_capability;
    let avatar_live = ctx.avatar_live;
    let avatar_placement = ctx.avatar_placement;
    let avatar_service = &mut *ctx.avatar_service;
    let collapsed_commit_folders = ctx.collapsed_commit_folders;
    let commit_file_focus = ctx.commit_file_focus;
    let commit_file_rows = &mut *ctx.commit_file_rows;
    let commit_file_state = &mut *ctx.commit_file_state;
    let commit_files = ctx.commit_files;
    let commit_files_filter = ctx.commit_files_filter;
    let commit_files_follow = ctx.commit_files_follow;
    let commit_filter_focus = ctx.commit_filter_focus;
    let commit_list_mode = ctx.commit_list_mode;
    let mut commit_message_for = *ctx.commit_message_for;
    let mut commit_message_scroll = *ctx.commit_message_scroll;
    let commit_message_user_rows = ctx.commit_message_user_rows;
    let commit_paths_cache = &mut *ctx.commit_paths_cache;
    let commit_ref_urls = &mut *ctx.commit_ref_urls;
    let commit_sort_desc = ctx.commit_sort_desc;
    let detail_avatar_cols = ctx.detail_avatar_cols;
    let detail_avatar_image = &mut *ctx.detail_avatar_image;
    let detail_avatar_rows = ctx.detail_avatar_rows;
    let field_editor = ctx.field_editor;
    let file_sel = ctx.file_sel;
    let frame_hits = &mut *ctx.frame_hits;
    let hosting = &mut *ctx.hosting;
    let hovered_target = ctx.hovered_target;
    let lfs = ctx.lfs;
    let log_order = ctx.log_order;
    let log_result = ctx.log_result;
    let log_state = ctx.log_state;
    let stashes_result = ctx.stashes_result;
    let status_area = ctx.status_area;
    let view_all_files = ctx.view_all_files;
    // A stash row is detailed as the commit it is (see
    // `stash_as_commit`): its files are what it changed on top
    // of its base.
    let selected_stash_commit: Option<CommitNode> =
        selected_log_stash(stashes_result, log_state, log_order)
            .as_ref()
            .and_then(stash_as_commit);
    // A comparison (the Log's multi-select) stands in for the selected
    // commit: a pseudo-commit carrying its synthetic key, so the file list
    // below needs no second implementation.
    let compare = ctx.compare;
    let compare_node = compare.as_ref().map(|c| c.node());
    let selected_summary = compare_node.as_ref().or_else(|| {
        log_result
            .as_ref()
            .ok()
            .and_then(|g| match log_order.entry(log_state.selected()?) {
                graph::LogEntry::Commit(i) => g.nodes.get(i),
                graph::LogEntry::Wip | graph::LogEntry::Stash(_) => None,
            })
            .or(selected_stash_commit.as_ref())
    });
    // The two blocks are separately-closed boxes, so one style
    // (`commit_file_focus`) must drive both borders for them to read as one
    // pane; a meta half that is always ACCENT makes the selection look
    // half-applied.
    let detail_border_style = if commit_file_focus {
        Style::default().fg(th().ui.border_focused)
    } else {
        Style::default()
    };

    // -- file toolbar: change summary, Path/Tree, View all files, filter --
    // Whether the list below is the whole tree rather than the
    // changed files -- what its labels have to say.
    let listing_tree = view_all_files
        && matches!(
            (selected_summary, &commit_files),
            (Some(commit), Some(outcome))
                if outcome.commit == commit.id
                    && matches!(outcome.tree, Some(Ok(_)))
        );
    let commit_paths: Vec<PathBuf> = match (selected_summary, &commit_files) {
        (Some(commit), Some(outcome)) if outcome.commit == commit.id => {
            // `View all files` swaps the question: the whole
            // tree at this revision instead of the paths this
            // commit touched. Falls back to the changed set
            // while the tree is in flight or failed, so the
            // pane never blanks out -- an empty list would
            // read as "this commit changed nothing".
            match (view_all_files, &outcome.tree) {
                (true, Some(Ok(paths))) => paths.clone(),
                _ => outcome
                    .files
                    .as_ref()
                    .map(|fs| fs.iter().map(|f| f.path.clone()).collect())
                    .unwrap_or_default(),
            }
        }
        _ => Vec::new(),
    };

    // Sorted here, before the tree/filter rows are built from
    // it, so every index that maps a row back to a path
    // (`commit_paths_cache`) sees the same order. Only the Path
    // list follows it: the Tree lists folders first, by name.
    let mut commit_paths = commit_paths;
    if commit_sort_desc {
        commit_paths.sort_by(|a, b| b.cmp(a));
    }

    // -- the message block: summary + description, scrollable --
    let message_source: &str = selected_summary
        .map(|c| {
            if c.message.trim().is_empty() {
                c.summary.as_str()
            } else {
                c.message.as_str()
            }
        })
        .unwrap_or("");
    let message_width = status_area.width.saturating_sub(4) as usize;
    // A comparison replaces the message with one card per selected row
    // (`compare_cards`); the block's scroll, splitter and scrollbar serve
    // them as they serve a message.
    let card_rows: Option<Vec<crate::compare_cards::CardRow>> = compare.as_ref().map(|c| {
        crate::compare_cards::build(
            c,
            &crate::compare_cards::CardLook {
                width: message_width,
                now: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map_or(0, |d| d.as_secs() as i64),
                avatars: avatar::card_avatar_mode(avatar_placement, avatar_capability),
                fallback: avatar_live.fallback,
                hovered: match hovered_target {
                    Some(HitTarget::CompareCard(i)) => Some(*i),
                    _ => None,
                },
                cursor_row: log_state.selected(),
            },
        )
    });
    let wrapped_rows: Vec<(String, bool, bool)> = match compare {
        Some(_) => Vec::new(),
        None => crate::commit_detail::message_display_rows(message_source, message_width),
    };
    let message_lines: Vec<(String, bool)> = wrapped_rows
        .iter()
        .map(|(text, summary, _)| (text.clone(), *summary))
        .collect();
    // The web addresses of the message, per display row (an address wider
    // than the pane is found whole), and all of them for the keyboard twin.
    let link_rows: Vec<(&str, bool)> = wrapped_rows
        .iter()
        .map(|(text, _, cont)| (text.as_str(), *cont))
        .collect();
    let row_urls = crate::message_links::row_links(&link_rows);
    crate::link_confirm::set_message_urls(crate::message_links::all_urls(&link_rows));
    let message_line_count = card_rows
        .as_ref()
        .map_or(message_lines.len(), |rows| rows.len());
    let selected_id = selected_summary.map(|c| c.id);
    if selected_id != commit_message_for {
        commit_message_for = selected_id;
        commit_message_scroll = 0;
    }
    let msg_rows = if card_rows.is_some() {
        compare_card_rows(
            status_area.height,
            message_line_count,
            commit_message_user_rows,
        )
    } else {
        message_rows(
            status_area.height,
            message_line_count,
            commit_message_user_rows,
        )
    };
    let layout = commit_detail_layout(status_area, msg_rows);
    commit_message_scroll =
        commit_message_scroll.min(message_line_count.saturating_sub(msg_rows as usize));
    let commit_message_total = message_line_count;
    let commit_message_rows_now = msg_rows;
    let commit_message_area = layout.message_area;
    let commit_detail_status_height = status_area.height;
    if selected_summary.is_some()
        && layout.message_area.width > 2
        && layout.message_area.height >= 3
    {
        // The splitter under the message: its bottom border row
        // without the corners, registered ahead of anything else.
        let a = layout.message_area;
        frame_hits.push((
            Rect::new(a.x + 1, a.y + a.height - 1, a.width - 2, 1),
            HitTarget::CommitMessageBoundary,
        ));
    }

    let meta_inner_width = layout.author_area.width as usize;
    let swap_hint = keymap::bracket(&keymap::current(), keymap::Action::LogCompareSwap);
    let swap_label = format!(
        " \u{21C4} {}{} ",
        gitpoulpi_i18n::msg::compare::SWAP.get(),
        swap_hint
    );
    let header_line: Line = match (compare, selected_summary) {
        (Some(c), _) => {
            let head = format!(" {} ", gitpoulpi_i18n::msg::compare::HEADER.get());
            let head_text = head.clone();
            let room = (layout.header_row.width as usize)
                .saturating_sub(display_width(&head) + display_width(&swap_label));
            let mut spans = vec![Span::styled(head, Style::default().fg(th().ui.accent))];
            // Only drawn where it fits: a rect with no glyph under it is a
            // click that does nothing.
            if layout.header_row.width as usize
                >= display_width(&swap_label) + display_width(&head_text)
            {
                let swap_w = display_width(&swap_label) as u16;
                let x = layout.header_row.right().saturating_sub(swap_w);
                let hovered = *hovered_target == Some(HitTarget::CompareSwap);
                let base = Style::default().fg(if c.can_swap {
                    th().ui.text
                } else {
                    th().ui.muted
                });
                spans.push(Span::raw(" ".repeat(room)));
                spans.push(Span::styled(
                    swap_label.clone(),
                    if hovered {
                        base.patch(plain_button_style(true))
                    } else {
                        base
                    },
                ));
                frame_hits.push((
                    Rect::new(x, layout.header_row.y, swap_w, 1),
                    HitTarget::CompareSwap,
                ));
            }
            Line::from(spans)
        }
        (None, Some(commit)) => Line::from(Span::styled(
            gitpoulpi_i18n::msg::commit_detail::header(short_oid(commit.id)),
            Style::default().fg(th().ui.accent),
        )),
        (None, None) => Line::from(gitpoulpi_i18n::msg::commit_detail::SELECT_A_COMMIT.get()),
    };
    // `#123` / `!45`: links only when the repository's issues /
    // pull requests are linked (see `HostingUi::token_bases`).
    let token_bases = hosting.token_bases();
    commit_ref_urls.clear();
    let mut reference_labels: Vec<(String, String)> = Vec::new();
    for link in crate::issue_tokens::links(message_source, &token_bases) {
        if !reference_labels.iter().any(|(l, _)| *l == link.reference) {
            reference_labels.push((link.reference, link.url));
        }
    }
    hosting.set_commit_references(reference_labels);
    let card_slots: Vec<(usize, usize)> = match &card_rows {
        Some(rows) => {
            let mut slots = Vec::new();
            for (i, row) in rows
                .iter()
                .skip(commit_message_scroll)
                .take(msg_rows as usize)
                .enumerate()
            {
                let y = layout.message_inner.y + i as u16;
                if let Some(card) = row.card {
                    // One target per painted line, so a card scrolled half
                    // out of view is still clickable where it shows.
                    frame_hits.push((
                        Rect::new(layout.message_inner.x, y, layout.message_inner.width, 1),
                        HitTarget::CompareCard(card),
                    ));
                    if row.first {
                        slots.push((i, card));
                    }
                }
            }
            slots
        }
        None => Vec::new(),
    };
    let message_visible: Vec<Line> = if let Some(rows) = &card_rows {
        rows.iter()
            .skip(commit_message_scroll)
            .take(msg_rows as usize)
            .map(|row| row.line.clone())
            .collect()
    } else {
        message_lines
            .iter()
            .skip(commit_message_scroll)
            .take(msg_rows as usize)
            .enumerate()
            .map(|(row_index, (text, is_summary))| {
                let base = if *is_summary {
                    Style::default()
                        .fg(th().ui.text)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(th().ui.text)
                };
                let tokens = crate::message_links::merge(
                    crate::issue_tokens::links(text, &token_bases),
                    row_urls
                        .get(commit_message_scroll + row_index)
                        .map_or(&[][..], |v| v.as_slice()),
                );
                let chars: Vec<char> = text.chars().collect();
                // Which links the pointer is on: they take the hover style.
                let mut hot: Vec<bool> = Vec::with_capacity(tokens.len());
                for token in &tokens {
                    let mut is_hot = false;
                    let before: String = chars[..token.start].iter().collect();
                    let word: String = chars[token.start..token.end].iter().collect();
                    let x = layout.message_inner.x + display_width(&before) as u16;
                    let y = layout.message_inner.y + row_index as u16;
                    if y < layout.message_inner.bottom() && x < layout.message_inner.right() {
                        let width =
                            (display_width(&word) as u16).min(layout.message_inner.right() - x);
                        frame_hits.push((
                            Rect::new(x, y, width, 1),
                            HitTarget::CommitReference(commit_ref_urls.len()),
                        ));
                        is_hot = *hovered_target
                            == Some(HitTarget::CommitReference(commit_ref_urls.len()));
                        commit_ref_urls.push((token.url.clone(), token.confirm));
                    }
                    hot.push(is_hot);
                }
                crate::issue_tokens::styled_line(text, base, &tokens, &hot)
            })
            .collect()
    };
    // The message bands: a right click or a double click on the summary or
    // the description amends that one property. After the reference tokens
    // above (inner first) and never over a comparison's cards.
    if card_rows.is_none() && selected_summary.is_some() {
        for (i, (_text, is_summary)) in message_lines
            .iter()
            .skip(commit_message_scroll)
            .take(msg_rows as usize)
            .enumerate()
        {
            let y = layout.message_inner.y + i as u16;
            if y >= layout.message_inner.bottom() {
                continue;
            }
            let part = if *is_summary {
                AmendScope::Summary
            } else {
                AmendScope::Description
            };
            frame_hits.push((
                Rect::new(layout.message_inner.x, y, layout.message_inner.width, 1),
                HitTarget::CommitDetailPart(part),
            ));
        }
    }
    // Where the author's name sits on the block's first line (offset and
    // width) and the email its hover tooltip shows.
    let mut author_name_hit: Option<(u16, u16, String)> = None;
    let author_lines: Vec<Line> = match selected_summary.filter(|_| compare.is_none()) {
        Some(commit) => {
            let parent = commit
                .parents
                .first()
                .map(|p| short_oid(*p))
                .unwrap_or_else(|| gitpoulpi_i18n::msg::commit_detail::ROOT.get().into());
            let parent_label = gitpoulpi_i18n::msg::commit_detail::parent(parent);
            let name = if commit.author_name.is_empty() {
                commit.author_email.clone()
            } else {
                commit.author_name.clone()
            };
            let with_avatar = avatar::enabled(avatar_placement);
            // Room the avatar takes before the text: its square
            // box plus a gap when pictures are drawn, else the
            // one-cell initials badge and a space.
            let lead = if avatar::should_fetch_images(avatar_placement, avatar_capability) {
                usize::from(detail_avatar_cols) + 1
            } else if with_avatar {
                2
            } else {
                0
            };
            // Never cut: a name wider than the pane scrolls with the line
            // (`hscroll`, the commit info), the parent label after it.
            let gap = meta_inner_width
                .saturating_sub(lead + display_width(&name) + display_width(&parent_label))
                .max(1);
            let mut first: Vec<Span> = Vec::new();
            if with_avatar {
                // The picture's own place on a terminal that draws images
                // (the whole box is reserved for it): while it is not
                // there -- loading, no account, a failed decode, a
                // terminal that refuses it -- the initials fill the
                // Log's two-cell slot instead of one lone cell in a
                // blank box. The picture replaces these cells.
                let glyph_cols = if lead > 2 {
                    usize::from(avatar::SLOT_COLS)
                } else {
                    1
                };
                let (glyph, glyph_color) = avatar::fallback_text(
                    avatar_live.fallback,
                    &commit.author_name,
                    &commit.author_email,
                    glyph_cols,
                );
                first.push(Span::styled(
                    glyph,
                    Style::default()
                        .fg(glyph_color)
                        .add_modifier(Modifier::BOLD),
                ));
                first.push(Span::raw(" ".repeat(lead.saturating_sub(glyph_cols))));
            }
            if !commit.author_email.is_empty() {
                author_name_hit = Some((
                    lead as u16,
                    display_width(&name) as u16,
                    commit.author_email.clone(),
                ));
            }
            first.push(Span::styled(name, Style::default().fg(th().ui.text)));
            first.push(Span::raw(" ".repeat(gap)));
            first.push(Span::styled(
                parent_label,
                Style::default().fg(th().ui.muted),
            ));
            vec![
                Line::from(first),
                Line::from(vec![
                    Span::raw(" ".repeat(lead)),
                    Span::styled(
                        format!("{} ", gitpoulpi_i18n::msg::commit_detail::AUTHORED.get()),
                        Style::default().fg(th().ui.muted),
                    ),
                    Span::styled(
                        // The detail panel has room to spell the
                        // date out, so it reads `date_verbose_format`
                        // rather than the Log column's
                        // `date_time_format`.
                        format_timestamp_full(commit.timestamp),
                        Style::default().fg(th().ui.text),
                    ),
                ]),
            ]
        }
        None => Vec::new(),
    };
    let counts_line: Line = {
        let shown = match (selected_summary, &commit_files) {
            (Some(commit), Some(outcome)) if outcome.commit == commit.id => {
                outcome.files.as_ref().ok()
            }
            _ => None,
        };
        match shown {
            Some(files) => {
                let (modified, added, deleted, renamed) =
                    change_counts(files.iter().map(|f| f.status));
                let mut spans: Vec<Span> = Vec::new();
                // `View all files` lists the whole tree, so say
                // how big it is ahead of what this commit changed.
                if listing_tree {
                    spans.push(Span::styled(
                        gitpoulpi_i18n::msg::commit_detail::files(commit_paths.len() as u64),
                        Style::default().fg(th().ui.muted),
                    ));
                }
                for (n, glyph, word, color) in [
                    (
                        modified,
                        "\u{270E}",
                        gitpoulpi_i18n::msg::commit_detail::modified(modified as u64),
                        th().status.warning,
                    ),
                    (
                        added,
                        "+",
                        gitpoulpi_i18n::msg::commit_detail::added(added as u64),
                        th().refs.local,
                    ),
                    (
                        deleted,
                        "-",
                        gitpoulpi_i18n::msg::commit_detail::deleted(deleted as u64),
                        th().status.danger,
                    ),
                    (
                        renamed,
                        ">",
                        gitpoulpi_i18n::msg::commit_detail::renamed(renamed as u64),
                        th().refs.remote,
                    ),
                ] {
                    if n > 0 {
                        if !spans.is_empty() {
                            spans.push(Span::raw("  "));
                        }
                        spans.push(Span::styled(
                            format!("{glyph} {n} {word}"),
                            Style::default().fg(color),
                        ));
                    }
                }
                if spans.is_empty() {
                    spans.push(Span::styled(
                        gitpoulpi_i18n::msg::commit_detail::NO_FILE_CHANGES.get(),
                        Style::default().fg(th().ui.muted),
                    ));
                }
                Line::from(spans)
            }
            None => Line::default(),
        }
    };

    let toolbar_line = {
        let bar = layout.toolbar_row;
        let mut spans: Vec<Span> = Vec::new();
        // Key hints only when the whole row still fits with them:
        // they must never truncate the checkbox label.
        let hints = bar.width as usize >= 6 + 2 + 12 + 4 + 2 + 24;
        // The sort toggle (`A->Z` / `Z->A`): path order of the
        // Path list. Two leading spaces so its Ctrl+N badge lands
        // on a blank cell.
        let sort_label = if commit_sort_desc {
            SORT_LABEL_DESC
        } else {
            SORT_LABEL
        };
        let sort_style = if *hovered_target == Some(HitTarget::CommitSortButton) {
            Style::default()
                .fg(th().ui.muted)
                .patch(plain_button_style(true))
        } else {
            Style::default().fg(th().ui.muted)
        };
        spans.push(Span::styled(sort_label, sort_style));
        frame_hits.push((
            Rect::new(bar.x, bar.y, display_width(sort_label) as u16, 1),
            HitTarget::CommitSortButton,
        ));
        spans.push(Span::raw("  "));
        let mut x = bar.x + display_width(sort_label) as u16 + 2;
        let km_now = keymap::current();
        for mode in [FileListMode::Path, FileListMode::Tree] {
            let label = match mode {
                FileListMode::Path => {
                    format!(" {} ", gitpoulpi_i18n::msg::commit_detail::MODE_PATH.get())
                }
                FileListMode::Tree if hints => format!(
                    " {}{} ",
                    gitpoulpi_i18n::msg::commit_detail::MODE_TREE.get(),
                    keymap::bracket(&km_now, keymap::Action::LogToggleFileMode)
                ),
                FileListMode::Tree => {
                    format!(" {} ", gitpoulpi_i18n::msg::commit_detail::MODE_TREE.get())
                }
            };
            let label_len = display_width(&label) as u16;
            spans.push(Span::styled(
                label,
                segmented_style(
                    commit_list_mode == mode,
                    *hovered_target == Some(HitTarget::FileListModeButton(mode)),
                ),
            ));
            frame_hits.push((
                Rect::new(x, bar.y, label_len, 1),
                HitTarget::FileListModeButton(mode),
            ));
            x += label_len;
        }
        // The whole tree at this revision (`list_tree`); the WIP row's toolbar
        // has the same toggle.
        let vaf = format!(
            "  {} {}{}",
            if view_all_files { "[x]" } else { "[ ]" },
            gitpoulpi_i18n::msg::commit_detail::VIEW_ALL_FILES.get(),
            if hints {
                keymap::bracket(&km_now, keymap::Action::LogViewAllFiles)
            } else {
                String::new()
            }
        );
        let vaf_len = display_width(&vaf) as u16;
        let vaf_style = if *hovered_target == Some(HitTarget::ViewAllFilesCheckbox) {
            Style::default()
                .fg(th().ui.muted)
                .patch(plain_button_style(true))
        } else {
            Style::default().fg(th().ui.muted)
        };
        // Kept in a comparison too (a comparison lists what differs, so the
        // tree is not shown): the setting is the user's, and comes back with
        // the single commit.
        spans.push(Span::styled(vaf, vaf_style));
        frame_hits.push((
            Rect::new(x, bar.y, vaf_len, 1),
            HitTarget::ViewAllFilesCheckbox,
        ));
        Line::from(spans)
    };

    let filter_line = {
        let focused = commit_filter_focus;
        // Same fixed-width rule as every other input here: the
        // label is chrome, the rest of the row is the field,
        // and a long filter scrolls inside it with the caret
        // pinned to its right edge instead of being clipped
        // off the end of the row.
        let filter_label = gitpoulpi_i18n::msg::commit_detail::FILTER_FILES.get();
        let label_w = display_width(filter_label) as u16;
        let field_w = layout.filter_row.width.saturating_sub(label_w);
        frame_hits.push((layout.filter_row, HitTarget::CommitFilterField));
        field_editor.drawn(
            MainField::CommitFilter,
            Rect::new(
                layout.filter_row.x + label_w,
                layout.filter_row.y,
                field_w,
                1,
            ),
            false,
        );
        let mut spans = vec![Span::styled(
            filter_label,
            Style::default().fg(th().ui.muted),
        )];
        if focused {
            let look = text_field::EditLook::new(
                Style::default()
                    .fg(th().ui.text)
                    .bg(crate::input_focus_bg()),
                "_",
            );
            spans.extend(
                text_field::edit_line(
                    commit_files_filter,
                    field_editor.caret(MainField::CommitFilter),
                    field_w as usize,
                    true,
                    look,
                )
                .spans,
            );
        } else {
            // Unfocused: the same hover treatment (`hover_style`) every other
            // title/section span in this file gets.
            let hovered = *hovered_target == Some(HitTarget::CommitFilterField);
            let base = Style::default()
                .fg(th().ui.text)
                .bg(crate::input_bg(false, hovered));
            spans.extend(text_field::resting_spans(
                commit_files_filter,
                field_w as usize,
                base,
            ));
        }
        Line::from(spans)
    };

    *commit_paths_cache = commit_paths.clone();
    *commit_file_rows = build_tree_rows(
        &commit_paths,
        commit_list_mode,
        collapsed_commit_folders,
        commit_files_filter,
    );
    let commit_files_len = commit_file_rows.len();

    let file_items: Vec<ListItem> = if commit_paths.is_empty() {
        match (selected_summary, &commit_files) {
            (Some(commit), Some(outcome)) if outcome.commit == commit.id => match &outcome.files {
                Err(e) => vec![ListItem::new(
                    gitpoulpi_i18n::msg::commit_detail::diff_error(e),
                )],
                _ => vec![ListItem::new(
                    gitpoulpi_i18n::msg::commit_detail::NO_FILE_CHANGES_PAREN.get(),
                )],
            },
            (Some(_), _) => vec![ListItem::new(
                gitpoulpi_i18n::msg::commit_detail::LOADING.get(),
            )],
            (None, _) => vec![],
        }
    } else {
        // Looked up by path: under View all files a row's
        // index is into the whole tree, and taking the status
        // by position put the changed file's marker on
        // whichever file happened to sort first.
        let shown_outcome = match (selected_summary, &commit_files) {
            (Some(commit), Some(outcome)) if outcome.commit == commit.id => Some(outcome),
            _ => None,
        };
        crate::file_rows::FileRows {
            rows: commit_file_rows,
            paths: &commit_paths,
            status_of: &|p| commit_file_status(shown_outcome, p),
            is_lfs: &|p| lfs.is_lfs(p),
            renamed_from: &|p| crate::file_panel::commit_file_renamed_from(shown_outcome, p),
            file_picked: &|p| file_sel.contains(FileList::Commit, p, false),
            folder_picked: &|p| file_sel.contains(FileList::Commit, p, true),
        }
        .items()
    };
    // The hint has to describe the click that is wired: a single click opens
    // the diff.
    let list_name = if listing_tree {
        gitpoulpi_i18n::msg::commit_detail::ALL_FILES.get()
    } else {
        gitpoulpi_i18n::msg::commit_detail::CHANGED_FILES.get()
    };
    let files_title = if commit_file_focus {
        format!(
            " {} ",
            gitpoulpi_i18n::msg::commit_detail::files_title_focused(list_name, commit_paths.len())
        )
    } else {
        format!(
            " {} ",
            gitpoulpi_i18n::msg::commit_detail::files_title(list_name, commit_paths.len())
        )
    };

    // The info lines scroll sideways together; a comparison's header keeps
    // its Swap button where it is.
    let info_w = author_lines
        .iter()
        .chain([&counts_line])
        .chain(compare.is_none().then_some(&header_line))
        .map(Line::width)
        .max()
        .unwrap_or(0);
    let info_off = crate::hscroll::claim(
        crate::hscroll::HPane::CommitInfo,
        layout.detail_area,
        info_w,
    );
    let header_line = if compare.is_some() {
        header_line
    } else {
        crate::hscroll::cut_line(header_line, info_off)
    };
    let author_lines: Vec<Line> = author_lines
        .into_iter()
        .map(|l| crate::hscroll::cut_line(l, info_off))
        .collect();
    let counts_line = crate::hscroll::cut_line(counts_line, info_off);

    let chrome = CommitDetailChrome {
        title: format!(
            " {} ",
            if compare.is_some() {
                gitpoulpi_i18n::msg::compare::TITLE.get()
            } else {
                gitpoulpi_i18n::msg::commit_detail::TITLE.get()
            }
        ),
        border_style: detail_border_style,
        header: header_line,
        message: message_visible,
        author: author_lines,
        counts: counts_line,
        toolbar: toolbar_line,
        filter: filter_line,
        files_title,
    };
    let layout = render_commit_detail_chrome(frame, status_area, msg_rows, &chrome);
    let commit_files_area = layout.files_area;
    crate::tooltip::set_author_email(None);
    if let Some((offset, width, email)) = author_name_hit {
        let area = layout.author_area;
        // The name scrolls with its line: clipped to the block.
        let start = i32::from(offset) - info_off as i32;
        let (from, to) = (
            start.max(0),
            (start + i32::from(width)).min(i32::from(area.width)),
        );
        if to > from && area.height > 0 {
            let rect = Rect::new(area.x + from as u16, area.y, (to - from) as u16, 1);
            frame_hits.push((rect, HitTarget::CommitAuthorName));
            crate::tooltip::set_author_email(Some(email));
        }
    }
    // The pointer over the message block and the info lines (each owns its
    // shape and its light, `pointer_map` + a bg patch): the message and the
    // description are editable (the I-beam, the whole block lit), the commit
    // id, the author and the date lines act on a click (the hand, the line
    // lit).
    if selected_summary.is_some() && compare.is_none() {
        let hot = |t: &HitTarget| *hovered_target == Some(t.clone());
        let block = layout.message_inner;
        crate::pointer_map::field(block);
        if hot(&HitTarget::CommitDetailPart(AmendScope::Summary))
            || hot(&HitTarget::CommitDetailPart(AmendScope::Description))
            || matches!(hovered_target, Some(HitTarget::CommitReference(_)))
        {
            light(frame.buffer_mut(), block);
        }
        let id_row = layout.header_row;
        if id_row.width > 0 && id_row.height > 0 {
            frame_hits.push((id_row, HitTarget::CommitIdLine));
            crate::pointer_map::hand(id_row);
            if hot(&HitTarget::CommitIdLine) {
                light(frame.buffer_mut(), id_row);
            }
        }
        let a = layout.author_area;
        if a.height > 0 {
            let first = Rect::new(a.x, a.y, a.width, 1);
            crate::pointer_map::hand(first);
            if hot(&HitTarget::CommitDetailPart(AmendScope::Author))
                || hot(&HitTarget::CommitAuthorName)
            {
                light(frame.buffer_mut(), first);
            }
        }
        if a.height > 1 {
            let second = Rect::new(a.x, a.y + 1, a.width, 1);
            crate::pointer_map::hand(second);
            if hot(&HitTarget::CommitDetailPart(AmendScope::Date)) {
                light(frame.buffer_mut(), second);
            }
        }
    }
    // The authored-date line (second row of the block) amends the date;
    // inner first, so before the whole block below.
    if selected_summary.is_some() && compare.is_none() && layout.author_area.height > 1 {
        let a = layout.author_area;
        frame_hits.push((
            Rect::new(a.x, a.y + 1, a.width, 1),
            HitTarget::CommitDetailPart(AmendScope::Date),
        ));
    }
    // The whole author block (after the name above: inner first).
    if selected_summary.is_some() && compare.is_none() && layout.author_area.height > 0 {
        frame_hits.push((
            layout.author_area,
            HitTarget::CommitDetailPart(AmendScope::Author),
        ));
    }
    // On a terminal that can draw images, replace the one
    // cell the initials badge above was just drawn into (the
    // author line's first column, inside the block's border)
    // with the real avatar once it has been fetched and
    // decoded. `status` also *starts* that fetch the first
    // time this commit is selected -- see `avatar::service`'s
    // module header for why that is safe to call every frame
    // (a `HashMap` lookup on every call after the first).
    if let Some(commit) = selected_summary.filter(|_| compare.is_none()) {
        if info_off == 0 && avatar::should_fetch_images(avatar_placement, avatar_capability) {
            let sha = commit.id.to_hex();
            if let avatar::AvatarStatus::Ready(bitmap) =
                avatar_service.status(avatar::AvatarQuery {
                    email: &commit.author_email,
                    sha: &sha,
                })
            {
                // The author block's first line starts with the
                // badge cell -- see where it is built above.
                let avatar_area = Rect::new(
                    layout.author_area.x,
                    layout.author_area.y,
                    detail_avatar_cols.min(layout.author_area.width / 3),
                    detail_avatar_rows.min(layout.author_area.height),
                );
                detail_avatar_image.render(frame.buffer_mut(), avatar_area, bitmap);
            }
        }
    }
    // The cards' pictures, over the initials slot of each first line: the
    // Log's own multi-picture renderer, memory lookups only.
    if let Some(c) = compare {
        if avatar::card_avatar_mode(avatar_placement, avatar_capability)
            == avatar::LogAvatarMode::Picture
        {
            let inner = layout.message_inner;
            let hovered = match hovered_target {
                Some(HitTarget::CompareCard(i)) => Some(*i),
                _ => None,
            };
            let wanted: Vec<(usize, usize, String)> = card_slots
                .iter()
                .filter_map(|&(line, card)| {
                    let k = c.cards.get(card)?;
                    if k.kind != crate::compare::CardKind::Commit {
                        return None;
                    }
                    Some((line, card, k.id?.to_hex()))
                })
                .collect();
            let queries: Vec<avatar::AvatarQuery> = wanted
                .iter()
                .map(|(_, card, sha)| avatar::AvatarQuery {
                    email: c.cards[*card].author_email.as_str(),
                    sha,
                })
                .collect();
            let pictures = avatar_service.pictures(&queries);
            let slots: Vec<avatar::AvatarSlot> = wanted
                .iter()
                .zip(pictures)
                .filter_map(|((line, card, _), picture)| {
                    let (author, bitmap) = picture?;
                    let area = crate::compare_cards::slot_area(inner, *line as u16)?;
                    let current = c.cards[*card].row;
                    Some(avatar::AvatarSlot {
                        area,
                        key: avatar::PictureKey {
                            author: author.to_string(),
                            tint: Some(avatar::RowTint::of(
                                log_state.selected() == Some(current),
                                hovered == Some(*card),
                            )),
                            dimmed: false,
                        },
                        bitmap,
                    })
                })
                .collect();
            ctx.log_avatar_images.render(frame.buffer_mut(), &slots);
        }
    }
    // Everything above the list counts as "meta" for the click guard: without
    // it, clicking the header or the toolbar falls through to the Status
    // branch and destroys the commit inspection.
    let commit_meta_area = layout.detail_area;
    // The message block's own scrollbar, over its right border.
    if let Some(track) = scrollbar::Track::vertical(
        layout.message_area,
        commit_message_total,
        commit_message_scroll,
    ) {
        scrollbar::render(frame, track, true, false);
    }

    // `Clear` first: `List` (like `Block` and `Paragraph`) only *styles* the
    // cells it draws no item into and never touches their symbols. On a
    // commit with fewer files than the pane is tall, the rows below the last
    // item would keep whatever was drawn there last (a tooltip or menu that
    // briefly covered the pane leaves border fragments and tinted patches).
    // Same fix as `PreferencesState::render_body`.
    frame.render_clipped(Clear, layout.files_inner);
    let commit_files_visible_rows = layout.files_inner.height.max(1) as usize;
    // Same throwaway-clone render as the
    // Log's, so a wheel-scrolled offset survives this frame
    // instead of snapping back to the selection.
    let mut commit_file_render_state = list_render_state(
        commit_file_state,
        commit_files_follow,
        commit_files_visible_rows,
    );
    crate::hscroll::render_list(
        frame,
        file_items,
        Style::default()
            .bg(th().ui.selection)
            .add_modifier(Modifier::BOLD),
        layout.files_inner,
        &mut commit_file_render_state,
        crate::hscroll::HPane::CommitFiles,
        files_list_body(layout.files_area),
        false,
    );
    *commit_file_state.offset_mut() = commit_file_render_state.offset();
    // The file row under the pointer lights across its whole width, like a
    // Refs row: the arrow and a hover highlight (not the placeholder lines).
    if let Some((_, band)) = crate::refs_sidebar::hovered_inner_row(
        layout.files_inner,
        commit_file_render_state.offset(),
        if commit_paths.is_empty() {
            0
        } else {
            commit_files_len
        },
    ) {
        crate::refs_sidebar::paint_row_hover(frame.buffer_mut(), band);
    }
    // A scrollbar over the
    // list's own bottom-anchored bordered rect, rendering-only
    // (see the Log's own site for why).
    if let Some(track) = scrollbar::Track::vertical(
        files_list_body(layout.files_area),
        commit_files_len,
        commit_file_state.offset(),
    ) {
        scrollbar::render(frame, track, true, false);
    }
    crate::hscroll::paint_bar(frame, crate::hscroll::HPane::CommitFiles, frame_hits, None);
    crate::hscroll::paint_bar(frame, crate::hscroll::HPane::CommitInfo, frame_hits, None);
    *ctx.commit_detail_status_height = commit_detail_status_height;
    *ctx.commit_files_area = commit_files_area;
    *ctx.commit_files_len = commit_files_len;
    *ctx.commit_files_visible_rows = commit_files_visible_rows;
    *ctx.commit_message_area = commit_message_area;
    *ctx.commit_message_for = commit_message_for;
    *ctx.commit_message_rows_now = commit_message_rows_now;
    *ctx.commit_message_scroll = commit_message_scroll;
    *ctx.commit_message_total = commit_message_total;
    *ctx.commit_meta_area = commit_meta_area;
}

/// Lights `rect` under the pointer: the hover background on the cells that
/// have none of their own (a link's own hover style stays).
pub(super) fn light(buf: &mut ratatui::buffer::Buffer, rect: Rect) {
    let rect = rect.intersection(buf.area);
    for y in rect.y..rect.bottom() {
        for x in rect.x..rect.right() {
            let cell = &mut buf[(x, y)];
            if cell.bg == ratatui::style::Color::Reset {
                cell.set_bg(th().ui.hover);
            }
        }
    }
}
