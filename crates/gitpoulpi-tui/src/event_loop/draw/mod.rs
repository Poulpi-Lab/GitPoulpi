//! The draw closure's panes, one file each: every pane's paint walk takes a
//! `…DrawCtx` of borrowed fields.

pub(super) mod central;
pub(super) mod chrome;
pub(super) mod column_chooser;
pub(super) mod footer;
pub(super) mod log;
pub(super) mod overlay_menus;
pub(super) mod overlays;
pub(crate) mod palette_overlay;
pub(super) mod right_panel;
pub(super) mod sidebar;
