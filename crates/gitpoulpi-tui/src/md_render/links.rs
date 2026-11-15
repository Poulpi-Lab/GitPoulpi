//! Links: bare web addresses and repository references found in plain text,
//! and the registry of targets a document's spans point at.
//!
//! TODO: the richer `xrefs` scan (mentions, SHAs, other trackers) replaces
//! the `issue_tokens` pass once the hosting slice lands.

use super::layout::{atoms, Atom};
use super::{Context, Doc, LinkId, LinkTarget, Look};

/// The id of `url` in `doc`, registering it on first use. `None` unless it
/// is a web address (nothing else is ever offered).
pub(super) fn register(doc: &mut Doc, url: &str) -> Option<LinkId> {
    if !crate::message_links::is_web_url(url) {
        return None;
    }
    if let Some(i) = doc.links.iter().position(|l| l.url == url) {
        return Some(i);
    }
    doc.links.push(LinkTarget {
        url: url.to_string(),
    });
    Some(doc.links.len() - 1)
}

/// `text` as atoms in `look`, its bare addresses and references turned into
/// link atoms.
pub(super) fn linked_atoms(text: &str, look: Look, doc: &mut Doc, ctx: &Context) -> Vec<Atom> {
    let mut found: Vec<(usize, usize, String)> = crate::message_links::find(text);
    for l in crate::issue_tokens::links(text, &ctx.bases) {
        found.push((l.start, l.end, l.url));
    }
    if found.is_empty() {
        return atoms(text, look, None);
    }
    found.sort_by_key(|&(s, e, _)| (s, std::cmp::Reverse(e)));
    let chars: Vec<char> = text.chars().collect();
    let mut out: Vec<Atom> = Vec::with_capacity(chars.len());
    let mut at = 0;
    for (start, end, url) in found {
        if start < at || end > chars.len() {
            continue; // overlaps an earlier link
        }
        let Some(id) = register(doc, &url) else {
            continue;
        };
        for &ch in &chars[at..start] {
            out.push(Atom {
                ch,
                look,
                link: None,
            });
        }
        for &ch in &chars[start..end] {
            out.push(Atom {
                ch,
                look: Look::Link,
                link: Some(id),
            });
        }
        at = end;
    }
    for &ch in &chars[at..] {
        out.push(Atom {
            ch,
            look,
            link: None,
        });
    }
    out
}

/// A click or Enter on a link: web addresses go through the "Open this
/// link?" question, which shows the real address first.
pub(crate) fn activate(doc: &Doc, id: LinkId) {
    if let Some(target) = doc.links.get(id) {
        crate::link_confirm::ask(vec![target.url.clone()]);
    }
}
