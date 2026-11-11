//! The commit-graph column.
//!
//! Self-contained: how the tree is laid out and drawn lives here, and the
//! Log renderer reaches it through a handful of `use` lines.
//!
//! Three files, one job each:
//!
//! - [`part`] -- the [`TreePart`](part::TreePart) vocabulary: one value per
//!   distinct visual piece a cell can hold, the [`Sides`](part::Sides) it
//!   connects to, and the single function that turns a part into styled
//!   characters. **Read its header before touching the glyphs**; it carries
//!   the font measurements behind the choice and what broke on Windows.
//! - [`layout`] -- lane assignment: history in, a grid of parts out.
//! - `tests` -- including the exhaustive connectivity check that made
//!   "is every node joined to its lanes?" a property instead of a hope.

mod cache;
#[cfg(test)]
mod cache_tests;
mod compact;
mod connector;
mod ghost_ref;
mod inline_image;
mod layout;
mod paging;
mod part;
mod raster;
#[cfg(test)]
mod raster_tests;
mod stash;
#[cfg(test)]
mod tests;
mod total;
mod walk_filter;

use ratatui::style::Color;

pub(crate) use cache::{LayoutInputs, LogLayout, LogLayoutCache};
#[cfg(test)]
pub(crate) use compact::COMPACT_W;
pub(crate) use compact::{
    compact_line, compact_raster_rows, is_compact, min_width, set_avatars_in_graph,
};
pub(crate) use connector::{banded_graph_cell, node_column, with_ref_connector};
pub(crate) use ghost_ref::GhostRef;
pub(crate) use inline_image::GraphImage;
#[cfg(test)]
pub(crate) use layout::build_graph_lines;
pub(crate) use layout::{build_graph_lines_with, text_line, GraphRow, WipAnchor};
pub(crate) use paging::{LazyLog, LogPageRequest};
pub(crate) use part::ref_connector_span;
pub use part::{LaneBackground, NodeSet};
#[cfg(test)]
pub(crate) use part::{NodeKind, Sides, TreeCell, TreePart};
pub(crate) use raster::{avatar_picture_px, NodeAvatar, RasterRow};
#[cfg(test)]
pub(crate) use stash::StashSlot;
pub(crate) use stash::{place_stashes, GraphStash, LogEntry, LogRows};
pub(crate) use total::{refs_key, CommitTotal, TotalShown};
pub(crate) use walk_filter::{load_walk_filter, walk_filter};

/// `ui.graph_lane_background` as this module's own enum.
///
/// The conversion lives here because the config crate must not know what a
/// `ratatui::Color` is; this `From` is the one place the two names meet.
impl From<gitpoulpi_config::GraphLaneBackground> for LaneBackground {
    fn from(value: gitpoulpi_config::GraphLaneBackground) -> LaneBackground {
        match value {
            gitpoulpi_config::GraphLaneBackground::Off => LaneBackground::Off,
            gitpoulpi_config::GraphLaneBackground::RowTint => LaneBackground::RowTint,
        }
    }
}

/// GitPoulpi's own lane colors (the default `ui.graph_color_theme`), one per
/// lane, cycling: flat, saturated branch colors. Explicit RGB
/// rather than `Color::Red`/`Green`/..., because palette-relative ANSI colors
/// read as washed-out on a custom dark theme. The colors live in the config
/// crate's lane-palette registry (`gitpoulpi_config::theme::lane_palettes`).
pub(crate) static LANE_PALETTE: std::sync::LazyLock<Vec<Color>> =
    std::sync::LazyLock::new(|| registry_palette("gitpoulpi"));

fn registry_palette(slug: &str) -> Vec<Color> {
    gitpoulpi_config::theme::lane_palette(slug)
        .map(|p| p.colors.into_iter().map(crate::theme::color_of).collect())
        .unwrap_or_default()
}

/// `ui.graph_color_theme` as a small `Copy` handle: "follow the theme"
/// (the default) or a lane palette slug of the registry (a built-in one or
/// a user theme's `graph_palette`). Slugs are interned, so the handle stays
/// `Copy` and cheap to compare in the layout cache key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct LaneTheme(Option<&'static str>);

impl LaneTheme {
    /// Use the active UI theme's own lane palette.
    pub const FOLLOW: LaneTheme = LaneTheme(None);

