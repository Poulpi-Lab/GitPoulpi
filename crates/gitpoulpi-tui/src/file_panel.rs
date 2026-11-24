//! The central File View.
//!
//! File View, Blame and File History (`FileInsight` in `file_view.rs`) take
//! the central column exactly as the Diff view does: the same one-row bar
//! on top -- File View / Diff View / Blame / History -- and a bordered
//! body whose `✕ Esc` goes back to the graph. `file_view.rs` still owns
//! the state and the rows; this module owns where they are drawn and what
//! the bar does, plus the two View all files rules that decide whether a
//! file opens as a diff or as a File View:
//!
//! - **36**: a file the commit did not change has no diff: View all files opens
//!   its content at that commit.
//! - **37**: View all files on the WIP row lists the whole working tree
//!   (HEAD's tree plus every path `status` reports), changed files
//!   marked; an unchanged file opens its bytes on disk.
//!
//! Kept out of `event_loop.rs` on purpose: everything here is a plain
//! function of its arguments, so it is unit-tested directly.

use super::*;
use gitpoulpi_i18n::msg::file_panel as msg;

use gitpoulpi_core::domain::{FileStatus, StatusEntry};

use crate::daemon_link::CommitFilesOutcome;
use crate::diff_view::DiffBarRender;

/// Keeps `central_view` in step with what is open. A `FileInsight` being
/// open *is* the File View, whichever of the half-dozen places opened it;
/// closing it falls back to the diff it was opened over, if any. An open
/// editor is on top of both, and closing it falls back
/// the same way to whichever of them is still open underneath.
pub(crate) fn sync_central_view(
    view: &mut CentralView,
    merge_open: bool,
    editor_open: bool,
    file_open: bool,
    diff_open: bool,
) {
    *view = match (*view, editor_open, file_open, diff_open) {
        // The merge editor is on top of everything, the editor included.
        _ if merge_open => CentralView::Merge,
        (_, true, _, _) => CentralView::Editor,
        (_, false, true, _) => CentralView::File,
        (CentralView::File | CentralView::Editor | CentralView::Merge, false, false, true) => {
            CentralView::Diff
        }
        (_, false, false, false) => CentralView::Log,
        (other, false, false, true) => other,
    };
}

/// Whether the File View's `Edit` button applies: a Content view - the working tree's, a working
/// diff's, or a commit's version (View all files on an unchanged file, or
/// a File View reached from a commit's diff). Edit always opens the
/// working-tree copy of the same path, never the commit's, so Blame and
/// History - which aren't "this file's bytes" - stay unofferable.
pub(crate) fn file_view_editable(insight: &FileInsight) -> bool {
    insight.request.kind == FileInsightKind::Content
}

/// Opens the editor on the working-tree copy of `path`,
/// or returns why it cannot. When neither view the editor sits over shows
/// this file, the working-tree File View of it is opened underneath, so
/// there is always something to come back to when the editor closes.
pub(crate) fn open_editor(
    root: &Path,
    path: &Path,
    head: Option<Oid>,
    link: &DaemonLink,
    editor: &mut Option<crate::editor::Editor>,
    file_insight: &mut Option<FileInsight>,
    diff_view_state: &Option<DiffViewState>,
) -> Result<(), String> {
    let opened = crate::editor::Editor::open(root, path)?;
    if editor_host_path(file_insight.as_ref(), diff_view_state.as_ref()) != Some(path) {
        let request = FileInsightRequest {
            path: path.to_path_buf(),
            at: head.unwrap_or(Oid([0u8; 20])),
            kind: FileInsightKind::Content,
            worktree: true,
            staged: false,
        };
        link.request_file_insight(request.clone());
        *file_insight = Some(FileInsight::loading(request));
    }
    *editor = Some(opened);
    Ok(())
}

/// The file the view under the editor shows: the File View's if one is
/// open (it is the one on top), else the diff's.
pub(crate) fn editor_host_path<'a>(
    file_insight: Option<&'a FileInsight>,
    diff: Option<&'a DiffViewState>,
) -> Option<&'a Path> {
    file_insight
        .map(|f| f.request.path.as_path())
        .or(diff.map(|d| d.path.as_path()))
}

/// Whether an open editor should give way: something opened another
/// file in the central column under it (a click in the Changed files
/// list, say). Only a clean editor does - unsaved work is never thrown
/// away by a click elsewhere; it stays on top until saved or discarded.
pub(crate) fn editor_should_yield(
    editor: &crate::editor::Editor,
    file_insight: Option<&FileInsight>,
    diff: Option<&DiffViewState>,
) -> bool {
    !editor.doc.is_dirty() && editor_host_path(file_insight, diff) != Some(editor.path.as_path())
}

