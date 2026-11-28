//! The commit graph's frontend-neutral model: lane assignment ([`layout`]),
//! the cell vocabulary it produces ([`part`]), stash rows and the Log's row
//! arithmetic ([`stash`]), lazy history paging ([`paging`]) and the load
//! settings ([`load`]).
//!
//! Nothing here knows how a cell is drawn or what a colour is: the layout
//! hands out [`ColorRef`]s and each frontend resolves them against its own
//! palette and theme.

pub mod layout;
pub mod load;
pub mod paging;
pub mod part;
pub mod stash;

pub use layout::{build_graph_grid_themed, GraphGrid, GraphRow, WipAnchor};
pub use load::LogLoadSettings;
pub use paging::{lookahead_rows, LazyLog, LogPageRequest};
pub use part::{ColorRef, NodeKind, Sides, TreeCell, TreePart};
pub use stash::{place_stashes, GraphStash, LogEntry, LogRows, StashSlot};

#[cfg(test)]
mod tests;
