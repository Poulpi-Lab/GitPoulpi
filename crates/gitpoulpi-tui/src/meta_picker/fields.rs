//! The metadata a create form (pull request, issue) carries: the chips
//! the form draws, the picker they open and the choices so far.

use std::sync::Arc;

use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use super::load::MetaTarget;
use super::{MetaField, MetaPicker, MetaSelection, PickerOutcome};
use crate::dialog_kit::{self, ButtonState};
use crate::hosting_task::ClientFactory;

/// What the form does with a key or click the picker took.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MetaFieldsOutcome {
    /// Not the picker's: the form handles it.
    Ignored,
    /// The picker used it.
    Handled,
}

#[derive(Default)]
pub(crate) struct MetaFields {
    pub(crate) selection: MetaSelection,
    /// What the choices were made for (`profile|project`): a form that
    /// changes provider or repository starts over, a login of one host
    /// means nothing on another.
    scope: Option<String>,
    pub(super) picker: Option<MetaPicker>,
}

impl MetaFields {
    /// Forgets the choices when the form now points elsewhere.
    pub(crate) fn set_scope(&mut self, scope: String) {
        if self.scope.as_deref() != Some(scope.as_str()) {
            self.selection = MetaSelection::default();
            self.scope = Some(scope);
        }
    }

    /// Whether the choices were made for `scope`.
    pub(crate) fn scope_is(&self, scope: &str) -> bool {
        self.scope.as_deref() == Some(scope)
    }

    pub(crate) fn is_open(&self) -> bool {
        self.picker.is_some()
    }

    /// A load is in flight (the spinner moves).
    pub(crate) fn busy(&self) -> bool {
        self.picker.as_ref().is_some_and(MetaPicker::busy)
    }

    pub(crate) fn open(
        &mut self,
        factory: Arc<dyn ClientFactory>,
        target: MetaTarget,
        field: MetaField,
    ) {
        self.picker = Some(MetaPicker::open(factory, target, field, &self.selection));
    }

    pub(crate) fn poll(&mut self) -> bool {
        self.picker.as_mut().is_some_and(MetaPicker::poll)
    }

    fn outcome(&mut self, outcome: PickerOutcome) {
        match outcome {
            PickerOutcome::Stay => {}
            PickerOutcome::Cancel => self.picker = None,
            PickerOutcome::Apply(applied) => {
                self.selection.apply(applied);
                self.picker = None;
            }
        }
    }

    pub(crate) fn on_key(&mut self, key: KeyEvent) -> MetaFieldsOutcome {
        let Some(picker) = self.picker.as_mut() else {
            return MetaFieldsOutcome::Ignored;
        };
        let outcome = picker.on_key(key);
        self.outcome(outcome);
        MetaFieldsOutcome::Handled
    }

    pub(crate) fn on_mouse(&mut self, mouse: MouseEvent) -> MetaFieldsOutcome {
        let Some(picker) = self.picker.as_mut() else {
            return MetaFieldsOutcome::Ignored;
        };
        let outcome = picker.on_mouse(mouse);
        self.outcome(outcome);
        MetaFieldsOutcome::Handled
    }

    pub(crate) fn paste(&mut self, text: &str) -> bool {
        match self.picker.as_mut() {
            Some(picker) => {
                picker.paste(text);
                true
            }
            None => false,
        }
    }

    /// Paints the open popup over the form (call it last).
    pub(crate) fn render_popup(&mut self, buf: &mut Buffer) {
        if let Some(picker) = self.picker.as_mut() {
            picker.render(buf.area, buf, None);
        }
    }

    /// Paints one row of chips (`Assignees: ann +1`, ...) from `x`, `y`,
    /// at most `width` wide, and returns each chip's rect from this walk.
    /// `focus` and `hover` say which chip lights.
    pub(crate) fn render_chips(
        &self,
        buf: &mut Buffer,
        origin: (u16, u16),
        width: u16,
        fields: &[MetaField],
        focus: Option<MetaField>,
        hover: Option<MetaField>,
    ) -> Vec<(Rect, MetaField)> {
        let (x0, y) = origin;
        let mut x = x0;
        let mut out = Vec::new();
        let right = x0 + width;
        for &field in fields {
            let label = format!("{}: {}", field.chip(), self.selection.summary(field));
            // The last chip may not fit whole: cut its label, keep the button.
            let room = right.saturating_sub(x);
            if room < 8 {
                break;
            }
            let max_label = usize::from(room.saturating_sub(2));
            let label = crate::fit(&label, max_label);
            let rect = dialog_kit::button(
                buf,
                x,
                y,
                &label,
                ButtonState::Normal {
                    hovered: hover == Some(field),
                    focused: focus == Some(field),
                },
            );
            if rect.width == 0 {
                break;
            }
            x = rect.right() + 1;
            out.push((rect, field));
        }
        out
    }
}
