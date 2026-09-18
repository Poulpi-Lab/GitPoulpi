//! Glue between the diff view and the rest of the loop: hunk/line
//! staging and discarding from the diff, the working-file diff state,
//! persisting the diff toggles, and opening File History / Blame (file
//! insight).

use super::*;
use crate::diff_view::DiffCopy;
use gitpoulpi_i18n::msg::diff_actions as msg;

/// Runs one of the Diff view's staging / discarding affordances on the
/// hunk under the cursor or the current line selection - the keyboard's
/// and the palette's entry point, as the tests call it. See
/// `diff_stage_action_at`.
#[cfg(test)]
pub(crate) fn diff_stage_action(
    action: DiffStageAction,
    state: &mut DiffViewState,
    rows: &[diff_view::DiffRow],
    hunk: Option<HunkId>,
) -> (Option<Action>, String) {
    diff_stage_action_at(action, state, rows, hunk, None)
}

/// Decides what one of the Diff view's staging / discarding affordances
/// does.
///
/// One entry point for all of them because all are reachable from the
/// keyboard, the palette, the hunk menu and the mouse, and a copy per
/// surface of "which half is this, and which direction does that mean"
/// would be that many chances to disagree. Taking `DiffStageAction` is also
/// what makes `palette.rs`'s reachability guard bite: the enum it matches
/// exhaustively is the one this function dispatches on.
///
/// `Hunk` / `DiscardHunk` act on `hunk` if given (a header button, the
/// hunk menu), else on the hunk the cursor is in - also the one whose
/// header is showing its buttons. `Lines` / `DiscardLines` act on the
/// changed lines of `range` if given (a hovered row's button, the menu
/// opened on a row), else on the line selection, else on the cursor row.
///
/// Returns the action to take and the line to show. For the two discards
/// the action is `Action::DiscardHunk` / `DiscardLines` and the line is
/// the confirmation prompt: `apply_diff_stage_at` never submits those
/// straight away.
///
/// Every refusal explains itself rather than doing nothing, like the
/// disabled diff-bar buttons.
///
/// `rows` is the row map the last render published; the hunk is taken from
/// it rather than from a position in the file, because `StageTarget::Hunk`
/// names the id the *backend's* diff assigned, which `render_diff` threaded
/// through.
pub(crate) fn diff_stage_action_at(
    action: DiffStageAction,
    state: &mut DiffViewState,
    rows: &[diff_view::DiffRow],
    hunk: Option<HunkId>,
    range: Option<(usize, usize)>,
) -> (Option<Action>, String) {
    // Selecting is not staging: it needs no index and no direction, so it
    // is answered before the working-diff check below.
    if action == DiffStageAction::SelectLines {
        return match state.selection_anchor {
            Some(_) => {
                state.selection_anchor = None;
                (None, msg::SELECTION_CLEARED.get().to_string())
            }
            None => {
                state.selection_anchor = Some(state.cursor);
                let km = crate::keymap::current();
                let key = |a| match km.shown(a) {
                    k if k.is_empty() => msg::UNBOUND.get().to_string(),
                    k => k,
                };
                (
                    None,
                    msg::selecting_lines(
                        key(crate::keymap::Action::DiffStageLines),
                        key(crate::keymap::Action::DiffDiscardLines),
                    ),
                )
            }
        };
    }
    let discard = matches!(
        action,
        DiffStageAction::DiscardHunk | DiffStageAction::DiscardLines
    );
    // The hunks of a diff read in a chosen encoding are not the backend's:
    // their ids would name other hunks (`diff_recode`).
    if state.encoding.is_some() && state.source.staging().is_some() {
        return (
            None,
            gitpoulpi_i18n::msg::diff_view::ENC_STAGING_OFF
                .get()
                .to_string(),
        );
    }
    let Some(staging) = state.source.staging() else {
        return (
            None,
            if discard {
                NOT_A_WORKING_DIFF_DISCARD
            } else {
                NOT_A_WORKING_DIFF
            }
            .get()
            .to_string(),
        );
    };
    if discard && !state.source.discards() {
        return (None, DISCARD_ON_THE_STAGED_HALF.get().to_string());
    }
    let op = if discard {
        LineOp::Discard
    } else if staging {
        LineOp::Stage
    } else {
        LineOp::Unstage
    };
    if matches!(
        action,
        DiffStageAction::Lines | DiffStageAction::DiscardLines
    ) {
        return diff_lines_action(state, rows, range, op);
    }
    let Some(hunk) = hunk.or_else(|| diff_view::hunk_at(rows, state.cursor)) else {
        return (None, msg::NO_HUNK.get().to_string());
    };
    let path = state.path.clone();
    let shown = state.path.display();
    match op {
        LineOp::Discard => (
            Some(Action::DiscardHunk {
                path: path.clone(),
                hunk,
            }),
            msg::discard_hunk_prompt(hunk.0, &shown),
        ),
        LineOp::Stage | LineOp::Unstage => (
            Some(if staging {
                Action::StageHunk { path, hunk }
            } else {
                Action::UnstageHunk { path, hunk }
            }),
            if staging {
                msg::staging_hunk(hunk.0, &shown)
            } else {
                msg::unstaging_hunk(hunk.0, &shown)
            },
        ),
    }
}

/// Submits what `diff_stage_action` decided and returns its message.
///
/// The decision is kept separate from the submission so it can be unit
/// tested: a `DaemonLink` needs a daemon, and what is worth pinning is
/// which `Action` (and `StageTarget`) each affordance produces.
pub(crate) fn apply_diff_stage(
    action: DiffStageAction,
    state: &mut DiffViewState,
    rows: &[diff_view::DiffRow],
    hunk: Option<HunkId>,
    link: &DaemonLink,
    input_mode: &mut InputMode,
) -> String {
    apply_diff_stage_at(action, state, rows, hunk, None, link, input_mode)
}

/// `apply_diff_stage` with an explicit row range (`diff_stage_action_at`).
///
/// A discard is never submitted from here: it opens the confirm modal every
/// other discard goes through, holding the exact target decided now
/// (`PendingDestructive::DiscardHunk` / `DiscardLines`). The returned line
/// is then a hint, since the modal carries the question itself.
#[allow(clippy::too_many_arguments)]
pub(crate) fn apply_diff_stage_at(
    action: DiffStageAction,
    state: &mut DiffViewState,
    rows: &[diff_view::DiffRow],
    hunk: Option<HunkId>,
    range: Option<(usize, usize)>,
    link: &DaemonLink,
    input_mode: &mut InputMode,
) -> String {
    let (submit, message) = diff_stage_action_at(action, state, rows, hunk, range);
    match submit.map(confirm_first) {
        Some(Ok(pending)) => {
            *input_mode = InputMode::Confirm {
                prompt: message,
                action: ConfirmAction::Destructive(pending),
            };
            msg::CONFIRM_HINT.get().to_string()
        }
        Some(Err(action)) => {
            link.submit_action(action);
            message
        }
        None => message,
    }
}

/// The destructive actions the Diff view can produce, as the confirm
/// modal's pending action; anything else back as it was.
// `Action` carries a whole `Ref` (merge, rebase), which `TagSync`'s remote lists
// made a few words bigger; the one caller matches on the result at once.
#[allow(clippy::result_large_err)]
pub(crate) fn confirm_first(action: Action) -> Result<PendingDestructive, Action> {
    match action {
        Action::DiscardHunk { path, hunk } => Ok(PendingDestructive::DiscardHunk { path, hunk }),
        Action::DiscardLines { path, selector } => {
            Ok(PendingDestructive::DiscardLines { path, selector })
        }
        other => Err(other),
    }
}

/// What a line-level affordance does with the lines it names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LineOp {
    Stage,
    Unstage,
    Discard,
}

