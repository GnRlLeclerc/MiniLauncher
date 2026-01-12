//! App callbacks

use std::fs;

use log::error;
use slint::{quit_event_loop, run_event_loop, run_event_loop_until_quit};

use crate::commands::{self};
use crate::config::{load_config_and_colors, watch_config};
use crate::daemon::run_daemon;
use crate::entries::Entry;
use crate::freedesktop::freedesktop_entries;
use crate::ipc::{Action, send_action, socket_path};
use crate::state::{invoke_with_appstate, set_ui_thread};

// Include Slint codegen components here
slint::include_modules!();

/// Set up and run the launcher UI
/// - `custom`: optional custom entries for the first run
/// - `daemon`: whether to start the daemon in the background
/// - `hidden`: whether to start the UI hidden (daemon only)
pub fn run_ui(custom: Option<Vec<Entry>>, daemon: bool, hidden: bool) {
    let launcher = Launcher::new().expect("Failed to create launcher UI");

    // Check straight away if a daemon is running,
    // so that go straight to sending actions to the socket if possible.
    if daemon {
        if run_daemon(launcher.as_weak()) {
            // A daemon is already running, send a command and exit
            match custom {
                Some(entries) => {
                    send_action(Action::RunCustom(entries))
                        .expect("Failed to send RunCustom action to daemon");
                }
                None => send_action(Action::Run).expect("Failed to send Run action to daemon"),
            };
            return;
        }
    }

    // Properly handle termination signals
    // Quitting the event loop this way means that the run_ui function will complete,
    // deleting the daemon socket file as well, properly cleaning up everything.
    if let Err(err) = ctrlc::set_handler(|| {
        let _ = quit_event_loop();
    }) {
        error!("Could not set termination signal handler: {}", err);
    }

    set_ui_thread(&launcher); // Mark this thread as the UI thread for the global app state
    register_callbacks(&launcher);
    load_config_and_colors(&launcher); // Load initial configuration

    // Initialize the data.
    // Always load the apps (even in --no-daemon mode, which is for debug)
    let apps = freedesktop_entries();
    invoke_with_appstate(|state| {
        state
            .set_apps(apps)
            .expect("Failed to set default apps in the app state");
        state
            .set_custom_entries(custom)
            .expect("Failed to set custom entries in the app state");

        state
            .set_launcher_entries(&launcher)
            .expect("Failed to set launcher entries in the app state");
    });

    // Store the watcher as a guard to keep it alive.
    let _watcher = watch_config(&launcher)
        .inspect_err(|err| error!("Could not start configuration file watcher: {}", err));

    // BUG: https://github.com/slint-ui/slint/issues/10341
    if !hidden {
        launcher.show().expect("Failed to show launcher UI");
    } else {
        // NOTE: lazy workaround, for now start the launcher even in daemon mode
        launcher.show().expect("Failed to hide launcher UI");
    }

    // Run the ui event loop
    match daemon {
        true => run_event_loop_until_quit(),
        false => run_event_loop(),
    }
    .expect("Failed to run event loop");

    if daemon {
        // Cleanup the socket on exit
        let _ = fs::remove_file(socket_path());
    }
}

/// Register app callbacks
pub fn register_callbacks(launcher: &Launcher) {
    // Register command running callback
    let handle = launcher.as_weak();
    launcher
        .global::<LauncherState>()
        .on_run_command(move |name| {
            if let Some(cmd) = invoke_with_appstate(|state| state.get_command(name.as_str())) {
                commands::run(&cmd);
                let _ = handle.unwrap().hide();
            }
        });

    // Register entry processing callback
    let handle = launcher.as_weak();
    launcher
        .global::<LauncherState>()
        .on_update_entries(move |input| {
            invoke_with_appstate(|state| {
                let _ = state.search_entries(&handle.unwrap(), &input);
            });
        });

    let handle = launcher.as_weak();
    launcher.global::<LauncherState>().on_hide(move || {
        let _ = handle.unwrap().hide();
    });
}
