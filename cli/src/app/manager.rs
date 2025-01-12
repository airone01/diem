use crate::{App, PackageManager, Provider};
use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use semver::Version;

pub struct AppManager {
    package_manager: PackageManager,
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
