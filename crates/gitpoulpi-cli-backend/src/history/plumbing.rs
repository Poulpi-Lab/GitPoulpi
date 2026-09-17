//! Tag and tree objects as bytes: what the engine needs to re-create an
//! annotated tag with a new target, and to rebuild a notes tree.

use gitpoulpi_core::domain::Oid;

use super::objects::Person;

/// Where an annotated tag points: its `object` and `type` headers.
pub(crate) struct TagTarget {
    pub(crate) object: Oid,
    pub(crate) kind: String,
}

/// The end of the header block (the blank line), `None` when there is none.
fn header_end(raw: &[u8]) -> usize {
    raw.windows(2)
        .position(|w| w == b"\n\n")
        .map_or(raw.len(), |p| p + 1)
}

pub(crate) fn tag_target(raw: &[u8]) -> Option<TagTarget> {
    let head = std::str::from_utf8(&raw[..header_end(raw)]).ok()?;
    let mut object = None;
    let mut kind = None;
    for line in head.lines() {
        if let Some(v) = line.strip_prefix("object ") {
            object = Oid::from_hex(v.trim());
        } else if let Some(v) = line.strip_prefix("type ") {
            kind = Some(v.trim().to_string());
        }
    }
    Some(TagTarget {
        object: object?,
        kind: kind?,
    })
}

/// The `tagger` header of an annotated tag, `None` when there is none or the
/// engine does not model it.
pub(crate) fn tagger_of(raw: &[u8]) -> Option<Person> {
    raw[..header_end(raw)]
        .split(|b| *b == b'\n')
        .find_map(|line| line.strip_prefix(b"tagger "))
        .and_then(Person::parse)
}

const SIGNATURE_STARTS: [&[u8]; 3] = [
    b"-----BEGIN PGP SIGNATURE-----",
    b"-----BEGIN SSH SIGNATURE-----",
    b"-----BEGIN SIGNED MESSAGE-----",
];

/// Where a tag message's signature begins: at the start of a line.
fn signature_at(message: &[u8]) -> Option<usize> {
    let mut at = 0;
    loop {
        let rest = &message[at..];
        if SIGNATURE_STARTS.iter().any(|s| rest.starts_with(s)) {
            return Some(at);
        }
        at += rest.iter().position(|b| *b == b'\n')? + 1;
    }
}

/// The tag with a new target. The headers keep their order and bytes, the
/// signature at the end of the message is dropped (it signed the old
/// target); the flag says whether there was one.
pub(crate) fn retarget_tag(
    raw: &[u8],
    new_object: Oid,
    tagger: Option<&Person>,
) -> (Vec<u8>, bool) {
    let end = header_end(raw);
    let mut out = Vec::with_capacity(raw.len());
    let mut first_object = true;
    let mut first_tagger = true;
    for line in raw[..end].split_inclusive(|b| *b == b'\n') {
        if first_object && line.starts_with(b"object ") {
            out.extend_from_slice(b"object ");
            out.extend_from_slice(new_object.to_hex().as_bytes());
            out.push(b'\n');
            first_object = false;
        } else if let (true, true, Some(person)) =
            (first_tagger, line.starts_with(b"tagger "), tagger)
        {
            out.extend_from_slice(b"tagger ");
            out.extend_from_slice(&person.line());
            out.push(b'\n');
            first_tagger = false;
        } else {
            out.extend_from_slice(line);
        }
    }
    let message = raw.get(end + 1..).unwrap_or(&[]);
    if raw.len() > end {
        out.push(b'\n');
    }
    match signature_at(message) {
        Some(at) => {
            out.extend_from_slice(&message[..at]);
            (out, true)
        }
        None => {
            out.extend_from_slice(message);
            (out, false)
        }
    }
}

/// One entry of a tree object.
#[derive(Clone)]
pub(crate) struct TreeEntry {
    pub(crate) mode: Vec<u8>,
    pub(crate) name: Vec<u8>,
    pub(crate) oid: Oid,
}

impl TreeEntry {
    pub(crate) fn is_tree(&self) -> bool {
        self.mode == b"40000"
    }
}

pub(crate) fn parse_tree(raw: &[u8]) -> Option<Vec<TreeEntry>> {
    let mut out = Vec::new();
    let mut pos = 0;
    while pos < raw.len() {
        let space = pos + raw[pos..].iter().position(|b| *b == b' ')?;
        let nul = space + 1 + raw[space + 1..].iter().position(|b| *b == 0)?;
        let id = raw.get(nul + 1..nul + 21)?;
        out.push(TreeEntry {
            mode: raw[pos..space].to_vec(),
            name: raw[space + 1..nul].to_vec(),
            oid: Oid(id.try_into().ok()?),
        });
        pos = nul + 21;
    }
    Some(out)
}

/// A tree object's body; git sorts entries by name with a trailing `/` on
/// directories.
pub(crate) fn serialise_tree(mut entries: Vec<TreeEntry>) -> Vec<u8> {
    let key = |e: &TreeEntry| {
        let mut k = e.name.clone();
        if e.is_tree() {
            k.push(b'/');
        }
        k
    };
    entries.sort_by_key(key);
    let mut out = Vec::new();
    for e in entries {
        out.extend_from_slice(&e.mode);
        out.push(b' ');
        out.extend_from_slice(&e.name);
        out.push(0);
        out.extend_from_slice(&e.oid.0);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tag(message: &str) -> Vec<u8> {
        format!(
            "object {}\ntype commit\ntag v1\ntagger T <t@example.com> 5 +0100\n\n{message}",
            "1".repeat(40)
        )
        .into_bytes()
    }

    #[test]
    fn a_tag_gets_a_new_target_and_loses_its_signature() {
        let new = Oid([2; 20]);
        let (out, signed) = retarget_tag(&tag("hello\n"), new, None);
        assert!(!signed);
        assert_eq!(
            String::from_utf8(out).unwrap(),
            format!(
                "object {}\ntype commit\ntag v1\ntagger T <t@example.com> 5 +0100\n\nhello\n",
                new.to_hex()
            )
        );
        let (out, signed) = retarget_tag(
            &tag("hello\n-----BEGIN PGP SIGNATURE-----\n\nabc\n-----END PGP SIGNATURE-----\n"),
            new,
            None,
        );
        assert!(signed);
        assert!(String::from_utf8(out).unwrap().ends_with("\n\nhello\n"));
        let target = tag_target(&tag("x")).unwrap();
        assert_eq!(
            (target.object, target.kind.as_str()),
            (Oid([0x11; 20]), "commit")
        );
    }

    #[test]
    fn trees_round_trip_in_git_order() {
        let entries = vec![
            TreeEntry {
                mode: b"40000".to_vec(),
                name: b"ab".to_vec(),
                oid: Oid([1; 20]),
            },
            TreeEntry {
                mode: b"100644".to_vec(),
                name: b"ab.txt".to_vec(),
                oid: Oid([2; 20]),
            },
        ];
        let body = serialise_tree(entries);
        let parsed = parse_tree(&body).unwrap();
        // "ab.txt" sorts before the directory "ab/" ('.' < '/').
        assert_eq!(parsed[0].name, b"ab.txt");
        assert_eq!(parsed[1].name, b"ab");
        assert_eq!(serialise_tree(parsed), body);
    }
}
