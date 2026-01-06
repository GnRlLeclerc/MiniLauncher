//! CLI args and styles
use anstyle::*;
use clap::{Parser, builder};

// https://stackoverflow.com/a/76916424
fn get_styles() -> builder::Styles {
    builder::Styles::styled()
        .usage(
            Style::new()
                .bold()
                .underline()
                .fg_color(Some(Color::Ansi(AnsiColor::Yellow))),
        )
        .header(
            Style::new()
                .bold()
                .underline()
                .fg_color(Some(Color::Ansi(AnsiColor::Yellow))),
        )
        .literal(Style::new().fg_color(Some(Color::Ansi(AnsiColor::Green))))
        .invalid(
            Style::new()
                .bold()
                .fg_color(Some(Color::Ansi(AnsiColor::Red))),
        )
        .error(
            Style::new()
                .bold()
                .fg_color(Some(Color::Ansi(AnsiColor::Red))),
        )
        .valid(
            Style::new()
                .bold()
                .underline()
                .fg_color(Some(Color::Ansi(AnsiColor::Green))),
        )
        .placeholder(Style::new().fg_color(Some(Color::Ansi(AnsiColor::White))))
}

/// Fancy and minimalistic app launcher
#[derive(Parser, Debug)]
#[command(version, about, long_about = None, styles = get_styles())]
pub struct Args {
    /// Refresh the application cache
    #[arg(short, long)]
    pub refresh: bool,
}