/// The file Edit in Working Directory opens from the central view (the
/// bar's `Edit`, the `e` key, the palette): the File View's or the
/// diff's - the working-tree copy of that path either way, a commit's
/// diff or File View included (extended 2026-09-25). Only Blame/History
/// (not "this file's bytes") are refused here; a path missing on disk
/// (deleted, renamed away, a deletion in the commit) is refused by
/// `Editor::open` itself once `open_editor` tries it, so both refusals
/// go through the one message the status line shows.
pub(crate) fn edit_target(
    central_view: CentralView,
    file_insight: Option<&FileInsight>,
    diff: Option<&DiffViewState>,
) -> Result<PathBuf, &'static str> {
    match central_view {
        CentralView::File => match file_insight {
            Some(f) if file_view_editable(f) => Ok(f.request.path.clone()),
            Some(_) => Err(msg::EDIT_OVERLAY.get()),
            None => Err(msg::OPEN_FILE_FIRST.get()),
        },
        CentralView::Diff => match diff {
            Some(d) => Ok(d.path.clone()),
            None => Err(msg::OPEN_FILE_FIRST.get()),
        },
        CentralView::Editor | CentralView::Merge => Err(msg::EDITOR_OPEN.get()),
        CentralView::Log => Err(msg::OPEN_DIFF_OR_VIEW.get()),
    }
}

/// Applies what a key, click or paste in the editor asked for: the
/// status message, the refresh after a save (the daemon re-reads the
/// status, so the Changed files list, the WIP row and a diff under the
/// editor follow), and closing it.
pub(crate) fn apply_editor_outcome(
    out: crate::editor::EditorOutcome,
    editor: &mut Option<crate::editor::Editor>,
    file_insight: &mut Option<FileInsight>,
    link: &DaemonLink,
    last_message: &mut Option<String>,
    config_path: &Option<PathBuf>,
) {
    if let Some(message) = out.message {
        *last_message = Some(message);
    }
    if out.toggle_follow {
        *last_message = Some(crate::editor_prefs::toggle_follow_selection(config_path));
    }
    if out.toggle_minimap {
        *last_message = Some(crate::diff_actions::toggle_minimap(config_path));
    }
    if out.saved {
        link.request_refresh();
        if let Some(ed) = editor.as_ref() {
            reload_after_save(&ed.path, file_insight, link);
        }
    }
    if out.close {
        *editor = None;
    }
}

/// After a save: a working-tree File View of the saved file is re-read,
/// so going back to it shows what was written.
pub(crate) fn reload_after_save(
    saved: &Path,
    file_insight: &mut Option<FileInsight>,
    link: &DaemonLink,
) {
    let stale = file_insight.as_ref().is_some_and(|f| {
        f.request.worktree && f.request.kind == FileInsightKind::Content && f.request.path == saved
    });
    if stale {
        let request = file_insight
            .as_ref()
            .expect("checked above")
            .request
            .clone();
        link.request_file_insight(request.clone());
        *file_insight = Some(FileInsight::loading(request));
    }
}

/// The File View's bar, in drawing order. The Diff view's own bar shows
/// File View / Diff View / Blame / History too, so switching between the
/// four is the same four buttons wherever you are.
pub(crate) const FILE_BAR_BUTTONS: &[DiffBarButton] = &[
    DiffBarButton::FileView,
    DiffBarButton::DiffView,
    DiffBarButton::Blame,
    DiffBarButton::History,
    DiffBarButton::EditWorkingFile,
];

fn file_bar_priority(button: DiffBarButton) -> u8 {
    match button {
        DiffBarButton::FileView => 1,
        DiffBarButton::Blame => 2,
        DiffBarButton::EditWorkingFile => 2,
        DiffBarButton::History => 3,
        _ => 4,
    }
}

fn kind_button(kind: FileInsightKind) -> DiffBarButton {
    match kind {
        FileInsightKind::Content
        | FileInsightKind::DiffContent(_)
        | FileInsightKind::DiffRecoded { .. }
        | FileInsightKind::ImageBytes(_)
        | FileInsightKind::BinaryState => DiffBarButton::FileView,
        FileInsightKind::Blame => DiffBarButton::Blame,
        FileInsightKind::History | FileInsightKind::HistoryPage(_) => DiffBarButton::History,
    }
}

/// Lays out the File View's bar for `width` columns. `has_diff` grays
/// Diff View out when there is no diff to go back to (an unchanged file
/// opened from View all files, or a File History opened from a menu).
/// `editable` shows the `Edit` button - a Content
/// view, working tree's or a commit's (`file_view_editable`); never
/// Blame/History.
#[cfg(test)]
pub(crate) fn file_bar(
    kind: FileInsightKind,
    has_diff: bool,
    editable: bool,
    hovered: Option<DiffBarButton>,
    overflow_hovered: bool,
    width: u16,
) -> DiffBarRender {
    file_bar_active(
        kind_button(kind),
        has_diff,
        editable,
        hovered,
        overflow_hovered,
        width,
    )
}

