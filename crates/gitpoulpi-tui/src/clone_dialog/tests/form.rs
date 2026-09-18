use super::*;

#[test]
fn the_folder_name_comes_from_the_url_whatever_its_shape() {
    for (url, name) in [
        ("https://github.com/owner/project.git", "project"),
        ("https://github.com/owner/project/", "project"),
        ("git@github.com:owner/project.git", "project"),
        ("ssh://git@host:22/group/sub/tool", "tool"),
        ("C:\\Projets\\fixture", "fixture"),
        ("/srv/git/thing.git", "thing"),
        ("", "repo"),
    ] {
        assert_eq!(repo_name_from_url(url), name, "{url}");
    }
}

#[test]
fn the_destination_is_where_plus_the_urls_name_and_shallow_is_depth_one() {
    let mut dialog = CloneDialog::new(Some(&abs("clones")));
    assert_eq!(dialog.focus, CloneFocus::Url);
    typed(&mut dialog, "https://example.com/team/app.git");
    assert_eq!(dialog.dest(), Some(abs("clones").join("app")));
    dialog.focus = CloneFocus::Shallow;
    dialog.on_key(key(KeyCode::Char(' ')));
    assert!(dialog.shallow);
    dialog.focus = CloneFocus::Clone;
    assert_eq!(dialog.on_key(key(KeyCode::Enter)), CloneOutcome::Stay);
    assert_eq!(
        settle(&mut dialog),
        CloneOutcome::Submit {
            url: "https://example.com/team/app.git".into(),
            dest: abs("clones").join("app"),
            opts: CloneOptions {
                shallow_depth: Some(1),
                ..CloneOptions::default()
            },
        }
    );
}

#[test]
fn an_incomplete_form_says_why_and_clones_nothing() {
    let mut dialog = CloneDialog::new(Some(&abs("clones")));
    assert!(dialog.plan().unwrap_err().contains("URL"));
    assert_eq!(dialog.on_key(key(KeyCode::Enter)), CloneOutcome::Stay);
    typed(&mut dialog, "has space");
    assert!(dialog.plan().unwrap_err().contains("no spaces"));
    dialog.url = "https://x/y".into();
    dialog.where_to.clear();
    assert!(dialog.plan().unwrap_err().contains("where to clone"));
    dialog.where_to = "relative".into();
    assert!(dialog.plan().unwrap_err().contains("full path"));
    let mut buf = Buffer::empty(Rect::new(0, 0, 100, 30));
    dialog.render(clone_rect(buf.area), &mut buf);
    assert!(dialog_kit::buffer_text(&buf).contains("full path"));
}

#[test]
fn the_folder_name_follows_the_url_until_the_user_edits_it() {
    let mut dialog = CloneDialog::new(Some(&abs("clones")));
    typed(&mut dialog, "https://example.com/team/app.git");
    assert_eq!(dialog.name, "app");
    // The URL changes: the untouched name follows.
    dialog.url.clear();
    dialog.edit.reset();
    typed(&mut dialog, "https://example.com/team/other.git");
    assert_eq!(dialog.name, "other");
    // Editing the name pins it.
    dialog.focus = CloneFocus::Name;
    typed(&mut dialog, "-x");
    assert_eq!(dialog.name, "other-x");
    dialog.focus = CloneFocus::Url;
    typed(&mut dialog, "z");
    assert_eq!(dialog.name, "other-x", "an edited name stays");
    assert_eq!(dialog.dest(), Some(abs("clones").join("other-x")));
    assert_eq!(dialog.plan().unwrap().1, abs("clones").join("other-x"));
}

#[test]
fn the_folder_name_is_validated() {
    for bad in ["", "a/b", r"a\b", "a:b", "x*", "..", ".", "name.", "a|b"] {
        assert!(validate_folder_name(bad).is_err(), "{bad:?}");
    }
    for good in ["repo", "my repo", "a.b", ".hidden", "app-2_x"] {
        assert!(validate_folder_name(good).is_ok(), "{good:?}");
    }
    let mut dialog = CloneDialog::new(Some(&abs("clones")));
    typed(&mut dialog, "https://example.com/team/app.git");
    dialog.name = "a/b".into();
    assert!(dialog.plan().unwrap_err().contains("folder name"));
    dialog.name.clear();
    assert!(dialog.plan().unwrap_err().contains("folder name"));
}