    /// The handle for a stored `ui.graph_color_theme` value; `theme` and an
    /// empty value follow the theme.
    pub fn from_slug(slug: &str) -> LaneTheme {
        let slug = slug.trim();
        if slug.is_empty() || slug == gitpoulpi_config::GRAPH_COLOR_FOLLOW_THEME {
            return LaneTheme::FOLLOW;
        }
        static POOL: std::sync::Mutex<Vec<&'static str>> = std::sync::Mutex::new(Vec::new());
        let mut pool = POOL.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(found) = pool.iter().find(|s| **s == slug) {
            return LaneTheme(Some(found));
        }
        let leaked: &'static str = Box::leak(slug.to_string().into_boxed_str());
        pool.push(leaked);
        LaneTheme(Some(leaked))
    }
}

/// The ordered lane colors of `theme`: the registry's palette when one is
/// named (an unknown name, e.g. a deleted user theme, follows the theme),
/// else the active theme's own.
pub(crate) fn lane_palette(theme: LaneTheme) -> &'static [Color] {
    use std::collections::HashMap;
    use std::sync::Mutex;
    static CACHE: Mutex<Option<HashMap<&'static str, &'static [Color]>>> = Mutex::new(None);
    let Some(slug) = theme.0 else {
        return theme_lane_palette();
    };
    let mut cache = CACHE.lock().unwrap_or_else(|e| e.into_inner());
    let map = cache.get_or_insert_with(HashMap::new);
    if let Some(found) = map.get(slug) {
        return found;
    }
    let palette = gitpoulpi_config::theme::lane_palette(slug).or_else(|| {
        // A user theme's `graph_palette`.
        let set = gitpoulpi_config::ConfigPaths::discover()
            .map(|p| gitpoulpi_config::theme::load_user_themes(&p.themes_dir()))?;
        gitpoulpi_config::theme::lane_palettes_with(&set.themes)
            .into_iter()
            .find(|p| p.slug == slug)
    });
    match palette {
        Some(p) if !p.colors.is_empty() => {
            let colors: Vec<Color> = p.colors.into_iter().map(crate::theme::color_of).collect();
            let leaked: &'static [Color] = Box::leak(colors.into_boxed_slice());
            map.insert(slug, leaked);
            leaked
        }
        _ => theme_lane_palette(),
    }
}

/// The active theme's own lane palette (`ui.graph_color_theme` = follow the
/// theme).
pub(crate) fn theme_lane_palette() -> &'static [Color] {
    crate::th().graph.palette
}

/// How the graph is drawn: the seam for the preferences UI. Every field is
/// `pub`, and `Default` is the only place the shipped choice is made.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct GraphSettings {
    /// Which commit-marker family to draw. See [`NodeSet`].
    pub node_set: NodeSet,
    /// Whether the cells a lane runs through carry a background in that
    /// lane's color. `ui.graph_lane_background`; off by default.
    pub lane_background: LaneBackground,
    /// `ui.graph_row_tint_strength`: how faded the `row_tint` band is.
    /// Part of the cache key.
    pub tint_strength: gitpoulpi_config::GraphRowTintStrength,
    /// How the column reaches the screen. See [`GraphRender`].
    pub render: GraphRender,
    /// `ui.graph_color_theme`: which lane palette. Part of the cache key.
    pub theme: LaneTheme,
}

/// Whether the Graph column is drawn as characters or as pixels.
///
/// **[`GraphRender::Image`] is the real rendering**: a commit marker is an
/// anti-aliased disc joined to its lanes with no gap, because disc and
/// strokes are laid into the same bitmap by [`raster::rasterize`].
/// [`GraphRender::Text`] is for terminals with no image protocol and for the
/// pty test harness, which has no pixels.
///
/// Under `Image` the text cells are still produced, as blanks of the same
/// width, so the Log table's layout, column arithmetic and hit targets do
/// not depend on which mode is in force; the image is composited over them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum GraphRender {
    /// `○` `●` `◎` plus box drawing. The gap where a lane meets a node is
    /// **kept**; see `part`'s module header for why closing it in characters
    /// was rejected.
    #[default]
    Text,
    /// Blank cells, for an image to be drawn over.
    Image,
}

