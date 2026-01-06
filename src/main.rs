// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::error::Error;

use clap::Parser;

use crate::cli::Args;

mod cli;

slint::include_modules!();

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    if args.refresh {
        println!("Refreshing application cache...");
        // Call the function to refresh the cache here
    } else {
        let ui = Launcher::new()?;
        ui.run()?;
    }

    Ok(())
}
