use super::*;

/// Clipping counts characters. A byte-based truncation would panic on
/// the first accented word in a source file.
#[test]
fn clipping_is_by_character_not_byte() {
    assert_eq!(clip("éléphant", 3), "élé");
    assert_eq!(clip("abc", 0), "");
    assert_eq!(clip("a\tb", 3), "a b");
}

/// Falsifies `clip_scrolled` directly: at `h_offset == 0` behavior is
/// unchanged; scrolled past the end shows just the marker rather than
/// panicking or silently returning the untouched string.
#[test]
fn clip_scrolled_matches_clip_at_zero_and_marks_when_scrolled() {
    assert_eq!(clip_scrolled("hello world", 0, 5), clip("hello world", 5));
    assert_eq!(clip_scrolled("hello world", 6, 5), "\u{ab}worl");
    assert_eq!(clip_scrolled("hi", 10, 5), "\u{ab}");
}
