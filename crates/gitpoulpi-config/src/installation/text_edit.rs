//! Edits of `config.toml` as text, so everything outside the edited table
//! keeps its bytes (comments, key order, blank lines). `None` from any
//! function means "this file is shaped in a way the line scanner does not
//! handle" (dotted or inline spelling of the table): the caller falls back to
//! a whole-document rewrite.

/// One `[table]` / `[[table]]` section: its header line up to the next header.
struct Block {
    start: usize,
    end: usize,
    name: Vec<String>,
}

fn unquote(segment: &str) -> String {
    let s = segment.trim();
    if s.len() >= 2
        && ((s.starts_with('"') && s.ends_with('"')) || (s.starts_with('\'') && s.ends_with('\'')))
    {
        s[1..s.len() - 1].to_string()
    } else {
        s.to_string()
    }
}

fn header_name(line: &str) -> Option<Vec<String>> {
    let t = line.trim_start();
    if !t.starts_with('[') {
        return None;
    }
    let inner = t.trim_start_matches('[');
    let inner = &inner[..inner.find(']')?];
    Some(inner.split('.').map(unquote).collect())
}

/// Sections plus the byte offset where the root scope (before the first
/// header) ends. Lines inside multi-line strings are never headers.
fn scan(raw: &str) -> (Vec<Block>, usize) {
    let mut blocks: Vec<Block> = Vec::new();
    let mut in_multi: Option<&str> = None;
    let mut offset = 0;
    let mut root_end = raw.len();
    for line in raw.split_inclusive('\n') {
        let start = offset;
        offset += line.len();
        let header = if in_multi.is_none() {
            header_name(line)
        } else {
            None
        };
        if let Some(name) = header {
            if blocks.is_empty() {
                root_end = start;
            }
            if let Some(last) = blocks.last_mut() {
                last.end = start;
            }
            blocks.push(Block {
                start,
                end: raw.len(),
                name,
            });
            continue;
        }
        for quote in ["\"\"\"", "'''"] {
            let n = line.matches(quote).count();
            match in_multi {
                Some(q) if q == quote && n % 2 == 1 => in_multi = None,
                None if n % 2 == 1 => in_multi = Some(quote),
                _ => {}
            }
        }
    }
    (blocks, root_end)
}

fn line_key(line: &str) -> Option<String> {
    let t = line.trim();
    if t.is_empty() || t.starts_with('#') {
        return None;
    }
    Some(unquote(t.split('=').next()?))
}

/// The root scope spells `table` as a dotted key or an inline table.
fn root_mentions(raw: &str, root_end: usize, table: &str) -> bool {
    raw[..root_end]
        .lines()
        .any(|l| line_key(l).is_some_and(|k| k == table || k.starts_with(&format!("{table}."))))
}

fn append_section(raw: &str, text: &str) -> String {
    let mut out = raw.to_string();
    if !out.is_empty() {
        if !out.ends_with('\n') {
            out.push('\n');
        }
        out.push('\n');
    }
    out.push_str(text);
    out
}

/// Replaces every section of `table` (`[table]`, `[table.x]`) with `new_text`
/// (empty: removes them), at the place of the first one, or at the end of the
/// file when there was none.
pub(super) fn replace_table(raw: &str, table: &str, new_text: &str) -> Option<String> {
    let (blocks, root_end) = scan(raw);
    if root_mentions(raw, root_end, table) {
        return None;
    }
    let mine: Vec<&Block> = blocks
        .iter()
        .filter(|b| b.name.first().is_some_and(|n| n == table))
        .collect();
    if mine.is_empty() {
        return Some(if new_text.is_empty() {
            raw.to_string()
        } else {
            append_section(raw, new_text)
        });
    }
    let mut out = String::with_capacity(raw.len() + new_text.len());
    let mut cursor = 0;
    for (i, b) in mine.iter().enumerate() {
        out.push_str(&raw[cursor..b.start]);
        if i == 0 {
            out.push_str(new_text);
            if !new_text.is_empty() && !new_text.ends_with('\n') {
                out.push('\n');
            }
            if !new_text.is_empty() && b.end < raw.len() {
                out.push('\n');
            }
        }
        cursor = b.end;
    }
    out.push_str(&raw[cursor..]);
    Some(out)
}

/// Sets `key = value` (`value` already TOML text) in the plain `[table]`
/// section: replaces the key's line, else adds one after the section's last
/// non-blank line, else appends a new section.
pub(super) fn set_key(raw: &str, table: &str, key: &str, value: &str) -> Option<String> {
    let (blocks, root_end) = scan(raw);
    if root_mentions(raw, root_end, table) {
        return None;
    }
    let line_text = format!("{key} = {value}\n");
    let Some(b) = blocks.iter().find(|b| b.name == [table]) else {
        return Some(append_section(raw, &format!("[{table}]\n{line_text}")));
    };
    let mut offset = b.start;
    let mut at = None;
    for (i, line) in raw[b.start..b.end].split_inclusive('\n').enumerate() {
        let start = offset;
        offset += line.len();
        if i == 0 {
            at = Some(offset);
        } else if line_key(line).as_deref() == Some(key) {
            return Some(format!("{}{line_text}{}", &raw[..start], &raw[offset..]));
        } else if !line.trim().is_empty() {
            at = Some(offset);
        }
    }
    let at = at.unwrap_or(b.end);
    let sep = if raw[..at].ends_with('\n') { "" } else { "\n" };
    Some(format!("{}{sep}{line_text}{}", &raw[..at], &raw[at..]))
}
