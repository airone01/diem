use crate::{App, PackageManager, Provider};
use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
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
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} [{elapsed_precise}] {msg}")
                .unwrap(),
        );

        pb.set_message(format!("Installing app: {} {}", app.name, app.version));

        // Install each package required by the app
        for package in &app.packages {
            self.package_manager
                .install_package(package, provider)
                .await?;
        }

        // Create symlinks for each command
        for cmd in &app.commands {
            let package = &app.packages[0]; // Usually commands come from the main package
            let package_dir = self
                .package_manager
                .get_package_dir(&package.name, &package.version);
            
            // Check if the binary exists - we need to be flexible with paths
            let target_path = package_dir.join(&cmd.path);
            println!("Checking if target exists: {}", target_path.display());
            
            if !target_path.exists() {
                // First attempt: Look for the file directly without any prefix
                let alternate_path = package_dir.join(cmd.path.file_name().unwrap_or_default());
                println!("Trying alternate path: {}", alternate_path.display());
                
                if alternate_path.exists() {
                    println!("Found binary at alternate path: {}", alternate_path.display());
                    // Create a temporary symlink to make the expected structure
                    let parent = target_path.parent().unwrap_or(&package_dir);
                    std::fs::create_dir_all(parent)?;
                    
                    #[cfg(unix)]
                    std::os::unix::fs::symlink(&alternate_path, &target_path)?;
                    
                    #[cfg(windows)]
                    std::os::windows::fs::symlink_file(&alternate_path, &target_path)?;
                } else {
                    // Do a global search in the package directory
                    let found = find_binary_in_dir(&package_dir, cmd.path.file_name().unwrap_or_default().to_string_lossy().to_string());
                    
                    if let Some(found_path) = found {
                        println!("Found binary through search: {}", found_path.display());
                        // Create a temporary symlink to make the expected structure
                        let parent = target_path.parent().unwrap_or(&package_dir);
                        std::fs::create_dir_all(parent)?;
                        
                        #[cfg(unix)]
                        std::os::unix::fs::symlink(&found_path, &target_path)?;
                        
                        #[cfg(windows)]
                        std::os::windows::fs::symlink_file(&found_path, &target_path)?;
                    } else {
                        return Err(anyhow::anyhow!("Command binary not found at expected path: {}", target_path.display()));
                    }
                }
            }
            
            self.package_manager
                .create_command_symlink(cmd, &package_dir)
                .await?;
        }

        pb.finish_with_message(format!(
            "Successfully installed {} {}",
            app.name, app.version
        ));
        Ok(())
    }

    pub async fn uninstall_app(&self, app_name: &str, version: Option<Version>) -> Result<()> {
        // TODO: Implement app uninstallation
        // This should:
        // 1. Remove all app packages
        // 2. Remove command symlinks
        // 3. Clean up any app-specific configuration
        unimplemented!()
    }
}
