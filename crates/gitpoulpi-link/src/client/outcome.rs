use super::host::NotifySend as _;
use super::*;

/// Deletes every `*.orig` file under `root`, skipping `.git` - what
/// `general.delete_orig_files_after_merge` asks for after a merge lands
/// cleanly. Returns how many were removed, best-effort: a file that
/// cannot be read or removed is skipped rather than aborting the sweep:
/// state that is nice to have must never take the app down.
pub(super) fn cleanup_orig_files(root: &std::path::Path) -> usize {
    fn walk(dir: &std::path::Path, count: &mut usize) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let Ok(file_type) = entry.file_type() else {
                continue;
            };
            if file_type.is_dir() {
                if path.file_name().and_then(|n| n.to_str()) == Some(".git") {
                    continue;
                }
                walk(&path, count);
            } else if path.extension().and_then(|e| e.to_str()) == Some("orig")
                && std::fs::remove_file(&path).is_ok()
            {
                *count += 1;
            }
        }
    }
    let mut count = 0;
    walk(root, &mut count);
    count
}

/// The status line for a pull's outcome.
pub(super) fn describe_pull(outcome: gitpoulpi_core::domain::PullOutcome) -> String {
    use gitpoulpi_core::domain::PullOutcome;
    use gitpoulpi_i18n::msg::link as l;
    match outcome {
        PullOutcome::UpToDate => l::UP_TO_DATE.get().to_string(),
        PullOutcome::FastForwarded(oid) => l::pulled_fast_forward(short(oid)),
        PullOutcome::Merged(oid) => l::pulled_merged(short(oid)),
        PullOutcome::Conflicted => l::PULL_CONFLICTED.get().to_string(),
    }
}

/// Turns a successful operation's typed outcome into a status-bar line
/// and swallows it, leaving `run_action`'s uniform `GitResult<()>`.
///
/// Every outcome-carrying action needs the same three steps: send the
/// message, keep the error, forget the value.
pub(super) fn report<T>(
    tx: &std_mpsc::Sender<String>,
    outcome: GitResult<T>,
    describe: impl FnOnce(T) -> String,
) -> GitResult<()> {
    match outcome {
        Ok(value) => {
            let _ = tx.send_notify(describe(value));
            Ok(())
        }
        Err(e) => Err(e),
    }
}

/// The status line after a reword. It names the old tip, because that is
/// the by-hand undo: `reset --soft` to it (also in `ORIG_HEAD` and the
/// reflog) puts the branch back. Undo (the journal) reverts a reword too.
pub fn reword_done_message(commit: Oid, outcome: &RewordOutcome) -> String {
    use gitpoulpi_i18n::msg::link as l;
    match outcome.replayed {
        0 => l::reword_done(
            short(commit),
            short(outcome.reworded),
            short(outcome.previous_head),
        ),
        n => l::reword_done_replayed(
            n as u64,
            short(commit),
            short(outcome.reworded),
            short(outcome.previous_head),
        ),
    }
}

/// The status line after an amend: like [`reword_done_message`], it names
/// the old tip (the by-hand undo) and says Undo reverts it.
pub fn amend_done_message(commit: Oid, outcome: &RewordOutcome) -> String {
    gitpoulpi_i18n::msg::link::amend_done(
        short(commit),
        short(outcome.reworded),
        short(outcome.previous_head),
    )
}

/// The status line after a split: the journal's own words for it, then the
/// new tip.
pub fn split_done_message(commit: Oid, outcome: &SplitOutcome) -> String {
    use gitpoulpi_i18n::msg::{split_ui, undo_label};
    split_ui::done(
        undo_label::split_commit(outcome.created.len() as u64, short(commit)),
        short(outcome.new_head),
    )
}

/// The newest commit each repository's last split created, waiting for its
/// window to select it in the Log (the status line cannot carry an id).
static SPLIT_CREATED: std::sync::Mutex<Vec<(std::path::PathBuf, Oid)>> =
    std::sync::Mutex::new(Vec::new());

/// Remembers that a split of the repository at `root` ended with `newest`.
pub(super) fn note_split_created(root: &std::path::Path, newest: Oid) {
    if let Ok(mut held) = SPLIT_CREATED.lock() {
        held.retain(|(r, _)| r != root);
        held.push((root.to_path_buf(), newest));
    }
}

/// The commit a finished split of the repository at `root` wants selected.
pub fn take_split_created(root: &std::path::Path) -> Option<Oid> {
    let mut held = SPLIT_CREATED.lock().ok()?;
    let at = held.iter().position(|(r, _)| r == root)?;
    Some(held.remove(at).1)
}

/// One request whose typed answer is turned into a status line.
pub(super) async fn reported<T: serde::de::DeserializeOwned>(
    conn: &LinkConn,
    tx: &std_mpsc::Sender<String>,
    command: OperationKind,
    args: serde_json::Value,
    describe: impl FnOnce(T) -> String,
) -> GitResult<()> {
    report(tx, conn.call(command, args).await, describe)
}

/// The status line for a rebase's outcome; `done` is what a completed one
/// says.
pub(super) fn describe_rebase(done: gitpoulpi_i18n::M) -> impl FnOnce(RebaseOutcome) -> String {
    use gitpoulpi_i18n::msg::link as l;
    move |outcome| match outcome {
        RebaseOutcome::Completed => done.get().to_string(),
        RebaseOutcome::Conflicted { paused_at, .. } => l::rebase_paused(short(paused_at)),
        RebaseOutcome::Stopped { at, .. } => l::rebase_stopped(short(at)),
    }
}
