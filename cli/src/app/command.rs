use std::path::PathBuf;

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct AppCommand {
    pub command: String,
    pub path: PathBuf,
    // TODO: Add more information about the command
}
