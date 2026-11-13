//! The right panel of the draw closure: the selected commit's detail, or the working-changes status and composer.

mod commit_detail;
mod wip;

use super::super::*;

pub(in crate::event_loop) struct RightPanelDrawCtx<'a> {
    pub(in crate::event_loop) active_status_section: StatusSection,
    pub(in crate::event_loop) ai_ui: &'a AiUi,
    pub(in crate::event_loop) avatar_capability: terminal_image::Capability,
    pub(in crate::event_loop) avatar_live: &'a avatar::LiveSettings,
    pub(in crate::event_loop) avatar_placement: gitpoulpi_config::AvatarPlacement,
    pub(in crate::event_loop) avatar_service: &'a mut avatar::AvatarService,
    pub(in crate::event_loop) log_avatar_images: &'a mut avatar::LogAvatarImages,
    pub(in crate::event_loop) collapsed_commit_folders: &'a HashSet<PathBuf>,
    pub(in crate::event_loop) collapsed_status_folders: &'a HashSet<PathBuf>,
    pub(in crate::event_loop) collapsed_status_sections: &'a HashSet<StatusSection>,
    pub(in crate::event_loop) commit_amend: bool,
    pub(in crate::event_loop) commit_skip_hooks: bool,
    /// A merge is stopped: its commit needs nothing staged.
    pub(in crate::event_loop) merging: bool,
    pub(in crate::event_loop) commit_description: &'a String,
    pub(in crate::event_loop) commit_detail_status_height: &'a mut u16,
    pub(in crate::event_loop) commit_file_focus: bool,
    pub(in crate::event_loop) commit_file_rows: &'a mut Vec<TreeRow>,
    pub(in crate::event_loop) commit_file_state: &'a mut ListState,
    pub(in crate::event_loop) commit_files: &'a Option<crate::daemon_link::CommitFilesOutcome>,
    pub(in crate::event_loop) commit_files_area: &'a mut Rect,
    pub(in crate::event_loop) commit_files_filter: &'a String,
    pub(in crate::event_loop) commit_files_follow: bool,
    pub(in crate::event_loop) commit_files_len: &'a mut usize,
    pub(in crate::event_loop) commit_files_visible_rows: &'a mut usize,
    pub(in crate::event_loop) commit_filter_focus: bool,
    pub(in crate::event_loop) commit_list_mode: FileListMode,
    pub(in crate::event_loop) commit_message_area: &'a mut Rect,
    pub(in crate::event_loop) commit_message_for: &'a mut Option<Oid>,
    pub(in crate::event_loop) commit_message_rows_now: &'a mut u16,
    pub(in crate::event_loop) commit_message_scroll: &'a mut usize,
    pub(in crate::event_loop) commit_message_total: &'a mut usize,
    pub(in crate::event_loop) commit_message_user_rows: u16,
    pub(in crate::event_loop) commit_meta_area: &'a mut Rect,
    pub(in crate::event_loop) commit_paths_cache: &'a mut Vec<PathBuf>,
    pub(in crate::event_loop) commit_ref_urls: &'a mut Vec<(String, bool)>,
    pub(in crate::event_loop) commit_sort_desc: bool,
    pub(in crate::event_loop) commit_summary: &'a String,
    pub(in crate::event_loop) composer_focus: Option<ComposerField>,
    pub(in crate::event_loop) conflicts: &'a Vec<gitpoulpi_core::domain::ConflictEntry>,
    pub(in crate::event_loop) detail_avatar_cols: u16,
    pub(in crate::event_loop) detail_avatar_image: &'a mut avatar::DetailAvatarImage,
    pub(in crate::event_loop) detail_avatar_rows: u16,
    pub(in crate::event_loop) field_editor: &'a crate::text_edit::FieldEditor<MainField>,
    pub(in crate::event_loop) focus: Focus,
    pub(in crate::event_loop) frame_hits: &'a mut Vec<(Rect, HitTarget)>,
    pub(in crate::event_loop) hosting: &'a mut HostingUi,
    pub(in crate::event_loop) hovered_status_row: Option<(StatusSection, usize)>,
    pub(in crate::event_loop) hovered_target: &'a Option<HitTarget>,
    pub(in crate::event_loop) lfs: &'a lfs_ui::LfsUi,
    pub(in crate::event_loop) link_failure: &'a Option<String>,
    pub(in crate::event_loop) log_order: &'a crate::graph::LogRows,
    pub(in crate::event_loop) log_result:
        &'a gitpoulpi_core::Result<gitpoulpi_core::domain::CommitGraph>,
    pub(in crate::event_loop) log_state: &'a TableState,
    pub(in crate::event_loop) overflow_slots: &'a mut OverflowSlots,
    pub(in crate::event_loop) pane_notice: &'a Option<(Vec<String>, Style)>,
    pub(in crate::event_loop) refs_result:
        &'a gitpoulpi_core::Result<Vec<gitpoulpi_core::domain::Ref>>,
    pub(in crate::event_loop) right_panel: RightPanel,
    pub(in crate::event_loop) stashes_result:
        &'a gitpoulpi_core::Result<Vec<gitpoulpi_core::domain::Stash>>,
    pub(in crate::event_loop) status_area: Rect,
    pub(in crate::event_loop) status_list_mode: FileListMode,
    pub(in crate::event_loop) status_result:
        &'a gitpoulpi_core::Result<gitpoulpi_core::domain::WorkingTree>,
    pub(in crate::event_loop) status_section_areas: &'a mut Vec<(StatusSection, Rect)>,
    pub(in crate::event_loop) status_section_lines:
        &'a mut Vec<(StatusSection, Vec<StatusPaneLine>)>,
    pub(in crate::event_loop) status_states: &'a mut StatusStates,
    pub(in crate::event_loop) view_all_files: bool,
    /// The comparison the Log's multi-select stands for, if any.
    pub(in crate::event_loop) compare: &'a Option<crate::compare::CompareSpec>,
    pub(in crate::event_loop) wip_all: &'a mut WipAllFiles,
    /// The files multi-selected in the lists, to paint highlighted.
    pub(in crate::event_loop) file_sel: &'a FileSelection,
}

pub(in crate::event_loop) fn draw_right_panel(frame: &mut Frame, ctx: &mut RightPanelDrawCtx<'_>) {
    // The right panel shows the selected commit's detail (driven by `right_panel`, not
    // `focus` -- see `RightPanel`'s doc comment), or the working changes.
    if let Some((lines, style)) = ctx.pane_notice {
        // Neutral loading text instead of "0 file changes" /
        // "(select a commit)" -- both of those are the ordinary
        // *empty* state, and while the daemon round trip hasn't
        // landed yet, empty is not what this is. Or, once the
        // link has stopped, the failure and how to retry.
        let title = format!(
            " {} ",
            if ctx.link_failure.is_some() {
                gitpoulpi_i18n::msg::commit_detail::LINK_LOST.get()
            } else {
                gitpoulpi_i18n::msg::commit_detail::LOADING_TITLE.get()
            }
        );
        let block = Block::default().title(title).borders(Borders::ALL);
        let inner = block.inner(ctx.status_area);
        frame.render_clipped(block, ctx.status_area);
        render_centered_message(frame, inner, lines, *style);
    } else if ctx.right_panel == RightPanel::CommitDetail {
        commit_detail::draw_commit_detail(frame, ctx);
    } else {
        wip::draw_wip(frame, ctx);
    }
}
