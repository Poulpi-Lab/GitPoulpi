use super::*;

#[test]
fn supersampling_is_on_by_default_and_clamped() {
    // The real function, split so the parse can be driven without
    // touching this process' environment.
    let parse = supersample_from;
    assert_eq!(parse(None), 2, "supersampling must be on by default");
    assert_eq!(parse(Some("1")), 1, "1 must be a real off switch");
    assert_eq!(parse(Some("0")), 1, "0 would make a zero-pixel bitmap");
    assert_eq!(parse(Some("99")), 4);
    assert_eq!(parse(Some("nonsense")), 2);
}

#[test]
fn base64_matches_rfc_4648_test_vectors() {
    assert_eq!(base64(b""), "");
    assert_eq!(base64(b"f"), "Zg==");
    assert_eq!(base64(b"fo"), "Zm8=");
    assert_eq!(base64(b"foo"), "Zm9v");
    assert_eq!(base64(b"foob"), "Zm9vYg==");
    assert_eq!(base64(b"fooba"), "Zm9vYmE=");
    assert_eq!(base64(b"foobar"), "Zm9vYmFy");
}

#[test]
fn an_absurd_cell_size_is_clamped_rather_than_trusted() {
    // `TIOCGWINSZ` can report zeros, and a zero divides by zero in
    // the rasteriser.
    assert_eq!(
        CellSize::sanitised(0, 0),
        CellSize {
            width: 4,
            height: 6
        }
    );
    assert_eq!(
        CellSize::sanitised(9999, 9999),
        CellSize {
            width: 64,
            height: 128
        }
    );
}

#[test]
fn an_encoded_image_is_a_complete_iterm2_file_escape() {
    let bitmap = Bitmap::new(18, 42, (34, 36, 46));
    let seq = encode(Protocol::Iterm2, &bitmap, 2, 2).expect("should encode");
    assert!(seq.starts_with("\u{1b}]1337;File="));
    assert!(seq.ends_with('\u{7}'), "the sequence must be terminated");
    // `doNotMoveCursor=1` is WezTerm's extension and the reason an
    // image can sit on the last row of the alternate screen without
    // scrolling the whole TUI. Losing it is silent until someone
    // selects the bottom commit.
    assert!(seq.contains(";doNotMoveCursor=1:"));
    assert!(seq.contains(";width=2;height=2;"));
    let payload = seq.rsplit_once(':').unwrap().1.trim_end_matches('\u{7}');
    assert!(
        payload.starts_with("iVBORw0KGgo"),
        "the payload must be a PNG"
    );
}

#[test]
fn a_text_only_terminal_encodes_nothing() {
    let bitmap = Bitmap::new(9, 21, (0, 0, 0));
    assert_eq!(encode(Protocol::None, &bitmap, 1, 1), None);
}

#[test]
fn a_reserved_image_area_is_never_emitted_by_ratatui() {
    use ratatui::style::Style;
    let area = Rect::new(2, 1, 3, 2);
    let mut buf = Buffer::empty(Rect::new(0, 0, 8, 4));
    buf.set_string(2, 1, "xxx", Style::default());
    ImageReservation.render(area, &mut buf);
    for y in area.top()..area.bottom() {
        for x in area.left()..area.right() {
            assert!(
                buf[(x, y)].skip,
                "cell {x},{y} is under the image and must never be emitted"
            );
            assert_eq!(buf[(x, y)].symbol(), " ");
        }
    }
    // Outside the area the buffer is untouched.
    assert!(!buf[(5, 1)].skip);
}

#[test]
fn an_image_is_written_at_its_anchor_and_what_covers_it_goes_on_top() {
    let mut over = ratatui::buffer::Cell::default();
    over.set_symbol("T");
    let mut out = Vec::new();
    write_image(&mut out, (4, 2), "<IMAGE>", &[(5, 3, over)]).unwrap();
    let out = String::from_utf8(out).unwrap();
    // Zero-based (4, 2) is one-based row 3, column 5.
    let placed = out
        .find("[3;5H")
        .expect("the cursor is moved to the anchor");
    let image = out.find("<IMAGE>").expect("the image is written");
    let popup = out.rfind('T').expect("the covering cell is written");
    assert!(placed < image && image < popup, "wrong order: {out:?}");
    assert!(out.starts_with("7"), "the cursor is saved first: {out:?}");
    assert!(out.ends_with("8"), "and restored last: {out:?}");
}

/// Per terminal: only WezTerm is known to blend PNG alpha; every other
/// iTerm2-protocol terminal keeps the opaque backdrop.
#[test]
fn only_wezterm_gets_the_transparent_iterm2_picture() {
    for program in ["iTerm.app", "mintty", "rio", "Bobcat"] {
        assert_eq!(
            protocol_for(RendererChoice::Auto, &env(&[("TERM_PROGRAM", program)])),
            Protocol::Iterm2,
            "{program}"
        );
    }
    // Forcing the protocol does not change what the terminal can blend.
    let forced = |pairs: &[(&str, &str)]| {
        super::super::protocol_for(
            RendererChoice::Auto,
            ImageProtocol::Iterm2,
            &env(pairs),
            &|_: &std::path::Path| None,
        )
    };
    assert_eq!(forced(&WEZTERM), Protocol::Iterm2Alpha);
    assert_eq!(forced(&[("TERM_PROGRAM", "mintty")]), Protocol::Iterm2);
    // Forced image mode inside herdr: not WezTerm's own cells.
    assert_eq!(
        protocol_for(RendererChoice::Image, &env(&wezterm_inside_herdr())),
        Protocol::Iterm2
    );
}

