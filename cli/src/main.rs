use anyhow::Result;
use clap::{CommandFactory as _, Parser as _};
use clap_complete::generate;
use colored::*;

use diem::{
    AppManager, Artifactory, Cli, Commands, Config, GithubProvider, PackageManager, Provider, ProviderManager,
    ProviderSource, ProvidersCommands, ArtifactoryCommands, ConfigCommands, 
    artifactory::manager::ArtifactoryManager,
    config::{ArtifactorySource, ArtifactorySubscription},
    utils::ui,
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
            println!("{}", ui::title(&format!("Installing: {}", app)));

            let pb = ui::spinner();
            pb.set_message("Initializing package manager...");
            
            let package_manager = PackageManager::new(cfg.install_dir.clone());
            let app_manager = AppManager::new(package_manager);
            let provider_manager = ProviderManager::new_from_config(&cfg);

            pb.set_message(format!("Finding app: {}", app.cyan()));
            let (app, provider) = provider_manager.find_app(&app, &cfg).await?;
            pb.finish_with_message(ui::success(&format!("Found app: {} in {}", 
                app.name.green(), provider.name.blue())));
                
            app_manager.install_app(&app, &provider).await?;
        }
        Commands::Remove { package } => {
            println!("{}", ui::title(&format!("Removing: {}", package)));
            
            let package_manager = PackageManager::new(cfg.install_dir.clone());
            package_manager.uninstall_package(&package, None).await?;
        }
        Commands::Update { package } => {
            let provider_manager = ProviderManager::new_from_config(&cfg);
            
            if let Some(pkg_name) = package {
                println!("{}", ui::title(&format!("Updating: {}", pkg_name)));
                
                let pb = ui::spinner();
                pb.set_message(format!("Finding app: {}", pkg_name.cyan()));
                
                // Find the app and update it
                let (app, provider) = provider_manager.find_app(&pkg_name, &cfg).await?;
                pb.finish_with_message(ui::success(&format!("Found app: {} in {}", 
                    app.name.green(), provider.name.blue())));
                
                let package_manager = PackageManager::new(cfg.install_dir.clone());
                let app_manager = AppManager::new(package_manager);
                
                for pkg in &app.packages {
                    app_manager.package_manager.update_package(pkg, &provider).await?;
                }
                
                println!("{}", ui::success(&format!("Updated app: {}", app.name)));
            } else {
                println!("{}", ui::title("Updating all packages"));
                
                // Get all apps from providers and update them
                let installed_packages = &cfg.packages;
                
                let package_manager = PackageManager::new(cfg.install_dir.clone());
                let app_manager = AppManager::new(package_manager);
                
                if installed_packages.is_empty() {
                    println!("{}", ui::warning("No packages installed"));
                    return Ok(());
                }
                
                println!("{}", ui::info(&format!("Found {} installed packages", installed_packages.len())));
                
                for package in installed_packages {
                    // Find the provider that has this package
                    if let Ok((app, provider)) = provider_manager.find_app(&package.name, &cfg).await {
                        println!("{}", ui::section(&format!("Updating: {}", app.name)));
                        for pkg in &app.packages {
                            app_manager.package_manager.update_package(pkg, &provider).await?;
                        }
                    }
                }
                
                println!("{}", ui::success("All packages updated successfully"));
            }
        }
        Commands::Providers { command } => match_providers_commands(cfg, command).await?,
        Commands::Artifactory { command } => match_artifactory_commands(&mut cfg, command).await?,
        Commands::Search { query } => search_apps(&cfg, &query).await?,
        Commands::List => list_available_apps(&cfg).await?,
        Commands::Sync => {
            println!("{}", ui::title("Synchronizing packages"));
            
            let pb = ui::spinner();
            pb.set_message("Syncing packages from sgoinfre to goinfre...");
            
            cfg.sync_goinfre_from_sgoinfre()?;
            
            pb.finish_with_message(ui::success("Synchronization completed successfully"));
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
            println!("{}", ui::title(&format!("Adding provider: {}", provider_name)));
            
            let pb = ui::spinner();
            pb.set_message("Validating provider format...");
            
            // Parse provider string (format: "github:owner/repo@ref:path")
            let parts: Vec<&str> = provider_name.split(':').collect();
            if parts.len() != 3 || parts[0] != "github" {
                pb.finish_with_message(ui::error("Invalid provider format"));
                anyhow::bail!("Invalid provider format. Expected: github:owner/repo@ref:path");
            }

            let repo_parts: Vec<&str> = parts[1].split('@').collect();
            if repo_parts.len() != 2 {
                pb.finish_with_message(ui::error("Invalid repository format"));
                anyhow::bail!("Invalid repository format. Expected: owner/repo@ref");
            }

            let owner_repo: Vec<&str> = repo_parts[0].split('/').collect();
            if owner_repo.len() != 2 {
                pb.finish_with_message(ui::error("Invalid owner/repo format"));
                anyhow::bail!("Invalid owner/repo format. Expected: owner/repo");
            }

            pb.set_message("Creating provider...");
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

            pb.set_message("Adding provider to configuration...");
            provider_manager.add_provider(provider)?;
            provider_manager.save_to_config(&mut cfg);
            confy::store("diem", "config", &cfg)?;
            
            pb.finish_with_message(ui::success(&format!("Added provider: {}", provider_name)));
        }
        ProvidersCommands::Remove { provider } => {
            println!("{}", ui::title(&format!("Removing provider: {}", provider)));
            
            let pb = ui::spinner();
            pb.set_message(format!("Removing provider: {}", provider.cyan()));
            
            provider_manager.remove_provider(&provider)?;
            provider_manager.save_to_config(&mut cfg);
            confy::store("diem", "config", &cfg)?;
            
            pb.finish_with_message(ui::success(&format!("Removed provider: {}", provider)));
        }
        ProvidersCommands::List => {
            println!("{}", ui::title("Installed Providers"));
            
            let providers = provider_manager.list_providers();
            if providers.is_empty() {
                println!("{}", ui::warning("No providers installed"));
                return Ok(());
            }
            
            let mut data = Vec::new();
            for (i, provider) in providers.iter().enumerate() {
                let provider_source = match &provider.source {
                    ProviderSource::Github(github) => {
                        format!("GitHub: {}/{} ({})", github.owner.cyan(), github.repo.green(), github.ref_.yellow())
                    }
                };
                
                let number = format!("{}.", i + 1).cyan();
                let name = provider.name.green().bold();
                println!("  {} {} - {}", number, name, provider_source);
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
    println!("{}", ui::title(&format!("Search Results for: {}", query.cyan())));
    
    let pb = ui::spinner();
    pb.set_message(format!("Searching for apps matching: {}", query.cyan()));
    
    let manager = ArtifactoryManager::new(cfg.clone());
    let results = manager.search_apps(query)?;
    
    if results.is_empty() {
        pb.finish_with_message(ui::warning(&format!("No apps found matching: {}", query)));
        return Ok(());
    }
    
    pb.finish_with_message(ui::success(&format!("Found {} artifactories with matching apps", results.len())));
    
    for (artifactory, apps) in results {
        println!("{}", ui::section(&format!("From artifactory: {}", artifactory.green())));
        
        for (i, app) in apps.iter().enumerate() {
            let number = format!("{}.", i + 1).cyan();
            println!("  {} {}", number, app.green());
        }
    }
    
    Ok(())
}

async fn list_available_apps(cfg: &Config) -> Result<()> {
    println!("{}", ui::title("Available Applications"));
    
    let pb = ui::spinner();
    pb.set_message("Loading subscribed artifactories...");
    
    let manager = ArtifactoryManager::new(cfg.clone());
    let artifactories = manager.load_all_subscribed();
    
    if artifactories.is_empty() {
        pb.finish_with_message(ui::warning("No artifactories found. Subscribe to an artifactory first."));
        return Ok(());
    }
    
    pb.finish_with_message(ui::success(&format!("Found {} artifactories", artifactories.len())));
    
    let mut app_count = 0;
    for result in artifactories {
        match result {
            Ok(artifactory) => {
                println!("{}", ui::section(&format!("From: {}", artifactory.name.green())));
                
                if artifactory.apps.is_empty() {
                    println!("  {}", ui::info("No apps available in this artifactory"));
                    continue;
                }
                
                for (i, app) in artifactory.apps.iter().enumerate() {
                    let number = format!("{}.", i + 1).cyan();
                    let name = app.name.green().bold();
                    let version = format!("v{}", app.version).yellow();
                    
                    let description = if let Some(desc) = &app.description {
                        format!(" - {}", desc.blue())
                    } else {
                        "".to_string()
                    };
                    
                    println!("  {} {} ({}){}",
                        number,
                        name,
                        version,
                        description
                    );
                    app_count += 1;
                }
            },
            Err(e) => {
                println!("  {}", ui::error(&format!("Error loading artifactory: {}", e)));
            }
        }
    }
    
    println!("\n{}", ui::success(&format!("Total apps available: {}", app_count)));
    
    Ok(())
}

async fn match_config_commands(cfg: &mut Config, command: ConfigCommands) -> Result<()> {
    match command {
        ConfigCommands::SetSgoinfre { path } => {
            println!("{}", ui::title("Configuration Update"));
            
            let pb = ui::spinner();
            pb.set_message(format!("Setting sgoinfre directory to: {}", path.display().to_string().cyan()));
            
            cfg.sgoinfre_dir = Some(path.clone());
            confy::store("diem", "config", &cfg)?;
            
            pb.finish_with_message(ui::success(&format!("Set sgoinfre directory to: {}", path.display())));
        },
        ConfigCommands::SetGoinfre { path } => {
            println!("{}", ui::title("Configuration Update"));
            
            let pb = ui::spinner();
            pb.set_message(format!("Setting goinfre directory to: {}", path.display().to_string().cyan()));
            
            cfg.goinfre_dir = Some(path.clone());
            confy::store("diem", "config", &cfg)?;
            
            pb.finish_with_message(ui::success(&format!("Set goinfre directory to: {}", path.display())));
        },
        ConfigCommands::SetSharedArtifactory { path } => {
            println!("{}", ui::title("Configuration Update"));
            
            let pb = ui::spinner();
            pb.set_message(format!("Setting shared artifactory directory to: {}", path.display().to_string().cyan()));
            
            cfg.shared_artifactory_dir = Some(path.clone());
            confy::store("diem", "config", &cfg)?;
            
            pb.finish_with_message(ui::success(&format!("Set shared artifactory directory to: {}", path.display())));
        },
        ConfigCommands::Show => {
            println!("{}", ui::title("Current Configuration"));
            
            let mut config_items = Vec::new();
            
            config_items.push(("Install directory", cfg.install_dir.display().to_string().as_str()));
            
            if let Some(sgoinfre) = &cfg.sgoinfre_dir {
                config_items.push(("Sgoinfre directory", sgoinfre.display().to_string().as_str()));
            } else {
                config_items.push(("Sgoinfre directory", "Not set".red().to_string().as_str()));
            }
            
            if let Some(goinfre) = &cfg.goinfre_dir {
                config_items.push(("Goinfre directory", goinfre.display().to_string().as_str()));
            } else {
                config_items.push(("Goinfre directory", "Not set".red().to_string().as_str()));
            }
            
            if let Some(shared) = &cfg.shared_artifactory_dir {
                config_items.push(("Shared artifactory", shared.display().to_string().as_str()));
            } else {
                config_items.push(("Shared artifactory", "Not set".red().to_string().as_str()));
            }
            
            config_items.push(("Subscribed artifactories", cfg.subscribed_artifactories.len().to_string().as_str()));
            
            // Create a key-value table
            println!("{}", ui::key_value_table("Settings", config_items));
            
            // If there are providers, list them
            if !cfg.providers.is_empty() {
                println!("{}", ui::section("Providers"));
                for (i, provider) in cfg.providers.iter().enumerate() {
                    println!("  {}. {}", (i+1).to_string().cyan(), provider.name.green());
                }
            }
            
            // If there are artifactories, list them
            if !cfg.subscribed_artifactories.is_empty() {
                println!("{}", ui::section("Subscribed Artifactories"));
                for (i, sub) in cfg.subscribed_artifactories.iter().enumerate() {
                    let source_type = match &sub.source {
                        ArtifactorySource::Local(_) => "Local".blue(),
                        ArtifactorySource::Remote(_) => "Remote".magenta(),
                    };
                    println!("  {}. {} ({})", (i+1).to_string().cyan(), sub.name.green(), source_type);
                }
            }
        },
    }
    
    Ok(())
}
