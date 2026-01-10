//! Parsing of freedesktop entries for app launching
//! Taken and adapted from https://github.com/FivEawE/desktopentries/blob/master/src/main.rs

use log::{info, warn};
use std::{
    collections::HashMap,
    env,
    io::{self, BufRead},
    path::{Path, PathBuf},
};

use sys_locale::get_locale;

use crate::entries::Entry;

/// Get freedesktop entries (filtering out no display ones)
pub fn freedesktop_entries() -> Vec<Entry> {
    // Get system locale for translations
    let locale = get_locale();
    let de = env::var("XDG_CURRENT_DESKTOP").ok();

    let code: Option<String> = locale
        .map(|l| l.split('-').next().map(|l| l.to_string()))
        .flatten();

    let mut entries: HashMap<String, Entry> = HashMap::new();
    let parser = FreedesktopParser::new(code, de);

    let xdg_data_dirs = env::var("XDG_DATA_DIRS");
    match xdg_data_dirs {
        Ok(value) => {
            for dir in value.split(':') {
                let mut path = PathBuf::from(dir);
                path.push("applications/");
                add_entries_from_path(&path, &mut entries, &parser);
            }
        }
        Err(_) => {
            let base_path = "/usr/share/applications/";
            info!("$XDG_DATA_DIRS not set, defaulting to {}", base_path);
            let path = Path::new(base_path);
            add_entries_from_path(&path, &mut entries, &parser);
        }
    };

    entries.into_values().collect()
}

/// Add all entries from a given path to the entries vector
fn add_entries_from_path(
    path: &Path,
    entries: &mut HashMap<String, Entry>,
    parser: &FreedesktopParser,
) {
    if let Ok(dir_entries) = path.read_dir() {
        dir_entries
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                let path = entry.path();
                path.is_file() && path.extension().map_or(false, |ext| ext == "desktop")
            })
            .for_each(|file| {
                let path = file.path();

                let _ = parser.parse(&path, entries).inspect_err(|_| {
                    warn!("Failed to parse freedesktop file: {:?}", path);
                });
            });
    }
}

struct FreedesktopParser {
    /// Precomputed key for the locale name
    locale_name_key: Option<String>,
    /// Precomputed key for the locale comment
    locale_comment_key: Option<String>,
    /// Desktop environment (to be checked against "NotShowIn")
    desktop_environment: Option<String>,
}

impl FreedesktopParser {
    pub fn new(locale_code: Option<String>, desktop_environment: Option<String>) -> Self {
        let locale_name_key = locale_code.as_ref().map(|code| format!("Name[{code}]"));
        let locale_comment_key = locale_code.as_ref().map(|code| format!("Comment[{code}]"));
        Self {
            locale_name_key,
            locale_comment_key,
            desktop_environment,
        }
    }

    /// Parse a Freedesktop file and add the entries to the list
    pub fn parse(&self, path: &Path, entries: &mut HashMap<String, Entry>) -> io::Result<()> {
        let mut entry = Entry::default();
        let mut valid = false;

        let file = std::fs::File::open(path)?;
        let reader = io::BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            let line = line.trim();

            // Ignore comments
            if line.starts_with('#') {
                continue;
            }

            // Start a new entry if the header is found
            if line == "[Desktop Entry]" {
                let e = entry;
                entry = Entry::default(); // Reset for the new entry
                if valid {
                    entries.insert(e.name.clone(), e);
                }
                valid = true;
                continue;
            } else if line.starts_with("[") {
                let e = entry;
                entry = Entry::default(); // Reset for the new entry
                if valid {
                    entries.insert(e.name.clone(), e);
                }
                // This is not a desktop entry, but an action, etc
                valid = false;
            }

            if !valid {
                continue;
            }

            let mut values = line.split('=');
            let key = values.next();
            let value = values.collect::<Vec<&str>>().join("=");

            let key = match key {
                Some(key) => key,
                None => continue,
            };

            let value = match value.is_empty() {
                false => value,
                true => continue,
            };

            // Compare locale key
            if let Some(name_key) = self.locale_name_key.as_ref() {
                if key == name_key {
                    entry.name = value.to_string();
                }
            }
            // Compare locale comment key
            if let Some(comment_key) = self.locale_comment_key.as_ref() {
                if key == comment_key {
                    entry.description = Some(value.to_string());
                }
            }

            // Match other predefined keys
            match key {
                "Name" => {
                    if entry.name.is_empty() {
                        entry.name = value.to_string();
                    }
                }
                "Exec" => entry.command = value.to_string(),
                "Comment" => {
                    if entry.command.is_empty() {
                        entry.command = value.to_string();
                    }
                }
                "Keywords" => {
                    entry.keywords = Some(
                        value
                            .split(';')
                            .filter(|s| !s.is_empty())
                            .map(|s| s.to_string())
                            .collect(),
                    );
                }
                "Icon" => entry.icon = Some(value.to_string()),
                "NoDisplay" => {
                    if value == "true" {
                        valid = false;
                    }
                }
                "NotShowIn" => {
                    if let Some(de) = self.desktop_environment.as_ref() {
                        if value.to_lowercase().contains(&de.to_lowercase()) {
                            valid = false;
                        }
                    }
                }
                "OnlyShowIn" => {
                    if let Some(de) = self.desktop_environment.as_ref() {
                        if !value.to_lowercase().contains(&de.to_lowercase()) {
                            valid = false;
                        }
                    }
                }
                "Terminal" => {
                    if value == "true" {
                        entry.terminal = true;
                    }
                }
                _ => {}
            }
        }

        // Add the last entry
        if valid {
            entries.insert(entry.name.clone(), entry);
        }

        Ok(())
    }
}
