// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::error::Error;

use clap::Parser;

use crate::{
    cli::Args,
    ipc::{Action, send_action},
};

mod cli;
mod commands;
mod config;
mod daemon;
mod entries;
mod ipc;
mod paths;
mod ui;

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    let args = Args::parse();

    if args.refresh {
        send_action(Action::Refresh).expect("Daemon is not running");
        return Ok(());
    } else if args.quit {
        send_action(Action::Quit).expect("Daemon is not running");
        return Ok(());
    } else {
        let mut entries = vec![];

        if let Some(path) = args.entries {
            entries = entries::read_from_file(&path);
        } else if args.stdin {
            entries = entries::read_from_stdin();
        } else {
            entries = entries::freedesktop_entries();
        }

        ui::run_ui(entries, !args.no_daemon, args.daemon);
    }

    Ok(())
}
