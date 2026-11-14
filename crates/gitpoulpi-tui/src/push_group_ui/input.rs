//! Keys and mouse of the dialog. Every mouse action has its key: rows by
//! Up/Down/Ctrl+J/K or 1-9, the combo by Enter/Space, the action by
//! Left/Right, `s` skips the row, `f` asks the remotes again, Tab walks the
//! buttons, Ctrl+Enter runs, Esc closes the combo and then the dialog.

use crossterm::event::{
    KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};
use gitpoulpi_core::domain::{GroupAction, GroupMode, OperationKind};
use gitpoulpi_i18n::msg::push_group as msg;

use super::model::{action_from_key, Btn, Part, Phase};
use super::{Core, Dlg, Effect, Step};
use crate::combo::ComboOutcome;
use crate::daemon_link::Action;
use crate::modes::PendingDestructive;

/// The buttons of the open step, in the order they are painted: the label,
/// whether it is the primary one and whether it can be pressed.
pub(super) fn buttons(
    dlg: &Dlg,
    run: Option<&super::model::RunState>,
) -> Vec<(Btn, String, bool, bool)> {
    let label = |m: gitpoulpi_i18n::M| m.get().to_string();
    match &dlg.step {
        Step::Select(s) => vec![
            (
                Btn::Next,
                label(msg::BTN_NEXT),
                true,
                s.ready && s.rows.iter().any(|r| r.checked),
            ),
            (Btn::Cancel, label(msg::BTN_CANCEL), false, true),
        ],
        Step::Plan(p) => vec![
            (Btn::FetchAgain, label(msg::BTN_FETCH_AGAIN), false, true),
            (
                Btn::SkipProblems,
                label(msg::BTN_SKIP_PROBLEMS),
                false,
                p.ready(),
            ),
            (Btn::Cancel, label(msg::BTN_CANCEL), false, true),
            (Btn::Run, label(msg::BTN_RUN), true, p.ready()),
        ],
        Step::Run => {
            let going = run.is_some_and(|r| r.going());
            if going {
                vec![
                    (Btn::Cancel, label(msg::BTN_CANCEL), false, true),
                    (Btn::Close, label(msg::BTN_CLOSE), true, true),
                ]
            } else {
                let mut out = Vec::new();
                if run.is_some_and(|r| r.has_retryable()) {
                    out.push((Btn::Retry, label(msg::BTN_RETRY), false, true));
                }
                out.push((Btn::Close, label(msg::BTN_CLOSE), true, true));
                out
            }
        }
    }
}

impl Core {
    pub(super) fn on_key(&mut self, key: KeyEvent) -> Effect {
        if key.kind != KeyEventKind::Press {
            return Effect::None;
        }
        let Some(dlg) = self.dlg.as_mut() else {
            return Effect::None;
        };
        // An open combo takes every key.
        if let Step::Plan(p) = &mut dlg.step {
            if let Some(row) = p.rows.iter_mut().find(|r| r.combo.is_open()) {
                if let ComboOutcome::Picked(value) = row.combo.on_key(key) {
                    if let Some(action) = action_from_key(&value) {
                        row.choose(action);
                    }
                }
                return Effect::None;
            }
        }
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        match key.code {
            KeyCode::Esc => {
                self.dlg = None;
                return Effect::None;
            }
            KeyCode::Enter if ctrl => return self.primary(),
            KeyCode::Tab | KeyCode::BackTab => {
                let n = buttons(dlg, self.run.as_ref()).len();
                dlg.btn_focus = match (dlg.btn_focus, key.code == KeyCode::BackTab) {
                    (None, false) => Some(0),
                    (None, true) => Some(n.saturating_sub(1)),
                    (Some(i), false) => (i + 1 < n).then_some(i + 1),
                    (Some(i), true) => i.checked_sub(1),
                };
                return Effect::None;
            }
            KeyCode::Enter if dlg.btn_focus.is_some() => {
                let list = buttons(dlg, self.run.as_ref());
                let pressed = dlg.btn_focus.and_then(|i| list.get(i).cloned());
                return match pressed {
                    Some((btn, _, _, true)) => self.press(btn),
                    _ => Effect::None,
                };
            }
            _ => {}
        }
        match &mut dlg.step {
            Step::Select(s) => {
                let n = s.rows.len();
                if let Some(to) =
                    crate::popup_list::nav(&key, s.cursor, n, s.scroll.viewport.get().max(1))
                {
                    s.cursor = to;
                    return Effect::None;
                }
                match key.code {
                    KeyCode::Char(' ') => {
                        if let Some(r) = s.rows.get_mut(s.cursor) {
                            r.checked = !r.checked;
                        }
                    }
                    KeyCode::Char('m' | 'M') => s.remember = !s.remember,
                    KeyCode::Char(d @ '1'..='9') if !ctrl => {
                        let i = d as usize - '1' as usize;
                        if i < n {
                            s.cursor = i;
                        }
                    }
                    KeyCode::Enter => return self.press(Btn::Next),
                    _ => {}
                }
                Effect::None
            }
            Step::Plan(p) => {
                let n = p.rows.len();
                if let Some(to) =
                    crate::popup_list::nav(&key, p.cursor, n, p.scroll.viewport.get().max(1))
                {
                    p.cursor = to;
                    return Effect::None;
                }
                match key.code {
                    KeyCode::Enter | KeyCode::Char(' ') => {
                        if let Some(r) = p.rows.get_mut(p.cursor) {
                            r.combo.open();
                        }
                    }
                    KeyCode::Left => {
                        if let Some(r) = p.rows.get_mut(p.cursor) {
                            r.step(-1);
                        }
                    }
                    KeyCode::Right => {
                        if let Some(r) = p.rows.get_mut(p.cursor) {
                            r.step(1);
                        }
                    }
                    KeyCode::Char('s' | 'S') if !ctrl => {
                        if let Some(r) = p.rows.get_mut(p.cursor) {
                            r.choose(GroupAction::Skip);
                        }
                    }
                    KeyCode::Char('f' | 'F') if !ctrl => return self.press(Btn::FetchAgain),
                    KeyCode::Char('t' | 'T') if !ctrl && p.mode == GroupMode::Push => {
                        p.tags = !p.tags
                    }
                    KeyCode::Char(d @ '1'..='9') if !ctrl => {
                        let i = d as usize - '1' as usize;
                        if i < n {
                            p.cursor = i;
                        }
                    }
                    _ => {}
                }
                Effect::None
            }
            Step::Run => {
                if matches!(key.code, KeyCode::Char('r' | 'R'))
                    && self
                        .run
                        .as_ref()
                        .is_some_and(|r| !r.going() && r.has_retryable())
                {
                    return self.press(Btn::Retry);
                }
                if key.code == KeyCode::Enter {
                    self.dlg = None;
                }
                Effect::None
            }
        }
    }

