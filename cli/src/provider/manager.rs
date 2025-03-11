use std::collections::HashMap;

use anyhow::Result;
use semver::Version;

use crate::{App, Artifactory, Config, config::ArtifactorySubscription};

use super::Provider;

pub struct ProviderManager {
    providers: HashMap<String, Provider>,
}

impl ProviderManager {
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
        }
    }

    pub fn new_from_config(config: &Config) -> Self {
        let mut providers = HashMap::new();
        for provider in &config.providers {
            providers.insert(provider.name.clone(), provider.clone());
        }
        Self { providers }
    }

    pub fn save_to_config(&self, config: &mut Config) {
        config.providers = self.providers.values().cloned().collect();
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

    pub async fn find_app(&self, app_spec: &str, config: &Config) -> Result<(App, Provider)> {
        // Parse app specification (format: app_name@version)
        let (app_name, version) = if app_spec.contains('@') {
            let parts: Vec<&str> = app_spec.split('@').collect();
            (parts[0].to_string(), Some(Version::parse(parts[1])?))
        } else {
            (app_spec.to_string(), None)
        };

        // First, search in registered providers
        let (apps_from_providers, using_providers) = self.find_app_in_providers(&app_name, &version).await?;
        
        // Then, search in subscribed artifactories
        let (apps_from_artifactories, using_artifactories) = 
            self.find_app_in_artifactories(&app_name, &version, config).await?;
        
        // Combine all found apps
        let mut all_apps = Vec::new();
        let mut all_sources = Vec::new();
        let mut provider_indices = Vec::new(); // Track which ones are from providers
        
        for (idx, app) in apps_from_providers.iter().enumerate() {
            all_apps.push(app.clone());
            all_sources.push(format!("Provider: {}", using_providers[idx].name));
            provider_indices.push(true); // Mark as coming from a provider
        }
        
        for (idx, app) in apps_from_artifactories.iter().enumerate() {
            all_apps.push(app.clone());
            all_sources.push(format!("Artifactory: {}", using_artifactories[idx]));
            provider_indices.push(false); // Mark as coming from an artifactory
        }
        
        if all_apps.is_empty() {
            anyhow::bail!("App {} not found in any provider or artifactory", app_spec);
        }
        
        // If we have multiple matches and no specific version was requested, ask the user to choose
        if all_apps.len() > 1 && version.is_none() {
            println!("Multiple versions of '{}' found:", app_name);
            for (i, (app, source)) in all_apps.iter().zip(all_sources.iter()).enumerate() {
                println!("  {}. {} v{} (from {})", i + 1, app.name, app.version, source);
            }
            
            // Ask the user to choose
            println!("Please choose which one to install (1-{}):", all_apps.len());
            let mut choice = String::new();
            std::io::stdin().read_line(&mut choice)?;
            
            let choice: usize = choice.trim().parse()
                .map_err(|_| anyhow::anyhow!("Invalid choice"))?;
                
            if choice < 1 || choice > all_apps.len() {
                anyhow::bail!("Invalid choice: {}", choice);
            }
            
            // Adjust for 0-based indexing
            let idx = choice - 1;
            
            // Create a dummy provider for artifactory sources
            if provider_indices[idx] {
                // It's from a regular provider
                let provider_idx = provider_indices[..idx].iter().filter(|&&is_provider| is_provider).count();
                Ok((all_apps[idx].clone(), using_providers[provider_idx].clone()))
            } else {
                // It's from an artifactory, create a dummy provider
                let artifactory_idx = provider_indices[..idx].iter().filter(|&&is_provider| !is_provider).count();
                let dummy_provider = Provider::create_dummy_for_artifactory(&using_artifactories[artifactory_idx])?;
                Ok((all_apps[idx].clone(), dummy_provider))
            }
        } else {
            // Single app found or specific version requested
            let idx = if version.is_some() {
                // If version was specified, find the exact match
                all_apps.iter().position(|app| {
                    if let Some(req_version) = &version {
                        app.version == *req_version
                    } else {
                        false
                    }
                }).unwrap_or(0)
            } else {
                // Otherwise take the first one
                0
            };
            
            if provider_indices[idx] {
                // It's from a regular provider
                let provider_idx = provider_indices[..idx].iter().filter(|&&is_provider| is_provider).count();
                Ok((all_apps[idx].clone(), using_providers[provider_idx].clone()))
            } else {
                // It's from an artifactory, create a dummy provider
                let artifactory_idx = provider_indices[..idx].iter().filter(|&&is_provider| !is_provider).count();
                let dummy_provider = Provider::create_dummy_for_artifactory(&using_artifactories[artifactory_idx])?;
                Ok((all_apps[idx].clone(), dummy_provider))
            }
        }
    }
    
    // Searches for an app in registered providers
    async fn find_app_in_providers(&self, app_name: &str, version: &Option<Version>) -> Result<(Vec<App>, Vec<Provider>)> {
        let mut found_apps = Vec::new();
        let mut providers_used = Vec::new();

        for provider in self.providers.values() {
            let artifactory_content = match provider.fetch_artifactory().await {
                Ok(content) => content,
                Err(_) => continue, // Skip providers that fail to fetch
            };
            
            let artifactory: Artifactory = match toml::from_str(&artifactory_content) {
                Ok(art) => art,
                Err(_) => continue, // Skip invalid artifactories
            };

            // Look for the app in the artifactory
            for app in artifactory.apps {
                if app.name == app_name {
                    if let Some(req_version) = version {
                        if app.version == *req_version {
                            found_apps.push(app);
                            providers_used.push(provider.clone());
                        }
                    } else {
                        found_apps.push(app);
                        providers_used.push(provider.clone());
                    }
                }
            }
        }

        Ok((found_apps, providers_used))
    }
    
    // Searches for an app in subscribed artifactories
    async fn find_app_in_artifactories(
        &self, 
        app_name: &str, 
        version: &Option<Version>,
        config: &Config
    ) -> Result<(Vec<App>, Vec<String>)> {
        let mut found_apps = Vec::new();
        let mut artifactories_used = Vec::new();
        
        for subscription in &config.subscribed_artifactories {
            let artifactory = match self.load_artifactory_from_subscription(subscription) {
                Ok(art) => art,
                Err(_) => continue, // Skip artifactories that fail to load
            };
            
            // Look for the app in the artifactory
            for app in artifactory.apps {
                if app.name == app_name {
                    if let Some(req_version) = version {
                        if app.version == *req_version {
                            found_apps.push(app);
                            artifactories_used.push(subscription.name.clone());
                        }
                    } else {
                        found_apps.push(app);
                        artifactories_used.push(subscription.name.clone());
                    }
                }
            }
        }
        
        Ok((found_apps, artifactories_used))
    }
    
    // Load an artifactory from a subscription
    fn load_artifactory_from_subscription(&self, subscription: &ArtifactorySubscription) -> Result<Artifactory> {
        match &subscription.source {
            crate::config::ArtifactorySource::Local(path) => {
                let content = std::fs::read_to_string(path)
                    .map_err(|e| anyhow::anyhow!("Failed to read artifactory: {}", e))?;
                    
                toml::from_str(&content)
                    .map_err(|e| anyhow::anyhow!("Failed to parse artifactory: {}", e))
            },
            crate::config::ArtifactorySource::Remote(_url) => {
                // This would normally make a network request to fetch the remote artifactory
                // For now, we'll return an error
                Err(anyhow::anyhow!("Remote artifactories not yet implemented"))
            }
        }
    }

    pub async fn fetch_all_artifactories(&self) -> Result<Vec<(String, String)>> {
        let mut artifactories = Vec::new();
        for (name, provider) in &self.providers {
            let content = provider.fetch_artifactory().await?;
            artifactories.push((name.clone(), content));
        }
        Ok(artifactories)
    }
}
