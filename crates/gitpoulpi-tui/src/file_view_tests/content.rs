use super::*;

#[test]
fn a_blame_row_shows_sha_author_date_and_the_line() {
    let insight = blame_insight(3);
    let rows = insight_rows(&insight, 80, 10);
    assert_eq!(rows.len(), 3);

    let first: String = rows[0].spans.iter().map(|s| s.content.as_ref()).collect();
    assert!(
        first.starts_with("01010101 "),
        "sha gutter first: {first:?}"
    );
    assert!(first.contains("Ada Lovelace"));
    assert!(first.contains("2023-11-14"), "date column: {first:?}");
    assert!(first.ends_with("line 0"));
}

/// A WIP file is titled WIP (and says staged for the index), never
/// with HEAD's sha; its "Not committed yet" lines point at no commit.
#[test]
fn a_wip_view_is_titled_wip_and_uncommitted_lines_point_nowhere() {
    let mut wip = request(FileInsightKind::Content);
    wip.worktree = true;
    let title = FileInsight::loading(wip.clone()).title();
    assert!(title.ends_with("@ WIP"), "{title}");
    assert!(!title.contains("abababab"), "{title}");
    wip.staged = true;
    assert!(FileInsight::loading(wip)
        .title()
        .ends_with("@ WIP (staged)"));

    let mut insight = blame_insight(2);
    if let FileInsightData::Blame(blame) = &mut insight.data {
        if let BlameContent::Lines(lines) = &mut blame.content {
            lines[0].commit = Oid([0; 20]);
            lines[0].author_name = "Not committed yet".into();
        }
    }
    let rows = insight_rows(&insight, 80, 10);
    let first: String = rows[0].spans.iter().map(|s| s.content.as_ref()).collect();
    assert!(first.starts_with("Not committed yet"), "{first:?}");
    assert!(first.ends_with("line 0"), "{first:?}");
    assert_eq!(insight.commit_at(0), None);
    assert_eq!(insight.commit_at(1), Some(oid(2)));
}

/// A notice (a deleted WIP file) is shown as the plain sentence.
#[test]
fn a_notice_payload_is_shown_as_its_sentence() {
    let req = request(FileInsightKind::Content);
    let mut insight = FileInsight::loading(req.clone());
    insight.accept(&req, FileInsightPayload::Notice("gone".into()));
    let rows = insight_rows(&insight, 80, 10);
    let text: String = rows[0].spans.iter().map(|s| s.content.as_ref()).collect();
    assert_eq!(text, "gone");
}

#[test]
fn enter_on_a_blame_row_points_at_that_line_s_commit() {
    let insight = blame_insight(3);
    assert_eq!(insight.commit_at(2), Some(oid(3)));
    assert_eq!(insight.commit_at(9), None);
}

/// A stale answer must not replace what is on screen now.
#[test]
fn an_answer_to_an_abandoned_request_is_dropped() {
    let mut insight = FileInsight::loading(request(FileInsightKind::Blame));
    let stale = FileInsightRequest {
        path: PathBuf::from("other.rs"),
        at: oid(0xab),
        kind: FileInsightKind::Blame,
        worktree: false,
        staged: false,
    };
    insight.accept(
        &stale,
        FileInsightPayload::Content(Blob {
            path: PathBuf::from("other.rs"),
            source: BlobSource::Commit(oid(0xab)),
            size: 0,
            image: None,
            content: BlobContent::Binary,
        }),
    );
    assert!(matches!(insight.data, FileInsightData::Loading));
}

#[test]
fn a_binary_file_says_so_instead_of_rendering_nothing() {
    let mut insight = FileInsight::loading(request(FileInsightKind::Content));
    insight.data = FileInsightData::Content(Blob {
        path: PathBuf::from("src/lib.rs"),
        source: BlobSource::Commit(oid(0xab)),
        size: 4096,
        image: None,
        content: BlobContent::Binary,
    });
    let rows = insight_rows(&insight, 80, 10);
    let text: String = rows[0].spans.iter().map(|s| s.content.as_ref()).collect();
    assert!(text.contains("binary file"), "{text:?}");
    assert!(text.contains("4096"));
}

/// An LFS blob shows what it stands for (size, short oid), not the
/// three lines of pointer text git stores.
#[test]
fn an_lfs_blob_renders_its_size_and_not_the_pointer_text() {
    let pointer = gitpoulpi_core::domain::LfsPointer {
        oid: "ab12cd34".repeat(8),
        size: 12_345_678,
    };
    let mut insight = FileInsight::loading(request(FileInsightKind::Content));
    insight.data = FileInsightData::Content(Blob {
        path: PathBuf::from("big.bin"),
        source: BlobSource::Commit(oid(0xab)),
        size: 130,
        image: None,
        content: BlobContent::LfsPointer(pointer.clone()),
    });
    let rows = insight_rows(&insight, 80, 10);
    let text: String = rows
        .iter()
        .flat_map(|r| r.spans.iter().map(|s| s.content.as_ref()))
        .collect();
    assert!(text.contains("Git LFS object"), "{text:?}");
    assert!(text.contains("12.3 MB"), "{text:?}");
    assert!(text.contains("ab12cd34"), "{text:?}");
    assert!(!text.contains("version"), "{text:?}");

    insight.data = FileInsightData::Blame(Blame {
        path: PathBuf::from("big.bin"),
        at: oid(0xab),
        size: 130,
        content: BlameContent::LfsPointer(pointer),
    });
    let rows = insight_rows(&insight, 80, 10);
    let text: String = rows[0].spans.iter().map(|s| s.content.as_ref()).collect();
    assert!(text.contains("Git LFS object"), "{text:?}");
}
