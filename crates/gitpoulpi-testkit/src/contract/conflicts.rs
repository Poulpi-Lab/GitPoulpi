//! Contract checks for the merge conflict editor's read side:
//! `conflict_entries` (the cheap listing) and `merge_document` (the three
//! stages, git's own merge of them, the labels and the state of the file
//! on disk). see the merge-conflict-editor design, 4.

use std::path::Path;

use gitpoulpi_core::domain::{
    aligned_rows, compose_resolution, with_fallback_encoding, BlobContent, CommitOptions,
    ConflictEntry, ConflictKind, ConflictOperation, ConflictPicks, ConflictRefusal,
    ConflictResolution, ConflictSide, Eol, MarkerStyle, MergeDocument, MergeRegion, OperationState,
    ResolveOptions, RowMark, WorktreeFileState, MAX_TEXT_BLOB_BYTES,
};
use gitpoulpi_core::{Error, GitBackend};

use crate::{ConflictFixture, FixtureOperation};

fn document(backend: &dyn GitBackend, fx: &ConflictFixture, path: &str) -> MergeDocument {
    backend
        .merge_document(&fx.repo_ref(), Path::new(path))
        .unwrap_or_else(|e| panic!("merge_document({path}) failed: {e}"))
}

/// The file git wrote for `doc`'s regions, as git's plain `merge` style
/// prints it.
fn rendered_like_git(doc: &MergeDocument) -> String {
    doc.render_markers(&MarkerStyle {
        marker_size: 7,
        diff3: false,
        ours_label: &doc.labels.git_ours_label,
        base_label: "",
        theirs_label: &doc.labels.git_theirs_label,
        eol: if doc.eol == Eol::Crlf { "\r\n" } else { "\n" },
        final_newline: doc.final_newline,
    })
}

fn text_lines(blob: &Option<gitpoulpi_core::domain::SideBlob>) -> Vec<String> {
    blob.as_ref().expect("the side exists").lines().to_vec()
}

// -- the listing --------------------------------------------------------------

pub fn check_conflict_entries_report_kind_and_stage_oids(backend: &dyn GitBackend) {
    // (fixture, path, kind, which of stages 1/2/3 exist)
    let cases: Vec<(ConflictFixture, &str, ConflictKind)> = vec![
        (
            ConflictFixture::text(),
            "notes.txt",
            ConflictKind::BothModified,
        ),
        (
            ConflictFixture::add_add(),
            "new.txt",
            ConflictKind::BothAdded,
        ),
        (
            ConflictFixture::delete_modify(true),
            "gone.txt",
            ConflictKind::DeletedByUs,
        ),
        (
            ConflictFixture::delete_modify(false),
            "gone.txt",
            ConflictKind::DeletedByThem,
        ),
        (
            ConflictFixture::binary(),
            "image.bin",
            ConflictKind::BothModified,
        ),
        (
            ConflictFixture::symlink(),
            "link",
            ConflictKind::BothModified,
        ),
        (
            ConflictFixture::submodule(),
            "sub",
            ConflictKind::BothModified,
        ),
        (
            ConflictFixture::mode_only(),
            "run.sh",
            ConflictKind::BothAdded,
        ),
        (
            ConflictFixture::rename_delete(),
            "b.txt",
            ConflictKind::DeletedByThem,
        ),
    ];
    for (fx, path, kind) in cases {
        let entries = backend.conflict_entries(&fx.repo_ref()).unwrap();
        assert_eq!(entries.len(), 1, "{path}: {entries:?}");
        let e = &entries[0];
        assert_eq!(e.path.to_str(), Some(path));
        assert_eq!(e.kind, kind, "{path}");
        for (stage, got) in [(1, e.base), (2, e.ours), (3, e.theirs)] {
            let want = fx.stage_oid(stage, path);
            assert_eq!(
                got.map(|s| s.oid.to_hex()),
                want,
                "{path}: stage {stage} oid"
            );
        }
    }

    // Modes come from the index, not the files.
    let fx = ConflictFixture::mode_only();
    let e = &backend.conflict_entries(&fx.repo_ref()).unwrap()[0];
    assert_eq!(e.ours.unwrap().mode, 0o100644);
    assert_eq!(e.theirs.unwrap().mode, 0o100755);
    assert!(e.mode_changed());
    assert_eq!(e.ours.unwrap().oid, e.theirs.unwrap().oid, "same blob");

    let fx = ConflictFixture::symlink();
    let e = &backend.conflict_entries(&fx.repo_ref()).unwrap()[0];
    assert!(e.is_symlink() && !e.is_submodule());
    assert_eq!(e.ours.unwrap().mode, 0o120000);

    let fx = ConflictFixture::submodule();
    let e = &backend.conflict_entries(&fx.repo_ref()).unwrap()[0];
    assert!(e.is_submodule());
    assert_eq!(e.theirs.unwrap().oid.to_hex(), "2".repeat(40));

    // rename/rename leaves one path per stage.
    let fx = ConflictFixture::rename_rename();
    let entries = backend.conflict_entries(&fx.repo_ref()).unwrap();
    let kinds: Vec<(&str, ConflictKind)> = entries
        .iter()
        .map(|e| (e.path.to_str().unwrap(), e.kind))
        .collect();
    assert_eq!(
        kinds,
        [
            ("a.txt", ConflictKind::BothDeleted),
            ("b.txt", ConflictKind::AddedByUs),
            ("c.txt", ConflictKind::AddedByThem),
        ]
    );
}

pub fn check_conflict_entries_are_sorted_and_empty_without_conflicts(backend: &dyn GitBackend) {
    let fx = ConflictFixture::multi_conflict();
    let paths: Vec<String> = backend
        .conflict_entries(&fx.repo_ref())
        .unwrap()
        .iter()
        .map(|e| e.path.to_string_lossy().replace('\\', "/"))
        .collect();
    assert_eq!(paths, ["a.txt", "b.txt", "docs/c.txt"]);

    let clean = ConflictFixture::none_unmerged();
    assert!(backend
        .conflict_entries(&clean.repo_ref())
        .unwrap()
        .is_empty());
}

// -- the document: regions -----------------------------------------------------

pub fn check_merge_document_regions_match_the_worktree_git_wrote(backend: &dyn GitBackend) {
    // Single-point conflicts, where git's plain merge style and the
    // zealous diff3 the document is built with place the markers alike.
    let mut cases: Vec<ConflictFixture> = vec![
        ConflictFixture::text(),
        ConflictFixture::multi_conflict(),
        ConflictFixture::crlf(),
        ConflictFixture::text_with(FixtureOperation::CherryPick),
        ConflictFixture::text_with(FixtureOperation::Revert),
        ConflictFixture::text_with(FixtureOperation::Rebase),
    ];
    for fx in cases.drain(..) {
        for path in fx.paths().to_vec() {
            let doc = document(backend, &fx, &path);
            assert!(doc.conflict_count() >= 1, "{path}");
            assert_eq!(doc.marker_size, 7);
            assert_eq!(doc.worktree, WorktreeFileState::Markers, "{path}");
            let want = String::from_utf8(fx.worktree_bytes(&path)).unwrap();
            assert_eq!(
                rendered_like_git(&doc),
                want,
                "{path} ({:?}): regions rendered with git's labels must be the file git wrote",
                fx.operation()
            );
        }
    }
}

