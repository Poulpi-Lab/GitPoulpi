//! The About window: version, build features, license and links, with the
//! octopus logo at its left when the build can draw pictures.
//!
//! Opened by the 🐙 in the title bar (a click) or the palette's
//! `Help: About GitPoulpi` (the keyboard twin). Modal while open: Esc, Enter
//! or a click outside it closes it, and nothing behind it hovers. The two
//! links open in the browser on a click, or with `s` (source) and `d` (docs). Process-wide state, like the
//! other small popups, because the window restarts on every tab switch.
//!
//! The logo is the 16x16 sprite of `crates/gitpoulpi-logo-build/colourways.json` (`logo_colors`),
//! drawn through the terminal's pixel protocol when there is one and as
//! half-blocks otherwise. A click on it spins it (a coin flip over about half
//! a second, driven by the event loop's frame timer) and lands on the next
//! colorway; `Space` does the same, a right click or `c` opens the color
//! list (a combobox), and the choice is saved as `ui.logo_color`.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::Frame;

use crate::combo::{Combo, ComboOption, ComboOutcome};
use crate::dialog_kit;
use crate::logo_colors;
use crate::{display_width, th};
use gitpoulpi_i18n::msg::about as msg;

/// The project's address, from the workspace manifest.
pub(crate) const REPOSITORY_URL: &str = env!("CARGO_PKG_REPOSITORY");
/// The user documentation's address.
pub(crate) const DOCS_URL: &str = concat!(env!("CARGO_PKG_REPOSITORY"), "/tree/main/docs/user");
const LICENSE: &str = env!("CARGO_PKG_LICENSE");
/// The license text on GitHub.
const LICENSE_URL: &str = concat!(env!("CARGO_PKG_REPOSITORY"), "/blob/main/LICENSE");
/// `name version url license` per line: the external crates the build uses directly
/// (`build_crates.rs`).
const CRATES: &str = include_str!(concat!(env!("OUT_DIR"), "/third_party_crates.txt"));
const VERSION: &str = env!("CARGO_PKG_VERSION");
/// The third-party license notices and credits (`THIRD-PARTY-LICENSES.txt`),
/// shown by the Licenses view.
const NOTICES: &str = gitpoulpi_core::licenses::THIRD_PARTY;

/// Logo box, in cells: square-ish on any font (half-blocks draw 16x16 pixels
/// in 16x8 cells, and the pixel path sizes its columns from the cell shape).
const LOGO_ROWS: u16 = 8;
const LOGO_COLS: u16 = 16;

static OPEN: AtomicBool = AtomicBool::new(false);

/// How long one spin takes.
const SPIN: Duration = Duration::from_millis(800);
/// Tentacle waves during one spin.
const WAVES: f32 = 2.0;
/// The text column's width: wide enough for the links on one line each,
/// never narrower than this.
const TEXT_MIN: u16 = 62;
/// Width of a link's `[k] ` access-key prefix.
const KEY_W: u16 = 4;

struct Spin {
    started: Instant,
    from: usize,
    to: usize,
}

struct State {
    /// The colorway shown (index into `logo_colors::ways()`); `None` until
    /// config installed one.
    way: Option<usize>,
    spin: Option<Spin>,
    picker: Option<Combo>,
    /// Where the logo was painted last frame: the click target.
    logo: Rect,
    /// The whole window, last frame: a click outside it closes it.
    window: Rect,
    /// The ✕ in its top border, last frame.
    close_button: Rect,
    /// Where the pointer was last seen (hover highlight of the ✕).
    pointer: Option<(u16, u16)>,
    /// Each painted link row and what a click on it does.
    links: Vec<(Rect, Target)>,
    /// The crate list shows instead of the facts, scrolled to `crates_offset`.
    crates_view: bool,
    crates_offset: usize,
    /// The crate row Enter opens (kept inside the visible rows when painted).
    crates_sel: usize,
    /// The crate list's rows, last frame (wheel target, page size).
    crates_area: Rect,
    /// The crate list's bordered stand-in the bar is drawn over, last frame.
    crates_body: Rect,
    /// Where on the thumb the pointer holds it while the bar is dragged.
    crates_grab: Option<i32>,
    /// Why the last link did not open, shown above the hint.
    notice: Option<String>,
    /// The Licenses view (the notices text, scrollable) shows instead of the
    /// facts, scrolled to `lic_offset`.
    lic_view: bool,
    lic_offset: usize,
    /// The notices' rows, last frame (wheel target, page size).
    lic_area: Rect,
    /// The notices' bordered stand-in the bar is drawn over, last frame.
    lic_body: Rect,
    lic_grab: Option<i32>,
}

static STATE: Mutex<State> = Mutex::new(State {
    way: None,
    spin: None,
    picker: None,
    logo: Rect::new(0, 0, 0, 0),
    window: Rect::new(0, 0, 0, 0),
    close_button: Rect::new(0, 0, 0, 0),
    pointer: None,
    links: Vec::new(),
    crates_view: false,
    crates_offset: 0,
    crates_sel: 0,
    crates_area: Rect::new(0, 0, 0, 0),
    crates_body: Rect::new(0, 0, 0, 0),
    crates_grab: None,
    notice: None,
    lic_view: false,
    lic_offset: 0,
    lic_area: Rect::new(0, 0, 0, 0),
    lic_body: Rect::new(0, 0, 0, 0),
    lic_grab: None,
});

/// What a click on a link row does.
#[derive(Clone, Copy, PartialEq, Debug)]
enum Target {
    Url(&'static str),
    /// Shows the third-party crate list.
    Crates,
    /// Shows the licenses and credits text.
    Licenses,
}

/// A crate line `name version url license` as its four columns (the license
/// is an SPDX expression and has spaces of its own; the url is `-` when the
/// crate has no link).
fn split_crate(line: &str) -> (&str, &str, &str, &str) {
    let (name, rest) = line.split_once(' ').unwrap_or((line, ""));
    let (version, rest) = rest.split_once(' ').unwrap_or((rest, ""));
    let (url, license) = rest.split_once(' ').unwrap_or((rest, "?"));
    // Semver build metadata (`1.2.1+spec-1.1.0`) only widens the column.
    let version = version.split_once('+').map_or(version, |(v, _)| v);
    (
        name,
        version,
        if url.is_empty() { "-" } else { url },
        license,
    )
}

/// A crate's web address without its scheme, as the list shows it.
fn short_url(url: &str) -> &str {
    url.split_once("://").map_or(url, |(_, rest)| rest)
}

/// Moves the crate row Enter opens by `delta`, scrolling it into view.
fn move_crate_sel(delta: isize) {
    with(|s| {
        let last = crates().count().saturating_sub(1);
        let page = usize::from(s.crates_area.height).max(1);
        let from = s
            .crates_sel
            .clamp(s.crates_offset, s.crates_offset + page - 1);
        s.crates_sel = from.saturating_add_signed(delta).min(last);
        if s.crates_sel < s.crates_offset {
            s.crates_offset = s.crates_sel;
        } else if s.crates_sel >= s.crates_offset + page {
            s.crates_offset = s.crates_sel + 1 - page;
        }
    });
}

/// Opens the selected crate's page (a crate without a link does nothing).
fn open_selected_crate() {
    let url = with(|s| crates().nth(s.crates_sel).map(|l| split_crate(l).2));
    if let Some(url) = url.filter(|u| *u != "-") {
        open_link(url);
    }
}

fn crates() -> impl Iterator<Item = &'static str> {
    CRATES.lines().filter(|l| !l.is_empty())
}

