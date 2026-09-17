//! What the content rules do to one blob. Every unique blob is read and
//! transformed once for the whole run: the answer is cached by blob id
//! (and by path when a rule's globs make the answer depend on it).
//!
//! Text rules skip binary blobs (a NUL in the first 8 KiB; `.gitattributes`
//! at each commit is deliberately not read, it would cost a lookup per
//! commit) and blobs over the rule's size limit, and count them. A pattern
//! rule also skips text that is not UTF-8; a secret literal is replaced
//! byte for byte, so a Latin-1 file is cleaned too.

use std::collections::HashMap;

use gitpoulpi_core::domain::rewrite::textconv::{self, EolAttr, Reencoded};
use gitpoulpi_core::domain::rewrite::{
    CompiledPattern, ContentFind, ContentReplace, EolRule, GlobSet, LineEnding, PathFilter,
    ReencodeRule, RewriteRefusal, TextEncoding, REMOVED_TEXT,
};
use gitpoulpi_core::domain::Oid;
use gitpoulpi_core::{Error, Result};

use super::lfs::{self, Lfs};
use super::pack::{self, Sink};
use super::walk::Reader;

/// The size limit of a text rule that names none.
pub(super) const DEFAULT_MAX_BLOB: u64 = 50 << 20;

/// How many leading bytes tell binary from text.
const SNIFF: usize = 8192;

fn invalid(detail: impl Into<String>) -> Error {
    Error::RewriteRefused(RewriteRefusal::InvalidRule, detail.into())
}

/// A regex that matches exactly `bytes`, whatever they are.
fn escape_bytes(bytes: &[u8]) -> String {
    match std::str::from_utf8(bytes) {
        Ok(text) => regex::escape(text),
        Err(_) => bytes.iter().map(|b| format!("(?-u:\\x{b:02X})")).collect(),
    }
}

pub(super) fn is_binary(bytes: &[u8]) -> bool {
    bytes[..bytes.len().min(SNIFF)].contains(&0)
}

enum Finder {
    Pattern {
        re: CompiledPattern,
        with: String,
    },
    Secrets {
        re: regex::bytes::Regex,
        with: Vec<u8>,
    },
}

impl Finder {
    /// The content after the replacement, `None` when nothing changes.
    fn apply(&self, bytes: &[u8]) -> Option<Vec<u8>> {
        match self {
            Finder::Pattern { re, with } => {
                let text = std::str::from_utf8(bytes).ok()?;
                if !re.is_match(text) {
                    return None;
                }
                let new = re.replace_all(text, with);
                (new != text).then(|| new.into_bytes())
            }
            Finder::Secrets { re, with } => {
                if !re.is_match(bytes) {
                    return None;
                }
                let new = re.replace_all(bytes, regex::bytes::NoExpand(with));
                Some(new.into_owned())
            }
        }
    }

    /// Whether text that is not UTF-8 can be searched.
    fn reads_bytes(&self) -> bool {
        matches!(self, Finder::Secrets { .. })
    }
}

struct Replace {
    bit: u64,
    globs: Option<GlobSet>,
    finder: Finder,
    max: u64,
}

enum Step {
    Replace(Replace),
    Over {
        bit: u64,
        bytes: u64,
    },
    /// S21: matching files become LFS pointers.
    LfsImport {
        bit: u64,
        globs: GlobSet,
        min: u64,
    },
    /// S21: matching pointers become the files again.
    LfsExport {
        bit: u64,
        globs: GlobSet,
    },
    /// S22: matching text files get one line ending.
    Eol {
        bit: u64,
        filter: PathFilter,
        target: LineEnding,
        follow: bool,
        max: u64,
    },
    /// S24bis: matching text files change encoding.
    Reencode {
        bit: u64,
        filter: PathFilter,
        target: TextEncoding,
        from: Option<TextEncoding>,
        fallback: TextEncoding,
        max: u64,
    },
}

