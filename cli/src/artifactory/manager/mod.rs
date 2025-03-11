use std::path::Path;
use std::fs;
use std::io;
use toml;

use crate::{Artifactory, config::{ArtifactorySource, ArtifactorySubscription, Config}};

pub struct ArtifactoryManager {
    config: Config,
}

impl ArtifactoryManager {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    // List all subscribed artifactories
    pub fn list_subscribed(&self) -> Vec<&ArtifactorySubscription> {
        self.config.subscribed_artifactories.iter().collect()
    }

    // Add a subscription to an artifactory
    pub fn add_subscription(&mut self, sub: ArtifactorySubscription) -> io::Result<()> {
        // Check if already subscribed
        if self.config.subscribed_artifactories.iter().any(|s| s.name == sub.name) {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                format!("Already subscribed to artifactory '{}'", sub.name),
            ));
        }

        self.config.subscribed_artifactories.push(sub);
        Ok(())
    }

    // Remove a subscription
    pub fn remove_subscription(&mut self, name: &str) -> io::Result<()> {
        let len_before = self.config.subscribed_artifactories.len();
        self.config.subscribed_artifactories.retain(|s| s.name != name);
        
        if len_before == self.config.subscribed_artifactories.len() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("No subscription found for '{}'", name),
            ));
        }

        Ok(())
    }

    // Load an artifactory from a local or remote source
    pub fn load_artifactory(&self, subscription: &ArtifactorySubscription) -> io::Result<Artifactory> {
        match &subscription.source {
            ArtifactorySource::Local(path) => self.load_from_file(path),
            ArtifactorySource::Remote(url) => self.load_from_url(url),
        }
    }

    // Load all subscribed artifactories
    pub fn load_all_subscribed(&self) -> Vec<Result<Artifactory, io::Error>> {
        self.config.subscribed_artifactories
            .iter()
            .map(|sub| self.load_artifactory(sub))
            .collect()
    }

    // Create a new artifactory
    pub fn create_artifactory(&self, artifactory: &Artifactory, path: &Path) -> io::Result<()> {
        let toml_content = toml::to_string_pretty(artifactory)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        
        fs::write(path, toml_content)
    }

    // Search for apps in all subscribed artifactories
    pub fn search_apps(&self, query: &str) -> io::Result<Vec<(String, Vec<String>)>> {
        let mut results = Vec::new();

        for sub in &self.config.subscribed_artifactories {
            match self.load_artifactory(sub) {
                Ok(artifactory) => {
                    let matching_apps: Vec<String> = artifactory.apps
                        .iter()
                        .filter(|app| app.name.contains(query))
                        .map(|app| app.name.clone())
                        .collect();
                    
                    if !matching_apps.is_empty() {
                        results.push((artifactory.name, matching_apps));
                    }
                }
                Err(_) => continue,
            }
        }

        Ok(results)
    }

    // Private methods
    fn load_from_file(&self, path: &Path) -> io::Result<Artifactory> {
        let content = fs::read_to_string(path)?;
        toml::from_str(&content).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    fn load_from_url(&self, url: &str) -> io::Result<Artifactory> {
        // This would use a HTTP client to fetch the artifactory file
        // For simplicity, we'll return an error for now
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "Remote artifactory loading not implemented yet",
        ))
    }
}