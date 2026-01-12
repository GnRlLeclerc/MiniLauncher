//! App configuration loading and watching

use std::{fs, path::PathBuf, time::Duration};

use log::{debug, trace, warn};
use notify::RecursiveMode;
use notify_debouncer_full::{
    DebouncedEvent, Debouncer, RecommendedCache, new_debouncer,
    notify::{self, EventKind, RecommendedWatcher},
};
use slint::ComponentHandle;

use super::{apply::apply_colors, colors::Colors};
use crate::ui::Launcher;

pub fn config_dir() -> PathBuf {
    dirs_next::config_dir()
        .expect("Could not determine config directory")
        .join("minilauncher")
}

pub fn colors_file() -> PathBuf {
    config_dir().join("colors.toml")
}

/// Load and apply configuration from files.
/// If no config is found, apply the default one.
/// Only done at startup.
pub fn load_config(launcher: &Launcher) {
    let colors = load_colors().unwrap_or_default();
    apply_colors(launcher, &colors);
}

/// Load colors configuration from file.
/// Returns None on error.
fn load_colors() -> Option<Colors> {
    let colors_file = colors_file();

    let contents = fs::read_to_string(colors_file)
        .inspect_err(|err| warn!("Could not read colors configuration file: {}", err))
        .ok()?;

    let colors: Colors = toml::from_str(&contents)
        .inspect_err(|err| warn!("Could not parse colors configuration file: {}", err))
        .ok()?;

    Some(colors)
}

/// Watch and reload configuration files on changes.
/// Returns errors on startup only (not fatal, but config won't be watched).
/// Store the returned watcher to keep it alive.
pub fn watch_config(
    launcher: &Launcher,
) -> notify::Result<Debouncer<RecommendedWatcher, RecommendedCache>> {
    let launcher = launcher.as_weak();

    let mut debouncer = new_debouncer(
        Duration::from_millis(100),
        None,
        move |events: Result<Vec<DebouncedEvent>, Vec<notify::Error>>| {
            trace!("Received file events");
            match events {
                Ok(events) => {
                    let mut colors_changed = false;
                    for event in events {
                        // Ignore non-modifying events
                        match event.kind {
                            EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) => {}
                            _ => continue,
                        }

                        // Check whether config files changed
                        for path in event.paths.iter() {
                            if let Some(filename) = path.file_name() {
                                let filename = filename.to_string_lossy();
                                match &filename as &str {
                                    "colors.toml" => {
                                        colors_changed = true;
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }

                    if colors_changed {
                        debug!("Colors configuration changed, reloading");
                        if let Some(colors) = load_colors() {
                            if let Err(err) = launcher.upgrade_in_event_loop(move |launcher| {
                                apply_colors(&launcher, &colors);
                            }) {
                                warn!("Failed to apply new color configuration: {}", err);
                            }
                        }
                    }
                }
                Err(err) => {
                    warn!("Watch debouncer errors: {:?}", err);
                }
            }
        },
    )?;

    debug!("Watching config directory: {:?}", config_dir());
    debouncer.watch(&config_dir(), RecursiveMode::NonRecursive)?;

    Ok(debouncer)
}