/// The slice of the Graph column that is on screen right now.
///
/// The Log's row arithmetic is the one place in this TUI that has bitten
/// repeatedly (`docs/developer/where-we-are.md`), so the image path does
/// **not** re-derive it. It is handed the facts the hit-testing side uses,
/// the scroll offset and the selected display index, and
/// [`visible_raster_rows`] walks them exactly the way `log_index_to_row`
/// does. A picture drawn one row out is worse than no picture.
///
/// Every Log row is one terminal line, the WIP row included.
pub(crate) struct ColumnWindow {
    /// How many terminal rows the column occupies.
    pub rows: u16,
    /// The table's scroll offset, in Log display indices. Read *after* the
    /// table has rendered, because that is when ratatui has settled it.
    pub offset: usize,
    /// The selected display index, if any. An image composites *above* text,
    /// so the selection band has to be drawn into the picture.
    pub selected: Option<usize>,
}

/// The rows of the Graph column that are on screen, as the rasteriser wants
/// them. `wip` is the WIP row ([`layout::GraphGrid::wip`]) and `stashes` the
/// stash rows ([`layout::GraphGrid::stashes`]); `order` is the Log's own
/// [`LogRows`], so a display index here is exactly the one the table drew.
pub(crate) fn visible_raster_rows(
    grid: &[GraphRow],
    wip: Option<&GraphRow>,
    stashes: &[GraphRow],
    order: &LogRows,
    window: &ColumnWindow,
) -> Vec<RasterRow> {
    let mut out: Vec<RasterRow> = Vec::new();
    let mut index = window.offset;
    while out.len() < usize::from(window.rows) {
        let row = match order.entry(index) {
            LogEntry::Wip => match wip {
                Some(wip) => wip,
                None => break,
            },
            LogEntry::Commit(i) => match grid.get(i) {
                Some(row) => row,
                None => break,
            },
            LogEntry::Stash(k) => match stashes.get(k) {
                Some(row) => row,
                None => break,
            },
        };
        out.push(RasterRow {
            cells: row.cells.clone(),
            selected: window.selected == Some(index),
            avatar: None,
            ref_connector: false,
        });
        index += 1;
    }
    out
}

/// One commit row on screen: where, and which commit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct VisibleCommit {
    /// Rows below the window's top.
    pub row: u16,
    /// The Log display index - what `ColumnWindow::selected` names.
    pub display: usize,
    /// Index into the commit list the table drew (`nodes[i]`).
    pub commit: usize,
}

/// The commit rows in `window`, walked exactly as [`visible_raster_rows`]
/// walks them, through `order`. The WIP and stash rows take their screen
/// line and are skipped (no author picture); the walk stops at the end of
/// the `commits` the table holds. Used by the Log's author avatars
/// (`avatar::LogAvatarImages`).
pub(crate) fn visible_commit_rows(
    order: &LogRows,
    commits: usize,
    window: &ColumnWindow,
) -> Vec<VisibleCommit> {
    let mut out = Vec::new();
    for row in 0..window.rows {
        let display = window.offset + usize::from(row);
        match order.entry(display) {
            LogEntry::Wip | LogEntry::Stash(_) => {}
            LogEntry::Commit(i) if i < commits => out.push(VisibleCommit {
                row,
                display,
                commit: i,
            }),
            LogEntry::Commit(_) => break,
        }
    }
    out
}

/// Puts each visible commit's author picture into its raster row (`ui.avatars
/// = "graph"`): `visible` is [`visible_commit_rows`] of the same window, so
/// `v.row` indexes `rows` exactly. Memory lookups only
/// ([`crate::avatar::AvatarService::pictures`]), never a read on the render
/// thread. Shared by the Log and the commit column, so the two cannot draw
/// the same author differently.
pub(crate) fn attach_node_avatars(
    rows: &mut [RasterRow],
    nodes: &[gitpoulpi_core::domain::CommitNode],
    visible: &[VisibleCommit],
    service: &mut crate::avatar::AvatarService,
) {
    let shas: Vec<String> = visible
        .iter()
        .map(|v| nodes[v.commit].id.to_hex())
        .collect();
    let queries: Vec<crate::avatar::AvatarQuery> = visible
        .iter()
        .zip(&shas)
        .map(|(v, sha)| crate::avatar::AvatarQuery {
            email: nodes[v.commit].author_email.as_str(),
            sha,
        })
        .collect();
    let pictures = service.pictures(&queries);
    for (v, picture) in visible.iter().zip(pictures) {
        if let (Some((author, bitmap)), Some(row)) = (picture, rows.get_mut(usize::from(v.row))) {
            row.avatar = Some(NodeAvatar {
                key: author.to_string(),
                bitmap: std::sync::Arc::clone(bitmap),
            });
        }
    }
}

