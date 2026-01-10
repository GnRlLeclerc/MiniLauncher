//! Apply configuration settings to the app

use slint::ComponentHandle;

use crate::{
    config::colors::Colors,
    ui::{Launcher, LauncherColors},
};

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
