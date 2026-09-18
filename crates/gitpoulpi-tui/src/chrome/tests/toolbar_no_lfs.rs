use super::*;

/// A build without the `lfs` feature paints no `LFS ▾` button and
/// registers no hit rect for it, whatever the per-repository setting says.
#[test]
fn the_toolbar_has_no_lfs_button_without_the_feature() {
    use ratatui::backend::TestBackend;
    let area = Rect::new(0, 0, 120, 1);
    let mut terminal = Terminal::new(TestBackend::new(120, 1)).unwrap();
    let mut out = None;
    terminal
        .draw(|frame| {
            let toolbar = toolbar_render(area, None, ToolbarEnabled::ALL);
            frame.render_clipped(Paragraph::new(toolbar.line.clone()), area);
            out = Some(toolbar);
        })
        .unwrap();
    let toolbar = out.unwrap();
    let buf = terminal.backend().buffer();
    let text: String = (0..120).map(|x| buf[(x, 0)].symbol().to_string()).collect();
    assert!(!text.contains("LFS"), "{text:?}");
    assert!(toolbar
        .hits
        .iter()
        .all(|(_, t)| *t != HitTarget::Toolbar(ToolbarButton::Lfs)));
}

/// The `LFS` button is painted and hit-testable when the repository shows
/// it, and neither when `lfs_hidden` (not initialized, not in use).
#[test]
#[cfg(feature = "lfs")]
fn the_toolbar_shows_the_lfs_button_only_when_not_hidden() {
    for (hidden, expect) in [(false, true), (true, false)] {
        let area = Rect::new(0, 0, 160, 1);
        let toolbar = toolbar_render(
            area,
            None,
            ToolbarEnabled {
                lfs_hidden: hidden,
                ..ToolbarEnabled::ALL
            },
        );
        let line: String = toolbar
            .line
            .spans
            .iter()
            .map(|s| s.content.as_ref())
            .collect();
        assert_eq!(line.contains("LFS"), expect, "hidden={hidden}: {line:?}");
        assert_eq!(
            toolbar
                .hits
                .iter()
                .any(|(_, t)| *t == HitTarget::Toolbar(ToolbarButton::Lfs)),
            expect,
            "hidden={hidden}"
        );
    }
}
