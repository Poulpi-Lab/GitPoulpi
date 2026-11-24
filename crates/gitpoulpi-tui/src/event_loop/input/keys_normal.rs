//! Normal-mode keys, first third: the global chords, the composer and
//! commit filter, and the diff view.

use super::commands::{CommandCtx, Flow};
use super::KeyEnv;
use crate::event_loop::*;
use gitpoulpi_i18n::msg;

/// `Flow::Handled` when an arm matched, `Flow::Continue` to try the next part.
pub(super) fn handle(cx: &mut CommandCtx<'_>, env: &KeyEnv<'_>, key: KeyEvent) -> io::Result<Flow> {
    let km = env.km;
    // The File History list's own keys, ahead of the Log's multi-select
    // (Shift+Up/Down extends the history range here).
    if commands::history::on_key(cx, km, key) {
        return Ok(Flow::Handled);
    }
    // Multi-select in the right pane's file lists (the Log's keys). Before
    // the diff keys: with a diff open, Shift+Down still extends the selection.
    if cx.composer_focus.is_none()
        && !*cx.commit_filter_focus
        && !cx.open_menu.is_open()
        && commands::select_key(cx, key, true)
    {
        return Ok(Flow::Handled);
    }
    // Expand/collapse the folders of the tree under the keyboard (`+`, `-`,
    // `*`, `_`): the refs sections and the right pane's file lists.
    if cx.composer_focus.is_none()
        && !*cx.commit_filter_focus
        && !cx.open_menu.is_open()
        && commands::fold_key(cx, km, &key)
    {
        return Ok(Flow::Handled);
    }
    // The commit message's dragged text: copied, or dropped, first.
    if cx.commit_text.has_selection() && cx.composer_focus.is_none() && !*cx.commit_filter_focus {
        let ctrl_c = key.modifiers.contains(KeyModifiers::CONTROL)
            && !key.modifiers.contains(KeyModifiers::ALT)
            && key.code == KeyCode::Char('c');
        if ctrl_c {
            if let Some(text) = cx.commit_text.selected() {
                *cx.last_message = Some(
                    cx.clipboard
                        .copy(gitpoulpi_i18n::msg::diff_view::WHAT_SELECTION.get(), &text),
                );
                return Ok(Flow::Handled);
            }
        } else if key.code == KeyCode::Esc {
            cx.commit_text.clear();
            return Ok(Flow::Handled);
        }
    }
    match key.code {
        // Esc stops a running AI request before it does
        // anything else (leaves a field, closes a view).
        KeyCode::Esc if cx.ai_ui.running() => {
            cx.ai_ui.cancel();
        }
        _ if km.is(Bind::GlobalGitPrefix, &key) => {
            *cx.input_mode = InputMode::Leader;
        }
        _ if km.is(Bind::GlobalWorkspaceSearch, &key) => {
            commands::run_command(cx, PaletteCommand::WorkspaceSearch);
        }
        _ if km.is(Bind::GlobalPalette, &key) => {
            *cx.input_mode = InputMode::CommandPalette {
                draft: String::new(),
                selected: 0,
            };
        }
        // The keyboard equivalent of the toolbar's gear,
        // spelled the way every desktop app spells it.
        // Listed in the palette as `App: Settings...`
        // so it is discoverable without being known.
        _ if km.is(Bind::GlobalSettings, &key) => {
            *cx.preferences = Some(PreferencesState::open(cx.repo_root));
        }
        // Undo / redo the last operation. A focused text
        // field wins: Ctrl+Z in the commit message must
        // never undo a commit (the editor has its own
        // text undo and never gets here).
        _ if (km.is(Bind::GlobalUndo, &key) || km.is(Bind::GlobalRedo, &key))
            && MainField::active(
                &InputMode::Normal,
                *cx.composer_focus,
                *cx.commit_filter_focus,
            )
            .is_none() =>
        {
            let cmd = if km.is(Bind::GlobalUndo, &key) {
                PaletteCommand::Undo
            } else {
                PaletteCommand::Redo
            };
            commands::run_command(cx, cmd);
        }
        // Show or hide the commit column beside the Diff / File View /
        // Blame / Edit views (the palette's `View: Toggle Commit Column`).
        _ if km.is(Bind::GlobalCommitColumn, &key)
            && MainField::active(
                &InputMode::Normal,
                *cx.composer_focus,
                *cx.commit_filter_focus,
            )
            .is_none() =>
        {
            commands::run_command(
                cx,
                PaletteCommand::CommitColumn(crate::commit_column::ColumnCmd::Toggle),
            );
        }
        // The Undo history panel and the blocked-undo offer: one key each.
        _ if (km.is(Bind::GlobalUndoHistory, &key) || km.is(Bind::GlobalUndoOffer, &key))
            && MainField::active(
                &InputMode::Normal,
                *cx.composer_focus,
                *cx.commit_filter_focus,
            )
            .is_none() =>
        {
            let cmd = if km.is(Bind::GlobalUndoHistory, &key) {
                PaletteCommand::UndoHistory
            } else {
                PaletteCommand::UndoOffer
            };
            commands::run_command(cx, cmd);
        }
        // ---- Commit composer, and the WIP row's inline
        // field (the same Summary) ----------------------
        // A text field with the keyboard comes before every pane shortcut, so
        // typing `j`, space or `.` into it types them. Chords (Ctrl/Alt) still
        // fall through, so Ctrl+P and friends keep working while a field is
        // focused.
        KeyCode::Esc
            if cx.composer_focus.is_some()
                && !cx.open_menu.is_file_menu()
                && !cx.open_menu.is_folder_menu() =>
        {
            *cx.composer_focus = None;
        }
        // The composer's `#` twin: pick an issue reference to type in.
        _ if cx.composer_focus.is_some() && km.is(Bind::ComposerInsertIssueRef, &key) => {
            let effect =
                cx.hosting
                    .run_command(HostingCommand::InsertIssueRef, (2, 2), &mut *cx.clipboard);
            commands::apply_hosting_effect(cx, effect);
        }
        // Every editing key: characters,
        // Backspace/Delete, arrows, Ctrl+A/U/K, Ctrl+C/X
        // with a selection -- one shared core for the
        // Summary (composer or WIP row), the Description
        // and the commit-files filter.
        _ if MainField::active(
            &InputMode::Normal,
            *cx.composer_focus,
            *cx.commit_filter_focus,
        )
        .is_some_and(|id| {
            let text = match id {
                MainField::Summary => &*cx.commit_summary,
                MainField::Description => &*cx.commit_description,
                _ => &*cx.commit_files_filter,
            };
            id.claims(&*cx.field_editor, text, &key)
        }) =>
        {
            let Some(id) = MainField::active(
                &InputMode::Normal,
                *cx.composer_focus,
                *cx.commit_filter_focus,
            ) else {
                return Ok(Flow::Handled);
            };
            let text = match id {
                MainField::Summary => &mut *cx.commit_summary,
                MainField::Description => &mut *cx.commit_description,
                _ => &mut *cx.commit_files_filter,
            };
            edit_main_field(
                &mut *cx.field_editor,
                id,
                text,
                &key,
                &mut *cx.clipboard,
                &mut *cx.last_message,
            );
        }
        // Summary -> Description -> commit, so the whole
        // composer is reachable without the mouse. From
        // the WIP row, Enter goes on to the Description in
        // the right pane exactly as it does from the
        // composer's own Summary -- and brings the keyboard
        // focus along, since that is where the caret is.
        // In the Description a plain Enter is a newline;
        // committing from there is `Ctrl+Enter` (see
        // `composer_enter`).
        KeyCode::Enter | KeyCode::Char(_)
            if cx.composer_focus.is_some()
                && (key.code == KeyCode::Enter || is_commit_chord(&key)) =>
        {
            let Some(field) = *cx.composer_focus else {
                return Ok(Flow::Handled);
            };
            match composer_enter(field, is_commit_chord(&key)) {
                ComposerEnter::Newline => {
                    cx.field_editor
                        .caret_mut(MainField::Description)
                        .insert(&mut *cx.commit_description, "\n");
                }
                ComposerEnter::Next(next) => {
                    if field == ComposerField::WipSummary {
                        *cx.focus = Focus::Status;
                        *cx.right_panel = RightPanel::Status;
                    }
                    *cx.composer_focus = Some(next);
                }
                ComposerEnter::Commit => {
                    commands::submit_composer(cx);
                }
            }
        }
        // Keyboard way into the WIP row's field: `c` in the
        // Log (the same key that focuses the composer from
        // the Status pane), or Enter with the WIP row
        // selected. `c` selects the WIP row first, so it
        // works from anywhere in the Log.
        KeyCode::Char('c')
            if *cx.focus == Focus::Log
                && !*cx.commit_file_focus
                && *cx.central_view == CentralView::Log
                && has_wip_row(&*cx.status_result) =>
        {
            cx.log_state.select(Some(0));
            *cx.log_follow = true;
            *cx.composer_focus = Some(ComposerField::WipSummary);
        }
        KeyCode::Enter
            if *cx.focus == Focus::Log
                && !*cx.commit_file_focus
                && *cx.central_view == CentralView::Log
                && wip_row_selected(&*cx.status_result, &*cx.log_state) =>
        {
            *cx.composer_focus = Some(ComposerField::WipSummary);
        }
        // While the Diff view owns the central column its keys come first, so
        // j/k scroll the diff rather than a hidden log selection. Arms it does
        // not claim (?, r, Ctrl+P, 1/2/3, ...) fall through so it never traps the
        // user, and every bar button has an equivalent here.
        // Before / after pictures: "Edit in" (the list, or the default editor).
        _ if *cx.central_view == CentralView::Diff
            && crate::image_view::split::active()
            && km.is(Bind::ImageEditIn, &key) =>
        {
            crate::image_view::edit_in::open(crate::image_view::edit_in::Target::Pane);
        }
        _ if *cx.central_view == CentralView::Diff
            && crate::image_view::split::active()
            && km.is(Bind::ImageEditDefault, &key) =>
        {
            *cx.last_message = Some(crate::image_view::edit_in::run_default(
                &crate::image_view::edit_in::Target::Pane,
                cx.repo_root,
            ));
        }
        // A binary file with OS apps: the same keys open its "Open in" list or
        // run the default app.
        _ if *cx.central_view == CentralView::Diff
            && crate::open_with::view::shown().is_some()
            && km.is(Bind::ImageEditIn, &key) =>
        {
            if let Some(subject) = crate::open_with::view::shown() {
                crate::open_with::popup::open(subject);
            }
        }
        _ if *cx.central_view == CentralView::Diff
            && crate::open_with::view::shown().is_some()
            && km.is(Bind::ImageEditDefault, &key) =>
        {
            if let Some(subject) = crate::open_with::view::shown() {
                *cx.last_message = Some(crate::open_with::popup::run_default(
                    &subject,
                    cx.repo_root,
                    cx.link,
                ));
            }
        }
        // Before / after pictures: the zoom keys, flip, the divider keys and
        // panning come before the text diff's own (unclaimed keys fall through).
        _ if *cx.central_view == CentralView::Diff
            && crate::image_view::split::active()
            && crate::image_view::split::on_key(km, &key) => {}
        // Git LFS: flip the card and the raw pointer text.
        _ if km.is(Bind::DiffLfsRaw, &key) && *cx.central_view == CentralView::Diff => {
            if let Some(s) = cx.diff_view_state.as_mut() {
                s.lfs_raw = !s.lfs_raw;
                s.reset_position();
                *cx.last_message = Some(if s.lfs_raw {
                    msg::input::showing_raw_pointer_hint(keymap::hint(
                        km,
                        Bind::DiffLfsRaw,
                        msg::input::HINT_BACK_TO_LFS_CARD.get(),
                    ))
                } else {
                    msg::input::showing_lfs_card_hint(keymap::hint(
                        km,
                        Bind::DiffLfsRaw,
                        msg::input::HINT_RAW_POINTER.get(),
                    ))
                });
            }
        }
        // The merge conflict editor on the diff's file, when it is conflicted.
        _ if km.is(Bind::DiffMergeEditor, &key) && diff_owns_keys(cx) => {
            commands::open_selected(cx);
        }
        // Edit in Working Directory.
        _ if km.is(Bind::DiffEdit, &key) && diff_owns_keys(cx) => {
            match edit_target(
                *cx.central_view,
                cx.file_insight.as_ref(),
                cx.diff_view_state.as_ref(),
            ) {
                Ok(path) => {
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
                }
                Err(why) => *cx.last_message = Some(why.to_string()),
            }
        }
        // Esc closes the view, but a line selection is the nearer thing to
        // cancel, so the first press clears it and only the second leaves.
        // `q` closes too: it must not fall through to "quit the application"
        // when pressed to leave a diff or a File View.
        _ if (key.code == KeyCode::Esc || km.is(Bind::DiffClose, &key))
            && *cx.central_view == CentralView::Diff =>
        {
            match cx.diff_view_state.as_mut() {
                // The text selection is
                // the innermost of all.
                Some(s) if s.text_sel.is_some() && key.code == KeyCode::Esc => {
                    s.text_sel = None;
                    *cx.last_message = Some(msg::input::TEXT_SELECTION_CLEARED.get().into());
                }
                Some(s) if s.selection_anchor.is_some() => {
                    s.selection_anchor = None;
                    *cx.last_message = Some(msg::input::LINE_SELECTION_CLEARED.get().into());
                }
                _ => {
                    *cx.central_view = CentralView::Log;
                    *cx.diff_view_state = None;
                }
            }
        }
        // Copy: the text selection, or
        // failing one the line under the cursor (the `v`
        // range when one is open). `Ctrl+C` and `y` alike.
        _ if km.is(Bind::DiffCopy, &key) && diff_owns_keys(cx) => {
            if let (Some(s), Some(render)) =
                (cx.diff_view_state.as_ref(), cx.diff_render.as_deref())
            {
                *cx.last_message = Some(match diff_view::yank(render, s) {
                    Ok((what, text)) => cx.clipboard.copy(what, &text),
                    Err(why) => why.to_string(),
                });
            }
        }
        // j/k move a cursor rather than scrolling the
        // page, because everything staging-related is
        // phrased as "the hunk/line under the cursor" --
        // a view with only a scroll offset has nothing to
        // point at. The scroll follows the cursor, so the
        // felt behavior at the top and bottom edges is
        // unchanged.
        KeyCode::Down | KeyCode::Char('j') if diff_owns_keys(cx) => {
            let height = cx.diff_body_area.height.saturating_sub(2);
            if let Some(s) = cx.diff_view_state.as_mut() {
                diff_view::move_cursor(s, 1, *cx.diff_rendered_len, height);
            }
        }
        KeyCode::Up | KeyCode::Char('k') if diff_owns_keys(cx) => {
            let height = cx.diff_body_area.height.saturating_sub(2);
            if let Some(s) = cx.diff_view_state.as_mut() {
                diff_view::move_cursor(s, -1, *cx.diff_rendered_len, height);
            }
        }
        KeyCode::PageDown if diff_owns_keys(cx) => {
            let height = cx.diff_body_area.height.saturating_sub(2);
            let page = height.max(1) as isize;
            if let Some(s) = cx.diff_view_state.as_mut() {
                diff_view::move_cursor(s, page, *cx.diff_rendered_len, height);
            }
        }
        KeyCode::PageUp if diff_owns_keys(cx) => {
            let height = cx.diff_body_area.height.saturating_sub(2);
            let page = height.max(1) as isize;
            if let Some(s) = cx.diff_view_state.as_mut() {
                diff_view::move_cursor(s, -page, *cx.diff_rendered_len, height);
            }
        }
        KeyCode::Home if diff_owns_keys(cx) => {
            let height = cx.diff_body_area.height.saturating_sub(2);
            if let Some(s) = cx.diff_view_state.as_mut() {
                diff_view::move_cursor(s, isize::MIN / 2, *cx.diff_rendered_len, height);
            }
        }
        KeyCode::End if diff_owns_keys(cx) => {
            let height = cx.diff_body_area.height.saturating_sub(2);
            if let Some(s) = cx.diff_view_state.as_mut() {
                // Clamped down to the real last row by
                // `move_cursor`, which is given the row
                // count the current mode/wrap combination
                // actually produced.
                diff_view::move_cursor(s, isize::MAX / 2, *cx.diff_rendered_len, height);
            }
        }
        // Horizontal scroll of the content column, same
        // shape as the File Insight overlay: independent of
        // the cursor, and a no-op while word wrap is on (a
        // wrapped line already fits the width).
        KeyCode::Char('h') | KeyCode::Left if diff_owns_keys(cx) => {
            if let Some(s) = cx.diff_view_state.as_mut() {
                s.scroll_h(-4);
            }
        }
        KeyCode::Char('l') | KeyCode::Right if diff_owns_keys(cx) => {
            if let Some(s) = cx.diff_view_state.as_mut() {
                s.scroll_h(4);
            }
        }
        // The hunk-under-the-cursor menu: the keyboard/`.` twin of the hover
        // buttons and of a right-click on the cursor row
        // -- the hunk's rows plus, on a changed row, its
        // line rows (or the selection's). Built by the
        // same `open_diff_hunk_menu` the right-click
        // uses, anchored on the cursor row.
        _ if km.is(Bind::GlobalContextMenu, &key) && diff_owns_keys(cx) => {
            if let Some(s) = cx.diff_view_state.as_ref() {
                let row = cx.diff_body_area.y + 1 + s.cursor.saturating_sub(s.scroll) as u16;
                match open_diff_hunk_menu(s, &*cx.diff_rows, s.cursor, cx.diff_body_area.x + 2, row)
                {
                    Some(menu) => *cx.open_menu = OpenMenu::DiffHunk(menu),
                    None => *cx.last_message = Some(msg::input::NO_HUNK_UNDER_CURSOR.get().into()),
                }
            }
        }
        // Stage/unstage the hunk under the cursor. Space
        // on purpose: it is what the third pane's file
        // rows already use for exactly this, one
        // granularity up.
        _ if km.is(Bind::DiffStageHunk, &key) && diff_owns_keys(cx) => {
            if let Some(s) = cx.diff_view_state.as_mut() {
                *cx.last_message = Some(apply_diff_stage(
                    DiffStageAction::Hunk,
                    s,
                    &*cx.diff_rows,
                    None,
                    cx.link,
                    &mut *cx.input_mode,
                ));
            }
        }
        // Discard: `x` the hunk under
        // the cursor, `X` the selected lines (or the
        // cursor's line) -- the Space / Enter pair's
        // destructive twins, both through the confirm
        // modal, Unstaged half only.
        _ if (km.is(Bind::DiffDiscardHunk, &key) || km.is(Bind::DiffDiscardLines, &key))
            && diff_owns_keys(cx) =>
        {
            if let Some(s) = cx.diff_view_state.as_mut() {
                *cx.last_message = Some(apply_diff_stage(
                    if km.is(Bind::DiffDiscardHunk, &key) {
                        DiffStageAction::DiscardHunk
                    } else {
                        DiffStageAction::DiscardLines
                    },
                    s,
                    &*cx.diff_rows,
                    None,
                    cx.link,
                    &mut *cx.input_mode,
                ));
            }
        }
        // Visual-style line selection: `v` anchors it here
        // (or drops it), j/k then extend it, Enter applies
        // it.
        _ if km.is(Bind::DiffSelectLines, &key) && diff_owns_keys(cx) => {
            if let Some(s) = cx.diff_view_state.as_mut() {
                *cx.last_message = Some(apply_diff_stage(
                    DiffStageAction::SelectLines,
                    s,
                    &*cx.diff_rows,
                    None,
                    cx.link,
                    &mut *cx.input_mode,
                ));
            }
        }
        // The key that stages lines does nothing useful on a conflicted
        // file's diff (git has no hunks to stage there): it opens the merge
        // editor instead, as Enter does on the file's row.
        _ if km.is(Bind::DiffStageLines, &key)
            && diff_owns_keys(cx)
            && cx
                .diff_view_state
                .as_ref()
                .is_some_and(|d| cx.conflicts.iter().any(|c| c.path == d.path)) =>
        {
            commands::open_selected(cx);
        }
        _ if km.is(Bind::DiffStageLines, &key) && diff_owns_keys(cx) => {
            if let Some(s) = cx.diff_view_state.as_mut() {
                *cx.last_message = Some(apply_diff_stage(
                    DiffStageAction::Lines,
                    s,
                    &*cx.diff_rows,
                    None,
                    cx.link,
                    &mut *cx.input_mode,
                ));
            }
        }
        // Next/previous hunk -- the bar's up/down arrows.
        _ if (km.is(Bind::DiffNextHunk, &key) || km.is(Bind::DiffPrevHunk, &key))
            && diff_owns_keys(cx) =>
        {
            if let Some(s) = cx.diff_view_state.as_mut() {
                if let Err(why) = diff_view::jump_to_hunk(
                    s,
                    &*cx.diff_hunk_starts,
                    km.is(Bind::DiffNextHunk, &key),
                    cx.diff_body_area.height.saturating_sub(2),
                ) {
                    *cx.last_message = Some(why.into());
                }
            }
        }
        _ if km.is(Bind::DiffToggleLayout, &key) && diff_owns_keys(cx) => {
            if let Some(s) = cx.diff_view_state.as_mut() {
                let next = s.mode.next();
                persist_diff_view_toggle(DiffViewToggle::Mode(next), s, cx.diff_config_path);
                *cx.last_message = Some(
                    match s.mode {
                        crate::diff_view::DiffViewMode::Inline => msg::input::DIFF_LAYOUT_INLINE,
                        crate::diff_view::DiffViewMode::Split => msg::input::DIFF_LAYOUT_SPLIT,
                    }
                    .get()
                    .into(),
                );
            }
        }
        // Hunks only <-> the whole file.
        // Capital: `h` scrolls left in the diff.
        _ if km.is(Bind::DiffToggleHunksOnly, &key) && diff_owns_keys(cx) => {
            if let Some(s) = cx.diff_view_state.as_mut() {
                persist_diff_view_toggle(DiffViewToggle::ToggleHunksOnly, s, cx.diff_config_path);
                *cx.last_message = Some(
                    if s.hunks_only {
                        msg::input::DIFF_HUNKS_ONLY
                    } else {
                        msg::input::DIFF_WHOLE_FILE
                    }
                    .get()
                    .into(),
                );
            }
        }
        _ if km.is(Bind::DiffChooseEncoding, &key) && diff_owns_keys(cx) => {
            if let Some(s) = cx.diff_view_state.as_ref() {
                crate::diff_encoding_menu::open(s.encoding);
            }
        }
        _ if km.is(Bind::DiffToggleMinimap, &key) && diff_owns_keys(cx) => {
            *cx.last_message = Some(toggle_minimap(cx.diff_config_path));
        }
        _ if km.is(Bind::DiffToggleWrap, &key) && diff_owns_keys(cx) => {
            if let Some(s) = cx.diff_view_state.as_mut() {
                // Wrapping changes how many rows a line
                // occupies, so it changes the numbering
                // the cursor and the selection live in --
                // the same reason the mode and whitespace
                // toggles reset it. `persist_diff_view_toggle`
                // -> `DiffViewToggle::apply` does that.
                persist_diff_view_toggle(DiffViewToggle::ToggleWordWrap, s, cx.diff_config_path);
                *cx.last_message = Some(
                    if s.word_wrap {
                        msg::input::WORD_WRAP_ON
                    } else {
                        msg::input::WORD_WRAP_OFF
                    }
                    .get()
                    .into(),
                );
            }
        }
        _ if km.is(Bind::DiffToggleWhitespace, &key) && diff_owns_keys(cx) => {
            if let Some(s) = cx.diff_view_state.as_mut() {
                persist_diff_view_toggle(DiffViewToggle::ToggleWhitespace, s, cx.diff_config_path);
                *cx.last_message = Some(
                    if s.show_whitespace {
                        msg::input::WHITESPACE_SHOWN
                    } else {
                        msg::input::WHITESPACE_HIDDEN
                    }
                    .get()
                    .into(),
                );
            }
        }
        _ => return Ok(Flow::Continue),
    }
    Ok(Flow::Handled)
}

/// The diff view is open and has the navigation keys. A pane jump away from
/// it (`commands::release_diff_keys`) hands them back to the focused list
/// while the diff stays on screen.
fn diff_owns_keys(cx: &CommandCtx<'_>) -> bool {
    *cx.central_view == CentralView::Diff
        && cx
            .diff_view_state
            .as_ref()
            .is_none_or(|s| !s.lists_have_keys)
}
