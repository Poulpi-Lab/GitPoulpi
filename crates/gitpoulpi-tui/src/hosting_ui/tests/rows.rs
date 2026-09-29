//! What the PULL REQUESTS and ISSUES sections list and how they paint: groups, filter, search, plus buttons, hover, reveal.

use super::*;

#[test]
fn sections_appear_only_when_a_profile_answers() {
    let dir = tempfile::tempdir().unwrap();
    let ui = HostingUi::with_parts(dir.path(), None, None, profiles, None);
    assert_eq!(ui.sections_visible(), (false, false));
    assert_eq!(ui.row_counts(), (0, 0));

    // Search, filter, the four group headers and "My" (expanded)'s one
    // pull request; provider, repository, search, filter and the issue.
    let (ui, _d) = ui_with(snapshot(vec![open_pr(53)], vec![issue(12)]));
    assert_eq!(ui.sections_visible(), (true, true));
    // Plus the pull requests' provider and repository dropdowns.
    assert_eq!(ui.row_counts(), (9, 5));

    // Unlinked: both sections only offer their provider dropdowns.
    let mut unlinked = snapshot(vec![], vec![]);
    unlinked.profile_id = None;
    unlinked.pulls_target = None;
    unlinked.issues_target = None;
    let (ui, _d) = ui_with(unlinked.clone());
    assert_eq!(ui.sections_visible(), (true, true));
    assert_eq!(ui.row_counts(), (2, 2));
    unlinked.issue_profiles.clear();
    unlinked.pull_profiles.clear();
    let (ui, _d) = ui_with(unlinked);
    assert_eq!(ui.sections_visible(), (false, false));
}

#[test]
fn reveal_finds_the_row_of_a_pull_request_or_issue_and_unfolds_its_group() {
    let (mut ui, _d) = ui_with(snapshot(vec![open_pr(1), open_pr(2)], vec![issue(5)]));
    // Folded groups open when the item lives in one.
    for g in PrGroup::ALL {
        ui.collapsed.insert(g);
    }
    let RevealOutcome::Row(row) = ui.reveal(HostingList::Pulls, 2) else {
        panic!("a row");
    };
    assert!(matches!(
        ui.rows(HostingList::Pulls)[row],
        HostingRow::Pull(_, i) if i == 1
    ));
    let RevealOutcome::Row(row) = ui.reveal(HostingList::Issues, 5) else {
        panic!("a row");
    };
    assert!(matches!(
        ui.rows(HostingList::Issues)[row],
        HostingRow::Issue(0)
    ));
    assert_eq!(ui.reveal(HostingList::Pulls, 99), RevealOutcome::NotListed);
    assert_eq!(ui.reveal(HostingList::Issues, 99), RevealOutcome::NotListed);
}

#[test]
fn reveal_leaves_the_users_search_alone_and_never_links_a_provider() {
    let (mut ui, _d) = ui_with(snapshot(vec![open_pr(1)], vec![]));
    ui.pr_query = "no-such-title".into();
    assert_eq!(ui.reveal(HostingList::Pulls, 1), RevealOutcome::NotListed);
    assert_eq!(ui.pr_query, "no-such-title");

    let mut unlinked = snapshot(vec![], vec![]);
    unlinked.profile_id = None;
    unlinked.issues_target = None;
    let (mut ui, _d) = ui_with(unlinked);
    assert_eq!(ui.reveal(HostingList::Pulls, 1), RevealOutcome::NotLinked);
    assert_eq!(ui.reveal(HostingList::Issues, 1), RevealOutcome::NotLinked);
}

