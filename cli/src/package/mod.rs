use serde::{Deserialize, Serialize};

use crate::App;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Package {
    pub name: String,
    pub app: Option<App>,
    pub license: String,
    pub source: Option<String>,
    pub dependencies: Vec<Package>,
    pub package_handler_version: u8,
}
