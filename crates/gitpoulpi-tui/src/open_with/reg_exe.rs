//! The real Windows registry reader: `reg.exe query <key>`, one process per
//! key, answers cached for the lookup. Only ever called from a lookup thread
//! (a spawn costs hundreds of milliseconds there), never the input loop.

use std::collections::HashMap;
use std::os::windows::process::CommandExt;
use std::process::{Command, Stdio};
use std::sync::Mutex;

use gitpoulpi_config::os_apps::windows::{parse_reg_query, Registry};

type Answer = (Vec<String>, Vec<(String, String)>);

#[derive(Default)]
pub(super) struct RegExe {
    cache: Mutex<HashMap<String, Answer>>,
}

impl RegExe {
    fn query(&self, key: &str) -> Answer {
        let lower = key.to_lowercase();
        if let Some(hit) = self.cache.lock().ok().and_then(|c| c.get(&lower).cloned()) {
            return hit;
        }
        let answer = Command::new("reg")
            .args(["query", key])
            .stdin(Stdio::null())
            .stderr(Stdio::null())
            .creation_flags(0x0800_0000) // CREATE_NO_WINDOW
            .output()
            .ok()
            .filter(|out| out.status.success())
            .map(|out| parse_reg_query(&String::from_utf8_lossy(&out.stdout)))
            .unwrap_or_default();
        if let Ok(mut cache) = self.cache.lock() {
            cache.insert(lower, answer.clone());
        }
        answer
    }
}

impl Registry for RegExe {
    fn subkeys(&self, key: &str) -> Vec<String> {
        self.query(key).0
    }

    fn values(&self, key: &str) -> Vec<(String, String)> {
        self.query(key).1
    }
}
