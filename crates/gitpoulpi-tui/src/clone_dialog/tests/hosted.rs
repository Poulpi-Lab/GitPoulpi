use super::*;
use crate::clone_dialog::groups;

#[test]
fn without_a_connected_profile_only_the_url_source_is_offered_and_it_says_how_to_add_one() {
    let mut dialog = CloneDialog::new(Some(&abs("clones")));
    let (text, _) = paint(&mut dialog, 120, 30);
    assert!(text.contains("Clone with URL"), "{text}");
    assert!(text.contains("connect a hosting"), "{text}");
    assert!(
        !text.contains("GitHub.com"),
        "no grayed unsupported sources"
    );
    dialog.focus = CloneFocus::Sources;
    assert_eq!(dialog.on_key(key(KeyCode::Down)), CloneOutcome::Stay);
    assert_eq!(dialog.source, 0);
}

#[test]
fn only_connected_supported_profiles_become_sources() {
    use gitpoulpi_config::{ConnectedUser, Integration, IntegrationKind};
    let mut connected = Integration::new(
        "corp",
        IntegrationKind::GitlabSelfManaged,
        "https://g.example.com",
    );
    connected.user = Some(ConnectedUser {
        login: "kai".into(),
        name: String::new(),
        id: String::new(),
    });
    let mut disabled = Integration::new("off", IntegrationKind::Github, "https://github.com");
    disabled.enabled = false;
    disabled.user = connected.user.clone();
    let never = Integration::new("nope", IntegrationKind::Github, "https://github.com");
    let mut jira = Integration::new(
        "jira",
        IntegrationKind::JiraCloud,
        "https://x.atlassian.net",
    );
    jira.user = connected.user.clone();
    let sources = hosted_sources(&[connected, disabled, never, jira]);
    assert_eq!(sources.len(), 1);
    assert_eq!(sources[0].profile_id, "corp");
    assert_eq!(sources[0].label, "GitLab \u{00B7} g.example.com");
}

#[test]
fn picking_a_hosted_source_asks_for_its_repositories_once_then_lists_and_searches_them() {
    let mut dialog = with_hosted();
    dialog.focus = CloneFocus::Sources;
    // The highlight moves at once, the list is asked for once it rests.
    assert_eq!(dialog.on_key(key(KeyCode::Down)), CloneOutcome::Stay);
    assert_eq!(dialog.source, 1);
    assert_eq!(dialog.poll(), CloneOutcome::Stay, "still within the rest");
    rest(&mut dialog);
    assert_eq!(dialog.poll(), CloneOutcome::LoadProjects("corp".into()));
    assert_eq!(dialog.poll(), CloneOutcome::Stay, "asked once");
    let (text, _) = paint(&mut dialog, 130, 34);
    assert!(text.contains("loading repositories"), "{text}");
    assert!(text.contains("GitLab \u{00B7} corp") && text.contains("GitHub \u{00B7} github.com"));

    dialog.set_projects(
        "corp",
        Ok(vec![
            project("team/app"),
            project("team/tool"),
            project("me/notes"),
        ]),
    );
    let (text, _) = paint(&mut dialog, 130, 34);
    for want in [
        "team/app",
        "team/tool",
        "me/notes",
        "about team/app",
        "Repository",
        "search repositories",
    ] {
        assert!(text.contains(want), "{want}\n{text}");
    }
    // Coming back to a loaded source does not ask again.
    dialog.on_key(key(KeyCode::Up));
    assert_eq!(dialog.source, 0);
    dialog.focus = CloneFocus::Sources;
    assert_eq!(dialog.on_key(key(KeyCode::Down)), CloneOutcome::Stay);

    // The search narrows the list as you type.
    dialog.focus = CloneFocus::Search;
    typed(&mut dialog, "tool");
    assert_eq!(dialog.filtered().len(), 1);
    let (text, _) = paint(&mut dialog, 130, 34);
    assert!(
        text.contains("team/tool") && !text.contains("team/app"),
        "{text}"
    );
    typed(&mut dialog, "zzz");
    let (text, _) = paint(&mut dialog, 130, 34);
    assert!(text.contains("no repository matches"), "{text}");
}

