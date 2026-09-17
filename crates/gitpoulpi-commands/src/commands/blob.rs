//! `gitpoulpi show --raw [--output FILE]`: a file's exact bytes.

use std::io::Write;
use std::path::{Path, PathBuf};

use gitpoulpi_core::domain::{BlobBody, BlobBytes, BlobSource, OperationKind};
use serde_json::{json, Value};

use super::emitted;
use crate::client::Session;
use crate::repo_cmd::Emitted;

pub(super) async fn show_raw(
    session: &mut Session,
    path: &str,
    source: BlobSource,
    output: Option<PathBuf>,
) -> Result<Emitted, String> {
    let blob: BlobBytes = session
        .call(
            OperationKind::ReadBlobBytes,
            // No cap: the caller asked for the file, whatever its size.
            json!({ "path": path, "source": source, "max": u64::MAX }),
        )
        .await?;
    let bytes = match blob.body {
        BlobBody::Bytes(bytes) => bytes,
        BlobBody::Lfs {
            object: Some(bytes),
            ..
        } => bytes,
        BlobBody::Lfs { pointer, .. } => {
            return Err(format!(
                "{path} is an LFS pointer and its object ({}) is not in the local store; \
                 run `gitpoulpi lfs fetch` first",
                pointer.oid
            ))
        }
        BlobBody::TooLarge => return Err(format!("{path} is too large to read")),
    };
    match output {
        Some(file) => {
            write_file(&file, &bytes)?;
            Ok(emitted(
                json!({ "raw": { "path": path, "size": bytes.len(), "output": file } }),
                format!("wrote {} bytes to {}", bytes.len(), file.display()),
            ))
        }
        None => {
            let mut out = std::io::stdout().lock();
            out.write_all(&bytes)
                .and_then(|()| out.flush())
                .map_err(|e| format!("cannot write to stdout: {e}"))?;
            Ok(emitted(Value::Null, ""))
        }
    }
}

fn write_file(file: &Path, bytes: &[u8]) -> Result<(), String> {
    std::fs::write(file, bytes).map_err(|e| format!("cannot write {}: {e}", file.display()))
}
