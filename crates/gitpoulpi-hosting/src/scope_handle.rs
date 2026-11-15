//! The scopes a client consults: the process's installed ones, or the ones a
//! test injected (the installed registry is process-global, so a test that
//! needs its own cannot share it).

use std::sync::Arc;

use crate::scope::{installed, RepoScopes};

#[derive(Clone, Default)]
pub(crate) struct ScopeHandle(Option<Arc<dyn RepoScopes>>);

impl ScopeHandle {
    #[cfg(test)]
    pub(crate) fn injected(scopes: Arc<dyn RepoScopes>) -> Self {
        Self(Some(scopes))
    }

    fn get(&self) -> Option<&dyn RepoScopes> {
        match &self.0 {
            Some(s) => Some(s.as_ref()),
            None => installed().map(|s| s.as_ref()),
        }
    }

    pub(crate) fn acts_as_bot(&self, id: &str, project: &str) -> bool {
        self.get().is_some_and(|s| s.acts_as_bot(id, project))
    }

    pub(crate) fn post_as(&self, id: &str, project: &str) -> Option<String> {
        self.get()
            .and_then(|s| s.post_as(id, project))
            .map(|n| n.trim().to_string())
            .filter(|n| !n.is_empty())
    }

    pub(crate) fn learned_login(&self, id: &str, project: &str) -> Option<String> {
        self.get().and_then(|s| s.learned_login(id, project))
    }

    pub(crate) fn learn_login(&self, id: &str, project: &str, login: &str) {
        if let Some(s) = self.get() {
            s.learn_login(id, project, login);
        }
    }
}
