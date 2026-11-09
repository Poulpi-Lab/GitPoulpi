use super::*;

// ---------------------------------------------------------------------
// State
// ---------------------------------------------------------------------

/// Everything remote-related one repository window holds, in one place
/// so the event loop carries one local rather than six.
pub(crate) struct RemoteUi {
    /// The remotes as the last refresh reported them (URLs included, for
    /// Edit and Copy link).
    pub(crate) remotes: Vec<Remote>,
    /// The remote chosen with "Set as default remote", persisted in
    /// `repo-state.toml`.
    pub(crate) chosen_default: Option<String>,
    /// The repository-local `core.sshCommand`, as the last refresh
    /// reported it.
    pub(crate) ssh_pin: Option<String>,
    /// Something (the push menu's identity row, the tab menu) asked for the
    /// SSH & credentials page; the window opens it at its next notice pass.
    pub(crate) want_repo_auth: bool,
    /// The one open remote-related menu: a remote's menu, a toolbar
    /// dropdown, Set upstream, a remote chooser, or the diverged dialog.
    pub(crate) menu: Option<ContextMenu<RemoteUiAction>>,
    pub(crate) form: Option<RemoteForm>,
    /// The checkout/merge/rebase/pull the daemon refused because local
    /// changes are in the way (`general.auto_stash = ask`), while its
    /// dialog is open: what "Stash changes and continue" resends.
    pub(super) pending_autostash: Option<Action>,
    repo_root: PathBuf,
    push_anchor: (u16, u16),
    pull_anchor: (u16, u16),
    lfs_anchor: (u16, u16),
    flow_anchor: (u16, u16),
}

impl RemoteUi {
    pub(crate) fn load(repo_root: &Path) -> Self {
        Self {
            remotes: Vec::new(),
            chosen_default: load_chosen_default_remote(repo_root),
            ssh_pin: None,
            want_repo_auth: false,
            menu: None,
            form: None,
            pending_autostash: None,
            repo_root: repo_root.to_path_buf(),
            push_anchor: (0, 1),
            pull_anchor: (0, 1),
            lfs_anchor: (0, 1),
            flow_anchor: (0, 1),
        }
    }

    pub(crate) fn names(&self) -> Vec<String> {
        self.remotes.iter().map(|r| r.name.clone()).collect()
    }

    pub(crate) fn default_remote(&self, refs: &[Ref]) -> Option<String> {
        default_remote(self.chosen_default.as_deref(), refs, &self.names())
    }

    /// Whether a menu or the popup is open - they own the keyboard.
    pub(crate) fn is_open(&self) -> bool {
        self.menu.is_some() || self.form.is_some()
    }

    pub(super) fn set_chosen_default(&mut self, chosen: Option<String>) {
        save_chosen_default_remote(&self.repo_root, chosen.as_deref());
        self.chosen_default = chosen;
    }

    /// The tooltip of one row of the open remote menu, if it has one.
    pub(crate) fn row_tooltip(&self, row: usize) -> Option<String> {
        self.menu.as_ref()?.row_action(row)?.tooltip()
    }

    pub(crate) fn open_remote_menu(
        &mut self,
        name: &str,
        hidden: bool,
        refs: &[Ref],
        col: u16,
        row: u16,
    ) {
        let is_default = self.default_remote(refs).as_deref() == Some(name);
        self.menu = Some(ContextMenu::new(
            msg::menu_title(name),
            remote_menu_entries(name, is_default, hidden),
            col,
            row,
        ));
    }

    pub(crate) fn open_pull_menu(&mut self, col: u16, row: u16) {
        self.menu = Some(ContextMenu::new(
            msg::PULL_TITLE.get(),
            pull_mode_entries(),
            col,
            row,
        ));
    }

    /// The toolbar's `LFS` dropdown.
    pub(crate) fn open_lfs_menu(&mut self, col: u16, row: u16) {
        self.menu = Some(ContextMenu::new(
            gitpoulpi_i18n::msg::lfs_ui::MENU_TITLE.get(),
            lfs_menu_entries(),
            col,
            row,
        ));
    }

    /// The toolbar's `Flow` dropdown.
    pub(crate) fn open_flow_menu(&mut self, col: u16, row: u16) {
        self.menu = Some(ContextMenu::new(
            gitpoulpi_i18n::msg::flow::TOOLBAR.get(),
            flow_menu_entries(),
            col,
            row,
        ));
    }

