//! Apply configuration settings to the app

use slint::ComponentHandle;

use crate::ui::{Launcher, LauncherColors, LauncherConfig};

use super::colors::Colors;
use super::config::Config;

/// Apply color scheme to the slint color singleton
pub fn apply_colors(launcher: &Launcher, colors: &Colors) {
    launcher
        .global::<LauncherColors>()
        .set_background(colors.background);
    launcher.global::<LauncherColors>().set_text(colors.text);
    launcher
        .global::<LauncherColors>()
        .set_border(colors.border);
    launcher
        .global::<LauncherColors>()
        .set_accent(colors.accent);
    launcher
        .global::<LauncherColors>()
        .set_highlight(colors.highlight);
}

/// Apply general configuration options to the launcher UI
pub fn apply_config(launcher: &Launcher, config: &Config) {
    launcher
        .global::<LauncherConfig>()
        .set_animation(config.animation_duration as i64);
    launcher
        .global::<LauncherConfig>()
        .set_border(config.border_width as f32);
    launcher
        .global::<LauncherConfig>()
        .set_radius(config.border_radius as f32);
    launcher
        .global::<LauncherConfig>()
        .set_mode(config.mode.clone());
    launcher
        .global::<LauncherConfig>()
        .set_max_entries(config.max_entries as i32);
    launcher
        .global::<LauncherConfig>()
        .set_font_family(config.font_family.clone().unwrap_or_default().into());
    launcher
        .global::<LauncherConfig>()
        .set_font_size(config.font_size as f32);
}
