//! What the Refs sidebar tells the ref commands: the selected row as a [`RefTarget`], a snapshot of the
//! refs, where the row is on screen (a menu opened from the keyboard appears there), and the mouse
//! gestures that end in a command (right click, double click, drag and drop).

use std::cell::RefCell;
use std::rc::Rc;

use gitpoulpi_app::refs_tree::RefRowKind;
use gitpoulpi_core::domain::TagWhere;
use gpui_kit::{
    div, px, Bounds, Context, IntoElement, ParentElement as _, Pixels, Point, Render, SharedString,
    Styled as _, Window,
};

use super::model::{Kind, Row, RowKind, Section};
use super::{RefsEvent, RefsPanel};
use crate::daemon::refs_ops::Snapshot;
use crate::ref_target::{RefTarget, SelectionFacts};
use crate::theme::Skin;

/// A ref picked up with the mouse (the drag payload).
#[derive(Clone)]
pub(super) struct DraggedRef(pub(super) crate::ref_target::RefTarget);

/// What follows the pointer while a ref is dragged.
pub(super) struct DragPreview(pub(super) String);

impl Render for DragPreview {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let skin = cx.global::<Skin>().clone();
        div()
            .px(px(8.0))
            .py(px(3.0))
            .rounded(px(3.0))
            .bg(skin.selection)
            .border_1()
            .border_color(skin.accent)
            .text_sm()
            .text_color(skin.text)
            .child(SharedString::from(self.0.clone()))
    }
}

/// Where the selected row was last drawn, in window coordinates.
pub type RowBounds = Rc<RefCell<Option<Bounds<Pixels>>>>;

/// The target a row stands for, `None` for a folder or a worktree.
pub fn target_of(section: &Section, row: &Row) -> Option<RefTarget> {
    match (&row.kind, section.kind) {
        (RowKind::Ref { full_name, .. }, kind) => {
            kind.ref_kind().map(|k| RefTarget::of_kind(k, full_name))
        }
        (RowKind::Stash { .. }, _) => row
            .key
            .strip_prefix("S:")
            .and_then(|n| n.parse().ok())
            .map(RefTarget::Stash),
        _ => None,
    }
}

impl RefsPanel {
    /// The refs and stashes as the panel shows them.
    pub fn snapshot(&self) -> Snapshot {
        Snapshot::new(self.refs.clone(), self.stashes.clone())
    }

    /// The selected row as a target.
    pub fn selected_target(&self) -> Option<RefTarget> {
        let (section, row) = self.cursor_row()?;
        target_of(&section, &row)
    }

    /// What is selected, for the palette's availability rules.
    pub fn selection_facts(&self) -> SelectionFacts {
        let mut facts = SelectionFacts {
            has_stash: !self.stashes.is_empty(),
            ..SelectionFacts::default()
        };
        let Some((_, row)) = self.cursor_row() else {
            return facts;
        };
        if matches!(row.kind, RowKind::Folder { .. }) {
            facts.folder = true;
            return facts;
        }
        match self.selected_target() {
            Some(RefTarget::Local(_)) => facts.local_branch = true,
            Some(RefTarget::Remote(_)) => facts.remote_branch = true,
            Some(RefTarget::Stash(_)) => facts.stash = true,
            Some(RefTarget::Tag(name)) => {
                facts.tag = true;
                let place = self.snapshot().tag_place(&name).place;
                facts.tag_unpushed = place == Some(TagWhere::Local);
                facts.tag_remote_only = place == Some(TagWhere::Remote);
                facts.tag_differs = place == Some(TagWhere::Differs);
            }
            None => {}
        }
        facts
    }

    /// Whether the ref is hidden from the graph, or soloed.
    pub fn hidden_or_solo(&self, target: &RefTarget) -> (bool, bool) {
        let name = target.name();
        let hidden = match target.kind() {
            Some(RefRowKind::Remote) => {
                gitpoulpi_app::refs_tree::hidden_in_graph(&self.hidden, &name, true)
            }
            Some(_) => gitpoulpi_app::refs_tree::hidden_in_graph(&self.hidden, &name, false),
            None => false,
        };
        (hidden, self.solo.contains(&name))
    }

