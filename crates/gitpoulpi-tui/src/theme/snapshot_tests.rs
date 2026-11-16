//! Golden snapshots, one per preset (plan 3, test 4): a 120x32 showcase of
//! the toolbar, the Settings Appearance page, a context menu, Log rows with
//! the graph, and a diff hunk, dumped as text plus per-row style runs printed
//! as role names (`ui.accent/ui.panel`), so a preset diff reads as meaning
//! rather than hex. Update with `INSTA_UPDATE=always cargo test -p
//! gitpoulpi-tui --lib theme::snapshot_tests`.

// The goldens only run in a full build (see the test below); the helpers
// are then unused elsewhere.
#![cfg_attr(
    not(all(
        feature = "lfs",
        feature = "hosting",
        feature = "ai",
        feature = "mcp",
        feature = "worktree",
        feature = "submodule"
    )),
    allow(dead_code)
)]

use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Terminal;

use super::roles::Theme;
use super::{override_theme, th};
use crate::ui_kit::ClippedRender as _;

const W: u16 = 120;
const H: u16 = 32;

/// The roles a color stands for in `theme`, joined with `+` (declaration
/// order); an unknown color prints as hex.
fn names(roles: &[(String, Color)], c: Color) -> String {
    if c == Color::Reset {
        return "-".into();
    }
    let hit: Vec<&str> = roles
        .iter()
        .filter(|(_, rc)| *rc == c)
        .map(|(n, _)| n.as_str())
        .collect();
    if !hit.is_empty() {
        return hit.join("+");
    }
    match c {
        Color::Rgb(r, g, b) => format!("#{r:02x}{g:02x}{b:02x}"),
        other => format!("{other:?}"),
    }
}

fn dump(theme: &Theme, buf: &Buffer) -> String {
    let roles = theme.roles();
    let mut out = String::new();
    for y in 0..buf.area.height {
        let line: String = (0..buf.area.width)
            .map(|x| buf[(x, y)].symbol().to_string())
            .collect();
        out.push_str(&format!("{y:02} |{}|\n", line.trim_end()));
    }
    out.push('\n');
    for y in 0..buf.area.height {
        let mut x = 0;
        while x < buf.area.width {
            let cell = &buf[(x, y)];
            let key = (cell.fg, cell.bg, cell.modifier);
            let mut end = x + 1;
            while end < buf.area.width {
                let c = &buf[(end, y)];
                if (c.fg, c.bg, c.modifier) != key {
                    break;
                }
                end += 1;
            }
            // Blank cells with the default style are the bulk of a frame.
            if key != (Color::Reset, Color::Reset, Modifier::empty()) {
                let m = if key.2.is_empty() {
                    String::new()
                } else {
                    format!(" {:?}", key.2)
                };
                out.push_str(&format!(
                    "{y:02} {x:03}-{:03} {}/{}{m}\n",
                    end - 1,
                    names(&roles, key.0),
                    names(&roles, key.1)
                ));
            }
            x = end;
        }
    }
    out
}

