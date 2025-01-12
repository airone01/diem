use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Package {
    pub name: String,
    pub license: String,
    pub source: Option<String>,
    pub dependencies: Vec<Package>,
    pub package_handler_version: u8,
}
