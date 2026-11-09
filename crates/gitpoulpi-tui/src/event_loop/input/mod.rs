//! Input handling: the one command layer, and the key-event dispatch (modal
//! guard chain, input modes, Normal mode).

/// The tab-level actions' borrow of the window's state (`tabs::TabCtx`).
macro_rules! tab_ctx {
    ($cx:expr) => {
        TabCtx {
            session: $cx.session,
            link: $cx.link,
            current: $cx.repo,
            cwd: $cx.cwd,
            dialogs: &mut *$cx.repo_dialogs,
            tabs: &mut *$cx.repo_tabs,
            active_tab: &mut *$cx.active_tab,
            switch_to: &mut *$cx.switch_to,
            close_window: &mut *$cx.close_window,
            show_management: &mut *$cx.show_management,
            management: &mut *$cx.management,
            repo_menu: &mut *$cx.repo_menu,
            input_mode: &mut *$cx.input_mode,
            clipboard: &mut *$cx.clipboard,
            pending_worktree_tab: &mut *$cx.pending_worktree_tab,
            remote_ui: &mut *$cx.remote_ui,
        }
    };
}

pub(super) mod commands;
mod keys_modes;
mod keys_normal;
mod keys_normal_lists;
mod keys_normal_nav;
mod keys_prefix;
pub(super) mod menu_actions;
mod modal;
mod mouse_drag;
mod mouse_left;
mod mouse_left_menus;
mod mouse_left_panes;
mod mouse_modes;
mod mouse_moved;
mod mouse_prelude;
mod mouse_right;
mod mouse_scroll;
#[cfg(test)]
mod msg_tests;
mod pointer;

use crate::event_loop::*;
use commands::CommandCtx;
pub(in crate::event_loop) use commands::Flow;
use crossterm::event::MouseEvent;
pub(in crate::event_loop) use keys_prefix::open_rewrite_proposed;
pub(in crate::event_loop) use mouse_modes::{overflow_menu_event, popup_list_mouse, ListMouse};
pub(in crate::event_loop) use mouse_prelude::main_field_event;

/// The per-event values the key handlers read besides the window state.
pub(in crate::event_loop) struct KeyEnv<'a> {
    pub(in crate::event_loop) km: &'a keymap::Keymap,
    /// The vim-style count prefix accumulated so far (1 when none).
    pub(in crate::event_loop) count: i32,
    pub(in crate::event_loop) palette_ctx: PaletteContext,
    pub(in crate::event_loop) unpushed_tags: &'a HashSet<String>,
}

/// Routes one key press. The order is the precedence between surfaces:
/// popups, dialogs and overlays first (`modal`), then the input mode that
/// has the keyboard, then the Normal-mode keymap.
pub(in crate::event_loop) fn handle_key(
    cx: &mut CommandCtx<'_>,
    env: &KeyEnv<'_>,
    key: KeyEvent,
) -> io::Result<Flow> {
    let slots = open_menu::MenuSlots::of(cx);
    let flow = handle_key_inner(cx, env, key);
    slots.keep_only_the_newest(cx);
    flow
}

/// What the Activity Log (an entry's remedy button) or the palette asked the
/// window to do.
pub(in crate::event_loop) fn apply_card_command(
    cx: &mut CommandCtx<'_>,
    command: crate::activity_log::Command,
) {
    use crate::activity_log::Command;
    match command {
        Command::Submit(action) => cx.link.submit_action(action),
        Command::Message(text) => *cx.last_message = Some(text),
        Command::OpenIntegrations => {
            let mut state = PreferencesState::open(cx.repo_root);
            let _ = state.show_page(Some(gitpoulpi_ipc::ui::SettingsPage::Integrations), None);
            *cx.preferences = Some(state);
        }
    }
}