/// Scrolls the crate list by `delta` rows, within its length.
fn scroll_crates(delta: isize) {
    with(|s| {
        let page = usize::from(s.crates_area.height).max(1);
        let max = crates().count().saturating_sub(page);
        s.crates_offset = s.crates_offset.saturating_add_signed(delta).min(max);
    });
}

/// The crate list's bar as the shared list-bar kit sees it.
fn crates_region(s: &State) -> crate::ui_kit::ListBarRegion {
    crate::ui_kit::ListBarRegion {
        bar: crate::ui_kit::ListBar::AboutCrates,
        area: s.crates_body,
        len: crates().count(),
        visible: usize::from(s.crates_area.height).max(1),
    }
}

/// The left button on the crate list's bar (thumb grab, track jump, arrow
/// step - all the shared kit's), `true` when it was the bar's.
fn press_crates_bar(at: (u16, u16)) -> bool {
    with(|s| {
        let region = crates_region(s);
        let Some(press) = crate::ui_kit::list_bar_press(region, s.crates_offset, at.0, at.1) else {
            return false;
        };
        if let Some(offset) = press.jump_to {
            s.crates_offset = offset;
        }
        s.crates_grab = (!press.arrow).then_some(press.grab);
        true
    })
}

/// The pointer moved with the thumb held: follow it, even off the track.
fn drag_crates_bar(row: u16) {
    with(|s| {
        if let Some(grab) = s.crates_grab {
            s.crates_offset =
                crate::ui_kit::list_bar_drag(crates_region(s), s.crates_offset, grab, row);
        }
    });
}

fn toggle_crates() {
    with(|s| {
        s.crates_view = !s.crates_view;
        s.crates_offset = 0;
        s.crates_sel = 0;
        s.crates_grab = None;
    });
}

fn notice_lines() -> &'static [&'static str] {
    static LINES: std::sync::OnceLock<Vec<&'static str>> = std::sync::OnceLock::new();
    LINES.get_or_init(|| NOTICES.lines().collect())
}

fn toggle_licenses() {
    with(|s| {
        s.lic_view = !s.lic_view;
        s.lic_offset = 0;
        s.lic_grab = None;
    });
}

/// Scrolls the licenses text by `delta` rows, within its length.
fn scroll_licenses(delta: isize) {
    with(|s| {
        let page = usize::from(s.lic_area.height).max(1);
        let max = notice_lines().len().saturating_sub(page);
        s.lic_offset = s.lic_offset.saturating_add_signed(delta).min(max);
    });
}

/// The licenses text's bar as the shared list-bar kit sees it.
fn licenses_region(s: &State) -> crate::ui_kit::ListBarRegion {
    crate::ui_kit::ListBarRegion {
        bar: crate::ui_kit::ListBar::AboutCrates,
        area: s.lic_body,
        len: notice_lines().len(),
        visible: usize::from(s.lic_area.height).max(1),
    }
}

fn press_licenses_bar(at: (u16, u16)) -> bool {
    with(|s| {
        let region = licenses_region(s);
        let Some(press) = crate::ui_kit::list_bar_press(region, s.lic_offset, at.0, at.1) else {
            return false;
        };
        if let Some(offset) = press.jump_to {
            s.lic_offset = offset;
        }
        s.lic_grab = (!press.arrow).then_some(press.grab);
        true
    })
}

fn drag_licenses_bar(row: u16) {
    with(|s| {
        if let Some(grab) = s.lic_grab {
            s.lic_offset =
                crate::ui_kit::list_bar_drag(licenses_region(s), s.lic_offset, grab, row);
        }
    });
}

/// Opens the window on the Licenses view (the palette's twin of `t`).
pub(crate) fn open_licenses() {
    open();
    with(|s| {
        s.lic_view = true;
        s.lic_offset = 0;
        s.crates_view = false;
    });
}

fn with<R>(f: impl FnOnce(&mut State) -> R) -> R {
    f(&mut STATE.lock().unwrap_or_else(|e| e.into_inner()))
}

/// Publishes the colour this installation wears (`launcher_sync::effective_color`:
/// its own entry, `ui.logo_color` only for the primary), at startup and on
/// every live settings change. Another installation's colour never arrives here.
pub(crate) fn install(slug: &str) {
    let idx = logo_colors::index_of(slug);
    with(|s| {
        if s.spin.is_none() {
            s.way = Some(idx);
        }
    });
}

pub(crate) fn open() {
    // Bars painted behind the window must not light under a stale pointer.
    crate::scrollbar::set_pointer(None);
    OPEN.store(true, Ordering::Relaxed);
}

pub(crate) fn close() {
    OPEN.store(false, Ordering::Relaxed);
    with(|s| {
        s.picker = None;
        s.notice = None;
        s.crates_view = false;
        s.crates_grab = None;
        s.lic_view = false;
        s.lic_grab = None;
        s.pointer = None;
    });
}

pub(crate) fn is_open() -> bool {
    OPEN.load(Ordering::Relaxed)
}

/// Whether a click at the point does something: the pointing hand. The cross,
/// a link, the logo (it spins) and a row of the crate list.
pub(crate) fn pointer_hand(col: u16, row: u16) -> bool {
    let at = (col, row).into();
    with(|s| {
        s.close_button.contains(at)
            || s.logo.contains(at)
            || s.links.iter().any(|(r, _)| r.contains(at))
            || (s.crates_view && s.crates_area.contains(at))
    })
}

/// The logo is mid-spin: the loop keeps its frame rate.
pub(crate) fn animating() -> bool {
    is_open() && with(|s| s.spin.is_some())
}

fn current(s: &State) -> usize {
    s.way
        .unwrap_or_else(|| logo_colors::index_of(logo_colors::default_slug()))
}

/// The colorway the title bar's octopus wears: the chosen one, else the
/// build's default.
pub(crate) fn way_index() -> usize {
    with(|s| current(s))
}

/// Starts the spin towards the next colorway (no-op while one runs).
fn start_spin() {
    with(|s| {
        if s.spin.is_none() {
            let from = current(s);
            let to = (from + 1) % logo_colors::ways().len();
            s.spin = Some(Spin {
                started: Instant::now(),
                from,
                to,
            });
        }
    });
}

/// Saves the colorway in `config.toml` (the file every Settings edit uses):
/// in this installation's entry, or in `ui.logo_color` when it is the primary
/// (the key old builds read). Never another installation's colour.
fn persist(idx: usize) {
    let slug = &logo_colors::ways()[idx].slug;
    if let Some(path) = gitpoulpi_config::Settings::resolve_ambient()
        .ok()
        .and_then(|s| s.file().map(std::path::Path::to_path_buf))
    {
        let cur = gitpoulpi_config::installation::current();
        let primary = crate::launcher_sync::effective().primary;
        let _ = write_colour(&path, &cur.identity.id, primary, &cur.config, slug);
    }
}

/// Writes `slug` as the colour of installation `id` into the file at `path`.
/// `seeded` supplies the entry when the file has none yet (a pinned id keeps
/// its seeded entry in memory only).
fn write_colour(
    path: &std::path::Path,
    id: &str,
    primary: bool,
    seeded: &gitpoulpi_config::Config,
    slug: &str,
) -> gitpoulpi_config::Result<()> {
    if primary {
        return gitpoulpi_config::set_in_file(path, "ui.logo_color", slug).map(drop);
    }
    let base = seeded
        .installation
        .settings
        .get(id)
        .cloned()
        .unwrap_or_default();
    gitpoulpi_config::set_installation_in_file(path, |i| {
        i.settings.entry(id.to_string()).or_insert(base).logo_color = slug.to_string();
    })
}

