//! `ai_worker` for a build without the `ai` feature: the ✦ controls never
//! show (`Suggest::enabled` is false), so this is never started; it answers
//! with the typed "not built" message if it ever were.

use std::sync::mpsc;

use super::ai::{Answer, AskInput};

pub(super) fn start(_: AskInput) -> mpsc::Receiver<Answer> {
    let (tx, rx) = mpsc::channel();
    let _ = tx.send(Err(
        gitpoulpi_core::Error::NotBuilt { feature: "ai" }.to_string()
    ));
    rx
}
