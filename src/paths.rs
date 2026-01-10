//! Application cache and configuration paths.
//! No need to cache these values, they are cheap to compute
//! and called a negligible amount of times anyway.

use std::path::PathBuf;

const APP_NAME: &str = "minilauncher";

pub fn config_dir() -> PathBuf {
    dirs_next::config_dir()
        .expect("Could not determine config directory")
        .join(APP_NAME)
}

pub fn colors_file() -> PathBuf {
    config_dir().join("colors.toml")
}
