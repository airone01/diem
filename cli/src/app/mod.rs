/// This files defines an App as defined in an app configuration file.
pub(crate) mod command;

use semver::Version;
use serde::Deserialize;

use crate::{AppCommand, Package};

#[derive(Debug, Clone, Deserialize)]
pub struct App {
    pub name: String,
    pub version: Version,
    pub package: Package,
    pub commands: Vec<AppCommand>,
    pub app_handler_version: u8,
}
