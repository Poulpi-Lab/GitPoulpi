//! "Update commit references in messages": an abbreviated commit id of 7 or
//! more hex characters in a message (`reverts 1a2b3c4`, `see 1a2b3c4d5e`) is
//! replaced by the new id of that commit, cut to the same length. Pure: the
//! caller says what each candidate means ([`HashLookup`]).

use crate::domain::Oid;

/// Shortest abbreviation that is taken for a commit id.
pub const MIN_ABBREV: usize = 7;

/// What the caller knows about a candidate id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HashLookup {
    /// A single commit of the rewritten set: its new full id in hex.
    New(String),
    /// A commit that keeps its id (or has none yet): leave the text.
    Same,
    /// Two or more commits of the set start with it.
    Ambiguous,
    /// No commit of the set starts with it.
    Unknown,
}

/// How a message went.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct HashStats {
    pub replaced: u64,
    /// Candidates left because they were ambiguous, unknown or pointed at a
    /// commit not re-created yet.
    pub left: u64,
}

impl std::ops::AddAssign for HashStats {
    fn add_assign(&mut self, o: Self) {
        self.replaced += o.replaced;
        self.left += o.left;
    }
}

fn word(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

/// `text` with every abbreviated id replaced as `lookup` says. A candidate
/// is a whole word of 7 to 40 lowercase hex characters; words that are only
/// digits are replaced when they match but never counted as left (they are
/// numbers far more often than ids).
pub fn update(text: &str, mut lookup: impl FnMut(&str) -> HashLookup) -> (String, HashStats) {
    let mut out = String::with_capacity(text.len());
    let mut stats = HashStats::default();
    let mut rest = text;
    while !rest.is_empty() {
        let start = rest.find(word).unwrap_or(rest.len());
        out.push_str(&rest[..start]);
        rest = &rest[start..];
        let end = rest.find(|c: char| !word(c)).unwrap_or(rest.len());
        let token = &rest[..end];
        rest = &rest[end..];
        if token.is_empty() {
            continue;
        }
        let candidate = (MIN_ABBREV..=40).contains(&token.len())
            && token
                .bytes()
                .all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'));
        if !candidate {
            out.push_str(token);
            continue;
        }
        match lookup(token) {
            HashLookup::New(full) if full.len() >= token.len() => {
                out.push_str(&full[..token.len()]);
                stats.replaced += 1;
            }
            HashLookup::Same => out.push_str(token),
            _ => {
                out.push_str(token);
                if !token.bytes().all(|b| b.is_ascii_digit()) {
                    stats.left += 1;
                }
            }
        }
    }
    (out, stats)
}

/// The commits of a rewritten set, for prefix lookups.
#[derive(Debug, Clone, Default)]
pub struct IdIndex {
    /// Lowercase hex ids, sorted, with the caller's position of each.
    ids: Vec<(String, usize)>,
}

/// The answer of [`IdIndex::find`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Found {
    One(usize),
    Many,
    None,
}

impl IdIndex {
    pub fn new(ids: impl IntoIterator<Item = Oid>) -> Self {
        let mut ids: Vec<(String, usize)> = ids
            .into_iter()
            .enumerate()
            .map(|(i, o)| (o.to_string(), i))
            .collect();
        ids.sort();
        Self { ids }
    }

    /// The commit whose id starts with `prefix` (lowercase hex).
    pub fn find(&self, prefix: &str) -> Found {
        let at = self.ids.partition_point(|(id, _)| id.as_str() < prefix);
        let hit = |i: usize| self.ids.get(i).filter(|(id, _)| id.starts_with(prefix));
        match (hit(at), hit(at + 1)) {
            (Some(_), Some(_)) => Found::Many,
            (Some((_, i)), None) => Found::One(*i),
            _ => Found::None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn oid(first: &[u8]) -> Oid {
        let mut b = [0u8; 20];
        b[..first.len()].copy_from_slice(first);
        Oid(b)
    }

    fn lookup_for<'a>(
        index: &'a IdIndex,
        new: &'a [&'a str],
    ) -> impl FnMut(&str) -> HashLookup + 'a {
        move |p| match index.find(p) {
            Found::One(i) => HashLookup::New(new[i].to_string()),
            Found::Many => HashLookup::Ambiguous,
            Found::None => HashLookup::Unknown,
        }
    }

    #[test]
    fn a_unique_abbreviation_is_replaced_at_the_same_length() {
        let index = IdIndex::new([
            oid(&[0x1a, 0x2b, 0x3c, 0x4d, 0x5e]),
            oid(&[0x9f, 0x00, 0x11]),
        ]);
        let new = ["ffeeddccbbaa99887766554433221100aabbccdd", "0000000000"];
        let (out, stats) = update(
            "Revert 1a2b3c4d, see also 1a2b3c4d5e and 1a2b3c4d.",
            lookup_for(&index, &new),
        );
        assert_eq!(out, "Revert ffeeddcc, see also ffeeddccbb and ffeeddcc.");
        assert_eq!(
            stats,
            HashStats {
                replaced: 3,
                left: 0
            }
        );
    }

    #[test]
    fn an_ambiguous_abbreviation_is_left_and_counted() {
        // Two commits start with 1a2b3c4.
        let index = IdIndex::new([
            oid(&[0x1a, 0x2b, 0x3c, 0x40]),
            oid(&[0x1a, 0x2b, 0x3c, 0x4f]),
        ]);
        let new = ["a".repeat(40), "b".repeat(40)];
        let new: Vec<&str> = new.iter().map(String::as_str).collect();
        let (out, stats) = update("x 1a2b3c4 y 1a2b3c40 z", lookup_for(&index, &new));
        assert_eq!(out, format!("x 1a2b3c4 y {} z", "a".repeat(8)));
        assert_eq!(
            stats,
            HashStats {
                replaced: 1,
                left: 1
            }
        );
    }

    #[test]
    fn unknown_short_and_embedded_words_are_left() {
        let index = IdIndex::new([oid(&[0x1a, 0x2b, 0x3c, 0x4d])]);
        let new = ["ffffffffffffffffffffffffffffffffffffffff"];
        let text = "abcdef (6 chars) 1a2b3c 1a2b3c4dxyz x1a2b3c4d 1234567 deadbeef";
        let (out, stats) = update(text, lookup_for(&index, &new));
        assert_eq!(out, text);
        // Only `deadbeef` counts: a 7-digit number is a number.
        assert_eq!(
            stats,
            HashStats {
                replaced: 0,
                left: 1
            }
        );
    }

    #[test]
    fn a_commit_that_keeps_its_id_is_not_a_problem() {
        let (out, stats) = update("see 1a2b3c4d", |_| HashLookup::Same);
        assert_eq!(out, "see 1a2b3c4d");
        assert_eq!(stats, HashStats::default());
    }

    #[test]
    fn multibyte_text_is_kept() {
        let (out, _) = update("é 1a2b3c4d ü", |_| HashLookup::New("f".repeat(40)));
        assert_eq!(out, "é ffffffff ü");
    }
}
