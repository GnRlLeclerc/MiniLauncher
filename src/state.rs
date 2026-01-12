//! Global app state
//! This state live in the UI thread only.
//!
//! While the state provided by Slint globals is used for direct display,
//! this state is used for more complex processing,
//! given that it can use the full rust types instead of being limited to Slint types.
//! It involves fast and frequent frontend operations (entry filtering via input text)
//! which is why it is kept in the UI thread in a lock-free way.
//!
//! https://mmapped.blog/posts/01-effective-rust-canisters#canister-state
//!
//! Because everything is in refcells and the daemon must not panic,
//! we check for borrow errors and silently ignore them when incompatible borrows collide.

use std::{
    cell::{BorrowError, BorrowMutError, Cell, RefCell},
    collections::HashMap,
    error::Error,
    iter,
    rc::Rc,
};

use nucleo_matcher::{
    Config, Matcher,
    pattern::{CaseMatching, Normalization, Pattern},
};
use slint::{ComponentHandle, Image, ModelRc, VecModel};
use unidecode::unidecode;

use crate::{
    commands::Command,
    entries::Entry,
    ui::{Launcher, LauncherConfig, LauncherEntry, LauncherState},
};

thread_local! {
    /// Marker to know whether the current thread is the UI thread.
    static IS_UI_THREAD: Cell<bool> = Cell::new(false);
    /// Private global app state. Although a copy of this state exists in each thread,
    /// only the UI thread can access its own, making it effectively a singleton.
    static APP_STATE: AppState = AppState::new();
}

/// Invoke a callback with access to the global app state.
/// This function can only be used from the UI thread, else it panics.
/// Call `invoke_from_event_loop` or `launcher.upgrade_in_event_loop`
/// to access the UI thread, then call this function.
pub fn invoke_with_appstate<F, T>(callback: F) -> T
where
    F: FnOnce(&AppState) -> T,
{
    if !IS_UI_THREAD.get() {
        panic!("Accessing app state from non-UI thread");
    }
    APP_STATE.with(|state| callback(state))
}

/// Mark the current thread as the UI thread.
/// The Launcher reference in the argument ensures that this function
/// is only called from the UI thread (safeguard).
pub fn set_ui_thread(_: &Launcher) {
    IS_UI_THREAD.set(true);
}

/// Preprocessed entries for easy use in the app launcher UI
#[derive(Debug, Default)]
pub struct ProcessedEntries {
    /// From entry name to entry command
    pub commands_by_name: HashMap<String, Command>,
    /// Launcher entries for display, indexed by their keywords
    /// (needed because nucleo fuzzy matcher outputs the string used for matching,
    /// so we need to map it back to the actual entry)
    pub entries_by_keywords: HashMap<String, LauncherEntry>,
}

impl ProcessedEntries {
    /// Process entries for display and filtering
    pub fn new(entries: Vec<Entry>) -> Self {
        let mut entry_commands = HashMap::new();
        let mut entries_by_keywords = HashMap::new();

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
            entries_by_keywords.insert(
                keywords,
                LauncherEntry {
                    name: e.name.into(),
                    icon: e
                        .icon
                        .and_then(|path| Image::load_from_path(&path).ok())
                        .unwrap_or_default(),
                    comment: e.comment.unwrap_or_default().into(),
                },
            );
        });

        Self {
            commands_by_name: entry_commands,
            entries_by_keywords,
        }
    }
}

/// Global application state
#[derive(Debug, Default)]
pub struct AppState {
    /// Desktop applications
    apps: RefCell<ProcessedEntries>,
    /// Custom entries.
    /// When not None, the launcher is in "custom entries" mode
    custom: RefCell<Option<ProcessedEntries>>,
    /// Fuzzy matcher (in a refcell because it needs to be mutable
    /// during matching)
    matcher: RefCell<Matcher>,
}

