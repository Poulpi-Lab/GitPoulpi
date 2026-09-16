//! History rewrite, slice S18 (the history-rewrite design):
//! the size analysis. Expected sizes are exact because the fixture's blobs
//! have a known length.

use std::ops::ControlFlow;

use gitpoulpi_core::domain::rewrite::{
    glob_for, BlobState, Recipe, RewriteRefusal, Rule, RuleKind, Scope, SignaturePolicy,
    SizeAnalysis,
};
use gitpoulpi_core::{Error, GitBackend};

use super::rewrite::{git_text, refused, run, run_dirs};
use crate::{FixtureRepo, FixtureRepoBuilder};

/// `n` bytes that do not repeat, so every blob is its own.
fn bytes(seed: u8, n: usize) -> Vec<u8> {
    let mut state = u32::from(seed)
        .wrapping_mul(2_654_435_761)
        .wrapping_add(12345);
    (0..n)
        .map(|_| {
            state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            (state >> 24) as u8
        })
        .collect()
}

fn put(repo: &FixtureRepo, files: &[(&str, &[u8])], message: &str) -> String {
    for (path, content) in files {
        let full = repo.root().join(path);
        if let Some(dir) = full.parent() {
            std::fs::create_dir_all(dir).unwrap();
        }
        std::fs::write(full, content).unwrap();
    }
    repo.git(&["add", "-A"]);
    repo.git(&["commit", "-q", "-m", message]);
    git_text(repo, &["rev-parse", "HEAD"])
}

fn analyze(backend: &dyn GitBackend, repo: &FixtureRepo, scope: &Scope, top: u32) -> SizeAnalysis {
    backend
        .rewrite_analyze(&repo.repo_ref(), scope, top, &mut |_| {
            ControlFlow::Continue(())
        })
        .expect("analyze")
}

const DUMP: usize = 400_000;
const VIDEO_OLD: usize = 250_000;
const VIDEO_NEW: usize = 260_000;
const MORE: usize = 150_000;

/// `c1` adds a big file that `c3` deletes and a big live one, `c2` adds a
/// second file next to the live one and replaces its content, `c4` is small.
struct Fixture {
    repo: FixtureRepo,
    c: [String; 4],
}

fn fixture() -> Fixture {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .with_initial()
        .unwrap()
        .build();
    let c1 = put(
        &repo,
        &[
            ("old/dump.sql", &bytes(1, DUMP)),
            ("assets/video.bin", &bytes(2, VIDEO_OLD)),
            ("small.txt", b"small\n"),
        ],
        "add data",
    );
    let c2 = put(
        &repo,
        &[
            ("assets/more.bin", &bytes(3, MORE)),
            ("assets/video.bin", &bytes(4, VIDEO_NEW)),
        ],
        "more assets",
    );
    repo.git(&["rm", "-q", "old/dump.sql"]);
    repo.git(&["commit", "-q", "-m", "drop the dump"]);
    let c3 = git_text(&repo, &["rev-parse", "HEAD"]);
    let c4 = put(&repo, &[("README.md", b"hello\n")], "readme");
    Fixture {
        repo,
        c: [c1, c2, c3, c4],
    }
}

