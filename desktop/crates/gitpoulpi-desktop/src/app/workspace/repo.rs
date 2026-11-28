//! What the window does about worktrees, submodules and remotes: running a [`RepoOp`] on a target (a Refs
//! row, or the selection), the Create worktree form, the removal questions, the remote forms and the
//! push, pull and fetch rows of a branch's menu. The LFS commands are in `repo_lfs.rs`, the file menu in
//! `file_menu.rs`.
//!
//! The rules live in `daemon::repo_ops` (pure, tested); this file only asks and submits. One submitted
//! action is one undo entry (an edited remote that changes its name and its URL is several, each
//! written on its own).

use std::rc::Rc;

use gitpoulpi_core::domain::{Remote, Worktree};
use gitpoulpi_i18n::msg;
use gitpoulpi_link::{Action, StepOutcome, Ticket};
use gpui_kit::{ClipboardItem, Context, Pixels, Point, Window};

use super::Workspace;
use crate::app::actions::with_workspace;
use crate::commands::file_ops;
use crate::commands::repo_ops::RepoOp;
use crate::daemon::repo_ops::{self as rules, WorktreeSeed};
use crate::dialogs::{self, Choice, Spec};
use crate::form_dialog::{self, Field, FormSpec};
use crate::ref_menu::{Item, RefMenu};
use crate::ref_target::RefTarget;
use crate::repo_menu;
use crate::strings;

/// What the window remembers about LFS between two commands (see `repo_lfs.rs`).
#[derive(Default)]
pub(in crate::app) struct RepoState {
    pub lfs: Option<super::repo_lfs::LfsView>,
    /// The working-changes lists show only the LFS files.
    pub lfs_only: bool,
}

impl Workspace {
    /// Runs a worktree, submodule, remote, LFS or file command. `target` is what a menu row or a click
    /// names; without one the Refs sidebar's selection is used.
    pub fn run_repo_op(
        &mut self,
        op: RepoOp,
        target: Option<RefTarget>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let target = target.or_else(|| self.refs.read(cx).selected_target());
        match op {
            RepoOp::WorktreeMenu => {
                let at = window.mouse_position();
                self.show_items(
                    repo_menu::worktree_plus(),
                    RefTarget::Worktree(String::new()),
                    at,
                    window,
                    cx,
                );
            }
            RepoOp::WorktreeFromHead => self.worktree_from_head(false, window, cx),
            RepoOp::WorktreeChoose => self.worktree_from_head(true, window, cx),
            RepoOp::WorktreeFromRef => self.worktree_from_ref(target, window, cx),
            RepoOp::WorktreeAtCommit => match self.log.read(cx).selected_commit() {
                Some(at) => self.worktree_form(rules::seed_for_commit(at), false, window, cx),
                None => self.say(msg::worktree_manage::SELECT_A_COMMIT.get(), cx),
            },
            RepoOp::WorktreeOpen => match self.worktree_of(&target, cx) {
                Some(w) if w.is_main => {
                    self.say(msg::worktree_manage::MAIN_CANNOT_BE_REMOVED.get(), cx)
                }
                Some(w) => self.open_repo(&w.path, cx),
                None => self.no_worktree(cx),
            },
            RepoOp::WorktreeOpenInMenu => match self.worktree_of(&target, cx) {
                Some(w) => {
                    let entries = file_ops::open_in_entries(&file_ops::External::current(cx));
                    let at = self.menu_origin(cx);
                    let key = RefTarget::Worktree(w.path.to_string_lossy().into_owned());
                    self.show_items(repo_menu::open_in_rows(&entries), key, at, window, cx);
                }
                None => self.no_worktree(cx),
            },
            RepoOp::WorktreeOpenIn(i) => match self.worktree_of(&target, cx) {
                Some(w) => {
                    let entry = file_ops::open_in_entries(&file_ops::External::current(cx))
                        .into_iter()
                        .nth(i as usize);
                    let branch = w.branch.clone().unwrap_or_default();
                    let said = match entry {
                        Some(e) => file_ops::run_open_in(&e, &w.path, &branch)
                            .map(|label| msg::open_in_run::opening(&label)),
                        None => Err(msg::open_in_run::no_entry(i as usize + 1)),
                    };
                    match said {
                        Ok(text) => self.say(text, cx),
                        Err(why) => crate::commands::status::set_error(cx, why),
                    }
                }
                None => self.no_worktree(cx),
            },
            RepoOp::WorktreeRemove => match self.worktree_of(&target, cx) {
                Some(w) => self.start_worktree_remove(w, window, cx),
                None => self.no_worktree(cx),
            },
            RepoOp::SubmoduleUpdate | RepoOp::SubmoduleInit | RepoOp::SubmoduleOpen => {
                self.submodule_op(op, target, cx)
            }
            RepoOp::SubmoduleUpdateAll => {
                self.backend.submit(rules::submodule_update_action(None));
                self.say(msg::input::UPDATING_SUBMODULES.get(), cx);
            }
            RepoOp::RemoteAdd => self.start_remote_add(window, cx),
            RepoOp::RemoteEdit => self.start_remote_edit(target, window, cx),
            RepoOp::RemoteRemove => self.start_remote_remove(target, window, cx),
            RepoOp::RemoteFetch => match self.remote_of(&target, cx) {
                Some(name) => {
                    self.backend.submit(Action::Fetch(name.clone()));
                    self.say(msg::remotes::fetching_one(name), cx);
                }
                None => self.say(msg::remotes::SELECT_REMOTE_FIRST.get(), cx),
            },
            RepoOp::RemoteCopyName => match self.remote_of(&target, cx) {
                Some(name) => {
                    cx.write_to_clipboard(ClipboardItem::new_string(name.clone()));
                    self.say(
                        msg::input::copied(msg::remotes::WHAT_REMOTE_NAME.get(), name),
                        cx,
                    );
                }
                None => self.say(msg::remotes::SELECT_REMOTE_FIRST.get(), cx),
            },
            RepoOp::RemoteActions => match self.remote_of(&target, cx) {
                Some(name) => {
                    let at = self.menu_origin(cx);
                    self.show_items(
                        repo_menu::remote_items(&name),
                        RefTarget::RemoteNode(name),
                        at,
                        window,
                        cx,
                    );
                }
                None => self.say(msg::remotes::SELECT_REMOTE_FIRST.get(), cx),
            },
            RepoOp::BranchRemotes => self.run_ref_op(
                crate::commands::refs_ops::RefOp::SetUpstream,
                target,
                window,
                cx,
            ),
            RepoOp::BranchPush | RepoOp::BranchPull | RepoOp::BranchFetch => {
                self.branch_transfer(op, target, cx)
            }
            RepoOp::File(op) => self.run_file_op(op, target, window, cx),
            other => self.run_lfs_op(other, target, window, cx),
        }
    }

