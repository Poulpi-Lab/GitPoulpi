//! The cards of the merge editor (plan §6): what the central column shows
//! instead of three text panes when a conflict is not text -- a binary or
//! LFS or too large file, a delete / modify, a rename leftover, a symlink,
//! a mode-only change, a submodule.
//!
//! [`Card::detect`] is pure (a `MergeDocument` in, a description and the
//! choices out), so every kind is unit tested without a terminal. Each
//! choice is one whole-file resolution (`TakeSide` or `Delete`), staged at
//! once, and has a key (`merge_card` keymap context), a button and a
//! palette command, all running the same [`MergeCmd`](super::MergeCmd).

use crate::keymap::term::TermKeymap;
use crate::th;
use gitpoulpi_core::domain::{human_size, BlobContent, ConflictKind, ConflictSide, MergeDocument};
use gitpoulpi_i18n::msg::merge as tr;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};

use crate::dialog_kit::{self, ButtonState};
use crate::keymap::{self, Action};
use crate::{display_width, fit, plain_button_style, tail_fit};

use super::view::MergeView;
use super::{MergeCmd, MergeHit};
use crate::image_view::split::thumbs;

/// Which kind of non-text conflict.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CardKind {
    Binary,
    LfsPointer,
    TooLarge,
    /// One side deleted the file, the other changed it.
    DeleteModify,
    /// Rename leftovers: the path exists on one side only (or none).
    Rename,
    Symlink,
    /// The content is the same, the file mode differs (the exec bit).
    ModeOnly,
    /// The mode and the content both differ.
    ModeAndContent,
    Submodule,
}

/// What a choice does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CardAction {
    /// The whole file of one stage, as git has it (a stage with no file
    /// removes the path).
    Side(ConflictSide),
    /// `git rm`.
    Delete,
    /// The configured external merge tool.
    External,
}

impl CardAction {
    pub(crate) fn cmd(self) -> MergeCmd {
        match self {
            CardAction::Side(ConflictSide::Ours) => MergeCmd::UseFileA,
            CardAction::Side(ConflictSide::Theirs) => MergeCmd::UseFileB,
            CardAction::Side(ConflictSide::Base) => MergeCmd::UseFileBase,
            CardAction::Delete => MergeCmd::Delete,
            CardAction::External => MergeCmd::ExternalTool,
        }
    }
}

/// One button of the card.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CardChoice {
    pub label: String,
    pub action: CardAction,
    /// The keymap action whose key the button shows.
    pub key: Action,
}

/// What one side of the conflict holds, in a few lines.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CardSide {
    pub side: ConflictSide,
    pub title: String,
    pub lines: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Card {
    pub kind: CardKind,
    /// The sentence naming the conflict.
    pub headline: String,
    pub sides: Vec<CardSide>,
    pub choices: Vec<CardChoice>,
    /// The stage `Keep` stands for (delete / modify, rename leftovers).
    pub keep: Option<ConflictSide>,
}

fn short_oid(oid: gitpoulpi_core::domain::Oid) -> String {
    oid.to_hex()[..crate::SHORT_SHA_LEN].to_string()
}

fn mode_text(mode: u32) -> String {
    format!("{mode:06o}")
}

impl Card {
    /// The card for `doc`, or `None` when the conflict is text and the
    /// three-pane editor shows it.
    pub(crate) fn detect(doc: &MergeDocument) -> Option<Card> {
        let e = &doc.entry;
        let content_conflict =
            matches!(e.kind, ConflictKind::BothModified | ConflictKind::BothAdded);
        let kind = if e.is_submodule() {
            CardKind::Submodule
        } else if e.is_symlink() {
            CardKind::Symlink
        } else if matches!(
            e.kind,
            ConflictKind::DeletedByUs | ConflictKind::DeletedByThem
        ) {
            CardKind::DeleteModify
        } else if !content_conflict {
            CardKind::Rename
        } else if e.mode_changed() {
            match (e.ours, e.theirs) {
                (Some(o), Some(t)) if o.oid == t.oid => CardKind::ModeOnly,
                _ => CardKind::ModeAndContent,
            }
        } else {
            let blobs = [&doc.base, &doc.ours, &doc.theirs];
            let any = |f: fn(&BlobContent) -> bool| {
                blobs
                    .iter()
                    .any(|b| b.as_ref().is_some_and(|b| f(&b.content)))
            };
            if any(|c| matches!(c, BlobContent::LfsPointer(_))) {
                CardKind::LfsPointer
            } else if any(|c| matches!(c, BlobContent::TooLarge)) {
                CardKind::TooLarge
            } else if any(|c| matches!(c, BlobContent::Binary)) {
                CardKind::Binary
            } else {
                return None;
            }
        };
        let sides = [ConflictSide::Ours, ConflictSide::Theirs, ConflictSide::Base]
            .into_iter()
            .filter(|s| *s != ConflictSide::Base || e.base.is_some())
            .map(|s| side_of(doc, kind, s))
            .collect();
        let (headline, choices, keep) = choices_for(doc, kind);
        Some(Card {
            kind,
            headline,
            sides,
            choices,
            keep,
        })
    }

