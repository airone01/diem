pub(crate) mod manager;

use semver::Version;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Package {
    pub name: String,
    pub version: Version,
    pub sha256: String,
    pub license: String,
    pub source: Option<String>,
    pub dependencies: Vec<Package>,
    pub package_handler_version: u8,
}
