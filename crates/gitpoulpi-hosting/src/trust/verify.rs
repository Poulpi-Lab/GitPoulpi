//! The account id check: the token still belongs to the account it was
//! trusted for.

use std::collections::HashSet;
use std::sync::Mutex;

use gitpoulpi_config::Integration;

use crate::client::HostingClient;
use crate::{Error, Result};

use super::{report, unix_now, Account, TrustGate, TrustProblem, TrustSubject};

/// Why the account is being asked.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerifyReason {
    /// A token was just stored: always check.
    TokenStored,
    /// The token came from `GITPOULPI_TOKEN_*` or the credential helper:
    /// once per process.
    ExternalToken,
    /// An ordinary use: when the last check is older than a day.
    Periodic,
}

/// How long a successful check is good for.
pub const RECHECK_SECS: u64 = 24 * 3600;

static CHECKED: Mutex<Option<HashSet<String>>> = Mutex::new(None);

fn first_time_this_process(id: &str) -> bool {
    CHECKED
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .get_or_insert_with(HashSet::new)
        .insert(id.to_string())
}

impl TrustGate {
    /// Compares the account `client`'s token belongs to with the recorded
    /// one, when [`VerifyReason`] says it is due. The **id** is compared, not
    /// the login (a login can be renamed or re-registered). A record with no
    /// account yet (a migrated one) learns it here.
    pub async fn verify_account(
        &self,
        integration: &Integration,
        client: &dyn HostingClient,
        reason: VerifyReason,
    ) -> Result<()> {
        let record = self.record_of(&integration.id);
        let due = match reason {
            VerifyReason::TokenStored => true,
            VerifyReason::ExternalToken => first_time_this_process(&integration.id),
            VerifyReason::Periodic => record
                .as_ref()
                .is_none_or(|r| unix_now().saturating_sub(r.account_checked_at) >= RECHECK_SECS),
        };
        if !due {
            return Ok(());
        }
        let user = match client.current_user().await {
            Ok(u) => u,
            // A kind that cannot say who the token is has nothing to compare.
            Err(Error::Unsupported(_) | Error::NotImplemented(_) | Error::BotToken) => {
                return Ok(())
            }
            Err(e) => return Err(e),
        };
        let new = Account {
            login: user.login,
            id: user.id,
        };
        self.check_account(integration, &new)
    }

    /// Compares `new` with the recorded account and, when it is the same one
    /// (or none was recorded yet), refreshes the record. The part of
    /// [`Self::verify_account`] for a caller that already holds the account,
    /// from a token check it just ran.
    pub fn check_account(&self, integration: &Integration, new: &Account) -> Result<()> {
        let record = self.record_of(&integration.id);
        if let Some(r) = &record {
            let same = if r.account_id.is_empty() || new.id.is_empty() {
                r.account_login.is_empty() || r.account_login == new.login
            } else {
                r.account_id == new.id
            };
            if !same {
                let problem = TrustProblem::AccountChanged {
                    subject: TrustSubject::Integration(integration.id.clone()),
                    old: Box::new(Account {
                        login: r.account_login.clone(),
                        id: r.account_id.clone(),
                    }),
                    new: Box::new(new.clone()),
                };
                report(&problem);
                return Err(Error::Trust(problem));
            }
        }
        self.record_after_check(integration, new)
    }
}
