use std::collections::HashMap;

use anyhow::Result;
use semver::Version;

use crate::{App, Artifactory, Config};

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

    pub async fn find_app(&self, app_spec: &str) -> Result<(App, Provider)> {
        // Parse app specification (format: app_name@version)
        let (app_name, version) = if app_spec.contains('@') {
            let parts: Vec<&str> = app_spec.split('@').collect();
            (parts[0].to_string(), Some(Version::parse(parts[1])?))
        } else {
            (app_spec.to_string(), None)
        };

        let mut found_app = None;
        let mut using_provider = None;

        for provider in self.providers.values() {
            let artifactory_content = provider.fetch_artifactory().await?;
            let artifactory: Artifactory = toml::from_str(&artifactory_content)?;

            // Look for the app in the artifactory
            for app in artifactory.apps {
                if app.name == app_name {
                    if let Some(req_version) = &version {
                        if app.version == *req_version {
                            found_app = Some(app);
                            using_provider = Some(provider.clone());
                            break;
                        }
                    } else {
                        // If no version specified, use the latest
                        match &found_app {
                            Some(existing) if app.version > existing.version => {
                                found_app = Some(app);
                                using_provider = Some(provider.clone());
                            }
                            None => {
                                found_app = Some(app);
                                using_provider = Some(provider.clone());
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        match (found_app, using_provider) {
            (Some(app), Some(provider)) => Ok((app, provider)),
            _ => anyhow::bail!("App {} not found in any provider", app_spec),
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
