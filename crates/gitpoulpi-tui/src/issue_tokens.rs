//! `#123` / `!45` / `ABC-12` in a commit message: issue and pull-request
//! references.
//!
//! A thin adapter over [`gitpoulpi_hosting::refs`]: the scan itself, and
//! what a token means for the linked sources, live there (shared with the
//! CLI). This module only turns the hits into drawable, clickable links.
//! Unlinked: an empty context, so no token is ever a link.

use crate::th;
use gitpoulpi_hosting::refs::{self, FoundRef};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};

/// What tokens resolve against: the repository's two linked sources.
pub(crate) type TokenBases = refs::RefContext;

/// One resolvable reference in a line of text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Link {
    /// Char offsets into the scanned text, `[start, end)`.
    pub(crate) start: usize,
    pub(crate) end: usize,
    /// What a human types (`#12`, `!4`, `ABC-12`).
    pub(crate) reference: String,
    pub(crate) url: String,
}

impl Link {
    fn from_found(found: FoundRef) -> Option<Self> {
        Some(Self {
            start: found.start,
            end: found.end,
            reference: found.reference,
            url: found.url?,
        })
    }
}

/// The references of `text` that resolve, with their URLs.
pub(crate) fn links(text: &str, bases: &TokenBases) -> Vec<Link> {
    refs::scan_message(text, bases)
        .into_iter()
        .filter_map(Link::from_found)
        .collect()
}

/// One message line as spans, the resolvable `links` underlined in the
/// accent color. `base` is the line's own style; `hot[i]` is whether the
/// pointer is on link `i`, which then takes the app's hover treatment (the
/// lifted background and bold) so a link reads as clickable before the click.
pub(crate) fn styled_line(text: &str, base: Style, links: &[Link], hot: &[bool]) -> Line<'static> {
    if links.is_empty() {
        return Line::from(Span::styled(text.to_string(), base));
    }
    let chars: Vec<char> = text.chars().collect();
    let mut spans = Vec::new();
    let mut at = 0;
    for (n, l) in links.iter().enumerate() {
        if l.start > at {
            spans.push(Span::styled(
                chars[at..l.start].iter().collect::<String>(),
                base,
            ));
        }
        let link = base.fg(th().ui.accent).add_modifier(Modifier::UNDERLINED);
        spans.push(Span::styled(
            chars[l.start..l.end].iter().collect::<String>(),
            if hot.get(n).copied().unwrap_or(false) {
                crate::theme::hover_style(link.add_modifier(Modifier::BOLD))
            } else {
                link
            },
        ));
        at = l.end;
    }
    if at < chars.len() {
        spans.push(Span::styled(chars[at..].iter().collect::<String>(), base));
    }
    Line::from(spans)
}

#[cfg(test)]
mod tests {
    use super::*;
    use gitpoulpi_config::ApiFamily;
    use gitpoulpi_hosting::refs::{IssueLinker, PullLinker};

    fn gitlab_issues_only() -> TokenBases {
        TokenBases {
            issues: Some(IssueLinker {
                family: ApiFamily::Gitlab,
                web_base: "https://h".into(),
                project: "p".into(),
                project_keys: Vec::new(),
                cards: Vec::new(),
            }),
            pulls: None,
        }
    }

    #[test]
    fn unlinked_means_no_links_at_all() {
        assert!(links("fix #1 and !2", &TokenBases::default()).is_empty());
        let l = links("fix #1 and !2", &gitlab_issues_only());
        assert_eq!(l.len(), 1);
        assert_eq!(l[0].url, "https://h/p/-/issues/1");
        assert_eq!(l[0].reference, "#1");
    }

    #[test]
    fn pulls_resolve_too() {
        let mut bases = gitlab_issues_only();
        bases.pulls = Some(PullLinker {
            family: ApiFamily::Gitlab,
            web_base: "https://h".into(),
            project: "p".into(),
        });
        let l = links("a !2", &bases);
        assert_eq!(l[0].url, "https://h/p/-/merge_requests/2");
    }

    /// The link under the pointer takes the hover treatment (lifted
    /// background, bold) and the plain text around it does not; a link at
    /// rest carries neither. The pointer over a commit message's plain text
    /// is the arrow with no style change at all.
    #[test]
    fn the_hovered_link_is_lit_and_the_plain_text_is_not() {
        let l = links("a #1 b", &gitlab_issues_only());
        let rest = styled_line("a #1 b", Style::default(), &l, &[false]);
        let lit = styled_line("a #1 b", Style::default(), &l, &[true]);
        assert_eq!(rest.spans[1].style.bg, None);
        assert_eq!(lit.spans[1].style.bg, Some(th().ui.hover));
        assert!(lit.spans[1].style.add_modifier.contains(Modifier::BOLD));
        assert!(lit.spans[1]
            .style
            .add_modifier
            .contains(Modifier::UNDERLINED));
        assert_eq!(lit.spans[0].style, rest.spans[0].style);
        assert_eq!(lit.spans[2].style, rest.spans[2].style);
        assert_eq!(lit.spans[0].style.bg, None);
    }

    #[test]
    fn styled_lines_underline_only_the_links() {
        let l = links("a #1 b", &gitlab_issues_only());
        let line = styled_line("a #1 b", Style::default(), &l, &[false]);
        let texts: Vec<&str> = line.spans.iter().map(|s| s.content.as_ref()).collect();
        assert_eq!(texts, vec!["a ", "#1", " b"]);
        assert!(line.spans[1]
            .style
            .add_modifier
            .contains(Modifier::UNDERLINED));
        assert!(!line.spans[0]
            .style
            .add_modifier
            .contains(Modifier::UNDERLINED));
    }
}
