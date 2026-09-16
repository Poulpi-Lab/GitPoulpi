use super::*;
use crate::domain::{ConflictKind, Oid, MODE_GITLINK};
use ConflictSide::{Base, Ours, Theirs};

fn lines(texts: &[&str]) -> Vec<String> {
    texts.iter().map(|s| s.to_string()).collect()
}

fn merged(texts: &[&str]) -> MergeRegion {
    MergeRegion::Merged {
        lines: lines(texts),
    }
}

fn conflict(base: &[&str], ours: &[&str], theirs: &[&str]) -> MergeRegion {
    MergeRegion::Conflict {
        base: lines(base),
        ours: lines(ours),
        theirs: lines(theirs),
    }
}

fn zero_oid() -> Oid {
    Oid::from_hex(&"0".repeat(40)).unwrap()
}

fn stage() -> StageEntry {
    StageEntry {
        mode: 0o100644,
        oid: zero_oid(),
    }
}

fn entry() -> ConflictEntry {
    ConflictEntry::from_stages("f.txt".into(), Some(stage()), Some(stage()), Some(stage())).unwrap()
}

fn doc(regions: Vec<MergeRegion>) -> MergeDocument {
    MergeDocument {
        entry: entry(),
        base: None,
        ours: None,
        theirs: None,
        regions,
        labels: SideLabels::default(),
        worktree: WorktreeFileState::Markers,
        eol: Eol::Lf,
        bom: false,
        final_newline: true,
        encoding: None,
        marker_size: 7,
    }
}

/// A tiny deterministic generator, so the property-style tests need no
/// dependency and fail the same way every run.
struct Lcg(u64);
impl Lcg {
    fn next(&mut self) -> usize {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (self.0 >> 33) as usize
    }
    fn below(&mut self, n: usize) -> usize {
        self.next() % n
    }
    fn line(&mut self) -> String {
        // Lookalikes included on purpose.
        ["a", "b", "c", "d", "", "=======", "<<<<<<< x", "fn x() {"][self.below(8)].to_string()
    }
    fn block(&mut self, max: usize) -> Vec<String> {
        (0..self.below(max + 1)).map(|_| self.line()).collect()
    }
    fn regions(&mut self) -> Vec<MergeRegion> {
        let mut out = Vec::new();
        for _ in 0..self.below(6) {
            if self.below(2) == 0 {
                let l = self.block(3);
                if !l.is_empty() {
                    out.push(MergeRegion::Merged { lines: l });
                }
            } else {
                out.push(MergeRegion::Conflict {
                    base: self.block(3),
                    ours: self.block(4),
                    theirs: self.block(4),
                });
            }
        }
        // Adjacent merged regions would parse as one.
        let mut fused: Vec<MergeRegion> = Vec::new();
        for r in out {
            match (fused.last_mut(), r) {
                (Some(MergeRegion::Merged { lines }), MergeRegion::Merged { lines: more }) => {
                    lines.extend(more)
                }
                (_, r) => fused.push(r),
            }
        }
        fused
    }
}

// ---- marker size ---------------------------------------------------------

#[test]
fn marker_size_is_seven_unless_a_line_looks_like_a_marker() {
    assert_eq!(marker_size_for(&[b"a\nb\n", b"", b"x"]), 7);
    assert_eq!(marker_size_for(&[b"title\n=======\n"]), 8);
    assert_eq!(marker_size_for(&[b"<<<<<<<<<<\n", b"=======\n"]), 11);
    // Only a *leading* run counts, and only of marker characters.
    assert_eq!(marker_size_for(&[b"a =======\n", b" =========\n"]), 7);
    assert_eq!(marker_size_for(&[b"|||||||||\n"]), 10);
}

// ---- parser --------------------------------------------------------------

#[test]
fn parses_a_diff3_merge() {
    let out =
        b"top\n<<<<<<< ours\nmine\n||||||| base\norig\n=======\ntheirs\n>>>>>>> theirs\nbottom\n";
    assert_eq!(
        parse_merge_file_output(out, 7).unwrap(),
        vec![
            merged(&["top"]),
            conflict(&["orig"], &["mine"], &["theirs"]),
            merged(&["bottom"]),
        ]
    );
}

#[test]
fn parses_a_plain_merge_with_empty_bases() {
    let out = b"<<<<<<< a\nx\n=======\ny\n>>>>>>> b\n";
    assert_eq!(
        parse_merge_file_output(out, 7).unwrap(),
        vec![conflict(&[], &["x"], &["y"])]
    );
}

