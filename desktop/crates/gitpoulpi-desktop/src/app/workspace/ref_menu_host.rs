//! The ref menu as a part of the window: opening it (a right click, a drop, the menu key), what its
//! events do, the overlay it is drawn in, and the auto-stash question.

use gitpoulpi_i18n::msg;
use gitpoulpi_link::Action;
use gpui_kit::{
    anchored, deferred, div, px, Anchor, AnyElement, AppContext as _, Context, Entity,
    InteractiveElement as _, IntoElement, ParentElement as _, Pixels, Point, Subscription, Window,
};

use super::Workspace;
use crate::app::actions::with_workspace;
use crate::daemon::refs_ops as rules;
use crate::dialogs::{self, Choice, Spec};
use crate::ref_menu::{self, MenuFacts, RefMenu, RefMenuEvent};
use crate::ref_target::RefTarget;

/// An open ref menu: what it acts on, where the keyboard goes back to, and its subscription.
pub(in crate::app) struct OpenMenu {
    pub(super) view: Entity<RefMenu>,
    target: RefTarget,
    /// For a drop menu, the ref that was dropped.
    source: Option<RefTarget>,
    back_to: Option<gpui_kit::FocusHandle>,
    _sub: Subscription,
}

impl Workspace {
    /// Where a menu opened by the keyboard appears: under the selected Refs row.
    pub(super) fn menu_origin(&self, cx: &gpui_kit::App) -> Point<Pixels> {
        self.refs
            .read(cx)
            .selected_row_origin()
            .unwrap_or_else(|| Point::new(px(48.0), px(120.0)))
    }

    /// The menu key on the pane that has the keyboard: the selected Refs row, else the first ref pill of
    /// the Log's selected commit.
    pub(super) fn open_menu_for_selection(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let in_refs = {
            use gpui_kit::Focusable as _;
            self.refs
                .read(cx)
                .focus_handle(cx)
                .contains_focused(window, cx)
        };
        if in_refs {
            if let Some(target) = self.refs.read(cx).selected_target() {
                let at = self.menu_origin(cx);
                return self.open_ref_menu(target, at, window, cx);
            }
        } else if let Some(target) = self.log.read(cx).first_ref_target() {
            let at = self.log.read(cx).selected_row_origin();
            return self.open_ref_menu(target, at, window, cx);
        }
        self.say(msg::input::NOTHING_FOR_MENU.get(), cx);
    }

    /// Opens the menu of `target` at `at`.
    pub fn open_ref_menu(
        &mut self,
        target: RefTarget,
        at: Point<Pixels>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let snap = self.snapshot(cx);
        let (hidden, solo) = self.refs.read(cx).hidden_or_solo(&target);
        let items = ref_menu::items_for(&target, &snap, MenuFacts { hidden, solo });
        self.show_menu(
            move |cx| RefMenu::new(items, at, cx),
            target,
            None,
            window,
            cx,
        );
    }

    /// Opens the menu a drop of `source` on `target` offers.
    pub fn open_drop_menu(
        &mut self,
        source: RefTarget,
        target: RefTarget,
        at: Point<Pixels>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let snap = self.snapshot(cx);
        let rows = ref_menu::drop_items(&snap, &source, &target);
        if rows.is_empty() {
            let text = msg::menu_drop::cannot_drop(source.name(), target.name());
            return self.say(text, cx);
        }
        let title = format!("{} \u{25B8} {}", source.name(), target.name());
        self.show_menu(
            move |cx| RefMenu::for_drop(title, rows, at, cx),
            target,
            Some(source),
            window,
            cx,
        );
    }

    pub(super) fn show_menu(
        &mut self,
        build: impl FnOnce(&mut Context<RefMenu>) -> RefMenu,
        target: RefTarget,
        source: Option<RefTarget>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let back_to = window.focused(cx);
        let view = cx.new(build);
        let sub = cx.subscribe_in(&view, window, |this, _, ev: &RefMenuEvent, window, cx| {
            this.on_menu_event(ev, window, cx)
        });
        use gpui_kit::Focusable as _;
        let focus = view.read(cx).focus_handle(cx);
        self.ref_menu = Some(OpenMenu {
            view,
            target,
            source,
            back_to,
            _sub: sub,
        });
        window.defer(cx, move |window, cx| focus.focus(window, cx));
        cx.notify();
    }