    /// Opens a menu of `items` at `at`; `target` is what its rows run on.
    pub(in crate::app) fn show_items(
        &mut self,
        items: Vec<Item>,
        target: RefTarget,
        at: Point<Pixels>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.show_menu(
            move |cx| RefMenu::new(items, at, cx),
            target,
            None,
            window,
            cx,
        );
    }

    /// The rows of the menu a Worktree, Submodule, remote or file target opens, `None` for the refs.
    pub(in crate::app) fn repo_target_items(
        &self,
        target: &RefTarget,
        cx: &gpui_kit::App,
    ) -> Option<Vec<Item>> {
        match target {
            RefTarget::Worktree(_) => self.worktree_of(&Some(target.clone()), cx).map(|w| {
                repo_menu::worktree_items(
                    &w,
                    &file_ops::open_in_entries(&file_ops::External::current(cx)),
                )
            }),
            RefTarget::Submodule(p) => {
                let refs = self.refs.read(cx);
                rules::find_submodule(refs.submodules(), p).map(repo_menu::submodule_items)
            }
            RefTarget::RemoteNode(name) => Some(repo_menu::remote_items(name)),
            _ => None,
        }
    }

    fn no_worktree(&mut self, cx: &mut Context<Self>) {
        self.say(msg::worktree_manage::SELECT_A_WORKTREE.get(), cx);
    }

    fn worktree_of(&self, target: &Option<RefTarget>, cx: &gpui_kit::App) -> Option<Worktree> {
        let Some(RefTarget::Worktree(path)) = target else {
            return None;
        };
        self.refs
            .read(cx)
            .worktrees()
            .iter()
            .find(|w| w.path.to_string_lossy() == path.as_str())
            .cloned()
    }