#[test]
fn parses_empty_sides_and_an_empty_output() {
    assert_eq!(parse_merge_file_output(b"", 7).unwrap(), vec![]);
    let out = b"<<<<<<< a\n||||||| o\nold\n=======\n>>>>>>> b\n";
    assert_eq!(
        parse_merge_file_output(out, 7).unwrap(),
        vec![conflict(&["old"], &[], &[])]
    );
}

#[test]
fn a_last_line_without_a_newline_is_still_a_line() {
    assert_eq!(
        parse_merge_file_output(b"a\nb", 7).unwrap(),
        vec![merged(&["a", "b"])]
    );
    // A blank last line is a line too.
    assert_eq!(
        parse_merge_file_output(b"a\n\n", 7).unwrap(),
        vec![merged(&["a", ""])]
    );
}

#[test]
fn carriage_returns_stay_in_the_lines_and_crlf_markers_parse() {
    let out = b"a\r\n<<<<<<< o\r\nx\r\n=======\r\ny\r\n>>>>>>> t\r\nb\r\n";
    assert_eq!(
        parse_merge_file_output(out, 7).unwrap(),
        vec![
            merged(&["a\r"]),
            conflict(&[], &["x\r"], &["y\r"]),
            merged(&["b\r"]),
        ]
    );
}

#[test]
fn lines_that_look_like_markers_of_another_size_are_content() {
    // Marker size 8: a 7-wide `=======` line is the file's own text.
    let out = b"<<<<<<<< a\n=======\nkept\n======== \n";
    assert!(parse_merge_file_output(out, 8).is_err(), "unterminated");
    let out = b"<<<<<<<< a\n=======\nkept\n========\nx\n>>>>>>>> b\n";
    assert_eq!(
        parse_merge_file_output(out, 8).unwrap(),
        vec![conflict(&[], &["=======", "kept"], &["x"])]
    );
    // And so is a marker followed by more marker characters or text.
    let out = b"=========\n<<<<<<<x\n";
    assert_eq!(
        parse_merge_file_output(out, 7).unwrap(),
        vec![merged(&["=========", "<<<<<<<x"])]
    );
}

#[test]
fn malformed_output_is_an_error_not_a_guess() {
    assert!(parse_merge_file_output(b"<<<<<<< a\nx\n", 7).is_err());
    assert!(parse_merge_file_output(b"x\n=======\n", 7).is_err());
    assert!(parse_merge_file_output(b"x\n>>>>>>> b\n", 7).is_err());
    assert!(parse_merge_file_output(b"<<<<<<< a\n<<<<<<< a\n", 7).is_err());
}

#[test]
fn rendering_then_parsing_gives_the_regions_back() {
    let mut rng = Lcg(7);
    for _ in 0..300 {
        let regions = rng.regions();
        let all: Vec<u8> = regions
            .iter()
            .flat_map(|r| [r.lines_of(Base), r.lines_of(Ours), r.lines_of(Theirs)].concat())
            .flat_map(|l| format!("{l}\n").into_bytes())
            .collect();
        let marker = marker_size_for(&[&all]);
        for diff3 in [true, false] {
            let text = render_with_markers(
                &regions,
                &MarkerStyle {
                    marker_size: marker,
                    diff3,
                    ours_label: "HEAD",
                    base_label: "base",
                    theirs_label: "feature",
                    eol: "\n",
                    final_newline: true,
                },
            );
            let parsed = parse_merge_file_output(text.as_bytes(), marker).unwrap();
            let expect: Vec<MergeRegion> = if diff3 {
                regions.clone()
            } else {
                regions
                    .iter()
                    .map(|r| match r {
                        MergeRegion::Conflict { ours, theirs, .. } => MergeRegion::Conflict {
                            base: vec![],
                            ours: ours.clone(),
                            theirs: theirs.clone(),
                        },
                        m => m.clone(),
                    })
                    .collect()
            };
            assert_eq!(parsed, expect, "diff3={diff3}\n{text}");
        }
    }
}

#[test]
fn rendering_matches_gits_own_layout() {
    let regions = vec![
        merged(&["a"]),
        conflict(&["o"], &["x"], &["y"]),
        merged(&["z"]),
    ];
    let style = |diff3| MarkerStyle {
        marker_size: 7,
        diff3,
        ours_label: "HEAD",
        base_label: "merged common ancestors",
        theirs_label: "feature",
        eol: "\n",
        final_newline: true,
    };
    assert_eq!(
        render_with_markers(&regions, &style(false)),
        "a\n<<<<<<< HEAD\nx\n=======\ny\n>>>>>>> feature\nz\n"
    );
    assert_eq!(
        render_with_markers(&regions, &style(true)),
        "a\n<<<<<<< HEAD\nx\n||||||| merged common ancestors\no\n=======\ny\n>>>>>>> feature\nz\n"
    );
    let crlf = MarkerStyle {
        eol: "\r\n",
        final_newline: false,
        ..style(false)
    };
    assert_eq!(
        render_with_markers(&regions, &crlf),
        "a\r\n<<<<<<< HEAD\r\nx\r\n=======\r\ny\r\n>>>>>>> feature\r\nz"
    );
}

