//! The rows of "Remotes of <branch>": one row per remote of the repository,
//! with a Push checkbox, a "Pull too" checkbox, the branch name used on that
//! remote and where the local branch stands against it. The dialog itself
//! (the upstream list, the buttons, the frame, the close rules) is the
//! prompt dialog (`PromptKind::BranchRemotes`); this module is the model of
//! the table, its keys and its paint.
//!
//! Apply sends **one** `BranchSetRemotes` (the upstream when it changed, the
//! push targets and the pull sources), so one Undo takes all of it back.
//! "Rename on remote..." opens a prompt dialog nested in this one (never a
//! status-line prompt).

mod render;
#[cfg(test)]
mod tests;

use std::cell::Cell;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use gitpoulpi_core::domain::{
    Remote, RemoteGroupStatus, RemoteRelation, RemoteTargetState, TargetRef,
};
use gitpoulpi_i18n::msg::branch_remotes as msg;

use crate::daemon_link::Reply;
use crate::prompt_dialog::PromptDialog;
use crate::scrollbar::{BarDrag, Track};

/// One remote of the branch.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Row {
    pub(crate) remote: String,
    /// The branch's name on that remote.
    pub(crate) dest: String,
    pub(crate) push: bool,
    pub(crate) pull: bool,
    /// The repository no longer has this remote (a rename or removal done
    /// with plain git); only Remove applies to the row.
    pub(crate) gone: bool,
    pub(crate) relation: Option<RemoteRelation>,
    /// The address pushes go to, lower-cased and without `.git` / trailing
    /// slashes, to tell two remotes that are one place.
    url: Option<String>,
}

enum Load {
    Loading,
    Ready,
    Failed(String),
}

/// What a key did in the table.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum RowsKey {
    /// Not a table key: the dialog goes on with it.
    Ignored,
    Handled,
    /// `Up` on the first row: the keyboard goes back to the upstream list.
    LeaveUp,
    /// `r`: rename the branch on the selected row's remote.
    Rename,
}

pub(crate) struct RemotesPane {
    pub(crate) branch: String,
    reply: Option<Reply<RemoteGroupStatus>>,
    load: Load,
    urls: Vec<(String, Option<String>)>,
    known: Vec<String>,
    pub(crate) rows: Vec<Row>,
    /// The push targets and pull sources as stored when the dialog opened
    /// (sorted config values), to tell a real change from none.
    initial: (Vec<String>, Vec<String>),
    pub(crate) initial_upstream: Option<String>,
    primary: Option<TargetRef>,
    pub(crate) cursor: usize,
    top: Cell<usize>,
    viewport: Cell<usize>,
    bar: Cell<Option<Track>>,
    bar_drag: BarDrag,
    /// The nested "Branch name on <remote>" prompt, with the row it renames.
    pub(crate) rename: Option<(Box<PromptDialog>, usize)>,
}

/// An address as compared for "same place": no case, no `.git`, no
/// trailing slash.
fn normalized(url: &str) -> String {
    let url = url.trim().trim_end_matches('/');
    url.strip_suffix(".git").unwrap_or(url).to_lowercase()
}

impl RemotesPane {
    pub(crate) fn new(
        branch: &str,
        remotes: &[Remote],
        upstream: Option<&str>,
        reply: Reply<RemoteGroupStatus>,
    ) -> Self {
        let mut pane = Self {
            branch: branch.to_string(),
            reply: Some(reply),
            load: Load::Loading,
            urls: remotes
                .iter()
                .map(|r| {
                    let url = r.push_url.as_deref().unwrap_or(&r.fetch_url);
                    (
                        r.name.clone(),
                        Some(normalized(url)).filter(|u| !u.is_empty()),
                    )
                })
                .collect(),
            known: remotes.iter().map(|r| r.name.clone()).collect(),
            rows: Vec::new(),
            initial: (Vec::new(), Vec::new()),
            initial_upstream: upstream.map(str::to_string),
            primary: None,
            cursor: 0,
            top: Cell::new(0),
            viewport: Cell::new(1),
            bar: Cell::new(None),
            bar_drag: BarDrag::default(),
            rename: None,
        };
        pane.set_primary(upstream);
        pane
    }

