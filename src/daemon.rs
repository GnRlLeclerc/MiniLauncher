//! Background daemon process

use log::{error, info, warn};
use slint::{ComponentHandle, Weak, quit_event_loop};
use std::{
    fs,
    io::{self, Read},
    os::unix::net::{UnixListener, UnixStream},
    path::{Path, PathBuf},
    thread,
};

use crate::{ipc::Action, ui::Launcher};

pub fn socket_path() -> PathBuf {
    let uid = nix::unistd::getuid();
    PathBuf::from("/run/user")
        .join(uid.to_string())
        .join("minilauncher.sock")
}

/// Run the background daemon.
/// Returns whether a daemon already exists,
/// in which case the current process should just
/// send a command to it and exit.
pub fn run_daemon(launcher: Weak<Launcher>) -> bool {
    let path = socket_path();

    if check_daemon_running(&path) {
        info!("Daemon is already running, exiting.");
        return true;
    }

    let listener = match UnixListener::bind(path) {
        Ok(listener) => listener,
        Err(e) => {
            error!("Failed to bind to socket: {}", e);
            return false;
        }
    };

    // Run the daemon in a background thread
    thread::spawn(move || {
        for stream in listener.incoming() {
            if let Some(action) = process_stream(stream) {
                match action {
                    Action::Refresh => {
                        info!("Received Refresh action");
                        // TODO: refresh in memory app cache
                    }
                    Action::Run => {
                        info!("Received Run action");
                        // Ignore event loop errors.
                        // If there is no event loop, the launcher is currently exiting.
                        let _ = launcher.upgrade_in_event_loop(|launcher| {
                            if let Err(err) = launcher.show() {
                                error!("Failed to show launcher UI: {}", err);
                            }
                        });
                    }
                    Action::RunCustom(entries) => {
                        info!("Received RunCustom action with {} entries", entries.len());
                        // TODO: swap out the launcher entries, then show the launcher

                        // Ignore event loop errors.
                        // If there is no event loop, the launcher is currently exiting.
                        let _ = launcher.upgrade_in_event_loop(|launcher| {
                            if let Err(err) = launcher.show() {
                                error!("Failed to show launcher UI: {}", err);
                            }
                        });
                    }
                    Action::Quit => {
                        info!("Received Quit action, shutting down the daemon.");
                        let _ = quit_event_loop();
                        break;
                    }
                }
            }
        }
    });

    false
}

/// Process an incoming stream connection.
/// Handles errors and logging internally.
fn process_stream(stream: Result<UnixStream, io::Error>) -> Option<Action> {
    match stream {
        Ok(mut stream) => {
            // Read incoming data and handle commands here
            let mut buffer = Vec::new();
            match stream.read_to_end(&mut buffer) {
                // Deserialize the action
                Ok(size) => {
                    if size == 0 {
                        info!("Received ping (checking if daemon is running)");
                        return None;
                    }

                    match serde_json::from_slice(&buffer) {
                        Ok(action) => Some(action),
                        Err(e) => {
                            warn!("Failed to deserialize action: {}", e);
                            None
                        }
                    }
                }
                Err(e) => {
                    warn!("Error reading from stream: {}", e);
                    return None;
                }
            }
        }
        Err(e) => {
            error!("Error accepting connection: {}", e);
            None
        }
    }
}

/// Check whether the daemon is already running.
/// Cleans up the socket file if it is orphaned.
pub fn check_daemon_running(path: &Path) -> bool {
    match UnixStream::connect(&path) {
        Ok(_) => true,
        Err(_) => {
            if path.exists() {
                let _ = fs::remove_file(path);
            }
            false
        }
    }
}
