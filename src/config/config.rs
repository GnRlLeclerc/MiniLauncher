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

fn default_animation_duration() -> u64 {
    200
}

fn default_border_width() -> u32 {
    2
}

fn default_border_radius() -> u32 {
    8
}

fn default_mode() -> Mode {
    Mode::Full
}

fn default_max_entries() -> u32 {
    5
}

fn default_font_family() -> Option<String> {
    None
}

fn default_font_size() -> u32 {
    11
}

fn default_opacity() -> f32 {
    1.0
}

/// App config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Animation duration in milliseconds
    #[serde(default = "default_animation_duration")]
    pub animation_duration: u64,
    /// Border thickness in pixels
    #[serde(default = "default_border_width")]
    pub border_width: u32,
    /// Border radius in pixels
    #[serde(default = "default_border_radius")]
    pub border_radius: u32,
    /// Launcher mode
    #[serde(with = "slint_mode", default = "default_mode")]
    pub mode: Mode,
    /// Maximum number of entries to show
    #[serde(default = "default_max_entries")]
    pub max_entries: u32,

    #[serde(default = "default_opacity")]
    pub opacity: f32,

    #[serde(default = "default_font_family")]
    pub font_family: Option<String>,
    #[serde(default = "default_font_size")]
    pub font_size: u32,
}

/// When config file is missing, provide default config
impl Default for Config {
    fn default() -> Self {
        Config {
            animation_duration: default_animation_duration(),
            border_width: default_border_width(),
            border_radius: default_border_radius(),
            mode: default_mode(),
            max_entries: default_max_entries(),
            opacity: default_opacity(),
            font_family: default_font_family(),
            font_size: default_font_size(),
        }
    }
}
