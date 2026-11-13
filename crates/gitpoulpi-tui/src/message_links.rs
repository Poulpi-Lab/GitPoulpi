//! Web addresses typed in a commit message (`https://...`, or the target of a
//! markdown `[text](https://...)`), as clickable links of the Commit Details
//! pane.
//!
//! The terminal's own address detection is not what opens them: with the
//! mouse captured a plain click goes to GitPoulpi, which asks first
//! (`link_confirm`) because the text is untrusted. Only `http` and `https`
//! are ever offered.

use crate::issue_tokens::Link;

/// Char ranges `[start, end)` of the web addresses in `text`, with the
/// address itself.
pub(crate) fn find(text: &str) -> Vec<(usize, usize, String)> {
    let chars: Vec<char> = text.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let Some(scheme) = scheme_at(&chars, i) else {
            i += 1;
            continue;
        };
        // A scheme inside a word (`xhttps://`) is not an address.
        if i > 0 && chars[i - 1].is_alphanumeric() {
            i += 1;
            continue;
        }
        let mut end = i + scheme;
        while end < chars.len() && !is_stop(chars[end]) {
            end += 1;
        }
        end = trim_tail(&chars, i, end);
        if end > i + scheme {
            out.push((i, end, chars[i..end].iter().collect()));
            i = end;
        } else {
            i += scheme;
        }
    }
    out
}

/// Length of `http://` / `https://` at `at` (any case), else `None`.
fn scheme_at(chars: &[char], at: usize) -> Option<usize> {
    for scheme in ["https://", "http://"] {
        let n = scheme.len();
        if at + n <= chars.len()
            && chars[at..at + n]
                .iter()
                .zip(scheme.chars())
                .all(|(c, s)| c.to_ascii_lowercase() == s)
        {
            return Some(n);
        }
    }
    None
}

fn is_stop(c: char) -> bool {
    c.is_whitespace() || c.is_control() || matches!(c, '<' | '>' | '"' | '`' | '|' | '\\')
}

/// Drops the punctuation that ends a sentence or closes a markdown link or
/// a bracket the address did not open.
fn trim_tail(chars: &[char], start: usize, mut end: usize) -> usize {
    while end > start {
        let last = chars[end - 1];
        let open = match last {
            ')' => Some('('),
            ']' => Some('['),
            '}' => Some('{'),
            '.' | ',' | ';' | ':' | '!' | '?' | '\'' | '*' | '_' => None,
            _ => break,
        };
        if let Some(open) = open {
            let opens = chars[start..end].iter().filter(|c| **c == open).count();
            let closes = chars[start..end].iter().filter(|c| **c == last).count();
            if opens >= closes {
                break;
            }
        }
        end -= 1;
    }
    end
}

/// Whether `url` is something the link dialog may offer and open: a web
/// address, nothing else.
pub(crate) fn is_web_url(url: &str) -> bool {
    let chars: Vec<char> = url.chars().collect();
    scheme_at(&chars, 0).is_some_and(|n| chars.len() > n) && !chars.iter().any(|c| is_stop(*c))
}

/// The addresses of every display row, as links per row. `rows` are the
/// wrapped lines with their "a broken word goes on in the next line" flag
/// (`commit_detail::message_display_rows`): an address wider than the pane is
/// found whole and each row it lands on carries its piece.
pub(crate) fn row_links(rows: &[(&str, bool)]) -> Vec<Vec<Link>> {
    let mut out: Vec<Vec<Link>> = vec![Vec::new(); rows.len()];
    let mut first = 0;
    while first < rows.len() {
        let mut last = first;
        while rows[last].1 && last + 1 < rows.len() {
            last += 1;
        }
        // The chain of rows one word was broken over, as one string.
        let mut text = String::new();
        let mut starts = Vec::new();
        for (row, _) in &rows[first..=last] {
            starts.push(text.chars().count());
            text.push_str(row);
        }
        for (start, end, url) in find(&text) {
            for (n, row_start) in starts.iter().enumerate() {
                let row_end = row_start + rows[first + n].0.chars().count();
                let (s, e) = (start.max(*row_start), end.min(row_end));
                if s < e {
                    out[first + n].push(Link {
                        start: s - row_start,
                        end: e - row_start,
                        reference: url.clone(),
                        url: url.clone(),
                        confirm: true,
                    });
                }
            }
        }
        first = last + 1;
    }
    out
}

/// Every distinct address of the rows, in reading order: what the
/// keyboard twin offers.
pub(crate) fn all_urls(rows: &[(&str, bool)]) -> Vec<String> {
    let mut urls: Vec<String> = Vec::new();
    for link in row_links(rows).into_iter().flatten() {
        if !urls.contains(&link.url) {
            urls.push(link.url);
        }
    }
    urls
}

/// `refs` (issue references of one row) without those that overlap an
/// address of the same row, then both merged in column order.
pub(crate) fn merge(refs: Vec<Link>, urls: &[Link]) -> Vec<Link> {
    let mut all: Vec<Link> = refs
        .into_iter()
        .filter(|r| !urls.iter().any(|u| r.start < u.end && u.start < r.end))
        .collect();
    all.extend(urls.iter().cloned());
    all.sort_by_key(|l| l.start);
    all
}

#[cfg(test)]
mod tests {
    use super::*;

    fn urls(text: &str) -> Vec<String> {
        find(text).into_iter().map(|(_, _, u)| u).collect()
    }

    #[test]
    fn plain_and_markdown_addresses_are_found_without_their_punctuation() {
        assert_eq!(urls("see https://a.test/x."), ["https://a.test/x"]);
        assert_eq!(urls("(https://a.test/x), then"), ["https://a.test/x"]);
        assert_eq!(
            urls("Bumps [flate2](https://github.com/rust-lang/flate2-rs) from 1 to 2."),
            ["https://github.com/rust-lang/flate2-rs"]
        );
        assert_eq!(
            urls("http://a.test/w_(x)_y and HTTPS://B.test"),
            ["http://a.test/w_(x)_y", "HTTPS://B.test"]
        );
        assert_eq!(urls("<https://a.test/>"), ["https://a.test/"]);
    }

    #[test]
    fn other_schemes_and_bare_schemes_are_not_addresses() {
        assert!(urls("file:///etc/passwd javascript:alert(1) ftp://x").is_empty());
        assert!(urls("https:// and xhttps://a.test").is_empty());
        assert!(is_web_url("https://a.test"));
        assert!(!is_web_url("javascript:alert(1)"));
        assert!(!is_web_url("https://a b"));
    }

    #[test]
    fn an_address_broken_over_rows_is_one_link_with_a_piece_per_row() {
        let rows = [("see", false), ("https://a.t", true), ("est/path", false)];
        let links = row_links(&rows);
        assert!(links[0].is_empty());
        assert_eq!((links[1][0].start, links[1][0].end), (0, 11));
        assert_eq!((links[2][0].start, links[2][0].end), (0, 8));
        assert_eq!(links[2][0].url, "https://a.test/path");
        assert_eq!(all_urls(&rows), ["https://a.test/path"]);
    }
}
