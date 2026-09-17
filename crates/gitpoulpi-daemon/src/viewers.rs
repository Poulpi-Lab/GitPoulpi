//! The daemon's viewer registry: which TUI windows are attached to this
//! session, and the routing of an agent's view intents (`gitpoulpi ui show
//! ...`) to exactly one of them. see the mcp-show-ui design.
//!
//! Everything here is **view-only**. The `Ui*` operations are answered by
//! this module before `dispatch` is ever reached, so they take no repository
//! lock, never reach a `GitBackend`, are never journaled and never broadcast.
//! An intent goes out as an `Event::UiIntent` pushed on the *chosen viewer's
//! own connection*; a third client subscribed to the session hears nothing.

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use gitpoulpi_core::domain::UiControl;
use gitpoulpi_ipc::ui::{
    RaiseAnswer, UiIntent, UiOutcome, UiQueryArgs, UiReportArgs, UiReportBody, UiShowArgs,
    ViewerId, ViewerInfo, NO_VIEWER_MESSAGE, RAISE_ANSWER_TIMEOUT_MS, UI_ANSWER_TIMEOUT_MS,
};
use gitpoulpi_ipc::{Event, Request, Response, ResponseError, ServerMessage};
use tokio::sync::{mpsc, oneshot};

/// How long a viewer whose connection dropped (a TUI window restarting into
/// another tab) keeps its pending intents, waiting for a re-attach with the
/// same id.
pub const REATTACH_GRACE: Duration = Duration::from_secs(5);

type OutTx = mpsc::Sender<ServerMessage>;

struct Entry {
    /// The connection that attached; a later attach with the same id replaces it.
    conn: u64,
    /// `None` while the connection is gone and the grace runs.
    out_tx: Option<OutTx>,
    info: ViewerInfo,
    focus_tick: u64,
    attach_tick: u64,
}

struct Pending {
    viewer: ViewerId,
    /// What to push again when the viewer re-attaches before it answered.
    event: Event,
    waiter: oneshot::Sender<UiReportBody>,
}

#[derive(Default)]
struct Inner {
    viewers: HashMap<ViewerId, Entry>,
    pending: HashMap<u64, Pending>,
    next_id: u64,
    tick: u64,
}

/// The registry. One per daemon, shared by every connection.
pub struct Viewers {
    inner: Mutex<Inner>,
    next_conn: std::sync::atomic::AtomicU64,
    grace: Duration,
}

impl Default for Viewers {
    fn default() -> Self {
        Self::with_grace(REATTACH_GRACE)
    }
}

/// Which viewer an intent goes to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Selection {
    Chosen(ViewerId),
    NoViewer,
    /// Viewers are attached, but every candidate's user turned agent view
    /// control off.
    Disabled,
    /// `--viewer` named a window that is not attached.
    Unknown(ViewerId),
}

impl Viewers {
    pub fn with_grace(grace: Duration) -> Self {
        Self {
            inner: Mutex::new(Inner::default()),
            next_conn: std::sync::atomic::AtomicU64::new(1),
            grace,
        }
    }

    pub fn grace(&self) -> Duration {
        self.grace
    }

    pub fn next_conn(&self) -> u64 {
        self.next_conn
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    }

    /// Registers (or refreshes) `info.id` on connection `conn`. Intents the
    /// viewer had not answered when its previous connection dropped are
    /// pushed again.
    pub fn attach(&self, conn: u64, out_tx: &OutTx, mut info: ViewerInfo) {
        let mut inner = self.inner.lock().unwrap();
        inner.tick += 1;
        let tick = inner.tick;
        let id = info.id.clone();
        let previous = inner.viewers.remove(&id);
        info.last_focus_at = previous.as_ref().and_then(|p| p.info.last_focus_at);
        let mut focus_tick = previous.as_ref().map_or(0, |p| p.focus_tick);
        if info.focused == Some(true) {
            focus_tick = tick;
            info.last_focus_at = Some(gitpoulpi_core::time::unix_now());
        }
        inner.viewers.insert(
            id.clone(),
            Entry {
                conn,
                out_tx: Some(out_tx.clone()),
                info,
                focus_tick,
                attach_tick: tick,
            },
        );
        let mut resend: Vec<(u64, Event)> = inner
            .pending
            .iter()
            .filter(|(_, p)| p.viewer == id)
            .map(|(n, p)| (*n, p.event.clone()))
            .collect();
        resend.sort_by_key(|(n, _)| *n);
        for (_, event) in resend {
            let _ = out_tx.try_send(ServerMessage::Event(event));
        }
    }