fn unbase64(text: &str) -> Vec<u8> {
    let value = |c: u8| match c {
        b'A'..=b'Z' => c - b'A',
        b'a'..=b'z' => c - b'a' + 26,
        b'0'..=b'9' => c - b'0' + 52,
        b'+' => 62,
        _ => 63,
    };
    let mut out = Vec::new();
    for chunk in text.trim_end_matches('=').as_bytes().chunks(4) {
        let mut n = 0u32;
        for (i, c) in chunk.iter().enumerate() {
            n |= u32::from(value(*c)) << (18 - 6 * i);
        }
        out.extend_from_slice(&n.to_be_bytes()[1..chunk.len()]);
    }
    out
}

/// The RGBA pixels of the PNG inside an iTerm2 escape, decoded the way the
/// terminal would.
pub(super) fn png_pixels(sequence: &str) -> (u32, u32, Vec<u8>) {
    let payload = sequence
        .rsplit_once(':')
        .unwrap()
        .1
        .trim_end_matches('\u{7}');
    let bytes = unbase64(payload);
    let mut reader = png::Decoder::new(std::io::Cursor::new(bytes))
        .read_info()
        .expect("a PNG");
    let mut buf = vec![0; reader.output_buffer_size().unwrap()];
    let info = reader.next_frame(&mut buf).expect("a frame");
    assert_eq!(info.color_type, png::ColorType::Rgba);
    (info.width, info.height, buf)
}

/// What WezTerm receives: an RGBA PNG whose untouched pixels have alpha 0.
#[test]
fn the_wezterm_escape_carries_a_transparent_rgba_png() {
    let mut bitmap = Bitmap::transparent(18, 42);
    bitmap.blend(9, 21, (64, 156, 255), 1.0);
    let seq = encode(Protocol::Iterm2Alpha, &bitmap, 2, 2).expect("should encode");
    let (w, h, px) = png_pixels(&seq);
    assert_eq!((w, h), (18, 42));
    assert_eq!(px[3], 0, "the top-left pixel must be fully transparent");
    let last = px.len() - 1;
    assert_eq!(
        px[last], 0,
        "the bottom-right pixel must be fully transparent"
    );
    let center = (21 * 18 + 9) * 4;
    assert_eq!(&px[center..center + 4], &[64, 156, 255, 255]);
    // The opaque flavour keeps its backdrop.
    let opaque = encode(Protocol::Iterm2, &Bitmap::new(18, 42, (34, 36, 46)), 2, 2).unwrap();
    assert_eq!(png_pixels(&opaque).2[3], 255);
}

#[test]
fn a_blank_reservation_keeps_the_row_background_and_is_not_skipped() {
    use ratatui::style::{Color, Style};
    let area = Rect::new(1, 0, 2, 2);
    let mut buf = Buffer::empty(Rect::new(0, 0, 4, 2));
    buf.set_string(
        1,
        0,
        "ab",
        Style::default().bg(Color::Rgb(52, 58, 90)).fg(Color::Red),
    );
    BlankReservation.render(area, &mut buf);
    let c = &buf[(1, 0)];
    assert!(!c.skip);
    assert_eq!(c.symbol(), " ");
    assert_eq!(c.bg, Color::Rgb(52, 58, 90));
    assert_eq!(c.fg, Color::Reset);
}

#[test]
fn a_sixel_picture_is_exactly_its_cells_in_pixels_and_blends() {
    let cell = cell_size();
    let bitmap = Bitmap::new(40, 40, (34, 36, 46));
    let encoded = encode_image(Protocol::Sixel, &bitmap, 2, 3).expect("should encode");
    let seq = encoded.sequence();
    assert!(seq.starts_with("\u{1b}P0;1;0q\"1;1;"), "{seq:?}");
    assert!(seq.ends_with("\u{1b}\\"), "the sequence must be terminated");
    let raster = format!("\"1;1;{};{}", 2 * cell.width, 3 * cell.height);
    assert!(seq.contains(&raster), "{raster} in {seq:?}");
    // Handled like a transparent iTerm2 picture: nothing places itself,
    // nothing is uploaded, the cells under it are real blanks.
    assert!(encoded.blends() && !encoded.places_itself());
    assert!(!encoded.needs_upload(Rect::new(0, 0, 80, 24)));
    let mut buf = Buffer::empty(Rect::new(0, 0, 4, 4));
    encoded.reserve(Rect::new(0, 0, 2, 3), &mut buf);
    assert!(buf[(0, 0)].modifier.contains(BLANK_RESERVED));
    assert!(!buf[(0, 0)].skip);
}
