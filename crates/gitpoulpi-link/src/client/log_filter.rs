//! The refs the link's Log reads leave out, shared between the render loop
//! and the link thread.
//!
//! A standing fact like `log_depth`: a refresh already running on the
//! 2-second ticker, a lazy page, the first load must all read the same
//! value, so it is a shared cell and not a message. It carries a
//! **generation** that moves only when the filter really changes: every
//! Log answer is stamped with the generation it was read under, and the
//! window drops an answer from an older one (it is a walk of other tips, so
//! its rows and offsets mean nothing for the current graph).

use std::sync::{Arc, Mutex};

use gitpoulpi_core::domain::RefFilter;

#[derive(Clone, Default)]
pub(crate) struct LogFilterCell(Arc<Mutex<(u64, RefFilter)>>);

impl LogFilterCell {
    /// A cell that starts as `filter`, generation 0.
    pub fn seeded(filter: RefFilter) -> Self {
        LogFilterCell(Arc::new(Mutex::new((0, filter))))
    }

    /// The generation and filter to read under, taken together.
    pub fn snapshot(&self) -> (u64, RefFilter) {
        self.0.lock().unwrap_or_else(|e| e.into_inner()).clone()
    }

    pub fn generation(&self) -> u64 {
        self.0.lock().unwrap_or_else(|e| e.into_inner()).0
    }

    /// Adopts `filter`; true (and a new generation) only when it differs.
    pub fn set(&self, filter: RefFilter) -> bool {
        let mut cell = self.0.lock().unwrap_or_else(|e| e.into_inner());
        if cell.1 == filter {
            return false;
        }
        *cell = (cell.0 + 1, filter);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hiding(name: &str) -> RefFilter {
        RefFilter {
            names: vec![name.into()],
            ..RefFilter::default()
        }
    }

    #[test]
    fn the_generation_moves_only_when_the_filter_changes() {
        let cell = LogFilterCell::seeded(hiding("a"));
        assert_eq!(cell.generation(), 0);
        assert!(!cell.set(hiding("a")), "same filter, same generation");
        assert_eq!(cell.generation(), 0);
        assert!(cell.set(hiding("b")));
        assert_eq!(cell.snapshot(), (1, hiding("b")));
        assert!(cell.set(RefFilter::default()));
        assert_eq!(cell.generation(), 2);
        assert!(!cell.set(RefFilter::default()));
    }
}