    /// Focus or tab changes. Ignored unless `conn` is the viewer's current connection.
    pub fn presence(&self, conn: u64, info: ViewerInfo) {
        let mut inner = self.inner.lock().unwrap();
        inner.tick += 1;
        let tick = inner.tick;
        let Some(entry) = inner.viewers.get_mut(&info.id) else {
            return;
        };
        if entry.conn != conn {
            return;
        }
        let gained = info.focused == Some(true) && entry.info.focused != Some(true);
        let last = entry.info.last_focus_at;
        entry.info = ViewerInfo {
            last_focus_at: last,
            ..info
        };
        if gained {
            entry.focus_tick = tick;
            entry.info.last_focus_at = Some(gitpoulpi_core::time::unix_now());
        }
    }

    /// A connection ended: its viewers stay known for the grace (so a window
    /// restart keeps its pending intents). Returns the viewers to expire later.
    pub fn detach_conn(&self, conn: u64) -> Vec<ViewerId> {
        let mut inner = self.inner.lock().unwrap();
        let mut detached = Vec::new();
        for (id, entry) in inner.viewers.iter_mut() {
            if entry.conn == conn && entry.out_tx.is_some() {
                entry.out_tx = None;
                detached.push(id.clone());
            }
        }
        detached
    }

    /// After the grace: forgets the viewer if it never re-attached, and
    /// answers its pending intents.
    pub fn expire(&self, id: &ViewerId, conn: u64) {
        let mut inner = self.inner.lock().unwrap();
        let still_gone = inner
            .viewers
            .get(id)
            .is_some_and(|e| e.conn == conn && e.out_tx.is_none());
        if !still_gone {
            return;
        }
        inner.viewers.remove(id);
        let ids: Vec<u64> = inner
            .pending
            .iter()
            .filter(|(_, p)| &p.viewer == id)
            .map(|(n, _)| *n)
            .collect();
        for n in ids {
            if let Some(p) = inner.pending.remove(&n) {
                let _ = p.waiter.send(UiReportBody::Outcome {
                    outcome: UiOutcome::refused("the window went away before it answered"),
                });
            }
        }
    }

    /// The attached viewers, most recently focused first.
    pub fn list(&self) -> Vec<ViewerInfo> {
        let inner = self.inner.lock().unwrap();
        let mut attached: Vec<&Entry> = inner
            .viewers
            .values()
            .filter(|e| e.out_tx.is_some())
            .collect();
        attached.sort_by_key(|e| std::cmp::Reverse((e.focus_tick, e.attach_tick)));
        attached.into_iter().map(|e| e.info.clone()).collect()
    }

    /// The attached window a launcher click should raise: `explicit`, else
    /// the most recently focused (then attached). Agent control is not
    /// consulted: a raise is the user's own click.
    pub fn most_recent(&self, explicit: Option<&ViewerId>) -> Option<(ViewerId, ViewerInfo)> {
        let inner = self.inner.lock().unwrap();
        let mut attached = inner.viewers.iter().filter(|(_, e)| e.out_tx.is_some());
        let pick = match explicit {
            Some(id) => attached.find(|(i, _)| *i == id),
            None => attached.max_by_key(|(_, e)| (e.focus_tick, e.attach_tick)),
        };
        pick.map(|(id, e)| (id.clone(), e.info.clone()))
    }