    pub(super) fn on_menu_event(
        &mut self,
        ev: &RefMenuEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match ev {
            RefMenuEvent::Refused(why) => self.say(why.clone(), cx),
            RefMenuEvent::Close => self.close_ref_menu(window, cx),
            RefMenuEvent::Run(op) => {
                let target = self.ref_menu.as_ref().map(|m| m.target.clone());
                self.close_ref_menu(window, cx);
                self.run_ref_op(*op, target, window, cx);
            }
            RefMenuEvent::Drop(op) => {
                let pair = self
                    .ref_menu
                    .as_ref()
                    .and_then(|m| m.source.clone().map(|s| (s, m.target.clone())));
                self.close_ref_menu(window, cx);
                if let Some((source, target)) = pair {
                    let plan = rules::drop_plan(&self.snapshot(cx), *op, &source, &target);
                    self.carry_out(plan, window, cx);
                }
            }
        }
    }

    /// Closes the menu and gives the keyboard back to where it was.
    pub fn close_ref_menu(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(menu) = self.ref_menu.take() {
            if let Some(handle) = menu.back_to {
                handle.focus(window, cx);
            }
            cx.notify();
        }
    }

    /// The open ref menu, for tests and the window.
    pub fn ref_menu(&self) -> Option<&Entity<RefMenu>> {
        self.ref_menu.as_ref().map(|m| &m.view)
    }

    /// The menu drawn over the window, at its position.
    pub(in crate::app) fn menu_overlay(&self, cx: &gpui_kit::App) -> Option<AnyElement> {
        let menu = self.ref_menu.as_ref()?;
        let at = menu.view.read(cx).position;
        Some(
            deferred(
                anchored()
                    .position(at)
                    .anchor(Anchor::TopLeft)
                    .snap_to_window_with_margin(px(8.0))
                    .child(div().occlude().child(menu.view.clone())),
            )
            .with_priority(2)
            .into_any_element(),
        )
    }

    /// Asks the auto-stash question for each checkout, merge, rebase or pull the daemon refused because
    /// local changes are in the way: stash them and continue (the stash is popped again afterwards), or
    /// cancel. Nothing is highlighted until a key is pressed.
    pub(in crate::app) fn ask_autostash(
        &mut self,
        needed: Vec<gitpoulpi_link::AutoStashNeeded>,
        cx: &mut Context<Self>,
    ) {
        for n in needed {
            with_workspace(cx, move |ws, window, cx| {
                use msg::remotes as m;
                let gitpoulpi_link::AutoStashNeeded {
                    operation,
                    action,
                    ticket,
                } = n;
                ws.say(m::autostash_message(&operation), cx);
                let spec = Spec {
                    title: m::autostash_title(&operation),
                    lines: vec![m::autostash_message(&operation)],
                    choices: vec![
                        Choice::new(m::AUTOSTASH_CONTINUE.get()),
                        Choice::new(msg::dialog::CANCEL.get()),
                    ],
                    hint: msg::diff_actions::CONFIRM_HINT.get().to_string(),
                    highlight_first: false,
                    yes_no_keys: false,
                };
                dialogs::open(
                    window,
                    cx,
                    spec,
                    move |ix, cx| {
                        if ix == 0 {
                            with_workspace(cx, move |ws, _, cx| {
                                let again = Action::AutoStashed(Box::new(action));
                                ws.backend.with_link(|link| link.submit_as(ticket, again));
                                ws.say(m::STASHING_CONTINUING.get(), cx);
                            });
                        } else {
                            gitpoulpi_link::report_ticket(
                                ticket,
                                gitpoulpi_link::StepOutcome::Canceled,
                            );
                        }
                    },
                    move || {
                        gitpoulpi_link::report_ticket(ticket, gitpoulpi_link::StepOutcome::Canceled)
                    },
                );
            });
        }
    }
}
