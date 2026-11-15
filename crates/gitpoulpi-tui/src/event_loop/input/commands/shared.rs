//! The window-level actions shared by the palette, the leader keys, the
//! toolbar and the menus: one function each.

use crate::event_loop::*;
use gitpoulpi_i18n::msg;

use super::{block_on_conflicts, CommandCtx};

/// How a keyboard or palette resize moves a block.
#[derive(Clone, Copy)]
pub(in crate::event_loop) enum Resize {
    By(i32),
    Reset,
}

/// Resizes the active Refs sidebar section by a number of rows (keyboard and
/// palette twin of dragging its bottom border), or with `Reset` gives every
/// section its automatic height back. Saved at once.
pub(in crate::event_loop) fn resize_sidebar_section(cx: &mut CommandCtx<'_>, how: Resize) {
    match how {
        Resize::Reset => {
            cx.sidebar_heights.clear();
            session_store::save_sidebar_heights(cx.session, &*cx.sidebar_heights);
            *cx.last_message = Some(msg::input::SECTIONS_AUTO_HEIGHT.get().into());
        }
        Resize::By(delta) => {
            let section = *cx.active_ref_section;
            match cx
                .sidebar_section_areas
                .iter()
                .position(|(s, _)| *s == section)
            {
                Some(index)
                    if cx
                        .sidebar_layout_specs
                        .get(index)
                        .is_some_and(|s| s.expanded) =>
                {
                    let current = cx.sidebar_section_areas[index].1.height as i32;
                    let height = sidebar_layout::clamp_section_height(
                        (current + delta).max(0) as u16,
                        *cx.sidebar_layout_avail,
                        &*cx.sidebar_layout_specs,
                        index,
                    );
                    sidebar_layout::store_height(&mut *cx.sidebar_heights, section.index(), height);
                    session_store::save_sidebar_heights(cx.session, &*cx.sidebar_heights);
                    *cx.last_message = Some(msg::input::section_rows(
                        (height - 2) as u64,
                        section.display_title(),
                    ));
                }
                _ => {
                    *cx.last_message =
                        Some(msg::input::section_not_expanded(section.display_title()));
                }
            }
        }
    }
}

/// "Reset Layout": every remembered size back to its default, in memory
/// and on disk. The Log column *order* and visibility, and which panes are
/// collapsed, are choices, not sizes, and stay.
pub(in crate::event_loop) fn reset_layout(cx: &mut CommandCtx<'_>) {
    let defaults = gitpoulpi_session::PaneLayout::default();
    *cx.divider1_pct = defaults.divider1_pct;
    *cx.divider2_pct = defaults.divider2_pct;
    cx.sidebar_heights.clear();
    *cx.commit_message_user_rows = 0;
    cx.log_column_overrides.clear();
    if let Some(view) = cx.merge.as_mut() {
        view.apply_layout_pref(&[], 0);
    }
    let mut layout = session_store::layout(cx.session);
    layout.divider1_pct = defaults.divider1_pct;
    layout.divider2_pct = defaults.divider2_pct;
    layout.sidebar_heights.clear();
    layout.commit_message_rows = 0;
    layout.merge_cols.clear();
    layout.merge_output_pct = 0;
    layout.picker_split_permille = 0;
    session_store::save_layout(cx.session, layout);
    crate::folder_picker::install_split(0);
    *cx.last_message = Some(msg::input::LAYOUT_RESET.get().into());
}

/// The commit message block of the right panel: resize by a number of rows
/// (keyboard and palette twin of dragging its bottom border; `Reset`
/// returns to the automatic height).
pub(in crate::event_loop) fn resize_commit_message(cx: &mut CommandCtx<'_>, how: Resize) {
    match how {
        Resize::Reset => {
            *cx.commit_message_user_rows = 0;
            session_store::save_commit_message_rows(cx.session, 0);
            *cx.last_message = Some(msg::input::MESSAGE_AUTO_HEIGHT.get().into());
        }
        Resize::By(delta) => {
            if *cx.right_panel == RightPanel::CommitDetail && cx.commit_message_area.height > 0 {
                // Build on the height already chosen, not on the last
                // painted one: keys that arrive in the same batch (a
                // paste, a fast typist, a fast agent, a Unix pty that
                // delivers them together) are handled before the next
                // repaint refreshes `commit_message_rows_now`, and each
                // would otherwise start from the same stale height.
                let max_rows = commit_detail::max_message_rows(*cx.commit_detail_status_height);
                let base = if *cx.commit_message_user_rows > 0 {
                    (*cx.commit_message_user_rows).min(max_rows)
                } else {
                    *cx.commit_message_rows_now
                };
                let rows = (base as i32 + delta).max(1) as u16;
                *cx.commit_message_user_rows = rows.min(max_rows);
                session_store::save_commit_message_rows(cx.session, *cx.commit_message_user_rows);
                *cx.last_message = Some(msg::input::message_rows(
                    *cx.commit_message_user_rows as u64,
                ));
            } else {
                *cx.last_message = Some(msg::input::SELECT_COMMIT_RESIZE.get().into());
            }
        }
    }
}

