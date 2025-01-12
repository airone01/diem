use clap::{CommandFactory as _, Parser as _};
use clap_complete::generate;

use diem::{Cli, Commands, Config};

/// The main entry point for the CLI application.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Cli::parse();

    match args.command {
        Commands::Completions { shell } => {
            let mut cmd = Cli::command();
            generate(
                shell,
                &mut cmd,
                Cli::command().get_name().to_string(),
                &mut std::io::stdout(),
            );
            Ok(())
        }
        _ => match_subcommands(args).await,
    }
}

/// For all commands that require the configuration file to be loaded,
/// this function will load the configuration file and then match the
/// subcommands.
async fn match_subcommands(args: Cli) -> anyhow::Result<()> {
    let cfg: Config = confy::load("diem", "config")?;
    dbg!(cfg);

    match args.command {
        Commands::Completions { .. } => unreachable!(),
        Commands::Install { package } => {
            unimplemented!("Install package: {}", package);
        }
        Commands::Remove { package } => {
            unimplemented!("Uninstall package: {}", package);
        }
        Commands::Update { package } => {
            unimplemented!("Update package: {:?}", package);
        }
        Commands::Providers { command } => match command {
            _ => unimplemented!("Provider command"),
        },
    }
}
