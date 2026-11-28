//! The Refs sidebar: Filter box, then the LOCAL, REMOTE, WORKTREES, STASHES and TAGS sections, each a
//! boxed list with a chevron, an icon, its title and a count, folders split on `/`, a check mark on the
//! checked-out branch, an arrow on tags that are also on the remote. Colours follow the theme
//! (`refs.local` / `refs.remote` / `refs.tag`).
//!
//! The rows are the shared `gitpoulpi_app::refs_tree` ones (see `model`); hiding a ref from the graph
//! and soloing it are the terminal UI's own sets, kept in the repository's `repo-state.toml` through the
//! daemon (`SetRefVisibility`), so both frontends agree. The Log reads the same sets
//! (`LogPanel::set_visibility`, fed by [`RefsEvent::VisibilityChanged`]).
//!
//! Everything the mouse does has a plain method: [`RefsPanel::move_selection`], [`activate`](
//! RefsPanel::activate), [`toggle_hidden`](RefsPanel::toggle_hidden), [`toggle_solo`](
//! RefsPanel::toggle_solo) and friends, which the keys below call and a shared keymap can call too.

mod model;

use std::collections::{HashMap, HashSet};
use std::ops::Range;
use std::sync::Arc;

use gitpoulpi_app::palette::vocab::refs_sidebar::SidebarSection;
use gitpoulpi_app::refs_sections::hidden_from_names;
use gitpoulpi_app::refs_tree::{
    hide_folder, load_collapsed, load_ref_visibility, refs_title, refs_under_folder,
    save_collapsed, split_hidden, toggle_folder_membership, toggle_hide_ref, CollapsedFolders,
    FolderHide, RefRowKind,
};
use gitpoulpi_app::tag_where::{snapshot_of, tooltip_lines, TagInfo};
use gitpoulpi_core::domain::{
    Oid, OperationKind, Ref, RefKind, Stash, Submodule, TagWhere, Worktree,
};
use gitpoulpi_i18n::msg;
use gpui_kit::assets::IconName as AssetIcon;
use gpui_kit::component::{
    h_flex,
    input::{Input, InputEvent, InputState},
    scroll::Scrollbar,
    v_flex, Icon, Sizable as _,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    actions, div, px, uniform_list, AnyElement, App, AppContext as _, Context, Entity,
    EventEmitter, FocusHandle, Focusable, FontWeight, Hsla, InteractiveElement as _, IntoElement,
    KeyBinding, MouseButton, ParentElement as _, Render, ScrollStrategy, SharedString,
    StatefulInteractiveElement as _, Styled as _, Subscription, Task, UniformListScrollHandle,
    WeakEntity, Window,
};
use model::{Inputs, Kind, Row, RowKind, Section};

use crate::daemon::Shared;
use crate::theme::Skin;

actions!(
    gp_refs,
    [RowUp, RowDown, RowLeft, RowRight, Activate, ToggleSection]
);

/// Key context of the rows; the Filter box sits outside it so typing a letter there stays typing.
const CONTEXT: &str = "GpRefs";

/// Only the keys that move around: the arrows and `h`/`j`/`k`/`l`, Enter, and Space for a section. The
/// commands (solo, clear solo, show all hidden, hide, expand all, collapse all) have no binding here:
/// the shared keymap resolves their keys and `commands::run` runs them, the same way for the toolbar,
/// the menus and the palette.
pub fn init(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("up", RowUp, Some(CONTEXT)),
        KeyBinding::new("k", RowUp, Some(CONTEXT)),
        KeyBinding::new("down", RowDown, Some(CONTEXT)),
        KeyBinding::new("j", RowDown, Some(CONTEXT)),
        KeyBinding::new("left", RowLeft, Some(CONTEXT)),
        KeyBinding::new("h", RowLeft, Some(CONTEXT)),
        KeyBinding::new("right", RowRight, Some(CONTEXT)),
        KeyBinding::new("l", RowRight, Some(CONTEXT)),
        KeyBinding::new("enter", Activate, Some(CONTEXT)),
        KeyBinding::new("space", ToggleSection, Some(CONTEXT)),
    ]);
}

