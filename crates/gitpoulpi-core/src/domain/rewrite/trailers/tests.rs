use super::*;
use crate::domain::rewrite::message::TextPattern;

fn kv(message: &str) -> Vec<(String, String)> {
    parse(message)
        .into_iter()
        .map(|t| (t.key, t.value))
        .collect()
}

fn pairs(list: &[(&str, &str)]) -> Vec<(String, String)> {
    list.iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

/// Cases whose expected output is what `interpret-trailers --parse` of
/// system git prints (the same messages are run through git in the
/// cli-backend's `trailer_grammar` test, so this table cannot drift).
pub const GRAMMAR: &[(&str, &[(&str, &str)])] = &[
    (
        "subject\n\nbody\n\nSigned-off-by: A <a@x>\nCo-authored-by: B <b@x>\n",
        &[("Signed-off-by", "A <a@x>"), ("Co-authored-by", "B <b@x>")],
    ),
    // The first paragraph is the title, never a trailer block.
    ("Signed-off-by: A <a@x>\n", &[]),
    ("Signed-off-by: A <a@x>\n\nbody\n", &[]),
    ("subject\nSigned-off-by: A <a@x>\n", &[]),
    // Continuation lines fold onto one line.
    (
        "s\n\nb\n\nFoo: bar\n  baz\n qux\nX: y\n",
        &[("Foo", "bar baz qux"), ("X", "y")],
    ),
    // A cherry-pick line belongs to the block but is not printed.
    (
        "s\n\nb\n\nFoo: x\n(cherry picked from commit abc)\n",
        &[("Foo", "x")],
    ),
    // A recognised line lets a third of the block be trailers.
    (
        "s\n\nb\n\nnot a trailer\nnor this\nnor this\nSigned-off-by: A <a@x>\n",
        &[("Signed-off-by", "A <a@x>")],
    ),
    ("s\n\nb\n\nnot a trailer\nFoo: x\n", &[]),
    // The divider ends the message.
    (
        "s\n\nb\n\nFoo: x\n---\n file | 1 +\nBar: y\n",
        &[("Foo", "x")],
    ),
    (
        "s\n\nb\n\nFoo: x\n# comment\nBar: y\n",
        &[("Foo", "x"), ("Bar", "y")],
    ),
    ("s\n\nb\n\nFoo bar: x\n", &[]),
    ("s\n\nb\n\nFoo:x\n", &[("Foo", "x")]),
    ("s\n\nb\n\nFoo:\nBar: y\n", &[("Foo", ""), ("Bar", "y")]),
    ("s\n\nb\n\nFoo=x\nBar: y\n", &[]),
    ("s\n\nb\n\nFoo: x\n\n\n", &[("Foo", "x")]),
    ("s\n\nb\n\nFoo: x", &[("Foo", "x")]),
    ("s\n\nb\n\nFoo : x\n", &[("Foo", "x")]),
    // A line of blanks ends the paragraph: only the last one counts.
    ("s\n\nb\n\nFoo: x\n   \nBar: y\n", &[("Bar", "y")]),
    ("s\n\nb\n\n Foo: x\n", &[]),
    ("s\n\nb\n\n1Foo: x\n_a: y\n", &[]),
    (
        "s\r\n\r\nb\r\n\r\nFoo: x\r\nBar: y\r\n",
        &[("Foo", "x"), ("Bar", "y")],
    ),
];

#[test]
fn the_grammar_cases_match_gits() {
    for (message, expected) in GRAMMAR {
        assert_eq!(kv(message), pairs(expected), "message {message:?}");
    }
}

#[test]
fn add_appends_to_the_block_and_leaves_the_rest_alone() {
    let m = "subject\n\nbody\n\nSigned-off-by: A <a@x>\n";
    assert_eq!(
        add(m, "Co-authored-by", "B <b@x>", TrailerIfExists::Add),
        "subject\n\nbody\n\nSigned-off-by: A <a@x>\nCo-authored-by: B <b@x>\n"
    );
}

#[test]
fn add_without_a_block_makes_one() {
    assert_eq!(
        add(
            "subject\n",
            "Signed-off-by",
            "A <a@x>",
            TrailerIfExists::Add
        ),
        "subject\n\nSigned-off-by: A <a@x>\n"
    );
    assert_eq!(
        add("subject\n\nbody text\n", "K", "v", TrailerIfExists::Add),
        "subject\n\nbody text\n\nK: v\n"
    );
    // No final newline stays none.
    assert_eq!(
        add("subject", "K", "v", TrailerIfExists::Add),
        "subject\n\nK: v"
    );
    // A one-paragraph message is a title, not a block.
    assert_eq!(
        add("Foo: x\n", "K", "v", TrailerIfExists::Add),
        "Foo: x\n\nK: v\n"
    );
    assert_eq!(add("", "K", "v", TrailerIfExists::Add), "K: v\n");
}

#[test]
fn add_keeps_trailing_blank_lines_and_the_divider_after_the_block() {
    assert_eq!(
        add("s\n\nFoo: x\n\n\n", "K", "v", TrailerIfExists::Add),
        "s\n\nFoo: x\nK: v\n\n\n"
    );
    assert_eq!(
        add("s\n\nFoo: x\n---\n f | 1\n", "K", "v", TrailerIfExists::Add),
        "s\n\nFoo: x\nK: v\n---\n f | 1\n"
    );
    assert_eq!(
        add("s\n\nb\n---\n f | 1\n", "K", "v", TrailerIfExists::Add),
        "s\n\nb\n\nK: v\n---\n f | 1\n"
    );
}

#[test]
fn add_follows_the_line_ending_of_the_block() {
    assert_eq!(
        add("s\r\n\r\nFoo: x\r\n", "K", "v", TrailerIfExists::Add),
        "s\r\n\r\nFoo: x\r\nK: v\r\n"
    );
}

#[test]
fn if_exists_add_skips_only_the_identical_trailer() {
    let m = "s\n\nSigned-off-by: A <a@x>\n";
    assert_eq!(add(m, "signed-off-by", "A <a@x>", TrailerIfExists::Add), m);
    assert_eq!(
        add(m, "Signed-off-by", "B <b@x>", TrailerIfExists::Add),
        "s\n\nSigned-off-by: A <a@x>\nSigned-off-by: B <b@x>\n"
    );
}

#[test]
fn if_exists_skip_leaves_a_message_that_has_the_key() {
    let m = "s\n\nSigned-off-by: A <a@x>\n";
    assert_eq!(add(m, "Signed-off-by", "B <b@x>", TrailerIfExists::Skip), m);
    assert_eq!(
        add(m, "Reviewed-by", "B <b@x>", TrailerIfExists::Skip),
        "s\n\nSigned-off-by: A <a@x>\nReviewed-by: B <b@x>\n"
    );
}

#[test]
fn if_exists_replace_swaps_every_trailer_of_the_key() {
    let m = "s\n\nSigned-off-by: A <a@x>\nFoo: 1\nSigned-off-by: C <c@x>\n  more\n";
    assert_eq!(
        add(m, "Signed-off-by", "B <b@x>", TrailerIfExists::Replace),
        "s\n\nFoo: 1\nSigned-off-by: B <b@x>\n"
    );
    // The same single trailer: nothing moves.
    let one = "s\n\nSigned-off-by: B <b@x>\nFoo: 1\n";
    assert_eq!(
        add(one, "Signed-off-by", "B <b@x>", TrailerIfExists::Replace),
        one
    );
    // The only trailer replaced.
    assert_eq!(
        add("s\n\nK: 1\n", "K", "2", TrailerIfExists::Replace),
        "s\n\nK: 2\n"
    );
}

#[test]
fn remove_takes_the_trailer_with_its_continuation_lines() {
    let m = "s\n\nb\n\nFoo: bar\n  baz\nX: y\n";
    assert_eq!(remove(m, "foo", None), "s\n\nb\n\nX: y\n");
    assert_eq!(remove(m, "Nope", None), m);
}

#[test]
fn remove_by_value_pattern() {
    let m = "s\n\nSigned-off-by: A <a@old.org>\nSigned-off-by: B <b@new.org>\n";
    let p = CompiledPattern::new(&TextPattern::literal("@old.org")).unwrap();
    assert_eq!(
        remove(m, "Signed-off-by", Some(&p)),
        "s\n\nSigned-off-by: B <b@new.org>\n"
    );
}

#[test]
fn removing_the_last_trailer_undoes_add() {
    for m in [
        "subject\n",
        "subject\n\nbody\n",
        "subject",
        "s\r\n\r\nb\r\n",
    ] {
        let added = add(m, "K", "v", TrailerIfExists::Add);
        assert_ne!(added, m);
        assert_eq!(remove(&added, "K", None), m, "message {m:?}");
    }
}

#[test]
fn map_text_changes_the_value_only() {
    let m = "s\n\nSigned-off-by: Al <al@old.org>\nCo-authored-by: Bo <bo@old.org>\n";
    let p = CompiledPattern::new(&TextPattern::literal("@old.org")).unwrap();
    assert_eq!(
        map_text(m, "Signed-off-by", &p, "@new.org"),
        "s\n\nSigned-off-by: Al <al@new.org>\nCo-authored-by: Bo <bo@old.org>\n"
    );
    // An `@old.org` in the body is not a trailer.
    let body = "mail al@old.org\n\nSigned-off-by: Al <al@old.org>\n";
    assert_eq!(
        map_text(body, "Signed-off-by", &p, "@new.org"),
        "mail al@old.org\n\nSigned-off-by: Al <al@new.org>\n"
    );
}

#[test]
fn map_identities_rewrites_name_and_email_trailers() {
    let m = "s\n\nSigned-off-by: Al <AL@old.org>\nCo-authored-by: Bo <bo@x.org>\nNote: plain\n";
    let out = map_identities(m, |id| {
        id.email
            .eq_ignore_ascii_case("al@old.org")
            .then(|| Ident::new("Alice", "alice@new.org"))
    });
    assert_eq!(
        out,
        "s\n\nSigned-off-by: Alice <alice@new.org>\nCo-authored-by: Bo <bo@x.org>\nNote: plain\n"
    );
}

#[test]
fn person_values_are_parsed_strictly() {
    assert_eq!(person_of("Al <a@x>"), Some(Ident::new("Al", "a@x")));
    assert_eq!(person_of("<a@x>"), Some(Ident::new("", "a@x")));
    assert_eq!(person_of("Al <a@x> # note"), None);
    assert_eq!(person_of("plain"), None);
    assert_eq!(person_text(&Ident::new("", "a@x")), "<a@x>");
}
