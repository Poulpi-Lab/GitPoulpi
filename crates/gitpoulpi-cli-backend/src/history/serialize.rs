//! Object ids: git hashes `<type> <len>\0<body>` with SHA-1.

use gitpoulpi_core::domain::Oid;
use sha1::{Digest, Sha1};

/// The id git gives an object of `kind` whose body is `body`.
pub(crate) fn object_id(kind: &str, body: &[u8]) -> Oid {
    let mut hasher = Sha1::new();
    hasher.update(format!("{kind} {}\0", body.len()).as_bytes());
    hasher.update(body);
    Oid(hasher.finalize().into())
}

pub(crate) fn commit_id(body: &[u8]) -> Oid {
    object_id("commit", body)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_framing_is_gits() {
        // git's well known ids for the empty blob and the empty tree.
        assert_eq!(
            object_id("blob", b"").to_hex(),
            "e69de29bb2d1d6434b8b29ae775ad8c2e48c5391"
        );
        assert_eq!(
            object_id("tree", b"").to_hex(),
            "4b825dc642cb6eb9a060e54bf8d69288fbee4904"
        );
    }
}
