//! The `HitTarget` registry, first-match hit lookup and the mouse-cursor shape.

use super::*;

/// One clickable/hoverable region registered during a frame's render pass
/// and consulted by the next mouse event - a single `Vec<(Rect,
/// HitTarget)>` rebuilt every frame (ratatui doesn't retain a widget tree),
/// so toolbar buttons and per-commit ref pills in the Log table share one
/// mechanism instead of each pane inventing its own.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum HitTarget {
    Toolbar(ToolbarButton),
    /// The small `▾` beside Push (push to a chosen remote, once) and
    /// beside Pull (choose what the Pull button does), a
    /// split button.
    ToolbarDropdown(ToolbarButton),
    /// One row of the open remote menu (`RemoteUi::menu`): a remote's
    /// own menu, a toolbar dropdown, Set upstream, a remote chooser or
    /// the diverged-push dialog.
    RemoteMenuItem(usize),
    /// A tab, field or button of the Add / Edit remote popup.
    RemoteForm(RemoteFormHit),
    /// The text area or a button of the Edit commit message popup.
    RewordForm(RewordPart),
    /// The `+` on the REMOTE section's header: Add remote.
    AddRemoteButton,
    /// The Status pane's "Git LFS is not installed" banner: opens the
    /// install page (the palette's `LFS: Open install page` is its
    /// keyboard twin).
    LfsInstallLink,
    /// The gear in the title bar, immediately left of the quit button with
    /// a gap of blank cells (a click aimed at it never quits), which opens
    /// the Preferences screen (`Ctrl+,`, or the palette's
    /// `App: Settings...`).
    ///
    /// Registered by `render_gear` from the quit button's rect, and
    /// suppressed entirely when the title (and "Open in") would reach it, so
    /// a narrow window loses the gear rather than painting it over them.
    Gear,
    /// The 🐙 at the title bar's left: opens the About window (the palette's
    /// `Help: About GitPoulpi` is its keyboard twin).
    About,
    /// The toolbar's profile combobox field (palette twin
    /// `Repository: Change Profile...`).
    ProfileField,
    /// A row of the open profile popup, from the rects its paint walk made.
    ProfileItem(usize),
    /// The profile popup's frame: nothing behind it hovers.
    ProfilePopup,
    /// The tab bar's workspace chip (palette twin `Workspace: Switch...`,
    /// `Alt+W`). Registered from the rect the tab bar's paint walk produced,
    /// and not at all when it did not fit.
    WorkspaceChip,
    /// A row of the open workspace popup, from the rects its paint walk made.
    WorkspaceItem(usize),
    /// The workspace popup's frame: nothing behind it hovers.
    WorkspacePopup,
    /// The `\u{2715} Esc` affordance on the Diff view's frame. The label
    /// always said the view was closable; only the keyboard half of it
    /// was ever wired.
    CloseDiff,
    /// The folder button at the tab bar's left-hand end: the Repository
    /// Management tab (`Ctrl+O`, or the palette's
    /// `Repository: Open...`). The maintainer asked for it in exactly this
    /// corner.
    ///
    /// Registered from the rect `repo_switcher::tab_bar` produced while
    /// drawing it - never a second "two columns at the left edge"
    /// computation - and not registered at all when the bar was too
    /// narrow to draw it, which is the gear's own rule.
    OpenRepositoryButton,
    /// The `+` right after the last repository tab (`Alt+T`, the palette's
    /// `Tab: New Tab`): a transient New tab. Registered from the rect the tab
    /// bar's paint walk produced, and not at all when it did not fit.
    NewTabButton,
    /// The `...` after the last tab that fit: opens the tab picker.
    TabOverflow,
    /// The Radar's pinned tab, right after Home (`Alt+R`, the palette's
    /// `Radar: Open`). Registered from the tab bar's paint walk.
    RadarTab,
    /// The Radar tab's `x`: closes it for this session.
    RadarClose,
    /// The Workspaces tab, right after the Radar's (open only; `Alt+J`, the
    /// `+` menu, the palette's `Workspace: Manage...`).
    WorkspacesTab,
    /// The Workspaces tab's `x`: closes it (remembered per session).
    WorkspacesClose,
    /// The Release notes tab, right after Home and the Radar (open only).
    NotesTab,
    /// The Release notes tab's `x`.
    NotesClose,
    /// The tab bar's Search button: the global search (`Ctrl+Alt+F`, the
    /// palette's `Search: Workspace...`). Registered from its paint.
    GlobalSearchButton,
    /// "Quit & cd here" / "Terminal here" at the tab bar's right-hand end,
    /// shown while the active tab is not the launch folder
    /// (`ui.tab_dir_follow = "buttons"`). Registered from the rects
    /// `render_tab_dir_buttons` painted.
    TabDir(crate::tab_dir::TabDirButton),
    /// A row of the open "Open in" popup, registered from the rects its paint
    /// walk produced (`open_in_bar::render_popup`).
    OpenInItem(usize),
    /// The "Open in" popup's frame and search line: nothing behind it hovers.
    OpenInPopup,
    /// The "Open in" button of a binary file in the Diff view and the File
    /// View (`open_with::view`), from the rect its paint produced.
    OpenWith,
    /// The primary ref pill rendered in a Log row's Ref column - a
    /// double-click checks it out exactly like a Refs-sidebar row does.
    LogRefPill {
        name: String,
        kind: RefRowKind,
    },
    /// A Refs sidebar section's header line (its chevron+name+count row) -
    /// a click toggles that section's collapsed state, the mouse
    /// equivalent of Enter/Space on the section while it's active.
    RefsSectionHeader(SidebarSection),
    /// The Refs sidebar's "Stop Soloing" button, shown in the outer
    /// block's title only while one or more refs are soloed - a click
    /// clears the whole solo set at once.
    StopSoloing,
    /// The Refs sidebar's "Show All" button, shown in the outer block's
    /// title (in Stop Soloing's slot) only while at least one ref is
    /// hidden and none is soloed - a click unhides every ref at once.
    ShowAllHiddenRefs,
    /// One ref listed inside the commit-hover tooltip's "other refs on
    /// this commit" list (only rendered when a commit has more than one
    /// ref pointing at it) - a double-click checks it out exactly like a
    /// Log-row/Refs-sidebar pill does. Registered per-line since the
    /// tooltip lists one ref per row.
    TooltipRefPill {
        name: String,
        kind: RefRowKind,
    },
    /// The Log table header's thin boundary strip at the right edge of a
    /// column - a mouse-down here starts a resize drag for that column
    /// (`Message` is never resizable, so it never gets one), spanning the
    /// column's full height (not just the header row) like the pane
    /// dividers already do, for the same easy-to-grab reason.
    LogColumnBoundary(LogColumn),
    /// The bottom border row of an expanded Refs sidebar section: a
    /// mouse-down here starts a drag that resizes that section (the
    /// separators between LOCAL / REMOTE / ...). Answered by
    /// `resize_grab_at`, like the column boundaries.
    SidebarSectionBoundary(SidebarSection),
    /// The boundary strip immediately to the LEFT of a resizable column
    /// whose left neighbor is `Message` (the one column that's never
    /// itself adjustable - see `LogColumn::adjustable`). `Message` has no
    /// stored width override to shrink/grow, so this boundary drags the
    /// *neighbor's* width instead, anchored to the neighbor's own right
    /// edge (growing left eats into `Message`, not the other way around).
    /// Without this, the Message/Author boundary had no hit target at all -
    /// dragging there silently did nothing.
    /// A Log table header cell itself (not its boundary) - a mouse-down
    /// followed by a drag past a neighboring header reorders columns; a
    /// plain click is a no-op (sorting-by-column isn't implemented); a
    /// right-click here opens the column show/hide menu instead (handled
    /// by checking the header row directly, not through this hit target,
    /// since right-click is a different `MouseEventKind` variant).
    LogColumnHeader(LogColumn),
    /// One row of the column show/hide menu (opened by right-clicking the
    /// Log header) - a click toggles that column's visibility and leaves
    /// the menu open, the usual column-picker convention.
    /// `Message` never gets one (see `LogColumn::adjustable`).
    ColumnMenuItem(LogColumn),
    /// One action row of the Log's column chooser (fit a column or all of
    /// them, reset the widths): a click runs it and closes the chooser.
    ColumnMenuAction(ChooserRow),
    /// The cross in the Log's column chooser's title.
    ColumnMenuClose,
    /// One row of a ref's right-click context menu (`ref_context_menu`),
    /// indexed into whatever `ref_context_actions` returned for that ref's
    /// kind - clicking it runs that action and closes the menu.
    RefContextMenuItem(usize),
    /// One row of a commit's right-click context menu
    /// (`log_context_menu`).
    LogContextMenuItem(usize),
    /// One row of the Log's WIP row's own right-click context menu
    /// (`wip_context_menu`): create a patch from the working tree, apply
    /// one, and the cheap existing whole-tree actions (stash, stage all,
    /// discard all). Its own `HitTarget`, not `LogContextMenuItem`,
    /// because the WIP row has no `Oid` to key that menu's state on.
    WipContextMenuItem(usize),
    /// One row of the commit-detail changed-files list's own menu
    /// (`commit_file_context_menu`), indexed into
    /// `commit_file_context_entries`.
    PaneContextMenuItem(usize),
    /// One row of the Diff view's per-hunk menu
    /// (`diff_hunk_context_menu`), indexed into
    /// `diff_hunk_context_entries`.
    DiffHunkContextMenuItem(usize),
    /// One row of the File History list's right-click menu (`history_view::
    /// actions::menu_entries`).
    HistoryContextMenuItem(usize),
    /// A row of the File History list, or the splitter beside it
    /// (`history_view::HistoryHit`).
    History(crate::history_view::HistoryHit),
    /// A row of the commit column left of the Diff / File View / Blame /
    /// Edit views, its splitter or its close cross (`commit_column`).
    CommitColumn(crate::commit_column::ColumnHit),
    /// One button in the central Diff view's top bar (`diff_view::
    /// DIFF_BAR_BUTTONS`). Registered from the same walk that draws the
    /// bar, so a button's rect can't drift from where it was painted.
    /// Disabled buttons still get a hit target on purpose - clicking one
    /// reports *why* it's inert (`DiffBarButton::pending_reason`) instead
    /// of swallowing the click in silence.
    DiffBarButton(DiffBarButton),
    /// A button inside the Diff view's body, revealed on hover exactly
    /// like the third pane's per-row button: a hunk header's `Stage Hunk`
    /// / `Unstage Hunk` / `Discard Hunk`, or a hovered changed row's
    /// `+ Stage` / `− Unstage` / `✖ Discard`. A hunk
    /// button carries the hunk's own id - the number the backend's diff
    /// gave it - rather than a row index, because that is what
    /// `StageTarget::Hunk` names and it survives the re-render the
    /// staging itself triggers.
    DiffBodyButton(diff_view::DiffBodyButton),
    /// One rendered row of the open diff, as an index into the rows the
    /// last frame produced. Clicking lands the cursor there; dragging
    /// from one to another sweeps a line selection.
    DiffRow(usize),
    /// ↑ / ↓ / ✕ in the Log's find bar. Registered from the same
    /// `(x_offset, width, button)` list the bar renders from, so the
    /// clickable rects can't drift from the painted glyphs.
    SearchBarButton(SearchBarButton),
    /// The third pane's `Path`/`Tree` segmented control. Carries the mode
    /// the half that was clicked selects, so clicking the already-active
    /// half is a no-op rather than a toggle - segmented controls don't
    /// toggle, they select.
    FileListModeButton(FileListMode),
    /// A working-changes section header's bulk button: `Stage All
    /// Changes` on the unstaged section, `Unstage All Changes` on the
    /// staged one.
    StatusBulkButton(StatusSection),
    /// A section header row itself - clicking collapses/expands it.
    StatusSectionHeader(StatusSection),
    /// The per-row `Stage File` / `Unstage File` button, revealed on
    /// hover. Carries the row's own section so a partially-staged file
    /// (which appears in both) stages or unstages according to the half
    /// that was actually clicked.
    StatusRowButton {
        section: StatusSection,
        path: PathBuf,
    },
    /// A folder row in Tree mode - clicking collapses/expands it.
    StatusFolderRow(PathBuf),
    /// A folder row's hover button, staging or unstaging the whole
    /// directory via `StageTarget::Folder`.
    StatusFolderButton {
        section: StatusSection,
        path: PathBuf,
    },
    /// `Collapse All` / `Expand All` above a file tree.
    CollapseAllButton {
        /// `true` collapses everything, `false` expands.
        collapse: bool,
    },
    /// One row of a file's right-click context menu, indexed into
    /// the open `Menu` (`status_pane::file_context_entries`), screen rows
    /// in order - separators and grayed rows register one too.
    FileContextMenuItem(usize),
    /// One row of a folder's right-click context menu,
    /// indexed into `folder_context_actions` for that row's section.
    FolderContextMenuItem(usize),
    /// The global "Discard all changes" trash button in the
    /// working-changes toolbar - always confirmed,
    /// over every Unstaged path.
    StatusDiscardAllButton,
    /// The commit composer's `Amend previous commit` checkbox.
    AmendCheckbox,
    /// The composer's Skip hooks checkbox (`--no-verify` for this commit).
    SkipHooksCheckbox,
    /// The composer's AI "Generate" button (only exists while `ai.enabled`).
    AiGenerate,
    /// The composer's "Compose commits with AI" button (opens the composer).
    AiCompose,
    /// The composer's `#` button: pick an issue reference to insert into the
    /// commit message (grayed, saying why, when no issues are linked).
    ComposerIssueRef,
    /// The working-changes header's right-hand AI "Explain working changes"
    /// sparkle button.
    StatusExplainButton,
    /// The commit composer's summary / description fields and its commit
    /// button.
    CommitSummaryField,
    CommitDescriptionField,
    CommitButton,
    /// The Log's WIP row's inline message field - the same Summary as
    /// `CommitSummaryField`, edited in place in the Log.
    WipMessageField,
    /// The commit-detail pane's `Filter files` box.
    CommitFilterField,
    /// The commit-detail toolbar's path-order toggle (A->Z / Z->A).
    CommitSortButton,
    /// The Swap button of a comparison's header (`compare`).
    CompareSwap,
    /// A card of the comparison panel; the index is into `CompareSpec::cards`.
    CompareCard(usize),
    /// A `#123` / `!45` in the commit message block; the index is into the
    /// event loop's per-frame URL list.
    CommitReference(usize),
    /// The author's name on the commit-detail pane's author line; its
    /// tooltip is the author's email (`tooltip::author_email`).
    CommitAuthorName,
    /// A text band of the commit-detail pane that names one property of the
    /// commit: its summary or description lines (`AmendScope::Summary` /
    /// `Description`), the authored-date line (`Date`) and the author block
    /// (`Author`). A right click opens that property's Amend menu and a
    /// double click opens the full Amend popup focused on it (the keyboard twins are `Ctrl+G Shift+E` /
    /// `Ctrl+G Shift+A` and the palette's `Commit: Edit ...` entries).
    CommitDetailPart(AmendScope),
    /// The `commit <id>` line of the commit-detail pane: a click copies the
    /// short id (the palette's `Copy short SHA`).
    CommitIdLine,
    /// The bottom border of the commit message block: a mouse-down here
    /// starts a drag that resizes it. Answered by `resize_grab_at`.
    CommitMessageBoundary,
    /// The `View all files` checkbox: the commit-detail pane's, and the
    /// working-changes toolbar's - one toggle.
    ViewAllFilesCheckbox,
    /// The Confirm/Cancel button in the `InputMode::Confirm` popup
    /// (`true` = confirm/delete, `false` = cancel) - a real modal with
    /// clickable buttons, not just a status-bar line with keyboard-only
    /// y/n, per "il faut que ça ressemble à une vraie popup".
    ConfirmButton(bool),
    /// The close cross of that popup: closes like Cancel.
    ConfirmClose,
    /// The chevron on the Refs or Status pane's top border that collapses
    /// it, handing its width to the Log pane in the middle - "collapse au
    /// besoin les panes de gauche et de droite pour laisser la place au
    /// center". Suppressed when the pane is too narrow to draw it, the
    /// gear's own rule.
    CollapsePane(PaneSide),
    /// The thin strip left where a collapsed pane used to be. A click
    /// restores it to whatever width it had before - `divider1_pct`/
    /// `divider2_pct` are never touched by collapsing, only ignored for a
    /// frame, so there is nothing to recompute here.
    RestorePane(PaneSide),
    /// The `\u{D7}` at the outer title bar's top-right corner - mouse
    /// parity for `q`, in the window-chrome corner. Registered
    /// only when it would not overlap the title text,
    /// the gear's own "no room, no button" rule.
    CloseApp,
    /// The thumb of a scrollbar on a scrollable central view (the diff,
    /// File View/Blame/History) - lights up on hover and starts a drag
    /// on mouse-down. See `scrollbar.rs`.
    ScrollbarThumb(ScrollBar),
    /// A header button of the image view (zoom out / in, Fit, 1:1).
    #[cfg_attr(not(feature = "image-view"), allow(dead_code))]
    Image(crate::image_view::pane::ImageHit),
    /// A scrollbar's track outside the thumb - a mouse-down here pages
    /// toward the click, like PageUp/PageDown.
    ScrollbarTrack(ScrollBar),
    /// The `⋯` a responsive button bar draws when its buttons do not all
    /// fit (`button_bar.rs`). A click opens the menu of
    /// the hidden buttons.
    BarOverflow(OverflowBar),
    /// One row of that menu. Registered at the *front* of the frame's
    /// hit list, since the menu is drawn over the panes and
    /// `find_hit_target` returns the first match.
    BarOverflowMenuItem(usize),
    /// The built-in editor's bar buttons, `✕ Esc`, Save / Discard /
    /// Cancel prompt and scrollbar thumbs.
    Editor(crate::editor::EditorHit),
    /// The Blame view's annotation cells, its resize border and its card /
    /// menu rows (`blame_view::render`). The popup's rects are registered at
    /// the *front* of the frame's hit list.
    Blame(crate::blame_view::BlameHit),
    /// The merge conflict editor's buttons, checkboxes, line gutters,
    /// badges, tabs, splitter and scrollbar thumbs
    /// (`merge_view::MergeView::render`).
    Merge(crate::merge_view::MergeHit),
    /// A row, action pill or button of the interactive-rebase /
    /// cherry-pick popup (`rebase_ui::RebaseForm`).
    RebaseForm(crate::rebase_ui::RebasePart),
    /// A part of the AI commit-composer review popup (`compose_ui.rs`).
    ComposeForm(crate::compose_ui::ComposePart),
    /// A part of the split-commit editor (`split_ui/render.rs`).
    SplitForm(crate::split_ui::SplitHit),
    /// A part of the history rewrite workbench (`rewrite_view/render.rs`).
    Rewrite(crate::rewrite_view::RewriteHit),
    /// Continue/Skip/Abort on the strip above the Log shown while a
    /// rebase is in progress (`rebase_banner.rs`).
    RebaseBanner(crate::rebase_banner::RebaseBannerButton),
    /// The merge / cherry-pick / revert banner above the Log
    /// (`operation_banner.rs`).
    OperationBanner(crate::operation_banner::OperationBannerButton),
    /// The `git am` strip or the stale `index.lock` strip above the Log
    /// (`stuck_banner.rs`).
    StuckBanner(crate::stuck_banner::StuckBannerButton),
    /// The Diff view's `Merge Editor` button, on a conflicted file.
    MergeEditorButton,
    /// Pop now / Show stash / Drop on the pending auto-stash strip
    /// (`autostash_banner.rs`).
    AutoStashBanner(crate::autostash_banner::AutoStashBannerButton),
    /// The proactive conflict badge's button (`conflict_badge.rs`): opens the
    /// predicted conflicts, like the palette's Conflicts: Show predicted
    /// conflicts.
    ConflictBadge,
    /// A pull-request / issue row, a section's filter chip, a row of the
    /// hosting menu, or the toolbar's linked-PR chip (`hosting_ui`).
    Hosting(HostingHit),
    /// A repository tab, its close cross, or the Home tab (index into
    /// `tab_hits`). The tab bar keeps its own rect list (`repo_switcher::
    /// TabHit`) that the clicks resolve against, so this is never
    /// registered in `hit_targets`: the pointer path builds it from that
    /// list so the hand has one rule (`desired_cursor_shape`) for tabs and
    /// buttons alike.
    RepoTab(usize),
    /// A row of a list whose rows are found by geometry, not registered
    /// (a working-changes file): a click selects it. It lights on hover and
    /// keeps the arrow. Only the pointer path builds it.
    ListRow,
    /// A control found by geometry, not registered (a Refs folder row that
    /// folds, the status bar's indicators): the hand. Only the pointer path
    /// builds it.
    VirtualHand,
}