    /// The push-to menu, or a status line saying why there is none.
    pub(crate) fn open_push_menu(&mut self, refs: &[Ref], col: u16, row: u16) -> Option<String> {
        let Some(head) = head_branch(refs) else {
            return Some(msg::DETACHED_PUSH.get().into());
        };
        let names = self.names();
        if names.is_empty() {
            return Some(msg::NO_REMOTE_ADD.get().into());
        }
        let default = self.default_remote(refs);
        let mut entries = push_to_entries(&names, default.as_deref());
        // Who the push signs in as, when the repository's profile is not
        // all default: a row that opens the page that chose it.
        if let Some(line) = crate::activity_log::identity::line_for(&self.repo_root) {
            entries.push(Entry::Separator);
            entries.push(Entry::action(line, RemoteUiAction::OpenRepoAuth));
        }
        self.menu = Some(ContextMenu::new(
            msg::push_to_title(&head.name),
            entries,
            col,
            row,
        ));
        None
    }

    pub(crate) fn open_upstream_menu(
        &mut self,
        branch: &HeadBranch,
        refs: &[Ref],
        col: u16,
        row: u16,
    ) {
        self.menu = Some(ContextMenu::new(
            msg::upstream_title(&branch.name),
            upstream_entries(
                &branch.name,
                refs,
                &self.names(),
                branch.upstream.as_deref(),
            ),
            col,
            row,
        ));
    }

    /// A menu of every remote, each row running `action(name)`.
    pub(crate) fn open_chooser(
        &mut self,
        caption: &str,
        refs: &[Ref],
        col: u16,
        row: u16,
        action: impl Fn(String) -> RemoteUiAction,
    ) -> Option<String> {
        let names = self.names();
        if names.is_empty() {
            return Some(msg::NO_REMOTE.get().into());
        }
        let default = self.default_remote(refs);
        self.menu = Some(ContextMenu::new(
            caption,
            remote_chooser_entries(&names, default.as_deref(), action),
            col,
            row,
        ));
        None
    }

    /// The diverged-push dialog, or just a status line for a stale lease
    /// (the answer to that one is "fetch, look, decide again").
    pub(crate) fn on_push_rejected(
        &mut self,
        rejection: &PushRejection,
        col: u16,
        row: u16,
    ) -> String {
        match rejection.reason {
            PushRejectReason::NonFastForward => {
                self.menu = Some(ContextMenu::new(
                    msg::diverged_title(&rejection.remote, &rejection.branch),
                    diverged_entries(rejection),
                    col,
                    row,
                ));
                msg::diverged_message(&rejection.remote, &rejection.branch)
            }
            PushRejectReason::StaleLease => msg::stale_lease(&rejection.remote, &rejection.branch),
        }
    }
}

impl RemoteUi {
    /// The Pull button, `Ctrl+G p` and the palette's `Git: Pull`: run
    /// whatever the button's dropdown says. Returns the status line.
    pub(crate) fn run_pull_button(&self, refs: &[Ref], link: &DaemonLink) -> String {
        let (mode, _) = sync_settings();
        self.run_pull_mode(mode, refs, link)
    }

    /// One specific pull mode, for the palette's per-mode entries.
    pub(crate) fn run_pull_mode(
        &self,
        mode: PullButtonMode,
        refs: &[Ref],
        link: &DaemonLink,
    ) -> String {
        let default = self.default_remote(refs);
        match pull_action(mode, head_branch(refs).as_ref(), default.as_deref()) {
            Ok((action, said)) => {
                link.submit_action(action);
                said
            }
            Err(why) => why,
        }
    }

    /// The Push button, `Ctrl+G P` and the palette's `Git: Push`: push to
    /// the default remote, or open the push-to menu when the setting
    /// says to ask (or there is no default to push to).
    pub(crate) fn run_push_button(&mut self, refs: &[Ref], link: &DaemonLink) -> Option<String> {
        let (_, behavior) = sync_settings();
        let default = self.default_remote(refs);
        match push_plan(
            behavior,
            head_branch(refs).as_ref(),
            default.as_deref(),
            !self.remotes.is_empty(),
        ) {
            PushPlan::Direct(action, said) => {
                link.submit_action(action);
                Some(said)
            }
            PushPlan::Ask => {
                let (col, row) = self.push_anchor;
                self.open_push_menu(refs, col, row)
            }
            PushPlan::Refuse(why) => Some(why),
        }
    }