pub fn check_merge_document_counts_every_conflict_of_a_file(backend: &dyn GitBackend) {
    let fx = ConflictFixture::multi_conflict();
    let counts: Vec<usize> = fx
        .paths()
        .iter()
        .map(|p| document(backend, &fx, p).conflict_count())
        .collect();
    assert_eq!(counts, [3, 1, 1]);

    let doc = document(backend, &fx, "a.txt");
    let first = doc.conflicts().next().unwrap();
    // The first conflict spans lines 3 and 4.
    assert_eq!(
        first.lines_of(ConflictSide::Ours),
        ["A3 ours".to_string(), "A4 ours".to_string()]
    );
    assert_eq!(
        first.lines_of(ConflictSide::Base),
        ["l3".to_string(), "l4".to_string()]
    );
    assert_eq!(
        first.lines_of(ConflictSide::Theirs),
        ["A3 theirs".to_string(), "A4 theirs".to_string()]
    );
}

pub fn check_merge_document_survives_marker_lookalikes(backend: &dyn GitBackend) {
    let fx = ConflictFixture::marker_lookalike();
    let doc = document(backend, &fx, "README.md");
    assert!(
        doc.marker_size >= 8,
        "the marker size must clear the `=======` lines (got {})",
        doc.marker_size
    );
    // The document keeps the `Title` underline and the lookalike lines as
    // content, and finds exactly one conflict.
    assert_eq!(doc.conflict_count(), 1, "{:#?}", doc.regions);
    let ours = text_lines(&doc.ours);
    let theirs = text_lines(&doc.theirs);
    assert!(ours.contains(&"<<<<<<< not a marker".to_string()));
    assert!(theirs.contains(&">>>>>>> not a marker".to_string()));
    let all_ours: Vec<ConflictPicks> = doc
        .conflicts()
        .map(|c| ConflictPicks::take(c, &[ConflictSide::Ours]))
        .collect();
    assert_eq!(
        compose_resolution(&doc, &all_ours)
            .lines()
            .collect::<Vec<_>>(),
        ours,
        "taking every conflict from ours gives ours"
    );
    let all_theirs: Vec<ConflictPicks> = doc
        .conflicts()
        .map(|c| ConflictPicks::take(c, &[ConflictSide::Theirs]))
        .collect();
    assert_eq!(
        compose_resolution(&doc, &all_theirs)
            .lines()
            .collect::<Vec<_>>(),
        theirs
    );
}

pub fn check_merge_document_composes_back_to_the_side_files(backend: &dyn GitBackend) {
    // `crlf` and `bom`: nothing but the conflict differs between the
    // sides, so a whole side composes to exactly that side's blob once the
    // BOM and the line endings are put back.
    for fx in [ConflictFixture::crlf(), ConflictFixture::crlf_bom()] {
        let path = fx.path().to_string();
        let doc = document(backend, &fx, &path);
        assert_eq!(doc.eol, Eol::Crlf, "{path}");
        assert_eq!(doc.bom, path == "bom.txt", "{path}");
        for (side, stage) in [(ConflictSide::Ours, 2), (ConflictSide::Theirs, 3)] {
            let picks: Vec<ConflictPicks> = doc
                .conflicts()
                .map(|c| ConflictPicks::take(c, &[side]))
                .collect();
            let mut bytes = Vec::new();
            if doc.bom {
                bytes.extend_from_slice(b"\xEF\xBB\xBF");
            }
            bytes.extend_from_slice(
                compose_resolution(&doc, &picks)
                    .replace('\n', "\r\n")
                    .as_bytes(),
            );
            assert_eq!(
                Some(bytes),
                fx.stage_bytes(stage, &path),
                "{path}: all {side:?} must be the stage-{stage} file"
            );
        }
        // No `\r` and no BOM survive inside the lines.
        for region in &doc.regions {
            for side in [ConflictSide::Ours, ConflictSide::Theirs, ConflictSide::Base] {
                assert!(region
                    .lines_of(side)
                    .iter()
                    .all(|l| !l.ends_with('\r') && !l.starts_with('\u{feff}')));
            }
        }
    }
}

pub fn check_merge_document_add_add_has_no_base(backend: &dyn GitBackend) {
    let fx = ConflictFixture::add_add();
    let doc = document(backend, &fx, "new.txt");
    assert_eq!(doc.entry.kind, ConflictKind::BothAdded);
    assert!(doc.base.is_none());
    assert!(doc.conflict_count() >= 1, "{:#?}", doc.regions);
    for c in doc.conflicts() {
        assert!(c.lines_of(ConflictSide::Base).is_empty());
    }
    assert_eq!(
        text_lines(&doc.ours),
        ["ours first".to_string(), "ours second".to_string()]
    );
}

pub fn check_merge_document_delete_modify_keeps_only_the_surviving_side(backend: &dyn GitBackend) {
    let fx = ConflictFixture::delete_modify(true);
    let doc = document(backend, &fx, "gone.txt");
    assert_eq!(doc.entry.kind, ConflictKind::DeletedByUs);
    assert!(doc.ours.is_none() && doc.base.is_some());
    assert_eq!(text_lines(&doc.theirs)[1], "changed");
    assert!(doc.regions.is_empty(), "nothing to merge");

    let fx = ConflictFixture::delete_modify(false);
    let doc = document(backend, &fx, "gone.txt");
    assert_eq!(doc.entry.kind, ConflictKind::DeletedByThem);
    assert!(doc.theirs.is_none());
    assert_eq!(text_lines(&doc.ours)[1], "changed");
    assert!(doc.regions.is_empty());

    // A rename leftover: only one side's stage at the path.
    let fx = ConflictFixture::rename_rename();
    let doc = document(backend, &fx, "b.txt");
    assert_eq!(doc.entry.kind, ConflictKind::AddedByUs);
    assert!(doc.ours.is_some() && doc.base.is_none() && doc.theirs.is_none());
    assert!(doc.regions.is_empty());
}

// -- the document: things that are not text --------------------------------------