/// The first registered hit target whose `Rect` contains `(col, row)` --
/// pulled out of the mouse-down handler as a pure function (no herdr mouse
/// injection exists to exercise it through a live terminal, so it's
/// covered by `#[cfg(test)]` unit tests directly instead of "code
/// reviewed").
pub(crate) fn find_hit_target(
    hit_targets: &[(Rect, HitTarget)],
    col: u16,
    row: u16,
) -> Option<HitTarget> {
    hit_targets
        .iter()
        .find(|(r, _)| r.contains((col, row).into()))
        .map(|(_, t)| t.clone())
}

/// Like `find_hit_target`, but keeps the rect as well.
///
/// A ref drag has to *light* the pill it is hovering, not merely name it,
/// and the rect it lights must be the one that was registered - deriving
/// it again from the pill's own text would re-open the class of bug where
/// the drawn edge and the hit-tested edge disagree. Same first-match rule
/// as `find_hit_target`, which keeps its own cheaper shape because every
/// other call site wants only the target.
pub(crate) fn find_hit_entry(
    hit_targets: &[(Rect, HitTarget)],
    col: u16,
    row: u16,
) -> Option<(Rect, HitTarget)> {
    hit_targets
        .iter()
        .find(|(r, _)| r.contains((col, row).into()))
        .map(|(r, t)| (*r, t.clone()))
}

