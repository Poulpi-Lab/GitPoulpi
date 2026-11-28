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
//! - [`layout`] -- the terminal's reading of the frontend-neutral lane
//!   assignment (`gitpoulpi_app::graph`): neutral colour references resolved
//!   against the lane palette, rows turned into `Line`s.
//! - `tests` -- including the exhaustive connectivity check that made
//!   "is every node joined to its lanes?" a property instead of a hope.

mod avatar_hover;
mod cache;
#[cfg(test)]
mod cache_tests;
mod compact;
mod connector;
mod ghost_ref;
mod inline_image;
mod layout;
mod part;
mod raster;
#[cfg(test)]
mod raster_tests;
#[cfg(test)]
mod tests;
mod total;

use ratatui::style::Color;

pub(crate) use avatar_hover::{
    picture_cells, set_slots as set_avatar_slots, slot_at as avatar_slot_at,
};
pub(crate) use cache::{LayoutInputs, LogLayout, LogLayoutCache};
#[cfg(test)]
pub(crate) use compact::COMPACT_W;
pub(crate) use compact::{
    compact_line, compact_raster_rows, is_compact, min_width, set_avatars_in_graph,
};
pub(crate) use connector::{banded_graph_cell, node_column, with_ref_connector};
pub(crate) use ghost_ref::GhostRef;
pub use gitpoulpi_app::graph::LogLoadSettings;
pub(crate) use gitpoulpi_app::graph::StashSlot;
pub(crate) use gitpoulpi_app::graph::{load_walk_filter, refs_key, walk_filter, TotalShown};
pub(crate) use gitpoulpi_app::graph::{place_stashes, GraphStash, LogEntry, LogRows};
pub(crate) use gitpoulpi_app::graph::{LazyLog, LogPageRequest};
pub(crate) use inline_image::GraphImage;
#[cfg(test)]
pub(crate) use layout::build_graph_lines;
pub(crate) use layout::{build_graph_lines_with, text_line, GraphRow, WipAnchor};
pub(crate) use part::ref_connector_span;
pub use part::{LaneBackground, NodeSet};
#[cfg(test)]
pub(crate) use part::{NodeKind, Sides, TreeCell, TreePart};
pub(crate) use raster::{avatar_picture_px, NodeAvatar, RasterRow};
pub(crate) use total::CommitTotal;

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
#[cfg(test)]
pub(crate) static LANE_PALETTE: std::sync::LazyLock<Vec<Color>> =
    std::sync::LazyLock::new(|| registry_palette("gitpoulpi"));

#[cfg(test)]
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

/// The lane palettes of the user themes (`<config>/themes/*.toml`), fed by
/// [`set_user_lane_palettes`] wherever the theme files are (re)read. A user
/// theme wins over a built-in lane palette of the same name.
type UserLanes = Vec<(String, Vec<gitpoulpi_config::theme::ThemeColor>)>;
static USER_LANES: std::sync::Mutex<UserLanes> = std::sync::Mutex::new(Vec::new());

type LaneCache = std::collections::HashMap<&'static str, &'static [Color]>;
static LANE_CACHE: std::sync::Mutex<Option<LaneCache>> = std::sync::Mutex::new(None);

/// Records the lane colors of `themes` (the user themes as just loaded).
/// Returns whether anything changed, so the caller can bump the theme
/// generation that keys the graph caches.
pub(crate) fn set_user_lane_palettes(themes: &[gitpoulpi_config::theme::ThemeSpec]) -> bool {
    let next: UserLanes = themes
        .iter()
        .filter(|t| !t.graph_palette.is_empty())
        .map(|t| (t.slug.clone(), t.graph_palette.clone()))
        .collect();
    let mut user = USER_LANES.lock().unwrap_or_else(|e| e.into_inner());
    if *user == next {
        return false;
    }
    *user = next;
    *LANE_CACHE.lock().unwrap_or_else(|e| e.into_inner()) = None;
    true
}

/// The ordered lane colors of `theme`: a user theme's palette, else the
/// registry's when one is named (an unknown name, e.g. a deleted user theme,
/// follows the theme), else the active theme's own.
pub(crate) fn lane_palette(theme: LaneTheme) -> &'static [Color] {
    let Some(slug) = theme.0 else {
        return theme_lane_palette();
    };
    let mut cache = LANE_CACHE.lock().unwrap_or_else(|e| e.into_inner());
    let map = cache.get_or_insert_with(Default::default);
    if let Some(found) = map.get(slug) {
        return found;
    }
    let user = USER_LANES
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .iter()
        .find(|(s, _)| s == slug)
        .map(|(_, colors)| colors.clone());
    let colors = user.or_else(|| gitpoulpi_config::theme::lane_palette(slug).map(|p| p.colors));
    match colors {
        Some(colors) if !colors.is_empty() => {
            let colors: Vec<Color> = colors.into_iter().map(crate::theme::color_of).collect();
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

#[cfg(test)]
mod user_lane_tests {
    use super::*;
    use gitpoulpi_config::theme::parse_user_themes;

    fn themes(text: &str) -> Vec<gitpoulpi_config::theme::ThemeSpec> {
        parse_user_themes(&[("lane-probe-x".to_string(), text.to_string())]).themes
    }

    #[test]
    fn a_user_theme_name_resolves_to_its_lane_colors_and_a_removed_one_follows_the_theme() {
        let probe = LaneTheme::from_slug("lane-probe-x");
        set_user_lane_palettes(&[]);
        // Unknown (never defined or deleted): the active theme's lanes.
        assert_eq!(lane_palette(probe), theme_lane_palette());
        assert!(set_user_lane_palettes(&themes(
            "graph_palette = [\"#0a0b0c\", \"#0a0b0d\"]\n"
        )));
        assert_eq!(
            lane_palette(probe),
            &[Color::Rgb(10, 11, 12), Color::Rgb(10, 11, 13)]
        );
        // Edited: the new colors replace the cached ones.
        assert!(set_user_lane_palettes(&themes(
            "graph_palette = [\"#0a0b0e\"]\n"
        )));
        assert_eq!(lane_palette(probe), &[Color::Rgb(10, 11, 14)]);
        // No change reported twice; deleted: falls back, no crash.
        assert!(!set_user_lane_palettes(&themes(
            "graph_palette = [\"#0a0b0e\"]\n"
        )));
        assert!(set_user_lane_palettes(&[]));
        assert_eq!(lane_palette(probe), theme_lane_palette());
    }
}
