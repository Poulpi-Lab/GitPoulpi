//! `GitBackend::convert_files`: the working-tree half of the right pane's
//! "Convert" menu. Plain file I/O over `gitpoulpi_core::textconv`, shared by
//! both backends so they write the same bytes.

use gitpoulpi_core::domain::RepoRef;
use gitpoulpi_core::textconv::{
    convert_bytes, ConvertOp, ConvertOutcome, ConvertReport, ConvertSkip, Eol,
};
use gitpoulpi_core::{Error, Result};

use crate::snapshot_ops::normalize_all;

pub fn convert_files(repo: &RepoRef, paths: &[String], op: ConvertOp) -> Result<ConvertReport> {
    if op == ConvertOp::Eol(Eol::Mixed) {
        return Err(Error::Backend("cannot convert line breaks to Mixed".into()));
    }
    let mut report = ConvertReport::default();
    for path in normalize_all(paths)? {
        let full = repo.root.join(&path);
        // A symlink is not followed: converting its target would write
        // outside what the path names.
        let is_file = std::fs::symlink_metadata(&full)
            .map(|m| m.is_file())
            .unwrap_or(false);
        if !is_file {
            report.skipped.push((path, ConvertSkip::Missing));
            continue;
        }
        let bytes = match std::fs::read(&full) {
            Ok(bytes) => bytes,
            Err(e) => {
                report.failed.push((path, e.to_string()));
                continue;
            }
        };
        match convert_bytes(&bytes, op) {
            ConvertOutcome::Unchanged => report.unchanged.push(path),
            ConvertOutcome::Skipped(why) => report.skipped.push((path, why)),
            ConvertOutcome::Changed(out) => match std::fs::write(&full, out) {
                Ok(()) => report.converted.push(path),
                Err(e) => report.failed.push((path, e.to_string())),
            },
        }
    }
    Ok(report)
}