/// The bytes of each conflict stage (1 base, 2 ours, 3 theirs) are readable
/// with `BlobSource::Stage`, exactly as the index holds them; a stage the
/// path has not got is an error, and so is a path that is not conflicted.
pub fn check_read_blob_bytes_of_a_conflict_stage(backend: &dyn GitBackend) {
    use gitpoulpi_core::domain::{BlobBody, BlobSource};
    let fx = ConflictFixture::binary();
    let repo = fx.repo_ref();
    for stage in 1..=3u8 {
        let got = backend
            .read_blob_bytes(
                &repo,
                Path::new("image.bin"),
                &BlobSource::Stage(stage),
                1 << 20,
            )
            .unwrap_or_else(|e| panic!("stage {stage}: {e}"));
        let want = fx.stage_bytes(stage, "image.bin").unwrap();
        assert_eq!(got.size, want.len() as u64, "stage {stage}");
        assert_eq!(got.body, BlobBody::Bytes(want), "stage {stage}");
    }
    let over = backend
        .read_blob_bytes(&repo, Path::new("image.bin"), &BlobSource::Stage(2), 3)
        .unwrap();
    assert_eq!(over.body, BlobBody::TooLarge);
    assert!(backend
        .read_blob_bytes(&repo, Path::new("image.bin"), &BlobSource::Stage(9), 100)
        .is_err());
    assert!(backend
        .read_blob_bytes(&repo, Path::new("nope.bin"), &BlobSource::Stage(2), 100)
        .is_err());
}

pub fn check_merge_document_binary_and_lfs_have_no_regions(backend: &dyn GitBackend) {
    let fx = ConflictFixture::binary();
    let doc = document(backend, &fx, "image.bin");
    for (side, stage) in [(&doc.base, 1), (&doc.ours, 2), (&doc.theirs, 3)] {
        let side = side.as_ref().unwrap();
        assert_eq!(side.content, BlobContent::Binary);
        assert_eq!(
            side.size,
            fx.stage_bytes(stage, "image.bin").unwrap().len() as u64
        );
    }
    assert!(doc.regions.is_empty());

    let fx = ConflictFixture::lfs_pointer();
    let doc = document(backend, &fx, "big.dat");
    match &doc.ours.as_ref().unwrap().content {
        BlobContent::LfsPointer(p) => {
            assert_eq!((p.oid.as_str(), p.size), ("b".repeat(64).as_str(), 200))
        }
        other => panic!("expected a pointer, got {other:?}"),
    }
    match &doc.theirs.as_ref().unwrap().content {
        BlobContent::LfsPointer(p) => assert_eq!(p.size, 300),
        other => panic!("expected a pointer, got {other:?}"),
    }
    assert!(doc.regions.is_empty());
}

pub fn check_merge_document_too_large(backend: &dyn GitBackend) {
    let fx = ConflictFixture::huge();
    let doc = document(backend, &fx, "huge.txt");
    for side in [&doc.base, &doc.ours, &doc.theirs] {
        let side = side.as_ref().unwrap();
        assert_eq!(side.content, BlobContent::TooLarge);
        assert!(side.size > MAX_TEXT_BLOB_BYTES);
    }
    assert!(doc.regions.is_empty());
}

pub fn check_merge_document_symlink_mode_only_and_submodule_have_no_text_merge(
    backend: &dyn GitBackend,
) {
    let fx = ConflictFixture::symlink();
    let doc = document(backend, &fx, "link");
    assert!(doc.entry.is_symlink());
    assert_eq!(text_lines(&doc.ours), ["target-ours".to_string()]);
    assert_eq!(text_lines(&doc.theirs), ["target-theirs".to_string()]);
    assert!(
        doc.regions.is_empty(),
        "a link target is not merged line by line"
    );

    // Same blob, different mode: nothing conflicts in the text.
    let fx = ConflictFixture::mode_only();
    let doc = document(backend, &fx, "run.sh");
    assert!(doc.entry.mode_changed());
    assert_eq!(doc.conflict_count(), 0);
    assert_eq!(
        doc.ours.as_ref().unwrap().oid,
        doc.theirs.as_ref().unwrap().oid
    );

    // A gitlink's oid is a commit elsewhere: it is reported, never read.
    let fx = ConflictFixture::submodule();
    let doc = document(backend, &fx, "sub");
    assert!(doc.entry.is_submodule());
    assert_eq!(doc.ours.as_ref().unwrap().oid.to_hex(), "3".repeat(40));
    assert_eq!(doc.theirs.as_ref().unwrap().oid.to_hex(), "2".repeat(40));
    assert!(doc.regions.is_empty());
}

pub fn check_merge_document_decodes_with_fallback_encoding(backend: &dyn GitBackend) {
    let fx = ConflictFixture::latin1();
    // Not UTF-8, no fallback: not text.
    let doc = document(backend, &fx, "latin1.txt");
    assert_eq!(doc.ours.as_ref().unwrap().content, BlobContent::Binary);
    assert!(doc.regions.is_empty());
    assert_eq!(doc.encoding, None);

    let doc = with_fallback_encoding(Some("latin1"), || document(backend, &fx, "latin1.txt"));
    assert_eq!(doc.encoding.as_deref(), Some("windows-1252"));
    assert_eq!(text_lines(&doc.base)[0], "café");
    assert_eq!(doc.conflict_count(), 1);
    let c = doc.conflicts().next().unwrap();
    assert_eq!(c.lines_of(ConflictSide::Ours), ["ours é".to_string()]);
    assert_eq!(c.lines_of(ConflictSide::Theirs), ["theirs è".to_string()]);
    assert_eq!(
        doc.regions[0],
        MergeRegion::Merged {
            lines: vec!["café".into(), "l2".into(), "l3".into()]
        }
    );
}

// -- the document: labels and the file on disk --------------------------------------

pub fn check_merge_document_labels_for_merge_cherry_pick_rebase(backend: &dyn GitBackend) {
    let short =
        |fx: &ConflictFixture, rev: &str| -> String { fx.git(&["rev-parse", "--short=7", rev]) };

    let fx = ConflictFixture::text();
    let l = document(backend, &fx, "notes.txt").labels;
    assert_eq!(l.op, ConflictOperation::Merge);
    assert_eq!(l.current, "Current · main");
    assert_eq!(l.incoming, "Incoming · feature");
    assert_eq!(l.base, "Base");
    assert_eq!(l.description, "merging feature into main");
    assert_eq!(
        (l.git_ours_label.as_str(), l.git_theirs_label.as_str()),
        ("HEAD", "feature")
    );

    let fx = ConflictFixture::text_with(FixtureOperation::CherryPick);
    let picked = short(&fx, "CHERRY_PICK_HEAD");
    let l = document(backend, &fx, "notes.txt").labels;
    assert_eq!(l.op, ConflictOperation::CherryPick);
    assert_eq!(l.current, "Current · main");
    assert_eq!(l.incoming, format!("Incoming · {picked} feature change"));
    assert_eq!(l.description, format!("cherry-picking {picked}"));

    let fx = ConflictFixture::text_with(FixtureOperation::Revert);
    let reverted = short(&fx, "REVERT_HEAD");
    let l = document(backend, &fx, "notes.txt").labels;
    assert_eq!(l.op, ConflictOperation::Revert);
    assert_eq!(l.current, "Current · main");
    assert_eq!(
        l.incoming,
        format!("Incoming · revert of {reverted} feature change")
    );
    assert_eq!(l.description, format!("reverting {reverted}"));

    // During a rebase git swaps the roles: ours is what we rebase onto.
    let fx = ConflictFixture::text_with(FixtureOperation::Rebase);
    let onto = short(&fx, "main");
    let stopped = short(&fx, "REBASE_HEAD");
    let l = document(backend, &fx, "notes.txt").labels;
    assert_eq!(l.op, ConflictOperation::Rebase);
    assert_eq!(l.current, format!("Current · onto {onto}"));
    assert_eq!(
        l.incoming,
        format!("Incoming · your commit {stopped} feature change")
    );
    assert_eq!(
        l.description,
        format!("rebasing feature onto {onto} · step 1/1")
    );
}