/// The line-level half of `diff_stage_action_at`: the changed lines `range`
/// covers, or without one the selection, or with no selection the single
/// line under the cursor.
///
/// Context lines are never part of a selection (see `DiffRow::lines`), so a
/// range covering only context says so instead of sending an empty
/// selector, which `git apply` would reject with an opaque patch-format
/// error.
pub(crate) fn diff_lines_action(
    state: &mut DiffViewState,
    rows: &[diff_view::DiffRow],
    range: Option<(usize, usize)>,
    op: LineOp,
) -> (Option<Action>, String) {
    let (from, to) = range.unwrap_or_else(|| state.selected_rows());
    // The selection is consumed either way: leaving it on screen after it
    // was applied invites a second application against a diff that has
    // already moved.
    state.selection_anchor = None;
    let Some((hunk, line_indices, spilled)) = diff_view::selected_lines(rows, from, to) else {
        return (None, msg::NOTHING_SELECTED.get().to_string());
    };
    if line_indices.is_empty() {
        return (None, msg::SELECTION_NO_LINES.get().to_string());
    }
    let count = line_indices.len();
    let selector = LineSelector { hunk, line_indices };
    let path = state.path.clone();
    let spill = if spilled { msg::SPILL.get() } else { "" };
    match op {
        LineOp::Discard => (
            Some(Action::DiscardLines { path, selector }),
            msg::discard_lines_prompt(count as u64, hunk.0, state.path.display(), spill),
        ),
        LineOp::Stage | LineOp::Unstage => {
            let staging = op == LineOp::Stage;
            (
                Some(if staging {
                    Action::StageLines { path, selector }
                } else {
                    Action::UnstageLines { path, selector }
                }),
                if staging {
                    msg::staging_lines(count as u64, hunk.0, spill)
                } else {
                    msg::unstaging_lines(count as u64, hunk.0, spill)
                },
            )
        }
    }
}

/// `diff_lines_action` for staging (`true`) or unstaging (`false`) the
/// selection.
#[cfg(test)]
pub(crate) fn diff_stage_lines(
    state: &mut DiffViewState,
    rows: &[diff_view::DiffRow],
    staging: bool,
) -> (Option<Action>, String) {
    let op = if staging {
        LineOp::Stage
    } else {
        LineOp::Unstage
    };
    diff_lines_action(state, rows, None, op)
}

/// Registers the Diff view's body hit targets for one frame: the body's
/// buttons (hunk header, hovered row), then one band per visible row.
///
/// The ORDER is the reason this is a function rather than two loops in the
/// draw closure. `find_hit_target` returns the first registered rect
/// containing the point, and a row band spans the full width, *including*
/// the buttons at its right end. Registered the other way round, the band
/// swallows every click on `Stage hunk` and merely moves the cursor (the
/// same bug the status pane's row buttons had). A function lets a test pin
/// the order against the real registration.
pub(crate) fn push_diff_body_hits(
    hits: &mut Vec<(Rect, HitTarget)>,
    inner: Rect,
    scroll: usize,
    visible_rows: usize,
    buttons: Vec<(diff_view::DiffBodyButton, Rect)>,
) {
    for (button, rect) in buttons {
        hits.push((rect, HitTarget::DiffBodyButton(button)));
    }
    for n in 0..visible_rows {
        hits.push((
            Rect::new(inner.x, inner.y + n as u16, inner.width, 1),
            HitTarget::DiffRow(scroll + n),
        ));
    }
}

/// The Diff view state for a file row of the third pane: which half it
/// came from is what decides whether its hunk button stages or unstages.
pub(crate) fn working_diff_state(section: StatusSection, path: PathBuf) -> DiffViewState {
    DiffViewState::for_working(
        match section {
            StatusSection::Unstaged => DiffSource::Unstaged,
            StatusSection::Staged => DiffSource::Staged,
        },
        path,
    )
    .resume_last_used()
}

/// The one path every Diff view toggle (the `m`/`w`/`s` keys, the bar's
/// mouse clicks, the palette's `DiffMode`/`ToggleWordWrap`/
/// `ToggleWhitespace`) goes through, so the three input surfaces cannot
/// drift on what persisting a toggle means.
///
/// `DiffViewToggle::apply` does the mutation and hands back the one
/// `config.toml` key that changed. This function republishes it as the
/// session's "last used" default (`live_settings::install_diff_view_prefs`,
/// read by every `DiffViewState::resume_last_used`) so the next diff opened
/// shows it before the key hits disk, and writes that one key through
/// `set_in_file`'s load-modify-save so a restart resumes it too.
///
/// The write is synchronous and on this thread, like
/// `PreferencesState::write_global` for every other global toggle: it
/// parses a few hundred bytes of TOML, not a process spawn. A failing write
/// (no config directory, a permissions error) is swallowed rather than
/// surfaced: the toggle still applies for the rest of the session, and a
/// keypress that sometimes prints a disk-error banner would be worse.
pub(crate) fn persist_diff_view_toggle(
    toggle: DiffViewToggle,
    state: &mut DiffViewState,
    config_path: &Option<PathBuf>,
) {
    let (key, value) = toggle.apply(state);
    live_settings::install_diff_view_prefs(live_settings::DiffViewPrefs {
        layout: state.mode.into(),
        hunks_only: state.hunks_only,
        show_whitespace: state.show_whitespace,
        word_wrap: state.word_wrap,
    });
    if let Some(path) = config_path {
        let _ = gitpoulpi_config::set_in_file(path, key, &value);
    }
}

/// Flips `ui.minimap` (the palette's Toggle Minimap and `M` in the diff),
/// publishes it for the views and writes it back to `config.toml`. Returns
/// the status-line message.
pub(crate) fn toggle_minimap(config_path: &Option<PathBuf>) -> String {
    let on = !live_settings::minimap();
    live_settings::install_minimap(on);
    if let Some(path) = config_path {
        let _ =
            gitpoulpi_config::set_in_file(path, "ui.minimap", if on { "true" } else { "false" });
    }
    if on {
        msg::MINIMAP_ON.get().to_string()
    } else {
        msg::MINIMAP_OFF.get().to_string()
    }
}

/// Why a staging action is refused on a commit's diff: one constant, since
/// the keyboard, the palette and the mouse ask the same question.
pub(crate) const NOT_A_WORKING_DIFF: gitpoulpi_i18n::M = msg::NOT_WORKING_DIFF;

/// Why a discard is refused on a commit's diff.
pub(crate) const NOT_A_WORKING_DIFF_DISCARD: gitpoulpi_i18n::M = msg::NOT_WORKING_DIFF_DISCARD;

/// Why a discard is refused on the Staged half: the working tree is
/// reverted to the *index*, so a staged change has to be unstaged first.
pub(crate) const DISCARD_ON_THE_STAGED_HALF: gitpoulpi_i18n::M = msg::DISCARD_ON_STAGED;

/// Which file the palette's File View / Blame / History should act on: the
/// third pane's selection first (where a user picks a file by name), then
/// the selected commit's changed-file list. `None` is a real answer: the
/// caller reports "select a file first" rather than guessing.
#[allow(clippy::too_many_arguments)]
pub(crate) fn insight_target_path(
    open_diff: Option<&DiffViewState>,
    status_result: &gitpoulpi_core::Result<WorkingTree>,
    status_states: &StatusStates,
    active_status_section: StatusSection,
    status_list_mode: FileListMode,
    collapsed_status_folders: &HashSet<PathBuf>,
    // The commit-detail list's selected *file* row, resolved to its path by
    // the caller: a row index is not an index into the changed files (Tree
    // mode's folder rows, View all files' whole tree).
    commit_file_path: Option<PathBuf>,
) -> Option<PathBuf> {
    // The open diff wins, so `Ctrl+P > Blame` and the bar's own Blame
    // button act on the same file.
    if let Some(state) = open_diff {
        return Some(state.path.clone());
    }
    if let Some(path) = selected_status_path_in(
        status_result,
        status_states,
        active_status_section,
        status_list_mode,
        collapsed_status_folders,
    ) {
        return Some(path);
    }
    commit_file_path
}

/// The request for looking at `path` as part of the WIP: File View reads
/// the disk (the index when `staged`), Blame and History start from HEAD.
/// `head` is `None` while HEAD is unborn; the request then carries the zero
/// oid, which no read sends to the backend.
pub(crate) fn wip_insight_request(
    kind: FileInsightKind,
    path: PathBuf,
    staged: bool,
    head: Option<Oid>,
) -> FileInsightRequest {
    FileInsightRequest {
        path,
        at: head.unwrap_or(Oid([0u8; 20])),
        kind,
        worktree: true,
        staged,
    }
}