/// Scrolls the commit message block (twin of the wheel).
pub(in crate::event_loop) fn scroll_commit_message(cx: &mut CommandCtx<'_>, delta: isize) {
    let max = cx
        .commit_message_total
        .saturating_sub(*cx.commit_message_rows_now as usize);
    let next = (*cx.commit_message_scroll as isize + delta).clamp(0, max as isize);
    *cx.commit_message_scroll = next as usize;
}

pub(in crate::event_loop) fn toggle_commit_sort(cx: &mut CommandCtx<'_>) {
    *cx.commit_sort_desc = !*cx.commit_sort_desc;
    cx.commit_file_state.select(None);
    *cx.last_message = Some(if *cx.commit_sort_desc {
        msg::input::FILES_SORTED_DESC.get().into()
    } else {
        msg::input::FILES_SORTED_ASC.get().into()
    });
}

/// Compare Selection (`v`, the palette): puts the keyboard in the file list
/// of the comparison the Log's selected rows stand for. With fewer than two
/// rows selected it says how to select them.
pub(in crate::event_loop) fn compare_selection(cx: &mut CommandCtx<'_>) {
    if cx.compare.is_none() {
        *cx.last_message = Some(msg::compare::SELECT_MORE.get().into());
        return;
    }
    *cx.right_panel = RightPanel::CommitDetail;
    *cx.focus = Focus::Log;
    *cx.commit_file_focus = true;
    if cx.commit_file_state.selected().is_none() && *cx.commit_files_len > 0 {
        cx.commit_file_state.select(Some(0));
    }
    *cx.commit_files_follow = true;
    *cx.last_message = Some(msg::compare::COMPARING.get().into());
}

/// Swap the two ends of the comparison (the header button, `x`, the
/// palette): the older end becomes the newer and the list shows the
/// opposite change. Not possible with the WIP row (always the newest end)
/// or a range of three or more.
pub(in crate::event_loop) fn compare_swap(cx: &mut CommandCtx<'_>) {
    match cx.compare.as_ref() {
        Some(c) if c.can_swap => {
            *cx.compare_swapped = !c.swapped;
            cx.commit_file_state.select(None);
            *cx.commit_files_follow = true;
        }
        Some(_) => *cx.last_message = Some(msg::compare::SWAP_REFUSED.get().into()),
        None => *cx.last_message = Some(msg::compare::SELECT_MORE.get().into()),
    }
}

/// `View all files` (commit-detail and WIP toolbars share the toggle): the
/// checkbox click, the `a` key and the palette command all run this.
pub(in crate::event_loop) fn toggle_view_all_files(cx: &mut CommandCtx<'_>) {
    *cx.view_all_files = !*cx.view_all_files;
    cx.commit_file_state.select(None);
    *cx.commit_files_follow = true;
    *cx.commit_file_focus = false;
    // The WIP row's list shares the toggle.
    cx.wip_all.state.select(None);
    cx.wip_all.follow = true;
}

/// `Path`/`Tree` for whichever file list the right pane is showing.
pub(in crate::event_loop) fn toggle_file_list_mode(cx: &mut CommandCtx<'_>) {
    if *cx.right_panel == RightPanel::CommitDetail {
        *cx.commit_list_mode = cx.commit_list_mode.toggled();
    } else {
        *cx.status_list_mode = cx.status_list_mode.toggled();
    }
}

/// AI "explain": one spelling for the menu rows, the palette and the
/// chord, so none of them can drift. The request runs on the AI worker;
/// the answer lands in a read-only popup.
pub(in crate::event_loop) fn explain_commit(cx: &mut CommandCtx<'_>, commit: Oid) {
    let message = loaded_commit(&*cx.log_result, commit)
        .map(|n| n.message.clone())
        .unwrap_or_default();
    cx.ai_ui.explain_into(
        cx.session,
        cx.repo,
        ExplainTarget::Commit {
            oid: commit,
            message,
            short: commit.to_hex()[..SHORT_SHA_LEN].to_string(),
        },
        &mut *cx.last_message,
    );
}

pub(in crate::event_loop) fn explain_wip(cx: &mut CommandCtx<'_>) {
    if !cx.ai_ui.enabled() {
        *cx.last_message = Some(msg::input::AI_OFF.get().into());
    } else if !has_wip_row(&*cx.status_result) {
        *cx.last_message = Some(msg::input::NO_CHANGES_TO_EXPLAIN.get().into());
    } else {
        cx.ai_ui.explain_into(
            cx.session,
            cx.repo,
            ExplainTarget::Wip,
            &mut *cx.last_message,
        )
    }
}

