//! The branches of a request found account-wide.
//!
//! GitHub's `/search/issues` and Gitea's `/repos/issues/search` list requests
//! as issues: no head or base ref. They are read afterwards, in one GraphQL
//! call per 100 requests (GitHub) or one detail call per request (Gitea), and
//! remembered per request and `updated_at` for the life of the process, so a
//! refresh asks only about requests it has not seen change. Everything here is
//! best effort: a failure leaves the branches empty, never fails the round.

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use super::{AccountItems, AccountPr};

/// What is remembered about one request.
#[derive(Clone)]
pub(crate) struct Known {
    pub(crate) source: String,
    pub(crate) target: String,
    pub(crate) fork: Option<String>,
}

/// Entries kept before the memory starts over.
const KEEP: usize = 4000;

fn memory() -> &'static Mutex<HashMap<String, Known>> {
    static MEMORY: OnceLock<Mutex<HashMap<String, Known>>> = OnceLock::new();
    MEMORY.get_or_init(Default::default)
}

/// `api root | project # number | updated_at`: a request that changed is read again.
pub(crate) fn key(api: &str, project: &str, number: u64, updated_at: &str) -> String {
    format!("{api}|{project}#{number}|{updated_at}")
}

pub(crate) fn recall(key: &str) -> Option<Known> {
    memory().lock().ok()?.get(key).cloned()
}

pub(crate) fn remember(key: String, known: Known) {
    if let Ok(mut map) = memory().lock() {
        if map.len() >= KEEP {
            map.clear();
        }
        map.insert(key, known);
    }
}

/// Fills the branches the memory knows; returns the indexes of the requests
/// still without any.
pub(crate) fn fill_known(api: &str, items: &mut AccountItems) -> Vec<usize> {
    let mut missing = Vec::new();
    for (i, row) in items.prs.iter_mut().enumerate() {
        if !row.pr.source_branch.is_empty() {
            continue;
        }
        let k = key(api, &row.project, row.pr.number, &row.pr.updated_at);
        match recall(&k) {
            Some(known) => apply(row, &known),
            None => missing.push(i),
        }
    }
    missing
}

pub(crate) fn apply(row: &mut AccountPr, known: &Known) {
    row.pr.source_branch = known.source.clone();
    row.pr.target_branch = known.target.clone();
    row.pr.source_project = known.fork.clone();
}
