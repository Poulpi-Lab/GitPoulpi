use super::model::*;
use super::scope::ScopeRepo;
use super::*;

use crate::daemon_link::RawQueue;
use crate::th;
use gitpoulpi_core::domain::{Oid, SearchHit};
use ratatui::backend::TestBackend;
use ratatui::Terminal;

fn hit(kind: SearchKind, label: &str, score: i32, target: SearchTarget) -> SearchHit {
    SearchHit {
        kind,
        label: label.into(),
        detail: String::new(),
        target,
        score,
        positions: Vec::new(),
        repo: None,
    }
}

fn branch(name: &str, score: i32) -> SearchHit {
    hit(
        SearchKind::Branch,
        name,
        score,
        SearchTarget::Ref {
            name: format!("refs/heads/{name}"),
        },
    )
}

fn commit(subject: &str, score: i32, n: u8) -> SearchHit {
    hit(
        SearchKind::Commit,
        subject,
        score,
        SearchTarget::Commit { id: Oid([n; 20]) },
    )
}

fn scope(names: &[&str]) -> Scope {
    Scope {
        workspace: None,
        repos: names
            .iter()
            .map(|n| ScopeRepo::new(std::path::Path::new(&format!("/nowhere/{n}"))))
            .collect(),
    }
}

fn search_over(names: &[&str]) -> WorkspaceSearch {
    // The queue outlives the search: a dropped receiver would answer every
    // request with an error at once.
    let queue: &'static RawQueue = Box::leak(Box::default());
    WorkspaceSearch::with_parts(scope(names), queue.requester(), Arc::new(Mutex::new(None)))
}

fn labels(search: &WorkspaceSearch) -> Vec<String> {
    search
        .rows()
        .iter()
        .map(|r| match r {
            Row::Heading(kind, n) => format!("# {} {n}", kind.name()),
            Row::Item(i) => search.items()[*i].label.clone(),
        })
        .collect()
}

#[test]
fn hits_of_every_repository_merge_by_score_under_one_heading_per_kind() {
    let items = vec![
        Item::from_hit(commit("fix parser", 100, 1), 0),
        Item::from_hit(branch("parser-rewrite", 50), 1),
        Item::from_hit(branch("parser", 300), 0),
        Item::from_hit(commit("parser tests", 900, 2), 1),
        Item::from_hit(branch("old-parser", 50), 0),
    ];
    let (kept, rows) = group(items);
    let text: Vec<String> = rows
        .iter()
        .map(|r| match r {
            Row::Heading(k, n) => format!("# {} {n}", k.name()),
            Row::Item(i) => format!("{}@{}", kept[*i].label, kept[*i].repo),
        })
        .collect();
    assert_eq!(
        text,
        [
            "# branch 3",
            "parser@0",
            "old-parser@0",
            "parser-rewrite@1",
            "# commit 2",
            "parser tests@1",
            "fix parser@0",
        ]
    );
}

#[test]
fn a_kind_is_cut_at_the_group_cap_without_hiding_the_others() {
    let mut items: Vec<Item> = (0..PER_GROUP + 20)
        .map(|i| Item::from_hit(commit(&format!("c{i:03}"), 1000 - i as i32, i as u8), 0))
        .collect();
    items.push(Item::from_hit(branch("main", 1), 0));
    let (kept, rows) = group(items);
    assert_eq!(
        kept.iter().filter(|i| i.kind == SearchKind::Commit).count(),
        PER_GROUP
    );
    assert!(matches!(rows[0], Row::Heading(SearchKind::Branch, 1)));
    assert!(rows
        .iter()
        .any(|r| matches!(r, Row::Heading(SearchKind::Commit, n) if *n == PER_GROUP)));
}

