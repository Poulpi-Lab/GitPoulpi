//! The Add / Edit remote popup: close cross, outside click, dirty guard.

use super::*;
use ratatui::backend::TestBackend;
use ratatui::Terminal;

fn remote(name: &str, fetch: &str) -> Remote {
    Remote {
        name: name.into(),
        fetch_url: fetch.into(),
        push_url: None,
    }
}

fn paint(form: &RemoteForm) -> (Vec<(Rect, RemoteFormHit)>, String) {
    let mut terminal = Terminal::new(TestBackend::new(100, 30)).unwrap();
    let mut hits = Vec::new();
    terminal
        .draw(|f| hits = form.render(f, f.area(), None))
        .unwrap();
    let buf = terminal.backend().buffer().clone();
    let text = (0..buf.area.height)
        .map(|y| {
            (0..buf.area.width)
                .map(|x| buf[(x, y)].symbol())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n");
    (hits, text)
}

#[test]
fn the_popup_has_a_cross_that_closes_and_no_esc_text() {
    let mut form = RemoteForm::add(vec![]);
    let (hits, text) = paint(&form);
    assert!(text.contains('\u{2715}'), "{text}");
    assert!(!text.contains("Esc"), "{text}");
    let (_, cross) = hits
        .iter()
        .find(|(_, h)| *h == RemoteFormHit::Close)
        .expect("the cross is a hit target");
    assert_eq!(form.on_click(*cross), FormOutcome::Close);
}

#[test]
fn a_click_outside_closes_a_pristine_form_and_is_swallowed_when_dirty() {
    let mut form = RemoteForm::add(vec![]);
    paint(&form);
    assert_eq!(form.on_outside_click(0, 0), Some(FormOutcome::Close));
    assert_eq!(form.on_outside_click(50, 15), None, "inside the popup");
    form.name = "up".into();
    assert_eq!(form.on_outside_click(0, 0), Some(FormOutcome::Stay));
    assert!(form.flash.get(), "the cross blinks on the next paint");
    paint(&form);
    assert!(!form.flash.get(), "and only once");
    // An edit left alone is pristine too.
    let r = remote("origin", "https://example.com/a.git");
    let mut form = RemoteForm::edit(&r, vec![]);
    paint(&form);
    assert_eq!(form.on_outside_click(0, 0), Some(FormOutcome::Close));
    form.fetch_url.push('x');
    assert_eq!(form.on_outside_click(0, 0), Some(FormOutcome::Stay));
}