// ---- compose -------------------------------------------------------------

fn text_of(rows: &[&str]) -> String {
    rows.iter().map(|l| format!("{l}\n")).collect()
}

#[test]
fn compose_takes_a_side_or_both_in_the_order_given() {
    let regions = vec![
        merged(&["top"]),
        conflict(&["o"], &["a1", "a2"], &["b1"]),
        merged(&["mid"]),
        conflict(&[], &["c"], &["d"]),
    ];
    let d = doc(regions.clone());
    let take = |sides: &[ConflictSide]| -> Vec<ConflictPicks> {
        regions
            .iter()
            .filter(|r| r.is_conflict())
            .map(|r| ConflictPicks::take(r, sides))
            .collect()
    };
    assert_eq!(
        compose_resolution(&d, &take(&[Ours])),
        text_of(&["top", "a1", "a2", "mid", "c"])
    );
    assert_eq!(
        compose_resolution(&d, &take(&[Theirs])),
        text_of(&["top", "b1", "mid", "d"])
    );
    assert_eq!(
        compose_resolution(&d, &take(&[Ours, Theirs])),
        text_of(&["top", "a1", "a2", "b1", "mid", "c", "d"])
    );
    assert_eq!(
        compose_resolution(&d, &take(&[Theirs, Ours])),
        text_of(&["top", "b1", "a1", "a2", "mid", "d", "c"])
    );
    assert_eq!(
        compose_resolution(&d, &take(&[Base])),
        text_of(&["top", "o", "mid"])
    );
    assert_eq!(compose_resolution(&d, &take(&[])), text_of(&["top", "mid"]));
    // A conflict with no entry resolves to nothing.
    assert_eq!(compose_resolution(&d, &[]), text_of(&["top", "mid"]));
}

#[test]
fn compose_keeps_file_order_within_a_side_and_ignores_stale_picks() {
    let d = doc(vec![conflict(&[], &["a", "b", "c"], &["x", "y"])]);
    let picks = ConflictPicks {
        picks: vec![
            Pick {
                side: Theirs,
                line: 1,
            },
            Pick {
                side: Ours,
                line: 2,
            },
            Pick {
                side: Ours,
                line: 0,
            },
            Pick {
                side: Ours,
                line: 0,
            },
            Pick {
                side: Ours,
                line: 9,
            },
        ],
        order: vec![Theirs, Ours],
    };
    assert_eq!(
        compose_resolution(&d, std::slice::from_ref(&picks)),
        text_of(&["y", "a", "c"])
    );
    // A side with picks but missing from `order` follows the listed ones.
    let unordered = ConflictPicks {
        order: vec![],
        ..picks
    };
    assert_eq!(
        compose_resolution(&d, &[unordered]),
        text_of(&["a", "c", "y"])
    );
}

#[test]
fn compose_honours_a_missing_final_newline() {
    let mut d = doc(vec![merged(&["a"]), conflict(&[], &["x"], &["y"])]);
    d.final_newline = false;
    let picks = [ConflictPicks::take(&d.regions[1], &[Ours])];
    assert_eq!(compose_resolution(&d, &picks), "a\nx");
}

#[test]
fn composing_a_whole_side_gives_that_sides_projection() {
    let mut rng = Lcg(42);
    for _ in 0..300 {
        let regions = rng.regions();
        let d = doc(regions.clone());
        for side in [Ours, Theirs, Base] {
            let picks: Vec<ConflictPicks> = regions
                .iter()
                .filter(|r| r.is_conflict())
                .map(|r| ConflictPicks::take(r, &[side]))
                .collect();
            let expect: String = regions
                .iter()
                .flat_map(|r| r.lines_of(side).to_vec())
                .map(|l| format!("{l}\n"))
                .collect();
            assert_eq!(
                compose_resolution(&d, &picks),
                expect,
                "{side:?} of {regions:?}"
            );
        }
        // Both sides: the two projections interleave per conflict, so the
        // line count is the sum, whichever order.
        for sides in [[Ours, Theirs], [Theirs, Ours]] {
            let picks: Vec<ConflictPicks> = regions
                .iter()
                .filter(|r| r.is_conflict())
                .map(|r| ConflictPicks::take(r, &sides))
                .collect();
            let want: usize = regions
                .iter()
                .map(|r| match r {
                    MergeRegion::Merged { lines } => lines.len(),
                    MergeRegion::Conflict { ours, theirs, .. } => ours.len() + theirs.len(),
                })
                .sum();
            assert_eq!(compose_resolution(&d, &picks).lines().count(), want);
        }
    }
}