/// Picks colorway `idx` outright and saves it.
fn choose(idx: usize) {
    with(|s| {
        s.way = Some(idx);
        s.spin = None;
    });
    persist(idx);
}

/// The color list; a build with one colorway has nothing to list, so it
/// stays closed (the spin still plays, in the same color).
fn open_picker() {
    if logo_colors::ways().len() < 2 {
        return;
    }
    with(|s| {
        // Opened like a context menu: one click picks.
        let mut combo = Combo::new("").with_click_picks();
        combo.set_options(
            logo_colors::ways()
                .iter()
                .map(|w| ComboOption::new(logo_colors::name(&w.slug), w.slug.clone()))
                .collect(),
        );
        combo.select(Some(logo_colors::ways()[current(s)].slug.clone()));
        combo.open();
        s.picker = Some(combo);
    });
}

fn settle(outcome: ComboOutcome) {
    if let ComboOutcome::Picked(slug) | ComboOutcome::Custom(slug) = outcome {
        choose(logo_colors::index_of(&slug));
    }
    with(|s| {
        if s.picker.as_ref().is_some_and(|c| !c.is_open()) {
            s.picker = None;
        }
    });
}

/// A key while the window is open. The color list takes every key while it
/// is open; otherwise `Esc`, `Enter` and `q` close, `Space` spins the logo,
/// `c` opens the color list and the rest is swallowed.
pub(crate) fn on_key(key: crossterm::event::KeyEvent) {
    use crossterm::event::KeyCode;
    if with(|s| s.picker.is_some()) {
        let outcome = with(|s| s.picker.as_mut().map(|c| c.on_key(key)));
        if let Some(outcome) = outcome {
            settle(outcome);
        }
        return;
    }
    if with(|s| s.lic_view) {
        let page = with(|s| s.lic_area.height.max(1)) as isize;
        match key.code {
            KeyCode::Esc | KeyCode::Char('t') => toggle_licenses(),
            KeyCode::Up | KeyCode::Char('k') => scroll_licenses(-1),
            KeyCode::Down | KeyCode::Char('j') => scroll_licenses(1),
            KeyCode::PageUp => scroll_licenses(-page),
            KeyCode::PageDown | KeyCode::Char(' ') => scroll_licenses(page),
            KeyCode::Home => scroll_licenses(isize::MIN / 2),
            KeyCode::End => scroll_licenses(isize::MAX / 2),
            KeyCode::Char('q') => close(),
            _ => {}
        }
        return;
    }
    if with(|s| s.crates_view) {
        let page = with(|s| s.crates_area.height.max(1)) as isize;
        match key.code {
            KeyCode::Esc | KeyCode::Char('l') => toggle_crates(),
            KeyCode::Up | KeyCode::Char('k') => move_crate_sel(-1),
            KeyCode::Down | KeyCode::Char('j') => move_crate_sel(1),
            KeyCode::PageUp => move_crate_sel(-page),
            KeyCode::PageDown => move_crate_sel(page),
            KeyCode::Home => move_crate_sel(isize::MIN / 2),
            KeyCode::End => move_crate_sel(isize::MAX / 2),
            KeyCode::Enter => open_selected_crate(),
            KeyCode::Char('q') => close(),
            _ => {}
        }
        return;
    }
    match key.code {
        KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q') => close(),
        KeyCode::Char('l') => toggle_crates(),
        KeyCode::Char('t') => toggle_licenses(),
        KeyCode::Char('i') => open_link(LICENSE_URL),
        KeyCode::Char(' ') => start_spin(),
        KeyCode::Char('c') => open_picker(),
        KeyCode::Char('s') => open_link(REPOSITORY_URL),
        KeyCode::Char('d') => open_link(DOCS_URL),
        _ => {}
    }
}

/// Opens `url` in the browser; a failure is shown in the window.
fn open_link(url: &'static str) {
    if let Err(why) = crate::repo_menu::open_url(url) {
        crate::activity_log::report_problem(
            crate::activity_log::ProblemSource::Tool,
            &gitpoulpi_i18n::msg::help_page::open_docs_failed(url, why),
        );
    }
    with(|s| s.notice = None);
}

/// A mouse event while the window is open. A click on the logo spins it, a
/// right click on it opens the color list, a click on a link opens it, a
/// click outside the window closes it (and, with the list open, only closes
/// the list).
pub(crate) fn on_mouse(mouse: crossterm::event::MouseEvent) {
    use crossterm::event::{MouseButton, MouseEventKind};
    with(|s| s.pointer = Some((mouse.column, mouse.row)));
    if with(|s| s.picker.is_some()) {
        let outcome = with(|s| s.picker.as_mut().map(|c| c.on_mouse(mouse)));
        if let Some(outcome) = outcome {
            settle(outcome);
        }
        return;
    }
    let at = (mouse.column, mouse.row).into();
    if with(|s| s.lic_view) {
        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left)
                if press_licenses_bar((mouse.column, mouse.row)) =>
            {
                return;
            }
            MouseEventKind::Drag(MouseButton::Left) if with(|s| s.lic_grab.is_some()) => {
                drag_licenses_bar(mouse.row);
                return;
            }
            MouseEventKind::Up(MouseButton::Left) => with(|s| s.lic_grab = None),
            _ => {}
        }
    }
    if with(|s| s.crates_view) {
        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left)
                if press_crates_bar((mouse.column, mouse.row)) =>
            {
                return;
            }
            MouseEventKind::Drag(MouseButton::Left) if with(|s| s.crates_grab.is_some()) => {
                drag_crates_bar(mouse.row);
                return;
            }
            MouseEventKind::Up(MouseButton::Left) => with(|s| s.crates_grab = None),
            _ => {}
        }
    }
    match mouse.kind {
        MouseEventKind::ScrollUp | MouseEventKind::ScrollDown
            if with(|s| s.lic_view && s.window.contains(at)) =>
        {
            scroll_licenses(if mouse.kind == MouseEventKind::ScrollUp {
                -3
            } else {
                3
            });
            return;
        }
        MouseEventKind::ScrollUp | MouseEventKind::ScrollDown
            if with(|s| s.crates_view && s.window.contains(at)) =>
        {
            scroll_crates(if mouse.kind == MouseEventKind::ScrollUp {
                -3
            } else {
                3
            });
            return;
        }
        _ => {}
    }
    let MouseEventKind::Down(button) = mouse.kind else {
        return;
    };
    if button_is_left(&mouse) && with(|s| s.close_button.contains(at)) {
        close();
        return;
    }
    if button_is_left(&mouse) {
        // A click in the crate list also selects its row (Enter then repeats it).
        with(|s| {
            if s.crates_view && s.crates_area.contains(at) {
                let row = usize::from(mouse.row - s.crates_area.y);
                s.crates_sel = (s.crates_offset + row).min(crates().count().saturating_sub(1));
            }
        });
    }
    let (on_logo, inside, link) = with(|s| {
        (
            s.logo.contains(at),
            s.window.contains(at),
            s.links
                .iter()
                .find(|(r, _)| r.contains(at))
                .map(|(_, u)| *u),
        )
    });
    match (button, link) {
        (MouseButton::Left, Some(Target::Url(url))) => open_link(url),
        (MouseButton::Left, Some(Target::Crates)) => toggle_crates(),
        (MouseButton::Left, Some(Target::Licenses)) => toggle_licenses(),
        (MouseButton::Left, None) if on_logo => start_spin(),
        (MouseButton::Right, _) if on_logo => open_picker(),
        _ if !inside => close(),
        _ => {}
    }
}

