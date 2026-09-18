//! Closing the Clone dialog: the cross, and what counts as typed work.

use super::CloneDialog;
use crate::dialog_kit::{CloseHit, Dismiss};

impl Dismiss for CloneDialog {
    fn is_dirty(&self) -> bool {
        // An open destination popup is Back, never "lose the form".
        self.confirm.is_none() && self.has_work()
    }

    fn close_hit(&self) -> &CloseHit {
        match self.confirm {
            Some(_) => &self.confirm_close,
            None => &self.close,
        }
    }

    fn close_hit_mut(&mut self) -> &mut CloseHit {
        match self.confirm {
            Some(_) => &mut self.confirm_close,
            None => &mut self.close,
        }
    }
}