#[test]
fn the_highlight_skips_headings_both_ways() {
    let (_, rows) = group(vec![
        Item::from_hit(branch("a", 2), 0),
        Item::from_hit(commit("b", 1, 1), 0),
    ]);
    // rows: 0 heading, 1 a, 2 heading, 3 b
    assert_eq!(skip_headings(&rows, 1, 2), 3);
    assert_eq!(skip_headings(&rows, 3, 2), 1);
    assert_eq!(skip_headings(&rows, 1, 0), 1);
    assert_eq!(skip_headings(&rows, 3, 9), 3);
    assert_eq!(first_item_row(&rows), Some(1));
}

#[test]
fn a_prefix_replaces_the_chips_and_hash_means_hosting() {
    let mut chips = [true; 8];
    assert_eq!(effective_kinds("x", &chips).len(), 8);
    assert_eq!(effective_kinds("c: x", &chips), [SearchKind::Commit]);
    assert_eq!(effective_kinds("pr: x", &chips), [SearchKind::PullRequest]);
    assert_eq!(
        effective_kinds("#12", &chips),
        [SearchKind::PullRequest, SearchKind::Issue]
    );
    chips[5] = false;
    assert!(!effective_kinds("x", &chips).contains(&SearchKind::Commit));
    // A prefix wins over a chip that is off.
    assert_eq!(effective_kinds("c: x", &chips), [SearchKind::Commit]);
}

#[test]
fn an_answer_of_an_older_generation_is_dropped() {
    let mut search = search_over(&["alpha", "bravo"]);
    search.draft = "pars".into();
    search.dispatch();
    let stale = search.generation();
    search.draft = "parse".into();
    search.dispatch();
    let current = search.generation();
    assert_eq!(current, stale + 1);

    let old = SearchHits {
        hits: vec![branch("parser-old", 10)],
        partial: false,
    };
    assert!(!search.apply_answer(stale, 0, Ok(old)));
    search.rebuild(true);
    assert!(search.items().is_empty());

    let new = SearchHits {
        hits: vec![branch("parser", 10)],
        partial: false,
    };
    assert!(search.apply_answer(current, 1, Ok(new)));
    search.rebuild(true);
    assert_eq!(labels(&search), ["# branch 1", "parser"]);
    assert_eq!(search.items()[0].repo, 1);
    // A repository that failed lists nothing and is not an error.
    assert!(search.apply_answer(current, 0, Err("gone".into())));
}

#[test]
fn the_highlight_follows_its_result_when_more_answers_arrive() {
    let mut search = search_over(&["alpha", "bravo"]);
    search.dispatch();
    let g = search.generation();
    search.apply_answer(
        g,
        0,
        Ok(SearchHits {
            hits: vec![branch("m", 10), branch("n", 5)],
            partial: false,
        }),
    );
    search.rebuild(false);
    // The repository kind is listed first (`r:` rows), then branches: move to "n".
    let n_row = search
        .rows()
        .iter()
        .position(|r| matches!(r, Row::Item(i) if search.items()[*i].label == "n"))
        .unwrap();
    search.selected = n_row;
    search.apply_answer(
        g,
        1,
        Ok(SearchHits {
            hits: vec![branch("zzz-better", 99)],
            partial: false,
        }),
    );
    search.rebuild(true);
    assert_eq!(search.selected_item().unwrap().label, "n");
    // Typing resets it to the first result.
    search.edited();
    assert_eq!(search.selected, first_item_row(search.rows()).unwrap());
}

#[test]
fn repositories_are_matched_locally_by_name_then_path() {
    let mut search = search_over(&["alpha", "bravo"]);
    search.draft = "r: brav".into();
    search.edited();
    assert_eq!(labels(&search), ["# repo 1", "bravo"]);
    assert_eq!(search.items()[0].positions, [0, 1, 2, 3]);
}

