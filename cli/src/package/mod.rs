use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Package {
    pub sha256: String,
    pub license: String,
    pub source: Option<String>,
    pub dependencies: Vec<Package>,
    pub package_handler_version: u8,
}