pub fn check_merge_document_reports_the_worktree_state(backend: &dyn GitBackend) {
    let fx = ConflictFixture::text();
    let path = "notes.txt";
    assert_eq!(
        document(backend, &fx, path).worktree,
        WorktreeFileState::Markers
    );

    // Resolved by hand in another editor, markers gone.
    std::fs::write(fx.root().join(path), "resolved by hand\n").unwrap();
    let doc = document(backend, &fx, path);
    assert_eq!(
        doc.worktree,
        WorktreeFileState::EditedNoMarkers { crlf: false }
    );
    // The regions are still git's merge of the stages, not the disk file.
    assert!(doc.conflict_count() >= 1);

    std::fs::write(fx.root().join(path), "resolved\r\nby hand\r\n").unwrap();
    assert_eq!(
        document(backend, &fx, path).worktree,
        WorktreeFileState::EditedNoMarkers { crlf: true }
    );

    std::fs::remove_file(fx.root().join(path)).unwrap();
    assert_eq!(
        document(backend, &fx, path).worktree,
        WorktreeFileState::Missing
    );
}

pub fn check_merge_document_of_a_path_that_is_not_conflicted_is_an_error(backend: &dyn GitBackend) {
    let fx = ConflictFixture::add_add();
    // Tracked and clean, but not unmerged.
    assert!(backend
        .merge_document(&fx.repo_ref(), Path::new("keep.txt"))
        .is_err());
    assert!(backend
        .merge_document(&fx.repo_ref(), Path::new("no/such/file"))
        .is_err());
    let clean = ConflictFixture::none_unmerged();
    assert!(backend
        .merge_document(&clean.repo_ref(), Path::new("f.txt"))
        .is_err());
}

// -- the document: aligning the panes --------------------------------------------------

pub fn check_merge_document_aligns_the_panes_with_the_real_files(backend: &dyn GitBackend) {
    let fx = ConflictFixture::text();
    let doc = document(backend, &fx, "notes.txt");
    let rows = aligned_rows(&doc.regions, doc.side_lines());
    assert_eq!(rows.degraded, [false; 3]);

    let ours = text_lines(&doc.ours);
    let theirs = text_lines(&doc.theirs);
    let base = text_lines(&doc.base);
    for (side, file) in [
        (ConflictSide::Base, &base),
        (ConflictSide::Ours, &ours),
        (ConflictSide::Theirs, &theirs),
    ] {
        let shown: Vec<usize> = rows.side(side).iter().filter_map(|r| r.line).collect();
        assert_eq!(
            shown,
            (0..file.len()).collect::<Vec<_>>(),
            "{side:?}: each line once"
        );
    }

    // Line 3 is the conflict, on the same row in both panes.
    let conflict_row = rows
        .rows
        .iter()
        .position(|r| r.conflict == Some(0))
        .unwrap();
    assert_eq!(rows.rows[conflict_row].lines[1], Some(2));
    assert_eq!(rows.rows[conflict_row].lines[2], Some(2));
    // Line 8 only theirs changed: git merged it, and our pane shows our
    // old line there as a tinted row.
    let row = rows
        .rows
        .iter()
        .find(|r| r.lines[1] == Some(7))
        .expect("our line 8 is on a row");
    assert_eq!(row.marks[1], RowMark::Changed);
    assert_eq!(row.marks[2], RowMark::Same);
    assert_eq!(
        rows.conflict_range(0, ConflictSide::Ours, ours.len()),
        Some(2..3)
    );
}

// -- resolving ------------------------------------------------------------------------

fn entries(backend: &dyn GitBackend, fx: &ConflictFixture) -> Vec<ConflictEntry> {
    backend.conflict_entries(&fx.repo_ref()).unwrap()
}

fn resolve(
    backend: &dyn GitBackend,
    fx: &ConflictFixture,
    path: &str,
    resolution: ConflictResolution,
    opts: &ResolveOptions,
) -> gitpoulpi_core::Result<()> {
    backend.resolve_conflict(&fx.repo_ref(), Path::new(path), resolution, opts)
}

fn resolve_ok(
    backend: &dyn GitBackend,
    fx: &ConflictFixture,
    path: &str,
    resolution: ConflictResolution,
) {
    resolve(backend, fx, path, resolution, &ResolveOptions::default())
        .unwrap_or_else(|e| panic!("resolve {path} failed: {e}"));
}

/// `git ls-files -s -- path`: one `<mode> <oid> <stage>\t<path>` line per
/// index entry.
fn index_lines(fx: &ConflictFixture, path: &str) -> Vec<String> {
    fx.git(&["ls-files", "-s", "--", path])
        .lines()
        .map(str::to_string)
        .collect()
}

fn refusal(result: gitpoulpi_core::Result<()>) -> ConflictRefusal {
    match result {
        Err(Error::ConflictRefused(r)) => r,
        other => panic!("expected a typed conflict refusal, got {other:?}"),
    }
}

pub fn check_resolve_with_content_preserves_crlf_and_bom(backend: &dyn GitBackend) {
    let fx = ConflictFixture::crlf_bom();
    let text = "\u{FEFF}l1\r\nl2\r\nresolved\r\n";
    resolve_ok(
        backend,
        &fx,
        "bom.txt",
        ConflictResolution::UseContent(text.into()),
    );
    // Byte for byte: the BOM stays three bytes, no line ending is touched.
    assert_eq!(
        fx.worktree_bytes("bom.txt"),
        text.as_bytes(),
        "bytes on disk"
    );
    assert_eq!(
        fx.stage_bytes(0, "bom.txt").unwrap(),
        text.as_bytes(),
        "the staged blob"
    );
    assert!(entries(backend, &fx).is_empty());
}

pub fn check_resolve_without_stage_keeps_the_path_conflicted(backend: &dyn GitBackend) {
    let fx = ConflictFixture::text();
    let before = entries(backend, &fx);
    let save_only = ResolveOptions {
        stage: false,
        expected: None,
    };
    resolve(
        backend,
        &fx,
        "notes.txt",
        ConflictResolution::UseContent("saved, not staged\n".into()),
        &save_only,
    )
    .unwrap();
    assert_eq!(fx.worktree_bytes("notes.txt"), b"saved, not staged\n");
    assert_eq!(entries(backend, &fx), before, "the stages are untouched");
    assert_eq!(
        index_lines(&fx, "notes.txt").len(),
        3,
        "still stages 1, 2 and 3"
    );
    // The editor reads the text back as "edited outside" while the stages
    // still say conflicted.
    assert_eq!(
        document(backend, &fx, "notes.txt").worktree,
        WorktreeFileState::EditedNoMarkers { crlf: false }
    );

    // A later Save & Mark Resolved against the same stages goes through.
    let expected = ResolveOptions {
        stage: true,
        expected: Some(ResolveOptions::oids_of(&before[0])),
    };
    resolve(
        backend,
        &fx,
        "notes.txt",
        ConflictResolution::UseContent("final\n".into()),
        &expected,
    )
    .unwrap();
    assert!(entries(backend, &fx).is_empty());
    assert_eq!(fx.stage_bytes(0, "notes.txt").unwrap(), b"final\n");
}