/// [`file_bar`] with the lit tab named: the File History's right side lights
/// `History` (the view the user is in) whatever the pane shows.
pub(crate) fn file_bar_active(
    active: DiffBarButton,
    has_diff: bool,
    editable: bool,
    hovered: Option<DiffBarButton>,
    overflow_hovered: bool,
    width: u16,
) -> DiffBarRender {
    let items: Vec<BarItem<DiffBarButton>> = FILE_BAR_BUTTONS
        .iter()
        .filter(|b| editable || **b != DiffBarButton::EditWorkingFile)
        .map(|button| BarItem {
            key: *button,
            width: display_width(&format!(" {} ", button.bar_label())) as u16,
            gap_before: 1,
            priority: file_bar_priority(*button),
        })
        .collect();
    let fit = fit_bar(&items, width, 1);
    let mut spans: Vec<Span<'static>> = Vec::new();
    let mut buttons = Vec::new();
    let mut x: u16 = 0;
    for (bx, item) in &fit.visible {
        let button = item.key;
        let label = button.bar_label();
        if *bx > x {
            spans.push(Span::raw(" ".repeat((bx - x) as usize)));
        }
        if button == DiffBarButton::History {
            // The column's `File | All` selector: two hits, one control.
            let (segments, segment_hits) = crate::commit_column::selector(hovered);
            spans.extend(segments);
            for (dx, w, b) in segment_hits {
                buttons.push((*bx + dx, w, b));
            }
            x = bx + item.width;
            continue;
        }
        let enabled = button != DiffBarButton::DiffView || has_diff;
        let style = if !enabled {
            Style::default()
                .fg(th().ui.muted)
                .add_modifier(Modifier::DIM)
        } else if button == active {
            Style::default()
                .bg(th().ui.accent)
                .fg(th().ui.on_accent)
                .add_modifier(Modifier::BOLD)
        } else if hovered == Some(button) {
            Style::default()
                .bg(th().ui.button_hover)
                .fg(th().ui.on_button)
        } else {
            Style::default().bg(th().ui.button).fg(th().ui.on_button)
        };
        spans.push(Span::styled(format!(" {label} "), style));
        buttons.push((*bx, item.width, button));
        x = bx + item.width;
    }
    if let Some((ox, _)) = fit.overflow_button {
        if ox > x {
            spans.push(Span::raw(" ".repeat((ox - x) as usize)));
        }
        spans.push(Span::styled(
            OVERFLOW_LABEL,
            chip_overflow_style(overflow_hovered),
        ));
    }
    DiffBarRender {
        line: Line::from(spans),
        buttons,
        overflow: fit.overflow_button,
        hidden: fit.overflow,
    }
}

/// What drawing the File View produced: the bar (for its hit targets and
/// overflow menu), the bordered body (`file_insight_area` - the key and
/// mouse handlers count rows from inside its border), and the `✕ Esc`.
pub(crate) struct FilePanelRender {
    pub(crate) bar: DiffBarRender,
    pub(crate) bar_area: Rect,
    pub(crate) body: Rect,
    pub(crate) close: Option<Rect>,
    /// The image pane's header buttons, when it drew instead of the text.
    pub(crate) image_hits: Vec<(Rect, crate::chrome::HitTarget)>,
    /// The inner rect the Blame view draws itself in (the caller owns the
    /// avatars it needs); `None` for every other view.
    pub(crate) blame_inner: Option<Rect>,
    /// Where the Blame view's minimap goes (`ui.minimap`), when it is on and
    /// the pane is wide enough.
    pub(crate) blame_minimap: Option<Rect>,
}

/// Draws the File View over the whole central column `area`.
#[cfg(test)]
#[allow(clippy::too_many_arguments)]
pub(crate) fn render_file_panel(
    frame: &mut Frame,
    area: Rect,
    insight: &mut FileInsight,
    has_diff: bool,
    editable: bool,
    hovered: Option<DiffBarButton>,
    overflow_hovered: bool,
    close_hovered: bool,
) -> FilePanelRender {
    render_file_panel_in(
        frame,
        area,
        insight,
        has_diff,
        editable,
        hovered,
        overflow_hovered,
        close_hovered,
        false,
        0,
    )
}

