//! Moving an annotated tag: the one pure step both backends share, so the
//! tag object they write is byte-identical.

use crate::{Error, Result};

/// The raw bytes of the annotated tag object `raw` (as `git cat-file tag`
/// prints it) re-pointed at the commit `to_hex`.
///
/// Decision (2026-11): moving a tag is not re-tagging. The message and the
/// whole `tagger` line (name, email, date) are kept verbatim; only the
/// `object` line changes, so the annotation stays the original author's
/// statement and a move is cheap to recognise. Refused, with nothing
/// written: a tag of something that is not a commit, and a signed tag
/// (the signature covers the old `object` line and would no longer verify).
pub fn retarget_tag_object(raw: &[u8], to_hex: &str) -> Result<Vec<u8>> {
    let refuse = |why: &str| Error::Backend(format!("cannot move this tag: {why}"));
    let end = raw
        .windows(2)
        .position(|w| w == b"\n\n")
        .map_or(raw.len(), |i| i + 1);
    let (header, body) = raw.split_at(end);
    let mut lines = header.split(|b| *b == b'\n');
    let first = lines.next().unwrap_or_default();
    let second = lines.next().unwrap_or_default();
    if !first.starts_with(b"object ") {
        return Err(refuse("its object is unreadable"));
    }
    if second != b"type commit" {
        return Err(refuse("it does not point at a commit"));
    }
    let contains = |needle: &[u8]| body.windows(needle.len()).any(|w| w == needle);
    if contains(b"-----BEGIN PGP SIGNATURE-----")
        || contains(b"-----BEGIN SSH SIGNATURE-----")
        || contains(b"-----BEGIN SIGNED MESSAGE-----")
    {
        return Err(refuse(
            "it is signed and the signature would no longer match",
        ));
    }
    let mut out = format!("object {to_hex}\n").into_bytes();
    out.extend_from_slice(&header[first.len() + 1..]);
    out.extend_from_slice(body);
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    const OLD: &str = "1111111111111111111111111111111111111111";
    const NEW: &str = "2222222222222222222222222222222222222222";

    fn tag(body: &str) -> Vec<u8> {
        format!("object {OLD}\ntype commit\ntag v1\ntagger A <a@x> 1700000000 +0200\n\n{body}")
            .into_bytes()
    }

    #[test]
    fn only_the_object_line_changes() {
        let out = retarget_tag_object(&tag("release one\n"), NEW).unwrap();
        assert_eq!(
            String::from_utf8(out).unwrap(),
            format!(
                "object {NEW}\ntype commit\ntag v1\ntagger A <a@x> 1700000000 +0200\n\nrelease one\n"
            )
        );
    }

    #[test]
    fn a_signed_tag_or_a_tag_of_a_tree_is_refused() {
        assert!(retarget_tag_object(&tag("x\n-----BEGIN PGP SIGNATURE-----\nabc\n"), NEW).is_err());
        let tree = format!("object {OLD}\ntype tree\ntag t\ntagger A <a@x> 1 +0000\n\nm\n");
        assert!(retarget_tag_object(tree.as_bytes(), NEW).is_err());
    }
}
