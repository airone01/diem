use serde::{Deserialize, Serialize};

use crate::{Package, Provider};

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Config {
    pub packages: Vec<Package>,
    pub providers: Vec<Provider>,
    pub config_handler_version: u8,
}