#[test]
fn the_pull_request_section_is_a_search_a_filter_and_four_groups_with_counts() {
    let mut mine = open_pr(1);
    mine.author = "kai".into();
    let mut assigned = pr(2, "b", PullRequestState::Open);
    assigned.author = "zed".into();
    assigned.assignees = vec!["KAI".into()];
    let mut review = pr(3, "c", PullRequestState::Open);
    review.author = "zed".into();
    review.reviewers = vec!["kai".into()];
    let mut other = pr(4, "d", PullRequestState::Open);
    other.author = "zed".into();
    let (mut ui, _d) = ui_with(snapshot(vec![mine, assigned, review, other], vec![]));
    // The three personal groups start open, "All" folded.
    let (screen, hits, rows) = draw_section(&ui, HostingList::Pulls, 44, 14, None);
    for want in [
        "PULL REQUESTS (4)",
        "Search pull requests",
        "Open",
        "My Pull Requests",
        "Assigned to me",
        "Awaiting my review",
        "All Pull Requests",
    ] {
        assert!(screen.contains(want), "{want}\n{screen}");
    }
    let count_of = |label: &str| -> String {
        rows.iter()
            .find(|r| r.contains(label))
            .unwrap_or_else(|| panic!("{label}\n{screen}"))
            .trim_end_matches([' ', '\u{2502}'])
            .rsplit(' ')
            .next()
            .unwrap()
            .to_string()
    };
    assert_eq!(count_of("My Pull Requests"), "1");
    assert_eq!(count_of("Assigned to me"), "1");
    assert_eq!(count_of("Awaiting my review"), "1");
    assert_eq!(count_of("All Pull Requests"), "4");
    assert!(screen.contains("!1  Change number 1"), "{screen}");
    assert!(!screen.contains("!4  Change number 4"), "All is folded");

    // Unfolding "All" lists everything; a group header is a control.
    let all = row_of(&ui, HostingList::Pulls, HostingRow::Group(PrGroup::All));
    assert!(ui.rows(HostingList::Pulls)[all].is_control());
    let mut clip = crate::clipboard::Clipboard::new();
    ui.activate_row(HostingList::Pulls, all, &mut clip);
    let (screen, _, _) = draw_section(&ui, HostingList::Pulls, 44, 20, None);
    assert!(screen.contains("!4  Change number 4"), "{screen}");

    // Row rects sit on the rows they paint, inside the borders.
    let (_, hits2, rows2) = draw_section(&ui, HostingList::Pulls, 44, 20, None);
    let _ = hits;
    for (i, row) in ui.rows(HostingList::Pulls).iter().enumerate() {
        let (rect, _) = hits2
            .iter()
            .find(|(_, t)| *t == HitTarget::Hosting(HostingHit::Row(HostingList::Pulls, i)))
            .unwrap_or_else(|| panic!("row {i} {row:?} has no hit rect"));
        assert_eq!((rect.x, rect.y, rect.width), (1, 1 + i as u16, 42));
        if let HostingRow::Pull(_, n) = row {
            assert!(
                rows2[rect.y as usize].contains(&format!("!{}", n + 1)),
                "{rows2:?}"
            );
        }
    }
}

#[test]
fn the_search_box_narrows_every_group_and_types_from_the_keyboard() {
    let mut a = open_pr(1);
    a.title = "Fix the parser".into();
    let mut b = open_pr(2);
    b.title = "Update docs".into();
    let (mut ui, _d) = ui_with(snapshot(vec![a, b], vec![]));
    let search = row_of(&ui, HostingList::Pulls, HostingRow::Search);
    let mut clip = crate::clipboard::Clipboard::new();
    assert_eq!(
        ui.activate_row(HostingList::Pulls, search, &mut clip),
        HostingEffect::None
    );
    assert_eq!(ui.editing, Some(HostingList::Pulls));
    for c in "parser".chars() {
        assert!(ui.on_search_key(key(KeyCode::Char(c))));
    }
    let (screen, _, _) = draw_section(&ui, HostingList::Pulls, 44, 14, None);
    assert!(screen.contains("parser"), "{screen}");
    assert!(screen.contains("Fix the parser"), "{screen}");
    assert!(!screen.contains("Update docs"), "{screen}");
    // Backspace edits, Ctrl+U clears, Enter leaves the box (text kept).
    ui.on_search_key(key(KeyCode::Backspace));
    assert_eq!(ui.pr_query, "parse");
    ui.on_search_key(KeyEvent::new(KeyCode::Char('u'), KeyModifiers::CONTROL));
    assert_eq!(ui.pr_query, "");
    ui.on_search_key(key(KeyCode::Char('d')));
    ui.on_search_key(key(KeyCode::Enter));
    assert_eq!(ui.editing, None);
    assert_eq!(ui.pr_query, "d");
    assert!(!ui.on_search_key(key(KeyCode::Char('x'))), "not editing");
}

/// The filter is a row: present at every width and after any number of
/// changes.
#[test]
fn the_filter_never_disappears_however_often_it_is_used_or_narrow_the_pane() {
    let (mut ui, _d) = ui_with(snapshot(vec![open_pr(53)], vec![issue(12)]));
    let mut clip = crate::clipboard::Clipboard::new();
    for round in 0..12 {
        for width in [20u16, 24, 30, 44] {
            let (screen, hits, _) = draw_section(&ui, HostingList::Pulls, width, 12, None);
            let filter = row_of(&ui, HostingList::Pulls, HostingRow::Filter);
            assert!(
                hits.iter()
                    .any(|(_, t)| *t
                        == HitTarget::Hosting(HostingHit::Row(HostingList::Pulls, filter))),
                "round {round} width {width}: no filter hit\n{screen}"
            );
            assert!(
                screen.contains(ui.pr_view.label().split(' ').next().unwrap()),
                "round {round} width {width}\n{screen}"
            );
        }
        // Open the dropdown and take the next choice, like a user.
        let filter = row_of(&ui, HostingList::Pulls, HostingRow::Filter);
        ui.activate_row(HostingList::Pulls, filter, &mut clip);
        assert!(ui.picker.is_some(), "round {round}");
        ui.on_picker_key(key(KeyCode::Down));
        ui.on_picker_key(key(KeyCode::Enter));
        assert!(ui.picker.is_none());
    }
}