impl AppState {
    pub fn new() -> Self {
        let mut config = Config::DEFAULT.clone();
        config.prefer_prefix = true;
        let matcher = Matcher::new(config);
        Self {
            apps: RefCell::new(ProcessedEntries::default()),
            custom: RefCell::new(None),
            matcher: RefCell::new(matcher),
        }
    }
    /// Refresh apps
    /// Might fail if the apps are being borrowed during a text search
    pub fn set_apps(&self, entries: Vec<Entry>) -> Result<(), BorrowMutError> {
        let processed = ProcessedEntries::new(entries);
        let mut apps = self.apps.try_borrow_mut()?;
        *apps = processed;
        Ok(())
    }

    /// Set custom entries
    /// Might fail if the custom entries are being borrowed during a text search
    pub fn set_custom_entries(&self, entries: Option<Vec<Entry>>) -> Result<(), BorrowMutError> {
        let processed = entries.map(|entries| ProcessedEntries::new(entries));
        let mut custom = self.custom.try_borrow_mut()?;
        *custom = processed;
        Ok(())
    }

    /// Search entries given a query.
    /// Set them back into the launcher.
    /// Might fail if the matcher or entries are being borrowed elsewhere.
    pub fn search_entries(&self, launcher: &Launcher, query: &str) -> Result<(), Box<dyn Error>> {
        let custom = self.custom.try_borrow()?;
        let entries = match custom.as_ref() {
            Some(custom) => custom,
            None => &self.apps.try_borrow()?,
        };
        let limit = launcher.global::<LauncherConfig>().get_max_entries() as usize;

        // Match normalized entries with the input by comparing keywords
        // Two steps:
        // 1. Filter by exact substring match
        // 2. Fuzzy filtering to order the remaining matches

        // Normalize the input
        let input = unidecode(&query.to_lowercase());
        let substrings = input.split_whitespace().collect::<Vec<_>>();

        // 1. Exact substring matching
        let matches = entries
            .entries_by_keywords
            .keys()
            .filter(|keywords| substrings.iter().all(|s| keywords.contains(s)))
            .collect::<Vec<_>>();

        // 2. Fuzzy matching to order the matches
        // There might be a possibility that the matcher is already borrowed
        // (high amount of entries, no debouncing of text input)
        // which is why we handle the error case by silently doing nothing.

        let mut matcher = self.matcher.try_borrow_mut()?;
        let mut matches = Pattern::parse(&input, CaseMatching::Ignore, Normalization::Smart)
            .match_list(matches, &mut matcher);

        matches.sort_by(|a, b| b.1.cmp(&a.1)); // Sort by descending score

        // Build and sort an entry model from the matches
        let entries = matches
            .into_iter()
            .take(limit)
            .map(|(keywords, _)| entries.entries_by_keywords[keywords].clone());

        set_launcher_entries(launcher, entries);

        Ok(())
    }

    /// Set the launcher initial entries
    /// Used on initial startup and subsequent wakes
    /// to reset the default shown entries
    /// Might fail if the entries are being replaced at exactly the same time
    pub fn set_launcher_entries(&self, launcher: &Launcher) -> Result<(), BorrowError> {
        let custom = self.custom.try_borrow()?;
        let entries = match custom.as_ref() {
            Some(custom) => custom,
            None => &self.apps.try_borrow()?,
        };
        let limit = launcher.global::<LauncherConfig>().get_max_entries() as usize;

        set_launcher_entries(
            launcher,
            entries.entries_by_keywords.values().take(limit).cloned(),
        );

        Ok(())
    }

    /// Get a command by its name
    pub fn get_command(&self, name: &str) -> Option<Command> {
        let custom = self.custom.try_borrow().ok()?;
        let entries = match custom.as_ref() {
            Some(custom) => custom,
            None => &self.apps.try_borrow().ok()?,
        };

        entries.commands_by_name.get(name).cloned()
    }
}

fn set_launcher_entries(launcher: &Launcher, entries: impl Iterator<Item = LauncherEntry>) {
    launcher
        .global::<LauncherState>()
        .set_entries(ModelRc::from(Rc::new(VecModel::from(
            entries.collect::<Vec<_>>(),
        ))));
}
