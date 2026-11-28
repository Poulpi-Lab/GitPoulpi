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
mod navigation;
mod ops;
mod rows;
mod visibility;

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
    actions, canvas, div, px, uniform_list, AnyElement, App, AppContext as _, Context, Entity,
    EventEmitter, FocusHandle, Focusable, FontWeight, Hsla, InteractiveElement as _, IntoElement,
    KeyBinding, MouseButton, ParentElement as _, Render, ScrollStrategy, SharedString,
    StatefulInteractiveElement as _, Styled as _, Subscription, Task, UniformListScrollHandle,
    WeakEntity, Window,
};
use model::{Inputs, Kind, Row, RowKind, Section};
pub use ops::{target_of, RowBounds};
use ops::{DragPreview, DraggedRef};

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
    /// A right click on a row: the window opens the ref menu there.
    Menu {
        target: crate::ref_target::RefTarget,
        at: gpui_kit::Point<gpui_kit::Pixels>,
    },
    /// A double click on a row: check the branch out (a stash: show its changes).
    Open(crate::ref_target::RefTarget),
    /// A ref dropped on another row: the window opens the drop menu.
    Dropped {
        source: crate::ref_target::RefTarget,
        target: crate::ref_target::RefTarget,
        at: gpui_kit::Point<gpui_kit::Pixels>,
    },
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
    /// Bumped by every change made here: a read of the stored sets taken under an older value is
    /// stale (the user changed something while it ran) and is thrown away.
    vis_gen: u64,
    /// `SetRefVisibility` calls this panel sent that the daemon has not answered. While any is out,
    /// the stored sets are mid-write, so what another frontend's event says cannot be told from our
    /// own echo and is not applied.
    vis_pending: usize,
    /// An event arrived while calls were out: re-read once the last one is answered.
    vis_stale: bool,
    /// Where the selected row was drawn (a menu opened by the keyboard appears there).
    row_bounds: RowBounds,
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
            vis_gen: 0,
            vis_pending: 0,
            vis_stale: false,
            row_bounds: RowBounds::default(),
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
                let ui = crate::prefs::PrefsStore::get(cx).ui();
                this.hierarchical = ui.ref_tree;
                this.hidden_sections = hidden_from_names(&ui.sidebar_sections_hidden);
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

    /// Whether another frontend's change is waiting for this panel's own writes to be answered.
    pub fn visibility_event_deferred(&self) -> bool {
        self.vis_stale
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
