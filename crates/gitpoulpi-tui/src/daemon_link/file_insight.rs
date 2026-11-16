use super::*;

/// How many untracked files get a synthesized all-added diff at once: a
/// fresh `node_modules` must not turn one diff request into thousands of
/// reads.
pub(super) const MAX_UNTRACKED_DIFFS: usize = 50;

/// Adds an all-added diff for every untracked text file `files` does not
/// already have, so the Unstaged half of the Diff view shows the files
/// the Unstaged list shows. Best effort: a file that cannot be read, or
/// is oversized, or binary and no picture, is left out.
pub(super) async fn append_untracked_diffs(
    conn: &LinkConn,
    files: &mut Vec<FileDiff>,
    only: Option<&Path>,
) {
    let Ok(status) = conn
        .call::<WorkingTree>(OperationKind::Status, serde_json::json!({}))
        .await
    else {
        return;
    };
    let untracked: Vec<_> = status
        .files
        .iter()
        .filter(|e| e.index.is_none() && e.worktree == Some(FileStatus::Untracked))
        .filter(|e| only.is_none_or(|p| e.path == p))
        .filter(|e| !files.iter().any(|f| f.path == e.path))
        .take(MAX_UNTRACKED_DIFFS)
        .collect();
    for entry in untracked {
        let Ok(blob) = conn
            .call::<Blob>(
                OperationKind::ReadBlob,
                serde_json::json!({ "path": entry.path, "source": BlobSource::WorkingTree }),
            )
            .await
        else {
            continue;
        };
        match &blob.content {
            BlobContent::Text { lines, .. } => {
                files.push(crate::wip_view::all_added_diff(entry.path.clone(), lines));
            }
            // A new picture has no text: a hunk-less added file, which the
            // Diff view draws as a "New file" picture (image view S8) rather
            // than "nothing left to show".
            BlobContent::Binary if blob.image.is_some() => files.push(FileDiff {
                renamed_from: None,
                similarity: None,
                path: entry.path.clone(),
                status: FileStatus::Added,
                hunks: Vec::new(),
            }),
            _ => {}
        }
    }
}

/// The blob source a WIP file's content comes from: the index for a file
/// opened from the Staged list, the disk otherwise.
pub(super) fn wip_source(request: &FileInsightRequest) -> BlobSource {
    if request.staged {
        BlobSource::Index
    } else {
        BlobSource::WorkingTree
    }
}

/// Why a WIP file has nothing to show, in the user's words.
pub(super) fn wip_missing_notice(request: &FileInsightRequest) -> String {
    if request.staged {
        gitpoulpi_i18n::msg::link::wip_deleted_in_index(request.path.display())
    } else {
        gitpoulpi_i18n::msg::link::wip_gone(request.path.display())
    }
}

/// The blob source a side of a diff names. `Parent` and `Head` are resolved
/// here, with the existing `Resolve` operation (`<sha>^1`, `HEAD`): a root
/// commit has no parent and an unborn branch no `HEAD`, which comes back as
/// the read's error ("no earlier version").
async fn diff_side_source(
    conn: &LinkConn,
    request: &FileInsightRequest,
    side: DiffContentSide,
) -> GitResult<BlobSource> {
    let rev = match side {
        DiffContentSide::Commit => return Ok(BlobSource::Commit(request.at)),
        DiffContentSide::Index => return Ok(BlobSource::Index),
        DiffContentSide::WorkingTree => return Ok(BlobSource::WorkingTree),
        DiffContentSide::Stage(n) => return Ok(BlobSource::Stage(n)),
        DiffContentSide::Parent => format!("{}^1", request.at.to_hex()),
        DiffContentSide::CommitAt(oid) => return Ok(BlobSource::Commit(oid)),
        DiffContentSide::ParentOf(oid) => format!("{}^1", oid.to_hex()),
        DiffContentSide::Head => "HEAD".to_string(),
    };
    let oid = conn
        .call::<Oid>(OperationKind::Resolve, serde_json::json!({ "rev": rev }))
        .await?;
    Ok(BlobSource::Commit(oid))
}

