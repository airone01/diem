pub(crate) mod github;
pub(crate) mod manager;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use futures_util::stream::StreamExt as _;

use std::path::PathBuf;
use reqwest;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Provider {
    pub name: String,
    pub source: ProviderSource,
    pub provider_handler_version: u8,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum ProviderSource {
    Github(github::GithubProvider),
    Artifactory(ArtifactoryProvider),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ArtifactoryProvider {
    pub path: PathBuf,
}

impl Provider {
    pub async fn fetch_artifactory(&self) -> Result<String> {
        match &self.source {
            ProviderSource::Github(github) => github.fetch_artifactory().await,
            ProviderSource::Artifactory(artifactory) => {
                std::fs::read_to_string(&artifactory.path)
                    .map_err(|e| anyhow::anyhow!("Failed to read artifactory file: {}", e))
            },
        }
    }

    pub async fn download_package(&self, package_path: &str, destination: &PathBuf) -> Result<()> {
        match &self.source {
            ProviderSource::Github(github) => {
                github.download_package(package_path, destination).await
            },
            ProviderSource::Artifactory(artifactory) => {
                println!("Package path: {}", package_path);

                // Check if this is a local file (relative or absolute path)
                if !package_path.starts_with("http://") && !package_path.starts_with("https://") {
                    // Handle local file copy - both relative and absolute paths
                    let source_path = if package_path.starts_with("/") {
                        // Absolute path
                        std::path::PathBuf::from(package_path)
                    } else {
                        // Relative path - relative to the artifactory path
                        if let Some(parent) = artifactory.path.parent() {
                            parent.join(package_path)
                        } else {
                            // Fallback to current directory if no parent
                            std::path::PathBuf::from(".").join(package_path)
                        }
                    };
                    
                    println!("Copying from local file: {}", source_path.display());
                    
                    // Ensure parent directory exists for destination
                    if let Some(parent) = destination.parent() {
                        tokio::fs::create_dir_all(parent).await
                            .map_err(|e| anyhow::anyhow!("Failed to create parent directories: {}", e))?;
                    }
                    
                    // Copy the file
                    tokio::fs::copy(&source_path, destination).await
                        .map_err(|e| anyhow::anyhow!("Failed to copy file: {}", e))?;
                    
                    println!("File copied successfully to: {}", destination.display());
                    return Ok(());
                }
                
                // This is for direct downloads from URLs
                let client = reqwest::Client::new();
                let response = client.get(package_path)
                    .send()
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to download package: {}", e))?;
                
                // Create progress bar for download
                let total_size = response.content_length().unwrap_or(0);
                let pb = indicatif::ProgressBar::new(total_size);
                pb.set_style(indicatif::ProgressStyle::default_bar()
                    .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
                    .unwrap()
                    .progress_chars("#>-"));
                
                // Ensure parent directory exists for destination
                if let Some(parent) = destination.parent() {
                    tokio::fs::create_dir_all(parent).await
                        .map_err(|e| anyhow::anyhow!("Failed to create parent directories: {}", e))?;
                }
                
                println!("Downloading to: {}", destination.display());
                
                // Stream the download with progress updates
                let mut file = tokio::fs::File::create(destination).await
                    .map_err(|e| anyhow::anyhow!("Failed to create destination file {}: {}", destination.display(), e))?;
                let mut stream = response.bytes_stream();
                
                let mut downloaded: u64 = 0;
                while let Some(chunk) = stream.next().await {
                    let chunk = chunk
                        .map_err(|e| anyhow::anyhow!("Failed to download chunk: {}", e))?;
                    tokio::io::AsyncWriteExt::write_all(&mut file, &chunk).await
                        .map_err(|e| anyhow::anyhow!("Failed to write to file: {}", e))?;
                    downloaded += chunk.len() as u64;
                    pb.set_position(downloaded);
                }
                
                pb.finish_with_message("Download complete");
                Ok(())
            }
        }
    }
    
    // Create a dummy provider for artifactories
    pub fn create_dummy_for_artifactory(artifactory_name: &str) -> Result<Self> {
        Ok(Self {
            name: format!("artifactory:{}", artifactory_name),
            source: ProviderSource::Artifactory(ArtifactoryProvider {
                path: PathBuf::new(), // We don't need the actual path here
            }),
            provider_handler_version: 1,
        })
    }
}
