//! The modal popups that own the mouse: the overflow menu, the command palette
//! and tab picker lists, and the commit-message, composer and rebase forms.

use super::commands::{self, CommandCtx, Flow};
use crate::event_loop::*;
use crossterm::event::MouseEvent;
use gitpoulpi_i18n::msg;

/// An open overflow menu owns the keyboard and every mouse press, like the other
/// popup menus. A row chosen by Enter or by a click is turned into a left click
/// at the `⋯` it hangs from, with `forced_hit` naming the hidden button - so the
/// ordinary click path runs that button's action, exactly as if it had been
/// visible. `Flow::Handled` when the menu consumed the event.
pub(in crate::event_loop) fn overflow_menu_event(
    cx: &mut CommandCtx<'_>,
    event: &mut Event,
    forced_hit: &mut Option<HitTarget>,
) -> Flow {
    if let Some(state) = cx.open_menu.overflow_mut() {
        let outcome = match &*event {
            Event::Key(key) if key.kind == KeyEventKind::Press => Some(state.menu.on_key(key.code)),
            Event::Key(_) => Some(MenuOutcome::Stay),
            Event::Mouse(m) if matches!(m.kind, MouseEventKind::Down(_)) => {
                match find_hit_target(&*cx.hit_targets, m.column, m.row) {
                    Some(HitTarget::BarOverflowMenuItem(i))
                        if m.kind == MouseEventKind::Down(MouseButton::Left) =>
                    {
                        Some(state.menu.activate(i))
                    }
                    Some(HitTarget::BarOverflowMenuItem(_)) => Some(MenuOutcome::Stay),
                    // Click-away dismisses without clicking
                    // through, the other menus' rule.
                    _ => Some(MenuOutcome::Close),
                }
            }
            // Pointer motion keeps driving the hover; the rest
            // (wheel, release) is harmless underneath.
            _ => None,
        };
        let button = state.button;
        match outcome {
            Some(MenuOutcome::Run(item)) => {
                cx.open_menu.close_overflow();
                *forced_hit = Some(item.hit_target());
                *event = Event::Mouse(crossterm::event::MouseEvent {
                    kind: MouseEventKind::Down(MouseButton::Left),
                    column: button.x,
                    row: button.y,
                    modifiers: KeyModifiers::NONE,
                });
            }
            Some(MenuOutcome::Close) => {
                cx.open_menu.close_overflow();
                return Flow::Handled;
            }
            Some(MenuOutcome::Stay | MenuOutcome::NotHandled) => return Flow::Handled,
            None => {}
        }
    }
    Flow::Continue
}

/// What a mouse event did to a popup list.
pub(in crate::event_loop) enum ListMouse {
    /// Not over a popup list: carry on.
    Pass,
    /// Consumed (a dismissal, a wheel notch, a click on no row).
    Swallow,
    /// A row was selected: run the list's own `Enter`.
    Enter,
    /// A palette row's "configure the shortcut" button: the palette's own
    /// Alt+K on that row.
    ConfigureKey,
}

fn list_selected(mode: &mut InputMode) -> Option<&mut usize> {
    match mode {
        InputMode::CommandPalette { selected, .. } | InputMode::TabPicker { selected, .. } => {
            Some(selected)
        }
        InputMode::ThemePicker(picker) => Some(&mut picker.selected),
        InputMode::LanguagePicker(picker) => Some(&mut picker.selected),
        InputMode::WorkspaceSearch(search) => Some(&mut search.selected),
        _ => None,
    }
}

/// A tab picker row was pressed and its drag has not ended.
fn picker_drag_armed(cx: &CommandCtx<'_>) -> bool {
    cx.tab_drag
        .as_ref()
        .is_some_and(|d| d.origin_kind == crate::tab_drag::DragFrom::Picker)
}