pub fn check_resolve_refuses_stale_expected_oids_and_changes_nothing(backend: &dyn GitBackend) {
    let fx = ConflictFixture::text();
    let before = entries(backend, &fx);
    let disk_before = fx.worktree_bytes("notes.txt");
    let [base, ours, theirs] = ResolveOptions::oids_of(&before[0]);

    for wrong in [
        [base, theirs, ours],
        [None, ours, theirs],
        [base, ours, None],
        [None, None, None],
    ] {
        let opts = ResolveOptions {
            stage: true,
            expected: Some(wrong),
        };
        let result = resolve(
            backend,
            &fx,
            "notes.txt",
            ConflictResolution::UseContent("must not be written\n".into()),
            &opts,
        );
        assert_eq!(refusal(result), ConflictRefusal::Stale, "{wrong:?}");
        assert_eq!(fx.worktree_bytes("notes.txt"), disk_before);
        assert_eq!(entries(backend, &fx), before);
    }

    // The same guard refuses a file somebody else already resolved.
    let good = ResolveOptions {
        stage: true,
        expected: Some([base, ours, theirs]),
    };
    resolve(
        backend,
        &fx,
        "notes.txt",
        ConflictResolution::TakeSide(ConflictSide::Ours),
        &good,
    )
    .unwrap();
    let again = resolve(
        backend,
        &fx,
        "notes.txt",
        ConflictResolution::UseContent("too late\n".into()),
        &good,
    );
    assert_eq!(refusal(again), ConflictRefusal::Stale);
    assert_ne!(fx.worktree_bytes("notes.txt"), b"too late\n");
}

pub fn check_resolve_with_encoded_content_writes_that_encodings_bytes(backend: &dyn GitBackend) {
    let fx = ConflictFixture::latin1();
    let text = "caf\u{e9}\nl2\nl3\nmerged \u{e9}\nl5\nl6\nl7\n";
    resolve_ok(
        backend,
        &fx,
        "latin1.txt",
        ConflictResolution::UseEncoded {
            text: text.into(),
            encoding: "windows-1252".into(),
        },
    );
    let want = b"caf\xE9\nl2\nl3\nmerged \xE9\nl5\nl6\nl7\n";
    assert_eq!(fx.worktree_bytes("latin1.txt"), want);
    assert_eq!(fx.stage_bytes(0, "latin1.txt").unwrap(), want);

    // A character the encoding cannot hold refuses, naming the line.
    let fx = ConflictFixture::latin1();
    let before = fx.worktree_bytes("latin1.txt");
    let result = resolve(
        backend,
        &fx,
        "latin1.txt",
        ConflictResolution::UseEncoded {
            text: "caf\u{e9}\n\u{65e5}\u{672c}\n".into(),
            encoding: "iso-8859-1".into(),
        },
        &ResolveOptions::default(),
    );
    assert_eq!(refusal(result), ConflictRefusal::Unencodable { line: 2 });
    assert_eq!(fx.worktree_bytes("latin1.txt"), before);
    assert_eq!(entries(backend, &fx).len(), 1, "still conflicted");

    // An unknown label is an error and changes nothing either.
    assert!(resolve(
        backend,
        &fx,
        "latin1.txt",
        ConflictResolution::UseEncoded {
            text: "x\n".into(),
            encoding: "no-such-encoding".into(),
        },
        &ResolveOptions::default(),
    )
    .is_err());
    assert_eq!(fx.worktree_bytes("latin1.txt"), before);
}

pub fn check_take_side_on_delete_modify(backend: &dyn GitBackend) {
    // Ours deleted the file, theirs modified it.
    let fx = ConflictFixture::delete_modify(true);
    resolve_ok(
        backend,
        &fx,
        "gone.txt",
        ConflictResolution::TakeSide(ConflictSide::Ours),
    );
    assert!(!fx.root().join("gone.txt").exists(), "ours deleted it");
    assert!(index_lines(&fx, "gone.txt").is_empty());
    assert!(entries(backend, &fx).is_empty());

    let fx = ConflictFixture::delete_modify(true);
    let theirs = fx.stage_bytes(3, "gone.txt").unwrap();
    resolve_ok(
        backend,
        &fx,
        "gone.txt",
        ConflictResolution::TakeSide(ConflictSide::Theirs),
    );
    assert_eq!(fx.worktree_bytes("gone.txt"), theirs, "the modified file");
    assert_eq!(fx.stage_bytes(0, "gone.txt").unwrap(), theirs);
    assert!(entries(backend, &fx).is_empty());

    // And the mirror: theirs deleted it, ours modified it.
    let fx = ConflictFixture::delete_modify(false);
    resolve_ok(
        backend,
        &fx,
        "gone.txt",
        ConflictResolution::TakeSide(ConflictSide::Theirs),
    );
    assert!(!fx.root().join("gone.txt").exists(), "theirs deleted it");
    assert!(index_lines(&fx, "gone.txt").is_empty());

    let fx = ConflictFixture::delete_modify(false);
    let ours = fx.stage_bytes(2, "gone.txt").unwrap();
    resolve_ok(
        backend,
        &fx,
        "gone.txt",
        ConflictResolution::TakeSide(ConflictSide::Ours),
    );
    assert_eq!(fx.stage_bytes(0, "gone.txt").unwrap(), ours);
    assert!(entries(backend, &fx).is_empty());
}

pub fn check_resolve_delete(backend: &dyn GitBackend) {
    for deleted_by_us in [true, false] {
        let fx = ConflictFixture::delete_modify(deleted_by_us);
        resolve_ok(backend, &fx, "gone.txt", ConflictResolution::Delete);
        assert!(!fx.root().join("gone.txt").exists());
        assert!(index_lines(&fx, "gone.txt").is_empty(), "no stage left");
        assert!(entries(backend, &fx).is_empty());
    }

    // Without staging only the file goes; the path stays conflicted.
    let fx = ConflictFixture::delete_modify(false);
    resolve(
        backend,
        &fx,
        "gone.txt",
        ConflictResolution::Delete,
        &ResolveOptions {
            stage: false,
            expected: None,
        },
    )
    .unwrap();
    assert!(!fx.root().join("gone.txt").exists());
    assert_eq!(entries(backend, &fx).len(), 1);
}