/// Opens the File Insight overlay on the WIP's `path` (a file of the
/// Changed files lists), for the Staged half when `staged`.
///
/// HEAD is resolved from `refs` rather than from the Log's selection:
/// blame and file history are questions about the branch the user is on.
pub(crate) fn open_wip_insight(
    kind: FileInsightKind,
    path: PathBuf,
    staged: bool,
    refs: &[gitpoulpi_core::domain::Ref],
    link: &DaemonLink,
    file_insight: &mut Option<FileInsight>,
) {
    open_insight_request(
        wip_insight_request(kind, path, staged, head_oid(refs)),
        link,
        file_insight,
    );
}

/// What the Diff view's File View / Blame / History buttons (and the hunk
/// menu's two rows) ask for. A commit's diff looks at that commit; a
/// working-tree half looks at the WIP, so the file is never resolved
/// against HEAD's tree (an untracked file is not in it).
pub(crate) fn diff_insight_request(
    state: &DiffViewState,
    kind: FileInsightKind,
    refs: &[gitpoulpi_core::domain::Ref],
) -> FileInsightRequest {
    match state.source {
        DiffSource::Commit => FileInsightRequest {
            path: state.path.clone(),
            at: state.commit,
            kind,
            worktree: false,
            staged: false,
        },
        // A comparison looks at its newer end: that commit, or the working
        // tree when the WIP row is the newer end.
        DiffSource::Compare => match state.compare {
            Some(
                gitpoulpi_core::domain::DiffTarget::CommitRange(_, to)
                | gitpoulpi_core::domain::DiffTarget::CommitSpan(_, to)
                | gitpoulpi_core::domain::DiffTarget::CommitVsParent(to),
            ) => FileInsightRequest {
                path: state.path.clone(),
                at: to,
                kind,
                worktree: false,
                staged: false,
            },
            _ => wip_insight_request(kind, state.path.clone(), false, head_oid(refs)),
        },
        DiffSource::Unstaged | DiffSource::Staged => wip_insight_request(
            kind,
            state.path.clone(),
            state.source == DiffSource::Staged,
            head_oid(refs),
        ),
    }
}

pub(crate) fn file_insight_owns_keys(
    file_insight: Option<&FileInsight>,
    input_mode: &InputMode,
) -> bool {
    file_insight.is_some() && matches!(input_mode, InputMode::Normal)
}

/// `open_file_insight` for a caller that already knows the revision: the
/// Diff view's bar blames the commit whose diff is on screen rather than
/// HEAD, because that is what its own title asks.
pub(crate) fn open_file_insight_at(
    kind: FileInsightKind,
    path: PathBuf,
    at: Oid,
    link: &DaemonLink,
    file_insight: &mut Option<FileInsight>,
) {
    open_insight_request(
        FileInsightRequest {
            path,
            at,
            kind,
            worktree: false,
            staged: false,
        },
        link,
        file_insight,
    );
}

/// Sends `request` and shows the overlay waiting for its answer.
pub(crate) fn open_insight_request(
    request: FileInsightRequest,
    link: &DaemonLink,
    file_insight: &mut Option<FileInsight>,
) {
    link.request_file_insight(request.clone());
    *file_insight = Some(FileInsight::loading(request));
}

/// The Diff view's per-hunk (and per-line) menu. `.`/`Shift+F10`/Menu open
/// it on the cursor row, right-click on the clicked row; in both cases for
/// that row's hunk and, when the row holds changes, for its lines (or the
/// whole selection, if the row is inside one): "Discard this
/// line / Stage this line".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DiffHunkContextAction {
    /// Stage or unstage the hunk, whichever direction the open half implies
    /// (as `DiffStageAction::Hunk` resolves it). Only offered on a
    /// working-tree diff: `diff_hunk_context_entries` leaves the row off on
    /// a commit's diff, since there is no direction to word the label with.
    ToggleStage,
    /// Revert the hunk in the working tree, confirmed. Unstaged half
    /// only.
    DiscardHunk,
    /// Stage or unstage the menu's lines (`DiffMenuTarget::rows`).
    Lines,
    /// Revert the menu's lines in the working tree, confirmed. Unstaged
    /// half only.
    DiscardLines,
    FileHistory,
    FileBlame,
    CopyPath,
    /// Copy text out of the diff: the menu twin of `Ctrl+C` / `y` and the
    /// palette's Copy commands. Rows come from `diff_copy_entries`; the
    /// caller runs them, since the copy needs the rendered text and the
    /// clipboard handle, which `apply_diff_hunk_context_action` lacks.
    Copy(DiffCopy),
}

impl DiffHunkContextAction {
    /// `staging: Some(true)` labels the toggle "Stage hunk", `Some(false)`
    /// "Unstage hunk"; the line rows get their wording from
    /// `diff_hunk_context_entries`, which knows how many lines they cover.
    pub(crate) fn label(self, staging: Option<bool>) -> &'static str {
        match self {
            DiffHunkContextAction::ToggleStage => match staging {
                Some(true) => msg::HUNK_STAGE.get(),
                Some(false) => msg::HUNK_UNSTAGE.get(),
                None => msg::HUNK_STAGE.get(),
            },
            DiffHunkContextAction::DiscardHunk => msg::HUNK_DISCARD.get(),
            DiffHunkContextAction::Lines => match staging {
                Some(false) => msg::LINES_UNSTAGE.get(),
                _ => msg::LINES_STAGE.get(),
            },
            DiffHunkContextAction::DiscardLines => msg::LINES_DISCARD.get(),
            DiffHunkContextAction::FileHistory => {
                gitpoulpi_i18n::msg::status_pane::FA_FILE_HISTORY.get()
            }
            DiffHunkContextAction::FileBlame => {
                gitpoulpi_i18n::msg::status_pane::FA_FILE_BLAME.get()
            }
            DiffHunkContextAction::CopyPath => gitpoulpi_i18n::msg::status_pane::FA_COPY_PATH.get(),
            DiffHunkContextAction::Copy(DiffCopy::Selection) => msg::COPY_SELECTION.get(),
            DiffHunkContextAction::Copy(DiffCopy::Line) => msg::COPY_LINE.get(),
            DiffHunkContextAction::Copy(DiffCopy::Hunk) => msg::COPY_HUNK.get(),
        }
    }
}

/// What the hunk menu was opened on: the hunk, and the rows its line
/// entries act on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DiffMenuTarget {
    pub(crate) hunk: HunkId,
    pub(crate) rows: (usize, usize),
}

/// The menu target for a menu opened on diff row `row`, and how many
/// changed lines its line entries would act on (0 = no line entries).
///
/// The rows are the open selection when `row` is inside it (a right-click
/// on a selected line acts on the selection, as in any editor), otherwise
/// `row` alone. `None` off every hunk.
pub(crate) fn diff_menu_target(
    state: &DiffViewState,
    rows: &[diff_view::DiffRow],
    row: usize,
) -> Option<(DiffMenuTarget, usize)> {
    let hunk = diff_view::hunk_at(rows, row)?;
    let range = match state.selection_anchor {
        Some(_) => {
            let (from, to) = state.selected_rows();
            if (from..=to).contains(&row) {
                (from, to)
            } else {
                (row, row)
            }
        }
        None => (row, row),
    };
    let lines = diff_view::selected_lines(rows, range.0, range.1)
        .filter(|(h, _, _)| *h == hunk)
        .map_or(0, |(_, l, _)| l.len());
    Some((DiffMenuTarget { hunk, rows: range }, lines))
}