/// The OS pointer shape to request (see `cursor.rs`) for the current
/// mouse position, in priority order: an active drag pins its own shape
/// regardless of exactly where the pointer has wandered since (dragging
/// past a boundary shouldn't flicker the cursor back to `Pointer`, and a
/// ref being carried over a button is still being carried); otherwise a
/// a resize grab strip under the pointer *right now*, as decided by the
/// one predicate `resize_grab_at` (which also decides what lights up and
/// what a press grabs - see its doc comment); a hit target under the
/// cursor right now; a hovered Refs-sidebar leaf row (checked separately -
/// plain ref rows aren't registered in `hit_targets` today, only
/// section headers are); anything else is the plain arrow.
///
/// `resize_grab` covers pane dividers *and* Log column boundaries, so
/// `HitTarget::LogColumnBoundary` deliberately falls through the match
/// below rather than answering for itself: two answers to "is this
/// resizable" is how the shape and the highlight drifted apart before.
///
/// The two hand shapes are *not* interchangeable and were once inverted
/// here. A ref under the pointer is a clickable thing like every row and
/// button in this UI, so it takes `Pointer`; `Grab`, the open hand, means
/// "you are carrying this" and so belongs to `dragging_ref` alone. See
/// `CursorShape::Grab`.
pub(crate) fn desired_cursor_shape(
    hit: Option<&HitTarget>,
    hovered_ref_row: Option<&str>,
    dragging_divider: Option<u8>,
    dragging_log_boundary: Option<LogBoundaryDrag>,
    dragging_ref: bool,
    resize_grab: Option<ResizeGrab>,
) -> CursorShape {
    if dragging_divider.is_some() || dragging_log_boundary.is_some() {
        return CursorShape::ColResize;
    }
    // Ahead of the hit targets below, for the same reason the resize drag
    // is: while a ref is in the air, what happens to be under the pointer
    // (a pill, a toolbar button, nothing at all) must not change the fact
    // that something is being carried.
    if dragging_ref {
        return CursorShape::Grab;
    }
    // The single predicate, ahead of the hit targets: exactly the cells
    // that light up and exactly the cells a press can grab, never a cell
    // more.
    if let Some(grab) = resize_grab {
        return match grab {
            ResizeGrab::SidebarSection(_) | ResizeGrab::CommitMessage => CursorShape::RowResize,
            _ => CursorShape::ColResize,
        };
    }
    match hit {
        // A text box of a form: the I-beam (one rule for every input box).
        Some(
            HitTarget::RewordForm(p)
            | HitTarget::RebaseForm(crate::rebase_ui::RebasePart::Editing(p))
            | HitTarget::ComposeForm(crate::compose_ui::ComposePart::Editing(p))
            | HitTarget::SplitForm(crate::split_ui::SplitHit::Edit(p)),
        ) if p.is_field() => return CursorShape::Text,
        Some(HitTarget::RemoteForm(crate::remotes_ui::RemoteFormHit::Part(
            crate::remotes_ui::RemoteFormPart::Name
            | crate::remotes_ui::RemoteFormPart::FetchUrl
            | crate::remotes_ui::RemoteFormPart::PushUrl,
        ))) => return CursorShape::Text,
        // A row of a dialog list (a rebase step, a proposed commit, a changed
        // file, a rewrite rule): the arrow, lit as a whole. Its buttons and
        // checkboxes are their own hit targets and keep the hand.
        Some(
            HitTarget::RebaseForm(crate::rebase_ui::RebasePart::Row(_))
            | HitTarget::ComposeForm(
                crate::compose_ui::ComposePart::Row(_)
                | crate::compose_ui::ComposePart::PaneFile(_),
            )
            | HitTarget::SplitForm(
                crate::split_ui::SplitHit::Part(_)
                | crate::split_ui::SplitHit::Row(_)
                | crate::split_ui::SplitHit::Modal,
            )
            | HitTarget::Rewrite(
                crate::rewrite_view::RewriteHit::Pane(_) | crate::rewrite_view::RewriteHit::Rule(_),
            ),
        ) => {}
        Some(
            HitTarget::Toolbar(_)
            | HitTarget::ToolbarDropdown(_)
            | HitTarget::RemoteForm(_)
            | HitTarget::RewordForm(_)
            | HitTarget::AddRemoteButton
            | HitTarget::LfsInstallLink
            | HitTarget::Gear
            | HitTarget::About
            | HitTarget::ProfileField
            | HitTarget::ProfilePopup
            | HitTarget::WorkspaceChip
            | HitTarget::WorkspacePopup
            | HitTarget::CloseDiff
            | HitTarget::OpenRepositoryButton
            | HitTarget::NewTabButton
            | HitTarget::TabOverflow
            | HitTarget::RadarTab
            | HitTarget::RadarClose
            | HitTarget::WorkspacesTab
            | HitTarget::WorkspacesClose
            | HitTarget::NotesTab
            | HitTarget::NotesClose
            | HitTarget::GlobalSearchButton
            | HitTarget::TabDir(_)
            | HitTarget::OpenInPopup
            | HitTarget::OpenWith
            | HitTarget::LogRefPill { .. }
            | HitTarget::RefsSectionHeader(_)
            | HitTarget::StopSoloing
            | HitTarget::ShowAllHiddenRefs
            | HitTarget::TooltipRefPill { .. }
            | HitTarget::LogColumnHeader(_)
            | HitTarget::ColumnMenuItem(_)
            | HitTarget::ColumnMenuAction(_)
            | HitTarget::ColumnMenuClose
            | HitTarget::ConfirmButton(_)
            | HitTarget::ConfirmClose
            | HitTarget::FileListModeButton(_)
            | HitTarget::CommitSortButton
            | HitTarget::CommitIdLine
            | HitTarget::CompareSwap
            | HitTarget::CommitReference(_)
            | HitTarget::StatusBulkButton(_)
            | HitTarget::StatusSectionHeader(_)
            | HitTarget::StatusRowButton { .. }
            | HitTarget::StatusFolderRow(_)
            | HitTarget::StatusFolderButton { .. }
            | HitTarget::CollapseAllButton { .. }
            | HitTarget::StatusDiscardAllButton
            | HitTarget::AmendCheckbox
            | HitTarget::SkipHooksCheckbox
            | HitTarget::AiGenerate
            | HitTarget::AiCompose
            | HitTarget::ComposerIssueRef
            | HitTarget::StatusExplainButton
            | HitTarget::CommitButton
            | HitTarget::ViewAllFilesCheckbox
            | HitTarget::CollapsePane(_)
            | HitTarget::RestorePane(_)
            | HitTarget::CloseApp
            | HitTarget::BarOverflow(_)
            | HitTarget::RebaseForm(_)
            | HitTarget::ComposeForm(_)
            | HitTarget::SplitForm(_)
            | HitTarget::Rewrite(_)
            | HitTarget::RebaseBanner(_)
            | HitTarget::OperationBanner(_)
            | HitTarget::StuckBanner(_)
            | HitTarget::MergeEditorButton
            | HitTarget::AutoStashBanner(_)
            | HitTarget::ConflictBadge
            | HitTarget::RepoTab(_)
            | HitTarget::VirtualHand,
        ) => return CursorShape::Pointer,
        // A row (a menu row, a card, a list row found by geometry) is not a
        // control: it has a hover highlight, the pointer stays the arrow.
        Some(
            HitTarget::RemoteMenuItem(_)
            | HitTarget::ProfileItem(_)
            | HitTarget::WorkspaceItem(_)
            | HitTarget::OpenInItem(_)
            | HitTarget::RefContextMenuItem(_)
            | HitTarget::LogContextMenuItem(_)
            | HitTarget::WipContextMenuItem(_)
            | HitTarget::PaneContextMenuItem(_)
            | HitTarget::DiffHunkContextMenuItem(_)
            | HitTarget::HistoryContextMenuItem(_)
            | HitTarget::FileContextMenuItem(_)
            | HitTarget::FolderContextMenuItem(_)
            | HitTarget::BarOverflowMenuItem(_)
            | HitTarget::CompareCard(_)
            | HitTarget::ListRow,
        ) => {}
        // A text field: the I-beam.
        Some(
            HitTarget::CommitSummaryField
            | HitTarget::CommitDescriptionField
            | HitTarget::WipMessageField
            | HitTarget::CommitFilterField,
        ) => return CursorShape::Text,
        // A disabled bar button is still a button: a pointer over it (plus
        // the message it prints on click) reads as "control, currently
        // unavailable", where the plain arrow would read as dead chrome.
        Some(HitTarget::DiffBarButton(_)) => return CursorShape::Pointer,
        Some(HitTarget::DiffBodyButton(_)) => return CursorShape::Pointer,
        Some(HitTarget::SearchBarButton(_)) => return CursorShape::Pointer,
        Some(HitTarget::ScrollbarThumb(_)) => return CursorShape::Pointer,
        // The image diff's divider is a resize handle, like the merge editor's.
        Some(HitTarget::Image(crate::image_view::pane::ImageHit::Divider)) => {
            return CursorShape::ColResize
        }
        Some(HitTarget::Image(_)) => return CursorShape::Pointer,
        Some(HitTarget::ScrollbarTrack(_)) => return CursorShape::Pointer,
        Some(HitTarget::Editor(_)) => return CursorShape::Pointer,
        Some(HitTarget::Blame(crate::blame_view::BlameHit::Handle)) => {
            return CursorShape::ColResize
        }
        Some(HitTarget::Blame(crate::blame_view::BlameHit::Op(_))) => return CursorShape::Pointer,
        Some(HitTarget::Blame(_)) => {}
        // The merge editor's resize handles take the resize pointer, not
        // the hand: the Output's title rule moves up and down, the
        // dividers between A / B / Base sideways.
        Some(HitTarget::Merge(crate::merge_view::MergeHit::Splitter)) => {
            return CursorShape::RowResize
        }
        Some(HitTarget::Merge(crate::merge_view::MergeHit::Divider(_))) => {
            return CursorShape::ColResize
        }
        // The Output is the editable result: the I-beam.
        Some(HitTarget::Merge(crate::merge_view::MergeHit::OutputLine(_))) => {
            return CursorShape::Text
        }
        Some(HitTarget::Merge(_)) => return CursorShape::Pointer,
        // A diff row is a text band to read and select, not a control: the
        // hand here would claim every line of every diff is a button.
        Some(HitTarget::DiffRow(_)) => {}
        // A history row is a list row, a click selects it: the hand, like
        // the cross. Its splitter is answered by `resize_grab` above.
        Some(HitTarget::History(crate::history_view::HistoryHit::Divider)) => {}
        Some(HitTarget::History(crate::history_view::HistoryHit::Row(_))) => {}
        Some(HitTarget::History(_)) => return CursorShape::Pointer,
        // A commit row is a plain table row like the Log's: the arrow (only
        // a ref pill takes the hand). Its cross, its reopen strip, its header
        // parts and the parts menu are clickable; the splitter and the
        // boundaries are answered by `resize_grab` above.
        Some(HitTarget::CommitColumn(
            crate::commit_column::ColumnHit::Row(_)
            | crate::commit_column::ColumnHit::Divider
            | crate::commit_column::ColumnHit::Boundary(_),
        )) => {}
        Some(HitTarget::CommitColumn(crate::commit_column::ColumnHit::ColumnsMenuItem(_))) => {}
        Some(HitTarget::CommitColumn(_)) => return CursorShape::Pointer,
        // The hosting lists' rows and the hosting menu's rows light on hover.
        Some(HitTarget::Hosting(
            HostingHit::Row(..) | HostingHit::MenuItem(_) | HostingHit::PickerItem(_),
        )) => {}
        Some(HitTarget::Hosting(_)) => return CursorShape::Pointer,
        // The author name only carries a tooltip: text, not a control.
        Some(HitTarget::CommitAuthorName | HitTarget::CommitDetailPart(_)) => {}
        // Answered by `resize_grab` above, on purpose -- see the doc
        // comment.
        Some(
            HitTarget::LogColumnBoundary(_)
            | HitTarget::SidebarSectionBoundary(_)
            | HitTarget::CommitMessageBoundary,
        ) => {}
        None => {}
    }
    // A ref row is a row like every other: the arrow, with the row lit (the
    // open hand stays reserved for a ref being carried).
    let _ = hovered_ref_row;
    CursorShape::Default
}