/// j/k/g/G walk the sources and the repositories like Down/Up/Home/End;
/// in the search field (and the URL field) they are typed letters.
#[test]
fn j_and_k_move_the_sources_and_the_repositories_but_type_in_the_fields() {
    let mut dialog = with_hosted();
    dialog.focus = CloneFocus::Sources;
    dialog.on_key(key(KeyCode::Char('j')));
    assert_eq!(dialog.source, 1);
    dialog.on_key(key(KeyCode::Char('j')));
    dialog.on_key(key(KeyCode::Char('k')));
    assert_eq!(dialog.source, 1);
    dialog.on_key(key(KeyCode::Char('g')));
    assert_eq!(dialog.source, 0);
    dialog.on_key(key(KeyCode::Char('G')));
    assert_eq!(dialog.source, dialog.sources_len() - 1);

    dialog.select_source(1);
    dialog.set_projects(
        "corp",
        Ok(vec![
            project("team/app"),
            project("team/tool"),
            project("me/n"),
        ]),
    );
    dialog.focus = CloneFocus::Repos;
    dialog.on_key(key(KeyCode::Char('j')));
    dialog.on_key(key(KeyCode::Char('j')));
    dialog.on_key(key(KeyCode::Char('k')));
    assert_eq!(dialog.repo_cursor, 1);
    dialog.on_key(key(KeyCode::Char('G')));
    assert_eq!(dialog.repo_cursor, 2);
    dialog.on_key(key(KeyCode::Char('g')));
    assert_eq!(dialog.repo_cursor, 0);

    dialog.focus = CloneFocus::Search;
    typed(&mut dialog, "jkgG");
    assert_eq!(dialog.search, "jkgG");
    dialog.select_source(0);
    dialog.focus = CloneFocus::Url;
    typed(&mut dialog, "jk");
    assert_eq!(dialog.url, "jk");
}

#[test]
fn choosing_a_repository_fills_the_url_and_clone_uses_it() {
    let mut dialog = with_hosted();
    dialog.source = 1;
    dialog.set_projects("corp", Ok(vec![project("team/app"), project("team/tool")]));
    dialog.focus = CloneFocus::Search;
    assert!(dialog.plan().unwrap_err().contains("choose a repository"));
    // Down moves through the list from the search box; Enter chooses.
    dialog.on_key(key(KeyCode::Down));
    dialog.on_key(key(KeyCode::Enter));
    assert_eq!(dialog.focus, CloneFocus::Clone);
    assert_eq!(dialog.chosen.as_deref(), Some("team/tool"));
    let (text, _) = paint(&mut dialog, 130, 34);
    assert!(text.contains("\u{2713} team/tool"), "{text}");
    assert!(
        text.contains("Full path:") && text.contains("tool"),
        "{text}"
    );
    assert_eq!(dialog.on_key(key(KeyCode::Enter)), CloneOutcome::Stay);
    assert_eq!(
        settle(&mut dialog),
        CloneOutcome::Submit {
            url: "https://gitlab.example.com/team/tool.git".into(),
            dest: abs("clones").join("tool"),
            opts: CloneOptions::default(),
        }
    );
    // Another source forgets the choice.
    dialog.focus = CloneFocus::Sources;
    dialog.on_key(key(KeyCode::Up));
    assert_eq!(dialog.source, 0);
    assert_eq!(dialog.chosen, None);
    assert!(dialog.plan().unwrap_err().contains("URL"));
}

#[test]
fn the_mouse_picks_sources_and_repositories_and_the_wheel_scrolls() {
    let mut dialog = with_hosted();
    let (_, rect) = paint(&mut dialog, 130, 34);
    let second = dialog.hit_rect(CloneHit::Source(1));
    assert_eq!(
        dialog.on_mouse(click(second.x + 2, second.y)),
        CloneOutcome::LoadProjects("corp".into())
    );
    let many: Vec<HostedProject> = (0..20).map(|i| project(&format!("team/r{i:02}"))).collect();
    dialog.set_projects("corp", Ok(many));
    let (text, _) = paint(&mut dialog, 130, 34);
    assert!(
        text.contains("team/r00") && !text.contains("team/r19"),
        "{text}"
    );
    // The wheel walks the list, scrolling it past the bottom row.
    for _ in 0..10 {
        dialog.on_mouse(MouseEvent {
            kind: MouseEventKind::ScrollDown,
            column: rect.x + 60,
            row: rect.y + 12,
            modifiers: KeyModifiers::NONE,
        });
    }
    let (text, _) = paint(&mut dialog, 130, 34);
    assert!(
        text.contains("team/r10") && !text.contains("team/r00"),
        "{text}"
    );
    // A click on a row chooses it.
    let row = dialog.hit_rect(CloneHit::Repo(10));
    dialog.on_mouse(click(row.x + 3, row.y));
    assert_eq!(dialog.chosen.as_deref(), Some("team/r10"));
    assert!(dialog.plan().is_ok());
    // Every hit stays inside the dialog.
    let (_, rect) = paint(&mut dialog, 130, 34);
    for (r, _) in &dialog.hits {
        assert!(rect.contains((r.x, r.y).into()), "{r:?}");
    }
}

