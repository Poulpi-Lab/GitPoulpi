//! The undo journal's in-memory copy and its write-behind.
//!
//! The file stays the truth: a copy is trusted only while the file's
//! stamp (modification time, length and, on Unix, inode) is the one seen
//! when the copy was made, so another process editing the journal is
//! noticed on the next read. Parsing and re-serialising 50 entries costs
//! more than a staging operation, so:
//!
//! * `cached` answers from memory while the stamp holds;
//! * `put_clean` records what was just read or synchronously written;
//! * `put_later` is for staging only: the entry is recorded after the
//!   operation and its trees are already pinned, so the bytes can follow
//!   on a background thread. A crash before that write loses the newest
//!   staging entries and leaves their pins, which the next entry with
//!   that id replaces (see `journal::pin_after_crash`); it never loses a
//!   snapshot or tears the file (temporary file, then rename).
//! * `flush` writes whatever is pending, for shutdown and for tests.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Sender};
use std::sync::{Mutex, OnceLock};
use std::time::SystemTime;

use gitpoulpi_core::journal::{Journal, JournalEntry};

type Stamp = (SystemTime, u64, u64);

struct Slot {
    journal: Journal,
    /// The file's stamp when `journal` was read or written; `None` for a
    /// journal that does not exist on disk (yet).
    stamp: Option<Stamp>,
    dirty: bool,
    generation: u64,
}

static STORE: Mutex<Option<HashMap<PathBuf, Slot>>> = Mutex::new(None);

fn with<R>(f: impl FnOnce(&mut HashMap<PathBuf, Slot>) -> R) -> R {
    let mut guard = STORE.lock().unwrap_or_else(|e| e.into_inner());
    f(guard.get_or_insert_with(HashMap::new))
}

fn stamp_of(meta: &std::fs::Metadata) -> Option<Stamp> {
    #[cfg(unix)]
    let ino = {
        use std::os::unix::fs::MetadataExt;
        meta.ino()
    };
    #[cfg(not(unix))]
    let ino = 0;
    Some((meta.modified().ok()?, meta.len(), ino))
}

/// The file's stamp now; `None` when there is no file.
pub fn stamp(path: &Path) -> Option<Stamp> {
    std::fs::metadata(path).ok().and_then(|m| stamp_of(&m))
}

/// The in-memory journal when it is still the file's (or newer, and not
/// yet written).
pub fn cached(path: &Path) -> Option<Journal> {
    let now = stamp(path);
    with(|store| {
        let slot = store.get(path)?;
        (slot.dirty || slot.stamp == now).then(|| slot.journal.clone())
    })
}

/// Remembers `journal` as what the file holds, `stamp` being the file's
/// stamp taken *before* it was read (or from the temporary file before
/// the rename that wrote it).
pub fn put_clean(path: &Path, journal: &Journal, stamp: Option<Stamp>) {
    with(|store| {
        let generation = store.get(path).map_or(0, |s| s.generation + 1);
        store.insert(
            path.to_path_buf(),
            Slot {
                journal: journal.clone(),
                stamp,
                dirty: false,
                generation,
            },
        );
    });
}

/// Forgets the copy of `path` (and any write still pending for it).
pub fn forget(path: &Path) {
    with(|store| {
        store.remove(path);
    });
}

/// Every entry's JSON as last written, per journal file, with the entry it
/// was made from. Serialising a journal costs time proportional to its
/// bytes (a discard entry lists every file it saved: 100 entries can be
/// a megabyte, 7 ms), yet an operation changes one or two entries, so an
/// unchanged entry reuses its bytes (comparing is far cheaper than
/// serialising). Only touched under `WRITE_LOCK`.
type Rendered = HashMap<PathBuf, HashMap<u64, (JournalEntry, Vec<u8>)>>;
static RENDERED: Mutex<Option<Rendered>> = Mutex::new(None);

/// The journal as JSON, byte for byte what `serde_json::to_vec` gives
/// (the field order of `Journal`), entry bytes reused while unchanged.
fn render(path: &Path, journal: &Journal) -> std::io::Result<Vec<u8>> {
    let mut guard = RENDERED.lock().unwrap_or_else(|e| e.into_inner());
    let known = guard
        .get_or_insert_with(HashMap::new)
        .entry(path.to_path_buf())
        .or_default();
    let mut next = HashMap::new();
    let mut out = Vec::with_capacity(4096);
    out.extend_from_slice(
        format!(
            "{{\"version\":{},\"next_id\":{},",
            journal.version, journal.next_id
        )
        .as_bytes(),
    );
    for (key, list) in [
        ("undo", &journal.undo),
        ("redo", &journal.redo),
        ("retired", &journal.retired),
    ] {
        out.extend_from_slice(format!("\"{key}\":[").as_bytes());
        for (n, entry) in list.iter().enumerate() {
            if n > 0 {
                out.push(b',');
            }
            let kept = match known.remove(&entry.id) {
                Some((old, bytes)) if old == *entry => (old, bytes),
                _ => (
                    {
                        #[cfg(test)]
                        SERIALISED.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        entry.clone()
                    },
                    serde_json::to_vec(entry).map_err(std::io::Error::other)?,
                ),
            };
            out.extend_from_slice(&kept.1);
            next.insert(entry.id, kept);
        }
        out.extend_from_slice(b"],");
    }
    out.extend_from_slice(b"\"pin_ns\":");
    out.extend_from_slice(&serde_json::to_vec(&journal.pin_ns).map_err(std::io::Error::other)?);
    out.push(b'}');
    *known = next;
    Ok(out)
}

