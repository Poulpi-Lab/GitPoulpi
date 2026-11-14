//! What a workspace asks of the daemon, off the input loop.
//!
//! Saving a workspace records each new repository's `origin` URL (a
//! repository read, so it goes through `GitBackend`, never through a `git`
//! call here), and Fetch all / Pull all / Clone missing walk a workspace's
//! repositories one after the other. All three are one thread on **their own
//! connection** to the session's daemon, like `clone_task`: the window's link
//! is bound to one repository, and a walk over ten must not queue behind (or in
//! front of) the refresh of the one on screen.
//!
//! Sequential on purpose: ten fetches at once would saturate the network and
//! the disk, and the progress line ("2 of 10") only means something in order.
//! A repository that fails is listed at the end and the walk goes on.
//!
//! The thread reports on a channel the window drains with `try_recv`
//! (`RepoDialogs::poll`); it ends when the work does, and a closed receiver
//! (the window ended) stops it at the next report.

use std::path::{Path, PathBuf};
use std::sync::mpsc;

use gitpoulpi_core::domain::{CloneOptions, OperationKind, PullMode, PullOptions, Remote, RepoRef};
use gitpoulpi_i18n::msg::workspaces as m;
use gitpoulpi_ipc::{Client, Request};

use crate::redraw::NotifySend as _;

/// A workspace as the dialog edits it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct Draft {
    /// `None`: a new workspace.
    pub(crate) id: Option<String>,
    pub(crate) name: String,
    pub(crate) color: Option<String>,
    pub(crate) description: String,
    pub(crate) profile: Option<String>,
    pub(crate) repos: Vec<PathBuf>,
    /// The AI instructions as the form edited them; `None` leaves what the
    /// workspace has (AI is off, the form never showed them).
    pub(crate) ai: Option<gitpoulpi_config::WorkspaceAi>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Bulk {
    Fetch,
    Pull,
}

/// A repository to clone again: where, and from where.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Missing {
    pub(crate) path: PathBuf,
    pub(crate) url: Option<String>,
}

/// What a running job tells the window.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Event {
    /// A status line for the walk so far.
    Progress(String),
    /// The workspace was written.
    Saved {
        id: String,
        name: String,
        created: bool,
    },
    /// The job ended: the line that says how it went.
    Done(String),
    /// A save that could not be written.
    Failed(String),
}

pub(crate) struct Receiver {
    rx: mpsc::Receiver<Event>,
    /// Set by Cancel; a walk looks at it before each repository.
    stop: std::sync::Arc<std::sync::atomic::AtomicBool>,
    /// The entry in the jobs list, set when the window takes the job (0:
    /// not listed, a save).
    pub(crate) job: u64,
    /// The workspace's name, for the Global Activity Log entry the end of a
    /// walk leaves.
    pub(crate) workspace: String,
}

impl Receiver {
    /// The Cancel the jobs list calls: the same flag, from any thread.
    pub(crate) fn stopper(&self) -> Stop {
        Stop::clone(&self.stop)
    }

    /// Cancel was asked for.
    pub(crate) fn stopped(&self) -> bool {
        self.stop.load(std::sync::atomic::Ordering::SeqCst)
    }

    pub(crate) fn try_recv(&self) -> Option<Event> {
        self.rx.try_recv().ok()
    }
}

fn spawn<F>(work: F) -> Receiver
where
    F: FnOnce(mpsc::Sender<Event>, tokio::runtime::Runtime) + Send + 'static,
{
    spawn_stoppable(move |tx, runtime, _| work(tx, runtime))
}

type Stop = std::sync::Arc<std::sync::atomic::AtomicBool>;