/// [`render_file_panel`]; `in_history` when the panel is the File History's
/// right side (its bar then lights `History`, not the pane's own kind).
#[allow(clippy::too_many_arguments)]
pub(crate) fn render_file_panel_in(
    frame: &mut Frame,
    area: Rect,
    insight: &mut FileInsight,
    has_diff: bool,
    editable: bool,
    hovered: Option<DiffBarButton>,
    overflow_hovered: bool,
    close_hovered: bool,
    in_history: bool,
    bar_extend: u16,
) -> FilePanelRender {
    let [bar_area, body] =
        Layout::vertical([Constraint::Length(1), Constraint::Min(0)]).areas(area);
    // The bar reaches left over the commit column / the History's list.
    let bar_area = Rect::new(
        bar_area.x - bar_extend,
        bar_area.y,
        bar_area.width + bar_extend,
        1,
    );
    let bar = file_bar_active(
        if in_history {
            DiffBarButton::History
        } else {
            kind_button(insight.request.kind)
        },
        has_diff,
        editable,
        hovered,
        overflow_hovered,
        bar_area.width,
    );
    frame.render_clipped(Paragraph::new(bar.line.clone()), bar_area);

    // Same plain-glyph treatment as the gear/app-close corner buttons and
    // the Diff view's own `\u{2715} Esc` (`event_loop.rs`) -- `hover_style`
    // is for elements that already carry their own base style, and its
    // subtle lift read as barely-there next to those two.
    let hint = match insight.request.kind {
        FileInsightKind::Content
        | FileInsightKind::DiffContent(_)
        | FileInsightKind::DiffRecoded { .. }
        | FileInsightKind::ImageBytes(_)
        | FileInsightKind::BinaryState => "",
        FileInsightKind::Blame | FileInsightKind::History | FileInsightKind::HistoryPage(_) => {
            msg::GOTO_COMMIT.get()
        }
    };
    let title = format!(" {}{hint} ", insight.title());
    let mut block = Block::default().title(title.clone());
    // The Blame view's heat legend, when the title leaves room for it.
    let blame_shown = insight.blame_lines().is_some();
    if blame_shown
        && body.width as usize > display_width(&title) + crate::blame_view::legend_width() + 3 + 12
    {
        block = block.title_top(Line::from(crate::blame_view::legend()).centered());
    }
    // The Blame view's minimap takes its columns as right padding, so the
    // code and the annotations stop where it starts.
    let map_pad = if blame_shown {
        // Not in a narrow pane: the code needs those columns more.
        if body.width >= crate::blame_view::MIN_MAP_BODY {
            crate::minimap::padding_for(body, crate::live_settings::minimap())
        } else {
            0
        }
    } else {
        0
    };
    let block = block
        .border_type(BorderType::Rounded)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(th().ui.border_focused))
        .padding(ratatui::widgets::Padding::right(map_pad));
    let inner = block.inner(body);
    frame.render_clipped(block, body);
    let close = crate::chrome::close_cross::paint(frame.buffer_mut(), body, close_hovered);

    if crate::image_view::pane::shows(insight) {
        let painted = crate::image_view::pane::render(frame, body, inner);
        return FilePanelRender {
            bar,
            bar_area,
            body,
            close,
            image_hits: painted.hits,
            blame_inner: None,
            blame_minimap: None,
        };
    }
    // A binary file the OS has apps for: the "Open in" row over the notice.
    if let Some(hits) = crate::open_with::view::render_file(frame, inner, insight) {
        return FilePanelRender {
            bar,
            bar_area,
            body,
            close,
            image_hits: hits,
            blame_inner: None,
            blame_minimap: None,
        };
    }
    if blame_shown {
        insight.clamp(inner.height as usize);
        return FilePanelRender {
            bar,
            bar_area,
            body,
            close,
            image_hits: Vec::new(),
            blame_inner: Some(inner),
            blame_minimap: (map_pad > 0)
                .then(|| crate::minimap::rect_in(body))
                .flatten(),
        };
    }
    let viewport = inner.height as usize;
    insight.clamp(viewport);
    let rows = insight_rows(insight, inner.width as usize, viewport);
    frame.render_clipped(Paragraph::new(rows), inner);
    // The file's text can be selected with the mouse: the I-beam over it.
    crate::text_select::register_pointer(
        inner,
        &crate::file_view::insight_segs(insight, inner.width as usize, viewport),
    );
    // A Git LFS object card: the actions for its real content under it.
    let card_hits = crate::open_with::view::render_file_card(frame, inner, insight);
    // The mouse text selection, painted over the rows
    // just drawn.
    if let Some(sel) = insight.text_sel {
        let segs = crate::file_view::insight_segs(insight, inner.width as usize, viewport);
        crate::text_select::paint(frame.buffer_mut(), inner, &segs, &sel, &*insight);
    }
    FilePanelRender {
        bar,
        bar_area,
        body,
        close,
        image_hits: card_hits.unwrap_or_default(),
        blame_inner: None,
        blame_minimap: None,
    }
}

