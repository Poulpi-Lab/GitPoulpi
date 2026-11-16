use super::*;
use crate::dialog_kit;
use crate::keymap::chord::Key;
use crate::th;

// -- drawing ------------------------------------------------------------

/// One draw of the view: its lines, one per row of `area`, and every
/// clickable rect, registered inner-first (a row's buttons before the
/// row band that covers them - `hit` returns the first match).
pub struct ManagerRender {
    pub lines: Vec<Line<'static>>,
    pub hits: Vec<(Rect, ManagerHit)>,
}

impl ManagerRender {
    pub fn hit(&self, col: u16, row: u16) -> Option<ManagerHit> {
        self.hits
            .iter()
            .find(|(r, _)| col >= r.x && col < r.x + r.width && row >= r.y && row < r.y + r.height)
            .map(|(_, h)| *h)
    }
}

pub const STAR_ON: &str = "\u{2605}";
pub const STAR_OFF: &str = "\u{2606}";
/// The row's `more` button. U+22EE is a mathematical symbol: text
/// presentation, one column, in the fonts a Windows terminal falls back
/// to - the project's glyph rule.
pub const MORE: &str = "\u{22EE}";

/// Rows above the list: title, blank, buttons, search, blank.
const CHROME_TOP: u16 = 5;
/// Rows below it: the footer.
const CHROME_BOTTOM: u16 = 1;

/// Builds one line left to right, recording where each piece landed.
pub(super) struct Cursor {
    spans: Vec<Span<'static>>,
    pub(super) x: u16,
    y: u16,
    pub(super) end: u16,
}

impl Cursor {
    pub(super) fn new(area: Rect, y: u16) -> Self {
        Self {
            spans: Vec::new(),
            x: area.x,
            y,
            end: area.x + area.width,
        }
    }

    pub(super) fn room(&self) -> usize {
        self.end.saturating_sub(self.x) as usize
    }

    /// Pushes `text` (truncated to what fits) and returns its rect.
    pub(super) fn push(&mut self, text: &str, style: Style) -> Rect {
        let text = fit(text, self.room());
        let width = display_width(&text) as u16;
        let rect = Rect::new(self.x, self.y, width, 1);
        if width > 0 {
            self.spans.push(Span::styled(text, style));
        }
        self.x += width;
        rect
    }

    pub(super) fn pad_to(&mut self, x: u16, style: Style) {
        if x > self.x {
            let gap = " ".repeat((x.min(self.end) - self.x) as usize);
            self.push(&gap, style);
        }
    }

    pub(super) fn line(self) -> Line<'static> {
        Line::from(self.spans)
    }
}

pub(super) fn button_style(hovered: bool) -> Style {
    Style::default()
        .fg(th().ui.on_button)
        .bg(button_bg(hovered))
}

/// Column widths for the repository rows, measured once per draw over
/// every repository (not only the visible ones), so the columns do not
/// jump as the list scrolls.
#[derive(Debug, Clone, Copy)]
struct Columns {
    name: usize,
    owner: usize,
    branch: usize,
}

fn measure(manager: &RepoManager, room: usize) -> Columns {
    let dup = duplicate_names(&manager.repos);
    let widest = |f: &dyn Fn(&KnownRepo) -> String, cap: usize| {
        manager
            .repos
            .iter()
            .map(|r| display_width(&f(r)))
            .max()
            .unwrap_or(0)
            .clamp(4, cap)
    };
    let mut cols = Columns {
        name: widest(&|r| r.name.clone(), 32),
        owner: widest(&|r| r.column_text(dup.contains(&r.name.to_lowercase())), 36),
        branch: widest(&|r| branch_text(r).0, 28),
    };
    // Narrow windows give up owner first, then squeeze the branch: the
    // name is what the user is scanning for.
    let fixed = 4 + 2 + 2 + 2 + 18;
    while cols.name + cols.owner + cols.branch + fixed > room && cols.owner > 0 {
        cols.owner = cols.owner.saturating_sub(4);
        if cols.owner < 4 {
            cols.owner = 0;
        }
    }
    while cols.name + cols.owner + cols.branch + fixed > room && cols.branch > 6 {
        cols.branch -= 1;
    }
    cols
}

