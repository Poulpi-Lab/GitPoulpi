//! The central block of the draw closure.

use super::super::*;
use crate::history_view::HistoryView;

pub(in crate::event_loop) struct CentralDrawCtx<'a> {
    pub(in crate::event_loop) central_view: CentralView,
    pub(in crate::event_loop) log_area: Rect,
    pub(in crate::event_loop) hovered_diff_row: Option<usize>,
    pub(in crate::event_loop) hovered_target: &'a Option<HitTarget>,
    pub(in crate::event_loop) commit_files: &'a Option<crate::daemon_link::CommitFilesOutcome>,
    pub(in crate::event_loop) working_diff: &'a Option<crate::daemon_link::WorkingDiffOutcome>,
    pub(in crate::event_loop) lfs: &'a crate::lfs_ui::LfsUi,
    pub(in crate::event_loop) frame_hits: &'a mut Vec<(Rect, HitTarget)>,
    pub(in crate::event_loop) overflow_slots: &'a mut Vec<(OverflowBar, Rect, Vec<OverflowItem>)>,
    pub(in crate::event_loop) diff_bar_area: &'a mut Rect,
    pub(in crate::event_loop) diff_body_area: &'a mut Rect,
    pub(in crate::event_loop) diff_longest_line: &'a mut usize,
    pub(in crate::event_loop) diff_rendered_len: &'a mut usize,
    pub(in crate::event_loop) diff_hunk_starts: &'a mut Vec<usize>,
    pub(in crate::event_loop) diff_rows: &'a mut std::rc::Rc<[diff_view::DiffRow]>,
    pub(in crate::event_loop) diff_render: &'a mut Option<std::rc::Rc<diff_view::DiffRender>>,
    pub(in crate::event_loop) diff_content_request: &'a mut Option<FileInsightRequest>,
    pub(in crate::event_loop) file_insight_area: &'a mut Rect,
    pub(in crate::event_loop) file_insight: &'a mut Option<FileInsight>,
    pub(in crate::event_loop) history: &'a mut Option<crate::history_view::HistoryView>,
    pub(in crate::event_loop) avatar_fallback: gitpoulpi_config::AvatarFallback,
    pub(in crate::event_loop) diff_view_state: &'a mut Option<DiffViewState>,
    pub(in crate::event_loop) editor: &'a mut Option<crate::editor::Editor>,
    pub(in crate::event_loop) merge: &'a mut Option<crate::merge_view::MergeView>,
    pub(in crate::event_loop) conflicts: &'a [gitpoulpi_core::domain::ConflictEntry],
    pub(in crate::event_loop) diff_minimap: &'a mut crate::minimap::Minimap,
    pub(in crate::event_loop) diff_content: &'a mut crate::diff_content::DiffContent,
    pub(in crate::event_loop) avatar_placement: gitpoulpi_config::AvatarPlacement,
    pub(in crate::event_loop) avatar_capability: crate::terminal_image::Capability,
    pub(in crate::event_loop) avatar_live: &'a avatar::LiveSettings,
    pub(in crate::event_loop) avatar_service: &'a mut avatar::AvatarService,
    pub(in crate::event_loop) log_avatar_images: &'a mut avatar::LogAvatarImages,
    pub(in crate::event_loop) commit_col: &'a mut crate::commit_column::CommitColumn,
    /// The Log's Changes numbers (the column's Changes part reads them).
    pub(in crate::event_loop) commit_changes:
        &'a std::collections::HashMap<Oid, gitpoulpi_core::domain::CommitStat>,
    /// The Log's layout: the commit column draws its rows from it.
    pub(in crate::event_loop) layout: &'a graph::LogLayout,
    pub(in crate::event_loop) selected_commit: Option<Oid>,
    /// More commits exist beyond the loaded ones (the column says so).
    pub(in crate::event_loop) more_commits: bool,
    pub(in crate::event_loop) node_set: graph::NodeSet,
    /// The commit column's own Graph picture (the Log's is `graph_image`):
    /// both are never on screen together, but their state is kept apart.
    pub(in crate::event_loop) commit_graph_image: &'a mut graph::GraphImage,
    pub(in crate::event_loop) ui_lane_background: graph::LaneBackground,
    pub(in crate::event_loop) ui_tint_strength: gitpoulpi_config::GraphRowTintStrength,
}

pub(in crate::event_loop) fn draw_central(frame: &mut Frame, ctx: &mut CentralDrawCtx<'_>) {
    draw_central_in(frame, ctx);
}

