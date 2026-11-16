use super::*;

fn entry(path: &str, index: Option<FileStatus>, worktree: Option<FileStatus>) -> StatusEntry {
    StatusEntry {
        renamed_from: None,
        path: PathBuf::from(path),
        index,
        worktree,
    }
}

/// The WIP list is HEAD's tree plus the status's paths.
#[test]
fn the_wip_list_is_the_head_tree_plus_new_files_changes_marked() {
    let status = WorkingTree {
        files: vec![
            entry("new.txt", None, Some(FileStatus::Untracked)),
            entry("src/a.rs", Some(FileStatus::Modified), None),
        ],
        ..WorkingTree::default()
    };
    let all = wip_all_paths(&paths(&["README.md", "src/a.rs"]), &status);
    assert_eq!(all, paths(&["README.md", "new.txt", "src/a.rs"]));
    assert_eq!(wip_status(wip_entry(&status, Path::new("README.md"))), None);
    assert_eq!(
        wip_status(wip_entry(&status, Path::new("src/a.rs"))),
        Some(FileStatus::Modified)
    );
    assert_eq!(
        wip_file_open(&status, Path::new("README.md")),
        WipFileOpen::Content {
            path: PathBuf::from("README.md")
        }
    );
    assert_eq!(
        wip_file_open(&status, Path::new("new.txt")),
        WipFileOpen::Diff {
            source: DiffSource::Unstaged,
            path: PathBuf::from("new.txt")
        }
    );
    assert_eq!(
        wip_file_open(&status, Path::new("src/a.rs")),
        WipFileOpen::Diff {
            source: DiffSource::Staged,
            path: PathBuf::from("src/a.rs")
        }
    );
}

/// The WIP list shows every file, the changed ones marked.
#[test]
fn the_wip_list_draws_every_file_with_changes_marked() {
    use ratatui::backend::TestBackend;
    let status = WorkingTree {
        files: vec![entry("src/a.rs", None, Some(FileStatus::Modified))],
        ..WorkingTree::default()
    };
    let mut wip = WipAllFiles {
        head_tree: Some((oid(1), paths(&["README.md", "src/a.rs"]))),
        ..WipAllFiles::default()
    };
    let mut terminal = Terminal::new(TestBackend::new(40, 6)).unwrap();
    terminal
        .draw(|frame| {
            wip.render(
                frame,
                Rect::new(0, 0, 40, 6),
                true,
                &status,
                FileListMode::Path,
                &HashSet::new(),
                &FileSelection::default(),
            );
        })
        .unwrap();
    let lines = buffer_text(terminal.backend().buffer());
    assert!(lines[0].contains("All files (2)"), "{lines:#?}");
    assert!(
        lines[1].contains("  README.md"),
        "unchanged, no marker: {lines:#?}"
    );
    assert!(
        lines[2].contains(&format!("{} src/a.rs", status_marker(FileStatus::Modified))),
        "{lines:#?}"
    );
    assert_eq!(
        wip.row_at(1),
        Some(0),
        "row_to_index from the bordered rect"
    );
    assert_eq!(wip.row_at(2), Some(1));
    assert_eq!(wip.row_at(0), None, "the title row is not a file");
    assert_eq!(
        wip.activate(0, &status),
        Some(WipActivation::Open(WipFileOpen::Content {
            path: PathBuf::from("README.md")
        }))
    );
}
