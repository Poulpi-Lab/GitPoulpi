//! Test support: a spawned link loop that is stopped, never leaked.
//!
//! A test that `tokio::spawn`s `link_loop_with_port` and returns leaves the
//! loop (and the refresh it runs) alive after the fixture repository is
//! deleted: a `for-each-ref` then races the deletion. `stop` aborts the loop
//! and waits for it; dropping the guard (a panicking test) aborts it too.

pub(super) struct LinkGuard(Option<tokio::task::JoinHandle<()>>);

impl LinkGuard {
    pub(super) fn spawn<F>(link: F) -> Self
    where
        F: std::future::Future + Send + 'static,
        F::Output: Send + 'static,
    {
        Self(Some(tokio::spawn(async move {
            let _ = link.await;
        })))
    }

    /// Aborts the loop and waits until it is gone.
    pub(super) async fn stop(mut self) {
        if let Some(task) = self.0.take() {
            task.abort();
            let _ = task.await;
        }
    }
}

impl Drop for LinkGuard {
    fn drop(&mut self) {
        if let Some(task) = self.0.take() {
            task.abort();
        }
    }
}
