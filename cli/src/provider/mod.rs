use anyhow::Result;
use serde::{Deserialize, Serialize};

use std::collections::HashMap;

pub(crate) mod github;

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
    pub async fn fetch_artifactory(&self) -> Result<Vec<u8>> {
        match &self.source {
            ProviderSource::Github(github) => github.fetch_artifactory().await,
        }
    }
}

pub struct ProviderManager {
    providers: HashMap<String, Provider>,
}

impl ProviderManager {
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
        }
    }

    pub fn add_provider(&mut self, provider: Provider) -> Result<()> {
        self.providers.insert(provider.name.clone(), provider);
        Ok(())
    }

    pub fn remove_provider(&mut self, name: &str) -> Result<()> {
        self.providers.remove(name);
        Ok(())
    }

    pub fn list_providers(&self) -> Vec<&Provider> {
        self.providers.values().collect()
    }

    pub async fn fetch_all_artifactories(&self) -> Result<Vec<(String, Vec<u8>)>> {
        let mut artifactories = Vec::new();
        for (name, provider) in &self.providers {
            let content = provider.fetch_artifactory().await?;
            artifactories.push((name.clone(), content));
        }
        Ok(artifactories)
    }
}
