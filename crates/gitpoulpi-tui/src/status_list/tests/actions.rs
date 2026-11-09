use super::*;

/// A folder's `Discard` builds the confirm modal
/// over exactly its own Unstaged paths, worded with the untracked
/// count `discard_confirmation_prompt` computes.
#[test]
fn folder_discard_opens_a_confirm_modal_naming_its_own_paths() {
    use gitpoulpi_core::domain::FileStatus;
    let tree = WorkingTree {
        files: vec![
            entry("src/a.rs", Some(FileStatus::Modified)),
            entry("src/b.rs", Some(FileStatus::Untracked)),
            entry("other/c.rs", Some(FileStatus::Modified)),
        ],
        ..WorkingTree::default()
    };
    let mut input_mode = InputMode::Normal;
    let mut last_message = None;
    apply_folder_context_action(
        FolderContextAction::Discard,
        Path::new("src"),
        &tree,
        &mut input_mode,
        &mut last_message,
    );
    match input_mode {
        InputMode::Confirm { prompt, action } => {
            assert_eq!(
                prompt,
                "Discard 2 files (1 untracked)? This cannot be undone."
            );
            match action {
                ConfirmAction::Destructive(PendingDestructive::DiscardPaths(mut paths)) => {
                    paths.sort();
                    assert_eq!(
                        paths,
                        vec![PathBuf::from("src/a.rs"), PathBuf::from("src/b.rs")]
                    );
                }
                _ => panic!("expected DiscardPaths, got a different destructive action"),
            }
        }
        _ => panic!("expected a confirm modal to stay open"),
    }
}

/// A folder with nothing Unstaged under it (everything inside it is
/// clean, or only staged) explains rather than opening an empty
/// confirm modal.
#[test]
fn discarding_an_empty_folder_says_so_instead_of_confirming_nothing() {
    let tree = WorkingTree {
        files: Vec::new(),
        ..WorkingTree::default()
    };
    let mut input_mode = InputMode::Normal;
    let mut last_message = None;
    apply_folder_context_action(
        FolderContextAction::Discard,
        Path::new("src"),
        &tree,
        &mut input_mode,
        &mut last_message,
    );
    assert!(matches!(input_mode, InputMode::Normal));
    assert!(last_message.unwrap().contains("nothing to discard"));
}

/// The global button/palette/key path (`discard_all_changes`) is the
/// same confirm modal over every Unstaged path in the whole tree, and
/// explains itself the same two ways `apply_folder_context_action`
/// does: no tree loaded yet, or nothing dirty.
#[test]
fn discard_all_changes_covers_every_unstaged_path_or_explains_why_not() {
    use gitpoulpi_core::domain::FileStatus;
    let tree = WorkingTree {
        files: vec![
            entry("a.rs", Some(FileStatus::Modified)),
            entry("b.rs", Some(FileStatus::Untracked)),
        ],
        ..WorkingTree::default()
    };
    let mut input_mode = InputMode::Normal;
    let mut last_message = None;
    discard_all_changes(Some(&tree), &mut input_mode, &mut last_message);
    assert!(matches!(
        input_mode,
        InputMode::Confirm {
            action: ConfirmAction::Destructive(PendingDestructive::DiscardPaths(_)),
            ..
        }
    ));

    let mut input_mode = InputMode::Normal;
    let mut last_message = None;
    discard_all_changes(None, &mut input_mode, &mut last_message);
    assert!(matches!(input_mode, InputMode::Normal));
    assert!(last_message.unwrap().contains("no working tree"));

    let empty = WorkingTree {
        files: Vec::new(),
        ..WorkingTree::default()
    };
    let mut input_mode = InputMode::Normal;
    let mut last_message = None;
    discard_all_changes(Some(&empty), &mut input_mode, &mut last_message);
    assert!(matches!(input_mode, InputMode::Normal));
    assert!(last_message.unwrap().contains("nothing to discard"));
}

/// The file menu opened before git's binary answer: the Treat row is a
/// placeholder nothing can activate; once the answer lands the same open
/// menu is rebuilt in place and offers what git says (Text for a `*.dat
/// binary` file whose bytes look like text), not what the sniff says.
#[test]
fn a_file_menu_opened_before_gits_answer_corrects_itself_in_place() {
    use gitpoulpi_core::domain::FileStatus;
    use ratatui::{backend::TestBackend, Terminal};
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("notes.dat"), "plain\n").unwrap();
    let rel = Path::new("notes.dat");
    let tree = Ok(WorkingTree {
        files: vec![entry("notes.dat", Some(FileStatus::Modified))],
        ..WorkingTree::default()
    });
    let lfs = crate::lfs_ui::LfsUi::default();
    let mut menu = open_file_menu(
        StatusSection::Unstaged,
        rel,
        &tree,
        &[],
        &lfs,
        dir.path(),
        2,
        2,
    );
    let text = |menu: &ContextMenu<FileContextAction>| {
        let mut terminal = Terminal::new(TestBackend::new(70, 40)).unwrap();
        terminal.draw(|f| drop(menu.render(f, None))).unwrap();
        terminal.backend().to_string()
    };
    let pending = text(&menu);
    assert!(pending.contains("Treat as binary or text (checking...)"));
    assert!(!pending.contains("Treat as binary >") && !pending.contains("Treat as text"));
    let actions: Vec<_> = (0..menu.row_count())
        .filter_map(|r| menu.row_action(r).copied())
        .collect();
    assert!(
        !actions.contains(&FileContextAction::TreatFileBinary),
        "nothing to activate on the sniff"
    );

    crate::binary_hint::record(dir.path(), rel, true);
    refresh_file_menu(
        &mut menu,
        StatusSection::Unstaged,
        rel,
        &tree,
        &[],
        &lfs,
        dir.path(),
    );
    let after = text(&menu);
    assert!(after.contains("Treat as text"), "{after}");
    assert!(!after.contains("(checking...)"));
}
