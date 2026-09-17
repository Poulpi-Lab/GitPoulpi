//! The message-encoding rule (S24): per commit, decide from the `encoding`
//! header and the bytes whether the message (and the names) become UTF-8.
//! Nothing is half done: a commit whose header would go but whose message or
//! names cannot all be decoded is left as it is and counted.

use gitpoulpi_core::domain::rewrite::encoding::{self, Fix};

use super::objects::Person;
use super::rules::Work;

/// What one author / committer line becomes.
enum Who {
    /// Same line.
    Keep,
    /// A decoded or repaired line.
    New(Person),
    /// Cannot be decided.
    Left,
}

/// One identity line given the commit's encoding: with the header going the
/// line is read in the declared encoding; `repair` also repairs mojibake in
/// a line that is UTF-8 already.
fn person(bytes: Option<&[u8]>, label: Option<&[u8]>, header_goes: bool, repair: bool) -> Who {
    let Some(bytes) = bytes else {
        return Who::Keep;
    };
    if bytes.is_ascii() {
        return Who::Keep;
    }
    let fixed = if header_goes {
        encoding::fix(bytes, label)
    } else if repair {
        encoding::fix(bytes, None)
    } else {
        return Who::Keep;
    };
    match fixed {
        Fix::Keep => Who::Keep,
        Fix::Left if header_goes => Who::Left,
        // Names that read like mangled UTF-8 but do not round-trip stay.
        Fix::Left => Who::Keep,
        Fix::Text { text, changed } => {
            if !changed && std::str::from_utf8(bytes).is_ok() {
                return Who::Keep;
            }
            match Person::parse(text.as_bytes()) {
                Some(p) => Who::New(p),
                None => Who::Left,
            }
        }
    }
}

/// Runs the rule on every commit; `(commits changed, commits left)`.
pub(super) fn stage(works: &mut [&mut Work], names: bool) -> (u64, u64) {
    let (mut fixed, mut left) = (0, 0);
    for w in works.iter_mut() {
        let label = w.commit.encoding().map(<[u8]>::to_vec);
        let label = label.as_deref();
        let message = encoding::fix(w.message(), label);
        let text = match message {
            Fix::Keep => None,
            Fix::Left => {
                left += 1;
                continue;
            }
            Fix::Text { text, changed } => Some((text, changed)),
        };
        // The header goes with a decoded message; the names were in the
        // same encoding, so they are decoded too (or the commit stays).
        let header_goes = label.is_some();
        let (author, committer) = (w.commit.author_bytes(), w.commit.committer_bytes());
        let a = person(author, label, header_goes, names);
        let c = person(committer, label, header_goes, names);
        if matches!(a, Who::Left) || matches!(c, Who::Left) {
            left += 1;
            continue;
        }
        let mut hit = header_goes;
        if let Some((text, changed)) = text {
            if changed {
                w.message = Some(text.into_bytes());
                hit = true;
            }
        }
        for (new, bytes, slot, slot0) in [
            (a, author, &mut w.author, &mut w.author0),
            (c, committer, &mut w.committer, &mut w.committer0),
        ] {
            if let Who::New(p) = new {
                hit |= slot.as_ref() != Some(&p);
                // A line that was not UTF-8 had no `Person`: its lossy text
                // is the "before" of the preview.
                if slot0.is_none() {
                    *slot0 =
                        bytes.and_then(|b| Person::parse(String::from_utf8_lossy(b).as_bytes()));
                }
                *slot = Some(p);
            }
        }
        w.drop_encoding = header_goes;
        fixed += u64::from(hit);
    }
    (fixed, left)
}
