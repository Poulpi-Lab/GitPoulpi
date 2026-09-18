//! The merge request a `merge-request` finish leaves for the client: the
//! daemon pushed the branch (it holds no hosting token), the window opens the
//! existing Create Pull Request form prefilled (or the compare URL when no
//! integration answers for the remote). The dialog [`offer`]s it when the
//! run ended; the window takes it once its refs show the push.

use std::sync::Mutex;
use std::time::Instant;

use gitpoulpi_core::workflow::MergeRequestSpec;

static PENDING: Mutex<Option<(MergeRequestSpec, Instant)>> = Mutex::new(None);

/// A finished run wants this merge request opened.
pub(crate) fn offer(spec: MergeRequestSpec) {
    if let Ok(mut slot) = PENDING.lock() {
        *slot = Some((spec, Instant::now()));
    }
    crate::redraw::poke();
    // The window gives up waiting for the refs after `WAIT`: wake it then.
    std::thread::spawn(|| {
        std::thread::sleep(WAIT + std::time::Duration::from_millis(100));
        crate::redraw::poke();
    });
}

/// How long the window waits for its refs to show the pushed branch.
pub(crate) const WAIT: std::time::Duration = std::time::Duration::from_secs(3);

/// The offered request, once `ready(spec, waited)` says the window can open
/// it (its refs know the pushed branch, or it waited long enough).
pub(crate) fn take_when(
    ready: impl Fn(&MergeRequestSpec, std::time::Duration) -> bool,
) -> Option<MergeRequestSpec> {
    let mut slot = PENDING.lock().ok()?;
    let (spec, since) = slot.as_ref()?;
    if ready(spec, since.elapsed()) {
        return slot.take().map(|(spec, _)| spec);
    }
    None
}