/// Commits the composer's message with its toggles (the Commit button,
/// `Ctrl+Enter` and `Ctrl+G Enter` all land here) and, when a message was
/// sent, clears the composer for the next commit.
pub(in crate::event_loop) fn submit_composer(cx: &mut CommandCtx<'_>) {
    // Git refuses to commit unmerged files: say which, and offer the editor.
    if block_on_conflicts(cx, crate::guide_dialog::Blocked::Commit) {
        return;
    }
    submit_commit(
        cx.link,
        &*cx.commit_summary,
        &*cx.commit_description,
        *cx.commit_amend,
        *cx.commit_skip_hooks,
        cx.commit_settings,
        &mut *cx.last_message,
    );
    if !cx.commit_summary.trim().is_empty() {
        cx.commit_summary.clear();
        cx.commit_description.clear();
        *cx.commit_amend = false;
        *cx.commit_skip_hooks = cx.commit_settings.skip_hooks_default;
        *cx.commit_template_applied = false;
        *cx.composer_focus = None;
    }
}

/// `Ctrl+G c`, `Ctrl+G Enter` and their palette entries: the Commit button
/// from anywhere. Same readiness check, hooks, confirmations and journal as
/// the button. When the button would not be ready, nothing is typed in the
/// status line: the keyboard goes to the commit message field of the right
/// pane (revealed if hidden or collapsed), and the status line names the
/// shortcut that commits from there (or says what is missing).
pub(in crate::event_loop) fn commit_now(cx: &mut CommandCtx<'_>) {
    if cx.management.is_some() {
        *cx.last_message = Some(msg::input::NO_REPO_TAB.get().into());
        return;
    }
    if cx.ai_ui.running() {
        *cx.last_message = Some(msg::input::COMMIT_NOW_AI_RUNNING.get().into());
        return;
    }
    let staged_count = cx
        .status_result
        .as_ref()
        .map(|t| section_paths(t, StatusSection::Staged).len())
        .unwrap_or(0);
    let merging = matches!(
        cx.operation_state,
        Some(gitpoulpi_core::domain::OperationState::Merge { .. })
    );
    if block_on_conflicts(cx, crate::guide_dialog::Blocked::Commit) {
        return;
    }
    match commit_readiness(
        &*cx.commit_summary,
        *cx.commit_amend,
        staged_count,
        merging,
        0,
    ) {
        Ok(()) => submit_composer(cx),
        Err(reason) => {
            // Missing staged files is said as is; a missing message is
            // the usual case, and the hint tells how to finish it.
            *cx.last_message = Some(if reason == msg::composer::NEED_STAGED {
                reason.get().to_string()
            } else {
                commit_hint()
            });
            focus_commit_message(cx);
        }
    }
}

/// The status-line hint after the keyboard moved into the commit message
/// field: the shortcut that commits, as currently bound.
fn commit_hint() -> String {
    msg::input::commit_now_hint(
        keymap::current()
            .shown_labels(keymap::Action::ComposerCommit)
            .into_iter()
            .next()
            .unwrap_or_default(),
    )
}

/// Puts the caret in the commit message field of the right pane, revealing
/// the pane when it is hidden or collapsed.
pub(in crate::event_loop) fn focus_commit_message(cx: &mut CommandCtx<'_>) {
    *cx.right_panel = RightPanel::Status;
    *cx.right_pane_collapsed = false;
    *cx.focus = Focus::Status;
    *cx.composer_focus = Some(ComposerField::Summary);
}

/// Concluding a merge whose result equals HEAD (no working-tree row): the
/// message field takes git's own merge message, ready to edit.
pub(in crate::event_loop) fn focus_merge_message(cx: &mut CommandCtx<'_>, message: String) {
    if cx.commit_summary.trim().is_empty() {
        *cx.commit_summary = message;
    }
    focus_commit_message(cx);
    *cx.last_message = Some(commit_hint());
}

/// AI commit message: refuses (with a status line) when AI is off or
/// nothing is staged - the Generate button's own rule, shared with its
/// palette entry and `Ctrl+G i`.
pub(in crate::event_loop) fn generate_commit_message(cx: &mut CommandCtx<'_>) {
    let staged = cx
        .status_result
        .as_ref()
        .map(|t| section_paths(t, StatusSection::Staged).len())
        .unwrap_or(0);
    if !cx.ai_ui.enabled() {
        *cx.last_message = Some(msg::input::AI_OFF.get().into());
    } else if staged == 0 {
        *cx.last_message = Some(msg::input::STAGE_FIRST_NOTHING_TO_DESCRIBE.get().into());
    } else {
        cx.ai_ui.start_into(
            cx.session,
            cx.repo,
            AiRequest::CommitMessage,
            &mut *cx.last_message,
        );
    }
}

