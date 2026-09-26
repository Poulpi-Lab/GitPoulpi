//! An edit of an existing request or issue: the picker, then the call that
//! sends its answer, as one modal the repository dialogs own
//! (`RepoDialog::Meta`).

use std::sync::Arc;
use std::time::Instant;

use crossterm::event::{KeyCode, KeyEvent, MouseEvent};
use gitpoulpi_i18n::msg;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use super::load::{start_edit, EditReceiver, MetaTarget};
use super::{Applied, MetaField, MetaPicker, MetaSelection, PickerOutcome};
use crate::hosting_task::ClientFactory;

/// What the edit modal asks of its owner.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum MetaEditOutcome {
    Stay,
    /// Closed without sending anything.
    Cancel,
    /// The host took the edit: say this, refresh the lists, close.
    Saved(String),
}

pub(crate) struct MetaEdit {
    picker: MetaPicker,
    factory: Arc<dyn ClientFactory>,
    target: MetaTarget,
    sending: Option<(EditReceiver, Instant, Applied)>,
}

impl MetaEdit {
    pub(crate) fn new(
        factory: Arc<dyn ClientFactory>,
        target: MetaTarget,
        field: MetaField,
    ) -> Self {
        let picker = MetaPicker::open(
            factory.clone(),
            target.clone(),
            field,
            &MetaSelection::default(),
        );
        Self {
            picker,
            factory,
            target,
            sending: None,
        }
    }

    /// Waiting for the host: the spinner moves.
    pub(crate) fn busy(&self) -> bool {
        self.sending.is_some() || self.picker.busy()
    }

    /// Drains the load and the edit. `Some` when the edit landed.
    pub(crate) fn poll(&mut self) -> Option<MetaEditOutcome> {
        self.picker.poll();
        let (rx, _, _) = self.sending.as_ref()?;
        let result = rx.try_recv()?;
        let (_, _, applied) = self.sending.take()?;
        match result {
            Ok(()) => Some(MetaEditOutcome::Saved(saved_text(&applied, &self.target))),
            Err(why) => {
                crate::activity_log::report_problem(
                    crate::activity_log::ProblemSource::Hosting,
                    &why,
                );
                self.picker
                    .say(msg::meta_picker::failed(&self.target.caption, &why));
                Some(MetaEditOutcome::Stay)
            }
        }
    }

    fn send(&mut self, applied: Applied) {
        self.picker.say(String::new());
        let rx = start_edit(self.factory.clone(), self.target.clone(), applied.clone());
        self.sending = Some((rx, Instant::now(), applied));
    }

    pub(crate) fn on_key(&mut self, key: KeyEvent) -> MetaEditOutcome {
        if self.sending.is_some() {
            // The call is on its way: Esc leaves (the edit still lands).
            return if key.code == KeyCode::Esc {
                MetaEditOutcome::Cancel
            } else {
                MetaEditOutcome::Stay
            };
        }
        let outcome = self.picker.on_key(key);
        self.outcome(outcome)
    }

    pub(crate) fn on_mouse(&mut self, mouse: MouseEvent) -> MetaEditOutcome {
        if self.sending.is_some() {
            return MetaEditOutcome::Stay;
        }
        let outcome = self.picker.on_mouse(mouse);
        self.outcome(outcome)
    }

    pub(crate) fn paste(&mut self, text: &str) {
        if self.sending.is_none() {
            self.picker.paste(text);
        }
    }

    fn outcome(&mut self, outcome: PickerOutcome) -> MetaEditOutcome {
        match outcome {
            PickerOutcome::Stay => MetaEditOutcome::Stay,
            PickerOutcome::Cancel => MetaEditOutcome::Cancel,
            PickerOutcome::Apply(applied) => {
                self.send(applied);
                MetaEditOutcome::Stay
            }
        }
    }

    pub(crate) fn render(&mut self, area: Rect, buf: &mut Buffer) {
        let sending = self.sending.as_ref().map(|(_, since, _)| *since);
        self.picker.render(area, buf, sending);
    }
}

fn saved_text(applied: &Applied, target: &MetaTarget) -> String {
    let item = &target.caption;
    match applied {
        Applied::List(MetaField::Assignees, _) => msg::meta_picker::saved_assignees(item),
        Applied::List(MetaField::Reviewers, _) => msg::meta_picker::saved_reviewers(item),
        Applied::List(..) => msg::meta_picker::saved_labels(item),
        Applied::Milestone(_) => msg::meta_picker::saved_milestone(item),
    }
}