/// The menu's rows for the diff currently open. `staging` is
/// `state.source.staging()`: `None` on a commit's diff, where the staging
/// and discarding rows are left off rather than shown inert (the "no index
/// to act on" reason of `NOT_A_WORKING_DIFF`). `lines` is how many changed
/// lines the menu's rows cover (see `diff_menu_target`); the Discard rows
/// appear only on the Unstaged half (`staging == Some(true)`).
pub(crate) fn diff_hunk_context_entries(
    staging: Option<bool>,
    lines: usize,
) -> Vec<context_menu::Entry<DiffHunkContextAction>> {
    use context_menu::Entry;
    let mut entries = Vec::new();
    if let Some(staging) = staging {
        entries.push(Entry::action(
            DiffHunkContextAction::ToggleStage.label(Some(staging)),
            DiffHunkContextAction::ToggleStage,
        ));
        if staging {
            entries.push(Entry::danger(
                DiffHunkContextAction::DiscardHunk.label(None),
                DiffHunkContextAction::DiscardHunk,
            ));
        }
        if lines > 0 {
            entries.push(Entry::Separator);
            entries.push(Entry::action(
                match (staging, lines == 1) {
                    (true, true) => msg::MENU_STAGE_LINE.get().to_string(),
                    (true, false) => msg::menu_stage_lines(lines),
                    (false, true) => msg::MENU_UNSTAGE_LINE.get().to_string(),
                    (false, false) => msg::menu_unstage_lines(lines),
                },
                DiffHunkContextAction::Lines,
            ));
            if staging {
                entries.push(Entry::danger(
                    if lines == 1 {
                        msg::MENU_DISCARD_LINE.get().to_string()
                    } else {
                        msg::menu_discard_lines(lines)
                    },
                    DiffHunkContextAction::DiscardLines,
                ));
            }
        }
        entries.push(Entry::Separator);
    }
    entries.push(Entry::action(
        DiffHunkContextAction::FileHistory.label(None),
        DiffHunkContextAction::FileHistory,
    ));
    entries.push(Entry::action(
        DiffHunkContextAction::FileBlame.label(None),
        DiffHunkContextAction::FileBlame,
    ));
    entries.push(Entry::action(
        DiffHunkContextAction::CopyPath.label(None),
        DiffHunkContextAction::CopyPath,
    ));
    entries
}

/// The Copy rows appended to the hunk menu, under a rule: "Copy selection"
/// leads when a text selection is open, then the line under the cursor (or
/// the `v` line range) and the whole hunk.
pub(crate) fn diff_copy_entries(
    has_selection: bool,
) -> Vec<context_menu::Entry<DiffHunkContextAction>> {
    use context_menu::Entry;
    let mut entries = vec![Entry::Separator];
    let mut copy = |what| {
        let action = DiffHunkContextAction::Copy(what);
        entries.push(Entry::action(action.label(None), action));
    };
    if has_selection {
        copy(DiffCopy::Selection);
    }
    copy(DiffCopy::Line);
    copy(DiffCopy::Hunk);
    entries
}

/// Runs one of the Diff view's Copy actions (the menu's Copy rows, the
/// palette's Copy commands) and returns the status-line sentence.
pub(crate) fn copy_from_diff(
    state: &Option<DiffViewState>,
    render: &Option<std::rc::Rc<diff_view::DiffRender>>,
    what: DiffCopy,
    clipboard: &mut crate::clipboard::Clipboard,
) -> Option<String> {
    let (state, render) = (state.as_ref()?, render.as_deref()?);
    Some(match diff_view::copy_text(render, state, what) {
        Ok((label, text)) => clipboard.copy(label, &text),
        Err(why) => why.to_string(),
    })
}

/// Builds the hunk menu for diff row `row`, anchored at `(col, y)` - the
/// one constructor the keyboard (`.` on the cursor row) and the mouse
/// (right-click) share, so they cannot offer different rows.
pub(crate) fn open_diff_hunk_menu(
    state: &DiffViewState,
    rows: &[diff_view::DiffRow],
    row: usize,
    col: u16,
    y: u16,
) -> Option<(DiffMenuTarget, ContextMenu<DiffHunkContextAction>)> {
    let (target, lines) = diff_menu_target(state, rows, row)?;
    let mut entries = diff_hunk_context_entries(state.source.staging(), lines);
    // The Copy rows close the menu.
    entries.extend(diff_copy_entries(state.text_sel.is_some()));
    Some((
        target,
        ContextMenu::new(msg::menu_title(target.hunk.0), entries, col, y),
    ))
}