#[test]
fn picks_are_all_or_nothing_neutral_for_merged_regions() {
    let r = merged(&["a"]);
    assert_eq!(ConflictPicks::take(&r, &[Ours]), ConflictPicks::none());
}

// ---- classify and assemble -------------------------------------------------

fn raw(bytes: &[u8]) -> RawStage {
    RawStage {
        entry: stage(),
        size: bytes.len() as u64,
        bytes: Some(bytes.to_vec()),
    }
}

#[test]
fn only_a_plain_text_content_conflict_gets_a_merge_job() {
    let e = entry();
    let (blobs, job) = classify_sides(
        &e,
        [Some(raw(b"o\n")), Some(raw(b"a\n")), Some(raw(b"b\n"))],
    );
    assert!(blobs.iter().all(Option::is_some));
    let job = job.expect("text on every side");
    assert_eq!((job.base.as_slice(), job.marker), (&b"o\n"[..], 7));

    // Binary on any side: no merge.
    let (blobs, job) = classify_sides(
        &e,
        [Some(raw(b"o\n")), Some(raw(b"a\0\n")), Some(raw(b"b\n"))],
    );
    assert!(job.is_none());
    assert_eq!(blobs[1].as_ref().unwrap().content, BlobContent::Binary);

    // Too large: reported, not read.
    let big = RawStage {
        entry: stage(),
        size: MAX_TEXT_BLOB_BYTES + 1,
        bytes: None,
    };
    let (blobs, job) = classify_sides(&e, [Some(raw(b"o\n")), Some(big), Some(raw(b"b\n"))]);
    assert!(job.is_none());
    assert_eq!(blobs[1].as_ref().unwrap().content, BlobContent::TooLarge);
    assert_eq!(blobs[1].as_ref().unwrap().size, MAX_TEXT_BLOB_BYTES + 1);

    // Add/add: the merge runs against an empty base.
    let add = ConflictEntry::from_stages("f".into(), None, Some(stage()), Some(stage())).unwrap();
    assert_eq!(add.kind, ConflictKind::BothAdded);
    let (_, job) = classify_sides(&add, [None, Some(raw(b"a\n")), Some(raw(b"b\n"))]);
    assert_eq!(job.unwrap().base, Vec::<u8>::new());

    // Delete/modify: nothing to merge.
    let dm = ConflictEntry::from_stages("f".into(), Some(stage()), None, Some(stage())).unwrap();
    assert_eq!(dm.kind, ConflictKind::DeletedByUs);
    let (_, job) = classify_sides(&dm, [Some(raw(b"o\n")), None, Some(raw(b"b\n"))]);
    assert!(job.is_none());

    // A submodule's oid is a commit elsewhere: never read.
    let gl = StageEntry {
        mode: MODE_GITLINK,
        oid: zero_oid(),
    };
    let sm = ConflictEntry::from_stages("sub".into(), Some(gl), Some(gl), Some(gl)).unwrap();
    assert!(sm.is_submodule());
    let g = RawStage {
        entry: gl,
        size: 0,
        bytes: None,
    };
    let (blobs, job) = classify_sides(&sm, [Some(g.clone()), Some(g.clone()), Some(g)]);
    assert!(job.is_none());
    assert_eq!(blobs[1].as_ref().unwrap().content, BlobContent::Binary);
}

#[test]
fn a_marker_lookalike_raises_the_marker_size_of_the_job() {
    let e = entry();
    let (_, job) = classify_sides(
        &e,
        [
            Some(raw(b"=======\n")),
            Some(raw(b"a\n")),
            Some(raw(b"b\n")),
        ],
    );
    assert_eq!(job.unwrap().marker, 8);
}

fn assembled(
    base: &[u8],
    ours: &[u8],
    theirs: &[u8],
    merged_out: &[u8],
    worktree: Option<&[u8]>,
) -> MergeDocument {
    let e = entry();
    let (blobs, job) = classify_sides(&e, [Some(raw(base)), Some(raw(ours)), Some(raw(theirs))]);
    let job = job.unwrap();
    MergeDocument::assemble(
        e,
        blobs,
        Some((job.marker, merged_out.to_vec())),
        SideLabels::default(),
        worktree,
    )
    .unwrap()
}