/// The jobs panel's Enter / double click on a job of another repository:
/// show that repository's tab. A job of the repository on screen, or of no
/// repository (a clone), only closes the panel.
fn apply_jobs_goto(cx: &mut CommandCtx<'_>) {
    let Some(id) = crate::jobs_ui::take_goto() else {
        return;
    };
    if id == cx.repo.id {
        return;
    }
    let Some(index) = cx
        .repo_tabs
        .iter()
        .position(|t| crate::tabs::repo_ref_for(t.path.clone()).id == id)
    else {
        return;
    };
    if let Some(message) = crate::tabs::apply_switcher_action(
        repo_switcher::SwitcherAction::Focus(index),
        cx.session,
        cx.link,
        cx.repo,
        &mut *cx.repo_dialogs,
        &mut *cx.repo_tabs,
        &mut *cx.active_tab,
        &mut *cx.switch_to,
        &mut *cx.close_window,
    ) {
        *cx.last_message = Some(message);
    }
    *cx.should_quit |= cx.switch_to.is_some() || *cx.close_window;
}

/// What the update panel asked of the window: a status line, or the Release
/// notes tab.
fn apply_update_panel(cx: &mut CommandCtx<'_>, event: crate::updates::panel::Event) {
    use crate::updates::panel::Event;
    match event {
        Event::None => {}
        Event::Message(line) => *cx.last_message = Some(line),
        Event::ReleaseNotes => {
            go_to_management(ManagementEntry::ReleaseNotes, &mut tab_ctx!(cx));
            *cx.should_quit |= cx.show_management.is_some();
        }
    }
}

fn handle_key_inner(cx: &mut CommandCtx<'_>, env: &KeyEnv<'_>, key: KeyEvent) -> io::Result<Flow> {
    // The Activity Log is modal and takes every key.
    // `Ctrl+N` starts the badge prefix over it (`activity_log::nav_targets`);
    // the key after it is read by `keys_prefix::navigation`.
    if key.kind == KeyEventKind::Press
        && crate::activity_log::is_open()
        && matches!(*cx.input_mode, InputMode::Normal)
    {
        if env.km.is(Bind::GlobalNavPrefix, &key) && !crate::activity_log::nav_targets().is_empty()
        {
            *cx.input_mode = InputMode::Navigation;
            return Ok(Flow::Handled);
        }
        if let Some(command) = crate::activity_log::on_panel_key(key) {
            apply_card_command(cx, command);
        }
        return Ok(Flow::Handled);
    }
    // So is the update panel.
    if key.kind == KeyEventKind::Press && crate::updates::panel::is_open() {
        let event = crate::updates::panel::on_key(key);
        apply_update_panel(cx, event);
        return Ok(Flow::Handled);
    }
    // So is the jobs panel; its Enter may ask for another repository's tab.
    if key.kind == KeyEventKind::Press && crate::jobs_ui::panel_open() {
        crate::jobs_ui::panel_key(key);
        apply_jobs_goto(cx);
        return Ok(Flow::Handled);
    }
    if modal::handle(cx, env, key)? == Flow::Handled {
        return Ok(Flow::Handled);
    }
    if !matches!(*cx.input_mode, InputMode::Normal) {
        return keys_modes::handle(cx, env, key);
    }
    if keys_normal::handle(cx, env, key)? == Flow::Handled
        || keys_normal_lists::handle(cx, env, key)? == Flow::Handled
    {
        return Ok(Flow::Handled);
    }
    keys_normal_nav::handle(cx, env, key)
}

/// Routes one mouse event. The popups and forms that own the mouse come first,
/// then the surfaces over the panes (`mouse_prelude`), then the handler for the
/// kind of event, which resolves what is under the pointer.
pub(in crate::event_loop) fn handle_mouse(
    cx: &mut CommandCtx<'_>,
    env: &KeyEnv<'_>,
    mouse: MouseEvent,
    forced_hit: &mut Option<HitTarget>,
) -> io::Result<Flow> {
    let slots = open_menu::MenuSlots::of(cx);
    cx.tooltip.take_fed();
    let mut ordinary = false;
    let flow = handle_mouse_inner(cx, env, mouse, forced_hit, &mut ordinary);
    // The ordinary path set the pointer in `mouse_moved`; every surface that
    // owns the mouse returned before it, and sets it here.
    if matches!(mouse.kind, MouseEventKind::Moved) && !ordinary {
        pointer::owned_surface(cx, mouse.column, mouse.row)?;
    }
    // A surface that owns the pointer (the Radar, the Workspaces body, a
    // popup...) returns before `mouse_moved` feeds the tooltip rule, which
    // would keep the tooltip of the target the pointer just left. Feed it
    // here: only a plain hint under the pointer keeps one.
    if matches!(mouse.kind, MouseEventKind::Moved) && !cx.tooltip.take_fed() {
        let key = find_hit_target(&*cx.hit_targets, mouse.column, mouse.row)
            .and_then(|t| crate::tooltip::hint_for(&t))
            .map(crate::tooltip::TooltipKey::Hint);
        cx.tooltip
            .pointer_moved(key, (mouse.column, mouse.row), Instant::now());
    }
    slots.keep_only_the_newest(cx);
    flow
}