/// Runs one row of a file's context menu on `path`: the menu's mouse
/// click, its keyboard Enter and the palette's twin commands all land
/// here, so none can drift. LFS rows and `EditFile` need state the
/// plain `apply_file_context_action` does not have.
pub(in crate::event_loop) fn apply_file_menu_action(
    cx: &mut CommandCtx<'_>,
    section: StatusSection,
    path: PathBuf,
    action: FileContextAction,
) {
    if is_lfs_file_action(action) {
        apply_lfs_file_action(
            action,
            &path,
            cx.remote_ui
                .default_remote(cx.refs_result.as_deref().unwrap_or(&[])),
            cx.link,
            &mut *cx.input_mode,
            &mut *cx.last_message,
        );
    } else if action == FileContextAction::OpenMergeEditor {
        super::conflicts::open_path(cx, path);
    } else if matches!(action, FileContextAction::Convert(_))
        && super::pane::editor_blocks(cx, [path.as_path()]).is_some()
    {
        *cx.last_message = Some(msg::pane_menu::editor_open(path.display()));
    } else if action == FileContextAction::EditFile {
        if let Err(why) = open_editor(
            cx.repo_root,
            &path,
            head_oid(cx.refs_result.as_deref().unwrap_or(&[])),
            cx.link,
            &mut *cx.editor,
            &mut *cx.file_insight,
            &*cx.diff_view_state,
        ) {
            *cx.last_message = Some(why);
        }
    } else {
        apply_file_context_action(
            action,
            section,
            cx.repo_root,
            &path,
            cx.link,
            cx.refs_result.as_deref().unwrap_or(&[]),
            &mut *cx.input_mode,
            &mut *cx.file_insight,
            &mut *cx.last_message,
            &mut *cx.patch_seed,
        );
    }
}

/// What a pick in the Diff view's encoding dropdown does: the view reads the
/// file in that encoding from now on, and the choice is remembered for the
/// path (`diff_recode`).
pub(in crate::event_loop) fn apply_diff_encoding_pick(
    cx: &mut CommandCtx<'_>,
    pick: crate::diff_encoding_menu::Pick,
) {
    if let Some(s) = cx.diff_view_state.as_mut() {
        *cx.last_message = Some(set_diff_encoding(s, pick));
    }
}

/// Sets the encoding of an open Diff view, remembers it for the path and
/// puts the view back at the top (the rows are a different diff), and
/// returns the status line's message.
pub(crate) fn set_diff_encoding(
    s: &mut DiffViewState,
    pick: crate::diff_encoding_menu::Pick,
) -> String {
    s.encoding = pick;
    crate::diff_recode::remember(&s.path, pick);
    s.reset_position();
    match pick {
        Some(c) => gitpoulpi_i18n::msg::diff_view::enc_chosen(c.name()),
        None => gitpoulpi_i18n::msg::diff_view::ENC_AUTO_CHOSEN
            .get()
            .to_string(),
    }
}

/// What a pick in the image header's "Edit in" popup does: runs the editor on
/// the picture, or opens Settings on the Images page for "Configure...".
pub(in crate::event_loop) fn apply_image_edit_pick(
    cx: &mut CommandCtx<'_>,
    pick: crate::image_view::edit_in::Pick,
) {
    use crate::image_view::edit_in::Pick;
    match pick {
        Pick::Entry(index) => {
            *cx.last_message = Some(crate::image_view::edit_in::run_entry(index, cx.repo_root));
        }
        Pick::Configure => {
            let mut state = PreferencesState::open(cx.repo_root);
            let _ = state.show_page(None, Some("ui.image_editor_default"));
            *cx.preferences = Some(state);
        }
    }
}

/// What a pick in the toolbar's profile popup does: stores the choice for this
/// repository, then asks whether to write the identity into its git config too.
pub(in crate::event_loop) fn apply_profile_pick(cx: &mut CommandCtx<'_>, id: String) {
    match crate::profile_bar::choose(cx.repo_root, &id) {
        Ok(chosen) => {
            *cx.last_message = Some(chosen.message);
            if let Some((name, email)) = chosen.identity {
                *cx.input_mode = InputMode::Confirm {
                    prompt: gitpoulpi_i18n::msg::profile_bar::confirm(&name, &email),
                    action: ConfirmAction::WriteProfileIdentity { name, email },
                };
            }
        }
        Err(error) => *cx.last_message = Some(error),
    }
}

/// What a pick in the tab bar's workspace combo does: swap to that workspace's
/// tabs, or go to the Home tab's Workspaces page.
pub(in crate::event_loop) fn apply_workspace_pick(
    cx: &mut CommandCtx<'_>,
    pick: crate::workspace_ui::chip::Pick,
) {
    if let Some(message) = crate::workspace_ui::apply::apply_pick(pick, &mut tab_ctx!(cx)) {
        *cx.last_message = Some(message);
    }
    *cx.should_quit |= cx.switch_to.is_some() || *cx.close_window || cx.show_management.is_some();
}

