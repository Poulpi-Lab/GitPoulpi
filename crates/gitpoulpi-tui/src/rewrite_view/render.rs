//! The workbench's paint walk and its mouse.
//!
//! One walk paints and registers: every rect in the returned hit list is the
//! one a glyph was drawn in (`where-we-are.md`, "Hit targets"). Specific
//! targets (a row's controls, a button) are registered before the bands that
//! contain them, and an open dropdown or question goes in front of
//! everything, so nothing behind it hovers or takes the click.
//!
//! The pointer reaches the same methods the keys do (`mod.rs`, `popup.rs`);
//! this file only maps rects onto them.

use std::cell::Cell;

use crate::wide_edge::Clear;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Block, BorderType, Borders, Widget};
use ratatui::Frame;

use gitpoulpi_i18n::msg::rewrite_view as t;

use super::{
    EditTarget, Field, FieldValue, Focus, Popup, PopupRow, PreviewTab, RewriteKeyOutcome,
    RewriteView, RuleRow, Run,
};
use crate::dialog_kit::{self, button, button_width, ButtonState};
use crate::text_field::{edit_line, edit_lines, EditLook};
use crate::theme::plain_button_style;
use crate::{display_width, th};

/// The clickable parts of the workbench.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RewriteHit {
    /// The close cross on the frame.
    Close,
    /// Empty room of a pane: focuses it.
    Pane(Focus),
    /// The scope dropdown.
    ScopeCombo,
    TagsCheck,
    RefsButton,
    SinceCheck,
    /// The recipe's name.
    RecipeName,
    /// Rule `.0`: select it.
    Rule(usize),
    RuleCheck(usize),
    RuleUp(usize),
    RuleDown(usize),
    RuleDelete(usize),
    AddRule,
    RestoreRule,
    /// Field `.0` of the detail pane.
    Field(usize),
    Tab(PreviewTab),
    FilterCombo,
    /// Row `.0` of the Commits tab.
    Row(usize),
    /// Protected branch `.0` of the Refs tab.
    ProtectedRow(usize),
    ProtectedAll,
    ProtectedNone,
    CheckTools,
    Preview,
    Apply,
    Cancel,
    Save,
    Load,
    Undo,
    ResultClose,
    /// The result panel: ref `.0` of the push list, the protected-branch
    /// check all / uncheck all, the push, the note, the commit map, the
    /// "Fetch and review" of rejected row `.0`, and map path `.0`.
    PushRow(usize),
    PushAll,
    PushNone,
    PushSend,
    CopyNote,
    OpenMaps,
    FetchReview(usize),
    MapCopy(usize),
    /// S21: "Push LFS objects" (result panel) and "Fetch LFS objects" (the
    /// refusal that lists the missing ones).
    LfsPush,
    LfsFetch,
    /// The Secrets tab: scan, finding `.0`, remove, check all, uncheck all.
    SecretsScan,
    SecretRow(usize),
    SecretsRemove,
    SecretsAll,
    SecretsNone,
    /// The Identities tab (S16): row `.0`, Merge into..., Refresh, check all,
    /// uncheck all, Undo, and the Summary's "Use .mailmap" offer.
    IdentRow(usize),
    IdentMerge,
    IdentRefresh,
    IdentAll,
    IdentNone,
    IdentUndo,
    IdentOffer,
    /// The "Merge into..." dialog: candidate `.0`, its cross, the panel (a
    /// click there does nothing) and the rest of the workbench (leaves it).
    IdentPick(usize),
    IdentPickCancel,
    IdentPickBody,
    IdentPickShield,
    /// The Size tab: run, row `.0`, remove paths, remove by size, check all,
    /// uncheck all, list `.0` and order `.0`.
    SizeRun,
    SizeRow(usize),
    SizeRemovePaths,
    SizeRemoveOver,
    SizeAll,
    SizeNone,
    SizeView(u8),
    SizeSort(u8),
    /// Row `.0` of the open dropdown.
    PopupRow(usize),
    /// Everything else while a dropdown is open: closes it.
    PopupShield,
    /// The blob diff overlay: its cross, file `.0`, the diff, and the rest
    /// of the workbench (a click there closes it).
    DrillClose,
    DrillPath(usize),
    DrillDiff,
    DrillShield,
    /// S23, the host's how-to: the button that opens it, its cross, link
    /// `.0`, Copy, Open, the panel itself and the rest of the workbench.
    HostHowto,
    HostClose,
    HostLink(usize),
    HostCopy,
    HostOpen,
    HostPanel,
    HostShield,
    /// The script editor overlay: its cross, the text area, parameter `.0`,
    /// the file path, the Load / Save / Test / Cancel buttons, line `.0` of
    /// the result table, the panel itself (a click there does nothing) and
    /// the rest of the workbench (a click there closes it).
    ScriptClose,
    ScriptEditor,
    ScriptParam(usize),
    ScriptPath,
    ScriptLoad,
    ScriptSave,
    ScriptTest,
    ScriptCancel,
    ScriptRow(usize),
    ScriptPanel,
    ScriptShield,
    /// The purge screen (S15): the button that opens it, its cross, its
    /// Close, the rest of the workbench (a click there closes it), the
    /// panel (a click there does nothing), the confirmation's cross and
    /// option `.0`, blocker `.0` with its fix and its copy, host `.0`'s
    /// link, the Purge and Check again buttons.
    PurgeOpen,
    PurgeClose,
    PurgeDone,
    PurgeShield,
    PurgeBody,
    PurgeCancel,
    PurgeAnswer(usize),
    PurgeRow(usize),
    PurgeFix(usize),
    PurgeCopy(usize),
    PurgeLink(usize),
    PurgeRun,
    PurgeCheck,
    /// The ✦ Write with AI button of the script editor, and its dialog: the
    /// request field, the sample box, the "what is sent" pane, Send,
    /// Cancel, the cross, the panel and the rest of the workbench.
    ScriptAi,
    AiText,
    AiSample,
    AiSent,
    AiSend,
    AiCancel,
    AiClose,
    AiPanel,
    AiShield,
    /// The ✦ Create Rules with AI button of the workbench, and its dialog:
    /// the description, the context box, the replace box, the "what is sent"
    /// pane, Send, Cancel (Close on the result list), the cross, the panel
    /// and the rest of the workbench.
    RulesAi,
    RaText,
    RaContext,
    RaReplace,
    RaSent,
    /// The "What is sent" header (shows or hides the pane) and the two
    /// scrollbars of the dialog: the description's and the pane's.
    RaSentHead,
    RaSentBar,
    RaTextBar,
    RaSend,
    RaCancel,
    RaClose,
    RaPanel,
    RaShield,
    /// Option `.0` of the open question.
    Answer(usize),
    /// The question's cross.
    QuestionCancel,
    /// Everything else while a question is open: leaves it unanswered.
    QuestionShield,
}

impl RewriteHit {
    /// Whether `Ctrl+N` gives this target a badge: not the empty room of a
    /// pane, a commit row (a list of its own), or what only shields.
    pub(crate) fn badged(self) -> bool {
        !matches!(
            self,
            RewriteHit::Pane(_)
                | RewriteHit::Row(_)
                | RewriteHit::QuestionShield
                | RewriteHit::PopupShield
                | RewriteHit::PopupRow(_)
                | RewriteHit::IdentPickBody
                | RewriteHit::IdentPickShield
                | RewriteHit::DrillShield
                | RewriteHit::HostPanel
                | RewriteHit::HostShield
                | RewriteHit::ScriptPanel
                | RewriteHit::ScriptShield
                | RewriteHit::PurgeShield
                | RewriteHit::PurgeBody
                | RewriteHit::AiPanel
                | RewriteHit::AiShield
                | RewriteHit::RaPanel
                | RewriteHit::RaShield
                | RewriteHit::RaSentBar
                | RewriteHit::RaTextBar
        )
    }