#[test]
fn tab_reaches_the_search_and_the_list_in_hosted_mode_only() {
    let mut dialog = with_hosted();
    dialog.source = 1;
    dialog.focus = CloneFocus::Where;
    let mut seen = Vec::new();
    for _ in 0..9 {
        dialog.on_key(key(KeyCode::Tab));
        seen.push(dialog.focus);
    }
    assert!(seen.contains(&CloneFocus::Search) && seen.contains(&CloneFocus::Repos));
    assert!(!seen.contains(&CloneFocus::Url));
    let mut url_mode = with_hosted();
    url_mode.focus = CloneFocus::Where;
    let mut seen = Vec::new();
    for _ in 0..9 {
        url_mode.on_key(key(KeyCode::Tab));
        seen.push(url_mode.focus);
    }
    assert!(seen.contains(&CloneFocus::Url) && !seen.contains(&CloneFocus::Search));
}

#[test]
fn a_hosted_choice_prefills_the_name_and_a_new_source_resets_it() {
    let mut dialog = with_hosted();
    dialog.source = 1;
    dialog.set_projects("corp", Ok(vec![project("team/app"), project("team/tool")]));
    dialog.choose_repo(1);
    assert_eq!(dialog.name, "tool");
    dialog.choose_repo(0);
    assert_eq!(dialog.name, "app");
    dialog.focus = CloneFocus::Name;
    typed(&mut dialog, "2");
    dialog.choose_repo(1);
    assert_eq!(dialog.name, "app2", "edited name survives a new choice");
    assert_eq!(dialog.dest(), Some(abs("clones").join("app2")));
    dialog.pick_source(0);
    assert_eq!(dialog.name, "", "a new source starts over");
}

#[test]
fn azure_devops_and_bitbucket_profiles_are_clone_sources_with_their_own_names() {
    use gitpoulpi_config::{ConnectedUser, Integration, IntegrationKind};
    let user = Some(ConnectedUser {
        login: "kai".into(),
        name: String::new(),
        id: String::new(),
    });
    let mut azure = Integration::new(
        "az",
        IntegrationKind::AzureDevops,
        "https://dev.azure.com/acme",
    );
    azure.user = user.clone();
    azure.name = "acme".into();
    let mut bitbucket = Integration::new("bb", IntegrationKind::Bitbucket, "https://bitbucket.org");
    bitbucket.user = user;
    bitbucket.name = "team".into();
    let labels: Vec<String> = hosted_sources(&[azure, bitbucket])
        .into_iter()
        .map(|s| s.label)
        .collect();
    assert_eq!(
        labels,
        vec![
            "Azure DevOps \u{00B7} acme".to_string(),
            "Bitbucket \u{00B7} team".to_string()
        ]
    );
}

fn private_project(full_name: &str) -> HostedProject {
    HostedProject {
        private: true,
        ..project(full_name)
    }
}

/// A GitLab source (login `kai`) loaded with repositories of three owners
/// plus the user's own.
fn grouped_dialog() -> CloneDialog {
    let mut dialog = with_hosted();
    dialog.source = 1;
    dialog.set_projects(
        "corp",
        Ok(vec![
            project("zoo/x"),
            project("acme/zed"),
            private_project("kai/secret"),
            project("Beta/y"),
            project("acme/alpha"),
            project("kai/Mine"),
        ]),
    );
    dialog
}

fn names(dialog: &CloneDialog) -> Vec<String> {
    dialog
        .filtered()
        .iter()
        .map(|p| p.full_name.clone())
        .collect()
}

#[test]
fn the_list_is_one_section_per_owner_own_first_then_alphabetical_case_insensitive() {
    let dialog = grouped_dialog();
    assert_eq!(
        names(&dialog),
        [
            "kai/Mine",
            "kai/secret",
            "acme/alpha",
            "acme/zed",
            "Beta/y",
            "zoo/x"
        ]
    );
    let rows = groups::rows(&dialog.grouped());
    let headers: Vec<String> = rows
        .iter()
        .filter_map(|r| match r {
            groups::ListRow::Header { owner, count } => Some(format!("{owner}:{count}")),
            groups::ListRow::Repo(_) => None,
        })
        .collect();
    assert_eq!(headers, ["kai:2", "acme:2", "Beta:1", "zoo:1"]);
    assert_eq!(rows.len(), 6 + 4);
}