/// What a pick in the tab bar's "Open in" popup does: runs the entry on the
/// active tab, or opens Settings for "Configure...".
pub(in crate::event_loop) fn apply_open_in_pick(
    cx: &mut CommandCtx<'_>,
    pick: crate::open_in_bar::Pick,
) {
    use crate::open_in_bar::Pick;
    match pick {
        Pick::Entry(index) => {
            let active = cx.repo_tabs.get(*cx.active_tab).map(|t| t.path.clone());
            *cx.last_message = Some(match active {
                Some(path) => crate::open_in_bar::run_entry(index, &path),
                None => gitpoulpi_i18n::msg::tab_dir::NO_TAB.get().into(),
            });
        }
        // Lands on the Settings "Open in" page, on its Default row.
        Pick::Configure => {
            let mut state = PreferencesState::open(cx.repo_root);
            let _ = state.show_page(None, Some("ui.open_in_default"));
            *cx.preferences = Some(state);
        }
    }
}

/// What the loop does after the Radar handled a key, a click or a palette
/// command: the one place its [`crate::radar::Outcome`] is carried out.
/// What the loop does with an event of the Release notes tab.
pub(in crate::event_loop) fn apply_notes_event(
    cx: &mut CommandCtx<'_>,
    event: crate::updates::notes::NotesEvent,
) {
    use crate::updates::notes::NotesEvent;
    match event {
        NotesEvent::None => {}
        NotesEvent::Message(message) => *cx.last_message = Some(message),
        NotesEvent::Close => close_notes_tab(&mut tab_ctx!(cx)),
    }
}

pub(in crate::event_loop) fn apply_radar_outcome(
    cx: &mut CommandCtx<'_>,
    outcome: crate::radar::Outcome,
) {
    use crate::radar::Outcome;
    match outcome {
        Outcome::None => {}
        Outcome::Message(message) => *cx.last_message = Some(message),
        Outcome::Close => {
            if let Some(manager) = cx.management.as_mut() {
                manager.radar.close();
            }
        }
        Outcome::Quit => {
            *cx.should_quit = true;
            *cx.quit_how = Some("q key on the Radar");
        }
        Outcome::CopyText(text) => {
            *cx.last_message = Some(cx.clipboard.copy(msg::input::WHAT_SELECTION.get(), &text));
        }
        // The repository's window starts next: what to do there travels in
        // `radar::pending`.
        Outcome::OpenRepo(path, pending) => {
            if let Some(pending) = pending {
                crate::radar::pending::set(pending);
            }
            let said = tab_ctx!(cx).switcher_action(repo_switcher::SwitcherAction::Open(path));
            if said.is_some() {
                *cx.last_message = said;
            }
        }
        Outcome::Setting(key) => {
            let mut state = PreferencesState::open(cx.repo_root);
            let _ = state.show_page(None, Some(key));
            *cx.preferences = Some(state);
        }
        Outcome::Integrations(profile) => {
            let mut state = PreferencesState::open(cx.repo_root);
            state.show_integration(profile.as_deref());
            *cx.preferences = Some(state);
        }
        Outcome::ApiReview(request) => open_api_review(cx, *request),
        Outcome::ApiConversation(request) => open_api_conversation(cx, *request),
        // The ordinary Clone dialog, prefilled; the clone's end carries the
        // review on (`after_clone`).
        Outcome::CloneReview(request) => {
            let note = msg::radar::msg_clone_review_note(&request.label);
            cx.repo_dialogs.open_clone_for_review(
                &request.clone_url,
                crate::after_clone::Carry {
                    number: request.number,
                    label: request.label,
                    integration: request.integration,
                    project: request.project,
                },
                note,
                cx.cwd,
            );
        }
    }
    *cx.should_quit |= cx.switch_to.is_some() || *cx.close_window;
}

/// The review of a request of a repository that is not cloned, through the
/// host's API, hosted by the Radar over the Home window.
#[cfg(all(feature = "hosting", feature = "review"))]
fn open_api_review(cx: &mut CommandCtx<'_>, request: crate::radar::ReviewRequest) {
    let all = crate::pull_request::load_integrations();
    let Some(integration) = all.iter().find(|i| i.id == request.integration).cloned() else {
        *cx.last_message = Some(msg::radar::why_no_integration(&request.integration));
        return;
    };
    let target = crate::review_view::ReviewTarget {
        integration,
        all,
        project: request.project,
        number: request.number,
        iid_label: request.label,
        title: request.title,
        remote: String::new(),
        web_url: request.web_url,
        source_branch: request.source_branch,
        target_branch: request.target_branch,
        patch_set: None,
        compare_patch_set: None,
        repo_root: None,
    };
    let config_dir =
        gitpoulpi_config::ConfigPaths::discover().map(|p| p.config_dir().to_path_buf());
    let view = crate::review_view::ReviewView::open_api(
        target,
        std::sync::Arc::new(crate::hosting_task::DefaultFactory),
        cx.ai_ui.enabled(),
        cx.session.to_string(),
        config_dir,
    );
    if let Some(manager) = cx.management.as_mut() {
        manager.radar.host_review(Box::new(view));
    }
}

