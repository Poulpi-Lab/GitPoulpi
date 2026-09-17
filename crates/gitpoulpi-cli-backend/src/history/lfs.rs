//! The LFS import / export rules of the rewrite (slice S21): our own
//! engine, not `git lfs migrate`, so the preview, the scope, the undo and the
//! push guard all apply.
//!
//! Import: a matching blob becomes the pointer text of spec v1 and its
//! content is written, once per distinct content, under `<run>/lfs/<oid>` while
//! the preview runs (a preview writes only inside its run directory). Apply
//! moves those files into the LFS store before any ref moves. Export reads
//! the objects back from the local store and refuses, with the list, when
//! some are missing.

use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use gitpoulpi_core::domain::{LfsPointer, Oid};
use gitpoulpi_core::{Error, Result};
use sha2::{Digest, Sha256};

use super::walk::Reader;

/// The directory of a run that holds the objects Apply installs.
pub(super) const DIR: &str = "lfs";

/// Pointers are tiny; a blob this long or longer is not one.
const POINTER_MAX: u64 = 1024;

fn io(what: &str, path: &Path, e: std::io::Error) -> Error {
    Error::Backend(format!("{what} {}: {e}", path.display()))
}

fn hex(digest: impl AsRef<[u8]>) -> String {
    digest.as_ref().iter().map(|b| format!("{b:02x}")).collect()
}

/// Writes a blob's bytes to a file and hashes them on the way; keeps the first
/// bytes, which tell an existing pointer from content.
struct Tee {
    file: File,
    hasher: Sha256,
    head: Vec<u8>,
}

impl Write for Tee {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.file.write_all(buf)?;
        self.hasher.update(buf);
        let room = (POINTER_MAX as usize).saturating_sub(self.head.len());
        self.head.extend_from_slice(&buf[..buf.len().min(room)]);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.file.flush()
    }
}

#[derive(Default)]
pub(super) struct Lfs {
    /// `<run>/lfs`; empty until a rule needs it.
    dir: PathBuf,
    /// The local LFS store (`objects/aa/bb/<oid>` below it), for the export.
    storage: PathBuf,
    /// Import: what each blob already looked at turned out to be, so the same
    /// content at two paths is streamed once. `None`: left as it is.
    done: HashMap<Oid, Option<(String, u64)>>,
    /// Import: the objects waiting in `dir`: oid and size.
    objects: BTreeMap<String, u64>,
    /// Import: distinct blobs that became pointers, and their bytes.
    pub(super) blobs: u64,
    pub(super) bytes: u64,
    /// Export: pointers that became files again.
    pub(super) restored: u64,
    /// Export: `(oid, path)` of every pointer whose object is not local.
    missing: Vec<(String, String)>,
    tmp: u64,
}

impl Lfs {
    pub(super) fn configure(&mut self, dir: PathBuf) {
        self.dir = dir.join(DIR);
    }

    pub(super) fn set_storage(&mut self, storage: PathBuf) {
        self.storage = storage;
    }

    /// Distinct objects Apply will install.
    pub(super) fn objects(&self) -> u64 {
        self.objects.len() as u64
    }

    /// The pointers whose object is not in the local store.
    pub(super) fn missing(&self) -> &[(String, String)] {
        &self.missing
    }

    fn temp(&mut self) -> Result<(PathBuf, File)> {
        std::fs::create_dir_all(&self.dir).map_err(|e| io("cannot create", &self.dir, e))?;
        self.tmp += 1;
        let path = self.dir.join(format!(".part-{}", self.tmp));
        let file = File::create(&path).map_err(|e| io("cannot write", &path, e))?;
        Ok((path, file))
    }

    /// Keeps a finished temporary under its oid (the content is the same
    /// whatever its path, so one file serves every blob with it).
    fn keep(&mut self, part: &Path, sha: &str, size: u64) -> Result<()> {
        let to = self.dir.join(sha);
        if self.objects.contains_key(sha) {
            let _ = std::fs::remove_file(part);
            return Ok(());
        }
        std::fs::rename(part, &to).map_err(|e| io("cannot write", &to, e))?;
        self.objects.insert(sha.to_string(), size);
        Ok(())
    }

    /// The pointer for a blob that stays text in git: `None` when it is
    /// smaller than `min` or already a pointer; else the oid and size, the
    /// content waiting in the run directory.
    pub(super) fn import_blob(
        &mut self,
        reader: &mut Reader,
        blob: Oid,
        min: u64,
    ) -> Result<Option<(String, u64)>> {
        if let Some(known) = self.done.get(&blob) {
            return Ok(known.clone());
        }
        let (part, file) = self.temp()?;
        let tee = Tee {
            file,
            hasher: Sha256::new(),
            head: Vec::new(),
        };
        let (size, tee) = reader.blob_copy(blob, |size| (size >= min).then_some(tee))?;
        let found = match tee {
            Some(tee) if !(size < POINTER_MAX && LfsPointer::parse(&tee.head).is_some()) => {
                let sha = hex(tee.hasher.finalize());
                drop(tee.file);
                self.keep(&part, &sha, size)?;
                self.blobs += 1;
                self.bytes += size;
                Some((sha, size))
            }
            _ => {
                let _ = std::fs::remove_file(&part);
                None
            }
        };
        self.done.insert(blob, found.clone());
        Ok(found)
    }

