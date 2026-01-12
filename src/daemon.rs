//! Background daemon process

use log::{error, info, warn};
use slint::{ComponentHandle, Weak, invoke_from_event_loop, quit_event_loop};
use std::{
    fs,
    io::{self, Read},
    os::unix::net::{UnixListener, UnixStream},
    path::Path,
    thread,
};

use crate::{
    entries::Entry,
    freedesktop::freedesktop_entries,
    ipc::{Action, socket_path},
    state::invoke_with_appstate,
    ui::Launcher,
};

/// Set the launcher entries for a new run, and then show the launcher UI.
/// If an error occurs, silently fails without crashing the daemon.
fn run_launcher_with_custom(launcher: &Weak<Launcher>, custom: Option<Vec<Entry>>) {
    // Ignore event loop errors.
    // If there is no event loop, the launcher is currently exiting.
    let _ = launcher.upgrade_in_event_loop(|launcher| {
        // Do not process actions while the launcher is already visible
        if launcher.window().is_visible() {
            info!("Launcher is already visible, ignoring Run action");
            return;
        }
        invoke_with_appstate(|state| {
            // Set custom entries
            if let Err(_) = state.set_custom_entries(custom) {
                error!("Failed to set custom entries: state custom entries were borrowed");
                return;
            }
            // Reset launcher entries
            if let Err(_) = state.set_launcher_entries(&launcher) {
                error!("Failed to set launcher entries: state launcher entries were borrowed");
                return;
            }
            if let Err(err) = launcher.show() {
                error!("Failed to show launcher UI: {}", err);
                return;
            }
        });
    });
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
                        let apps = freedesktop_entries();
                        let _ = invoke_from_event_loop(move || {
                            if let Err(_) = invoke_with_appstate(|state| state.set_apps(apps)) {
                                warn!("Failed to refresh entries: state apps were borrowed");
                            }
                        });
                    }
                    Action::Run => {
                        info!("Received Run action");
                        run_launcher_with_custom(&launcher, None);
                    }
                    Action::RunCustom(entries) => {
                        info!("Received RunCustom action with {} entries", entries.len());
                        run_launcher_with_custom(&launcher, Some(entries));
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
                    None
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