fn button_is_left(mouse: &crossterm::event::MouseEvent) -> bool {
    mouse.kind == crossterm::event::MouseEventKind::Down(crossterm::event::MouseButton::Left)
}

/// The compiled-in optional features, space separated.
fn features() -> String {
    let on: Vec<&str> = crate::build_features::FLAGS
        .iter()
        .filter(|(_, on)| *on)
        .map(|(name, _)| *name)
        .collect();
    if on.is_empty() {
        "-".to_string()
    } else {
        on.join(" ")
    }
}

/// How a text line is painted.
#[derive(Clone, Copy, PartialEq, Debug)]
enum Look {
    Title,
    Plain,
    /// Small print (the trademark notice).
    Muted,
    /// A link (a click follows it); the access key is shown on the line that
    /// starts it.
    Link(Target, Option<char>),
}

/// The text lines beside the logo, wrapped to `width`; a link too long for
/// it breaks anywhere rather than being cut.
fn lines(width: usize) -> Vec<(String, Look)> {
    let mut out = vec![
        ("GitPoulpi".to_string(), Look::Title),
        (msg::TAGLINE.get().to_string(), Look::Plain),
        (String::new(), Look::Plain),
        (msg::version(VERSION), Look::Plain),
        (
            format!("[i] {}", msg::license(LICENSE)),
            Look::Link(Target::Url(LICENSE_URL), Some('i')),
        ),
    ];
    for text in wrap(&msg::features(features()), width) {
        out.push((text, Look::Plain));
    }
    for (key, text, url) in [
        ('s', msg::repository(REPOSITORY_URL), REPOSITORY_URL),
        ('d', msg::docs(DOCS_URL), DOCS_URL),
    ] {
        for (n, piece) in wrap_hard(&format!("[{key}] {text}"), width)
            .into_iter()
            .enumerate()
        {
            out.push((piece, Look::Link(Target::Url(url), (n == 0).then_some(key))));
        }
    }
    out.push((
        format!("[l] {}", msg::crates(crates().count())),
        Look::Link(Target::Crates, Some('l')),
    ));
    out.push((
        format!("[t] {}", msg::LICENSES.get()),
        Look::Link(Target::Licenses, Some('t')),
    ));
    out.push((String::new(), Look::Plain));
    let notice = format!("{} {}", msg::INSPIRATION.get(), msg::TRADEMARK.get());
    for text in wrap(&notice, width) {
        out.push((text, Look::Muted));
    }
    out
}

/// The width the text column wants: the widest link line, at least `TEXT_MIN`.
fn wanted_width() -> u16 {
    [msg::repository(REPOSITORY_URL), msg::docs(DOCS_URL)]
        .iter()
        .map(|t| display_width(t) as u16 + KEY_W)
        .max()
        .unwrap_or(0)
        .max(TEXT_MIN)
}

/// Cuts `text` into pieces of at most `width` columns, anywhere.
fn wrap_hard(text: &str, width: usize) -> Vec<String> {
    let mut out = vec![String::new()];
    for ch in text.chars() {
        let last = out.last_mut().expect("never empty");
        if !last.is_empty() && display_width(last) + display_width(&ch.to_string()) > width {
            out.push(String::new());
        }
        out.last_mut().expect("never empty").push(ch);
    }
    out
}

/// Greedy word wrap on spaces, by display width.
fn wrap(text: &str, width: usize) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut line = String::new();
    for word in text.split(' ') {
        let fits = line.is_empty() || display_width(&line) + 1 + display_width(word) <= width;
        if !fits {
            out.push(std::mem::take(&mut line));
        }
        if !line.is_empty() {
            line.push(' ');
        }
        line.push_str(word);
    }
    if !line.is_empty() {
        out.push(line);
    }
    out
}

/// What this frame of the spin shows: the colorway, the horizontal scale,
/// whether the sprite is mirrored and the tentacles' sway (0 at rest). A finished spin lands (and is saved).
fn frame_state() -> (usize, f32, bool, f32) {
    let landed = with(|s| {
        let done = s
            .spin
            .as_ref()
            .is_some_and(|sp| sp.started.elapsed() >= SPIN);
        if done {
            let to = s.spin.take().map(|sp| sp.to);
            if let Some(to) = to {
                s.way = Some(to);
            }
            to
        } else {
            None
        }
    });
    if let Some(to) = landed {
        persist(to);
    }
    with(|s| match &s.spin {
        Some(sp) => {
            let t = (sp.started.elapsed().as_secs_f32() / SPIN.as_secs_f32()).clamp(0.0, 1.0);
            let scale = (std::f32::consts::PI * t).cos().abs();
            let wiggle = (t * WAVES).fract().max(f32::EPSILON);
            if t < 0.5 {
                (sp.from, scale, false, wiggle)
            } else {
                (sp.to, scale, true, wiggle)
            }
        }
        None => (current(s), 1.0, false, 0.0),
    })
}

/// Which way the octopus looks at a pointer at `pointer` (a cell), seen from
/// its eyes, which sit in the logo `logo` at `logo_colors::EYES` (the middle
/// of the pair, as fractions of the logo's width and height): left of the
/// eyes' column it looks left, right of it right, above their row up, below
/// it down, and straight ahead on them (the cell or two the eyes cover). The
/// two axes are independent, so a pointer up and to the left is (-1, -1).
/// `(0, 0)` when the pointer is unknown.
fn look_toward(pointer: Option<(u16, u16)>, logo: Rect) -> logo_colors::Look {
    let Some((px, py)) = pointer else {
        return (0, 0);
    };
    if logo.width == 0 || logo.height == 0 {
        return (0, 0);
    }
    // Everything in half cells: a cell's middle is `2 * n + 1`.
    let (fx, fy) = logo_colors::EYES;
    let ex = i32::from(logo.x) * 2 + (f32::from(logo.width) * 2.0 * fx).round() as i32;
    let ey = i32::from(logo.y) * 2 + (f32::from(logo.height) * 2.0 * fy).round() as i32;
    let step = |d: i32| if d.abs() <= 1 { 0 } else { d.signum() as i8 };
    (
        step(i32::from(px) * 2 + 1 - ex),
        step(i32::from(py) * 2 + 1 - ey),
    )
}

/// The window moves (its size follows its text).
const WIN: dialog_kit::Win = dialog_kit::Win::movable("about");

