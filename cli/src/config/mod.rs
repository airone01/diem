use directories::BaseDirs;
use serde::{Deserialize, Serialize};

use std::path::PathBuf;

use crate::{Package, Provider};

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub packages: Vec<Package>,
    pub providers: Vec<Provider>,
    pub install_dir: PathBuf,
    pub config_handler_version: u8,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            packages: Vec::new(),
            providers: Vec::new(),
            install_dir: default_install_dir(),
            config_handler_version: 0,
        }
    }
}

fn default_install_dir() -> PathBuf {
    BaseDirs::new()
        .expect("Could not determine base directories")
        .executable_dir()
        .expect("Could not determine executable directory")
        .join("diem")
        .join("packages")
}

impl Config {
    pub fn ensure_dirs_exist(&self) -> std::io::Result<()> {
        std::fs::create_dir_all(&self.install_dir)?;
        Ok(())
    }
}
