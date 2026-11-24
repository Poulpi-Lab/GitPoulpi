use super::*;

/// The diff of an LFS pointer file: three lines of `oid`/`size` text
/// would say nothing to a person, so a card with sizes and short oids
/// replaces them, and there is nothing hunk- or line-stageable in it.
fn lfs_pointer_diff(old_size: u64, new_size: u64) -> FileDiff {
    let old = format!(
        "version https://git-lfs.github.com/spec/v1\noid sha256:{}\nsize {old_size}\n",
        "a".repeat(64)
    );
    let new = format!(
        "version https://git-lfs.github.com/spec/v1\noid sha256:{}\nsize {new_size}\n",
        "b".repeat(64)
    );
    let mut lines = vec![line(
        LineOrigin::Context,
        "version https://git-lfs.github.com/spec/v1",
    )];
    lines.extend(old.lines().skip(1).map(|l| line(LineOrigin::Delete, l)));
    lines.extend(new.lines().skip(1).map(|l| line(LineOrigin::Add, l)));
    file(vec![hunk(lines)])
}

#[test]
fn an_lfs_file_diff_is_a_card_with_sizes_not_pointer_text_and_has_no_hunk_buttons() {
    let diff = lfs_pointer_diff(1_200_000, 1_400_000);
    let card = render_diff(&diff, DiffViewMode::Inline, true, false, 80);
    let text = plain(&card.lines).join("\n");
    assert!(text.contains("Git LFS object"), "{text}");
    assert!(text.contains("1.2 MB -> 1.4 MB"), "{text}");
    assert!(text.contains("aaaaaaaa"), "the short old oid: {text}");
    assert!(!text.contains("oid sha256"), "no raw pointer text: {text}");
    assert!(card.hunk_starts.is_empty(), "nothing to jump to or stage");
    assert!(
        card.rows.iter().all(|r| r.hunk.is_none() && !r.header),
        "no row may belong to a hunk, or a hunk button would appear"
    );

    // `r` flips to the raw pointer diff, hunks and all.
    let raw = render_diff_with(
        &diff,
        &DiffRenderOpts {
            mode: DiffViewMode::Inline,
            hunks_only: true,
            show_whitespace: true,
            word_wrap: false,
            width: 80,
            h_offset: 0,
            full_file: None,
            lfs_raw: true,
            lfs_local: None,
            intraline: false,
        },
    );
    assert!(plain(&raw.lines).join("\n").contains("oid sha256"));
    assert!(!raw.hunk_starts.is_empty());
}

#[test]
fn the_lfs_card_says_whether_the_new_object_is_stored_locally() {
    let diff = lfs_pointer_diff(10, 2000);
    let card_text = |local| {
        let change = gitpoulpi_core::domain::lfs_change(&diff).unwrap();
        plain(&lfs_card(&change, local).lines).join("\n")
    };
    assert!(card_text(Some(true)).contains("stored locally"));
    assert!(card_text(Some(false)).contains("not downloaded"));
    assert!(card_text(Some(false)).contains("2.0 KB"));
}
