//! The two dropdowns of the diff pane's bar: the encoding list and the `⋯`
//! list of the buttons the bar had no room for. Both are the combobox
//! (`combo.rs`) hung under the button, modal while open: the keyboard and the
//! mouse are the menu's, a click away closes it and is consumed, and nothing
//! behind it hovers (`ReviewView::takes_palette` says so).

use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use crate::combo::{Combo, ComboOption, ComboOutcome};
use crate::diff_view::{DiffBarButton, DiffViewState};
use crate::editor_charset::Charset;

/// What a menu is a list of.
pub(super) enum BarMenuKind {
    Encoding,
    /// The buttons behind the `⋯`, in bar order.
    Overflow(Vec<DiffBarButton>),
}

/// What an event did to the open menu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum BarMenuOutcome {
    Stay,
    Closed,
    /// An encoding was picked (`None`: Auto-detect).
    Encoding(Option<Charset>),
    /// A hidden button was chosen: run it as its own click.
    Button(DiffBarButton),
}

pub(super) struct BarMenu {
    kind: BarMenuKind,
    combo: Combo,
    anchor: Rect,
}

impl BarMenu {
    pub(super) fn encoding(current: Option<Charset>, anchor: Rect) -> Self {
        Self {
            kind: BarMenuKind::Encoding,
            combo: crate::diff_encoding_menu::combo(current),
            anchor,
        }
    }

    pub(super) fn overflow(
        hidden: Vec<DiffBarButton>,
        state: &DiffViewState,
        anchor: Rect,
    ) -> Self {
        let mut combo = Combo::new("").with_click_picks();
        combo.set_options(
            hidden
                .iter()
                .enumerate()
                .map(|(i, b)| ComboOption::new(b.label_for(state), i.to_string()))
                .collect(),
        );
        combo.open();
        Self {
            kind: BarMenuKind::Overflow(hidden),
            combo,
            anchor,
        }
    }

    fn settle(&mut self, outcome: ComboOutcome) -> BarMenuOutcome {
        let picked = match outcome {
            ComboOutcome::Picked(value) | ComboOutcome::Custom(value) => Some(value),
            _ => None,
        };
        let pick = picked.and_then(|value| match &self.kind {
            BarMenuKind::Encoding => {
                crate::diff_encoding_menu::pick_of(&value).map(BarMenuOutcome::Encoding)
            }
            BarMenuKind::Overflow(hidden) => value
                .parse::<usize>()
                .ok()
                .and_then(|i| hidden.get(i))
                .map(|b| BarMenuOutcome::Button(*b)),
        });
        match pick {
            Some(p) => p,
            None if self.combo.is_open() => BarMenuOutcome::Stay,
            None => BarMenuOutcome::Closed,
        }
    }

    pub(super) fn on_key(&mut self, key: KeyEvent) -> BarMenuOutcome {
        let outcome = self.combo.on_key(key);
        self.settle(outcome)
    }

    pub(super) fn on_mouse(&mut self, mouse: MouseEvent) -> BarMenuOutcome {
        let outcome = self.combo.on_mouse(mouse);
        self.settle(outcome)
    }

    pub(super) fn paste(&mut self, text: &str) {
        self.combo.paste(text);
    }

    /// Paints the popup over the page, hung under its button.
    pub(super) fn render(&mut self, buf: &mut Buffer, bounds: Rect) {
        self.combo.set_anchor(self.anchor);
        self.combo.render_popup(buf, bounds);
    }
}
