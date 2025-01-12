use serde::{Deserialize, Serialize};

use crate::App;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Artifactory {
    pub name: String,
    pub apps: Vec<App>,
    pub artifactory_handler_version: u8,
}
