// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{collections::HashMap, env, error::Error, iter, process::exit, rc::Rc};

use clap::Parser;
use nucleo_matcher::{
    Config, Matcher,
    pattern::{CaseMatching, Normalization, Pattern},
};
use slint::{ModelRc, VecModel};
use unidecode::unidecode;

use crate::{cli::Args, commands::Command};

mod apps;
mod cli;
mod commands;
mod config;
mod entries;
mod freedesktop;
mod paths;

slint::include_modules!();

const MAX_DISPLAYED_ENTRIES: usize = 5;

fn main() -> Result<(), Box<dyn Error>> {
    let mut config = Config::DEFAULT.clone();
    config.prefer_prefix = true;
    let mut matcher = Matcher::new(config);
    let args = Args::parse();

    if args.refresh {
        apps::regenerate_cache();
    } else {
        let mut entry_commands = HashMap::new();
        let mut keyword_to_entry_index = HashMap::new();
        let mut launcher_entries = Vec::new();

        let colors = config::read_colors();

        let mut entries_list = vec![];

        if let Some(path) = args.entries {
            entries_list = entries::read_from_file(&path);
        } else if args.stdin {
            entries_list = entries::read_from_stdin();
        } else {
            entries_list = apps::get_app_entries();
        }

        entries_list.into_iter().for_each(|e| {
            let keywords = iter::once(e.name.as_str())
                .chain(e.keywords.unwrap_or_default().iter().map(String::as_str))
                .collect::<Vec<_>>()
                .join(" ")
                .to_lowercase();
            let keywords = unidecode(&keywords);

            // Insert the command
            entry_commands.insert(
                e.name.clone(),
                Command {
                    command: e.command,
                    terminal: e.terminal,
                },
            );

            // Add the display entry
            launcher_entries.push(LauncherEntry {
                name: e.name.into(),
                icon: e.icon.unwrap_or_default().into(),
                description: e.description.unwrap_or_default().into(),
            });

            // Map keywords to entry index
            keyword_to_entry_index.insert(keywords, launcher_entries.len() - 1);
        });

        let initial_entries = launcher_entries
            .iter()
            .take(MAX_DISPLAYED_ENTRIES)
            .cloned()
            .collect::<Vec<_>>();

        let ui = Launcher::new()?;
        let ui_handle = ui.as_weak();

        // Set initial states
        ui.global::<LauncherState>()
            .set_entries(ModelRc::from(Rc::new(VecModel::from(initial_entries))));

        if let Some(colors) = colors {
            ui.global::<LauncherColors>()
                .set_background(colors.background);
            ui.global::<LauncherColors>().set_text(colors.text);
            ui.global::<LauncherColors>().set_border(colors.border);
            ui.global::<LauncherColors>().set_accent(colors.accent);
            ui.global::<LauncherColors>()
                .set_highlight(colors.highlight);
        }

        ui.global::<LauncherState>().on_run_command(move |name| {
            if let Some(cmd) = entry_commands.get(name.as_str()) {
                commands::run(cmd);

                // Close the launcher after running the command
                exit(0);
            }
        });

        ui.global::<LauncherState>()
            .on_update_entries(move |input| {
                // Match normalized entries with the input by comparing keywords
                // Two steps:
                // 1. Filter by exact substring match
                // 2. Fuzzy filtering to order the remaining matches

                // Normalize the input
                let input = unidecode(&input.to_lowercase());
                let substrings = input.split_whitespace().collect::<Vec<_>>();

                // 1. Exact substring matching
                let matches = keyword_to_entry_index
                    .keys()
                    .filter(|keywords| substrings.iter().all(|s| keywords.contains(s)))
                    .collect::<Vec<_>>();

                // 2. Fuzzy matching to order the matches
                let mut matches =
                    Pattern::parse(&input, CaseMatching::Ignore, Normalization::Smart)
                        .match_list(matches, &mut matcher);

                matches.sort_by(|a, b| b.1.cmp(&a.1)); // Sort by descending score

                // Build and sort an entry model from the matches
                let entries = matches
                    .into_iter()
                    .take(MAX_DISPLAYED_ENTRIES)
                    .map(|(keywords, _)| {
                        let index = keyword_to_entry_index[keywords];
                        let entry = &launcher_entries[index];
                        // NOTE: cloning an entry = cloning pointers to SharedStrings,
                        // not cloning Strings themselves
                        entry.clone()
                    })
                    .collect::<Vec<_>>();

                ui_handle
                    .unwrap()
                    .global::<LauncherState>()
                    .set_entries(ModelRc::from(Rc::new(VecModel::from(entries))));
            });
        ui.run()?;
    }

    Ok(())
}
