use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AppCommand {
    pub command: String,
    pub path: PathBuf,
    // TODO: Add more information about the command
}
