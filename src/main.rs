// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::error::Error;

use clap::{CommandFactory, Parser};

use crate::{
    cli::Args,
    ipc::{Action, send_action},
};

mod cli;
mod commands;
mod config;
mod daemon;
mod entries;
mod freedesktop;
mod ipc;
mod state;
mod ui;

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    let args = Args::parse();

    if let Some(shell) = args.completions {
        shell.generate(&mut Args::command(), &mut std::io::stdout());
        return Ok(());
    }

    if args.refresh {
        if let Err(_) = send_action(Action::Refresh) {
            eprintln!("Daemon is not running");
        }
        return Ok(());
    } else if args.quit {
        if let Err(_) = send_action(Action::Quit) {
            eprintln!("Daemon is not running");
        }
        return Ok(());
    } else {
        let mut custom = None;

        if let Some(path) = args.entries {
            custom = Some(entries::read_from_file(&path));
        } else if args.stdin {
            custom = Some(entries::read_from_stdin());
        }
        ui::run_ui(custom, !args.no_daemon, args.daemon);
    }

    Ok(())
}
