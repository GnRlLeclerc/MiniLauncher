//! Communication with the minilauncher daemon

use serde::{Deserialize, Serialize};

use crate::entries::Entry;

/// Launcher IPC actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Action {
    /// Refresh application and icon cache
    Refresh,
    /// Run in app launcher mode
    Run,
    /// Run with custom entries (pass the entries)
    RunCustom(Vec<Entry>),
}

/// Possible responses from the daemon
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Response {
    /// Acknowledgement of action received
    Ack,
    /// Error occurred
    Error(String),
}
