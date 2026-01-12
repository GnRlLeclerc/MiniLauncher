//! Launcher configuration

use serde::{Deserialize, Serialize};

use crate::ui::Mode;

/// Serialize and deserialize the slint Mode enum using serde
mod slint_mode {
    use crate::ui::Mode;
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Mode, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "full" => Ok(Mode::Full),
            "compact" => Ok(Mode::Compact),
            "lines" => Ok(Mode::Lines),
            "icons" => Ok(Mode::Icons),
            _ => Err(serde::de::Error::custom("unknown mode")),
        }
    }

    pub fn serialize<S>(mode: &Mode, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = match mode {
            Mode::Full => "full",
            Mode::Compact => "compact",
            Mode::Lines => "lines",
            Mode::Icons => "icons",
        };
        serializer.serialize_str(s)
    }
}

fn default_animation() -> u64 {
    200
}

fn default_border() -> u32 {
    2
}

fn default_radius() -> u32 {
    8
}

fn default_mode() -> Mode {
    Mode::Full
}

fn default_max_entries() -> u32 {
    5
}

/// App config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Animation duration in milliseconds
    #[serde(default = "default_animation")]
    pub animation: u64,
    /// Border thickness in pixels
    #[serde(default = "default_border")]
    pub border: u32,
    /// Border radius in pixels
    #[serde(default = "default_radius")]
    pub radius: u32,
    /// Launcher mode
    #[serde(with = "slint_mode", default = "default_mode")]
    pub mode: Mode,
    /// Maximum number of entries to show
    #[serde(default = "default_max_entries")]
    pub max_entries: u32,
}

/// When config file is missing, provide default config
impl Default for Config {
    fn default() -> Self {
        Config {
            animation: default_animation(),
            border: default_border(),
            radius: default_radius(),
            mode: default_mode(),
            max_entries: default_max_entries(),
        }
    }
}