#[test]
fn without_a_known_login_no_section_is_special() {
    let mut dialog = grouped_dialog();
    dialog.source = 2;
    dialog.set_projects("hub", Ok(vec![project("kai/a"), project("acme/b")]));
    assert_eq!(names(&dialog), ["acme/b", "kai/a"]);
}

#[test]
fn the_search_filters_across_sections_and_an_empty_section_loses_its_header() {
    let mut dialog = grouped_dialog();
    let (text, _) = paint(&mut dialog, 130, 34);
    for header in ["kai (2)", "acme (2)"] {
        assert!(text.contains(header), "{header}\n{text}");
    }
    dialog.focus = CloneFocus::Search;
    typed(&mut dialog, "alpha");
    assert_eq!(names(&dialog), ["acme/alpha"]);
    let (text, _) = paint(&mut dialog, 130, 34);
    assert!(text.contains("acme (1)"), "{text}");
    for gone in ["kai (", "Beta (", "zoo ("] {
        assert!(!text.contains(gone), "{gone} header must disappear\n{text}");
    }
}

#[test]
fn up_and_down_walk_the_repositories_and_never_stop_on_a_header() {
    let mut dialog = grouped_dialog();
    dialog.focus = CloneFocus::Repos;
    let mut seen = vec![dialog.repo_cursor];
    for _ in 0..5 {
        dialog.on_key(key(KeyCode::Down));
        seen.push(dialog.repo_cursor);
    }
    assert_eq!(seen, [0, 1, 2, 3, 4, 5]);
    dialog.on_key(key(KeyCode::Down));
    assert_eq!(dialog.repo_cursor, 5, "stops on the last repository");
    // 10 rows, 6 visible: the cursor on the last repo scrolled the list.
    assert_eq!(dialog.repo_offset, 4);
    dialog.on_key(key(KeyCode::Home));
    assert_eq!((dialog.repo_cursor, dialog.repo_offset), (0, 0));
    dialog.on_key(key(KeyCode::End));
    assert_eq!(dialog.repo_cursor, 5);
    // Going back up onto the first repo of a section brings its header.
    for _ in 0..2 {
        dialog.on_key(key(KeyCode::Up));
    }
    assert_eq!(names(&dialog)[dialog.repo_cursor], "acme/zed");
    dialog.on_key(key(KeyCode::Up));
    assert_eq!(names(&dialog)[dialog.repo_cursor], "acme/alpha");
    assert_eq!(dialog.repo_offset, 3, "the acme header (row 3) is not cut");
    let (text, _) = paint(&mut dialog, 130, 34);
    assert!(text.contains("acme (2)"), "{text}");
}

#[test]
fn a_private_repository_has_a_lock_and_a_header_has_no_hit() {
    let mut dialog = grouped_dialog();
    let (text, _) = paint(&mut dialog, 130, 34);
    assert_eq!(text.matches('\u{1F512}').count(), 1, "{text}");
    let lock_line = text
        .lines()
        .find(|l| l.contains('\u{1F512}'))
        .expect("a lock row");
    assert!(lock_line.contains("kai/secret"), "{lock_line}");
    // Only repositories are targets: the header row answers nothing.
    let first = dialog.hit_rect(CloneHit::Repo(0));
    dialog.on_mouse(click(first.x + 3, first.y - 1));
    assert_eq!(dialog.chosen, None);
    dialog.on_mouse(click(first.x + 3, first.y));
    assert_eq!(dialog.chosen.as_deref(), Some("kai/Mine"));
}

#[test]
fn the_list_scrollbar_drags_the_view() {
    let mut dialog = with_hosted();
    dialog.source = 1;
    let many: Vec<HostedProject> = (0..30).map(|i| project(&format!("team/r{i:02}"))).collect();
    dialog.set_projects("corp", Ok(many));
    paint(&mut dialog, 130, 34);
    let track = dialog.list_bar.expect("31 rows overflow 6");
    let down = |row| MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: track.rect.x,
        row,
        modifiers: KeyModifiers::NONE,
    };
    // Pressing the bare track brings the thumb under the pointer.
    dialog.on_mouse(down(track.rect.bottom() - 3));
    assert!(dialog.repo_offset > 10, "{}", dialog.repo_offset);
    assert_eq!(dialog.chosen, None, "a bar press never chooses a row");
    dialog.on_mouse(MouseEvent {
        kind: MouseEventKind::Up(MouseButton::Left),
        ..down(0)
    });
}

