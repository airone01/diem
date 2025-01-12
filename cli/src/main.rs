use anyhow::Result;
use clap::{CommandFactory as _, Parser as _};
use clap_complete::generate;

use std::str::FromStr;

use diem::{
    Artifactory, Cli, Commands, Config, GithubProvider, PackageManager, Provider, ProviderSource,
    ProvidersCommands,
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
        _ => match_subcommands(args).await,
    }
}

/// For all commands that require the configuration file to be loaded,
/// this function will load the configuration file and then match the
/// subcommands.
async fn match_subcommands(args: Cli) -> anyhow::Result<()> {
    let mut cfg: Config = confy::load("diem", "config")?;
    cfg.ensure_dirs_exist()?;

    match args.command {
        Commands::Completions { .. } => unreachable!(),
        Commands::Install { package } => {
            println!("Installing package: {}", package);
            install_package(&cfg, &package).await?;
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

async fn install_package(cfg: &Config, package_spec: &str) -> Result<()> {
    // Parse package specification (format: package_name@version)
    let (package_name, version) = if package_spec.contains('@') {
        let parts: Vec<&str> = package_spec.split('@').collect();
        (parts[0].to_string(), Some(parts[1].to_string()))
    } else {
        (package_spec.to_string(), None)
    };

    // Initialize package manager
    let package_manager = PackageManager::new(cfg.install_dir.clone());

    // Check if already installed
    if package_manager
        .is_package_installed(&package_name, version.as_deref())
        .await
    {
        println!("Package {} is already installed", package_spec);
        return Ok(());
    }

    // Fetch artifactories from all providers
    let mut found_package = None;
    let mut using_provider = None;

    for provider in &cfg.providers {
        let artifactory_content = provider.fetch_artifactory().await?;
        let artifactory: Artifactory = serde_json::from_slice(&artifactory_content)?;

        // Look for the package in the artifactory's apps
        for app in artifactory.apps {
            for package in app.packages {
                if package.name == package_name {
                    if let Some(req_version) = &version {
                        let req_version = semver::Version::from_str(req_version)?;
                        let pkg_version = semver::Version::from_str(&package.version)?;
                        if pkg_version == req_version {
                            found_package = Some(package);
                            using_provider = Some(provider);
                            break;
                        }
                    } else {
                        // If no version specified, use the latest
                        if let Some(existing) = &found_package {
                            let existing_version = semver::Version::from_str(&existing.version)?;
                            let pkg_version = semver::Version::from_str(&package.version)?;
                            if pkg_version > existing_version {
                                found_package = Some(package);
                                using_provider = Some(provider);
                            }
                        } else {
                            found_package = Some(package);
                            using_provider = Some(provider);
                        }
                    }
                }
            }
        }
    }

    // Install the package if found
    if let (Some(package), Some(provider)) = (found_package, using_provider) {
        package_manager.install_package(&package, provider).await?;
        println!("Successfully installed {}", package_spec);
        Ok(())
    } else {
        anyhow::bail!("Package {} not found in any provider", package_spec);
    }
}
