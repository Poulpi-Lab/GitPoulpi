//! The conflict fixtures must stop on exactly the stages the contract
//! checks assume, or those checks would pass or fail for the wrong reason.

use gitpoulpi_testkit::{ConflictFixture, FixtureOperation};

/// `path -> stages present`, from `git ls-files -u`.
fn stages(f: &ConflictFixture) -> Vec<(String, String)> {
    let mut by_path: Vec<(String, String)> = Vec::new();
    for line in f.git(&["ls-files", "-u"]).lines() {
        let (meta, path) = line.split_once('\t').unwrap();
        let stage = meta.rsplit(' ').next().unwrap();
        match by_path.iter_mut().find(|(p, _)| p == path) {
            Some((_, s)) => s.push_str(stage),
            None => by_path.push((path.to_string(), stage.to_string())),
        }
    }
    by_path
}

fn check(f: ConflictFixture, want: &[(&str, &str)]) {
    let got = stages(&f);
    let got: Vec<(&str, &str)> = got.iter().map(|(p, s)| (p.as_str(), s.as_str())).collect();
    assert_eq!(got, want);
}

#[test]
fn every_fixture_stops_on_the_stages_it_promises() {
    check(ConflictFixture::text(), &[("notes.txt", "123")]);
    check(
        ConflictFixture::multi_conflict(),
        &[("a.txt", "123"), ("b.txt", "123"), ("docs/c.txt", "123")],
    );
    check(ConflictFixture::marker_lookalike(), &[("README.md", "123")]);
    check(ConflictFixture::add_add(), &[("new.txt", "23")]);
    check(ConflictFixture::delete_modify(true), &[("gone.txt", "13")]);
    check(ConflictFixture::delete_modify(false), &[("gone.txt", "12")]);
    check(ConflictFixture::binary(), &[("image.bin", "123")]);
    check(ConflictFixture::lfs_pointer(), &[("big.dat", "123")]);
    check(ConflictFixture::crlf(), &[("crlf.txt", "123")]);
    check(ConflictFixture::crlf_bom(), &[("bom.txt", "123")]);
    check(ConflictFixture::latin1(), &[("latin1.txt", "123")]);
    check(ConflictFixture::symlink(), &[("link", "123")]);
    check(ConflictFixture::mode_only(), &[("run.sh", "23")]);
    check(ConflictFixture::submodule(), &[("sub", "123")]);
    check(
        ConflictFixture::rename_rename(),
        &[("a.txt", "1"), ("b.txt", "2"), ("c.txt", "3")],
    );
    check(ConflictFixture::rename_delete(), &[("b.txt", "12")]);
    check(ConflictFixture::huge(), &[("huge.txt", "123")]);
    for op in [
        FixtureOperation::CherryPick,
        FixtureOperation::Revert,
        FixtureOperation::Rebase,
    ] {
        check(ConflictFixture::text_with(op), &[("notes.txt", "123")]);
    }
}

#[test]
fn the_operation_markers_are_in_the_git_dir() {
    let f = ConflictFixture::text();
    assert!(f.root().join(".git/MERGE_HEAD").exists());
    let f = ConflictFixture::text_with(FixtureOperation::CherryPick);
    assert!(f.root().join(".git/CHERRY_PICK_HEAD").exists());
    let f = ConflictFixture::text_with(FixtureOperation::Revert);
    assert!(f.root().join(".git/REVERT_HEAD").exists());
    let f = ConflictFixture::text_with(FixtureOperation::Rebase);
    assert!(f.root().join(".git/rebase-merge").exists());
}
