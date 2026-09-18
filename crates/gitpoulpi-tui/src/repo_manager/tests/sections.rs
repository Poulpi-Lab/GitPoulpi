use super::*;

#[test]
fn each_section_lists_its_repositories_in_its_own_order() {
    let m = manager();
    assert_eq!(names(&m, Section::Open), ["beta", "alpha"], "tab order");
    assert_eq!(
        names(&m, Section::Favorites),
        ["delta", "alpha"],
        "starring order"
    );
    assert_eq!(
        names(&m, Section::Recent),
        ["alpha", "gamma"],
        "newest first"
    );
    assert_eq!(
        names(&m, Section::All),
        ["alpha", "beta", "delta", "gamma"],
        "the union, by name"
    );
}

#[test]
fn the_search_filters_every_section_by_name_or_path() {
    let mut m = manager();
    handle_key(&mut m, Key::Char('/'), false);
    for c in "ALP".chars() {
        handle_key(&mut m, Key::Char(c), false);
    }
    assert_eq!(m.query, "ALP");
    for section in Section::ORDER {
        for name in names(&m, section) {
            assert_eq!(name, "alpha", "{section:?}");
        }
    }
    assert!(names(&m, Section::Favorites).contains(&"alpha".to_string()));
    // An empty section says so rather than vanishing.
    m.query = "zzz".into();
    assert!(m.rows().contains(&ManagerRow::Empty(Section::Open)));
    // Path matches too.
    m.query = "/work/del".into();
    assert_eq!(names(&m, Section::All), ["delta"]);
}

#[test]
fn collapsing_a_section_hides_its_rows_and_counts_survive() {
    let mut m = manager();
    let before = m.rows().len();
    m.toggle_section(Section::All);
    assert_eq!(m.rows().len(), before - 4);
    m.set_collapsed_all(true);
    assert_eq!(m.rows().len(), 4, "headers only");
    m.set_collapsed_all(false);
    assert_eq!(m.rows().len(), before);
}

#[test]
fn two_repositories_with_one_name_show_their_parent_folder() {
    let mut a = repo("app");
    a.path = PathBuf::from("/work/ptm/utilitaires/app");
    let mut b = repo("app");
    b.path = PathBuf::from("/work/scratch/app");
    assert_eq!(a.column_text(false), "utilitaires");
    assert!(a.column_text(true).contains("ptm/utilitaires"));
    assert!(b.column_text(true).contains("scratch"));
    let dup = duplicate_names(&[a, b, repo("solo")]);
    assert!(dup.contains("app") && !dup.contains("solo"));
}

/// `set_repos` keeps what the probe learned and keeps the cursor on
/// the same repository, so starring one does not blank the row or
/// jump the selection.
#[test]
fn replacing_the_repositories_keeps_facts_and_the_selection() {
    let mut m = manager();
    m.learn(
        Path::new("/work/delta"),
        RepoFacts {
            branch: Some("trunk".into()),
            ..RepoFacts::default()
        },
    );
    let delta_row = m
        .rows()
        .iter()
        .position(|r| matches!(r, ManagerRow::Repo { repo, .. } if m.repos[*repo].name == "delta"))
        .unwrap();
    m.selected = delta_row;
    let mut repos = m.repos.clone();
    for r in &mut repos {
        r.facts = None;
    }
    repos[0].favorite = None; // alpha un-starred: rows shift
    m.set_repos(repos);
    assert_eq!(m.selected_repo().unwrap().name, "delta");
    assert_eq!(
        m.selected_repo()
            .unwrap()
            .facts
            .as_ref()
            .unwrap()
            .branch
            .as_deref(),
        Some("trunk")
    );
}
