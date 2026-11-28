//! The workspace search's neutral half: which repositories a search covers
//! ([`scope`]) and how the answers of every repository merge into one grouped
//! list a frontend paints ([`model`]). The daemon `Search` fan-out, the
//! debounce and the popup stay in the frontend.

pub mod model;
pub mod scope;
