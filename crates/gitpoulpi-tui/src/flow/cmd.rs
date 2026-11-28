//! The Flow commands: what the toolbar's `Flow ▾` dropdown, the palette
//! (`Flow: ...`, one `Start <Type>...` per topic type) and the leader keys
//! all run, so no surface can drift from another.
//!
//! The commands themselves never touch the repository: they ask the window
//! (through [`request`] and [`take_request`], the pattern `safety_panel`
//! uses) to open a dialog, the dropdown, the toggle or the Settings page.

#[allow(unused_imports)]
pub(crate) use gitpoulpi_app::palette::vocab::flow::FlowCmd;
use std::sync::Mutex;

use gitpoulpi_core::workflow::BranchKind;
use gitpoulpi_i18n::msg::flow as msg;

use super::view;

/// Why a flow command does nothing right now: it reads the window's workflow
/// state, so it is not part of the shared vocabulary.
pub(crate) trait FlowCmdExt {
    fn unavailable(self) -> Option<&'static str>;
}

impl FlowCmdExt for FlowCmd {
    /// Why the command does nothing right now (`None`: it can run).
    fn unavailable(self) -> Option<&'static str> {
        if self == FlowCmd::Settings {
            return None;
        }
        if self == FlowCmd::Resume {
            return (!super::daemon::has_stopped()).then_some(msg::WHY_NOTHING_STOPPED.get());
        }
        let Some(view) = view::configured() else {
            return Some(msg::WHY_NOT_CONFIGURED.get());
        };
        let head = view
            .report
            .branches
            .iter()
            .find(|b| b.is_head && b.remote.is_none());
        match self {
            FlowCmd::Menu
            | FlowCmd::StartAny
            | FlowCmd::ToggleGrouping
            | FlowCmd::Settings
            | FlowCmd::Resume => None,
            FlowCmd::NewIssueAny | FlowCmd::PickUp if cfg!(not(feature = "hosting")) => {
                Some(msg::ISSUE_WHY_HOSTING.get())
            }
            FlowCmd::NewIssueAny | FlowCmd::PickUp => None,
            FlowCmd::NewIssue(i) if cfg!(not(feature = "hosting")) => {
                let _ = i;
                Some(msg::ISSUE_WHY_HOSTING.get())
            }
            FlowCmd::NewIssue(i) => {
                (usize::from(i) >= view.topics().len()).then_some(msg::WHY_NO_SUCH_TYPE.get())
            }
            FlowCmd::Start(i) => {
                (usize::from(i) >= view.topics().len()).then_some(msg::WHY_NO_SUCH_TYPE.get())
            }
            FlowCmd::Finish => match head {
                Some(b) if b.kind == BranchKind::Topic => None,
                Some(_) => Some(msg::WHY_NOT_A_TOPIC.get()),
                None => Some(msg::WHY_NO_BRANCH.get()),
            },
            FlowCmd::Sync => match head {
                Some(b) if b.parent.is_some() => None,
                Some(_) => Some(msg::WHY_NO_PARENT.get()),
                None => Some(msg::WHY_NO_BRANCH.get()),
            },
            FlowCmd::Promote => {
                let any = view
                    .report
                    .workflow
                    .bases
                    .iter()
                    .any(|b| b.environment && b.parent.is_some());
                (!any).then_some(msg::WHY_NO_ENVIRONMENT.get())
            }
            FlowCmd::SetParent => head.is_none().then_some(msg::WHY_NO_BRANCH.get()),
        }
    }
}

static REQUEST: Mutex<Option<FlowCmd>> = Mutex::new(None);

/// Asks the window to run `cmd` (its next notice pass picks it up).
pub(crate) fn request(cmd: FlowCmd) {
    if let Ok(mut slot) = REQUEST.lock() {
        *slot = Some(cmd);
    }
    crate::redraw::poke();
}

/// The command waiting for the window, once.
pub(crate) fn take_request() -> Option<FlowCmd> {
    REQUEST.lock().ok().and_then(|mut slot| slot.take())
}

/// The rows of the `Flow ▾` dropdown: `(label, command)`.
pub(crate) fn menu_rows() -> Vec<(String, FlowCmd)> {
    let Some(view) = view::configured() else {
        return Vec::new();
    };
    let mut rows = Vec::new();
    for (i, topic) in view.topics().iter().enumerate() {
        if let Ok(i) = u8::try_from(i) {
            rows.push((msg::start_type(topic.display_label()), FlowCmd::Start(i)));
        }
    }
    #[cfg(feature = "hosting")]
    {
        for (i, topic) in view.topics().iter().enumerate() {
            if let Ok(i) = u8::try_from(i) {
                rows.push((
                    msg::issue_menu_new(topic.display_label()),
                    FlowCmd::NewIssue(i),
                ));
            }
        }
        rows.push((msg::ISSUE_MENU_PICK_UP.get().into(), FlowCmd::PickUp));
    }
    rows.push((msg::MENU_FINISH.get().into(), FlowCmd::Finish));
    rows.push((msg::MENU_SYNC.get().into(), FlowCmd::Sync));
    if view
        .report
        .workflow
        .bases
        .iter()
        .any(|b| b.environment && b.parent.is_some())
    {
        rows.push((msg::MENU_PROMOTE.get().into(), FlowCmd::Promote));
    }
    rows.push((msg::MENU_SET_PARENT.get().into(), FlowCmd::SetParent));
    if super::daemon::has_stopped() {
        rows.push((msg::MENU_RESUME.get().into(), FlowCmd::Resume));
    }
    rows.push((
        if view::plain() {
            msg::MENU_GROUP_ON.get().into()
        } else {
            msg::MENU_GROUP_OFF.get().into()
        },
        FlowCmd::ToggleGrouping,
    ));
    rows.push((msg::MENU_SETTINGS.get().into(), FlowCmd::Settings));
    rows
}
