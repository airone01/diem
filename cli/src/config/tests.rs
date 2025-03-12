#[cfg(test)]
mod tests {
    use crate::config::{Config, copy_dir_contents};
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert!(config.packages.is_empty());
        assert!(config.providers.is_empty());
        assert_eq!(config.config_handler_version, 0);
    }

    #[test]
    fn test_ensure_dirs_exist() {
        let temp_dir = tempdir().unwrap();
        let base_path = temp_dir.path();
        
        let install_dir = base_path.join("install");
        let sgoinfre_dir = base_path.join("sgoinfre");
        let goinfre_dir = base_path.join("goinfre");
        let shared_dir = base_path.join("shared");
        
        let config = Config {
            packages: Vec::new(),
            providers: Vec::new(),
            install_dir: install_dir.clone(),
            sgoinfre_dir: Some(sgoinfre_dir.clone()),
            goinfre_dir: Some(goinfre_dir.clone()),
            subscribed_artifactories: Vec::new(),
            shared_artifactory_dir: Some(shared_dir.clone()),
            config_handler_version: 0,
        };
        
        // Ensure directories exist
        let result = config.ensure_dirs_exist();
        assert!(result.is_ok());
        
        // Verify directories were created
        assert!(install_dir.exists());
        assert!(sgoinfre_dir.exists());
        assert!(goinfre_dir.exists());
        assert!(shared_dir.exists());
    }

    #[test]
    fn test_sync_goinfre_from_sgoinfre() {
        let temp_dir = tempdir().unwrap();
        let base_path = temp_dir.path();
        
        let sgoinfre_dir = base_path.join("sgoinfre");
        let goinfre_dir = base_path.join("goinfre");
        
        // Create the sgoinfre directory and a test file
        fs::create_dir_all(&sgoinfre_dir).unwrap();
        let test_file_path = sgoinfre_dir.join("test_file.txt");
        fs::write(&test_file_path, "test content").unwrap();
        
        let config = Config {
            packages: Vec::new(),
            providers: Vec::new(),
            install_dir: base_path.join("install"),
            sgoinfre_dir: Some(sgoinfre_dir),
            goinfre_dir: Some(goinfre_dir.clone()),
            subscribed_artifactories: Vec::new(),
            shared_artifactory_dir: None,
            config_handler_version: 0,
        };
        
        // Sync directories
        let result = config.sync_goinfre_from_sgoinfre();
        assert!(result.is_ok());
        
        // Verify the file was copied
        let synced_file_path = goinfre_dir.join("test_file.txt");
        assert!(synced_file_path.exists());
        assert_eq!(fs::read_to_string(synced_file_path).unwrap(), "test content");
    }
    
    #[test]
    fn test_copy_dir_contents() {
        let temp_dir = tempdir().unwrap();
        let base_path = temp_dir.path();
        
        let src_dir = base_path.join("src");
        let dst_dir = base_path.join("dst");
        
        // Create source directory with nested files
        fs::create_dir_all(&src_dir).unwrap();
        fs::create_dir_all(&src_dir.join("subdir")).unwrap();
        
        fs::write(src_dir.join("file1.txt"), "content1").unwrap();
        fs::write(src_dir.join("subdir").join("file2.txt"), "content2").unwrap();
        
        // Copy directory contents
        let result = copy_dir_contents(&src_dir, &dst_dir);
        assert!(result.is_ok());
        
        // Verify files were copied
        assert!(dst_dir.exists());
        assert!(dst_dir.join("file1.txt").exists());
        assert!(dst_dir.join("subdir").exists());
        assert!(dst_dir.join("subdir").join("file2.txt").exists());
        
        assert_eq!(fs::read_to_string(dst_dir.join("file1.txt")).unwrap(), "content1");
        assert_eq!(fs::read_to_string(dst_dir.join("subdir").join("file2.txt")).unwrap(), "content2");
    }
}