/// Runs a hunk-menu action. The staging and discarding rows go through
/// `apply_diff_stage_at` with the menu's own hunk / rows, so discards go
/// through the confirm modal (`input_mode`). `refs`, `link` and
/// `file_insight` are what `open_insight_request` needs for the two insight
/// rows, resolved through `diff_insight_request` like the bar's own buttons.
#[allow(clippy::too_many_arguments)]
pub(crate) fn apply_diff_hunk_context_action(
    action: DiffHunkContextAction,
    target: DiffMenuTarget,
    state: &mut DiffViewState,
    rows: &[diff_view::DiffRow],
    refs: &[gitpoulpi_core::domain::Ref],
    link: &DaemonLink,
    file_insight: &mut Option<FileInsight>,
    last_message: &mut Option<String>,
    input_mode: &mut InputMode,
) {
    let staged = match action {
        DiffHunkContextAction::ToggleStage => {
            Some((DiffStageAction::Hunk, Some(target.hunk), None))
        }
        DiffHunkContextAction::DiscardHunk => {
            Some((DiffStageAction::DiscardHunk, Some(target.hunk), None))
        }
        DiffHunkContextAction::Lines => Some((DiffStageAction::Lines, None, Some(target.rows))),
        DiffHunkContextAction::DiscardLines => {
            Some((DiffStageAction::DiscardLines, None, Some(target.rows)))
        }
        _ => None,
    };
    if let Some((a, hunk, range)) = staged {
        *last_message = Some(apply_diff_stage_at(
            a, state, rows, hunk, range, link, input_mode,
        ));
        return;
    }
    match action {
        DiffHunkContextAction::FileHistory | DiffHunkContextAction::FileBlame => {
            let kind = if action == DiffHunkContextAction::FileHistory {
                FileInsightKind::History
            } else {
                FileInsightKind::Blame
            };
            open_insight_request(diff_insight_request(state, kind, refs), link, file_insight);
        }
        // OSC 52 goes to the terminal, so it is written to the same stdout
        // ratatui draws on, as `FileContextAction::CopyPath` does.
        DiffHunkContextAction::CopyPath => {
            let seq = osc52_copy(&state.path.to_string_lossy());
            use std::io::Write;
            let mut out = std::io::stdout();
            let _ = out.write_all(seq.as_bytes());
            let _ = out.flush();
            *last_message = Some(gitpoulpi_i18n::msg::status_list::copied_clipboard(
                state.path.display(),
            ));
        }
        // Answered by `apply_diff_stage_at` above.
        DiffHunkContextAction::ToggleStage
        | DiffHunkContextAction::DiscardHunk
        | DiffHunkContextAction::Lines
        | DiffHunkContextAction::DiscardLines => {}
        // Run by the caller (see the variant's doc comment).
        DiffHunkContextAction::Copy(_) => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::*;

    /// The menu's rows as their labels, separators as `---`.
    fn menu_labels(staging: Option<bool>, lines: usize) -> Vec<String> {
        diff_hunk_context_entries(staging, lines)
            .iter()
            .map(|e| match e {
                context_menu::Entry::Action { label, .. } => label.clone(),
                context_menu::Entry::Separator => "---".to_string(),
                other => panic!("unexpected row {other:?}"),
            })
            .collect()
    }

    /// On a working-tree half the toggle leads, worded for the direction;
    /// the Unstaged half adds Discard hunk, and a menu opened on changed
    /// lines adds their own rows (Stage / Discard on the Unstaged half,
    /// Unstage alone on the Staged one).
    #[test]
    fn the_hunk_menu_offers_the_hunk_and_line_actions_of_the_open_half() {
        assert_eq!(
            menu_labels(Some(true), 0),
            [
                "Stage hunk",
                "Discard hunk",
                "---",
                "File History",
                "File Blame",
                "Copy file path"
            ]
        );
        assert_eq!(
            menu_labels(Some(true), 1),
            [
                "Stage hunk",
                "Discard hunk",
                "---",
                "Stage this line",
                "Discard this line",
                "---",
                "File History",
                "File Blame",
                "Copy file path"
            ]
        );
        assert_eq!(
            menu_labels(Some(false), 3),
            [
                "Unstage hunk",
                "---",
                "Unstage 3 lines",
                "---",
                "File History",
                "File Blame",
                "Copy file path"
            ]
        );
    }

    /// A commit's diff has no index to stage into (`state.source.staging()`
    /// is `None`): the staging and discarding rows are left off rather than
    /// shown disabled, even with lines under the menu.
    #[test]
    fn the_hunk_menu_drops_the_toggle_entirely_on_a_commits_diff() {
        let entries = diff_hunk_context_entries(None, 2);
        assert_eq!(entries.len(), 3, "no staging row on a commit's diff");
        for entry in &entries {
            match entry {
                context_menu::Entry::Action { action, .. } => assert!(
                    matches!(
                        action,
                        DiffHunkContextAction::FileHistory
                            | DiffHunkContextAction::FileBlame
                            | DiffHunkContextAction::CopyPath
                    ),
                    "{action:?} must not appear when there is no staging direction"
                ),
                other => panic!("expected only Action rows, got {other:?}"),
            }
        }
    }

    /// `persist_diff_view_toggle` is the one path the keyboard, the mouse and
    /// the palette funnel through. This pins both halves of "resume the last
    /// view mode": the in-memory snapshot every newly-opened diff reads, and
    /// the file a restart reads.
    ///
    /// Red without the `install_diff_view_prefs` call (the "second diff"
    /// assertion keeps showing Inline) and red without the `set_in_file`
    /// call (the `Config::load` assertion keeps `"inline"`).
    #[test]
    fn persisting_a_toggle_updates_both_the_live_session_and_config_toml() {
        // See `live_settings::DIFF_VIEW_PREFS_TEST_LOCK`: this test and
        // `diff_view.rs`'s `resume_last_used` test share one process-wide
        // static and race without it.
        let _guard = live_settings::DIFF_VIEW_PREFS_TEST_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("config.toml");
        let diff_config_path = Some(config_path.clone());

        // Start from the schema defaults, as a fresh install does.
        live_settings::install_diff_view_prefs(live_settings::DiffViewPrefs::const_default());

        let mut first = DiffViewState::for_commit(oid(1), 0, PathBuf::from("a.rs"));
        assert_eq!(first.mode, diff_view::DiffViewMode::Inline);

        persist_diff_view_toggle(
            DiffViewToggle::Mode(diff_view::DiffViewMode::Split),
            &mut first,
            &diff_config_path,
        );
        assert_eq!(first.mode, diff_view::DiffViewMode::Split);

        // Within the session: a *second* diff, opened after the toggle,
        // resumes Split without ever touching disk.
        let second = DiffViewState::for_commit(oid(2), 0, PathBuf::from("b.rs")).resume_last_used();
        assert_eq!(second.mode, diff_view::DiffViewMode::Split);

        // Across a restart: config.toml on disk actually says so.
        let on_disk = gitpoulpi_config::Config::load(&config_path).unwrap();
        assert_eq!(on_disk.ui.diff_layout, gitpoulpi_config::DiffLayout::Split);

        // The other two toggles go through the same function and persist
        // independently of the mode.
        persist_diff_view_toggle(
            DiffViewToggle::ToggleWordWrap,
            &mut first,
            &diff_config_path,
        );
        persist_diff_view_toggle(
            DiffViewToggle::ToggleWhitespace,
            &mut first,
            &diff_config_path,
        );
        let on_disk = gitpoulpi_config::Config::load(&config_path).unwrap();
        assert!(on_disk.ui.diff_word_wrap);
        assert!(!on_disk.ui.diff_show_whitespace);
        // The mode set earlier must survive: `set_in_file`'s load-modify-save
        // keeps an unrelated key's write from clobbering it.
        assert_eq!(on_disk.ui.diff_layout, gitpoulpi_config::DiffLayout::Split);

        // Leave the process-wide static as every other test expects it.
        live_settings::install_diff_view_prefs(live_settings::DiffViewPrefs::const_default());
    }

    /// No config directory on this machine (`diff_config_path` is `None`)
    /// must not panic or lose the in-memory toggle; only the disk half is
    /// unavailable.
    #[test]
    fn persisting_a_toggle_with_no_config_path_still_updates_the_live_session() {
        let _guard = live_settings::DIFF_VIEW_PREFS_TEST_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        live_settings::install_diff_view_prefs(live_settings::DiffViewPrefs::const_default());
        let mut state = DiffViewState::for_commit(oid(3), 0, PathBuf::from("c.rs"));
        persist_diff_view_toggle(DiffViewToggle::ToggleWordWrap, &mut state, &None);
        assert!(state.word_wrap);
        let resumed =
            DiffViewState::for_commit(oid(4), 0, PathBuf::from("d.rs")).resume_last_used();
        assert!(resumed.word_wrap);
        live_settings::install_diff_view_prefs(live_settings::DiffViewPrefs::const_default());
    }

    /// `Enter` means two things here: "stage the selected lines" in the Diff
    /// view and "go to this blamed line's commit" in the File Insight
    /// overlay, both in the same event `match`. What keeps one keystroke from
    /// doing both is that the overlay's arm sits *above* the `input_mode`
    /// dispatch and swallows the key; this pins that precedence. Without the
    /// `file_insight.is_some()` half the overlay never opens; without the
    /// `InputMode::Normal` half typing "blame" into the palette scrolls the
    /// overlay instead of filtering commands.
    #[test]
    fn the_file_insight_overlay_takes_keys_only_while_it_is_open_and_nothing_else_is() {
        let insight = FileInsight::loading(FileInsightRequest {
            path: PathBuf::from("f.txt"),
            at: Oid([1u8; 20]),
            kind: FileInsightKind::Blame,
            worktree: false,
            staged: false,
        });

        // Open, nothing else active: the overlay gets the key, so the
        // Diff view's Space/v/Enter never run.
        assert!(file_insight_owns_keys(Some(&insight), &InputMode::Normal));

        // Closed: every key falls through to the normal dispatch, which
        // is what keeps hunk/line staging working.
        assert!(!file_insight_owns_keys(None, &InputMode::Normal));

        // Open, but a text-entry mode is on top of it -- that mode owns
        // the keyboard, letters included.
        assert!(!file_insight_owns_keys(
            Some(&insight),
            &InputMode::CommandPalette {
                draft: "blame".into(),
                selected: 0,
            }
        ));
        assert!(!file_insight_owns_keys(
            Some(&insight),
            &InputMode::FilteringRefs(String::new())
        ));
    }

    /// The Diff view's Blame/File View/History buttons must never send the
    /// all-zero placeholder `Oid` that `DiffViewState::for_working` carries:
    /// it would ask the backend for an object that exists in no repository.
    #[test]
    fn diff_view_insight_asks_for_the_commit_for_a_commit_and_the_wip_for_the_working_tree() {
        let commit = Oid([7u8; 20]);
        let head = Oid([9u8; 20]);
        let refs = vec![gitpoulpi_core::domain::Ref {
            kind: RefKind::LocalBranch {
                name: "main".into(),
            },
            target: head,
            visibility: gitpoulpi_core::domain::RefVisibility::Shown,
            is_head: true,
            upstream: None,
            tag_sync: None,
        }];

        let on_commit = DiffViewState::for_commit(commit, 0, PathBuf::from("f.txt"));
        for kind in [
            FileInsightKind::Content,
            FileInsightKind::Blame,
            FileInsightKind::History,
        ] {
            let request = diff_insight_request(&on_commit, kind, &refs);
            assert_eq!((request.at, request.worktree), (commit, false));
        }

        for source in [DiffSource::Unstaged, DiffSource::Staged] {
            let working = DiffViewState::for_working(source, PathBuf::from("f.txt"));
            assert_ne!(
                working.commit, head,
                "the fixture is pointless if for_working already carried HEAD"
            );
            for kind in [
                FileInsightKind::Content,
                FileInsightKind::Blame,
                FileInsightKind::History,
            ] {
                let request = diff_insight_request(&working, kind, &refs);
                assert_eq!(request.at, head, "{source:?} walks from HEAD");
                assert!(request.worktree, "{source:?} {kind:?} is a WIP request");
                assert_eq!(request.staged, source == DiffSource::Staged);
            }
        }

        // An unborn HEAD still opens: the zero oid is "unborn", never sent.
        let working = DiffViewState::for_working(DiffSource::Unstaged, PathBuf::from("f.txt"));
        let request = diff_insight_request(&working, FileInsightKind::History, &[]);
        assert!(request.is_unborn() && request.worktree);
    }

    // ---- hunk- and line-level staging from the Diff view -------------

    fn staging_rows() -> (diff_view::DiffRender, gitpoulpi_core::domain::FileDiff) {
        use gitpoulpi_core::domain::{DiffLine, FileDiff, FileStatus, Hunk, HunkId, LineOrigin};
        let mk = |origin, content: &str| DiffLine {
            origin,
            content: content.to_string(),
            staged: false,
        };
        let file = FileDiff {
            path: PathBuf::from("multi.txt"),
            status: FileStatus::Modified,
            hunks: vec![
                Hunk {
                    id: HunkId(0),
                    old_start: 1,
                    old_lines: 3,
                    new_start: 1,
                    new_lines: 3,
                    lines: vec![
                        mk(LineOrigin::Context, "line1"),
                        mk(LineOrigin::Delete, "line2"),
                        mk(LineOrigin::Add, "line2-changed"),
                    ],
                },
                Hunk {
                    id: HunkId(1),
                    old_start: 17,
                    old_lines: 3,
                    new_start: 17,
                    new_lines: 3,
                    lines: vec![
                        mk(LineOrigin::Context, "line18"),
                        mk(LineOrigin::Delete, "line19"),
                        mk(LineOrigin::Add, "line19-changed"),
                    ],
                },
            ],
        };
        let rendered =
            diff_view::render_diff(&file, diff_view::DiffViewMode::Inline, true, false, 80);
        (rendered, file)
    }

    /// The Diff view's staging affordances produce exactly the
    /// `StageTarget`s `gitpoulpi stage --hunk` / `--lines` produce: a
    /// hunk id from the backend's own diff, and line indices into that
    /// hunk's `lines`. The direction comes from which half of the third
    /// pane the diff was opened on, never from the key that was pressed.
    #[test]
    fn the_diff_views_staging_produces_the_same_targets_the_cli_does() {
        let (rendered, _) = staging_rows();
        let mut state = DiffViewState::for_working(DiffSource::Unstaged, "multi.txt".into());
        // Cursor on the second hunk's add row.
        state.cursor = rendered.hunk_starts[1] + 3;

        let (action, message) =
            diff_stage_action(DiffStageAction::Hunk, &mut state, &rendered.rows, None);
        match action.expect("a hunk under the cursor is stageable") {
            Action::StageHunk { path, hunk } => {
                assert_eq!(path, PathBuf::from("multi.txt"));
                assert_eq!(hunk, HunkId(1), "the hunk the cursor is in, not the first");
            }
            _ => panic!("the Unstaged half must STAGE, got something else"),
        }
        assert!(message.contains("staging hunk 1"), "got {message:?}");

        // The Staged half is the same affordance in the other direction.
        let mut staged = DiffViewState::for_working(DiffSource::Staged, "multi.txt".into());
        staged.cursor = rendered.hunk_starts[0];
        let (action, _) =
            diff_stage_action(DiffStageAction::Hunk, &mut staged, &rendered.rows, None);
        assert!(matches!(
            action.expect("stageable"),
            Action::UnstageHunk {
                hunk: HunkId(0),
                ..
            }
        ));

        // An explicit hunk (what the mouse passes, from the button's own
        // hit target) wins over the cursor's.
        let (action, _) = diff_stage_action(
            DiffStageAction::Hunk,
            &mut state,
            &rendered.rows,
            Some(HunkId(0)),
        );
        assert!(matches!(
            action.expect("stageable"),
            Action::StageHunk {
                hunk: HunkId(0),
                ..
            }
        ));
    }

    /// In the whole-file layout (Hunks off) the hunks sit among the file's
    /// unchanged lines, and staging still names the backend's own hunk id and
    /// line indices (the hunk under the cursor, the `.` menu's hunk, a line
    /// selection) while a row between hunks belongs to none and says so.
    /// Checked in both layouts.
    #[test]
    fn staging_works_the_same_in_the_whole_file_layout() {
        let (_, file) = staging_rows();
        let content: Vec<String> = (1..=20)
            .map(|n| match n {
                2 => "line2-changed".to_string(),
                19 => "line19-changed".to_string(),
                n => format!("line{n}"),
            })
            .collect();
        for mode in [
            diff_view::DiffViewMode::Inline,
            diff_view::DiffViewMode::Split,
        ] {
            let rendered = diff_view::render_diff_with(
                &file,
                &diff_view::DiffRenderOpts {
                    mode,
                    hunks_only: false,
                    show_whitespace: true,
                    word_wrap: false,
                    width: 100,
                    h_offset: 0,
                    full_file: Some(&content),
                    lfs_raw: false,
                    lfs_local: None,
                },
            );
            // Gap rows are there: line 4..=16 between the hunks.
            assert!(rendered.lines.len() >= 20, "{mode:?}: whole file drawn");
            let mut state = DiffViewState::for_working(DiffSource::Unstaged, "multi.txt".into());
            state.hunks_only = false;
            state.mode = mode;

            // The second hunk's changed row (inline: header+ctx+del+add;
            // split: header+ctx+pair).
            let changed_row = rendered.hunk_starts[1] + 2;
            state.cursor = changed_row;
            let (action, _) =
                diff_stage_action(DiffStageAction::Hunk, &mut state, &rendered.rows, None);
            assert!(
                matches!(
                    action,
                    Some(Action::StageHunk {
                        hunk: HunkId(1),
                        ..
                    })
                ),
                "{mode:?}: {action:?}"
            );
            // The `.` menu anchors on the same hunk.
            assert_eq!(
                diff_view::hunk_at(&rendered.rows, changed_row),
                Some(HunkId(1))
            );

            // A line selection over the whole second hunk.
            state.selection_anchor = Some(rendered.hunk_starts[1]);
            state.cursor = rendered.hunk_starts[1] + 3;
            let (action, _) = diff_stage_lines(&mut state, &rendered.rows, true);
            match action {
                Some(Action::StageLines { selector, .. }) => {
                    assert_eq!(selector.hunk, HunkId(1), "{mode:?}");
                    assert_eq!(selector.line_indices, vec![1, 2], "{mode:?}");
                }
                other => panic!("{mode:?}: {other:?}"),
            }

            // A row between the hunks (new line 10) belongs to none.
            let gap_row = rendered.hunk_starts[1] - 3;
            state.cursor = gap_row;
            let (action, message) =
                diff_stage_action(DiffStageAction::Hunk, &mut state, &rendered.rows, None);
            assert!(action.is_none(), "{mode:?}");
            assert_eq!(message, "no hunk under the cursor");
        }
    }

    /// A line range stages a `LineSelector` naming one hunk and the
    /// indices of the changed lines inside it - the numbering
    /// `StageTarget::Line` is defined in.
    #[test]
    fn a_line_selection_stages_the_changed_lines_of_one_hunk() {
        let (rendered, _) = staging_rows();
        let mut state = DiffViewState::for_working(DiffSource::Unstaged, "multi.txt".into());
        // Select the whole of the second hunk, header included.
        state.selection_anchor = Some(rendered.hunk_starts[1]);
        state.cursor = rendered.hunk_starts[1] + 3;
        let (action, message) =
            diff_stage_action(DiffStageAction::Lines, &mut state, &rendered.rows, None);
        match action.expect("a selection with changes in it") {
            Action::StageLines { path, selector } => {
                assert_eq!(path, PathBuf::from("multi.txt"));
                assert_eq!(selector.hunk, HunkId(1));
                assert_eq!(selector.line_indices, vec![1, 2]);
            }
            _ => panic!("expected StageLines"),
        }
        assert!(message.contains("2 lines"), "got {message:?}");
        assert_eq!(
            state.selection_anchor, None,
            "the selection is consumed, or a second Enter re-applies it against a diff that moved"
        );

        // With no selection open, the cursor row alone is the target.
        state.cursor = rendered.hunk_starts[0] + 2; // the delete
        let (action, _) =
            diff_stage_action(DiffStageAction::Lines, &mut state, &rendered.rows, None);
        match action.expect("one line is still a selection") {
            Action::StageLines { selector, .. } => {
                assert_eq!(selector.line_indices, vec![1]);
            }
            _ => panic!("expected StageLines"),
        }

        // A row with no changes on it sends nothing at all.
        state.cursor = rendered.hunk_starts[0] + 1; // the context line
        let (action, message) =
            diff_stage_action(DiffStageAction::Lines, &mut state, &rendered.rows, None);
        assert!(action.is_none());
        assert!(message.contains("no added or removed lines"), "{message}");
    }

    /// A commit's diff has no index to stage into, and the hunk ids in it
    /// mean something else entirely - so the affordance refuses, and
    /// says why, rather than sending a request that would silently stage
    /// an unrelated hunk of the working tree.
    #[test]
    fn a_commits_diff_refuses_to_stage_and_explains_itself() {
        let (rendered, _) = staging_rows();
        let mut state = DiffViewState::for_commit(oid(3), 0, "multi.txt".into());
        for action in [DiffStageAction::Hunk, DiffStageAction::Lines] {
            let (submitted, message) =
                diff_stage_action(action, &mut state, &rendered.rows, Some(HunkId(0)));
            assert!(submitted.is_none(), "{action:?} must not submit anything");
            assert!(message.contains("working tree"), "got {message:?}");
        }
        assert_eq!(
            DiffSource::Commit.hunk_label(),
            None,
            "and no button is drawn there either"
        );
        assert_eq!(DiffSource::Unstaged.hunk_label(), Some("Stage hunk"));
        assert_eq!(DiffSource::Staged.hunk_label(), Some("Unstage hunk"));
    }

    /// The hunks of a diff read in a chosen encoding are `diff_recode`'s, not
    /// the backend's: staging one would name another hunk, so it is refused
    /// with the way out (and Auto-detect stages as before).
    #[test]
    fn staging_is_off_while_an_encoding_is_chosen() {
        let (rendered, _) = staging_rows();
        let mut state = DiffViewState::for_working(DiffSource::Unstaged, "multi.txt".into());
        state.encoding = Some(crate::editor_charset::Charset::Windows1252);
        for action in [
            DiffStageAction::Hunk,
            DiffStageAction::Lines,
            DiffStageAction::DiscardHunk,
            DiffStageAction::DiscardLines,
        ] {
            let (act, why) = diff_stage_action(action, &mut state, &rendered.rows, None);
            assert!(act.is_none(), "{action:?} submitted");
            assert!(why.contains("Auto-detect"), "{action:?}: {why}");
        }
        state.encoding = None;
        let (act, _) = diff_stage_action(DiffStageAction::Hunk, &mut state, &rendered.rows, None);
        assert!(act.is_some(), "Auto-detect stages as before");
    }

    /// Selecting is not staging: `v` works the same on any diff, and
    /// submits nothing.
    #[test]
    fn starting_a_line_selection_submits_nothing_and_toggles() {
        let (rendered, _) = staging_rows();
        let mut state = DiffViewState::for_working(DiffSource::Unstaged, "multi.txt".into());
        state.cursor = 3;
        let (action, message) = diff_stage_action(
            DiffStageAction::SelectLines,
            &mut state,
            &rendered.rows,
            None,
        );
        assert!(action.is_none());
        assert_eq!(state.selection_anchor, Some(3));
        assert!(message.contains("selecting lines"), "{message}");
        let (_, message) = diff_stage_action(
            DiffStageAction::SelectLines,
            &mut state,
            &rendered.rows,
            None,
        );
        assert_eq!(state.selection_anchor, None);
        assert!(message.contains("cleared"), "{message}");
    }

    /// `find_hit_target` returns the FIRST registered rect containing the
    /// point, and a diff row's band spans the full width *including* the hunk
    /// button at its right end. Registered band-first, the band swallows
    /// every click on `Stage hunk` and merely moves the cursor, so the render
    /// pass pushes the button first.
    #[test]
    fn the_hunk_button_is_registered_before_the_band_that_covers_it() {
        let inner = Rect::new(2, 5, 60, 10);
        let mut rendered = staging_rows().0;
        // The button as the render pass paints it: on hunk 1's header,
        // which with a scroll of 4 sits on the second visible row.
        let header_row = rendered.hunk_starts[1];
        let scroll = header_row - 1;
        let (dx, bw) =
            diff_view::hunk_button(&mut rendered.lines, header_row, inner.width, "Stage hunk")
                .unwrap();
        let button_y = inner.y + 1;
        let mut hits: Vec<(Rect, HitTarget)> = Vec::new();
        push_diff_body_hits(
            &mut hits,
            inner,
            scroll,
            inner.height as usize,
            vec![(
                diff_view::DiffBodyButton::Hunk(HunkId(1)),
                Rect::new(inner.x + dx, button_y, bw, 1),
            )],
        );

        let button_x = inner.x + dx + 1;
        assert_eq!(
            find_hit_target(&hits, button_x, button_y),
            Some(HitTarget::DiffBodyButton(diff_view::DiffBodyButton::Hunk(
                HunkId(1)
            ))),
            "the band swallowed the button - it must be registered first"
        );
        // ...and the band still answers everywhere else on that row, with
        // the scroll folded in so the row index is the DIFF's, not the
        // screen's.
        assert_eq!(
            find_hit_target(&hits, inner.x + 1, button_y),
            Some(HitTarget::DiffRow(header_row))
        );
        assert_eq!(
            find_hit_target(&hits, inner.x + 1, inner.y),
            Some(HitTarget::DiffRow(scroll))
        );
        // The order that breaks it, pinned so the rule stays visible.
        let backwards: Vec<(Rect, HitTarget)> = hits.iter().cloned().rev().collect();
        assert_eq!(
            find_hit_target(&backwards, button_x, button_y),
            Some(HitTarget::DiffRow(header_row)),
            "band-first swallows the button - this is what must not be shipped"
        );
    }

    // ---- discarding from the Diff view --------------

    /// Discard hunk names the hunk under the cursor (or the one a button /
    /// the menu passes), and its message is the confirmation prompt --
    /// `apply_diff_stage_at` turns that into the confirm modal via
    /// `confirm_first`, never a submission.
    #[test]
    fn discard_hunk_targets_the_hunk_and_words_the_confirmation() {
        let (rendered, _) = staging_rows();
        let mut state = DiffViewState::for_working(DiffSource::Unstaged, "multi.txt".into());
        state.cursor = rendered.hunk_starts[1] + 2;
        let (action, prompt) = diff_stage_action(
            DiffStageAction::DiscardHunk,
            &mut state,
            &rendered.rows,
            None,
        );
        let action = action.expect("a hunk under the cursor is discardable");
        assert_eq!(
            action,
            Action::DiscardHunk {
                path: "multi.txt".into(),
                hunk: HunkId(1)
            }
        );
        assert!(
            prompt.starts_with("Discard hunk 1 of multi.txt?")
                && prompt.contains("cannot be undone"),
            "{prompt}"
        );
        // Never submitted as is: it goes to the confirm modal, and the
        // modal's `y` gives back exactly this action.
        match confirm_first(action.clone()) {
            Ok(pending) => assert_eq!(pending.into_action(), action),
            Err(a) => panic!("a discard must be confirmed first, got {a:?}"),
        }
        // A button's / the menu's own hunk wins over the cursor's.
        let (action, _) = diff_stage_action(
            DiffStageAction::DiscardHunk,
            &mut state,
            &rendered.rows,
            Some(HunkId(0)),
        );
        assert!(matches!(
            action,
            Some(Action::DiscardHunk {
                hunk: HunkId(0),
                ..
            })
        ));
        // Staging is not confirmed.
        let (stage, _) = diff_stage_action(DiffStageAction::Hunk, &mut state, &rendered.rows, None);
        assert!(confirm_first(stage.unwrap()).is_err());
    }

    /// Only the Unstaged half discards: the Staged half and a commit's diff
    /// refuse, each saying why, and send nothing.
    #[test]
    fn discarding_is_refused_on_the_staged_half_and_on_a_commit() {
        let (rendered, _) = staging_rows();
        let mut staged = DiffViewState::for_working(DiffSource::Staged, "multi.txt".into());
        staged.cursor = rendered.hunk_starts[0] + 2;
        let mut commit = DiffViewState::for_commit(oid(3), 0, "multi.txt".into());
        commit.cursor = staged.cursor;
        for action in [DiffStageAction::DiscardHunk, DiffStageAction::DiscardLines] {
            let (sent, why) = diff_stage_action(action, &mut staged, &rendered.rows, None);
            assert!(sent.is_none(), "{action:?} on the Staged half");
            assert_eq!(why, DISCARD_ON_THE_STAGED_HALF.get());
            let (sent, why) = diff_stage_action(action, &mut commit, &rendered.rows, None);
            assert!(sent.is_none(), "{action:?} on a commit");
            assert_eq!(why, NOT_A_WORKING_DIFF_DISCARD.get());
        }
        assert!(!DiffSource::Staged.discards());
        assert!(!DiffSource::Commit.discards());
        assert!(DiffSource::Unstaged.discards());
    }

    /// Discard lines: the selection (a removal and its addition), the
    /// cursor line alone, or an explicit row range (a hovered row's
    /// button, the menu) - in both layouts and with Hunks only on or off,
    /// always the backend's own line indices.
    #[test]
    fn discard_lines_names_the_selected_changes_in_every_layout() {
        let (_, file) = staging_rows();
        let content: Vec<String> = (1..=20)
            .map(|n| match n {
                2 => "line2-changed".to_string(),
                19 => "line19-changed".to_string(),
                n => format!("line{n}"),
            })
            .collect();
        for mode in [
            diff_view::DiffViewMode::Inline,
            diff_view::DiffViewMode::Split,
        ] {
            for hunks_only in [true, false] {
                let rendered = diff_view::render_diff_with(
                    &file,
                    &diff_view::DiffRenderOpts {
                        mode,
                        hunks_only,
                        show_whitespace: true,
                        word_wrap: false,
                        width: 100,
                        h_offset: 0,
                        full_file: (!hunks_only).then_some(&content),
                        lfs_raw: false,
                        lfs_local: None,
                    },
                );
                let label = format!("{mode:?} hunks_only={hunks_only}");
                let mut state =
                    DiffViewState::for_working(DiffSource::Unstaged, "multi.txt".into());
                let header = rendered.hunk_starts[1];
                // The whole second hunk selected: its removal and addition.
                state.selection_anchor = Some(header);
                state.cursor = header + 3;
                let (action, prompt) = diff_stage_action(
                    DiffStageAction::DiscardLines,
                    &mut state,
                    &rendered.rows,
                    None,
                );
                match action {
                    Some(Action::DiscardLines { selector, .. }) => {
                        assert_eq!(selector.hunk, HunkId(1), "{label}");
                        assert_eq!(selector.line_indices, vec![1, 2], "{label}");
                    }
                    other => panic!("{label}: {other:?}"),
                }
                assert!(
                    prompt.starts_with("Discard 2 changed lines"),
                    "{label}: {prompt}"
                );
                assert_eq!(state.selection_anchor, None, "{label}: consumed");

                // One changed row, by explicit range (what the hovered
                // row's `✖ Discard` passes): inline, the addition row
                // alone; split, the paired row holds both.
                let changed = (header..rendered.rows.len())
                    .find(|&r| rendered.rows[r].lines.iter().flatten().any(|&i| i == 2))
                    .unwrap();
                let (action, _) = diff_stage_action_at(
                    DiffStageAction::DiscardLines,
                    &mut state,
                    &rendered.rows,
                    None,
                    Some((changed, changed)),
                );
                let expected = match mode {
                    diff_view::DiffViewMode::Inline => vec![2],
                    diff_view::DiffViewMode::Split => vec![1, 2],
                };
                match action {
                    Some(Action::DiscardLines { selector, .. }) => {
                        assert_eq!(selector.line_indices, expected, "{label}")
                    }
                    other => panic!("{label}: {other:?}"),
                }

                // The same row's `+ Stage` stages exactly those lines.
                let (action, _) = diff_stage_action_at(
                    DiffStageAction::Lines,
                    &mut state,
                    &rendered.rows,
                    None,
                    Some((changed, changed)),
                );
                assert!(
                    matches!(&action, Some(Action::StageLines { selector, .. }) if selector.line_indices == expected),
                    "{label}: {action:?}"
                );

                // A context row has nothing to discard.
                let (action, why) = diff_stage_action_at(
                    DiffStageAction::DiscardLines,
                    &mut state,
                    &rendered.rows,
                    None,
                    Some((header + 1, header + 1)),
                );
                assert!(action.is_none(), "{label}");
                assert!(why.contains("no added or removed lines"), "{label}: {why}");
            }
        }
    }

    /// The Staged half's line-level action is Unstage, from every surface
    /// (keys, a hovered row's `− Unstage`, the menu).
    #[test]
    fn a_staged_rows_line_action_unstages_it() {
        let (rendered, _) = staging_rows();
        let mut state = DiffViewState::for_working(DiffSource::Staged, "multi.txt".into());
        let row = rendered.hunk_starts[0] + 2; // the removal
        let (action, _) = diff_stage_action_at(
            DiffStageAction::Lines,
            &mut state,
            &rendered.rows,
            None,
            Some((row, row)),
        );
        match action {
            Some(Action::UnstageLines { selector, .. }) => {
                assert_eq!(selector.line_indices, vec![1])
            }
            other => panic!("{other:?}"),
        }
    }

    /// The menu opened on a row acts on the selection when the row is in
    /// it, on that row alone otherwise, and counts the changed lines its
    /// line entries would name (0: no line entries).
    #[test]
    fn the_menu_target_is_the_selection_around_the_row_or_the_row() {
        let (rendered, _) = staging_rows();
        let mut state = DiffViewState::for_working(DiffSource::Unstaged, "multi.txt".into());
        let h1 = rendered.hunk_starts[1];
        state.selection_anchor = Some(h1);
        state.cursor = h1 + 3;

        let (target, lines) = diff_menu_target(&state, &rendered.rows, h1 + 2).unwrap();
        assert_eq!(target.hunk, HunkId(1));
        assert_eq!(target.rows, (h1, h1 + 3), "inside the selection: all of it");
        assert_eq!(lines, 2);

        let h0 = rendered.hunk_starts[0];
        let (target, lines) = diff_menu_target(&state, &rendered.rows, h0 + 1).unwrap();
        assert_eq!(target.hunk, HunkId(0));
        assert_eq!(target.rows, (h0 + 1, h0 + 1), "outside it: that row");
        assert_eq!(lines, 0, "a context row: no line entries");

        // The menu actually built for it.
        let (_, menu) = open_diff_hunk_menu(&state, &rendered.rows, h1 + 2, 0, 0).unwrap();
        let labels: Vec<String> = (0..menu.row_count()).map(|r| menu.row_label(r)).collect();
        assert!(
            labels.contains(&"Discard 2 lines".to_string()),
            "{labels:?}"
        );
        assert!(labels.contains(&"Discard hunk".to_string()), "{labels:?}");
    }

    /// Every body button maps to the action its label promises, and is
    /// registered ahead of the row band it sits on.
    #[test]
    fn body_buttons_act_as_labeled_and_win_over_the_row_band() {
        use diff_view::DiffBodyButton as B;
        assert_eq!(
            B::Hunk(HunkId(2)).action(),
            (DiffStageAction::Hunk, Some(HunkId(2)), None)
        );
        assert_eq!(
            B::DiscardHunk(HunkId(2)).action(),
            (DiffStageAction::DiscardHunk, Some(HunkId(2)), None)
        );
        assert_eq!(
            B::Line(7).action(),
            (DiffStageAction::Lines, None, Some((7, 7)))
        );
        assert_eq!(
            B::DiscardLine(7).action(),
            (DiffStageAction::DiscardLines, None, Some((7, 7)))
        );

        let inner = Rect::new(0, 0, 60, 5);
        let mut hits = Vec::new();
        push_diff_body_hits(
            &mut hits,
            inner,
            0,
            5,
            vec![
                (B::DiscardLine(3), Rect::new(40, 3, 11, 1)),
                (B::Line(3), Rect::new(52, 3, 8, 1)),
            ],
        );
        assert_eq!(
            find_hit_target(&hits, 41, 3),
            Some(HitTarget::DiffBodyButton(B::DiscardLine(3)))
        );
        assert_eq!(
            find_hit_target(&hits, 53, 3),
            Some(HitTarget::DiffBodyButton(B::Line(3)))
        );
        assert_eq!(find_hit_target(&hits, 5, 3), Some(HitTarget::DiffRow(3)));
    }
}
