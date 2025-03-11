use crate::{App, PackageManager, Provider, utils::ui};
use anyhow::Result;
use colored::*;
use semver::Version;

// Helper function to recursively search for a binary in a directory
fn find_binary_in_dir(dir: &std::path::Path, filename: String) -> Option<std::path::PathBuf> {
    if !dir.is_dir() {
        return None;
    }
    
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return None,
    };
    
    for entry in entries {
        if let Ok(entry) = entry {
            let path = entry.path();
            
            if path.is_dir() {
                if let Some(found) = find_binary_in_dir(&path, filename.clone()) {
                    return Some(found);
                }
            } else if path.file_name()
                .and_then(|name| name.to_str())
                .map_or(false, |name| name == filename)
            {
                return Some(path);
            }
        }
    }
    
    None
}

pub struct AppManager {
    pub package_manager: PackageManager,
}

impl AppManager {
    pub fn new(package_manager: PackageManager) -> Self {
        Self { package_manager }
    }

    pub async fn install_app(&self, app: &App, provider: &Provider) -> Result<()> {
        let pb = ui::spinner();

        pb.set_message(format!("Installing app: {} {}", app.name.cyan(), app.version.to_string().yellow()));

        // Install each package required by the app
        for package in &app.packages {
            self.package_manager
                .install_package(package, provider)
                .await?;
        }

        // Create symlinks for each command
        println!("{}", ui::section("Setting up commands"));
        
        for (i, cmd) in app.commands.iter().enumerate() {
            let package = &app.packages[0]; // Usually commands come from the main package
            let package_dir = self
                .package_manager
                .get_package_dir(&package.name, &package.version);
            
            pb.set_message(format!("Setting up command [{}/{}]: {}", 
                (i+1).to_string().yellow(), 
                app.commands.len().to_string().yellow(), 
                cmd.command.cyan()));
            
            // Check if the binary exists - we need to be flexible with paths
            let target_path = package_dir.join(&cmd.path);
            println!("{}", ui::info(&format!("Checking path: {}", target_path.display())));
            
            if !target_path.exists() {
                // First attempt: Look for the file directly without any prefix
                let alternate_path = package_dir.join(cmd.path.file_name().unwrap_or_default());
                println!("{}", ui::info(&format!("Trying alternate path: {}", alternate_path.display())));
                
                if alternate_path.exists() {
                    println!("{}", ui::success(&format!("Found binary at: {}", alternate_path.display())));
                    // Create a temporary symlink to make the expected structure
                    let parent = target_path.parent().unwrap_or(&package_dir);
                    std::fs::create_dir_all(parent)?;
                    
                    #[cfg(unix)]
                    std::os::unix::fs::symlink(&alternate_path, &target_path)?;
                    
                    #[cfg(windows)]
                    std::os::windows::fs::symlink_file(&alternate_path, &target_path)?;
                } else {
                    // Do a global search in the package directory
                    println!("{}", ui::info("Searching for binary in package directory..."));
                    let found = find_binary_in_dir(&package_dir, cmd.path.file_name().unwrap_or_default().to_string_lossy().to_string());
                    
                    if let Some(found_path) = found {
                        println!("{}", ui::success(&format!("Found binary at: {}", found_path.display())));
                        // Create a temporary symlink to make the expected structure
                        let parent = target_path.parent().unwrap_or(&package_dir);
                        std::fs::create_dir_all(parent)?;
                        
                        #[cfg(unix)]
                        std::os::unix::fs::symlink(&found_path, &target_path)?;
                        
                        #[cfg(windows)]
                        std::os::windows::fs::symlink_file(&found_path, &target_path)?;
                    } else {
                        return Err(anyhow::anyhow!("{}", 
                            ui::error(&format!("Command binary not found at expected path: {}", target_path.display()))));
                    }
                }
            }
            
            self.package_manager
                .create_command_symlink(cmd, &package_dir)
                .await?;
                
            println!("{}", ui::success(&format!("Command '{}' is now available", cmd.command)));
        }

        pb.finish_with_message(ui::success(&format!(
            "Successfully installed {} {}",
            app.name, app.version
        )));
        
        // Display a summary of what was installed
        println!("\n{}", ui::title(&format!("App: {} {}", app.name, app.version)));
        if let Some(description) = &app.description {
            println!("{}", description);
        }
        println!("\n{} {}", "Packages:".cyan().bold(), app.packages.len().to_string().yellow());
        for package in &app.packages {
            println!("  • {} {}", package.name.green(), package.version.to_string().yellow());
        }
        
        println!("\n{} {}", "Commands:".cyan().bold(), app.commands.len().to_string().yellow());
        for cmd in &app.commands {
            println!("  • {} → {}", cmd.command.green(), cmd.path.display().to_string().blue());
        }
        
        Ok(())
    }

    pub async fn uninstall_app(&self, _app_name: &str, _version: Option<Version>) -> Result<()> {
        // TODO: Implement app uninstallation
        // This should:
        // 1. Remove all app packages
        // 2. Remove command symlinks
        // 3. Clean up any app-specific configuration
        unimplemented!()
    }
}