/// Loads the blob into `content` unless it is there; `true` when it is over
/// `max` bytes (and so not loaded).
fn load(reader: &mut Reader, content: &mut Option<Vec<u8>>, oid: Oid, max: u64) -> Result<bool> {
    if content.is_none() {
        match reader.blob(oid, max)? {
            (_, Some(body)) => *content = Some(body),
            (_, None) => return Ok(true),
        }
    }
    Ok(content.as_ref().is_some_and(|c| c.len() as u64 > max))
}

/// What happens to a blob.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Decision {
    Keep,
    /// The file leaves the tree (a blob over the size limit).
    Drop,
    New(Oid),
}

/// A blob, whether it is a plain file, its path when an answer depends on it
/// and what `.gitattributes` says.
type CacheKey = (Oid, bool, Option<String>, EolAttr);

pub(super) struct Blobs {
    steps: Vec<Step>,
    /// Some rule's answer depends on the path, so the cache keys on it.
    by_path: bool,
    cache: HashMap<CacheKey, (Decision, u64)>,
    pub(super) changed: u64,
    pub(super) skipped_binary: u64,
    pub(super) skipped_large: u64,
    pub(super) skipped_undecodable: u64,
    pub(super) lfs: Lfs,
}

impl Blobs {
    pub(super) fn new() -> Self {
        Self {
            steps: Vec::new(),
            by_path: false,
            cache: HashMap::new(),
            changed: 0,
            skipped_binary: 0,
            skipped_large: 0,
            skipped_undecodable: 0,
            lfs: Lfs::default(),
        }
    }

