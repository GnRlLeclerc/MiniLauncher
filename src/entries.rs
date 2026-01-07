//! Launcher entries

/// An entry in the app launcher
#[derive(Debug, Clone, Default, rkyv::Serialize, rkyv::Deserialize, rkyv::Archive)]
pub struct Entry {
    /// Entry name
    pub name: String,
    /// Entry command (ran if the entry is selected)
    pub command: String,
    /// Optional entry picture
    pub icon: Option<String>,
    /// Optional entry description
    pub description: Option<String>,
    /// Keywords for filtering
    pub keywords: Option<Vec<String>>,
    /// Whether this entry should be executed within a new shell window (TUI)
    pub terminal: bool,
}
