//! The sensitive-files question's "Show in file": the window opens the
//! finding's file in the File View, scrolled to and highlighting the line
//! where the secret sits, and brings the question back when the File View
//! closes (`stage_guard_dialog` keeps the state of that round trip).

use crate::event_loop::*;
use crate::file_view::{FileInsight, FileInsightData};
use init::WindowState;

impl WindowState {
    /// One step per tick: open the asked file, scroll to its line once it has
    /// loaded, end the round trip when the File View is gone.
    pub(in crate::event_loop) fn sync_stage_guard_peek(&mut self) {
        if !crate::stage_guard_dialog::peeking() {
            return;
        }
        if let Some((path, line)) = crate::stage_guard_dialog::peek_request(&self.repo.root) {
            let head = head_oid(self.refs_result.as_deref().unwrap_or(&[]));
            let request = crate::diff_actions::wip_insight_request(
                FileInsightKind::Content,
                PathBuf::from(path),
                false,
                head,
            );
            self.link.request_file_insight(request.clone());
            self.diff_view_state = None;
            self.file_insight = Some(FileInsight::loading(request));
            self.central_view = CentralView::File;
            if line.is_none() {
                crate::stage_guard_dialog::peek_applied();
            }
            crate::redraw::poke();
            return;
        }
        let Some(insight) = self.file_insight.as_mut() else {
            crate::stage_guard_dialog::peek_closed();
            return;
        };
        let Some(line) = crate::stage_guard_dialog::peek_line() else {
            return;
        };
        match &insight.data {
            FileInsightData::Loading => {}
            FileInsightData::Failed(_) => crate::stage_guard_dialog::peek_applied(),
            _ => {
                let len = insight.len();
                if len > 0 {
                    let viewport =
                        (self.file_insight_area.height.saturating_sub(2) as usize).max(1);
                    let row = (line as usize).saturating_sub(1).min(len - 1);
                    insight.selected = row;
                    insight.offset = super::agent_look::scroll_for(row, viewport);
                    insight.clamp(viewport);
                    insight.highlight = Some((row, row));
                }
                crate::stage_guard_dialog::peek_applied();
                crate::redraw::poke();
            }
        }
    }
}
