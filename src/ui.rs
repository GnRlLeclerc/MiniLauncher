//! App callbacks

use log::error;
use nucleo_matcher::pattern::{CaseMatching, Normalization, Pattern};
use nucleo_matcher::{Config, Matcher};
use slint::{ModelRc, VecModel, run_event_loop, run_event_loop_until_quit};
use unidecode::unidecode;

use crate::commands::{self, Command};
use crate::config::{load_config, watch_config};
use crate::daemon::run_daemon;
use crate::entries::Entry;
use crate::ipc::{Action, send_action};
use std::collections::HashMap;
use std::iter;
use std::rc::Rc;

const MAX_DISPLAYED_ENTRIES: usize = 5;

// Include Slint codegen components here
slint::include_modules!();

/// Set up and run the launcher UI
/// - `daemon`: whether to start the daemon in the background
/// - `hidden`: whether to start the UI hidden (daemon only)
pub fn run_ui(entries: Vec<Entry>, daemon: bool, hidden: bool) {
    let launcher = Launcher::new().expect("Failed to create launcher UI");

    register_callbacks(entries, &launcher);
    load_config(&launcher); // Load initial configuration

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

    if daemon {
        if run_daemon(launcher.as_weak()) {
            // A daemon is already running, send a command and exit
            send_action(Action::Run).expect("Failed to send Run action to daemon");
            return;
        }
    }

    // Run the ui event loop
    match daemon {
        true => run_event_loop_until_quit(),
        false => run_event_loop(),
    }
    .expect("Failed to run event loop");
}

/// Register app callbacks
pub fn register_callbacks(entries: Vec<Entry>, launcher: &Launcher) {
    let mut entry_commands = HashMap::new();
    let mut keyword_to_entry_index = HashMap::new();
    let mut launcher_entries = Vec::new();

    // Process entries into convenient structures
    entries.into_iter().for_each(|e| {
        let keywords = iter::once(e.name.as_str())
            .chain(e.keywords.unwrap_or_default().iter().map(String::as_str))
            .collect::<Vec<_>>()
            .join(" ")
            .to_lowercase();
        let keywords = unidecode(&keywords);

        // Insert the command
        entry_commands.insert(
            e.name.clone(),
            Command {
                command: e.command,
                terminal: e.terminal,
            },
        );

        // Add the display entry
        launcher_entries.push(LauncherEntry {
            name: e.name.into(),
            icon: e.icon.unwrap_or_default().into(),
            description: e.description.unwrap_or_default().into(),
        });

        // Map keywords to entry index
        keyword_to_entry_index.insert(keywords, launcher_entries.len() - 1);

        // Set initial entries
        let initial_entries = launcher_entries
            .iter()
            .take(MAX_DISPLAYED_ENTRIES)
            .cloned()
            .collect::<Vec<_>>();
        launcher
            .global::<LauncherState>()
            .set_entries(ModelRc::from(Rc::new(VecModel::from(initial_entries))));
    });

    // Register command running callback
    let handle = launcher.as_weak();
    launcher
        .global::<LauncherState>()
        .on_run_command(move |name| {
            if let Some(cmd) = entry_commands.get(name.as_str()) {
                commands::run(cmd);
                let _ = handle.unwrap().hide();
            }
        });

    let mut config = Config::DEFAULT.clone();
    config.prefer_prefix = true;
    let mut matcher = Matcher::new(config);

    // Register entry processing callback
    let handle = launcher.as_weak();
    launcher
        .global::<LauncherState>()
        .on_update_entries(move |input| {
            // Match normalized entries with the input by comparing keywords
            // Two steps:
            // 1. Filter by exact substring match
            // 2. Fuzzy filtering to order the remaining matches

            // Normalize the input
            let input = unidecode(&input.to_lowercase());
            let substrings = input.split_whitespace().collect::<Vec<_>>();

            // 1. Exact substring matching
            let matches = keyword_to_entry_index
                .keys()
                .filter(|keywords| substrings.iter().all(|s| keywords.contains(s)))
                .collect::<Vec<_>>();

            // 2. Fuzzy matching to order the matches
            let mut matches = Pattern::parse(&input, CaseMatching::Ignore, Normalization::Smart)
                .match_list(matches, &mut matcher);

            matches.sort_by(|a, b| b.1.cmp(&a.1)); // Sort by descending score

            // Build and sort an entry model from the matches
            let entries = matches
                .into_iter()
                .take(MAX_DISPLAYED_ENTRIES)
                .map(|(keywords, _)| {
                    let index = keyword_to_entry_index[keywords];
                    let entry = &launcher_entries[index];
                    // NOTE: cloning an entry = cloning pointers to SharedStrings,
                    // not cloning Strings themselves
                    entry.clone()
                })
                .collect::<Vec<_>>();

            handle
                .unwrap()
                .global::<LauncherState>()
                .set_entries(ModelRc::from(Rc::new(VecModel::from(entries))));
        });

    let handle = launcher.as_weak();
    launcher.global::<LauncherState>().on_hide(move || {
        let _ = handle.unwrap().hide();
    });
}
