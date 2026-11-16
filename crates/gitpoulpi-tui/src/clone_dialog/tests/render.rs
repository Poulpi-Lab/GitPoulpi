use super::*;

#[test]
fn while_cloning_the_form_is_frozen_and_shows_progress() {
    let mut dialog = CloneDialog::new(Some(&abs("clones")));
    typed(&mut dialog, "https://x/y.git");
    dialog.progress = Some(
        CloneProgress::new(gitpoulpi_core::domain::ClonePhase::Receiving, 40).with_counts(4, 10),
    );
    typed(&mut dialog, "zzz");
    assert_eq!(dialog.url, "https://x/y.git");
    dialog.focus = CloneFocus::Clone;
    assert_eq!(dialog.on_key(key(KeyCode::Enter)), CloneOutcome::Stay);
    let mut buf = Buffer::empty(Rect::new(0, 0, 120, 30));
    dialog.render(clone_rect(buf.area), &mut buf);
    let text = dialog_kit::buffer_text(&buf);
    assert!(
        text.contains("40%") && text.contains("Receiving objects"),
        "{text}"
    );
    assert!(text.contains("Cloning"), "{text}");
}

#[test]
#[cfg(feature = "hosting")]
fn it_renders_at_two_widths_and_the_mouse_reaches_every_control() {
    for width in [80u16, 150] {
        let mut dialog = CloneDialog::new(Some(&abs("clones")));
        dialog.url = "https://example.com/team/app.git".into();
        dialog.sync_name();
        let mut buf = Buffer::empty(Rect::new(0, 0, width, 30));
        let rect = clone_rect(buf.area);
        dialog.render(rect, &mut buf);
        let text = dialog_kit::buffer_text(&buf);
        for needle in [
            "Clone a repository",
            "CLONE FROM",
            "Clone with URL",
            "Where to clone to",
            "URL",
            "Full path:",
            "Shallow clone",
            "Sparse checkout",
            "Clone",
            "Cancel",
        ] {
            assert!(text.contains(needle), "{width}: {needle}\n{text}");
        }
        for (r, _) in &dialog.hits {
            assert!(rect.contains((r.x, r.y).into()), "{r:?}");
        }
        let click_hit = |dialog: &mut CloneDialog, hit| {
            let r = dialog.hit_rect(hit);
            dialog.on_mouse(click(r.x + 1, r.y))
        };
        assert_eq!(
            click_hit(&mut dialog, CloneHit::Part(CloneFocus::Browse)),
            CloneOutcome::Browse(abs("clones"))
        );
        click_hit(&mut dialog, CloneHit::Part(CloneFocus::Shallow));
        assert!(dialog.shallow);
        click_hit(&mut dialog, CloneHit::Part(CloneFocus::Url));
        assert_eq!(dialog.focus, CloneFocus::Url);
        // Clone waits for the destination probe, then goes.
        assert_eq!(
            click_hit(&mut dialog, CloneHit::Part(CloneFocus::Clone)),
            CloneOutcome::Stay
        );
        assert!(matches!(settle(&mut dialog), CloneOutcome::Submit { .. }));
        assert_eq!(
            click_hit(&mut dialog, CloneHit::Part(CloneFocus::Cancel)),
            CloneOutcome::Cancel
        );
    }
}

/// The selected source row is shown by its highlight alone: no arrow glyph
/// before it, and the label starts at the list's own left edge.
#[test]
fn the_selected_source_row_has_no_arrow_marker() {
    let mut dialog = CloneDialog::new(Some(&abs("clones")));
    let mut buf = Buffer::empty(Rect::new(0, 0, 120, 30));
    dialog.render(clone_rect(buf.area), &mut buf);
    let text = dialog_kit::buffer_text(&buf);
    assert!(!text.contains('\u{25B8}'), "{text}");
    let (band, _) = *dialog
        .hits
        .iter()
        .find(|(_, h)| *h == CloneHit::Source(dialog.source))
        .expect("the selected source row");
    assert_ne!(
        buf[(band.x, band.y)].symbol(),
        " ",
        "the label starts at the edge"
    );
}

#[test]
fn a_window_too_small_for_the_form_says_so() {
    let mut dialog = CloneDialog::new(Some(&abs("clones")));
    let mut buf = Buffer::empty(Rect::new(0, 0, 50, 10));
    dialog.render(clone_rect(buf.area), &mut buf);
    let text = dialog_kit::buffer_text(&buf);
    assert!(text.contains("Window too small"), "{text}");
    assert!(text.contains('\u{2715}'), "{text}");
}
