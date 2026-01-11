//! Parsing of freedesktop entries for app launching

use linicon_theme::get_icon_theme;
use unidecode::unidecode;

use crate::entries::Entry;

use freedesktop_desktop_entry::{DesktopEntry, Group, Iter, default_paths, get_languages_from_env};
use freedesktop_icons::lookup;

/// Get a localized value from a group given a list of locales
/// Fallsback to the default one if not found
fn get_with_locale<'a>(group: &'a Group, key: &str, locales: &[String]) -> Option<&'a str> {
    let value = group.0.get(key)?;
    for locale in locales {
        if let Some(localized) = value.1.get(&locale[0..2]) {
            return Some(localized.as_str());
        }
    }

    Some(value.0.as_str())
}

/// Process a single freedesktop entry.
/// Returns None if some critical fields are missing,
/// or if the entry should not be displayed
/// Entry keywords are preprocessed to be lowercase and unaccented.
fn process_entry(
    entry: &DesktopEntry,
    locales: &[String],
    icon_theme: &Option<String>,
) -> Option<Entry> {
    let entry = entry.groups.0.get("Desktop Entry")?;

    // Check NoDisplay
    if let Some((value, _)) = entry.0.get("NoDisplay")
        && value == "true"
    {
        return None;
    }

    let name = get_with_locale(entry, "Name", locales)?.to_string();
    let command = entry.0.get("Exec")?.0.clone();
    let icon = entry.0.get("Icon").and_then(|(icon, _)| match icon_theme {
        Some(theme) => lookup(icon).with_theme(theme).find(),
        None => lookup(icon).find(),
    });
    let comment = get_with_locale(entry, "Comment", locales).map(|s| s.to_string());
    let keywords = get_with_locale(entry, "Keywords", locales).map(|s| {
        s.split(';')
            .filter(|s| !s.is_empty())
            .map(|s| unidecode(s).to_lowercase().to_string())
            .collect::<Vec<_>>()
    });
    let terminal = entry.0.get("Terminal").map_or(false, |s| s.0 == "true");

    Some(Entry {
        name,
        command,
        icon,
        comment,
        keywords,
        terminal,
    })
}

/// Parse freedesktop entries
pub fn freedesktop_entries() -> Vec<Entry> {
    let theme = get_icon_theme();
    let locales = get_languages_from_env();

    Iter::new(default_paths())
        .entries(Some(&locales))
        .filter_map(|entry| process_entry(&entry, &locales, &theme))
        .collect()
}
