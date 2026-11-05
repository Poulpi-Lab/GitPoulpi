//! The one popup menu a window can have open over its panes.
//!
//! A single enum value cannot hold two menus, so opening any menu replaces
//! whichever was open, and one click-away / Esc path closes whichever it is.
//! The Log's column show/hide checklist is a kind too. (The tab menu lives in
//! `TabCtx` and the remote and hosting menus in their own states; they are
//! modal for the mouse and out of scope here.)

use super::*;

pub(super) type RefMenu = (SidebarSection, String, ContextMenu<RefContextAction>);
pub(super) type LogMenu = (Oid, ContextMenu<LogContextAction>);
pub(super) type FileMenu = (StatusSection, PathBuf, ContextMenu<FileContextAction>);
pub(super) type FolderMenu = (StatusSection, PathBuf, ContextMenu<FolderContextAction>);
/// A file-list menu acting on paths: a commit file, a folder, the empty area
/// or a multi-selection (`pane_menu`).
pub(super) type PaneMenu = (PaneTarget, ContextMenu<PaneAction>);
pub(super) type HistoryMenu = (usize, ContextMenu<crate::history_view::HistoryAction>);
pub(super) type DiffHunkMenu = (DiffMenuTarget, ContextMenu<DiffHunkContextAction>);

#[derive(Default)]
pub(super) enum OpenMenu {
    #[default]
    None,
    Ref(RefMenu),
    Log(LogMenu),
    Wip(ContextMenu<WipContextAction>),
    File(FileMenu),
    Folder(FolderMenu),
    Pane(PaneMenu),
    DiffHunk(DiffHunkMenu),
    History(HistoryMenu),
    /// The commit column header's parts checklist (`commit_column::Part`).
    CommitColumns(ContextMenu<crate::commit_column::Part>),
    Overflow(OverflowMenuState),
    /// The Log header's column show/hide checklist (drawn inline, not a
    /// `ContextMenu`).
    Column(crate::log_table::ColumnMenu),
}

macro_rules! variant_accessors {
    ($($variant:ident($ty:ty): $is:ident, $get:ident, $get_mut:ident, $take:ident, $close:ident;)*) => {
        // The accessor set is uniform per kind; not every kind uses every one.
        #[allow(dead_code)]
        impl OpenMenu {
            $(
                pub(super) fn $is(&self) -> bool {
                    matches!(self, OpenMenu::$variant(_))
                }

                pub(super) fn $get(&self) -> Option<&$ty> {
                    match self {
                        OpenMenu::$variant(menu) => Some(menu),
                        _ => None,
                    }
                }

                pub(super) fn $get_mut(&mut self) -> Option<&mut $ty> {
                    match self {
                        OpenMenu::$variant(menu) => Some(menu),
                        _ => None,
                    }
                }

                /// Takes the menu out when it is this kind, else leaves
                /// whatever is open alone.
                pub(super) fn $take(&mut self) -> Option<$ty> {
                    if self.$is() {
                        match std::mem::take(self) {
                            OpenMenu::$variant(menu) => Some(menu),
                            _ => None,
                        }
                    } else {
                        None
                    }
                }

                /// Closes the menu only when it is this kind.
                pub(super) fn $close(&mut self) {
                    if self.$is() {
                        *self = OpenMenu::None;
                    }
                }
            )*
        }
    };
}

variant_accessors! {
    Ref(RefMenu): is_ref_menu, ref_menu, ref_menu_mut, take_ref_menu, close_ref_menu;
    Log(LogMenu): is_log_menu, log_menu, log_menu_mut, take_log_menu, close_log_menu;
    Wip(ContextMenu<WipContextAction>): is_wip_menu, wip_menu, wip_menu_mut, take_wip_menu, close_wip_menu;
    File(FileMenu): is_file_menu, file_menu, file_menu_mut, take_file_menu, close_file_menu;
    Folder(FolderMenu): is_folder_menu, folder_menu, folder_menu_mut, take_folder_menu, close_folder_menu;
    Pane(PaneMenu): is_pane_menu, pane_menu, pane_menu_mut, take_pane_menu, close_pane_menu;
    DiffHunk(DiffHunkMenu): is_diff_hunk_menu, diff_hunk_menu, diff_hunk_menu_mut, take_diff_hunk_menu, close_diff_hunk_menu;
    History(HistoryMenu): is_history_menu, history_menu, history_menu_mut, take_history_menu, close_history_menu;
    CommitColumns(ContextMenu<crate::commit_column::Part>): is_commit_columns_menu, commit_columns_menu, commit_columns_menu_mut, take_commit_columns_menu, close_commit_columns_menu;
    Overflow(OverflowMenuState): is_overflow, overflow, overflow_mut, take_overflow, close_overflow;
}

impl OpenMenu {
    pub(super) fn is_open(&self) -> bool {
        !matches!(self, OpenMenu::None)
    }

    pub(super) fn close(&mut self) {
        *self = OpenMenu::None;
    }

    pub(super) fn is_column_menu(&self) -> bool {
        matches!(self, OpenMenu::Column(_))
    }

