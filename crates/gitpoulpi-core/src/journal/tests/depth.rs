//! The default depth is 100 (`general.undo_depth`): a full journal must stay
//! cheap to load, save, plan on and list. Entry bodies are a few hundred
//! bytes; snapshot blobs are git objects, not JSON.

use super::*;

#[test]
fn a_full_default_depth_journal_is_cheap_to_load_save_and_list() {
    assert_eq!(DEFAULT_MAX_ENTRIES, 100);
    let mut j = Journal::default();
    for n in 0..150u64 {
        let id = j.begin(
            OperationKind::Commit,
            format!("Commit \"change {n}\" (a81b9e4)"),
            n as i64,
            Some(main()),
            true,
            vec![("refs/heads/main".into(), Some(oid((n % 200) as u8)))],
        );
        j.entry_mut(id).unwrap().pins = (0..3)
            .map(|k| Pin {
                name: pin_name("", id, k),
                oid: oid(k as u8),
            })
            .collect();
        j.entry_mut(id).unwrap().files = Some(PathChange {
            paths: vec!["a.txt".into(), "b.txt".into()],
            before: Snapshot {
                tree: oid(50),
                hashes: vec![
                    file("a.txt", Some(2), Some(1)),
                    file("b.txt", Some(3), None),
                ],
            },
            after: Snapshot {
                tree: oid(60),
                hashes: vec![file("a.txt", Some(1), Some(1))],
            },
        });
        j.finish(
            id,
            Some(main()),
            vec![RefChange {
                name: "refs/heads/main".into(),
                old: Some(oid((n % 200) as u8)),
                new: Some(oid(((n + 1) % 200) as u8)),
            }],
            vec![],
            DEFAULT_MAX_ENTRIES,
        );
    }
    assert_eq!(j.undo.len(), 100, "the bound holds");
    let start = std::time::Instant::now();
    for _ in 0..20 {
        let text = serde_json::to_string(&j).unwrap();
        assert!(text.len() < 400_000, "journal.json is {} bytes", text.len());
        let back: Journal = serde_json::from_str(&text).unwrap();
        assert_eq!(back.undo.len(), 100);
        let listed: Vec<_> = back.undo.iter().map(JournalEntry::listed).collect();
        assert_eq!(listed.len(), 100);
        let pins: usize = back.undo.iter().map(|e| e.pins.len()).sum();
        assert_eq!(pins, 300);
    }
    // 20 round trips of 100 entries: far below a frame budget each.
    assert!(
        start.elapsed() < std::time::Duration::from_secs(5),
        "took {:?}",
        start.elapsed()
    );
}
