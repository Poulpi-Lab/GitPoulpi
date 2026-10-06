//! A minimal YAML frontmatter reader/writer for issue notes.
//!
//! Not a YAML parser: it understands exactly what an Obsidian properties
//! block holds (`key: scalar`, `key: [a, b]`, `key:` + `- a` lines) and
//! keeps **everything else byte for byte**. The block is split into
//! segments, one per top-level key (plus any leading comment lines); a
//! segment keeps its original text, including its line endings, and is only
//! regenerated when the key is set through [`Frontmatter::set_scalar`] /
//! [`Frontmatter::set_list`]. Key order never changes.

/// One top-level entry of the block, or the comment lines before the first.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Segment {
    /// `None` for lines before the first key.
    key: Option<String>,
    /// The exact original text, line terminators included.
    text: String,
}

/// The parsed frontmatter block (the lines between the two `---` fences).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Frontmatter {
    segments: Vec<Segment>,
}

/// A file split into its pieces; joining them gives the original bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Document {
    /// A UTF-8 byte order mark, kept as is.
    pub bom: String,
    /// `None` when the file does not open with a frontmatter block.
    pub front: Option<Frontmatter>,
    /// The opening fence line, terminator included (`---\r\n`).
    open: String,
    /// The closing fence line, terminator included (it may have none at EOF).
    close: String,
    /// Everything after the closing fence, untouched.
    pub body: String,
    /// `"\r\n"` when the file uses CRLF, else `"\n"`.
    pub eol: &'static str,
}

fn line_end(text: &str) -> &'static str {
    if text.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    }
}

/// The text of `line` without its terminator.
fn bare(line: &str) -> &str {
    line.trim_end_matches(['\n', '\r'])
}

impl Document {
    /// Splits `text`; never fails (a file without a block has `front: None`).
    pub fn parse(text: &str) -> Document {
        let eol = line_end(text);
        let (bom, rest) = match text.strip_prefix('\u{feff}') {
            Some(r) => ("\u{feff}".to_string(), r),
            None => (String::new(), text),
        };
        let lines: Vec<&str> = rest.split_inclusive('\n').collect();
        let plain = |body: &str| Document {
            bom: bom.clone(),
            front: None,
            open: String::new(),
            close: String::new(),
            body: body.to_string(),
            eol,
        };
        let Some(first) = lines.first() else {
            return plain(rest);
        };
        if bare(first).trim_end() != "---" {
            return plain(rest);
        }
        let Some(close_at) = lines
            .iter()
            .skip(1)
            .position(|l| matches!(bare(l).trim_end(), "---" | "..."))
            .map(|p| p + 1)
        else {
            return plain(rest);
        };
        let block: String = lines[1..close_at].concat();
        Document {
            bom,
            front: Some(Frontmatter::parse(&block)),
            open: (*first).to_string(),
            close: lines[close_at].to_string(),
            body: lines[close_at + 1..].concat(),
            eol,
        }
    }

    /// The file's bytes again.
    pub fn render(&self) -> String {
        let mut out = self.bom.clone();
        if let Some(front) = &self.front {
            out.push_str(&self.open);
            out.push_str(&front.render());
            out.push_str(&self.close);
        }
        out.push_str(&self.body);
        out
    }

    /// Makes sure there is a block to write into, with the file's line
    /// ending.
    pub fn ensure_front(&mut self) -> &mut Frontmatter {
        if self.front.is_none() {
            self.front = Some(Frontmatter::default());
            self.open = format!("---{}", self.eol);
            self.close = format!("---{}", self.eol);
        }
        self.front.as_mut().expect("just set")
    }
}

/// Whether `line` starts a new top-level key, and which one.
fn key_of(line: &str) -> Option<String> {
    let first = line.chars().next()?;
    if first.is_whitespace() || matches!(first, '#' | '-' | '[' | '{' | '"' | '\'' | '?') {
        return None;
    }
    let (key, rest) = line.split_once(':')?;
    // `key:` or `key: value`, never `a:b`.
    if !(rest.is_empty() || rest.starts_with([' ', '\t'])) {
        return None;
    }
    let key = key.trim_end();
    (!key.is_empty()).then(|| key.to_string())
}