    /// Runs `then` once the action under `ticket` succeeded.
    pub(in crate::app) fn when_done(
        &mut self,
        ticket: Ticket,
        cx: &mut Context<Self>,
        then: impl FnOnce(&mut Self, &mut Context<Self>) + 'static,
    ) {
        let notifier = self.backend.notifier().clone();
        let mut seen = notifier.epoch();
        let mut then = Some(then);
        cx.spawn(async move |this, cx| loop {
            for outcome in gitpoulpi_link::take_for(ticket) {
                match outcome {
                    StepOutcome::Ok(_) => {
                        if let Some(f) = then.take() {
                            let _ = this.update(cx, |this, cx| f(this, cx));
                        }
                        return;
                    }
                    StepOutcome::Failed { .. } | StepOutcome::Canceled => return,
                    StepOutcome::NeedsUser(_) => {}
                }
            }
            notifier.changed(&mut seen).await;
        })
        .detach();
    }

    // -- worktrees -------------------------------------------------------------------------------------

    fn worktree_from_head(&mut self, choose: bool, window: &mut Window, cx: &mut Context<Self>) {
        self.with_head(window, cx, move |this, head, window, cx| {
            let refs = this.snapshot(cx).refs;
            match rules::seed_from_head(&refs, Some(head)) {
                Some(seed) => this.worktree_form(seed, choose, window, cx),
                None => this.say(msg::worktree_manage::NO_HEAD.get(), cx),
            }
        });
    }

    fn worktree_from_ref(
        &mut self,
        target: Option<RefTarget>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let snap = self.snapshot(cx);
        let worktrees = self.refs.read(cx).worktrees().to_vec();
        let kind = match &target {
            Some(RefTarget::Local(n)) => rules::SeedKind::Local(n),
            Some(RefTarget::Remote(n)) => rules::SeedKind::Remote(n),
            Some(RefTarget::Tag(n)) => rules::SeedKind::Tag(n),
            _ => return self.say(msg::input::SELECT_REF_REFS.get(), cx),
        };
        match rules::seed_for_ref(kind, &snap.refs, &worktrees) {
            Some(seed) => self.worktree_form(seed, false, window, cx),
            None => self.say(msg::input::SELECT_REF_REFS.get(), cx),
        }
    }

    /// The Create worktree form: the folder, the branch (a new name, an existing branch, or empty) and
    /// whether to open the worktree afterwards. `choose` lists the branches that are free.
    fn worktree_form(
        &mut self,
        seed: WorktreeSeed,
        choose: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let root = self.backend.repo_root();
        let refs = self.snapshot(cx).refs;
        let worktrees = self.refs.read(cx).worktrees().to_vec();
        let name = rules::generated_worktree_name();
        let branch = match (&seed.start, seed.branch_busy) {
            _ if choose => String::new(),
            (gitpoulpi_core::domain::WorktreeStart::Branch(b), false) => b.clone(),
            (gitpoulpi_core::domain::WorktreeStart::Commit(_), _)
                if !seed.new_branch.is_empty() =>
            {
                seed.new_branch.clone()
            }
            _ => name.clone(),
        };
        let folder =
            rules::default_worktree_path(&root, if branch.is_empty() { &name } else { &branch });
        let mut intro = vec![msg::worktree::title(&seed.label)];
        if choose {
            let free: Vec<String> = refs
                .iter()
                .filter_map(|r| match &r.kind {
                    gitpoulpi_core::domain::RefKind::LocalBranch { name }
                        if !worktrees.iter().any(|w| w.branch.as_deref() == Some(name)) =>
                    {
                        Some(name.clone())
                    }
                    _ => None,
                })
                .collect();
            intro.push(free.join(", "));
        }
        let spec = FormSpec {
            title: msg::worktree::title(&seed.label),
            intro: if choose {
                intro[1..].to_vec()
            } else {
                Vec::new()
            },
            fields: vec![
                Field::text(
                    strings::FOLDER_LABEL,
                    msg::worktree::FOLDER_HINT.get(),
                    &folder.to_string_lossy(),
                ),
                Field::text(
                    strings::WORKTREE_BRANCH_LABEL,
                    msg::worktree::NEED_BRANCH.get(),
                    &branch,
                ),
                Field::check(strings::OPEN_WORKTREE_AFTER, true),
            ],
            submit: msg::dialog::CREATE.get().to_string(),
            cancel: msg::dialog::CANCEL.get().to_string(),
        };
        let check = {
            let (seed, refs, worktrees) = (seed.clone(), refs.clone(), worktrees.clone());
            move |v: &form_dialog::Values| {
                rules::plan_worktree(&seed, v.text(0), v.text(1), &refs, &worktrees).map(|_| ())
            }
        };
        form_dialog::open(window, cx, spec, check, move |v, _, cx| {
            let planned = rules::plan_worktree(&seed, v.text(0), v.text(1), &refs, &worktrees);
            let open = v.check(2);
            if let Ok((path, opts)) = planned {
                with_workspace(cx, move |ws, _, cx| ws.add_worktree(path, opts, open, cx));
            }
        });
    }

