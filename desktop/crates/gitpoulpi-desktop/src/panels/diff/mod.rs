//! The centre pane when a file is open: GitPoulpi swaps the Log for the diff of the clicked file
//! (Esc goes back).
//!
//! What the rows are (which hunks the whitespace toggle keeps, line numbers, Split pairing, changed
//! words, the binary notice and the LFS card) is the shared `gitpoulpi_app::diff` model (`rows`); this
//! file is the desktop's own layout: a virtualised list (`gpui::list`, so a huge diff paints only the
//! rows in view), word wrap with the line numbers kept in their own column, or one line per row with a
//! horizontal scroll, and the theme's diff colours.

mod paint;
mod rows;
mod stage;

use std::sync::Arc;

use gitpoulpi_core::domain::{FileDiff, LineOrigin};
use gitpoulpi_i18n::msg;
use gpui_kit::assets::IconName as AssetIcon;
use gpui_kit::component::{h_flex, scroll::Scrollbar, spinner::Spinner, v_flex, Icon};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    actions, div, list, px, AnyElement, App, AppContext as _, Context, EventEmitter, FocusHandle,
    Focusable, FontWeight, InteractiveElement as _, IntoElement, KeyBinding, ListAlignment,
    ListOffset, ListState, ParentElement as _, Render, ScrollHandle,
    StatefulInteractiveElement as _, Styled as _, Subscription, Task, Window,
};
use paint::{char_width, Paint};
use rows::{Opts, Rows};
pub use stage::{DiffSource, PartRequest};

use crate::daemon::staging::PartOp;
use crate::theme::Skin;
use gitpoulpi_i18n::msg::diff_actions as msg_actions;

actions!(
    gp_diff,
    [
        CloseDiff,
        NextDiffFile,
        PrevDiffFile,
        CursorUp,
        CursorDown,
        ExtendUp,
        ExtendDown
    ]
);

/// Key context of the pane. Only Escape (the palette's close key) and `[` / `]` (previous and next file,
/// which the terminal UI's diff has no command for) are bound here. The registry's diff keys (`n`, `p`,
/// `m`, `w`, `s`, `q`) are resolved by the shared keymap and run through `commands::run`.
const CONTEXT: &str = "GpDiffPane";

pub fn init(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("escape", CloseDiff, Some(CONTEXT)),
        KeyBinding::new("]", NextDiffFile, Some(CONTEXT)),
        KeyBinding::new("[", PrevDiffFile, Some(CONTEXT)),
        // The cursor row (the hunk and lines a stage acts on); `j` / `k` are the terminal UI's spelling.
        KeyBinding::new("up", CursorUp, Some(CONTEXT)),
        KeyBinding::new("k", CursorUp, Some(CONTEXT)),
        KeyBinding::new("down", CursorDown, Some(CONTEXT)),
        KeyBinding::new("j", CursorDown, Some(CONTEXT)),
        KeyBinding::new("shift-up", ExtendUp, Some(CONTEXT)),
        KeyBinding::new("shift-down", ExtendDown, Some(CONTEXT)),
    ]);
}

pub enum DiffPaneEvent {
    Close,
    /// Stage, unstage or discard a hunk or some lines: the window carries it out.
    Part(PartRequest),
    /// A line for the status line (why a request did nothing).
    Say(String),
}

/// Height of one text row.
const LINE_H: f32 = 18.0;
/// Text size of the diff.
const TEXT_PX: f32 = 12.0;

pub struct DiffPane {
    files: Arc<Vec<FileDiff>>,
    selected: usize,
    opts: Opts,
    wrap: bool,
    rows: Arc<Rows>,
    list: ListState,
    hscroll: ScrollHandle,
    loading: bool,
    revision: u64,
    source: DiffSource,
    /// The row the keyboard is on, and where a line selection started (`v`).
    cursor: usize,
    anchor: Option<usize>,
    /// Set by a refresh after a stage: the cursor row to restore once the rows are rebuilt.
    keep_cursor: Option<usize>,
    focus: FocusHandle,
    _task: Option<Task<()>>,
    _subs: Vec<Subscription>,
}

