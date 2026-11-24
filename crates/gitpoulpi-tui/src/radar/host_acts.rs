//! The Radar's writes on the host: Merge, Close, Mark as ready, and the Clone
//! of a repository that is not cloned.
//!
//! Merge and Close change the host and cannot be undone from here, so both
//! stop at a confirmation that names the request, its branches and (for a
//! merge) the method. Mark as ready is reversible on the host (the request can
//! be made a draft again), like the other metadata edits it asks for nothing.
//! The call runs on its own thread; the result comes back through `poll`, which
//! asks every host again so the row changes (a merged or closed request leaves
//! the list, a ready one loses its draft chip).

use std::sync::mpsc;

use gitpoulpi_hosting::{HostingClient, MergeMethod};

use super::*;
use crate::dialog_kit::{Confirm, ConfirmEvent};

/// What a button of the confirmation answers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Choice {
    Merge(MergeMethod),
    Close,
    Cancel,
}

/// The write to make.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum HostAct {
    Merge(MergeMethod),
    Close,
    Ready,
}

/// Everything the thread needs, as plain text.
#[derive(Debug, Clone)]
pub(super) struct HostCall {
    pub(super) act: HostAct,
    pub(super) host_id: String,
    pub(super) project: String,
    pub(super) number: u64,
    /// A request rather than an issue.
    pub(super) pr: bool,
    pub(super) label: String,
}

/// The confirmation open over the list.
pub(super) struct Asked {
    dialog: Confirm<Choice>,
    call: HostCall,
}

/// A call in flight.
pub(super) struct Acting {
    rx: mpsc::Receiver<Result<(), String>>,
    call: HostCall,
}

/// Why `act` cannot run on `item` now; `None` when it can. Pure, so the menu,
/// the buttons, the keys and the palette read one answer.
pub(super) fn why_not(item: &Item, act: Act) -> Option<String> {
    let unsupported = || {
        Some(if item.host_label.is_empty() {
            msg::radar::WHY_NO_HOST.get().to_string()
        } else {
            msg::radar::why_not_supported(item.host_label.clone())
        })
    };
    match act {
        Act::Merge => {
            if item.kind != ItemKind::Pr {
                return Some(msg::radar::MSG_NOT_PR.get().to_string());
            }
            if item.host_id.is_empty() || !item.acts.merge {
                return unsupported();
            }
            if item.draft {
                return Some(msg::radar::WHY_MERGE_DRAFT.get().to_string());
            }
            if item.has_conflicts == Some(true) {
                return Some(msg::radar::why_merge_conflicts(item.target.clone()));
            }
            if item.checks == Some(gitpoulpi_hosting::ChecksState::Failing) {
                return Some(msg::radar::WHY_MERGE_CHECKS.get().to_string());
            }
            None
        }
        Act::CloseItem => {
            if item.kind == ItemKind::Wip {
                return Some(msg::radar::MSG_SELECT_ROW.get().to_string());
            }
            if item.host_id.is_empty() || !item.acts.close {
                return unsupported();
            }
            None
        }
        Act::Ready => {
            if item.kind != ItemKind::Pr {
                return Some(msg::radar::MSG_NOT_PR.get().to_string());
            }
            if !item.draft {
                return Some(msg::radar::WHY_NOT_DRAFT.get().to_string());
            }
            if item.host_id.is_empty() || !item.acts.ready {
                return unsupported();
            }
            None
        }
        Act::CloneRepo => {
            if item.kind == ItemKind::Wip {
                return Some(msg::radar::MSG_SELECT_ROW.get().to_string());
            }
            if let Some(path) = &item.repo {
                return Some(msg::radar::why_already_cloned(path.display()));
            }
            if item.clone_url.is_empty() || item.integration.is_empty() {
                return Some(msg::radar::msg_no_clone_url(&item.host_project));
            }
            None
        }
        _ => None,
    }
}

fn method_label(method: MergeMethod) -> &'static str {
    match method {
        MergeMethod::Merge => msg::radar::METHOD_MERGE.get(),
        MergeMethod::Squash => msg::radar::METHOD_SQUASH.get(),
        MergeMethod::Rebase => msg::radar::METHOD_REBASE.get(),
    }
}

impl RadarUi {
    /// Merge, Close, Mark as ready on `item`: the refusal, the confirmation,
    /// or the call.
    pub(super) fn host_act(&mut self, item: &Item, act: Act) -> Outcome {
        if let Some(why) = why_not(item, act) {
            return self.say(why);
        }
        if self.acting.is_some() {
            return self.say(msg::radar::MSG_HOST_BUSY.get().to_string());
        }
        let pr = item.kind == ItemKind::Pr;
        let call = |act| HostCall {
            act,
            host_id: item.host_id.clone(),
            project: item.host_project.clone(),
            number: item.number,
            pr,
            label: item.label.clone(),
        };
        match act {
            Act::Ready => {
                let call = call(HostAct::Ready);
                self.spawn(call)
            }
            Act::Merge => {
                let branches = item.branches_text();
                let body =
                    msg::radar::merge_body(item.title.clone(), branches, item.host_project.clone());
                let mut buttons: Vec<(String, Choice)> = MergeMethod::ALL
                    .into_iter()
                    .filter(|m| item.acts.takes(*m))
                    .map(|m| (method_label(m).to_string(), Choice::Merge(m)))
                    .collect();
                buttons.push((msg::radar::CONFIRM_CANCEL.get().to_string(), Choice::Cancel));
                let dialog = Confirm::new(
                    msg::radar::merge_title(item.label.clone()),
                    body,
                    buttons,
                    Choice::Cancel,
                )
                .stacked()
                .keep_breaks()
                .min_width(60);
                self.confirm = Some(Asked {
                    dialog,
                    call: call(HostAct::Merge(MergeMethod::Merge)),
                });
                Outcome::None
            }
            _ => {
                let body = if pr {
                    msg::radar::close_body(
                        item.title.clone(),
                        item.branches_text(),
                        item.host_project.clone(),
                    )
                } else {
                    msg::radar::close_issue_body(item.title.clone(), item.host_project.clone())
                };
                let dialog = Confirm::new(
                    msg::radar::close_title(item.label.clone()),
                    body,
                    vec![
                        (msg::radar::CONFIRM_CLOSE.get().to_string(), Choice::Close),
                        (msg::radar::CONFIRM_CANCEL.get().to_string(), Choice::Cancel),
                    ],
                    Choice::Cancel,
                )
                .stacked()
                .keep_breaks()
                .min_width(60);
                self.confirm = Some(Asked {
                    dialog,
                    call: call(HostAct::Close),
                });
                Outcome::None
            }
        }
    }

