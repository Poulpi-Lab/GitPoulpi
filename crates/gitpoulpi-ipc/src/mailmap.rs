//! The `.mailmap` requests (slice S16, the history-rewrite design):
//! the `args` of one `OperationKind` each.
//!
//! | `OperationKind` | `args` | answer |
//! |---|---|---|
//! | `MailmapIdentities` | [`IdentitiesArgs`] | `IdentityList` (a read) |
//! | `MailmapResolve` | [`ResolveArgs`] | `Vec<Ident>` (a read) |
//! | `MailmapAdd` | [`AddArgs`] | `MailmapAdded` (journaled: undo restores the file) |

use gitpoulpi_core::domain::rewrite::{Ident, Scope, ScopeOptions};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct IdentitiesArgs {
    #[serde(default)]
    pub scope: Scope,
    #[serde(default)]
    pub options: ScopeOptions,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolveArgs {
    pub idents: Vec<Ident>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AddArgs {
    /// The identity the lines map to.
    pub canonical: Ident,
    /// The identities that become `canonical`.
    pub from: Vec<Ident>,
    /// Repo-relative `.mailmap` to write; the root one when absent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
}