/// Writes `journal` to `path`: temporary file, then rename. Returns the
/// stamp of the file written.
fn write_file(path: &Path, journal: &Journal) -> std::io::Result<Option<Stamp>> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let tmp = path.with_extension(format!("json.tmp{}", std::process::id()));
    std::fs::write(&tmp, render(path, journal)?)?;
    let stamp = std::fs::metadata(&tmp).ok().and_then(|m| stamp_of(&m));
    std::fs::rename(&tmp, path)?;
    Ok(stamp)
}

/// Serialises every journal file write. Without it the write-behind thread
/// could rename an older snapshot over a newer synchronous save (losing or
/// resurrecting a `Pending` entry), and two writers shared one temporary file.
static WRITE_LOCK: Mutex<()> = Mutex::new(());

/// Writes `journal` synchronously and records it as the file's content,
/// superseding any write-behind still pending for `path`.
pub fn write_through(path: &Path, journal: &Journal) -> std::io::Result<()> {
    let _write = WRITE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let stamp = write_file(path, journal)?;
    put_clean(path, journal, stamp);
    Ok(())
}

fn writer() -> &'static Mutex<Sender<()>> {
    static WRITER: OnceLock<Mutex<Sender<()>>> = OnceLock::new();
    WRITER.get_or_init(|| {
        let (tx, rx) = channel::<()>();
        std::thread::Builder::new()
            .name("gitpoulpi-journal-writer".into())
            .spawn(move || {
                while rx.recv().is_ok() {
                    // One flush serves every message queued meanwhile.
                    while rx.try_recv().is_ok() {}
                    flush();
                }
            })
            .expect("the journal writer thread starts");
        Mutex::new(tx)
    })
}

/// `journal` is the file's next content; the bytes are written behind.
pub fn put_later(path: &Path, journal: &Journal) {
    with(|store| {
        let old = store.remove(path);
        store.insert(
            path.to_path_buf(),
            Slot {
                journal: journal.clone(),
                stamp: old.as_ref().and_then(|s| s.stamp),
                dirty: true,
                generation: old.map_or(0, |s| s.generation + 1),
            },
        );
    });
    let _ = writer().lock().unwrap_or_else(|e| e.into_inner()).send(());
}

/// Writes every pending journal. A failed write stays pending.
pub fn flush() {
    let _write = WRITE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let pending: Vec<(PathBuf, Journal, u64)> = with(|store| {
        store
            .iter()
            .filter(|(_, s)| s.dirty)
            .map(|(p, s)| (p.clone(), s.journal.clone(), s.generation))
            .collect()
    });
    for (path, journal, generation) in pending {
        match write_file(&path, &journal) {
            Ok(stamp) => with(|store| {
                if let Some(slot) = store.get_mut(&path) {
                    if slot.generation == generation {
                        slot.dirty = false;
                        slot.stamp = stamp;
                    }
                }
            }),
            Err(e) => tracing::warn!("could not write the undo journal {}: {e}", path.display()),
        }
    }
}

/// Entries serialised by `render` (tests only).
#[cfg(test)]
static SERIALISED: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

#[cfg(test)]
mod tests {
    use super::*;
    use gitpoulpi_core::domain::OperationKind;
    use std::sync::atomic::Ordering;

    fn journal(count: usize) -> Journal {
        let mut j = Journal::default();
        for n in 0..count {
            j.begin(
                OperationKind::BranchCreate,
                format!("entry {n}"),
                n as i64,
                None,
                false,
                vec![],
            );
        }
        j.pin_ns = "ns".into();
        j
    }

    /// Recording one more operation at depth 100 serialises that entry
    /// only, and the bytes are exactly `serde_json`'s: the cost of an
    /// operation does not grow with the journal.
    #[test]
    fn a_save_serialises_only_the_entries_that_changed() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("journal.json");
        let mut j = journal(100);
        write_through(&path, &j).unwrap();
        assert_eq!(
            std::fs::read(&path).unwrap(),
            serde_json::to_vec(&j).unwrap()
        );

        let before = SERIALISED.load(Ordering::Relaxed);
        j.begin(
            OperationKind::Commit,
            "one more".into(),
            1,
            None,
            false,
            vec![],
        );
        write_through(&path, &j).unwrap();
        j.undo[40].label = "edited".into();
        write_through(&path, &j).unwrap();
        // Dropped from the history: its bytes leave with it.
        j.undo.remove(0);
        write_through(&path, &j).unwrap();
        let after = SERIALISED.load(Ordering::Relaxed);
        // Other tests share the counter, so allow a margin, never 100+.
        assert!(
            after - before < 50,
            "re-serialised {} entries",
            after - before
        );
        assert_eq!(
            std::fs::read(&path).unwrap(),
            serde_json::to_vec(&j).unwrap()
        );
        let guard = RENDERED.lock().unwrap();
        assert_eq!(guard.as_ref().unwrap()[&path].len(), 100);
    }
}
