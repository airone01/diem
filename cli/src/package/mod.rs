use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use tokio::fs;

use crate::Provider;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub sha256: String,
    pub license: String,
    pub source: Option<String>,
    pub dependencies: Vec<Package>,
    pub package_handler_version: u8,
}

pub struct PackageManager {
    install_dir: PathBuf,
}

impl PackageManager {
    pub fn new(install_dir: PathBuf) -> Self {
        Self { install_dir }
    }

    fn install_package_internal<'a>(
        &'a self,
        package: &'a Package,
        provider: &'a Provider,
        pb: &'a ProgressBar,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + 'a>> {
        Box::pin(async move {
            pb.set_message(format!("Installing package: {}", package.name));

            // First, handle dependencies recursively
            for dep in &package.dependencies {
                self.install_package_internal(dep, provider, pb).await?;
            }

            // Determine package destination
            let package_dir = self.install_dir.join(&package.name).join(&package.version);
            if package_dir.exists() {
                pb.finish_with_message(format!("Package {} is already installed", package.name));
                return Ok(());
            }

            // Create package directory
            fs::create_dir_all(&package_dir).await?;

            // Download package
            if let Some(source) = &package.source {
                pb.set_message(format!("Downloading package: {}", package.name));

                // Download to a temporary location
                let temp_path = package_dir.join("package.tmp");
                provider.download_package(source, &temp_path).await?;

                // Verify checksum
                pb.set_message(format!("Verifying package: {}", package.name));
                let content = fs::read(&temp_path).await?;
                let mut hasher = Sha256::new();
                hasher.update(&content);
                let hash = format!("{:x}", hasher.finalize());

                if hash != package.sha256 {
                    fs::remove_dir_all(&package_dir).await?;
                    anyhow::bail!(
                        "Checksum verification failed for package: {}. Expected: {}, Got: {}",
                        package.name,
                        package.sha256,
                        hash
                    );
                }

                // Extract package
                pb.set_message(format!("Extracting package: {}", package.name));
                if temp_path.extension().map_or(false, |ext| ext == "zip") {
                    let file = std::fs::File::open(&temp_path)?;
                    let mut archive = zip::ZipArchive::new(file)?;
                    archive.extract(&package_dir)?;
                } else if temp_path
                    .extension()
                    .map_or(false, |ext| ext == "tar" || ext == "gz")
                {
                    let file = std::fs::File::open(&temp_path)?;
                    let tar = flate2::read::GzDecoder::new(file);
                    let mut archive = tar::Archive::new(tar);
                    archive.unpack(&package_dir)?;
                }

                // Clean up temporary file
                fs::remove_file(temp_path).await?;
            }

            pb.finish_with_message(format!("Successfully installed {}", package.name));
            Ok(())
        })
    }

    pub async fn install_package(&self, package: &Package, provider: &Provider) -> Result<()> {
        // Create progress bar
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} [{elapsed_precise}] {msg}")
                .unwrap(),
        );

        // Call the internal implementation with proper boxing
        self.install_package_internal(package, provider, &pb)
            .await?;

        // Determine package destination
        let package_dir = self.install_dir.join(&package.name).join(&package.version);
        if package_dir.exists() {
            pb.finish_with_message(format!("Package {} is already installed", package.name));
            return Ok(());
        }

        // Create package directory
        fs::create_dir_all(&package_dir).await?;

        // Download package
        if let Some(source) = &package.source {
            pb.set_message(format!("Downloading package: {}", package.name));

            // Download to a temporary location
            let temp_path = package_dir.join("package.tmp");
            provider.download_package(source, &temp_path).await?;

            // Verify checksum
            pb.set_message(format!("Verifying package: {}", package.name));
            let content = fs::read(&temp_path).await?;
            let mut hasher = Sha256::new();
            hasher.update(&content);
            let hash = format!("{:x}", hasher.finalize());

            if hash != package.sha256 {
                fs::remove_dir_all(&package_dir).await?;
                anyhow::bail!(
                    "Checksum verification failed for package: {}. Expected: {}, Got: {}",
                    package.name,
                    package.sha256,
                    hash
                );
            }

            // Extract package
            pb.set_message(format!("Extracting package: {}", package.name));
            if temp_path.extension().map_or(false, |ext| ext == "zip") {
                let file = std::fs::File::open(&temp_path)?;
                let mut archive = zip::ZipArchive::new(file)?;
                archive.extract(&package_dir)?;
            } else if temp_path
                .extension()
                .map_or(false, |ext| ext == "tar" || ext == "gz")
            {
                let file = std::fs::File::open(&temp_path)?;
                let tar = flate2::read::GzDecoder::new(file);
                let mut archive = tar::Archive::new(tar);
                archive.unpack(&package_dir)?;
            }

            // Clean up temporary file
            fs::remove_file(temp_path).await?;
        }

        pb.finish_with_message(format!("Successfully installed {}", package.name));
        Ok(())
    }

    pub async fn uninstall_package(&self, package_name: &str, version: Option<&str>) -> Result<()> {
        let package_dir = self.install_dir.join(package_name);

        if let Some(version) = version {
            // Remove specific version
            let version_dir = package_dir.join(version);
            if version_dir.exists() {
                fs::remove_dir_all(version_dir).await?;
                println!("Uninstalled {} version {}", package_name, version);
            } else {
                println!("Version {} of {} is not installed", version, package_name);
            }
        } else {
            // Remove all versions
            if package_dir.exists() {
                fs::remove_dir_all(package_dir).await?;
                println!("Uninstalled all versions of {}", package_name);
            } else {
                println!("Package {} is not installed", package_name);
            }
        }

        Ok(())
    }

    pub async fn is_package_installed(&self, package_name: &str, version: Option<&str>) -> bool {
        let package_dir = self.install_dir.join(package_name);

        if let Some(version) = version {
            package_dir.join(version).exists()
        } else {
            package_dir.exists()
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PackageError {
    #[error("Package not found: {0}")]
    NotFound(String),

    #[error("Checksum verification failed")]
    ChecksumMismatch,

    #[error("Failed to extract package: {0}")]
    ExtractionError(#[from] std::io::Error),

    #[error("Invalid package format")]
    InvalidFormat,
}
