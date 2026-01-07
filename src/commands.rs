//! Process commands

use std::{env, process::Stdio};

/// Commands to run for each launcher entry
pub struct Command {
    pub command: String,
    /// Whether to run this command in a terminal (TUI apps)
    pub terminal: bool,
}

fn get_default_terminal() -> Vec<String> {
    if let Ok(terminal) = env::var("TERMINAL") {
        return vec![terminal];
    }

    // Check for DE-specific terminals
    if let Ok(desktop_environment) = env::var("XDG_CURRENT_DESKTOP") {
        match desktop_environment.as_str() {
            "GNOME" => return vec!["kgx".into(), "-e".into()],
            "KDE" => return vec!["konsole".into()],
            "XFCE" => return vec!["xfce4-terminal".into()],
            "LXQt" => return vec!["lxterminal".into()],
            _ => {}
        }
    }

    // Default to xterm if no terminal is found
    vec!["xterm".into()]
}

/// Run a command
pub fn run(cmd: &Command) {
    let mut command = cmd
        .command
        .split_whitespace()
        // Filter out freedesktop template arguments
        .filter(|&arg| match arg {
            "%U" | "%u" | "%F" | "%f" | "@@" | "@@u" => false,
            _ => true,
        })
        .map(|arg| arg.to_string())
        .collect::<Vec<_>>();

    if cmd.terminal {
        for arg in get_default_terminal().into_iter().rev() {
            command.insert(0, arg);
        }
    }

    let home_dir = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());

    let _ = std::process::Command::new(&command[0])
        .args(command[1..].iter())
        .env("SHLVL", "0")
        .current_dir(home_dir)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
}
