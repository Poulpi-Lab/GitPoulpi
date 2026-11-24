//! The open author popup: a search field, the Clear filter row, the authors.

use super::{build_authors, AuthorEntry};
use gitpoulpi_core::domain::CommitNode;
use gitpoulpi_core::fuzzy::{score, Anchor};
use std::collections::BTreeSet;

/// One row of the list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AuthorRow {
    /// Row 0, always shown: drops the whole filter.
    Clear,
    /// An author, by index into the popup's entries.
    Author(usize),
}

/// What `InputMode::AuthorFilter` carries.
#[derive(Debug)]
pub(crate) struct AuthorFilterPopup {
    pub(crate) draft: String,
    /// Index into [`AuthorFilterPopup::rows`].
    pub(crate) selected: usize,
    entries: Vec<AuthorEntry>,
    /// The chosen authors' keys, kept in step with the window's set by the
    /// handlers that toggle (the paint needs them for the ticks).
    pub(crate) chosen: BTreeSet<String>,
    /// One initials badge per author (`ui.avatars` is not off).
    pub(crate) avatars: bool,
}

impl AuthorFilterPopup {
    /// Built once when the popup opens, from the loaded history (the Log's
    /// shown nodes) and the authors already chosen. Opens on the first author.
    pub(crate) fn open(nodes: &[CommitNode], chosen: &BTreeSet<String>, avatars: bool) -> Self {
        let entries = build_authors(nodes, chosen);
        Self {
            draft: String::new(),
            selected: usize::from(!entries.is_empty()),
            entries,
            chosen: chosen.clone(),
            avatars,
        }
    }

    pub(crate) fn entry(&self, index: usize) -> Option<&AuthorEntry> {
        self.entries.get(index)
    }

    /// The Clear row, then the authors matching the field (the palette's
    /// fuzzy scorer over "name email"; best match first, ties keep the
    /// most-commits-first order). An empty field lists everyone.
    pub(crate) fn rows(&self) -> Vec<AuthorRow> {
        let query = self.draft.trim().to_lowercase();
        let mut hits: Vec<(i32, usize)> = self
            .entries
            .iter()
            .enumerate()
            .filter_map(|(i, e)| {
                if query.is_empty() {
                    return Some((0, i));
                }
                let hay = format!("{} {}", e.name, e.email).to_lowercase();
                score(&hay, &query, Anchor::Whole).map(|s| (s, i))
            })
            .collect();
        hits.sort_by_key(|(s, i)| (-*s, *i));
        std::iter::once(AuthorRow::Clear)
            .chain(hits.into_iter().map(|(_, i)| AuthorRow::Author(i)))
            .collect()
    }

    /// The row under the highlight.
    pub(crate) fn highlighted(&self) -> Option<AuthorRow> {
        self.rows().get(self.selected).copied()
    }

    /// The highlight after the field changed: the first author when any
    /// matches, else the Clear row.
    pub(crate) fn field_edited(&mut self) {
        self.selected = usize::from(self.rows().len() > 1);
    }
}
