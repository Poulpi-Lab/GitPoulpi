//! Moving the selection and opening, closing and activating rows, sections and folders.

use super::*;

impl RefsPanel {
    // -- selection and keys ---------------------------------------------------

    pub(super) fn select(&mut self, kind: Kind, ix: usize, cx: &mut Context<Self>) {
        let Some(row) = self.section(kind).and_then(|s| s.rows.get(ix)) else {
            return;
        };
        self.cursor = Some((kind, row.key.clone()));
        self.scrolls
            .entry(kind)
            .or_default()
            .scroll_to_item(ix, ScrollStrategy::Nearest);
        cx.notify();
    }

    /// Moves the selection by `delta` rows across the open sections (the first move selects the first
    /// row).
    pub fn move_selection(&mut self, delta: isize, cx: &mut Context<Self>) {
        let vis = self.visible();
        if vis.is_empty() {
            return;
        }
        let at = self.cursor.as_ref().and_then(|(kind, key)| {
            vis.iter().position(|(k, i)| {
                k == kind
                    && self
                        .section(*k)
                        .and_then(|s| s.rows.get(*i))
                        .is_some_and(|r| &r.key == key)
            })
        });
        let next = match at {
            Some(p) => (p as isize + delta).clamp(0, vis.len() as isize - 1) as usize,
            None if delta >= 0 => 0,
            None => vis.len() - 1,
        };
        let (kind, ix) = vis[next];
        self.select(kind, ix, cx);
    }

    /// Enter: reveals the selected ref (or stash) in the Log, folds or unfolds a folder.
    pub fn activate(&mut self, cx: &mut Context<Self>) {
        let Some((section, row)) = self.cursor_row() else {
            return;
        };
        match &row.kind {
            RowKind::Folder { .. } => self.toggle_folder(&section, &row, None, cx),
            _ => {
                if let Some(oid) = row.target() {
                    cx.emit(RefsEvent::Selected(oid));
                }
            }
        }
    }

    /// Left: folds the selected folder, else moves to the folder around the row.
    pub fn collapse_selected(&mut self, cx: &mut Context<Self>) {
        let Some((section, row)) = self.cursor_row() else {
            return;
        };
        if matches!(row.kind, RowKind::Folder { expanded: true, .. }) {
            self.toggle_folder(&section, &row, Some(false), cx);
            return;
        }
        let at = section
            .rows
            .iter()
            .position(|r| r.key == row.key)
            .unwrap_or(0);
        let parent = section.rows[..at]
            .iter()
            .rposition(|r| r.depth < row.depth && matches!(r.kind, RowKind::Folder { .. }));
        if let Some(p) = parent {
            self.select(section.kind, p, cx);
        }
    }

    /// Right: unfolds the selected folder, else moves into it.
    pub fn expand_selected(&mut self, cx: &mut Context<Self>) {
        let Some((section, row)) = self.cursor_row() else {
            return;
        };
        match row.kind {
            RowKind::Folder {
                expanded: false, ..
            } => self.toggle_folder(&section, &row, Some(true), cx),
            RowKind::Folder { .. } => self.move_selection(1, cx),
            _ => {}
        }
    }

    /// Folds or unfolds a folder row (`want` forces the state) and remembers it for the repository.
    pub(super) fn toggle_folder(
        &mut self,
        section: &Section,
        row: &Row,
        want: Option<bool>,
        cx: &mut Context<Self>,
    ) {
        let (Some(kind), RowKind::Folder { path, expanded, .. }) =
            (section.kind.ref_kind(), &row.kind)
        else {
            return;
        };
        if want.is_some_and(|w| w == *expanded) {
            return;
        }
        let key = (kind, path.clone());
        if !self.collapsed.remove(&key) {
            self.collapsed.insert(key);
        }
        save_collapsed(&self.backend.repo_root(), &self.collapsed);
        self.rebuild(cx);
    }

    /// `=`: unfolds every folder.
    pub fn expand_all(&mut self, cx: &mut Context<Self>) {
        self.collapsed.clear();
        save_collapsed(&self.backend.repo_root(), &self.collapsed);
        self.rebuild(cx);
    }

    /// `-`: folds every folder.
    pub fn collapse_all(&mut self, cx: &mut Context<Self>) {
        self.collapsed = model::all_folders(&self.refs, self.hierarchical);
        save_collapsed(&self.backend.repo_root(), &self.collapsed);
        self.rebuild(cx);
    }

    /// Folds or unfolds the section the selection is in.
    pub fn toggle_selected_section(&mut self, cx: &mut Context<Self>) {
        if let Some((kind, _)) = self.cursor.clone() {
            self.toggle_section(kind, cx);
        }
    }

    pub(super) fn toggle_section(&mut self, kind: Kind, cx: &mut Context<Self>) {
        if !self.closed.remove(&kind) {
            self.closed.insert(kind);
        }
        cx.notify();
    }
}
