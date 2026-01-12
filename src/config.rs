//! Parse, apply and watch configuration options

mod apply;
mod colors;
mod config;
mod watch;

pub use watch::{load_config_and_colors, watch_config};