impl Frontmatter {
    fn parse(block: &str) -> Frontmatter {
        let mut segments: Vec<Segment> = Vec::new();
        for line in block.split_inclusive('\n') {
            match key_of(bare(line)) {
                Some(key) => segments.push(Segment {
                    key: Some(key),
                    text: line.to_string(),
                }),
                None => match segments.last_mut() {
                    Some(last) => last.text.push_str(line),
                    None => segments.push(Segment {
                        key: None,
                        text: line.to_string(),
                    }),
                },
            }
        }
        Frontmatter { segments }
    }

    pub fn render(&self) -> String {
        self.segments.iter().map(|s| s.text.as_str()).collect()
    }

    /// Top-level keys in file order.
    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.segments.iter().filter_map(|s| s.key.as_deref())
    }

    fn segment(&self, key: &str) -> Option<&Segment> {
        self.segments.iter().find(|s| {
            s.key
                .as_deref()
                .is_some_and(|k| k.eq_ignore_ascii_case(key))
        })
    }

    /// The raw text after `key:` on its first line, comment removed.
    fn inline_value(&self, key: &str) -> Option<String> {
        let seg = self.segment(key)?;
        let first = bare(seg.text.lines().next().unwrap_or(""));
        let (_, value) = first.split_once(':')?;
        Some(strip_comment(value).trim().to_string())
    }

    /// A scalar value (`status: todo`); `None` when the key is absent or empty.
    pub fn scalar(&self, key: &str) -> Option<String> {
        let value = self.inline_value(key)?;
        if value.is_empty() || value.starts_with('[') {
            return None;
        }
        Some(unquote(&value))
    }

    /// A list value: flow (`[a, b]`), block (`- a` lines) or a lone scalar.
    pub fn list(&self, key: &str) -> Vec<String> {
        let Some(seg) = self.segment(key) else {
            return Vec::new();
        };
        let value = self.inline_value(key).unwrap_or_default();
        if let Some(inner) = value.strip_prefix('[') {
            // A flow list may continue on the next lines until `]`.
            let mut joined = inner.to_string();
            if !joined.contains(']') {
                for line in seg.text.lines().skip(1) {
                    joined.push(' ');
                    joined.push_str(strip_comment(line).trim());
                    if joined.contains(']') {
                        break;
                    }
                }
            }
            let inner = joined.split(']').next().unwrap_or("");
            return split_flow(inner);
        }
        if !value.is_empty() {
            return vec![unquote(&value)];
        }
        seg.text
            .lines()
            .skip(1)
            .filter_map(|l| {
                let t = strip_comment(l).trim().to_string();
                let item = t.strip_prefix('-')?;
                let item = item.trim();
                (!item.is_empty()).then(|| unquote(item))
            })
            .collect()
    }

    /// Sets a scalar, keeping the key's place; appends when new. An empty
    /// `value` writes `key:` (a blank property Obsidian shows as empty).
    pub fn set_scalar(&mut self, key: &str, value: &str, eol: &str) {
        let rendered = render_scalar(value);
        let text = if rendered.is_empty() {
            format!("{key}:{eol}")
        } else {
            format!("{key}: {rendered}{eol}")
        };
        self.put(key, text);
    }

    /// Sets a number (`id: 12`), never quoted.
    pub fn set_number(&mut self, key: &str, value: u64, eol: &str) {
        self.put(key, format!("{key}: {value}{eol}"));
    }

    /// Sets a list, keeping the key's place and, when the old value was a
    /// block list, its style. Empty list: `key: []`.
    pub fn set_list(&mut self, key: &str, items: &[String], eol: &str) {
        let block = self.segment(key).is_some_and(|s| {
            self.inline_value(key).unwrap_or_default().is_empty()
                && s.text
                    .lines()
                    .skip(1)
                    .any(|l| l.trim_start().starts_with('-'))
        });
        let text = if items.is_empty() {
            format!("{key}: []{eol}")
        } else if block {
            let mut t = format!("{key}:{eol}");
            for item in items {
                t.push_str(&format!("  - {}{eol}", render_scalar(item)));
            }
            t
        } else {
            let list: Vec<String> = items.iter().map(|i| render_flow_item(i)).collect();
            format!("{key}: [{}]{eol}", list.join(", "))
        };
        self.put(key, text);
    }

    fn put(&mut self, key: &str, text: String) {
        match self.segments.iter_mut().find(|s| {
            s.key
                .as_deref()
                .is_some_and(|k| k.eq_ignore_ascii_case(key))
        }) {
            Some(seg) => seg.text = text,
            None => {
                if let Some(last) = self.segments.last_mut() {
                    if !last.text.ends_with('\n') {
                        last.text.push('\n');
                    }
                }
                self.segments.push(Segment {
                    key: Some(key.to_string()),
                    text,
                });
            }
        }
    }
}

