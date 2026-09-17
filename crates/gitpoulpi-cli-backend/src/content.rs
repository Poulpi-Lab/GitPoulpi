//! `read_blob` and `blame` over the system `git` binary.
//!
//! Both go through `process::run`/`run_bytes`, so the `-c` config
//! overrides and the cleared `GIT_DIR` family apply.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use gitpoulpi_core::domain::{
    Blame, BlameContent, BlameLine, Blob, BlobBody, BlobBytes, BlobContent, BlobSource, LfsPointer,
    Oid, RepoRef, MAX_TEXT_BLOB_BYTES,
};
use gitpoulpi_core::{Error, Result};

use crate::process::{run, run_bytes, run_lenient, run_z_fields};

/// See `GitBackend::read_blob`.
///
/// The size comes from `cat-file -s` (object header only), so an oversized
/// blob is rejected before any content is piped. The working tree is read
/// straight off disk, not through `git show :path`, which would apply
/// smudge filters and return a rendering instead of the editor's bytes.
pub fn read_blob(repo: &RepoRef, path: &Path, source: &BlobSource) -> Result<Blob> {
    let rel = repo_relative(path)?;
    let (size, content, image) = match source {
        BlobSource::WorkingTree => {
            let full = repo.root.join(&rel);
            let size = std::fs::metadata(&full)
                .map_err(|e| Error::Backend(format!("{rel}: {e}")))?
                .len();
            if size > MAX_TEXT_BLOB_BYTES {
                (size, BlobContent::TooLarge, None)
            } else {
                let raw =
                    std::fs::read(&full).map_err(|e| Error::Backend(format!("{rel}: {e}")))?;
                let content = BlobContent::from_bytes(&raw);
                let image = Blob::image_of(&content, &raw);
                (size, content, image)
            }
        }
        // `:<path>` is stage 0 of the index. An unmerged path has no
        // stage 0, and git says so rather than picking a side -- which is
        // the behavior wanted here too.
        BlobSource::Index => read_object(repo, &format!(":{rel}"))?,
        BlobSource::Commit(oid) => read_object(repo, &format!("{}:{rel}", oid.to_hex()))?,
        BlobSource::Stage(n) => read_object(repo, &stage_spec(*n, &rel)?)?,
    };

    Ok(Blob {
        path: PathBuf::from(rel),
        source: source.clone(),
        size,
        content,
        image,
    })
}

/// Header-then-content for one `<rev>:<path>` object spec.
fn read_object(
    repo: &RepoRef,
    spec: &str,
) -> Result<(u64, BlobContent, Option<gitpoulpi_core::domain::ImageInfo>)> {
    let size: u64 = run(&repo.root, &["cat-file", "-s", spec])?
        .trim()
        .parse()
        .map_err(|e| Error::Backend(format!("git cat-file -s {spec} gave no size: {e}")))?;
    if size > MAX_TEXT_BLOB_BYTES {
        return Ok((size, BlobContent::TooLarge, None));
    }
    // `blob` (rather than `-p`) both pins the type -- a directory at this
    // path is an error, not a tree listing -- and keeps git from applying
    // any textconv rendering on the way out.
    let raw = run_bytes(&repo.root, &["cat-file", "blob", spec])?;
    let content = BlobContent::from_bytes(&raw);
    let image = Blob::image_of(&content, &raw);
    Ok((size, content, image))
}

/// `:<n>:<path>`, one stage of an unmerged path.
fn stage_spec(stage: u8, rel: &str) -> Result<String> {
    if (1..=3).contains(&stage) {
        Ok(format!(":{stage}:{rel}"))
    } else {
        Err(Error::Backend(format!("{rel} has no stage {stage}")))
    }
}