/// How much history the Log asks the daemon for. `pub` and defaulted here;
/// [`daemon_link`](crate::daemon_link) is the only consumer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LogLoadSettings {
    /// How many commits to fetch for the graph up front ("Initial Commits in
    /// Graph").
    pub initial_commits: usize,
    /// "Lazy load commits in graph": additional commits are loaded
    /// when you reach the earliest commit in the graph. Off, you get
    /// [`initial_commits`] and nothing more.
    ///
    /// [`initial_commits`]: LogLoadSettings::initial_commits
    pub lazy_load: bool,
    /// How many commits one lazily-loaded page fetches. Only consulted
    /// when [`lazy_load`](LogLoadSettings::lazy_load) is on.
    pub page_commits: usize,
    /// The ceiling on how many commits the client will ever hold, or
    /// `None` for no ceiling. `general.max_commits_in_graph`.
    pub max_commits: Option<usize>,
}

// The numbers live in `gitpoulpi-config`, which the Preferences screen reads
// and writes, and are aliased here rather than restated so there is one
// default of each in the workspace.
impl LogLoadSettings {
    /// The "Initial commits in graph" default (5000, minimum 500): far
    /// more than any window shows, so scrolling always has somewhere to go,
    /// while staying bounded on huge repositories. 5000 `CommitNode`s are a few
    /// megabytes per session; later pages load as the scroll nears the end.
    pub const DEFAULT_INITIAL_COMMITS: usize =
        gitpoulpi_config::DEFAULT_INITIAL_COMMITS_IN_GRAPH as usize;

    /// The floor a configured value is clamped to: a window can be 200+ rows
    /// on a large display, and the initial page must still fill it without
    /// knowing the window size at request time.
    #[cfg(test)]
    pub const MIN_INITIAL_COMMITS: usize = gitpoulpi_config::MIN_INITIAL_COMMITS_IN_GRAPH as usize;

    /// One lazily-loaded page; lives in `gitpoulpi-config` like the two above.
    pub const DEFAULT_PAGE_COMMITS: usize =
        gitpoulpi_config::DEFAULT_COMMITS_PER_LAZY_PAGE as usize;

    /// Lazy loading is **on** by default: history is
    /// effectively unbounded up to [`max_commits`](LogLoadSettings::max_commits)
    /// while only the initial page is paid for up front.
    pub const DEFAULT_LAZY_LOAD: bool = true;

    /// The shipped ceiling; see [`gitpoulpi_config::DEFAULT_MAX_COMMITS_IN_GRAPH`].
    pub const DEFAULT_MAX_COMMITS: usize = gitpoulpi_config::DEFAULT_MAX_COMMITS_IN_GRAPH as usize;

    /// Clamps a configured count into the supported range.
    #[cfg(test)]
    pub fn with_initial_commits(count: usize) -> LogLoadSettings {
        LogLoadSettings {
            initial_commits: count.max(Self::MIN_INITIAL_COMMITS),
            ..LogLoadSettings::default()
        }
    }

    /// Everything this module needs from the user's config. The clamping is
    /// `gitpoulpi-config`'s, not a second copy.
    pub fn from_config(general: &gitpoulpi_config::GeneralConfig) -> LogLoadSettings {
        LogLoadSettings {
            initial_commits: general.initial_commits_in_graph_clamped() as usize,
            lazy_load: general.lazy_load_commits_in_graph,
            page_commits: general.commits_per_lazy_page_clamped() as usize,
            max_commits: general.max_commits_in_graph_effective().map(|n| n as usize),
        }
    }
}

impl Default for LogLoadSettings {
    fn default() -> LogLoadSettings {
        LogLoadSettings {
            initial_commits: Self::DEFAULT_INITIAL_COMMITS,
            lazy_load: Self::DEFAULT_LAZY_LOAD,
            page_commits: Self::DEFAULT_PAGE_COMMITS,
            max_commits: Some(Self::DEFAULT_MAX_COMMITS),
        }
    }
}