#[test]
fn hosting_caches_are_matched_locally_and_a_prefix_narrows() {
    let mut search = search_over(&["alpha", "bravo"]);
    let cache: CachedItems = serde_json::from_str(
        r##"{"prs":[{"number":12,"iid_label":"#12","title":"Fix login","author":"ana",
              "source_branch":"fix/login","state":"open"}],
            "issues":[{"number":7,"iid_label":"#7","title":"Login is slow","author":"cy"}]}"##,
    )
    .unwrap();
    *search.caches.lock().unwrap() = Some(vec![CachedItems::default(), cache]);
    search.draft = "login".into();
    search.tick(Instant::now());
    let text = labels(&search);
    assert!(text.contains(&"# pull_request 1".to_string()), "{text:?}");
    assert!(text.contains(&"# issue 1".to_string()), "{text:?}");
    assert!(search.items().iter().all(|i| i.repo == 1));
    search.draft = "i: login".into();
    search.edited();
    assert_eq!(labels(&search), ["# issue 1", "#7 Login is slow"]);
    // Enter on it reveals the issue row of that repository.
    assert_eq!(
        search.go_intent(),
        Some(UiIntent::ShowHosting {
            repo: search.scope().repos[1].repo.id.clone(),
            item: HostingItem::Issue(7)
        })
    );
}

#[test]
fn a_pull_request_result_can_be_reviewed_and_nothing_else_can() {
    let mut search = search_over(&["alpha", "bravo"]);
    let cache: CachedItems = serde_json::from_str(
        r##"{"prs":[{"number":12,"iid_label":"#12","title":"Fix login","author":"ana",
              "source_branch":"fix/login","state":"open"}],
            "issues":[{"number":7,"iid_label":"#7","title":"Login is slow","author":"cy"}]}"##,
    )
    .unwrap();
    *search.caches.lock().unwrap() = Some(vec![CachedItems::default(), cache]);
    search.draft = "login".into();
    search.tick(Instant::now());
    search.draft = "pr: login".into();
    search.edited();
    assert_eq!(
        search.review_request(),
        crate::build_features::REVIEW.then(|| (search.scope().repos[1].repo.root.clone(), 12))
    );
    search.draft = "i: login".into();
    search.edited();
    assert_eq!(search.review_request(), None, "an issue is not reviewed");
}

#[test]
fn enter_builds_the_intent_that_reveals_each_kind() {
    let mut search = search_over(&["alpha"]);
    search.dispatch();
    let g = search.generation();
    let id = search.scope().repos[0].repo.id.clone();
    let oid = Oid([7; 20]);
    let hits = vec![
        branch("topic", 100),
        hit(
            SearchKind::Stash,
            "wip",
            90,
            SearchTarget::Stash { index: 2, id: None },
        ),
        commit("fix it", 80, 7),
        hit(
            SearchKind::Worktree,
            "wt",
            70,
            SearchTarget::Worktree {
                path: "/nowhere/wt".into(),
            },
        ),
    ];
    search.apply_answer(
        g,
        0,
        Ok(SearchHits {
            hits,
            partial: false,
        }),
    );
    search.rebuild(false);
    let go = |search: &mut WorkspaceSearch, label: &str| {
        search.selected = search
            .rows()
            .iter()
            .position(|r| matches!(r, Row::Item(i) if search.items()[*i].label == label))
            .unwrap();
        search.go_intent().unwrap()
    };
    assert_eq!(
        go(&mut search, "topic"),
        UiIntent::ShowRef {
            repo: id.clone(),
            name: "refs/heads/topic".into()
        }
    );
    assert_eq!(
        go(&mut search, "wip"),
        UiIntent::ShowRef {
            repo: id.clone(),
            name: "stash@{2}".into()
        }
    );
    assert_eq!(
        go(&mut search, "fix it"),
        UiIntent::ShowCommit {
            repo: id.clone(),
            commit: oid,
            also: Vec::new(),
            files: false,
            focus: None
        }
    );
    assert_eq!(
        go(&mut search, "wt"),
        UiIntent::ShowTab {
            repo: RepoId::from_path(std::path::Path::new("/nowhere/wt"))
        }
    );
    assert_eq!(
        go(&mut search, "alpha"),
        UiIntent::ShowTab {
            repo: RepoId::from_path(std::path::Path::new("/nowhere/alpha"))
        }
    );
    // A heading is not a result.
    search.selected = 0;
    assert!(search.is_heading(0) && search.go_intent().is_none());
}