/// Paints the window over `frame`. Call it with the other popups.
pub(crate) fn render(frame: &mut Frame) {
    if !is_open() {
        return;
    }
    let area = frame.area();
    // The Licenses view is a wide reading window without the logo.
    let licenses_view = with(|s| s.lic_view);
    let with_logo = !licenses_view;
    let logo_w = if with_logo { LOGO_COLS + 3 } else { 0 };
    let room = area.width.saturating_sub(logo_w + 4).max(20);
    let text_w = if licenses_view {
        area.width.saturating_sub(8).clamp(30, 100)
    } else {
        wanted_width().min(room)
    };
    let inner_w = logo_w + text_w;
    let text = if licenses_view {
        Vec::new()
    } else {
        lines(usize::from(text_w))
    };
    let notice = with(|s| s.notice.clone());
    let rows = if licenses_view {
        area.height.saturating_sub(8).max(8)
    } else {
        text.len() as u16 + u16::from(notice.is_some())
    };
    let inner_h = rows.max(if with_logo { LOGO_ROWS } else { 0 }) + 2;
    let rect = WIN.place(area, dialog_kit::centered(area, inner_w + 4, inner_h + 2));
    let buf = frame.buffer_mut();
    dialog_kit::panel(buf, rect, msg::TITLE.get());
    // The window's close control, where every window has it.
    // A picker open over the window owns the pointer: nothing under it hovers.
    let hovered = with(|s| {
        s.picker.is_none()
            && s.pointer.is_some_and(|(x, y)| {
                Rect::new(rect.right().saturating_sub(5), rect.y, 3, 1).contains((x, y).into())
            })
    });
    let close_button = crate::chrome::close_cross::paint(buf, rect, hovered).unwrap_or_default();
    with(|s| s.close_button = close_button);
    WIN.decorate(buf, rect, Some(close_button), with(|s| s.picker.is_none()));
    let (x0, y0) = (rect.x + 2, rect.y + 1);
    if with_logo {
        let area = Rect::new(x0, y0 + 1, LOGO_COLS, LOGO_ROWS);
        let (way, scale, mirrored, wiggle) = frame_state();
        // The eyes follow the pointer, except mid-spin (they stay centered) and
        // while a picker owns the pointer.
        let look = with(|s| {
            if s.spin.is_some() || s.picker.is_some() {
                (0, 0)
            } else {
                look_toward(s.pointer, area)
            }
        });
        // While the color list is open the logo is half-blocks, never a
        // picture: a picture re-sent beside a freshly painted list is what
        // tore it under ConPTY (see `logo::render`).
        let picture = with(|s| s.picker.is_none());
        logo::render(
            buf,
            area,
            &logo_colors::bitmap(&logo_colors::ways()[way], scale, mirrored, wiggle, look),
            picture,
        );
        with(|s| s.logo = area);
    }
    let text_x = x0 + logo_w;
    let max = rect.right().saturating_sub(text_x + 2);
    let mut links = Vec::new();
    let crates_view = with(|s| s.crates_view);
    if !with_logo {
        with(|s| s.logo = Rect::new(0, 0, 0, 0));
    }
    if licenses_view {
        let (back, scrollbar_rows) = paint_licenses(buf, text_x, y0 + 1, max, rows);
        links.extend(scrollbar_rows);
        links.push((back, Target::Licenses));
    } else if crates_view {
        let (back, rows) = paint_crates(buf, text_x, y0 + 1, max, text.len() as u16);
        // The rows first: nothing registered later may sit over them.
        links.extend(rows);
        links.push((back, Target::Crates));
    } else {
        for (i, (line, look)) in text.iter().enumerate() {
            let y = y0 + 1 + i as u16;
            let base = Style::default().bg(th().ui.panel);
            let style = match look {
                Look::Title => base.fg(th().ui.accent).add_modifier(Modifier::BOLD),
                Look::Plain => base.fg(th().ui.text),
                Look::Muted => base.fg(th().ui.muted),
                Look::Link(target, key) => {
                    let w = (display_width(line) as u16).min(max);
                    links.push((Rect::new(text_x, y, w, 1), *target));
                    if key.is_some() {
                        // The access key: bold and not underlined, then the label.
                        let key_style = base.fg(th().ui.accent).add_modifier(Modifier::BOLD);
                        dialog_kit::put(
                            buf,
                            text_x,
                            y,
                            &line[..KEY_W as usize - 1],
                            max,
                            key_style,
                        );
                        let rest = &line[KEY_W as usize..];
                        let x = text_x + KEY_W;
                        let link = base.fg(th().ui.accent).add_modifier(Modifier::UNDERLINED);
                        dialog_kit::put(buf, x, y, rest, max.saturating_sub(KEY_W), link);
                        continue;
                    }
                    base.fg(th().ui.accent).add_modifier(Modifier::UNDERLINED)
                }
            };
            dialog_kit::put(buf, text_x, y, line, max, style);
        }
    }
    if let Some(notice) = &notice {
        let y = y0 + 1 + text.len() as u16;
        dialog_kit::put(
            buf,
            text_x,
            y,
            notice,
            max,
            Style::default().fg(th().status.danger).bg(th().ui.panel),
        );
    }
    with(|s| {
        s.window = rect;
        s.links = links;
    });
    let hint = if licenses_view {
        msg::LICENSES_HINT.get()
    } else if crates_view {
        msg::CRATES_HINT.get()
    } else {
        msg::CLOSE_HINT.get()
    };
    let hint_x = rect.right().saturating_sub(display_width(hint) as u16 + 2);
    dialog_kit::put(
        buf,
        hint_x,
        rect.bottom().saturating_sub(2),
        hint,
        display_width(hint) as u16,
        Style::default().fg(th().ui.muted).bg(th().ui.panel),
    );
    // Last, so the window's text never paints over the open list.
    with(|s| {
        let logo = s.logo;
        if let Some(combo) = s.picker.as_mut() {
            combo.set_anchor(logo);
            combo.render_popup(buf, buf.area);
        }
    });
}

/// The licenses and credits text in the text column: a title (the mouse way
/// back, lit under the pointer), then `rows - 1` lines from `lic_offset`, with
/// a scrollbar when it does not fit. Returns the title's rect and no row
/// links (the text has none).
fn paint_licenses(
    buf: &mut Buffer,
    x: u16,
    y: u16,
    width: u16,
    rows: u16,
) -> (Rect, Vec<(Rect, Target)>) {
    let all = notice_lines();
    let title = format!("\u{2039} {}", msg::LICENSES_TITLE.get());
    let back = Rect::new(x, y, (display_width(&title) as u16).min(width), 1);
    let hovered = with(|s| s.pointer.is_some_and(|(c, r)| back.contains((c, r).into())));
    let title_style = Style::default()
        .fg(th().ui.accent)
        .bg(th().ui.panel)
        .add_modifier(Modifier::BOLD);
    let title_style = if hovered {
        title_style.patch(crate::theme::plain_button_style(true))
    } else {
        title_style
    };
    dialog_kit::put(buf, x, y, &title, width, title_style);
    let area = Rect::new(x, y + 1, width, rows.saturating_sub(1));
    let page = usize::from(area.height);
    let body = Rect::new(x, area.y.saturating_sub(1), width, area.height + 2);
    let offset = with(|s| {
        s.lic_area = area;
        s.lic_body = body;
        s.lic_offset = s.lic_offset.min(all.len().saturating_sub(page));
        s.lic_offset
    });
    let style = Style::default().fg(th().ui.text).bg(th().ui.panel);
    let row_w = width.saturating_sub(1);
    for (i, line) in all.iter().enumerate().skip(offset).take(page) {
        let y = area.y + (i - offset) as u16;
        dialog_kit::put(buf, x, y, &" ".repeat(usize::from(row_w)), row_w, style);
        dialog_kit::put(buf, x, y, line, row_w, style);
    }
    if let Some(track) = crate::scrollbar::Track::vertical(body, all.len(), offset) {
        let pointer = with(|s| s.pointer);
        crate::scrollbar::with_pointer(pointer, || {
            crate::scrollbar::render_buf(buf, track, true, false)
        });
    }
    (back, Vec::new())
}