/// Runs one of the File View bar's buttons. Returns a status-line
/// message when the button has something to say instead of acting.
pub(crate) fn apply_file_bar_button(
    button: DiffBarButton,
    file_insight: &mut Option<FileInsight>,
    diff_view_state: &mut Option<DiffViewState>,
    link: &DaemonLink,
    config_path: &Option<std::path::PathBuf>,
) -> Option<String> {
    let current = file_insight.as_ref()?.request.clone();
    let kind = match button {
        DiffBarButton::DiffView => {
            if diff_view_state.is_some() {
                *file_insight = None;
                return None;
            }
            return Some(msg::no_diff_to_show(current.path.display()));
        }
        DiffBarButton::FileView => FileInsightKind::Content,
        DiffBarButton::Blame => FileInsightKind::Blame,
        DiffBarButton::History => FileInsightKind::History,
        DiffBarButton::ToggleMinimap => {
            return Some(crate::diff_actions::toggle_minimap(config_path));
        }
        _ => return None,
    };
    if kind != current.kind {
        let request = FileInsightRequest { kind, ..current };
        link.request_file_insight(request.clone());
        *file_insight = Some(FileInsight::loading(request));
    }
    None
}

/// Esc, `q` and the title's `✕`: back to the graph, like the Diff view's
/// own close - not back to a diff the File View may have been opened
/// over, which is what the bar's Diff View button is for.
pub(crate) fn close_file_panel(
    file_insight: &mut Option<FileInsight>,
    diff_view_state: &mut Option<DiffViewState>,
) {
    *file_insight = None;
    *diff_view_state = None;
}

/// Keys the File View lets through to the rest of the window even though
/// it otherwise owns the keyboard: moving focus between panes, and every
/// `Ctrl` chord (the palette, the `Ctrl+N` navigation prefix, ...). It is
/// a pane in the central column now, not a modal. `Ctrl+C` is the one
/// chord kept: it copies the view's text.
pub(crate) fn file_panel_passes_through(key: &KeyEvent) -> bool {
    // The File View's own chords (copy is `Ctrl+C` by default) stay here
    // rather than passing through to the window.
    // Only the copy chord: a bare-letter action of the view (Blame's `n`,
    // `p`, `g`...) also matches under Ctrl, and must not swallow Ctrl+P.
    let copy = crate::keymap::current().is(crate::keymap::Action::FileInsightCopy, key);
    matches!(key.code, KeyCode::Tab | KeyCode::BackTab)
        || (key.modifiers.contains(KeyModifiers::CONTROL) && !copy)
}

/// What opening a row of the commit-detail file list should show.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CommitFileOpen {
    /// The commit changed it: its diff, by index into `outcome.files`.
    Diff { file_index: usize, path: PathBuf },
    /// View all files, and the commit did not touch it: its content at
    /// the commit.
    Content { path: PathBuf },
}

/// Resolves a file row - `path_index` into `paths`, the list the rows
/// were built from - to what it opens. With View all files off, `paths`
/// *is* the changed list and every row is a diff; with it on, `paths` is
/// the whole tree and the row's position says nothing about
/// `outcome.files`, so the file is looked up there by path.
pub(crate) fn commit_file_open(
    outcome: &CommitFilesOutcome,
    paths: &[PathBuf],
    path_index: usize,
) -> Option<CommitFileOpen> {
    let path = paths.get(path_index)?;
    let changed = outcome.files.as_ref().ok();
    match changed.and_then(|files| files.iter().position(|f| &f.path == path)) {
        Some(file_index) => Some(CommitFileOpen::Diff {
            file_index,
            path: path.clone(),
        }),
        None => Some(CommitFileOpen::Content { path: path.clone() }),
    }
}

/// The row's index into `outcome.files`, or `None` for a file the commit
/// did not change - what the commit-file context menu carries.
pub(crate) fn changed_file_index(
    outcome: &CommitFilesOutcome,
    paths: &[PathBuf],
    path_index: usize,
) -> Option<usize> {
    match commit_file_open(outcome, paths, path_index)? {
        CommitFileOpen::Diff { file_index, .. } => Some(file_index),
        CommitFileOpen::Content { .. } => None,
    }
}

/// The status a View all files row shows, looked up by *path*, never by position in
/// `outcome.files`.
pub(crate) fn commit_file_status(
    outcome: Option<&CommitFilesOutcome>,
    path: &Path,
) -> Option<FileStatus> {
    outcome?
        .files
        .as_ref()
        .ok()?
        .iter()
        .find(|f| f.path == path)
        .map(|f| f.status)
}

/// Where the commit's file `path` was renamed or moved from, if it was.
pub(crate) fn commit_file_renamed_from(
    outcome: Option<&CommitFilesOutcome>,
    path: &Path,
) -> Option<PathBuf> {
    outcome?
        .files
        .as_ref()
        .ok()?
        .iter()
        .find(|f| f.path == path)?
        .renamed_from
        .clone()
}