pub enum RefsEvent {
    /// Show this commit in the Log (Enter or a click on a ref).
    Selected(Oid),
    /// The hidden or soloed refs changed; the Log re-reads its walk.
    VisibilityChanged {
        hidden: HashSet<String>,
        solo: HashSet<String>,
    },
}

pub struct RefsPanel {
    backend: Shared,
    refs: Vec<Ref>,
    stashes: Vec<Stash>,
    worktrees: Vec<Worktree>,
    submodules: Vec<Submodule>,
    /// Where each tag lives, for the place marker and its tooltip.
    tags: Arc<HashMap<String, TagInfo>>,
    /// `ui.sidebar_sections_hidden`.
    hidden_sections: HashSet<SidebarSection>,
    /// `ui.ref_tree`: folders on `/`.
    hierarchical: bool,
    collapsed: CollapsedFolders,
    hidden: HashSet<String>,
    solo: HashSet<String>,
    /// Sections the user folded away (the row list is skipped, the header stays).
    closed: HashSet<Kind>,
    sections: Vec<Arc<Section>>,
    scrolls: HashMap<Kind, UniformListScrollHandle>,
    /// The selected row: its section and `Row::key`, so a rebuild keeps it.
    cursor: Option<(Kind, String)>,
    /// A line the last action wants to say (a hide that did nothing, and why).
    note: Option<String>,
    filter: Entity<InputState>,
    query: String,
    focus: FocusHandle,
    _task: Option<Task<()>>,
    _subs: Vec<Subscription>,
}

impl EventEmitter<RefsEvent> for RefsPanel {}

fn section_icon(kind: Kind) -> AssetIcon {
    match kind {
        Kind::Local => AssetIcon::Laptop,
        Kind::Remote => AssetIcon::Globe,
        Kind::Worktrees => AssetIcon::Leaf,
        Kind::Stashes => AssetIcon::Package,
        Kind::Tags => AssetIcon::Tag,
        Kind::Submodules => AssetIcon::Folder,
    }
}

const ROW_H: f32 = 24.0;

/// The colours a row is drawn with.
#[derive(Clone)]
struct Palette {
    text: Hsla,
    muted: Hsla,
    folder: Hsla,
    warning: Hsla,
    accent: Hsla,
    selection: Hsla,
    hover: Hsla,
    section: [Hsla; 6],
}

impl RefsPanel {
    pub fn new(backend: Shared, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let filter =
            cx.new(|cx| InputState::new(window, cx).placeholder(msg::sidebar::FILTER_HINT.get()));
        let subs = vec![cx.subscribe(&filter, |this, input, ev: &InputEvent, cx| {
            if matches!(ev, InputEvent::Change) {
                this.query = input.read(cx).value().to_string();
                this.rebuild(cx);
            }
        })];
        let root = backend.repo_root();
        let (hidden, solo) = load_ref_visibility(&root);
        let mut this = Self {
            backend,
            refs: Vec::new(),
            stashes: Vec::new(),
            worktrees: Vec::new(),
            submodules: Vec::new(),
            tags: Arc::default(),
            hidden_sections: HashSet::new(),
            hierarchical: true,
            collapsed: load_collapsed(&root),
            hidden,
            solo,
            closed: HashSet::new(),
            sections: Vec::new(),
            scrolls: HashMap::new(),
            cursor: None,
            note: None,
            filter,
            query: String::new(),
            focus: cx.focus_handle(),
            _task: None,
            _subs: subs,
        };
        this.reload(cx);
        this
    }

