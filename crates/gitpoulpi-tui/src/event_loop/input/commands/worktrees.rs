//! The worktree commands of the palette and the header `+` menu. The real
//! code is `worktrees_real.rs`; a build without the `worktree` feature has
//! inert stand-ins (nothing opens them: no row, no header, no palette entry).

#[cfg(feature = "worktree")]
#[path = "worktrees_real.rs"]
mod real;
#[cfg(feature = "worktree")]
pub(in crate::event_loop) use real::{open_plus_menu, open_row_menu, run};

#[cfg(not(feature = "worktree"))]
mod stub {
    use crate::event_loop::input::commands::CommandCtx;
    use crate::event_loop::PaletteCommand;

    pub(in crate::event_loop) fn open_plus_menu(_cx: &mut CommandCtx<'_>, _col: u16, _row: u16) {}

    pub(in crate::event_loop) fn open_row_menu(_cx: &mut CommandCtx<'_>) {}

    pub(in crate::event_loop) fn run(_cx: &mut CommandCtx<'_>, _cmd: PaletteCommand) -> bool {
        false
    }
}
#[cfg(not(feature = "worktree"))]
#[allow(unused_imports)]
pub(in crate::event_loop) use stub::{open_plus_menu, open_row_menu, run};