    /// Ctrl+Enter: the primary button of the step.
    fn primary(&mut self) -> Effect {
        let Some(dlg) = self.dlg.as_ref() else {
            return Effect::None;
        };
        let list = buttons(dlg, self.run.as_ref());
        match list.into_iter().find(|b| b.2 && b.3) {
            Some((btn, ..)) => self.press(btn),
            None => Effect::None,
        }
    }

    /// A button, by mouse or key.
    pub(super) fn press(&mut self, btn: Btn) -> Effect {
        let Some(requester) = self.requester.clone() else {
            return Effect::None;
        };
        let Some(dlg) = self.dlg.as_mut() else {
            return Effect::None;
        };
        let branch = dlg.branch.clone();
        match btn {
            Btn::Close => {
                self.dlg = None;
                Effect::None
            }
            Btn::Cancel => {
                if matches!(dlg.step, Step::Run) {
                    for job in crate::jobs_ui::cancellable()
                        .into_iter()
                        .filter(|j| j.kind == OperationKind::RemoteGroupRun)
                    {
                        crate::jobs_ui::cancel(job.id);
                    }
                } else {
                    self.dlg = None;
                }
                Effect::None
            }
            Btn::Next => {
                let Step::Select(s) = &dlg.step else {
                    return Effect::None;
                };
                if !s.ready || s.targets().is_empty() {
                    return Effect::None;
                }
                let targets = s.targets();
                let remember = s.remember.then(|| s.remember_action(&branch)).flatten();
                let plan = self.ask_plan(&requester, &branch, Some(targets), GroupMode::Push);
                if let Some(d) = self.dlg.as_mut() {
                    d.step = Step::Plan(plan);
                    d.btn_focus = None;
                }
                match remember {
                    Some(action) => Effect::Submit(vec![action]),
                    None => Effect::None,
                }
            }
            Btn::FetchAgain => {
                let Step::Plan(p) = &dlg.step else {
                    return Effect::None;
                };
                let targets = p.targets.clone();
                let tags = p.tags;
                let mode = p.mode;
                let mut plan = self.ask_plan(&requester, &branch, targets, mode);
                plan.tags = tags;
                if let Some(d) = self.dlg.as_mut() {
                    d.step = Step::Plan(plan);
                }
                Effect::None
            }
            Btn::SkipProblems => {
                if let Step::Plan(p) = &mut dlg.step {
                    p.skip_problems();
                }
                Effect::None
            }
            Btn::Run => {
                let Step::Plan(p) = &dlg.step else {
                    return Effect::None;
                };
                if !p.ready() {
                    return Effect::None;
                }
                let request = p.request(&branch);
                let force = p.force_rows();
                if force.is_empty() {
                    self.begin_run(&request);
                    return Effect::Submit(vec![Action::RemoteGroupRun(Box::new(request))]);
                }
                let total: u32 = force.iter().map(|(_, n)| *n).sum();
                let remotes = force.iter().map(|(r, _)| *r).collect::<Vec<_>>().join(", ");
                Effect::Confirm {
                    prompt: msg::force_confirm(u64::from(total), &branch, remotes),
                    pending: PendingDestructive::GroupForcePush {
                        request: Box::new(request),
                    },
                }
            }
            Btn::Retry => {
                let run_id = self.run.as_ref().and_then(|r| r.run_id.clone());
                let prefer = super::prefer_rebase();
                if let Some(run) = self.run.as_mut() {
                    run.phase = Phase::Going;
                }
                Effect::Submit(vec![Action::RemoteGroupRetry {
                    expect_run_id: run_id,
                    prefer_rebase: prefer,
                }])
            }
        }
    }