/// Which of the paths list the central view is showing right now, for
/// the "click the open row again to close it" rule
/// (`commit_file_click`): a diff of this commit, or a File View of this
/// commit's file.
pub(crate) fn commit_open_path_index(
    diff: Option<&DiffViewState>,
    file: Option<&FileInsight>,
    central_view: CentralView,
    commit: Option<Oid>,
    paths: &[PathBuf],
) -> Option<usize> {
    let commit = commit?;
    let open_path = match central_view {
        CentralView::Diff => diff
            .filter(|d| d.source == DiffSource::Commit && d.commit == commit)
            .map(|d| &d.path)?,
        CentralView::File => file
            .filter(|f| {
                f.request.kind == FileInsightKind::Content
                    && f.request.at == commit
                    && !f.request.worktree
            })
            .map(|f| &f.request.path)?,
        CentralView::Log | CentralView::Editor | CentralView::Merge => return None,
    };
    paths.iter().position(|p| p == open_path)
}

/// Opens a commit-detail row: the diff, or (36) the File View at the
/// commit. The single place the Enter key and the click agree on.
pub(crate) fn open_commit_file(
    open: CommitFileOpen,
    commit: Oid,
    link: &DaemonLink,
    file_insight: &mut Option<FileInsight>,
    diff_view_state: &mut Option<DiffViewState>,
    central_view: &mut CentralView,
    compare: Option<&crate::compare::CompareSpec>,
) {
    match open {
        CommitFileOpen::Diff { file_index, path } => {
            *file_insight = None;
            *central_view = CentralView::Diff;
            // The list of a comparison is published under its synthetic key.
            let state = match compare.filter(|c| c.id == commit) {
                Some(c) => DiffViewState::for_compare(c.id, c.target.clone(), file_index, path),
                None => DiffViewState::for_commit(commit, file_index, path),
            };
            *diff_view_state = Some(state.resume_last_used());
        }
        CommitFileOpen::Content { path } => {
            *diff_view_state = None;
            open_file_insight_at(FileInsightKind::Content, path, commit, link, file_insight);
            *central_view = CentralView::File;
        }
    }
}

// -- View all files on the WIP row ------------------

/// The whole working tree: every path in HEAD's tree plus every path the
/// status reports (new and untracked files are not in HEAD; deleted ones
/// are still listed, marked). Sorted, no duplicates.
pub(crate) fn wip_all_paths(head_tree: &[PathBuf], status: &WorkingTree) -> Vec<PathBuf> {
    let mut all: Vec<PathBuf> = head_tree.to_vec();
    all.extend(status.files.iter().map(|f| f.path.clone()));
    all.sort();
    all.dedup();
    all
}

/// The status entry for `path`, if the working tree has changed it.
pub(crate) fn wip_entry<'a>(status: &'a WorkingTree, path: &Path) -> Option<&'a StatusEntry> {
    status.files.iter().find(|f| f.path == path)
}

/// The marker a WIP View all files row carries: the unstaged change
/// first (it is what is on disk), else the staged one.
pub(crate) fn wip_status(entry: Option<&StatusEntry>) -> Option<FileStatus> {
    let entry = entry?;
    entry.worktree.or(entry.index)
}

/// What opening a WIP View all files row shows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum WipFileOpen {
    /// A changed file: its diff in the half that holds the change
    /// (Unstaged first, as the marker is).
    Diff { source: DiffSource, path: PathBuf },
    /// Unchanged: the bytes on disk, in the File View.
    Content { path: PathBuf },
}

pub(crate) fn wip_file_open(status: &WorkingTree, path: &Path) -> WipFileOpen {
    match wip_entry(status, path) {
        Some(e) if e.worktree.is_some() => WipFileOpen::Diff {
            source: DiffSource::Unstaged,
            path: path.to_path_buf(),
        },
        Some(e) if e.index.is_some() => WipFileOpen::Diff {
            source: DiffSource::Staged,
            path: path.to_path_buf(),
        },
        _ => WipFileOpen::Content {
            path: path.to_path_buf(),
        },
    }
}

/// Opens a WIP View all files row. `head` is what Blame/History walk
/// from once the File View is open; the content itself is read from
/// disk.
pub(crate) fn open_wip_file(
    open: WipFileOpen,
    head: Option<Oid>,
    link: &DaemonLink,
    file_insight: &mut Option<FileInsight>,
    diff_view_state: &mut Option<DiffViewState>,
    central_view: &mut CentralView,
) {
    match open {
        WipFileOpen::Diff { source, path } => {
            *file_insight = None;
            *central_view = CentralView::Diff;
            *diff_view_state = Some(DiffViewState::for_working(source, path).resume_last_used());
        }
        WipFileOpen::Content { path } => {
            *diff_view_state = None;
            let request = FileInsightRequest {
                path,
                // Unborn HEAD: nothing to blame yet, but the content
                // still reads from disk; the zero oid is never sent for
                // a working-tree read.
                at: head.unwrap_or(Oid([0u8; 20])),
                kind: FileInsightKind::Content,
                worktree: true,
                staged: false,
            };
            link.request_file_insight(request.clone());
            *file_insight = Some(FileInsight::loading(request));
            *central_view = CentralView::File;
        }
    }
}