/// The branch cell's text, and whether it is a placeholder (dimmed).
fn branch_text(repo: &KnownRepo) -> (String, bool) {
    if !repo.exists {
        return (msg::home::GONE_FROM_DISK.get().into(), true);
    }
    if !repo.is_repo {
        return (msg::home::NOT_A_REPOSITORY.get().into(), true);
    }
    match (&repo.facts, &repo.branch_hint) {
        (Some(f), _) => (
            format!(
                "\u{2387} {}",
                f.branch
                    .clone()
                    .unwrap_or_else(|| msg::home::UNBORN.get().into())
            ),
            false,
        ),
        (None, Some(hint)) => (format!("\u{2387} {hint}"), true),
        (None, None) => ("\u{2026}".into(), true),
    }
}

/// Renders the view into `area`. Takes `&mut` only to keep the selection
/// scrolled into view and to remember how tall the list was.
pub fn render(manager: &mut RepoManager, area: Rect) -> ManagerRender {
    let mut lines: Vec<Line<'static>> = Vec::new();
    let mut hits: Vec<(Rect, ManagerHit)> = Vec::new();
    let hover = manager.hover;
    let width = area.width as usize;

    // Title.
    let mut c = Cursor::new(area, area.y);
    c.push(
        &format!(
            " {}",
            if manager.new_tab {
                msg::tab_bar::NEW_TAB.get()
            } else if manager.page == HomePage::Workspaces {
                msg::workspaces::PAGE_WORKSPACES.get()
            } else {
                msg::home::TITLE.get()
            }
        ),
        Style::default()
            .fg(th().ui.text)
            .add_modifier(Modifier::BOLD),
    );
    lines.push(c.line());
    if manager.page == HomePage::Workspaces {
        super::workspaces::render(manager, area, &mut lines, &mut hits);
        lines.push(super::workspaces::footer(manager, width));
        lines.truncate(area.height as usize);
        return ManagerRender { lines, hits };
    }
    lines.push(Line::default());

    // Buttons: Browse / Clone / Init on the left (Integrations will join
    // them), Collapse all / Expand all on the right.
    let y = area.y + 2;
    let mut c = Cursor::new(area, y);
    c.push(" ", Style::default());
    for button in [
        ManagerButton::Browse,
        ManagerButton::Clone,
        ManagerButton::Init,
    ] {
        let hovered = hover == Some(ManagerHit::Button(button));
        let key = button.key().to_string();
        let text = if key.is_empty() {
            format!(" {} ", button.label())
        } else {
            format!(" {} ({key}) ", button.label())
        };
        let rect = c.push(&text, button_style(hovered));
        hits.push((rect, ManagerHit::Button(button)));
        c.push(" ", Style::default());
    }
    let right_width: usize = [ManagerButton::CollapseAll, ManagerButton::ExpandAll]
        .iter()
        .map(|b| display_width(b.label()) + 2 + 1)
        .sum();
    if c.room() > right_width + 1 {
        c.pad_to(area.x + area.width - right_width as u16, Style::default());
        for button in [ManagerButton::CollapseAll, ManagerButton::ExpandAll] {
            let hovered = hover == Some(ManagerHit::Button(button));
            let rect = c.push(&format!(" {} ", button.label()), button_style(hovered));
            hits.push((rect, ManagerHit::Button(button)));
            c.push(" ", Style::default());
        }
    }
    lines.push(c.line());

    // Search.
    let y = area.y + 3;
    let mut c = Cursor::new(area, y);
    let focused = manager.search_focused;
    let label = " / ";
    c.push(
        label,
        Style::default().fg(if focused {
            th().ui.accent
        } else {
            th().ui.muted
        }),
    );
    let field_width = width.saturating_sub(display_width(label) + 2).min(60);
    let field_style = Style::default()
        .fg(if manager.query.is_empty() && !focused {
            th().ui.muted
        } else {
            th().ui.text
        })
        .bg(crate::input_bg(focused, hover == Some(ManagerHit::Search)));
    let start = c.x;
    manager.sync_search();
    manager.search_edit.begin_frame();
    manager
        .search_edit
        .drawn((), Rect::new(start, y, field_width as u16, 1), false);
    if manager.query.is_empty() && !focused {
        let prompt = msg::home::search_prompt(crate::keymap::paren(
            &crate::keymap::current(),
            crate::keymap::Action::HomeSearch,
        ));
        c.push(&prompt, field_style);
        c.push(
            &" ".repeat(field_width.saturating_sub(display_width(&prompt))),
            field_style,
        );
    } else {
        let look =
            text_field::EditLook::new(field_style, "_").caret_style(field_style.fg(th().ui.accent));
        for span in text_field::edit_line(
            &manager.query,
            manager.search_edit.caret(()),
            field_width,
            focused,
            look,
        )
        .spans
        {
            c.push(&span.content, span.style);
        }
    }
    hits.push((
        Rect::new(start, y, c.x.saturating_sub(start), 1),
        ManagerHit::Search,
    ));
    lines.push(c.line());
    lines.push(Line::default());

    // The list.
    let list_top = area.y + CHROME_TOP;
    let list_height = area.height.saturating_sub(CHROME_TOP + CHROME_BOTTOM) as usize;
    manager.list_height = list_height.max(1);
    let rows = manager.rows();
    if manager.follow && list_height > 0 {
        if manager.selected < manager.scroll {
            manager.scroll = manager.selected;
        } else if manager.selected >= manager.scroll + list_height {
            manager.scroll = manager.selected + 1 - list_height;
        }
    }
    manager.scroll = manager
        .scroll
        .min(rows.len().saturating_sub(list_height.max(1)));
    // A list longer than its room keeps the last column for the scrollbar.
    manager.bar = crate::scrollbar::Track::column(
        Rect::new(
            (area.x + area.width).saturating_sub(1),
            list_top,
            1,
            list_height as u16,
        ),
        rows.len(),
        list_height,
        manager.scroll,
    );
    let bar = manager.bar.is_some();
    let list_area = Rect::new(area.x, area.y, area.width - u16::from(bar), area.height);
    let list_w = list_area.width as usize;
    let cols = measure(manager, list_w);

    for (offset, row) in rows
        .iter()
        .enumerate()
        .skip(manager.scroll)
        .take(list_height)
    {
        let y = list_top + (offset - manager.scroll) as u16;
        // The row `Enter` opens is shown selected from the first draw
        // band from the first draw.
        let selected = offset == manager.selected && !manager.search_focused;
        let line = match *row {
            ManagerRow::Header(section) => header_line(manager, section, list_area, y, &mut hits),
            ManagerRow::Empty(section) => {
                let mut c = Cursor::new(list_area, y);
                c.push(
                    &format!("      {}", section.empty_text()),
                    Style::default()
                        .fg(th().ui.muted)
                        .add_modifier(Modifier::ITALIC),
                );
                c.line()
            }
            ManagerRow::Repo { section, repo } => repo_line(
                manager, section, repo, offset, cols, list_area, y, &mut hits,
            ),
        };
        let line = if selected {
            highlight(line, list_w, th().ui.selection)
        } else if matches_row(hover, offset, *row) {
            highlight(line, list_w, th().ui.hover)
        } else {
            line
        };
        lines.push(line);
    }
    while lines.len() < (CHROME_TOP as usize + list_height) {
        lines.push(Line::default());
    }

    // Footer: the last message, or the keys.
    let footer = match manager.mode_hint.as_ref().or(manager.message.as_ref()) {
        Some(message) => Line::from(Span::styled(
            fit(&format!(" {message}"), width),
            Style::default().fg(th().status.warning),
        )),
        None => {
            let km = crate::keymap::current();
            use crate::keymap::Action as Bind;
            let hints: Vec<(String, &'static str)> = if manager.search_focused {
                vec![
                    (
                        msg::home::HINT_TYPE.get().into(),
                        msg::home::HINT_FILTER.get(),
                    ),
                    (
                        dialog_kit::key_name(Key::Enter),
                        msg::home::HINT_OPEN_BEST.get(),
                    ),
                    (
                        dialog_kit::key_name(Key::Esc),
                        msg::home::HINT_CLEAR_SEARCH.get(),
                    ),
                ]
            } else {
                let mut hints = vec![(
                    dialog_kit::key_name(Key::Enter),
                    msg::dialog::HINT_OPEN.get(),
                )];
                for (action, what) in [
                    (Bind::HomeFavorite, msg::home::HINT_FAVORITE.get()),
                    (Bind::HomeCloseRepo, msg::home::HINT_CLOSE_REMOVE.get()),
                    (Bind::GlobalContextMenu, msg::home::HINT_MENU.get()),
                    (Bind::HomeSearch, msg::home::HINT_SEARCH.get()),
                ] {
                    // An unbound action has no key to advertise.
                    let shown = km.shown(action);
                    if !shown.is_empty() {
                        hints.push((shown, what));
                    }
                }
                hints.push((dialog_kit::key_name(Key::Space), msg::home::HINT_FOLD.get()));
                hints.push((dialog_kit::key_name(Key::Esc), msg::home::HINT_BACK.get()));
                let quit = km.shown(Bind::GlobalQuit);
                if !quit.is_empty() {
                    hints.push((quit, msg::home::HINT_QUIT.get()));
                }
                hints
            };
            let mut spans = vec![Span::raw(" ")];
            for (key, what) in hints {
                spans.push(Span::styled(
                    key.to_string(),
                    Style::default().fg(th().ui.accent),
                ));
                spans.push(Span::styled(
                    format!(" {what}   "),
                    Style::default().fg(th().ui.muted),
                ));
            }
            Line::from(spans)
        }
    };
    lines.push(footer);

    lines.truncate(area.height as usize);
    ManagerRender { lines, hits }
}