#[cfg(not(all(feature = "hosting", feature = "review")))]
fn open_api_review(cx: &mut CommandCtx<'_>, _request: crate::radar::ReviewRequest) {
    *cx.last_message = Some(msg::meta_picker::UNSUPPORTED.get().to_string());
}

/// The conversation of an item of a repository that is not cloned, hosted by
/// the Radar over the Home window: the profile and the project are all the
/// view needs.
#[cfg(feature = "conversation")]
fn open_api_conversation(cx: &mut CommandCtx<'_>, request: crate::radar::ConversationRequest) {
    use gitpoulpi_hosting::conversation::ItemKey;
    let all = crate::pull_request::load_integrations();
    let Some(integration) = all.iter().find(|i| i.id == request.integration).cloned() else {
        *cx.last_message = Some(msg::radar::why_no_integration(&request.integration));
        return;
    };
    if !request.pr && !crate::item_view::supports_issues(integration.kind) {
        *cx.last_message = Some(msg::item_view::WHY_NO_CONVERSATION.get().to_string());
        return;
    }
    let target = crate::item_view::ItemTarget {
        integration,
        all,
        project: request.project,
        key: if request.pr {
            ItemKey::pull_request(request.number)
        } else {
            ItemKey::issue(request.number)
        },
        reference: request.label,
        title: request.title,
        web_url: request.web_url,
        repo_root: None,
        bases: Default::default(),
    };
    let view = crate::item_view::ItemView::open(
        target,
        std::sync::Arc::new(crate::hosting_task::DefaultFactory),
    );
    if let Some(manager) = cx.management.as_mut() {
        manager.radar.host_item(Box::new(view));
    }
}

#[cfg(not(feature = "conversation"))]
fn open_api_conversation(cx: &mut CommandCtx<'_>, _request: crate::radar::ConversationRequest) {
    *cx.last_message = Some(msg::meta_picker::UNSUPPORTED.get().to_string());
}

