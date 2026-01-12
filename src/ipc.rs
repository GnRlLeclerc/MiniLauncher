//! Communication with the minilauncher daemon

use std::{
    io::{self, Write},
    os::unix::net::UnixStream,
    path::PathBuf,
};

use serde::{Deserialize, Serialize};

use crate::entries::Entry;

pub fn socket_path() -> PathBuf {
    let uid = nix::unistd::getuid();
    PathBuf::from("/run/user")
        .join(uid.to_string())
        .join("minilauncher.sock")
}

/// Launcher IPC actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Action {
    /// Refresh application and icon cache
    Refresh,
    /// Run in app launcher mode
    Run,
    /// Run with custom entries (pass the entries)
    RunCustom(Vec<Entry>),
    /// Quit the daemon
    Quit,
}

/// Send an action to the background daemon.
/// Returns an error if the daemon is not running.
pub fn send_action(action: Action) -> io::Result<()> {
    let path = socket_path();

    match UnixStream::connect(&path) {
        Ok(mut stream) => {
            let bytes = serde_json::to_vec(&action).expect("Failed to serialize action");
            stream
                .write_all(&bytes)
                .expect("Failed to send action to daemon");

            Ok(())
        }
        Err(err) => Err(err),
    }
}
