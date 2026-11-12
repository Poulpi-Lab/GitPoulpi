use super::*;

/// The palette's sync and remote commands, in one place. `selected_remote`
/// is the REMOTE section's selection (acted on directly); without one,
/// the command asks which remote with a chooser menu. `selected_branch`
/// is a selected LOCAL branch, which Set upstream prefers over HEAD's.
pub(crate) fn run_remote_palette_command(
    command: PaletteCommand,
    selected_remote: Option<String>,
    selected_branch: Option<String>,
    ctx: RemoteUiCtx<'_>,
) {
    let (col, row) = ctx.ui.push_anchor();
    let refs = ctx.refs;
    // A command that names one remote: act on the selection, or ask.
    let on_remote = |make: fn(String) -> RemoteUiAction, caption: &str, ctx: RemoteUiCtx<'_>| {
        match selected_remote.clone() {
            Some(remote) => apply_remote_ui_action(make(remote), ctx),
            None => {
                if let Some(why) = ctx.ui.open_chooser(caption, refs, col, row, false, make) {
                    *ctx.last_message = Some(why);
                }
            }
        }
    };
    // Edit and Remove also reach a leftover namespace (no remote configured,
    // branches still there): Edit adds it back, Remove deletes its branches.
    let on_remote_or_leftover =
        |make: fn(String) -> RemoteUiAction, caption: &str, ctx: RemoteUiCtx<'_>| {
            match selected_remote.clone() {
                Some(remote) => apply_remote_ui_action(make(remote), ctx),
                None => {
                    if let Some(why) = ctx.ui.open_chooser(caption, refs, col, row, true, make) {
                        *ctx.last_message = Some(why);
                    }
                }
            }
        };
    match command {
        PaletteCommand::Fetch => {
            ctx.link.submit_action(Action::FetchAll);
            *ctx.last_message = Some(msg::FETCHING_ALL.get().into());
        }
        PaletteCommand::Pull => *ctx.last_message = Some(ctx.ui.run_pull_button(refs, ctx.link)),
        PaletteCommand::PullWithMode(mode) => {
            *ctx.last_message = Some(ctx.ui.run_pull_mode(mode, refs, ctx.link))
        }
        PaletteCommand::ChoosePullButtonAction => {
            let (c, r) = ctx.ui.pull_anchor();
            ctx.ui.open_pull_menu(c, r);
        }
        PaletteCommand::LfsMenu => {
            let (c, r) = ctx.ui.lfs_anchor();
            ctx.ui.open_lfs_menu(c, r);
        }
        PaletteCommand::Push => *ctx.last_message = ctx.ui.run_push_button(refs, ctx.link),
        PaletteCommand::PushToRemote => {
            *ctx.last_message = ctx.ui.open_push_menu(refs, col, row);
        }
        PaletteCommand::ForcePushWithLease => {
            let head = head_branch(refs);
            let remote = head.as_ref().and_then(|h| {
                remote_for_branch(h, &ctx.ui.names(), ctx.ui.default_remote(refs).as_deref())
            });
            match (head, remote) {
                (Some(head), Some(remote)) => apply_remote_ui_action(
                    RemoteUiAction::ForcePush {
                        remote,
                        branch: head.name,
                        after_fetch: false,
                    },
                    ctx,
                ),
                (None, _) => *ctx.last_message = Some(msg::DETACHED_PUSH.get().into()),
                (Some(_), None) => *ctx.last_message = Some(no_default_remote()),
            }
        }
        PaletteCommand::SetUpstream => {
            let branch = selected_branch
                .and_then(|b| local_branch(refs, &b))
                .or_else(|| head_branch(refs));
            match branch {
                Some(branch) => ctx.ui.open_upstream_menu(&branch, refs, col, row),
                None => *ctx.last_message = Some(msg::SELECT_LOCAL_BRANCH.get().into()),
            }
        }
        PaletteCommand::AddRemote => apply_remote_ui_action(RemoteUiAction::AddRemote, ctx),
        PaletteCommand::RemoveRemote => {
            on_remote_or_leftover(RemoteUiAction::RemoveRemote, msg::REMOVE_WHICH.get(), ctx)
        }
        PaletteCommand::EditRemote => {
            on_remote_or_leftover(RemoteUiAction::EditRemote, msg::EDIT_WHICH.get(), ctx)
        }
        PaletteCommand::SetDefaultRemote => on_remote(
            RemoteUiAction::SetDefaultRemote,
            msg::DEFAULT_FOR_REPO.get(),
            ctx,
        ),
        PaletteCommand::CopyRemoteName => on_remote(
            RemoteUiAction::CopyRemoteName,
            msg::COPY_NAME_WHICH.get(),
            ctx,
        ),
        PaletteCommand::CopyRemoteLink => on_remote(
            RemoteUiAction::CopyRemoteLink,
            msg::COPY_LINK_WHICH.get(),
            ctx,
        ),
        PaletteCommand::FetchRemote => {
            on_remote(RemoteUiAction::FetchRemote, msg::FETCH_WHICH.get(), ctx)
        }
        PaletteCommand::HideRemoteInGraph => {
            on_remote(RemoteUiAction::HideRemote, msg::HIDE_ALL_WHICH.get(), ctx)
        }
        PaletteCommand::ShowRemoteInGraph => {
            on_remote(RemoteUiAction::ShowRemote, msg::SHOW_ALL_WHICH.get(), ctx)
        }
        PaletteCommand::RemoteActions => match selected_remote {
            Some(remote) => {
                let hidden = crate::refs_tree::remote_hidden(&*ctx.hidden_from_graph, &remote);
                ctx.ui.open_remote_menu(&remote, hidden, refs, col, row)
            }
            None => *ctx.last_message = Some(msg::SELECT_REMOTE_FIRST.get().into()),
        },
        _ => {}
    }
}

