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
//! The terminal loop checks [`epoch`] a few dozen times per second while
//! idle, which costs nothing next to a frame; crossterm offers no way to wake
//! its own `poll` from another thread. A frontend that can be woken (a
//! desktop window) installs a waker with [`install_waker`] and is called on
//! every [`poke`]. Nothing here knows about a terminal.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};

static EPOCH: AtomicU64 = AtomicU64::new(0);

type Waker = Arc<dyn Fn() + Send + Sync>;

static WAKER: RwLock<Option<Waker>> = RwLock::new(None);

/// Installs (or, with `None`, removes) the callback run after every [`poke`],
/// so a frontend can schedule a frame. It runs on the poking thread: keep it
/// short and never call [`poke`] from it.
#[allow(dead_code)] // Called by the second frontend; the terminal polls the epoch.
pub(crate) fn install_waker(waker: Option<Waker>) {
    if let Ok(mut slot) = WAKER.write() {
        *slot = waker;
    }
}

/// Something the UI thread has not seen changed: draw again.
pub(crate) fn poke() {
    EPOCH.fetch_add(1, Ordering::SeqCst);
    // Cloned out so the callback never runs under the lock.
    let waker = WAKER.read().ok().and_then(|slot| slot.clone());
    if let Some(waker) = waker {
        waker();
    }
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

    #[test]
    fn an_installed_waker_runs_on_every_poke_until_removed() {
        use std::sync::atomic::AtomicUsize;
        static CALLS: AtomicUsize = AtomicUsize::new(0);
        // Other tests poke from their own threads: count only this one's.
        let me = std::thread::current().id();
        install_waker(Some(Arc::new(move || {
            if std::thread::current().id() == me {
                CALLS.fetch_add(1, Ordering::SeqCst);
            }
        })));
        poke();
        assert_eq!(CALLS.load(Ordering::SeqCst), 1);
        install_waker(None);
        let seen = CALLS.load(Ordering::SeqCst);
        poke();
        assert_eq!(CALLS.load(Ordering::SeqCst), seen);
    }
}