    /// The same for content already in memory (an earlier rule changed it).
    pub(super) fn import_bytes(&mut self, bytes: &[u8], min: u64) -> Result<Option<(String, u64)>> {
        let size = bytes.len() as u64;
        if size < min || (size < POINTER_MAX && LfsPointer::parse(bytes).is_some()) {
            return Ok(None);
        }
        let sha = hex(Sha256::digest(bytes));
        if !self.objects.contains_key(&sha) {
            let (part, mut file) = self.temp()?;
            file.write_all(bytes)
                .map_err(|e| io("cannot write", &part, e))?;
            drop(file);
            self.keep(&part, &sha, size)?;
        }
        self.blobs += 1;
        self.bytes += size;
        Ok(Some((sha, size)))
    }

    /// The file of a pointer, read from the local store and checked against
    /// the pointer's size and hash; `None` (and noted) when it is not there.
    pub(super) fn export(&mut self, pointer: &LfsPointer, path: &str) -> Option<Vec<u8>> {
        let file = self.storage.join(pointer.storage_rel_path());
        let content = File::open(&file).ok().and_then(|mut f| {
            let mut body = Vec::with_capacity(pointer.size.min(1 << 30) as usize);
            f.read_to_end(&mut body).ok()?;
            Some(body)
        });
        match content {
            Some(body)
                if body.len() as u64 == pointer.size
                    && hex(Sha256::digest(&body)) == pointer.oid =>
            {
                self.restored += 1;
                Some(body)
            }
            _ => {
                let entry = (pointer.oid.clone(), path.to_string());
                if !self.missing.contains(&entry) {
                    self.missing.push(entry);
                }
                None
            }
        }
    }
}

/// The pointer blob a converted file becomes.
pub(super) fn pointer_blob(oid: &str, size: u64) -> Vec<u8> {
    LfsPointer {
        oid: oid.to_string(),
        size,
    }
    .to_text()
    .into_bytes()
}

/// The refusal detail for missing objects: one `<oid> <path>` per line,
/// capped, so the workbench can offer to fetch them.
pub(super) fn missing_detail(missing: &[(String, String)]) -> String {
    const LISTED: usize = 50;
    let mut lines: Vec<String> = missing
        .iter()
        .take(LISTED)
        .map(|(oid, path)| format!("{oid} {path}"))
        .collect();
    if missing.len() > LISTED {
        lines.push("...".into());
    }
    lines.join("\n")
}

/// The local LFS store: `lfs.storage` (relative to the git dir) else
/// `<common dir>/lfs`.
pub(super) fn storage_dir(root: &Path, common_dir: &Path) -> PathBuf {
    let configured = crate::process::git(root, &["config", "--get", "lfs.storage"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .filter(|v| !v.is_empty());
    match configured {
        Some(p) => {
            let p = PathBuf::from(p);
            if p.is_absolute() {
                p
            } else {
                common_dir.join(p)
            }
        }
        None => common_dir.join("lfs"),
    }
}

/// Moves the objects a preview wrote into the store: atomically (a rename,
/// else a copy to a temporary name and a rename), once per oid, never over an
/// object that is there.
pub(super) fn install(run_dir: &Path, storage: &Path) -> Result<u64> {
    let from = run_dir.join(DIR);
    let Ok(entries) = std::fs::read_dir(&from) else {
        return Ok(0);
    };
    let mut written = 0;
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        let pointer = LfsPointer {
            oid: name.clone(),
            size: 0,
        };
        // Only finished objects (a 64-hex name); a `.part-` leftover is not one.
        if name.len() != 64 || !name.bytes().all(|b| b.is_ascii_hexdigit()) {
            continue;
        }
        let to = storage.join(pointer.storage_rel_path());
        if to.is_file() {
            continue;
        }
        if let Some(parent) = to.parent() {
            std::fs::create_dir_all(parent).map_err(|e| io("cannot create", parent, e))?;
        }
        if std::fs::rename(entry.path(), &to).is_err() {
            let part = to.with_extension("part");
            std::fs::copy(entry.path(), &part).map_err(|e| io("cannot write", &part, e))?;
            std::fs::rename(&part, &to).map_err(|e| io("cannot write", &to, e))?;
        }
        written += 1;
    }
    Ok(written)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_pointer_is_spec_v1_text() {
        let oid = "4d7a214614ab2935c943f9e0ff69d22eadbb8f32b1258daaa5e2ca24d17e2393";
        assert_eq!(
            String::from_utf8(pointer_blob(oid, 12345)).unwrap(),
            format!("version https://git-lfs.github.com/spec/v1\noid sha256:{oid}\nsize 12345\n")
        );
    }

    #[test]
    fn the_missing_list_is_capped() {
        let many: Vec<(String, String)> = (0..60)
            .map(|i| (format!("{i:064}"), format!("f{i}")))
            .collect();
        let detail = missing_detail(&many);
        assert_eq!(detail.lines().count(), 51);
        assert!(detail.ends_with("..."));
    }
}