#[test]
fn assemble_strips_cr_and_bom_and_remembers_them() {
    let d = assembled(
        b"\xEF\xBB\xBFo\r\n",
        b"\xEF\xBB\xBFa\r\n",
        b"\xEF\xBB\xBFb\r\n",
        b"<<<<<<< x\r\n\xEF\xBB\xBFa\r\n=======\r\n\xEF\xBB\xBFb\r\n>>>>>>> y\r\n",
        Some(b"<<<<<<< x\r\nz\r\n=======\r\n>>>>>>> y\r\n"),
    );
    assert!(d.bom);
    assert_eq!(d.eol, Eol::Crlf);
    assert_eq!(d.worktree, WorktreeFileState::Markers);
    assert_eq!(d.regions, vec![conflict(&[], &["a"], &["b"])]);
    assert_eq!(d.ours.as_ref().unwrap().lines(), ["a"]);
    assert_eq!(d.theirs.as_ref().unwrap().lines(), ["b"]);
    assert!(d.final_newline);
}

#[test]
fn the_output_eol_follows_the_worktree_then_stage_two() {
    let two_way = b"<<<<<<< x\na\n=======\nb\n>>>>>>> y\n";
    // Uniform CRLF on disk wins over an LF stage 2.
    let d = assembled(b"o\n", b"a\n", b"b\n", two_way, Some(b"x\r\ny\r\n"));
    assert_eq!(d.eol, Eol::Crlf);
    // Uniform LF on disk wins over a CRLF stage 2.
    let d = assembled(b"o\r\n", b"a\r\n", b"b\r\n", two_way, Some(b"x\ny\n"));
    assert_eq!(d.eol, Eol::Lf);
    // Mixed or absent: stage 2's.
    let d = assembled(b"o\r\n", b"a\r\n", b"b\r\n", two_way, Some(b"x\r\ny\n"));
    assert_eq!(d.eol, Eol::Crlf);
    let d = assembled(b"o\n", b"a\n", b"b\n", two_way, None);
    assert_eq!(d.eol, Eol::Lf);
    assert_eq!(d.worktree, WorktreeFileState::Missing);
}

#[test]
fn the_worktree_state_tells_markers_from_hand_edits_from_a_clean_merge() {
    let clean = b"same\nmerged\n";
    let d = assembled(b"o\n", b"a\n", b"b\n", clean, Some(b"same\nmerged\n"));
    assert_eq!(d.worktree, WorktreeFileState::CleanMatchesMerge);
    let d = assembled(b"o\n", b"a\n", b"b\n", clean, Some(b"same\r\nmerged\r\n"));
    assert_eq!(
        d.worktree,
        WorktreeFileState::CleanMatchesMerge,
        "CR is not content"
    );
    let d = assembled(b"o\n", b"a\n", b"b\n", clean, Some(b"hand edited\r\n"));
    assert_eq!(
        d.worktree,
        WorktreeFileState::EditedNoMarkers { crlf: true }
    );
    let two_way = b"<<<<<<< x\na\n=======\nb\n>>>>>>> y\n";
    let d = assembled(b"o\n", b"a\n", b"b\n", two_way, Some(b"resolved\n"));
    assert_eq!(
        d.worktree,
        WorktreeFileState::EditedNoMarkers { crlf: false }
    );
}

#[test]
fn a_merge_without_a_final_newline_says_so() {
    let d = assembled(
        b"o",
        b"a",
        b"b",
        b"<<<<<<< x\na\n=======\nb\n>>>>>>> y\nend",
        None,
    );
    assert!(!d.final_newline);
    assert_eq!(d.regions.last(), Some(&merged(&["end"])));
}

#[test]
fn conflict_kinds_come_from_the_stages_that_exist() {
    use ConflictKind::*;
    let k = ConflictKind::from_stages;
    assert_eq!(k(true, true, true), Some(BothModified));
    assert_eq!(k(false, true, true), Some(BothAdded));
    assert_eq!(k(true, false, true), Some(DeletedByUs));
    assert_eq!(k(true, true, false), Some(DeletedByThem));
    assert_eq!(k(true, false, false), Some(BothDeleted));
    assert_eq!(k(false, true, false), Some(AddedByUs));
    assert_eq!(k(false, false, true), Some(AddedByThem));
    assert_eq!(k(false, false, false), None);
}

