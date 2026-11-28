//! How a finished AI request ended, for whoever waits on it (a sequence step).

/// The end of a request, as a waiter sees it: the answer itself is applied by
/// the frontend that asked, so the end carries only the outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AiEnd {
    Ok,
    Failed(String),
    Canceled,
}
