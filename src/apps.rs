//! Parse freedesktop entries

use std::{fs, path::Path};

use rkyv::{self, rancor::Error, vec::ArchivedVec};

use crate::{
    entries::{ArchivedEntry, Entry},
    freedesktop::freedesktop_entries,
    paths,
};

/// Get app entries either from cache or regenerate them from scratch.
/// Used when running the launcher in app launcher mode.
pub fn get_app_entries() -> Vec<Entry> {
    let path = paths::apps_cache();

    if let Some(entries) = read_cache(&path) {
        return entries;
    }

    println!("Regenerating app cache...");
    let entries = freedesktop_entries().into_values().collect();
    write_cache(&path, &entries);

    entries
}

/// Regenerate app entries cache from scratch.
/// Used when running the launcher with --refresh
pub fn regenerate_cache() {
    let path = paths::apps_cache();
    println!("Regenerating app cache...");
    let entries = freedesktop_entries().into_values().collect();
    write_cache(&path, &entries);
}

/// Read app entries from cache.
/// Returns None if anything fails, in which event
/// the launcher will regenerate the cache from scratch.
fn read_cache(path: &Path) -> Option<Vec<Entry>> {
    if !path.exists() {
        return None;
    }

    if let Ok(bytes) = fs::read(path)
        && let Ok(archive) = rkyv::access::<ArchivedVec<ArchivedEntry>, Error>(&bytes)
        && let Ok(entries) = rkyv::deserialize::<Vec<Entry>, Error>(archive)
    {
        Some(entries)
    } else {
        None
    }
}

/// Write app entries to cache.
fn write_cache(path: &Path, entries: &Vec<Entry>) {
    let bytes = rkyv::to_bytes::<Error>(entries).expect("Failed to serialize app entries");

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("Failed to create cache directory");
    }
    fs::write(path, bytes).expect("Failed to write app entries to cache");
}