/// See `GitBackend::read_blob_bytes`.
pub fn read_blob_bytes(
    repo: &RepoRef,
    path: &Path,
    source: &BlobSource,
    max: u64,
) -> Result<BlobBytes> {
    let rel = repo_relative(path)?;
    let spec = match source {
        BlobSource::WorkingTree => String::new(),
        BlobSource::Index => format!(":{rel}"),
        BlobSource::Commit(oid) => format!("{}:{rel}", oid.to_hex()),
        BlobSource::Stage(n) => stage_spec(*n, &rel)?,
    };
    let size = if let BlobSource::WorkingTree = source {
        std::fs::metadata(repo.root.join(&rel))
            .map_err(|e| Error::Backend(format!("{rel}: {e}")))?
            .len()
    } else {
        run(&repo.root, &["cat-file", "-s", &spec])?
            .trim()
            .parse()
            .map_err(|e| Error::Backend(format!("git cat-file -s {spec} gave no size: {e}")))?
    };
    let body = if size > max {
        BlobBody::TooLarge
    } else {
        let raw = if let BlobSource::WorkingTree = source {
            std::fs::read(repo.root.join(&rel))
                .map_err(|e| Error::Backend(format!("{rel}: {e}")))?
        } else {
            run_bytes(&repo.root, &["cat-file", "blob", &spec])?
        };
        body_of(repo, raw, max)
    };
    Ok(BlobBytes {
        path: PathBuf::from(rel),
        source: source.clone(),
        size,
        body,
    })
}

/// `raw` as a [`BlobBody`]: an LFS pointer is resolved to its local object
/// (shared with the git2 backend so both answer alike).
pub fn body_of(repo: &RepoRef, raw: Vec<u8>, max: u64) -> BlobBody {
    match LfsPointer::parse(&raw) {
        Some(pointer) => {
            let object = lfs_local_object(repo, &pointer, max);
            BlobBody::Lfs { pointer, object }
        }
        None => BlobBody::Bytes(raw),
    }
}

/// The real content of an LFS object when it sits in the local store
/// (`<storage>/objects/aa/bb/<oid>`, `lfs.storage` honoured) and is within
/// `max` bytes. Costs one `git rev-parse` and one `git config` spawn.
pub fn lfs_local_object(repo: &RepoRef, pointer: &LfsPointer, max: u64) -> Option<Vec<u8>> {
    let oid = pointer.oid.as_str();
    if oid.len() < 4 || !oid.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    let common = run(&repo.root, &["rev-parse", "--git-common-dir"]).ok()?;
    let common = repo.root.join(common.trim());
    let configured = run_lenient(&repo.root, &["config", "--get", "lfs.storage"])
        .ok()
        .filter(|(ok, _)| *ok)
        .map(|(_, v)| v.trim().to_string())
        .filter(|v| !v.is_empty());
    let storage = match configured {
        Some(p) => common.join(p),
        None => common.join("lfs"),
    };
    let file = storage
        .join("objects")
        .join(&oid[0..2])
        .join(&oid[2..4])
        .join(oid);
    if std::fs::metadata(&file).ok()?.len() > max {
        return None;
    }
    std::fs::read(file).ok()
}