#[test]
fn the_filter_dropdown_lists_every_choice_and_a_pick_asks_the_host_again() {
    let (mut ui, _d) = ui_with(snapshot(vec![open_pr(53)], vec![]));
    let mut clip = crate::clipboard::Clipboard::new();
    let filter = row_of(&ui, HostingList::Pulls, HostingRow::Filter);
    assert_eq!(
        ui.activate_row(HostingList::Pulls, filter, &mut clip),
        HostingEffect::Focus(HostingList::Pulls)
    );
    // Painted over the section, with its rows registered first.
    let mut states = SidebarStates::default();
    let mut hits = Vec::new();
    let mut terminal = Terminal::new(TestBackend::new(44, 14)).unwrap();
    terminal
        .draw(|f| {
            ui.render_section(
                f,
                Rect::new(0, 0, 44, 14),
                HostingList::Pulls,
                true,
                true,
                None,
                &mut states,
                &mut hits,
            );
            ui.render_picker(f, None, &mut hits);
        })
        .unwrap();
    let buf = terminal.backend().buffer().clone();
    let text: String = (0..14)
        .map(|y| (0..44).map(|x| buf[(x, y)].symbol()).collect::<String>() + "\n")
        .collect();
    for want in ["Open", "Merged / closed", "All"] {
        assert!(text.contains(want), "{want}\n{text}");
    }
    let items: Vec<&(Rect, HitTarget)> = hits
        .iter()
        .filter(|(_, t)| matches!(t, HitTarget::Hosting(HostingHit::PickerItem(_))))
        .collect();
    assert_eq!(items.len(), 3);
    assert!(std::ptr::eq(items[0], &hits[0]), "spliced to the front");
    // A click on "All" picks it.
    assert_eq!(
        ui.picker_click(2),
        HostingEffect::Message("pull requests: All".into())
    );
    assert_eq!(ui.pr_view, PrView::All);
    assert!(ui.picker.is_none());
}

#[test]
fn a_pull_request_section_has_a_plus_that_overlays_the_title_when_narrow() {
    let (ui, _d) = ui_with(snapshot(vec![open_pr(53)], vec![]));
    let (screen, hits, rows) = draw_section(&ui, HostingList::Pulls, 44, 8, None);
    let plus = hits
        .iter()
        .find(|(_, t)| *t == HitTarget::Hosting(HostingHit::NewPr))
        .expect("the + is registered");
    let painted: String = rows[0]
        .chars()
        .skip(plus.0.x as usize)
        .take(plus.0.width as usize)
        .collect();
    assert_eq!(painted, " + ", "{screen}");
    let plus_at = hits
        .iter()
        .position(|(_, t)| *t == HitTarget::Hosting(HostingHit::NewPr))
        .unwrap();
    let header_at = hits
        .iter()
        .position(|(_, t)| matches!(t, HitTarget::RefsSectionHeader(_)))
        .unwrap();
    assert!(plus_at < header_at, "inner targets first");

    // Too narrow to sit beside the title: the + overlays its end and
    // stays registered (it is never dropped while it fits the borders).
    let (_, hits, rows) = draw_section(&ui, HostingList::Pulls, 22, 5, None);
    let plus = hits
        .iter()
        .find(|(_, t)| *t == HitTarget::Hosting(HostingHit::NewPr))
        .expect("the + overlays the title when narrow")
        .0;
    let painted: String = rows[0]
        .chars()
        .skip(plus.x as usize)
        .take(plus.width as usize)
        .collect();
    // At rest the title stays whole (the button is clickable but not
    // drawn); with the header hovered it is drawn over the title's end.
    assert_ne!(painted, " + ");
    let header = HitTarget::RefsSectionHeader(HostingList::Pulls.section());
    let (_, _, rows) = draw_section(&ui, HostingList::Pulls, 22, 5, Some(header));
    let painted: String = rows[0]
        .chars()
        .skip(plus.x as usize)
        .take(plus.width as usize)
        .collect();
    assert_eq!(painted, " + ");
    // Never a hit rect outside the area, at any width.
    for w in 8..60 {
        let (_, hits, _) = draw_section(&ui, HostingList::Pulls, w, 8, None);
        for (rect, _) in hits {
            assert!(rect.right() <= w && rect.bottom() <= 8, "w={w} {rect:?}");
        }
    }
}

