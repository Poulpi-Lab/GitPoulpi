//! The event loop's input modes (`InputMode`: normal, leader chord, text
//! prompts, confirmations, palette...) and the pending actions a
//! confirmation or a name prompt carries.

use super::*;

/// The main window's text fields, as the one shared editing core
/// (`text_edit::FieldEditor`) knows them: whichever of these has the
/// keyboard owns the window's single caret and selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MainField {
    RefsFilter,
    LogFind,
    RefName,
    Palette,
    /// The tab picker's filter field (`Ctrl+N Tab`, `Repository: Select
    /// Tab...`).
    TabPicker,
    /// The theme picker's filter field (`Ctrl+N t`, `App: Change Theme...`).
    ThemePicker,
    /// The language picker's filter field (`Ctrl+N l`, `App: Change Language...`).
    LanguagePicker,
    /// The workspace search's query field (`Ctrl+Alt+F`, `Search: Workspace`).
    WorkspaceSearch,
    /// The commit summary, in the composer or in the WIP row (one
    /// buffer, so one field).
    Summary,
    Description,
    CommitFilter,
}

impl MainField {
    /// The field that has the keyboard, from the state that says so.
    pub(crate) fn active(
        input_mode: &InputMode,
        composer_focus: Option<ComposerField>,
        commit_filter_focus: bool,
    ) -> Option<MainField> {
        match input_mode {
            InputMode::FilteringRefs(_) => Some(MainField::RefsFilter),
            InputMode::SearchingLog(_) => Some(MainField::LogFind),
            InputMode::InlineName { .. } => Some(MainField::RefName),
            InputMode::CommandPalette { .. } => Some(MainField::Palette),
            InputMode::TabPicker { .. } => Some(MainField::TabPicker),
            InputMode::ThemePicker(_) => Some(MainField::ThemePicker),
            InputMode::LanguagePicker(_) => Some(MainField::LanguagePicker),
            InputMode::WorkspaceSearch(_) => Some(MainField::WorkspaceSearch),
            InputMode::Normal => match composer_focus {
                Some(field) if field.edits_summary() => Some(MainField::Summary),
                Some(_) => Some(MainField::Description),
                None => commit_filter_focus.then_some(MainField::CommitFilter),
            },
            _ => None,
        }
    }

    pub(crate) fn multiline(self) -> bool {
        self == MainField::Description
    }

    /// Whether `key` is an editing key for this field (then it must not
    /// reach any pane shortcut).
    pub(crate) fn claims(
        self,
        editor: &crate::text_edit::FieldEditor<MainField>,
        text: &str,
        key: &KeyEvent,
    ) -> bool {
        editor.claims(self, text, key, self.multiline())
    }
}

/// Applies an editing key to main-window field `id` whose text is
/// `text`, putting a `Ctrl+C`/`Ctrl+X` selection on the clipboard. Returns
/// what the key did, so the caller can react to an edit (reset a search
/// position, a palette selection).
pub(crate) fn edit_main_field(
    editor: &mut crate::text_edit::FieldEditor<MainField>,
    id: MainField,
    text: &mut String,
    key: &KeyEvent,
    clipboard: &mut crate::clipboard::Clipboard,
    last_message: &mut Option<String>,
) -> crate::text_edit::KeyEdit {
    let edit = editor.on_key(id, text, key, id.multiline());
    if let crate::text_edit::KeyEdit::Copy(s) | crate::text_edit::KeyEdit::Cut(s) = &edit {
        *last_message = Some(clipboard.copy(gitpoulpi_i18n::msg::input::WHAT_SELECTION.get(), s));
    }
    edit
}

/// What a change to a field's text invalidates: the find bar's current
/// hit and the palette's selected row both index a list the new text
/// recomputes.
pub(crate) fn main_field_edited(id: MainField, input_mode: &mut InputMode) {
    match (id, input_mode) {
        (MainField::LogFind, InputMode::SearchingLog(search)) => search.current = 0,
        (MainField::Palette, InputMode::CommandPalette { selected, .. }) => *selected = 0,
        (MainField::TabPicker, InputMode::TabPicker { selected, .. }) => *selected = 0,
        (MainField::LanguagePicker, InputMode::LanguagePicker(picker)) => picker.selected = 0,
        (MainField::WorkspaceSearch, InputMode::WorkspaceSearch(search)) => search.edited(),
        (MainField::ThemePicker, InputMode::ThemePicker(picker)) => {
            picker.selected = 0;
            picker.sync_preview();
        }
        _ => {}
    }
}

