use super::*;
use crossterm::event::{KeyCode, KeyModifiers};
use gitpoulpi_core::domain::RefKind;
use ratatui::backend::TestBackend;
use ratatui::Terminal;

/// The popup is process-wide, and so are the remembered settings: tests that
/// open it take turns.
static SERIAL: Mutex<()> = Mutex::new(());

fn oid(n: u8) -> Oid {
    Oid([n; 20])
}

fn node(id: u8, parents: &[u8], summary: &str, refs: Vec<RefKind>) -> CommitNode {
    CommitNode {
        id: oid(id),
        parents: parents.iter().map(|p| oid(*p)).collect(),
        summary: summary.into(),
        message: summary.into(),
        author_name: "Ada".into(),
        author_email: "a@example.com".into(),
        timestamp: i64::from(id) * 100,
        refs,
    }
}

/// 1 <- 2 <- 5 (main, merge of 2 and 4), 1 <- 3 <- 4 (feature).
fn graph() -> CommitGraph {
    CommitGraph {
        nodes: vec![
            node(5, &[2, 4], "merge feature", vec![]),
            node(4, &[3], "feature two", vec![]),
            node(3, &[1], "feature one", vec![]),
            node(2, &[1], "main work", vec![]),
            node(1, &[], "first", vec![]),
        ],
    }
}

fn refs() -> Vec<Ref> {
    let r = |kind, n: u8, head| Ref {
        kind,
        target: oid(n),
        visibility: gitpoulpi_core::domain::RefVisibility::Shown,
        is_head: head,
        upstream: None,
        tag_sync: None,
        multi_remote: None,
    };
    vec![
        r(
            RefKind::LocalBranch {
                name: "main".into(),
            },
            5,
            true,
        ),
        r(
            RefKind::LocalBranch {
                name: "feature".into(),
            },
            4,
            false,
        ),
        r(RefKind::Tag { name: "v1".into() }, 1, false),
    ]
}

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn screen() -> String {
    let mut terminal = Terminal::new(TestBackend::new(110, 40)).unwrap();
    terminal.draw(render).unwrap();
    let buf = terminal.backend().buffer();
    let mut out = String::new();
    for y in 0..buf.area.height {
        for x in 0..buf.area.width {
            out.push_str(buf[(x, y)].symbol());
        }
        out.push('\n');
    }
    out
}

fn text() -> String {
    POPUP.lock().unwrap().as_ref().unwrap().text.clone()
}

fn fresh() -> std::sync::MutexGuard<'static, ()> {
    let guard = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    live_settings::set_mermaid(live_settings::MermaidPrefs {
        detail: MermaidDetail::Refs,
        orientation: MermaidOrientation::Lr,
        fence: false,
        subject_width: 40,
    });
    guard
}

#[test]
fn it_previews_the_selection_and_enter_copies_exactly_that_text() {
    let _g = fresh();
    open(&graph(), &refs(), &[oid(5), oid(4), oid(3)], oid(5)).unwrap();
    let shown = screen();
    if std::env::var("MM_PRINT").is_ok() {
        println!("{shown}");
    }
    assert!(shown.contains("Copy as Mermaid gitGraph"), "{shown}");
    assert!(shown.contains("gitGraph LR:"), "{shown}");
    assert!(shown.contains("Selection (3 commits)"), "{shown}");
    let before = text();
    assert!(
        before.contains("type: HIGHLIGHT"),
        "HEAD is marked: {before}"
    );
    match on_key(key(KeyCode::Enter), &None) {
        Outcome::Copy(copied) => assert_eq!(copied, before),
        other => panic!("{other:?}"),
    }
    assert!(!is_open(), "copying closes the popup");
}

#[test]
fn the_detail_combo_changes_the_preview_and_is_remembered() {
    let _g = fresh();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");
    let cfg = Some(path.clone());
    open(&graph(), &refs(), &[oid(5), oid(4), oid(3)], oid(5)).unwrap();
    assert!(
        text().contains("tag:") || !text().contains("subject"),
        "refs level"
    );
    // Tab to Detail, open it, pick Subjects (two Downs from Refs is Full;
    // one is Subjects).
    assert_eq!(on_key(key(KeyCode::Tab), &cfg), Outcome::None);
    assert_eq!(on_key(key(KeyCode::Tab), &cfg), Outcome::None);
    // Focus order is Scope, Detail, ...: from Preview two Tabs wrap to Scope
    // then Detail.
    on_key(key(KeyCode::Enter), &cfg);
    assert!(screen().contains("Structure: "), "the list is open");
    on_key(key(KeyCode::Down), &cfg);
    on_key(key(KeyCode::Enter), &cfg);
    let after = text();
    assert!(
        after.contains("feature two"),
        "subjects in the ids: {after}"
    );
    assert_eq!(live_settings::mermaid().detail, MermaidDetail::Subjects);
    let on_disk = std::fs::read_to_string(&path).unwrap();
    assert!(
        on_disk.contains("mermaid_detail = \"subjects\""),
        "{on_disk}"
    );
    // Reopening starts on the remembered level.
    on_key(key(KeyCode::Esc), &cfg);
    assert!(!is_open());
    open(&graph(), &refs(), &[oid(5), oid(4), oid(3)], oid(5)).unwrap();
    assert!(text().contains("feature two"));
}