/// What the loop does with a `HostingEffect` (`hosting_ui`): the few
/// outcomes that need state only the window owns. Spelled once and used by
/// the menu, the sidebar rows, the toolbar chip and the palette, so none of
/// them can drift.
pub(in crate::event_loop) fn apply_hosting_effect(cx: &mut CommandCtx<'_>, effect: HostingEffect) {
    match effect {
        HostingEffect::None => {}
        HostingEffect::Message(message) => {
            *cx.last_message = Some(terminology::tr(&message).into_owned())
        }
        HostingEffect::Preferences(profile) => {
            let mut state = PreferencesState::open(cx.repo_root);
            state.show_integration(profile.as_deref());
            *cx.preferences = Some(state);
        }
        HostingEffect::RadarOptions => {
            let mut state = PreferencesState::open(cx.repo_root);
            state.show_radar_options();
            *cx.preferences = Some(state);
        }
        HostingEffect::Checkout { branch, remote } => {
            let refs = cx.refs_result.as_deref().unwrap_or(&[]);
            match hosting_ui::branch_target(refs, &remote, &branch) {
                Some(target) => {
                    let message = checkout_ref_now(
                        &target,
                        &*cx.refs_result,
                        &*cx.remote_names,
                        cx.link,
                        &mut *cx.input_mode,
                    );
                    if !message.is_empty() {
                        *cx.last_message = Some(message);
                    }
                }
                None => {
                    *cx.last_message = Some(msg::input::branch_not_here(
                        &branch,
                        keymap::paren(&keymap::current(), Bind::GitFetchAll),
                    ))
                }
            }
        }
        HostingEffect::CheckoutPullRequestHead {
            remote,
            remote_ref,
            branch,
        } => {
            cx.link.submit_action(Action::CheckoutPullRequestHead {
                remote,
                remote_ref,
                branch: branch.clone(),
            });
            *cx.last_message = Some(msg::input::fetching_pr(&branch));
        }
        HostingEffect::ViewBranch { branch, remote } => {
            let refs = cx.refs_result.as_deref().unwrap_or(&[]);
            let tip = hosting_ui::branch_target(refs, &remote, &branch).and_then(|target| {
                refs.iter()
                    .find(|r| ref_full_name(r) == target)
                    .map(|r| r.target)
            });
            match tip {
                Some(oid)
                    if select_log_commit(
                        &*cx.log_result,
                        &*cx.log_order,
                        &mut *cx.log_state,
                        oid,
                    ) =>
                {
                    *cx.focus = Focus::Log;
                    *cx.central_view = CentralView::Log;
                    *cx.log_follow = true;
                    *cx.last_message = Some(msg::input::showing_tip(&branch));
                }
                Some(_) => *cx.last_message = Some(msg::input::tip_older(&branch)),
                None => {
                    *cx.last_message = Some(msg::input::branch_not_here(
                        &branch,
                        keymap::paren(&keymap::current(), Bind::GitFetchAll),
                    ))
                }
            }
        }
        HostingEffect::NewBranch(name) => {
            *cx.input_mode = InputMode::InlineName {
                op: RefNameOp::NewBranch,
                value: name,
            };
        }
        HostingEffect::StartPullRequest(branch) => {
            *cx.last_message = Some(start_pull_request_now(
                &branch,
                cx.refs_result.as_deref().unwrap_or(&[]),
                &cx.remote_ui.remotes,
                &mut *cx.pull_request_seed,
            ));
        }
        HostingEffect::CreatePullRequest => {
            *cx.last_message = Some(pull_request::create_pull_request_now(
                cx.refs_result.as_deref().unwrap_or(&[]),
                &cx.remote_ui.remotes,
                &mut *cx.pull_request_seed,
            ));
        }
        #[cfg(feature = "hosting")]
        HostingEffect::NewIssue(seed) => {
            cx.repo_dialogs.open_issue(*seed);
        }
        #[cfg(feature = "hosting")]
        HostingEffect::Board(target) => {
            cx.repo_dialogs.open_board(*target);
        }
        #[cfg(feature = "hosting")]
        HostingEffect::Item(target) => {
            cx.repo_dialogs.open_item(*target);
        }
        #[cfg(feature = "hosting")]
        HostingEffect::PushForReview { remote, branch } => {
            *cx.last_message = Some(msg::hosting_ui::sending_for_review(&branch));
            cx.link.submit_action(Action::PushForReview {
                remote,
                branch,
                topic: None,
            });
        }
        #[cfg(feature = "hosting")]
        HostingEffect::Review(target) => {
            cx.repo_dialogs.open_review(
                *target,
                cx.link.activity_handle().raw.requester(),
                cx.ai_ui.enabled(),
                cx.session,
            );
        }
        #[cfg(feature = "hosting")]
        HostingEffect::EditMeta(target, field) => {
            cx.repo_dialogs.open_meta(*target, field);
        }
        #[cfg(feature = "hosting")]
        HostingEffect::InsertIssueRef(reference) => {
            // Into the field last focused (the Description when that has
            // the caret, else the Summary), and the keyboard goes back to it.
            let field = match *cx.composer_focus {
                Some(ComposerField::Description) => ComposerField::Description,
                Some(ComposerField::WipSummary) => ComposerField::WipSummary,
                _ => ComposerField::Summary,
            };
            let (id, text) = if field == ComposerField::Description {
                (MainField::Description, &mut *cx.commit_description)
            } else {
                (MainField::Summary, &mut *cx.commit_summary)
            };
            let caret = cx.field_editor.caret_mut(id);
            let typed = crate::issue_ref_picker::insertion(text, caret.cursor(text), &reference);
            caret.insert(text, &typed);
            *cx.composer_focus = Some(field);
            if field != ComposerField::WipSummary {
                *cx.focus = Focus::Status;
            }
            *cx.last_message = Some(msg::composer::issue_ref_inserted(&reference));
        }
        HostingEffect::Focus(list) => {
            *cx.focus = Focus::Refs;
            *cx.active_ref_section = list.section();
            cx.collapsed_ref_sections.remove(list.section().title());
            if let Some(index) = cx.hosting.focus_row(list) {
                cx.sidebar_states.select(list.section(), Some(index));
            }
        }
    }
}

/// Moves the Log cursor to the row of comparison card `index` and flashes
/// it (the reveal flash), without touching the multi-selection: the
/// comparison is a function of the selection, not of the cursor.
pub(in crate::event_loop) fn compare_goto_card(cx: &mut CommandCtx<'_>, index: usize) {
    let Some(row) = cx
        .compare
        .as_ref()
        .and_then(|c| c.cards.get(index))
        .map(|c| c.row)
    else {
        return;
    };
    cx.log_state.select(Some(row));
    *cx.log_follow = true;
    // The Log owns the cursor now: its keys (Space, `n`) act on that row.
    *cx.focus = Focus::Log;
    cx.reveal.flash = Some((row, Instant::now() + crate::ref_reveal::FLASH_FOR));
}

