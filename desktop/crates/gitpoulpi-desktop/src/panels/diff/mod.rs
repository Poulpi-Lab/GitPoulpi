//! The centre pane when a file is open: GitPoulpi swaps the Log for the diff of the clicked file
//! (Esc goes back).
//!
//! What the rows are (which hunks the whitespace toggle keeps, line numbers, Split pairing, changed
//! words, the binary notice and the LFS card) is the shared `gitpoulpi_app::diff` model (`rows`); this
//! file is the desktop's own layout: a virtualised list (`gpui::list`, so a huge diff paints only the
//! rows in view), word wrap with the line numbers kept in their own column, or one line per row with a
//! horizontal scroll, and the theme's diff colours.

mod rows;

use std::ops::Range;
use std::sync::Arc;

use gitpoulpi_core::domain::{FileDiff, LineOrigin};
use gitpoulpi_i18n::msg;
use gpui_kit::assets::IconName as AssetIcon;
use gpui_kit::component::{h_flex, scroll::Scrollbar, spinner::Spinner, v_flex, Icon};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    actions, div, list, px, AnyElement, App, AppContext as _, Context, EventEmitter, FocusHandle,
    Focusable, FontStyle, FontWeight, HighlightStyle, Hsla, InteractiveElement as _, IntoElement,
    KeyBinding, ListAlignment, ListOffset, ListState, ParentElement as _, Render, ScrollHandle,
    SharedString, StatefulInteractiveElement as _, Styled as _, StyledText, Subscription, Task,
    TextRun, Window,
};
use rows::{body_look, Line, Look, Opts, Rows};

use crate::fonts;
use crate::theme::{DiffColors, Skin};

actions!(
    gp_diff,
    [
        CloseDiff,
        NextDiffFile,
        PrevDiffFile,
        NextHunk,
        PrevHunk,
        ToggleLayout,
        ToggleWrap,
        ToggleWhitespace
    ]
);

/// Key context of the pane. The letters are the terminal UI's own (`diff.next_hunk` `n`,
/// `diff.prev_hunk` `p`, `diff.toggle_layout` `m`, `diff.toggle_wrap` `w`, `diff.toggle_whitespace`
/// `s`, `diff.close` `q`).
const CONTEXT: &str = "GpDiffPane";

pub fn init(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("escape", CloseDiff, Some(CONTEXT)),
        KeyBinding::new("q", CloseDiff, Some(CONTEXT)),
        KeyBinding::new("]", NextDiffFile, Some(CONTEXT)),
        KeyBinding::new("[", PrevDiffFile, Some(CONTEXT)),
        KeyBinding::new("n", NextHunk, Some(CONTEXT)),
        KeyBinding::new("p", PrevHunk, Some(CONTEXT)),
        KeyBinding::new("m", ToggleLayout, Some(CONTEXT)),
        KeyBinding::new("w", ToggleWrap, Some(CONTEXT)),
        KeyBinding::new("s", ToggleWhitespace, Some(CONTEXT)),
    ]);
}

