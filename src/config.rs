//! Configuration options read from config files

use serde::Deserialize;
use slint::Color;

/// Deserialize slint::Color from hex color strings
mod slint_hex_color {
    use serde::Deserializer;
    use slint::Color;

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Color, D::Error>
    where
        D: Deserializer<'de>,
    {
        let color = hex_color::rgb::deserialize(deserializer)?;
        Ok(Color::from_rgb_u8(color.r, color.g, color.b))
    }
}

/// App color scheme
#[derive(Debug, Clone, Deserialize)]
pub struct Colors {
    #[serde(with = "slint_hex_color")]
    pub background: Color,
    #[serde(with = "slint_hex_color")]
    pub text: Color,
    #[serde(with = "slint_hex_color")]
    pub border: Color,
    #[serde(with = "slint_hex_color")]
    pub accent: Color,
    #[serde(with = "slint_hex_color")]
    pub highlight: Color,
}

/// Read the colors from the config file
pub fn read_colors() -> Option<Colors> {
    let colors_file = crate::paths::colors_file();
    if colors_file.exists() {
        let contents = std::fs::read_to_string(colors_file).ok()?;
        match toml::from_str(&contents) {
            Ok(colors) => Some(colors),
            Err(err) => {
                eprintln!("Failed to parse colors config: {}", err);
                None
            }
        }
    } else {
        None
    }
}