    /// A part of the ✦ Create Rules dialog (its panel included).
    pub(crate) fn is_rules_ai(self) -> bool {
        matches!(
            self,
            RewriteHit::RaText
                | RewriteHit::RaTextBar
                | RewriteHit::RaContext
                | RewriteHit::RaReplace
                | RewriteHit::RaSent
                | RewriteHit::RaSentHead
                | RewriteHit::RaSentBar
                | RewriteHit::RaSend
                | RewriteHit::RaCancel
                | RewriteHit::RaClose
                | RewriteHit::RaPanel
        )
    }

    /// A part of the purge screen (the wheel scrolls its body).
    pub(crate) fn is_purge(self) -> bool {
        matches!(
            self,
            RewriteHit::PurgeClose
                | RewriteHit::PurgeDone
                | RewriteHit::PurgeBody
                | RewriteHit::PurgeRow(_)
                | RewriteHit::PurgeFix(_)
                | RewriteHit::PurgeCopy(_)
                | RewriteHit::PurgeLink(_)
        )
    }
}

/// What the last paint learned, for the next key or click. Interior
/// mutability: the workbench is drawn from a shared reference.
#[derive(Default)]
pub(super) struct Paint {
    /// The frame as last painted (a click outside is a dismissal).
    pub frame: Cell<Rect>,
    /// First rule and first row shown.
    pub rules_top: Cell<usize>,
    pub rows_top: Cell<usize>,
    pub rows_visible: Cell<usize>,
    pub drill_rows: Cell<usize>,
    /// The script editor's text area, its file path field and the room for
    /// result lines.
    pub script_text: Cell<Rect>,
    pub script_path: Cell<Rect>,
    pub script_rows: Cell<usize>,
    pub detail_top: Cell<usize>,
    /// First finding of the Secrets tab and first row of the push list shown.
    pub secrets_top: Cell<usize>,
    pub size_top: Cell<usize>,
    pub push_top: Cell<usize>,
    pub flash: Cell<bool>,
}

/// Where a dropdown hangs from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Anchor {
    Scope,
    Add,
    Field(usize),
    Save,
    Load,
    Filter,
    Refs,
}

/// A paint pass: the buffer, the pointer, and the targets found so far.
pub(super) struct Ui<'a> {
    pub buf: &'a mut Buffer,
    pub hovered: Option<RewriteHit>,
    /// Specific targets.
    pub fine: Vec<(Rect, RewriteHit)>,
    /// The bands that contain them.
    pub coarse: Vec<(Rect, RewriteHit)>,
    pub anchors: Vec<(Anchor, Rect)>,
}

impl Ui<'_> {
    pub(super) fn hot(&self, hit: RewriteHit) -> bool {
        self.hovered == Some(hit)
    }

    /// A button; its target is registered even when disabled, so a click on
    /// it says why nothing happened.
    pub(super) fn button(
        &mut self,
        x: u16,
        y: u16,
        label: &str,
        hit: RewriteHit,
        primary: bool,
        enabled: bool,
    ) -> u16 {
        let hovered = self.hot(hit);
        let state = if !enabled {
            ButtonState::Disabled
        } else if primary {
            ButtonState::Primary {
                hovered,
                focused: false,
            }
        } else {
            ButtonState::Normal {
                hovered,
                focused: false,
            }
        };
        let rect = button(self.buf, x, y, label, state);
        self.fine.push((rect, hit));
        rect.width
    }

    /// A dropdown field: `value  v`, returns its rect.
    pub(super) fn combo(
        &mut self,
        rect: Rect,
        value: &str,
        hit: RewriteHit,
        anchor: Anchor,
        focused: bool,
    ) {
        let bg = crate::input_bg(focused, self.hot(hit));
        dialog_kit::fill(self.buf, rect, Style::default().bg(bg));
        let text_w = rect.width.saturating_sub(3);
        dialog_kit::put(
            self.buf,
            rect.x + 1,
            rect.y,
            value,
            text_w,
            Style::default().fg(th().ui.text).bg(bg),
        );
        dialog_kit::put(
            self.buf,
            rect.right().saturating_sub(2),
            rect.y,
            "\u{25BE}",
            1,
            Style::default()
                .fg(if focused {
                    th().ui.accent
                } else {
                    th().ui.muted
                })
                .bg(bg),
        );
        self.fine.push((rect, hit));
        self.anchors.push((anchor, rect));
    }

    pub(super) fn text(&mut self, x: u16, y: u16, max: u16, text: &str, style: Style) -> u16 {
        dialog_kit::put(self.buf, x, y, text, max, style).width
    }
}

pub(super) fn dim() -> Style {
    Style::default().fg(th().ui.muted).bg(th().ui.panel)
}

pub(super) fn plain() -> Style {
    Style::default().fg(th().ui.text).bg(th().ui.panel)
}

pub(super) fn warn() -> Style {
    Style::default().fg(th().status.warning).bg(th().ui.panel)
}

pub(super) fn bad() -> Style {
    Style::default().fg(th().status.danger).bg(th().ui.panel)
}

pub(super) fn bad_bold() -> Style {
    bad().add_modifier(Modifier::BOLD)
}

pub(super) fn bold(accent: bool) -> Style {
    Style::default()
        .fg(if accent { th().ui.accent } else { th().ui.text })
        .bg(th().ui.panel)
        .add_modifier(Modifier::BOLD)
}

/// A band with the selection or hover tint, for a row.
pub(super) fn row_bg(selected: bool, hover: bool) -> ratatui::style::Color {
    if selected {
        th().ui.selection
    } else if hover {
        th().ui.hover
    } else {
        th().ui.panel
    }
}

/// A titled pane box; the border follows the focus.
pub(super) fn pane(buf: &mut Buffer, area: Rect, title: &str, focused: bool) -> Rect {
    Block::default()
        .title(format!(" {title} "))
        .title_style(bold(focused))
        .border_type(BorderType::Rounded)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(if focused {
            th().ui.accent
        } else {
            th().ui.muted
        }))
        .style(Style::default().bg(th().ui.panel))
        .render(area, buf);
    Rect::new(
        area.x + 1,
        area.y + 1,
        area.width.saturating_sub(2),
        area.height.saturating_sub(2),
    )
}

/// `text` cut into lines of at most `width` columns, at spaces.
pub(super) fn wrap(text: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut line = String::new();
    for word in text.split_whitespace() {
        let need = display_width(&line) + display_width(word) + usize::from(!line.is_empty());
        if need > width && !line.is_empty() {
            lines.push(std::mem::take(&mut line));
        }
        if !line.is_empty() {
            line.push(' ');
        }
        line.push_str(word);
    }
    if !line.is_empty() {
        lines.push(line);
    }
    lines
}