    /// The choice bound to `action`, if the card has it.
    #[cfg(test)]
    pub(crate) fn has(&self, action: CardAction) -> bool {
        self.choices.iter().any(|c| c.action == action)
    }
}

fn side_of(doc: &MergeDocument, kind: CardKind, side: ConflictSide) -> CardSide {
    let l = &doc.labels;
    let (letter, label) = match side {
        ConflictSide::Ours => ("A", &l.current),
        ConflictSide::Theirs => ("B", &l.incoming),
        ConflictSide::Base => ("O", &l.base),
    };
    let label = super::labels::side(label, side);
    let title = format!("{letter} \u{b7} {label}");
    let (stage, blob) = (doc.entry.stage(side), doc.blob(side));
    let mut lines = Vec::new();
    match stage {
        None => lines.push(tr::CARD_SIDE_DELETED.get().to_string()),
        Some(st) => {
            let size = blob.map_or(0, |b| b.size);
            match (kind, blob.map(|b| &b.content)) {
                (CardKind::Submodule, _) => lines.push(tr::card_commit(short_oid(st.oid))),
                (CardKind::Symlink, Some(BlobContent::Text { lines: t, .. })) => {
                    lines.push(tr::card_points_to(t.first().cloned().unwrap_or_default()));
                    lines.push(tr::card_detail(
                        short_oid(st.oid),
                        human_size(size),
                        mode_text(st.mode),
                    ));
                }
                (_, Some(BlobContent::LfsPointer(p))) => {
                    lines.push(tr::card_lfs_object(
                        p.oid.chars().take(12).collect::<String>(),
                        human_size(p.size),
                    ));
                    lines.push(tr::card_detail(
                        short_oid(st.oid),
                        human_size(size),
                        mode_text(st.mode),
                    ));
                }
                _ => lines.push(tr::card_detail(
                    short_oid(st.oid),
                    human_size(size),
                    mode_text(st.mode),
                )),
            }
        }
    }
    CardSide { side, title, lines }
}

fn choice(label: String, action: CardAction, key: Action) -> CardChoice {
    CardChoice { label, action, key }
}