    /// Picks the window an intent goes to.
    ///
    /// 1. an explicit `--viewer` wins;
    /// 2. `Highlight`/`Back` follow the window `prefer` (this client's last
    ///    target) when it is still attached;
    /// 3. an intent naming a repository goes to a window whose active tab is
    ///    it, else one with it open in a tab, else any window;
    /// 4. ties go to the most recently focused, then the most recently attached.
    ///
    /// Windows whose user switched agent control off are skipped.
    pub fn select(
        &self,
        intent: &UiIntent,
        explicit: Option<&ViewerId>,
        prefer: Option<&ViewerId>,
    ) -> Selection {
        let inner = self.inner.lock().unwrap();
        let attached: Vec<(&ViewerId, &Entry)> = inner
            .viewers
            .iter()
            .filter(|(_, e)| e.out_tx.is_some())
            .collect();
        if let Some(id) = explicit {
            return match inner.viewers.get(id).filter(|e| e.out_tx.is_some()) {
                None => Selection::Unknown(id.clone()),
                Some(e) if !e.info.agent_control => Selection::Disabled,
                Some(_) => Selection::Chosen(id.clone()),
            };
        }
        if attached.is_empty() {
            return Selection::NoViewer;
        }
        let enabled: Vec<(&ViewerId, &Entry)> = attached
            .into_iter()
            .filter(|(_, e)| e.info.agent_control)
            .collect();
        if enabled.is_empty() {
            return Selection::Disabled;
        }
        if intent.follows_previous_target() {
            if let Some(p) = prefer.filter(|p| enabled.iter().any(|(id, _)| id == p)) {
                return Selection::Chosen(p.clone());
            }
        }
        let best = |pool: Vec<&(&ViewerId, &Entry)>| -> Option<ViewerId> {
            pool.into_iter()
                .max_by_key(|(_, e)| (e.focus_tick, e.attach_tick))
                .map(|(id, _)| (*id).clone())
        };
        if let Some(repo) = intent.repo() {
            let active: Vec<_> = enabled
                .iter()
                .filter(|(_, e)| e.info.active_repo.as_ref() == Some(repo))
                .collect();
            if let Some(id) = best(active) {
                return Selection::Chosen(id);
            }
            let open: Vec<_> = enabled
                .iter()
                .filter(|(_, e)| e.info.tabs.contains(repo))
                .collect();
            if let Some(id) = best(open) {
                return Selection::Chosen(id);
            }
        }
        match best(enabled.iter().collect()) {
            Some(id) => Selection::Chosen(id),
            None => Selection::NoViewer,
        }
    }

    /// Pushes `event` (built from the new request id) to `viewer`'s own
    /// connection and registers the waiter. `Err` when the viewer's queue
    /// is full or it is unknown.
    fn submit(
        &self,
        viewer: &ViewerId,
        make: impl FnOnce(u64) -> Event,
    ) -> Result<(u64, oneshot::Receiver<UiReportBody>), String> {
        let mut inner = self.inner.lock().unwrap();
        inner.next_id += 1;
        let id = inner.next_id;
        let event = make(id);
        let Some(entry) = inner.viewers.get(viewer) else {
            return Err("that window is no longer attached".into());
        };
        if let Some(tx) = &entry.out_tx {
            match tx.try_send(ServerMessage::Event(event.clone())) {
                Ok(()) | Err(mpsc::error::TrySendError::Closed(_)) => {}
                Err(mpsc::error::TrySendError::Full(_)) => {
                    return Err("the window is not keeping up".into())
                }
            }
        }
        let (waiter, rx) = oneshot::channel();
        inner.pending.insert(
            id,
            Pending {
                viewer: viewer.clone(),
                event,
                waiter,
            },
        );
        Ok((id, rx))
    }

    fn cancel(&self, id: u64) {
        self.inner.lock().unwrap().pending.remove(&id);
    }

    /// A viewer's answer. `false` when `conn` is not the connection of the
    /// viewer the request went to, or the request is unknown (answered
    /// already, or timed out).
    pub fn report(&self, conn: u64, report: UiReportArgs) -> bool {
        let mut inner = self.inner.lock().unwrap();
        let Some(pending) = inner.pending.get(&report.id) else {
            return false;
        };
        let owner_conn = inner.viewers.get(&pending.viewer).map(|e| e.conn);
        if owner_conn != Some(conn) {
            return false;
        }
        let pending = inner.pending.remove(&report.id).expect("checked above");
        pending.waiter.send(report.body).is_ok()
    }
}

