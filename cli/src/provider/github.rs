use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GithubProvider {
    pub owner: String,
    pub repo: String,
    pub ref_: String,
    pub path: String,
}

impl GithubProvider {
    pub async fn fetch_artifactory(&self) -> Result<Vec<u8>> {
        let client = Client::new();
        let url = format!(
            "https://raw.githubusercontent.com/{}/{}/{}/{}",
            self.owner, self.repo, self.ref_, self.path
        );

        let response = client.get(&url).send().await?;
        let bytes = response.bytes().await?;
        Ok(bytes.to_vec())
    }

    pub async fn download_package(&self, package_path: &str, destination: &PathBuf) -> Result<()> {
        let client = Client::new();
        let url = format!(
            "https://raw.githubusercontent.com/{}/{}/{}/{}",
            self.owner, self.repo, self.ref_, package_path
        );

        let response = client.get(&url).send().await?;
        let bytes = response.bytes().await?;

        // Create parent directories if they don't exist
        if let Some(parent) = destination.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Write the package to disk
        std::fs::write(destination, bytes)?;
        Ok(())
    }
}

// Add a custom error type for provider-specific errors
#[derive(Debug, thiserror::Error)]
pub enum GithubProviderError {
    #[error("Failed to fetch artifactory: {0}")]
    FetchError(#[from] reqwest::Error),
    #[error("Failed to write package: {0}")]
    IoError(#[from] std::io::Error),
}