pub fn check_take_side_on_submodule(backend: &dyn GitBackend) {
    for (side, commit) in [
        (ConflictSide::Theirs, "2".repeat(40)),
        (ConflictSide::Ours, "3".repeat(40)),
        (ConflictSide::Base, "1".repeat(40)),
    ] {
        let fx = ConflictFixture::submodule();
        resolve_ok(backend, &fx, "sub", ConflictResolution::TakeSide(side));
        // The gitlink holds that side's commit; no checkout was needed.
        assert_eq!(
            index_lines(&fx, "sub"),
            [format!("160000 {commit} 0\tsub")],
            "{side:?}"
        );
        assert!(entries(backend, &fx).is_empty());
    }
}

pub fn check_take_side_keeps_symlink_and_exec_bit(backend: &dyn GitBackend) {
    let fx = ConflictFixture::symlink();
    let theirs_oid = fx.stage_oid(3, "link").unwrap();
    resolve_ok(
        backend,
        &fx,
        "link",
        ConflictResolution::TakeSide(ConflictSide::Theirs),
    );
    assert_eq!(
        index_lines(&fx, "link"),
        [format!("120000 {theirs_oid} 0\tlink")]
    );
    #[cfg(unix)]
    assert_eq!(
        std::fs::read_link(fx.root().join("link")).unwrap(),
        Path::new("target-theirs")
    );

    // Same blob on both sides, only the exec bit differs.
    let fx = ConflictFixture::mode_only();
    let blob = fx.stage_oid(2, "run.sh").unwrap();
    resolve_ok(
        backend,
        &fx,
        "run.sh",
        ConflictResolution::TakeSide(ConflictSide::Theirs),
    );
    assert_eq!(
        index_lines(&fx, "run.sh"),
        [format!("100755 {blob} 0\trun.sh")]
    );
    let fx = ConflictFixture::mode_only();
    resolve_ok(
        backend,
        &fx,
        "run.sh",
        ConflictResolution::TakeSide(ConflictSide::Ours),
    );
    assert_eq!(
        index_lines(&fx, "run.sh"),
        [format!("100644 {blob} 0\trun.sh")]
    );
}

pub fn check_unresolve_restores_stages_and_markers(backend: &dyn GitBackend) {
    // Text conflicts: the file comes back byte for byte as git wrote it.
    let cases: Vec<(ConflictFixture, &str)> = vec![
        (ConflictFixture::text(), "notes.txt"),
        (ConflictFixture::multi_conflict(), "a.txt"),
        (ConflictFixture::crlf(), "crlf.txt"),
        (ConflictFixture::add_add(), "new.txt"),
        (
            ConflictFixture::text_with(FixtureOperation::CherryPick),
            "notes.txt",
        ),
        (
            ConflictFixture::text_with(FixtureOperation::Rebase),
            "notes.txt",
        ),
    ];
    for (fx, path) in cases {
        let before = entries(backend, &fx);
        let entry = before
            .iter()
            .find(|e| e.path.to_str() == Some(path))
            .unwrap();
        let disk_before = fx.worktree_bytes(path);
        resolve_ok(
            backend,
            &fx,
            path,
            ConflictResolution::UseContent("resolved by hand\n".into()),
        );
        assert!(entries(backend, &fx)
            .iter()
            .all(|e| e.path.to_str() != Some(path)));

        backend
            .unresolve_conflict(&fx.repo_ref(), Path::new(path), entry)
            .unwrap_or_else(|e| panic!("unresolve {path} ({:?}): {e}", fx.operation()));
        assert_eq!(entries(backend, &fx), before, "{path}: the stages are back");
        assert_eq!(
            fx.worktree_bytes(path),
            disk_before,
            "{path} ({:?}): the file git wrote is back",
            fx.operation()
        );
        // The editor can work on it again, and resolving again works.
        assert_eq!(
            document(backend, &fx, path).worktree,
            WorktreeFileState::Markers
        );
        resolve_ok(
            backend,
            &fx,
            path,
            ConflictResolution::TakeSide(ConflictSide::Theirs),
        );
        assert!(entries(backend, &fx)
            .iter()
            .all(|e| e.path.to_str() != Some(path)));
    }

    // Non-text: git leaves the surviving side in the working tree.
    let fx = ConflictFixture::binary();
    let before = entries(backend, &fx);
    resolve_ok(
        backend,
        &fx,
        "image.bin",
        ConflictResolution::TakeSide(ConflictSide::Theirs),
    );
    backend
        .unresolve_conflict(&fx.repo_ref(), Path::new("image.bin"), &before[0])
        .unwrap();
    assert_eq!(entries(backend, &fx), before);
    assert_eq!(
        fx.worktree_bytes("image.bin"),
        fx.stage_bytes(2, "image.bin").unwrap()
    );

    let fx = ConflictFixture::delete_modify(true);
    let before = entries(backend, &fx);
    resolve_ok(backend, &fx, "gone.txt", ConflictResolution::Delete);
    backend
        .unresolve_conflict(&fx.repo_ref(), Path::new("gone.txt"), &before[0])
        .unwrap();
    assert_eq!(entries(backend, &fx), before);
    assert_eq!(
        fx.worktree_bytes("gone.txt"),
        fx.stage_bytes(3, "gone.txt").unwrap()
    );

    // A path that is still conflicted is refused, and left alone.
    let fx = ConflictFixture::text();
    let before = entries(backend, &fx);
    let disk = fx.worktree_bytes("notes.txt");
    let result = backend.unresolve_conflict(&fx.repo_ref(), Path::new("notes.txt"), &before[0]);
    assert_eq!(refusal(result), ConflictRefusal::AlreadyConflicted);
    assert_eq!(fx.worktree_bytes("notes.txt"), disk);
}

// -- the operation in progress -------------------------------------------------------