/// Everything the WIP row's View all files list keeps between frames.
/// One struct so the event loop holds one local for it, not six.
#[derive(Debug)]
pub(crate) struct WipAllFiles {
    /// HEAD's tree, and the HEAD it was listed at.
    pub(crate) head_tree: Option<(Oid, Vec<PathBuf>)>,
    /// The HEAD a tree listing was last asked for (answers for any other
    /// commit are not ours).
    pub(crate) requested: Option<Oid>,
    pub(crate) state: ListState,
    /// The list this frame drew: every path, and the rows built from it.
    pub(crate) paths: Vec<PathBuf>,
    pub(crate) rows: Vec<TreeRow>,
    /// The list's bordered area as last drawn; empty when not drawn.
    pub(crate) area: Rect,
    /// Does the offset still follow the selection on
    /// render - see `ui_kit::list_render_state`. `true` (ratatui's own
    /// behavior) unless a mouse-wheel scroll last touched this list.
    pub(crate) follow: bool,
}

impl Default for WipAllFiles {
    fn default() -> Self {
        WipAllFiles {
            head_tree: None,
            requested: None,
            state: ListState::default(),
            paths: Vec::new(),
            rows: Vec::new(),
            area: Rect::default(),
            follow: true,
        }
    }
}

impl WipAllFiles {
    /// The HEAD whose tree should be asked for now, if any: the list is
    /// wanted and the tree held (or asked for) is not HEAD's.
    pub(crate) fn tree_to_request(&mut self, wanted: bool, head: Option<Oid>) -> Option<Oid> {
        let head = head.filter(|_| wanted)?;
        if self.requested == Some(head) {
            return None;
        }
        self.requested = Some(head);
        Some(head)
    }

    /// Takes a commit-files answer that carries the tree asked for.
    pub(crate) fn accept(&mut self, outcome: &CommitFilesOutcome) {
        if Some(outcome.commit) != self.requested {
            return;
        }
        if let Some(Ok(paths)) = &outcome.tree {
            self.head_tree = Some((outcome.commit, paths.clone()));
        }
    }

    /// Whether the list was drawn last frame (and so owns input).
    pub(crate) fn shown(&self) -> bool {
        self.area.height > 0
    }

    /// Keyboard/click motion: moves the selection and marks the list as
    /// following again, so the view snaps back onto it.
    pub(crate) fn move_by(&mut self, delta: i32) {
        move_selection(&mut self.state, self.rows.len(), delta);
        self.follow = true;
    }

    /// Mouse-wheel motion: moves the viewport only,
    /// leaving the selection exactly where it was.
    pub(crate) fn scroll_by(&mut self, delta: i32) {
        let visible = self.area.height.saturating_sub(2) as usize;
        let new_offset = scroll_offset(self.state.offset(), self.rows.len(), visible, delta);
        *self.state.offset_mut() = new_offset;
        self.follow = false;
    }

    /// The list row under screen row `row`, if any.
    pub(crate) fn row_at(&self, row: u16) -> Option<usize> {
        row_to_index(self.area, self.state.offset(), 1, row, self.rows.len())
    }

