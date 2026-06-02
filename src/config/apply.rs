//! Apply configuration settings to the app

use slint::ComponentHandle;

use crate::ui::{Launcher, LauncherColors, LauncherConfig};

use super::colors::Colors;
use super::config::Config;

/// Apply color scheme to the slint color singleton
pub fn apply_colors(launcher: &Launcher, colors: &Colors) {
    let target = launcher.global::<LauncherColors>();
    target.set_background(colors.background);
    target.set_text(colors.text);
    target.set_border(colors.border);
    target.set_accent(colors.accent);
    target.set_highlight(colors.highlight);
}

/// Apply general configuration options to the launcher UI
pub fn apply_config(launcher: &Launcher, config: &Config) {
    let target = launcher.global::<LauncherConfig>();
    target.set_animation(config.animation_duration as i64);
    target.set_border(config.border_width as f32);
    target.set_radius(config.border_radius as f32);
    target.set_mode(config.mode.clone());
    target.set_max_entries(config.max_entries as i32);
    target.set_font_family(config.font_family.clone().unwrap_or_default().into());
    target.set_font_size(config.font_size as f32);
    target.set_opacity(config.opacity);
}