fn spawn_stoppable<F>(work: F) -> Receiver
where
    F: FnOnce(mpsc::Sender<Event>, tokio::runtime::Runtime, Stop) + Send + 'static,
{
    let (tx, rx) = mpsc::channel();
    let stop = Stop::default();
    let flag = Stop::clone(&stop);
    std::thread::spawn(move || {
        match tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
        {
            Ok(runtime) => work(tx, runtime, flag),
            Err(e) => {
                let _ = tx.send_notify(Event::Failed(e.to_string()));
            }
        }
    });
    Receiver {
        rx,
        stop,
        job: 0,
        workspace: String::new(),
    }
}

/// `https://user:token@host/x.git` -> `https://host/x.git`: a recorded URL
/// never carries a credential.
pub(crate) fn without_userinfo(url: &str) -> String {
    let Some((scheme, rest)) = url.split_once("://") else {
        return url.to_string();
    };
    let end = rest.find('/').unwrap_or(rest.len());
    match rest[..end].rfind('@') {
        Some(at) => format!("{scheme}://{}", &rest[at + 1..]),
        None => url.to_string(),
    }
}

/// A connection for repository `path`'s requests.
struct Conn {
    client: Client,
    session: String,
    next: u64,
}

impl Conn {
    async fn open(session: &str) -> Result<Self, String> {
        let endpoint = crate::daemon_link::ensure_daemon(session).await?;
        let client = Client::connect(&endpoint)
            .await
            .map_err(gitpoulpi_i18n::msg::task_error::daemon_unreachable)?;
        Ok(Self {
            client,
            session: session.to_string(),
            next: 1,
        })
    }

    async fn call(
        &mut self,
        repo: Option<&Path>,
        command: OperationKind,
        args: serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        let id = self.next;
        self.next += 1;
        let request = Request {
            id,
            session: self.session.clone(),
            repo: repo.map(|p| RepoRef::from_root(p).id),
            command,
            args,
        };
        let response = self
            .client
            .call(&request)
            .await
            .map_err(|e| e.to_string())?;
        response.result.map_err(|e| e.message)
    }

    /// The repository's `origin` URL, credentials stripped.
    async fn origin(&mut self, path: &Path) -> Option<String> {
        let value = self
            .call(Some(path), OperationKind::Remotes, serde_json::json!({}))
            .await
            .ok()?;
        let remotes: Vec<Remote> = serde_json::from_value(value).ok()?;
        remotes
            .iter()
            .find(|r| r.name == "origin")
            .map(|r| without_userinfo(&r.fetch_url))
            .filter(|u| !u.is_empty())
    }
}

/// Writes `draft` into `workspaces.toml` (`config_dir`), recording the origin
/// of every repository that has none yet. `config_dir` is explicit so a test
/// can drive the file half with `conn = None`.
fn write_draft(
    config_dir: &Path,
    draft: &Draft,
    origins: &std::collections::HashMap<PathBuf, String>,
) -> Result<(String, String, bool), String> {
    use super::store;
    let mut all = store::load_in(config_dir);
    let name = draft.name.trim().to_string();
    let (id, created) = match &draft.id {
        Some(id) if all.get(id).is_some() => (id.clone(), false),
        _ => (all.create(&name).id.clone(), true),
    };
    let ws = all.get_mut(&id).ok_or_else(|| id.clone())?;
    ws.name = name.clone();
    ws.color = draft.color.clone().filter(|c| !c.is_empty());
    ws.description = draft.description.trim().to_string();
    ws.profile = draft.profile.clone().filter(|p| !p.is_empty());
    if let Some(ai) = &draft.ai {
        ws.ai_instructions = ai.general.clone();
        ws.ai_overrides = ai
            .actions
            .iter()
            .filter(|(_, v)| !v.text.trim().is_empty())
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
    }
    let gone: Vec<PathBuf> = ws
        .repos
        .iter()
        .filter(|r| {
            !draft
                .repos
                .iter()
                .any(|d| gitpoulpi_session::same_repo_path(d, r))
        })
        .cloned()
        .collect();
    for path in gone {
        ws.remove_repo(&path);
    }
    for path in &draft.repos {
        ws.add_repo(path, origins.get(path).map(String::as_str));
    }
    if !store::save_in(config_dir, &all) {
        return Err(m::save_failed(file_label(config_dir)));
    }
    Ok((id, name, created))
}