#[test]
fn the_pull_request_and_issue_plus_buttons_look_like_the_remote_one_and_name_their_action() {
    let (mut ui, _d) = ui_with(snapshot(vec![open_pr(53)], vec![issue(12)]));
    // ISSUES is linked in `ui_with` only through the snapshot: no
    // saved source yet, so no `+` and no hit.
    let (_, hits, _) = draw_section(&ui, HostingList::Issues, 60, 8, None);
    assert!(
        !hits
            .iter()
            .any(|(_, t)| *t == HitTarget::Hosting(HostingHit::NewIssue)),
        "no + while the ISSUES section is unlinked"
    );
    ui.set_issues_source("corp", "g/p".into());

    for (list, hit, tooltip) in [
        (HostingList::Pulls, HostingHit::NewPr, "+ New pull request"),
        (HostingList::Issues, HostingHit::NewIssue, "+ New issue"),
    ] {
        let target = HitTarget::Hosting(hit);
        // At rest: the bare glyph, in the REMOTE `+`'s own color.
        let (_, hits, rows) = draw_section(&ui, list, 60, 8, None);
        let rect = hits.iter().find(|(_, t)| *t == target).expect("a +").0;
        let painted: String = rows[0]
            .chars()
            .skip(rect.x as usize)
            .take(rect.width as usize)
            .collect();
        assert_eq!(painted, " + ");
        assert_eq!(rect.right() + 1, 60, "one cell in from the border");
        let mut states = SidebarStates::default();
        let mut terminal = Terminal::new(TestBackend::new(60, 8)).unwrap();
        terminal
            .draw(|f| {
                ui.render_section(
                    f,
                    Rect::new(0, 0, 60, 8),
                    list,
                    true,
                    true,
                    None,
                    &mut states,
                    &mut Vec::new(),
                );
            })
            .unwrap();
        let cell = terminal.backend().buffer()[(rect.x + 1, 0)].clone();
        assert_eq!(cell.fg, plain_button_style(false).fg.unwrap(), "{list:?}");

        // Hovered: the tooltip names the action.
        let (screen, hits, _) = draw_section(&ui, list, 60, 8, Some(target.clone()));
        assert!(screen.contains(tooltip), "{screen}");
        assert!(hits.iter().any(|(_, t)| *t == target));
    }
}

#[test]
fn hovering_a_row_or_the_plus_uses_the_strong_fill() {
    let (ui, _d) = ui_with(snapshot(vec![open_pr(53)], vec![]));
    let strong = plain_button_style(true).bg;
    let bg_at = |hovered: Option<HitTarget>, x: u16, y: u16| {
        let mut states = SidebarStates::default();
        let mut hits = Vec::new();
        let mut terminal = Terminal::new(TestBackend::new(40, 8)).unwrap();
        terminal
            .draw(|f| {
                ui.render_section(
                    f,
                    Rect::new(0, 0, 40, 8),
                    HostingList::Pulls,
                    true,
                    false,
                    hovered.as_ref(),
                    &mut states,
                    &mut hits,
                );
            })
            .unwrap();
        terminal.backend().buffer()[(x, y)].bg
    };
    // The "My Pull Requests" header row (index 2) and the PR under it.
    let group = row_of(&ui, HostingList::Pulls, HostingRow::Group(PrGroup::Mine));
    let group_row = HitTarget::Hosting(HostingHit::Row(HostingList::Pulls, group));
    assert_ne!(bg_at(None, 6, 1 + group as u16), strong.unwrap());
    assert_eq!(Some(bg_at(Some(group_row), 6, 1 + group as u16)), strong);
    let plus = HitTarget::Hosting(HostingHit::NewPr);
    // Same geometry as the REMOTE `+`: one cell in from the border.
    assert_ne!(bg_at(None, 37, 0), strong.unwrap());
    assert_eq!(Some(bg_at(Some(plus), 37, 0)), strong);
}

