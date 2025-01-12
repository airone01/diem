use std::path::PathBuf;

use clap::builder::styling::{Color, Style};
use clap::{Parser, Subcommand};

/// A package manager
#[derive(Debug, Parser)]
#[command(name = "git")]
#[command(about = "A package manager", long_about = None)]
#[command(styles = get_styles())]
#[command(arg_required_else_help = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Path to run in
    #[arg(long, env, hide_env = true)]
    #[arg(global = true, value_name = "PATH")]
    #[arg(default_value = ".", hide_default_value = true)]
    pub cwd: PathBuf,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Install a package
    #[command(aliases = ["i", "in", "add", "get"])]
    #[command(long_about = "Install one or more packages for the current user")]
    #[command(arg_required_else_help = true)]
    Install {
        /// The package to install
        package: String,
    },

    /// Uninstall a package
    #[command(aliases = ["un", "rm", "delete", "uninstall"])]
    #[command(long_about = "Uninstall one or more packages for the current user")]
    #[command(arg_required_else_help = true)]
    Remove {
        /// The package to uninstall
        package: String,
    },

    /// Update packages
    #[command(aliases = ["up", "upgrade"])]
    #[command(
        long_about = "Update and upgrade one or more packages or all packages for the current user"
    )]
    Update {
        /// The package to update
        package: Option<String>,
    },
}

// Original color
const BLURPLE: (u8, u8, u8) = (90, 69, 254);

// Generic gradient
const PURPLE: (u8, u8, u8) = (239, 0, 199);
const PINK: (u8, u8, u8) = (255, 43, 137);
const ORANGE: (u8, u8, u8) = (255, 128, 89);
const GOLD: (u8, u8, u8) = (255, 194, 72);
const YELLOW: (u8, u8, u8) = (249, 248, 113);

// From switch palette
const BRIGHT_MAGENTA: (u8, u8, u8) = (228, 180, 255);

// From spot palette
const BRIGHTER_MAGENTA: (u8, u8, u8) = (248, 235, 255);

// From skip gradient
const BRIGHT_GREEN: (u8, u8, u8) = (87, 251, 219);

pub fn get_styles() -> clap::builder::Styles {
    clap::builder::Styles::styled()
        .usage(
            Style::new()
                .bold()
                .underline()
                .fg_color(Some(Color::Rgb(BLURPLE.into()))),
        )
        .header(
            Style::new()
                .bold()
                .underline()
                .fg_color(Some(Color::Rgb(BLURPLE.into()))),
        )
        .literal(Style::new().fg_color(Some(Color::Rgb(BRIGHT_MAGENTA.into()))))
        .invalid(Style::new().bold().fg_color(Some(Color::Rgb(PINK.into()))))
        .error(Style::new().bold().fg_color(Some(Color::Rgb(PINK.into()))))
        .valid(
            Style::new()
                .bold()
                .underline()
                .fg_color(Some(Color::Rgb(BRIGHT_GREEN.into()))),
        )
        .placeholder(Style::new().fg_color(Some(Color::Rgb(BRIGHTER_MAGENTA.into()))))
}