/// The headline, the buttons and what `Keep` stands for.
fn choices_for(
    doc: &MergeDocument,
    kind: CardKind,
) -> (String, Vec<CardChoice>, Option<ConflictSide>) {
    let e = &doc.entry;
    let mut out = Vec::new();
    let mut keep = None;
    let headline = match kind {
        CardKind::DeleteModify | CardKind::Rename => {
            // The side that still has the file.
            keep = [ConflictSide::Ours, ConflictSide::Theirs]
                .into_iter()
                .find(|s| e.stage(*s).is_some());
            if kind == CardKind::DeleteModify {
                if e.kind == ConflictKind::DeletedByUs {
                    tr::CARD_DELETED_IN_A.get().to_string()
                } else {
                    tr::CARD_DELETED_IN_B.get().to_string()
                }
            } else {
                tr::CARD_RENAME.get().to_string()
            }
        }
        CardKind::Binary => tr::CARD_BINARY.get().to_string(),
        CardKind::LfsPointer => tr::CARD_LFS.get().to_string(),
        CardKind::TooLarge => tr::CARD_TOO_LARGE.get().to_string(),
        CardKind::Symlink => tr::CARD_SYMLINK.get().to_string(),
        CardKind::ModeOnly => tr::card_mode(
            e.ours.map(|s| mode_text(s.mode)).unwrap_or_default(),
            e.theirs.map(|s| mode_text(s.mode)).unwrap_or_default(),
        ),
        CardKind::ModeAndContent => tr::CARD_MODE_CONTENT.get().to_string(),
        CardKind::Submodule => tr::CARD_SUBMODULE.get().to_string(),
    };
    match kind {
        CardKind::DeleteModify | CardKind::Rename => {
            if let Some(side) = keep {
                out.push(choice(
                    tr::CARD_KEEP_FILE.get().to_string(),
                    CardAction::Side(side),
                    Action::MergeCardKeep,
                ));
            }
            out.push(choice(
                if kind == CardKind::DeleteModify {
                    tr::CARD_ACCEPT_DELETE.get().to_string()
                } else {
                    tr::CARD_DELETE.get().to_string()
                },
                CardAction::Delete,
                Action::MergeCardDelete,
            ));
        }
        _ => {
            out.push(choice(
                tr::CARD_KEEP_A.get().to_string(),
                CardAction::Side(ConflictSide::Ours),
                Action::MergeCardA,
            ));
            out.push(choice(
                tr::CARD_TAKE_B.get().to_string(),
                CardAction::Side(ConflictSide::Theirs),
                Action::MergeCardB,
            ));
            if e.base.is_some() {
                out.push(choice(
                    tr::CARD_TAKE_BASE.get().to_string(),
                    CardAction::Side(ConflictSide::Base),
                    Action::MergeCardBase,
                ));
            }
        }
    }
    if matches!(
        kind,
        CardKind::Binary | CardKind::LfsPointer | CardKind::TooLarge | CardKind::ModeAndContent
    ) {
        out.push(choice(
            tr::CARD_EXTERNAL.get().to_string(),
            CardAction::External,
            Action::MergeCardExternal,
        ));
    }
    (headline, out, keep)
}

/// Rows a side's preview needs to be worth showing, and the most it takes.
const MIN_THUMB_ROWS: u16 = 4;
const MAX_THUMB_ROWS: u16 = 14;

/// The index stage of a side: 1 base, 2 A (ours), 3 B (theirs).
fn stage_number(side: ConflictSide) -> u8 {
    match side {
        ConflictSide::Base => 1,
        ConflictSide::Ours => 2,
        ConflictSide::Theirs => 3,
    }
}

fn card_has_stage(card: &Card, side: ConflictSide) -> bool {
    card.sides.iter().any(|s| s.side == side)
}

fn side_color(side: ConflictSide) -> ratatui::style::Color {
    match side {
        ConflictSide::Ours => crate::th().merge.ours,
        ConflictSide::Theirs => crate::th().merge.theirs,
        ConflictSide::Base => crate::th().merge.base,
    }
}

impl MergeView {
    /// The card this view shows instead of the panes, if any.
    pub(crate) fn card(&self) -> Option<&Card> {
        self.card.as_ref()
    }

    /// The command a card key stands for, or `None` for a key that is not
    /// a card's.
    pub(super) fn card_cmd(&self, key: &crossterm::event::KeyEvent) -> Option<MergeCmd> {
        let card = self.card.as_ref()?;
        let action = keymap::current().action_in(keymap::Ctx::MergeCard, key)?;
        // Only the buttons the card shows have keys.
        if !card.choices.iter().any(|c| c.key == action) {
            return None;
        }
        Some(match action {
            Action::MergeCardA => MergeCmd::UseFileA,
            Action::MergeCardB => MergeCmd::UseFileB,
            Action::MergeCardBase => MergeCmd::UseFileBase,
            Action::MergeCardKeep => MergeCmd::Keep,
            Action::MergeCardDelete => MergeCmd::Delete,
            Action::MergeCardExternal => MergeCmd::ExternalTool,
            _ => return None,
        })
    }

    /// A key on a card: its buttons' keys, the file keys and Esc; nothing
    /// else reaches the (empty) Output.
    pub(super) fn card_key(&mut self, key: &crossterm::event::KeyEvent) -> super::Outcome {
        if let Some(cmd) = self.card_cmd(key) {
            return self.run(cmd);
        }
        match keymap::current().action_in(keymap::Ctx::Merge, key) {
            Some(Action::MergeNextFile) => return self.run(MergeCmd::NextFile),
            Some(Action::MergePrevFile) => return self.run(MergeCmd::PrevFile),
            _ => {}
        }
        if key.code == crossterm::event::KeyCode::Esc {
            return self.run(MergeCmd::Close);
        }
        super::Outcome::default()
    }