/// A file the content sniff calls binary but `.gitattributes` says to treat
/// as text (the "Treat as Text" rule) shows as text in the File View, as it
/// does in the diff. `binary_paths` answers `false` for such a file only when
/// the attribute decides - or when the working copy has no NUL, which is why
/// the disk is sniffed first: a binary old version of a file that is text now
/// stays binary.
async fn honour_text_attribute(conn: &LinkConn, source: &BlobSource, blob: Blob) -> Blob {
    if blob.content != BlobContent::Binary || blob.image.is_some() {
        return blob;
    }
    if !crate::binary_hint::sniff(&conn.repo.root, &blob.path) {
        return blob;
    }
    let path = blob.path.to_string_lossy().into_owned();
    let forced_text = conn
        .call::<Vec<bool>>(
            OperationKind::BinaryPaths,
            serde_json::json!({ "paths": [path] }),
        )
        .await
        .ok()
        .and_then(|v| v.first().copied())
        == Some(false);
    if !forced_text {
        return blob;
    }
    let bytes = conn
        .call::<gitpoulpi_core::domain::BlobBytes>(
            OperationKind::ReadBlobBytes,
            serde_json::json!({
                "path": blob.path,
                "source": source,
                "max": gitpoulpi_core::domain::MAX_TEXT_BLOB_BYTES,
            }),
        )
        .await;
    match bytes.map(|b| b.body) {
        Ok(gitpoulpi_core::domain::BlobBody::Bytes(raw)) => Blob {
            content: BlobContent::from_bytes_as_text(&raw),
            ..blob
        },
        _ => blob,
    }
}

/// The exact bytes of one side of a diff (`None`: the side is empty).
async fn read_side_bytes(
    conn: &LinkConn,
    request: &FileInsightRequest,
    side: Option<DiffContentSide>,
) -> GitResult<Option<Vec<u8>>> {
    use gitpoulpi_core::domain::BlobBody;
    let Some(side) = side else {
        return Ok(None);
    };
    let source = diff_side_source(conn, request, side).await?;
    let bytes = conn
        .call::<gitpoulpi_core::domain::BlobBytes>(
            OperationKind::ReadBlobBytes,
            serde_json::json!({
                "path": request.path,
                "source": source,
                "max": gitpoulpi_core::domain::MAX_TEXT_BLOB_BYTES,
            }),
        )
        .await?;
    match bytes.body {
        BlobBody::Bytes(raw) => Ok(Some(raw)),
        BlobBody::Lfs {
            object: Some(raw), ..
        } => Ok(Some(raw)),
        BlobBody::TooLarge => Err(gitpoulpi_core::Error::Backend(
            gitpoulpi_i18n::msg::diff_view::ENC_TOO_LARGE
                .get()
                .to_string(),
        )),
        BlobBody::Lfs { .. } => Err(gitpoulpi_core::Error::Backend(
            gitpoulpi_i18n::msg::diff_view::ENC_LFS_MISSING
                .get()
                .to_string(),
        )),
    }
}