fn matches_row(hover: Option<ManagerHit>, offset: usize, row: ManagerRow) -> bool {
    match (hover, row) {
        (Some(ManagerHit::Row(i)), _) => i == offset,
        (Some(ManagerHit::Header(s)), ManagerRow::Header(r)) => s == r,
        _ => false,
    }
}

/// Paints `bg` behind every span that has none of its own, and pads the
/// line to the full width - a band that stops at the end of the text
/// reads as a torn row.
pub(super) fn highlight(line: Line<'static>, width: usize, bg: Color) -> Line<'static> {
    let used = line.width();
    let mut spans: Vec<Span<'static>> = line
        .spans
        .into_iter()
        .map(|s| {
            if s.style.bg.is_none() {
                let style = s.style.bg(bg);
                s.style(style)
            } else {
                s
            }
        })
        .collect();
    if used < width {
        spans.push(Span::styled(
            " ".repeat(width - used),
            Style::default().bg(bg),
        ));
    }
    Line::from(spans)
}

fn header_line(
    manager: &RepoManager,
    section: Section,
    area: Rect,
    y: u16,
    hits: &mut Vec<(Rect, ManagerHit)>,
) -> Line<'static> {
    let collapsed = manager.collapsed.contains(&section);
    let count = manager.section_repos(section).len();
    let mut c = Cursor::new(area, y);
    let start = c.x;
    c.push(
        &format!(" {} ", crate::fold_mark::glyph(collapsed)),
        Style::default().fg(th().ui.accent),
    );
    c.push(
        &section.title().to_uppercase(),
        Style::default()
            .fg(th().ui.text)
            .add_modifier(Modifier::BOLD),
    );
    c.push(&format!("  {count}"), Style::default().fg(th().ui.muted));

    // The section's own button, right-aligned, registered before the
    // header band that covers it. Only offered when it has something to
    // act on.
    let has_any = match section {
        Section::Open => manager.repos.iter().any(|r| r.tab.is_some()),
        Section::Recent => manager.repos.iter().any(|r| r.recent.is_some()),
        _ => false,
    };
    if let (Some(label), true) = (section.action_label(), has_any) {
        // Clipped, not dropped, when the row is short.
        let label = fit(label, c.room().saturating_sub(4).max(3));
        let text = format!(" {label} ");
        let w = display_width(&text) as u16;
        if c.room() > w as usize + 2 {
            c.pad_to(area.x + area.width - w - 1, Style::default());
            let hovered = manager.hover == Some(ManagerHit::SectionAction(section));
            let rect = c.push(&text, button_style(hovered));
            hits.push((rect, ManagerHit::SectionAction(section)));
        }
    }
    hits.push((
        Rect::new(start, y, area.width, 1),
        ManagerHit::Header(section),
    ));
    c.line()
}

