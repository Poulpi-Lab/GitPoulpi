use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct StashId(pub u32);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stash {
    pub id: StashId,
    pub message: String,
    pub branch_at_creation: String,
    pub timestamp: i64,
    /// The stash's own commit (`stash@{n}` resolved).
    ///
    /// `Option` and `serde(default)` for one reason only: a daemon built
    /// before these two fields existed still answers a newer client, and
    /// its stash list must keep deserializing. Both backends always fill
    /// them (`check_stash_push_pop`).
    #[serde(default)]
    pub commit: Option<super::Oid>,
    /// The commit the stash was made on: the stash commit's first parent.
    /// The Log draws the stash beside it, joined by a dashed connector.
    #[serde(default)]
    pub base: Option<super::Oid>,
}
