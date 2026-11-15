//! Buffer tests: every block type at several widths, link hit rects, wide
//! glyphs, emoji machinery, stale cells, highlighted code.

use std::collections::BTreeSet;

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};

use super::*;

const WIDTHS: [usize; 5] = [12, 20, 40, 80, 120];

fn doc(src: &str, width: usize) -> Doc {
    render(src, width, &Context::default())
}

fn texts(src: &str, width: usize) -> Vec<String> {
    doc(src, width).lines.iter().map(MdLine::text).collect()
}

fn base() -> Style {
    Style::default().fg(Color::Rgb(200, 200, 200))
}

/// The whole document painted into a buffer `width` cells wide, plus the
/// rects of every link span.
fn paint_doc(d: &Doc, width: u16) -> (Buffer, Vec<(Rect, LinkId)>) {
    let mut buf = Buffer::empty(Rect::new(0, 0, width, d.lines.len().max(1) as u16));
    let mut hits = Vec::new();
    for (y, line) in d.lines.iter().enumerate() {
        hits.extend(paint(&mut buf, 0, y as u16, width, line, base(), false));
    }
    (buf, hits)
}

fn row(buf: &Buffer, y: u16) -> String {
    let mut out = String::new();
    let mut x = 0;
    while x < buf.area.width {
        let sym = buf[(x, y)].symbol();
        out.push_str(sym);
        x += crate::cell_text::cells(sym).max(1) as u16;
    }
    out.trim_end().to_string()
}

const EVERYTHING: &str = "\
# Title one

## Title two

### Title three

