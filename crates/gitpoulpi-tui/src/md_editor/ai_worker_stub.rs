//! `ai_worker` for a build without the `ai` feature: the owners of an editor
//! only bind the helper while AI is on, so this is never started; it answers
//! with the typed "not built" message if it ever were.

use std::sync::atomic::AtomicBool;
use std::sync::{mpsc, Arc};

use super::ai::{AiScope, AiTask, Answer};

pub(super) struct Input {
    pub(super) scope: AiScope,
    pub(super) task: AiTask,
    pub(super) cancel: Arc<AtomicBool>,
    pub(super) read_repo: bool,
    pub(super) context: Arc<std::sync::Mutex<Option<String>>>,
    #[cfg(test)]
    pub(super) fake: Option<super::ai::Fake>,
}

pub(super) fn start(_: Input) -> mpsc::Receiver<Answer> {
    let (tx, rx) = mpsc::channel();
    let _ = tx.send(Err(
        gitpoulpi_core::Error::NotBuilt { feature: "ai" }.to_string()
    ));
    rx
}