/// Per-connection state of a client that sends `UiShow`.
pub struct ConnUi {
    pub id: u64,
    bucket: TokenBucket,
    /// The viewer this client's last intent went to, so `Highlight` and
    /// `Back` land in the same window.
    last_target: Option<ViewerId>,
}

impl ConnUi {
    pub fn new(id: u64) -> Self {
        Self {
            id,
            bucket: TokenBucket::new(Instant::now()),
            last_target: None,
        }
    }
}

/// Burst 5, refill 1 per second.
pub struct TokenBucket {
    tokens: f64,
    last: Instant,
}

impl TokenBucket {
    pub const BURST: f64 = 5.0;
    pub const REFILL_PER_SEC: f64 = 1.0;

    pub fn new(now: Instant) -> Self {
        Self {
            tokens: Self::BURST,
            last: now,
        }
    }

    pub fn try_take(&mut self, now: Instant) -> bool {
        let elapsed = now.saturating_duration_since(self.last).as_secs_f64();
        self.last = now;
        self.tokens = (self.tokens + elapsed * Self::REFILL_PER_SEC).min(Self::BURST);
        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            true
        } else {
            false
        }
    }
}

fn ok(request_id: u64, value: impl serde::Serialize) -> Response {
    Response {
        request_id,
        result: Ok(serde_json::to_value(value).unwrap_or(serde_json::Value::Null)),
    }
}

fn err(request_id: u64, message: impl Into<String>) -> Response {
    Response {
        request_id,
        result: Err(ResponseError {
            message: message.into(),
            unsupported: false,
            kind: None,
            detail: None,
        }),
    }
}

fn answer_timeout(requested: Option<u64>) -> Duration {
    Duration::from_millis(
        requested
            .unwrap_or(UI_ANSWER_TIMEOUT_MS)
            .min(UI_ANSWER_TIMEOUT_MS),
    )
}

/// Answers one `Ui*` request. Never touches a repository.
pub async fn handle(
    viewers: &Viewers,
    conn: &mut ConnUi,
    out_tx: &OutTx,
    request: &Request,
) -> Response {
    use gitpoulpi_core::domain::OperationKind as K;
    let id = request.id;
    match request.command {
        K::UiAttach | K::UiPresence => {
            let info: ViewerInfo = match serde_json::from_value(request.args.clone()) {
                Ok(i) => i,
                Err(e) => return err(id, format!("malformed viewer info: {e}")),
            };
            if request.command == K::UiAttach {
                viewers.attach(conn.id, out_tx, info);
            } else {
                viewers.presence(conn.id, info);
            }
            ok(id, serde_json::Value::Null)
        }
        K::UiReport => match serde_json::from_value::<UiReportArgs>(request.args.clone()) {
            Ok(report) => {
                if viewers.report(conn.id, report) {
                    ok(id, serde_json::Value::Null)
                } else {
                    err(id, "no pending request for that id on this connection")
                }
            }
            Err(e) => err(id, format!("malformed report: {e}")),
        },
        K::UiShow => match serde_json::from_value::<UiShowArgs>(request.args.clone()) {
            Ok(args) => ok(id, show(viewers, conn, args).await),
            Err(e) => err(id, format!("malformed ui intent: {e}")),
        },
        K::UiQuery => match serde_json::from_value::<UiQueryArgs>(request.args.clone()) {
            Ok(UiQueryArgs::Viewers) => ok(id, viewers.list()),
            Ok(UiQueryArgs::Raise { viewer }) => raise(viewers, id, viewer).await,
            Ok(UiQueryArgs::State {
                viewer,
                include_text,
            }) => state(viewers, id, viewer, include_text).await,
            Err(e) => err(id, format!("malformed ui query: {e}")),
        },
        other => err(id, format!("{other:?} is not a view-control operation")),
    }
}

