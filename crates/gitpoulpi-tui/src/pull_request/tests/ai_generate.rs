use super::*;

#[test]
fn the_ai_generate_hook_is_off_but_wired() {
    let mut d = dialog();
    assert!(!d.order().contains(&PrFocus::Generate));
    d.ai_generate = true;
    assert!(d.order().contains(&PrFocus::Generate));
    let (text, _) = paint(&mut d, 110, 34);
    assert!(
        text.contains("\u{2726} Generate title and description"),
        "{text}"
    );
    let row = text
        .lines()
        .position(|l| l.contains(GENERATE_LABEL.get()))
        .unwrap();
    let col = text
        .lines()
        .nth(row)
        .unwrap()
        .chars()
        .position(|c| c == '\u{2726}')
        .unwrap();
    let mut buf = Buffer::empty(Rect::new(0, 0, 110, 34));
    d.render(pull_request_rect(buf.area), &mut buf);
    assert_eq!(buf[(col as u16, row as u16)].fg, th().status.ai);
    d.focus = PrFocus::Generate;
    assert!(matches!(
        d.on_key(key(KeyCode::Enter)),
        PullRequestOutcome::Generate(_)
    ));
    d.set_generated("T".into(), "B".into());
    assert_eq!((d.title.as_str(), d.body.as_str()), ("T", "B"));
}

#[test]
fn the_generate_button_follows_the_ai_switch_and_locks_while_generating() {
    let mut d = dialog();
    assert!(!d.order().contains(&PrFocus::Generate));
    d.set_ai_generate(true);
    assert!(d.order().contains(&PrFocus::Generate));
    d.focus = PrFocus::Generate;
    // Fill in what the generator needs, then press it.
    d.from_branch.select(Some("feature".into()));
    d.to_branch.select(Some("main".into()));
    let first = d.on_key(key(KeyCode::Enter));
    if let PullRequestOutcome::Generate(req) = first {
        assert_eq!(req.source_branch, "feature");
        assert_eq!(req.target_branch, "main");
        assert!(d.generating);
        // A second press while it runs does nothing.
        assert!(matches!(
            d.on_key(key(KeyCode::Enter)),
            PullRequestOutcome::Stay
        ));
        let (text, _) = paint(&mut d, 110, 34);
        assert!(text.contains("Generating..."), "{text}");
        d.generation_failed("AI: nope".into());
        assert!(!d.generating);
        assert_eq!(d.message.as_deref(), Some("AI: nope"));
    } else {
        // The fixture's seed may not pre-fill the repositories: say so
        // rather than pass vacuously.
        panic!("expected Generate, the form said: {:?}", d.message);
    }
    d.set_ai_generate(false);
    assert!(!d.order().contains(&PrFocus::Generate));
    assert_ne!(d.focus, PrFocus::Generate);
}