#[test]
fn entry_helpers_read_modes() {
    let exec = StageEntry {
        mode: 0o100755,
        oid: zero_oid(),
    };
    let e =
        ConflictEntry::from_stages("f".into(), Some(stage()), Some(stage()), Some(exec)).unwrap();
    assert!(e.mode_changed());
    assert!(!e.is_symlink() && !e.is_submodule());
    assert_eq!(e.stage(Theirs), Some(exec));
    assert!(!entry().mode_changed());
}

#[test]
fn has_conflict_markers_needs_a_real_marker_line() {
    assert!(has_conflict_markers(b"a\n<<<<<<< HEAD\nb\n"));
    assert!(has_conflict_markers(b"a\r\n>>>>>>>\r\n"));
    assert!(has_conflict_markers(b"<<<<<<<<<< x\n"));
    assert!(!has_conflict_markers(b"a\n<<<<<<<x\n=======\n"));
    assert!(!has_conflict_markers(b"a << b\n"));
}

// ---- alignment ---------------------------------------------------------------

/// The ours file and the theirs file a set of regions describes when each
/// side's own changes are exactly its conflict blocks.
fn projection(regions: &[MergeRegion], side: ConflictSide) -> Vec<String> {
    regions
        .iter()
        .flat_map(|r| r.lines_of(side).to_vec())
        .collect()
}

fn shown(rows: &AlignedRows, side: ConflictSide) -> Vec<Option<usize>> {
    rows.side(side).iter().map(|r| r.line).collect()
}

#[test]
fn conflict_blocks_line_up_and_the_shorter_side_gets_fillers() {
    let regions = vec![
        merged(&["a"]),
        conflict(&["o"], &["x1", "x2", "x3"], &["y1"]),
        merged(&["z"]),
    ];
    let ours = projection(&regions, Ours);
    let theirs = projection(&regions, Theirs);
    let rows = aligned_rows(&regions, [None, Some(&ours), Some(&theirs)]);
    assert_eq!(rows.degraded, [false; 3]);
    assert_eq!(
        shown(&rows, Ours),
        [Some(0), Some(1), Some(2), Some(3), Some(4)]
    );
    assert_eq!(
        shown(&rows, Theirs),
        [Some(0), Some(1), None, None, Some(2)]
    );
    let marks: Vec<RowMark> = rows.side(Theirs).iter().map(|r| r.mark).collect();
    assert_eq!(
        marks,
        [
            RowMark::Same,
            RowMark::Same,
            RowMark::Absent,
            RowMark::Absent,
            RowMark::Same
        ]
    );
    assert_eq!(
        rows.rows.iter().map(|r| r.conflict).collect::<Vec<_>>(),
        [None, Some(0), Some(0), Some(0), None]
    );
    assert_eq!(rows.conflict_range(0, Ours, ours.len()), Some(1..4));
    assert_eq!(rows.conflict_range(0, Theirs, theirs.len()), Some(1..2));
}

#[test]
fn a_change_only_the_other_side_made_shows_as_a_tinted_row() {
    // Theirs also changed `t` -> `t2` away from the conflict; ours still
    // has `t`.
    let regions = vec![
        conflict(&["o"], &["x"], &["y"]),
        merged(&["keep", "t2", "end"]),
    ];
    let ours: Vec<String> = lines(&["x", "keep", "t", "end"]);
    let theirs: Vec<String> = lines(&["y", "keep", "t2", "end"]);
    let rows = aligned_rows(&regions, [None, Some(&ours), Some(&theirs)]);
    assert_eq!(rows.rows.len(), 4, "row for row: {rows:?}");
    let o = rows.side(Ours);
    assert_eq!(o[2].line, Some(2));
    assert_eq!(o[2].mark, RowMark::Changed);
    assert_eq!(rows.side(Theirs)[2].mark, RowMark::Same);
}

#[test]
fn a_line_the_other_side_removed_still_has_a_row() {
    // Theirs deleted `gone`; git merged the deletion in. Ours file still
    // shows it, on a row of its own, so every line of A is on screen.
    let regions = vec![merged(&["a", "b"])];
    let ours = lines(&["a", "gone", "b"]);
    let theirs = lines(&["a", "b"]);
    let rows = aligned_rows(&regions, [None, Some(&ours), Some(&theirs)]);
    assert_eq!(shown(&rows, Ours), [Some(0), Some(1), Some(2)]);
    assert_eq!(shown(&rows, Theirs), [Some(0), None, Some(1)]);
    assert_eq!(rows.side(Ours)[1].mark, RowMark::Extra);
}

