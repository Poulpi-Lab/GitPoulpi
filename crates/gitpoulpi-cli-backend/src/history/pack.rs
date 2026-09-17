//! A temporary pack of the objects a preview created: undeltified objects
//! (type and size header, then a zlib stream) behind a `PACK` header, closed
//! by the SHA-1 of everything before it. `git index-pack` recomputes every
//! id when Apply indexes it, which is a free cross-check of our hashing;
//! `git gc` deltifies later.

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

use flate2::write::ZlibEncoder;
use flate2::Compression;
use gitpoulpi_core::domain::Oid;
use gitpoulpi_core::{Error, Result};
use sha1::{Digest, Sha1};

use super::serialize::object_id;
use super::store::PACK;

/// Pack object type numbers.
pub(crate) const COMMIT: u8 = 1;
pub(crate) const TREE: u8 = 2;
pub(crate) const BLOB: u8 = 3;
pub(crate) const TAG: u8 = 4;

/// The offset index beside the pack: 28-byte records (id, offset), sorted by
/// id, so a new object can be read back without indexing the pack.
pub(crate) const INDEX: &str = "objects.idx";

/// A pack with no object: 12 bytes of header and the 20 of trailer.
pub(crate) const EMPTY_PACK_LEN: u64 = 32;

fn kind_name(kind: u8) -> &'static str {
    match kind {
        COMMIT => "commit",
        TREE => "tree",
        BLOB => "blob",
        _ => "tag",
    }
}

fn io(e: std::io::Error) -> Error {
    Error::Backend(format!("writing the rewrite pack failed: {e}"))
}

/// Writes a pack as objects arrive (a content run creates trees and blobs
/// while it walks, far too many to hold). The object count is patched into
/// the header and the SHA-1 trailer computed when the run ends. An object
/// added twice is written once.
pub(crate) struct PackWriter {
    file: BufWriter<File>,
    pos: u64,
    offsets: HashMap<Oid, u64>,
    compressed: Vec<u8>,
}

impl PackWriter {
    pub(crate) fn create(path: &Path) -> Result<Self> {
        // Read as well: the trailer hashes the finished file.
        let raw = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)
            .map_err(io)?;
        let mut file = BufWriter::with_capacity(1 << 20, raw);
        // The header, with a count of zero until the end.
        file.write_all(b"PACK").map_err(io)?;
        file.write_all(&2u32.to_be_bytes()).map_err(io)?;
        file.write_all(&0u32.to_be_bytes()).map_err(io)?;
        Ok(Self {
            file,
            pos: 12,
            offsets: HashMap::new(),
            compressed: Vec::new(),
        })
    }

    /// Adds an object and returns its id.
    pub(crate) fn add(&mut self, kind: u8, body: &[u8]) -> Result<Oid> {
        let id = object_id(kind_name(kind), body);
        if self.offsets.contains_key(&id) {
            return Ok(id);
        }
        self.offsets.insert(id, self.pos);
        let header = object_header(kind, body.len() as u64);
        self.compressed.clear();
        let mut z = ZlibEncoder::new(&mut self.compressed, Compression::fast());
        z.write_all(body).map_err(io)?;
        z.finish().map_err(io)?;
        self.file.write_all(&header).map_err(io)?;
        self.file.write_all(&self.compressed).map_err(io)?;
        self.pos += (header.len() + self.compressed.len()) as u64;
        Ok(id)
    }

    /// Patches the count, appends the trailer and writes the offset index
    /// to `index`.
    pub(crate) fn finish(self, index: &Path) -> Result<()> {
        let mut file = self.file.into_inner().map_err(|e| io(e.into_error()))?;
        file.seek(SeekFrom::Start(8)).map_err(io)?;
        file.write_all(&(self.offsets.len() as u32).to_be_bytes())
            .map_err(io)?;
        file.seek(SeekFrom::Start(0)).map_err(io)?;
        let mut hasher = Sha1::new();
        let mut buf = vec![0u8; 1 << 20];
        loop {
            let n = file.read(&mut buf).map_err(io)?;
            if n == 0 {
                break;
            }
            hasher.update(&buf[..n]);
        }
        let trailer: [u8; 20] = hasher.finalize().into();
        file.seek(SeekFrom::End(0)).map_err(io)?;
        file.write_all(&trailer).map_err(io)?;
        file.flush().map_err(io)?;

        let mut records: Vec<(&Oid, &u64)> = self.offsets.iter().collect();
        records.sort_by_key(|(id, _)| id.0);
        let mut out = Vec::with_capacity(records.len() * 28);
        for (id, offset) in records {
            out.extend_from_slice(&id.0);
            out.extend_from_slice(&offset.to_be_bytes());
        }
        std::fs::write(index, out).map_err(io)
    }
}