/// The command palette and the tab picker are modal lists that look clickable,
/// so they own the mouse: a click on a row selects it, a double click runs the keyboard's own
/// `Enter` (a second copy of that code would drift), a click outside
/// dismisses, the wheel scrolls the window, and nothing reaches what is drawn
/// underneath.
pub(in crate::event_loop) fn popup_list_mouse(
    cx: &mut CommandCtx<'_>,
    m: &MouseEvent,
    palette: Option<PalettePopup>,
    picker: Option<PalettePopup>,
) -> ListMouse {
    let popup = match &*cx.input_mode {
        InputMode::CommandPalette { .. } => palette,
        InputMode::TabPicker { .. }
        | InputMode::ThemePicker(_)
        | InputMode::LanguagePicker(_)
        | InputMode::WorkspaceSearch(_) => picker,
        _ => None,
    };
    let Some(popup) = popup else {
        crate::chrome::set_popup_pointer(None);
        return ListMouse::Pass;
    };
    let (mcol, mrow) = (m.column, m.row);
    crate::chrome::set_popup_pointer(Some((mcol, mrow)));
    // The modal list owns the pointer shape: its text field is the I-beam,
    // its cross and chips are hands, its rows and everything behind it the
    // arrow (what the paint registered, never the toolbar underneath).
    if matches!(m.kind, MouseEventKind::Moved) {
        let shape = crate::pointer_map::shape_at(mcol, mrow);
        if shape != *cx.last_cursor_shape
            && cursor::emit_cursor_shape(cx.terminal.backend_mut(), shape).is_ok()
        {
            *cx.last_cursor_shape = shape;
        }
    }
    let inside = popup.popup.contains((mcol, mrow).into());
    let current = list_selected(&mut *cx.input_mode).map_or(0, |s| *s);
    match m.kind {
        // The scrollbar first: its column is the list's right border.
        MouseEventKind::Down(MouseButton::Left)
            if crate::popup_list::View::bar_press(mcol, mrow, current) => {}
        // The theme picker's All / Dark / Light / Colored segments.
        MouseEventKind::Down(MouseButton::Left)
            if matches!(&*cx.input_mode, InputMode::ThemePicker(_))
                && popup
                    .segments
                    .iter()
                    .any(|r| r.contains((mcol, mrow).into())) =>
        {
            if let InputMode::ThemePicker(picker) = &mut *cx.input_mode {
                if let Some(i) = popup
                    .segments
                    .iter()
                    .position(|r| r.contains((mcol, mrow).into()))
                {
                    picker.set_filter(crate::theme_picker::ThemeFilter::ALL[i]);
                }
            }
        }
        // The workspace search's close cross.
        MouseEventKind::Down(MouseButton::Left)
            if matches!(&*cx.input_mode, InputMode::WorkspaceSearch(_))
                && crate::workspace_search::close_at(mcol, mrow) =>
        {
            *cx.input_mode = InputMode::Normal;
        }
        // The workspace search's kind chips (Alt+1..8 are their keys).
        MouseEventKind::Down(MouseButton::Left)
            if matches!(&*cx.input_mode, InputMode::WorkspaceSearch(_))
                && crate::workspace_search::chip_at(mcol, mrow).is_some() =>
        {
            if let (InputMode::WorkspaceSearch(search), Some(slot)) = (
                &mut *cx.input_mode,
                crate::workspace_search::chip_at(mcol, mrow),
            ) {
                search.toggle_chip(slot);
            }
        }
        // A group heading is not a result: nothing to select or open.
        MouseEventKind::Down(MouseButton::Left)
            if matches!(&*cx.input_mode, InputMode::WorkspaceSearch(search)
                if palette_row_at(&popup, mcol, mrow).is_some_and(|i| search.is_heading(i))) => {}
        // A click selects; the second click on the same row validates (the
        // keyboard's own `Enter`).
        MouseEventKind::Down(MouseButton::Left) => match palette_row_at(&popup, mcol, mrow) {
            Some(index)
                if matches!(&*cx.input_mode, InputMode::CommandPalette { .. })
                    && mcol + 1 + crate::palette::KEY_BUTTON_W >= popup.list.right() =>
            {
                if let Some(selected) = list_selected(&mut *cx.input_mode) {
                    *selected = index;
                }
                return ListMouse::ConfigureKey;
            }
            Some(index) => {
                if let Some(selected) = list_selected(&mut *cx.input_mode) {
                    *selected = index;
                }
                // A press on a tab picker row may become a reorder drag.
                *cx.tab_drag = None;
                if let InputMode::TabPicker { draft, .. } = &*cx.input_mode {
                    let from = filter_tab_indices(&*cx.repo_tabs, draft)
                        .get(index)
                        .copied();
                    if let Some((from, tab)) = from.and_then(|f| Some((f, cx.repo_tabs.get(f)?))) {
                        *cx.tab_drag = Some(crate::tab_drag::TabDrag::armed(
                            from,
                            tab.name.clone(),
                            crate::tab_drag::DragFrom::Picker,
                            mcol,
                            mrow,
                        ));
                    }
                }
                if crate::popup_list::View::click(index) {
                    return ListMouse::Enter;
                }
            }
            // Outside the popup dismisses it. Inside but on no row (the input
            // box, the hint band) does nothing, and in particular does not
            // reach what is behind it.
            None if !inside => {
                if let InputMode::ThemePicker(picker) = &mut *cx.input_mode {
                    picker.restore();
                }
                if let InputMode::LanguagePicker(picker) = &mut *cx.input_mode {
                    picker.restore(&mut *cx.ui_graph_theme);
                }
                *cx.input_mode = InputMode::Normal;
            }
            None => {}
        },
        MouseEventKind::Drag(MouseButton::Left) if picker_drag_armed(cx) => {
            if let (InputMode::TabPicker { draft, .. }, Some(drag)) =
                (&*cx.input_mode, cx.tab_drag.as_mut())
            {
                let filtered = filter_tab_indices(&*cx.repo_tabs, draft);
                drag.at = (mcol, mrow);
                if !drag.started && drag.passes_threshold(mcol, mrow) {
                    drag.started = true;
                    drag.source_rect = filtered
                        .iter()
                        .position(|i| *i == drag.from)
                        .and_then(|pos| crate::tab_drag::row_rect(&popup, pos));
                }
                if drag.started {
                    drag.over = palette_row_at(&popup, mcol, mrow).and_then(|pos| {
                        Some(crate::tab_drag::DropAt {
                            to: *filtered.get(pos)?,
                            mark: crate::tab_drag::row_rect(&popup, pos)?,
                            vertical: false,
                        })
                    });
                }
            }
        }
        MouseEventKind::Drag(MouseButton::Left) => crate::popup_list::View::bar_drag(mrow, current),
        MouseEventKind::Up(MouseButton::Left) if picker_drag_armed(cx) => {
            if let Some(drag) = cx.tab_drag.take().filter(|d| d.started) {
                if let Some(over) = drag.over {
                    *cx.last_message = crate::tabs::reorder_tab(
                        cx.session,
                        &mut *cx.repo_tabs,
                        &mut *cx.active_tab,
                        drag.from,
                        over.to,
                    );
                    // The highlight follows the dropped tab.
                    if let InputMode::TabPicker { draft, selected } = &mut *cx.input_mode {
                        if let Some(at) = filter_tab_indices(&*cx.repo_tabs, draft)
                            .iter()
                            .position(|i| *i == over.to)
                        {
                            *selected = at;
                        }
                    }
                }
            }
            crate::popup_list::View::bar_release();
        }
        MouseEventKind::Up(MouseButton::Left) => crate::popup_list::View::bar_release(),
        // The wheel scrolls the window; the highlight stays where it is.
        MouseEventKind::ScrollDown if inside => {
            crate::popup_list::View::wheel(crate::popup_list::WHEEL_STEP as isize, current)
        }
        MouseEventKind::ScrollUp if inside => {
            crate::popup_list::View::wheel(-(crate::popup_list::WHEEL_STEP as isize), current)
        }
        _ => {}
    }
    if let InputMode::ThemePicker(picker) = &mut *cx.input_mode {
        picker.sync_preview();
    }
    if let InputMode::LanguagePicker(picker) = &mut *cx.input_mode {
        picker.sync_preview(&mut *cx.ui_graph_theme);
    }
    ListMouse::Swallow
}

