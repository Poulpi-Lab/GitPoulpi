//! The fallback every caller falls back to: a colored initials badge, no
//! network involved.
//!
//! This is what is shown when avatars are off, when the terminal cannot draw
//! images ([`crate::terminal_image::Protocol::None`]: a pane multiplexer, a
//! text-only terminal, `ui.graph_renderer = "text"`), or when a fetch fails
//! or has not completed yet. It is not a degraded mode: initials are the
//! normal look for any author with no Gravatar account, and the
//! color is derived from the author's identity, so the badge is a real visual
//! cue in every terminal.

use ratatui::style::Color;

/// One or two letters standing in for `author_name`.
///
/// - Two words ("Ada Lovelace") -> the first letter of the first and the
///   first letter of the last: `"AL"`.
/// - One word ("cli-bot", "root") -> its first (up to) two letters,
///   uppercased: `"CL"`. Single-word identities are common for bots and CI
///   accounts, and one letter alone reads as a typo.
/// - Empty or whitespace-only -> `"?"`, rather than an empty string a caller
///   would have to special-case.
///
/// Operates on Unicode scalar values via `chars()`, not bytes: an author
/// name is free text and a byte-sliced UTF-8 boundary panics or produces
/// garbage outside ASCII.
pub(crate) fn initials(author_name: &str) -> String {
    let mut words = author_name.split_whitespace();
    let Some(first) = words.next() else {
        return "?".to_string();
    };
    match words.next() {
        Some(last) => {
            let a = first.chars().next();
            let b = last.chars().next();
            match (a, b) {
                (Some(a), Some(b)) => [a, b].into_iter().flat_map(char::to_uppercase).collect(),
                _ => "?".to_string(),
            }
        }
        None => {
            let letters: String = first.chars().take(2).flat_map(char::to_uppercase).collect();
            if letters.is_empty() {
                "?".to_string()
            } else {
                letters
            }
        }
    }
}

/// A fixed, readable-on-dark-and-light palette to pick a badge color from,
/// rather than an arbitrary RGB triple from the hash, which can land too
/// close to the terminal's own background or foreground to read. The badges
/// are explicit `Color::Rgb` (never palette-relative, which the user's theme
/// may render into their background) picked for contrast against both a
/// typical dark and a typical light terminal background.
const PALETTE: [(u8, u8, u8); 8] = [
    (0xE5, 0x73, 0x73), // red
    (0xF0, 0x93, 0x43), // orange
    (0xD4, 0xB1, 0x06), // amber
    (0x66, 0xBB, 0x6A), // green
    (0x4D, 0xB6, 0xAC), // teal
    (0x64, 0x9C, 0xE8), // blue
    (0x9C, 0x7C, 0xE8), // violet
    (0xE0, 0x66, 0xA6), // pink
];

/// A color for `email`, stable across calls and process restarts: a pure
/// function of the bytes, not of insertion order or a `HashMap`'s
/// randomized hasher. FNV-1a rather than `DefaultHasher`, whose algorithm
/// is **not** guaranteed stable across Rust versions ("the same author gets
/// a different color after a `rustc` upgrade").
pub(crate) fn color_for(email: &str) -> Color {
    let normalized = email.trim().to_lowercase();
    let hash = fnv1a(normalized.as_bytes());
    let (r, g, b) = PALETTE[(hash % PALETTE.len() as u64) as usize];
    Color::Rgb(r, g, b)
}

fn fnv1a(bytes: &[u8]) -> u64 {
    const OFFSET: u64 = 0xcbf29ce484222325;
    const PRIME: u64 = 0x100000001b3;
    let mut hash = OFFSET;
    for &byte in bytes {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(PRIME);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_words_take_the_first_letter_of_each() {
        assert_eq!(initials("Ada Lovelace"), "AL");
        assert_eq!(initials("grace hopper"), "GH");
    }

    #[test]
    fn a_middle_name_is_ignored() {
        assert_eq!(initials("Ada Augusta Lovelace"), "AA");
    }

    #[test]
    fn one_word_takes_its_own_first_two_letters() {
        assert_eq!(initials("root"), "RO");
        assert_eq!(initials("x"), "X");
    }

    #[test]
    fn empty_or_blank_falls_back_to_a_question_mark() {
        assert_eq!(initials(""), "?");
        assert_eq!(initials("   "), "?");
    }

    /// Red if the comparison operand is switched to a different string.
    #[test]
    fn non_ascii_names_do_not_panic_and_still_produce_two_chars() {
        assert_eq!(initials("Śebastien Géiser"), "ŚG");
        assert_eq!(initials("北京"), "北京"); // one CJK "word": its own first two chars
    }

    #[test]
    fn color_is_deterministic_for_the_same_email() {
        assert_eq!(
            color_for("person@example.com"),
            color_for("person@example.com")
        );
    }

    #[test]
    fn color_is_case_and_whitespace_insensitive_like_the_gravatar_hash() {
        assert_eq!(
            color_for("Person@Example.com"),
            color_for("  person@example.com  ")
        );
    }

    #[test]
    fn different_emails_usually_land_on_different_colors() {
        // Not a proof (the palette has only 8 slots, so collisions are
        // expected), but five emails covering fewer than five distinct colors
        // would mean the hash barely mixes the input.
        let emails = [
            "alice@example.com",
            "bob@example.com",
            "carol@example.com",
            "dave@example.com",
            "erin@example.com",
        ];
        let distinct: std::collections::HashSet<_> = emails
            .iter()
            .map(|e| format!("{:?}", color_for(e)))
            .collect();
        assert!(distinct.len() >= 3, "colors barely varied: {distinct:?}");
    }

    #[test]
    fn every_palette_color_is_reachable() {
        // Red if PALETTE is shrunk to one entry (only one distinct color
        // reachable).
        let mut seen = std::collections::HashSet::new();
        for i in 0..2000u32 {
            seen.insert(format!("{:?}", color_for(&format!("user{i}@example.com"))));
        }
        assert_eq!(seen.len(), PALETTE.len(), "not every palette slot was hit");
    }
}