#[test]
fn adjacent_conflicts_and_conflicts_at_the_edges_align() {
    let regions = vec![
        conflict(&[], &["a"], &["b"]),
        conflict(&["m"], &["c", "d"], &[]),
        merged(&["mid"]),
        conflict(&[], &[], &["tail"]),
    ];
    let ours = projection(&regions, Ours);
    let theirs = projection(&regions, Theirs);
    let rows = aligned_rows(&regions, [None, Some(&ours), Some(&theirs)]);
    assert_eq!(rows.degraded, [false; 3]);
    assert_eq!(
        rows.rows.iter().map(|r| r.conflict).collect::<Vec<_>>(),
        [Some(0), Some(1), Some(1), None, Some(2)]
    );
    assert_eq!(
        shown(&rows, Ours),
        [Some(0), Some(1), Some(2), Some(3), None]
    );
    assert_eq!(
        shown(&rows, Theirs),
        [Some(0), None, None, Some(1), Some(2)]
    );
    // An empty block is an empty range where the block sits.
    assert_eq!(rows.conflict_range(1, Theirs, theirs.len()), Some(1..1));
    assert_eq!(rows.conflict_range(2, Ours, ours.len()), Some(4..4));
    assert_eq!(rows.conflict_range(9, Ours, ours.len()), None);
}

#[test]
fn the_base_pane_aligns_like_the_others() {
    let regions = vec![merged(&["a"]), conflict(&["o1", "o2"], &["x"], &["y"])];
    let base = projection(&regions, Base);
    let ours = projection(&regions, Ours);
    let theirs = projection(&regions, Theirs);
    let rows = aligned_rows(&regions, [Some(&base), Some(&ours), Some(&theirs)]);
    assert_eq!(rows.rows.len(), 3);
    assert_eq!(shown(&rows, Base), [Some(0), Some(1), Some(2)]);
    assert_eq!(shown(&rows, Ours), [Some(0), Some(1), None]);
}

#[test]
fn side_rows_alone_follow_the_frame() {
    let regions = vec![merged(&["a"]), conflict(&[], &["x", "x2"], &["y"])];
    let ours = projection(&regions, Ours);
    let rows = side_rows(&ours, &regions, Ours);
    assert_eq!(rows.len(), 3);
    assert_eq!(rows[1].conflict, Some(0));
    assert_eq!(rows[2].line, Some(2));
}

#[test]
fn every_line_of_every_side_is_on_exactly_one_row() {
    let mut rng = Lcg(2024);
    for _ in 0..400 {
        let regions = rng.regions();
        // Each side's file: its projection, with a few lines the *other*
        // side's auto-merge would have removed or changed.
        let mut files: Vec<Vec<String>> = Vec::new();
        for side in [Base, Ours, Theirs] {
            let mut f = projection(&regions, side);
            for _ in 0..rng.below(3) {
                let at = rng.below(f.len() + 1);
                if rng.below(2) == 0 {
                    f.insert(at, format!("extra{}", rng.below(100)));
                } else if at < f.len() {
                    f[at] = format!("changed{}", rng.below(100));
                }
            }
            files.push(f);
        }
        let rows = aligned_rows(
            &regions,
            [Some(&files[0]), Some(&files[1]), Some(&files[2])],
        );
        for (s, side) in [Base, Ours, Theirs].into_iter().enumerate() {
            let present: Vec<usize> = shown(&rows, side).into_iter().flatten().collect();
            let want: Vec<usize> = (0..files[s].len()).collect();
            assert_eq!(
                present, want,
                "{side:?}: every line once, in order\n{regions:?}\n{files:?}"
            );
        }
        // Conflict rows exist for every conflict.
        let n = regions.iter().filter(|r| r.is_conflict()).count();
        for c in 0..n {
            assert!(
                rows.rows.iter().any(|r| r.conflict == Some(c)) || {
                    // A conflict whose three blocks are all empty has no rows.
                    regions
                        .iter()
                        .filter(|r| r.is_conflict())
                        .nth(c)
                        .is_some_and(|r| {
                            [Base, Ours, Theirs]
                                .iter()
                                .all(|&s| r.lines_of(s).is_empty())
                        })
                }
            );
        }
    }
}

#[test]
fn identical_files_never_degrade() {
    let mut rng = Lcg(5);
    for _ in 0..200 {
        let regions = rng.regions();
        let ours = projection(&regions, Ours);
        let theirs = projection(&regions, Theirs);
        let rows = aligned_rows(&regions, [None, Some(&ours), Some(&theirs)]);
        assert_eq!(rows.degraded, [false; 3], "{regions:?}");
    }
}

// -- whitespace-only conflicts ------------------------------------------------

