//! How an AI request ended and the process-wide record of ends nobody has
//! taken yet, so a waiter (a sequence step) can read the outcome of a request
//! whichever window applied it. The request and answer types, the run and the
//! popups stay in the frontend.

use gitpoulpi_config::AiAction;

/// How a finished request ended, for whoever waits on it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AiEnd {
    Ok,
    Failed(String),
    Canceled,
}

/// The ends nobody has taken yet, by kind of request. Process-wide: the
/// window that polled the answer may not be the one that asked, and a tab
/// switch rebuilds it.
static FINISHED: std::sync::Mutex<Vec<(AiAction, AiEnd)>> = std::sync::Mutex::new(Vec::new());

/// Makes the end of a request visible to [`take_finished`]; call it once the
/// answer has been applied. Only the newest end of a kind is kept.
pub fn record_finished((kind, end): (AiAction, AiEnd)) {
    let mut ends = FINISHED.lock().unwrap_or_else(|e| e.into_inner());
    ends.retain(|(k, _)| *k != kind);
    ends.push((kind, end));
}

/// The end of the last finished request of `kind`, once. `None` while it
/// runs (or nothing of that kind finished since the last call).
pub fn take_finished(kind: AiAction) -> Option<AiEnd> {
    let mut ends = FINISHED.lock().unwrap_or_else(|e| e.into_inner());
    let at = ends.iter().position(|(k, _)| *k == kind)?;
    Some(ends.remove(at).1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_newest_end_of_a_kind_is_taken_once() {
        record_finished((AiAction::CommitMessage, AiEnd::Canceled));
        record_finished((AiAction::CommitMessage, AiEnd::Ok));
        assert_eq!(take_finished(AiAction::CommitMessage), Some(AiEnd::Ok));
        assert_eq!(take_finished(AiAction::CommitMessage), None);
    }
}