async fn show(viewers: &Viewers, conn: &mut ConnUi, mut args: UiShowArgs) -> UiOutcome {
    args.intent.sanitize();
    args.origin.sanitize();
    // The commands layer checks it too: a client that skips it is refused here.
    if let UiIntent::Propose { proposal, .. } = &args.intent {
        if let Some(why) = proposal.problem() {
            return UiOutcome::refused(why);
        }
    }
    if args.origin.control == UiControl::Off {
        return UiOutcome::refused("agent view control is off for this client (ui_control = off)");
    }
    if !conn.bucket.try_take(Instant::now()) {
        return UiOutcome::refused("rate limited");
    }
    let target = match viewers.select(
        &args.intent,
        args.viewer.as_ref(),
        conn.last_target.as_ref(),
    ) {
        Selection::Chosen(v) => v,
        Selection::NoViewer => {
            return UiOutcome::NoViewer {
                message: NO_VIEWER_MESSAGE.to_string(),
            }
        }
        Selection::Disabled => {
            return UiOutcome::refused("the user turned agent view control off in that window")
        }
        Selection::Unknown(v) => {
            return UiOutcome::refused(format!("no attached window `{v}` (see `ui viewers`)"))
        }
    };
    conn.last_target = Some(target.clone());

    let UiShowArgs {
        intent,
        origin,
        timeout_ms,
        ..
    } = args;
    let viewer = target.clone();
    let submitted = viewers.submit(&target, |intent_id| Event::UiIntent {
        viewer,
        intent_id,
        intent,
        origin,
    });
    let (intent_id, rx) = match submitted {
        Ok(v) => v,
        Err(reason) => return UiOutcome::refused(reason),
    };
    match tokio::time::timeout(answer_timeout(timeout_ms), rx).await {
        Ok(Ok(UiReportBody::Outcome { outcome })) => outcome,
        Ok(Ok(UiReportBody::State { state })) => UiOutcome::Applied { state },
        Ok(Err(_)) => UiOutcome::refused("the window went away before it answered"),
        Err(_) => {
            viewers.cancel(intent_id);
            UiOutcome::refused("the window did not answer")
        }
    }
}

/// `UiQuery(Raise)`: asks one window to bring its terminal forward and says
/// whether it did. Never an error: "no window" and "it could not" are
/// answers the launcher acts on (it starts a new window instead).
async fn raise(viewers: &Viewers, request_id: u64, explicit: Option<ViewerId>) -> Response {
    let answer = |raised: bool, viewer: Option<ViewerInfo>, reason: Option<String>| {
        ok(
            request_id,
            RaiseAnswer {
                raised,
                viewer,
                reason,
            },
        )
    };
    let Some((target, info)) = viewers.most_recent(explicit.as_ref()) else {
        return answer(false, None, Some(NO_VIEWER_MESSAGE.to_string()));
    };
    let viewer = target.clone();
    let (raise_id, rx) =
        match viewers.submit(&target, |raise_id| Event::RaiseWindow { viewer, raise_id }) {
            Ok(v) => v,
            Err(reason) => return answer(false, Some(info), Some(reason)),
        };
    let wait = Duration::from_millis(RAISE_ANSWER_TIMEOUT_MS);
    match tokio::time::timeout(wait, rx).await {
        Ok(Ok(UiReportBody::Outcome {
            outcome: UiOutcome::Refused { reason },
        })) => answer(false, Some(info), Some(reason)),
        Ok(Ok(_)) => answer(true, Some(info), None),
        Ok(Err(_)) => answer(
            false,
            Some(info),
            Some("the window went away before it answered".into()),
        ),
        Err(_) => {
            viewers.cancel(raise_id);
            answer(false, Some(info), Some("the window did not answer".into()))
        }
    }
}