#[test]
fn trailing_whitespace_and_cr_differences_keep_our_side() {
    let base = lines(&["a b"]);
    assert_eq!(
        whitespace_choice(&base, &lines(&["a b  "]), &lines(&["a b\t"])),
        Some(Ours)
    );
    assert_eq!(
        whitespace_choice(&[], &lines(&["x", "y "]), &lines(&["x", "y"])),
        Some(Ours)
    );
}

#[test]
fn a_side_equal_to_the_base_gives_way_to_the_other() {
    let base = lines(&["    a"]);
    assert_eq!(
        whitespace_choice(&base, &lines(&["    a"]), &lines(&["  a"])),
        Some(Theirs)
    );
    assert_eq!(
        whitespace_choice(&base, &lines(&["a"]), &lines(&["    a"])),
        Some(Ours)
    );
}

#[test]
fn indentation_and_blank_line_differences_are_left_to_the_person() {
    let base = lines(&["a"]);
    // Both changed the indentation, differently: which one is right?
    assert_eq!(
        whitespace_choice(&base, &lines(&["  a"]), &lines(&["\ta"])),
        None
    );
    assert!(is_whitespace_only(&lines(&["  a"]), &lines(&["\ta"])));
    // A blank line added on one side, the base differing from both.
    assert_eq!(
        whitespace_choice(&lines(&["z"]), &lines(&["a", ""]), &lines(&["a"])),
        None
    );
    // Real differences are not whitespace-only at all.
    assert!(!is_whitespace_only(&lines(&["a"]), &lines(&["b"])));
    assert!(!is_whitespace_only(&lines(&["a"]), &lines(&["a"])));
    assert_eq!(
        whitespace_choice(&base, &lines(&["a"]), &lines(&["b"])),
        None
    );
}

fn text_blob() -> Option<SideBlob> {
    Some(SideBlob {
        oid: zero_oid(),
        mode: 0o100644,
        size: 1,
        content: BlobContent::Text {
            lines: lines(&["x"]),
            crlf: false,
            final_newline: true,
        },
    })
}

#[test]
fn a_file_resolves_only_when_every_conflict_is_unambiguous() {
    let mut d = doc(vec![
        merged(&["top"]),
        conflict(&["a"], &["a "], &["a"]),
        merged(&["mid"]),
        conflict(&["b"], &["b"], &["b\t"]),
    ]);
    d.base = text_blob();
    d.ours = text_blob();
    d.theirs = text_blob();
    let picks = whitespace_resolution(&d).expect("both are trailing-whitespace conflicts");
    assert_eq!(picks.len(), 2);
    assert_eq!(compose_resolution(&d, &picks), "top\na \nmid\nb\t\n");

    // One real conflict spoils the file: nothing is picked for any.
    d.regions.push(conflict(&["c"], &["c1"], &["c2"]));
    assert_eq!(whitespace_resolution(&d), None);
    // As does one ambiguous one.
    d.regions.pop();
    d.regions.push(conflict(&["c"], &["  c"], &["\tc"]));
    assert_eq!(whitespace_resolution(&d), None);
}

#[test]
fn only_untouched_plain_text_conflicts_are_considered() {
    let mut d = doc(vec![conflict(&["a"], &["a "], &["a"])]);
    d.base = text_blob();
    d.ours = text_blob();
    d.theirs = text_blob();
    assert!(whitespace_resolution(&d).is_some());

    let mut edited = d.clone();
    edited.worktree = WorktreeFileState::EditedNoMarkers { crlf: false };
    assert_eq!(whitespace_resolution(&edited), None, "somebody edited it");

    let mut binary = d.clone();
    binary.ours = Some(SideBlob {
        content: BlobContent::Binary,
        ..text_blob().unwrap()
    });
    assert_eq!(whitespace_resolution(&binary), None);

    let mut none = d.clone();
    none.regions = vec![merged(&["a"])];
    assert_eq!(whitespace_resolution(&none), None, "no conflict at all");
}

#[test]
fn side_changes_marks_added_changed_and_removed_lines() {
    let base = lines(&["a", "b", "c", "d"]);
    let side = lines(&["a", "B", "c", "new", "d"]);
    let ch = side_changes(&base, &side);
    assert_eq!(
        ch.lines,
        vec![
            LineChange::Same,
            LineChange::Changed,
            LineChange::Same,
            LineChange::Added,
            LineChange::Same
        ]
    );
    assert!(ch.removed_before.iter().all(|r| !r));
    let ch = side_changes(&base, &lines(&["a", "d"]));
    assert_eq!(ch.lines, vec![LineChange::Same, LineChange::Same]);
    assert_eq!(ch.removed_before, vec![false, true, false]);
}