#[allow(clippy::too_many_arguments)]
fn repo_line(
    manager: &RepoManager,
    section: Section,
    repo: usize,
    offset: usize,
    cols: Columns,
    area: Rect,
    y: u16,
    hits: &mut Vec<(Rect, ManagerHit)>,
) -> Line<'static> {
    let known = &manager.repos[repo];
    let dim = !known.available();
    let text_fg = if dim { th().ui.muted } else { th().ui.text };
    let hover = manager.hover;

    // The actions go on the right; measure them first so the columns
    // know how much room they have.
    let close_text = format!(" {} ", msg::home::ROW_CLOSE.get());
    let remove_text = format!(" {} ", msg::home::ROW_REMOVE.get());
    let switch_text = format!(" {} ", msg::home::ROW_SWITCH.get());
    let open_text = format!(" {} ", msg::home::ROW_OPEN.get());
    let close_label = match section {
        Section::Open => Some(close_text.as_str()),
        Section::Recent | Section::All => Some(remove_text.as_str()),
        Section::Favorites if !known.exists => Some(remove_text.as_str()),
        Section::Favorites => None,
    };
    let open_label = if known.tab.is_some() {
        switch_text.as_str()
    } else {
        open_text.as_str()
    };
    // The buttons degrade as the row narrows: all of them, then without the
    // Close / Remove button, then the menu button alone. What a dropped button
    // did stays in that menu (Open, Close the tab, Remove from the list), and
    // Enter / double click still open the row.
    let width_of = |open: bool, close: bool| {
        (if open {
            display_width(open_label) + 1
        } else {
            0
        }) + if close {
            close_label.map(|l| display_width(l) + 1).unwrap_or(0)
        } else {
            0
        } + display_width(MORE)
            + 2
            + 1
    };
    // The gutter and star, a name worth reading, and the gap after it.
    let min_content = 6 + 10 + 2;
    let room = area.width as usize;
    let (show_open, show_close) = if room >= min_content + width_of(true, true) {
        (true, true)
    } else if room >= min_content + width_of(true, false) {
        (true, false)
    } else {
        (false, false)
    };
    let close_label = close_label.filter(|_| show_close);
    let actions_width = width_of(show_open, show_close);

    let mut c = Cursor::new(area, y);
    let start = c.x;
    // The gutter stays blank on the selected row: the band marks it, and an
    // arrow here was mistaken for a section's fold arrow.
    c.push("  ", Style::default());
    c.push(" ", Style::default());
    let star_hovered = hover == Some(ManagerHit::Star(offset));
    let star = c.push(
        if known.favorite.is_some() {
            STAR_ON
        } else {
            STAR_OFF
        },
        Style::default()
            .fg(if known.favorite.is_some() || star_hovered {
                th().status.warning
            } else {
                th().ui.muted
            })
            .add_modifier(if star_hovered {
                Modifier::BOLD
            } else {
                Modifier::empty()
            }),
    );
    c.push(" ", Style::default());

    let content_end = (area.x + area.width).saturating_sub(actions_width as u16);
    let cell = |text: &str, w: usize| format!("{:<w$}  ", fit(text, w), w = w);
    let name_style = Style::default()
        .fg(text_fg)
        .add_modifier(if known.tab.is_some() {
            Modifier::BOLD
        } else {
            Modifier::empty()
        });
    c.end = content_end;
    c.push(&cell(&known.name, cols.name), name_style);
    if cols.owner > 0 {
        c.push(
            &cell(
                &known.column_text(
                    duplicate_names(&manager.repos).contains(&known.name.to_lowercase()),
                ),
                cols.owner,
            ),
            Style::default().fg(th().ui.muted),
        );
    }
    let (branch, placeholder) = branch_text(known);
    c.push(
        &cell(&branch, cols.branch),
        Style::default().fg(if placeholder {
            th().ui.muted
        } else {
            th().refs.local
        }),
    );
    match &known.facts {
        Some(f) => {
            if f.ahead > 0 {
                c.push(
                    &format!("\u{2191}{} ", f.ahead),
                    Style::default().fg(th().ui.accent),
                );
            }
            if f.behind > 0 {
                c.push(
                    &format!("\u{2193}{} ", f.behind),
                    Style::default().fg(th().ui.accent),
                );
            }
            if f.modified > 0 {
                c.push(
                    &format!("\u{270E} {} ", f.modified),
                    Style::default().fg(th().status.warning),
                );
            }
            if f.added > 0 {
                c.push(
                    &format!("+ {} ", f.added),
                    Style::default().fg(th().refs.local),
                );
            }
        }
        None if known.available() => {
            c.push("\u{2026}", Style::default().fg(th().ui.muted));
        }
        None => {}
    }
    // The path takes whatever the columns left, so it never pushes the
    // name, owner or branch out; too little room and it is simply absent.
    if let Some(path) = path_cell(known, c.room()) {
        c.push(&path, Style::default().fg(th().ui.muted));
    }
    c.end = area.x + area.width;

    // Actions, right-aligned, registered before the row band.
    c.pad_to(content_end, Style::default());
    let mut action = |c: &mut Cursor, label: &str, hit: ManagerHit| {
        let hovered = hover == Some(hit);
        let rect = c.push(label, button_style(hovered));
        hits.push((rect, hit));
        c.push(" ", Style::default());
    };
    if show_open {
        if known.available() {
            action(&mut c, open_label, ManagerHit::RowOpen(offset));
        } else {
            c.push(&" ".repeat(display_width(open_label) + 1), Style::default());
        }
    }
    if let Some(label) = close_label {
        action(&mut c, label, ManagerHit::RowClose(offset));
    }
    action(&mut c, &format!(" {MORE} "), ManagerHit::RowMenu(offset));

    hits.push((star, ManagerHit::Star(offset)));
    hits.push((Rect::new(start, y, area.width, 1), ManagerHit::Row(offset)));
    c.line()
}