fn draw_central_in(frame: &mut Frame, ctx: &mut CentralDrawCtx<'_>) {
    let central_view = ctx.central_view;
    let mut log_area = ctx.log_area;
    let hovered_diff_row = ctx.hovered_diff_row;
    let hovered_target = ctx.hovered_target;
    let commit_files = ctx.commit_files;
    let working_diff = ctx.working_diff;
    let lfs = ctx.lfs;
    let frame_hits = &mut *ctx.frame_hits;
    let overflow_slots = &mut *ctx.overflow_slots;
    let diff_bar_area = &mut *ctx.diff_bar_area;
    let diff_body_area = &mut *ctx.diff_body_area;
    let diff_longest_line = &mut *ctx.diff_longest_line;
    let diff_rendered_len = &mut *ctx.diff_rendered_len;
    let diff_hunk_starts = &mut *ctx.diff_hunk_starts;
    let diff_rows = &mut *ctx.diff_rows;
    let diff_render = &mut *ctx.diff_render;
    let diff_content_request = &mut *ctx.diff_content_request;
    let file_insight_area = &mut *ctx.file_insight_area;
    let file_insight = &mut *ctx.file_insight;
    let diff_view_state = &mut *ctx.diff_view_state;
    let editor = &mut *ctx.editor;
    let merge = &mut *ctx.merge;
    let conflicts = ctx.conflicts;
    let diff_minimap = &mut *ctx.diff_minimap;
    let diff_content = &mut *ctx.diff_content;
    let avatar_service = &mut *ctx.avatar_service;
    let log_avatar_images = &mut *ctx.log_avatar_images;
    // An image diff the previous frame did not draw is forgotten here.
    crate::image_view::split::begin_frame();
    crate::open_with::view::begin_frame();
    crate::open_with::view::set_hovered(hovered_target.as_ref());
    // The File History split: the list takes the left of the column and the
    // diff below is drawn into what is left.
    // The right side is the Diff view, or (File / Blame mode) the File View
    // panel of the window's `file_insight`, both below the mode control.
    let mut history_tab = false;
    // The view's bar row is above the column and the list, and spans the
    // central column's whole width: `bar_extend` is how far it reaches left
    // of the area the view's body is drawn in.
    let bar_rows: u16 = u16::from(matches!(
        central_view,
        CentralView::Diff | CentralView::File
    ));
    let mut bar_extend: u16 = 0;
    if let Some(history) = ctx.history.as_mut() {
        let shown = match central_view {
            CentralView::Diff => true,
            CentralView::File => file_insight
                .as_ref()
                .is_some_and(|f| history.owns_insight(&f.request)),
            _ => false,
        };
        if shown && diff_view_state.as_ref().is_some_and(HistoryView::owns) {
            history_tab = true;
            // The bar row spans the whole central column whatever the list
            // does: the list starts below it, so no button moves when it
            // opens, closes or changes width.
            let top = bar_rows;
            let below = Rect::new(
                log_area.x,
                log_area.y + top,
                log_area.width,
                log_area.height.saturating_sub(top),
            );
            history.column = below;
            let rest = if history.hidden {
                // Hidden with its cross: a strip brings it back by mouse
                // (Alt+C and the palette's Toggle are the keyboard twins).
                let cols = crate::commit_column::STRIP_COLS.min(below.width);
                let strip = Rect::new(below.x, below.y, cols, below.height);
                let hot = *hovered_target
                    == Some(HitTarget::History(crate::history_view::HistoryHit::Reopen));
                frame_hits.extend(crate::history_view::render_strip(
                    frame, strip, history, hot,
                ));
                Rect::new(below.x + cols, below.y, below.width - cols, below.height)
            } else {
                let (list, rest) = history.split(below);
                let paint = crate::history_view::ListPaint {
                    fallback: ctx.avatar_fallback,
                    mode: avatar::card_avatar_mode(ctx.avatar_placement, ctx.avatar_capability),
                    hovered_row: match hovered_target {
                        Some(HitTarget::History(crate::history_view::HistoryHit::Row(i))) => {
                            Some(*i)
                        }
                        _ => None,
                    },
                    close_hot: matches!(
                        hovered_target,
                        Some(HitTarget::History(crate::history_view::HistoryHit::Close))
                    ),
                    divider_hot: history.dragging_split
                        || matches!(
                            hovered_target,
                            Some(HitTarget::History(crate::history_view::HistoryHit::Divider))
                        ),
                    now: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map_or(0, |d| d.as_secs() as i64),
                };
                frame_hits.extend(crate::history_view::render_list(
                    frame,
                    list,
                    history,
                    &paint,
                    Some((&mut *avatar_service, &mut *log_avatar_images)),
                ));
                rest
            };
            bar_extend = rest.x - log_area.x;
            log_area = Rect::new(rest.x, log_area.y, rest.width, log_area.height);
            // Nothing touches the file from here: no diff to wait for.
            if history.first_loaded && history.entries.is_empty() {
                *ctx.diff_bar_area = Rect::default();
                *ctx.diff_body_area = Rect::default();
                *ctx.diff_rendered_len = 0;
                *ctx.diff_rows = std::rc::Rc::from(Vec::new());
                *ctx.diff_render = None;
                return;
            }
        }
    }
    // The commit column, cut out of the central column's left: only beside a
    // view of one commit's file, never in the File History (its own list).
    ctx.commit_col.hide();
    let below = Rect::new(
        log_area.x,
        log_area.y + bar_rows,
        log_area.width,
        log_area.height.saturating_sub(bar_rows),
    );
    ctx.commit_col.column = below;
    ctx.commit_col.view = log_area;
    if !history_tab
        && crate::commit_column::viewed_commit(
            central_view,
            diff_view_state.as_ref(),
            file_insight.as_ref(),
        )
        .is_some()
    {
        if ctx.commit_col.wanted(frame.area().width) {
            if let Some((list, rest)) = ctx.commit_col.split(below) {
                let paint = crate::commit_column::ColumnPaint {
                    layout: ctx.layout,
                    selected: ctx.selected_commit,
                    hovered: hovered_target.clone(),
                    focused: ctx.commit_col.keys,
                    more: ctx.more_commits,
                    node_set: ctx.node_set,
                    changes: ctx.commit_changes,
                };
                // The Graph part, as pixels where the Log's is
                // (`commit_column::picture`): the same settings.
                let mut picture = crate::commit_column::ColumnPicture {
                    image: &mut *ctx.commit_graph_image,
                    avatars: avatar::in_graph(ctx.avatar_placement, ctx.avatar_capability),
                    service: &mut *avatar_service,
                    lane_background: ctx.ui_lane_background,
                    tint_strength: ctx.ui_tint_strength,
                };
                let hits = crate::commit_column::render(
                    frame,
                    list,
                    &mut *ctx.commit_col,
                    &paint,
                    Some(&mut picture),
                );
                frame_hits.extend(hits);
                // The view keeps the bar row above the column (and its
                // whole width): only the body is cut.
                bar_extend = rest.x - log_area.x;
                log_area = Rect::new(rest.x, log_area.y, rest.width, log_area.height);
                ctx.commit_col.view = log_area;
            }
        } else if ctx.commit_col.split(below).is_some() {
            // Hidden, and it would fit: a strip brings it back by mouse
            // (Alt+C and the palette's Toggle are its keyboard twins).
            let cols = crate::commit_column::STRIP_COLS.min(below.width);
            let strip = Rect::new(below.x, below.y, cols, below.height);
            let rest = Rect::new(below.x + cols, below.y, below.width - cols, below.height);
            let hot = *hovered_target
                == Some(HitTarget::CommitColumn(
                    crate::commit_column::ColumnHit::Reopen,
                ));
            let hits = crate::commit_column::render_strip(frame, strip, &mut *ctx.commit_col, hot);
            frame_hits.extend(hits);
            bar_extend = rest.x - log_area.x;
            log_area = Rect::new(rest.x, log_area.y, rest.width, log_area.height);
            ctx.commit_col.view = log_area;
        }
    }
    // What the bar's column control names (the bar is drawn below): what is
    // shown is the mode, and the mode is what comes back when it is hidden.
    if history_tab && ctx.history.as_ref().is_some_and(|h| !h.hidden) {
        ctx.commit_col.mode = crate::commit_column::ColumnMode::File;
    } else if ctx.commit_col.area.width > 0 {
        ctx.commit_col.mode = crate::commit_column::ColumnMode::All;
    }
    crate::commit_column::note_mode(ctx.commit_col.mode);
    crate::commit_column::note_painted(if history_tab {
        if ctx.history.as_ref().is_some_and(|h| h.hidden) {
            crate::commit_column::ColumnState::Off
        } else {
            crate::commit_column::ColumnState::File
        }
    } else if ctx.commit_col.area.width > 0 {
        crate::commit_column::ColumnState::Graph
    } else {
        crate::commit_column::ColumnState::Off
    });
    if central_view == CentralView::Log {
        // Painted by `draw_log`.
    } else if let (CentralView::Merge, Some(view)) = (central_view, merge.as_mut()) {
        // The merge conflict editor; every hit rect comes from its paint walk.
        let hovered = match &hovered_target {
            Some(HitTarget::Merge(h)) => Some(*h),
            _ => None,
        };
        for (rect, hit) in view.render(frame, log_area, hovered) {
            frame_hits.push((rect, HitTarget::Merge(hit)));
        }
    } else if let (CentralView::Editor, Some(ed)) = (central_view, editor.as_mut()) {
        // The editor, over the File View or diff it was opened from.
        let hovered = match &hovered_target {
            Some(HitTarget::Editor(h)) => Some(*h),
            _ => None,
        };
        for (rect, hit) in ed.render(frame, log_area, hovered) {
            frame_hits.push((rect, HitTarget::Editor(hit)));
        }
    } else if let (CentralView::File, Some(insight)) = (central_view, file_insight.as_mut()) {
        // File View / Blame / History in the central column, in the Diff view's place, with its bar.
        let editable = file_view_editable(insight);
        crate::image_view::pane::set_hovered(hovered_target.as_ref());
        let drawn = render_file_panel_in(
            frame,
            log_area,
            insight,
            diff_view_state.is_some(),
            editable,
            hover(hovered_target).diff_button(),
            *hovered_target == Some(HitTarget::BarOverflow(OverflowBar::DiffBar)),
            hover(hovered_target).close_diff(),
            history_tab,
            bar_extend,
        );
        let bar = drawn.bar_area;
        *diff_bar_area = bar;
        *file_insight_area = drawn.body;
        for (dx, w, button) in &drawn.bar.buttons {
            frame_hits.push((
                Rect::new(bar.x + dx, bar.y, *w, 1),
                HitTarget::DiffBarButton(*button),
            ));
        }
        if let Some((dx, w)) = drawn.bar.overflow {
            let rect = Rect::new(bar.x + dx, bar.y, w, 1);
            frame_hits.push((rect, HitTarget::BarOverflow(OverflowBar::DiffBar)));
            overflow_slots.push((
                OverflowBar::DiffBar,
                rect,
                drawn
                    .bar
                    .hidden
                    .iter()
                    .map(|b| OverflowItem::Diff(*b))
                    .collect(),
            ));
        }
        if let Some(rect) = drawn.close {
            frame_hits.push((rect, HitTarget::CloseDiff));
        }

        frame_hits.extend(drawn.image_hits.iter().cloned());

        // The Blame view paints itself (it needs the avatars); its rows and
        // border are ordinary hit targets, its card / menu go first.
        if let Some(inner) = drawn.blame_inner {
            let hovered = match hovered_target {
                Some(HitTarget::Blame(h)) => Some(*h),
                _ => None,
            };
            let keys = crate::keymap::current();
            let mut env = crate::blame_view::BlameEnv {
                hovered,
                now: Instant::now(),
                now_secs: crate::blame_view::now_secs(),
                mode: avatar::card_avatar_mode(ctx.avatar_placement, ctx.avatar_capability),
                fallback: ctx.avatar_live.fallback,
                keys: &keys,
                pictures: Some((&mut *avatar_service, &mut *log_avatar_images)),
            };
            let out = crate::blame_view::render(frame.buffer_mut(), inner, insight, &mut env);
            frame_hits.extend(out.hits.into_iter().map(|(r, h)| (r, HitTarget::Blame(h))));
            frame_hits.splice(
                0..0,
                out.popup_hits
                    .into_iter()
                    .map(|(r, h)| (r, HitTarget::Blame(h))),
            );
            if let Some(rect) = drawn.blame_minimap {
                crate::blame_view::paint_minimap(frame, rect, insight);
            }
        }

        // Scrollbars, same border-overlay convention as the diff view's (`scrollbar.rs`'s
        // module doc); ordinary `HitTarget`s like every other diff-bar control. An image
        // has no text rows (`len` 0), so only the image pane's own bars are drawn.
        if let Some(track) = scrollbar::Track::vertical(drawn.body, insight.len(), insight.offset) {
            let hovered =
                *hovered_target == Some(HitTarget::ScrollbarThumb(ScrollBar::FileInsightVertical));
            scrollbar::render(frame, track, true, hovered);
            let thumb = Rect::new(
                track.rect.x,
                track.rect.y + track.thumb_start,
                track.rect.width,
                track.thumb_len.max(1),
            );
            frame_hits.push((
                thumb,
                HitTarget::ScrollbarThumb(ScrollBar::FileInsightVertical),
            ));
        }
        if let Some(track) =
            scrollbar::Track::horizontal(drawn.body, insight.longest_line(), insight.h_offset)
        {
            let hovered = *hovered_target
                == Some(HitTarget::ScrollbarThumb(ScrollBar::FileInsightHorizontal));
            scrollbar::render(frame, track, false, hovered);
            let thumb = Rect::new(
                track.rect.x + track.thumb_start,
                track.rect.y,
                track.thumb_len.max(1),
                track.rect.height,
            );
            frame_hits.push((
                thumb,
                HitTarget::ScrollbarThumb(ScrollBar::FileInsightHorizontal),
            ));
        }
    } else if let Some(diff_state) = diff_view_state.as_ref().filter(|d| {
        !matches!(d.source, DiffSource::Commit | DiffSource::Compare)
            && conflicts.iter().any(|c| c.path == d.path)
    }) {
        // A conflicted file has no diff worth showing ("not possible"): a
        // card offers the merge editor instead, its button the same hit
        // target as the Diff bar's. Every rect comes from this walk.
        let title = format!(" {} ", diff_state.path.display());
        let close_hovered = hover(hovered_target).close_diff();
        let block = Block::default()
            .title(title)
            .border_type(BorderType::Rounded)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(th().ui.accent));
        let inner = block.inner(log_area);
        frame.render_clipped(block, log_area);
        if let Some(rect) =
            crate::chrome::close_cross::paint(frame.buffer_mut(), log_area, close_hovered)
        {
            frame_hits.push((rect, HitTarget::CloseDiff));
        }
        // Nothing of the old diff stays clickable or scrollable.
        *diff_bar_area = Rect::default();
        *diff_body_area = Rect::default();
        *diff_longest_line = 0;
        *diff_rendered_len = 0;
        diff_hunk_starts.clear();
        *diff_rows = std::rc::Rc::from(Vec::new());
        *diff_render = None;
        let title = gitpoulpi_i18n::msg::merge::CONFLICT_CARD_TITLE.get();
        let hint = gitpoulpi_i18n::msg::merge::CONFLICT_CARD_HINT.get();
        let label = gitpoulpi_i18n::msg::merge::BUTTON_MERGE_EDITOR.get();
        let button_w = crate::dialog_kit::button_width(label);
        if inner.height >= 4 && inner.width > 0 {
            let top = inner.y + inner.height.saturating_sub(4) / 2;
            let centered = |text: &str, y: u16, style: Style, frame: &mut Frame| {
                frame.render_clipped(
                    Paragraph::new(Line::from(Span::styled(text.to_string(), style)))
                        .alignment(ratatui::layout::Alignment::Center),
                    Rect::new(inner.x, y, inner.width, 1),
                );
            };
            centered(
                title,
                top,
                Style::default().add_modifier(Modifier::BOLD),
                frame,
            );
            centered(hint, top + 1, Style::default(), frame);
            let x = inner.x + inner.width.saturating_sub(button_w) / 2;
            let lit = *hovered_target == Some(HitTarget::MergeEditorButton);
            let rect = crate::dialog_kit::button(
                frame.buffer_mut(),
                x,
                top + 3,
                label,
                crate::dialog_kit::ButtonState::Primary {
                    hovered: lit,
                    focused: false,
                },
            );
            frame_hits.push((rect, HitTarget::MergeEditorButton));
        }
    } else if let Some(diff_state) = diff_view_state.as_mut() {
        // The Diff view takes over the whole central column: one row for the button bar,
        // the rest for the diff. The path and the close affordance sit in the body block's title.
        let [bar, body] =
            Layout::vertical([Constraint::Length(1), Constraint::Min(0)]).areas(log_area);
        // The bar reaches over the column and the File History's list: it
        // does not move when they open, close or change width.
        let bar = Rect::new(bar.x - bar_extend, bar.y, bar.width + bar_extend, 1);
        *diff_bar_area = bar;
        *diff_body_area = body;

        // A conflicted file's diff offers the merge editor, right-aligned: its
        // room is taken off the bar's width, so the other buttons give way
        // (behind the `⋯`) before it does. The key and the palette entry
        // always work.
        let conflicted = conflicts.iter().any(|c| c.path == diff_state.path);
        let merge_label = gitpoulpi_i18n::msg::merge::BUTTON_MERGE_EDITOR.get();
        let merge_w = crate::dialog_kit::button_width(merge_label);
        let reserve = if conflicted && bar.width > merge_w * 2 {
            merge_w + 1
        } else {
            0
        };
        let diff_bar = diff_view::diff_bar(
            diff_state,
            hover(hovered_target).diff_button(),
            *hovered_target == Some(HitTarget::BarOverflow(OverflowBar::DiffBar)),
            bar.width - reserve,
        );
        for (dx, w, button) in diff_bar.buttons {
            let rect = Rect::new(bar.x + dx, bar.y, w, 1);
            if button == diff_view::DiffBarButton::Encoding {
                crate::diff_encoding_menu::note_button(rect);
            }
            frame_hits.push((rect, HitTarget::DiffBarButton(button)));
        }
        // The buttons that did not fit, behind a `⋯` whose menu re-dispatches each as its own click.
        if let Some((dx, w)) = diff_bar.overflow {
            let rect = Rect::new(bar.x + dx, bar.y, w, 1);
            if diff_bar
                .hidden
                .contains(&diff_view::DiffBarButton::Encoding)
            {
                // Hidden behind the `⋯`: the dropdown hangs from it.
                crate::diff_encoding_menu::note_button(rect);
            }
            frame_hits.push((rect, HitTarget::BarOverflow(OverflowBar::DiffBar)));
            overflow_slots.push((
                OverflowBar::DiffBar,
                rect,
                diff_bar
                    .hidden
                    .iter()
                    .map(|b| OverflowItem::Diff(*b))
                    .collect(),
            ));
        }
        frame.render_clipped(
            Paragraph::new(diff_bar.line),
            Rect::new(bar.x, bar.y, bar.width - reserve, 1),
        );
        if reserve > 0 {
            let lit = *hovered_target == Some(HitTarget::MergeEditorButton);
            let rect = crate::dialog_kit::button(
                frame.buffer_mut(),
                bar.right() - merge_w,
                bar.y,
                merge_label,
                crate::dialog_kit::ButtonState::Primary {
                    hovered: lit,
                    focused: false,
                },
            );
            frame_hits.push((rect, HitTarget::MergeEditorButton));
        }

        // The file is looked up by index into the very same `Vec<FileDiff>` the changed-files
        // list is built from: no second daemon round trip.
        //
        // The `outcome.commit == diff_state.commit` check is not optional: checking a branch
        // out from the sidebar (still reachable with the diff open) moves the Log selection,
        // which refetches `commit_files` for a *different* commit, and indexing by position
        // alone would swap in file #i of that other commit under the original title.
        //
        // The two working-tree sources are looked up by PATH instead: their file list is a
        // diff of what is currently uncommitted, and it changes shape the moment this view's
        // Stage Hunk button is pressed (a fully staged file leaves the Unstaged half).
        //
        // A commit's list arrives without hunks and the open file's hunks are fetched alone
        // (`working_diff`, keyed by target and path). Until they land the title reads
        // "(loading...)", never an empty diff that reads as "no changes". A commit whose
        // backend sent the whole diff with the list (`hunks_pending` false) needs no second
        // round trip.
        let fetched = |target: &gitpoulpi_core::domain::DiffTarget| {
            working_diff.as_ref().and_then(|outcome| {
                (&outcome.target == target
                    && outcome.path.as_deref() == Some(diff_state.path.as_path()))
                .then_some(outcome)?
                .files
                .as_ref()
                .ok()?
                .iter()
                .find(|f| f.path == diff_state.path)
            })
        };
        let open_file = match diff_state.diff_target() {
            None => fetched(&gitpoulpi_core::domain::DiffTarget::CommitVsParent(
                diff_state.commit,
            ))
            .or_else(|| {
                commit_files.as_ref().and_then(|outcome| {
                    (outcome.commit == diff_state.commit && !outcome.hunks_pending)
                        .then_some(outcome)?
                        .files
                        .as_ref()
                        .ok()?
                        .get(diff_state.file_index)
                })
            }),
            Some(target) => fetched(&target),
        };
        // A chosen encoding swaps in the diff re-read from the two sides' bytes
        // (`diff_recode`); until it lands, or when it cannot be read, the diff as the
        // backend decoded it stays, with a word in the title. Staging is off meanwhile.
        let recoded = match (diff_state.encoding, open_file) {
            (Some(charset), Some(f)) => Some(crate::diff_recode::resolve(
                &crate::diff_recode::Want::of_state(diff_state, charset),
                f,
            )),
            _ => None,
        };
        let mut enc_note = String::new();
        let mut forced_lines = None;
        let open_file = match &recoded {
            Some(crate::diff_recode::Status::Ready(r)) => {
                forced_lines = Some(std::sync::Arc::clone(&r.new_lines));
                Some(&r.diff)
            }
            Some(crate::diff_recode::Status::Loading { since })
                if since.elapsed() >= crate::diff_recode::FEEDBACK_AFTER =>
            {
                enc_note = gitpoulpi_i18n::msg::diff_view::enc_reading(
                    diff_state.encoding.map_or("", |c| c.name()),
                );
                open_file
            }
            Some(crate::diff_recode::Status::Failed(why)) => {
                enc_note = gitpoulpi_i18n::msg::diff_view::enc_failed(
                    diff_state.encoding.map_or("", |c| c.name()),
                    why,
                );
                open_file
            }
            _ => open_file,
        };
        diff_content.set_forced(forced_lines);
        let title = match diff_state.source {
            DiffSource::Commit | DiffSource::Compare => match open_file {
                Some(f) => format!(" {} ", f.path.display()),
                None => format!(" {} ", gitpoulpi_i18n::msg::central::LOADING.get()),
            },
            // Which half this is decides whether the per-hunk button stages or unstages, so
            // it is named in the title. The path comes from the clicked row, so it never blanks
            // out while the diff is in flight.
            DiffSource::Unstaged => {
                format!(
                    " {} ",
                    gitpoulpi_i18n::msg::central::diff_unstaged(diff_state.path.display())
                )
            }
            DiffSource::Staged => {
                format!(
                    " {} ",
                    gitpoulpi_i18n::msg::central::diff_staged(diff_state.path.display())
                )
            }
        };
        let title = if enc_note.is_empty() {
            title
        } else {
            format!("{}- {enc_note} ", title)
        };
        // The close button is a plain glyph with no fill at rest, like the gear and the
        // app close cross, so it uses `plain_button_style` (same idle color, strong filled
        // hover) and not `hover_style`'s subtle lift.
        let close_hovered = hover(hovered_target).close_diff();
        // The minimap (`ui.minimap`) takes its columns as right padding, so the text, the
        // wrap width and the horizontal scroll all stop where it starts.
        let minimap_pad = minimap::padding_for(body, live_settings::minimap());
        let body_block = Block::default()
            .title(title)
            .border_type(BorderType::Rounded)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(th().ui.accent))
            .padding(ratatui::widgets::Padding::right(minimap_pad));
        let inner = body_block.inner(body);
        frame.render_clipped(body_block, body);
        // The close affordance's rect is *read back* from the rendered border row rather
        // than recomputed from the title's width and alignment: a hit target derived from a
        // second guess at ratatui's title layout drifts silently ("register hit targets
        // inner-first" in `where-we-are.md`).
        if let Some(rect) =
            crate::chrome::close_cross::paint(frame.buffer_mut(), body, close_hovered)
        {
            frame_hits.push((rect, HitTarget::CloseDiff));
        }

        // A picture (a binary with an image extension): before and after as
        // pictures, split or inline. `None` when the bytes are no picture,
        // which leaves today's notice below.
        if let Some(f) = open_file.filter(|f| crate::image_view::split::wants(diff_state, f)) {
            crate::image_view::split::set_hovered(hovered_target.as_ref());
            if let Some(hits) = crate::image_view::split::render(frame, inner, diff_state, f) {
                frame_hits.extend(hits);
                // No text rows: nothing of the text diff's maps may point at stale rows.
                *diff_longest_line = 0;
                *diff_rendered_len = 0;
                diff_hunk_starts.clear();
                *diff_rows = std::rc::Rc::from(Vec::new());
                *diff_render = None;
                diff_state.ready = true;
                return;
            }
        }

        // A binary file the OS has apps for: the "Open in" row over the old
        // notice. `None` (no hunks to explain, no app, text) leaves today's view.
        if open_file.is_some_and(|f| {
            f.hunks.is_empty()
                && !diff_state.lfs_raw
                && gitpoulpi_core::domain::lfs_change(f).is_none()
        }) {
            if let Some(hits) = crate::open_with::view::render_diff(frame, inner, diff_state) {
                frame_hits.extend(hits);
                *diff_longest_line = 0;
                *diff_rendered_len = 0;
                diff_hunk_starts.clear();
                *diff_rows = std::rc::Rc::from(Vec::new());
                *diff_render = None;
                diff_state.ready = true;
                return;
            }
        }

        // Rendered (or taken from the cache) by `diff_content`, which also clamps `h_offset`
        // against this file first and asks for the whole file when the Hunks toggle is off.
        // `diff_longest_line` feeds the horizontal scrollbar's content length: the render
        // pass publishes what the mouse handlers need, since they run outside this closure.
        *diff_longest_line = open_file
            .map(|f| {
                diff_view::h_track_content(
                    diff_content.longest(f),
                    diff_state.mode,
                    inner.width,
                    body.width.saturating_sub(2),
                )
            })
            .unwrap_or(0);
        diff_state.ready = false;
        let rendered = match open_file {
            Some(f) => {
                // Whether the LFS object behind an LFS file's diff is in the local store (drives
                // the card's last line).
                diff_state.lfs_local = lfs.file(&diff_state.path).map(|file| file.local);
                *diff_content_request = diff_content.sync(diff_state, f);
                // The rows drawn below are the real diff once the whole-file content (if asked
                // for) has settled.
                diff_state.ready = diff_content.settled();
                diff_content.render(f, diff_state, inner.width)
            }
            None => std::rc::Rc::new(diff_view::DiffRender::notice(match diff_state.source {
                DiffSource::Commit | DiffSource::Compare => {
                    gitpoulpi_i18n::msg::central::WAITING_DIFF.get()
                }
                // Staging the last hunk of a file empties its half and the file drops out of that
                // diff altogether; say so rather than sit on "waiting".
                _ => gitpoulpi_i18n::msg::central::NOTHING_LEFT.get(),
            })),
        };
        *diff_rendered_len = rendered.lines.len();
        *diff_hunk_starts = rendered.hunk_starts.clone();
        // Clamped against the freshly rendered row count, so a mode/wrap/whitespace toggle
        // that shortens the diff never leaves the view scrolled past its end. The cursor
        // rides the same numbering.
        let max_scroll = diff_rendered_len.saturating_sub(inner.height.max(1) as usize);
        diff_state.scroll = diff_state.scroll.min(max_scroll);
        diff_state.cursor = diff_state.cursor.min(diff_rendered_len.saturating_sub(1));
        if let Some(anchor) = diff_state.selection_anchor {
            diff_state.selection_anchor = Some(anchor.min(diff_rendered_len.saturating_sub(1)));
        }

        // The visible window, with the per-hunk Stage/Unstage
        // button and the cursor/selection bands painted on it.
        let (visible, buttons) = diff_view::visible_window(
            &rendered,
            diff_state,
            inner,
            hovered_diff_row,
            match hovered_target {
                Some(HitTarget::DiffBodyButton(b)) => Some(*b),
                _ => None,
            },
        );
        push_diff_body_hits(
            &mut *frame_hits,
            inner,
            diff_state.scroll,
            visible.len(),
            buttons,
        );
        *diff_rows = std::rc::Rc::clone(&rendered.rows);
        frame.render_clipped(Paragraph::new(visible), inner);
        // The text selection, over the rows.
        diff_view::paint_text_selection(frame.buffer_mut(), inner, &rendered, diff_state);
        *diff_render = Some(std::rc::Rc::clone(&rendered));

        // Scrollbars, drawn over the block's own border (`scrollbar.rs`'s module doc) and
        // registered into `frame_hits` from the geometry just drawn: one walk, both the
        // paint and the hit rect.
        if let Some(track) = scrollbar::Track::vertical(body, *diff_rendered_len, diff_state.scroll)
        {
            let hovered =
                *hovered_target == Some(HitTarget::ScrollbarThumb(ScrollBar::DiffVertical));
            scrollbar::render(frame, track, true, hovered);
            let ticks = diff_view::ruler_marks(&rendered, diff_state);
            if !ticks.is_empty() {
                let cells = scroll_marks::project(track.inner_len(), *diff_rendered_len, ticks);
                scroll_marks::paint(frame, track, &cells);
            }
            let thumb = Rect::new(
                track.rect.x,
                track.rect.y + track.thumb_start,
                track.rect.width,
                track.thumb_len.max(1),
            );
            frame_hits.push((thumb, HitTarget::ScrollbarThumb(ScrollBar::DiffVertical)));
            frame_hits.push((
                track.rect,
                HitTarget::ScrollbarTrack(ScrollBar::DiffVertical),
            ));
        }
        if minimap_pad > 0 {
            if let Some(rect) = minimap::rect_in(body) {
                let key = std::rc::Rc::as_ptr(&rendered) as *const () as usize as u64;
                diff_minimap.pin(rendered.clone());
                diff_minimap.draw(
                    frame,
                    rect,
                    key,
                    *diff_rendered_len,
                    (diff_state.scroll, inner.height as usize),
                    |i| diff_view::minimap_row(&rendered, i),
                );
                frame_hits.push((rect, HitTarget::ScrollbarTrack(ScrollBar::DiffMinimap)));
            }
        }
        if !diff_state.word_wrap {
            if let Some(track) =
                scrollbar::Track::horizontal(body, *diff_longest_line, diff_state.h_offset)
            {
                let hovered =
                    *hovered_target == Some(HitTarget::ScrollbarThumb(ScrollBar::DiffHorizontal));
                scrollbar::render(frame, track, false, hovered);
                let thumb = Rect::new(
                    track.rect.x + track.thumb_start,
                    track.rect.y,
                    track.thumb_len.max(1),
                    track.rect.height,
                );
                frame_hits.push((thumb, HitTarget::ScrollbarThumb(ScrollBar::DiffHorizontal)));
                frame_hits.push((
                    track.rect,
                    HitTarget::ScrollbarTrack(ScrollBar::DiffHorizontal),
                ));
            }
        }
    }
}