    pub(super) fn on_mouse(&mut self, mouse: MouseEvent) -> Effect {
        let Some(dlg) = self.dlg.as_mut() else {
            return Effect::None;
        };
        let at = ratatui::layout::Position::new(mouse.column, mouse.row);
        // An open combo is modal: its popup first, a click away closes it
        // and is consumed.
        if let Step::Plan(p) = &mut dlg.step {
            if let Some(row) = p.rows.iter_mut().find(|r| r.combo.is_open()) {
                match row.combo.on_mouse(mouse) {
                    ComboOutcome::Picked(value) => {
                        if let Some(action) = action_from_key(&value) {
                            row.choose(action);
                        }
                    }
                    ComboOutcome::Ignored => {
                        if matches!(mouse.kind, MouseEventKind::Down(_)) {
                            row.combo.close();
                        }
                    }
                    _ => {}
                }
                return Effect::None;
            }
        }
        // The table's scrollbar.
        let (mut scroll, len) = match &mut dlg.step {
            Step::Select(s) => (Some(&mut s.scroll), s.rows.len()),
            Step::Plan(p) => (Some(&mut p.scroll), p.rows.len()),
            Step::Run => (None, 0),
        };
        if let Some(scroll) = scroll.as_deref_mut() {
            if let Some(top) = scroll
                .drag
                .on_mouse(scroll.bar.get(), mouse, len, scroll.top.get())
            {
                scroll.top.set(top);
                return Effect::None;
            }
        }
        match mouse.kind {
            MouseEventKind::ScrollUp | MouseEventKind::ScrollDown => {
                let delta = if mouse.kind == MouseEventKind::ScrollUp {
                    -1
                } else {
                    1
                };
                if let Some(scroll) = scroll.filter(|_| dlg.frame.contains(at)) {
                    let view = scroll.viewport.get().max(1);
                    let max = len.saturating_sub(view);
                    scroll
                        .top
                        .set((scroll.top.get() as isize + delta).clamp(0, max as isize) as usize);
                }
                Effect::None
            }
            MouseEventKind::Moved | MouseEventKind::Drag(_) => {
                dlg.close.set_hover(mouse.column, mouse.row);
                dlg.hover = crate::dialog_kit::hit_at(&dlg.hits, mouse.column, mouse.row);
                Effect::None
            }
            MouseEventKind::Down(MouseButton::Left) => {
                if dlg.close.hit(mouse.column, mouse.row) {
                    self.dlg = None;
                    return Effect::None;
                }
                match crate::dialog_kit::hit_at(&dlg.hits, mouse.column, mouse.row) {
                    Some(part) => self.click(part),
                    None => {
                        // A click away from the frame is swallowed and
                        // makes the cross blink; inside it, nothing.
                        if !dlg.frame.contains(at) {
                            dlg.close.request_flash();
                        }
                        Effect::None
                    }
                }
            }
            _ => Effect::None,
        }
    }

    fn click(&mut self, part: Part) -> Effect {
        let Some(dlg) = self.dlg.as_mut() else {
            return Effect::None;
        };
        match (part, &mut dlg.step) {
            (Part::Button(btn), _) => return self.press(btn),
            (Part::Row(i), Step::Select(s)) => s.cursor = usize::from(i),
            (Part::Row(i), Step::Plan(p)) => p.cursor = usize::from(i),
            (Part::Check(i), Step::Select(s)) => {
                s.cursor = usize::from(i);
                if let Some(r) = s.rows.get_mut(usize::from(i)) {
                    r.checked = !r.checked;
                }
            }
            (Part::Combo(i), Step::Plan(p)) => {
                p.cursor = usize::from(i);
                if let Some(r) = p.rows.get_mut(usize::from(i)) {
                    r.combo.open();
                }
            }
            (Part::Remember, Step::Select(s)) => s.remember = !s.remember,
            (Part::Tags, Step::Plan(p)) => p.tags = !p.tags,
            _ => {}
        }
        Effect::None
    }
}
