use super::*;

mod help;
mod hit;
mod hover;
mod main_window_i18n;
mod navigation;
// The toolbar goldens include the `LFS ▾` button: built with `lfs` only.
#[cfg(feature = "lfs")]
mod toolbar;
#[cfg(not(feature = "lfs"))]
mod toolbar_no_lfs;
mod widgets;