pub fn check_operation_state_and_abort(backend: &dyn GitBackend) {
    let rev = |fx: &ConflictFixture, spec: &str| fx.git(&["rev-parse", spec]);

    // (fixture, what operation_state must say)
    let merge = ConflictFixture::text();
    match backend.operation_state(&merge.repo_ref()).unwrap() {
        Some(OperationState::Merge {
            head_name,
            their_label,
        }) => {
            assert_eq!(head_name.as_deref(), Some("main"));
            assert_eq!(their_label, "feature");
        }
        other => panic!("merge: {other:?}"),
    }
    let head = rev(&merge, "HEAD");
    backend.operation_abort(&merge.repo_ref()).unwrap();
    assert_eq!(backend.operation_state(&merge.repo_ref()).unwrap(), None);
    assert_eq!(rev(&merge, "HEAD"), head);
    assert!(!merge.root().join(".git/MERGE_HEAD").exists());
    assert!(entries(backend, &merge).is_empty());
    assert_eq!(merge.git(&["status", "--porcelain"]), "", "worktree clean");

    let pick = ConflictFixture::text_with(FixtureOperation::CherryPick);
    let picked = rev(&pick, "feature");
    let head = rev(&pick, "HEAD");
    match backend.operation_state(&pick.repo_ref()).unwrap() {
        Some(OperationState::CherryPick { commit }) => assert_eq!(commit.to_hex(), picked),
        other => panic!("cherry-pick: {other:?}"),
    }
    backend.operation_abort(&pick.repo_ref()).unwrap();
    assert_eq!(backend.operation_state(&pick.repo_ref()).unwrap(), None);
    assert_eq!(rev(&pick, "HEAD"), head);
    assert_eq!(pick.git(&["status", "--porcelain"]), "");

    let revert = ConflictFixture::text_with(FixtureOperation::Revert);
    let reverted = rev(&revert, "HEAD~1");
    let head = rev(&revert, "HEAD");
    match backend.operation_state(&revert.repo_ref()).unwrap() {
        Some(OperationState::Revert { commit }) => assert_eq!(commit.to_hex(), reverted),
        other => panic!("revert: {other:?}"),
    }
    backend.operation_abort(&revert.repo_ref()).unwrap();
    assert_eq!(backend.operation_state(&revert.repo_ref()).unwrap(), None);
    assert_eq!(rev(&revert, "HEAD"), head);
    assert_eq!(revert.git(&["status", "--porcelain"]), "");

    let rebase = ConflictFixture::text_with(FixtureOperation::Rebase);
    let onto = rev(&rebase, "main");
    let tip = rev(&rebase, "feature");
    match backend.operation_state(&rebase.repo_ref()).unwrap() {
        Some(OperationState::Rebase(state)) => {
            assert_eq!(state.head_name.as_deref(), Some("feature"));
            assert_eq!(state.onto.to_hex(), onto);
            assert!(state.conflicted);
        }
        other => panic!("rebase: {other:?}"),
    }
    backend.operation_abort(&rebase.repo_ref()).unwrap();
    assert_eq!(backend.operation_state(&rebase.repo_ref()).unwrap(), None);
    assert_eq!(rev(&rebase, "HEAD"), tip, "back on the feature tip");
    assert_eq!(rebase.git(&["symbolic-ref", "--short", "HEAD"]), "feature");
    assert_eq!(rebase.git(&["status", "--porcelain"]), "");
}

/// A file the merge brought in whose cached stat data went stale (rewritten
/// with the same bytes, another mtime: what an antivirus, an indexer or a
/// line-ending filter does on Windows) must not stop the abort: git's own
/// `merge --abort` says "Entry 'x' not uptodate. Cannot merge." there, the
/// backends refresh the index and retry.
pub fn check_operation_abort_survives_a_stale_index_entry(backend: &dyn GitBackend) {
    let fx = ConflictFixture::text_with_clean_side_file();
    let head = fx.git(&["rev-parse", "HEAD"]);
    let side = fx.root().join("side.txt");
    let bytes = std::fs::read(&side).unwrap();
    std::fs::write(&side, &bytes).unwrap();
    let later = std::time::SystemTime::now() + std::time::Duration::from_secs(120);
    std::fs::File::options()
        .write(true)
        .open(&side)
        .unwrap()
        .set_modified(later)
        .unwrap();
    // The repro is real: git alone refuses (and changes nothing).
    assert!(
        fx.repo().try_git(&["reset", "--merge"]).is_none(),
        "the stale stat data must make a plain reset --merge fail"
    );
    backend.operation_abort(&fx.repo_ref()).unwrap();
    assert_eq!(backend.operation_state(&fx.repo_ref()).unwrap(), None);
    assert_eq!(fx.git(&["rev-parse", "HEAD"]), head);
    assert_eq!(fx.git(&["status", "--porcelain"]), "", "worktree clean");
    assert_eq!(
        std::fs::read(&side).unwrap(),
        b"side one\nside two\n",
        "the merged-in change is gone"
    );
}

/// A file with real unstaged changes the abort would overwrite is a typed
/// refusal naming it; the merge stays stopped and nothing is touched.
/// Discarding that file's changes lets the abort go through.
pub fn check_operation_abort_names_files_with_local_changes(backend: &dyn GitBackend) {
    let fx = ConflictFixture::text_with_clean_side_file();
    let side = fx.root().join("side.txt");
    std::fs::write(&side, "my own work\n").unwrap();
    match backend.operation_abort(&fx.repo_ref()) {
        Err(Error::ConflictRefused(ConflictRefusal::LocalChanges { paths })) => {
            assert_eq!(paths, vec!["side.txt".to_string()]);
        }
        other => panic!("expected LocalChanges, got {other:?}"),
    }
    assert!(matches!(
        backend.operation_state(&fx.repo_ref()).unwrap(),
        Some(OperationState::Merge { .. })
    ));
    assert_eq!(std::fs::read(&side).unwrap(), b"my own work\n");
    // The way out: drop that file's changes (back to what the merge staged),
    // then the abort succeeds.
    fx.git(&["checkout", "--", "side.txt"]);
    backend.operation_abort(&fx.repo_ref()).unwrap();
    assert_eq!(backend.operation_state(&fx.repo_ref()).unwrap(), None);
    assert_eq!(fx.git(&["status", "--porcelain"]), "");
}

/// Takes our side of every conflicted path and stages it: the resolved
/// result then equals HEAD.
fn resolve_all_ours(fx: &ConflictFixture, path: &str) {
    fx.git(&["checkout", "--ours", path]);
    fx.git(&["add", path]);
}

/// A merge whose resolution equals HEAD ("take ours" everywhere) can still
/// be concluded: the commit has nothing staged beyond HEAD and is the
/// merge commit, with both parents (maintainer rule: no state left only by
/// editing a file or running plain git).
pub fn check_commit_concludes_a_merge_whose_result_equals_head(backend: &dyn GitBackend) {
    let fx = ConflictFixture::text();
    let head = fx.git(&["rev-parse", "HEAD"]);
    let theirs = fx.git(&["rev-parse", "feature"]);
    let tree = fx.git(&["rev-parse", "HEAD^{tree}"]);
    resolve_all_ours(&fx, "notes.txt");
    assert_eq!(
        fx.git(&["diff", "--cached", "--name-only"]),
        "",
        "nothing staged"
    );
    let oid = backend
        .commit(
            &fx.repo_ref(),
            "Merge feature, ours",
            &CommitOptions::default(),
        )
        .expect("the merge commit is allowed with nothing staged");
    assert_eq!(backend.operation_state(&fx.repo_ref()).unwrap(), None);
    assert_eq!(fx.git(&["rev-parse", "HEAD"]), oid.to_hex());
    assert_eq!(
        fx.git(&["rev-list", "--parents", "-n1", "HEAD"]),
        format!("{} {head} {theirs}", oid.to_hex())
    );
    assert_eq!(fx.git(&["rev-parse", "HEAD^{tree}"]), tree);
    assert!(!fx.root().join(".git/MERGE_HEAD").exists());
}