impl EventEmitter<DiffPaneEvent> for DiffPane {}

impl DiffPane {
    pub fn new(_window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self {
            files: Arc::default(),
            selected: 0,
            opts: Opts::default(),
            wrap: false,
            rows: Arc::default(),
            list: ListState::new(0, ListAlignment::Top, px(400.0)),
            hscroll: ScrollHandle::new(),
            loading: false,
            revision: 0,
            source: DiffSource::Commit,
            cursor: 0,
            anchor: None,
            keep_cursor: None,
            focus: cx.focus_handle(),
            _task: None,
            _subs: Vec::new(),
        }
    }

    /// Shows file `ix` of `files`; the arrows and `[` / `]` move through the rest.
    pub fn show(
        &mut self,
        files: Arc<Vec<FileDiff>>,
        ix: usize,
        source: DiffSource,
        cx: &mut Context<Self>,
    ) {
        self.files = files;
        self.source = source;
        self.keep_cursor = None;
        self.select(ix, cx);
    }

    fn select(&mut self, ix: usize, cx: &mut Context<Self>) {
        self.selected = ix.min(self.files.len().saturating_sub(1));
        self.anchor = None;
        self.rebuild(cx);
    }

    /// Builds the rows of the selected file off the UI thread: a diff of tens of thousands of lines
    /// must not stall a frame.
    fn rebuild(&mut self, cx: &mut Context<Self>) {
        self.revision += 1;
        let rev = self.revision;
        let Some(file) = self.files.get(self.selected).cloned() else {
            self.rows = Arc::default();
            self.list.reset(0);
            self.loading = false;
            cx.notify();
            return;
        };
        let opts = self.opts;
        self.loading = true;
        self._task = Some(cx.spawn(async move |this, cx| {
            let built = cx
                .background_spawn(async move { rows::build(&file, opts) })
                .await;
            let _ = this.update(cx, |this, cx| {
                if this.revision != rev {
                    return;
                }
                this.loading = false;
                this.list.reset(built.lines.len());
                this.rows = Arc::new(built);
                this.cursor = match this.keep_cursor.take() {
                    Some(row) => row.min(this.rows.lines.len().saturating_sub(1)),
                    None => this.home_row(),
                };
                cx.notify();
            });
        }));
        cx.notify();
    }

    pub fn step(&mut self, d: isize, cx: &mut Context<Self>) {
        if self.files.is_empty() {
            return;
        }
        let ix = self
            .selected
            .saturating_add_signed(d)
            .min(self.files.len() - 1);
        self.select(ix, cx);
    }

    /// `m`: Side by side or Inline.
    pub fn toggle_layout(&mut self, cx: &mut Context<Self>) {
        self.opts.split = !self.opts.split;
        self.rebuild(cx);
    }

    /// `s`: show or fold whitespace-only changes.
    pub fn toggle_whitespace(&mut self, cx: &mut Context<Self>) {
        self.opts.show_whitespace = !self.opts.show_whitespace;
        self.rebuild(cx);
    }

    /// `w`: word wrap, or one line per row with a horizontal scroll.
    pub fn toggle_wrap(&mut self, cx: &mut Context<Self>) {
        self.wrap = !self.wrap;
        cx.notify();
    }

    /// `n` / `p`: the next or previous hunk header at the top of the pane.
    pub fn step_hunk(&mut self, forward: bool, cx: &mut Context<Self>) {
        let top = self.list.logical_scroll_top().item_ix;
        let starts = &self.rows.hunk_starts;
        let target = if forward {
            starts.iter().copied().find(|s| *s > top)
        } else {
            starts.iter().copied().rev().find(|s| *s < top)
        };
        if let Some(ix) = target {
            self.list.scroll_to(ListOffset {
                item_ix: ix,
                offset_in_item: px(0.0),
            });
            cx.notify();
        }
    }

