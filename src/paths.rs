//! Application cache and configuration paths.

use std::path::PathBuf;

const APP_NAME: &str = "minilauncher";

/// Path to the cache file for app entries.
pub fn apps_cache() -> PathBuf {
    dirs_next::cache_dir()
        .expect("Could not determine cache directory")
        .join(APP_NAME)
        .join("apps.rkyv")
}

pub fn config_dir() -> PathBuf {
    dirs_next::config_dir()
        .expect("Could not determine config directory")
        .join(APP_NAME)
}
