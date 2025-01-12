/// This files defines an App as defined in an app configuration file.
pub(crate) mod command;
pub(crate) mod manager;

use semver::Version;
use serde::{Deserialize, Serialize};

use crate::{AppCommand, Package};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct App {
    pub name: String,
    pub packages: Vec<Package>,
    pub version: Version,
    pub commands: Vec<AppCommand>,
    pub app_handler_version: u8,
}