/// The Edit commit message popup is modal: a left click lands on it or is
/// swallowed, and every other mouse event is swallowed outright, so nothing
/// underneath moves while it is open.
pub(super) fn editing_commit_message(
    cx: &mut CommandCtx<'_>,
    mouse: MouseEvent,
) -> io::Result<Flow> {
    // A selection dragged inside the summary or the
    // description follows the pointer until released.
    if let InputMode::EditingCommitMessage(form) = &mut *cx.input_mode {
        match mouse.kind {
            MouseEventKind::ScrollDown => form.on_wheel(3),
            MouseEventKind::ScrollUp => form.on_wheel(-3),
            MouseEventKind::Drag(MouseButton::Left) => {
                form.on_drag(mouse.column, mouse.row);
            }
            MouseEventKind::Up(MouseButton::Left) => form.on_release(),
            _ => {}
        }
    }
    if let (MouseEventKind::Down(MouseButton::Left), InputMode::EditingCommitMessage(form)) =
        (mouse.kind, &mut *cx.input_mode)
    {
        let part = reword_hit_at(&*cx.hit_targets, mouse.column, mouse.row);
        // A click outside the popup is Esc; one inside on no control is not.
        let outcome = match part {
            Some(part) => Some(form.on_press(part, mouse.column, mouse.row)),
            None if form.is_outside(mouse.column, mouse.row) => Some(form.request_close()),
            None => None,
        };
        if let Some(outcome) = outcome {
            match outcome {
                RewordFormOutcome::Stay => {}
                RewordFormOutcome::Close => {
                    *cx.last_message = Some(form.canceled_message().into());
                    *cx.input_mode = InputMode::Normal;
                }
                RewordFormOutcome::Submit(action) => {
                    crate::ui_intent::proposal::reword_confirmed(&action);
                    *cx.last_message = Some(form.submitted_message());
                    cx.link.submit_action(action);
                    *cx.input_mode = InputMode::Normal;
                }
                RewordFormOutcome::ModifyDeeper => {
                    let commit = form.commit;
                    *cx.input_mode = InputMode::Normal;
                    super::keys_prefix::open_rewrite_from_menu(
                        cx,
                        crate::log_menu::RewriteFrom::FromHere,
                        commit,
                    );
                }
            }
        }
    }
    Ok(Flow::Handled)
}

