//! Reading the few fields of an X.509 certificate a person compares: the
//! fingerprint of its key, subject, issuer and validity.
//!
//! A small DER walker rather than a certificate crate: it reads a leaf the
//! TLS stack has *already* parsed once, only to display it and to hash its
//! SubjectPublicKeyInfo. It never decides trust; verification stays with
//! webpki (`tls::GuardedVerifier`).

use sha2::{Digest, Sha256};

use super::store::CertInfo;

/// One DER element: tag, the bytes of its value, and what follows it.
struct Tlv<'a> {
    tag: u8,
    /// The whole element, header included.
    full: &'a [u8],
    value: &'a [u8],
    rest: &'a [u8],
}

fn read(input: &[u8]) -> Option<Tlv<'_>> {
    let tag = *input.first()?;
    let first = *input.get(1)?;
    let (len, header) = if first < 0x80 {
        (first as usize, 2)
    } else {
        let n = (first & 0x7f) as usize;
        if n == 0 || n > 4 {
            return None;
        }
        let mut len = 0usize;
        for b in input.get(2..2 + n)? {
            len = (len << 8) | *b as usize;
        }
        (len, 2 + n)
    };
    let end = header.checked_add(len)?;
    Some(Tlv {
        tag,
        full: input.get(..end)?,
        value: input.get(header..end)?,
        rest: input.get(end..)?,
    })
}

/// Lowercase hex.
pub fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// SHA-256 of `bytes`, as lowercase hex.
pub fn sha256_hex(bytes: &[u8]) -> String {
    hex(&Sha256::digest(bytes))
}

fn attribute_name(oid: &[u8]) -> Option<&'static str> {
    match oid {
        [0x55, 0x04, 0x03] => Some("CN"),
        [0x55, 0x04, 0x0a] => Some("O"),
        [0x55, 0x04, 0x0b] => Some("OU"),
        [0x55, 0x04, 0x06] => Some("C"),
        [0x55, 0x04, 0x07] => Some("L"),
        [0x55, 0x04, 0x08] => Some("ST"),
        _ => None,
    }
}

/// `CN=host, O=Org` for a Name.
fn render_name(name: &Tlv) -> String {
    let mut parts = Vec::new();
    let mut sets = name.value;
    while let Some(set) = read(sets) {
        sets = set.rest;
        let mut seqs = set.value;
        while let Some(seq) = read(seqs) {
            seqs = seq.rest;
            let Some(oid) = read(seq.value) else { continue };
            let Some(value) = read(oid.rest) else {
                continue;
            };
            if let Some(label) = attribute_name(oid.value) {
                parts.push(format!("{label}={}", String::from_utf8_lossy(value.value)));
            }
        }
    }
    parts.join(", ")
}

/// `UTCTime` / `GeneralizedTime` as `YYYY-MM-DDTHH:MM:SSZ`.
fn render_time(time: &Tlv) -> String {
    let text = String::from_utf8_lossy(time.value).into_owned();
    let digits = text.trim_end_matches('Z');
    let full = match (time.tag, digits.len()) {
        (0x17, 12) => {
            let yy: u32 = digits[..2].parse().unwrap_or(0);
            let century = if yy >= 50 { "19" } else { "20" };
            format!("{century}{digits}")
        }
        (0x18, 14) => digits.to_string(),
        _ => return text,
    };
    format!(
        "{}-{}-{}T{}:{}:{}Z",
        &full[0..4],
        &full[4..6],
        &full[6..8],
        &full[8..10],
        &full[10..12],
        &full[12..14]
    )
}

impl CertInfo {
    /// The key fingerprint and display fields of a DER certificate, or
    /// `None` when it is not one.
    pub fn from_der(der: &[u8]) -> Option<CertInfo> {
        let cert = read(der)?;
        let tbs = read(cert.value)?;
        let mut rest = tbs.value;
        let mut field = read(rest)?;
        // The optional [0] version.
        if field.tag == 0xa0 {
            rest = field.rest;
            field = read(rest)?;
        }
        // `field` is the serial number now; then the signature algorithm.
        let _serial = field;
        let sig_alg = read(_serial.rest)?;
        let issuer = read(sig_alg.rest)?;
        let validity = read(issuer.rest)?;
        let subject = read(validity.rest)?;
        let spki = read(subject.rest)?;
        let not_before = read(validity.value)?;
        let not_after = read(not_before.rest)?;
        Some(CertInfo {
            spki_sha256: sha256_hex(spki.full),
            subject: render_name(&subject),
            issuer: render_name(&issuer),
            not_before: render_time(&not_before),
            not_after: render_time(&not_after),
        })
    }
}