fn file_label(config_dir: &Path) -> String {
    gitpoulpi_session::Workspaces::file_in(config_dir)
        .display()
        .to_string()
}

/// Saves `draft`; the origin of each repository the workspace does not know
/// yet is read through the daemon first.
pub(crate) fn save(session: String, draft: Draft) -> Receiver {
    spawn(move |tx, runtime| {
        let Some(dir) = crate::session_store::config_dir() else {
            let _ = tx.send_notify(Event::Failed(m::SAVE_NO_CONFIG.get().into()));
            return;
        };
        let known = super::store::load_in(&dir);
        let had = draft.id.as_deref().and_then(|id| known.get(id));
        let mut origins = std::collections::HashMap::new();
        runtime.block_on(async {
            let mut conn = None;
            for path in &draft.repos {
                if had.is_some_and(|w| w.remote_of(path).is_some()) || !path.exists() {
                    continue;
                }
                if conn.is_none() {
                    conn = Conn::open(&session).await.ok();
                }
                if let Some(conn) = conn.as_mut() {
                    if let Some(url) = conn.origin(path).await {
                        origins.insert(path.clone(), url);
                    }
                }
            }
        });
        let _ = tx.send_notify(match write_draft(&dir, &draft, &origins) {
            Ok((id, name, created)) => Event::Saved { id, name, created },
            Err(why) => Event::Failed(why),
        });
    })
}

/// Records `path`'s origin in workspace `id` (a repository added from a tab
/// menu or the open tabs).
pub(crate) fn add_with_origin(session: String, id: String, path: PathBuf) -> Receiver {
    spawn(move |tx, runtime| {
        let origin = runtime.block_on(async {
            match Conn::open(&session).await {
                Ok(mut conn) => conn.origin(&path).await,
                Err(_) => None,
            }
        });
        let name = super::store::load()
            .get(&id)
            .map(|w| w.name.clone())
            .unwrap_or_default();
        let repo = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let _ = tx.send_notify(
            match super::store::add_repo(&id, &path, origin.as_deref()) {
                Some(true) => Event::Done(m::added_to(repo, name)),
                Some(false) => Event::Done(m::already_in(repo, name)),
                None => Event::Failed(m::save_failed(path.display())),
            },
        );
    })
}

/// Fetches or pulls every repository of `repos`, in order.
pub(crate) fn bulk(session: String, kind: Bulk, name: String, repos: Vec<PathBuf>) -> Receiver {
    spawn_stoppable(move |tx, runtime, stop| {
        let total = repos.len();
        let mut done = 0usize;
        let mut failed: Vec<String> = Vec::new();
        runtime.block_on(async {
            let mut conn = None;
            for (i, path) in repos.iter().enumerate() {
                if stop.load(std::sync::atomic::Ordering::SeqCst) {
                    break;
                }
                let shown = label(path);
                let line = match kind {
                    Bulk::Fetch => m::fetching_progress(&name, i, total as u64),
                    Bulk::Pull => m::pulling_progress(&name, i, total as u64),
                };
                if tx.send_notify(Event::Progress(line)).is_err() {
                    return;
                }
                if !path.exists() {
                    failed.push(m::failure(shown, m::MISSING_ON_DISK.get()));
                    continue;
                }
                if conn.is_none() {
                    match Conn::open(&session).await {
                        Ok(c) => conn = Some(c),
                        Err(e) => {
                            failed.push(m::failure(shown, e));
                            continue;
                        }
                    }
                }
                let conn = conn.as_mut().expect("opened above");
                // Fast-forward only: a walk over many repositories must
                // never stop on a merge conflict in the one nobody is
                // looking at.
                let result = match kind {
                    Bulk::Fetch => {
                        let opts = crate::live_settings::fetch_options();
                        conn.call(
                            Some(path),
                            OperationKind::FetchAll,
                            serde_json::json!({ "opts": opts }),
                        )
                        .await
                    }
                    Bulk::Pull => {
                        let opts = PullOptions {
                            mode: PullMode::FastForwardOnly,
                            remote: None,
                            ..PullOptions::default()
                        };
                        conn.call(
                            Some(path),
                            OperationKind::Pull,
                            serde_json::json!({ "opts": opts }),
                        )
                        .await
                    }
                };
                match result {
                    Ok(_) => done += 1,
                    Err(e) => failed.push(m::failure(shown, e)),
                }
            }
        });
        let list = failed.join("; ");
        let text = match (kind, failed.is_empty()) {
            (Bulk::Fetch, true) => m::fetch_done(&name, done as u64, total as u64),
            (Bulk::Fetch, false) => m::fetch_done_failed(&name, done as u64, total as u64, list),
            (Bulk::Pull, true) => m::pull_done(&name, done as u64, total as u64),
            (Bulk::Pull, false) => m::pull_done_failed(&name, done as u64, total as u64, list),
        };
        let _ = tx.send_notify(Event::Done(text));
    })
}