    pub fn reload(&mut self, cx: &mut Context<Self>) {
        let refs = self.backend.refs();
        let stashes = self.backend.stashes();
        let worktrees = self.backend.worktrees();
        let submodules = self.backend.submodules();
        self._task = Some(cx.spawn(async move |this, cx| {
            let refs = refs.await.unwrap_or_default();
            let stashes = stashes.await.unwrap_or_default();
            let worktrees = worktrees.await.unwrap_or_default();
            let submodules = submodules.await.unwrap_or_default();
            let _ = this.update(cx, |this, cx| {
                let ui = gitpoulpi_config::Settings::resolve_ambient()
                    .ok()
                    .map(|s| s.config().ui.clone());
                this.hierarchical = ui.as_ref().is_none_or(|ui| ui.ref_tree);
                this.hidden_sections = ui
                    .map(|ui| hidden_from_names(&ui.sidebar_sections_hidden))
                    .unwrap_or_default();
                this.tags = Arc::new(snapshot_of(&refs));
                this.refs = refs;
                this.stashes = stashes;
                this.worktrees = worktrees;
                this.submodules = submodules;
                this.rebuild(cx);
            });
        }));
    }

    fn rebuild(&mut self, cx: &mut Context<Self>) {
        let filter = self.query.clone();
        self.sections = model::build_sections(&Inputs {
            refs: &self.refs,
            stashes: &self.stashes,
            worktrees: &self.worktrees,
            submodules: &self.submodules,
            hidden_sections: &self.hidden_sections,
            filter: &filter,
            hierarchical: self.hierarchical,
            collapsed: &self.collapsed,
        })
        .into_iter()
        .map(Arc::new)
        .collect();
        if let Some((kind, key)) = &self.cursor {
            let kept = self
                .section(*kind)
                .is_some_and(|s| s.rows.iter().any(|r| &r.key == key));
            if !kept {
                self.cursor = None;
            }
        }
        cx.notify();
    }

    pub fn focus_filter(&self, window: &mut Window, cx: &mut App) {
        self.filter.read(cx).focus_handle(cx).focus(window, cx);
    }

    // -- what is shown, for callers and tests ---------------------------------

    /// The hidden refs (a hidden remote is its name with a trailing `/`).
    pub fn hidden(&self) -> &HashSet<String> {
        &self.hidden
    }

    pub fn solo(&self) -> &HashSet<String> {
        &self.solo
    }

    /// The labels of the rows shown in a section, folds applied.
    pub fn row_labels(&self, kind: Kind) -> Vec<String> {
        self.section(kind)
            .map(|s| s.rows.iter().map(|r| r.label.clone()).collect())
            .unwrap_or_default()
    }

    /// The section titles with their counts, in order.
    pub fn section_titles(&self) -> Vec<String> {
        self.sections
            .iter()
            .map(|s| format!("{} ({})", s.kind.title(), s.count))
            .collect()
    }

    /// The label of the selected row.
    pub fn selected_label(&self) -> Option<String> {
        let (section, row) = self.cursor_row()?;
        let _ = section;
        Some(row.label.clone())
    }

    fn section(&self, kind: Kind) -> Option<&Arc<Section>> {
        self.sections.iter().find(|s| s.kind == kind)
    }

    fn cursor_row(&self) -> Option<(Arc<Section>, Row)> {
        let (kind, key) = self.cursor.as_ref()?;
        let section = self.section(*kind)?.clone();
        let row = section.rows.iter().find(|r| &r.key == key)?.clone();
        Some((section, row))
    }

    /// Open sections' rows in display order, as `(section, row index)`.
    fn visible(&self) -> Vec<(Kind, usize)> {
        self.sections
            .iter()
            .filter(|s| !self.closed.contains(&s.kind))
            .flat_map(|s| (0..s.rows.len()).map(move |i| (s.kind, i)))
            .collect()
    }

    // -- selection and keys ---------------------------------------------------