/// The largest blobs come out largest first with the right paths, first and
/// last commit and state; folders add unique blob bytes; the deleted file is
/// listed; the totals add up; the cap cuts and says so; a scope limits it;
/// canceling stops it; nothing is written.
pub fn check_rewrite_analyze_ranks_blobs_folders_and_deleted_files(backend: &dyn GitBackend) {
    let f = fixture();
    f.repo.git(&["branch", "topic", &f.c[0]]);
    f.repo.git(&["checkout", "-q", "topic"]);
    put(
        &f.repo,
        &[("topic/huge.bin", &bytes(9, 900_000))],
        "topic work",
    );
    f.repo.git(&["checkout", "-q", "main"]);

    let before = (git_text(&f.repo, &["for-each-ref"]), run_dirs(&f.repo));
    let a = analyze(backend, &f.repo, &Scope::default(), 50);
    assert_eq!(a.commits, 5, "the initial commit and four more");
    let order: Vec<(&str, u64)> = a.blobs.iter().map(|b| (b.path.as_str(), b.size)).collect();
    assert_eq!(
        &order[..4],
        &[
            ("old/dump.sql", DUMP as u64),
            ("assets/video.bin", VIDEO_NEW as u64),
            ("assets/video.bin", VIDEO_OLD as u64),
            ("assets/more.bin", MORE as u64),
        ],
        "{order:?}"
    );
    let [dump, video_new, video_old, more] = &a.blobs[..4] else {
        unreachable!()
    };
    assert_eq!(dump.state, BlobState::Deleted);
    assert_eq!(
        (dump.first_commit.to_hex(), dump.last_commit.to_hex()),
        (f.c[0].clone(), f.c[2].clone()),
        "added in c1, removed in c3"
    );
    assert_eq!(video_new.state, BlobState::InHead);
    assert_eq!(video_new.first_commit.to_hex(), f.c[1]);
    assert_eq!(video_new.last_commit.to_hex(), f.c[1]);
    assert_eq!(video_old.state, BlobState::Replaced);
    assert_eq!(video_old.first_commit.to_hex(), f.c[0]);
    assert_eq!(video_old.last_commit.to_hex(), f.c[1], "replaced in c2");
    assert_eq!(more.state, BlobState::InHead);
    assert!(a.blobs.iter().all(|b| b.disk_size > 0 && b.more_paths == 0));
    assert!(
        a.blobs.iter().all(|b| b.path != "topic/huge.bin"),
        "the topic branch is outside the scope"
    );

    // Folders: unique blob bytes below each, whether HEAD still has files in it.
    let folder = |name: &str| {
        a.folders
            .iter()
            .find(|f| f.path == name)
            .unwrap_or_else(|| panic!("no folder {name}: {:?}", a.folders))
    };
    assert_eq!(a.folders[0].path, "assets", "{:?}", a.folders);
    assert_eq!(folder("assets").size, (VIDEO_OLD + VIDEO_NEW + MORE) as u64);
    assert_eq!(folder("assets").blobs, 3);
    assert!(folder("assets").in_head);
    assert_eq!(folder("old").size, DUMP as u64);
    assert!(!folder("old").in_head);

    // Deleted from HEAD, still in the history.
    assert_eq!(a.deleted.len(), 1, "{:?}", a.deleted);
    let d = &a.deleted[0];
    assert_eq!(
        (d.path.as_str(), d.versions, d.size),
        ("old/dump.sql", 1, DUMP as u64)
    );
    assert_eq!(
        (d.first_commit.to_hex(), d.last_commit.to_hex()),
        (f.c[0].clone(), f.c[2].clone())
    );
    assert_eq!(a.deleted_paths, 1);

    // Totals add up, and the history-only part is what removal could free.
    assert_eq!(a.bytes_total, a.bytes_in_head + a.bytes_history_only);
    assert_eq!(a.bytes_history_only, (DUMP + VIDEO_OLD) as u64);
    assert!(a.bytes_in_head >= (VIDEO_NEW + MORE) as u64);
    assert!(a.disk_total > 0 && a.blobs_total >= 6);

    // The cap cuts every list and says how many it left out.
    let capped = analyze(backend, &f.repo, &Scope::default(), 2);
    assert_eq!(capped.blobs.len(), 2);
    assert_eq!(capped.blobs[0].size, DUMP as u64);
    assert_eq!(capped.blobs_cut, a.blobs_total - 2);
    assert_eq!(capped.folders.len(), 2);
    assert_eq!(capped.bytes_total, a.bytes_total, "totals ignore the cap");

    // A scan reads and writes nothing; the scope matters.
    assert_eq!(
        (git_text(&f.repo, &["for-each-ref"]), run_dirs(&f.repo)),
        before
    );
    let all = analyze(backend, &f.repo, &Scope::AllLocal, 50);
    assert_eq!(all.blobs[0].path, "topic/huge.bin");
    assert_eq!(all.blobs[0].size, 900_000);
    assert_eq!(all.commits, 6);

    // Canceling stops it with the typed error; an empty repository is refused.
    let result = backend.rewrite_analyze(&f.repo.repo_ref(), &Scope::default(), 50, &mut |_| {
        ControlFlow::Break(())
    });
    assert!(matches!(result, Err(Error::Canceled { .. })), "{result:?}");
    let empty = FixtureRepoBuilder::new().unwrap().build();
    refused(
        backend.rewrite_analyze(&empty.repo_ref(), &Scope::default(), 50, &mut |_| {
            ControlFlow::Continue(())
        }),
        RewriteRefusal::EmptyScope,
    );
}