    /// The Clone action: the ordinary Clone dialog on the row's project.
    pub(super) fn clone_repo(&mut self, item: &Item) -> Outcome {
        if let Some(why) = why_not(item, Act::CloneRepo) {
            return self.say(why);
        }
        Outcome::CloneRepo(Box::new(CloneRequest {
            integration: item.integration.clone(),
            project: item.host_project.clone(),
            clone_url: item.clone_url.clone(),
        }))
    }

    pub(super) fn confirm_key(&mut self, key: KeyEvent) -> Outcome {
        let Some(asked) = self.confirm.as_mut() else {
            return Outcome::None;
        };
        let event = asked.dialog.on_key(key);
        self.settle_confirm(event)
    }

    pub(super) fn confirm_mouse(&mut self, mouse: MouseEvent) -> Outcome {
        let Some(asked) = self.confirm.as_mut() else {
            return Outcome::None;
        };
        let event = asked.dialog.on_mouse(mouse);
        self.settle_confirm(event)
    }

    fn settle_confirm(&mut self, event: ConfirmEvent<Choice>) -> Outcome {
        let ConfirmEvent::Chose(choice) = event else {
            return Outcome::None;
        };
        let Some(asked) = self.confirm.take() else {
            return Outcome::None;
        };
        let mut call = asked.call;
        match choice {
            Choice::Cancel => Outcome::None,
            Choice::Merge(method) => {
                call.act = HostAct::Merge(method);
                self.spawn(call)
            }
            Choice::Close => self.spawn(call),
        }
    }

    pub(super) fn paint_confirm(&mut self, buf: &mut ratatui::buffer::Buffer, body: Rect) {
        if let Some(asked) = self.confirm.as_mut() {
            asked.dialog.render(buf, body);
        }
    }

    fn spawn(&mut self, call: HostCall) -> Outcome {
        let (tx, rx) = mpsc::channel();
        let job = call.clone();
        std::thread::spawn(move || {
            let _ = tx.send(run_call(&job));
        });
        self.acting = Some(Acting { rx, call });
        self.say(msg::radar::MSG_ASKING_HOST.get().to_string())
    }

    /// Takes the answer of the call in flight: the line to show, and the
    /// lists asked again so the row changes.
    pub(super) fn acting_poll(&mut self) -> bool {
        let Some(acting) = self.acting.as_ref() else {
            return false;
        };
        let answer = match acting.rx.try_recv() {
            Ok(answer) => answer,
            Err(mpsc::TryRecvError::Empty) => return false,
            Err(mpsc::TryRecvError::Disconnected) => Err("the call stopped".to_string()), // i18n-english
        };
        let Some(acting) = self.acting.take() else {
            return false;
        };
        let label = acting.call.label.clone();
        let text = match answer {
            Ok(()) => {
                self.reload();
                match acting.call.act {
                    HostAct::Merge(_) => msg::radar::msg_merged(label),
                    HostAct::Close => msg::radar::msg_closed(label),
                    HostAct::Ready => msg::radar::msg_ready(label),
                }
            }
            Err(why) => msg::radar::msg_host_failed(label, why),
        };
        self.message = Some(text.clone());
        self.said = Some(text);
        true
    }
}

/// Runs the call on the host (a thread of its own, its own runtime).
fn run_call(call: &HostCall) -> Result<(), String> {
    use crate::hosting_task::ClientFactory as _;
    let all = crate::pull_request::load_integrations();
    let integration = all
        .iter()
        .find(|i| i.id == call.host_id)
        .ok_or_else(|| msg::radar::why_no_integration(&call.host_id))?;
    let client = crate::hosting_task::DefaultFactory
        .client(integration, &all)
        .map_err(|e| e.to_string())?;
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| e.to_string())?;
    runtime
        .block_on(call_host(client.as_ref(), call))
        .map_err(|e| e.to_string())
}

async fn call_host(client: &dyn HostingClient, call: &HostCall) -> gitpoulpi_hosting::Result<()> {
    match (call.act, call.pr) {
        (HostAct::Merge(method), _) => {
            client
                .merge_pull_request(&call.project, call.number, method)
                .await
        }
        (HostAct::Ready, _) => {
            client
                .mark_pull_request_ready(&call.project, call.number)
                .await
        }
        (HostAct::Close, true) => client.close_pull_request(&call.project, call.number).await,
        (HostAct::Close, false) => {
            client
                .set_issue_state(&call.project, call.number, true)
                .await
        }
    }
}
