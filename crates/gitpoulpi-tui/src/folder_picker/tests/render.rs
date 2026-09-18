use super::*;

/// Two widths: everything that matters is on screen and nothing is
/// drawn outside the dialog.
#[test]
fn it_renders_at_a_narrow_and_a_wide_width() {
    let tmp = tree();
    for (width, height) in [(80u16, 24u16), (160, 40)] {
        let mut picker = picker_at(tmp.path());
        let mut buf = Buffer::empty(Rect::new(0, 0, width, height));
        let rect = picker_rect(buf.area);
        picker.render(rect, &mut buf);
        let text = dialog_kit::buffer_text(&buf);
        for needle in [
            "Open a repository",
            "alpha",
            "FOLDERS",
            "Cancel",
            "Open",
            "Selected:",
        ] {
            assert!(text.contains(needle), "{width}: {needle}\n{text}");
        }
        for (hit, _) in picker.hits() {
            assert!(
                rect.contains((hit.x, hit.y).into()),
                "{hit:?} outside {rect:?}"
            );
        }
    }
}

// splitter, show hidden, new folder -------------------
