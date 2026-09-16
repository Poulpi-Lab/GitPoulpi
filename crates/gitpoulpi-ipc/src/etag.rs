//! Conditional reads: a client that already holds the answer to a `Log` or
//! `Refs` read says so, and the daemon answers "not modified" instead of
//! re-sending megabytes the client would only throw away.
//!
//! Wire shape, compatible both ways without a protocol bump:
//! * the client adds [`IF_NONE_MATCH_KEY`] (the [`of`] hash of the
//!   serialized answer it holds) to the request's `args` object. A daemon
//!   from before this existed ignores the unknown key and answers in full;
//! * a daemon that finds the held answer current replies with
//!   [`NOT_MODIFIED_JSON`]. It only ever does so for a request that carried
//!   the key, so a client from before this existed never sees it.

use serde_json::Value;

/// The `args` key carrying the etag the client holds.
pub const IF_NONE_MATCH_KEY: &str = "if_none_match";

/// The reply meaning "what you hold is current".
pub const NOT_MODIFIED_JSON: &[u8] = br#"{"not_modified":true}"#;

/// The etag of a serialized answer: FNV-1a 64, so client and daemon agree
/// whatever the build (`DefaultHasher` makes no such promise).
pub fn of(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for &b in bytes {
        hash ^= u64::from(b);
        hash = hash.wrapping_mul(0x0100_0000_01b3);
    }
    hash
}

/// The etag of a decoded answer: the hash of its compact serialization,
/// which is exactly the bytes the daemon cached for it.
pub fn of_value(value: &Value) -> u64 {
    of(&serde_json::to_vec(value).unwrap_or_default())
}

/// `args` with the key added (a non-object is left alone).
pub fn with_if_none_match(mut args: Value, etag: u64) -> Value {
    if let Some(map) = args.as_object_mut() {
        map.insert(IF_NONE_MATCH_KEY.into(), Value::from(etag));
    }
    args
}

/// The etag a request's `args` carries, if any.
pub fn if_none_match(args: &Value) -> Option<u64> {
    args.get(IF_NONE_MATCH_KEY)?.as_u64()
}

/// `args` without the key: what identifies the read itself.
pub fn without_if_none_match(args: &Value) -> Value {
    let mut args = args.clone();
    if let Some(map) = args.as_object_mut() {
        map.remove(IF_NONE_MATCH_KEY);
    }
    args
}

/// Whether a decoded result is the "not modified" reply.
pub fn is_not_modified(value: &Value) -> bool {
    value.as_object().is_some_and(|m| {
        m.len() == 1 && m.get("not_modified").and_then(Value::as_bool) == Some(true)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_key_round_trips_and_is_stripped() {
        let args = serde_json::json!({"opts": {"limit": 3}});
        let with = with_if_none_match(args.clone(), 42);
        assert_eq!(if_none_match(&with), Some(42));
        assert_eq!(if_none_match(&args), None);
        assert_eq!(without_if_none_match(&with), args);
    }

    #[test]
    fn the_reply_is_recognized_and_a_real_answer_is_not() {
        let reply: Value = serde_json::from_slice(NOT_MODIFIED_JSON).unwrap();
        assert!(is_not_modified(&reply));
        assert!(!is_not_modified(&serde_json::json!([])));
        assert!(!is_not_modified(&serde_json::json!({"nodes": []})));
    }

    #[test]
    fn the_hash_is_stable() {
        assert_eq!(of(b""), 0xcbf2_9ce4_8422_2325);
        assert_eq!(of(b"a"), 0xaf63_dc4c_8601_ec8c);
    }
}