    pub fn is_split(&self) -> bool {
        self.opts.split
    }

    pub fn shows_whitespace(&self) -> bool {
        self.opts.show_whitespace
    }

    pub fn wraps(&self) -> bool {
        self.wrap
    }

    /// Rows of the file as built (header, notice and code rows), for tests and the status line.
    pub fn row_count(&self) -> usize {
        self.rows.lines.len()
    }

    pub fn focus_diff(&self, window: &mut Window, cx: &mut App) {
        self.focus.focus(window, cx);
    }
}

impl Focusable for DiffPane {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}

impl Render for DiffPane {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let skin = cx.global::<Skin>().clone();
        let (border, text, muted, accent, success, danger, hover) = (
            skin.border,
            skin.text,
            skin.muted,
            skin.accent,
            skin.success,
            skin.danger,
            skin.hover,
        );
        let file = self.files.get(self.selected);
        let (path, adds, dels) = file
            .map(|f| {
                let count = |o: LineOrigin| {
                    f.hunks
                        .iter()
                        .flat_map(|h| &h.lines)
                        .filter(|l| l.origin == o)
                        .count()
                };
                (
                    f.path.to_string_lossy().into_owned(),
                    count(LineOrigin::Add),
                    count(LineOrigin::Delete),
                )
            })
            .unwrap_or_default();
        let n = self.files.len();
        let btn = |id: &'static str, icon: AssetIcon| {
            div()
                .id(id)
                .p(px(4.0))
                .rounded(px(3.0))
                .cursor_pointer()
                .hover(move |d| d.bg(hover))
                .child(Icon::new(icon).size(px(14.0)).text_color(text))
        };
        let toggle = |id: &'static str, label: &'static str, on: bool| {
            div()
                .id(id)
                .px(px(8.0))
                .h(px(20.0))
                .flex()
                .items_center()
                .text_xs()
                .rounded(px(3.0))
                .cursor_pointer()
                .text_color(if on { accent } else { muted })
                .when(on, |d| d.bg(hover))
                .hover(move |d| d.bg(hover))
                .child(label)
        };

        let cw = char_width(window);
        let paint = Paint {
            rows: self.rows.clone(),
            colors: skin.diff,
            wrap: self.wrap,
            text_w: (self.rows.max_chars as f32 + 2.0) * cw,
            cw,
            cursor: self.cursor,
            selection: self.selection(),
            source: self.source,
            skin: skin.clone(),
            me: cx.entity().downgrade(),
        };
        // Inline: gutter (12) + text; Side by side: two halves of gutter (6) + text.
        let content_w = if self.wrap {
            0.0
        } else if self.opts.split {
            2.0 * (6.0 * cw + paint.text_w) + 1.0
        } else {
            12.0 * cw + paint.text_w
        };
        let body: AnyElement = if self.loading {
            v_flex()
                .size_full()
                .items_center()
                .justify_center()
                .child(Spinner::new())
                .into_any_element()
        } else {
            let items = list(self.list.clone(), move |ix, _w, _cx| paint.row(ix))
                .size_full()
                .into_any_element();
            let scroller = if self.wrap {
                div().size_full().child(items).into_any_element()
            } else {
                div()
                    .id("diff-x")
                    .size_full()
                    .overflow_x_scroll()
                    .track_scroll(&self.hscroll)
                    .child(div().h_full().w_full().min_w(px(content_w)).child(items))
                    .into_any_element()
            };
            div()
                .relative()
                .size_full()
                .child(scroller)
                .child(Scrollbar::vertical(&self.list))
                .when(!self.wrap, |d| {
                    d.child(Scrollbar::horizontal(&self.hscroll))
                })
                .into_any_element()
        };
        v_flex()
            .size_full()
            .key_context(CONTEXT)
            .track_focus(&self.focus)
            .on_action(cx.listener(|_, _: &CloseDiff, _, cx| cx.emit(DiffPaneEvent::Close)))
            .on_action(cx.listener(|this, _: &NextDiffFile, _, cx| this.step(1, cx)))
            .on_action(cx.listener(|this, _: &PrevDiffFile, _, cx| this.step(-1, cx)))
            .on_action(cx.listener(|this, _: &CursorUp, _, cx| this.move_cursor(-1, false, cx)))
            .on_action(cx.listener(|this, _: &CursorDown, _, cx| this.move_cursor(1, false, cx)))
            .on_action(cx.listener(|this, _: &ExtendUp, _, cx| this.move_cursor(-1, true, cx)))
            .on_action(cx.listener(|this, _: &ExtendDown, _, cx| this.move_cursor(1, true, cx)))
            .border_1()
            .border_color(border)
            .rounded(px(3.0))
            .child(
                h_flex()
                    .flex_none()
                    .min_h(px(30.0))
                    .px(px(8.0))
                    .gap(px(8.0))
                    .items_center()
                    .flex_wrap()
                    .border_b_1()
                    .border_color(border)
                    .child(
                        btn("back", AssetIcon::ArrowLeft)
                            .on_click(cx.listener(|_, _, _, cx| cx.emit(DiffPaneEvent::Close))),
                    )
                    .child(
                        div()
                            .font_weight(FontWeight::SEMIBOLD)
                            .truncate()
                            .min_w_0()
                            .child(path),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(success)
                            .child(format!("+{adds}")),
                    )
                    .child(div().text_xs().text_color(danger).child(format!("-{dels}")))
                    .child(div().flex_1())
                    .child(div().text_xs().text_color(muted).child(format!(
                        "{} / {}",
                        if n == 0 { 0 } else { self.selected + 1 },
                        n
                    )))
                    .child(
                        btn("prev", AssetIcon::ChevronLeft)
                            .on_click(cx.listener(|this, _, _, cx| this.step(-1, cx))),
                    )
                    .child(
                        btn("next", AssetIcon::ChevronRight)
                            .on_click(cx.listener(|this, _, _, cx| this.step(1, cx))),
                    )
                    .child(
                        toggle(
                            "mode",
                            if self.opts.split {
                                msg::diff_view::B_SPLIT.get()
                            } else {
                                msg::diff_view::B_INLINE.get()
                            },
                            true,
                        )
                        .on_click(cx.listener(|this, _, _, cx| this.toggle_layout(cx))),
                    )
                    .child(
                        toggle("ws", msg::diff_view::B_WS.get(), self.opts.show_whitespace)
                            .on_click(cx.listener(|this, _, _, cx| this.toggle_whitespace(cx))),
                    )
                    .child(
                        toggle("wrap", msg::diff_view::B_WRAP.get(), self.wrap)
                            .on_click(cx.listener(|this, _, _, cx| this.toggle_wrap(cx))),
                    )
                    .when(
                        self.source.is_working() && self.selection().is_some(),
                        |d| {
                            let staged = self.source == DiffSource::Staged;
                            d.child(
                                toggle(
                                    "lines-stage",
                                    if staged {
                                        msg_actions::LINES_UNSTAGE.get()
                                    } else {
                                        msg_actions::LINES_STAGE.get()
                                    },
                                    true,
                                )
                                .on_click(
                                    cx.listener(|this, _, _, cx| this.request_toggle_lines(cx)),
                                ),
                            )
                            .when(!staged, |d| {
                                d.child(
                                    toggle(
                                        "lines-discard",
                                        msg_actions::LINES_DISCARD.get(),
                                        false,
                                    )
                                    .text_color(danger)
                                    .on_click(cx.listener(
                                        |this, _, _, cx| this.request_lines(PartOp::Discard, cx),
                                    )),
                                )
                            })
                        },
                    ),
            )
            .child(div().flex_1().min_h_0().p(px(4.0)).child(body))
    }
}
