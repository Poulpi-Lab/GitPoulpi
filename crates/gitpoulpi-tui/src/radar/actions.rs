//! The Radar's Actions column: small buttons at the right of a request or
//! issue row. A primary button (Review on a request waiting for the user,
//! Start work on a cloned issue), one that opens the page on the host (the
//! remote glyph) and one that opens the conversation. They are the clicks of
//! the keys `e` / `n`, `o` and `i` (and of the palette), so a mouse action
//! always has its keyboard twin.
//!
//! Slots are fixed per frame (a row without a button leaves its cell empty),
//! so the same button sits in the same column on every row. Narrow, the
//! primary button's word becomes a glyph before any other column goes.

use super::{Act, Item, ItemKind};
use crate::display_width;
use gitpoulpi_i18n::msg;

/// The globe the app already uses for "remote".
const BROWSER_GLYPH: &str = crate::theme::REMOTE_ICON;
const CONVERSATION_GLYPH: &str = "\u{2261}";
const REVIEW_GLYPH: &str = "\u{25CE}";
const START_GLYPH: &str = "\u{2192}";

/// Which button of the column.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Slot {
    Primary,
    Browser,
    Conversation,
}

/// One button of a row.
#[derive(Debug, Clone, Copy)]
pub(super) struct Button {
    pub(super) slot: Slot,
    pub(super) act: Act,
    glyph: &'static str,
    label: &'static str,
}

impl Button {
    /// The text on the button: its word when `icons` is off, else its glyph.
    pub(super) fn text(&self, icons: bool) -> String {
        if self.slot == Slot::Primary && !icons {
            format!(" {} ", self.label)
        } else {
            format!(" {} ", self.glyph)
        }
    }

    /// The key that does the same on the selected row.
    pub(super) fn key(&self) -> char {
        match self.act {
            Act::OpenBrowser => 'o',
            Act::Review => 'e',
            Act::StartWork => 'n',
            _ => 'i',
        }
    }

    /// The tooltip: the action and its key.
    pub(super) fn tip(&self) -> String {
        format!("{} ({})", super::act_label(self.act), self.key())
    }
}

/// The buttons a row offers, in column order.
pub(super) fn buttons(item: &Item) -> Vec<Button> {
    let mut out = Vec::new();
    let cloned = item.repo.is_some();
    match item.kind {
        ItemKind::Pr if item.review && crate::build_features::REVIEW => out.push(Button {
            slot: Slot::Primary,
            act: Act::Review,
            glyph: REVIEW_GLYPH,
            label: msg::radar::ACTION_REVIEW.get(),
        }),
        ItemKind::Issue if cloned => out.push(Button {
            slot: Slot::Primary,
            act: Act::StartWork,
            glyph: START_GLYPH,
            label: msg::radar::ACTION_START_WORK.get(),
        }),
        _ => {}
    }
    if matches!(item.kind, ItemKind::Pr | ItemKind::Issue) && !item.url.is_empty() {
        out.push(Button {
            slot: Slot::Browser,
            act: Act::OpenBrowser,
            glyph: BROWSER_GLYPH,
            label: "",
        });
    }
    // The same rule as the row menu: an issue, or a request not cloned.
    let talks = match item.kind {
        ItemKind::Issue => cloned || !item.integration.is_empty(),
        ItemKind::Pr => !cloned && !item.integration.is_empty(),
        ItemKind::Wip => false,
    };
    if talks && crate::build_features::CONVERSATION {
        out.push(Button {
            slot: Slot::Conversation,
            act: Act::Conversation,
            glyph: CONVERSATION_GLYPH,
            label: "",
        });
    }
    out
}

/// What the column holds on at least one row of the list.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(super) struct Spec {
    /// The widest primary word.
    pub(super) primary_word: usize,
    pub(super) primary: bool,
    pub(super) browser: bool,
    pub(super) conversation: bool,
}

impl Spec {
    pub(super) fn of<'a>(items: impl Iterator<Item = &'a Item>) -> Self {
        let mut spec = Spec::default();
        for item in items {
            for b in buttons(item) {
                match b.slot {
                    Slot::Primary => {
                        spec.primary = true;
                        spec.primary_word = spec.primary_word.max(display_width(b.label));
                    }
                    Slot::Browser => spec.browser = true,
                    Slot::Conversation => spec.conversation = true,
                }
            }
        }
        spec
    }

    pub(super) fn any(&self) -> bool {
        self.primary || self.browser || self.conversation
    }

    /// The width of one slot (a space either side of its text).
    pub(super) fn slot_width(&self, slot: Slot, icons: bool) -> usize {
        match slot {
            Slot::Primary if !icons => self.primary_word + 2,
            Slot::Primary => 3,
            Slot::Browser => display_width(BROWSER_GLYPH) + 2,
            Slot::Conversation => 3,
        }
    }

    /// The slots the column holds, left to right.
    pub(super) fn slots(&self) -> Vec<Slot> {
        let mut out = Vec::new();
        if self.primary {
            out.push(Slot::Primary);
        }
        if self.browser {
            out.push(Slot::Browser);
        }
        if self.conversation {
            out.push(Slot::Conversation);
        }
        out
    }

    /// The column's width: its slots and one cell between them.
    pub(super) fn width(&self, icons: bool) -> usize {
        let slots = self.slots();
        slots
            .iter()
            .map(|s| self.slot_width(*s, icons))
            .sum::<usize>()
            + slots.len().saturating_sub(1)
    }
}
