pub(crate) mod github;
pub(crate) mod manager;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Provider {
    pub name: String,
    pub source: ProviderSource,
    pub provider_handler_version: u8,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum ProviderSource {
    Github(github::GithubProvider),
}

impl Provider {
    pub async fn fetch_artifactory(&self) -> Result<String> {
        match &self.source {
            ProviderSource::Github(github) => github.fetch_artifactory().await,
        }
    }

    pub async fn download_package(&self, package_path: &str, destination: &PathBuf) -> Result<()> {
        match &self.source {
            ProviderSource::Github(github) => {
                github.download_package(package_path, destination).await
            }
        }
    }
}