#[test]
fn the_debounce_holds_the_request_back_and_an_edit_restarts_it() {
    let mut search = search_over(&["alpha"]);
    search.edited();
    let first = Instant::now();
    search.tick(first);
    assert_eq!(search.generation(), 0, "still inside the debounce");
    assert!(search.animating());
    search.tick(first + DEBOUNCE * 2);
    assert_eq!(search.generation(), 1);
    assert_eq!(search.pending.len(), 1);
    // The spinner waits for the 150 ms rule.
    assert!(search.spinner().is_none());
}

#[test]
fn the_scope_is_the_active_workspace_else_the_open_tabs() {
    let dir = tempfile::tempdir().unwrap();
    let repos: Vec<_> = ["one", "two", "three"]
        .iter()
        .map(|n| {
            let p = dir.path().join(n);
            std::fs::create_dir_all(&p).unwrap();
            p
        })
        .collect();
    let tabs = vec![RepoTab {
        path: repos[0].clone(),
        name: "one".into(),
        branch: None,
    }];
    // No session file: the open tabs.
    let none = scope::search_scope_in(dir.path(), "s", &tabs);
    assert!(none.workspace.is_none());
    assert_eq!(none.repos.len(), 1);

    let mut all = gitpoulpi_session::Workspaces::default();
    {
        let w = all.create("Acme");
        w.add_repo(&repos[1], None);
        w.add_repo(&repos[2], None);
        w.add_repo(&repos[1], None);
    }
    all.save(&gitpoulpi_session::Workspaces::file_in(dir.path()))
        .unwrap();
    let mut session = gitpoulpi_session::GlobalSession {
        active_workspace: Some("acme".into()),
        ..Default::default()
    };
    std::fs::create_dir_all(dir.path().join("sessions")).unwrap();
    session
        .save(&dir.path().join("sessions").join("s.toml"))
        .unwrap();

    let ws = scope::search_scope_in(dir.path(), "s", &tabs);
    assert_eq!(ws.workspace.as_deref(), Some("Acme"));
    let names: Vec<_> = ws.repos.iter().map(|r| r.name.as_str()).collect();
    assert_eq!(
        names,
        ["two", "three"],
        "the workspace's repositories, once each"
    );

    // A stale id names nothing: back to the tabs.
    session.active_workspace = Some("gone".into());
    session
        .save(&dir.path().join("sessions").join("s.toml"))
        .unwrap();
    assert!(scope::search_scope_in(dir.path(), "s", &tabs)
        .workspace
        .is_none());
}