/// The run's pack, created with the first object (a run that creates none
/// still ends with an empty pack: Apply expects the file).
pub(crate) struct Sink {
    dir: PathBuf,
    writer: Option<PackWriter>,
}

impl Sink {
    pub(crate) fn new(dir: PathBuf) -> Self {
        Self { dir, writer: None }
    }

    fn writer(&mut self) -> Result<&mut PackWriter> {
        if self.writer.is_none() {
            std::fs::create_dir_all(&self.dir).map_err(|e| {
                Error::Backend(format!("cannot create {}: {e}", self.dir.display()))
            })?;
            self.writer = Some(PackWriter::create(&self.dir.join(PACK))?);
        }
        Ok(self.writer.as_mut().expect("just created"))
    }

    pub(crate) fn add(&mut self, kind: u8, body: &[u8]) -> Result<Oid> {
        self.writer()?.add(kind, body)
    }

    /// The run's directory.
    pub(crate) fn dir(&self) -> &Path {
        &self.dir
    }

    pub(crate) fn finish(mut self) -> Result<()> {
        self.writer()?;
        let writer = self.writer.take().expect("created above");
        writer.finish(&self.dir.join(INDEX))
    }
}

/// Reads objects back from a finished pack through its offset index.
pub(crate) struct PackReader {
    pack: PathBuf,
    index: Vec<u8>,
}

impl PackReader {
    /// `None` when the pack or its index is gone.
    pub(crate) fn open(pack: &Path, index: &Path) -> Option<Self> {
        pack.is_file().then_some(())?;
        Some(Self {
            pack: pack.to_path_buf(),
            index: std::fs::read(index).ok()?,
        })
    }

    /// The body of object `id`; `None` when the pack does not have it.
    pub(crate) fn get(&self, id: Oid) -> Option<Vec<u8>> {
        let (mut lo, mut hi) = (0usize, self.index.len() / 28);
        let offset = loop {
            if lo >= hi {
                return None;
            }
            let mid = (lo + hi) / 2;
            let rec = &self.index[mid * 28..mid * 28 + 28];
            match rec[..20].cmp(&id.0[..]) {
                std::cmp::Ordering::Equal => {
                    break u64::from_be_bytes(rec[20..28].try_into().ok()?)
                }
                std::cmp::Ordering::Less => lo = mid + 1,
                std::cmp::Ordering::Greater => hi = mid,
            }
        };
        let mut file = File::open(&self.pack).ok()?;
        file.seek(SeekFrom::Start(offset)).ok()?;
        let mut reader = BufReader::new(file);
        // The type and size header: the zlib stream knows where it ends.
        let mut byte = [0u8; 1];
        reader.read_exact(&mut byte).ok()?;
        while byte[0] & 0x80 != 0 {
            reader.read_exact(&mut byte).ok()?;
        }
        let mut body = Vec::new();
        flate2::read::ZlibDecoder::new(reader)
            .read_to_end(&mut body)
            .ok()?;
        Some(body)
    }
}

/// The type in bits 4-6 and the size in 4 + 7n bits, low bits first.
fn object_header(kind: u8, size: u64) -> Vec<u8> {
    let mut out = Vec::with_capacity(4);
    let mut byte = (kind << 4) | (size & 0x0f) as u8;
    let mut rest = size >> 4;
    while rest > 0 {
        out.push(byte | 0x80);
        byte = (rest & 0x7f) as u8;
        rest >>= 7;
    }
    out.push(byte);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn objects_read_back_from_a_finished_pack() {
        let dir = tempfile::tempdir().unwrap();
        let (pack, idx) = (dir.path().join("p.pack"), dir.path().join("p.idx"));
        let mut w = PackWriter::create(&pack).unwrap();
        let body = vec![b'q'; 5000];
        let a = w.add(BLOB, &body).unwrap();
        let b = w.add(BLOB, b"small").unwrap();
        assert_eq!(w.add(BLOB, b"small").unwrap(), b, "written once");
        w.finish(&idx).unwrap();
        let r = PackReader::open(&pack, &idx).unwrap();
        assert_eq!(r.get(a).unwrap(), body);
        assert_eq!(r.get(b).unwrap(), b"small");
        assert!(r.get(Oid([9; 20])).is_none());
    }

    #[test]
    fn sizes_are_varints_after_the_type() {
        assert_eq!(object_header(COMMIT, 5), vec![0x15]);
        // 300 = 0b1_0010_1100: low nibble 0xc, then 300 >> 4 = 18.
        assert_eq!(object_header(COMMIT, 300), vec![0x9c, 18]);
    }
}
