#[cfg(test)]
mod tests {
    use super::*;
    use crate::Package;
    use semver::Version;
    use std::str::FromStr;

    #[test]
    fn test_package_serialization() {
        // Create a sample Package
        let package = Package {
            name: "test-package".to_string(),
            version: Version::from_str("1.0.0").unwrap(),
            sha256: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".to_string(),
            license: "MIT".to_string(),
            source: Some("test-package-1.0.0.tar.gz".to_string()),
            dependencies: Vec::new(),
            package_handler_version: 0,
        };

        // Serialize to TOML
        let toml_str = toml::to_string(&package).unwrap();
        
        // Deserialize back
        let deserialized: Package = toml::from_str(&toml_str).unwrap();
        
        // Check equality
        assert_eq!(package.name, deserialized.name);
        assert_eq!(package.version, deserialized.version);
        assert_eq!(package.sha256, deserialized.sha256);
        assert_eq!(package.license, deserialized.license);
        assert_eq!(package.source, deserialized.source);
        assert_eq!(package.package_handler_version, deserialized.package_handler_version);
    }

    #[test]
    fn test_package_with_dependencies() {
        // Create a dependency package
        let dep_package = Package {
            name: "dependency".to_string(),
            version: Version::from_str("0.5.0").unwrap(),
            sha256: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".to_string(),
            license: "MIT".to_string(),
            source: Some("dependency-0.5.0.tar.gz".to_string()),
            dependencies: Vec::new(),
            package_handler_version: 0,
        };

        // Create a package with dependencies
        let package = Package {
            name: "test-package".to_string(),
            version: Version::from_str("1.0.0").unwrap(),
            sha256: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".to_string(),
            license: "MIT".to_string(),
            source: Some("test-package-1.0.0.tar.gz".to_string()),
            dependencies: vec![dep_package.clone()],
            package_handler_version: 0,
        };

        // Serialize to TOML
        let toml_str = toml::to_string(&package).unwrap();
        
        // Deserialize back
        let deserialized: Package = toml::from_str(&toml_str).unwrap();
        
        // Check dependency
        assert_eq!(deserialized.dependencies.len(), 1);
        assert_eq!(deserialized.dependencies[0].name, dep_package.name);
        assert_eq!(deserialized.dependencies[0].version, dep_package.version);
    }
}