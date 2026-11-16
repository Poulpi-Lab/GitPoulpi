//! The Overview shares the timeline and the docked composer with the issue
//! view: the same keys, the same rows, the same buttons, the same write path.
//!
//! Falsified by routing the Overview's keys back to the review's own
//! (`overview_key` returning `None`) and by dropping `overview_mouse` from
//! `on_mouse`.

use super::*;
use crate::conversation_ui::{Btn, Hit as THit};
use gitpoulpi_hosting::conversation::{PageCursor, TimelineEntry, TimelinePage};

fn click_at(c: u16, r: u16) -> MouseEvent {
    MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: c,
        row: r,
        modifiers: KeyModifiers::NONE,
    }
}

fn on_overview() -> Rig {
    let mut r = rig(GITHUB_CAPS);
    r.screen();
    r
}

#[test]
fn r_replies_into_the_selected_discussion_and_the_text_goes_through_the_shared_write() {
    let mut r = on_overview();
    // The description is entry 0; tomas's comment is the next.
    r.key(plain(KeyCode::Down));
    r.key(plain(KeyCode::Char('r')));
    assert!(r.view.ov.composing_reply());
    assert!(r.view.ov.composer_focused());
    let s = r.screen();
    assert!(s.contains("Reply to tomas"), "{s}");
    r.type_text("Because");
    r.key(ctrl_enter());
    wait(&mut r.view, "the reply", |v| v.ov.write_idle());
    assert_eq!(r.conv.lock().unwrap().write_log, ["reply|d9|Because"]);
    // The host's own copy is in the timeline and the box is empty again.
    assert!(!r.view.ov.composer_focused());
    assert_eq!(r.view.ov.composer_text(), "");
}

#[test]
fn capital_q_quotes_the_selected_entry_into_a_new_comment() {
    let mut r = on_overview();
    r.key(plain(KeyCode::Down));
    r.key(plain(KeyCode::Char('Q')));
    assert!(!r.view.ov.composing_reply());
    let text = r.view.ov.composer_text();
    assert!(
        text.starts_with("> **tomas**\n> Looks promising."),
        "{text}"
    );
}

#[test]
fn the_buttons_of_the_selected_entry_are_the_issue_views() {
    let mut r = on_overview();
    r.key(plain(KeyCode::Down));
    let s = r.screen();
    assert!(s.contains("Reply") && s.contains("Quote"), "{s}");
    let rect = r
        .view
        .ov
        .conv()
        .hits
        .iter()
        .find_map(|(rect, h)| matches!(h, THit::Btn(Btn::Reply, _)).then_some(*rect))
        .expect("a Reply button");
    let effect = r.view.on_mouse(click_at(rect.x, rect.y));
    assert_eq!(effect, ReviewEffect::None);
    assert!(r.view.ov.composing_reply(), "the click is the key `r`");
}

#[test]
fn m_loads_older_entries_like_in_an_issue() {
    let mut r = on_overview();
    {
        let mut s = r.conv.lock().unwrap();
        let mut first = s.page.clone().unwrap();
        first.older = Some(PageCursor("1".into()));
        let older = TimelinePage {
            entries: vec![TimelineEntry::Comment(
                crate::conversation_ui::tests::comment(
                    "1",
                    "carol",
                    "The very first remark.",
                    "2026-10-01T08:00:00Z",
                ),
            )],
            older: None,
            ..first.clone()
        };
        s.page = Ok(first);
        s.older = Some(older);
    }
    r.view.ov.request_refresh();
    wait(&mut r.view, "the refresh", |v| v.ov.conv().older.is_some());
    assert!(!r.screen().contains("The very first remark."));
    r.key(plain(KeyCode::Char('m')));
    wait(&mut r.view, "the older page", |v| {
        v.ov.conv().older.is_none()
    });
    let s = r.screen();
    assert!(s.contains("The very first remark."), "{s}");
}

#[test]
fn the_timeline_wins_the_clashing_letters_and_the_review_keeps_the_rest() {
    let mut r = on_overview();
    // `m` is "load older" here, not the milestone picker.
    r.key(plain(KeyCode::Char('m')));
    assert!(r.view.meta.is_none());
    assert_eq!(
        r.view.message.as_deref(),
        Some(gitpoulpi_i18n::msg::item_view::WHY_NO_OLDER.get())
    );
    // `f` still opens Finish review, `Alt+2` the Files page.
    r.key(plain(KeyCode::Char('f')));
    assert!(r.view.finish.is_some());
    r.key(plain(KeyCode::Esc));
    r.key(alt('2'));
    assert_eq!(r.view.tab, Tab::Files);
}

#[test]
fn escape_leaves_the_composer_first_and_the_review_second() {
    let mut r = on_overview();
    r.key(plain(KeyCode::Char('c')));
    assert!(r.view.ov.composer_focused());
    // Typed words stay; Esc gives the keyboard back to the timeline.
    r.type_text("half");
    assert_eq!(r.key(plain(KeyCode::Esc)), ReviewEffect::None);
    assert!(!r.view.ov.composer_focused());
    assert_eq!(r.view.ov.composer_text(), "half");
    assert_eq!(r.key(plain(KeyCode::Esc)), ReviewEffect::Close);
}

#[test]
fn the_conversation_comment_command_opens_the_composer_from_any_page() {
    let mut r = on_overview();
    r.on_files();
    assert_eq!(
        r.view.run(ReviewCmd::ConversationComment),
        ReviewEffect::None
    );
    assert_eq!(r.view.tab, Tab::Overview);
    assert!(r.view.ov.composer_focused());
    // The palette's conversation rows follow the page.
    assert!(r.view.item_avail().is_some());
    r.key(plain(KeyCode::Esc));
    r.key(alt('2'));
    assert!(r.view.item_avail().is_none());
}

#[test]
fn another_window_writing_to_the_request_reads_it_again() {
    use gitpoulpi_ipc::{HostingChange, HostingChangeArgs, HostingItemKind};
    let mut r = on_overview();
    let reads = r.conv.lock().unwrap().reads.len();
    let change = |number: u64| HostingChangeArgs {
        integration_id: "corp".into(),
        project: "g/p".into(),
        kind: HostingItemKind::PullRequest,
        number,
        id: String::new(),
        change: HostingChange::Posted,
        origin: String::new(),
    };
    r.view.on_hosting_changed(&change(77));
    r.view.poll();
    assert_eq!(r.conv.lock().unwrap().reads.len(), reads, "another request");
    r.view.on_hosting_changed(&change(9));
    wait(&mut r.view, "the refresh", |v| {
        v.ov.loaded() && v.host_rx.is_none()
    });
    assert!(r.conv.lock().unwrap().reads.len() > reads);
}