async fn state(
    viewers: &Viewers,
    request_id: u64,
    explicit: Option<ViewerId>,
    include_text: bool,
) -> Response {
    // Like a repo-less intent: the most recently focused window.
    let probe = UiIntent::Back;
    let target = match viewers.select(&probe, explicit.as_ref(), None) {
        Selection::Chosen(v) => v,
        Selection::NoViewer => return err(request_id, NO_VIEWER_MESSAGE),
        Selection::Disabled => return err(request_id, "the user turned agent view control off"),
        Selection::Unknown(v) => return err(request_id, format!("no attached window `{v}`")),
    };
    let viewer = target.clone();
    let (query_id, rx) = match viewers.submit(&target, |query_id| Event::UiStateRequest {
        viewer,
        query_id,
        include_text,
    }) {
        Ok(v) => v,
        Err(reason) => return err(request_id, reason),
    };
    match tokio::time::timeout(answer_timeout(None), rx).await {
        Ok(Ok(UiReportBody::State { state })) => ok(request_id, state),
        Ok(Ok(UiReportBody::Outcome { outcome })) => match outcome {
            UiOutcome::Refused { reason } => err(request_id, reason),
            other => err(request_id, format!("the window answered {other:?}")),
        },
        Ok(Err(_)) => err(request_id, "the window went away before it answered"),
        Err(_) => {
            viewers.cancel(query_id);
            err(request_id, "the window did not answer")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gitpoulpi_core::domain::RepoId;

    fn repo(n: &str) -> RepoId {
        RepoId(n.into())
    }

    fn info(id: &str, active: Option<&str>, tabs: &[&str], focused: Option<bool>) -> ViewerInfo {
        ViewerInfo {
            id: ViewerId(id.into()),
            pid: 1,
            active_repo: active.map(repo),
            tabs: tabs.iter().map(|t| repo(t)).collect(),
            focused,
            agent_control: true,
            last_focus_at: None,
        }
    }

    fn attach(v: &Viewers, conn: u64, i: ViewerInfo) -> mpsc::Receiver<ServerMessage> {
        let (tx, rx) = mpsc::channel(8);
        v.attach(conn, &tx, i);
        rx
    }

    fn commit_intent(r: &str) -> UiIntent {
        UiIntent::ShowTab { repo: repo(r) }
    }

    fn chosen(s: Selection) -> String {
        match s {
            Selection::Chosen(v) => v.0,
            other => panic!("expected a viewer, got {other:?}"),
        }
    }

    #[test]
    fn selection_prefers_active_repo_then_open_tab_then_any() {
        let v = Viewers::default();
        let _a = attach(&v, 1, info("a", Some("/x"), &["/x", "/y"], None));
        let _b = attach(&v, 2, info("b", Some("/y"), &["/y"], None));
        let _c = attach(&v, 3, info("c", Some("/z"), &["/z"], None));
        assert_eq!(chosen(v.select(&commit_intent("/y"), None, None)), "b");
        assert_eq!(chosen(v.select(&commit_intent("/x"), None, None)), "a");
        // Nobody has /w: the most recently attached (all unfocused) wins.
        assert_eq!(chosen(v.select(&commit_intent("/w"), None, None)), "c");
    }

    #[test]
    fn open_tab_beats_a_window_that_only_has_another_active_repo() {
        let v = Viewers::default();
        let _a = attach(&v, 1, info("a", Some("/x"), &["/x", "/y"], None));
        let _b = attach(&v, 2, info("b", Some("/z"), &["/z"], None));
        assert_eq!(chosen(v.select(&commit_intent("/y"), None, None)), "a");
    }

    #[test]
    fn ties_go_to_the_most_recent_focus_then_the_latest_attach() {
        let v = Viewers::default();
        let _a = attach(&v, 1, info("a", Some("/x"), &["/x"], Some(true)));
        let _b = attach(&v, 2, info("b", Some("/x"), &["/x"], None));
        // b attached later but a was focused (at its attach), which outranks it.
        assert_eq!(chosen(v.select(&commit_intent("/x"), None, None)), "a");
        // b gains focus.
        v.presence(2, info("b", Some("/x"), &["/x"], Some(true)));
        assert_eq!(chosen(v.select(&commit_intent("/x"), None, None)), "b");
        // a loses and regains it.
        v.presence(1, info("a", Some("/x"), &["/x"], Some(false)));
        v.presence(1, info("a", Some("/x"), &["/x"], Some(true)));
        assert_eq!(chosen(v.select(&commit_intent("/x"), None, None)), "a");
    }

    #[test]
    fn an_explicit_viewer_overrides_and_an_unknown_one_is_named() {
        let v = Viewers::default();
        let _a = attach(&v, 1, info("a", Some("/x"), &["/x"], Some(true)));
        let _b = attach(&v, 2, info("b", Some("/y"), &["/y"], None));
        let b = ViewerId("b".into());
        assert_eq!(
            v.select(&commit_intent("/x"), Some(&b), None),
            Selection::Chosen(b)
        );
        let nope = ViewerId("nope".into());
        assert_eq!(
            v.select(&commit_intent("/x"), Some(&nope), None),
            Selection::Unknown(nope)
        );
    }

    #[test]
    fn highlight_and_back_follow_the_previous_target() {
        let v = Viewers::default();
        let _a = attach(&v, 1, info("a", Some("/x"), &["/x"], Some(true)));
        let _b = attach(&v, 2, info("b", Some("/y"), &["/y"], None));
        let prev = ViewerId("b".into());
        assert_eq!(
            chosen(v.select(&UiIntent::Back, None, Some(&prev))),
            "b",
            "back lands where the first intent did"
        );
        assert_eq!(
            chosen(v.select(&UiIntent::ShowHelp { topic: None }, None, Some(&prev))),
            "a",
            "other repo-less intents go to the focused window"
        );
    }

    #[test]
    fn no_viewer_and_disabled_viewers() {
        let v = Viewers::default();
        assert_eq!(v.select(&UiIntent::Back, None, None), Selection::NoViewer);
        let mut off = info("a", None, &[], Some(true));
        off.agent_control = false;
        let _a = attach(&v, 1, off);
        assert_eq!(v.select(&UiIntent::Back, None, None), Selection::Disabled);
        let _b = attach(&v, 2, info("b", None, &[], None));
        assert_eq!(chosen(v.select(&UiIntent::Back, None, None)), "b");
    }

    #[test]
    fn a_detached_viewer_is_not_selectable_but_keeps_its_slot_for_the_grace() {
        let v = Viewers::default();
        let rx = attach(&v, 1, info("a", Some("/x"), &["/x"], None));
        drop(rx);
        let gone = v.detach_conn(1);
        assert_eq!(gone, vec![ViewerId("a".into())]);
        assert_eq!(v.select(&UiIntent::Back, None, None), Selection::NoViewer);
        assert!(v.list().is_empty());
        // A re-attach brings it back.
        let _rx = attach(&v, 2, info("a", Some("/y"), &["/y"], None));
        assert_eq!(chosen(v.select(&UiIntent::Back, None, None)), "a");
        // The old connection's late detach must not unseat the new one.
        assert!(v.detach_conn(1).is_empty());
        assert_eq!(v.list().len(), 1);
    }

    #[test]
    fn the_bucket_allows_a_burst_of_five_then_one_per_second() {
        let t0 = Instant::now();
        let mut b = TokenBucket::new(t0);
        for i in 0..5 {
            assert!(b.try_take(t0), "burst {i}");
        }
        assert!(!b.try_take(t0), "sixth is limited");
        assert!(!b.try_take(t0 + Duration::from_millis(500)));
        assert!(b.try_take(t0 + Duration::from_millis(1100)));
        assert!(!b.try_take(t0 + Duration::from_millis(1200)));
        // Never banks more than the burst.
        let later = t0 + Duration::from_secs(600);
        for _ in 0..5 {
            assert!(b.try_take(later));
        }
        assert!(!b.try_take(later));
    }

    #[test]
    fn answer_timeout_is_capped() {
        assert_eq!(answer_timeout(None), Duration::from_secs(8));
        assert_eq!(answer_timeout(Some(100)), Duration::from_millis(100));
        assert_eq!(answer_timeout(Some(60_000)), Duration::from_secs(8));
    }
}