impl RewriteView {
    /// Draws the workbench over `area` and returns its click targets, from
    /// the walk that painted them.
    pub(crate) fn render(
        &self,
        frame: &mut Frame,
        area: Rect,
        hovered: Option<RewriteHit>,
    ) -> Vec<(Rect, RewriteHit)> {
        let area = area.intersection(frame.buffer_mut().area);
        self.paint.frame.set(area);
        let mut ui = Ui {
            buf: frame.buffer_mut(),
            hovered,
            fine: Vec::new(),
            coarse: Vec::new(),
            anchors: Vec::new(),
        };
        dialog_kit::panel(ui.buf, area, t::TITLE.get());
        if let Some(cross) =
            crate::chrome::close_cross::paint(ui.buf, area, ui.hot(RewriteHit::Close))
        {
            if self.paint.flash.take() {
                crate::chrome::close_cross::flash(ui.buf, area);
            }
            ui.fine.push((cross, RewriteHit::Close));
        }
        let inner = Rect::new(
            area.x + 1,
            area.y + 1,
            area.width.saturating_sub(2),
            area.height.saturating_sub(2),
        );
        if inner.width < 56 || inner.height < 15 {
            ui.text(
                inner.x + 1,
                inner.y,
                inner.width.saturating_sub(2),
                t::TOO_SMALL.get(),
                dim(),
            );
            return ui.fine;
        }
        let content = Rect::new(inner.x + 1, inner.y, inner.width - 2, inner.height);
        let top_h: u16 = if content.height >= 32 {
            12
        } else if content.height >= 24 {
            9
        } else {
            7
        };
        let scope_bar = Rect::new(content.x, content.y, content.width, 1);
        let top = Rect::new(content.x, content.y + 1, content.width, top_h);
        let bottom_h = 2;
        let preview = Rect::new(
            content.x,
            top.bottom(),
            content.width,
            content.bottom().saturating_sub(top.bottom() + bottom_h),
        );
        let notice = Rect::new(content.x, content.bottom() - 2, content.width, 1);
        let actions = Rect::new(content.x, content.bottom() - 1, content.width, 1);

        self.paint_scope_bar(&mut ui, scope_bar);
        let rules_w = (content.width * 38 / 100).clamp(26, 48);
        let rules = Rect::new(top.x, top.y, rules_w, top.height);
        let detail = Rect::new(
            rules.right() + 1,
            top.y,
            content.width.saturating_sub(rules_w + 1),
            top.height,
        );
        self.paint_rules(&mut ui, rules);
        self.paint_detail(&mut ui, detail);
        self.paint_preview(&mut ui, preview);
        self.paint_notice(&mut ui, notice);
        self.paint_actions(&mut ui, actions);

        let Ui {
            buf,
            fine,
            coarse,
            anchors,
            hovered,
        } = ui;
        let mut hits = fine;
        hits.extend(coarse);
        // The dropdown or the question goes in front of everything.
        let mut front: Vec<(Rect, RewriteHit)> = Vec::new();
        if self.has_target_picker() {
            self.paint_target_picker(buf, area);
        } else if self.purge.is_some() {
            front = self.paint_purge(buf, area, hovered);
        } else if self.identities.pick.is_some() {
            front = self.paint_identity_pick(buf, area, hovered);
        } else if self.question.is_some() {
            front = self.paint_question(buf, area, hovered);
        } else if self.rules_ai_open() {
            front = self.paint_rules_ai(buf, area, hovered);
        } else if self.script.ai.dialog.is_some() {
            front = self.paint_ai(buf, area, hovered);
        } else if self.script.editor.is_some() {
            front = self.paint_script(buf, area, hovered);
        } else if self.popup.is_some() {
            front = self.paint_popup(buf, area, &anchors, hovered);
        } else if self.drill.is_some() {
            front = self.paint_drill(buf, area, hovered);
        } else if self.host.howto.is_some() {
            front = self.paint_howto(buf, area, hovered);
        }
        front.extend(hits);
        front
    }

    // ----------------------------------------------------------- scope bar

    fn paint_scope_bar(&self, ui: &mut Ui<'_>, bar: Rect) {
        let focused = self.focus == Focus::Scope;
        let mut x = bar.x;
        let label = t::SCOPE_LABEL.get();
        x += ui.text(x, bar.y, bar.width, label, bold(focused)) + 1;
        let kind = self.scope_kind().label().get();
        let w = (display_width(kind) as u16 + 5).clamp(18, 32);
        ui.combo(
            Rect::new(x, bar.y, w, 1),
            kind,
            RewriteHit::ScopeCombo,
            Anchor::Scope,
            focused,
        );
        x += w + 2;
        if self.scope_kind() == super::ScopeKind::Refs {
            let wb = button_width(t::CHOOSE_REFS.get());
            if x + wb < bar.right() {
                x += ui.button(
                    x,
                    bar.y,
                    t::CHOOSE_REFS.get(),
                    RewriteHit::RefsButton,
                    false,
                    true,
                ) + 2;
            }
        }
        if let Some((id, on)) = self.since_offer() {
            let hex = id.to_hex();
            let text = t::since_check(&hex[..hex.len().min(7)]);
            let r = dialog_kit::checkbox(
                ui.buf,
                x,
                bar.y,
                bar.right().saturating_sub(x),
                &text,
                on,
                dialog_kit::CheckState::Enabled {
                    hovered: ui.hot(RewriteHit::SinceCheck),
                    focused: false,
                },
            );
            ui.fine.push((r, RewriteHit::SinceCheck));
            x += r.width + 2;
        }
        let tags_on = self.recipe.scope_options.include_tags;
        let r = dialog_kit::checkbox(
            ui.buf,
            x,
            bar.y,
            bar.right().saturating_sub(x),
            t::TAGS_CHECK.get(),
            tags_on,
            dialog_kit::CheckState::Enabled {
                hovered: ui.hot(RewriteHit::TagsCheck),
                focused: false,
            },
        );
        ui.fine.push((r, RewriteHit::TagsCheck));
        x += r.width + 2;
        if let Some(c) = self.scope_counts() {
            let text = format!(
                "\u{b7} {}, {}, {}",
                t::counts_branches(u64::from(c.branches)),
                t::counts_tags(u64::from(c.tags)),
                t::counts_commits(c.commits)
            );
            ui.text(x, bar.y, bar.right().saturating_sub(x), &text, dim());
        }
        ui.coarse.push((bar, RewriteHit::Pane(Focus::Scope)));
    }

    // --------------------------------------------------------------- rules