    /// Draws the list into `area` (a bordered block) and rebuilds
    /// `paths`/`rows` from HEAD's tree and the status.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn render(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        focused: bool,
        status: &WorkingTree,
        mode: FileListMode,
        collapsed: &HashSet<PathBuf>,
        sel: &FileSelection,
    ) {
        let head_tree: &[PathBuf] = self
            .head_tree
            .as_ref()
            .map(|(_, t)| t.as_slice())
            .unwrap_or(&[]);
        self.paths = wip_all_paths(head_tree, status);
        self.rows = build_tree_rows(&self.paths, mode, collapsed, "");
        let loading = self.head_tree.is_none();
        let title = if loading {
            msg::ALL_FILES_LOADING.get().to_string()
        } else {
            msg::all_files_title(self.paths.len(), status.files.len())
        };
        let style = if focused {
            Style::default()
                .fg(th().ui.accent)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
                .fg(th().ui.muted)
                .add_modifier(Modifier::BOLD)
        };
        let block = Block::default()
            .title(title)
            .border_type(BorderType::Rounded)
            .borders(Borders::ALL)
            .border_style(style);
        let inner = block.inner(area);
        frame.render_clipped(block, area);
        // The *bordered* rect, which is what `row_to_index` counts from
        // (its `top_offset` of 1 is the title row).
        self.area = area;
        if self.rows.is_empty() {
            self.state.select(None);
        } else if let Some(sel) = self.state.selected() {
            self.state.select(Some(sel.min(self.rows.len() - 1)));
        }
        let items: Vec<ListItem> = self
            .rows
            .iter()
            .map(|row| wip_row_item(row, &self.paths, status, sel))
            .collect();
        // Render through a throwaway clone so a
        // wheel-scrolled offset survives the frame instead of snapping
        // back to the selection -- see `ui_kit::list_render_state`.
        let visible = inner.height.max(1) as usize;
        let mut render_state = list_render_state(&self.state, self.follow, visible);
        frame.render_stateful_clipped(
            List::new(items).highlight_style(
                Style::default()
                    .bg(th().ui.selection)
                    .add_modifier(Modifier::BOLD),
            ),
            inner,
            &mut render_state,
        );
        *self.state.offset_mut() = render_state.offset();
        // A scrollbar over the list's own
        // right border, rendering-only (see the Log's own site in
        // `event_loop.rs` for why).
        if let Some(track) = scrollbar::Track::vertical(area, self.rows.len(), self.state.offset())
        {
            scrollbar::render(frame, track, true, false);
        }
    }

    /// What activating row `i` does: a folder to collapse, or a file to
    /// open.
    pub(crate) fn activate(&self, i: usize, status: &WorkingTree) -> Option<WipActivation> {
        match self.rows.get(i)? {
            TreeRow::Folder { path, .. } => Some(WipActivation::ToggleFolder(path.clone())),
            TreeRow::File { index, .. } => {
                let path = self.paths.get(*index)?;
                Some(WipActivation::Open(wip_file_open(status, path)))
            }
        }
    }
}

/// Selects row `i` of the WIP View all files list and does what it
/// means: a folder collapses or expands, a changed file opens its diff,
/// an unchanged one its working-tree content. The click and the Enter
/// key both come through here.
#[allow(clippy::too_many_arguments)]
pub(crate) fn activate_wip_row(
    wip: &mut WipAllFiles,
    i: usize,
    status: &gitpoulpi_core::Result<WorkingTree>,
    head: Option<Oid>,
    link: &DaemonLink,
    file_insight: &mut Option<FileInsight>,
    diff_view_state: &mut Option<DiffViewState>,
    central_view: &mut CentralView,
    collapsed_folders: &mut HashSet<PathBuf>,
) {
    let empty = WorkingTree::default();
    let status = status.as_ref().unwrap_or(&empty);
    wip.state.select(Some(i));
    wip.follow = true;
    match wip.activate(i, status) {
        Some(WipActivation::ToggleFolder(path)) => toggle_in(collapsed_folders, path),
        Some(WipActivation::Open(open)) => open_wip_file(
            open,
            head,
            link,
            file_insight,
            diff_view_state,
            central_view,
        ),
        None => {}
    }
}

/// See [`WipAllFiles::activate`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum WipActivation {
    ToggleFolder(PathBuf),
    Open(WipFileOpen),
}

/// One drawn row of the WIP View all files list.
pub(crate) fn wip_row_item(
    row: &TreeRow,
    paths: &[PathBuf],
    status: &WorkingTree,
    sel: &FileSelection,
) -> ListItem<'static> {
    let picked = match row {
        TreeRow::Folder { path, .. } => sel.contains(FileList::WipAll, path, true),
        TreeRow::File { index, .. } => paths
            .get(*index)
            .is_some_and(|p| sel.contains(FileList::WipAll, p, false)),
    };
    let item = match row {
        TreeRow::Folder {
            depth,
            label,
            collapsed,
            ..
        } => {
            let chevron = crate::fold_mark::glyph(*collapsed);
            ListItem::new(Line::from(Span::styled(
                format!("{}{chevron} {label}/", "  ".repeat(*depth)),
                Style::default()
                    .fg(th().ui.muted)
                    .add_modifier(Modifier::BOLD),
            )))
        }
        TreeRow::File {
            depth,
            label,
            index,
        } => {
            let status = paths
                .get(*index)
                .and_then(|p| wip_status(wip_entry(status, p)));
            let marker = status.map(status_marker).unwrap_or(" ");
            let color = status.map(file_status_color).unwrap_or(th().ui.muted);
            let name_style = if status.is_some() {
                Style::default().fg(th().ui.text)
            } else {
                Style::default().fg(th().ui.muted)
            };
            ListItem::new(Line::from(vec![
                Span::styled(
                    format!("{}{marker} ", "  ".repeat(*depth)),
                    Style::default().fg(color),
                ),
                Span::styled(label.clone(), name_style),
            ]))
        }
    };
    if picked {
        item.style(Style::default().bg(th().ui.selection))
    } else {
        item
    }
}

#[cfg(test)]
#[path = "file_panel_tests/mod.rs"]
mod tests;
