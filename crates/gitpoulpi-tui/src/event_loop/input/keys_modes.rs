//! Keys while an input mode other than Normal has the keyboard: the commit
//! prompt, the find bar, forms, the leader and navigation prefixes, the
//! command palette and the tab picker.

use super::commands::{self, CommandCtx, Flow};
use super::{keys_prefix, KeyEnv};
use crate::event_loop::*;
use crossterm::event::MouseEvent;
use gitpoulpi_i18n::msg;

/// Handles `key` for the current non-Normal input mode.
pub(super) fn handle(cx: &mut CommandCtx<'_>, env: &KeyEnv<'_>, key: KeyEvent) -> io::Result<Flow> {
    let km = env.km;
    let palette_ctx = env.palette_ctx;
    match &mut *cx.input_mode {
        InputMode::ComposingCommitMessage(msg) => match key.code {
            _ if MainField::CommitPrompt.claims(&*cx.field_editor, msg, &key) => {
                edit_main_field(
                    &mut *cx.field_editor,
                    MainField::CommitPrompt,
                    msg,
                    &key,
                    &mut *cx.clipboard,
                    &mut *cx.last_message,
                );
            }
            KeyCode::Enter => {
                if !msg.trim().is_empty() {
                    cx.link.submit_action(Action::Commit {
                        message: cx.commit_settings.prepared(msg),
                        amend: false,
                        no_verify: *cx.commit_skip_hooks,
                        push: cx.commit_settings.push_after_commit,
                    });
                    *cx.last_message = Some(msg::input::committing(&msg));
                } else {
                    *cx.last_message = Some(msg::input::COMMIT_MESSAGE_EMPTY.get().into());
                }
                *cx.input_mode = InputMode::Normal;
            }
            KeyCode::Esc => *cx.input_mode = InputMode::Normal,
            _ => {}
        },
        InputMode::FilteringRefs(draft) => match key.code {
            _ if MainField::RefsFilter.claims(&*cx.field_editor, draft, &key) => {
                edit_main_field(
                    &mut *cx.field_editor,
                    MainField::RefsFilter,
                    draft,
                    &key,
                    &mut *cx.clipboard,
                    &mut *cx.last_message,
                );
            }
            KeyCode::Enter => {
                // Folders holding a match open themselves in the filter's own
                // view, see `tree_fold::FilterOverlay`.
                *cx.refs_filter = draft.clone();
                *cx.input_mode = InputMode::Normal;
            }
            KeyCode::Esc => *cx.input_mode = InputMode::Normal,
            _ => {}
        },
        // Every key here is swallowed by this arm: while the
        // find bar has the keyboard, typing "q" types a `q`
        // rather than quitting the app, and "3" types a `3`
        // rather than switching panes.
        InputMode::SearchingLog(search) => {
            let mut closed = false;
            match key.code {
                _ if MainField::LogFind.claims(&*cx.field_editor, &search.query, &key) => {
                    // Editing the query invalidates the old
                    // position: jump back to the first hit
                    // rather than leaving the counter pointing
                    // at a match that no longer exists.
                    if edit_main_field(
                        &mut *cx.field_editor,
                        MainField::LogFind,
                        &mut search.query,
                        &key,
                        &mut *cx.clipboard,
                        &mut *cx.last_message,
                    ) == text_edit::KeyEdit::Edited
                    {
                        search.current = 0;
                    }
                }
                KeyCode::Esc => closed = true,
                KeyCode::Enter | KeyCode::Down => {
                    let hits = log_search_hits(
                        &*cx.log_result,
                        &*cx.refs_result,
                        &*cx.solo_refs,
                        &search.query,
                    );
                    let forward = !key.modifiers.contains(KeyModifiers::SHIFT);
                    search.current = log_search::step(search.current, hits.len(), forward);
                }
                KeyCode::Up => {
                    let hits = log_search_hits(
                        &*cx.log_result,
                        &*cx.refs_result,
                        &*cx.solo_refs,
                        &search.query,
                    );
                    search.current = log_search::step(search.current, hits.len(), false);
                }
                _ => {}
            }
            if closed {
                *cx.input_mode = InputMode::Normal;
            } else if let InputMode::SearchingLog(search) = &mut *cx.input_mode {
                // Move the Log's selection onto the current
                // hit, which is also what scrolls it into view
                // -- ratatui's `TableState` follows its
                // selection, so no manual offset arithmetic
                // (and no hand-rolled `log_area.y + ...`) is
                // needed here.
                let hits = log_search_hits(
                    &*cx.log_result,
                    &*cx.refs_result,
                    &*cx.solo_refs,
                    &search.query,
                );
                search.current = log_search::clamp_current(search.current, hits.len());
                if let Some(&node_index) = hits.get(search.current) {
                    cx.log_state
                        .select(Some(cx.log_order.shown_display_index(node_index)));
                    *cx.log_follow = true;
                }
            }
        }
        InputMode::ComposingRefName { op, value } => match key.code {
            // With AI on, `Ctrl+G` in the stash prompt asks the
            // model for the message (the palette's `AI:
            // Generate Stash Message` is the same).
            _ if km.is(Bind::PromptAiStashMessage, &key)
                && matches!(op, RefNameOp::StashMessage)
                && cx.ai_ui.enabled() =>
            {
                cx.ai_ui.start_into(
                    cx.session,
                    cx.repo,
                    AiRequest::StashMessage,
                    &mut *cx.last_message,
                );
            }
            // The one prompt whose empty answer is fine.
            KeyCode::Enter if matches!(op, RefNameOp::StashMessage) => {
                let message = value.trim().to_string();
                if message.is_empty() {
                    cx.link.submit_action(Action::StashPush);
                } else {
                    cx.link.submit_action(Action::StashPushWith { message });
                }
                *cx.last_message = Some(msg::input::STASHING.get().into());
                *cx.input_mode = InputMode::Normal;
            }
            // A box an agent's proposal just opened: an Enter already on its way must
            // not confirm it.
            KeyCode::Enter if crate::ui_intent::proposal::guards_enter() => {}
            _ if MainField::RefName.claims(&*cx.field_editor, value, &key) => {
                *cx.ref_name_error = None;
                *cx.new_file_error = None;
                edit_main_field(
                    &mut *cx.field_editor,
                    MainField::RefName,
                    value,
                    &key,
                    &mut *cx.clipboard,
                    &mut *cx.last_message,
                );
            }
            KeyCode::Enter => {
                let name = value.trim().to_string();
                // Set by a two-step prompt (annotated tag: name,
                // then message) to keep the prompt open on the
                // next step instead of closing it.
                let mut next_ref_name_mode: Option<InputMode> = None;
                // Inline branch/tag naming: a refused name keeps
                // the box open with the reason on its row.
                let refused = inline_target(op, cx.refs_result.as_deref().ok().and_then(head_oid))
                    .and_then(|(kind, _)| {
                        validate_new_ref(kind, value, cx.refs_result.as_deref().unwrap_or(&[]))
                            .err()
                    });
                if let Some(why) = refused {
                    *cx.ref_name_error = Some(why);
                } else if name.is_empty()
                    && !matches!(op, RefNameOp::CreateFile(_) | RefNameOp::Setting(_))
                {
                    *cx.last_message = Some(msg::input::NAME_EMPTY.get().into());
                } else {
                    match op {
                        // Validated and created here; a refused
                        // name keeps the prompt open with the
                        // reason in its label. The new file is
                        // opened in the built-in editor and shows
                        // up in Unstaged as untracked.
                        RefNameOp::CreateFile(folder) => {
                            match crate::new_file::create(cx.repo_root, folder, &name) {
                                Ok(relative) => {
                                    cx.link.request_refresh();
                                    *cx.last_message =
                                        Some(msg::input::created(relative.display()));
                                    if let Err(why) = open_editor(
                                        cx.repo_root,
                                        &relative,
                                        head_oid(cx.refs_result.as_deref().unwrap_or(&[])),
                                        cx.link,
                                        &mut *cx.editor,
                                        &mut *cx.file_insight,
                                        &*cx.diff_view_state,
                                    ) {
                                        *cx.last_message = Some(why);
                                    }
                                }
                                Err(why) => {
                                    // Keeps the prompt open (see
                                    // the check below) and shows
                                    // the reason in its label.
                                    *cx.ref_name_error = Some(why.clone());
                                    *cx.new_file_error = Some(why);
                                }
                            }
                        }
                        // The page's own validation: a refused value keeps the
                        // prompt open with the reason in its label.
                        RefNameOp::Setting(row) => match palette::submit_setting(*row, &name) {
                            Ok(message) => *cx.last_message = Some(message),
                            Err(why) => {
                                *cx.ref_name_error = Some(why.clone());
                                *cx.new_file_error = Some(why.clone());
                                *cx.last_message = Some(why);
                            }
                        },
                        RefNameOp::NewBranch => {
                            match cx.refs_result.as_ref().ok().and_then(|refs| head_oid(refs)) {
                                Some(at) => {
                                    cx.link.submit_action(Action::BranchCreate {
                                        name: name.clone(),
                                        at,
                                    });
                                    *cx.last_message = Some(msg::input::creating_branch(&name));
                                }
                                None => {
                                    *cx.last_message =
                                        Some(msg::input::NO_COMMIT_FOR_BRANCH.get().into());
                                }
                            }
                        }
                        RefNameOp::NewTag => {
                            match cx.refs_result.as_ref().ok().and_then(|refs| head_oid(refs)) {
                                Some(at) => {
                                    cx.link.submit_action(Action::TagCreate {
                                        name: name.clone(),
                                        at,
                                    });
                                    *cx.last_message = Some(msg::input::creating_tag(&name));
                                }
                                None => {
                                    *cx.last_message =
                                        Some(msg::input::NO_COMMIT_FOR_TAG.get().into());
                                }
                            }
                        }
                        // The `...Here` twins: the commit is
                        // already chosen, so there is no HEAD
                        // to resolve and no way for this to
                        // fail on an empty repository.
                        RefNameOp::NewBranchAt(at) => {
                            crate::ui_intent::proposal::confirmed(
                                crate::ui_intent::proposal::Dialog::Branch(*at),
                                &name,
                            );
                            cx.link.submit_action(Action::BranchCreate {
                                name: name.clone(),
                                at: *at,
                            });
                            *cx.last_message = Some(msg::input::creating_branch_at(
                                &name,
                                &at.to_hex()[..SHORT_SHA_LEN],
                            ));
                        }
                        RefNameOp::NewTagAt(at) => {
                            crate::ui_intent::proposal::confirmed(
                                crate::ui_intent::proposal::Dialog::Tag(*at),
                                &name,
                            );
                            cx.link.submit_action(Action::TagCreate {
                                name: name.clone(),
                                at: *at,
                            });
                            *cx.last_message = Some(msg::input::creating_tag_at(
                                &name,
                                &at.to_hex()[..SHORT_SHA_LEN],
                            ));
                        }
                        RefNameOp::NewAnnotatedTagAt(at) => {
                            // An agent's tag proposal pre-fills the message.
                            let message =
                                crate::ui_intent::proposal::tag_message(*at).unwrap_or_default();
                            if !message.is_empty() {
                                cx.field_editor.reset();
                            }
                            next_ref_name_mode = Some(InputMode::ComposingRefName {
                                op: RefNameOp::AnnotatedTagMessage {
                                    at: *at,
                                    name: name.clone(),
                                },
                                value: message,
                            });
                        }
                        // `name` is the message here, as for
                        // `AnnotateTag`.
                        RefNameOp::AnnotatedTagMessage { at, name: tag } => {
                            crate::ui_intent::proposal::confirmed(
                                crate::ui_intent::proposal::Dialog::Tag(*at),
                                &format!("{tag}\n\n{name}"),
                            );
                            cx.link.submit_action(Action::TagCreateAnnotated {
                                name: tag.clone(),
                                at: *at,
                                message: name.clone(),
                            });
                            *cx.last_message = Some(msg::input::creating_annotated_tag_at(
                                &tag,
                                &at.to_hex()[..SHORT_SHA_LEN],
                            ));
                        }
                        RefNameOp::RenameBranch(old_name) => {
                            cx.link.submit_action(Action::BranchRename {
                                old_name: old_name.clone(),
                                new_name: name.clone(),
                            });
                            *cx.last_message = Some(msg::input::renaming(&old_name, &name));
                        }
                        // `name` here is the *message*, not a
                        // ref name -- this op reuses the
                        // one-line prompt, not its meaning.
                        // An empty message is refused rather
                        // than sent: git would reject it and
                        // the error would read as a bug.
                        RefNameOp::AnnotateTag(tag) => {
                            if name.trim().is_empty() {
                                *cx.last_message =
                                    Some(msg::input::ANNOTATED_TAG_NEEDS_MESSAGE.get().into());
                            } else {
                                cx.link.submit_action(Action::TagAnnotate {
                                    name: tag.clone(),
                                    message: name.clone(),
                                });
                                *cx.last_message = Some(msg::input::annotating_tag(&tag));
                            }
                        }
                        // `StashMessage` is handled by the arm above (it must accept an empty
                        // message).
                        RefNameOp::StashMessage => {}
                        RefNameOp::SetUpstream(branch) => {
                            cx.link.submit_action(Action::BranchSetUpstream {
                                branch: branch.clone(),
                                upstream: Some(name.clone()),
                            });
                            *cx.last_message = Some(msg::input::setting_upstream(&branch, &name));
                        }
                        // Several patterns may be typed, separated
                        // by spaces (`*.psd *.ai`).
                        RefNameOp::LfsTrack => {
                            let patterns: Vec<String> =
                                name.split_whitespace().map(str::to_string).collect();
                            cx.link.submit_action(Action::LfsTrack {
                                patterns,
                                opts: gitpoulpi_core::domain::LfsTrackOptions::default(),
                            });
                            *cx.last_message = Some(msg::input::lfs_tracking(&name));
                        }
                        RefNameOp::LfsUntrack => {
                            cx.link.submit_action(Action::LfsUntrack(
                                name.split_whitespace().map(str::to_string).collect(),
                            ));
                            *cx.last_message = Some(msg::input::lfs_untracking(&name));
                        }
                    }
                }
                if cx.ref_name_error.is_none() {
                    *cx.new_file_error = None;
                    *cx.input_mode = next_ref_name_mode.take().unwrap_or(InputMode::Normal);
                }
            }
            KeyCode::Esc => {
                *cx.new_file_error = None;
                if cx.ai_ui.running() {
                    cx.ai_ui.cancel();
                }
                *cx.input_mode = InputMode::Normal;
            }
            _ => {}
        },
        InputMode::Confirm { prompt: _, action } => match key.code {
            KeyCode::Enter | KeyCode::Char('y') | KeyCode::Char('Y') => {
                let action = action.clone();
                *cx.input_mode = InputMode::Normal;
                if let Some(message) = confirm_now(
                    action,
                    cx.session,
                    cx.link,
                    cx.repo,
                    &mut *cx.repo_dialogs,
                    &mut *cx.repo_tabs,
                    &mut *cx.active_tab,
                    &mut *cx.switch_to,
                    &mut *cx.close_window,
                ) {
                    *cx.last_message = Some(message);
                }
                *cx.should_quit |= cx.switch_to.is_some() || *cx.close_window;
            }
            KeyCode::Esc | KeyCode::Char('n') | KeyCode::Char('N') => {
                *cx.input_mode = InputMode::Normal;
                *cx.last_message = Some(msg::input::CANCELED.get().into());
            }
            _ => {}
        },
        // The Edit commit message popup owns every key while
        // open (`reword_ui.rs` has the whole keyboard model).
        InputMode::EditingCommitMessage(form) => {
            if key.kind == KeyEventKind::Press && form.badges {
                // The key after `Ctrl+N`: a badge presses its control, anything
                // else (Esc included) just takes the badges away.
                form.badges = false;
                if let Some((col, row)) = badge_cell(cx, key) {
                    for kind in [
                        MouseEventKind::Down(MouseButton::Left),
                        MouseEventKind::Up(MouseButton::Left),
                    ] {
                        let click = MouseEvent {
                            kind,
                            column: col,
                            row,
                            modifiers: KeyModifiers::NONE,
                        };
                        super::mouse_modes::editing_commit_message(cx, click)?;
                    }
                }
                return Ok(Flow::Handled);
            }
            if key.kind == KeyEventKind::Press
                && km.is(Bind::GlobalNavPrefix, &key)
                && form.badge_ready()
            {
                form.badges = true;
                return Ok(Flow::Handled);
            }
            if key.kind == KeyEventKind::Press {
                match form.on_key(key) {
                    RewordFormOutcome::Stay => {}
                    RewordFormOutcome::Close => {
                        *cx.last_message = Some(form.canceled_message().into());
                        *cx.input_mode = InputMode::Normal;
                    }
                    RewordFormOutcome::Submit(action) => {
                        crate::ui_intent::proposal::reword_confirmed(&action);
                        *cx.last_message = Some(form.submitted_message());
                        cx.link.submit_action(action);
                        *cx.input_mode = InputMode::Normal;
                    }
                    RewordFormOutcome::ModifyDeeper => {
                        let commit = form.commit;
                        *cx.input_mode = InputMode::Normal;
                        super::keys_prefix::open_rewrite_from_menu(
                            cx,
                            crate::log_menu::RewriteFrom::FromHere,
                            commit,
                        );
                    }
                }
            }
        }
        // The interactive rebase / cherry-pick popup owns
        // every key while open, same reason as above.
        InputMode::InteractiveRebase(form) => {
            if key.kind == KeyEventKind::Press {
                match form.on_key(key) {
                    RebaseFormOutcome::Stay => {}
                    RebaseFormOutcome::Close => {
                        *cx.input_mode = InputMode::Normal;
                        *cx.last_message = Some(msg::input::REBASE_CANCELED.get().into());
                    }
                    RebaseFormOutcome::Submit(action) => {
                        *cx.last_message = Some(msg::input::RUNNING_PLAN.get().into());
                        cx.link.submit_action(action);
                        *cx.input_mode = InputMode::Normal;
                    }
                    RebaseFormOutcome::Split(commit, upstream) => {
                        commands::split_from_rebase(cx, commit, upstream)
                    }
                }
            }
        }
        // The AI commit-composer review owns every key while open.
        InputMode::ComposeCommits(form) => {
            if key.kind == KeyEventKind::Press
                && key.code == KeyCode::Char('p')
                && key.modifiers.contains(KeyModifiers::CONTROL)
                && !form.is_moving()
                && !form.is_editing()
            {
                if let InputMode::ComposeCommits(f) = std::mem::replace(
                    &mut *cx.input_mode,
                    InputMode::CommandPalette {
                        draft: String::new(),
                        selected: 0,
                    },
                ) {
                    *cx.compose_stash = Some(f);
                }
            } else if key.kind == KeyEventKind::Press {
                match form.on_key(key) {
                    ComposeOutcome::Stay => {}
                    ComposeOutcome::Close => {
                        *cx.input_mode = InputMode::Normal;
                        *cx.last_message = Some(msg::input::COMPOSER_CANCELED.get().into());
                    }
                    ComposeOutcome::Submit(action) => {
                        *cx.last_message = Some(msg::input::CREATING_COMMITS.get().into());
                        cx.link.submit_action(action);
                        *cx.input_mode = InputMode::Normal;
                    }
                }
            }
        }
        // The split-commit editor owns every key while open.
        InputMode::SplitCommit(form) => {
            if key.kind == KeyEventKind::Press {
                match form.on_key(key) {
                    SplitFormOutcome::Stay => {}
                    SplitFormOutcome::Close => {
                        let back = form.return_to();
                        *cx.input_mode = InputMode::Normal;
                        if let Some(upstream) = back {
                            commands::reopen_rebase(cx, upstream);
                        }
                    }
                    // Stays open until the daemon answers: a refusal keeps the
                    // plan (`notices.rs` closes it on success).
                    SplitFormOutcome::Submit(action) => {
                        crate::ui_intent::proposal::split_confirmed(&action);
                        *cx.last_message = Some(msg::split_ui::SPLITTING.get().into());
                        cx.link.submit_action(action);
                        form.begin_applying();
                    }
                }
            }
        }
        // The history rewrite workbench owns every key while open.
        InputMode::Rewrite(view) => {
            if key.kind == KeyEventKind::Press && view.badges {
                // The key after `Ctrl+N`, as for the amend dialog above.
                view.badges = false;
                if let Some((col, row)) = badge_cell(cx, key) {
                    for kind in [
                        MouseEventKind::Down(MouseButton::Left),
                        MouseEventKind::Up(MouseButton::Left),
                    ] {
                        let click = MouseEvent {
                            kind,
                            column: col,
                            row,
                            modifiers: KeyModifiers::NONE,
                        };
                        super::mouse_modes::rewrite(cx, click)?;
                    }
                }
                return Ok(Flow::Handled);
            }
            if key.kind == KeyEventKind::Press
                && km.is(Bind::GlobalNavPrefix, &key)
                && view.badge_ready()
            {
                view.badges = true;
                return Ok(Flow::Handled);
            }
            if key.kind == KeyEventKind::Press {
                match view.on_key(key) {
                    crate::rewrite_view::RewriteKeyOutcome::Stay => {}
                    crate::rewrite_view::RewriteKeyOutcome::Close => {
                        view.close();
                        *cx.input_mode = InputMode::Normal;
                    }
                    crate::rewrite_view::RewriteKeyOutcome::Undo => {
                        // Closed first: a second Undo here would take back an
                        // older entry than the rewrite the panel showed.
                        view.close();
                        *cx.input_mode = InputMode::Normal;
                        commands::run_command(cx, PaletteCommand::Undo);
                    }
                    crate::rewrite_view::RewriteKeyOutcome::FetchReview => {
                        // A rejected push: look at what the remote has now.
                        view.close();
                        *cx.input_mode = InputMode::Normal;
                        commands::run_command(cx, PaletteCommand::Fetch);
                    }
                }
            }
        }
        InputMode::Leader => return keys_prefix::leader(cx, env, key),
        InputMode::Navigation => return keys_prefix::navigation(cx, key),
        InputMode::CommandPalette { draft, selected } => {
            // Recomputed on every keystroke, not persisted --
            // it's cheap (a few dozen entries) and always
            // needs to match exactly what the popup is
            // currently showing.
            let filtered_count = || palette_matches(draft, palette_ctx).len();
            let nav_to = crate::popup_list::nav(
                &key,
                *selected,
                filtered_count(),
                crate::popup_list::View::page(),
            );
            match key.code {
                _ if nav_to.is_some() => *selected = nav_to.unwrap_or(0),
                _ if MainField::Palette.claims(&*cx.field_editor, draft, &key) => {
                    if edit_main_field(
                        &mut *cx.field_editor,
                        MainField::Palette,
                        draft,
                        &key,
                        &mut *cx.clipboard,
                        &mut *cx.last_message,
                    ) == text_edit::KeyEdit::Edited
                    {
                        *selected = 0;
                    }
                }
                KeyCode::Esc => *cx.input_mode = InputMode::Normal,
                // Alt+K: Settings > Keyboard & mouse, filtered on the row's
                // action (the row's key button).
                _ if km.is(Bind::PaletteConfigureShortcut, &key) => {
                    let matches = palette_matches(draft, palette_ctx);
                    match matches
                        .get(*selected)
                        .and_then(|m| palette::bind_action(m.entry))
                    {
                        Some(action) => {
                            *cx.input_mode = InputMode::Normal;
                            let mut state = PreferencesState::open(cx.repo_root);
                            state.show_keybindings_for(action);
                            *cx.preferences = Some(state);
                        }
                        None => {
                            *cx.last_message =
                                Some(gitpoulpi_i18n::msg::palette::NO_OWN_SHORTCUT.get().into());
                        }
                    }
                }
                // Navigation is `popup_list::nav` (arrows, Ctrl+N/P/J/K, Home/End,
                // PgUp/PgDn), above, never plain j/k: this is a text field, and the
                // catalogue is full of commands you can't type without those letters
                // ("hunk", "blame", "stash pop").
                KeyCode::Enter => 'dispatch: {
                    let matches = palette_matches(draft, palette_ctx);
                    let chosen = matches
                        .get(*selected)
                        .map(|m| (m.entry.name(), m.entry.command, m.unavailable));
                    // `Settings: <label>...` on a choice narrows the palette to
                    // its choices instead of running.
                    if let Some((_, PaletteCommand::Setting(op), None)) = chosen {
                        if let Some(query) = palette::setting_sub_query(op) {
                            *draft = query;
                            *selected = 0;
                            let end = draft.len();
                            cx.field_editor
                                .caret_mut(MainField::Palette)
                                .set_cursor(draft, end, false);
                            break 'dispatch;
                        }
                    }
                    *cx.input_mode = InputMode::Normal;
                    let Some((name, cmd, unavailable)) = chosen else {
                        *cx.last_message =
                            Some(gitpoulpi_i18n::msg::palette::NO_MATCH_STATUS.get().into());
                        break 'dispatch;
                    };
                    // An inapplicable command reports why
                    // instead of half-running: the popup
                    // already grayed it out, so reaching here
                    // means the user asked on purpose and is
                    // owed the reason. `break` out of this
                    // block, never `continue` -- the rest of
                    // the loop iteration still has to run.
                    if let Some(reason) = unavailable {
                        *cx.last_message = Some(msg::input::command_unavailable(name, reason));
                        // Home covers the window's status bar:
                        // its own footer shows the reason.
                        if let Some(manager) = cx.management.as_mut() {
                            manager.message = cx.last_message.clone();
                        }
                        break 'dispatch;
                    }
                    *cx.last_message = Some(msg::input::ran(name));
                    commands::run_command(cx, cmd);
                }
                _ => {}
            }
        }
        // The tab picker: typing narrows the
        // list (through the shared `MainField`/`field_editor`
        // machinery, same as the command palette's own draft),
        // Up/Down or Ctrl+P/Ctrl+N move the selection, Enter
        // switches to the selected (*filtered*) tab, Esc
        // closes with nothing done.
        InputMode::TabPicker { draft, selected } => {
            let filtered_count = || filter_tab_indices(&*cx.repo_tabs, draft).len();
            let nav_to = crate::popup_list::nav(
                &key,
                *selected,
                filtered_count(),
                crate::popup_list::View::page(),
            );
            match key.code {
                _ if nav_to.is_some() => *selected = nav_to.unwrap_or(0),
                // Alt+Up/Down reorder the highlighted tab (the keyboard twin
                // of dragging its row); the highlight follows the tab.
                KeyCode::Up | KeyCode::Down if key.modifiers.contains(KeyModifiers::ALT) => {
                    let indices = filter_tab_indices(&*cx.repo_tabs, draft);
                    let moved = indices.get(*selected).copied().and_then(|from| {
                        tab_move_neighbour(from, cx.repo_tabs.len(), key.code == KeyCode::Down)
                            .map(|to| (from, to))
                    });
                    if let Some((from, to)) = moved {
                        *cx.last_message = reorder_tab(
                            cx.session,
                            &mut *cx.repo_tabs,
                            &mut *cx.active_tab,
                            from,
                            to,
                        );
                        if let Some(at) = filter_tab_indices(&*cx.repo_tabs, draft)
                            .iter()
                            .position(|i| *i == to)
                        {
                            *selected = at;
                        }
                    }
                }
                _ if MainField::TabPicker.claims(&*cx.field_editor, draft, &key) => {
                    if edit_main_field(
                        &mut *cx.field_editor,
                        MainField::TabPicker,
                        draft,
                        &key,
                        &mut *cx.clipboard,
                        &mut *cx.last_message,
                    ) == text_edit::KeyEdit::Edited
                    {
                        *selected = 0;
                    }
                }
                KeyCode::Esc => *cx.input_mode = InputMode::Normal,
                KeyCode::Enter => {
                    let indices = filter_tab_indices(&*cx.repo_tabs, draft);
                    let chosen = indices.get(*selected).copied();
                    *cx.input_mode = InputMode::Normal;
                    match chosen {
                        Some(index) => {
                            if let Some(message) = apply_switcher_action(
                                repo_switcher::SwitcherAction::Focus(index),
                                cx.session,
                                cx.link,
                                cx.repo,
                                &mut *cx.repo_dialogs,
                                &mut *cx.repo_tabs,
                                &mut *cx.active_tab,
                                &mut *cx.switch_to,
                                &mut *cx.close_window,
                            ) {
                                *cx.last_message = Some(message);
                            }
                            *cx.should_quit |= cx.switch_to.is_some() || *cx.close_window;
                        }
                        None => *cx.last_message = Some(msg::input::NO_MATCHING_TAB.get().into()),
                    }
                }
                _ => {}
            }
        }
        // The theme picker: same keys as the tab picker; every move of the
        // highlight previews the theme, Enter keeps it and writes
        // `ui.theme`, Esc restores the previous one.
        InputMode::ThemePicker(picker) => {
            let nav_to = crate::popup_list::nav(
                &key,
                picker.selected,
                picker.filtered().len(),
                crate::popup_list::View::page(),
            );
            let alt = key.modifiers.contains(KeyModifiers::ALT);
            match key.code {
                _ if nav_to.is_some() => picker.selected = nav_to.unwrap_or(0),
                // The prefilter's keyboard twin: Tab / Shift+Tab cycle it,
                // Alt+1..4 pick All / Dark / Light / Colored.
                KeyCode::Tab => picker.set_filter(picker.filter.next(false)),
                KeyCode::BackTab => picker.set_filter(picker.filter.next(true)),
                KeyCode::Char(c @ '1'..='4') if alt => picker
                    .set_filter(crate::theme_picker::ThemeFilter::ALL[(c as u8 - b'1') as usize]),
                _ if MainField::ThemePicker.claims(&*cx.field_editor, &picker.draft, &key) => {
                    if edit_main_field(
                        &mut *cx.field_editor,
                        MainField::ThemePicker,
                        &mut picker.draft,
                        &key,
                        &mut *cx.clipboard,
                        &mut *cx.last_message,
                    ) == text_edit::KeyEdit::Edited
                    {
                        picker.selected = 0;
                    }
                }
                KeyCode::Esc => {
                    picker.restore();
                    *cx.input_mode = InputMode::Normal;
                }
                KeyCode::Enter => {
                    *cx.last_message = Some(picker.commit());
                    *cx.input_mode = InputMode::Normal;
                }
                _ => {}
            }
            if let InputMode::ThemePicker(picker) = &mut *cx.input_mode {
                picker.sync_preview();
            }
        }
        // The language / graph color picker: the theme picker's keys and live
        // preview; Enter writes the key, Esc restores the previous value.
        InputMode::LanguagePicker(picker) => {
            match key.code {
                _ if crate::popup_list::nav(
                    &key,
                    picker.selected,
                    picker.filtered().len(),
                    crate::popup_list::View::page(),
                )
                .is_some() =>
                {
                    picker.selected = crate::popup_list::nav(
                        &key,
                        picker.selected,
                        picker.filtered().len(),
                        crate::popup_list::View::page(),
                    )
                    .unwrap_or(0);
                }
                _ if MainField::LanguagePicker.claims(&*cx.field_editor, &picker.draft, &key) => {
                    if edit_main_field(
                        &mut *cx.field_editor,
                        MainField::LanguagePicker,
                        &mut picker.draft,
                        &key,
                        &mut *cx.clipboard,
                        &mut *cx.last_message,
                    ) == text_edit::KeyEdit::Edited
                    {
                        picker.selected = 0;
                    }
                }
                KeyCode::Esc => {
                    picker.restore(&mut *cx.ui_graph_theme);
                    *cx.input_mode = InputMode::Normal;
                }
                KeyCode::Enter if picker.kind == crate::language_picker::PickerKind::RuleFile => {
                    // Writes the rule into the file chosen. Nothing matching
                    // the filter keeps the picker open.
                    if let Some((action, message)) = picker.rule_file_action() {
                        if let (Some(job), crate::daemon_link::Action::TreatAs { treat, .. }) =
                            (picker.job.as_ref(), &action)
                        {
                            // The file menu's next row reads what this will do.
                            for path in &job.paths {
                                crate::binary_hint::record(
                                    cx.repo_root,
                                    std::path::Path::new(path),
                                    *treat == gitpoulpi_core::domain::Treat::Binary,
                                );
                            }
                        }
                        let (extra, file) = picker.rule_file_extra_actions();
                        let many = extra.len() + 1;
                        cx.link.submit_action(action);
                        for more in extra {
                            cx.link.submit_action(more);
                        }
                        *cx.last_message = Some(if many > 1 {
                            msg::pane_menu::ignoring_many(many, file)
                        } else {
                            message
                        });
                        *cx.input_mode = InputMode::Normal;
                    } else {
                        *cx.last_message = Some(msg::overlays::NO_MATCHING_RULE_FILE.get().into());
                    }
                }
                KeyCode::Enter => {
                    *cx.last_message = Some(picker.commit());
                    if picker.kind == crate::language_picker::PickerKind::GraphColorTheme {
                        *cx.ui_graph_theme = crate::graph::LaneTheme::from_slug(&picker.current);
                    }
                    *cx.input_mode = InputMode::Normal;
                }
                _ => {}
            }
            if let InputMode::LanguagePicker(picker) = &mut *cx.input_mode {
                picker.sync_preview(&mut *cx.ui_graph_theme);
            }
        }
        // The workspace search: the pickers' list keys (headings are skipped),
        // Alt+1..8 toggle the kind chips (Alt+0: all), Enter goes there.
        InputMode::WorkspaceSearch(search) => {
            let nav_to = crate::popup_list::nav(
                &key,
                search.selected,
                search.rows().len(),
                crate::popup_list::View::page(),
            );
            match key.code {
                _ if nav_to.is_some() => search.move_to(nav_to.unwrap_or(0)),
                _ if search.on_chip_key(&key) => {}
                _ if MainField::WorkspaceSearch.claims(&*cx.field_editor, &search.draft, &key) => {
                    if edit_main_field(
                        &mut *cx.field_editor,
                        MainField::WorkspaceSearch,
                        &mut search.draft,
                        &key,
                        &mut *cx.clipboard,
                        &mut *cx.last_message,
                    ) == text_edit::KeyEdit::Edited
                    {
                        search.edited();
                    }
                }
                KeyCode::Esc => *cx.input_mode = InputMode::Normal,
                // Alt+R: review the highlighted pull / merge request in the
                // app, in its repository's window.
                KeyCode::Char('r')
                    if crate::build_features::REVIEW
                        && key.modifiers.contains(KeyModifiers::ALT) =>
                {
                    match (search.review_request(), search.go_intent()) {
                        (Some((repo, number)), Some(intent)) => {
                            crate::radar::pending::set(crate::radar::pending::Pending::Review {
                                number,
                                repo: Some(repo),
                            });
                            cx.ui_viewer.go(intent);
                            *cx.input_mode = InputMode::Normal;
                        }
                        _ => {
                            *cx.last_message = Some(msg::search::NOT_A_REQUEST.get().into());
                        }
                    }
                }
                KeyCode::Enter => match search.go_intent() {
                    Some(intent) => {
                        cx.ui_viewer.go(intent);
                        *cx.input_mode = InputMode::Normal;
                    }
                    None => *cx.last_message = Some(msg::search::NO_RESULT_SELECTED.get().into()),
                },
                _ => {}
            }
        }
        InputMode::Normal => {}
    }
    Ok(Flow::Handled)
}

/// The cell a pressed badge stands for, from the list the last paint handed
/// out (`nav_badge_targets`): `None` for Esc or a character with no badge.
fn badge_cell(cx: &CommandCtx<'_>, key: KeyEvent) -> Option<(u16, u16)> {
    let KeyCode::Char(c) = key.code else {
        return None;
    };
    cx.nav_badge_targets
        .iter()
        .find_map(|(badge, target)| match target {
            FocusTarget::Dialog(col, row) if *badge == c => Some((*col, *row)),
            _ => None,
        })
}