/// Keyboard twin of a click on a card: the next compared row after the Log
/// cursor, wrapping to the first.
pub(in crate::event_loop) fn compare_goto_next(cx: &mut CommandCtx<'_>) {
    let Some(c) = cx.compare.as_ref() else {
        *cx.last_message = Some(msg::compare::SELECT_MORE.get().into());
        return;
    };
    let cursor = cx.log_state.selected();
    let next = c
        .cards
        .iter()
        .position(|card| cursor.is_none_or(|cur| card.row > cur))
        .unwrap_or(0);
    compare_goto_card(cx, next);
}

/// The rebase popup's "split...": leaves the popup for the split editor on
/// `commit`. The popup is rebuilt on `upstream` when the editor is left (or
/// after the split went through), whatever the plan said before.
pub(in crate::event_loop) fn split_from_rebase(
    cx: &mut CommandCtx<'_>,
    commit: Oid,
    upstream: Oid,
) {
    match crate::log_menu::split_precheck(&*cx.log_result, &*cx.refs_result, commit) {
        Ok(()) => {
            crate::split_ui::expect_return(commit, upstream);
            *cx.input_mode = InputMode::Normal;
            cx.link.request_split_source(commit);
        }
        Err(why) => *cx.last_message = Some(why),
    }
}

/// Rebuilds the interactive-rebase popup on `upstream` (after the split
/// editor opened from it closed).
pub(in crate::event_loop) fn reopen_rebase(cx: &mut CommandCtx<'_>, upstream: Oid) {
    match crate::log_menu::rebase_form_for(&*cx.log_result, &*cx.refs_result, upstream) {
        Ok(form) => *cx.input_mode = InputMode::InteractiveRebase(Box::new(form)),
        Err(why) => *cx.last_message = Some(why),
    }
}

/// "Split commit with AI...": refuses with a status line when AI is off or
/// the commit cannot be split; otherwise starts the proposal. The editor
/// opens when it lands (`window/tick.rs`), prefilled, or empty with the
/// reason when the AI failed.
pub(in crate::event_loop) fn split_commit_with_ai(cx: &mut CommandCtx<'_>, commit: Oid) {
    if !cx.ai_ui.enabled() {
        *cx.last_message = Some(msg::input::AI_OFF.get().into());
        return;
    }
    if let Err(why) = crate::log_menu::split_precheck(&*cx.log_result, &*cx.refs_result, commit) {
        *cx.last_message = Some(why);
        return;
    }
    cx.ai_ui.start_into(
        cx.session,
        cx.repo,
        AiRequest::SplitCommit {
            oid: commit,
            short: commit.to_hex()[..SHORT_SHA_LEN].to_string(),
        },
        &mut *cx.last_message,
    );
}

/// A pane jump moved the keyboard to a list: an open diff stops claiming
/// `j`/`k`/arrows/paging (see `DiffViewState::lists_have_keys`). Every way of
/// focusing a pane (`Ctrl+N <badge>`, a click, the palette) calls this.
pub(in crate::event_loop) fn release_diff_keys(cx: &mut CommandCtx<'_>) {
    if let Some(s) = cx.diff_view_state.as_mut() {
        s.lists_have_keys = true;
    }
}

/// The diff's own badge: the diff takes the navigation keys back.
pub(in crate::event_loop) fn claim_diff_keys(cx: &mut CommandCtx<'_>) {
    if let Some(s) = cx.diff_view_state.as_mut() {
        s.lists_have_keys = false;
    }
}

/// Moves a main-window pane divider one notch (the palette's widen / narrow
/// commands and `Ctrl+N Left`/`Right`/`Shift+Left`/`Shift+Right`).
pub(in crate::event_loop) fn resize_pane(
    cx: &mut CommandCtx<'_>,
    resize: crate::pane_layout::PaneResize,
) {
    use crate::pane_layout::PaneResize as R;
    // A collapsed pane comes back first: widening something that is
    // not there would change nothing visible.
    match resize {
        R::RefsWider | R::RefsNarrower => *cx.left_pane_collapsed = false,
        R::StatusWider | R::StatusNarrower => *cx.right_pane_collapsed = false,
    }
    (*cx.divider1_pct, *cx.divider2_pct) = resize.apply(*cx.divider1_pct, *cx.divider2_pct);
    let (pane, pct) = match resize {
        R::RefsWider | R::RefsNarrower => (true, *cx.divider1_pct),
        R::StatusWider | R::StatusNarrower => (false, 100 - *cx.divider2_pct),
    };
    *cx.last_message = Some(if pane {
        gitpoulpi_i18n::msg::input::refs_pane_width(u64::from(pct))
    } else {
        gitpoulpi_i18n::msg::input::status_pane_width(u64::from(pct))
    });
    persist_pane_layout(
        cx.session,
        *cx.divider1_pct,
        *cx.divider2_pct,
        cx.left_auto_collapse.persisted(*cx.left_pane_collapsed),
        cx.right_auto_collapse.persisted(*cx.right_pane_collapsed),
    );
}