Plain **bold** *italic* ~~gone~~ `code` and a [link](https://example.com/a) here.

- one
  - nested two
    - nested three
- [x] done task
- [ ] open task

1. first
2. second

> quoted words that wrap around the edge

> [!WARNING]
> careful now

```rust
fn main() { let value = 42; }
```

| name | value |
|------|-------|
| a    | 1     |
| bb   | 22    |

line one<br>line two

<!-- hidden -->

![diagram](https://img.test/a.png)

Footnote[^1] and $x^2$ stay plain.

[^1]: the note

---

done
";

#[test]
fn every_block_fits_every_width_and_paints_inside_it() {
    for width in WIDTHS {
        let d = doc(EVERYTHING, width);
        assert!(!d.lines.is_empty());
        for line in &d.lines {
            assert!(
                line.width() <= width,
                "width {width}: {:?} is {} cells",
                line.text(),
                line.width()
            );
        }
        let (buf, _) = paint_doc(&d, width as u16);
        for y in 0..buf.area.height {
            assert!(
                crate::cell_text::cells(&row(&buf, y)) <= width,
                "width {width} row {y}"
            );
        }
    }
}

#[test]
fn headings_get_a_rule_from_level_two_down_only() {
    let t = texts("# One\n\n## Two\n\n### Three\n", 40);
    assert_eq!(
        t,
        [
            "One",
            "\u{2500}\u{2500}\u{2500}",
            "",
            "Two",
            "\u{2500}\u{2500}\u{2500}",
            "",
            "Three"
        ]
    );
    for width in WIDTHS {
        assert_eq!(texts("# One", width).len(), 2);
    }
}

#[test]
fn lists_nest_and_wrap_under_their_text() {
    let t = texts("- one\n  - two\n    - three\n1. a\n2. b\n", 40);
    assert_eq!(
        t,
        [
            "\u{2022} one",
            "  \u{25E6} two",
            "    \u{2022} three",
            "",
            "1. a",
            "2. b"
        ]
    );
    for width in WIDTHS {
        let t = texts("- alpha beta gamma delta epsilon zeta eta theta", width);
        assert!(
            t.iter().all(|l| crate::cell_text::cells(l) <= width),
            "{t:?}"
        );
        for cont in &t[1..] {
            assert!(cont.starts_with("  "), "{t:?}");
        }
    }
}

#[test]
fn task_marks_are_dim_and_never_links() {
    let d = doc("- [x] done\n- [ ] open\n", 40);
    let t: Vec<String> = d.lines.iter().map(MdLine::text).collect();
    assert_eq!(t, ["\u{2022} [x] done", "\u{2022} [ ] open"]);
    let mark = d.lines[0].spans.iter().find(|s| s.text == "[x] ").unwrap();
    assert_eq!(mark.look, Look::Dim);
    assert!(d
        .lines
        .iter()
        .all(|l| l.spans.iter().all(|s| s.link.is_none())));
    let (_, hits) = paint_doc(&d, 40);
    assert!(hits.is_empty());
}

#[test]
fn quotes_have_a_bar_and_alerts_a_label() {
    let t = texts("> a quote\n", 40);
    assert_eq!(t, ["\u{2502} a quote"]);
    let t = texts("> [!NOTE]\n> text here\n", 40);
    assert_eq!(t, ["\u{2502} Note", "\u{2502} text here"]);
    for width in WIDTHS {
        let t = texts("> one two three four five six seven eight nine ten", width);
        assert!(t.iter().all(|l| l.starts_with('\u{2502}')), "{t:?}");
    }
}

#[test]
fn tables_are_boxed_when_they_fit_else_header_value_lines() {
    let src = "| a | b |\n|---|---|\n| 1 | 2 |\n";
    assert_eq!(
        texts(src, 40),
        [
            "\u{250C}\u{2500}\u{2500}\u{2500}\u{252C}\u{2500}\u{2500}\u{2500}\u{2510}",
            "\u{2502} a \u{2502} b \u{2502}",
            "\u{251C}\u{2500}\u{2500}\u{2500}\u{253C}\u{2500}\u{2500}\u{2500}\u{2524}",
            "\u{2502} 1 \u{2502} 2 \u{2502}",
            "\u{2514}\u{2500}\u{2500}\u{2500}\u{2534}\u{2500}\u{2500}\u{2500}\u{2518}",
        ]
    );
    let wide = "| first column | second column |\n|---|---|\n| aaaa | bbbb |\n";
    assert!(texts(wide, 60)[0].starts_with('\u{250C}'));
    let t = texts(wide, 20);
    assert!(t.iter().all(|l| !l.contains('\u{2502}')), "{t:?}");
    assert_eq!(t[0], "first column: aaaa");
    assert!(t.iter().any(|l| l.starts_with("second column")));
    for width in WIDTHS {
        for l in texts(wide, width) {
            assert!(crate::cell_text::cells(&l) <= width, "{l:?}");
        }
    }
}

#[test]
fn inline_html_follows_the_rules() {
    assert_eq!(texts("a<br>b", 40), ["a", "b"]);
    assert_eq!(
        texts("x <!-- gone --> y", 40),
        ["x  y"].map(|s| s.replace("  ", " "))
    );
    assert_eq!(texts("<b>kept</b> text", 40), ["kept text"]);
    let t = texts(
        "<details>\n<summary>Title</summary>\n\nbody here\n\n</details>\n",
        40,
    );
    assert_eq!(t, ["Title", "", "body here"]);
    let t = texts("<!--\nmulti\nline\n-->\n\nafter", 40);
    assert_eq!(t, ["after"]);
}

#[test]
fn images_are_links_and_footnotes_and_math_plain() {
    let d = doc("![diagram](https://img.test/a.png)", 40);
    assert_eq!(d.lines[0].text(), "[image: diagram]");
    assert_eq!(d.links[0].url, "https://img.test/a.png");
    assert_eq!(texts("![](rel.png)", 40), ["[image]"]);
    assert_eq!(texts("see[^n] $x^2$\n\n[^n]: note", 40)[0], "see[^n] $x^2$");
}

#[test]
fn link_hit_rects_are_the_underlined_cells() {
    let src = "go to https://example.com/some/very/long/path/that/wraps now and [a label with words](https://x.test/y) end";
    for width in WIDTHS {
        let d = doc(src, width);
        let (buf, hits) = paint_doc(&d, width as u16);
        assert!(!hits.is_empty());
        let mut from_hits: BTreeSet<(u16, u16)> = BTreeSet::new();
        for (r, _) in &hits {
            for x in r.x..r.right() {
                from_hits.insert((x, r.y));
            }
        }
        let mut underlined: BTreeSet<(u16, u16)> = BTreeSet::new();
        for y in 0..buf.area.height {
            for x in 0..buf.area.width {
                if buf[(x, y)].modifier.contains(Modifier::UNDERLINED)
                    && buf[(x, y)].symbol() != " "
                {
                    underlined.insert((x, y));
                }
            }
        }
        // Every visible underlined glyph is inside a hit rect and every hit
        // cell is underlined.
        assert!(underlined.is_subset(&from_hits), "width {width}");
        for &(x, y) in &from_hits {
            assert!(
                buf[(x, y)].modifier.contains(Modifier::UNDERLINED),
                "width {width}: ({x},{y}) is a hit but not underlined"
            );
        }
        // The url's text survives the wraps in full.
        let joined: String = (0..buf.area.height).map(|y| row(&buf, y)).collect();
        assert!(
            joined.contains("example.com/some") || width < 20,
            "{joined}"
        );
    }
}

#[test]
fn a_link_target_is_a_web_address_only() {
    let d = doc(
        "[rel](docs/a.md) [js](javascript:alert) [ok](https://ok.test)",
        80,
    );
    assert_eq!(d.links.len(), 1);
    assert_eq!(d.links[0].url, "https://ok.test");
}

#[test]
fn wide_glyphs_are_never_cut() {
    let src = "\u{4e2d}\u{6587}\u{5b57}\u{7b26}\u{6d4b}\u{8bd5}\u{4e2d}\u{6587}\u{5b57}\u{7b26}";
    for width in [9usize, 12, 13, 20] {
        let t = texts(src, width);
        assert_eq!(t.concat(), src, "width {width}");
        let d = doc(src, width);
        let (buf, _) = paint_doc(&d, width as u16);
        let painted: String = (0..buf.area.height).map(|y| row(&buf, y)).collect();
        assert_eq!(painted, src, "width {width}");
    }
}

#[test]
fn emoji_machinery_and_controls_never_reach_a_cell() {
    let src = "heart \u{2764}\u{FE0F} up \u{1F44D}\u{1F3FD} join \u{200D} esc \u{1b}[31mred\u{7} end \u{301}x";
    let s = sanitize(src);
    for bad in ['\u{FE0F}', '\u{200D}', '\u{1F3FD}', '\u{1b}', '\u{7}'] {
        assert!(!s.contains(bad), "{bad:?}");
    }
    assert!(!s.starts_with('\u{301}'));
    for width in WIDTHS {
        let d = doc(src, width);
        let (buf, _) = paint_doc(&d, width as u16);
        for y in 0..buf.area.height {
            for x in 0..buf.area.width {
                let sym = buf[(x, y)].symbol();
                assert!(
                    !sym.chars().any(|c| matches!(
                        c,
                        '\u{FE0F}' | '\u{FE0E}' | '\u{200D}' | '\u{1F3FB}'..='\u{1F3FF}' | '\u{1b}'
                    )),
                    "{sym:?}"
                );
            }
        }
    }
}

#[test]
fn a_narrower_layout_leaves_no_stale_cells() {
    let wide = doc("aaaa bbbb cccc dddd eeee", 40);
    let narrow = doc("zz", 10);
    let mut buf = Buffer::empty(Rect::new(0, 0, 40, 2));
    for y in 0..2 {
        for x in 0..40 {
            buf[(x, y)].set_symbol("X");
        }
    }
    paint(&mut buf, 0, 0, 40, &wide.lines[0], base(), false);
    paint(&mut buf, 0, 0, 40, &narrow.lines[0], base(), false);
    assert_eq!(row(&buf, 0), "zz");
    // The second row was never painted: untouched.
    assert!(row(&buf, 1).starts_with('X'));
}

#[test]
fn code_blocks_are_highlighted_and_on_the_panel() {
    for width in WIDTHS {
        let d = doc("```rust\nfn main() { let value = 42; }\n```\n", width);
        let (buf, _) = paint_doc(&d, width as u16);
        let body = base().fg.unwrap();
        let mut colours: BTreeSet<String> = BTreeSet::new();
        for y in 0..buf.area.height {
            for x in 0..buf.area.width {
                let c = &buf[(x, y)];
                if c.symbol() != " " {
                    assert_eq!(
                        c.bg,
                        th().ui.side_bg,
                        "width {width}: code sits on the panel"
                    );
                    colours.insert(format!("{:?}", c.fg));
                }
            }
        }
        assert!(colours.len() > 1, "width {width}: one colour only");
        assert!(
            colours.iter().any(|c| *c != format!("{body:?}")),
            "width {width}: nothing differs from the body"
        );
    }
}

#[test]
fn code_is_capped_and_never_wrapped_into_the_next_block() {
    let many = format!("```\n{}```\n", "x\n".repeat(CODE_BLOCK_LINES + 25));
    let t = texts(&many, 40);
    assert_eq!(t.len(), CODE_BLOCK_LINES + 1);
    assert_eq!(t.last().unwrap(), "\u{2026} +25");
    let long = format!("```\n{}\n```\n", "y".repeat(30));
    for l in texts(&long, 12) {
        assert!(crate::cell_text::cells(&l) <= 12);
    }
}

#[test]
fn repository_style_references_and_plain_urls_link() {
    let d = doc("see https://a.test/x.", 80);
    assert_eq!(d.links.len(), 1);
    assert_eq!(d.links[0].url, "https://a.test/x");
}

#[test]
fn the_cache_is_keyed_by_width_and_text() {
    let mut cache = DocCache::default();
    let ctx = Context::default();
    let a = cache.get_text("d", "hello world", 40, &ctx);
    let b = cache.get_text("d", "hello world", 40, &ctx);
    assert!(std::rc::Rc::ptr_eq(&a, &b));
    let c = cache.get_text("d", "hello world", 8, &ctx);
    assert!(!std::rc::Rc::ptr_eq(&a, &c));
    let d = cache.get_text("d", "hello there", 40, &ctx);
    assert!(!std::rc::Rc::ptr_eq(&a, &d));
    assert_eq!(cache.len(), 3);
}

#[test]
fn rendering_never_panics_on_hostile_input() {
    for src in [
        "",
        "\n\n\n",
        "[](",
        "| a |\n|---|\n",
        "> > > > > > nested\n",
        "- - - - - - x\n",
        "<a href=\"https://x.test\">y",
        "<",
        "`",
        "```",
        "\u{301}\u{301}",
    ] {
        for width in [0usize, 1, 4, 12] {
            let _ = doc(src, width);
        }
    }
}

/// Budgets of the plan (release): 500 comments of about 1 KB at 100 columns
/// lay out in under 40 ms, a cache hit costs microseconds, painting from a
/// ready layout stays under 2 ms.
#[test]
#[ignore = "release-mode benchmark: cargo test -p gitpoulpi-tui --release md_render_bench -- --ignored"]
fn md_render_bench() {
    let comment = "Thanks for the report, this looks **related** to #12 and to \
https://example.com/docs/page. A few notes:\n\n\
- the `parse` step drops the trailing newline when the input is *empty*\n\
- the second pass [reads the config](https://example.com/config) twice\n\
- tests for the wide-character case are missing\n\n\
> quoted context from the earlier message that wraps over more than one line of the pane\n\n\
Longer paragraph with enough words to wrap a few times at one hundred columns, mentioning \
`Vec<String>`, a path like crates/core/src/lib.rs and some more filler text to reach a \
realistic size for a typical review comment on a pull request or an issue thread. \
Another sentence follows so that the paragraph is long enough to wrap across several rows.\n\n\
| a | b |\n|---|---|\n| 1 | 2 |\n";
    let ctx = Context::default();
    let comments: Vec<String> = (0..500)
        .map(|n| format!("{comment}\nnumber {n}\n"))
        .collect();
    // The grammar compile of the first fence is paid once per process.
    let _ = render("```rust\nfn a() {}\n```", 100, &ctx);
    // Best of five: a loaded machine only ever makes a run slower.
    let mut layout = std::time::Duration::MAX;
    let mut docs: Vec<Doc> = Vec::new();
    for _ in 0..5 {
        let start = std::time::Instant::now();
        docs = comments.iter().map(|c| render(c, 100, &ctx)).collect();
        layout = layout.min(start.elapsed());
    }
    let mut cache = DocCache::default();
    let _ = cache.get_text("bench", &comments[0], 100, &ctx);
    let start = std::time::Instant::now();
    let _ = cache.get_text("bench", &comments[0], 100, &ctx);
    let hit = start.elapsed();
    let mut buf = Buffer::empty(Rect::new(0, 0, 100, 50));
    let start = std::time::Instant::now();
    for (y, line) in docs
        .iter()
        .flat_map(|d| d.lines.iter())
        .take(50)
        .enumerate()
    {
        paint(&mut buf, 0, y as u16, 100, line, base(), false);
    }
    let painted = start.elapsed();
    eprintln!("layout 500 x 1 KB: {layout:?}, cache hit: {hit:?}, paint 50 lines: {painted:?}");
    assert!(layout.as_millis() < 40, "layout {layout:?}");
    assert!(hit.as_micros() < 500, "cache hit {hit:?}");
    assert!(painted.as_micros() < 2000, "paint {painted:?}");
}

#[test]
fn alert_labels_follow_the_language() {
    let src = "> [!TIP]\n> text\n";
    assert_eq!(texts(src, 40)[0], "\u{2502} Tip");
    let fr = gitpoulpi_i18n::with_lang(gitpoulpi_i18n::Lang::Fr, || texts(src, 40));
    assert_eq!(fr[0], "\u{2502} Astuce");
}

fn host_context() -> Context {
    Context {
        xrefs: gitpoulpi_hosting::xrefs::XrefContext {
            family: Some(gitpoulpi_config::ApiFamily::Github),
            web_base: "https://github.com".into(),
            project: "o/r".into(),
            ..Default::default()
        },
        ..Default::default()
    }
}

#[test]
fn host_references_become_links_through_xrefs() {
    let d = render(
        "fixes #12, cc @mara, see o/other#3 and 0123abc\n\n`#99 in code`",
        80,
        &host_context(),
    );
    let urls: Vec<&str> = d.links.iter().map(|l| l.url.as_str()).collect();
    assert!(
        urls.contains(&"https://github.com/o/r/issues/12"),
        "{urls:?}"
    );
    assert!(urls.contains(&"https://github.com/mara"), "{urls:?}");
    assert!(
        urls.contains(&"https://github.com/o/other/issues/3"),
        "{urls:?}"
    );
    assert!(
        urls.contains(&"https://github.com/o/r/commit/0123abc"),
        "{urls:?}"
    );
    assert!(
        !urls.iter().any(|u| u.contains("/99")),
        "code is never linked: {urls:?}"
    );
    // The hit rect is the underlined cells of `#12`.
    let (buf, hits) = paint_doc(&d, 80);
    let id = d
        .links
        .iter()
        .position(|l| l.url.ends_with("/issues/12"))
        .unwrap();
    let rect = hits.iter().find(|(_, i)| *i == id).unwrap().0;
    assert_eq!(rect.width, 3);
    let shown: String = row(&buf, rect.y)
        .chars()
        .skip(rect.x as usize)
        .take(3)
        .collect();
    assert_eq!(shown, "#12");
}

#[test]
fn without_a_host_a_bare_number_is_not_a_link() {
    let d = doc("fixes #12 and https://example.com/x", 80);
    let urls: Vec<&str> = d.links.iter().map(|l| l.url.as_str()).collect();
    assert_eq!(urls, ["https://example.com/x"]);
}

#[test]
fn merge_requests_are_linked_with_a_bang() {
    let ctx = Context {
        xrefs: gitpoulpi_hosting::xrefs::XrefContext {
            family: Some(gitpoulpi_config::ApiFamily::Gitlab),
            web_base: "https://gl.test".into(),
            project: "g/p".into(),
            ..Default::default()
        },
        ..Default::default()
    };
    let d = render("merged by !7", 80, &ctx);
    assert_eq!(d.links.len(), 1, "{:?}", d.links);
    assert!(
        d.links[0].url.ends_with("/g/p/-/merge_requests/7"),
        "{:?}",
        d.links
    );
}

const DIAGRAM: &str = "before\n\n```mermaid\ngraph TD\n  A --> B\n```\n\nafter";

fn diagram_context() -> Context {
    Context {
        diagrams: true,
        ..Default::default()
    }
}

#[cfg(feature = "mermaid")]
#[test]
fn a_mermaid_block_gets_an_open_diagram_button_that_carries_its_script() {
    let d = render(DIAGRAM, 60, &diagram_context());
    let lines: Vec<String> = d.lines.iter().map(MdLine::text).collect();
    let button = lines
        .iter()
        .position(|l| l.contains("[ Open diagram ]"))
        .unwrap_or_else(|| panic!("a button: {lines:?}"));
    let code = lines.iter().position(|l| l.contains("A --> B")).unwrap();
    assert!(button > code, "under the block: {lines:?}");
    assert!(lines[button + 1..].iter().any(|l| l == "after"));
    let diagrams: Vec<&str> = d
        .links
        .iter()
        .filter_map(|l| l.diagram.as_deref())
        .collect();
    assert_eq!(diagrams, ["graph TD\n  A --> B"]);
    // The button is a link span with a hit rect.
    let (_, hits) = paint_doc(&d, 60);
    assert!(hits.iter().any(|(_, id)| d.links[*id].diagram.is_some()));
}

#[test]
fn other_blocks_and_callers_that_cannot_open_a_diagram_get_no_button() {
    // The caller did not ask (the composer's preview).
    let d = render(DIAGRAM, 60, &Context::default());
    assert!(d.lines.iter().all(|l| !l.text().contains("Open diagram")));
    assert!(d.links.iter().all(|l| l.diagram.is_none()));
    // Not a mermaid block.
    let d = render("```rust\nlet a = 1;\n```", 60, &diagram_context());
    assert!(d.links.is_empty());
}