    /// The remote (and its branch) a `remote/branch` upstream names; the
    /// longest remote name wins (`grp/sub` is a legal remote name).
    fn parse_upstream(&self, upstream: &str) -> Option<TargetRef> {
        self.known
            .iter()
            .filter(|r| {
                upstream
                    .strip_prefix(r.as_str())
                    .is_some_and(|rest| rest.starts_with('/') && rest.len() > 1)
            })
            .max_by_key(|r| r.len())
            .map(|r| TargetRef {
                remote: r.clone(),
                branch: upstream[r.len() + 1..].to_string(),
            })
    }

    /// The upstream list changed: that remote becomes the primary row.
    pub(crate) fn set_primary(&mut self, upstream: Option<&str>) {
        self.primary = upstream.and_then(|u| self.parse_upstream(u));
    }

    pub(crate) fn is_ready(&self) -> bool {
        matches!(self.load, Load::Ready)
    }

    pub(crate) fn failure(&self) -> Option<&str> {
        match &self.load {
            Load::Failed(why) => Some(why),
            _ => None,
        }
    }

    fn is_primary(&self, row: &Row) -> bool {
        self.primary
            .as_ref()
            .is_some_and(|p| p.remote == row.remote)
    }

    /// The row's destination as shown: the upstream's branch for the
    /// primary row, its own otherwise.
    pub(crate) fn dest_shown<'a>(&'a self, row: &'a Row) -> &'a str {
        match &self.primary {
            Some(p) if p.remote == row.remote => &p.branch,
            _ => &row.dest,
        }
    }

    /// Takes the daemon's answer once it has come. `true` when the rows
    /// were (re)built.
    pub(crate) fn poll(&mut self) -> bool {
        let Some(answer) = self.reply.as_ref().and_then(Reply::take) else {
            return false;
        };
        self.reply = None;
        match answer {
            Ok(status) => {
                self.build(&status);
                self.load = Load::Ready;
            }
            Err(why) => self.load = Load::Failed(crate::daemon_link::error_text::error_text(&why)),
        }
        true
    }

    fn build(&mut self, status: &RemoteGroupStatus) {
        let state_of = |remote: &str| -> Option<&RemoteTargetState> {
            status.targets.iter().find(|t| t.remote == remote)
        };
        let mut rows: Vec<Row> = Vec::new();
        for name in &self.known {
            let target = state_of(name);
            let in_push = status.remotes.push_to.iter().find(|t| t.remote == *name);
            let in_pull = status.remotes.pull_from.iter().find(|t| t.remote == *name);
            rows.push(Row {
                remote: name.clone(),
                dest: target
                    .map(|t| t.branch.clone())
                    .or_else(|| in_push.or(in_pull).map(|t| t.branch.clone()))
                    .unwrap_or_else(|| self.branch.clone()),
                push: in_push.is_some(),
                pull: in_pull.is_some(),
                gone: false,
                relation: target.map(|t| t.relation),
                url: self
                    .urls
                    .iter()
                    .find(|(r, _)| r == name)
                    .and_then(|(_, u)| u.clone()),
            });
        }
        for gone in &status.remotes.missing {
            let in_push = status.remotes.push_to.iter().find(|t| t.remote == *gone);
            let in_pull = status.remotes.pull_from.iter().find(|t| t.remote == *gone);
            rows.push(Row {
                remote: gone.clone(),
                dest: in_push
                    .or(in_pull)
                    .map_or_else(|| self.branch.clone(), |t| t.branch.clone()),
                push: in_push.is_some(),
                pull: in_pull.is_some(),
                gone: true,
                relation: None,
                url: None,
            });
        }
        // The upstream the daemon reports is what the list started on.
        if self.primary.is_none() {
            self.primary = status
                .remotes
                .primary
                .clone()
                .filter(|p| self.known.contains(&p.remote));
        }
        let mut push: Vec<String> = status
            .remotes
            .push_to
            .iter()
            .map(|t| t.to_value(&self.branch))
            .collect();
        let mut pull: Vec<String> = status
            .remotes
            .pull_from
            .iter()
            .map(|t| t.to_value(&self.branch))
            .collect();
        push.sort();
        pull.sort();
        self.initial = (push, pull);
        self.rows = rows;
        self.cursor = 0;
    }

