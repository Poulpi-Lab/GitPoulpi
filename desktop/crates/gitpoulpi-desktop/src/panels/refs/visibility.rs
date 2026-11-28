//! Hiding and soloing refs: the sets, the note a hide that did nothing leaves, and the writes to the
//! daemon with their echo handling (`vis_gen` / `vis_pending`).

use super::*;

impl RefsPanel {
    // -- hide and solo --------------------------------------------------------

    /// Hides the ref with this full name from the graph, or shows it again (a local branch or tag's
    /// name, a remote branch's `remote/name`). A remote branch of a remote hidden as a whole stays
    /// hidden with it, and the panel says so.
    pub fn toggle_hidden(&mut self, full_name: &str, cx: &mut Context<Self>) {
        let kind = self.ref_kind_of(full_name);
        self.note = toggle_hide_ref(&mut self.hidden, kind, full_name);
        self.visibility_changed(cx);
    }

    /// Solos the ref with this full name (the Log then shows only what it reaches), or clears it.
    pub fn toggle_solo(&mut self, full_name: &str, cx: &mut Context<Self>) {
        self.note = None;
        if !self.solo.remove(full_name) {
            self.solo.insert(full_name.to_string());
        }
        self.visibility_changed(cx);
    }

    /// Hides the selected row: a ref, or every ref under a folder or remote.
    pub fn toggle_hidden_selected(&mut self, cx: &mut Context<Self>) {
        let Some((section, row)) = self.cursor_row() else {
            return;
        };
        self.hide_row(&section, &row, cx);
    }

    /// Solos the selected row: a ref, or every ref under a folder or remote.
    pub fn toggle_solo_selected(&mut self, cx: &mut Context<Self>) {
        let Some((section, row)) = self.cursor_row() else {
            return;
        };
        self.solo_row(&section, &row, cx);
    }

    pub(super) fn hide_row(&mut self, section: &Section, row: &Row, cx: &mut Context<Self>) {
        let kind = section.kind.ref_kind();
        match &row.kind {
            RowKind::Ref { full_name, .. } => {
                self.note = toggle_hide_ref(&mut self.hidden, kind, full_name);
            }
            RowKind::Folder {
                path,
                is_remote_root,
                ..
            } => match hide_folder(&mut self.hidden, kind, path, *is_remote_root) {
                FolderHide::Done(note) => self.note = Some(note),
                FolderHide::Plain => {
                    self.note = None;
                    let under = refs_under_folder(None, &section.names, path);
                    toggle_folder_membership(&mut self.hidden, &under);
                }
            },
            _ => return,
        }
        self.visibility_changed(cx);
    }

    pub(super) fn solo_row(&mut self, section: &Section, row: &Row, cx: &mut Context<Self>) {
        self.note = None;
        match &row.kind {
            RowKind::Ref { full_name, .. } => {
                if !self.solo.remove(full_name) {
                    self.solo.insert(full_name.clone());
                }
            }
            RowKind::Folder { path, .. } => {
                let under = refs_under_folder(None, &section.names, path);
                toggle_folder_membership(&mut self.solo, &under);
            }
            _ => return,
        }
        self.visibility_changed(cx);
    }

    /// Stops soloing.
    pub fn clear_solo(&mut self, cx: &mut Context<Self>) {
        if self.solo.is_empty() {
            return;
        }
        self.solo.clear();
        self.note = None;
        self.visibility_changed(cx);
    }

    /// Shows every hidden ref (and remote) again.
    pub fn show_all_hidden(&mut self, cx: &mut Context<Self>) {
        if self.hidden.is_empty() {
            return;
        }
        self.hidden.clear();
        self.note = None;
        self.visibility_changed(cx);
    }

    pub(super) fn ref_kind_of(&self, full_name: &str) -> Option<RefRowKind> {
        self.refs.iter().find_map(|r| match &r.kind {
            RefKind::LocalBranch { name } if name == full_name => Some(RefRowKind::Local),
            RefKind::Tag { name } if name == full_name => Some(RefRowKind::Tag),
            RefKind::RemoteBranch { remote, name } if format!("{remote}/{name}") == full_name => {
                Some(RefRowKind::Remote)
            }
            _ => None,
        })
    }

    /// The sets changed: redraw, tell the Log, and write them where the terminal UI reads them.
    pub(super) fn visibility_changed(&mut self, cx: &mut Context<Self>) {
        self.rebuild(cx);
        cx.emit(RefsEvent::VisibilityChanged {
            hidden: self.hidden.clone(),
            solo: self.solo.clone(),
        });
        let (hidden, hidden_remotes) = split_hidden(&self.hidden);
        let mut solo: Vec<String> = self.solo.iter().cloned().collect();
        solo.sort();
        self.vis_gen += 1;
        self.vis_pending += 1;
        let call = self.backend.call::<serde_json::Value>(
            OperationKind::SetRefVisibility,
            serde_json::json!({
                "mode": "replace",
                "hidden": hidden,
                "solo": solo,
                "hidden_remotes": hidden_remotes,
            }),
        );
        cx.spawn(async move |this, cx| {
            let _ = call.await;
            let _ = this.update(cx, |this, cx| {
                this.vis_pending -= 1;
                if this.vis_pending == 0 && std::mem::take(&mut this.vis_stale) {
                    this.reread_visibility(cx);
                }
            });
        })
        .detach();
    }

    /// Another frontend (the CLI, the terminal UI) changed the hidden or soloed refs
    /// (`LinkEvent::RefVisibilityChanged`): shows what is stored now, in this panel and in the Log.
    ///
    /// The daemon may also echo this panel's own change. That is harmless (the stored sets equal the
    /// shown ones, nothing changes), but the stored sets lag behind a change made here while its call
    /// is out, and a read taken then would undo it: so events wait for the calls, and a read that a
    /// newer local change overtook is dropped (`vis_gen`).
    pub fn on_visibility_event(&mut self, cx: &mut Context<Self>) {
        if self.vis_pending > 0 {
            self.vis_stale = true;
            return;
        }
        self.reread_visibility(cx);
    }

    pub(super) fn reread_visibility(&mut self, cx: &mut Context<Self>) {
        let generation = self.vis_gen;
        let root = self.backend.repo_root();
        cx.spawn(async move |this, cx| {
            let stored = cx
                .background_spawn(async move { load_ref_visibility(&root) })
                .await;
            let _ = this.update(cx, |this, cx| {
                if this.vis_gen != generation || this.vis_pending > 0 {
                    return;
                }
                let (hidden, solo) = stored;
                if hidden == this.hidden && solo == this.solo {
                    return;
                }
                this.hidden = hidden;
                this.solo = solo;
                this.note = None;
                this.rebuild(cx);
                cx.emit(RefsEvent::VisibilityChanged {
                    hidden: this.hidden.clone(),
                    solo: this.solo.clone(),
                });
            });
        })
        .detach();
    }
}