/// What the terminal is handed: the matched characters of a label are bold in
/// the accent color, the rest is not.
#[test]
fn the_popup_paints_headings_icons_chips_and_highlighted_matches() {
    let _lang = gitpoulpi_i18n::override_lang(gitpoulpi_i18n::Lang::En);
    let mut search = search_over(&["alpha", "bravo"]);
    search.dispatch();
    let g = search.generation();
    let mut parser = branch("parser-rewrite", 500);
    parser.positions = vec![0, 1, 2, 3];
    parser.detail = "origin/parser-rewrite".into();
    search.apply_answer(
        g,
        1,
        Ok(SearchHits {
            hits: vec![parser, commit("fix the parser", 100, 3)],
            partial: false,
        }),
    );
    search.draft = "pars".into();
    search.rebuild(false);

    let mut terminal = Terminal::new(TestBackend::new(120, 40)).unwrap();
    let editor = crate::text_edit::FieldEditor::default();
    let mut popup = None;
    terminal
        .draw(|f| popup = Some(render(f, &search, &editor)))
        .unwrap();
    let popup = popup.unwrap();
    let buf = terminal.backend().buffer();
    let line_of = |y: u16| -> String {
        (0..buf.area.width)
            .map(|x| buf[(x, y)].symbol().to_string())
            .collect::<String>()
    };
    let screen: Vec<String> = (0..buf.area.height).map(line_of).collect();
    let row_of = |needle: &str| -> u16 {
        screen
            .iter()
            .position(|l| l.contains(needle))
            .unwrap_or_else(|| panic!("no `{needle}` on\n{}", screen.join("\n"))) as u16
    };

    // Title, chips, group headings, the repository chip.
    assert!(screen
        .iter()
        .any(|l| l.contains("Search open repositories (2)")));
    assert!(screen
        .iter()
        .any(|l| l.contains("Branches") && l.contains("Commits")));
    let branch_row = row_of("parser-rewrite");
    assert!(screen[branch_row as usize].contains("bravo"), "repo chip");
    assert!(
        screen[branch_row as usize].contains("origin/parser-rewrite"),
        "detail"
    );
    assert!(screen.iter().any(|l| l
        .trim_start_matches(['\u{2502}', ' '])
        .starts_with("Branches (1)")));

    // The first four characters of the label are highlighted, the fifth is not.
    let col = (0..buf.area.width)
        .find(|&x| buf[(x, branch_row)].symbol() == "p" && buf[(x + 1, branch_row)].symbol() == "a")
        .unwrap();
    for i in 0..4 {
        let cell = &buf[(col + i, branch_row)];
        assert_eq!(cell.fg, th().ui.accent, "char {i} is highlighted");
        assert!(cell.modifier.contains(ratatui::style::Modifier::BOLD));
    }
    assert_ne!(
        buf[(col + 4, branch_row)].fg,
        th().ui.accent,
        "the rest is plain"
    );

    // The rects come from this walk: the list holds the rows, and the chips
    // answer where they were drawn.
    assert!(popup.count >= 4 && popup.list.height > 0);
    let chips_row = screen
        .iter()
        .position(|l| l.contains("Repositories") && l.contains("Issues"))
        .unwrap() as u16;
    let repo_chip_x = (0..buf.area.width)
        .find(|&x| buf[(x, chips_row)].symbol() == "R")
        .unwrap();
    assert_eq!(chip_at(repo_chip_x, chips_row), Some(0));
    assert_eq!(chip_at(0, 0), None);
}

#[test]
fn a_chip_that_is_off_is_not_searched_and_is_painted_plain() {
    let mut search = search_over(&["alpha"]);
    search.toggle_chip(5);
    assert!(!search.chips[5]);
    assert!(!effective_kinds(&search.draft, &search.chips).contains(&SearchKind::Commit));
    search.reset_chips();
    assert!(search.chips.iter().all(|c| *c));
}

#[test]
fn a_long_label_is_cut_with_an_ellipsis_and_keeps_its_highlight() {
    let spans = view::highlighted(
        "abcdefghij",
        &[0, 1, 9],
        6,
        ratatui::style::Style::default(),
    );
    let text: String = spans.iter().map(|s| s.content.to_string()).collect();
    assert_eq!(text, "abcde\u{2026}");
    assert_eq!(spans[0].content, "ab");
    assert_ne!(spans[0].style, spans[1].style);
}