    fn paint_rules(&self, ui: &mut Ui<'_>, area: Rect) {
        let focused = self.focus == Focus::Rules;
        let inner = pane(ui.buf, area, t::RULES_TITLE.get(), focused);
        ui.coarse.push((area, RewriteHit::Pane(Focus::Rules)));
        if inner.height < 3 || inner.width < 12 {
            return;
        }
        let rows = self.rule_rows();
        let list_h = (inner.height - 2) as usize;
        let mut top = self.paint.rules_top.get();
        let cursor = self.rules.cursor;
        if cursor < top {
            top = cursor;
        } else if cursor >= top + list_h {
            top = cursor + 1 - list_h;
        }
        top = top.min(rows.len().saturating_sub(list_h));
        self.paint.rules_top.set(top);

        if rows.is_empty() {
            let lines = wrap(t::EMPTY_RULES.get(), inner.width as usize - 1);
            for (i, line) in lines.iter().take(list_h).enumerate() {
                ui.text(
                    inner.x + 1,
                    inner.y + i as u16,
                    inner.width - 1,
                    line,
                    dim(),
                );
            }
        }
        for (slot, row) in rows.iter().enumerate().skip(top).take(list_h) {
            let y = inner.y + (slot - top) as u16;
            let band = Rect::new(inner.x, y, inner.width, 1);
            let selected = slot == cursor;
            let hover = matches!(
                ui.hovered,
                Some(
                    RewriteHit::Rule(h) | RewriteHit::RuleCheck(h) | RewriteHit::RuleUp(h)
                        | RewriteHit::RuleDown(h) | RewriteHit::RuleDelete(h)
                ) if h == slot
            );
            let bg = row_bg(selected && focused, hover);
            dialog_kit::fill(ui.buf, band, Style::default().bg(bg));
            let missing = self.missing_tools(slot);
            let greyed = !missing.is_empty();
            let mut right = band.right();
            if selected {
                let n = rows.len();
                let controls = [
                    (" \u{25B2} ", RewriteHit::RuleUp(slot), slot > 0),
                    (" \u{25BC} ", RewriteHit::RuleDown(slot), slot + 1 < n),
                    (" \u{2715} ", RewriteHit::RuleDelete(slot), true),
                ];
                for (label, hit, enabled) in controls.into_iter().rev() {
                    let cw = display_width(label) as u16;
                    if right < band.x + 14 + cw {
                        break;
                    }
                    right -= cw;
                    let hot = ui.hot(hit);
                    let style = if enabled {
                        plain_button_style(hot).bg(if hot { th().ui.button_hover } else { bg })
                    } else {
                        Style::default().fg(th().ui.disabled).bg(bg)
                    };
                    dialog_kit::put(ui.buf, right, y, label, cw, style);
                    if enabled {
                        ui.fine.push((Rect::new(right, y, cw, 1), hit));
                    }
                }
            }
            let check = if row.enabled { "[x]" } else { "[ ]" };
            let base = if greyed {
                Style::default().fg(th().ui.disabled).bg(bg)
            } else {
                Style::default().fg(th().ui.text).bg(bg)
            };
            let cw = ui.text(band.x + 1, y, 3, check, base);
            ui.fine.push((
                Rect::new(band.x + 1, y, cw + 1, 1),
                RewriteHit::RuleCheck(slot),
            ));
            let mut x = band.x + 5;
            let head = format!("{} ", row.number);
            x += ui.text(x, y, right.saturating_sub(x), &head, base);
            let name = row_name(row);
            let name_w = ui.text(x, y, right.saturating_sub(x), &name, base);
            x += name_w + 2;
            if x + 3 < right {
                let (note, style) = if greyed {
                    (
                        t::needs_tool(missing[0].tool.name()),
                        Style::default().fg(th().status.warning).bg(bg),
                    )
                } else {
                    (
                        row.detail.clone(),
                        Style::default().fg(th().ui.muted).bg(bg),
                    )
                };
                ui.text(x, y, right.saturating_sub(x + 1), &note, style);
            }
            ui.coarse.push((band, RewriteHit::Rule(slot)));
        }

        // The Add row, then the recipe's name.
        let add_y = inner.bottom() - 2;
        let label = t::ADD_RULE.get();
        let w = (display_width(label) as u16 + 4).min(inner.width);
        let add_rect = Rect::new(inner.x, add_y, w, 1);
        let hot = ui.hot(RewriteHit::AddRule);
        let bg = row_bg(false, hot);
        dialog_kit::fill(ui.buf, add_rect, Style::default().bg(bg));
        ui.text(
            add_rect.x + 1,
            add_y,
            w - 1,
            &format!("{label} \u{25BE}"),
            Style::default().fg(th().ui.accent).bg(bg),
        );
        ui.fine.push((add_rect, RewriteHit::AddRule));
        ui.anchors.push((Anchor::Add, add_rect));
        if self.rules.can_restore() {
            let rl = t::RESTORE_RULE.get();
            let rw = display_width(rl) as u16 + 2;
            let x = add_rect.right() + 1;
            if x + rw <= inner.right() {
                let hot = ui.hot(RewriteHit::RestoreRule);
                ui.text(
                    x,
                    add_y,
                    rw,
                    &format!(" {rl}"),
                    plain_button_style(hot).bg(if hot {
                        th().ui.button_hover
                    } else {
                        th().ui.panel
                    }),
                );
                ui.fine
                    .push((Rect::new(x, add_y, rw, 1), RewriteHit::RestoreRule));
            }
        }
        let name_y = inner.bottom() - 1;
        let head = t::RECIPE.get();
        let hw = display_width(head) as u16 + 1;
        ui.text(inner.x, name_y, hw, head, dim());
        let field = Rect::new(
            inner.x + hw + 1,
            name_y,
            inner.width.saturating_sub(hw + 1),
            1,
        );
        if field.width > 3 {
            let editing = matches!(
                self.editing.as_ref().map(|e| e.target),
                Some(EditTarget::Name)
            );
            let hot = ui.hot(RewriteHit::RecipeName);
            let bg = crate::input_bg(editing, hot);
            dialog_kit::fill(ui.buf, field, Style::default().bg(bg));
            if let (true, Some(ed)) = (editing, self.editing.as_ref()) {
                let line = edit_line(
                    &ed.text,
                    Some(&ed.caret),
                    field.width as usize - 1,
                    true,
                    EditLook::new(Style::default().fg(th().ui.text).bg(bg), "\u{2588}"),
                );
                ui.buf
                    .set_line(field.x + 1, field.y, &line, field.width - 1);
            } else if self.recipe.name.is_empty() {
                ui.text(
                    field.x + 1,
                    name_y,
                    field.width - 1,
                    t::RECIPE_UNNAMED.get(),
                    Style::default().fg(th().ui.muted).bg(bg),
                );
            } else {
                ui.text(
                    field.x + 1,
                    name_y,
                    field.width - 1,
                    &self.recipe.name,
                    Style::default().fg(th().ui.text).bg(bg),
                );
            }
            ui.fine.push((field, RewriteHit::RecipeName));
        }
    }

    // -------------------------------------------------------------- detail

