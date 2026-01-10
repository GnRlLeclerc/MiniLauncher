//! App color scheme configuration

use serde::{Deserialize, Serialize};
use slint::Color;

/// Serialize and deserialize slint::Color from hex color strings
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

    pub fn serialize<S>(color: &Color, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let color = hex_color::HexColor {
            r: color.red(),
            g: color.green(),
            b: color.blue(),
            a: 255,
        };
        hex_color::rgb::serialize(&color, serializer)
    }
}

macro_rules! default_color {
    ($name:ident, $r:expr, $g:expr, $b:expr) => {
        paste::paste! {
            const [<DEFAULT_ $name:upper>]: Color =
                Color::from_rgb_u8($r, $g, $b);

            fn [<default_ $name>]() -> Color {
                [<DEFAULT_ $name:upper>]
            }
        }
    };
}

// Default colors (from Catppuccin Mocha)
default_color!(background, 30, 30, 46); // base
default_color!(text, 205, 214, 244); // text
default_color!(border, 69, 71, 90); // surface1
default_color!(accent, 203, 166, 247); // mauve
default_color!(highlight, 147, 153, 178); // surface2

/// App color scheme
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Colors {
    #[serde(with = "slint_hex_color", default = "default_background")]
    pub background: Color,
    #[serde(with = "slint_hex_color", default = "default_text")]
    pub text: Color,
    #[serde(with = "slint_hex_color", default = "default_border")]
    pub border: Color,
    #[serde(with = "slint_hex_color", default = "default_accent")]
    pub accent: Color,
    #[serde(with = "slint_hex_color", default = "default_highlight")]
    pub highlight: Color,
}

/// When color config file is missing, provide default colors
impl Default for Colors {
    fn default() -> Self {
        Colors {
            background: default_background(),
            text: default_text(),
            border: default_border(),
            accent: default_accent(),
            highlight: default_highlight(),
        }
    }
}