/// Clones each of `missing` again from its recorded remote.
pub(crate) fn clone_missing(session: String, name: String, missing: Vec<Missing>) -> Receiver {
    spawn_stoppable(move |tx, runtime, stop| {
        let total = missing.len();
        let mut done = 0usize;
        let mut failed: Vec<String> = Vec::new();
        runtime.block_on(async {
            let mut conn = None;
            for (i, item) in missing.iter().enumerate() {
                if stop.load(std::sync::atomic::Ordering::SeqCst) {
                    break;
                }
                let shown = label(&item.path);
                if tx
                    .send_notify(Event::Progress(m::cloning_progress(&name, i, total as u64)))
                    .is_err()
                {
                    return;
                }
                let Some(url) = item.url.as_deref() else {
                    failed.push(m::failure(shown, m::NO_REMOTE.get()));
                    continue;
                };
                if conn.is_none() {
                    match Conn::open(&session).await {
                        Ok(c) => conn = Some(c),
                        Err(e) => {
                            failed.push(m::failure(shown, e));
                            continue;
                        }
                    }
                }
                let conn = conn.as_mut().expect("opened above");
                let result = conn
                    .call(
                        None,
                        OperationKind::Clone,
                        serde_json::json!({
                            "url": url,
                            "dest": item.path,
                            "opts": CloneOptions::default(),
                        }),
                    )
                    .await;
                match result {
                    Ok(_) => done += 1,
                    Err(e) => failed.push(m::failure(shown, e)),
                }
            }
        });
        let list = failed.join("; ");
        let text = if failed.is_empty() {
            m::clone_done(&name, done as u64, total as u64)
        } else {
            m::clone_done_failed(&name, done as u64, total as u64, list)
        };
        let _ = tx.send_notify(Event::Done(text));
    })
}