/// See `GitBackend::blame`.
///
/// `--line-porcelain` (the human format is locale- and width-dependent)
/// repeats the full header per line, so each line parses independently.
/// The *lines* come from `read_blob`, not blame's TAB-prefixed records:
/// blame emits raw bytes, so a CRLF checkout would carry a trailing `\r`
/// that the file viewer (`BlobContent::from_bytes`) does not.
pub fn blame(repo: &RepoRef, path: &Path, at: Oid) -> Result<Blame> {
    let rel = repo_relative(path)?;
    let blob = read_blob(repo, path, &BlobSource::Commit(at))?;
    let lines = match &blob.content {
        BlobContent::Text { lines, .. } => lines.clone(),
        BlobContent::Binary => {
            return Ok(Blame {
                path: blob.path,
                at,
                size: blob.size,
                content: BlameContent::Binary,
            })
        }
        BlobContent::TooLarge => {
            return Ok(Blame {
                path: blob.path,
                at,
                size: blob.size,
                content: BlameContent::TooLarge,
            })
        }
        BlobContent::LfsPointer(pointer) => {
            return Ok(Blame {
                path: blob.path,
                at,
                size: blob.size,
                content: BlameContent::LfsPointer(pointer.clone()),
            })
        }
    };

    // `run` validates UTF-8, and blame echoes the file's own bytes back
    // in its `\t` records -- but a file that is not valid UTF-8 has
    // already returned `Binary` above, so anything reaching here decodes.
    let rev = at.to_hex();
    let raw = run_bytes(
        &repo.root,
        &[
            "blame",
            "--line-porcelain",
            // A `textconv` filter would have blame attribute lines of a
            // *converted* rendering (the text of a PDF, say) that have no
            // relationship to the lines the file viewer shows.
            "--no-textconv",
            "--no-ext-diff",
            &rev,
            "--",
            &rel,
        ],
    )?;
    // Only the attribution is used from this, never the file's own text
    // (`\t` records), so a non-UTF-8 file must not fail it.
    let raw = String::from_utf8_lossy(&raw);

    let entries = parse_line_porcelain(&raw);
    if entries.len() < lines.len() {
        return Err(Error::Backend(format!(
            "git blame returned {} lines for {rel}, but the file at {rev} has {}",
            entries.len(),
            lines.len()
        )));
    }

    let out = lines
        .into_iter()
        .zip(entries)
        .map(|(content, entry)| BlameLine {
            commit: entry.commit,
            author_name: entry.author_name,
            author_email: entry.author_email,
            timestamp: entry.timestamp,
            committer_time: entry.committer_time,
            summary: entry.summary,
            final_line: entry.final_line,
            orig_line: entry.orig_line,
            orig_path: PathBuf::from(entry.filename),
            previous: entry.previous,
            content,
        })
        .collect();

    Ok(Blame {
        path: blob.path,
        at,
        size: blob.size,
        content: BlameContent::Lines(out),
    })
}

/// One line's worth of `--line-porcelain` header, before the file's own
/// text is attached to it.
struct PorcelainEntry {
    commit: Oid,
    author_name: String,
    author_email: String,
    timestamp: i64,
    committer_time: i64,
    summary: String,
    final_line: u32,
    orig_line: u32,
    /// The `filename` header: the path in `commit`.
    filename: String,
    /// The `previous <parent> <path>` header, absent for a line born in a
    /// root commit or in a commit whose parent lacks the file.
    previous: Option<(Oid, PathBuf)>,
}

/// Parses `git blame --line-porcelain`.
///
/// Each record starts with `<sha> <orig-line> <final-line> [<count>]`,
/// carries `author`/`author-mail`/`author-time`/`summary` (plus
/// committer fields and optional `boundary`/`previous`, all ignored), and
/// ends with the file's own line prefixed by a literal TAB. Only that TAB
/// line is a reliable record terminator - header keys are not a fixed
/// set - so it, not a blank line, is what closes an entry here.
fn parse_line_porcelain(raw: &str) -> Vec<PorcelainEntry> {
    let mut out = Vec::new();
    let mut current: Option<PorcelainEntry> = None;
    // `summary` is per-commit in git's own output only for the first
    // occurrence in plain `--porcelain`; `--line-porcelain` repeats it,
    // but a cache keeps this correct even if that ever changes.
    let mut summaries: HashMap<Oid, String> = HashMap::new();

    for line in raw.lines() {
        if line.starts_with('\t') {
            if let Some(mut entry) = current.take() {
                if entry.summary.is_empty() {
                    entry.summary = summaries.get(&entry.commit).cloned().unwrap_or_default();
                } else {
                    summaries.insert(entry.commit, entry.summary.clone());
                }
                out.push(entry);
            }
            continue;
        }

        if let Some(entry) = current.as_mut() {
            if let Some(rest) = line.strip_prefix("author-mail ") {
                // git wraps the address in angle brackets.
                entry.author_email = rest.trim().trim_matches(['<', '>']).to_string();
            } else if let Some(rest) = line.strip_prefix("author-time ") {
                entry.timestamp = rest.trim().parse().unwrap_or_default();
            } else if let Some(rest) = line.strip_prefix("author ") {
                entry.author_name = rest.to_string();
            } else if let Some(rest) = line.strip_prefix("summary ") {
                entry.summary = rest.to_string();
            } else if let Some(rest) = line.strip_prefix("committer-time ") {
                entry.committer_time = rest.trim().parse().unwrap_or_default();
            } else if let Some(rest) = line.strip_prefix("filename ") {
                entry.filename = unquote_c(rest);
            } else if let Some(rest) = line.strip_prefix("previous ") {
                if let Some((sha, path)) = rest.split_once(' ') {
                    entry.previous =
                        Oid::from_hex(sha).map(|oid| (oid, PathBuf::from(unquote_c(path))));
                }
            }
            continue;
        }

        // A header line: `<sha> <orig> <final> [<lines-in-group>]`.
        let mut fields = line.split(' ');
        let Some(commit) = fields.next().and_then(Oid::from_hex) else {
            continue;
        };
        let Some(orig_line) = fields.next().and_then(|f| f.parse().ok()) else {
            continue;
        };
        let Some(final_line) = fields.next().and_then(|f| f.parse().ok()) else {
            continue;
        };
        current = Some(PorcelainEntry {
            commit,
            author_name: String::new(),
            author_email: String::new(),
            timestamp: 0,
            committer_time: 0,
            summary: String::new(),
            final_line,
            orig_line,
            filename: String::new(),
            previous: None,
        });
    }

    out
}