#[test]
fn the_issue_filter_and_search_work_like_the_pull_request_ones() {
    let mut fix = issue(1);
    fix.title = "Crash on start".into();
    let (mut ui, _d) = ui_with(snapshot(vec![], vec![fix, issue(2)]));
    let mut clip = crate::clipboard::Clipboard::new();
    let filter = row_of(&ui, HostingList::Issues, HostingRow::Filter);
    ui.activate_row(HostingList::Issues, filter, &mut clip);
    let labels: Vec<String> = ui
        .picker
        .as_ref()
        .unwrap()
        .combo
        .options
        .iter()
        .map(|o| o.label.clone())
        .collect();
    assert_eq!(labels, vec!["Open", "Assigned to me", "Closed", "All"]);
    ui.close_picker();
    let search = row_of(&ui, HostingList::Issues, HostingRow::Search);
    ui.activate_row(HostingList::Issues, search, &mut clip);
    for c in "crash".chars() {
        ui.on_search_key(key(KeyCode::Char(c)));
    }
    let listed = ui
        .rows(HostingList::Issues)
        .iter()
        .filter(|r| matches!(r, HostingRow::Issue(_)))
        .count();
    assert_eq!(listed, 1, "only the matching issue is listed");
}

#[test]
#[cfg(feature = "board")]
fn the_issues_header_has_a_board_button_left_of_the_plus_once_a_source_is_chosen() {
    let (mut ui, _d) = ui_with(snapshot(vec![], vec![issue(12)]));
    let board = HitTarget::Hosting(HostingHit::Board);
    let (_, hits, _) = draw_section(&ui, HostingList::Issues, 60, 8, None);
    assert!(
        hits.iter().all(|(_, t)| *t != board),
        "no Board while unlinked"
    );
    ui.set_issues_source("corp", "g/p".into());
    let (_, hits, rows) = draw_section(&ui, HostingList::Issues, 60, 8, None);
    let rect = hits
        .iter()
        .find(|(_, t)| *t == board)
        .expect("a Board button")
        .0;
    let plus = hits
        .iter()
        .find(|(_, t)| *t == HitTarget::Hosting(HostingHit::NewIssue))
        .expect("the +")
        .0;
    assert!(rect.right() <= plus.x, "{rect:?} is left of {plus:?}");
    let painted: String = rows[0]
        .chars()
        .skip(rect.x as usize)
        .take(rect.width as usize)
        .collect();
    assert_eq!(painted, " Board ");
    // Nothing outside the area, at any width.
    for w in 12..60 {
        let (_, hits, _) = draw_section(&ui, HostingList::Issues, w, 8, None);
        for (rect, _) in hits {
            assert!(rect.right() <= w && rect.bottom() <= 8, "w={w} {rect:?}");
        }
    }
}

/// What the sidebar's rows register for the pointer after a paint: the
/// comboboxes and the group headers are the hand, the search box the I-beam,
/// the items the arrow. Wherever it is drawn, a combobox is the same shape
/// (the Settings page's registers the hand too).
#[test]
fn the_issues_pane_and_the_settings_page_register_the_hand_on_their_comboboxes() {
    use crate::pointer_map::{self, Kind};
    let (mut ui, _d) = ui_with(snapshot(vec![open_pr(53)], vec![issue(12)]));
    ui.set_issues_source("corp", "g/p".into());
    for list in [HostingList::Pulls, HostingList::Issues] {
        pointer_map::begin_frame();
        let _ = draw_section(&ui, list, 50, 16, None);
        let rects: Vec<(usize, Rect)> = ui
            .row_rects
            .borrow()
            .iter()
            .filter(|(l, _, _)| *l == list)
            .map(|(_, i, r)| (*i, *r))
            .collect();
        let rows = ui.rows(list);
        let mut combos = 0;
        for (i, rect) in rects {
            let kind = pointer_map::at(rect.x + 3, rect.y);
            match &rows[i] {
                HostingRow::Filter
                | HostingRow::IssueProvider
                | HostingRow::IssueRepo
                | HostingRow::PullProvider
                | HostingRow::PullRepo => {
                    combos += 1;
                    assert_eq!(kind, Some(Kind::Hand), "{list:?} combobox {i}");
                }
                HostingRow::Search => assert_eq!(kind, Some(Kind::Field), "{list:?} search"),
                HostingRow::Group(_) => assert_eq!(kind, Some(Kind::Hand), "{list:?} group"),
                HostingRow::Pull(..) | HostingRow::Issue(_) | HostingRow::Notice(_) => {
                    assert_eq!(kind, Some(Kind::Plain), "{list:?} item {i}")
                }
                _ => {}
            }
        }
        assert!(combos > 0, "{list:?} paints at least one combobox");
    }
    let dir = tempfile::tempdir().unwrap();
    assert_eq!(
        crate::preferences::tests::render::first_combobox_pointer_kind(dir.path()),
        Some(Kind::Hand),
        "the Settings combobox is the hand like the pane's"
    );
}