impl MainField {
    /// The text of `field`, when it is the one `input_mode` holds or
    /// one of the composer/filter buffers.
    pub(crate) fn text<'a>(
        self,
        input_mode: &'a mut InputMode,
        commit_summary: &'a mut String,
        commit_description: &'a mut String,
        commit_files_filter: &'a mut String,
    ) -> Option<&'a mut String> {
        match (self, input_mode) {
            (MainField::RefsFilter, InputMode::FilteringRefs(s))
            | (MainField::RefName, InputMode::InlineName { value: s, .. })
            | (MainField::Palette, InputMode::CommandPalette { draft: s, .. })
            | (MainField::TabPicker, InputMode::TabPicker { draft: s, .. }) => Some(s),
            (MainField::ThemePicker, InputMode::ThemePicker(picker)) => Some(&mut picker.draft),
            (MainField::LanguagePicker, InputMode::LanguagePicker(picker)) => {
                Some(&mut picker.draft)
            }
            (MainField::WorkspaceSearch, InputMode::WorkspaceSearch(search)) => {
                Some(&mut search.draft)
            }
            (MainField::LogFind, InputMode::SearchingLog(search)) => Some(&mut search.query),
            (MainField::Summary, _) => Some(commit_summary),
            (MainField::Description, _) => Some(commit_description),
            (MainField::CommitFilter, _) => Some(commit_files_filter),
            _ => None,
        }
    }
}

/// What the render loop's `KeyCode::Char`/etc. events mean right now:
/// ordinary navigation, characters typed into the commit-message composer or
/// the refs sidebar's search filter, or waiting for the second key of a
/// `Ctrl+G` git-action chord (see `leader_hint` and the match arm in the
/// event loop).
pub(crate) enum InputMode {
    Normal,
    FilteringRefs(String),
    /// The Log's find bar is open and owns the keyboard. Holding the
    /// `LogSearch` *inside* the mode rather than in a parallel `Option` means
    /// the bar is visible exactly when this variant is active, and every
    /// global key is swallowed by this arm instead of scattered
    /// `if searching` guards.
    SearchingLog(LogSearch),
    /// `Ctrl+N` was pressed: the navigation/display prefix. The next key is
    /// looked up in `navigation_key_action` (`chrome.rs`): `h`/`l`
    /// collapse/restore the Refs/Status pane, `1`/`2`/`3` focus a sub-pane
    /// (with a number badge drawn over each candidate pane while this mode is
    /// open), `,` opens settings and `?` toggles help. `Esc` or any other key
    /// cancels back to `Normal`; unlike `Leader`, nothing stays pending.
    Navigation,
    /// Typing a new branch or tag name, a branch's new name (rename) or a new
    /// file name - see `RefNameOp` for which and what happens on Enter. The
    /// name is typed in place (a Log row, a Refs row, the Status list) or in
    /// the small name box (`ref_name_box.rs`), never drawn in the footer.
    InlineName {
        op: RefNameOp,
        value: String,
    },
    Leader,
    /// `selected` indexes into the *currently filtered* match list (by
    /// `draft`), not `PALETTE_COMMANDS` itself - clamped back into range
    /// on every keystroke since the filtered list's length changes as the
    /// user types.
    CommandPalette {
        draft: String,
        selected: usize,
    },
    /// A destructive action is awaiting y/n confirmation: branch/tag delete
    /// and stash drop route through this instead of firing immediately.
    /// `prompt` is the question shown in the footer; `action` is what
    /// Enter/`y` actually submits.
    Confirm {
        prompt: String,
        action: ConfirmAction,
    },
    /// The Edit commit message popup (`reword_ui.rs`) is open and owns
    /// the keyboard. A mode rather than a side `Option` for the reason
    /// `SearchingLog` holds its bar: the popup is visible exactly while
    /// this is the mode, and every global key is swallowed by its arm.
    EditingCommitMessage(Box<RewordForm>),
    /// The interactive rebase / multi-commit cherry-pick popup
    /// (`rebase_ui.rs`) is open and owns the keyboard - same reason as
    /// `EditingCommitMessage`.
    InteractiveRebase(Box<RebaseForm>),
    /// The AI commit-composer review popup (`compose_ui.rs`): owns the
    /// keyboard while open, like the rebase popup.
    ComposeCommits(Box<ComposeForm>),
    /// The split-commit editor (`split_ui/`): owns the keyboard while open,
    /// like the composer review.
    SplitCommit(Box<SplitForm>),
    /// The history rewrite workbench (`rewrite_view/`): owns the keyboard
    /// while open, like the split editor.
    Rewrite(Box<crate::rewrite_view::RewriteView>),
    /// The tab picker: `Ctrl+N Tab` or the palette's
    /// `Repository: Select Tab...`. A popup listing every open repository
    /// tab, filtered by `draft` as you type (the command palette's scorer);
    /// `selected` indexes the *filtered* list, as in `CommandPalette`, since
    /// its length changes on every keystroke. Enter switches to the chosen
    /// tab (`repo_switcher::SwitcherAction::Focus`); Esc closes with nothing
    /// done.
    TabPicker {
        draft: String,
        selected: usize,
    },
    /// The theme picker (`Ctrl+N t`, `App: Change Theme...`): a popup like
    /// the tab picker's, over every theme, previewing the highlighted one;
    /// Enter keeps it (and writes `ui.theme`), Esc restores the previous.
    ThemePicker(Box<crate::theme_picker::ThemePicker>),
    /// The language picker (`Ctrl+N l`, `App: Change Language...`): Enter
    /// applies the highlighted language and writes `ui.language`.
    LanguagePicker(Box<crate::language_picker::LanguagePicker>),
    /// The workspace search (`Ctrl+Alt+F`, `Search: Workspace`): a fuzzy
    /// finder over every repository of the active workspace (else the open
    /// tabs), its popup owns the keyboard and the mouse like the pickers.
    #[cfg_attr(not(feature = "global-search"), allow(dead_code))]
    WorkspaceSearch(Box<crate::workspace_search::WorkspaceSearch>),
    /// A guiding dialog (`guide_dialog.rs`): conflicts must be resolved
    /// first, or an AI input is over its limit. Owns keys and mouse while
    /// open; Esc, the cross and an outside click cancel.
    Guide(Box<crate::guide_dialog::GuideDialog>),
    /// A question that used to be typed in the status line (stash message,
    /// tag message, ...): a small dialog (`prompt_dialog/`) that owns the
    /// keyboard and the mouse while open, like `EditingCommitMessage`.
    Prompt(Box<crate::prompt_dialog::PromptDialog>),
}

