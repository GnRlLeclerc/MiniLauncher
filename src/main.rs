// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::error::Error;

use clap::Parser;
use slint::ComponentHandle;

use crate::{
    callbacks::register_callbacks,
    cli::Args,
    config::{load_config, watch_config},
    daemon::run_daemon,
};

mod apps;
mod callbacks;
mod cli;
mod commands;
mod config;
mod daemon;
mod entries;
mod freedesktop;
mod ipc;
mod paths;

/// Slint UI modules
mod ui {
    slint::include_modules!();
}

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    let args = Args::parse();

    if args.refresh {
        apps::regenerate_cache();
    } else {
        let mut entries = vec![];

        if let Some(path) = args.entries {
            entries = entries::read_from_file(&path);
        } else if args.stdin {
            entries = entries::read_from_stdin();
        } else {
            entries = apps::get_app_entries();
        }

        let ui = ui::Launcher::new()?;
        load_config(&ui).await;
        register_callbacks(entries, &ui);

        let colors = config::read_colors();
        if let Some(colors) = colors {
            apply_colors(&ui, &colors);
        }

        ui.run()?;
    }

    Ok(())
}
