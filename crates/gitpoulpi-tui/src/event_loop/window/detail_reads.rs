//! The selection-driven reads: the selected commit's changed files, the
//! working-tree or commit diff the Diff view shows, and whole-file reads.

use crate::event_loop::*;
use init::WindowState;

impl WindowState {
    /// Works out what the Log's multi-select compares (see `compare`), once
    /// per frame: a pure function of the selection and the Log's rows. A
    /// different set of selected rows forgets a swap.
    pub(in crate::event_loop) fn refresh_compare(&mut self) {
        if self.log_multi.len() < 2 {
            self.compare = None;
            self.compare_swapped = false;
            self.compare_keys.clear();
            return;
        }
        let display = display_commits(
            &self.log_result,
            &self.stashes_result,
            &self.log_order,
            self.log_len,
        );
        let keys = self.log_multi.in_display_order(&display);
        if keys != self.compare_keys {
            self.compare_swapped = false;
            self.compare_keys = keys;
        }
        self.compare = crate::compare::resolve(
            &self.log_multi,
            &display,
            &self.log_result,
            &self.stashes_result,
            self.compare_swapped,
        );
    }

    /// Asks for one file's diff, unless the File History already holds it.
    fn request_diff(&mut self, wanted: &daemon_link::FileDiffRequest) {
        let cached = self.history.as_mut().and_then(|h| h.cached(wanted));
        match cached {
            Some(files) => {
                self.working_diff = Some(daemon_link::WorkingDiffOutcome {
                    target: wanted.target.clone(),
                    path: wanted.path.clone(),
                    files: Ok(files),
                });
            }
            None => self.link.request_working_diff(wanted.clone()),
        }
    }

    /// The file a open menu's "Treat as ..." row is about, when it is one.
    fn menu_hint_file(&self) -> Option<std::path::PathBuf> {
        match &self.open_menu {
            OpenMenu::File((_, path, _)) => Some(path.clone()),
            OpenMenu::Pane((target, _)) if target.kind == PaneKind::File => {
                target.first().map(Path::to_path_buf)
            }
            _ => None,
        }
    }

    /// An open file menu whose row waits for git's binary answer asks for it
    /// (the Diff view only asks about the file it shows).
    fn ask_menu_hint(&self) {
        if let Some(path) = self.menu_hint_file() {
            crate::binary_hint::ask_pending(&self.link, &self.repo.root, &path);
        }
    }

    /// Git's answer for `rel` landed: the open menu about that file rebuilds
    /// its rows in place (cursor kept), so the placeholder row becomes the
    /// right one without reopening the menu.
    fn refresh_menu_hint(&mut self, rel: &Path) {
        if self.menu_hint_file().as_deref() != Some(rel)
            || !crate::binary_hint::take_landed(&self.repo.root, rel)
        {
            return;
        }
        match &mut self.open_menu {
            OpenMenu::File((section, path, menu)) => {
                crate::status_list::refresh_file_menu(
                    menu,
                    *section,
                    path,
                    &self.status_result,
                    &self.conflicts,
                    &self.lfs,
                    &self.repo.root,
                );
            }
            OpenMenu::Pane((target, menu)) => {
                menu.replace_entries(crate::event_loop::input::commands::pane_menu_entries(
                    &self.repo.root,
                    target,
                ));
            }
            _ => {}
        }
    }

