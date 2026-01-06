use clap::Parser;

use crate::cli::Args;

mod cli;

fn main() {
    let args = Args::parse();

    if args.refresh {
        println!("Refreshing application cache...");
        // Call the function to refresh the cache here
    } else {
        println!("Running app launcher...");
        // Call the function to run the app launcher here
    }
}
