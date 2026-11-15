//! A `RepoScopes` for the conversation tests: a configured name, a learned
//! login that is remembered, and whether the repository uses a token command.

use std::sync::{Arc, Mutex};

use crate::scope::RepoScopes;
use crate::scope_handle::ScopeHandle;
use crate::Result;

#[derive(Default)]
pub(crate) struct FakeScopes {
    pub post_as: Option<String>,
    pub learned: Mutex<Option<String>>,
    pub bot: bool,
}

impl FakeScopes {
    pub(crate) fn handle(self) -> (ScopeHandle, Arc<FakeScopes>) {
        let me = Arc::new(self);
        (ScopeHandle::injected(me.clone()), me)
    }

    pub(crate) fn learned(&self) -> Option<String> {
        self.learned.lock().unwrap().clone()
    }
}

#[async_trait::async_trait]
impl RepoScopes for FakeScopes {
    fn assignment_blocked(&self, _: &str, _: &str) -> bool {
        false
    }

    async fn token(&self, _: &str, _: &str, _: bool) -> Option<Result<String>> {
        None
    }

    fn acts_as_bot(&self, _: &str, _: &str) -> bool {
        self.bot
    }

    fn post_as(&self, _: &str, _: &str) -> Option<String> {
        self.post_as.clone()
    }

    fn learned_login(&self, _: &str, _: &str) -> Option<String> {
        self.learned()
    }

    fn learn_login(&self, _: &str, _: &str, login: &str) {
        *self.learned.lock().unwrap() = Some(login.to_string());
    }
}