    /// Where the selected row is on screen, once it has been drawn.
    pub fn selected_row_origin(&self) -> Option<Point<Pixels>> {
        self.row_bounds
            .borrow()
            .map(|b| Point::new(b.origin.x + Pixels::from(24.0), b.origin.y + b.size.height))
    }

    /// Selects the row of `target`, wherever its section is, and scrolls it into view.
    pub fn select_target(&mut self, target: &RefTarget, cx: &mut Context<Self>) -> bool {
        let found = self.sections.iter().find_map(|s| {
            s.rows
                .iter()
                .position(|r| target_of(s, r).as_ref() == Some(target))
                .map(|ix| (s.kind, ix))
        });
        match found {
            Some((kind, ix)) => {
                self.select(kind, ix, cx);
                true
            }
            None => false,
        }
    }

    /// Shows the target's commit in the Log.
    pub fn reveal(&mut self, target: &RefTarget, cx: &mut Context<Self>) -> bool {
        match self.snapshot().oid_of(target) {
            Some(oid) => {
                cx.emit(RefsEvent::Selected(oid));
                true
            }
            None => false,
        }
    }

    /// A right click on a row: selects it and asks the window for its menu.
    pub(super) fn row_menu(
        &mut self,
        kind: Kind,
        ix: usize,
        at: Point<Pixels>,
        cx: &mut Context<Self>,
    ) {
        self.select(kind, ix, cx);
        if let Some(target) = self.selected_target() {
            cx.emit(RefsEvent::Menu { target, at });
        }
    }

    /// A double click on a row: checks a branch out, detaches onto a tag, shows a stash's changes.
    pub(super) fn row_double_click(&mut self, kind: Kind, ix: usize, cx: &mut Context<Self>) {
        self.select(kind, ix, cx);
        if let Some(target) = self.selected_target() {
            cx.emit(RefsEvent::Open(target));
        }
    }

    /// A ref picked up and dropped on another row.
    pub(super) fn row_dropped(
        &mut self,
        source: RefTarget,
        kind: Kind,
        ix: usize,
        at: Point<Pixels>,
        cx: &mut Context<Self>,
    ) {
        let Some(target) = self
            .section(kind)
            .and_then(|s| s.rows.get(ix).and_then(|r| target_of(s, r)))
        else {
            return;
        };
        cx.emit(RefsEvent::Dropped { source, target, at });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn section(kind: Kind, rows: Vec<Row>) -> Section {
        Section {
            kind,
            count: rows.len(),
            names: Vec::new(),
            rows,
        }
    }

    #[test]
    fn rows_name_their_targets() {
        let local = Row {
            key: "L:main".into(),
            label: "main".into(),
            depth: 0,
            kind: RowKind::Ref {
                full_name: "main".into(),
                target: None,
                is_head: true,
                tag_where: None,
            },
        };
        let stash = Row {
            key: "S:3".into(),
            label: "stash@{3}".into(),
            depth: 0,
            kind: RowKind::Stash { target: None },
        };
        let folder = Row {
            key: "F".into(),
            label: "f".into(),
            depth: 0,
            kind: RowKind::Folder {
                path: "f".into(),
                is_remote_root: false,
                expanded: true,
            },
        };
        let local_section = section(Kind::Local, vec![local.clone(), folder.clone()]);
        assert_eq!(
            target_of(&local_section, &local),
            Some(RefTarget::Local("main".into()))
        );
        assert_eq!(target_of(&local_section, &folder), None);
        let stashes = section(Kind::Stashes, vec![stash.clone()]);
        assert_eq!(target_of(&stashes, &stash), Some(RefTarget::Stash(3)));
    }
}