/// The window's share of a keystroke (the daemon's is in `perf-budgets.md`):
/// ten repositories answering forty hits each, ten hosting caches of 300
/// items, merged, grouped and painted. The budget for the whole path is
/// 150 ms with the daemon's 46 ms in it, so this must stay a small slice.
#[test]
fn a_keystroke_over_ten_repositories_costs_the_window_a_few_milliseconds() {
    let names: Vec<String> = (0..10).map(|i| format!("repo{i}")).collect();
    let refs: Vec<&str> = names.iter().map(String::as_str).collect();
    let mut search = search_over(&refs);
    let caches: Vec<CachedItems> = (0..10)
        .map(|r| {
            let issues: Vec<serde_json::Value> = (0..300)
                .map(|i| {
                    serde_json::json!({
                        "number": i, "iid_label": format!("#{i}"),
                        "title": format!("parser issue {r}-{i} about login"),
                        "author": "ana", "state": "open"
                    })
                })
                .collect();
            serde_json::from_value(serde_json::json!({ "issues": issues })).unwrap()
        })
        .collect();
    *search.caches.lock().unwrap() = Some(caches);
    search.dispatch();
    let g = search.generation();
    for r in 0..10 {
        let hits: Vec<SearchHit> = (0..40)
            .map(|i| {
                commit(
                    &format!("parser fix {r}-{i}"),
                    1000 - i as i32,
                    (r * 40 + i) as u8,
                )
            })
            .collect();
        search.apply_answer(
            g,
            r,
            Ok(SearchHits {
                hits,
                partial: false,
            }),
        );
    }
    search.tick(Instant::now());

    let editor = crate::text_edit::FieldEditor::default();
    let mut terminal = Terminal::new(TestBackend::new(160, 50)).unwrap();
    // The fastest round, not the mean: a loaded machine only ever adds time,
    // so the minimum is the cost of the work and a regression raises it too.
    let mut per_key = Duration::MAX;
    const ROUNDS: u32 = 20;
    for i in 0..ROUNDS {
        let started = Instant::now();
        search.draft = if i % 2 == 0 { "parser" } else { "parse" }.into();
        search.edited();
        search.rebuild(true);
        terminal
            .draw(|f| {
                render(f, &search, &editor);
            })
            .unwrap();
        per_key = per_key.min(started.elapsed());
    }
    eprintln!("window share of one keystroke over 10 repositories: {per_key:?}");
    assert!(!search.items().is_empty());
    // Generous for a debug build; a release build is an order of magnitude less.
    assert!(per_key < Duration::from_millis(60), "{per_key:?}");
}

/// The pointer over the global search: the I-beam on its text field, the
/// hand on the close cross and the kind chips, the arrow on a result row, a
/// group heading and the empty areas (the popup covers the toolbar and the
/// panes, whose hands must not leak through).
#[test]
fn the_pointer_over_the_global_search_follows_the_pointer_rules() {
    use crate::cursor::CursorShape;
    let mut search = search_over(&["alpha"]);
    let g = search.generation();
    search.apply_answer(
        g,
        1,
        Ok(SearchHits {
            hits: vec![
                branch("parser-rewrite", 500),
                commit("fix the parser", 100, 3),
            ],
            partial: false,
        }),
    );
    search.draft = "pars".into();
    search.rebuild(false);
    let mut terminal = Terminal::new(TestBackend::new(120, 40)).unwrap();
    let editor = crate::text_edit::FieldEditor::default();
    crate::pointer_map::begin_frame();
    let mut popup = None;
    terminal
        .draw(|f| popup = Some(render(f, &search, &editor)))
        .unwrap();
    let popup = popup.unwrap();
    let shape = |c: u16, r: u16| crate::pointer_map::shape_at(c, r);
    // The text field: the draft row inside the input box.
    let field_row = popup.popup.y + 1;
    assert_eq!(shape(popup.popup.x + 3, field_row), CursorShape::Text);
    // The kind chips line: the first chip is a control.
    let chips_row = popup.popup.y + 3;
    let chip_col = (popup.popup.x..popup.popup.right())
        .find(|c| crate::workspace_search::chip_at(*c, chips_row).is_some())
        .expect("a chip");
    assert_eq!(shape(chip_col, chips_row), CursorShape::Pointer);
    // The close cross.
    let cross = (popup.popup.x..popup.popup.right())
        .find(|c| crate::workspace_search::close_at(*c, popup.popup.y))
        .expect("the cross");
    assert_eq!(shape(cross, popup.popup.y), CursorShape::Pointer);
    // A heading, a result row and the area outside the popup: the arrow.
    assert_eq!(shape(popup.list.x + 3, popup.list.y), CursorShape::Default);
    assert_eq!(
        shape(popup.list.x + 3, popup.list.y + 1),
        CursorShape::Default
    );
    assert_eq!(shape(0, 0), CursorShape::Default);
    assert_eq!(shape(119, 39), CursorShape::Default);
    // The label of the input box (its border row) is not the field.
    assert_eq!(
        shape(popup.popup.x + 3, popup.popup.y),
        CursorShape::Default
    );
}