pub enum DiffPaneEvent {
    Close,
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
            focus: cx.focus_handle(),
            _task: None,
            _subs: Vec::new(),
        }
    }

    /// Shows file `ix` of `files`; the arrows and `[` / `]` move through the rest.
    pub fn show(&mut self, files: Arc<Vec<FileDiff>>, ix: usize, cx: &mut Context<Self>) {
        self.files = files;
        self.select(ix, cx);
    }

    fn select(&mut self, ix: usize, cx: &mut Context<Self>) {
        self.selected = ix.min(self.files.len().saturating_sub(1));
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

/// The width of one monospace character at the diff's size.
fn char_width(window: &Window) -> f32 {
    let mut font = window.text_style().font();
    font.family = fonts::MONO_FAMILY.into();
    let sample = "0000000000";
    let run = TextRun {
        len: sample.len(),
        font,
        color: Hsla::default(),
        background_color: None,
        underline: None,
        strikethrough: None,
    };
    window
        .text_system()
        .shape_line(sample.into(), px(TEXT_PX), &[run], None)
        .width
        .as_f32()
        / sample.len() as f32
}

/// A run of spans as one text with the looks laid over it.
fn text_of(spans: &[gitpoulpi_app::diff::StyledSpan], c: &DiffColors) -> StyledText {
    let mut text = String::new();
    let mut runs: Vec<(Range<usize>, HighlightStyle)> = Vec::new();
    for s in spans {
        if s.text.is_empty() {
            continue;
        }
        let start = text.len();
        text.push_str(&s.text);
        let look = Look::of(s).style(c);
        runs.push((
            start..text.len(),
            HighlightStyle {
                color: Some(look.fg),
                background_color: look.bg,
                font_weight: look.bold.then_some(FontWeight::BOLD),
                font_style: look.italic.then_some(FontStyle::Italic),
                ..HighlightStyle::default()
            },
        ));
    }
    StyledText::new(SharedString::from(text)).with_highlights(runs)
}

/// What the row closure of the list needs; owned so the closure outlives `render`.
struct Paint {
    rows: Arc<Rows>,
    colors: DiffColors,
    wrap: bool,
    /// Width of one text column in pixels when not wrapping.
    text_w: f32,
    cw: f32,
}

impl Paint {
    /// One half (or a whole Inline line): its gutter in a column of its own, then the text, which
    /// wraps under itself rather than under the line number.
    fn half(&self, line: &gitpoulpi_app::diff::StyledLine) -> gpui_kit::Div {
        let (gutter, rest) = match line.spans.split_first() {
            Some((g, rest)) if g.role == gitpoulpi_app::diff::DiffRole::Gutter => (Some(g), rest),
            _ => (None, line.spans.as_slice()),
        };
        let bg = body_look(line).and_then(|l| l.row_bg(&self.colors));
        let gutter_w = gutter.map(|g| g.text.chars().count() as f32 * self.cw);
        h_flex()
            .items_start()
            .min_h(px(LINE_H))
            .when_some(bg, |d, bg| d.bg(bg))
            .when_some(gutter.zip(gutter_w), |d, (g, w)| {
                d.child(
                    div()
                        .flex_none()
                        .w(px(w))
                        .child(text_of(std::slice::from_ref(g), &self.colors)),
                )
            })
            .child(if self.wrap {
                div().flex_1().min_w_0().child(text_of(rest, &self.colors))
            } else {
                div()
                    .flex_none()
                    .w(px(self.text_w))
                    .whitespace_nowrap()
                    .child(text_of(rest, &self.colors))
            })
    }

    fn row(&self, ix: usize) -> AnyElement {
        let base = div()
            .w_full()
            .font_family(fonts::MONO_FAMILY)
            .text_size(px(TEXT_PX))
            .line_height(px(LINE_H));
        match &self.rows.lines[ix] {
            Line::Full(line) => base.child(self.half(line)).into_any_element(),
            Line::Split(left, right) => {
                let blank = || div().flex_1().min_w_0().min_h(px(LINE_H));
                let side = |l: &Option<gitpoulpi_app::diff::StyledLine>| match l {
                    Some(l) => self.half(l).flex_1().min_w_0(),
                    None => blank(),
                };
                base.child(
                    h_flex()
                        .w_full()
                        .items_stretch()
                        .child(side(left))
                        .child(
                            div()
                                .w(px(1.0))
                                .flex_none()
                                .bg(self.colors.gutter_fg)
                                .opacity(0.4),
                        )
                        .child(side(right)),
                )
                .into_any_element()
            }
        }
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
            .on_action(cx.listener(|this, _: &NextHunk, _, cx| this.step_hunk(true, cx)))
            .on_action(cx.listener(|this, _: &PrevHunk, _, cx| this.step_hunk(false, cx)))
            .on_action(cx.listener(|this, _: &ToggleLayout, _, cx| this.toggle_layout(cx)))
            .on_action(cx.listener(|this, _: &ToggleWrap, _, cx| this.toggle_wrap(cx)))
            .on_action(cx.listener(|this, _: &ToggleWhitespace, _, cx| this.toggle_whitespace(cx)))
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
                    ),
            )
            .child(div().flex_1().min_h_0().p(px(4.0)).child(body))
    }
}
