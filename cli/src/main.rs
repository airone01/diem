use clap::{CommandFactory as _, Parser as _};
use clap_complete::generate;

use diem::{Cli, Commands, Config, GithubProvider, Provider, ProviderSource, ProvidersCommands};

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
    let mut cfg: Config = confy::load("diem", "config")?;

    match args.command {
        Commands::Completions { .. } => unreachable!(),
        Commands::Install { package } => {
            println!("Installing package: {}", package);
            // TODO: Implement package installation
            unimplemented!()
        }
        Commands::Remove { package } => {
            println!("Removing package: {}", package);
            // TODO: Implement package removal
            unimplemented!()
        }
        Commands::Update { package } => {
            if let Some(pkg) = package {
                println!("Updating package: {}", pkg);
            } else {
                println!("Updating all packages");
            }
            // TODO: Implement package updates
            unimplemented!()
        }
        Commands::Providers { command } => match command {
            ProvidersCommands::Add {
                provider: provider_name,
            } => {
                // Parse provider string (format: "github:owner/repo@ref:path")
                let parts: Vec<&str> = provider_name.split(':').collect();
                if parts.len() != 2 || parts[0] != "github" {
                    anyhow::bail!("Invalid provider format. Expected: github:owner/repo@ref:path");
                }

                let repo_parts: Vec<&str> = parts[1].split('@').collect();
                if repo_parts.len() != 2 {
                    anyhow::bail!("Invalid repository format. Expected: owner/repo@ref");
                }

                let owner_repo: Vec<&str> = repo_parts[0].split('/').collect();
                if owner_repo.len() != 2 {
                    anyhow::bail!("Invalid owner/repo format. Expected: owner/repo");
                }

                let path_parts: Vec<&str> = repo_parts[1].split(':').collect();
                if path_parts.len() != 2 {
                    anyhow::bail!("Invalid ref:path format. Expected: ref:path");
                }

                let provider = Provider {
                    name: provider_name.clone(),
                    source: ProviderSource::Github(GithubProvider {
                        owner: owner_repo[0].to_string(),
                        repo: owner_repo[1].to_string(),
                        ref_: path_parts[0].to_string(),
                        path: path_parts[1].to_string(),
                    }),
                    provider_handler_version: 1,
                };

                // Add provider to config
                cfg.providers.push(provider);
                confy::store("diem", "config", &cfg)?;
                println!("Added provider: {}", provider_name);
            }
            ProvidersCommands::Remove { provider } => {
                cfg.providers.retain(|p| p.name != provider);
                confy::store("diem", "config", &cfg)?;
                println!("Removed provider: {}", provider);
            }
            ProvidersCommands::List => {
                println!("Installed providers:");
                for provider in &cfg.providers {
                    println!("  - {}", provider.name);
                }
            }
        },
    }
    Ok(())
}