    fn add_worktree(
        &mut self,
        path: std::path::PathBuf,
        opts: gitpoulpi_core::domain::WorktreeAddOptions,
        open: bool,
        cx: &mut Context<Self>,
    ) {
        let said = msg::repo_dialogs::creating_worktree(path.display());
        let ticket = self
            .backend
            .submit_tracked(rules::worktree_add_action(path.clone(), opts));
        self.say(said, cx);
        if open {
            self.when_done(ticket, cx, move |ws, cx| ws.open_repo(&path, cx));
        }
    }

    /// Removing a worktree: what its folder holds decides the question (nothing, or a stash and a discard
    /// are offered). The folder is deleted for good; only its branch survives.
    fn start_worktree_remove(&mut self, w: Worktree, window: &mut Window, cx: &mut Context<Self>) {
        use msg::worktree_manage as m;
        if w.is_main {
            return self.say(m::MAIN_CANNOT_BE_REMOVED.get(), cx);
        }
        self.say(m::LOADING.get(), cx);
        let changes = self.backend.worktree_changes(&w.path);
        cx.spawn_in(window, async move |this, cx| {
            let changes = changes.await;
            let _ = this.update_in(cx, |this, window, cx| {
                crate::commands::status::clear(cx);
                match changes {
                    Ok(list) => this.ask_worktree_remove(w, list.len(), window, cx),
                    Err(why) => crate::commands::status::set_error(cx, why),
                }
            });
        })
        .detach();
    }

