//! Laid-out documents, kept per (document id, version, width, theme
//! generation) so a frame only lays out what changed.

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

use super::{Context, Doc};

/// Documents kept; the least recently used goes first.
const CAPACITY: usize = 96;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Key {
    id: String,
    updated: u64,
    width: usize,
    theme: u64,
}

/// The version stamp of a text, for callers whose items carry no
/// `updated_at`.
pub(crate) fn stamp(text: &str) -> u64 {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    text.hash(&mut h);
    h.finish()
}

#[derive(Default)]
pub(crate) struct DocCache {
    docs: HashMap<Key, (Rc<Doc>, u64)>,
    tick: u64,
}

impl DocCache {
    /// The laid-out `text` of document `id` at `width`; `updated` is the
    /// item's version (its `updated_at`, or [`stamp`] of the text).
    pub(crate) fn get(
        &mut self,
        id: &str,
        updated: u64,
        text: &str,
        width: usize,
        ctx: &Context,
    ) -> Rc<Doc> {
        self.tick += 1;
        let key = Key {
            id: id.to_string(),
            updated,
            width,
            theme: crate::theme::theme_generation(),
        };
        if let Some((doc, used)) = self.docs.get_mut(&key) {
            *used = self.tick;
            return Rc::clone(doc);
        }
        if self.docs.len() >= CAPACITY {
            if let Some(old) = self
                .docs
                .iter()
                .min_by_key(|(_, (_, used))| *used)
                .map(|(k, _)| k.clone())
            {
                self.docs.remove(&old);
            }
        }
        let doc = Rc::new(super::render(text, width, ctx));
        self.docs.insert(key, (Rc::clone(&doc), self.tick));
        doc
    }

    /// Like [`DocCache::get`], versioned by the text itself.
    pub(crate) fn get_text(
        &mut self,
        id: &str,
        text: &str,
        width: usize,
        ctx: &Context,
    ) -> Rc<Doc> {
        self.get(id, stamp(text), text, width, ctx)
    }

    pub(crate) fn len(&self) -> usize {
        self.docs.len()
    }
}
