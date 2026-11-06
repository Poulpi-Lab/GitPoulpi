//! The policy as the running machine has it: the policy file of the OS, and
//! on Windows the `HKLM\SOFTWARE\Policies\GitPoulpi\Updates` key over it
//! (`gitpoulpi_config::policy` merges the layers).

use std::collections::BTreeMap;

use gitpoulpi_config::policy::{self, PolicySource};

/// The real [`PolicySource`].
pub struct SystemPolicySource;

impl PolicySource for SystemPolicySource {
    fn file_text(&self) -> Option<String> {
        let path = policy::policy_paths().into_iter().next()?;
        std::fs::read_to_string(path).ok()
    }

    fn registry_values(&self) -> Option<BTreeMap<String, String>> {
        #[cfg(windows)]
        {
            crate::winreg::policy_values()
        }
        #[cfg(not(windows))]
        {
            None
        }
    }
}
