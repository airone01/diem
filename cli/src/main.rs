use anyhow::Result;
use clap::{CommandFactory as _, Parser as _};
use clap_complete::generate;

use diem::{
    AppManager, Artifactory, Cli, Commands, Config, GithubProvider, PackageManager, Provider, ProviderManager,
    ProviderSource, ProvidersCommands, ArtifactoryCommands, ConfigCommands, 
    artifactory::manager::ArtifactoryManager,
    config::{ArtifactorySource, ArtifactorySubscription},
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
    let mut cfg: Config = confy::load("diem", "config")?;
    cfg.ensure_dirs_exist()?;
    
    // Sync goinfre from sgoinfre on startup if needed
    cfg.sync_goinfre_from_sgoinfre()?;

    match args.command {
        Commands::Completions { .. } => unreachable!(),
        Commands::Install { app } => {
            println!("Installing app: {}", app);

            let package_manager = PackageManager::new(cfg.install_dir.clone());
            let app_manager = AppManager::new(package_manager);
            let provider_manager = ProviderManager::new_from_config(&cfg);

            let (app, provider) = provider_manager.find_app(&app, &cfg).await?;
            app_manager.install_app(&app, &provider).await?;
        }
        Commands::Remove { package } => {
            println!("Removing package: {}", package);
            
            let package_manager = PackageManager::new(cfg.install_dir.clone());
            package_manager.uninstall_package(&package, None).await?;
        }
        Commands::Update { package } => {
            let provider_manager = ProviderManager::new_from_config(&cfg);
            
            if let Some(pkg_name) = package {
                println!("Updating package: {}", pkg_name);
                
                // Find the app and update it
                let (app, provider) = provider_manager.find_app(&pkg_name, &cfg).await?;
                
                let package_manager = PackageManager::new(cfg.install_dir.clone());
                let app_manager = AppManager::new(package_manager);
                
                for pkg in &app.packages {
                    app_manager.package_manager.update_package(pkg, &provider).await?;
                }
                
                println!("Updated app: {}", app.name);
            } else {
                println!("Updating all packages");
                
                // Get all apps from providers and update them
                let installed_packages = &cfg.packages;
                
                let package_manager = PackageManager::new(cfg.install_dir.clone());
                let app_manager = AppManager::new(package_manager);
                
                for package in installed_packages {
                    // Find the provider that has this package
                    if let Ok((app, provider)) = provider_manager.find_app(&package.name, &cfg).await {
                        for pkg in &app.packages {
                            app_manager.package_manager.update_package(pkg, &provider).await?;
                        }
                    }
                }
                
                println!("All packages updated");
            }
        }
        Commands::Providers { command } => match_providers_commands(cfg, command).await?,
        Commands::Artifactory { command } => match_artifactory_commands(&mut cfg, command).await?,
        Commands::Search { query } => search_apps(&cfg, &query).await?,
        Commands::List => list_available_apps(&cfg).await?,
        Commands::Sync => {
            println!("Syncing packages from sgoinfre to goinfre...");
            cfg.sync_goinfre_from_sgoinfre()?;
            println!("Sync complete");
        },
        Commands::Config { command } => match_config_commands(&mut cfg, command).await?,
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

async fn match_artifactory_commands(cfg: &mut Config, command: ArtifactoryCommands) -> Result<()> {
    let artifactory_manager = ArtifactoryManager::new(cfg.clone());
    
    match command {
        ArtifactoryCommands::Subscribe { name, source, auto_update } => {
            // Determine if it's a local path or a URL
            let source = if source.starts_with("http://") || source.starts_with("https://") {
                ArtifactorySource::Remote(source)
            } else {
                let path = std::path::PathBuf::from(&source);
                
                // Validate the path exists and is a valid TOML file
                if !path.exists() {
                    anyhow::bail!("Artifactory file not found: {}", source);
                }
                
                if !path.is_file() {
                    anyhow::bail!("Path is not a file: {}", source);
                }
                
                if path.extension().map_or(true, |ext| ext != "toml") {
                    anyhow::bail!("Artifactory file must be a TOML file");
                }
                
                // Try to parse the TOML to verify it's valid
                let content = std::fs::read_to_string(&path)
                    .map_err(|e| anyhow::anyhow!("Failed to read artifactory file: {}", e))?;
                    
                toml::from_str::<Artifactory>(&content)
                    .map_err(|e| anyhow::anyhow!("Invalid artifactory file: {}", e))?;
                
                ArtifactorySource::Local(path)
            };
            
            let subscription = ArtifactorySubscription {
                name: name.clone(),
                source,
                auto_update,
            };
            
            let mut manager = ArtifactoryManager::new(cfg.clone());
            manager.add_subscription(subscription)?;
            
            // Save subscriptions to config
            cfg.subscribed_artifactories = manager.list_subscribed().into_iter().cloned().collect();
            confy::store("diem", "config", &cfg)?;
            
            println!("Subscribed to artifactory: {}", name);
        },
        ArtifactoryCommands::Unsubscribe { name } => {
            let mut manager = ArtifactoryManager::new(cfg.clone());
            manager.remove_subscription(&name)?;
            
            // Save subscriptions to config
            cfg.subscribed_artifactories = manager.list_subscribed().into_iter().cloned().collect();
            confy::store("diem", "config", &cfg)?;
            
            println!("Unsubscribed from artifactory: {}", name);
        },
        ArtifactoryCommands::List => {
            let manager = ArtifactoryManager::new(cfg.clone());
            let subscriptions = manager.list_subscribed();
            
            if subscriptions.is_empty() {
                println!("No artifactory subscriptions found");
                return Ok(());
            }
            
            println!("Subscribed artifactories:");
            for sub in subscriptions {
                let source_desc = match &sub.source {
                    ArtifactorySource::Local(path) => format!("Local: {}", path.display()),
                    ArtifactorySource::Remote(url) => format!("Remote: {}", url),
                };
                
                println!("  - {} ({})", sub.name, source_desc);
                println!("    Auto-update: {}", if sub.auto_update { "Yes" } else { "No" });
            }
        },
        ArtifactoryCommands::Create { name, path, public, description, maintainer } => {
            let artifactory = Artifactory {
                name: name.clone(),
                description,
                apps: Vec::new(),
                maintainer,
                public,
                artifactory_handler_version: 1,
            };
            
            artifactory_manager.create_artifactory(&artifactory, &path)?;
            println!("Created artifactory: {} at {}", name, path.display());
        },
        ArtifactoryCommands::AddApp { artifactory: art_path, app: app_path } => {
            // Load artifactory
            let art_content = std::fs::read_to_string(&art_path)?;
            let mut artifactory: Artifactory = toml::from_str(&art_content)?;
            
            // Load app
            let app_content = std::fs::read_to_string(&app_path)?;
            let app = toml::from_str(&app_content)?;
            
            // Add app to artifactory
            artifactory.apps.push(app);
            
            // Save updated artifactory
            let updated_content = toml::to_string_pretty(&artifactory)?;
            std::fs::write(&art_path, updated_content)?;
            
            println!("Added app to artifactory: {}", art_path.display());
        },
    }
    
    Ok(())
}

async fn search_apps(cfg: &Config, query: &str) -> Result<()> {
    let manager = ArtifactoryManager::new(cfg.clone());
    let results = manager.search_apps(query)?;
    
    if results.is_empty() {
        println!("No apps found matching: {}", query);
        return Ok(());
    }
    
    println!("Apps found for query '{}': ", query);
    for (artifactory, apps) in results {
        println!("  From artifactory '{}': ", artifactory);
        for app in apps {
            println!("    - {}", app);
        }
    }
    
    Ok(())
}

async fn list_available_apps(cfg: &Config) -> Result<()> {
    let manager = ArtifactoryManager::new(cfg.clone());
    let artifactories = manager.load_all_subscribed();
    
    if artifactories.is_empty() {
        println!("No artifactories found. Subscribe to an artifactory first.");
        return Ok(());
    }
    
    println!("Available apps:");
    for result in artifactories {
        match result {
            Ok(artifactory) => {
                println!("  From '{}': ", artifactory.name);
                for app in artifactory.apps {
                    println!("    - {} (v{})", app.name, app.version);
                }
            },
            Err(e) => {
                println!("  Error loading artifactory: {}", e);
            }
        }
    }
    
    Ok(())
}

async fn match_config_commands(cfg: &mut Config, command: ConfigCommands) -> Result<()> {
    match command {
        ConfigCommands::SetSgoinfre { path } => {
            cfg.sgoinfre_dir = Some(path.clone());
            confy::store("diem", "config", &cfg)?;
            println!("Set sgoinfre directory to: {}", path.display());
        },
        ConfigCommands::SetGoinfre { path } => {
            cfg.goinfre_dir = Some(path.clone());
            confy::store("diem", "config", &cfg)?;
            println!("Set goinfre directory to: {}", path.display());
        },
        ConfigCommands::SetSharedArtifactory { path } => {
            cfg.shared_artifactory_dir = Some(path.clone());
            confy::store("diem", "config", &cfg)?;
            println!("Set shared artifactory directory to: {}", path.display());
        },
        ConfigCommands::Show => {
            println!("Current configuration:");
            println!("  Install directory: {}", cfg.install_dir.display());
            
            if let Some(sgoinfre) = &cfg.sgoinfre_dir {
                println!("  Sgoinfre directory: {}", sgoinfre.display());
            } else {
                println!("  Sgoinfre directory: Not set");
            }
            
            if let Some(goinfre) = &cfg.goinfre_dir {
                println!("  Goinfre directory: {}", goinfre.display());
            } else {
                println!("  Goinfre directory: Not set");
            }
            
            if let Some(shared) = &cfg.shared_artifactory_dir {
                println!("  Shared artifactory directory: {}", shared.display());
            } else {
                println!("  Shared artifactory directory: Not set");
            }
            
            println!("  Subscribed artifactories: {}", cfg.subscribed_artifactories.len());
        },
    }
    
    Ok(())
}