    /// The open chooser: its keyboard cursor (an index into
    /// `log_table::chooser_rows`) and the column its "fit" row acts on.
    pub(super) fn column_menu(&self) -> Option<crate::log_table::ColumnMenu> {
        match self {
            OpenMenu::Column(menu) => Some(*menu),
            _ => None,
        }
    }

    pub(super) fn column_menu_mut(&mut self) -> Option<&mut crate::log_table::ColumnMenu> {
        match self {
            OpenMenu::Column(menu) => Some(menu),
            _ => None,
        }
    }

    pub(super) fn close_column_menu(&mut self) {
        if self.is_column_menu() {
            *self = OpenMenu::None;
        }
    }

    /// The menus no key owner handles before the Normal-mode keys, so
    /// `Esc` there closes them: the file and folder menus (click-driven)
    /// and the column checklist.
    pub(super) fn esc_closes_here(&self) -> bool {
        matches!(
            self,
            OpenMenu::File(_) | OpenMenu::Folder(_) | OpenMenu::Column(_)
        )
    }

    /// A right-click context menu (not the overflow or column menus):
    /// the ones a click anywhere else dismisses without acting.
    pub(super) fn is_context_menu(&self) -> bool {
        !matches!(
            self,
            OpenMenu::None | OpenMenu::Overflow(_) | OpenMenu::Column(_)
        )
    }

    /// Paints whichever context or overflow menu is open and returns its
    /// rows as hit targets, to go in front of the pane targets: a popup
    /// sits over the panes, and `find_hit_target` takes the first match.
    pub(super) fn render(
        &self,
        frame: &mut Frame,
        hovered_row: Option<usize>,
    ) -> Vec<(Rect, HitTarget)> {
        fn rows(rects: Vec<Rect>, target: impl Fn(usize) -> HitTarget) -> Vec<(Rect, HitTarget)> {
            rects
                .into_iter()
                .enumerate()
                .map(|(i, rect)| (rect, target(i)))
                .collect()
        }
        match self {
            OpenMenu::None | OpenMenu::Column(_) => Vec::new(),
            OpenMenu::Ref((_, _, menu)) => rows(
                menu.render(frame, hovered_row),
                HitTarget::RefContextMenuItem,
            ),
            OpenMenu::Log((_, menu)) => rows(
                menu.render(frame, hovered_row),
                HitTarget::LogContextMenuItem,
            ),
            OpenMenu::Wip(menu) => rows(
                menu.render(frame, hovered_row),
                HitTarget::WipContextMenuItem,
            ),
            OpenMenu::File((_, _, menu)) => rows(
                menu.render(frame, hovered_row),
                HitTarget::FileContextMenuItem,
            ),
            OpenMenu::Folder((_, _, menu)) => rows(
                menu.render(frame, hovered_row),
                HitTarget::FolderContextMenuItem,
            ),
            OpenMenu::Pane((_, menu)) => rows(
                menu.render(frame, hovered_row),
                HitTarget::PaneContextMenuItem,
            ),
            OpenMenu::DiffHunk((_, menu)) => rows(
                menu.render(frame, hovered_row),
                HitTarget::DiffHunkContextMenuItem,
            ),
            OpenMenu::History((_, menu)) => rows(
                menu.render(frame, hovered_row),
                HitTarget::HistoryContextMenuItem,
            ),
            OpenMenu::CommitColumns(menu) => {
                let mut hits = rows(menu.render(frame, hovered_row), |i| {
                    HitTarget::CommitColumn(crate::commit_column::ColumnHit::ColumnsMenuItem(i))
                });
                if let Some(cross) = menu.cross_rect() {
                    hits.insert(
                        0,
                        (
                            cross,
                            HitTarget::CommitColumn(
                                crate::commit_column::ColumnHit::ColumnsMenuClose,
                            ),
                        ),
                    );
                }
                hits
            }
            OpenMenu::Overflow(state) => rows(
                state.menu.render(frame, hovered_row),
                HitTarget::BarOverflowMenuItem,
            ),
        }
    }
}

/// Which of the three menu slots a window has were open before an input
/// event: `OpenMenu`, the remote / push / pull menu (`RemoteUi::menu`) and
/// the hosting menu (`HostingUi::menu`). The last two live in their own
/// states, so a single enum cannot keep them exclusive; this does, after
/// every key and mouse event, whichever way the event opened its menu
/// (right-click, keyboard, palette).
#[derive(Clone, Copy)]
pub(super) struct MenuSlots {
    open: bool,
    remote: bool,
    hosting: bool,
}

impl MenuSlots {
    pub(super) fn of(cx: &input::commands::CommandCtx<'_>) -> Self {
        Self {
            open: cx.open_menu.is_open(),
            remote: cx.remote_ui.menu.is_some(),
            hosting: cx.hosting.menu.is_some(),
        }
    }

    /// A menu this event opened closes the ones that were already open.
    pub(super) fn keep_only_the_newest(self, cx: &mut input::commands::CommandCtx<'_>) {
        let now = Self::of(cx);
        let opened = (now.open && !self.open)
            || (now.remote && !self.remote)
            || (now.hosting && !self.hosting);
        if !opened {
            return;
        }
        if self.open {
            cx.open_menu.close();
        }
        if self.remote {
            cx.remote_ui.menu = None;
        }
        if self.hosting {
            cx.hosting.menu = None;
        }
    }
}
