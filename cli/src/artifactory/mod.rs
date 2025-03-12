pub mod manager;

#[cfg(test)]
mod tests;

use serde::{Deserialize, Serialize};

use crate::App;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Artifactory {
    pub name: String,
    pub description: Option<String>,
    pub apps: Vec<App>,
    pub maintainer: Option<String>,
    pub public: bool,
    pub artifactory_handler_version: u8,
}