/// What the confirm modal carries out if the user says yes.
///
/// Two kinds, because not everything worth confirming is a repository
/// mutation: a branch delete goes to the daemon as an `Action`, but closing
/// a tab edits this window's session file and touches nothing on disk
/// (ADR-0006), so it cannot be an `Action`.
#[derive(Clone)]
pub(crate) enum ConfirmAction {
    /// Submitted to the daemon.
    Destructive(PendingDestructive),
    /// Handled in this process: closes the tab at this index.
    CloseTab(usize),
    /// Handled in this process: closes the tabs for these repositories -
    /// *Close other tabs*, *Close tabs to the right*, *Close all
    /// tabs*. By path, since closing one renumbers the rest.
    CloseTabs(Vec<PathBuf>),
    /// Create `local` tracking the remote branch `upstream` and check it
    /// out. Submitted to the daemon, but it creates rather than deletes,
    /// so it does not get the red button.
    CreateTrackingBranch { local: String, upstream: String },
    /// Open https://git-lfs.com in the browser - what the "Git LFS is
    /// not installed" error dialog offers. Handled in this process.
    OpenLfsInstallPage,
    /// A risky undo or redo the daemon asked the user about
    /// (`NeedsConfirmation`); confirming sends it again with `confirmed`.
    UndoRedo {
        direction: gitpoulpi_core::domain::Direction,
        expect: Option<u64>,
    },
    /// A risky entry of "Undo whole sequence": confirming sends the entries
    /// still to undo again, the first one `confirmed`.
    UndoSequence {
        group: gitpoulpi_core::domain::SequenceGroup,
    },
    /// *Start over* for the pending auto-stash; confirming submits it
    /// with `confirmed`.
    AutoStashStartOver,
    /// Drop the blocked latest undo entry from the journal.
    UndoForget { expect: Option<u64> },
    /// Move a branch back to the reflog position a blocked undo offered.
    ReflogRestore {
        refname: String,
        oid: gitpoulpi_core::domain::Oid,
    },
    /// Write the chosen profile's identity into the repository's local git
    /// config (`user.name`, `user.email`; journaled like any config write).
    WriteProfileIdentity { name: String, email: String },
    /// Delete this workspace (`workspaces.toml` and what it parked in the
    /// session file). Handled in this process; the repositories are never
    /// touched.
    DeleteWorkspace(String),
    /// Move a tag although the target does not descend from its commit
    /// (the palette's *Move Selected Tag to Selected Commit*; the drop menu
    /// carries the same choice in its own row). Submitted forced.
    MoveTag {
        name: String,
        target: gitpoulpi_core::domain::Oid,
        target_label: String,
        remotes: Vec<String>,
    },
}