/// The AI commit-composer review: hover, click (the wheel scrolls the list).
pub(super) fn composer_review(cx: &mut CommandCtx<'_>, mouse: MouseEvent) -> io::Result<Flow> {
    // The list's scrollbar takes its own presses and the drag of its thumb.
    if let InputMode::ComposeCommits(form) = &mut *cx.input_mode {
        if form.on_bar_mouse(mouse) {
            return Ok(Flow::Handled);
        }
    }
    match mouse.kind {
        MouseEventKind::Moved => {
            *cx.hovered_target = find_hit_target(&*cx.hit_targets, mouse.column, mouse.row);
        }
        _ if matches!(&*cx.input_mode, InputMode::ComposeCommits(f) if f.is_moving()) => {
            if let InputMode::ComposeCommits(form) = &mut *cx.input_mode {
                form.on_move_mouse(mouse);
            }
        }
        MouseEventKind::Down(MouseButton::Left) => {
            if let (Some(part), InputMode::ComposeCommits(form)) = (
                compose_hit_at(&*cx.hit_targets, mouse.column, mouse.row),
                &mut *cx.input_mode,
            ) {
                match form.on_press(part, mouse.column, mouse.row) {
                    ComposeOutcome::Stay => {}
                    ComposeOutcome::Close => {
                        *cx.input_mode = InputMode::Normal;
                        *cx.last_message = Some(msg::input::COMPOSER_CANCELED.get().into());
                    }
                    ComposeOutcome::Submit(action) => {
                        *cx.last_message = Some(msg::input::CREATING_COMMITS.get().into());
                        cx.link.submit_action(action);
                        *cx.input_mode = InputMode::Normal;
                    }
                }
            }
        }
        MouseEventKind::Drag(MouseButton::Left) => {
            let part = compose_hit_at(&*cx.hit_targets, mouse.column, mouse.row);
            if let InputMode::ComposeCommits(form) = &mut *cx.input_mode {
                if !form.on_text_drag(mouse.column, mouse.row) {
                    form.on_file_drag(part);
                }
            }
        }
        MouseEventKind::Up(MouseButton::Left) => {
            let part = compose_hit_at(&*cx.hit_targets, mouse.column, mouse.row);
            if let InputMode::ComposeCommits(form) = &mut *cx.input_mode {
                form.on_release();
                form.on_drop(part);
            }
        }
        // The wheel scrolls the list; the selection stays where it is.
        MouseEventKind::ScrollUp | MouseEventKind::ScrollDown => {
            if let InputMode::ComposeCommits(form) = &mut *cx.input_mode {
                form.on_wheel(if mouse.kind == MouseEventKind::ScrollUp {
                    -3
                } else {
                    3
                });
            }
        }
        _ => {}
    }
    Ok(Flow::Handled)
}