/// The crate list in the text column: a title, then `rows - 1` rows from
/// `crates_offset`, with a scrollbar when it does not fit. The title is the
/// mouse way back to the facts ("‹ Third-party crates (n)", lit under the
/// pointer); its rect is returned for the window's links, with the rect of
/// every painted row that has a link (a click on it opens the crate's page).
fn paint_crates(
    buf: &mut Buffer,
    x: u16,
    y: u16,
    width: u16,
    rows: u16,
) -> (Rect, Vec<(Rect, Target)>) {
    let all: Vec<&str> = crates().collect();
    let title = format!("\u{2039} {}", msg::crates_title(all.len()));
    let back = Rect::new(x, y, (display_width(&title) as u16).min(width), 1);
    let hovered = with(|s| {
        s.picker.is_none() && s.pointer.is_some_and(|(c, r)| back.contains((c, r).into()))
    });
    let title_style = Style::default()
        .fg(th().ui.accent)
        .bg(th().ui.panel)
        .add_modifier(Modifier::BOLD);
    let title_style = if hovered {
        title_style.patch(crate::theme::plain_button_style(true))
    } else {
        title_style
    };
    dialog_kit::put(buf, x, y, &title, width, title_style);
    let area = Rect::new(x, y + 1, width, rows.saturating_sub(1));
    let page = usize::from(area.height);
    let offset = with(|s| {
        s.crates_area = area;
        s.crates_body = Rect::new(x, area.y.saturating_sub(1), width, area.height + 2);
        s.crates_offset = s.crates_offset.min(all.len().saturating_sub(page));
        s.crates_sel = s
            .crates_sel
            .min((s.crates_offset + page).saturating_sub(1))
            .max(s.crates_offset)
            .min(all.len().saturating_sub(1));
        s.crates_offset
    });
    let (selected, pointer) = with(|s| (s.crates_sel, s.pointer));
    let free = with(|s| s.picker.is_none());
    let style = Style::default().fg(th().ui.text).bg(th().ui.panel);
    let rows: Vec<(&str, &str, &str, &str)> = all.iter().copied().map(split_crate).collect();
    let name_w = rows.iter().map(|r| display_width(r.0)).max().unwrap_or(0);
    let version_w = rows.iter().map(|r| display_width(r.1)).max().unwrap_or(0);
    let url_w = rows
        .iter()
        .map(|r| display_width(short_url(r.2)))
        .max()
        .unwrap_or(0);
    // The link column leaves the license room for a plain `MIT OR Apache-2.0`.
    let url_w =
        url_w.min(usize::from(width.saturating_sub(1)).saturating_sub(name_w + version_w + 4 + 22));
    let mut links = Vec::new();
    let row_w = width.saturating_sub(1);
    for (i, (name, version, url, license)) in rows.iter().enumerate().skip(offset).take(page) {
        let y = area.y + (i - offset) as u16;
        let row = Rect::new(x, y, row_w, 1);
        let linked = *url != "-";
        let hovered = free && linked && pointer.is_some_and(|(c, r)| row.contains((c, r).into()));
        let base = if i == selected {
            style.bg(th().ui.selection)
        } else {
            style
        };
        let head = format!("{name:<name_w$}  {version:<version_w$}  ");
        let lx = x + display_width(&head) as u16;
        let tail_x = lx + url_w as u16 + 2;
        dialog_kit::put(buf, x, y, &" ".repeat(usize::from(row_w)), row_w, base);
        dialog_kit::put(buf, x, y, &head, row_w, base);
        if linked {
            let link = base.fg(th().ui.accent).add_modifier(Modifier::UNDERLINED);
            let link = if hovered {
                link.patch(crate::theme::plain_button_style(true))
            } else {
                link
            };
            dialog_kit::put(buf, lx, y, short_url(url), url_w as u16, link);
            links.push((row, Target::Url(url)));
        }
        let room = (x + row_w).saturating_sub(tail_x);
        dialog_kit::put(buf, tail_x, y, license, room, base);
    }
    // The bar takes the bordered box's right edge, like every list's.
    let body = Rect::new(x, area.y.saturating_sub(1), width, area.height + 2);
    if let Some(track) = crate::scrollbar::Track::vertical(body, all.len(), offset) {
        let pointer = with(|s| s.pointer);
        crate::scrollbar::with_pointer(pointer, || {
            crate::scrollbar::render_buf(buf, track, true, false)
        });
    }
    (back, links)
}

/// Call once per frame after every popup is drawn (see `DetailAvatarImage`).
pub(crate) fn finish_frame(buf: &mut Buffer) {
    logo::finish_frame(buf);
}

/// Call after `Terminal::draw` returns.
pub(crate) fn write_pending<W: std::io::Write>(out: &mut W) -> std::io::Result<()> {
    logo::write_pending(out)
}

/// Tells the pixel path what the terminal can draw.
pub(crate) fn init(capability: crate::terminal_image::Capability) {
    logo::init(capability);
}

mod logo {
    use std::cell::RefCell;

    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;
    use ratatui::style::Color;

    use crate::avatar::DetailAvatarImage;
    use crate::terminal_image::{Bitmap, Capability, CellSize};

    /// What `render` reserved: the box, the cells as it left them and the
    /// sprite, so `finish_frame` can tell a popup was drawn over the
    /// picture and put half-blocks in its place.
    struct Reserved {
        area: Rect,
        pixels: Rect,
        cells: Vec<ratatui::buffer::Cell>,
        bitmap: Bitmap,
    }

    thread_local! {
        static RESERVED: RefCell<Option<Reserved>> = const { RefCell::new(None) };
        static IMAGE: RefCell<Option<DetailAvatarImage>> = const { RefCell::new(None) };
        static CELL: RefCell<Option<CellSize>> = const { RefCell::new(None) };
    }

    pub(super) fn init(capability: Capability) {
        IMAGE.with(|i| *i.borrow_mut() = Some(DetailAvatarImage::new(capability)));
        CELL.with(|c| *c.borrow_mut() = Some(capability.cell));
    }

    /// Paints `bitmap` into `area`: a picture when the terminal draws them
    /// and `picture` is set, else half-blocks (two pixels per cell).
    ///
    /// A picture is never left under or beside something drawn over it: a
    /// popup that touches its cells makes `finish_frame` withdraw it for
    /// half-blocks (the popup's cells, the picture's cells and the
    /// terminal's own redraw -- ConPTY's on Windows -- no longer have to
    /// agree on which cells an escape-sequence picture owns), and it comes
    /// back when they are free. A caller that knows a popup is opening
    /// passes `picture: false` to skip even the frame it is re-sent in.
    pub(super) fn render(buf: &mut Buffer, area: Rect, bitmap: &Bitmap, picture: bool) {
        RESERVED.with(|r| *r.borrow_mut() = None);
        let pixels = pixel_area(area);
        let drawn = picture
            && IMAGE.with(|i| {
                i.borrow_mut()
                    .as_mut()
                    .is_some_and(|img| img.render(buf, pixels, bitmap))
            });
        if drawn {
            let cells = cells(buf, area);
            RESERVED.with(|r| {
                *r.borrow_mut() = Some(Reserved {
                    area,
                    pixels,
                    cells,
                    bitmap: bitmap.clone(),
                });
            });
        } else {
            half_blocks(buf, area, bitmap);
        }
    }

    fn cells(buf: &Buffer, area: Rect) -> Vec<ratatui::buffer::Cell> {
        let mut out = Vec::new();
        for y in area.top()..area.bottom() {
            for x in area.left()..area.right() {
                out.push(buf[(x, y)].clone());
            }
        }
        out
    }

    /// Index of `(x, y)` in a snapshot of `area`.
    fn index(area: Rect, x: u16, y: u16) -> usize {
        usize::from(y - area.y) * usize::from(area.width) + usize::from(x - area.x)
    }