    /// Where the toolbar's Push / Pull buttons were last drawn, so a menu
    /// opened from the keyboard or the palette hangs off the same button
    /// a click would have opened it from.
    pub(crate) fn set_toolbar_anchors(
        &mut self,
        push: (u16, u16),
        pull: (u16, u16),
        lfs: (u16, u16),
        flow: (u16, u16),
    ) {
        self.push_anchor = push;
        self.pull_anchor = pull;
        self.lfs_anchor = lfs;
        self.flow_anchor = flow;
    }

    pub(crate) fn flow_anchor(&self) -> (u16, u16) {
        self.flow_anchor
    }

    pub(crate) fn lfs_anchor(&self) -> (u16, u16) {
        self.lfs_anchor
    }

    pub(crate) fn pull_anchor(&self) -> (u16, u16) {
        self.pull_anchor
    }

    pub(crate) fn push_anchor(&self) -> (u16, u16) {
        self.push_anchor
    }

    /// The daemon refused `needed.operation` because local changes are in
    /// the way and `general.auto_stash` is `ask`: open the dialog (stash
    /// and continue, or cancel). Returns the status line.
    pub(crate) fn on_autostash_needed(
        &mut self,
        needed: AutoStashNeeded,
        col: u16,
        row: u16,
    ) -> String {
        let operation = needed.operation;
        // A double-click can send the same refused request twice: one
        // dialog is enough.
        if self.menu.is_some() && self.pending_autostash.as_ref() == Some(&needed.action) {
            return msg::autostash_message(&operation);
        }
        self.pending_autostash = Some(needed.action);
        self.menu = Some(ContextMenu::new(
            msg::autostash_title(&operation),
            autostash_entries(),
            col,
            row,
        ));
        msg::autostash_message(&operation)
    }
}

/// The auto-stash dialog's answers. The stash is popped again once the
/// operation is done; with `general.auto_stash = always` there is no dialog,
/// which is what its "always" answer sets.
pub(crate) fn autostash_entries() -> Vec<Entry<RemoteUiAction>> {
    vec![
        Entry::action(
            glyph_label("\u{21E9}", msg::AUTOSTASH_CONTINUE.get()),
            RemoteUiAction::AutoStashContinue,
        ),
        Entry::action(
            glyph_label("\u{21BB}", msg::AUTOSTASH_ALWAYS.get()),
            RemoteUiAction::AutoStashAlways,
        ),
        Entry::Separator,
        Entry::action(
            gitpoulpi_i18n::msg::dialog::CANCEL.get(),
            RemoteUiAction::Cancel,
        ),
    ]
}

/// The remote a REMOTE-section row belongs to: a remote's own row names
/// it, a folder or a branch under it names it by prefix.
pub(crate) fn remote_of_row(row: &RefTreeRow, remotes: &[String]) -> Option<String> {
    let name = match row {
        RefTreeRow::Folder { path, .. } => path.as_str(),
        RefTreeRow::Leaf { full_name, .. } => full_name.as_str(),
    };
    if remotes.iter().any(|r| r == name) {
        return Some(name.to_string());
    }
    upstream_remote(name, remotes)
}

/// The remote selected in the REMOTE section, when that section is the
/// active one - what the palette's remote commands act on before they
/// fall back to asking.
#[allow(clippy::too_many_arguments)]
pub(crate) fn selected_sidebar_remote(
    refs_result: &gitpoulpi_core::Result<Vec<Ref>>,
    refs_filter: &str,
    section: SidebarSection,
    selected: Option<usize>,
    hierarchical: bool,
    collapsed_folders: &CollapsedFolders,
    remotes: &[String],
) -> Option<String> {
    if section != SidebarSection::Remote {
        return None;
    }
    let refs = refs_result.as_ref().ok()?;
    let groups = group_refs(refs, refs_filter);
    let rows = ref_display_rows(
        groups.names(RefRowKind::Remote),
        RefRowKind::Remote,
        hierarchical,
        collapsed_folders,
        remotes,
    );
    remote_of_row(rows.get(selected?)?, remotes)
}