/// The remote popup's or menu's own target under the pointer, looked up
/// among *its* targets only. It is drawn over the panes but registered
/// after them, and `find_hit_target` returns the first match - so a
/// pane target under a popup row would otherwise take the click.
pub(crate) fn remote_hit_at(hits: &[(Rect, HitTarget)], col: u16, row: u16) -> Option<HitTarget> {
    hits.iter()
        .find(|(rect, target)| {
            rect.contains((col, row).into())
                && matches!(
                    target,
                    HitTarget::RemoteMenuItem(_) | HitTarget::RemoteForm(_)
                )
        })
        .map(|(_, target)| target.clone())
}

/// Push one tag to the default remote. Returns the status line.
pub(crate) fn push_tag_now(tag: &str, ui: &RemoteUi, refs: &[Ref], link: &DaemonLink) -> String {
    match ui.default_remote(refs) {
        Some(remote) => {
            let said = msg::pushing_tag(tag, &remote);
            link.submit_action(Action::TagPush {
                remote,
                selection: TagSelection::Named(vec![tag.to_string()]),
            });
            said
        }
        None => no_default_remote(),
    }
}

/// Push every unpushed tag to the default remote.
pub(crate) fn push_all_tags_now(ui: &RemoteUi, refs: &[Ref], link: &DaemonLink) -> String {
    match ui.default_remote(refs) {
        Some(remote) => {
            let said = msg::pushing_all_tags(&remote);
            link.submit_action(Action::TagPush {
                remote,
                selection: TagSelection::AllUnpushed,
            });
            said
        }
        None => no_default_remote(),
    }
}

/// Everything [`apply_remote_ui_action`] needs besides the action.
pub(crate) struct RemoteUiCtx<'a> {
    pub(crate) ui: &'a mut RemoteUi,
    pub(crate) refs: &'a [Ref],
    pub(crate) link: &'a DaemonLink,
    pub(crate) clipboard: &'a mut clipboard::Clipboard,
    pub(crate) input_mode: &'a mut InputMode,
    pub(crate) last_message: &'a mut Option<String>,
    /// For `HideRemote`/`ShowRemote` (the remote-name menu's twin of its
    /// row's eye) and its persistence - the same `hidden_from_graph`/`repo_root` the Refs
    /// sidebar's own hide/solo actions read and write, via
    /// `refs_tree::submit_ref_visibility`.
    pub(crate) hidden_from_graph: &'a mut std::collections::HashSet<String>,
    pub(crate) solo_refs: &'a std::collections::HashSet<String>,
}

