//! What the repository's host protects, for the history rewrite workbench
//! (S23). Hosting is opt-in per repository: the host is asked only when the
//! user linked the repository (a pulls or issues source, or a pinned
//! profile); an unlinked repository still gets the how-to's settings links,
//! built from its remote's URL, and no request is made for it.

use std::sync::mpsc;

use crate::redraw::NotifySend as _;
use crate::rewrite_view::HostHandle;

use super::*;

impl HostingUi {
    /// The host of this repository, and (when it is linked) the lookup of
    /// its protected branches on its own thread, answering once. The outer
    /// `None`: the repository has not been read yet, ask again; the inner
    /// one: no host can be named.
    pub(crate) fn rewrite_host(&self) -> Option<Option<HostHandle>> {
        let inputs = self.inputs.as_ref()?;
        Some(self.rewrite_host_of(inputs))
    }

    fn rewrite_host_of(&self, inputs: &Inputs) -> Option<HostHandle> {
        let integrations = (self.integrations)();
        let link = self
            .pulls_source
            .as_ref()
            .or(self.issues_source.as_ref())
            .map(|s| (s.integration.as_str(), s.project.as_str()));
        let located = gitpoulpi_hosting::protection::locate(
            &inputs.remotes,
            &integrations,
            link,
            self.override_id.as_deref(),
            None,
        )?;
        let base = located.protection(None);
        let (Some(factory), Some(integration), true) = (
            self.factory.clone(),
            located.integration.clone(),
            located.linked,
        ) else {
            return Some(HostHandle { base, answer: None });
        };
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let answer = (|| {
                let runtime = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .map_err(|e| e.to_string())?;
                let client = factory
                    .client(&integration, &integrations)
                    .map_err(|e| gitpoulpi_hosting::protection::unknown_reason(&e))?;
                Ok::<_, String>(runtime.block_on(gitpoulpi_hosting::protection::lookup(
                    &*client,
                    &located.project,
                )))
            })();
            let lookup =
                answer.unwrap_or_else(gitpoulpi_hosting::protection::ProtectionLookup::Unknown);
            let _ = tx.send_notify(located.protection(Some(lookup)));
        });
        Some(HostHandle {
            base,
            answer: Some(rx),
        })
    }
}