impl ConfirmAction {
    /// The word on the confirming button.
    pub(crate) fn label(&self) -> String {
        use gitpoulpi_i18n::msg::confirm_button as b;
        let word = match self {
            ConfirmAction::Destructive(
                PendingDestructive::ForcePush { .. }
                | PendingDestructive::GroupForcePush { .. }
                | PendingDestructive::ForcePushTag { .. },
            ) => b::FORCE_PUSH,
            ConfirmAction::Destructive(PendingDestructive::GroupAbort { .. }) => {
                gitpoulpi_i18n::msg::push_group::STRIP_ABORT
            }
            ConfirmAction::Destructive(PendingDestructive::RemoveRemote(_)) => b::REMOVE,
            ConfirmAction::Destructive(_) => b::DELETE,
            ConfirmAction::CloseTab(_) => b::CLOSE_TAB,
            ConfirmAction::CloseTabs(_) => b::CLOSE_TABS,
            ConfirmAction::CreateTrackingBranch { .. } => b::CREATE_BRANCH,
            ConfirmAction::OpenLfsInstallPage => b::OPEN_INSTALL_PAGE,
            ConfirmAction::UndoRedo {
                direction: gitpoulpi_core::domain::Direction::Undo,
                ..
            } => b::UNDO_ANYWAY,
            ConfirmAction::UndoSequence { .. } => b::UNDO_ANYWAY,
            ConfirmAction::UndoRedo { .. } => b::REDO_ANYWAY,
            ConfirmAction::AutoStashStartOver => b::START_OVER,
            ConfirmAction::UndoForget { .. } => gitpoulpi_i18n::msg::undo_journal::FORGET_BUTTON,
            ConfirmAction::ReflogRestore { .. } => {
                gitpoulpi_i18n::msg::undo_safety::CONFIRM_RESTORE
            }
            ConfirmAction::WriteProfileIdentity { .. } => {
                gitpoulpi_i18n::msg::profile_bar::WRITE_BUTTON
            }
            ConfirmAction::DeleteWorkspace(_) => b::DELETE,
            ConfirmAction::MoveTag { .. } => b::MOVE_TAG,
        };
        format!(" {} ", word.get())
    }

    /// Whether that button is the red one. Closing a tab is not
    /// destructive, and painting it red would teach the user that red
    /// means nothing.
    pub(crate) fn is_destructive(&self) -> bool {
        matches!(self, ConfirmAction::Destructive(_))
    }
}

/// Where a branch rename's box is drawn, before the name box takes over.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RenameSpot {
    /// Over the branch's LOCAL row in the Refs sidebar.
    Refs,
    /// In the Ref column of the branch tip's Log row.
    Pill,
    /// The palette: the Refs row if the sidebar draws it, else the pill if
    /// the Log draws it, else the name box.
    Auto,
}

/// What a `InlineName` input's Enter should do with the typed name.
pub(crate) enum RefNameOp {
    NewBranch,
    NewTag,
    /// The commit context menu's `Create branch here...` /
    /// `Create tag here...`. Identical to the two above except for
    /// pointing at a chosen commit instead of at HEAD, which is why they
    /// are variants of the same prompt rather than a second one: the
    /// field, its editing, Esc and Enter all behave as they already do.
    NewBranchAt(Oid),
    NewTagAt(Oid),
    /// `Create annotated tag here...`, step one: the tag's name.
    NewAnnotatedTagAt(Oid),
    /// Rename `name` in place: `spot` says where the box is drawn
    /// (`RenameSpot`).
    RenameBranch {
        name: String,
        spot: RenameSpot,
    },
    /// Type the name (with its extension) of a new empty file, relative to
    /// this folder (empty = the repository root): `Create File` and a
    /// folder's `Create a file in this folder`. See `new_file.rs`.
    CreateFile(std::path::PathBuf),
}

