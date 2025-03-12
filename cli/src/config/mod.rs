use directories::BaseDirs;
use serde::{Deserialize, Serialize};

use std::path::{Path, PathBuf};
use std::fs;
use std::io;

use crate::{Package, Provider};

#[cfg(test)]
mod tests;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    pub packages: Vec<Package>,
    pub providers: Vec<Provider>,
    pub install_dir: PathBuf,
    pub sgoinfre_dir: Option<PathBuf>,
    pub goinfre_dir: Option<PathBuf>,
    pub subscribed_artifactories: Vec<ArtifactorySubscription>,
    pub shared_artifactory_dir: Option<PathBuf>,
    pub config_handler_version: u8,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ArtifactorySubscription {
    pub name: String,
    pub source: ArtifactorySource,
    pub auto_update: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum ArtifactorySource {
    Local(PathBuf),
    Remote(String),
}

impl Default for Config {
    fn default() -> Self {
        Self {
            packages: Vec::new(),
            providers: Vec::new(),
            install_dir: default_install_dir(),
            sgoinfre_dir: default_sgoinfre_dir(),
            goinfre_dir: default_goinfre_dir(),
            subscribed_artifactories: Vec::new(),
            shared_artifactory_dir: None,
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

fn default_sgoinfre_dir() -> Option<PathBuf> {
    let base_dirs = BaseDirs::new()
        .expect("Could not determine base directories");
    
    // Get the username
    let username = std::env::var("USER").unwrap_or_else(|_| {
        if let Some(username) = base_dirs.home_dir().file_name() {
            username.to_string_lossy().to_string()
        } else {
            "unknown".to_string()
        }
    });
    
    // Try both formats: ~/sgoinfre for symbolic links and /sgoinfre/username for direct path
    let home_sgoinfre = base_dirs.home_dir().join("sgoinfre").join("diem");
    let root_sgoinfre = PathBuf::from("/sgoinfre").join(&username).join("diem");
    
    // Check which one exists and use it
    if home_sgoinfre.exists() || std::fs::create_dir_all(&home_sgoinfre).is_ok() {
        Some(home_sgoinfre)
    } else if root_sgoinfre.exists() || std::fs::create_dir_all(&root_sgoinfre).is_ok() {
        Some(root_sgoinfre)
    } else {
        None
    }
}

fn default_goinfre_dir() -> Option<PathBuf> {
    let base_dirs = BaseDirs::new()
        .expect("Could not determine base directories");
    
    // Get the username
    let username = std::env::var("USER").unwrap_or_else(|_| {
        if let Some(username) = base_dirs.home_dir().file_name() {
            username.to_string_lossy().to_string()
        } else {
            "unknown".to_string()
        }
    });
    
    // Try both formats: ~/goinfre for symbolic links and /goinfre/username for direct path
    let home_goinfre = base_dirs.home_dir().join("goinfre").join("diem");
    let root_goinfre = PathBuf::from("/goinfre").join(&username).join("diem");
    
    // Check which one exists and use it
    if home_goinfre.exists() || std::fs::create_dir_all(&home_goinfre).is_ok() {
        Some(home_goinfre)
    } else if root_goinfre.exists() || std::fs::create_dir_all(&root_goinfre).is_ok() {
        Some(root_goinfre)
    } else {
        None
    }
}

impl Config {
    pub fn ensure_dirs_exist(&self) -> std::io::Result<()> {
        std::fs::create_dir_all(&self.install_dir)?;
        
        if let Some(sgoinfre) = &self.sgoinfre_dir {
            std::fs::create_dir_all(sgoinfre)?;
            
            // Create the shared artifactory directory within sgoinfre if not set explicitly
            if self.shared_artifactory_dir.is_none() {
                let shared_dir = sgoinfre.join("shared_artifactory");
                std::fs::create_dir_all(&shared_dir)?;
            }
        }
        
        if let Some(goinfre) = &self.goinfre_dir {
            std::fs::create_dir_all(goinfre)?;
        }
        
        if let Some(shared) = &self.shared_artifactory_dir {
            std::fs::create_dir_all(shared)?;
        }
        
        Ok(())
    }
    
    pub fn sync_goinfre_from_sgoinfre(&self) -> std::io::Result<()> {
        let sgoinfre = match &self.sgoinfre_dir {
            Some(dir) => dir,
            None => return Ok(()),
        };
        
        let goinfre = match &self.goinfre_dir {
            Some(dir) => dir,
            None => return Ok(()),
        };
        
        // Ensure directories exist
        std::fs::create_dir_all(goinfre)?;
        
        // Copy all packages from sgoinfre to goinfre
        copy_dir_contents(sgoinfre, goinfre)?;
        
        println!("Synchronized packages from sgoinfre to goinfre");
        
        // Create symlinks to binaries in PATH if needed
        self.ensure_binaries_symlinked()?;
        
        Ok(())
    }
    
    // Ensures that binaries in install_dir are properly symlinked to user's PATH
    fn ensure_binaries_symlinked(&self) -> std::io::Result<()> {
        let base_dirs = BaseDirs::new()
            .expect("Could not determine base directories");
            
        let bin_dir = base_dirs
            .executable_dir()
            .expect("Could not determine executable directory");
            
        // Make sure bin directory exists
        std::fs::create_dir_all(bin_dir)?;
        
        // For each installed package that has commands, ensure they're symlinked
        // This would be implemented with a call to iterate over all installed packages
        // and recreate their command symlinks
        
        Ok(())
    }
}

// Helper function to copy directory contents recursively
fn copy_dir_contents(src: &Path, dst: &Path) -> io::Result<()> {
    if !src.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Source is not a directory",
        ));
    }

    if !dst.exists() {
        fs::create_dir_all(dst)?;
    }

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if src_path.is_dir() {
            copy_dir_contents(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}