    fn value_of(&self, row: &Row) -> String {
        TargetRef {
            remote: row.remote.clone(),
            branch: row.dest.clone(),
        }
        .to_value(&self.branch)
    }

    /// The push targets and pull sources the rows now say, as config values.
    fn lists(&self) -> (Vec<String>, Vec<String>) {
        let mut push = Vec::new();
        let mut pull = Vec::new();
        for row in self.rows.iter().filter(|r| !r.gone && !self.is_primary(r)) {
            if row.push {
                push.push(self.value_of(row));
            }
            if row.pull {
                pull.push(self.value_of(row));
            }
        }
        (push, pull)
    }

    /// The first gone row left in the table.
    fn gone_left(&self) -> Option<&Row> {
        self.rows.iter().find(|r| r.gone)
    }

    /// What Apply sends: `Ok(None)` when nothing changed, `Err` with the
    /// reason Apply is refused.
    pub(crate) fn request(
        &self,
        chosen_upstream: Option<String>,
    ) -> Result<Option<crate::daemon_link::Action>, String> {
        if !self.is_ready() {
            return Err(msg::LOADING.get().to_string());
        }
        if let Some(gone) = self.gone_left() {
            return Err(msg::gone_first(&gone.remote));
        }
        let (mut push, mut pull) = self.lists();
        push.sort();
        pull.sort();
        let upstream_changed =
            chosen_upstream.is_some() && chosen_upstream != self.initial_upstream;
        if !upstream_changed && (push.clone(), pull.clone()) == self.initial {
            return Ok(None);
        }
        // Back in the order the rows show them.
        let (push, pull) = self.lists();
        Ok(Some(crate::daemon_link::Action::BranchSetRemotes {
            branch: self.branch.clone(),
            upstream: upstream_changed.then(|| chosen_upstream.unwrap_or_default()),
            push_to: push,
            pull_from: pull,
        }))
    }

    /// Two checked remotes that push to the same address, by name.
    pub(crate) fn same_url_warning(&self) -> Option<String> {
        let sends: Vec<&Row> = self
            .rows
            .iter()
            .filter(|r| !r.gone && (r.push || self.is_primary(r)))
            .collect();
        for (i, a) in sends.iter().enumerate() {
            let Some(url) = &a.url else { continue };
            if let Some(b) = sends[i + 1..].iter().find(|b| b.url.as_ref() == Some(url)) {
                return Some(msg::same_url(&a.remote, &b.remote));
            }
        }
        None
    }

    fn select(&mut self, row: usize) {
        self.cursor = row.min(self.rows.len().saturating_sub(1));
        self.keep_cursor_visible();
    }

    pub(crate) fn select_row(&mut self, row: usize) {
        self.select(row);
    }

    fn keep_cursor_visible(&mut self) {
        let view = self.viewport.get().max(1);
        let top = self.top.get();
        if self.cursor < top {
            self.top.set(self.cursor);
        } else if self.cursor >= top + view {
            self.top.set(self.cursor + 1 - view);
        }
    }

    pub(crate) fn toggle_push(&mut self, row: usize) {
        let primary = self.rows.get(row).is_some_and(|r| self.is_primary(r));
        if let Some(r) = self.rows.get_mut(row).filter(|r| !r.gone) {
            if !primary {
                r.push = !r.push;
            }
        }
        self.select(row);
    }

