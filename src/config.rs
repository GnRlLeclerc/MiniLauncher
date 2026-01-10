//! Parse, apply and watch configuration options

mod apply;
mod colors;
mod watch;

pub use watch::{load_config, watch_config};