/// Undoes C-style quoting of a path in porcelain headers (a path with a
/// quote, backslash, control or non-ASCII byte is wrapped in `"` with
/// `\\`, `\"`, `\t`, `\n` and octal escapes).
fn unquote_c(raw: &str) -> String {
    let Some(inner) = raw.strip_prefix('"').and_then(|r| r.strip_suffix('"')) else {
        return raw.to_string();
    };
    let bytes = inner.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] != b'\\' || i + 1 >= bytes.len() {
            out.push(bytes[i]);
            i += 1;
            continue;
        }
        i += 1;
        match bytes[i] {
            b'n' => out.push(b'\n'),
            b't' => out.push(b'\t'),
            b'r' => out.push(b'\r'),
            c @ b'0'..=b'7' => {
                let mut value = u32::from(c - b'0');
                let mut taken = 1;
                while taken < 3 && i + 1 < bytes.len() && (b'0'..=b'7').contains(&bytes[i + 1]) {
                    i += 1;
                    value = value * 8 + u32::from(bytes[i] - b'0');
                    taken += 1;
                }
                out.push(value as u8);
            }
            other => out.push(other),
        }
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// Domain paths are repo-relative and git spells them with `/` on every
/// platform; a `PathBuf` join on Windows yields backslashes no `<rev>:<path>`
/// or pathspec matches. Normalizing here is why this crate needs no
/// `cfg(windows)`.
pub(crate) fn repo_relative(path: &Path) -> Result<String> {
    let text = path
        .to_str()
        .ok_or_else(|| Error::Backend(format!("path {} is not valid utf-8", path.display())))?;
    Ok(text.replace('\\', "/").trim_start_matches("./").to_string())
}

/// Every path at `at`, sorted, with `/` separators.
///
/// `-z` because without it git quotes paths containing a space, a quote or
/// a non-ASCII byte. `ls-tree` already sorts and uses `/`; the contract
/// suite pins both.
pub fn list_tree(repo: &RepoRef, at: Oid) -> Result<Vec<PathBuf>> {
    // `<oid>^{tree}`, not the bare oid: `ls-tree` also accepts a tag or a
    // blob, and the explicit form turns "no tree here" into an error.
    let spec = format!("{}^{{tree}}", at.to_hex());
    let fields = run_z_fields(
        &repo.root,
        &["ls-tree", "-r", "-z", "--name-only", "--full-tree", &spec],
    )?;
    Ok(fields.into_iter().map(PathBuf::from).collect())
}