    fn paint_detail(&self, ui: &mut Ui<'_>, area: Rect) {
        let focused = self.focus == Focus::Detail;
        let title = self
            .rule_rows()
            .get(self.rules.cursor)
            .map(row_name)
            .unwrap_or_else(|| t::DETAIL_TITLE.get().to_string());
        let inner = pane(ui.buf, area, &title, focused);
        ui.coarse.push((area, RewriteHit::Pane(Focus::Detail)));
        if inner.height < 2 || inner.width < 20 {
            return;
        }
        if self.recipe.rules.is_empty() {
            ui.text(
                inner.x + 1,
                inner.y,
                inner.width - 1,
                t::DETAIL_NONE.get(),
                dim(),
            );
            return;
        }
        let mut y = inner.y;
        let missing = self.missing_tools(self.rules.cursor);
        if let Some(first) = missing.first() {
            let again = t::CHECK_AGAIN.get();
            let aw = button_width(again);
            let text = t::tool_missing(first.tool.name(), first.hint.get());
            ui.text(
                inner.x + 1,
                y,
                inner.width.saturating_sub(aw + 3),
                &text,
                warn(),
            );
            if inner.width > aw + 6 {
                ui.button(
                    inner.right() - aw - 1,
                    y,
                    again,
                    RewriteHit::CheckTools,
                    false,
                    true,
                );
            }
            y += 1;
        }
        let fields = &self.detail.fields;
        let label_w = fields
            .iter()
            .map(|f| display_width(f.title()))
            .max()
            .unwrap_or(8)
            .clamp(8, 22) as u16;
        let value_x = inner.x + 1 + label_w + 2;
        let value_w = inner.right().saturating_sub(value_x + 1);
        if value_w < 6 {
            return;
        }
        // The window follows the cursor.
        let rows_room = inner.bottom().saturating_sub(y) as usize;
        let heights: Vec<usize> = fields
            .iter()
            .enumerate()
            .map(|(i, f)| self.field_height(i, f, rows_room))
            .collect();
        let mut top = self
            .paint
            .detail_top
            .get()
            .min(fields.len().saturating_sub(1));
        if self.detail.cursor < top {
            top = self.detail.cursor;
        }
        while top < self.detail.cursor
            && heights[top..=self.detail.cursor].iter().sum::<usize>() > rows_room
        {
            top += 1;
        }
        self.paint.detail_top.set(top);
        let mut all_shown = true;
        for (i, field) in fields.iter().enumerate().skip(top) {
            let h = heights[i] as u16;
            if y + h > inner.bottom() {
                all_shown = false;
                break;
            }
            self.paint_field(ui, i, field, y, h, inner, label_w, value_x, value_w);
            y += h;
        }
        // A script's check: OK, or the first problem with its line.
        if all_shown && self.shows_check() && y < inner.bottom() {
            if let Some((line, is_bad)) =
                super::script::detail_check_line(self, inner.width as usize - 2)
            {
                ui.text(
                    inner.x + 1,
                    y,
                    inner.width - 1,
                    &line,
                    if is_bad { bad() } else { dim() },
                );
                y += 1;
            }
        }
        // What the kind does that its fields do not say, if there is room.
        if let (true, Some(rule)) = (all_shown, self.recipe.rules.get(self.rules.cursor)) {
            for note in super::rules_form::notes(rule) {
                for line in wrap(note.get(), inner.width as usize - 2) {
                    if y >= inner.bottom() {
                        return;
                    }
                    ui.text(inner.x + 1, y, inner.width - 1, &line, dim());
                    y += 1;
                }
            }
        }
    }