/// Least cells worth showing a path in: two spaces and a few characters.
const MIN_PATH_CELLS: usize = 14;

/// The path cell: two spaces, then the path shortened with `~` and cut in
/// the middle to `room` cells. `None` without a path or without room.
fn path_cell(known: &KnownRepo, room: usize) -> Option<String> {
    if known.path.as_os_str().is_empty() || room < MIN_PATH_CELLS {
        return None;
    }
    let home = gitpoulpi_config::home_dir();
    let shown = home_shortened(&known.path, home.as_deref());
    Some(format!("  {}", mid_fit(&shown, room - 2)))
}

/// `path` with a leading `home` replaced by `~`.
pub(super) fn home_shortened(path: &Path, home: Option<&Path>) -> String {
    if let Some(rest) = home.and_then(|h| path.strip_prefix(h).ok()) {
        let rest = rest.to_string_lossy();
        return if rest.is_empty() {
            "~".to_string()
        } else {
            let sep = if cfg!(windows) { '\\' } else { '/' };
            format!("~{sep}{rest}")
        };
    }
    path.to_string_lossy().into_owned()
}

/// `text` cut in the middle with an ellipsis to fit `width` cells, keeping
/// the start (the root) and the end (the folder's own name).
pub(super) fn mid_fit(text: &str, width: usize) -> String {
    if display_width(text) <= width {
        return text.to_string();
    }
    if width <= 1 {
        return "\u{2026}".repeat(width);
    }
    let budget = width - 1;
    let head = budget / 3;
    let tail = budget - head;
    let chars: Vec<char> = text.chars().collect();
    let mut start = String::new();
    let mut used = 0;
    for &ch in &chars {
        let w = display_width(&ch.to_string());
        if used + w > head {
            break;
        }
        start.push(ch);
        used += w;
    }
    let mut end: Vec<char> = Vec::new();
    let mut used = 0;
    for &ch in chars.iter().rev() {
        let w = display_width(&ch.to_string());
        if used + w > tail {
            break;
        }
        end.push(ch);
        used += w;
    }
    end.reverse();
    format!("{start}\u{2026}{}", end.into_iter().collect::<String>())
}