    fn select(&mut self, kind: Kind, ix: usize, cx: &mut Context<Self>) {
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
    fn toggle_folder(
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

    fn toggle_section(&mut self, kind: Kind, cx: &mut Context<Self>) {
        if !self.closed.remove(&kind) {
            self.closed.insert(kind);
        }
        cx.notify();
    }

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

    fn hide_row(&mut self, section: &Section, row: &Row, cx: &mut Context<Self>) {
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

    fn solo_row(&mut self, section: &Section, row: &Row, cx: &mut Context<Self>) {
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

    fn ref_kind_of(&self, full_name: &str) -> Option<RefRowKind> {
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
    fn visibility_changed(&mut self, cx: &mut Context<Self>) {
        self.rebuild(cx);
        cx.emit(RefsEvent::VisibilityChanged {
            hidden: self.hidden.clone(),
            solo: self.solo.clone(),
        });
        let (hidden, hidden_remotes) = split_hidden(&self.hidden);
        let mut solo: Vec<String> = self.solo.iter().cloned().collect();
        solo.sort();
        // Fire and forget: the call runs on the daemon runtime whether or not it is awaited.
        drop(self.backend.call::<serde_json::Value>(
            OperationKind::SetRefVisibility,
            serde_json::json!({
                "mode": "replace",
                "hidden": hidden,
                "solo": solo,
                "hidden_remotes": hidden_remotes,
            }),
        ));
    }

    // -- drawing --------------------------------------------------------------

    /// The pane's title and the label of its button, from the shared rule (`refs_title`).
    fn title_parts(&self) -> (String, Option<&'static str>) {
        let total: usize = self
            .sections
            .iter()
            .filter(|s| s.kind.ref_kind().is_some())
            .map(|s| s.count)
            .sum();
        let remote_names: Vec<String> = self
            .section(Kind::Remote)
            .map(|s| s.names.clone())
            .unwrap_or_default();
        let hidden = model::hidden_count(&self.hidden, &remote_names);
        let (title, button) = refs_title(self.solo.len(), total, hidden);
        (title.trim().to_string(), button)
    }

    fn row_element(
        me: &WeakEntity<RefsPanel>,
        section: &Arc<Section>,
        ix: usize,
        salt: usize,
        view: &RowView,
        pal: &Palette,
    ) -> AnyElement {
        let row = &section.rows[ix];
        let kind = section.kind;
        let (hidden, soloed) = (
            model::row_hidden(section, row, &view.hidden),
            model::row_soloed(section, row, &view.solo),
        );
        let selected = view.cursor.as_deref() == Some(row.key.as_str());
        let color = pal.section[Kind::ALL.iter().position(|k| *k == kind).unwrap_or(0)];
        let (folder, is_head, tag_where, expanded, remote_root) = match &row.kind {
            RowKind::Folder {
                expanded,
                is_remote_root,
                ..
            } => (true, false, None, *expanded, *is_remote_root),
            RowKind::Ref {
                is_head, tag_where, ..
            } => (false, *is_head, *tag_where, false, false),
            _ => (false, false, None, false, false),
        };
        let refish = kind.ref_kind().is_some();
        // Where a tag lives, in words: the shared tooltip.
        let tip: Option<String> = match &row.kind {
            RowKind::Ref {
                full_name,
                tag_where: Some(_),
                ..
            } => view
                .tags
                .get(full_name)
                .map(|info| tooltip_lines(full_name, info).join("\n")),
            _ => None,
        };
        let (text, muted, accent) = (pal.text, pal.muted, pal.accent);
        let hover_bg = pal.hover;
        let sel_bg = pal.selection;
        let label_color = if hidden {
            muted
        } else if folder {
            pal.text
        } else {
            color
        };
        let me_row = me.clone();
        let (sec_idx, row_idx) = (section.clone(), ix);
        let btn = |id: &'static str, icon: AssetIcon, active: bool, tint: Hsla| {
            div()
                .id((id, salt + ix))
                .flex_none()
                .p(px(3.0))
                .rounded(px(3.0))
                .cursor_pointer()
                .opacity(if active { 1.0 } else { 0.0 })
                .group_hover("ref-row", |s| s.opacity(1.0))
                .hover(move |d| d.bg(hover_bg))
                .child(
                    Icon::new(icon)
                        .size(px(12.0))
                        .text_color(if active { tint } else { muted }),
                )
        };
        let hide_btn = {
            let (me, section, row) = (me.clone(), section.clone(), row.clone());
            btn(
                "ref-hide",
                if hidden {
                    AssetIcon::EyeOff
                } else {
                    AssetIcon::Eye
                },
                hidden,
                muted,
            )
            .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                cx.stop_propagation();
                let _ = me.update(cx, |this, cx| this.hide_row(&section, &row, cx));
            })
        };
        let solo_btn = {
            let (me, section, row) = (me.clone(), section.clone(), row.clone());
            btn("ref-solo", AssetIcon::Star, soloed, accent).on_mouse_down(
                MouseButton::Left,
                move |_, _, cx| {
                    cx.stop_propagation();
                    let _ = me.update(cx, |this, cx| this.solo_row(&section, &row, cx));
                },
            )
        };
        h_flex()
            .id(("ref-row", salt + ix))
            .group("ref-row")
            .w_full()
            .h(px(ROW_H))
            .flex_none()
            .items_center()
            .gap(px(5.0))
            .pl(px(8.0 + row.depth as f32 * 14.0))
            .pr(px(4.0))
            .cursor_pointer()
            .when(selected, |d| d.bg(sel_bg))
            .hover(move |d| d.bg(hover_bg))
            .child(
                div()
                    .w(px(12.0))
                    .flex_none()
                    .text_xs()
                    .text_color(text)
                    .child(if folder {
                        ""
                    } else if is_head {
                        "\u{2713}"
                    } else {
                        ""
                    }),
            )
            .when(folder, |d| {
                d.child(
                    Icon::new(if expanded {
                        AssetIcon::ChevronDown
                    } else {
                        AssetIcon::ChevronRight
                    })
                    .size(px(11.0))
                    .text_color(muted),
                )
                .child(
                    Icon::new(if remote_root {
                        AssetIcon::Globe
                    } else if expanded {
                        AssetIcon::FolderOpen
                    } else {
                        AssetIcon::Folder
                    })
                    .size(px(14.0))
                    .text_color(if remote_root { color } else { pal.folder }),
                )
            })
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .truncate()
                    .text_sm()
                    .text_color(label_color)
                    .when(is_head, |d| d.font_weight(FontWeight::BOLD))
                    .when(hidden, |d| d.opacity(0.6))
                    .child(SharedString::from(row.label.clone())),
            )
            .when_some(tag_where, |d, w| {
                d.child(
                    div()
                        .flex_none()
                        .text_xs()
                        .text_color(match w {
                            TagWhere::Local | TagWhere::Differs => pal.warning,
                            TagWhere::Remote => pal.section[1],
                            TagWhere::Both => muted,
                        })
                        .child(model::tag_glyph(w)),
                )
            })
            .when(refish, |d| d.child(hide_btn).child(solo_btn))
            .when_some(tip, |d, tip| {
                d.tooltip(move |window, cx| {
                    gpui_kit::component::tooltip::Tooltip::new(tip.clone()).build(window, cx)
                })
            })
            .on_click(move |_, window, cx| {
                let section = sec_idx.clone();
                let _ = me_row.update(cx, |this, cx| {
                    window.focus(&this.focus, cx);
                    this.select(section.kind, row_idx, cx);
                    let row = &section.rows[row_idx];
                    match row.kind {
                        RowKind::Folder { .. } => {
                            let row = row.clone();
                            this.toggle_folder(&section, &row, None, cx);
                        }
                        _ => {
                            if let Some(oid) = row.target() {
                                cx.emit(RefsEvent::Selected(oid));
                            }
                        }
                    }
                });
            })
            .into_any_element()
    }

    fn section_box(
        &mut self,
        si: usize,
        pal: &Palette,
        border: Hsla,
        cx: &mut Context<Self>,
    ) -> gpui_kit::Div {
        let section = self.sections[si].clone();
        let kind = section.kind;
        let ki = Kind::ALL.iter().position(|k| *k == kind).unwrap_or(0);
        let color = pal.section[ki];
        let open = !self.closed.contains(&kind);
        let rows = section.rows.len();
        let tall = rows > 9;
        let body_h = (rows.max(1) as f32 * ROW_H + 8.0).min(240.0);
        let soloing = !self.solo.is_empty() && kind.ref_kind().is_some();
        let count_text = if soloing {
            let soloed = section
                .names
                .iter()
                .filter(|n| self.solo.contains(*n))
                .count();
            format!("{} ({soloed}/{})", kind.title(), section.count)
        } else {
            format!("{} ({})", kind.title(), section.count)
        };
        let scroll = self.scrolls.entry(kind).or_default().clone();
        let view = RowView {
            tags: self.tags.clone(),
            hidden: self.hidden.clone(),
            solo: self.solo.clone(),
            cursor: self
                .cursor
                .as_ref()
                .filter(|(k, _)| *k == kind)
                .map(|(_, key)| key.clone()),
        };
        let me = cx.entity().downgrade();
        let salt = (ki + 1) * 1_000_000;
        let pal_rows = pal.clone();
        let rows_section = section.clone();
        let list = uniform_list(
            SharedString::from(format!("refs-{ki}")),
            rows,
            move |range: Range<usize>, _window, _cx| {
                range
                    .map(|ix| Self::row_element(&me, &rows_section, ix, salt, &view, &pal_rows))
                    .collect::<Vec<_>>()
            },
        )
        .size_full()
        .track_scroll(&scroll);
        let header = h_flex()
            .id(("sec-head", si))
            .flex_none()
            .h(px(28.0))
            .px(px(8.0))
            .gap(px(6.0))
            .items_center()
            .cursor_pointer()
            .on_click(cx.listener(move |this, _, _, cx| this.toggle_section(kind, cx)))
            .child(
                Icon::new(if open {
                    AssetIcon::ChevronDown
                } else {
                    AssetIcon::ChevronRight
                })
                .size(px(12.0))
                .text_color(pal.muted),
            )
            .child(
                Icon::new(section_icon(kind))
                    .size(px(15.0))
                    .text_color(color),
            )
            .child(
                div()
                    .min_w_0()
                    .truncate()
                    .text_xs()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(pal.text)
                    .child(count_text),
            )
            .child(div().flex_1())
            .when(kind == Kind::Remote, |d| {
                d.child(
                    Icon::new(AssetIcon::Plus)
                        .size(px(13.0))
                        .text_color(pal.muted),
                )
            });
        let mut boxed = v_flex()
            .w_full()
            .border_1()
            .border_color(border)
            .rounded(px(3.0))
            .overflow_hidden()
            .child(header);
        if open && rows > 0 {
            let body = div()
                .relative()
                .child(list)
                .child(Scrollbar::vertical(&scroll));
            boxed = if tall {
                boxed
                    .flex_1()
                    .min_h(px(120.0))
                    .child(body.flex_1().min_h_0())
            } else {
                boxed.flex_none().child(body.h(px(body_h)))
            };
        }
        boxed
    }
}

/// What a row needs besides the section: the sets and the selection.
struct RowView {
    tags: Arc<HashMap<String, TagInfo>>,
    hidden: HashSet<String>,
    solo: HashSet<String>,
    cursor: Option<String>,
}

impl Focusable for RefsPanel {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}

impl Render for RefsPanel {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let skin = cx.global::<Skin>().clone();
        let pal = Palette {
            text: skin.text,
            muted: skin.muted,
            folder: skin.warning,
            warning: skin.warning,
            accent: skin.accent,
            selection: skin.selection,
            hover: skin.hover,
            section: [
                skin.local,
                skin.remote,
                skin.local,
                skin.warning,
                skin.tag,
                skin.muted,
            ],
        };
        let mut sections = v_flex().flex_1().min_h_0().gap(px(6.0)).overflow_hidden();
        for si in 0..self.sections.len() {
            sections = sections.child(self.section_box(si, &pal, skin.border, cx));
        }
        let focused = self.focus.contains_focused(window, cx);
        let (title, button) = self.title_parts();
        let soloing = !self.solo.is_empty();
        let button = button.map(|label| (label, soloing));
        v_flex()
            .size_full()
            .bg(skin.side_bg)
            .p(px(6.0))
            .gap(px(6.0))
            .child(
                h_flex()
                    .flex_none()
                    .h(px(22.0))
                    .items_center()
                    .justify_between()
                    .gap(px(6.0))
                    .child(
                        div()
                            .min_w_0()
                            .truncate()
                            .text_sm()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(skin.text)
                            .child(title),
                    )
                    .child(
                        h_flex()
                            .flex_none()
                            .gap(px(6.0))
                            .items_center()
                            .when_some(button, |d, (label, solo)| {
                                let hover = skin.hover;
                                d.child(
                                    div()
                                        .id("refs-title-button")
                                        .px(px(6.0))
                                        .h(px(18.0))
                                        .flex()
                                        .items_center()
                                        .rounded(px(3.0))
                                        .text_xs()
                                        .text_color(skin.accent)
                                        .cursor_pointer()
                                        .hover(move |d| d.bg(hover))
                                        .child(label.trim().to_string())
                                        .on_click(cx.listener(move |this, _, _, cx| {
                                            if solo {
                                                this.clear_solo(cx)
                                            } else {
                                                this.show_all_hidden(cx)
                                            }
                                        })),
                                )
                            })
                            .child(
                                Icon::new(AssetIcon::ChevronLeft)
                                    .size(px(14.0))
                                    .text_color(skin.muted),
                            ),
                    ),
            )
            .when_some(self.note.clone(), |d, note| {
                d.child(
                    div()
                        .flex_none()
                        .px(px(4.0))
                        .text_xs()
                        .text_color(skin.muted)
                        .child(note),
                )
            })
            .child(
                v_flex()
                    .flex_none()
                    .border_1()
                    .border_color(skin.border)
                    .rounded(px(3.0))
                    .p(px(4.0))
                    .gap(px(2.0))
                    .child(
                        div()
                            .px(px(4.0))
                            .text_xs()
                            .text_color(skin.muted)
                            .child(msg::sidebar::FILTER_TITLE.get()),
                    )
                    .child(Input::new(&self.filter).small().cleanable(true)),
            )
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .flex()
                    .flex_col()
                    .key_context(CONTEXT)
                    .track_focus(&self.focus)
                    .on_action(cx.listener(|this, _: &RowUp, _, cx| this.move_selection(-1, cx)))
                    .on_action(cx.listener(|this, _: &RowDown, _, cx| this.move_selection(1, cx)))
                    .on_action(cx.listener(|this, _: &RowLeft, _, cx| this.collapse_selected(cx)))
                    .on_action(cx.listener(|this, _: &RowRight, _, cx| this.expand_selected(cx)))
                    .on_action(cx.listener(|this, _: &Activate, _, cx| this.activate(cx)))
                    .on_action(cx.listener(|this, _: &ToggleSection, _, cx| {
                        this.toggle_selected_section(cx)
                    }))
                    .rounded(px(3.0))
                    .border_1()
                    .border_color(if focused {
                        skin.border_focused
                    } else {
                        gpui_kit::transparent_black()
                    })
                    .child(sections),
            )
            .on_mouse_down(MouseButton::Left, |_, _, _| {})
    }
}

/// The kind of section a ref belongs to, for callers that hold a [`Ref`].
pub fn kind_of(r: &Ref) -> Kind {
    match r.kind {
        RefKind::LocalBranch { .. } => Kind::Local,
        RefKind::RemoteBranch { .. } => Kind::Remote,
        RefKind::Tag { .. } => Kind::Tags,
    }
}

pub use model::Kind as SectionKind;
