#[cfg(test)]
mod tests {
    use super::*;
    use crate::{App, AppCommand, Package, Artifactory};
    use std::path::PathBuf;
    use semver::Version;
    use std::str::FromStr;

    #[test]
    fn test_artifactory_serialization() {
        // Create a sample Artifactory
        let app_command = AppCommand {
            command: "test".to_string(),
            path: PathBuf::from("bin/test"),
        };

        let package = Package {
            name: "test-package".to_string(),
            version: Version::from_str("1.0.0").unwrap(),
            sha256: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".to_string(),
            license: "MIT".to_string(),
            source: Some("test-package-1.0.0.tar.gz".to_string()),
            dependencies: Vec::new(),
            package_handler_version: 0,
        };

        let app = App {
            name: "test-app".to_string(),
            packages: vec![package],
            version: Version::from_str("1.0.0").unwrap(),
            commands: vec![app_command],
            license: "MIT".to_string(),
            app_handler_version: 0,
            description: Some("Test application".to_string()),
        };

        let artifactory = Artifactory {
            name: "Test Artifactory".to_string(),
            description: Some("Test description".to_string()),
            apps: vec![app],
            maintainer: Some("Test User".to_string()),
            public: true,
            artifactory_handler_version: 1,
        };

        // Serialize to TOML
        let toml_str = toml::to_string(&artifactory).unwrap();
        
        // Deserialize back
        let deserialized: Artifactory = toml::from_str(&toml_str).unwrap();
        
        // Check equality
        assert_eq!(artifactory.name, deserialized.name);
        assert_eq!(artifactory.description, deserialized.description);
        assert_eq!(artifactory.maintainer, deserialized.maintainer);
        assert_eq!(artifactory.public, deserialized.public);
        assert_eq!(artifactory.artifactory_handler_version, deserialized.artifactory_handler_version);
        
        // Check app
        assert_eq!(artifactory.apps.len(), deserialized.apps.len());
        assert_eq!(artifactory.apps[0].name, deserialized.apps[0].name);
        assert_eq!(artifactory.apps[0].version, deserialized.apps[0].version);
        assert_eq!(artifactory.apps[0].description, deserialized.apps[0].description);
    }
}