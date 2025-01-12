use directories::BaseDirs;
use serde::{Deserialize, Serialize};

use std::path::PathBuf;

use crate::{Package, Provider};

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Config {
    pub packages: Vec<Package>,
    pub providers: Vec<Provider>,
    #[serde(default = "default_install_dir")]
    pub install_dir: PathBuf,
    pub config_handler_version: u8,
}

fn default_install_dir() -> PathBuf {
    BaseDirs::new()
        .map(|dirs| dirs.data_local_dir().join("diem").join("packages"))
        .unwrap_or_else(|| PathBuf::from("."))
}

impl Config {
    pub fn ensure_dirs_exist(&self) -> std::io::Result<()> {
        std::fs::create_dir_all(&self.install_dir)?;
        Ok(())
    }
}
