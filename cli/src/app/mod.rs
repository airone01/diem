/// This files defines an App as defined in an app configuration file.
pub(crate) mod command;

use semver::Version;
use serde::{Deserialize, Serialize};

use crate::AppCommand;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct App {
    pub name: String,
    pub version: Version,
    pub commands: Vec<AppCommand>,
    pub app_handler_version: u8,
}