/// A destructive `Action` gated behind `InputMode::Confirm`.
#[derive(Clone)]
pub(crate) enum PendingDestructive {
    DeleteBranch(String),
    DeleteTag(String),
    /// Delete a tag from a remote, keeping the local one - the only
    /// destructive action in this list that changes something *other
    /// people* can see, which is why its prompt says so rather than
    /// warning about local work.
    DeleteRemoteTag {
        remote: String,
        name: String,
    },
    /// Delete a tag on a remote *and* locally. Outward-facing like
    /// `DeleteRemoteTag`; Undo restores the local tag only.
    DeleteTagBoth {
        remote: String,
        name: String,
    },
    /// Delete a branch from a remote, and with `and_local` the local
    /// branch of that name as well. Changes something other people can see.
    DeleteRemoteBranch {
        remote: String,
        name: String,
        and_local: Option<String>,
    },
    StashDrop(gitpoulpi_core::domain::StashId),
    /// The pending auto-stash, by its commit (see `Action::AutoStashDrop`).
    AutoStashDrop(
        gitpoulpi_core::domain::StashId,
        Option<gitpoulpi_core::domain::Oid>,
    ),
    /// Throwing away a file's uncommitted changes - the same
    /// no-undo-possible category as the three above, so it takes the
    /// same route through the confirm modal.
    DiscardFile {
        path: PathBuf,
        source: Oid,
    },
    /// The multi-path sibling of `DiscardFile` above: a
    /// folder's worth of Unstaged rows, or the global "Discard all
    /// changes" button. Same no-undo-possible category, same route.
    DiscardPaths(Vec<PathBuf>),
    /// Remove one file from disk (the file context menu's "Delete file").
    DeleteFile(PathBuf),
    /// Several files removed from disk at once (a selection).
    DeleteFiles(Vec<PathBuf>),
    /// Several files overwritten with their version from a commit.
    RestoreFiles {
        paths: Vec<PathBuf>,
        source: Oid,
    },
    /// One hunk of a file's unstaged changes, or some of its changed
    /// lines, reverted in the working tree from the Diff view. Same no-undo-possible category as
    /// `DiscardFile`.
    DiscardHunk {
        path: PathBuf,
        hunk: HunkId,
    },
    DiscardLines {
        path: PathBuf,
        selector: LineSelector,
    },
    /// `reset --hard`, and only that. Soft and mixed resets go straight
    /// through: they move the branch pointer but leave every byte of work in
    /// the index or working tree, and the move is in the reflog. Hard throws
    /// the working tree away with nothing holding a copy, the same "no undo
    /// possible" category as `DiscardFile` above, so it takes the same route.
    ResetHard {
        target: Oid,
    },
    /// Revert rewrites the working tree from a commit's inverse, so any
    /// uncommitted edit it touches is gone. It is confirmed for that, not
    /// for the new commit it makes (which is an ordinary, revertible
    /// commit).
    Revert {
        commit: Oid,
        /// The parent that stays the mainline, for a merge commit.
        mainline: Option<u32>,
    },
    /// Abort a rebase in progress, from `rebase_banner`'s Abort button --
    /// throws away everything replayed so far, restoring the branch and
    /// HEAD to where they were before it started.
    AbortRebase,
    /// Abort the merge, cherry-pick or revert in progress, from the
    /// operation banner: the merge / pick is thrown away and HEAD, the
    /// index and the working tree go back to where they were.
    AbortOperation,
    /// Abort the `git am` in progress, from its banner.
    AbortAm,
    /// Delete the stale `.git/index.lock`, from its banner.
    RemoveIndexLock,
    /// Skip the cherry-pick or revert in progress, from the operation
    /// banner: nothing is recorded for it and its resolutions are dropped.
    SkipOperation,
    /// `git remote remove`: its remote-tracking branches go with it.
    RemoveRemote(String),
    /// The diverged-push dialog's force push - always
    /// `--force-with-lease`, and always confirmed first.
    ForcePush {
        remote: String,
        branch: String,
        /// Fetch first, then lease against what was fetched (the
        /// diverged dialog, see `RemoteUiAction::ForcePush`).
        after_fetch: bool,
        /// The ticket of the push this answers (a sequence step): the
        /// confirmed force push reports under it.
        ticket: Option<crate::daemon_link::Ticket>,
    },
    /// A grouped push with at least one force push (with lease): confirmed
    /// naming the remotes and the commits they lose, then run as written.
    GroupForcePush {
        request: Box<gitpoulpi_core::domain::RemoteGroupRunRequest>,
    },
    /// Force-push a tag that differs to `remote` (leased on the last fetch):
    /// the remote's tag is replaced for everyone.
    ForcePushTag {
        remote: String,
        name: String,
    },
    /// Abort the merge or rebase of a paused grouped push (the strip's
    /// "Abort group"); pushes already done stay.
    GroupAbort {
        run_id: String,
    },
    /// Break somebody else's Git LFS lock on a file. It changes what
    /// *other people* see (their lock disappears), so it is confirmed.
    LfsForceUnlock {
        remote: String,
        path: PathBuf,
    },
}