fn handle_mouse_inner(
    cx: &mut CommandCtx<'_>,
    env: &KeyEnv<'_>,
    mouse: MouseEvent,
    forced_hit: &mut Option<HitTarget>,
    ordinary: &mut bool,
) -> io::Result<Flow> {
    // The Activity Log window, the status line's error and the status bar's
    // Activity button come before anything else; nothing behind them hovers.
    if let crate::activity_log::Routed::Consumed(command) = crate::activity_log::route_mouse(mouse)
    {
        *cx.hovered_target = None;
        if let Some(command) = command {
            apply_card_command(cx, command);
        }
        return Ok(Flow::Handled);
    }
    // The jobs panel, and the status bar's jobs indicator (its label opens
    // the panel, its [x] cancels).
    if crate::jobs_ui::route_mouse(mouse) {
        *cx.hovered_target = None;
        apply_jobs_goto(cx);
        return Ok(Flow::Handled);
    }
    // A tall context menu's scrollbar and wheel (the popup floats over all).
    if crate::context_menu::on_mouse(mouse) {
        *cx.hovered_target = None;
        return Ok(Flow::Handled);
    }
    // The update panel is modal: it takes every mouse event while open.
    if let Some(event) = crate::updates::panel::route_mouse(mouse) {
        *cx.hovered_target = None;
        apply_update_panel(cx, event);
        return Ok(Flow::Handled);
    }
    // The version chip in the title bar: the update panel when a release is
    // announced, else the Release notes tab.
    // Settings covers the status bar: its stale chip rect must not take a click.
    if crate::updates::chip::route_mouse(mouse)
        && matches!(*cx.input_mode, InputMode::Normal)
        && cx.preferences.is_none()
    {
        *cx.hovered_target = None;
        if crate::updates::chip::opens_panel() {
            crate::updates::panel::open();
        } else {
            apply_update_panel(cx, crate::updates::panel::Event::ReleaseNotes);
        }
        return Ok(Flow::Handled);
    }
    match &*cx.input_mode {
        InputMode::EditingCommitMessage(_) => {
            return mouse_modes::editing_commit_message(cx, mouse)
        }
        InputMode::Guide(_) => return mouse_modes::guide(cx, mouse),
        InputMode::ComposeCommits(_) => return mouse_modes::composer_review(cx, mouse),
        InputMode::SplitCommit(_) => return mouse_modes::split_commit(cx, mouse),
        InputMode::Rewrite(_) => return mouse_modes::rewrite(cx, mouse),
        InputMode::InteractiveRebase(_) => return mouse_modes::interactive_rebase(cx, mouse),
        _ => {}
    }
    let Some(g) = mouse_prelude::prelude(cx, mouse)? else {
        return Ok(Flow::Handled);
    };
    match mouse.kind {
        MouseEventKind::Down(MouseButton::Left) => mouse_left::handle(cx, &g, mouse, forced_hit),
        MouseEventKind::Down(MouseButton::Right) => mouse_right::handle(cx, env, &g, mouse),
        MouseEventKind::Drag(MouseButton::Left) => mouse_drag::drag(cx, &g, mouse),
        MouseEventKind::Up(MouseButton::Left) => mouse_drag::up(cx, &g, mouse),
        MouseEventKind::ScrollLeft
        | MouseEventKind::ScrollRight
        | MouseEventKind::ScrollUp
        | MouseEventKind::ScrollDown => mouse_scroll::handle(cx, &g, mouse),
        MouseEventKind::Moved => {
            *ordinary = true;
            mouse_moved::handle(cx, &g, mouse)
        }
        _ => Ok(Flow::Continue),
    }
}