/// `value` without a trailing ` # comment` (outside quotes).
fn strip_comment(value: &str) -> String {
    let mut quote: Option<char> = None;
    let mut prev = ' ';
    for (i, c) in value.char_indices() {
        match quote {
            Some(q) if c == q => quote = None,
            Some(_) => {}
            None if c == '"' || c == '\'' => quote = Some(c),
            None if c == '#' && prev.is_whitespace() => return value[..i].to_string(),
            None => {}
        }
        prev = c;
    }
    value.to_string()
}

fn unquote(value: &str) -> String {
    let v = value.trim();
    if let Some(inner) = v.strip_prefix('"').and_then(|r| r.strip_suffix('"')) {
        let mut out = String::new();
        let mut chars = inner.chars();
        while let Some(c) = chars.next() {
            if c == '\\' {
                match chars.next() {
                    Some('n') => out.push('\n'),
                    Some('t') => out.push('\t'),
                    Some(other) => out.push(other),
                    None => out.push('\\'),
                }
            } else {
                out.push(c);
            }
        }
        return out;
    }
    if let Some(inner) = v.strip_prefix('\'').and_then(|r| r.strip_suffix('\'')) {
        return inner.replace("''", "'");
    }
    v.to_string()
}

/// Splits the inside of `[a, "b, c", d]` on top-level commas.
fn split_flow(inner: &str) -> Vec<String> {
    let mut items = Vec::new();
    let mut cur = String::new();
    let mut quote: Option<char> = None;
    for c in inner.chars() {
        match quote {
            Some(q) => {
                cur.push(c);
                if c == q {
                    quote = None;
                }
            }
            None if c == '"' || c == '\'' => {
                quote = Some(c);
                cur.push(c);
            }
            None if c == ',' => {
                items.push(std::mem::take(&mut cur));
            }
            None => cur.push(c),
        }
    }
    items.push(cur);
    items
        .into_iter()
        .map(|i| unquote(i.trim()))
        .filter(|i| !i.is_empty())
        .collect()
}

/// A scalar as YAML text: plain when safe, double-quoted otherwise.
pub fn render_scalar(value: &str) -> String {
    let v = value.replace(['\r', '\n'], " ");
    let v = v.trim_end();
    if v.is_empty() {
        return String::new();
    }
    if needs_quotes(v, false) {
        quoted(v)
    } else {
        v.to_string()
    }
}

fn render_flow_item(value: &str) -> String {
    if needs_quotes(value, true) {
        quoted(value)
    } else {
        value.to_string()
    }
}

fn quoted(v: &str) -> String {
    format!("\"{}\"", v.replace('\\', "\\\\").replace('"', "\\\""))
}