#[test]
fn a_non_empty_target_folder_is_warned_about() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::create_dir(tmp.path().join("app")).unwrap();
    std::fs::write(tmp.path().join("app").join("f.txt"), "x").unwrap();
    let mut dialog = CloneDialog::new(Some(tmp.path()));
    typed(&mut dialog, "https://example.com/team/app.git");
    settle(&mut dialog);
    let (text, _) = dialog.dest_warning().unwrap();
    assert!(text.contains("not empty"));
    let (text, _) = paint(&mut dialog, 100, 30);
    assert!(text.contains("not empty"), "{text}");
    assert!(text.contains("Folder name"), "{text}");
    // A folder that is absent or empty is no warning.
    std::fs::remove_file(tmp.path().join("app").join("f.txt")).unwrap();
    dialog.name = "other".into();
    settle(&mut dialog);
    assert!(dialog.dest_warning().is_none());
}

#[test]
fn sparse_checkout_needs_directories_and_sends_them_as_clone_options() {
    let mut dialog = CloneDialog::new(Some(&abs("clones")));
    typed(&mut dialog, "https://example.com/team/app.git");
    assert_eq!(dialog.plan().unwrap().2.sparse, None);

    // Checking the box puts the keyboard in the directories field.
    dialog.focus = CloneFocus::Sparse;
    dialog.on_key(key(KeyCode::Char(' ')));
    assert!(dialog.sparse);
    assert_eq!(dialog.focus, CloneFocus::SparseDirs);
    assert!(
        dialog.plan().unwrap_err().contains("directory"),
        "{:?}",
        dialog.plan()
    );
    assert_eq!(dialog.on_key(key(KeyCode::Enter)), CloneOutcome::Stay);
    assert!(dialog.message.is_some());

    typed(&mut dialog, "src, docs/guide");
    let (_, _, opts) = dialog.plan().unwrap();
    assert_eq!(
        opts.sparse,
        Some(vec!["src".to_string(), "docs/guide".to_string()])
    );

    // A directory git could not use is refused with its name.
    dialog.sparse_dirs = "src ../x".into();
    assert!(dialog.plan().unwrap_err().contains("../x"));

    // Unchecked again: the typed directories are kept but not sent.
    dialog.sparse_dirs = "src".into();
    dialog.focus = CloneFocus::Sparse;
    dialog.on_key(key(KeyCode::Char(' ')));
    assert!(!dialog.sparse);
    assert_eq!(dialog.plan().unwrap().2.sparse, None);
}

#[test]
fn the_directories_field_is_in_the_tab_order_only_while_checked() {
    let mut dialog = CloneDialog::new(Some(&abs("clones")));
    assert!(!dialog.order().contains(&CloneFocus::SparseDirs));
    dialog.sparse = true;
    let order = dialog.order();
    let at = order.iter().position(|f| *f == CloneFocus::Sparse).unwrap();
    assert_eq!(order[at + 1], CloneFocus::SparseDirs);
}

#[test]
fn the_directories_field_is_drawn_and_clickable_when_checked() {
    let mut dialog = CloneDialog::new(Some(&abs("clones")));
    let (text, _) = paint(&mut dialog, 120, 30);
    assert!(!text.contains("directories to check out"), "{text}");
    dialog.sparse = true;
    let (text, rect) = paint(&mut dialog, 120, 30);
    assert!(text.contains("directories to check out"), "{text}");
    let field = dialog.hit_rect(CloneHit::Part(CloneFocus::SparseDirs));
    assert!(rect.contains((field.x, field.y).into()));
    dialog.on_mouse(click(field.x + 1, field.y));
    assert_eq!(dialog.focus, CloneFocus::SparseDirs);
    typed(&mut dialog, "lib");
    assert_eq!(dialog.sparse_dirs, "lib");
}
