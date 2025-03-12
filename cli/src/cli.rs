use std::path::PathBuf;

use clap::builder::styling::{Color, Style};
use clap::{Parser, Subcommand};
use clap_complete::Shell;

/// A package manager
#[derive(Debug, Parser)]
#[command(name = "diem")]
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
    Install {
        /// The app to install
        app: String,
    },

    /// Uninstall a package
    #[command(aliases = ["un", "rm", "delete", "uninstall"])]
    #[command(long_about = "Uninstall one or more packages for the current user")]
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

    /// Configure the package providers
    #[command(long_about = "Configure the package providers for the current user")]
    Providers {
        /// The provider to configure
        #[command(subcommand)]
        command: ProvidersCommands,
    },
    
    /// Manage artifactories
    #[command(aliases = ["art", "registry"])]
    #[command(long_about = "Subscribe to, create, or manage artifactories (package registries)")]
    Artifactory {
        /// The artifactory command to run
        #[command(subcommand)]
        command: ArtifactoryCommands,
    },
    
    /// Search for packages in subscribed artifactories
    #[command(aliases = ["s", "find"])]
    #[command(long_about = "Search for packages in all subscribed artifactories")]
    Search {
        /// The query to search for
        query: String,
    },
    
    /// List all apps available from subscribed artifactories
    #[command(aliases = ["ls"])]
    #[command(long_about = "List all available apps from subscribed artifactories")]
    List,
    
    /// Sync packages between sgoinfre and goinfre directories
    #[command(long_about = "Sync packages from sgoinfre to goinfre directory")]
    Sync,
    
    /// Configure directories used by diem
    #[command(aliases = ["cfg", "dirs"])]
    #[command(long_about = "Configure the directories used by diem")]
    Config {
        /// The configuration command to run
        #[command(subcommand)]
        command: ConfigCommands,
    },

    /// Generate shell completions for a given shell
    #[command(aliases = ["complete"])]
    Completions {
        /// The shell to generate completions for
        shell: Shell,
    },
}

#[derive(Debug, Subcommand)]
pub enum ProvidersCommands {
    /// Add a provider
    Add {
        /// The provider to add
        provider: String,
    },

    /// Remove a provider
    Remove {
        /// The provider to remove
        provider: String,
    },

    /// List all providers
    List,
}

#[derive(Debug, Subcommand)]
pub enum ArtifactoryCommands {
    /// Subscribe to an artifactory
    Subscribe {
        /// The name for this artifactory subscription
        name: String,
        
        /// The path to the artifactory file or URL
        source: String,
        
        /// Whether to automatically update from this artifactory
        #[arg(short, long)]
        auto_update: bool,
    },
    
    /// Unsubscribe from an artifactory
    Unsubscribe {
        /// The name of the artifactory to unsubscribe from
        name: String,
    },
    
    /// List all subscribed artifactories
    List,
    
    /// Create a new artifactory
    Create {
        /// The name of the artifactory
        name: String,
        
        /// The path where to save the artifactory
        #[arg(short, long)]
        path: PathBuf,
        
        /// Whether this artifactory should be publicly accessible
        #[arg(short, long)]
        public: bool,
        
        /// Description of the artifactory
        #[arg(short, long)]
        description: Option<String>,
        
        /// Maintainer of the artifactory
        #[arg(short, long)]
        maintainer: Option<String>,
    },
    
    /// Add an app to an artifactory
    AddApp {
        /// Path to the artifactory file
        artifactory: PathBuf,
        
        /// Path to the app definition file
        app: PathBuf,
    },
}

#[derive(Debug, Subcommand)]
pub enum ConfigCommands {
    /// Set the sgoinfre directory
    #[command(name = "set-sgoinfre")]
    SetSgoinfre {
        /// Path to the sgoinfre directory
        path: PathBuf,
    },
    
    /// Set the goinfre directory
    #[command(name = "set-goinfre")]
    SetGoinfre {
        /// Path to the goinfre directory
        path: PathBuf,
    },
    
    /// Set the shared artifactory directory
    #[command(name = "set-shared-artifactory")]
    SetSharedArtifactory {
        /// Path to the shared artifactory directory
        path: PathBuf,
    },
    
    /// Set the installation directory
    #[command(name = "set-install-dir")]
    SetInstallDir {
        /// Path to the installation directory
        path: PathBuf,
    },
    
    /// Show the current configuration
    Show,
}

// Original color
const BLURPLE: (u8, u8, u8) = (90, 69, 254);

// Generic gradient
// const PURPLE: (u8, u8, u8) = (239, 0, 199);
const PINK: (u8, u8, u8) = (255, 43, 137);
// const ORANGE: (u8, u8, u8) = (255, 128, 89);
// const GOLD: (u8, u8, u8) = (255, 194, 72);
// const YELLOW: (u8, u8, u8) = (249, 248, 113);

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