/// An empty cherry-pick or revert is left with Skip: HEAD, index and
/// working tree untouched, the operation ended. Continue (a plain commit)
/// stays what git does - refused, pointing to Skip.
pub fn check_operation_skip_ends_an_empty_cherry_pick_or_revert(backend: &dyn GitBackend) {
    for op in [FixtureOperation::CherryPick, FixtureOperation::Revert] {
        let fx = ConflictFixture::text_with(op);
        let head = fx.git(&["rev-parse", "HEAD"]);
        resolve_all_ours(&fx, "notes.txt");
        // The pick is now empty: committing it is refused, nothing moves.
        let commit = backend.commit(&fx.repo_ref(), "empty", &CommitOptions::default());
        assert!(commit.is_err(), "{op:?}: an empty pick is not committed");
        assert_eq!(fx.git(&["rev-parse", "HEAD"]), head);
        assert!(backend.operation_state(&fx.repo_ref()).unwrap().is_some());

        backend.operation_skip(&fx.repo_ref()).unwrap();
        assert_eq!(
            backend.operation_state(&fx.repo_ref()).unwrap(),
            None,
            "{op:?}"
        );
        assert_eq!(fx.git(&["rev-parse", "HEAD"]), head, "{op:?}");
        assert_eq!(fx.git(&["status", "--porcelain"]), "", "{op:?}");
    }
}

/// Skip also drops a pick that is still conflicted, and refuses a merge
/// (nothing to skip) or no operation, changing nothing.
pub fn check_operation_skip_refusals(backend: &dyn GitBackend) {
    let pick = ConflictFixture::text_with(FixtureOperation::CherryPick);
    let head = pick.git(&["rev-parse", "HEAD"]);
    backend.operation_skip(&pick.repo_ref()).unwrap();
    assert_eq!(backend.operation_state(&pick.repo_ref()).unwrap(), None);
    assert_eq!(pick.git(&["rev-parse", "HEAD"]), head);
    assert_eq!(pick.git(&["status", "--porcelain"]), "");

    let merge = ConflictFixture::text();
    assert!(backend.operation_skip(&merge.repo_ref()).is_err());
    assert!(matches!(
        backend.operation_state(&merge.repo_ref()).unwrap(),
        Some(OperationState::Merge { .. })
    ));

    let clean = ConflictFixture::none_unmerged();
    match backend.operation_skip(&clean.repo_ref()) {
        Err(Error::ConflictRefused(ConflictRefusal::NoOperation)) => {}
        other => panic!("expected NoOperation, got {other:?}"),
    }
}

pub fn check_operation_abort_without_operation_is_a_typed_refusal(backend: &dyn GitBackend) {
    let clean = ConflictFixture::none_unmerged();
    assert_eq!(backend.operation_state(&clean.repo_ref()).unwrap(), None);
    let head = clean.git(&["rev-parse", "HEAD"]);
    match backend.operation_abort(&clean.repo_ref()) {
        Err(Error::ConflictRefused(ConflictRefusal::NoOperation)) => {}
        other => panic!("expected NoOperation, got {other:?}"),
    }
    assert_eq!(clean.git(&["rev-parse", "HEAD"]), head);
}

/// Redoing a resolution restores its snapshot over the conflicted path:
/// the three stages go, the resolved entry (or nothing, for a deletion)
/// takes their place.
pub fn check_snapshot_restore_replaces_a_conflicted_path_with_its_resolution(
    backend: &dyn GitBackend,
) {
    let fx = ConflictFixture::text();
    let repo = fx.repo_ref();
    let original = entries(backend, &fx);
    resolve_ok(
        backend,
        &fx,
        "notes.txt",
        ConflictResolution::UseContent("done\n".into()),
    );
    let snapshot = backend
        .snapshot_create(&repo, &["notes.txt".to_string()])
        .unwrap();
    backend
        .unresolve_conflict(&repo, Path::new("notes.txt"), &original[0])
        .unwrap();
    assert_eq!(entries(backend, &fx).len(), 1);
    backend
        .snapshot_restore(&repo, snapshot.tree, &["notes.txt".to_string()])
        .unwrap();
    assert!(entries(backend, &fx).is_empty(), "no stage left");
    assert_eq!(fx.stage_bytes(0, "notes.txt").unwrap(), b"done\n");
    assert_eq!(fx.worktree_bytes("notes.txt"), b"done\n");

    // A deletion: nothing in the snapshot, so nothing may be left behind.
    let fx = ConflictFixture::delete_modify(true);
    let repo = fx.repo_ref();
    let original = entries(backend, &fx);
    resolve_ok(backend, &fx, "gone.txt", ConflictResolution::Delete);
    let snapshot = backend
        .snapshot_create(&repo, &["gone.txt".to_string()])
        .unwrap();
    backend
        .unresolve_conflict(&repo, Path::new("gone.txt"), &original[0])
        .unwrap();
    backend
        .snapshot_restore(&repo, snapshot.tree, &["gone.txt".to_string()])
        .unwrap();
    assert!(entries(backend, &fx).is_empty());
    assert!(index_lines(&fx, "gone.txt").is_empty());
    assert!(!fx.root().join("gone.txt").exists());
}

/// Unstaging (one file, everything, the root folder) during a merge takes
/// back the staged resolutions and leaves every still-unmerged path's
/// conflict stages and marker file exactly as they were: an unmerged path
/// has no staged change to unstage.
pub fn check_unstage_one_path_during_a_merge_keeps_the_other_conflict_stages(
    backend: &dyn GitBackend,
) {
    use gitpoulpi_core::domain::{FileStatus, StageTarget};
    for (label, target) in [
        ("file", StageTarget::File("a.txt".into())),
        ("all", StageTarget::All),
        ("root folder", StageTarget::Folder(".".into())),
    ] {
        let fx = ConflictFixture::multi_conflict();
        let paths = fx.paths().to_vec();
        assert!(paths.len() >= 3);
        let resolved = "a.txt";
        let before = entries(backend, &fx);
        let disk_before: Vec<Vec<u8>> = paths.iter().map(|p| fx.worktree_bytes(p)).collect();
        resolve_ok(
            backend,
            &fx,
            resolved,
            ConflictResolution::UseContent("resolved by hand\n".into()),
        );
        backend.unstage(&fx.repo_ref(), &target).unwrap();
        let expected: Vec<_> = before
            .iter()
            .filter(|e| e.path.to_str() != Some(resolved))
            .cloned()
            .collect();
        assert_eq!(
            entries(backend, &fx),
            expected,
            "{label}: the other paths keep their stages"
        );
        for (p, bytes) in paths.iter().zip(&disk_before) {
            if p != resolved {
                assert_eq!(&fx.worktree_bytes(p), bytes, "{label} {p}: markers kept");
            }
        }
        assert_eq!(
            fx.worktree_bytes(resolved),
            b"resolved by hand\n".to_vec(),
            "{label}: the resolution stays in the working tree"
        );
        let tree = backend.status(&fx.repo_ref()).unwrap();
        for p in paths.iter().filter(|p| *p != resolved) {
            let e = tree
                .files
                .iter()
                .find(|e| e.path.to_str() == Some(p.as_str()))
                .unwrap_or_else(|| panic!("{p} missing from status"));
            assert_eq!(
                e.worktree,
                Some(FileStatus::Conflicted),
                "{label} {p}: {e:?}"
            );
        }
    }
}