    pub(crate) fn toggle_pull(&mut self, row: usize) {
        let primary = self.rows.get(row).is_some_and(|r| self.is_primary(r));
        if let Some(r) = self.rows.get_mut(row).filter(|r| !r.gone) {
            if !primary {
                r.pull = !r.pull;
            }
        }
        self.select(row);
    }

    /// Forgets a gone row (it is not written back).
    pub(crate) fn remove(&mut self, row: usize) {
        if self.rows.get(row).is_some_and(|r| r.gone) {
            self.rows.remove(row);
            self.select(row);
        }
    }

    pub(crate) fn has_rows(&self) -> bool {
        !self.rows.is_empty()
    }

    /// Whether the selected row can be renamed (a real remote that is not
    /// the upstream's).
    pub(crate) fn rename_error(&self) -> Option<String> {
        match self.rows.get(self.cursor) {
            None => Some(msg::RENAME_NO_ROW.get().to_string()),
            Some(r) if r.gone => Some(msg::RENAME_NO_ROW.get().to_string()),
            Some(r) if self.is_primary(r) => Some(msg::RENAME_UPSTREAM_ROW.get().to_string()),
            Some(_) => None,
        }
    }

    /// The selected row's remote and current branch name.
    pub(crate) fn rename_target(&self) -> Option<(usize, String, String)> {
        self.rows
            .get(self.cursor)
            .map(|r| (self.cursor, r.remote.clone(), r.dest.clone()))
    }

    /// The nested prompt answered: `dest` is the branch's new name there.
    /// A name differing from the branch's own is a real change; checking
    /// Push too is what makes it useful, so a renamed row turns Push on.
    pub(crate) fn set_dest(&mut self, row: usize, dest: &str) {
        if let Some(r) = self.rows.get_mut(row) {
            r.dest = dest.to_string();
            if dest != self.branch {
                r.push = true;
            }
        }
    }

    /// Wheel over the table.
    pub(crate) fn wheel(&mut self, delta: isize) {
        let view = self.viewport.get().max(1);
        let max = self.rows.len().saturating_sub(view);
        self.top
            .set((self.top.get() as isize + delta).clamp(0, max as isize) as usize);
    }

    /// The scrollbar's mouse events; `true` when it took one.
    pub(crate) fn bar_mouse(&mut self, mouse: crossterm::event::MouseEvent) -> bool {
        let rows = self.rows.len();
        match self
            .bar_drag
            .on_mouse(self.bar.get(), mouse, rows, self.top.get())
        {
            Some(top) => {
                self.top.set(top);
                true
            }
            None => false,
        }
    }

    /// The table's keys while it has the keyboard.
    pub(crate) fn on_key(&mut self, key: &KeyEvent) -> RowsKey {
        if self.rows.is_empty() {
            return RowsKey::Ignored;
        }
        let last = self.rows.len() - 1;
        let view = self.viewport.get().max(1);
        if key.code == KeyCode::Up && self.cursor == 0 && key.modifiers.is_empty() {
            return RowsKey::LeaveUp;
        }
        if let Some(to) = crate::popup_list::nav(key, self.cursor, last + 1, view) {
            self.select(to);
            return RowsKey::Handled;
        }
        if !(key.modifiers - KeyModifiers::SHIFT).is_empty() {
            return RowsKey::Ignored;
        }
        match key.code {
            KeyCode::Char(' ') => self.toggle_push(self.cursor),
            KeyCode::Char('p' | 'P') => self.toggle_pull(self.cursor),
            KeyCode::Char('r' | 'R') => return RowsKey::Rename,
            KeyCode::Delete => self.remove(self.cursor),
            _ => return RowsKey::Ignored,
        }
        RowsKey::Handled
    }
}
