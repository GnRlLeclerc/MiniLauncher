//! Launcher entries

use std::{
    fs,
    io::{self, Read},
    path::{Path, PathBuf},
};

mod freedesktop;

pub use freedesktop::freedesktop_entries;

/// An entry in the app launcher
#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
pub struct Entry {
    /// Entry name
    pub name: String,
    /// Entry command (ran if the entry is selected)
    pub command: String,
    /// Optional entry picture
    pub icon: Option<PathBuf>,
    /// Optional entry description
    pub comment: Option<String>,
    /// Keywords for filtering
    pub keywords: Option<Vec<String>>,
    /// Whether this entry should be executed within a new shell window (TUI)
    #[serde(default)]
    pub terminal: bool,
}

/// Read entries from a file (TOML or JSON)
pub fn read_from_file(path: &Path) -> Vec<Entry> {
    let contents =
        fs::read_to_string(path).expect(&format!("Failed to read entries file {:?}", path));

    if path.extension().and_then(|s| s.to_str()) == Some("toml") {
        toml::from_str(&contents).expect("Failed to parse entries from toml file")
    } else if path.extension().and_then(|s| s.to_str()) == Some("json") {
        serde_json::from_str(&contents).expect("Failed to parse entries from json file")
    } else {
        panic!("Unsupported entries file format: {:?}", path);
    }
}

/// Read JSON entries from stdin.
/// Resilient to colored inputs
pub fn read_from_stdin() -> Vec<Entry> {
    let mut bytes = Vec::new();
    io::stdin()
        .read_to_end(&mut bytes)
        .expect("Failed to read entries from stdin");
    let stripped = strip_ansi_escapes::strip(&bytes);
    serde_json::from_slice(&stripped).expect("Failed to parse entries from stdin")
}