    pub(in crate::event_loop) fn apply_detail_reads(&mut self, refreshed: bool) {
        self.refresh_compare();
        // Whenever the Log pane's selection lands on a different commit,
        // fetch that commit's changed files for the detail panel (see the
        // status_title/status_items branch above); a change back to no
        // selection, or a stale in-flight result for an abandoned
        // selection, is handled below rather than shown.
        // `central_view == Diff` and `diff_view_state == Some(_)` are two
        // halves of one fact, set together at every open/close site. This
        // restores the invariant rather than trusting it: the alternative
        // failure is a blank central column with no visible way out, since
        // the Diff render branch is what draws the Esc affordance.
        // The File View is the same kind of state: an open
        // `FileInsight` is the central column, whoever opened it.
        sync_central_view(
            &mut self.central_view,
            self.merge.is_some(),
            self.editor.is_some(),
            self.file_insight.is_some(),
            self.diff_view_state.is_some(),
        );
        // A stash row shows the stash as the commit it is: its own commit
        // against its base (its first parent) is exactly what it changed.
        // A comparison replaces the selected commit: its files travel the
        // same channel under a synthetic key.
        let selected_commit = self.compare.as_ref().map(|c| c.id).or_else(|| {
            selected_log_commit(&self.log_result, &self.log_state, &self.log_order).or_else(|| {
                selected_log_stash(&self.stashes_result, &self.log_state, &self.log_order)
                    .and_then(|s| s.commit)
            })
        });
        // Re-requested when the *toggle* changes as well as the
        // selection: the tree is only fetched when it is wanted, so
        // turning `View all files` on for a commit already on screen has
        // to go and get it.
        let wants_tree_for =
            selected_commit.filter(|_| self.view_all_files && self.compare.is_none());
        if selected_commit != self.requested_commit_files
            || wants_tree_for != self.requested_commit_tree
        {
            self.requested_commit_files = selected_commit;
            self.requested_commit_tree = wants_tree_for;
            self.commit_files = None;
            self.commit_files_len = 0;
            self.commit_file_focus = false;
            self.commit_file_state.select(None);
            self.commit_files_follow = true;
            if let Some(compare) = &self.compare {
                self.link
                    .request_compare_files(compare.id, compare.target.clone());
            } else if let Some(oid) = selected_commit {
                self.link.request_commit_files(oid, self.view_all_files);
                // The rows either side, fetched in the
                // background so the next j/k lands on a commit the link
                // already holds. Sent after the request above, which the
                // link serves first either way.
                self.link.prefetch_commit_files(log_neighbor_commits(
                    &self.log_result,
                    &self.log_state,
                    &self.log_order,
                ));
            }
        }
        // A comparison with the working tree goes stale as files are saved:
        // asked again on every refresh, the old list staying on screen
        // until the new one lands.
        if let Some(compare) = self
            .compare
            .as_ref()
            .filter(|c| refreshed && c.reads_worktree())
        {
            self.link
                .request_compare_files(compare.id, compare.target.clone());
        }
        // The WIP row's View all files lists HEAD's tree
        // (plus what `status` adds), fetched through the same
        // commit-files read the commit-detail toggle uses.
        if let Some(head) = self.wip_all.tree_to_request(
            self.view_all_files && self.right_panel == RightPanel::Status,
            head_oid(self.refs_result.as_deref().unwrap_or(&[])),
        ) {
            self.link.request_head_tree(head);
        }
        while let Ok(outcome) = self.link.commit_files_rx.try_recv() {
            self.wip_all.accept(&outcome);
            if Some(outcome.commit) == self.requested_commit_files {
                self.commit_files_len = outcome.files.as_ref().map(|f| f.len()).unwrap_or(0);
                self.commit_files = Some(outcome);
            }
        }

        // The working tree's own diff, for the half the Diff view is
        // showing. Re-asked whenever the open half changes and again on
        // every refresh, rather than cached: staging a hunk is itself a
        // mutation, the daemon broadcasts `RepoChanged`, and the refresh
        // that follows is exactly when this diff has to be refetched --
        // otherwise the view would keep offering to stage a hunk that is
        // already in the index.
        //
        // One file at a time: the view shows one file, so only that file's hunks
        // are asked for -- by path, for a commit as much as for the working tree.
        // A commit's hunks never change, so they are asked once; the working
        // halves again on every refresh.
        let wanted_diff = self
            .diff_view_state
            .as_ref()
            .filter(|_| self.central_view == CentralView::Diff)
            .map(|s| {
                let target = match s.diff_target() {
                    Some(target) => target,
                    None => gitpoulpi_core::domain::DiffTarget::CommitVsParent(s.commit),
                };
                // A moved file's other name rides along, so the diff is the
                // content against the old path (`rename_pairs`).
                let also = s.also_path.clone().or_else(|| {
                    let tree = self.status_result.as_ref().ok()?;
                    let entry = tree.files.iter().find(|e| e.path == s.path)?;
                    let from = entry.renamed_from.clone()?;
                    match s.source {
                        crate::diff_view::DiffSource::Unstaged if entry.index.is_none() => {
                            Some(from)
                        }
                        crate::diff_view::DiffSource::Staged
                            if entry.index == Some(gitpoulpi_core::domain::FileStatus::Renamed) =>
                        {
                            Some(from)
                        }
                        _ => None,
                    }
                });
                daemon_link::FileDiffRequest::file(target, s.path.clone()).with_also(also)
            });
        let matches_wanted = |outcome: &daemon_link::WorkingDiffOutcome| {
            wanted_diff
                .as_ref()
                .is_some_and(|w| w.target == outcome.target && w.path == outcome.path)
        };
        match (&wanted_diff, &self.working_diff) {
            (None, _) => self.working_diff = None,
            (Some(wanted), Some(outcome)) if !matches_wanted(outcome) => {
                self.working_diff = None;
                self.request_diff(wanted);
            }
            (Some(wanted), None) => self.request_diff(wanted),
            (Some(wanted), Some(_)) => {
                // A commit's, or two commits', hunks never change.
                let commit = !matches!(
                    wanted.target,
                    gitpoulpi_core::domain::DiffTarget::WorkingTreeVsIndex
                        | gitpoulpi_core::domain::DiffTarget::IndexVsHead
                        | gitpoulpi_core::domain::DiffTarget::CommitVsWorkingTree(_)
                );
                if refreshed && !commit {
                    self.link.request_working_diff(wanted.clone());
                }
            }
        }
        while let Ok(outcome) = self.link.working_diff_rx.try_recv() {
            if matches_wanted(&outcome) {
                if let (Some(history), Some(wanted), Ok(files)) =
                    (self.history.as_mut(), wanted_diff.as_ref(), &outcome.files)
                {
                    history.remember(wanted, files);
                }
                self.working_diff = Some(outcome);
            }
        }
        // Whether git diffs the file on show as binary: the file menu's
        // "Treat as ..." row reads it (`binary_hint`).
        crate::binary_hint::ask(
            &self.link,
            self.diff_view_state
                .as_ref()
                .filter(|_| self.central_view == CentralView::Diff)
                .map(|s| s.path.as_path()),
        );
        self.ask_menu_hint();
        while let Ok(outcome) = self.link.file_insight_rx.try_recv() {
            if let Ok(daemon_link::FileInsightPayload::BinaryState(binary)) = &outcome.result {
                crate::binary_hint::record(&self.repo.root, &outcome.request.path, *binary);
                self.refresh_menu_hint(&outcome.request.path);
                continue;
            }
            if outcome.request.kind == daemon_link::FileInsightKind::BinaryState {
                // No answer will come: the sniff stands in, so the row
                // does not stay a placeholder.
                crate::binary_hint::record_failed(&self.repo.root, &outcome.request.path);
                self.refresh_menu_hint(&outcome.request.path);
                continue;
            }
            // The File History's pages share it too.
            let outcome = match self.history.as_mut() {
                Some(history) => match history.offer(outcome) {
                    Some(other) => other,
                    None => continue,
                },
                None => outcome,
            };
            // The Diff view's re-reads in a chosen encoding too.
            let Some(outcome) = crate::diff_recode::offer(outcome) else {
                continue;
            };
            // The Diff view's whole-file reads share this channel; they
            // are taken here and never reach the File View.
            let Some(outcome) = self.diff_content.offer(outcome) else {
                continue;
            };
            // So does the read behind a binary file's "Open in" copy.
            let Some(outcome) = crate::open_with::offer(outcome) else {
                continue;
            };
            // So do the image diff's two pictures and the File View's image.
            let Some(outcome) = crate::image_view::split::offer(outcome) else {
                continue;
            };
            let Some(outcome) = crate::image_view::pane::offer(outcome) else {
                continue;
            };
            if let Some(insight) = self.file_insight.as_mut() {
                match outcome.result {
                    Ok(payload) => insight.accept(&outcome.request, payload),
                    Err(e) => insight.fail(&outcome.request, e.to_string()),
                }
            }
        }
        // A File View that opened on a picture asks for its bytes.
        crate::image_view::pane::sync(self.file_insight.as_ref(), &self.link);
        // The commit column picked another commit: the view goes along.
        self.follow_commit_column();
    }
}
