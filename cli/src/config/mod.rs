use serde::{Deserialize, Serialize};

use crate::Package;

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Config {
    pub packages: Vec<Package>,
    pub config_handler_version: u8,
}