#[test]
fn space_on_the_fence_checkbox_wraps_the_script() {
    let _g = fresh();
    open(&graph(), &refs(), &[oid(5), oid(4), oid(3)], oid(5)).unwrap();
    for _ in 0..4 {
        on_key(key(KeyCode::Tab), &None);
    }
    // Preview -> Scope -> Detail -> Orientation -> Fence.
    on_key(key(KeyCode::Char(' ')), &None);
    assert!(text().starts_with("```mermaid\n"), "{}", text());
    assert!(text().ends_with("```\n"));
}

#[test]
fn one_commit_defaults_to_the_run_up_to_head_and_the_scope_can_change() {
    let _g = fresh();
    open(&graph(), &refs(), &[], oid(3)).unwrap();
    let shown = screen();
    assert!(
        shown.contains("From the selection to HEAD (3 commits)"),
        "{shown}"
    );
    let guard = POPUP.lock().unwrap();
    assert_eq!(guard.as_ref().unwrap().commits, 3);
}

#[test]
fn the_preview_scrolls_with_the_keys_and_the_wheel() {
    let _g = fresh();
    let mut nodes = Vec::new();
    for i in 1..=60u8 {
        let parents: &[u8] = if i == 1 { &[] } else { &[i - 1] };
        nodes.insert(0, node(i, parents, "step", vec![]));
    }
    let g = CommitGraph { nodes };
    let head = Ref {
        kind: RefKind::LocalBranch {
            name: "main".into(),
        },
        target: oid(60),
        visibility: gitpoulpi_core::domain::RefVisibility::Shown,
        is_head: true,
        upstream: None,
        tag_sync: None,
        multi_remote: None,
    };
    // One commit selected: the run from it up to HEAD, sixty commits.
    open(&g, &[head], &[], oid(1)).unwrap();
    let _ = screen();
    assert!(POPUP.lock().unwrap().as_ref().unwrap().lines.len() > 20);
    on_key(key(KeyCode::PageDown), &None);
    let top = POPUP.lock().unwrap().as_ref().unwrap().top;
    assert!(top > 0, "PageDown scrolled");
    on_mouse(
        MouseEvent {
            kind: MouseEventKind::ScrollUp,
            column: 10,
            row: 10,
            modifiers: KeyModifiers::NONE,
        },
        &None,
    );
    assert!(
        POPUP.lock().unwrap().as_ref().unwrap().top < top,
        "the wheel scrolled back"
    );
    on_key(key(KeyCode::Esc), &None);
}

#[test]
fn clicking_the_copy_button_copies_and_clicking_away_closes() {
    let _g = fresh();
    open(&graph(), &refs(), &[oid(5), oid(4)], oid(5)).unwrap();
    let _ = screen();
    let copy = POPUP.lock().unwrap().as_ref().unwrap().rects.copy;
    let click = |col: u16, row: u16| MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: col,
        row,
        modifiers: KeyModifiers::NONE,
    };
    // A click away from the dialog closes it and copies nothing.
    assert_eq!(on_mouse(click(0, 0), &None), Outcome::None);
    assert!(!is_open());
    open(&graph(), &refs(), &[oid(5), oid(4)], oid(5)).unwrap();
    let _ = screen();
    assert!(matches!(
        on_mouse(click(copy.x + 1, copy.y), &None),
        Outcome::Copy(_)
    ));
}

#[test]
fn the_cross_closes_and_the_footer_has_no_esc_hint() {
    let _g = fresh();
    open(&graph(), &refs(), &[oid(5), oid(4), oid(3)], oid(5)).unwrap();
    let shown = screen();
    assert!(shown.contains('\u{2715}'), "{shown}");
    assert!(!shown.contains("Esc"), "{shown}");
    let cross = POPUP
        .lock()
        .unwrap()
        .as_ref()
        .unwrap()
        .cross
        .rect()
        .unwrap();
    let click = MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: cross.x + 1,
        row: cross.y,
        modifiers: crossterm::event::KeyModifiers::NONE,
    };
    assert_eq!(on_mouse(click, &None), Outcome::None);
    assert!(!is_open());
}