    fn field_height(&self, index: usize, field: &Field, room: usize) -> usize {
        let editing = matches!(
            self.editing.as_ref().map(|e| e.target),
            Some(EditTarget::Field(f)) if f == index
        );
        if field.multiline && editing {
            4.min(room.max(1))
        } else if field.error.is_some() && room >= 2 {
            // The reason sits right under the value.
            2
        } else {
            1
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn paint_field(
        &self,
        ui: &mut Ui<'_>,
        index: usize,
        field: &Field,
        y: u16,
        h: u16,
        inner: Rect,
        label_w: u16,
        value_x: u16,
        value_w: u16,
    ) {
        let selected = index == self.detail.cursor && self.focus == Focus::Detail;
        let hit = RewriteHit::Field(index);
        let hot = ui.hot(hit);
        let line_bg = row_bg(selected, false);
        dialog_kit::fill(
            ui.buf,
            Rect::new(inner.x, y, inner.width, 1),
            Style::default().bg(line_bg),
        );
        let label_style = if field.error.is_some() {
            Style::default().fg(th().status.danger).bg(line_bg)
        } else {
            Style::default().fg(th().ui.text).bg(line_bg)
        };
        ui.text(inner.x + 1, y, label_w, field.title(), label_style);
        let error_row = field.error.is_some() && h >= 2 && !(field.multiline && h > 2);
        let value = Rect::new(value_x, y, value_w, if error_row { 1 } else { h });
        let editing = self
            .editing
            .as_ref()
            .filter(|e| e.target == EditTarget::Field(index));
        match &field.value {
            FieldValue::Choice { options, index: at } => {
                let text = options.get(*at).map(|m| m.get()).unwrap_or("");
                let w = (display_width(text) as u16 + 5).clamp(14, value_w);
                ui.combo(
                    Rect::new(value_x, y, w, 1),
                    text,
                    hit,
                    Anchor::Field(index),
                    selected,
                );
                self.paint_hint(ui, field, value_x + w + 1, y, inner.right(), line_bg);
            }
            FieldValue::Toggle(on) => {
                let r = dialog_kit::checkbox(
                    ui.buf,
                    value_x,
                    y,
                    value_w,
                    "",
                    *on,
                    dialog_kit::CheckState::Enabled {
                        hovered: hot,
                        focused: selected,
                    },
                );
                ui.fine.push((Rect::new(r.x, r.y, r.width.max(3), 1), hit));
            }
            FieldValue::Text(text) => {
                let bg = crate::input_bg(editing.is_some(), hot);
                dialog_kit::fill(ui.buf, value, Style::default().bg(bg));
                let style = Style::default().fg(th().ui.text).bg(bg);
                if let Some(ed) = editing {
                    let look = EditLook::new(style, "\u{2588}");
                    let (shown, caret) = if field.secret {
                        ed.caret.masked(&ed.text, '\u{2022}')
                    } else {
                        (ed.text.clone(), ed.caret.clone())
                    };
                    if field.multiline {
                        let lines = edit_lines(
                            &shown,
                            Some(&caret),
                            value_w as usize - 1,
                            h as usize,
                            true,
                            look,
                        );
                        for (i, line) in lines.iter().enumerate() {
                            ui.buf
                                .set_line(value_x + 1, y + i as u16, line, value_w - 1);
                        }
                    } else {
                        let line =
                            edit_line(&shown, Some(&caret), value_w as usize - 1, true, look);
                        ui.buf.set_line(value_x + 1, y, &line, value_w - 1);
                    }
                } else if text.is_empty() {
                    if let Some(hint) = field.hint {
                        ui.text(
                            value_x + 1,
                            y,
                            value_w - 1,
                            hint.get(),
                            Style::default().fg(th().ui.muted).bg(bg),
                        );
                    }
                } else {
                    let shown = if field.secret {
                        t::secret_values(
                            text.lines().filter(|l| !l.trim().is_empty()).count() as u64
                        )
                    } else if field.multiline {
                        first_line(text)
                    } else {
                        text.clone()
                    };
                    ui.text(value_x + 1, y, value_w - 1, &shown, style);
                }
                ui.fine.push((value, hit));
            }
        }
        if let (true, Some(why)) = (error_row, field.error.as_ref()) {
            let bg = th().ui.panel;
            dialog_kit::fill(
                ui.buf,
                Rect::new(inner.x, y + 1, inner.width, 1),
                Style::default().bg(bg),
            );
            ui.text(value_x, y + 1, value_w, why, bad());
        }
        ui.coarse.push((Rect::new(inner.x, y, inner.width, 1), hit));
    }

    fn paint_hint(
        &self,
        ui: &mut Ui<'_>,
        field: &Field,
        x: u16,
        y: u16,
        right: u16,
        bg: ratatui::style::Color,
    ) {
        if let Some(hint) = field.hint {
            if x + 8 < right {
                ui.text(
                    x,
                    y,
                    right - x - 1,
                    hint.get(),
                    Style::default().fg(th().ui.muted).bg(bg),
                );
            }
        }
    }

    // ------------------------------------------------------ notice, actions

    fn paint_notice(&self, ui: &mut Ui<'_>, row: Rect) {
        let (text, style) = if let Some(n) = &self.notice {
            (n.text.clone(), if n.error { bad() } else { plain() })
        } else if let Some(e) = self.detail.current().and_then(|f| f.error.clone()) {
            (e, bad())
        } else if let Some(m) = self.missing_tools(self.rules.cursor).first() {
            (t::tool_missing(m.tool.name(), m.hint.get()), warn())
        } else if let Some(why) = self
            .apply_blocker()
            .filter(|_| !matches!(self.run, Run::Done(_)))
        {
            (why, dim())
        } else if self.plan_has_pushed() && !matches!(self.run, Run::Done(_)) {
            (t::FOOTER_PUSHED.get().to_string(), warn())
        } else {
            (String::new(), dim())
        };
        // S23: a branch the host protects blocks the rewrite: the way out.
        let mut width = row.width;
        if self.howto_offered() {
            let label = gitpoulpi_i18n::msg::rewrite_host::BTN_HOWTO.get();
            let w = button_width(label);
            if row.width > w + 24 {
                ui.button(
                    row.right() - w,
                    row.y,
                    label,
                    RewriteHit::HostHowto,
                    false,
                    true,
                );
                width = row.width - w - 1;
            }
        }
        if !text.is_empty() {
            ui.text(row.x, row.y, width, &text, style);
        }
    }

    fn paint_actions(&self, ui: &mut Ui<'_>, row: Rect) {
        let busy = self.busy();
        let mut x = row.x;
        x += ui.button(
            x,
            row.y,
            t::BTN_PREVIEW.get(),
            RewriteHit::Preview,
            !self.can_apply(),
            self.can_preview(),
        ) + 1;
        x += ui.button(
            x,
            row.y,
            t::BTN_APPLY.get(),
            RewriteHit::Apply,
            self.can_apply(),
            self.can_apply(),
        ) + 1;
        if matches!(self.run, Run::Previewing { .. }) {
            ui.button(
                x,
                row.y,
                t::BTN_CANCEL.get(),
                RewriteHit::Cancel,
                false,
                true,
            );
        }
        // Save and Load hang at the right end.
        let load = format!("{} \u{25BE}", t::BTN_LOAD.get());
        let save = format!("{} \u{25BE}", t::BTN_SAVE.get());
        let lw = button_width(&load);
        let sw = button_width(&save);
        if row.width > lw + sw + 28 {
            let lx = row.right() - lw;
            let sx = lx - 1 - sw;
            // ✦ Create Rules with AI: only with AI built and turned on, left
            // of Save when there is room.
            if self.rules_ai_offered() {
                let label = Self::rules_ai_button_label();
                let aw = button_width(&label);
                let left_end = x + if matches!(self.run, Run::Previewing { .. }) {
                    button_width(t::BTN_CANCEL.get()) + 1
                } else {
                    0
                };
                if sx > left_end + aw + 1 {
                    let ax = sx - 1 - aw;
                    let enabled = !busy && !self.rules_ai.busy;
                    let w = ui.button(ax, row.y, &label, RewriteHit::RulesAi, false, enabled);
                    if enabled {
                        Self::tint_rules_ai_button(ui.buf, Rect::new(ax, row.y, w, 1));
                    }
                }
            }
            let (a, b) = (
                ui.button(sx, row.y, &save, RewriteHit::Save, false, !busy),
                ui.button(lx, row.y, &load, RewriteHit::Load, false, !busy),
            );
            ui.anchors.push((Anchor::Save, Rect::new(sx, row.y, a, 1)));
            ui.anchors.push((Anchor::Load, Rect::new(lx, row.y, b, 1)));
        }
    }

    pub(super) fn plan_has_pushed(&self) -> bool {
        self.plan.as_ref().is_some_and(|p| {
            p.refs.iter().any(|r| {
                r.included
                    && matches!(
                        r.pushed,
                        gitpoulpi_core::domain::rewrite::PushState::OnRemote { .. }
                    )
            })
        })
    }

    // ------------------------------------------------------------- popups

    fn paint_popup(
        &self,
        buf: &mut Buffer,
        bounds: Rect,
        anchors: &[(Anchor, Rect)],
        hovered: Option<RewriteHit>,
    ) -> Vec<(Rect, RewriteHit)> {
        let mut hits = Vec::new();
        let rows = self.popup_rows();
        let key = match &self.popup {
            Some(Popup::AddRule { .. }) => Anchor::Add,
            Some(Popup::Load { .. }) => Anchor::Load,
            Some(Popup::Choice { field, .. }) => Anchor::Field(*field),
            Some(Popup::Scope { .. }) => Anchor::Scope,
            Some(Popup::Save { .. }) => Anchor::Save,
            Some(Popup::Filter { .. }) => Anchor::Filter,
            Some(Popup::Refs { .. }) => Anchor::Refs,
            None => return hits,
        };
        let anchor = anchors
            .iter()
            .find(|(k, _)| *k == key)
            .map(|(_, r)| *r)
            .or_else(|| {
                anchors
                    .iter()
                    .find(|(k, _)| *k == Anchor::Scope)
                    .map(|(_, r)| *r)
            })
            .unwrap_or(Rect::new(bounds.x + 2, bounds.y + 2, 20, 1));
        let widest = rows
            .iter()
            .map(|r| {
                display_width(&r.label)
                    + r.disabled
                        .or(r.note)
                        .map_or(0, |n| display_width(n.get()) + 2)
                    + if r.checked.is_some() { 4 } else { 0 }
            })
            .max()
            .unwrap_or(8) as u16
            + 4;
        let width = widest
            .max(anchor.width)
            .min(bounds.width.saturating_sub(2))
            .max(12);
        let headings = rows.iter().filter(|r| r.heading.is_some()).count();
        let want = (rows.len() + headings) as u16 + 2;
        let below = anchor.bottom();
        let room_below = bounds.bottom().saturating_sub(below + 1);
        let room_above = anchor.y.saturating_sub(bounds.y + 1);
        let (height, y) = if want <= room_below {
            (want, below)
        } else if want <= room_above {
            (want, anchor.y - want)
        } else if room_below >= room_above {
            (room_below.max(4), below)
        } else {
            let h = room_above.max(4);
            (h, anchor.y.saturating_sub(h))
        };
        let height = height.min(bounds.height.saturating_sub(2));
        let x = anchor
            .x
            .min(bounds.right().saturating_sub(width + 1))
            .max(bounds.x + 1);
        let y = y
            .min(bounds.bottom().saturating_sub(height + 1))
            .max(bounds.y + 1);
        let popup = Rect::new(x, y, width, height);
        Clear.render(popup, buf);
        Block::default()
            .border_type(BorderType::Rounded)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(th().ui.accent))
            .style(Style::default().bg(th().ui.panel))
            .render(popup, buf);
        let cursor = self.popup_cursor();
        let room = height.saturating_sub(2) as usize;
        // Rows with their headings, windowed around the cursor.
        let mut lines: Vec<(Option<usize>, &PopupRow, bool)> = Vec::new();
        for (i, row) in rows.iter().enumerate() {
            if row.heading.is_some() {
                lines.push((None, row, true));
            }
            lines.push((Some(i), row, false));
        }
        let at = lines
            .iter()
            .position(|(i, _, _)| *i == Some(cursor))
            .unwrap_or(0);
        let first = (at + 1)
            .saturating_sub(room)
            .min(lines.len().saturating_sub(room));
        for (slot, (index, row, heading)) in lines.iter().skip(first).take(room).enumerate() {
            let ry = popup.y + 1 + slot as u16;
            let band = Rect::new(popup.x + 1, ry, popup.width - 2, 1);
            if *heading {
                if let Some(h) = row.heading {
                    dialog_kit::put(buf, band.x, ry, h.get(), band.width, bold(true));
                }
                continue;
            }
            let Some(i) = index else { continue };
            let hot = hovered == Some(RewriteHit::PopupRow(*i));
            let bg = row_bg(*i == cursor, hot);
            dialog_kit::fill(buf, band, Style::default().bg(bg));
            let mut x = band.x + 1;
            if let Some(on) = row.checked {
                x += dialog_kit::put(
                    buf,
                    x,
                    ry,
                    if on { "[x] " } else { "[ ] " },
                    4,
                    Style::default().fg(th().ui.text).bg(bg),
                )
                .width;
            }
            let style = Style::default()
                .fg(if row.disabled.is_some() {
                    th().ui.disabled
                } else if row.current {
                    th().ui.accent
                } else {
                    th().ui.text
                })
                .bg(bg);
            x += dialog_kit::put(
                buf,
                x,
                ry,
                &row.label,
                band.right().saturating_sub(x),
                style,
            )
            .width;
            if let Some(note) = row.disabled.or(row.note) {
                dialog_kit::put(
                    buf,
                    x + 2,
                    ry,
                    note.get(),
                    band.right().saturating_sub(x + 2),
                    Style::default().fg(th().ui.muted).bg(bg),
                );
            }
            hits.push((band, RewriteHit::PopupRow(*i)));
        }
        hits.push((bounds, RewriteHit::PopupShield));
        hits
    }

    // ----------------------------------------------------------- questions

    fn paint_question(
        &self,
        buf: &mut Buffer,
        bounds: Rect,
        hovered: Option<RewriteHit>,
    ) -> Vec<(Rect, RewriteHit)> {
        let mut hits = Vec::new();
        let Some(q) = self.question_view() else {
            return hits;
        };
        let width = bounds.width.saturating_sub(6).clamp(30, 76);
        let prompt = wrap(&q.prompt, width as usize - 4);
        let height = (prompt.len() + q.options.len() + 5) as u16;
        let rect = dialog_kit::centered(bounds, width, height);
        dialog_kit::panel(buf, rect, q.title.get());
        if let Some(cross) = crate::chrome::close_cross::paint(
            buf,
            rect,
            hovered == Some(RewriteHit::QuestionCancel),
        ) {
            hits.push((cross, RewriteHit::QuestionCancel));
        }
        let mut y = rect.y + 2;
        for line in &prompt {
            dialog_kit::put(buf, rect.x + 2, y, line, rect.width - 4, plain());
            y += 1;
        }
        y += 1;
        for (i, option) in q.options.iter().enumerate() {
            if y >= rect.bottom() - 1 {
                break;
            }
            let band = Rect::new(rect.x + 1, y, rect.width - 2, 1);
            let hot = hovered == Some(RewriteHit::Answer(i));
            let bg = row_bg(i == q.cursor, hot);
            dialog_kit::fill(buf, band, Style::default().bg(bg));
            dialog_kit::put(
                buf,
                band.x + 1,
                y,
                &format!("{}  {}", i + 1, option.get()),
                band.width - 2,
                Style::default().fg(th().ui.text).bg(bg),
            );
            hits.push((band, RewriteHit::Answer(i)));
            y += 1;
        }
        hits.push((bounds, RewriteHit::QuestionShield));
        hits
    }

    // --------------------------------------------------------------- mouse

    /// Whether `(col, row)` is outside the workbench as last painted.
    pub(crate) fn is_outside(&self, col: u16, row: u16) -> bool {
        let frame = self.paint.frame.get();
        frame.width > 0 && !frame.contains((col, row).into())
    }

    /// Puts `hit` into effect: the mouse's way into the same methods the
    /// keys use.
    pub(crate) fn on_press(&mut self, hit: RewriteHit, _col: u16, _row: u16) -> RewriteKeyOutcome {
        // A press anywhere but on the field being typed ends the typing.
        if self.editing.is_some() {
            let same = matches!(
                (hit, self.editing.as_ref().map(|e| e.target)),
                (RewriteHit::Field(f), Some(EditTarget::Field(e))) if f == e
            ) || matches!(
                (hit, self.editing.as_ref().map(|e| e.target)),
                (RewriteHit::RecipeName, Some(EditTarget::Name))
            );
            if same {
                return RewriteKeyOutcome::Stay;
            }
            self.stop_editing(true);
        }
        match hit {
            RewriteHit::Close => return self.request_close(),
            RewriteHit::Pane(focus) => self.focus = focus,
            RewriteHit::ScopeCombo => {
                self.focus = Focus::Scope;
                self.open_scope_menu();
            }
            RewriteHit::TagsCheck => self.toggle_tags(),
            RewriteHit::RefsButton => self.open_refs_menu(),
            RewriteHit::SinceCheck => self.toggle_since(),
            RewriteHit::RecipeName => self.edit_name(),
            RewriteHit::Rule(i) => {
                self.focus = Focus::Rules;
                self.select_rule(i);
            }
            RewriteHit::RuleCheck(i) => {
                self.focus = Focus::Rules;
                self.select_rule(i);
                self.toggle_rule();
            }
            RewriteHit::RuleUp(i) => {
                self.select_rule(i);
                self.move_rule(-1);
            }
            RewriteHit::RuleDown(i) => {
                self.select_rule(i);
                self.move_rule(1);
            }
            RewriteHit::RuleDelete(i) => {
                self.select_rule(i);
                self.delete_rule();
            }
            RewriteHit::AddRule => self.open_add_menu(),
            RewriteHit::RestoreRule => self.restore_rule(),
            RewriteHit::Field(i) => {
                self.focus = Focus::Detail;
                self.detail.cursor = i.min(self.detail.fields.len().saturating_sub(1));
                self.open_field(i);
            }
            RewriteHit::Tab(tab) => {
                self.focus = Focus::Preview;
                self.tab = tab;
                self.refusal = None;
            }
            RewriteHit::FilterCombo => self.open_filter_menu(),
            RewriteHit::Row(i) => {
                self.focus = Focus::Preview;
                self.rows.cursor = i;
                self.want_rows(i);
                // Two presses on the same row, a human apart: its files.
                let now = std::time::Instant::now();
                let double = self.last_row_press.is_some_and(|(row, at)| {
                    row == i && now.duration_since(at) < std::time::Duration::from_millis(600)
                });
                self.last_row_press = (!double).then_some((i, now));
                if double {
                    self.open_drill(i);
                }
            }
            RewriteHit::ScriptClose
            | RewriteHit::ScriptEditor
            | RewriteHit::ScriptParam(_)
            | RewriteHit::ScriptPath
            | RewriteHit::ScriptLoad
            | RewriteHit::ScriptSave
            | RewriteHit::ScriptTest
            | RewriteHit::ScriptCancel
            | RewriteHit::ScriptRow(_)
            | RewriteHit::ScriptAi
            | RewriteHit::ScriptPanel
            | RewriteHit::ScriptShield => self.on_script_press(hit, _col, _row),
            RewriteHit::PurgeOpen => self.open_purge(false),
            RewriteHit::PurgeClose
            | RewriteHit::PurgeDone
            | RewriteHit::PurgeShield
            | RewriteHit::PurgeBody
            | RewriteHit::PurgeCancel
            | RewriteHit::PurgeAnswer(_)
            | RewriteHit::PurgeRow(_)
            | RewriteHit::PurgeFix(_)
            | RewriteHit::PurgeCopy(_)
            | RewriteHit::PurgeLink(_)
            | RewriteHit::PurgeRun
            | RewriteHit::PurgeCheck => return self.on_purge_press(hit),
            RewriteHit::AiText
            | RewriteHit::AiSample
            | RewriteHit::AiSent
            | RewriteHit::AiSend
            | RewriteHit::AiCancel
            | RewriteHit::AiClose
            | RewriteHit::AiPanel
            | RewriteHit::AiShield => self.on_ai_press(hit, _col),
            RewriteHit::RulesAi => self.open_rules_ai(),
            RewriteHit::RaText
            | RewriteHit::RaContext
            | RewriteHit::RaReplace
            | RewriteHit::RaSent
            | RewriteHit::RaSentHead
            | RewriteHit::RaSentBar
            | RewriteHit::RaTextBar
            | RewriteHit::RaSend
            | RewriteHit::RaCancel
            | RewriteHit::RaClose
            | RewriteHit::RaPanel
            | RewriteHit::RaShield => self.on_rules_ai_press(hit, _col, _row),
            RewriteHit::DrillClose | RewriteHit::DrillShield => self.close_drill(),
            RewriteHit::DrillPath(i) => {
                self.drill_focus_diff(false);
                self.drill_select(i);
            }
            RewriteHit::DrillDiff => self.drill_focus_diff(true),
            RewriteHit::HostHowto => self.open_howto(),
            RewriteHit::HostPanel => {}
            RewriteHit::HostClose
            | RewriteHit::HostShield
            | RewriteHit::HostLink(_)
            | RewriteHit::HostCopy
            | RewriteHit::HostOpen => self.on_howto_press(hit),
            RewriteHit::ProtectedRow(i) => {
                self.focus = Focus::Preview;
                self.refs_cursor = i;
                if let Some(entry) = self.protected().get(i) {
                    let name = entry.name.clone();
                    self.toggle_protected(&name);
                }
            }
            RewriteHit::ProtectedAll => self.check_all_protected(),
            RewriteHit::ProtectedNone => self.uncheck_all_protected(),
            RewriteHit::CheckTools => self.check_tools_again(),
            RewriteHit::Preview => self.preview(),
            RewriteHit::Apply => self.apply(),
            RewriteHit::Cancel => self.cancel(),
            RewriteHit::Save => self.open_save_menu(),
            RewriteHit::Load => self.open_load_menu(),
            RewriteHit::Undo => return RewriteKeyOutcome::Undo,
            RewriteHit::ResultClose => return RewriteKeyOutcome::Close,
            RewriteHit::PushRow(_)
            | RewriteHit::PushAll
            | RewriteHit::PushNone
            | RewriteHit::PushSend
            | RewriteHit::CopyNote
            | RewriteHit::OpenMaps
            | RewriteHit::FetchReview(_)
            | RewriteHit::MapCopy(_)
            | RewriteHit::LfsPush
            | RewriteHit::LfsFetch => return self.on_push_press(hit),
            RewriteHit::SecretsScan
            | RewriteHit::SecretRow(_)
            | RewriteHit::SecretsRemove
            | RewriteHit::SecretsAll
            | RewriteHit::SecretsNone => self.on_secrets_press(hit),
            RewriteHit::IdentRow(_)
            | RewriteHit::IdentMerge
            | RewriteHit::IdentRefresh
            | RewriteHit::IdentAll
            | RewriteHit::IdentNone
            | RewriteHit::IdentUndo
            | RewriteHit::IdentOffer
            | RewriteHit::IdentPick(_)
            | RewriteHit::IdentPickCancel
            | RewriteHit::IdentPickBody
            | RewriteHit::IdentPickShield => return self.on_identities_press(hit),
            RewriteHit::SizeRun
            | RewriteHit::SizeRow(_)
            | RewriteHit::SizeRemovePaths
            | RewriteHit::SizeRemoveOver
            | RewriteHit::SizeAll
            | RewriteHit::SizeNone
            | RewriteHit::SizeView(_)
            | RewriteHit::SizeSort(_) => self.on_size_press(hit),
            RewriteHit::PopupRow(i) => self.popup_pick(i),
            RewriteHit::PopupShield => self.popup = None,
            RewriteHit::Answer(i) => self.answer(i),
            RewriteHit::QuestionCancel | RewriteHit::QuestionShield => self.dismiss_question(),
        }
        RewriteKeyOutcome::Stay
    }

    /// The cross, Esc and an outside click: closes, unless a job runs (the
    /// cross then stops the preview first, like Esc).
    pub(crate) fn request_close(&mut self) -> RewriteKeyOutcome {
        if self.purge.is_some() {
            return self.purge_close();
        }
        if matches!(self.run, Run::Previewing { .. }) || self.scanning() || self.analysing() {
            self.cancel();
            return RewriteKeyOutcome::Stay;
        }
        if self.busy() {
            return RewriteKeyOutcome::Stay;
        }
        RewriteKeyOutcome::Close
    }

    /// A press outside the frame: closes an empty workbench; one holding
    /// rules swallows it and the cross flashes (the recipe is work).
    pub(crate) fn outside_press(&mut self) -> RewriteKeyOutcome {
        if self.purge.is_some() {
            return self.purge_outside_press();
        }
        if self.script.ai.dialog.is_some() || self.rules_ai.is_open() {
            return RewriteKeyOutcome::Stay;
        }
        if self.script.editor.is_some() {
            self.close_script_editor();
            return RewriteKeyOutcome::Stay;
        }
        if self.drill.is_some() {
            self.close_drill();
            return RewriteKeyOutcome::Stay;
        }
        if self.host.howto.is_some() {
            self.close_howto();
            return RewriteKeyOutcome::Stay;
        }
        if self.question.is_some()
            || self.popup.is_some()
            || self.editing.is_some()
            || self.identities.pick.is_some()
        {
            return RewriteKeyOutcome::Stay;
        }
        if self.recipe.rules.is_empty() || matches!(self.run, Run::Done(_)) {
            return self.request_close();
        }
        self.paint.flash.set(true);
        RewriteKeyOutcome::Stay
    }

    fn select_rule(&mut self, index: usize) {
        let delta = index as isize - self.rules.cursor as isize;
        if delta != 0 {
            self.move_rule_cursor(delta);
        }
    }

    /// The wheel over `focus`'s pane: moves its cursor.
    pub(crate) fn on_wheel(&mut self, focus: Focus, delta: isize) {
        match focus {
            Focus::Rules => self.move_rule_cursor(delta),
            Focus::Detail => self.detail.move_cursor(delta),
            Focus::Preview => self.move_preview_cursor(delta),
            Focus::Scope => {}
        }
    }
}

/// A rule's name: the user's own, else the kind's.
pub(super) fn row_name(row: &RuleRow) -> String {
    row.custom
        .clone()
        .unwrap_or_else(|| row.label.get().to_string())
}

fn first_line(text: &str) -> String {
    let mut lines = text.lines();
    let first = lines.next().unwrap_or("").to_string();
    let more = lines.count();
    if more == 0 {
        first
    } else {
        format!("{first} (+{more})")
    }
}

/// The target at `(col, row)`: the first registered one.
pub(crate) fn rewrite_hit_at(
    hits: &[(Rect, crate::chrome::HitTarget)],
    col: u16,
    row: u16,
) -> Option<RewriteHit> {
    use crate::chrome::HitTarget;
    hits.iter().find_map(|(rect, target)| match target {
        HitTarget::Rewrite(hit) if rect.contains((col, row).into()) => Some(*hit),
        _ => None,
    })
}
