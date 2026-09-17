use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionDaemonInfo {
    pub session_name: String,
    pub pid: u32,
    pub uptime_seconds: u64,
    pub connected_clients: u32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DaemonStatus {
    pub sessions: Vec<SessionDaemonInfo>,
}