fn needs_quotes(v: &str, flow: bool) -> bool {
    let first = v.chars().next().unwrap_or(' ');
    let lower = v.to_ascii_lowercase();
    v != v.trim()
        || v.contains(": ")
        || v.ends_with(':')
        || v.contains(" #")
        || (flow && v.contains([',', '[', ']', '{', '}']))
        || matches!(
            first,
            '[' | '{' | '&' | '*' | '!' | '|' | '>' | '\'' | '"' | '%' | '@' | '`' | '#' | ','
        )
        || ((first == '-' || first == '?') && (v.len() == 1 || v[1..].starts_with(' ')))
        || matches!(
            lower.as_str(),
            "true" | "false" | "yes" | "no" | "on" | "off" | "null" | "~"
        )
        || v.parse::<f64>().is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    const NOTE: &str = "---\nid: 12\ntitle: Crash on empty name\nstatus: todo   # free text\nassignees: [kai, \"a b\"]\nlabels:\n  - bug\n  - ui\nmilestone: v0.1\n# my own comment\ncssclasses: wide\ncreated: 2026-10-14\n---\nBody line 1\n\nBody 2\n";

    #[test]
    fn a_note_round_trips_byte_for_byte() {
        let doc = Document::parse(NOTE);
        assert_eq!(doc.render(), NOTE);
        let crlf = NOTE.replace('\n', "\r\n");
        let doc = Document::parse(&crlf);
        assert_eq!(doc.eol, "\r\n");
        assert_eq!(doc.render(), crlf);
    }

    #[test]
    fn values_are_read_leniently() {
        let doc = Document::parse(NOTE);
        let f = doc.front.as_ref().unwrap();
        assert_eq!(f.scalar("id").as_deref(), Some("12"));
        assert_eq!(f.scalar("status").as_deref(), Some("todo"));
        assert_eq!(f.scalar("title").as_deref(), Some("Crash on empty name"));
        assert_eq!(f.list("assignees"), ["kai", "a b"]);
        assert_eq!(f.list("labels"), ["bug", "ui"]);
        assert_eq!(f.list("missing"), Vec::<String>::new());
        assert_eq!(f.scalar("MILESTONE").as_deref(), Some("v0.1"));
        let single = Document::parse("---\nlabels: bug\n---\n");
        assert_eq!(single.front.unwrap().list("labels"), ["bug"]);
    }

    #[test]
    fn setting_a_key_touches_only_that_key() {
        let mut doc = Document::parse(NOTE);
        let eol = doc.eol;
        let f = doc.front.as_mut().unwrap();
        f.set_scalar("status", "doing", eol);
        f.set_list("labels", &["bug".into(), "a: b".into()], eol);
        f.set_list("assignees", &[], eol);
        assert_eq!(
            doc.render(),
            "---\nid: 12\ntitle: Crash on empty name\nstatus: doing\nassignees: []\nlabels:\n  - bug\n  - \"a: b\"\nmilestone: v0.1\n# my own comment\ncssclasses: wide\ncreated: 2026-10-14\n---\nBody line 1\n\nBody 2\n"
        );
    }

    #[test]
    fn a_new_key_is_appended_and_crlf_is_kept() {
        let crlf = NOTE.replace('\n', "\r\n");
        let mut doc = Document::parse(&crlf);
        let eol = doc.eol;
        doc.front
            .as_mut()
            .unwrap()
            .set_scalar("updated", "2026-10-15", eol);
        let out = doc.render();
        assert!(out.contains("created: 2026-10-14\r\nupdated: 2026-10-15\r\n---\r\n"));
        assert!(!out.replace("\r\n", "").contains('\n'), "no bare LF added");
    }

    #[test]
    fn a_note_without_a_block_gets_one_on_demand() {
        let mut doc = Document::parse("# Title\n\ntext\n");
        assert!(doc.front.is_none());
        assert_eq!(doc.render(), "# Title\n\ntext\n");
        let eol = doc.eol;
        doc.ensure_front().set_number("id", 3, eol);
        assert_eq!(doc.render(), "---\nid: 3\n---\n# Title\n\ntext\n");
    }

    #[test]
    fn an_unclosed_block_is_just_a_body() {
        let doc = Document::parse("---\nid: 1\nno close\n");
        assert!(doc.front.is_none());
        assert_eq!(doc.render(), "---\nid: 1\nno close\n");
    }

    #[test]
    fn bom_and_odd_scalars_survive() {
        let text = "\u{feff}---\nid: 1\ntitle: \"Say \\\"hi\\\": now\"\n---\n";
        let doc = Document::parse(text);
        assert_eq!(doc.render(), text);
        assert_eq!(
            doc.front.unwrap().scalar("title").as_deref(),
            Some("Say \"hi\": now")
        );
        assert_eq!(render_scalar("plain"), "plain");
        assert_eq!(render_scalar("a: b"), "\"a: b\"");
        assert_eq!(render_scalar("123"), "\"123\"");
        assert_eq!(render_scalar("true"), "\"true\"");
        assert_eq!(render_scalar("- x"), "\"- x\"");
        assert_eq!(render_scalar("two\nlines"), "two lines");
    }

    #[test]
    fn a_colon_inside_a_value_is_not_a_key() {
        let doc = Document::parse("---\nurl: http://x/y\nnote: a:b\n---\n");
        let f = doc.front.unwrap();
        assert_eq!(f.keys().collect::<Vec<_>>(), ["url", "note"]);
        assert_eq!(f.scalar("url").as_deref(), Some("http://x/y"));
    }
}