    /// The pixel path's box: the same rows, with the columns that make it
    /// square for this terminal's cell shape.
    fn pixel_area(area: Rect) -> Rect {
        let cell = CELL.with(|c| *c.borrow()).unwrap_or(CellSize::FALLBACK);
        let cols =
            ((u32::from(area.height) * cell.height + cell.width / 2) / cell.width.max(1)) as u16;
        Rect::new(area.x, area.y, cols.max(2).min(area.width), area.height)
    }

    /// A zero- or one-cell-wide area made `clamp(2, width)` panic.
    #[cfg(test)]
    #[test]
    fn pixel_area_survives_a_tiny_area() {
        for (w, h) in [(0, 0), (0, 5), (1, 1), (1, 9)] {
            let r = pixel_area(Rect::new(0, 0, w, h));
            assert!(r.width <= w);
        }
    }

    fn half_blocks(buf: &mut Buffer, area: Rect, bitmap: &Bitmap) {
        let panel = match crate::th().ui.panel {
            Color::Rgb(r, g, b) => (r, g, b),
            _ => (24, 26, 38),
        };
        let sample = |cx: u32, py: u32| -> Color {
            let x = (cx * bitmap.width / u32::from(area.width).max(1)).min(bitmap.width - 1);
            let y =
                (py * bitmap.height / (u32::from(area.height) * 2).max(1)).min(bitmap.height - 1);
            match bitmap.rgba(x, y) {
                Some((r, g, b, a)) if a > 127 => Color::Rgb(r, g, b),
                _ => Color::Rgb(panel.0, panel.1, panel.2),
            }
        };
        for cy in 0..area.height {
            for cx in 0..area.width {
                if let Some(cell) = buf.cell_mut((area.x + cx, area.y + cy)) {
                    cell.set_symbol("\u{2580}")
                        .set_fg(sample(u32::from(cx), u32::from(cy) * 2))
                        .set_bg(sample(u32::from(cx), u32::from(cy) * 2 + 1));
                }
            }
        }
    }

    pub(super) fn finish_frame(buf: &mut Buffer) {
        if let Some(r) = RESERVED.with(|r| r.borrow_mut().take()) {
            let now = cells(buf, r.area);
            let touched = (r.pixels.top()..r.pixels.bottom()).any(|y| {
                (r.pixels.left()..r.pixels.right())
                    .any(|x| now[index(r.area, x, y)] != r.cells[index(r.area, x, y)])
            });
            if touched {
                IMAGE.with(|i| {
                    if let Some(img) = i.borrow_mut().as_mut() {
                        img.cancel();
                    }
                });
                // Half-blocks on every cell of the box no popup took.
                let mut shot = Buffer::empty(r.area);
                half_blocks(&mut shot, r.area, &r.bitmap);
                for y in r.area.top()..r.area.bottom() {
                    for x in r.area.left()..r.area.right() {
                        let i = index(r.area, x, y);
                        if now[i] == r.cells[i] {
                            buf[(x, y)] = shot[(x, y)].clone();
                        }
                    }
                }
            }
        }
        IMAGE.with(|i| {
            if let Some(img) = i.borrow_mut().as_mut() {
                img.finish_frame(buf);
            }
        });
    }

    pub(super) fn write_pending<W: std::io::Write>(out: &mut W) -> std::io::Result<()> {
        IMAGE.with(|i| match i.borrow_mut().as_mut() {
            Some(img) => img.write_pending(out),
            None => Ok(()),
        })
    }
}

/// The build script's license parsing, run as unit tests too.
#[cfg(test)]
#[allow(dead_code)]
#[path = "../build_crates.rs"]
mod build_crates;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn about_click_writes_only_this_installations_colour() {
        use gitpoulpi_config::{Config, InstallationEntry};
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("config.toml");
        std::fs::write(
            &file,
            "[ui]\nlogo_color = \"violet\"\n\n[installation.settings.bbbb]\nlogo_color = \"sun\"\n",
        )
        .unwrap();
        let mut seeded = Config::default();
        seeded.installation.settings.insert(
            "aaaa".into(),
            InstallationEntry {
                logo_color: "teal".into(),
                name: "GitPoulpi (cargo)".into(),
                ..Default::default()
            },
        );
        // A non-primary installation: its entry changes, `ui.*` and the
        // other installation's entry do not.
        write_colour(&file, "aaaa", false, &seeded, "green").unwrap();
        let config = Config::load(&file).unwrap();
        assert_eq!(config.installation.settings["aaaa"].logo_color, "green");
        assert_eq!(
            config.installation.settings["aaaa"].name, "GitPoulpi (cargo)",
            "the seeded entry came along"
        );
        assert_eq!(config.installation.settings["bbbb"].logo_color, "sun");
        assert_eq!(config.ui.logo_color, "violet");
        // The primary writes the key old builds read, and nothing else.
        write_colour(&file, "aaaa", true, &seeded, "ocean").unwrap();
        let config = Config::load(&file).unwrap();
        assert_eq!(config.ui.logo_color, "ocean");
        assert_eq!(config.installation.settings["aaaa"].logo_color, "green");
        assert_eq!(config.installation.settings["bbbb"].logo_color, "sun");
    }

    /// The reference is the eyes (half-way across the logo, 6/16 down), not
    /// the logo's edge: a pointer just beside the eyes, still over the logo,
    /// already looks that way.
    #[test]
    fn the_eyes_follow_the_pointer_from_their_own_position() {
        // A 16 x 8 logo at (40, 10): the eyes' cell columns are 47 / 48 and
        // their rows 12 / 13.
        let logo = Rect::new(40, 10, 16, 8);
        let at = |x, y| look_toward(Some((x, y)), logo);
        assert_eq!(at(47, 12), (0, 0), "on the eyes");
        assert_eq!(at(48, 13), (0, 0), "on the eyes");
        assert_eq!(at(44, 12), (-1, 0), "left of the eyes, over the logo");
        assert_eq!(at(52, 12), (1, 0), "right of the eyes, over the logo");
        assert_eq!(at(47, 10), (0, -1), "above the eyes, over the logo");
        assert_eq!(at(47, 16), (0, 1), "below the eyes, over the logo");
        assert_eq!(at(5, 2), (-1, -1), "top left");
        assert_eq!(at(80, 2), (1, -1), "top right");
        assert_eq!(at(5, 40), (-1, 1), "bottom left");
        assert_eq!(at(80, 36), (1, 1), "bottom right");
        assert_eq!(at(5, 12), (-1, 0), "far left, on the eyes' row");
        assert_eq!(at(120, 13), (1, 0), "far right, on the eyes' row");
        assert_eq!(look_toward(None, logo), (0, 0), "no pointer yet");
    }

    #[test]
    fn a_crate_line_splits_into_name_version_and_license() {
        assert_eq!(
            split_crate("serde 1.0.1 https://github.com/serde-rs/serde MIT OR Apache-2.0"),
            (
                "serde",
                "1.0.1",
                "https://github.com/serde-rs/serde",
                "MIT OR Apache-2.0"
            )
        );
        assert_eq!(split_crate("x 1.0.0 - ?"), ("x", "1.0.0", "-", "?"));
        assert_eq!(
            split_crate("toml 1.2.1+spec-1.1.0 https://x MIT").1,
            "1.2.1",
            "build metadata is not displayed"
        );
        assert_eq!(
            short_url("https://crates.io/crates/x/1.0.0"),
            "crates.io/crates/x/1.0.0"
        );
    }

    #[test]
    fn every_crate_has_a_web_address_and_a_registry_crate_a_crates_io_one() {
        let all: Vec<_> = crates().map(split_crate).collect();
        assert!(!all.is_empty());
        for (name, _, url, _) in &all {
            assert!(
                url.starts_with("https://") || url.starts_with("http://"),
                "{name}: {url}"
            );
        }
        let ratatui = all.iter().find(|c| c.0 == "ratatui").expect("ratatui");
        assert!(ratatui.2.contains("ratatui"), "{ratatui:?}");
    }

    #[test]
    fn wrapping_breaks_on_spaces_within_the_width() {
        let lines = wrap("aaa bbb ccc ddd", 7);
        assert_eq!(lines, vec!["aaa bbb", "ccc ddd"]);
        assert_eq!(wrap("", 7), Vec::<String>::new());
    }
}