fn review_dialog(project: &str) -> CloneDialog {
    CloneDialog::new(Some(&abs("clones")))
        .with_hosted(hosted())
        .with_url("https://gitlab.example.com/acme/app.git")
        .with_target("corp", project, "note".into())
}

#[test]
fn a_review_clone_opens_in_the_account_mode_and_selects_the_project_once_listed() {
    let mut dialog = review_dialog("acme/app");
    assert!(dialog.in_account_mode());
    assert_eq!(dialog.source, 1);
    assert_eq!(dialog.message.as_deref(), Some("note"));
    // The list is asked for at once, no rest.
    assert_eq!(dialog.poll(), CloneOutcome::LoadProjects("corp".into()));
    let list: Vec<_> = (0..9)
        .map(|i| project(&format!("team{i}/lib")))
        .chain([project("acme/other"), project("acme/app")])
        .collect();
    dialog.set_projects("corp", Ok(list));
    assert_eq!(dialog.chosen.as_deref(), Some("acme/app"));
    assert_eq!(
        dialog.hosted_url.as_deref(),
        Some("https://gitlab.example.com/acme/app.git")
    );
    assert_eq!(dialog.name, "app");
    assert_eq!(dialog.message.as_deref(), Some("note"));
    let (text, _) = paint(&mut dialog, 130, 34);
    assert!(text.contains("acme/app"), "{text}");
    assert!(dialog.dest().is_some());
}

#[test]
fn a_project_missing_from_the_list_keeps_the_mode_with_the_search_on_it() {
    let mut dialog = review_dialog("acme/gone");
    dialog.poll();
    dialog.set_projects("corp", Ok(vec![project("acme/app")]));
    assert!(dialog.in_account_mode());
    assert_eq!(dialog.chosen, None);
    assert_eq!(dialog.search, "acme/gone");
    assert!(
        dialog.message.as_deref().unwrap().contains("acme/gone"),
        "{:?}",
        dialog.message
    );
}

#[test]
fn a_profile_that_is_no_source_falls_back_to_the_url_prefill() {
    let dialog = CloneDialog::new(Some(&abs("clones")))
        .with_hosted(hosted())
        .with_url("https://x.example.com/a/b.git")
        .with_target("unknown", "a/b", "note".into());
    assert!(!dialog.in_account_mode());
    assert_eq!(dialog.source, 0);
    assert_eq!(dialog.url, "https://x.example.com/a/b.git");
    assert_eq!(dialog.name, "b");
}

/// The source and repository rows of the dialog are list rows: the arrow,
/// and the whole row lights under the pointer (the same wash as the Refs
/// pane's rows).
#[test]
fn the_dialogs_list_rows_keep_the_arrow_and_light_as_a_whole() {
    use crate::pointer_map::{self, Kind};
    let mut dialog = with_hosted();
    paint(&mut dialog, 130, 34);
    let second = dialog.hit_rect(CloneHit::Source(1));
    dialog.on_mouse(click(second.x + 2, second.y));
    dialog.set_projects(
        "corp",
        Ok((0..4).map(|i| project(&format!("team/r{i}"))).collect()),
    );
    pointer_map::begin_frame();
    paint(&mut dialog, 130, 34);
    for hit in [CloneHit::Source(0), CloneHit::Repo(1)] {
        let row = dialog.hit_rect(hit);
        assert_eq!(
            pointer_map::at(row.x + 2, row.y),
            Some(Kind::Plain),
            "{hit:?}"
        );
        assert_eq!(
            pointer_map::shape_at(row.x + 2, row.y),
            crate::cursor::CursorShape::Default
        );
    }
    let row = dialog.hit_rect(CloneHit::Repo(2));
    dialog.hover = Some(CloneHit::Repo(2));
    let mut buf = ratatui::buffer::Buffer::empty(ratatui::layout::Rect::new(0, 0, 130, 34));
    let rect = dialog.rect(buf.area);
    dialog.render(rect, &mut buf);
    for x in row.x..row.right() {
        assert_eq!(buf[(x, row.y)].bg, crate::th().ui.hover, "col {x}");
    }
    let rest = dialog.hit_rect(CloneHit::Repo(3));
    assert_ne!(buf[(rest.x + 2, rest.y)].bg, crate::th().ui.hover);
}