/// The split-commit editor: modal like the other full views. Open, the
/// list of commits and the message editor own the pointer; a press on a part
/// acts, a press outside the frame is a dismissal (swallowed when the plan
/// was edited), the wheel scrolls the changes.
pub(super) fn split_commit(cx: &mut CommandCtx<'_>, mouse: MouseEvent) -> io::Result<Flow> {
    use crate::split_ui::{split_hit_at, SplitFormOutcome, SplitHit};
    let InputMode::SplitCommit(form) = &mut *cx.input_mode else {
        return Ok(Flow::Handled);
    };
    if form.assign_combo_open() {
        form.on_combo_mouse(mouse);
        *cx.hovered_target = None;
        return Ok(Flow::Handled);
    }
    if form.changes_bar_mouse(mouse) {
        return Ok(Flow::Handled);
    }
    let part = split_hit_at(&*cx.hit_targets, mouse.column, mouse.row);
    let mut outcome = SplitFormOutcome::Stay;
    match mouse.kind {
        MouseEventKind::Moved => {
            *cx.hovered_target = find_hit_target(&*cx.hit_targets, mouse.column, mouse.row);
        }
        MouseEventKind::Down(MouseButton::Left) => match part {
            Some(part) => outcome = form.on_press(part, mouse.column, mouse.row),
            None if form.is_outside(mouse.column, mouse.row) && form.editing().is_none() => {
                outcome = form.request_dismiss(crate::dialog_kit::DismissKind::Outside);
            }
            None => {}
        },
        MouseEventKind::Drag(MouseButton::Left) => {
            if let Some(editor) = form.editing_mut() {
                editor.on_drag(mouse.column, mouse.row);
            } else if let Some(SplitHit::Row(row) | SplitHit::Assign(row)) = part {
                form.extend_selection_to(row);
            }
        }
        MouseEventKind::Up(MouseButton::Left) => {
            if let Some(editor) = form.editing_mut() {
                editor.on_release();
            }
        }
        MouseEventKind::ScrollUp | MouseEventKind::ScrollDown => {
            let step = if mouse.kind == MouseEventKind::ScrollUp {
                -3
            } else {
                3
            };
            if let Some(editor) = form.editing_mut() {
                editor.on_wheel(step);
            } else {
                form.scroll_changes(step);
            }
        }
        _ => {}
    }
    match outcome {
        SplitFormOutcome::Stay => {}
        SplitFormOutcome::Close => {
            let back = form.return_to();
            *cx.input_mode = InputMode::Normal;
            if let Some(upstream) = back {
                commands::reopen_rebase(cx, upstream);
            }
        }
        SplitFormOutcome::Submit(action) => {
            crate::ui_intent::proposal::split_confirmed(&action);
            *cx.last_message = Some(msg::split_ui::SPLITTING.get().into());
            cx.link.submit_action(action);
            form.begin_applying();
        }
    }
    Ok(Flow::Handled)
}

