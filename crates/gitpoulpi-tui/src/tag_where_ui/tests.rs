use super::*;
use crate::test_support::oid;
use gitpoulpi_core::domain::{RefKind, RefVisibility, TagSync};

pub(crate) fn tag(name: &str, at: u8, local: bool, pushed: &[&str], differs: &[&str]) -> Ref {
    Ref {
        kind: RefKind::Tag { name: name.into() },
        target: oid(at),
        visibility: RefVisibility::Shown,
        is_head: false,
        upstream: None,
        tag_sync: Some(TagSync {
            pushed_to: pushed.iter().map(|s| s.to_string()).collect(),
            differs_on: differs.iter().map(|s| s.to_string()).collect(),
            local,
        }),
    }
}

fn four() -> Vec<Ref> {
    vec![
        tag("only-here", 1, true, &[], &[]),
        tag("only-there", 2, false, &["origin"], &[]),
        tag("both", 3, true, &["origin"], &[]),
        tag("moved", 4, true, &[], &["origin"]),
    ]
}

#[test]
fn each_tag_is_told_apart_by_where_it_lives() {
    let snap = snapshot_of(&four());
    assert_eq!(snap["only-here"].place, TagWhere::Local);
    assert_eq!(snap["only-there"].place, TagWhere::Remote);
    assert_eq!(snap["both"].place, TagWhere::Both);
    assert_eq!(snap["moved"].place, TagWhere::Differs);
    assert_eq!(snap["moved"].all_remotes(), vec!["origin".to_string()]);
}

#[test]
fn the_four_markers_are_distinct_text_glyphs() {
    let glyphs: Vec<&str> = [
        TagWhere::Local,
        TagWhere::Remote,
        TagWhere::Both,
        TagWhere::Differs,
    ]
    .into_iter()
    .map(glyph)
    .collect();
    let mut unique = glyphs.clone();
    unique.sort();
    unique.dedup();
    assert_eq!(unique.len(), 4, "{glyphs:?}");
    // The standing glyph rule: never an emoji-presentation codepoint.
    for g in glyphs {
        assert!(!g.contains('\u{FE0F}'), "{g}");
    }
}

#[test]
fn the_tooltip_says_where_in_words_and_what_a_fetch_does() {
    install(&four());
    let local = tooltip_lines("only-here").unwrap().join("\n");
    assert!(local.contains("Only here"), "{local}");
    assert!(
        !local.contains("brings back"),
        "nothing to say about a fetch: {local}"
    );

    let remote = tooltip_lines("only-there").unwrap().join("\n");
    assert!(remote.contains("Only on origin"), "{remote}");
    assert!(remote.contains("brings back"), "{remote}");

    let both = tooltip_lines("both").unwrap().join("\n");
    assert!(
        both.contains("Here and on origin, at the same commit"),
        "{both}"
    );
    assert!(both.contains("Auto-prune"), "the setting is named: {both}");

    let differs = tooltip_lines("moved").unwrap().join("\n");
    assert!(differs.contains("different commits"), "{differs}");

    assert!(tooltip_lines("unknown").is_none());
}

#[test]
fn the_hover_key_follows_the_tag_and_its_place() {
    install(&four());
    set_hover(Some(("both".into(), 3, 4)));
    let both = hover_key().unwrap();
    set_hover(Some(("moved".into(), 3, 4)));
    assert_ne!(hover_key().unwrap(), both);
    set_hover(Some(("not-a-tag".into(), 3, 4)));
    assert_eq!(hover_key(), None, "no tooltip for an unknown name");
    set_hover(None);
    assert!(hover_tooltip().is_none());
}