    pub(super) fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }

    /// `found` holds the values the scanner found for this rule (see
    /// `scan::resolve`); `None` when the recipe was not resolved.
    pub(super) fn add_replace(
        &mut self,
        bit: u64,
        rule: &ContentReplace,
        found: Option<&[Vec<u8>]>,
    ) -> Result<()> {
        let globs = if rule.paths.is_empty() {
            None
        } else {
            self.by_path = true;
            Some(GlobSet::new(&rule.paths).map_err(invalid)?)
        };
        let finder = match &rule.find {
            ContentFind::Pattern { pattern } => Finder::Pattern {
                re: CompiledPattern::new(pattern).map_err(invalid)?,
                with: rule.with.clone(),
            },
            ContentFind::Secrets { literals, scan } => {
                // With the scanner, a literal shaped like a finding id names
                // a finding (resolved into `found`), it is not text to find.
                let mut values: Vec<Vec<u8>> = Vec::new();
                for l in literals {
                    if *scan && l.finding_id().is_some() {
                        continue;
                    }
                    if l.0.is_empty() {
                        return Err(invalid("a secret value is empty"));
                    }
                    values.push(l.0.as_bytes().to_vec());
                }
                if *scan {
                    let Some(found) = found else {
                        return Err(invalid("the scanner's findings were not resolved"));
                    };
                    values.extend(found.iter().filter(|v| !v.is_empty()).cloned());
                    // Nothing found, nothing to replace.
                    if values.is_empty() {
                        return Ok(());
                    }
                }
                // The longest first, so one that contains another wins.
                values.sort_by_key(|s| std::cmp::Reverse(s.len()));
                values.dedup();
                let alternation = values
                    .iter()
                    .map(|s| escape_bytes(s))
                    .collect::<Vec<_>>()
                    .join("|");
                let re = regex::bytes::RegexBuilder::new(&alternation)
                    .size_limit(1 << 26)
                    .build()
                    // Never echo the pattern: it holds the secrets.
                    .map_err(|_| invalid("the secret values cannot be searched for"))?;
                let with = if rule.with.is_empty() {
                    REMOVED_TEXT.as_bytes().to_vec()
                } else {
                    rule.with.as_bytes().to_vec()
                };
                Finder::Secrets { re, with }
            }
        };
        self.steps.push(Step::Replace(Replace {
            bit,
            globs,
            finder,
            max: rule.max_blob_bytes.unwrap_or(DEFAULT_MAX_BLOB),
        }));
        Ok(())
    }

    pub(super) fn add_over(&mut self, bit: u64, bytes: u64) {
        self.steps.push(Step::Over { bit, bytes });
    }

    pub(super) fn add_lfs_import(&mut self, bit: u64, patterns: &[String], min: u64) -> Result<()> {
        self.by_path = true;
        let globs = GlobSet::new(patterns).map_err(invalid)?;
        self.steps.push(Step::LfsImport { bit, globs, min });
        Ok(())
    }

    pub(super) fn add_lfs_export(&mut self, bit: u64, patterns: &[String]) -> Result<()> {
        self.by_path = true;
        let globs = GlobSet::new(patterns).map_err(invalid)?;
        self.steps.push(Step::LfsExport { bit, globs });
        Ok(())
    }

    pub(super) fn add_eol(&mut self, bit: u64, rule: &EolRule) -> Result<()> {
        let filter = PathFilter::new(&rule.paths, &rule.exclude).map_err(invalid)?;
        self.by_path |= !rule.paths.is_empty() || !rule.exclude.is_empty();
        self.steps.push(Step::Eol {
            bit,
            filter,
            target: rule.target,
            follow: rule.follow_attributes,
            max: rule.max_blob_bytes.unwrap_or(DEFAULT_MAX_BLOB),
        });
        Ok(())
    }

    pub(super) fn add_reencode(&mut self, bit: u64, rule: &ReencodeRule) -> Result<()> {
        let filter = PathFilter::new(&rule.paths, &rule.exclude).map_err(invalid)?;
        self.by_path |= !rule.paths.is_empty() || !rule.exclude.is_empty();
        self.steps.push(Step::Reencode {
            bit,
            filter,
            target: rule.target,
            from: rule.from,
            fallback: rule.fallback,
            max: rule.max_blob_bytes.unwrap_or(DEFAULT_MAX_BLOB),
        });
        Ok(())
    }

    /// The decision for blob `oid` at `path` and the bits of the rules that
    /// acted. `regular` is a plain file (not a symlink); `attr` is what the
    /// commit's `.gitattributes` says about the path (`Unset` unless a rule
    /// follows the attributes).
    pub(super) fn decide(
        &mut self,
        reader: &mut Reader,
        sink: &mut Sink,
        oid: Oid,
        path: &str,
        regular: bool,
        attr: EolAttr,
    ) -> Result<(Decision, u64)> {
        let key = (oid, regular, self.by_path.then(|| path.to_string()), attr);
        if let Some(found) = self.cache.get(&key) {
            return Ok(*found);
        }
        let answer = self.compute(reader, sink, oid, path, regular, attr)?;
        self.cache.insert(key, answer);
        Ok(answer)
    }

    fn compute(
        &mut self,
        reader: &mut Reader,
        sink: &mut Sink,
        oid: Oid,
        path: &str,
        regular: bool,
        attr: EolAttr,
    ) -> Result<(Decision, u64)> {
        let mut content: Option<Vec<u8>> = None;
        let (mut hit, mut changed) = (0u64, false);
        let (mut binary, mut large, mut undecodable) = (false, false, false);
        for step in &self.steps {
            match step {
                Step::Replace(r) => {
                    if !regular || r.globs.as_ref().is_some_and(|g| !g.is_match(path)) {
                        continue;
                    }
                    if content.is_none() {
                        match reader.blob(oid, r.max)? {
                            (_, Some(body)) => content = Some(body),
                            (_, None) => {
                                large = true;
                                continue;
                            }
                        }
                    } else if content.as_ref().is_some_and(|c| c.len() as u64 > r.max) {
                        large = true;
                        continue;
                    }
                    let bytes = content.as_deref().unwrap_or_default();
                    if is_binary(bytes)
                        || (!r.finder.reads_bytes() && std::str::from_utf8(bytes).is_err())
                    {
                        binary = true;
                        continue;
                    }
                    if let Some(new) = r.finder.apply(bytes) {
                        content = Some(new);
                        changed = true;
                        hit |= r.bit;
                    }
                }
                Step::Eol {
                    bit,
                    filter,
                    target,
                    follow,
                    max,
                } => {
                    if !regular || !filter.matches(path) {
                        continue;
                    }
                    let target = match (follow, attr) {
                        (true, EolAttr::Skip) => continue,
                        (true, EolAttr::Force(e)) => e,
                        _ => *target,
                    };
                    if load(reader, &mut content, oid, *max)? {
                        large = true;
                        continue;
                    }
                    let bytes = content.as_deref().unwrap_or_default();
                    if is_binary(bytes) {
                        binary = true;
                        continue;
                    }
                    if let Some(new) = textconv::convert_eol(bytes, target) {
                        content = Some(new);
                        changed = true;
                        hit |= bit;
                    }
                }
                Step::Reencode {
                    bit,
                    filter,
                    target,
                    from,
                    fallback,
                    max,
                } => {
                    if !regular || !filter.matches(path) {
                        continue;
                    }
                    if load(reader, &mut content, oid, *max)? {
                        large = true;
                        continue;
                    }
                    let bytes = content.as_deref().unwrap_or_default();
                    if is_binary(bytes) && !textconv::is_utf16_text(bytes, *from) {
                        binary = true;
                        continue;
                    }
                    match textconv::reencode(bytes, *target, *from, *fallback) {
                        Reencoded::Unchanged => {}
                        Reencoded::New(new) => {
                            content = Some(new);
                            changed = true;
                            hit |= bit;
                        }
                        Reencoded::Undecodable => undecodable = true,
                    }
                }
                Step::Over { bit, bytes } => {
                    let size = match &content {
                        Some(c) => c.len() as u64,
                        None => {
                            let (size, body) = reader.blob(oid, *bytes)?;
                            content = body;
                            size
                        }
                    };
                    if size > *bytes {
                        self.skipped_binary += u64::from(binary);
                        self.skipped_large += u64::from(large);
                        self.skipped_undecodable += u64::from(undecodable);
                        return Ok((Decision::Drop, hit | bit));
                    }
                }
                Step::LfsImport { bit, globs, min } => {
                    if !regular || !globs.is_match(path) {
                        continue;
                    }
                    let found = match &content {
                        Some(bytes) => self.lfs.import_bytes(bytes, *min)?,
                        None => self.lfs.import_blob(reader, oid, *min)?,
                    };
                    if let Some((sha, size)) = found {
                        content = Some(lfs::pointer_blob(&sha, size));
                        changed = true;
                        hit |= bit;
                    }
                }
                Step::LfsExport { bit, globs } => {
                    if !regular || !globs.is_match(path) {
                        continue;
                    }
                    let owned;
                    let head: Option<&[u8]> = match &content {
                        Some(bytes) => Some(bytes.as_slice()),
                        None => {
                            owned = reader.blob(oid, 1024)?.1;
                            owned.as_deref()
                        }
                    };
                    let Some(pointer) = head.and_then(gitpoulpi_core::domain::LfsPointer::parse)
                    else {
                        continue;
                    };
                    if let Some(real) = self.lfs.export(&pointer, path) {
                        content = Some(real);
                        changed = true;
                        hit |= bit;
                    }
                }
            }
        }
        self.skipped_binary += u64::from(binary);
        self.skipped_large += u64::from(large);
        self.skipped_undecodable += u64::from(undecodable);
        if !changed {
            return Ok((Decision::Keep, hit));
        }
        let new = sink.add(pack::BLOB, content.as_deref().unwrap_or_default())?;
        self.changed += 1;
        Ok((Decision::New(new), hit))
    }
}