/// The history rewrite workbench: modal like the other full views. A press
/// on a part acts (an open dropdown or question owns the pointer first), a
/// press outside the frame is a dismissal, the wheel moves the cursor of the
/// pane under the pointer.
pub(super) fn rewrite(cx: &mut CommandCtx<'_>, mouse: MouseEvent) -> io::Result<Flow> {
    use crate::rewrite_view::{rewrite_hit_at, RewriteHit, RewriteKeyOutcome};
    let part = rewrite_hit_at(&*cx.hit_targets, mouse.column, mouse.row);
    let mut outcome = RewriteKeyOutcome::Stay;
    {
        let InputMode::Rewrite(view) = &mut *cx.input_mode else {
            return Ok(Flow::Handled);
        };
        // The folder picker of an extraction owns the pointer.
        if view.has_target_picker() {
            view.target_mouse(mouse);
            return Ok(Flow::Handled);
        }
        match mouse.kind {
            MouseEventKind::Moved => {
                *cx.hovered_target = find_hit_target(&*cx.hit_targets, mouse.column, mouse.row);
            }
            MouseEventKind::Down(MouseButton::Left) => match part {
                Some(part) => outcome = view.on_press(part, mouse.column, mouse.row),
                None if view.is_outside(mouse.column, mouse.row) => {
                    outcome = view.outside_press();
                }
                None => {}
            },
            // A held scrollbar thumb follows the pointer, even off the bar.
            MouseEventKind::Drag(MouseButton::Left) => {
                view.bar_drag(mouse.row);
            }
            MouseEventKind::Up(MouseButton::Left) => {
                view.bar_release();
            }
            MouseEventKind::ScrollUp | MouseEventKind::ScrollDown => {
                let step = if mouse.kind == MouseEventKind::ScrollUp {
                    -3
                } else {
                    3
                };
                match part {
                    Some(hit) if hit.is_purge() => view.purge_wheel(step),
                    Some(RewriteHit::ScriptEditor | RewriteHit::ScriptRow(_)) => {
                        view.on_script_wheel(part, step)
                    }
                    Some(RewriteHit::AiSent) => view.on_ai_wheel(step),
                    Some(RewriteHit::RaSent | RewriteHit::RaPanel) => view.on_rules_ai_wheel(step),
                    Some(RewriteHit::DrillDiff) => view.drill_scroll(step),
                    Some(RewriteHit::DrillPath(_)) => {
                        view.wheel_id(crate::rewrite_view::BarId::DrillFiles, step);
                    }
                    Some(RewriteHit::Pane(focus)) => view.on_wheel(focus, mouse.row, step),
                    Some(RewriteHit::Rule(_) | RewriteHit::RuleCheck(_)) => {
                        view.on_wheel(crate::rewrite_view::Focus::Rules, mouse.row, step)
                    }
                    Some(RewriteHit::Field(_)) => {
                        view.on_wheel(crate::rewrite_view::Focus::Detail, mouse.row, step)
                    }
                    Some(
                        RewriteHit::Row(_)
                        | RewriteHit::ProtectedRow(_)
                        | RewriteHit::PushRow(_)
                        | RewriteHit::SecretRow(_)
                        | RewriteHit::SizeRow(_)
                        | RewriteHit::IdentRow(_),
                    ) => view.on_wheel(crate::rewrite_view::Focus::Preview, mouse.row, step),
                    Some(RewriteHit::HostLink(_) | RewriteHit::HostPanel) => {
                        view.wheel_id(crate::rewrite_view::BarId::Howto, step);
                    }
                    Some(RewriteHit::IdentPick(_)) => {
                        view.wheel_id(crate::rewrite_view::BarId::IdentPick, step);
                    }
                    Some(RewriteHit::PopupRow(_)) => {
                        view.wheel_id(crate::rewrite_view::BarId::Popup, step);
                    }
                    Some(RewriteHit::Bar(id)) => {
                        view.wheel_id(id, step);
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
    match outcome {
        RewriteKeyOutcome::Stay => {}
        RewriteKeyOutcome::Close => {
            if let InputMode::Rewrite(view) = &mut *cx.input_mode {
                view.close();
            }
            *cx.input_mode = InputMode::Normal;
        }
        RewriteKeyOutcome::Undo => {
            // The workbench closes: a second Undo here would take back an
            // older entry than the rewrite it showed.
            if let InputMode::Rewrite(view) = &mut *cx.input_mode {
                view.close();
            }
            *cx.input_mode = InputMode::Normal;
            commands::run_command(cx, PaletteCommand::Undo);
        }
        RewriteKeyOutcome::FetchReview => {
            if let InputMode::Rewrite(view) = &mut *cx.input_mode {
                view.close();
            }
            *cx.input_mode = InputMode::Normal;
            commands::run_command(cx, PaletteCommand::Fetch);
        }
    }
    Ok(Flow::Handled)
}

/// The interactive rebase / cherry-pick popup, same rule as the others.
pub(super) fn interactive_rebase(cx: &mut CommandCtx<'_>, mouse: MouseEvent) -> io::Result<Flow> {
    // The list's scrollbar takes its own presses and the drag of its thumb;
    // the wheel scrolls the list under the pointer.
    if let InputMode::InteractiveRebase(form) = &mut *cx.input_mode {
        if form.on_bar_mouse(mouse) {
            return Ok(Flow::Handled);
        }
        match mouse.kind {
            MouseEventKind::ScrollUp => form.on_wheel(-3),
            MouseEventKind::ScrollDown => form.on_wheel(3),
            _ => {}
        }
    }
    match (mouse.kind, &mut *cx.input_mode) {
        (MouseEventKind::Moved, _) => {
            *cx.hovered_target = find_hit_target(&*cx.hit_targets, mouse.column, mouse.row);
        }
        (MouseEventKind::Drag(MouseButton::Left), InputMode::InteractiveRebase(form)) => {
            if !form.on_text_drag(mouse.column, mouse.row) {
                let part = rebase_hit_at(&*cx.hit_targets, mouse.column, mouse.row);
                form.on_drag(part);
            }
        }
        (MouseEventKind::Up(MouseButton::Left), InputMode::InteractiveRebase(form)) => {
            form.on_release();
        }
        _ => {}
    }
    if let (MouseEventKind::Down(MouseButton::Left), InputMode::InteractiveRebase(form)) =
        (mouse.kind, &mut *cx.input_mode)
    {
        if let Some(part) = rebase_hit_at(&*cx.hit_targets, mouse.column, mouse.row) {
            match form.on_press(part, mouse.column, mouse.row) {
                RebaseFormOutcome::Stay => {}
                RebaseFormOutcome::Close => {
                    *cx.input_mode = InputMode::Normal;
                    *cx.last_message = Some(msg::input::REBASE_CANCELED.get().into());
                }
                RebaseFormOutcome::Submit(action) => {
                    *cx.last_message = Some(msg::input::RUNNING_PLAN.get().into());
                    cx.link.submit_action(action);
                    *cx.input_mode = InputMode::Normal;
                }
                RebaseFormOutcome::Split(commit, upstream) => {
                    commands::split_from_rebase(cx, commit, upstream)
                }
            }
        }
    }
    Ok(Flow::Handled)
}
