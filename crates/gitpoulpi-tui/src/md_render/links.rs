//! Links: bare web addresses and cross references found in plain text, and
//! the registry of targets a document's spans point at.
//!
//! References come from `gitpoulpi_hosting::xrefs::scan` (the host's grammar
//! over the repository's linked sources: `#N`, `!N`, mentions, commit ids,
//! cross-repository references), so the CLI and the app agree on what a token
//! means. Bare addresses the scan does not own go through `message_links`.

use gitpoulpi_hosting::spec::spec_for;
use gitpoulpi_hosting::xrefs::{self, ItemTarget, XrefTarget};

use super::layout::{atoms, Atom};
use super::{Context, Doc, LinkId, LinkTarget, Look};
use gitpoulpi_hosting::refs::RefKind;

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
        xref: None,
        diagram: None,
    });
    Some(doc.links.len() - 1)
}

/// Registers a reference's target. `url` is empty for what only the app can
/// open (a local note).
fn register_xref(doc: &mut Doc, url: String, target: XrefTarget) -> LinkId {
    if let Some(i) = doc
        .links
        .iter()
        .position(|l| l.url == url && l.xref.as_ref() == Some(&target))
    {
        return i;
    }
    doc.links.push(LinkTarget {
        url,
        xref: Some(target),
        diagram: None,
    });
    doc.links.len() - 1
}

/// The page of an item on its host.
fn item_url(item: &ItemTarget) -> Option<String> {
    let spec = spec_for(item.family);
    match item.kind {
        RefKind::PullRequest if item.number > 0 => {
            (spec.pr_web_url)(&item.web_base, &item.project, item.number)
        }
        _ if item.web_base.is_empty() => None,
        _ => {
            let id = if item.number > 0 {
                item.number.to_string()
            } else {
                item.key.trim_start_matches(['#', '!']).to_string()
            };
            (spec.issue_web_url)(&item.web_base, &item.project, &id)
        }
    }
}

/// What one run of characters of the text becomes.
enum Piece {
    /// A link: the address (maybe empty) and what it meant.
    Link(String, Option<XrefTarget>),
    /// Text with a look of its own and no link (a label chip, a mention the
    /// host has no page for).
    Styled(Look),
}

/// The address of a reference's target, when it has one.
fn piece(x: XrefTarget) -> Option<Piece> {
    Some(match x {
        XrefTarget::InApp(ref item) => {
            let url = item_url(item).unwrap_or_default();
            Piece::Link(url, Some(x))
        }
        XrefTarget::Web(ref url) => Piece::Link(url.clone(), Some(x.clone())),
        XrefTarget::Mention {
            url: Some(ref url), ..
        }
        | XrefTarget::Commit {
            url: Some(ref url), ..
        } => Piece::Link(url.clone(), Some(x.clone())),
        XrefTarget::Mention { url: None, .. } => Piece::Styled(Look::Marker),
        XrefTarget::Commit { url: None, .. } => Piece::Styled(Look::Code),
        XrefTarget::Chip { .. } => Piece::Styled(Look::Code),
    })
}

/// `text` as atoms in `look`, its bare addresses and references turned into
/// link atoms.
pub(super) fn linked_atoms(text: &str, look: Look, doc: &mut Doc, ctx: &Context) -> Vec<Atom> {
    let mut found: Vec<(usize, usize, Piece)> = Vec::new();
    for x in xrefs::scan(text, &ctx.xrefs) {
        if let Some(p) = piece(x.target) {
            found.push((x.start, x.end, p));
        }
    }
    for (start, end, url) in crate::message_links::find(text) {
        if found.iter().all(|(s, e, _)| end <= *s || start >= *e) {
            found.push((start, end, Piece::Link(url, None)));
        }
    }
    if found.is_empty() {
        return atoms(text, look, None);
    }
    found.sort_by_key(|(s, e, _)| (*s, std::cmp::Reverse(*e)));
    let chars: Vec<char> = text.chars().collect();
    let mut out: Vec<Atom> = Vec::with_capacity(chars.len());
    let mut at = 0;
    for (start, end, piece) in found {
        if start < at || end > chars.len() {
            continue; // overlaps an earlier link
        }
        let (run_look, id) = match piece {
            Piece::Link(url, target) => {
                let id = match target {
                    Some(t) => Some(register_xref(doc, url, t)),
                    None => register(doc, &url),
                };
                match id {
                    Some(id) => (Look::Link, Some(id)),
                    None => continue,
                }
            }
            Piece::Styled(l) => (l, None),
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
                look: run_look,
                link: id,
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
    if let Some(target) = doc.links.get(id).filter(|t| !t.url.is_empty()) {
        crate::link_confirm::ask(vec![target.url.clone()]);
    }
}
