//! Redraw on demand: how a worker thread tells the render loop "something
//! changed, draw again".
//!
//! The loop (`event_loop/window/idle.rs`) draws only when something may have
//! changed: an input event, a timer coming due, or [`poke`]. Every worker
//! that hands a result to the UI thread does so through
//! [`NotifySend::send_notify`] (a channel send followed by a poke) or calls
//! [`poke`] itself after it changed shared state. The poke comes *after* the
//! send, so a loop that wakes on it always finds the message.
//!
//! The loop checks [`epoch`] a few dozen times per second while idle, which
//! costs nothing next to a frame; crossterm offers no way to wake its own
//! `poll` from another thread.

use std::sync::atomic::{AtomicU64, Ordering};

static EPOCH: AtomicU64 = AtomicU64::new(0);

/// Something the UI thread has not seen changed: draw again.
pub(crate) fn poke() {
    EPOCH.fetch_add(1, Ordering::SeqCst);
}

/// Moves on every [`poke`].
pub(crate) fn epoch() -> u64 {
    EPOCH.load(Ordering::SeqCst)
}

/// `send`, then [`poke`]: the way a worker answers the render loop.
pub(crate) trait NotifySend<T> {
    fn send_notify(&self, value: T) -> Result<(), std::sync::mpsc::SendError<T>>;
}

impl<T> NotifySend<T> for std::sync::mpsc::Sender<T> {
    fn send_notify(&self, value: T) -> Result<(), std::sync::mpsc::SendError<T>> {
        let sent = self.send(value);
        poke();
        sent
    }
}

impl<T> NotifySend<T> for std::sync::mpsc::SyncSender<T> {
    fn send_notify(&self, value: T) -> Result<(), std::sync::mpsc::SendError<T>> {
        let sent = self.send(value);
        poke();
        sent
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_notified_send_moves_the_epoch_after_the_value_is_queued() {
        let (tx, rx) = std::sync::mpsc::channel();
        let before = epoch();
        tx.send_notify(7).unwrap();
        assert!(epoch() > before);
        assert_eq!(rx.try_recv(), Ok(7));
    }
}