    /// A click on the card's `i`th button.
    pub(super) fn card_choice(&mut self, i: usize) -> super::Outcome {
        match self.card.as_ref().and_then(|c| c.choices.get(i)) {
            Some(choice) => {
                let cmd = match (choice.action, choice.key) {
                    (_, Action::MergeCardKeep) => MergeCmd::Keep,
                    (action, _) => action.cmd(),
                };
                self.run(cmd)
            }
            None => super::Outcome::default(),
        }
    }

    /// Paints the card over `area` and returns its hit rects.
    pub(super) fn render_card(
        &mut self,
        buf: &mut Buffer,
        area: Rect,
        hovered: Option<MergeHit>,
    ) -> Vec<(Rect, MergeHit)> {
        let mut hits = Vec::new();
        let Some(card) = self.card.clone() else {
            return hits;
        };
        let x0 = area.x + 1;
        let x1 = area.right().saturating_sub(1);
        let inner_w = x1.saturating_sub(x0);
        // The frame.
        let edge = Style::default().fg(th().ui.accent);
        ratatui::widgets::Widget::render(
            ratatui::widgets::Block::default()
                .borders(ratatui::widgets::Borders::ALL)
                .border_type(ratatui::widgets::BorderType::Rounded)
                .border_style(edge),
            area,
            buf,
        );
        // Title row.
        let (k, n) = self.file_pos;
        let right = format!(" {} ", tr::file_of(k, n));
        let (prev, next, close) = (" \u{2039} ", " \u{203a} ", " \u{2715} ");
        let rw = (display_width(&right)
            + display_width(prev)
            + display_width(next)
            + display_width(close)) as u16;
        let left_room = inner_w.saturating_sub(rw + 1) as usize;
        let mut title = format!(
            " {}",
            tr::title(tail_fit(&self.path().to_string_lossy(), left_room))
        );
        let desc = super::labels::description(&self.model.merge().labels.description);
        if !desc.is_empty() {
            title.push_str(&format!(" \u{2500}\u{2500} {desc}"));
        }
        title.push(' ');
        dialog_kit::put(
            buf,
            x0,
            area.y,
            &fit(&title, left_room),
            left_room as u16,
            Style::default()
                .fg(th().ui.text)
                .add_modifier(Modifier::BOLD),
        );
        let mut x = x1.saturating_sub(rw);
        x += dialog_kit::put(
            buf,
            x,
            area.y,
            &right,
            rw,
            Style::default().fg(th().ui.muted),
        )
        .width;
        for (label, hit) in [
            (prev, MergeHit::PrevFile),
            (next, MergeHit::NextFile),
            (close, MergeHit::Close),
        ] {
            let w = display_width(label) as u16;
            let style = if hovered == Some(hit) {
                plain_button_style(true)
            } else {
                Style::default().fg(th().ui.text)
            };
            dialog_kit::put(buf, x, area.y, label, w, style);
            hits.push((Rect::new(x, area.y, w, 1), hit));
            x += w;
        }
        if inner_w < 8 || area.height < 8 {
            return hits;
        }
        // The headline.
        let mut y = area.y + 2;
        dialog_kit::put(
            buf,
            x0 + 1,
            y,
            &fit(&card.headline, inner_w.saturating_sub(2) as usize),
            inner_w.saturating_sub(2),
            Style::default()
                .fg(th().ui.text)
                .add_modifier(Modifier::BOLD),
        );
        y += 2;
        // The sides, in boxes side by side (stacked when narrow).
        let side_by_side = inner_w >= 70 && card.sides.len() <= 3;
        let boxes = card.sides.len().max(1) as u16;
        let text_h = card.sides.iter().map(|s| s.lines.len()).max().unwrap_or(1) as u16 + 2;
        let bottom_reserved = 5u16;
        // A picture conflict also shows each side's picture (read-only, Fit)
        // in the room the buttons leave; too little room, no previews.
        let thumb_h = if side_by_side
            && matches!(card.kind, CardKind::Binary | CardKind::LfsPointer)
            && thumbs::wanted(self.path())
        {
            let room = area
                .bottom()
                .saturating_sub(y + text_h + 1 + bottom_reserved);
            if room >= MIN_THUMB_ROWS {
                room.min(MAX_THUMB_ROWS)
            } else {
                0
            }
        } else {
            0
        };
        if thumb_h > 0 {
            let e = &self.model.merge().entry;
            let stages: Vec<(u8, gitpoulpi_core::domain::Oid)> = card
                .sides
                .iter()
                .filter_map(|s| e.stage(s.side).map(|st| (stage_number(s.side), st.oid)))
                .collect();
            thumbs::follow(self.path(), &stages);
        }
        let box_h = text_h + thumb_h;
        let mut rects = Vec::new();
        if side_by_side {
            let w = (inner_w.saturating_sub(2) / boxes).max(10);
            for i in 0..boxes {
                rects.push(Rect::new(x0 + 1 + i * w, y, w.saturating_sub(1), box_h));
            }
            y += box_h + 1;
        } else {
            for _ in 0..boxes {
                if y + box_h + bottom_reserved > area.bottom() {
                    break;
                }
                rects.push(Rect::new(x0 + 1, y, inner_w.saturating_sub(2), box_h));
                y += box_h;
            }
            y += 1;
        }
        for (side, rect) in card.sides.iter().zip(rects) {
            if rect.right() > x1 || rect.bottom() >= area.bottom() {
                continue;
            }
            let color = side_color(side.side);
            ratatui::widgets::Widget::render(
                ratatui::widgets::Block::default()
                    .borders(ratatui::widgets::Borders::ALL)
                    .border_type(ratatui::widgets::BorderType::Rounded)
                    .border_style(Style::default().fg(color))
                    .title(format!(
                        " {} ",
                        fit(&side.title, rect.width.saturating_sub(4) as usize)
                    ))
                    .title_style(Style::default().fg(color).add_modifier(Modifier::BOLD)),
                rect,
                buf,
            );
            for (i, line) in side.lines.iter().enumerate() {
                dialog_kit::put(
                    buf,
                    rect.x + 1,
                    rect.y + 1 + i as u16,
                    &fit(line, rect.width.saturating_sub(2) as usize),
                    rect.width.saturating_sub(2),
                    Style::default().fg(th().ui.text),
                );
            }
            if thumb_h > 0 && card_has_stage(&card, side.side) {
                let top = rect.y + 1 + side.lines.len() as u16;
                thumbs::paint(
                    buf,
                    Rect::new(
                        rect.x + 1,
                        top,
                        rect.width.saturating_sub(2),
                        thumb_h.min(rect.bottom().saturating_sub(top + 1)),
                    ),
                    stage_number(side.side),
                );
            }
        }
        // The buttons, wrapping onto a second row when they do not fit.
        let mut bx = x0 + 1;
        let mut by = y.min(area.bottom().saturating_sub(4));
        for (i, c) in card.choices.iter().enumerate() {
            let label = format!("{} ({})", c.label, dialog_kit::key_label(c.key));
            let w = dialog_kit::button_width(&label);
            if bx + w > x1 && bx > x0 + 1 {
                bx = x0 + 1;
                by += 1;
            }
            if by + 2 > area.bottom() {
                break;
            }
            let lit = hovered == Some(MergeHit::Choice(i));
            let state = if i == 0 {
                ButtonState::Primary {
                    hovered: lit,
                    focused: false,
                }
            } else {
                ButtonState::Normal {
                    hovered: lit,
                    focused: false,
                }
            };
            let rect = dialog_kit::button(buf, bx, by, &label, state);
            hits.push((rect, MergeHit::Choice(i)));
            bx += w + 1;
        }
        // What choosing does, and the keys that are not buttons.
        let note_y = area.bottom().saturating_sub(2);
        if note_y > by {
            dialog_kit::put(
                buf,
                x0 + 1,
                note_y,
                &fit(tr::CARD_NOTE.get(), inner_w.saturating_sub(2) as usize),
                inner_w.saturating_sub(2),
                Style::default().fg(th().ui.muted),
            );
        }
        hits
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gitpoulpi_core::domain::{
        ConflictEntry, Eol, Oid, SideBlob, SideLabels, StageEntry, WorktreeFileState,
    };

    const REG: u32 = 0o100644;

    fn st(n: u8, mode: u32) -> StageEntry {
        StageEntry {
            mode,
            oid: Oid([n; 20]),
        }
    }

    fn text_blob(n: u8, mode: u32, line: &str) -> SideBlob {
        SideBlob {
            oid: Oid([n; 20]),
            mode,
            size: line.len() as u64,
            content: BlobContent::Text {
                lines: vec![line.to_string()],
                crlf: false,
                final_newline: true,
            },
        }
    }

    fn blob(n: u8, content: BlobContent) -> SideBlob {
        SideBlob {
            oid: Oid([n; 20]),
            mode: REG,
            size: 2048,
            content,
        }
    }

    fn doc(
        base: Option<(StageEntry, SideBlob)>,
        ours: Option<(StageEntry, SideBlob)>,
        theirs: Option<(StageEntry, SideBlob)>,
    ) -> MergeDocument {
        let entry = ConflictEntry::from_stages(
            "f".into(),
            base.as_ref().map(|b| b.0),
            ours.as_ref().map(|b| b.0),
            theirs.as_ref().map(|b| b.0),
        )
        .unwrap();
        MergeDocument {
            entry,
            base: base.map(|b| b.1),
            ours: ours.map(|b| b.1),
            theirs: theirs.map(|b| b.1),
            regions: Vec::new(),
            labels: SideLabels::default(),
            worktree: WorktreeFileState::Missing,
            eol: Eol::Lf,
            bom: false,
            final_newline: true,
            encoding: None,
            marker_size: 7,
        }
    }

    fn pair(n: u8, content: BlobContent) -> (StageEntry, SideBlob) {
        (st(n, REG), blob(n, content))
    }

    fn actions(card: &Card) -> Vec<CardAction> {
        card.choices.iter().map(|c| c.action).collect()
    }

    #[test]
    fn binary_offers_a_b_and_base() {
        let d = doc(
            Some(pair(1, BlobContent::Binary)),
            Some(pair(2, BlobContent::Binary)),
            Some(pair(3, BlobContent::Binary)),
        );
        let c = Card::detect(&d).unwrap();
        assert_eq!(c.kind, CardKind::Binary);
        assert_eq!(
            actions(&c),
            [
                CardAction::Side(ConflictSide::Ours),
                CardAction::Side(ConflictSide::Theirs),
                CardAction::Side(ConflictSide::Base),
                CardAction::External
            ]
        );
        assert_eq!(c.sides.len(), 3);
        assert!(c.sides[0].lines[0].contains("2.0 KB"), "{:?}", c.sides);
    }

    #[test]
    fn add_add_binary_has_no_base_choice() {
        let d = doc(
            None,
            Some(pair(2, BlobContent::Binary)),
            Some(pair(3, BlobContent::Binary)),
        );
        let c = Card::detect(&d).unwrap();
        assert!(!c.has(CardAction::Side(ConflictSide::Base)));
        assert_eq!(c.sides.len(), 2);
    }

    #[test]
    fn lfs_pointers_show_their_objects() {
        let p = |n: u8| {
            BlobContent::LfsPointer(gitpoulpi_core::domain::LfsPointer {
                oid: format!("{n:02x}").repeat(32),
                size: 5_000_000,
            })
        };
        let d = doc(
            Some(pair(1, p(1))),
            Some(pair(2, p(2))),
            Some(pair(3, p(3))),
        );
        let c = Card::detect(&d).unwrap();
        assert_eq!(c.kind, CardKind::LfsPointer);
        assert!(
            c.sides[1].lines[0].contains("030303030303"),
            "{:?}",
            c.sides
        );
        assert!(c.sides[0].lines[0].contains("5.0 MB"), "{:?}", c.sides);
    }

    #[test]
    fn too_large_offers_the_external_tool() {
        let d = doc(
            Some(pair(1, BlobContent::TooLarge)),
            Some(pair(2, BlobContent::TooLarge)),
            Some(pair(3, BlobContent::TooLarge)),
        );
        let c = Card::detect(&d).unwrap();
        assert_eq!(c.kind, CardKind::TooLarge);
        assert!(c.has(CardAction::External));
    }

    #[test]
    fn delete_modify_both_ways() {
        let text = |n| {
            pair(
                n,
                BlobContent::Text {
                    lines: vec!["x".into()],
                    crlf: false,
                    final_newline: true,
                },
            )
        };
        // Deleted by us: stage 2 missing.
        let d = doc(Some(text(1)), None, Some(text(3)));
        let c = Card::detect(&d).unwrap();
        assert_eq!(c.kind, CardKind::DeleteModify);
        assert_eq!(c.keep, Some(ConflictSide::Theirs));
        assert_eq!(
            actions(&c),
            [CardAction::Side(ConflictSide::Theirs), CardAction::Delete]
        );
        assert_eq!(c.headline, tr::CARD_DELETED_IN_A.get());
        assert_eq!(c.sides[0].lines, [tr::CARD_SIDE_DELETED.get()]);
        // Deleted by them: stage 3 missing.
        let d = doc(Some(text(1)), Some(text(2)), None);
        let c = Card::detect(&d).unwrap();
        assert_eq!(c.keep, Some(ConflictSide::Ours));
        assert_eq!(c.headline, tr::CARD_DELETED_IN_B.get());
    }

    #[test]
    fn rename_leftovers_keep_or_delete() {
        let t = || {
            pair(
                2,
                BlobContent::Text {
                    lines: vec!["x".into()],
                    crlf: false,
                    final_newline: true,
                },
            )
        };
        let c = Card::detect(&doc(None, Some(t()), None)).unwrap();
        assert_eq!(c.kind, CardKind::Rename);
        assert_eq!(c.keep, Some(ConflictSide::Ours));
        assert_eq!(
            actions(&c),
            [CardAction::Side(ConflictSide::Ours), CardAction::Delete]
        );
        // Both deleted: only the deletion is left.
        let d = doc(Some(t()), None, None);
        let c = Card::detect(&d).unwrap();
        assert_eq!(c.keep, None);
        assert_eq!(actions(&c), [CardAction::Delete]);
    }

    #[test]
    fn symlink_shows_both_targets() {
        let link = |n: u8, target: &str| (st(n, 0o120000), text_blob(n, 0o120000, target));
        let d = doc(
            Some(link(1, "old")),
            Some(link(2, "a/b")),
            Some(link(3, "c/d")),
        );
        let c = Card::detect(&d).unwrap();
        assert_eq!(c.kind, CardKind::Symlink);
        assert!(c.sides[0].lines[0].contains("a/b"), "{:?}", c.sides);
        assert!(c.sides[1].lines[0].contains("c/d"));
    }

    #[test]
    fn mode_only_names_both_modes() {
        let t = |m: u32| (st(5, m), text_blob(5, m, "same"));
        let d = doc(Some(t(0o100644)), Some(t(0o100644)), Some(t(0o100755)));
        let c = Card::detect(&d).unwrap();
        assert_eq!(c.kind, CardKind::ModeOnly);
        assert!(c.headline.contains("100644") && c.headline.contains("100755"));
        // A different blob as well: a different card.
        let d = doc(
            Some(t(0o100644)),
            Some((st(5, 0o100644), text_blob(5, 0o100644, "x"))),
            Some((st(6, 0o100755), text_blob(6, 0o100755, "y"))),
        );
        assert_eq!(Card::detect(&d).unwrap().kind, CardKind::ModeAndContent);
    }

    #[test]
    fn submodule_shows_both_commits() {
        let sm = |n: u8| {
            (
                st(n, 0o160000),
                SideBlob {
                    oid: Oid([n; 20]),
                    mode: 0o160000,
                    size: 0,
                    content: BlobContent::TooLarge,
                },
            )
        };
        let d = doc(Some(sm(1)), Some(sm(2)), Some(sm(3)));
        let c = Card::detect(&d).unwrap();
        assert_eq!(c.kind, CardKind::Submodule);
        assert!(c.sides[0].lines[0].contains("0202020"), "{:?}", c.sides);
    }

    /// The card paints inside any terminal the editor accepts, names the
    /// conflict, and registers a hit rect per button (maintainer rule 6) plus
    /// the header's.
    #[test]
    fn a_card_paints_its_headline_sides_and_every_button_at_every_size() {
        use ratatui::backend::TestBackend;
        use ratatui::Terminal;
        let docs = [
            doc(
                Some(pair(1, BlobContent::Binary)),
                Some(pair(2, BlobContent::Binary)),
                Some(pair(3, BlobContent::Binary)),
            ),
            doc(
                Some(pair(1, BlobContent::TooLarge)),
                Some(pair(2, BlobContent::TooLarge)),
                Some(pair(3, BlobContent::TooLarge)),
            ),
        ];
        for d in docs {
            for (w, h) in [(60u16, 14u16), (80, 24), (100, 30), (200, 50)] {
                let model = crate::merge_view::model::MergeModel::new(d.clone());
                let mut view = MergeView::new(model);
                let mut hits = Vec::new();
                let mut terminal = Terminal::new(TestBackend::new(w, h)).unwrap();
                terminal
                    .draw(|f| hits = view.render(f, f.area(), None))
                    .unwrap();
                let text = dialog_kit::buffer_text(terminal.backend().buffer());
                let card = view.card().unwrap();
                let area = Rect::new(0, 0, w, h);
                assert!(hits.iter().all(|(r, _)| area.contains(r.as_position())));
                for needed in [MergeHit::Close, MergeHit::NextFile] {
                    assert!(hits.iter().any(|(_, x)| *x == needed), "{needed:?}@{w}x{h}");
                }
                if h >= 24 {
                    assert!(text.contains(&card.headline[..20]), "{w}x{h}\n{text}");
                    for i in 0..card.choices.len() {
                        assert!(
                            hits.iter().any(|(_, x)| *x == MergeHit::Choice(i)),
                            "button {i} at {w}x{h}\n{text}"
                        );
                    }
                    assert!(
                        text.contains("A \u{b7}") && text.contains("B \u{b7}"),
                        "{text}"
                    );
                }
            }
        }
    }

    /// The golden text of a card at 110x24 (`GITPOULPI_UPDATE_GOLDEN=1`
    /// regenerates it, as the render goldens).
    fn golden(name: &str, d: MergeDocument) {
        use ratatui::backend::TestBackend;
        use ratatui::Terminal;
        let mut view = MergeView::new(crate::merge_view::model::MergeModel::new(d));
        let mut terminal = Terminal::new(TestBackend::new(110, 24)).unwrap();
        terminal
            .draw(|f| {
                view.render(f, f.area(), None);
            })
            .unwrap();
        let buf = terminal.backend().buffer();
        let mut got = String::new();
        for y in 0..buf.area.height {
            let row: String = (0..buf.area.width).map(|x| buf[(x, y)].symbol()).collect();
            got.push_str(row.trim_end());
            got.push('\n');
        }
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src/merge_view/golden")
            .join(format!("{name}.txt"));
        if std::env::var_os("GITPOULPI_UPDATE_GOLDEN").is_some() {
            std::fs::write(&path, &got).unwrap();
            return;
        }
        let want = std::fs::read_to_string(&path)
            .unwrap_or_else(|_| panic!("missing golden {path:?}; GITPOULPI_UPDATE_GOLDEN=1"));
        assert_eq!(got, want, "golden {name} differs");
    }

    #[test]
    fn golden_binary_card() {
        golden(
            "merge_card_binary",
            doc(
                Some(pair(1, BlobContent::Binary)),
                Some(pair(2, BlobContent::Binary)),
                Some(pair(3, BlobContent::Binary)),
            ),
        );
    }

    #[test]
    fn golden_binary_card_in_french() {
        let _lang = gitpoulpi_i18n::override_lang(gitpoulpi_i18n::Lang::Fr);
        let _terms = gitpoulpi_i18n::override_git_terms(false);
        golden(
            "merge_card_binary_fr",
            doc(
                Some(pair(1, BlobContent::Binary)),
                Some(pair(2, BlobContent::Binary)),
                Some(pair(3, BlobContent::Binary)),
            ),
        );
    }

    #[test]
    fn golden_binary_card_in_french_translated_terms() {
        let _lang = gitpoulpi_i18n::override_lang(gitpoulpi_i18n::Lang::Fr);
        let _terms = gitpoulpi_i18n::override_git_terms(true);
        golden(
            "merge_card_binary_fr_local",
            doc(
                Some(pair(1, BlobContent::Binary)),
                Some(pair(2, BlobContent::Binary)),
                Some(pair(3, BlobContent::Binary)),
            ),
        );
    }

    #[test]
    fn golden_delete_modify_card() {
        let t = |n| {
            pair(
                n,
                BlobContent::Text {
                    lines: vec!["x".into()],
                    crlf: false,
                    final_newline: true,
                },
            )
        };
        golden(
            "merge_card_delete_modify",
            doc(Some(t(1)), None, Some(t(3))),
        );
    }

    #[test]
    fn text_conflicts_have_no_card() {
        let t = |n| {
            pair(
                n,
                BlobContent::Text {
                    lines: vec!["x".into()],
                    crlf: false,
                    final_newline: true,
                },
            )
        };
        assert!(Card::detect(&doc(Some(t(1)), Some(t(2)), Some(t(3)))).is_none());
        assert!(Card::detect(&doc(None, Some(t(2)), Some(t(3)))).is_none());
    }
}
