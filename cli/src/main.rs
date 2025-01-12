use anyhow::Result;
use clap::{CommandFactory as _, Parser as _};
use clap_complete::generate;

use diem::{
    AppManager, Cli, Commands, Config, GithubProvider, PackageManager, Provider, ProviderManager,
    ProviderSource, ProvidersCommands,
};

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
        _ => match_commands(args).await,
    }
}

/// For all commands that require the configuration file to be loaded,
/// this function will load the configuration file and then match the
/// subcommands.
async fn match_commands(args: Cli) -> anyhow::Result<()> {
    let cfg: Config = confy::load("diem", "config")?;
    cfg.ensure_dirs_exist()?;

    match args.command {
        Commands::Completions { .. } => unreachable!(),
        Commands::Install { app } => {
            println!("Installing app: {}", app);

            let package_manager = PackageManager::new(cfg.install_dir.clone());
            let app_manager = AppManager::new(package_manager);
            let provider_manager = ProviderManager::new_from_config(&cfg);

            let (app, provider) = provider_manager.find_app(&app).await?;
            app_manager.install_app(&app, &provider).await?;
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
        Commands::Providers { command } => match_providers_commands(cfg, command).await?,
    }
    Ok(())
}

async fn match_providers_commands(mut cfg: Config, command: ProvidersCommands) -> Result<()> {
    let mut provider_manager = ProviderManager::new_from_config(&cfg);
    match command {
        ProvidersCommands::Add {
            provider: provider_name,
        } => {
            // Parse provider string (format: "github:owner/repo@ref:path")
            let parts: Vec<&str> = provider_name.split(':').collect();
            if parts.len() != 3 || parts[0] != "github" {
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

            let provider = Provider {
                name: provider_name.clone(),
                source: ProviderSource::Github(GithubProvider {
                    owner: owner_repo[0].to_string(),
                    repo: owner_repo[1].to_string(),
                    ref_: repo_parts[1].to_string(),
                    path: parts[2].to_string(),
                }),
                provider_handler_version: 1,
            };

            provider_manager.add_provider(provider)?;
            provider_manager.save_to_config(&mut cfg);
            confy::store("diem", "config", &cfg)?;
            println!("Added provider: {}", provider_name);
        }
        ProvidersCommands::Remove { provider } => {
            provider_manager.remove_provider(&provider)?;
            provider_manager.save_to_config(&mut cfg);
            confy::store("diem", "config", &cfg)?;
            println!("Removed provider: {}", provider);
        }
        ProvidersCommands::List => {
            println!("Installed providers:");
            for provider in provider_manager.list_providers() {
                println!("  - {}", provider.name);
            }
        }
    }
    Ok(())
}