/// A repository's name in a status line: its folder.
fn label(path: &Path) -> String {
    path.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.display().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use gitpoulpi_session::Workspaces;

    #[test]
    fn credentials_are_stripped_from_a_recorded_url() {
        assert_eq!(
            without_userinfo("https://me:tok@host.example/o/r.git"),
            "https://host.example/o/r.git"
        );
        assert_eq!(
            without_userinfo("git@host.example:o/r.git"),
            "git@host.example:o/r.git"
        );
        assert_eq!(without_userinfo("https://host/o@x/r"), "https://host/o@x/r");
    }

    #[test]
    fn saving_a_draft_creates_then_edits_and_keeps_the_ids_stable() {
        let dir = tempfile::tempdir().unwrap();
        let a = dir.path().join("a");
        let b = dir.path().join("b");
        std::fs::create_dir_all(&a).unwrap();
        std::fs::create_dir_all(&b).unwrap();
        let mut origins = std::collections::HashMap::new();
        origins.insert(a.clone(), "git@h:o/a.git".to_string());
        let draft = Draft {
            name: " Acme ".into(),
            color: Some("green".into()),
            description: "the stack".into(),
            profile: Some("work".into()),
            repos: vec![a.clone(), b.clone()],
            ..Draft::default()
        };
        let (id, name, created) = write_draft(dir.path(), &draft, &origins).unwrap();
        assert_eq!(
            (id.as_str(), name.as_str(), created),
            ("acme", "Acme", true)
        );
        let all = Workspaces::load(&Workspaces::file_in(dir.path()));
        let ws = all.get("acme").unwrap();
        assert_eq!(ws.repos, vec![a.clone(), b.clone()]);
        assert_eq!(ws.remote_of(&a), Some("git@h:o/a.git"));
        assert_eq!(ws.profile.as_deref(), Some("work"));

        // Edit: renamed, `b` dropped, no profile; the id stays.
        let edit = Draft {
            id: Some("acme".into()),
            name: "Acme Corp".into(),
            repos: vec![a.clone()],
            ..Draft::default()
        };
        let (id, _, created) = write_draft(dir.path(), &edit, &Default::default()).unwrap();
        assert_eq!((id.as_str(), created), ("acme", false));
        let ws = Workspaces::load(&Workspaces::file_in(dir.path()))
            .get("acme")
            .unwrap()
            .clone();
        assert_eq!(ws.name, "Acme Corp");
        assert_eq!(ws.repos, vec![a.clone()]);
        assert_eq!(ws.profile, None);
        assert_eq!(ws.remote_of(&a), Some("git@h:o/a.git"), "a kept URL stays");
    }

    #[test]
    fn the_ai_instructions_are_saved_with_the_workspace_and_left_alone_when_the_form_had_none() {
        use gitpoulpi_config::{RepoAiInstructions, RepoInstructionsMode, WorkspaceAi};
        let dir = tempfile::tempdir().unwrap();
        let ai = WorkspaceAi {
            general: RepoAiInstructions {
                text: "Use the glossary.".into(),
                mode: RepoInstructionsMode::Replace,
            },
            actions: [
                (
                    "code_review".to_string(),
                    RepoAiInstructions {
                        text: "Flag missing tests.".into(),
                        mode: RepoInstructionsMode::Add,
                    },
                ),
                ("stash_message".to_string(), RepoAiInstructions::default()),
            ]
            .into(),
        };
        let draft = Draft {
            name: "Acme".into(),
            ai: Some(ai),
            ..Draft::default()
        };
        write_draft(dir.path(), &draft, &Default::default()).unwrap();
        let load = || Workspaces::load(&Workspaces::file_in(dir.path()));
        let ws = load().get("acme").unwrap().clone();
        assert_eq!(ws.ai_instructions.text, "Use the glossary.");
        assert_eq!(ws.ai_overrides.len(), 1, "an empty entry is not stored");
        assert_eq!(ws.ai_overrides["code_review"].text, "Flag missing tests.");

        // A form that never showed the AI row (AI off) keeps what is there.
        let rename = Draft {
            id: Some("acme".into()),
            name: "Acme Corp".into(),
            ..Draft::default()
        };
        write_draft(dir.path(), &rename, &Default::default()).unwrap();
        let ws = load().get("acme").unwrap().clone();
        assert_eq!(ws.name, "Acme Corp");
        assert_eq!(ws.ai_instructions.text, "Use the glossary.");

        // Cleared in the form: gone from the file.
        let cleared = Draft {
            id: Some("acme".into()),
            name: "Acme Corp".into(),
            ai: Some(WorkspaceAi::default()),
            ..Draft::default()
        };
        write_draft(dir.path(), &cleared, &Default::default()).unwrap();
        let raw = std::fs::read_to_string(Workspaces::file_in(dir.path())).unwrap();
        assert!(
            !raw.contains("ai_instructions") && !raw.contains("code_review"),
            "{raw}"
        );
    }
}
