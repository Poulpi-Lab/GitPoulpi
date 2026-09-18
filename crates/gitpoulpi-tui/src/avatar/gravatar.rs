//! Building a Gravatar avatar URL, and nothing else -- no network call
//! lives in this file, so every case below is a pure-function test.
//!
//! ## What was verified, and where
//!
//! Checked against the current Gravatar documentation (April-August 2025):
//!
//! - **The hash is SHA-256, not MD5.** `docs.gravatar.com/rest/hash/` and
//!   `docs.gravatar.com/rest/api-data-specifications/` both say, in those
//!   words, *"All Gravatar requests require a proper email hash as
//!   identifier. Don't use MD5!"*, and give the same three steps: trim
//!   leading/trailing whitespace, lowercase the whole address, SHA-256 the
//!   result. `docs.gravatar.com/guides/llms-txt/` (a page Gravatar
//!   maintains specifically to correct assistants relying on older
//!   training data) repeats it under the heading "CRITICAL: Always use
//!   SHA256 for all Gravatar operations" and warns that "information from
//!   before April 2025 is very likely outdated" -- i.e. an MD5 hash,
//!   which is what most existing Gravatar integrations and Stack Overflow
//!   answers still show, is the *legacy* spelling. MD5 is not used here.
//! - **`?d=404` is documented, current behavior**, from
//!   `docs.gravatar.com/sdk/images/`: passing `d=404` makes Gravatar
//!   "not load any image if none is associated with the email hash,
//!   instead return an HTTP 404 (File Not Found) response" rather than a
//!   generated placeholder. That is the *not-found* signal this module
//!   needs -- without it, every author with no Gravatar account would get
//!   a generic mystery-person image indistinguishable from a real one,
//!   and the initials fallback ([`super::initials`]) would never be
//!   reached.
//! - **`?s=`** sets the image's pixel size (square), documented on the
//!   same page.
//!
//! The avatar image endpoint (`gravatar.com/avatar/<hash>`) is separate
//! from Gravatar's authenticated Profile API (`api.gravatar.com/v3/...`)
//! and needs no API key -- confirmed by the same page, and by
//! `docs.gravatar.com/rest/api-data-specifications/`'s explicit "Avatar
//! Endpoint API requests do NOT count towards the [profile] limits".

use sha2::{Digest, Sha256};

/// The identifier Gravatar's docs call "the universal identifier": SHA-256
/// of the trimmed, lowercased email, hex-encoded lowercase.
pub(crate) fn email_hash(email: &str) -> String {
    let normalized = email.trim().to_lowercase();
    let digest = Sha256::digest(normalized.as_bytes());
    let mut out = String::with_capacity(64);
    for byte in digest {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

/// The avatar URL for `email`, requesting a `size`x`size` image and a 404
/// (rather than a generated placeholder) when the author has none. `size`
/// is the raw pixel size to request - callers pick it from the terminal's
/// cell metrics and the supersample factor, the same way the commit graph
/// does (`terminal_image::supersample`).
pub(crate) fn avatar_url(email: &str, size: u32) -> String {
    format!(
        "https://gravatar.com/avatar/{}?s={}&d=404",
        email_hash(email),
        size.clamp(1, 2048)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `email_hash`'s result for the exact address Gravatar's own docs
    /// use as their worked example (`docs.gravatar.com/rest/hash/`,
    /// `docs.gravatar.com/rest/api-data-specifications/`), independently
    /// verified against `printf '%s' "myemailaddress@example.com" |
    /// sha256sum` on this machine rather than trusted from either source
    /// alone. Pins the whole pipeline - trim, lowercase, SHA-256, hex --
    /// against one value nobody can move by editing this file.
    #[test]
    fn matches_gravatars_own_published_worked_example() {
        assert_eq!(
            email_hash("MyEmailAddress@example.com "),
            "84059b07d4be67b806386c0aad8070a23f18836bbaae342275dc0a83414c32ee"
        );
    }

    #[test]
    fn whitespace_and_case_are_normalized_before_hashing() {
        let plain = email_hash("person@example.com");
        assert_eq!(email_hash("  person@example.com  "), plain);
        assert_eq!(email_hash("Person@Example.COM"), plain);
        assert_eq!(email_hash("PERSON@EXAMPLE.COM"), plain);
    }

    /// Falsified: temporarily hashed with MD5 (a 32-hex-char digest)
    /// instead, watched this fail (wrong length, wrong value), restored
    /// SHA-256.
    #[test]
    fn the_hash_is_sha256_not_md5() {
        let hash = email_hash("person@example.com");
        assert_eq!(hash.len(), 64, "SHA-256 hex is 64 characters, MD5 is 32");
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn two_different_emails_hash_differently() {
        assert_ne!(
            email_hash("alice@example.com"),
            email_hash("bob@example.com")
        );
    }

    #[test]
    fn the_url_carries_the_hash_the_size_and_a_404_not_found_default() {
        let url = avatar_url("person@example.com", 32);
        assert!(url.starts_with("https://gravatar.com/avatar/"));
        assert!(url.contains(&email_hash("person@example.com")));
        assert!(url.contains("s=32"));
        assert!(
            url.contains("d=404"),
            "without d=404 a missing avatar answers with a generic \
             placeholder instead of 404, and the initials fallback would \
             never be reached: {url}"
        );
    }

    #[test]
    fn an_absurd_size_is_clamped_rather_than_sent_verbatim() {
        assert!(avatar_url("a@b.com", 0).contains("s=1"));
        assert!(avatar_url("a@b.com", 999_999).contains("s=2048"));
    }
}
