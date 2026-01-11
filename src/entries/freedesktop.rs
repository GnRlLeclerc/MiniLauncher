//! Parsing of freedesktop entries for app launching
//! Taken and adapted from https://github.com/FivEawE/desktopentries/blob/master/src/main.rs

use ini::{Ini, Properties};
use log::info;
use std::{
    collections::HashMap,
    env,
    path::{Path, PathBuf},
};

use sys_locale::get_locale;

use crate::entries::Entry;

/// Get freedesktop entries (filtering out no display ones)
pub fn freedesktop_entries() -> Vec<Entry> {
    let mut entries: HashMap<String, Entry> = HashMap::new();
    let parser = FreedesktopParser::new();

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

                match parser.parse(&path) {
                    Some(entry) => {
                        entries.insert(entry.name.clone(), entry);
                    }
                    None => (),
                }
            });
    }
}

struct FreedesktopParser {
    /// Precomputed key for the locale name
    locale_name_key: Option<String>,
    /// Precomputed key for the locale comment
    locale_comment_key: Option<String>,
    /// Precomputed key for the locale keywords
    locale_keywords_key: Option<String>,
    /// Desktop environment (to be checked against "NotShowIn")
    desktop_environment: Option<String>,
}

impl FreedesktopParser {
    pub fn new() -> Self {
        let locale_code = get_locale().and_then(|l| l.split('-').next().map(|s| s.to_string()));
        let locale_name_key = locale_code.as_ref().map(|code| format!("Name[{code}]"));
        let locale_comment_key = locale_code.as_ref().map(|code| format!("Comment[{code}]"));
        let locale_keywords_key = locale_code.as_ref().map(|code| format!("Keywords[{code}]"));
        let desktop_environment = env::var("XDG_CURRENT_DESKTOP")
            .ok()
            .map(|s| s.to_lowercase());
        Self {
            locale_name_key,
            locale_comment_key,
            locale_keywords_key,
            desktop_environment,
        }
    }

    /// Check whether an entry should be shown or not
    fn show(&self, entry: &Properties) -> bool {
        if entry.get("NoDisplay").map_or(false, |s| s == "true") {
            return false;
        }

        if let Some(de) = &self.desktop_environment {
            if entry
                .get("NotShowIn")
                .map_or(false, |s| s.to_lowercase().contains(de))
            {
                return false;
            }
            if entry
                .get("OnlyShowIn")
                .map_or(false, |s| !s.to_lowercase().contains(de))
            {
                return true;
            }
        }

        true
    }

    /// Parse a Freedesktop file and add the entries to the list
    fn parse(&self, path: &Path) -> Option<Entry> {
        let config = Ini::load_from_file(path).ok()?;
        let section = config.section(Some("Desktop Entry"))?;

        if !self.show(section) {
            return None;
        }

        let terminal = section.get("Terminal").map_or(false, |s| s == "true");
        let icon = section.get("Icon").map(|s| s.to_string());
        let command = section.get("Exec")?.to_string();
        let name = self
            .locale_name_key
            .as_ref()
            .and_then(|key| section.get(key))
            .or_else(|| section.get("Name"))?
            .to_string();
        let comment = self
            .locale_comment_key
            .as_ref()
            .and_then(|key| section.get(key))
            .or_else(|| section.get("Comment"))
            .map(|s| s.to_string());
        let keywords = self
            .locale_keywords_key
            .as_ref()
            .and_then(|key| section.get(key))
            .or_else(|| section.get("Keywords"))
            .map(|s| {
                s.split(';')
                    .filter(|s| !s.is_empty())
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>()
            });

        Some(Entry {
            name,
            command,
            icon,
            comment,
            keywords,
            terminal,
        })
    }
}