    fn ask_worktree_remove(
        &mut self,
        w: Worktree,
        changes: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        use msg::worktree_manage as m;
        let path = w.path.clone();
        let shown = path.display().to_string();
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| shown.clone());
        let (lines, choices) = match rules::remove_ask(changes) {
            rules::RemoveAsk::Clean => (
                vec![match &w.branch {
                    Some(b) => m::clean_text(&shown, b),
                    None => m::clean_text_detached(&shown),
                }],
                vec![
                    Choice::destructive(m::REMOVE.get()),
                    Choice::new(m::CANCEL.get()),
                ],
            ),
            rules::RemoveAsk::Dirty(n) => (
                vec![
                    m::dirty_count(n as u64, &shown),
                    m::force_text(n as u64, &shown),
                ],
                vec![
                    Choice::new(m::STASH_REMOVE.get()),
                    Choice::destructive(m::FORCE_YES.get()),
                    Choice::new(m::CANCEL.get()),
                ],
            ),
        };
        let dirty = changes > 0;
        let spec = Spec {
            title: m::remove_title(&name),
            lines,
            choices,
            hint: msg::diff_actions::CONFIRM_HINT.get().to_string(),
            // A destructive answer is never the default: a stray Enter answers nothing.
            highlight_first: false,
            yes_no_keys: false,
        };
        dialogs::open(
            window,
            cx,
            spec,
            move |ix, cx| {
                let stash = dirty && ix == 0;
                let force = dirty && ix == 1;
                let go = if dirty { ix <= 1 } else { ix == 0 };
                if !go {
                    return;
                }
                with_workspace(cx, move |ws, _, cx| {
                    let mut opts = gitpoulpi_core::domain::WorktreeRemoveOptions {
                        force,
                        ..Default::default()
                    };
                    if stash {
                        opts.stash = Some(m::stash_default(&name));
                    }
                    let action = Action::WorktreeRemove {
                        path: path.clone(),
                        opts: Box::new(opts),
                        delete_branch: None,
                    };
                    ws.backend.submit(action);
                    ws.say(m::removing(path.display()), cx);
                });
            },
            || {},
        );
    }

    // -- submodules ------------------------------------------------------------------------------------

    fn submodule_op(&mut self, op: RepoOp, target: Option<RefTarget>, cx: &mut Context<Self>) {
        let Some(RefTarget::Submodule(path)) = target else {
            return self.say(strings::SELECT_SUBMODULE, cx);
        };
        let Some(sub) = rules::find_submodule(self.refs.read(cx).submodules(), &path).cloned()
        else {
            return self.say(msg::submodules_ui::no_longer(&path), cx);
        };
        match op {
            RepoOp::SubmoduleUpdate => {
                self.backend
                    .submit(rules::submodule_update_action(Some(&path)));
                self.say(msg::input::submodule_updating(&path), cx);
            }
            RepoOp::SubmoduleInit => {
                self.backend.submit(rules::submodule_init_action(&path));
                self.say(msg::input::submodule_registering(&path), cx);
            }
            _ => match rules::submodule_folder(&self.backend.repo_root(), &sub) {
                Ok(folder) => self.open_repo(&folder, cx),
                Err(why) => self.say(why, cx),
            },
        }
    }

    // -- remotes ---------------------------------------------------------------------------------------

    /// The remote a target names: a remote node, a remote branch's remote, else the only remote.
    fn remote_of(&self, target: &Option<RefTarget>, cx: &gpui_kit::App) -> Option<String> {
        let names = self.snapshot(cx).remote_names();
        match target {
            Some(RefTarget::RemoteNode(n)) => Some(n.clone()),
            Some(RefTarget::Remote(full)) => names
                .iter()
                .filter(|n| {
                    full.strip_prefix(n.as_str())
                        .is_some_and(|r| r.starts_with('/'))
                })
                .max_by_key(|n| n.len())
                .cloned(),
            _ if names.len() == 1 => names.into_iter().next(),
            _ => None,
        }
    }

    /// Reads the remotes, then runs `then` with them.
    fn with_remotes(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
        then: impl FnOnce(&mut Self, Vec<Remote>, &mut Window, &mut Context<Self>) + 'static,
    ) {
        let remotes = self.backend.remotes();
        cx.spawn_in(window, async move |this, cx| {
            let list = remotes.await;
            let _ = this.update_in(cx, |this, window, cx| match list {
                Ok(list) => then(this, list, window, cx),
                Err(why) => crate::commands::status::set_error(cx, why),
            });
        })
        .detach();
    }

    fn start_remote_add(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        use msg::remotes as m;
        self.with_remotes(window, cx, |_, remotes, window, cx| {
            let spec = FormSpec {
                title: m::FORM_ADD_TITLE.get().to_string(),
                intro: Vec::new(),
                fields: vec![
                    Field::text(strings::REMOTE_NAME_LABEL, m::WHAT_REMOTE_NAME.get(), ""),
                    Field::text(
                        strings::REMOTE_FETCH_URL_LABEL,
                        "https://host/owner/repo.git",
                        "",
                    ),
                ],
                submit: m::FORM_ADD_BUTTON.get().to_string(),
                cancel: msg::dialog::CANCEL.get().to_string(),
            };
            let check = Rc::new(move |v: &form_dialog::Values| {
                rules::validate_remote_name(v.text(0), &remotes, None)?;
                rules::validate_remote_url(v.text(1)).map_err(m::fetch_url_error)
            });
            let validate = check.clone();
            form_dialog::open(
                window,
                cx,
                spec,
                move |v| validate(v),
                move |v, _, cx| {
                    let (name, url) = (v.text(0).to_string(), v.text(1).to_string());
                    with_workspace(cx, move |ws, _, cx| {
                        ws.backend.submit(Action::RemoteAdd { name, url });
                        ws.say(msg::remotes::FORM_ADD_TITLE.get(), cx);
                    });
                },
            );
        });
    }

    fn start_remote_edit(
        &mut self,
        target: Option<RefTarget>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        use msg::remotes as m;
        let Some(name) = self.remote_of(&target, cx) else {
            return self.say(m::SELECT_REMOTE_FIRST.get(), cx);
        };
        self.with_remotes(window, cx, move |this, remotes, window, cx| {
            let Some(old) = remotes.iter().find(|r| r.name == name).cloned() else {
                return this.say(msg::git_actions::gone(&name, ""), cx);
            };
            let spec = FormSpec {
                title: m::form_edit_title(&name),
                intro: Vec::new(),
                fields: vec![
                    Field::text(strings::REMOTE_NAME_LABEL, "", &old.name),
                    Field::text(strings::REMOTE_FETCH_URL_LABEL, "", &old.fetch_url),
                    Field::text(
                        strings::REMOTE_PUSH_URL_LABEL,
                        m::FORM_PUSH_PLACEHOLDER.get(),
                        old.push_url.as_deref().unwrap_or(""),
                    ),
                ],
                submit: m::FORM_SAVE.get().to_string(),
                cancel: msg::dialog::CANCEL.get().to_string(),
            };
            let kept = old.name.clone();
            let check = move |v: &form_dialog::Values| {
                rules::validate_remote_name(v.text(0), &remotes, Some(&kept))?;
                rules::validate_remote_url(v.text(1)).map_err(m::fetch_url_error)?;
                if !v.text(2).is_empty() {
                    rules::validate_remote_url(v.text(2))?;
                }
                Ok(())
            };
            form_dialog::open(window, cx, spec, check, move |v, _, cx| {
                let actions = rules::remote_edit_actions(&old, v.text(0), v.text(1), v.text(2));
                with_workspace(cx, move |ws, _, cx| {
                    let said = msg::remotes::form_edit_title(&old.name);
                    for action in actions {
                        ws.backend.submit(action);
                    }
                    ws.say(said, cx);
                });
            });
        });
    }

    fn start_remote_remove(
        &mut self,
        target: Option<RefTarget>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        use msg::remotes as m;
        let Some(name) = self.remote_of(&target, cx) else {
            return self.say(m::SELECT_REMOTE_FIRST.get(), cx);
        };
        let text = m::remove_confirm(&name);
        let mut lines = text.lines().map(str::to_string);
        let title = lines.next().unwrap_or_default();
        let spec = Spec {
            title,
            lines: lines.collect(),
            choices: vec![
                Choice::destructive(m::remove_remote(&name)),
                Choice::new(msg::dialog::CANCEL.get()),
            ],
            hint: msg::diff_actions::CONFIRM_HINT.get().to_string(),
            highlight_first: false,
            yes_no_keys: true,
        };
        dialogs::open(
            window,
            cx,
            spec,
            move |ix, cx| {
                if ix == 0 {
                    with_workspace(cx, move |ws, _, cx| {
                        ws.backend.submit(Action::RemoteRemove(name.clone()));
                        ws.say(m::remove_remote(&name), cx);
                    });
                }
            },
            || {},
        );
    }

    /// The push, pull and fetch rows of a branch's menu: the branch's remote is its upstream's, else
    /// `origin`, else the only one.
    fn branch_transfer(&mut self, op: RepoOp, target: Option<RefTarget>, cx: &mut Context<Self>) {
        let snap = self.snapshot(cx);
        let branch = match target {
            Some(RefTarget::Local(n)) => Some(n),
            _ => snap.head().map(|h| h.0.to_string()),
        };
        let Some(branch) = branch else {
            return self.say(strings::PUSH_DETACHED, cx);
        };
        if op == RepoOp::BranchPull {
            let action = Action::Pull(gitpoulpi_core::domain::PullOptions::default());
            let said = action.activity_label();
            self.backend.submit(action);
            return self.say(said, cx);
        }
        let remotes = self.backend.remotes();
        cx.spawn(async move |this, cx| {
            let remotes = remotes.await;
            let _ = this.update(cx, |this, cx| {
                let remotes = match remotes {
                    Ok(r) => r,
                    Err(why) => return crate::commands::status::set_error(cx, why),
                };
                let Some(remote) = rules::remote_of_branch(&snap.refs, &remotes, &branch) else {
                    return this.say(msg::remotes::NO_REMOTE_ADD.get(), cx);
                };
                if op == RepoOp::BranchFetch {
                    this.backend.submit(Action::Fetch(remote.clone()));
                    return this.say(msg::remotes::fetching_one(remote), cx);
                }
                let has_upstream = snap.refs.iter().any(|r| {
                    matches!(&r.kind, gitpoulpi_core::domain::RefKind::LocalBranch { name } if *name == branch)
                        && r.upstream.is_some()
                });
                this.backend.submit(Action::Push {
                    remote: remote.clone(),
                    opts: gitpoulpi_core::domain::PushOptions {
                        branch: Some(branch.clone()),
                        set_upstream: !has_upstream,
                        ..Default::default()
                    },
                });
                this.say(msg::remotes::pushing_to(&branch, &remote), cx);
            });
        })
        .detach();
    }
}