fn showcase(buf_theme: &Theme) -> Buffer {
    use crate::chrome::{toolbar_render_with, ToolbarEnabled};
    use gitpoulpi_core::domain::{DiffLine, FileDiff, FileStatus, Hunk, HunkId, LineOrigin};
    let _g = override_theme(buf_theme);
    let mut terminal = Terminal::new(TestBackend::new(W, H)).unwrap();
    let dir = tempfile::tempdir().unwrap();
    let mut prefs = crate::preferences::tests::state_at(dir.path(), "Appearance");
    terminal
        .draw(|frame| {
            let t = th();
            // What `draw_frame` does first: a theme that paints its own
            // background fills the frame.
            if t.ui.bg != Color::Reset {
                frame.render_widget(
                    ratatui::widgets::Block::default().style(Style::default().bg(t.ui.bg)),
                    frame.area(),
                );
            }
            // Toolbar.
            let bar = Rect::new(0, 0, W, 1);
            let tb = toolbar_render_with(
                bar,
                None,
                None,
                ToolbarEnabled::ALL,
                crate::chrome::ToolbarPlacement::LEGACY,
            );
            frame.render_clipped(Paragraph::new(tb.line.clone()), bar);
            // Settings, Appearance page, on the left.
            prefs.render(frame, Rect::new(0, 1, 62, 22));
            // Log rows with the graph, top right.
            let nodes = vec![
                crate::test_support::commit(1, &[2]),
                crate::test_support::commit(2, &[3]),
                crate::test_support::commit(3, &[]),
            ];
            let refs = vec![crate::test_support::local_branch_at(
                "main",
                crate::test_support::oid(1),
            )];
            let graph = crate::graph::build_graph_lines(&nodes, &refs);
            let log = Rect::new(63, 1, 57, 5);
            let mut lines: Vec<Line> = vec![Line::from(Span::styled(
                "Graph  Message                  Author   Date",
                Style::default()
                    .fg(t.ui.header_fg)
                    .add_modifier(Modifier::BOLD),
            ))];
            for (i, (cell, _)) in graph.rows.iter().enumerate() {
                let mut spans = cell.spans.clone();
                spans.push(Span::styled(
                    format!(" commit {}            ", i + 1),
                    Style::default().fg(t.ui.text),
                ));
                spans.push(Span::styled("a   ", Style::default().fg(t.ui.muted)));
                spans.push(Span::styled("2d ago", Style::default().fg(t.ui.muted)));
                let mut line = Line::from(spans);
                if i == 1 {
                    line = line.style(Style::default().bg(t.ui.selection));
                }
                lines.push(line);
            }
            frame.render_clipped(Paragraph::new(lines), log);
            // A diff hunk, right.
            let line = |origin, s: &str| DiffLine {
                origin,
                content: s.to_string(),
                staged: false,
            };
            let file = FileDiff {
                renamed_from: None,
                similarity: None,
                path: "src/lib.rs".into(),
                status: FileStatus::Modified,
                hunks: vec![Hunk {
                    id: HunkId(0),
                    old_start: 10,
                    old_lines: 3,
                    new_start: 10,
                    new_lines: 3,
                    lines: vec![
                        line(LineOrigin::Context, "fn main() {"),
                        line(LineOrigin::Delete, "    let x = 1;"),
                        line(LineOrigin::Add, "    let x = 2;"),
                        line(LineOrigin::Context, "}"),
                    ],
                }],
            };
            let diff = crate::diff_view::render_diff(
                &file,
                crate::diff_view::DiffViewMode::Inline,
                false,
                false,
                57,
            );
            frame.render_clipped(Paragraph::new(diff.lines.clone()), Rect::new(63, 7, 57, 8));
            // An open context menu over the lower right.
            let ctx = crate::log_menu::MenuContext {
                current_branch: Some("main".into()),
                ..crate::log_menu::MenuContext::default()
            };
            let menu = crate::context_menu::Menu::new(
                "a1b2c3d",
                crate::log_menu::commit_menu_entries(&ctx, false),
                80,
                15,
            );
            drop(menu.render(frame, Some(1)));
        })
        .unwrap();
    terminal.backend().buffer().clone()
}

/// The presets imported from terminalcolors.com that have a golden: one
/// dark and one light per distinct palette style. The ~90 others are still
/// rendered (a panic fails) and pass the contrast test, but a 35 KB golden
/// each would bury the directory.
const IMPORTED_GOLDENS: &[&str] = &[
    "ayu-mirage",
    "github-dark-dimmed",
    "gruvbox-light-hard-contrast",
    "nightfox",
    "zenbones-zenwritten-light",
];

/// The showcase draws the whole window (the LFS button, the Integrations,
/// AI and MCP pages...), so the goldens are those of a full build.
#[cfg(all(
    feature = "lfs",
    feature = "hosting",
    feature = "ai",
    feature = "mcp",
    feature = "worktree",
    feature = "submodule"
))]
#[test]
fn every_preset_has_a_golden_showcase() {
    for spec in gitpoulpi_config::theme::presets() {
        let theme = Theme::from(&spec);
        let buf = showcase(&theme);
        if gitpoulpi_config::theme::is_imported(&spec.slug)
            && !IMPORTED_GOLDENS.contains(&spec.slug.as_str())
        {
            continue;
        }
        insta::assert_snapshot!(spec.slug.clone(), dump(&theme, &buf));
    }
}

#[test]
fn the_dump_names_roles_and_flags_unknown_colors() {
    let dark = Theme::dark();
    let roles = dark.roles();
    let accent = names(&roles, dark.ui.accent);
    assert!(accent.starts_with("ui.accent"), "{accent}");
    assert!(accent.contains("+status.info"), "{accent}");
    assert_eq!(names(&roles, Color::Reset), "-");
    assert_eq!(names(&roles, Color::Rgb(1, 2, 3)), "#010203");
}