/// One file-centric read, shaped by the request's `kind`.
pub(super) async fn run_file_insight(
    conn: &LinkConn,
    request: &FileInsightRequest,
) -> GitResult<FileInsightPayload> {
    match request.kind {
        FileInsightKind::Content => {
            let source = if request.worktree {
                wip_source(request)
            } else {
                BlobSource::Commit(request.at)
            };
            let blob = conn
                .call::<Blob>(
                    OperationKind::ReadBlob,
                    serde_json::json!({ "path": request.path, "source": source }),
                )
                .await;
            match blob {
                Ok(blob) => Ok(FileInsightPayload::Content(
                    honour_text_attribute(conn, &source, blob).await,
                )),
                // A WIP file the disk or the index no longer has is a
                // state of the WIP, not a failure to report.
                Err(_) if request.worktree => {
                    Ok(FileInsightPayload::Notice(wip_missing_notice(request)))
                }
                Err(e) => Err(e),
            }
        }
        FileInsightKind::Blame if request.worktree => {
            let blob = conn
                .call::<Blob>(
                    OperationKind::ReadBlob,
                    serde_json::json!({ "path": request.path, "source": wip_source(request) }),
                )
                .await;
            let Ok(blob) = blob else {
                return Ok(FileInsightPayload::Notice(wip_missing_notice(request)));
            };
            let lines = match &blob.content {
                BlobContent::Text { lines, .. } => lines.clone(),
                other => {
                    let content = match other {
                        BlobContent::Binary => BlameContent::Binary,
                        BlobContent::LfsPointer(p) => BlameContent::LfsPointer(p.clone()),
                        _ => BlameContent::TooLarge,
                    };
                    return Ok(FileInsightPayload::Blame(Blame {
                        path: blob.path,
                        at: request.at,
                        size: blob.size,
                        content,
                    }));
                }
            };
            // HEAD's attribution of the same path. A failure here means
            // HEAD has no such file (new or untracked) or no commit at
            // all: every line is then uncommitted.
            let head = if request.is_unborn() {
                None
            } else {
                conn.call::<Blame>(
                    OperationKind::Blame,
                    serde_json::json!({ "path": request.path, "at": request.at }),
                )
                .await
                .ok()
            };
            let head_lines = head.as_ref().map(|b| b.content.lines());
            let attributed = crate::wip_view::attribute(head_lines, &lines);
            Ok(FileInsightPayload::Blame(Blame {
                path: blob.path,
                at: request.at,
                size: blob.size,
                content: BlameContent::Lines(attributed),
            }))
        }
        FileInsightKind::History => Ok(FileInsightPayload::Notice(String::new())),
        FileInsightKind::HistoryPage(skip) => conn
            .call::<gitpoulpi_core::domain::FileHistoryPage>(
                OperationKind::FileHistory,
                serde_json::json!({
                    "path": request.path,
                    "start": request.at,
                    "skip": skip,
                    "limit": crate::history_view::PAGE,
                }),
            )
            .await
            .map(FileInsightPayload::HistoryPage),
        FileInsightKind::DiffContent(side) => {
            let source = diff_side_source(conn, request, side).await?;
            conn.call::<Blob>(
                OperationKind::ReadBlob,
                serde_json::json!({ "path": request.path, "source": source }),
            )
            .await
            .map(FileInsightPayload::Content)
        }
        FileInsightKind::DiffRecoded { old, new, charset } => {
            let old = read_side_bytes(conn, request, old).await?;
            let new = read_side_bytes(conn, request, new).await?;
            crate::diff_recode::recode(&request.path, old.as_deref(), new.as_deref(), charset)
                .map(|(diff, new_lines)| FileInsightPayload::Recoded { diff, new_lines })
                .map_err(gitpoulpi_core::Error::Backend)
        }
        FileInsightKind::BinaryState => conn
            .call::<Vec<bool>>(
                OperationKind::BinaryPaths,
                serde_json::json!({ "paths": [request.path] }),
            )
            .await
            .map(|v| FileInsightPayload::BinaryState(v.first().copied().unwrap_or(false))),
        FileInsightKind::ImageBytes(side) => {
            let source = diff_side_source(conn, request, side).await?;
            conn.call::<gitpoulpi_core::domain::BlobBytes>(
                OperationKind::ReadBlobBytes,
                serde_json::json!({
                    "path": request.path,
                    "source": source,
                    "max": crate::live_settings::image_view_max_bytes(),
                }),
            )
            .await
            .map(FileInsightPayload::ImageBytes)
        }
        FileInsightKind::Blame => conn
            .call::<Blame>(
                OperationKind::Blame,
                serde_json::json!({ "path": request.path, "at": request.at }),
            )
            .await
            .map(FileInsightPayload::Blame),
    }
}
