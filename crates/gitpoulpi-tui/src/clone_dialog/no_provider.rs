//! Stand-ins for the hosting types the Clone dialog's repository browser
//! uses, in a build with no provider: the dialog never has a hosted
//! source there, so these are never constructed.
#![allow(dead_code)]

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HostedProject {
    pub(crate) name: String,
    pub(crate) full_name: String,
    pub(crate) private: bool,
    pub(crate) description: Option<String>,
}

impl HostedProject {
    pub(crate) fn owner(&self) -> &str {
        ""
    }

    pub(crate) fn matches(&self, _query: &str) -> bool {
        false
    }

    pub(crate) fn clone_url(&self, _ssh: bool) -> Option<&str> {
        None
    }
}

#[derive(Debug, Clone)]
pub(crate) enum ProjectsState {
    Loading,
    Ready(Vec<HostedProject>),
    Failed(String),
}