#[cfg(test)]
mod render_tests {
    use super::*;
    use ratatui::widgets::Widget;
    use ratatui::{backend::TestBackend, Terminal};

    #[test]
    fn the_window_paints_its_facts_and_closes_with_the_state() {
        let mut term = Terminal::new(TestBackend::new(100, 30)).unwrap();
        open();
        assert!(is_open());
        on_key(crossterm::event::KeyEvent::from(
            crossterm::event::KeyCode::Char('x'),
        ));
        assert!(is_open());
        term.draw(render).unwrap();
        let text = crate::dialog_kit::buffer_text(term.backend().buffer());
        close();
        assert!(text.contains("About GitPoulpi"), "{text}");
        assert!(text.contains("License: MIT"), "{text}");
        assert!(text.contains("github.com/Poulpi-Lab/GitPoulpi"), "{text}");
        open();
        // Space starts a spin (it lands, and is saved, only after half a
        // second: not exercised here, which would write the real config).
        on_key(crossterm::event::KeyEvent::from(
            crossterm::event::KeyCode::Char(' '),
        ));
        assert!(animating());
        let (_, scale, _, _) = frame_state();
        assert!((0.0..=1.0).contains(&scale));
        with(|s| s.spin = None);
        term.draw(render).unwrap();
        close();
        term.draw(render).unwrap();
        assert!(!crate::dialog_kit::buffer_text(term.backend().buffer()).contains("License"));
        // The Licenses view (same test: the window's state is process-wide).
        open();
        term.draw(render).unwrap();
        let facts = crate::dialog_kit::buffer_text(term.backend().buffer());
        assert!(facts.contains("not affiliated with"), "{facts}");
        assert!(facts.contains("Licenses and credits (t: read)"), "{facts}");
        on_key(crossterm::event::KeyEvent::from(
            crossterm::event::KeyCode::Char('t'),
        ));
        term.draw(render).unwrap();
        let first = crate::dialog_kit::buffer_text(term.backend().buffer());
        assert!(
            first.contains("third-party licenses and credits"),
            "{first}"
        );
        assert!(!first.contains("License: MIT"), "the facts are replaced");
        on_key(crossterm::event::KeyEvent::from(
            crossterm::event::KeyCode::PageDown,
        ));
        term.draw(render).unwrap();
        let second = crate::dialog_kit::buffer_text(term.backend().buffer());
        assert_ne!(first, second, "PageDown scrolls");
        assert!(with(|s| s.lic_offset) > 0);
        on_key(crossterm::event::KeyEvent::from(
            crossterm::event::KeyCode::Esc,
        ));
        assert!(!with(|s| s.lic_view) && is_open(), "Esc goes back first");
        close();
        // Same test again: the window's state is one process-wide static.
        a_popup_over_the_logo_swaps_the_picture_for_half_blocks_and_back();
        // And the frame: it moves and arranges, and has no resize grip.
        let mut big = Terminal::new(TestBackend::new(160, 50)).unwrap();
        open();
        crate::dialog_kit::frame::testing::check_gestures(false, || {
            big.draw(render).unwrap();
        });
        close();
    }

    /// One frame as the event loop runs it: the window, then `over`
    /// (a popup), `finish_frame`, `write_pending`, then the terminal model.
    fn frame(
        term: &mut Terminal<TestBackend>,
        vt: &mut crate::terminal_image::vt_sim::Vt,
        prev: &mut ratatui::buffer::Buffer,
        over: impl FnOnce(&mut ratatui::Frame),
    ) -> String {
        term.draw(|f| {
            render(f);
            over(f);
            finish_frame(f.buffer_mut());
        })
        .unwrap();
        let buf = term.backend().buffer().clone();
        let mut out = Vec::new();
        write_pending(&mut out).unwrap();
        vt.draw(prev, &buf);
        vt.feed(&out);
        *prev = buf;
        String::from_utf8(out).unwrap()
    }

    /// The logo's picture never sits under or beside what is drawn over it
    /// (the maintainer's torn logo under ConPTY, with the colourway list open):
    /// while the list is open, or any popup touches the picture, the logo is
    /// half-blocks and no picture tile is left; it is a picture again after.
    fn a_popup_over_the_logo_swaps_the_picture_for_half_blocks_and_back() {
        use crate::terminal_image::{Capability, CellSize, Protocol};
        let screen = Rect::new(0, 0, 100, 30);
        let mut term = Terminal::new(TestBackend::new(100, 30)).unwrap();
        let mut vt = crate::terminal_image::vt_sim::Vt::new(screen);
        let mut prev = ratatui::buffer::Buffer::empty(screen);
        init(Capability {
            protocol: Protocol::Iterm2Alpha,
            cell: CellSize {
                width: 9,
                height: 21,
            },
        });
        open();
        let first = frame(&mut term, &mut vt, &mut prev, |_| {});
        assert!(first.contains("]1337;File="), "the picture is sent");
        let logo = with(|s| s.logo);
        assert!(!logo.is_empty());
        assert!(!vt.tiles_in(logo).is_empty(), "a picture is on the logo");
        let sent = |o: &str| o.contains("]1337;File=");
        // A popup over a corner of the picture: half-blocks, no tile.
        let corner = Rect::new(logo.x, logo.y, 3, 2);
        let covered = frame(&mut term, &mut vt, &mut prev, |f| {
            crate::wide_edge::Clear.render(corner, f.buffer_mut());
        });
        assert!(!sent(&covered), "no picture is sent under a popup");
        assert!(vt.tiles_in(logo).is_empty(), "no tile survives");
        let far = (logo.right() - 1, logo.bottom() - 1);
        assert_eq!(
            term.backend().buffer()[far].symbol(),
            "\u{2580}",
            "half-blocks where the popup is not"
        );
        // Popup gone: the picture comes back.
        let back = frame(&mut term, &mut vt, &mut prev, |_| {});
        assert!(sent(&back), "{back:?}");
        assert!(!vt.tiles_in(logo).is_empty());
        // The colourway list open: no picture is sent in the frame it opens.
        open_picker();
        let listed = frame(&mut term, &mut vt, &mut prev, |_| {});
        assert!(!sent(&listed), "no picture beside the open list");
        assert!(vt.tiles_in(logo).is_empty());
        with(|s| s.picker = None);
        let closed = frame(&mut term, &mut vt, &mut prev, |_| {});
        assert!(sent(&closed), "restored when the list closes");
        close();
    }

    #[test]
    fn the_embedded_notices_name_libgit2_and_the_crates() {
        let text = NOTICES;
        assert!(text.contains("libgit2") && text.contains("serde"));
        assert!(notice_lines().len() > 1000);
    }
}