impl PendingDestructive {
    /// The ticket the confirmed action reports under, when it has one.
    pub(crate) fn ticket(&self) -> Option<crate::daemon_link::Ticket> {
        match self {
            PendingDestructive::ForcePush { ticket, .. } => *ticket,
            _ => None,
        }
    }

    pub(crate) fn into_action(self) -> Action {
        match self {
            PendingDestructive::DeleteBranch(name) => Action::BranchDelete(name),
            PendingDestructive::DeleteTag(name) => Action::TagDelete(name),
            PendingDestructive::DeleteRemoteTag { remote, name } => {
                Action::TagDeleteRemote { remote, name }
            }
            PendingDestructive::DeleteTagBoth { remote, name } => {
                Action::TagDeleteBoth { remote, name }
            }
            PendingDestructive::DeleteRemoteBranch {
                remote,
                name,
                and_local,
            } => Action::BranchDeleteRemote {
                remote,
                name,
                and_local,
            },
            PendingDestructive::StashDrop(id) => Action::StashDrop(id),
            PendingDestructive::AutoStashDrop(id, commit) => Action::AutoStashDrop(id, commit),
            PendingDestructive::DiscardFile { path, source } => {
                Action::DiscardFile { path, source }
            }
            PendingDestructive::DiscardPaths(paths) => Action::DiscardPaths(paths),
            PendingDestructive::DeleteFile(path) => Action::DeleteFile(path),
            PendingDestructive::DeleteFiles(paths) => Action::DeleteFiles(paths),
            PendingDestructive::RestoreFiles { paths, source } => {
                Action::RestoreFiles { paths, source }
            }
            PendingDestructive::DiscardHunk { path, hunk } => Action::DiscardHunk { path, hunk },
            PendingDestructive::DiscardLines { path, selector } => {
                Action::DiscardLines { path, selector }
            }
            PendingDestructive::ResetHard { target } => Action::Reset {
                target,
                mode: gitpoulpi_core::domain::ResetMode::Hard,
            },
            PendingDestructive::Revert { commit, mainline } => Action::Revert { commit, mainline },
            PendingDestructive::AbortRebase => Action::RebaseAbort,
            PendingDestructive::AbortOperation => Action::OperationAbort,
            PendingDestructive::AbortAm => Action::AmAbort,
            PendingDestructive::RemoveIndexLock => Action::IndexLockRemove,
            PendingDestructive::SkipOperation => Action::OperationSkip,
            PendingDestructive::RemoveRemote(name) => Action::RemoteRemove(name),
            PendingDestructive::LfsForceUnlock { remote, path } => Action::LfsUnlock {
                remote,
                path,
                force: true,
            },
            PendingDestructive::GroupForcePush { request } => Action::RemoteGroupRun(request),
            PendingDestructive::ForcePushTag { remote, name } => {
                Action::TagForcePush { remote, name }
            }
            PendingDestructive::GroupAbort { run_id } => Action::RemoteGroupAbort {
                expect_run_id: Some(run_id),
            },
            PendingDestructive::ForcePush {
                remote,
                branch,
                after_fetch: true,
                ..
            } => Action::FetchThenForcePush { remote, branch },
            PendingDestructive::ForcePush {
                remote,
                branch,
                after_fetch: false,
                ..
            } => Action::Push {
                remote,
                opts: gitpoulpi_core::domain::PushOptions {
                    force_with_lease: true,
                    set_upstream: false,
                    branch: Some(branch),
                    ..Default::default()
                },
            },
        }
    }
}