/// Runs one remote action, whichever surface it came from.
pub(crate) fn apply_remote_ui_action(action: RemoteUiAction, ctx: RemoteUiCtx<'_>) {
    let RemoteUiCtx {
        ui,
        refs,
        link,
        clipboard,
        input_mode,
        last_message,
        hidden_from_graph,
        solo_refs,
    } = ctx;
    let message: Option<String> = match action {
        RemoteUiAction::CopyRemoteName(name) => {
            Some(clipboard.copy(msg::WHAT_REMOTE_NAME.get(), &name))
        }
        RemoteUiAction::CopyRemoteLink(name) => match ui.remotes.iter().find(|r| r.name == name) {
            Some(remote) => match remote_web_url(&remote.fetch_url) {
                Some(link_url) => Some(clipboard.copy(msg::WHAT_REMOTE_LINK.get(), &link_url)),
                None => Some(msg::not_web_url(&remote.fetch_url)),
            },
            None => Some(crate::git_actions::gone(&name)),
        },
        RemoteUiAction::SetDefaultRemote(name) => {
            ui.set_chosen_default(Some(name.clone()));
            Some(msg::default_now(&name))
        }
        RemoteUiAction::EditRemote(name) => match ui.remotes.iter().find(|r| r.name == name) {
            Some(remote) => {
                ui.form = Some(RemoteForm::edit(remote, ui.names()));
                None
            }
            // Nothing to edit, but the branches say it was once a remote:
            // the form that gives it an address again, never an error.
            None if leftover_names(refs, &ui.names(), ui.loaded).contains(&name) => {
                ui.form = Some(RemoteForm::add_named(ui.names(), &name));
                None
            }
            None => Some(crate::git_actions::gone(&name)),
        },
        RemoteUiAction::AddAsRemote(name) => {
            ui.form = Some(RemoteForm::add_named(ui.names(), &name));
            None
        }
        RemoteUiAction::RemoveLeftover(name) => {
            *input_mode = InputMode::Confirm {
                prompt: msg::remove_leftover_confirm(&name),
                action: ConfirmAction::Destructive(PendingDestructive::RemoveRemote(name)),
            };
            None
        }
        RemoteUiAction::AddRemote => {
            ui.form = Some(RemoteForm::add(ui.names()));
            None
        }
        RemoteUiAction::RemoveRemote(name) => {
            let leftover = leftover_names(refs, &ui.names(), ui.loaded).contains(&name);
            *input_mode = InputMode::Confirm {
                prompt: if leftover {
                    msg::remove_leftover_confirm(&name)
                } else {
                    msg::remove_confirm(&name)
                },
                action: ConfirmAction::Destructive(PendingDestructive::RemoveRemote(name)),
            };
            None
        }
        RemoteUiAction::FetchRemote(name) => {
            link.submit_action(Action::Fetch(name.clone()));
            Some(msg::fetching_one(name))
        }
        RemoteUiAction::Flow(cmd) => {
            crate::flow::request(cmd);
            None
        }
        RemoteUiAction::Lfs(item) => {
            let remote = ui.default_remote(refs);
            Some(item.run(remote, link))
        }
        RemoteUiAction::FetchAllRemotes => {
            link.submit_action(Action::FetchAll);
            Some(msg::FETCHING_ALL.get().into())
        }
        RemoteUiAction::RunPull(mode) => Some(ui.run_pull_mode(mode, refs, link)),
        RemoteUiAction::PushTo(remote) => match head_branch(refs) {
            Some(head) => {
                link.submit_action(push_action(&remote, &head, false));
                Some(msg::pushing_to(&head.name, &remote))
            }
            None => Some(msg::DETACHED_PUSH.get().into()),
        },
        RemoteUiAction::SetUpstream { branch, upstream } => {
            let said = match &upstream {
                Some(up) => msg::setting_upstream(&branch, up),
                None => msg::unsetting_upstream(&branch),
            };
            link.submit_action(Action::BranchSetUpstream { branch, upstream });
            Some(said)
        }
        RemoteUiAction::TypeUpstream { branch } => {
            let prefill = ui
                .default_remote(refs)
                .map(|r| format!("{r}/{branch}"))
                .unwrap_or_default();
            *input_mode = InputMode::ComposingRefName {
                op: RefNameOp::SetUpstream(branch),
                value: prefill,
            };
            None
        }
        RemoteUiAction::PullThenPush { remote, branch } => {
            link.submit_action(Action::PullThenPush {
                remote: remote.clone(),
                branch: branch.clone(),
            });
            Some(msg::pulling_then_pushing(&remote, &branch))
        }
        RemoteUiAction::ForcePush {
            remote,
            branch,
            after_fetch,
        } => {
            let prompt = if after_fetch {
                msg::force_confirm_fetch(&branch, &remote)
            } else {
                msg::force_confirm(&branch, &remote)
            };
            *input_mode = InputMode::Confirm {
                prompt,
                action: ConfirmAction::Destructive(PendingDestructive::ForcePush {
                    remote,
                    branch,
                    after_fetch,
                }),
            };
            None
        }
        RemoteUiAction::HideRemote(remote) => {
            crate::refs_tree::set_remote_hidden(hidden_from_graph, &remote, true);
            refs_tree::submit_ref_visibility(link, hidden_from_graph, solo_refs);
            Some(msg::remote_hidden(&remote))
        }
        RemoteUiAction::ShowRemote(remote) => {
            crate::refs_tree::set_remote_hidden(hidden_from_graph, &remote, false);
            refs_tree::submit_ref_visibility(link, hidden_from_graph, solo_refs);
            Some(msg::remote_shown(&remote))
        }
        // The SSH & credentials page: the window opens it at its next
        // notice pass (this has no preferences to open).
        RemoteUiAction::OpenRepoAuth => {
            ui.want_repo_auth = true;
            None
        }
        RemoteUiAction::AutoStashContinue => match ui.pending_autostash.take() {
            Some(action) => {
                link.submit_action(Action::AutoStashed(Box::new(action)));
                Some(msg::STASHING_CONTINUING.get().to_string())
            }
            None => None,
        },
        // The daemon reads `general.auto_stash` on every blocked operation,
        // so the file is all there is to change. This operation goes out
        // already wrapped, whether the write worked or not.
        RemoteUiAction::AutoStashAlways => match ui.pending_autostash.take() {
            Some(action) => {
                let saved = match gitpoulpi_config::ConfigPaths::discover() {
                    Some(paths) => gitpoulpi_config::set_in_file(
                        &paths.config_file(),
                        "general.auto_stash",
                        "always",
                    )
                    .map(|_| ())
                    .map_err(|err| err.to_string()),
                    None => Err(gitpoulpi_i18n::msg::settings_screen::NO_CONFIG_DIR
                        .get()
                        .to_string()),
                };
                link.submit_action(Action::AutoStashed(Box::new(action)));
                Some(match saved {
                    Ok(()) => msg::AUTOSTASH_ALWAYS_SAVED.get().to_string(),
                    Err(err) => crate::activity_log::report_problem_clear(
                        crate::activity_log::ProblemSource::Save,
                        &msg::autostash_always_not_saved(err),
                    ),
                })
            }
            None => None,
        },
        RemoteUiAction::Cancel => {
            ui.pending_autostash = None;
            None
        }
    };
    if let Some(message) = message {
        *last_message = Some(message);
    }
}

/// Sends a validated popup's actions, and makes the default-remote
/// setting follow a rename of the default remote.
pub(crate) fn submit_remote_form(
    submit: FormSubmit,
    ui: &mut RemoteUi,
    link: &DaemonLink,
    hidden_from_graph: &mut std::collections::HashSet<String>,
    solo_refs: &std::collections::HashSet<String>,
) -> String {
    for action in submit.actions {
        link.submit_action(action);
    }
    if let Some((old, new)) = submit.renamed {
        if ui.chosen_default.as_deref() == Some(old.as_str()) {
            ui.set_chosen_default(Some(new.clone()));
        }
        // The hide follows the remote to its new name (the flag, and the
        // branches hidden one by one: their names carry the remote's).
        if crate::refs_tree::rename_remote_in_hidden(hidden_from_graph, &old, &new) {
            refs_tree::submit_ref_visibility(link, hidden_from_graph, solo_refs);
        }
    }
    submit.summary
}