/// A merge commit's own content is seen, an unusual file name is kept
/// whole, and the same content at two paths is one blob with `more_paths`.
pub fn check_rewrite_analyze_reads_merges_odd_names_and_shared_content(backend: &dyn GitBackend) {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .with_initial()
        .unwrap()
        .build();
    let shared = bytes(5, 120_000);
    put(
        &repo,
        &[("a.bin", &shared), ("dir/b.bin", &shared)],
        "twice the same",
    );
    repo.git(&["checkout", "-q", "-b", "side"]);
    put(&repo, &[("side.txt", b"side\n")], "side");
    repo.git(&["checkout", "-q", "main"]);
    put(&repo, &[("main.txt", b"main\n")], "main");
    repo.git(&["merge", "--no-ff", "-q", "-m", "merge", "side"]);
    let odd = "wei rd/na me [1] (é).bin";
    put(&repo, &[(odd, &bytes(6, 80_000))], "odd name");

    let a = analyze(backend, &repo, &Scope::default(), 50);
    assert_eq!(a.blobs[0].size, 120_000);
    assert_eq!(a.blobs[0].more_paths, 1, "one blob, two paths");
    assert_eq!(a.blobs[0].state, BlobState::InHead);
    assert_eq!(a.blobs[1].path, odd);
    assert_eq!(a.blobs[1].size, 80_000);
    assert_eq!(a.commits, 6);
    // Content that two paths share counts once in the folder and the totals.
    let dir = a.folders.iter().find(|f| f.path == "dir").unwrap();
    assert_eq!((dir.size, dir.blobs), (120_000, 1));
}

/// What the analysis shows turns into rules that work: removing the deleted
/// file's path drops it from every commit and the next analysis no longer
/// lists it; a size rule drops the other big blobs; undo is the ref moves.
pub fn check_rewrite_analysis_rows_turn_into_working_rules(backend: &dyn GitBackend) {
    let f = fixture();
    let a = analyze(backend, &f.repo, &Scope::default(), 50);
    let path = a.deleted[0].path.clone();
    let mut recipe = Recipe::new(
        "remove",
        Scope::default(),
        vec![Rule::new(RuleKind::RemovePaths {
            globs: vec![glob_for(&path)],
        })],
    );
    recipe.options.signatures = Some(SignaturePolicy::Strip);
    let old_tip = git_text(&f.repo, &["rev-parse", "main"]);
    run(backend, &f.repo, &recipe);
    let log = git_text(&f.repo, &["log", "--all", "--format=", "--name-only"]);
    assert!(!log.contains("old/dump.sql"), "{log}");
    let after = analyze(backend, &f.repo, &Scope::default(), 50);
    assert!(after.deleted.is_empty(), "{:?}", after.deleted);
    assert!(after.blobs.iter().all(|b| b.path != "old/dump.sql"));
    assert_eq!(
        after.bytes_history_only, VIDEO_OLD as u64,
        "only the old video version is left to free"
    );
    assert_ne!(git_text(&f.repo, &["rev-parse", "main"]), old_tip);

    // The size rule: everything over 200 kB goes, wherever it is.
    let mut by_size = Recipe::new(
        "big blobs",
        Scope::default(),
        vec![Rule::new(RuleKind::RemoveBlobsOver { bytes: 200_000 })],
    );
    by_size.options.signatures = Some(SignaturePolicy::Strip);
    run(backend, &f.repo, &by_size);
    let last = analyze(backend, &f.repo, &Scope::default(), 50);
    assert!(
        last.blobs.iter().all(|b| b.size <= 200_000),
        "{:?}",
        last.blobs
    );
    assert_eq!(last.blobs[0].size, MORE as u64);
}
